//! Seeded integer-only map generation (spec v0.4, section 4).

use crate::{
    integer::integer_sqrt,
    state::{
        CivilizationState, CoreDisc, GalaxyState, HazardKind, HazardZone, LaneHazard, LaneState,
        PlanetState, Point, Richness, StartingArchetype, SystemState,
    },
};
use rand::rngs::StdRng;
use rand::{seq::SliceRandom, Rng, SeedableRng};

const Q30: u128 = 1 << 30;
const CORDIC_GAIN_Q30: i64 = 652_032_874;
const PI_Q30: i64 = 3_373_259_426;
const ATAN_Q30: [i64; 31] = [
    843_314_857,
    497_837_829,
    263_043_837,
    133_525_159,
    67_021_687,
    33_543_516,
    16_775_851,
    8_388_437,
    4_194_283,
    2_097_149,
    1_048_576,
    524_288,
    262_144,
    131_072,
    65_536,
    32_768,
    16_384,
    8_192,
    4_096,
    2_048,
    1_024,
    512,
    256,
    128,
    64,
    32,
    16,
    8,
    4,
    2,
    1,
];

#[derive(Debug, Clone)]
pub struct GalaxyConfig {
    pub seed: u64,
    pub systems_per_civ_min: u32,
    pub systems_per_civ_max: u32,
    pub planets_per_two_systems: u32,
    pub coordinate_radius: u32,
    pub core_radius_percent: u32,
    pub spiral_turns: u32,
    pub jitter_max: u32,
    pub maximum_lane_length: u32,
    pub bridge_lanes_per_arm_pair: u32,
    pub hazard_radius: u32,
    pub retry_limit: u32,
    pub richness_weights: [u32; 4],
    pub base_yields: [u32; 4],
    pub population_capacities: [u32; 4],
    pub regional_factor_milli: [u32; 2],
    pub minimum_home_distance: u32,
    pub neighborhood_hops: u32,
    pub neighborhood_yield_tolerance_percent: u32,
    pub starting_territory_milli: u32,
    pub territory_tolerance_percent: u32,
}

