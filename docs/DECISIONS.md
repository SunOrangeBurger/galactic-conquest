# Decisions

## M0 — Workspace and parameters

- The v0.4 clarification defines economy constant `B` as the income of a
  Balanced civilization's tick-0 home system. The conflicting older phrase
  “average home-system base yield” is superseded. This was confirmed by the
  project owner on 2026-11-02.
- The workspace pins Rust 1.99.0, the stable toolchain installed when M0
  began; this was confirmed by the project owner on 2026-11-02.
- The API version published by the contract is `0.4`, matching the
  specification version. Section 11's `0.3` example label was stale; this
  was confirmed by the project owner on 2026-11-02.
- M2 map defaults in `docs/M2-mapgen-proposal.md` were approved by the project
  owner on 2026-11-02 and copied into `params.toml`. The map radius, spiral
  turns, jitter, lane cap, hazard radius and population capacities remain
  tuneable parameters for the fairness audit.
- The owner approved the deterministic M2 generation procedure in
  `docs/M2-mapgen-proposal.md` on 2026-11-02, including integer spiral/CORDIC
  placement, farthest-point home selection, fairness tie-breaks and a 10,000
  attempt cap.
- Regional richness multipliers scale progressively by richness tier before
  normalization (Poor ×1, Average ×m, Rich ×m², Exceptional ×m³), so the
  regional factor does not cancel out. This was separately approved on
  2026-11-02.
- Nebula lane travel time may reach 8 ticks when the base 1–6 tick travel
  time receives its +2 tick modifier. The owner confirmed this interpretation
  on 2026-11-02.
- To keep the approved two-turn spiral connected at the minimum 8-civ scale,
  the owner approved increasing the maximum lane length from 350 to 700 units
  on 2026-11-02.
- Scattered void and nebula hazard centers use seeded integer coordinates in
  the galaxy disc; void placements are redrawn if they disconnect the graph.
  The owner confirmed this on 2026-11-02.
- The owner requested a connected non-hazard detour around the impassable
  black-hole core and nebula lanes. The generator will add clear core-safe
  links after hazards when needed, respecting the lane degree cap.
- The 10-system-per-civ map did not provide enough five-hop-separated home
  candidates at eight civs. The owner approved a seeded uniform map density of
  18–22 systems per civ on 2026-11-02; the attempt sub-seed determines the
  selected count and therefore makes it reproducible.
- To preserve viable T0 = 150 homes at this density, the owner approved
  scaling planet count with system count at about 2.5 planets per system
  (45–55 planets per civ) on 2026-11-02.
- The owner approved constructing T0-eligible home candidates while keeping
  four planets per candidate and the galaxy-wide richness totals. The generator
  swaps richness classes and balances three-hop neighborhood yields before
  final home selection and fairness validation.
- `params.toml` is hashed as its exact UTF-8 file bytes using FNV-1a 64-bit.
  This deliberately preserves formatting as part of the configuration
  identity and avoids nondeterministic map iteration. The hash is an
  identifier, not an integrity/security primitive. State hashing remains
  BLAKE3 as specified for the replay log.
