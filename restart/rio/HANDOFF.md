# Rio source handoff

Owned files are confined to `restart/rio/**`; shared Cargo registration and native
production hooks belong to the coordinator. No build/test/game producer was run
by this worker, so no verification run directory or build intermediate requires
worker cleanup.

Public contract and primary dependencies are in README.md. Shared fixture API:

- `BodyFixture { name, entry, k, arg, wram, steps }`.
- `ServiceKind`: Horizontal=0, Vertical=1, Angle=2.
- `ServiceStep { kind, result: u16, patches: Vec<(usize,u8)> }`.
- `ScriptedServices::new(steps)` records `Call { kind, k, amount: i32 }`; angle
  uses amount zero. `complete()` detects unconsumed requests. `admitted` supports
  atomic refusal and `fail_at` injects terminal errors after copyback patches.

Independent actual-C harness must invoke the matching original definition with
the shared full-WRAM input and patched runtime ROM, compare complete WRAM,
Done/Continue output and ordered H/V/angle calls, and enforce complete script
consumption. A script may change `cur_enemy_index` while cached actor writes
remain at the original base; animation helper writes then target the new index.

Pending gates: coordinator Cargo registration, semantic controls, compiled-C
body comparison, native transaction adapter controls, authentic native terrain
and all-owner composition, natural live encounter, retained authentication and
coordinator cleanup. Scripted services prove actor bodies only. Production
admission must validate approved owned writes before terrain calls because Rio
updates RNG/animation/speeds before requesting terrain. Waver's whole-WRAM
equality admission cannot be reused unchanged.
