//! Public HTTP/JSON contract for Galactic Conquest Phase 0 (specification v0.4).
//!
//! This crate contains wire types only. Simulation and validation logic live
//! in the `sim` and `server` crates.

#![recursion_limit = "256"]

use serde::{Deserialize, Serialize};

pub mod schema;

pub const API_VERSION: &str = "0.4";

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct TickRequest {
    pub api_version: String,
    pub tick: u64,
    pub game_year: u64,
    pub deadline_ms: u64,
    pub r#match: MatchView,
    pub your_civilization: CivilizationView,
    pub visible_state: VisibleState,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub last_tick_report: Option<TickReport>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields, rename_all = "snake_case")]
pub enum MatchState {
    Lobby,
    Smoke,
    Armed,
    Running,
    Paused,
    Finished,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct MatchView {
    pub state: MatchState,
    pub systems_total: u32,
    pub planets_total: u32,
    pub civs_alive: u32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct CivilizationView {
    pub id: String,
    pub archetype: String,
    pub civilization_score: f64,
    pub resources: Resources,
    pub score_breakdown: ScoreBreakdown,
    pub multipliers: Multipliers,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Resources {
    pub credits: i64,
    pub income_per_tick: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ScoreBreakdown {
    pub territory: f64,
    pub military: f64,
    pub economy: f64,
    pub population: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Multipliers {
    pub military: f64,
    pub economy: f64,
    pub population: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct VisibleState {
    pub owned_planets: Vec<OwnedPlanet>,
    pub systems: Vec<SystemIntel>,
    pub your_fleets: Vec<YourFleet>,
    pub visible_enemy_fleets: Vec<EnemyFleet>,
    pub build_queue: Vec<BuildQueueItem>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields, rename_all = "snake_case")]
pub enum Richness {
    Poor,
    Average,
    Rich,
    Exceptional,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct OwnedPlanet {
    pub planet_id: String,
    pub system_id: String,
    pub richness: Richness,
    pub base_yield: u32,
    pub pop: f64,
    pub pop_capacity: f64,
    pub garrison: f64,
    pub garrison_target: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct SystemIntel {
    pub system_id: String,
    pub tier: SystemTier,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub last_seen_tick: Option<u64>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub planets: Vec<PlanetSnapshot>,
    pub lanes: Vec<LaneIntel>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields, rename_all = "snake_case")]
pub enum SystemTier {
    Snapshot,
    AdjacentFog,
    Scouted,
    Owned,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct PlanetSnapshot {
    pub planet_id: String,
    pub richness: Richness,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub owner: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub garrison: Option<f64>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields, rename_all = "snake_case")]
pub enum Hazard {
    None,
    Slow,
    Void,
    Nebula,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct LaneIntel {
    pub lane_id: String,
    pub to: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub travel_ticks: Option<u8>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub travel_ticks_est: Option<[u8; 2]>,
    pub hazard: Hazard,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields, rename_all = "snake_case")]
pub enum FleetKind {
    War,
    Scout,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct InTransit {
    pub lane_id: String,
    pub to: String,
    pub arrives_tick: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct YourFleet {
    pub fleet_id: String,
    pub kind: FleetKind,
    pub strength: Option<f64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub system_id: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub in_transit: Option<InTransit>,
    pub busy: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct EnemyFleet {
    pub fleet_id: String,
    pub owner: String,
    pub strength: f64,
    pub system_id: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct BuildQueueItem {
    pub planet_id: String,
    pub unit: UnitKind,
    pub quantity: u32,
    pub spawns_tick: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields, rename_all = "snake_case")]
pub enum UnitKind {
    Warship,
    Scout,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct TickReport {
    pub tick: u64,
    pub actions: Vec<ActionReport>,
    pub events: Vec<AgentEvent>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ActionReport {
    pub index: usize,
    pub status: ActionStatus,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub code: Option<ActionErrorCode>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub message: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields, rename_all = "snake_case")]
pub enum ActionStatus {
    Ok,
    Rejected,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type", deny_unknown_fields, rename_all = "snake_case")]
pub enum AgentEvent {
    Battle {
        planet_id: String,
        winner: Option<String>,
        your_strength_before: f64,
        your_strength_after: f64,
    },
    PlanetColonized {
        planet_id: String,
    },
    PlanetCaptured {
        planet_id: String,
    },
    CivEliminated {
        civ: String,
    },
}

/// Agent -> server. Unknown fields at the response level are ignored for
/// forward compatibility; unknown fields within an action are rejected.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AgentResponse {
    pub tick: u64,
    pub actions: Vec<AgentAction>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case", deny_unknown_fields)]
pub enum AgentAction {
    InvestResources {
        target: InvestmentTarget,
        amount: i64,
    },
    Build {
        planet_id: String,
        unit: UnitKind,
        quantity: u32,
    },
    MoveFleet {
        fleet_id: String,
        target_system: String,
    },
    Attack {
        fleet_id: String,
        target_planet: String,
    },
    Scout {
        fleet_id: String,
        target_system: String,
    },
    Colonize {
        fleet_id: String,
        target_planet: String,
    },
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields, rename_all = "snake_case")]
pub enum InvestmentTarget {
    Military,
    Economy,
    Population,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields, rename_all = "snake_case")]
pub enum ActionErrorCode {
    TooManyActions,
    UnknownActionType,
    InvalidField,
    TargetNotFoundOrNotVisible,
    NotOwner,
    FleetBusy,
    FleetNotInSystem,
    FleetWrongKind,
    ConflictingOrders,
    NoPath,
    InsufficientCredits,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields, rename_all = "snake_case")]
pub enum ResponseErrorCode {
    MalformedJson,
    SchemaViolation,
    WrongTick,
    LateSubmission,
    CivEliminated,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ResponseError {
    pub code: ResponseErrorCode,
    pub message: String,
}

/// Static omniscient map used by the read-only spectator service.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct SpectatorMap {
    pub api_version: String,
    pub systems: Vec<SpectatorSystem>,
    pub lanes: Vec<SpectatorLane>,
    pub hazards: Vec<HazardZone>,
    pub core: CoreDisc,
    pub planets: Vec<SpectatorPlanetStatic>,
    pub civs: Vec<SpectatorCivStatic>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct SpectatorSystem {
    pub id: String,
    pub x: i32,
    pub y: i32,
    pub region: u8,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct SpectatorLane {
    pub id: String,
    pub a: String,
    pub b: String,
    pub travel_ticks: u8,
    pub hazard: Hazard,
    pub blocked: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields, rename_all = "snake_case")]
pub enum HazardKind {
    Void,
    Nebula,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct HazardZone {
    pub id: String,
    pub kind: HazardKind,
    pub x: i32,
    pub y: i32,
    pub radius: u32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct CoreDisc {
    pub x: i32,
    pub y: i32,
    pub radius: u32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct SpectatorPlanetStatic {
    pub id: String,
    pub system_id: String,
    pub richness: Richness,
    pub base_yield: u32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct SpectatorCivStatic {
    pub id: String,
    pub name: String,
    pub color: u8,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct SpectatorTick {
    pub api_version: String,
    pub tick: u64,
    pub game_year: u64,
    pub r#match: SpectatorMatchView,
    pub civs: Vec<SpectatorCiv>,
    pub planets: Vec<SpectatorPlanetState>,
    pub fleets: Vec<SpectatorFleet>,
    pub events: Vec<SpectatorEvent>,
    pub dominance: DominanceView,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct SpectatorMatchView {
    pub state: MatchState,
    pub time_remaining_ms: u64,
    pub civs_alive: u32,
    pub delay_ticks: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct SpectatorCiv {
    pub id: String,
    pub name: String,
    pub color: u8,
    pub alive: bool,
    pub civilization_score: f64,
    pub score_breakdown: SpectatorScoreBreakdown,
    pub planets_owned: u32,
    pub territory_share: f64,
    pub war_strength: f64,
    pub credits: i64,
    pub archetype: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub elimination_tick: Option<u64>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct SpectatorScoreBreakdown {
    pub territory: f64,
    pub military: f64,
    pub economy: f64,
    pub population: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct SpectatorPlanetState {
    pub planet_id: String,
    pub owner: Option<String>,
    pub garrison: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct SpectatorFleet {
    pub fleet_id: String,
    pub owner: String,
    pub kind: FleetKind,
    pub strength: f64,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub system_id: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub in_transit: Option<SpectatorTransit>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct SpectatorTransit {
    pub lane_id: String,
    pub to: String,
    pub elapsed: u8,
    pub total: u8,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case", deny_unknown_fields)]
pub enum SpectatorEvent {
    Battle {
        planet_id: String,
        parties: Vec<String>,
        winner: Option<String>,
        strength_before: std::collections::BTreeMap<String, f64>,
        strength_after: std::collections::BTreeMap<String, f64>,
    },
    PlanetColonized {
        planet_id: String,
        owner: String,
    },
    PlanetCaptured {
        planet_id: String,
        owner: String,
    },
    CivEliminated {
        civ: String,
    },
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct DominanceView {
    pub leader: Option<String>,
    pub share: f64,
    pub threshold: f64,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn action_wire_shape_and_unknown_action_fields() {
        let valid =
            r#"{"tick":14,"actions":[{"type":"scout","fleet_id":"f5","target_system":"S15"}]}"#;
        let parsed: AgentResponse = serde_json::from_str(valid).expect("documented action parses");
        assert_eq!(parsed.tick, 14);
        assert!(serde_json::from_str::<AgentResponse>(
            r#"{"tick":14,"actions":[{"type":"scout","fleet_id":"f5","target_system":"S15","extra":1}]}"#
        )
        .is_err());
    }
}
