//! Start-of-tick action validation. Visibility and route knowledge are applied
//! by the server when it builds each civ's masked request.

use crate::state::GalaxyState;
use contract::{ActionErrorCode, AgentAction, UnitKind};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ActionValidationError {
    pub action_index: usize,
    pub code: ActionErrorCode,
    pub message: String,
}

pub fn validate_actions(
    state: &GalaxyState,
    civ_external_id: &str,
    actions: &[AgentAction],
) -> (Vec<AgentAction>, Vec<ActionValidationError>) {
    let civ_id = state
        .civilizations
        .iter()
        .find(|civ| civ.external_id == civ_external_id)
        .map(|civ| civ.id);
    let mut available = civ_id
        .and_then(|id| state.civilizations.get(id as usize))
        .map(|civ| civ.credits)
        .unwrap_or(0);
    let mut valid = Vec::new();
    let mut errors = Vec::new();
    let mut ordered_fleets = Vec::new();

    for (index, action) in actions.iter().enumerate() {
        if index >= 200 {
            errors.push(ActionValidationError {
                action_index: index,
                code: ActionErrorCode::TooManyActions,
                message: "the per-tick action limit is 200".into(),
            });
            continue;
        }

        let fleet_id = match action {
            AgentAction::MoveFleet { fleet_id, .. }
            | AgentAction::Attack { fleet_id, .. }
            | AgentAction::Scout { fleet_id, .. }
            | AgentAction::Colonize { fleet_id, .. } => Some(fleet_id),
            _ => None,
        };
        if let Some(fleet_id) = fleet_id {
            if ordered_fleets.iter().any(|previous| previous == fleet_id) {
                errors.push(ActionValidationError {
                    action_index: index,
                    code: ActionErrorCode::ConflictingOrders,
                    message: "the first action naming a fleet takes precedence".into(),
                });
                continue;
            }
            ordered_fleets.push(fleet_id.clone());
        }

        let mut cost = 0_i64;
        let result = match action {
            AgentAction::InvestResources { amount, .. } => {
                if *amount <= 0 {
                    Err((ActionErrorCode::InvalidField, "amount must be positive"))
                } else {
                    cost = *amount;
                    Ok(())
                }
            }
            AgentAction::Build {
                planet_id,
                unit,
                quantity,
                ..
            } => match parse_id(planet_id, 'P').and_then(|id| state.planets.get(id as usize)) {
                None => Err((
                    ActionErrorCode::TargetNotFoundOrNotVisible,
                    "planet is not visible",
                )),
                Some(planet) if Some(planet.owner.unwrap_or(u32::MAX)) != civ_id => {
                    Err((ActionErrorCode::NotOwner, "planet is not owned by this civ"))
                }
                Some(_) if *quantity == 0 => {
                    Err((ActionErrorCode::InvalidField, "quantity must be positive"))
                }
                Some(_) => {
                    let unit_cost = match unit {
                        UnitKind::Warship => 5_i64,
                        UnitKind::Scout => 40_i64,
                    };
                    cost = unit_cost.saturating_mul(i64::from(*quantity));
                    Ok(())
                }
            },
            AgentAction::MoveFleet {
                fleet_id,
                target_system,
            }
            | AgentAction::Scout {
                fleet_id,
                target_system,
            } => {
                if !owns_fleet(state, civ_id, fleet_id) {
                    Err((ActionErrorCode::NotOwner, "fleet is not owned by this civ"))
                } else if let Some(fleet) = get_fleet(state, fleet_id) {
                    if fleet.busy || fleet.in_transit.is_some() {
                        Err((ActionErrorCode::FleetBusy, "fleet is not idle"))
                    } else if fleet.system_id.is_none() {
                        Err((
                            ActionErrorCode::FleetNotInSystem,
                            "fleet is not in a system",
                        ))
                    } else if matches!(action, AgentAction::Scout { .. })
                        && fleet.kind != crate::state::FleetKind::Scout
                    {
                        Err((
                            ActionErrorCode::FleetWrongKind,
                            "scout action needs a scout fleet",
                        ))
                    } else if matches!(action, AgentAction::MoveFleet { .. })
                        && fleet.kind != crate::state::FleetKind::War
                    {
                        Err((
                            ActionErrorCode::FleetWrongKind,
                            "move_fleet needs a war fleet",
                        ))
                    } else if parse_id(target_system, 'S')
                        .is_none_or(|id| id as usize >= state.systems.len())
                    {
                        Err((
                            ActionErrorCode::TargetNotFoundOrNotVisible,
                            "target system is not visible",
                        ))
                    } else {
                        Ok(())
                    }
                } else {
                    Err((ActionErrorCode::NotOwner, "fleet is not owned by this civ"))
                }
            }
            AgentAction::Attack {
                fleet_id,
                target_planet,
            }
            | AgentAction::Colonize {
                fleet_id,
                target_planet,
            } => {
                if !owns_fleet(state, civ_id, fleet_id) {
                    Err((ActionErrorCode::NotOwner, "fleet is not owned by this civ"))
                } else if let Some(fleet) = get_fleet(state, fleet_id) {
                    if fleet.busy || fleet.in_transit.is_some() {
                        Err((ActionErrorCode::FleetBusy, "fleet is not idle"))
                    } else if fleet.system_id.is_none() {
                        Err((
                            ActionErrorCode::FleetNotInSystem,
                            "fleet is not in a system",
                        ))
                    } else if fleet.kind != crate::state::FleetKind::War {
                        Err((ActionErrorCode::FleetWrongKind, "action needs a war fleet"))
                    } else if let Some(planet) =
                        parse_id(target_planet, 'P').and_then(|id| state.planets.get(id as usize))
                    {
                        if state.planets[planet.id as usize].system_id != fleet.system_id.unwrap() {
                            Err((
                                ActionErrorCode::TargetNotFoundOrNotVisible,
                                "planet is in a different system",
                            ))
                        } else if matches!(action, AgentAction::Attack { .. })
                            && planet.owner.is_none_or(|owner| Some(owner) == civ_id)
                        {
                            Err((
                                ActionErrorCode::TargetNotFoundOrNotVisible,
                                "attack target must be enemy-owned",
                            ))
                        } else if matches!(action, AgentAction::Colonize { .. })
                            && planet.owner.is_some()
                        {
                            Err((
                                ActionErrorCode::TargetNotFoundOrNotVisible,
                                "colonize target must be unowned",
                            ))
                        } else {
                            if matches!(action, AgentAction::Colonize { .. }) {
                                cost = 150;
                            }
                            Ok(())
                        }
                    } else {
                        Err((
                            ActionErrorCode::TargetNotFoundOrNotVisible,
                            "target planet is not visible",
                        ))
                    }
                } else {
                    Err((ActionErrorCode::NotOwner, "fleet is not owned by this civ"))
                }
            }
        };

        let result = result.and({
            if cost > available {
                Err((
                    ActionErrorCode::InsufficientCredits,
                    "action exceeds remaining start-of-tick credits",
                ))
            } else {
                Ok(())
            }
        });
        match result {
            Ok(()) => {
                available -= cost;
                valid.push(action.clone());
            }
            Err((code, message)) => errors.push(ActionValidationError {
                action_index: index,
                code,
                message: message.to_string(),
            }),
        }
    }
    (valid, errors)
}