impl Default for GalaxyConfig {
    fn default() -> Self {
        Self {
            seed: 42,
            systems_per_civ_min: 18,
            systems_per_civ_max: 22,
            planets_per_two_systems: 5,
            coordinate_radius: 1_000,
            core_radius_percent: 15,
            spiral_turns: 2,
            jitter_max: 25,
            maximum_lane_length: 700,
            bridge_lanes_per_arm_pair: 2,
            hazard_radius: 50,
            retry_limit: 10_000,
            richness_weights: [40, 35, 20, 5],
            base_yields: [4, 8, 14, 24],
            population_capacities: [1_000, 2_000, 3_500, 6_000],
            regional_factor_milli: [800, 1_300],
            minimum_home_distance: 5,
            neighborhood_hops: 3,
            neighborhood_yield_tolerance_percent: 10,
            starting_territory_milli: 150_000,
            territory_tolerance_percent: 5,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum GenerationError {
    InvalidCivCount,
    DuplicateCivId,
    TooFewSystemsForArms,
    InsufficientBridgeLanes,
    DegreeConstraintUnsatisfied,
    DisconnectedLaneGraph,
    NoFairHomeworldSet,
    RetryLimitExceeded,
    SeedOverflow,
}

/// Creates one reproducible map and starting assignment. Civilization IDs
/// are sorted before any seeded shuffles, so input order never affects output.
pub fn generate(config: &GalaxyConfig, civ_ids: &[String]) -> Result<GalaxyState, GenerationError> {
    if !(8..=20).contains(&civ_ids.len()) {
        return Err(GenerationError::InvalidCivCount);
    }
    let mut ordered_civs = civ_ids.to_vec();
    ordered_civs.sort();
    if ordered_civs.windows(2).any(|pair| pair[0] == pair[1]) {
        return Err(GenerationError::DuplicateCivId);
    }

    for attempt in 0..config.retry_limit {
        let seed = config
            .seed
            .checked_add(u64::from(attempt))
            .ok_or(GenerationError::SeedOverflow)?;
        match generate_once(config, &ordered_civs, seed, attempt) {
            Ok(state) => return Ok(state),
            Err(
                GenerationError::DisconnectedLaneGraph
                | GenerationError::NoFairHomeworldSet
                | GenerationError::DegreeConstraintUnsatisfied
                | GenerationError::InsufficientBridgeLanes,
            ) => {}
            Err(other) => return Err(other),
        }
    }
    Err(GenerationError::RetryLimitExceeded)
}

fn generate_once(
    config: &GalaxyConfig,
    civ_ids: &[String],
    seed: u64,
    attempt: u32,
) -> Result<GalaxyState, GenerationError> {
    let mut rng = StdRng::seed_from_u64(seed);
    let systems_per_civ = rng.gen_range(config.systems_per_civ_min..=config.systems_per_civ_max);
    let system_count = (civ_ids.len() as u32)
        .checked_mul(systems_per_civ)
        .ok_or(GenerationError::TooFewSystemsForArms)?;
    if system_count < 20 || system_count < config.bridge_lanes_per_arm_pair * 4 {
        return Err(GenerationError::TooFewSystemsForArms);
    }
    let core_radius = config
        .coordinate_radius
        .saturating_mul(config.core_radius_percent)
        / 100;
    let mut systems = place_systems(config, system_count, core_radius, &mut rng)?;
    let (mut lanes, speed_unit) = build_lanes(config, &systems, core_radius)?;
    let mut hazards = place_hazards(config, &systems, core_radius, &mut lanes, &mut rng)?;
    ensure_clear_routes(config, &systems, core_radius, &hazards, &mut lanes)?;
    canonicalize_lanes(&mut lanes);
    for lane in &mut lanes {
        lane.travel_ticks = ceil_div(lane.length, speed_unit).clamp(1, 6) as u8;
        if lane.hazard == LaneHazard::Nebula {
            lane.travel_ticks = lane.travel_ticks.saturating_add(2);
        }
    }
    link_lanes(&mut systems, &lanes);

    let planet_count = system_count
        .checked_mul(config.planets_per_two_systems)
        .ok_or(GenerationError::TooFewSystemsForArms)?;
    let planet_count = planet_count.div_ceil(2);
    let distances = all_pairs_distances(&systems, &lanes);
    let candidate_hubs = choose_candidate_hubs(config, &distances, civ_ids.len())
        .ok_or(GenerationError::NoFairHomeworldSet)?;
    let mut planets = generate_planets(
        config,
        &mut systems,
        planet_count,
        &candidate_hubs,
        &mut rng,
    );
    enrich_candidate_hubs(
        config,
        &systems,
        &mut planets,
        &candidate_hubs,
        civ_ids.len(),
    )?;
    balance_home_neighborhoods(config, &mut planets, &candidate_hubs, &distances)?;
    let total_value: u64 = planets
        .iter()
        .map(|planet| u64::from(planet.base_yield))
        .sum();
    let homes = select_homes(
        config,
        &systems,
        &planets,
        total_value,
        civ_ids.len(),
        &distances,
    )
    .ok_or(GenerationError::NoFairHomeworldSet)?;

    let mut shuffled_homes = homes;
    shuffled_homes.shuffle(&mut rng);
    let archetypes = shuffled_archetypes(civ_ids.len(), &mut rng);
    let mut civilizations = Vec::with_capacity(civ_ids.len());
    for (index, ((external_id, home_system), archetype)) in civ_ids
        .iter()
        .zip(shuffled_homes)
        .zip(archetypes)
        .enumerate()
    {
        let civ_id = index as u32;
        for planet_id in &systems[home_system as usize].planet_ids {
            planets[*planet_id as usize].owner = Some(civ_id);
        }
        civilizations.push(CivilizationState {
            id: civ_id,
            external_id: external_id.clone(),
            archetype,
            home_system,
            credits: 0,
            military_investment: crate::fixed::Fixed::ZERO,
            economy_investment: crate::fixed::Fixed::ZERO,
            population_investment: crate::fixed::Fixed::ZERO,
            alive: true,
        });
    }

    // Keep hazards sorted and all entity vectors in dense stable-ID order.
    hazards.sort_by_key(|hazard| hazard.id);
    Ok(GalaxyState {
        seed,
        generation_attempts: attempt,
        systems,
        lanes,
        hazards,
        core: CoreDisc {
            center: Point { x: 0, y: 0 },
            radius: core_radius,
        },
        planets,
        fleets: Vec::new(),
        civilizations,
    })
}

fn place_systems(
    config: &GalaxyConfig,
    count: u32,
    core_radius: u32,
    rng: &mut StdRng,
) -> Result<Vec<SystemState>, GenerationError> {
    let mut systems = Vec::with_capacity(count as usize);
    let base = count / 4;
    let remainder = count % 4;
    for arm in 0..4_u32 {
        let arm_count = base + u32::from(arm < remainder);
        if arm_count < 2 {
            return Err(GenerationError::TooFewSystemsForArms);
        }
        let inner_radius = core_radius
            .saturating_add(config.jitter_max.saturating_mul(2))
            .max(1);
        let outer_radius = config
            .coordinate_radius
            .saturating_sub(config.jitter_max.saturating_mul(2));
        let radii = geometric_radii(inner_radius, outer_radius.max(inner_radius), arm_count);
        for index in 0..arm_count {
            let angle_millidegrees = i64::from(arm) * 90_000
                + i64::from(index) * i64::from(config.spiral_turns) * 360_000
                    / i64::from(arm_count - 1);
            let (sin, cos) = cordic_sin_cos(angle_millidegrees);
            let radius = i64::from(radii[index as usize]);
            let jitter_x = rng.gen_range(-(config.jitter_max as i32)..=config.jitter_max as i32);
            let jitter_y = rng.gen_range(-(config.jitter_max as i32)..=config.jitter_max as i32);
            let x = (cos * radius / Q30 as i64 + i64::from(jitter_x)) as i32;
            let y = (sin * radius / Q30 as i64 + i64::from(jitter_y)) as i32;
            let id = systems.len() as u32;
            systems.push(SystemState {
                id,
                position: Point { x, y },
                region: arm as u8,
                planet_ids: Vec::new(),
                lane_ids: Vec::new(),
            });
        }
    }
    Ok(systems)
}

fn geometric_radii(start: u32, end: u32, count: u32) -> Vec<u32> {
    if count <= 1 {
        return vec![end];
    }
    let ratio_q = u128::from(end.max(start)) * Q30 / u128::from(start);
    let mut low = Q30;
    let mut high = Q30 * 2;
    while low + 1 < high {
        let mid = (low + high) / 2;
        if q30_pow(mid, count - 1) >= ratio_q {
            high = mid;
        } else {
            low = mid;
        }
    }
    let mut radii = Vec::with_capacity(count as usize);
    let mut factor = Q30;
    for index in 0..count {
        let radius = u128::from(start) * factor / Q30;
        radii.push(if index + 1 == count {
            end
        } else {
            radius as u32
        });
        factor = factor * high / Q30;
    }
    radii
}

fn q30_pow(base: u128, exponent: u32) -> u128 {
    let mut result = Q30;
    for _ in 0..exponent {
        result = result.saturating_mul(base) / Q30;
    }
    result
}

fn cordic_sin_cos(angle_millidegrees: i64) -> (i64, i64) {
    match angle_millidegrees.rem_euclid(360_000) {
        0 => return (0, Q30 as i64),
        90_000 => return (Q30 as i64, 0),
        180_000 => return (0, -(Q30 as i64)),
        270_000 => return (-(Q30 as i64), 0),
        _ => {}
    }
    let mut angle = (i128::from(angle_millidegrees) * i128::from(PI_Q30) / 180_000) as i64;
    let tau = PI_Q30 * 2;
    angle %= tau;
    if angle > PI_Q30 {
        angle -= tau;
    } else if angle < -PI_Q30 {
        angle += tau;
    }
    let half_pi = PI_Q30 / 2;
    let mut flip = 1_i64;
    if angle > half_pi {
        angle -= PI_Q30;
        flip = -1;
    } else if angle < -half_pi {
        angle += PI_Q30;
        flip = -1;
    }

    let mut x = CORDIC_GAIN_Q30;
    let mut y = 0_i64;
    let mut z = angle;
    for (index, atan) in ATAN_Q30.iter().enumerate() {
        let old_x = x;
        if z >= 0 {
            x -= y >> index;
            y += old_x >> index;
            z -= atan;
        } else {
            x += y >> index;
            y -= old_x >> index;
            z += atan;
        }
    }
    (y * flip, x * flip)
}

fn build_lanes(
    config: &GalaxyConfig,
    systems: &[SystemState],
    core_radius: u32,
) -> Result<(Vec<LaneState>, u32), GenerationError> {
    let mut candidates = Vec::new();
    for a in 0..systems.len() {
        for b in a + 1..systems.len() {
            let length = distance(systems[a].position, systems[b].position);
            if length <= config.maximum_lane_length
                && !segment_intersects_circle(
                    systems[a].position,
                    systems[b].position,
                    Point { x: 0, y: 0 },
                    core_radius,
                )
            {
                candidates.push((length, a as u32, b as u32));
            }
        }
    }
    candidates.sort_unstable();
    let speed_unit = lane_speed_unit(&candidates);
    let mut edges = Vec::new();
    let mut degree = vec![0_u8; systems.len()];
    let mut edge_set = std::collections::BTreeSet::new();

    // Build a short, degree-bounded backbone within each spiral arm.
    for arm in 0..4_u8 {
        let arm_ids: Vec<usize> = systems
            .iter()
            .filter(|system| system.region == arm)
            .map(|system| system.id as usize)
            .collect();
        let mut parent: Vec<usize> = (0..systems.len()).collect();
        let mut remaining = arm_ids.len().saturating_sub(1);
        for (length, a, b) in &candidates {
            if systems[*a as usize].region != arm || systems[*b as usize].region != arm {
                continue;
            }
            if degree[*a as usize] >= 5 || degree[*b as usize] >= 5 {
                continue;
            }
            let mut root_a = *a as usize;
            while parent[root_a] != root_a {
                root_a = parent[root_a];
            }
            let mut root_b = *b as usize;
            while parent[root_b] != root_b {
                root_b = parent[root_b];
            }
            if root_a == root_b {
                continue;
            }
            parent[root_a] = root_b;
            edges.push((*length, *a, *b));
            edge_set.insert((*a, *b));
            degree[*a as usize] += 1;
            degree[*b as usize] += 1;
            remaining -= 1;
            if remaining == 0 {
                break;
            }
        }
        if remaining != 0 || arm_ids.len() < 2 {
            return Err(GenerationError::DisconnectedLaneGraph);
        }
    }

    // Two bridge lanes for each adjacent arm pair. Core-safe route completion
    // below guarantees a bypass network if hazards remove any of these links.
    for arm in 0..4_u8 {
        let next_arm = (arm + 1) % 4;
        let pair_candidates: Vec<_> = candidates
            .iter()
            .copied()
            .filter(|(_, a, b)| {
                systems[*a as usize].region == arm && systems[*b as usize].region == next_arm
                    || systems[*a as usize].region == next_arm && systems[*b as usize].region == arm
            })
            .collect();
        let mut selected = 0;
        for candidate in pair_candidates {
            let (length, a, b) = candidate;
            if degree[a as usize] >= 5 || degree[b as usize] >= 5 || !edge_set.insert((a, b)) {
                continue;
            }
            edges.push((length, a, b));
            degree[a as usize] += 1;
            degree[b as usize] += 1;
            selected += 1;
            if selected == config.bridge_lanes_per_arm_pair {
                break;
            }
        }
        if selected < config.bridge_lanes_per_arm_pair {
            return Err(GenerationError::InsufficientBridgeLanes);
        }
    }

    for (length, a, b) in &candidates {
        if (degree[*a as usize] < 3 && degree[*b as usize] < 5
            || degree[*b as usize] < 3 && degree[*a as usize] < 5)
            && edge_set.insert((*a, *b))
        {
            edges.push((*length, *a, *b));
            degree[*a as usize] += 1;
            degree[*b as usize] += 1;
        }
    }
    if degree.iter().any(|degree| *degree < 3 || *degree > 5) {
        return Err(GenerationError::DegreeConstraintUnsatisfied);
    }
    edges.sort_unstable_by_key(|edge| (edge.1, edge.2));
    let lanes: Vec<LaneState> = edges
        .into_iter()
        .enumerate()
        .map(|(index, (length, a, b))| LaneState {
            id: index as u32,
            a,
            b,
            length,
            travel_ticks: 1,
            hazard: LaneHazard::None,
            blocked: false,
        })
        .collect();
    if !is_connected(systems.len(), &lanes) {
        return Err(GenerationError::DisconnectedLaneGraph);
    }
    Ok((lanes, speed_unit))
}

fn place_hazards(
    config: &GalaxyConfig,
    systems: &[SystemState],
    core_radius: u32,
    lanes: &mut Vec<LaneState>,
    rng: &mut StdRng,
) -> Result<Vec<HazardZone>, GenerationError> {
    let count = (systems.len() as u32).div_ceil(20);
    let void_count = count.div_ceil(2);
    let mut hazards = Vec::with_capacity(count as usize);
    let mut kinds = vec![HazardKind::Void; void_count as usize];
    kinds.extend(std::iter::repeat_n(
        HazardKind::Nebula,
        (count - void_count) as usize,
    ));
    kinds.shuffle(rng);

    for (zone_id, kind) in kinds.into_iter().enumerate() {
        let mut placed = false;
        for _ in 0..config.retry_limit {
            let center = Point {
                x: rng.gen_range(
                    -(config.coordinate_radius as i32)..=config.coordinate_radius as i32,
                ),
                y: rng.gen_range(
                    -(config.coordinate_radius as i32)..=config.coordinate_radius as i32,
                ),
            };
            if distance(center, Point { x: 0, y: 0 }) <= core_radius
                || distance(center, Point { x: 0, y: 0 }) > config.coordinate_radius
            {
                continue;
            }
            let mut candidate_lanes = lanes.clone();
            let mut affected_lanes = 0;
            for lane in &mut candidate_lanes {
                let a = systems[lane.a as usize].position;
                let b = systems[lane.b as usize].position;
                if segment_intersects_circle(a, b, center, config.hazard_radius) {
                    affected_lanes += 1;
                    match kind {
                        HazardKind::Void => {
                            lane.blocked = true;
                            lane.hazard = LaneHazard::Void;
                        }
                        HazardKind::Nebula if !lane.blocked => lane.hazard = LaneHazard::Nebula,
                        HazardKind::Nebula => {}
                    }
                }
            }
            if affected_lanes > 0 {
                let zone = HazardZone {
                    id: zone_id as u32,
                    kind,
                    center,
                    radius: config.hazard_radius,
                };
                let mut candidate_hazards = hazards.clone();
                candidate_hazards.push(zone.clone());
                if ensure_clear_routes(
                    config,
                    systems,
                    core_radius,
                    &candidate_hazards,
                    &mut candidate_lanes,
                )
                .is_ok()
                {
                    *lanes = candidate_lanes;
                    hazards.push(zone);
                    placed = true;
                    break;
                }
            }
        }
        if !placed {
            return Err(GenerationError::DisconnectedLaneGraph);
        }
    }
    Ok(hazards)
}

fn generate_planets(
    config: &GalaxyConfig,
    systems: &mut [SystemState],
    total_planets: u32,
    candidate_hubs: &[u32],
    rng: &mut StdRng,
) -> Vec<PlanetState> {
    let mut counts = vec![1_u32; systems.len()];
    for hub in candidate_hubs {
        counts[*hub as usize] = 4;
    }
    let mut remaining = total_planets.saturating_sub(counts.iter().sum::<u32>());
    let mut order: Vec<usize> = (0..systems.len()).collect();
    order.shuffle(rng);
    while remaining > 0 {
        let mut placed = false;
        for index in &order {
            if counts[*index] < 4 && remaining > 0 {
                counts[*index] += 1;
                remaining -= 1;
                placed = true;
            }
        }
        if !placed {
            break;
        }
    }

    let regional_factors: Vec<u32> = (0..4)
        .map(|_| rng.gen_range(config.regional_factor_milli[0]..=config.regional_factor_milli[1]))
        .collect();
    let mut planets = Vec::with_capacity(total_planets as usize);
    for (system_index, count) in counts.into_iter().enumerate() {
        let system_id = systems[system_index].id;
        let region = systems[system_index].region;
        let factor = regional_factors[region as usize];
        for _ in 0..count {
            let richness = draw_richness(config, factor, rng);
            let class = richness_index(richness);
            let id = planets.len() as u32;
            planets.push(PlanetState {
                id,
                system_id,
                richness,
                base_yield: config.base_yields[class],
                population_capacity: config.population_capacities[class],
                owner: None,
                population: crate::fixed::Fixed::ZERO,
                garrison: crate::fixed::Fixed::ZERO,
            });
            systems[system_index].planet_ids.push(id);
        }
    }
    planets
}

fn choose_candidate_hubs(
    config: &GalaxyConfig,
    distances: &[Vec<u32>],
    civ_count: usize,
) -> Option<Vec<u32>> {
    let mut best: Option<(u32, Vec<u32>)> = None;
    for first in 0..distances.len() {
        let mut hubs = vec![first as u32];
        let mut minimum = u32::MAX;
        while hubs.len() < civ_count {
            let next = (0..distances.len())
                .filter(|candidate| !hubs.contains(&(*candidate as u32)))
                .filter_map(|candidate| {
                    let nearest = hubs
                        .iter()
                        .map(|hub| distances[candidate][*hub as usize])
                        .min()?;
                    (nearest >= config.minimum_home_distance).then_some((nearest, candidate as u32))
                })
                .max_by(|a, b| a.0.cmp(&b.0).then_with(|| b.1.cmp(&a.1)));
            let Some((distance, id)) = next else {
                break;
            };
            minimum = minimum.min(distance);
            hubs.push(id);
        }
        if hubs.len() == civ_count && best.as_ref().is_none_or(|current| minimum > current.0) {
            best = Some((minimum, hubs));
        }
    }
    best.map(|(_, hubs)| hubs)
}

fn enrich_candidate_hubs(
    config: &GalaxyConfig,
    systems: &[SystemState],
    planets: &mut Vec<PlanetState>,
    hubs: &[u32],
    civ_count: usize,
) -> Result<(), GenerationError> {
    let total_value: u64 = planets
        .iter()
        .map(|planet| u64::from(planet.base_yield))
        .sum();
    let tolerance = territory_tolerance_units(
        total_value,
        config.starting_territory_milli,
        config.territory_tolerance_percent,
    );
    let mut combinations = Vec::new();
    for a in 0..4 {
        for b in a..4 {
            for c in b..4 {
                for d in c..4 {
                    let classes = [a, b, c, d];
                    let value = classes
                        .iter()
                        .map(|class| u64::from(config.base_yields[*class]))
                        .sum();
                    let deviation = territory_deviation(
                        value,
                        total_value,
                        civ_count,
                        config.starting_territory_milli,
                    );
                    if deviation <= tolerance {
                        combinations.push((deviation, classes));
                    }
                }
            }
        }
    }
    combinations.sort_unstable();
    let hub_ids: std::collections::BTreeSet<u32> = hubs.iter().copied().collect();
    for hub in hubs {
        let ids = &systems[*hub as usize].planet_ids;
        let mut applied = false;
        for (_, classes) in &combinations {
            let mut trial = planets.clone();
            let mut possible = true;
            for (planet_id, target_class) in ids.iter().zip(classes) {
                let home_index = *planet_id as usize;
                if richness_index(trial[home_index].richness) == *target_class {
                    continue;
                }
                let donor = trial.iter().position(|planet| {
                    !hub_ids.contains(&planet.system_id)
                        && richness_index(planet.richness) == *target_class
                });
                let Some(donor_index) = donor else {
                    possible = false;
                    break;
                };
                let old = (
                    trial[home_index].richness,
                    trial[home_index].base_yield,
                    trial[home_index].population_capacity,
                );
                trial[home_index].richness = trial[donor_index].richness;
                trial[home_index].base_yield = trial[donor_index].base_yield;
                trial[home_index].population_capacity = trial[donor_index].population_capacity;
                trial[donor_index].richness = old.0;
                trial[donor_index].base_yield = old.1;
                trial[donor_index].population_capacity = old.2;
            }
            if possible {
                *planets = trial;
                applied = true;
                break;
            }
        }
        if !applied {
            return Err(GenerationError::NoFairHomeworldSet);
        }
    }
    Ok(())
}

fn balance_home_neighborhoods(
    config: &GalaxyConfig,
    planets: &mut [PlanetState],
    hubs: &[u32],
    distances: &[Vec<u32>],
) -> Result<(), GenerationError> {
    let hub_set: std::collections::BTreeSet<u32> = hubs.iter().copied().collect();
    let membership: Vec<Vec<bool>> = planets
        .iter()
        .map(|planet| {
            hubs.iter()
                .map(|hub| {
                    distances[*hub as usize][planet.system_id as usize] <= config.neighborhood_hops
                })
                .collect()
        })
        .collect();
    let mut sums = vec![0_i64; hubs.len()];
    for (planet, flags) in planets.iter().zip(&membership) {
        for (index, included) in flags.iter().enumerate() {
            if *included {
                sums[index] += i64::from(planet.base_yield);
            }
        }
    }
    for _ in 0..128 {
        let current = neighborhood_spread(&sums);
        if current <= u128::from(config.neighborhood_yield_tolerance_percent) * 10_000 {
            return Ok(());
        }
        let mut best: Option<(u128, usize, usize, Vec<i64>)> = None;
        for first in 0..planets.len() {
            if hub_set.contains(&planets[first].system_id) {
                continue;
            }
            for second in first + 1..planets.len() {
                if hub_set.contains(&planets[second].system_id)
                    || planets[first].base_yield == planets[second].base_yield
                    || membership[first] == membership[second]
                {
                    continue;
                }
                let delta =
                    i64::from(planets[second].base_yield) - i64::from(planets[first].base_yield);
                let mut trial = sums.clone();
                for index in 0..hubs.len() {
                    trial[index] += delta
                        * (i64::from(membership[first][index])
                            - i64::from(membership[second][index]));
                }
                let spread = neighborhood_spread(&trial);
                if spread < current && best.as_ref().is_none_or(|value| spread < value.0) {
                    best = Some((spread, first, second, trial));
                }
            }
        }
        let Some((_, first, second, next_sums)) = best else {
            break;
        };
        let old = (
            planets[first].richness,
            planets[first].base_yield,
            planets[first].population_capacity,
        );
        planets[first].richness = planets[second].richness;
        planets[first].base_yield = planets[second].base_yield;
        planets[first].population_capacity = planets[second].population_capacity;
        planets[second].richness = old.0;
        planets[second].base_yield = old.1;
        planets[second].population_capacity = old.2;
        sums = next_sums;
    }
    Err(GenerationError::NoFairHomeworldSet)
}

fn neighborhood_spread(sums: &[i64]) -> u128 {
    let total: i64 = sums.iter().sum();
    if total <= 0 {
        return u128::MAX;
    }
    let count = sums.len() as i64;
    sums.iter()
        .map(|sum| (sum * count - total).unsigned_abs() as u128 * 1_000_000 / total as u128)
        .max()
        .unwrap_or(u128::MAX)
}

/// Ensures there is a connected route using only unblocked lanes that avoid
/// every hazard and the black-hole core. New links are added after hazard
/// placement, so they are checked against the complete hazard set.
fn ensure_clear_routes(
    config: &GalaxyConfig,
    systems: &[SystemState],
    core_radius: u32,
    hazards: &[HazardZone],
    lanes: &mut Vec<LaneState>,
) -> Result<(), GenerationError> {
    let mut components = DisjointSet::new(systems.len());
    let mut degree = vec![0_u8; systems.len()];
    let mut existing = std::collections::BTreeSet::new();
    for lane in lanes.iter() {
        degree[lane.a as usize] += 1;
        degree[lane.b as usize] += 1;
        existing.insert((lane.a, lane.b));
        if !lane.blocked && lane.hazard == LaneHazard::None {
            components.union(lane.a as usize, lane.b as usize);
        }
    }

    let mut candidates = Vec::new();
    for a in 0..systems.len() {
        for b in a + 1..systems.len() {
            let first = systems[a].position;
            let second = systems[b].position;
            let length = distance(first, second);
            if length > config.maximum_lane_length
                || segment_intersects_circle(first, second, Point { x: 0, y: 0 }, core_radius)
                || hazards
                    .iter()
                    .any(|zone| segment_intersects_circle(first, second, zone.center, zone.radius))
            {
                continue;
            }
            candidates.push((length, a as u32, b as u32));
        }
    }
    candidates.sort_unstable();
    for (length, a, b) in candidates {
        if components.count == 1 {
            break;
        }
        if existing.contains(&(a, b))
            || degree[a as usize] >= 5
            || degree[b as usize] >= 5
            || components.same(a as usize, b as usize)
        {
            continue;
        }
        lanes.push(LaneState {
            id: lanes.len() as u32,
            a,
            b,
            length,
            travel_ticks: 1,
            hazard: LaneHazard::None,
            blocked: false,
        });
        existing.insert((a, b));
        degree[a as usize] += 1;
        degree[b as usize] += 1;
        components.union(a as usize, b as usize);
    }
    if components.count != 1 {
        return Err(GenerationError::DisconnectedLaneGraph);
    }
    Ok(())
}

fn canonicalize_lanes(lanes: &mut [LaneState]) {
    lanes.sort_unstable_by_key(|lane| (lane.a, lane.b));
    for (index, lane) in lanes.iter_mut().enumerate() {
        lane.id = index as u32;
    }
}

struct DisjointSet {
    parent: Vec<usize>,
    count: usize,
}

impl DisjointSet {
    fn new(size: usize) -> Self {
        Self {
            parent: (0..size).collect(),
            count: size,
        }
    }

    fn find(&mut self, item: usize) -> usize {
        if self.parent[item] != item {
            self.parent[item] = self.find(self.parent[item]);
        }
        self.parent[item]
    }

    fn same(&mut self, a: usize, b: usize) -> bool {
        self.find(a) == self.find(b)
    }

    fn union(&mut self, a: usize, b: usize) {
        let root_a = self.find(a);
        let root_b = self.find(b);
        if root_a != root_b {
            self.parent[root_b] = root_a;
            self.count -= 1;
        }
    }
}

fn draw_richness(config: &GalaxyConfig, factor: u32, rng: &mut StdRng) -> Richness {
    let mut weights = [0_u64; 4];
    let mut multiplier = 1_u128;
    for (index, weight) in weights.iter_mut().enumerate() {
        *weight = (u128::from(config.richness_weights[index]) * multiplier) as u64;
        multiplier = multiplier * u128::from(factor) / 1_000;
    }
    let total: u64 = weights.iter().sum();
    let draw = rng.gen_range(0..total);
    let mut boundary = 0;
    for (index, weight) in weights.iter().enumerate() {
        boundary += weight;
        if draw < boundary {
            return [
                Richness::Poor,
                Richness::Average,
                Richness::Rich,
                Richness::Exceptional,
            ][index];
        }
    }
    Richness::Exceptional
}

fn select_homes(
    config: &GalaxyConfig,
    systems: &[SystemState],
    planets: &[PlanetState],
    total_value: u64,
    civ_count: usize,
    distances: &[Vec<u32>],
) -> Option<Vec<u32>> {
    let yields: Vec<u64> = systems
        .iter()
        .map(|system| {
            system
                .planet_ids
                .iter()
                .map(|id| u64::from(planets[*id as usize].base_yield))
                .sum()
        })
        .collect();
    let neighborhoods: Vec<u64> = (0..systems.len())
        .map(|home| {
            systems
                .iter()
                .enumerate()
                .filter(|(other, _)| distances[home][*other] <= config.neighborhood_hops)
                .map(|(other, _)| yields[other])
                .sum()
        })
        .collect();
    let territory_tolerance = territory_tolerance_units(
        total_value,
        config.starting_territory_milli,
        config.territory_tolerance_percent,
    );
    let eligible: Vec<bool> = yields
        .iter()
        .map(|value| {
            territory_deviation(
                *value,
                total_value,
                civ_count,
                config.starting_territory_milli,
            ) <= territory_tolerance
        })
        .collect();
    if eligible.iter().filter(|candidate| **candidate).count() < civ_count {
        return None;
    }
    let mut best: Option<(u32, u64, u64, Vec<u32>)> = None;
    for first in 0..systems.len() {
        if !eligible[first] {
            continue;
        }
        let mut chosen = vec![first as u32];
        let mut minimum_distance = u32::MAX;
        while chosen.len() < civ_count {
            let next = (0..systems.len())
                .filter(|candidate| eligible[*candidate] && !chosen.contains(&(*candidate as u32)))
                .filter_map(|candidate| {
                    let nearest = chosen
                        .iter()
                        .map(|home| distances[candidate][*home as usize])
                        .min()?;
                    if nearest < config.minimum_home_distance {
                        return None;
                    }
                    Some((candidate as u32, nearest))
                })
                .max_by(|(a, da), (b, db)| {
                    da.cmp(db)
                        .then_with(|| {
                            home_candidate_tie_score(
                                *a,
                                *b,
                                &neighborhoods,
                                &yields,
                                total_value,
                                civ_count,
                                config.starting_territory_milli,
                            )
                        })
                        .then_with(|| b.cmp(a))
                });
            let Some((candidate, distance)) = next else {
                break;
            };
            minimum_distance = minimum_distance.min(distance);
            chosen.push(candidate);
        }
        if chosen.len() != civ_count {
            continue;
        }
        let neighborhood_mean = chosen
            .iter()
            .map(|id| neighborhoods[*id as usize])
            .sum::<u64>()
            / chosen.len() as u64;
        let neighborhood_deviation = chosen
            .iter()
            .map(|id| neighborhoods[*id as usize].abs_diff(neighborhood_mean))
            .max()
            .unwrap_or(0);
        let t0_deviation = chosen
            .iter()
            .map(|id| {
                territory_deviation(
                    yields[*id as usize],
                    total_value,
                    civ_count,
                    config.starting_territory_milli,
                )
            })
            .max()
            .unwrap_or(u64::MAX);
        if !chosen.iter().all(|id| {
            percent_deviation_within(
                neighborhoods[*id as usize],
                neighborhood_mean,
                config.neighborhood_yield_tolerance_percent,
            )
        }) {
            continue;
        }
        let score = (
            minimum_distance,
            neighborhood_deviation,
            t0_deviation,
            chosen,
        );
        let replace = best.as_ref().is_none_or(|current| {
            score.0 > current.0
                || score.0 == current.0 && (score.1, score.2) < (current.1, current.2)
        });
        if replace {
            best = Some(score);
        }
    }
    best.map(|(_, _, _, homes)| homes)
}

fn home_candidate_tie_score(
    a: u32,
    b: u32,
    neighborhoods: &[u64],
    yields: &[u64],
    total_value: u64,
    civ_count: usize,
    target_milli: u32,
) -> std::cmp::Ordering {
    let target_neighborhood = neighborhoods.iter().sum::<u64>() / neighborhoods.len() as u64;
    let a_score = (
        neighborhoods[a as usize].abs_diff(target_neighborhood),
        territory_deviation(yields[a as usize], total_value, civ_count, target_milli),
    );
    let b_score = (
        neighborhoods[b as usize].abs_diff(target_neighborhood),
        territory_deviation(yields[b as usize], total_value, civ_count, target_milli),
    );
    b_score.cmp(&a_score)
}

fn territory_deviation(
    home_value: u64,
    total_value: u64,
    civ_count: usize,
    target_milli: u32,
) -> u64 {
    let actual = u128::from(home_value) * civ_count as u128 * 1_000;
    let target = u128::from(total_value) * u128::from(target_milli / 1_000);
    actual.abs_diff(target).min(u128::from(u64::MAX)) as u64
}

fn territory_tolerance_units(total_value: u64, target_milli: u32, tolerance_percent: u32) -> u64 {
    (u128::from(total_value) * u128::from(target_milli / 1_000) * u128::from(tolerance_percent)
        / 100)
        .min(u128::from(u64::MAX)) as u64
}

fn percent_deviation_within(value: u64, mean: u64, tolerance_percent: u32) -> bool {
    if mean == 0 {
        return value == 0;
    }
    u128::from(value.abs_diff(mean)) * 100 <= u128::from(mean) * u128::from(tolerance_percent)
}

fn all_pairs_distances(systems: &[SystemState], lanes: &[LaneState]) -> Vec<Vec<u32>> {
    let mut adjacency = vec![Vec::new(); systems.len()];
    for lane in lanes.iter().filter(|lane| !lane.blocked) {
        adjacency[lane.a as usize].push(lane.b as usize);
        adjacency[lane.b as usize].push(lane.a as usize);
    }
    let mut all = vec![vec![u32::MAX; systems.len()]; systems.len()];
    for (start, row) in all.iter_mut().enumerate() {
        let mut queue = std::collections::VecDeque::from([start]);
        row[start] = 0;
        while let Some(node) = queue.pop_front() {
            let next_distance = row[node] + 1;
            for neighbor in &adjacency[node] {
                if row[*neighbor] == u32::MAX {
                    row[*neighbor] = next_distance;
                    queue.push_back(*neighbor);
                }
            }
        }
    }
    all
}

fn shuffled_archetypes(count: usize, rng: &mut StdRng) -> Vec<StartingArchetype> {
    let mut result = Vec::with_capacity(count);
    while result.len() < count {
        let mut round = StartingArchetype::ALL;
        round.shuffle(rng);
        result.extend(round);
    }
    result.truncate(count);
    result
}

fn link_lanes(systems: &mut [SystemState], lanes: &[LaneState]) {
    for lane in lanes {
        systems[lane.a as usize].lane_ids.push(lane.id);
        systems[lane.b as usize].lane_ids.push(lane.id);
    }
    for system in systems {
        system.lane_ids.sort_unstable();
    }
}

fn lane_speed_unit(candidates: &[(u32, u32, u32)]) -> u32 {
    let mut lengths: Vec<u32> = candidates.iter().map(|candidate| candidate.0).collect();
    lengths.sort_unstable();
    lengths[lengths.len() / 2].div_ceil(2).max(1)
}

fn is_connected(system_count: usize, lanes: &[LaneState]) -> bool {
    if system_count == 0 {
        return false;
    }
    let mut adjacency = vec![Vec::new(); system_count];
    for lane in lanes.iter().filter(|lane| !lane.blocked) {
        adjacency[lane.a as usize].push(lane.b as usize);
        adjacency[lane.b as usize].push(lane.a as usize);
    }
    let mut seen = vec![false; system_count];
    let mut stack = vec![0];
    seen[0] = true;
    while let Some(node) = stack.pop() {
        for next in &adjacency[node] {
            if !seen[*next] {
                seen[*next] = true;
                stack.push(*next);
            }
        }
    }
    seen.into_iter().all(|value| value)
}

fn distance(a: Point, b: Point) -> u32 {
    let dx = i128::from(a.x) - i128::from(b.x);
    let dy = i128::from(a.y) - i128::from(b.y);
    let square = (dx * dx + dy * dy) as u128;
    let floor = integer_sqrt(square);
    (if floor * floor == square {
        floor
    } else {
        floor + 1
    }) as u32
}

fn segment_intersects_circle(a: Point, b: Point, center: Point, radius: u32) -> bool {
    let ax = i128::from(a.x) - i128::from(center.x);
    let ay = i128::from(a.y) - i128::from(center.y);
    let bx = i128::from(b.x) - i128::from(center.x);
    let by = i128::from(b.y) - i128::from(center.y);
    let dx = bx - ax;
    let dy = by - ay;
    let length_sq = dx * dx + dy * dy;
    let radius_sq = i128::from(radius) * i128::from(radius);
    if length_sq == 0 {
        return ax * ax + ay * ay <= radius_sq;
    }
    let projection = -(ax * dx + ay * dy);
    if projection <= 0 {
        return ax * ax + ay * ay <= radius_sq;
    }
    if projection >= length_sq {
        return bx * bx + by * by <= radius_sq;
    }
    let cross = ax * dy - ay * dx;
    cross * cross <= radius_sq * length_sq
}

fn ceil_div(value: u32, divisor: u32) -> u32 {
    value.div_ceil(divisor)
}

fn richness_index(richness: Richness) -> usize {
    match richness {
        Richness::Poor => 0,
        Richness::Average => 1,
        Richness::Rich => 2,
        Richness::Exceptional => 3,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn cordic_matches_cardinal_directions() {
        assert_eq!(cordic_sin_cos(0), (0, Q30 as i64));
        let (sin, cos) = cordic_sin_cos(90_000);
        assert!(sin.abs_diff(Q30 as i64) < 20);
        assert!(cos.abs() < 20);
    }

    #[test]
    fn integer_segment_circle_test_detects_crossing_and_miss() {
        assert!(segment_intersects_circle(
            Point { x: -100, y: 0 },
            Point { x: 100, y: 0 },
            Point { x: 0, y: 0 },
            10,
        ));
        assert!(!segment_intersects_circle(
            Point { x: -100, y: 20 },
            Point { x: 100, y: 20 },
            Point { x: 0, y: 0 },
            10,
        ));
    }

    #[test]
    fn same_seed_and_civ_set_produce_same_map() {
        let civs: Vec<String> = (0..8).map(|index| format!("team_{index:02}")).collect();
        let config = GalaxyConfig::default();
        let first = generate(&config, &civs).expect("map generation succeeds");
        let second = generate(&config, &civs).expect("map generation succeeds");
        assert_eq!(first, second);
    }
}
