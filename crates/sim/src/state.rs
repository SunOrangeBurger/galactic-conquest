//! Dense, ordered simulation state. Entity references use integer indices;
//! serialization and API IDs are derived from those stable indices.

use crate::fixed::Fixed;
use serde::{Deserialize, Serialize};

pub type SystemId = u32;
pub type LaneId = u32;
pub type PlanetId = u32;
pub type CivilizationId = u32;
pub type FleetId = u32;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Richness {
    Poor,
    Average,
    Rich,
    Exceptional,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum HazardKind {
    Void,
    Nebula,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum LaneHazard {
    None,
    Slow,
    Void,
    Nebula,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum FleetKind {
    War,
    Scout,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum StartingArchetype {
    Warlord,
    Industrialist,
    Populist,
    Explorer,
    Balanced,
}

impl StartingArchetype {
    pub const ALL: [Self; 5] = [
        Self::Warlord,
        Self::Industrialist,
        Self::Populist,
        Self::Explorer,
        Self::Balanced,
    ];
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub struct Point {
    pub x: i32,
    pub y: i32,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct SystemState {
    pub id: SystemId,
    pub position: Point,
    pub region: u8,
    pub planet_ids: Vec<PlanetId>,
    pub lane_ids: Vec<LaneId>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct LaneState {
    pub id: LaneId,
    pub a: SystemId,
    pub b: SystemId,
    pub length: u32,
    pub travel_ticks: u8,
    pub hazard: LaneHazard,
    pub blocked: bool,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct HazardZone {
    pub id: u32,
    pub kind: HazardKind,
    pub center: Point,
    pub radius: u32,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct CoreDisc {
    pub center: Point,
    pub radius: u32,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct PlanetState {
    pub id: PlanetId,
    pub system_id: SystemId,
    pub richness: Richness,
    pub base_yield: u32,
    pub population_capacity: u32,
    pub owner: Option<CivilizationId>,
    pub population: Fixed,
    pub garrison: Fixed,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct FleetState {
    pub id: FleetId,
    pub owner: CivilizationId,
    pub kind: FleetKind,
    pub strength: Fixed,
    pub system_id: Option<SystemId>,
    pub in_transit: Option<TransitState>,
    pub busy: bool,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct TransitState {
    pub lane_id: LaneId,
    pub destination: SystemId,
    pub elapsed_ticks: u8,
    pub total_ticks: u8,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct CivilizationState {
    pub id: CivilizationId,
    pub external_id: String,
    pub archetype: StartingArchetype,
    pub home_system: SystemId,
    pub credits: i64,
    pub military_investment: Fixed,
    pub economy_investment: Fixed,
    pub population_investment: Fixed,
    pub alive: bool,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct GalaxyState {
    pub seed: u64,
    pub generation_attempts: u32,
    pub systems: Vec<SystemState>,
    pub lanes: Vec<LaneState>,
    pub hazards: Vec<HazardZone>,
    pub core: CoreDisc,
    pub planets: Vec<PlanetState>,
    pub fleets: Vec<FleetState>,
    pub civilizations: Vec<CivilizationState>,
}

impl Default for GalaxyState {
    fn default() -> Self {
        Self {
            seed: 0,
            generation_attempts: 0,
            systems: Vec::new(),
            lanes: Vec::new(),
            hazards: Vec::new(),
            core: CoreDisc {
                center: Point { x: 0, y: 0 },
                radius: 150,
            },
            planets: Vec::new(),
            fleets: Vec::new(),
            civilizations: Vec::new(),
        }
    }
}
