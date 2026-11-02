# M2 map generator defaults — owner approved

The v0.4 specification defines the map's shape and fairness checks but omits
several numeric values and tie-breaking details. The project owner approved
these defaults and the generation procedure on 2026-11-02. They remain
tuneable parameters for the fairness audit.

| Item | Proposed default | Rationale |
| --- | --- | --- |
| Systems per civ | Seeded uniform integer 18–22 | The owner approved this range to provide enough candidate home systems for eight starts at five-hop spacing under the 700-unit lane cap and degree 3–5. |
| Coordinate radius | 1,000 integer units | Gives a simple bounded coordinate plane; core radius follows the specified 15% rule (150 units). |
| Spiral shape | Four arms over two turns, with seeded integer jitter of at most 25 units | Makes the specified logarithmic four-arm layout concrete while keeping coordinates integer-only. |
| Maximum lane length | 700 units | Allows adjacent outer systems on the approved two-turn spiral to connect at 8-civ map scale. |
| Lane speed unit | `ceil(median candidate-lane length / 2)` | Makes a typical (median) lane take two ticks using the specified ceiling rule. |
| Hazard-zone radius | 50 units | A small, visible local hazard relative to the 1,000-unit map radius. |
| Population capacity by richness | Poor 1,000; Average 2,000; Rich 3,500; Exceptional 6,000 | Preserves the same relative scale as the specified yields (4, 8, 14, 24), with a 1,000-person Poor baseline. |
| Regional richness odds | Scale by richness tier: Poor ×1, Average ×m, Rich ×m², Exceptional ×m³, then renormalize | Makes high-m regions richer and low-m regions poorer without changing total probability. |
| Bridges | Two per adjacent arm pair | Uses the lower end of the specified 2–3 bridge lanes. |
| Selection ties | Break by stable ascending system/lane id | Keeps map generation and fairness checks deterministic. |
| Scattered hazard centers | Seeded integer points sampled uniformly in the galaxy disc, outside the black-hole core | Avoids placing hazards on systems while retaining reproducible placement. |
| Core and nebula bypass | Ensure the graph of lanes untouched by all hazards and the core is connected; add shortest eligible links (up to degree 5) after hazard placement if needed | Provides an alternate clear route when direct lanes are blocked or slowed. |

The map radius, spiral turns, jitter, hazard radius, lane cap, capacity values,
and systems-per-civ range are named in `params.toml` so the fairness audit can
tune them.

## Deterministic generation procedure

These algorithm details are not fixed by v0.4 and affect maps and starting
fairness. The owner approved this procedure:

1. Draw systems per civ uniformly from 18 through 22 using the attempt seed.
   Place systems on each arm along an integer geometric progression from the
   core edge to the map radius. Use a Q30 CORDIC sine/cosine implementation
   and seeded integer jitter; no runtime floating point.
2. Build each arm's local lanes, add the configured bridge lanes between
   adjacent arms, then add nearest-neighbor lanes up to degree 3–5 while
   respecting the maximum length and core exclusion. Reject/reseed if the
   graph is disconnected after hazard application.
3. Place `ceil(system_count / 20)` hazards at seeded integer points in the
   galaxy disc, split
   as evenly as possible between void and nebula. A void hazard blocks every
   lane whose segment intersects its radius; a nebula marks each lane whose
   segment intersects it. Reject and redraw a zone that affects no lane; a
   void zone is also redrawn if it disconnects the graph.
4. Generate exactly 25 planets per 10-system civ block: assign two per
   system, then give one extra planet to five systems selected by seeded
   shuffle. For other counts, distribute the remainder by the same seeded
   shuffle. This keeps the requested approximate 2.5 average.
5. Choose home systems with deterministic farthest-point selection, stable-id
   tie breaks, and a five-hop minimum. Among feasible sets, prefer the set
   minimizing the largest absolute home-neighborhood-yield deviation, then
   the largest T0 deviation. Reject and reseed if either fairness threshold
   fails.
6. Retry failed seeds using monotonically increasing sub-seeds, with a cap
   of 10,000 attempts; report a generation error if none passes.
7. After hazard placement, ensure a connected route made entirely of
   non-blocked, non-slow lanes that do not intersect the black-hole core or
   any scattered hazard. Add eligible clear lanes up to the degree cap of 5
   as needed, preferring shorter links. This provides a clear detour around
   the central core and around slower nebula lanes.

The outer attempt uses the match seed for attempt zero and `seed + attempt`
for later attempts, with checked integer addition. The match header records
the final sub-seed and the number of retries.
