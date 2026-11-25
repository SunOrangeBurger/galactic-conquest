# Galactic Conquest Phase 0 Progress

**As of 25 November 2026, Phase 0 is about 27% complete by milestone count.**
Three of the eleven planned milestones (M0–M10) are marked complete. This
estimate weights milestones equally and counts completed milestones only.
The remaining milestones include the core game loop.

## Milestone status

| Milestone | Status | Evidence |
| --- | --- | --- |
| M0 — Workspace setup | Complete | Rust workspace, CI workflow, and parameter syntax and hash checker. |
| M1 — API v0.4 | Complete | Agent and spectator wire types, JSON Schemas, and example validation. |
| M2 — Deterministic map generation | Complete | Seeded maps, fair home assignments, hazard-free routes, and regression coverage at 8, 12, and 20 civilizations. |
| M3–M5 | Open | Action validation and an HTTP submission skeleton exist. The server starts from an empty galaxy, and the tick resolver and match coordinator are pending. Combat calibration is identified as M4 in the parameters. |
| M6 — Replay verification | Open | State hashing exists, and the replay executable is a stub. |
| M7 — Spectator dashboard | Open | Spectator schemas and examples exist. The dashboard is pending. |
| M8–M10 | Open | The checked-in README identifies the milestone range. The full v0.4 milestone definitions and acceptance checklist are outside this checkout. |

## Approved plan changes

| Area | Change and rationale |
| --- | --- |
| Map density | Increased from 10 to a seeded 18–22 systems per civilization to make five-hop home spacing feasible. Raised the planet density to roughly 2.5 per system to retain viable starting territory. |
| Lane length and hazards | Raised the lane cap from 350 to 700 units so the two-turn spiral could connect at minimum scale. Nebula lanes can take up to eight ticks after their two-tick modifier. The owner also approved seeded scatter for hazards and required a connected network of clear bypass lanes. |
| Richness and home fairness | Approved progressive richness weighting by tier, since a uniform multiplier would cancel during normalization. Approved four-planet home candidates, richness swaps that preserve galaxy-wide totals, and balancing of three-hop neighborhood yields before home selection. |
| Specification clarifications | Economy constant B uses a Balanced civilization’s tick-zero home-system income. The published contract version is 0.4. Both supersede conflicting wording in the earlier materials. |

The original decisions and map defaults are recorded in [DECISIONS.md](DECISIONS.md) and [the M2 notes](M2-mapgen-proposal.md).

## Implementation changes and remaining gaps

- **M2 balancing.** Commit `837fedd` changed the swap objective to penalize all
  neighborhoods outside the fairness band. This addressed repeated retries
  at 20 civilizations. The approved spacing and yield thresholds still
  apply, but the same seed can produce a different galaxy across commits.
- **Hazard placement.** The M2 proposal says the placement rule avoids hazards
  on systems. The current generator checks the hazard center against the core
  and galaxy boundary and checks lane effects, but does not exclude hazard
  circles that overlap systems. The M2 regression suite checks lane effects
  and clear-route connectivity.
- **Configuration and generation metadata.** The map generator uses Rust
  configuration defaults. Editing `params.toml` alone does not change map
  generation or the validator's hardcoded costs. The generated state exposes
  the accepted sub-seed and retry count. Writing these and the parameter hash
  into a match header depends on match orchestration.
- **Playability.** The submission handler validates actions and discards
  accepted orders. Baseline bots exist, but there is no full match to run
  them through. The aggressive bot can pair a target with a fleet in another
  system, which the validator rejects.
- **Reproducibility checks.** At `837fedd`, all 15 workspace tests, formatting,
  and Clippy passed locally on Rust 1.91.1. The repository pins Rust 1.99.0,
  which was not verified locally. On Windows, CRLF checkout conversion can
  cause the exact-text schema check to report a mismatch and change the
  raw-byte parameter hash. The schema and examples passed with LF files.

The next progress baseline is a seeded headless match that consumes orders,
advances ticks, and records state hashes. It would demonstrate that the
completed map and contract milestones work together in a match.
