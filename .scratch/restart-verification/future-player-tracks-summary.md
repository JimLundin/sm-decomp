# Bounded player and weapon source findings

Source researcher:/root/live_tank_route_recovery; root consolidates its early
findings after stopping further exploration at the time bound. This is a source
candidate map, not complete call closure or an implementation assignment.

| Candidate | Exact recovered-C anchors reported | Reusable main Rust | Proposed restart ownership |
| --- | --- | --- | --- |
| Normal movement/contact | Beta90E722 ->90E725 ->RunSamusMovementHandler90E94B;90A337 is the28-handler TABLE anchor, not an executable kernel root | collision.rs maps all28normal movement handlers; samus.rs/samus_control.rs contain player helpers. Full state/contact/asset domain still needs scope freeze. | restart/player-motion/; movement/collision files stay with one owner. |
| Input/pose state | Alfa90E692 ->90E695 ->Samus_CallInputHandler90E90F ->918000 | Main input/pose/control implementations are reuse material. Input/pose transition tables and callable domain must be selected before assignment. | restart/player-input/; shared frame scheduler remains integrator-owned. |
| Weapon selection/creation | Addressless Samus_HandleHudSpecificBehaviorAndProjs, sm_90.c:5278, dispatches28movement selectors | projectile_flight.rs contains charged/hyper/missile firing beyond restart's retained ordinary uncharged domain. Existing flight/special/power-bomb modules supply further reuse candidates. | restart/player-weapons/; changes to existing restart projectiles get a separate exact claim. |

Recovered C authority is578f90b in the retained scroll/native source tree;
main Rust source is193cad9 reuse, not current restart acceptance. Input, movement
and weapon source edits can be separated after their interface contracts are
frozen. Runtime writes still follow the original scheduler and shared state order.

The complete beta root remains research-needed for gamma/minimap/hit/palette
and scheduler composition. Owning movement kernels inside native beta does not
establish whole-frame Rust ownership. Existing camera/projectile/item acceptance
is not relabeled as acceptance of these candidate domains. Player contact and
enemy damage/grab interact through explicit service contracts, not overlapping
source ownership.

Research produced early findings but no final child note before its time bound;
root stopped exploration and publishes this limited summary. No build/game/test/
capture, implementation claim or source mutation. Cleanup0bytes; unresolved
closure remains future scope work, not an unbounded prerequisite for this chart.

Later bounded contract research26 refines the first candidate to vertical
motion/contact: Samus_MoveUp9093EC and Samus_MoveDown909440, solid-enemy
checkerA0A8F0 and bounded position/displacement/pose-feedback helpers. Terrain
949763 remains a typed synchronous native service. Exact admission/state/fixture
details belong in next-player-motion-contract.md when complete; no28-mode or
full beta ownership follows from this smaller kernel proposal.
