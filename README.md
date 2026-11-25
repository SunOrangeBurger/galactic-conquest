# Galactic Conquest — Phase 0

This repository implements Phase 0 of the Galactic Conquest specification
v0.4. Work proceeds through milestones M0–M10 in order. Alliances, technology
trees, civil war, espionage, sabotage, dark forest broadcasts, loans and fleet
merge/split remain out of scope.

## Workspace

- `crates/contract` — public agent and spectator wire types.
- `crates/sim` — deterministic fixed-point state, map generation, hashing and
  baseline action validation; tick resolution is still in progress.
- `crates/server` — HTTP server and match orchestration.
- `crates/replay` — replay verification and history export, delivered in M6.
- `crates/bots` — baseline agents and smoke scenarios.
- `dashboard` — offline spectator frontend, delivered in M7.
- `tools` — calibration and build utilities.
- `params.toml` — named tuning parameters; its byte hash is emitted by
  `params-check` and will be recorded in match headers.

## Build

Rust 1.99.0 is pinned in `rust-toolchain.toml`.

```sh
cargo build --workspace
cargo test --workspace
cargo run -p params-check -- params.toml
```

M0 (workspace setup), M1 (API v0.4 contracts and schemas), and M2
(deterministic map generation) are complete. The [M2 map-generation
notes](docs/M2-mapgen-proposal.md) document the defaults, fairness checks, and
verification command.

See [Phase 0 progress](docs/PROGRESS.md) for the milestone estimate and
deviations from the plan.

The simulation tick loop and match orchestration are still in progress.
Replay verification and the spectator dashboard are scheduled for M6 and M7.
