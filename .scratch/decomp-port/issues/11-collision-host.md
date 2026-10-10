# Actual PLM callback collision harness
Status: resolved
Blocked by: 07
Triage: ready-for-agent

Build the actual-C callback seam described in /tmp/sm-port-collision-closure.md.
Synchronous PLM setup must preserve full WRAM, displacement mutation through
cur_coll_amt32 and callback return/effects; no record-only substitution. Avoid
overlapping mutable Rust/C borrows by explicit state transfer. Compile actual
pinned bank84 and dependencies, retain identities, and measure remaining C hits.
Own tools/port_collision.py and tests plus harness documentation.

## Completion evidence

Implemented in `2eb17f2` on `decomp-port-collision-harness`, merged current
integration tip before reporting. `tools/port_collision.py` links the actual
pinned C bank tree to a guarded Rust static-library ABI with isolated owned
WRAM transfers. It compares complete final WRAM, raw return, ordered actual
SpawnPLM arguments/amount mutation/return, complete callback before/after WRAM,
and explicit C gameplay callback/CPU fallback measurements. Its retained
artifact consumer rejects changed callback evidence and dependencies.

`python3 -m unittest discover -s tools -p test_port_collision.py` passed on the
final committed helper/guards (56.715 seconds, 2026-10-10). The actual ROM
quicksand callback clamps displacement and changes WRAM identically; wrong Rust
stores and retained callback tampering are rejected. This marked infrastructure
probe is not a gameplay translation claim. Gameplay fixture coverage belongs to
12 and live callback exercise belongs to the integration runner.

Details: `docs/DECOMP_COLLISION_HARNESS.md`.