fn owns_fleet(state: &GalaxyState, civ_id: Option<u32>, fleet_id: &str) -> bool {
    get_fleet(state, fleet_id).is_some_and(|fleet| Some(fleet.owner) == civ_id)
}

fn get_fleet<'a>(state: &'a GalaxyState, fleet_id: &str) -> Option<&'a crate::state::FleetState> {
    parse_id(fleet_id, 'f').and_then(|id| state.fleets.get(id as usize))
}

fn parse_id(text: &str, prefix: char) -> Option<u32> {
    text.strip_prefix(prefix)?.parse().ok()
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{
        fixed::Fixed,
        state::{
            CivilizationState, FleetKind, FleetState, PlanetState, Richness, StartingArchetype,
        },
    };

    fn sample_state() -> GalaxyState {
        let mut state = GalaxyState::default();
        state.civilizations.push(CivilizationState {
            id: 0,
            external_id: "team_a".into(),
            archetype: StartingArchetype::Balanced,
            home_system: 0,
            credits: 1_000,
            military_investment: Fixed::ZERO,
            economy_investment: Fixed::ZERO,
            population_investment: Fixed::ZERO,
            alive: true,
        });
        state.systems.push(crate::state::SystemState {
            id: 0,
            position: crate::state::Point { x: 0, y: 0 },
            region: 0,
            planet_ids: vec![0],
            lane_ids: Vec::new(),
        });
        state.planets.push(PlanetState {
            id: 0,
            system_id: 0,
            richness: Richness::Average,
            base_yield: 8,
            population_capacity: 2_000,
            owner: Some(0),
            population: Fixed::ZERO,
            garrison: Fixed::ZERO,
        });
        state.fleets.push(FleetState {
            id: 0,
            owner: 0,
            kind: FleetKind::War,
            strength: Fixed::ONE,
            system_id: Some(0),
            in_transit: None,
            busy: false,
        });
        state
    }

    #[test]
    fn rejects_unknown_target_but_keeps_valid_investment() {
        let actions = vec![
            AgentAction::Attack {
                fleet_id: "f0".into(),
                target_planet: "P77".into(),
            },
            AgentAction::InvestResources {
                target: contract::InvestmentTarget::Military,
                amount: 100,
            },
        ];
        let (valid, errors) = validate_actions(&sample_state(), "team_a", &actions);
        assert_eq!(valid.len(), 1);
        assert_eq!(errors.len(), 1);
        assert!(matches!(
            errors[0].code,
            ActionErrorCode::TargetNotFoundOrNotVisible
        ));
    }
}
