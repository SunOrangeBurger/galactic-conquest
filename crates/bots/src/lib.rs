//! Baseline bots for smoke-testing and fairness auditing (design doc
//! Section 2.4, Section 12 step 2/6): random, always-aggressive, turtle.
//!
//! These are deliberately dumb — the point is to run them from every
//! starting archetype/position to confirm no starting condition or trivial
//! strategy dominates (Section 0 Definition of Done), not to play well.

use contract::{AgentAction, AgentResponse, InvestmentTarget, TickRequest};
use rand::Rng;

pub trait Bot {
    fn decide(&mut self, request: &TickRequest) -> AgentResponse;
}

/// Picks a uniformly random legal-shaped action each tick, or does nothing.
/// Useful as a noise floor: any strategy that can't consistently beat
/// Random from every starting position is broken, not just weak.
pub struct RandomBot {
    rng: rand::rngs::ThreadRng,
}

impl Default for RandomBot {
    fn default() -> Self {
        RandomBot {
            rng: rand::thread_rng(),
        }
    }
}

impl Bot for RandomBot {
    fn decide(&mut self, request: &TickRequest) -> AgentResponse {
        let mut actions = Vec::new();

        if request.your_civilization.resources.credits > 0
            && self.rng.gen_bool(0.5)
            && !request.visible_state.owned_planets.is_empty()
        {
            actions.push(AgentAction::InvestResources {
                target: InvestmentTarget::Military,
                amount: self
                    .rng
                    .gen_range(1..=request.your_civilization.resources.credits),
            });
        }

        AgentResponse {
            tick: request.tick,
            actions,
        }
    }
}

/// Attacks the nearest visible target every single tick with everything
/// it has, no economic investment. Tests whether raw early aggression is a
/// dominant strategy (it shouldn't be, given diminishing-returns combat).
pub struct AlwaysAggressiveBot;

impl Bot for AlwaysAggressiveBot {
    fn decide(&mut self, request: &TickRequest) -> AgentResponse {
        let mut actions = Vec::new();

        let target = request
            .visible_state
            .systems
            .iter()
            .flat_map(|system| system.planets.iter())
            .find(|planet| {
                planet
                    .owner
                    .as_deref()
                    .is_some_and(|owner| owner != request.your_civilization.id)
            });
        if let (Some(target), Some(fleet)) = (target, request.visible_state.your_fleets.first()) {
            actions.push(AgentAction::Attack {
                fleet_id: fleet.fleet_id.clone(),
                target_planet: target.planet_id.clone(),
            });
        }

        AgentResponse {
            tick: request.tick,
            actions,
        }
    }
}

/// Never attacks, invests everything into economy/military buildup at home,
/// never moves fleets off owned planets. Tests whether pure turtling is
/// either dominant or an auto-loss.
pub struct TurtleBot;

impl Bot for TurtleBot {
    fn decide(&mut self, request: &TickRequest) -> AgentResponse {
        let credits = request.your_civilization.resources.credits;
        let actions = if credits > 0 {
            vec![AgentAction::InvestResources {
                target: InvestmentTarget::Economy,
                amount: credits,
            }]
        } else {
            Vec::new()
        };
        AgentResponse {
            tick: request.tick,
            actions,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use contract::{
        CivilizationView, MatchState, MatchView, Multipliers, Resources, ScoreBreakdown,
        VisibleState,
    };

    fn empty_request() -> TickRequest {
        TickRequest {
            api_version: contract::API_VERSION.into(),
            tick: 1,
            game_year: 1,
            deadline_ms: 200,
            r#match: MatchView {
                state: MatchState::Running,
                systems_total: 1,
                planets_total: 1,
                civs_alive: 1,
            },
            your_civilization: CivilizationView {
                id: "test".into(),
                archetype: "balanced".into(),
                civilization_score: 900.0,
                resources: Resources {
                    credits: 100,
                    income_per_tick: 0.0,
                },
                score_breakdown: ScoreBreakdown {
                    territory: 0.0,
                    military: 300.0,
                    economy: 300.0,
                    population: 300.0,
                },
                multipliers: Multipliers {
                    military: 1.0,
                    economy: 1.0,
                    population: 1.0,
                },
            },
            visible_state: VisibleState {
                owned_planets: vec![],
                systems: vec![],
                your_fleets: vec![],
                visible_enemy_fleets: vec![],
                build_queue: vec![],
            },
            last_tick_report: None,
        }
    }

    #[test]
    fn turtle_bot_never_attacks() {
        let mut bot = TurtleBot;
        let response = bot.decide(&empty_request());
        assert!(!response
            .actions
            .iter()
            .any(|a| matches!(a, AgentAction::Attack { .. })));
    }

    #[test]
    fn bots_do_not_panic_on_empty_visible_state() {
        let mut random = RandomBot::default();
        let mut aggressive = AlwaysAggressiveBot;
        let mut turtle = TurtleBot;
        random.decide(&empty_request());
        aggressive.decide(&empty_request());
        turtle.decide(&empty_request());
    }
}
