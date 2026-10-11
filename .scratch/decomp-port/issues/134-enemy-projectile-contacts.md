# Shared enemy projectile contact detection
Status: resolved
Blocked by: None
Triage: ready-for-agent

Translate pinned A09894/A09923 Samus contacts and A0996C/A099F9 projectile
contacts into `enemy_projectile_contacts.rs`, with ordinary ROM dispatcher
hooks. Preserve descending enemy projectile / ascending player projectile
iteration, C integer promotion in overlaps, flag gates, damage/frozen-health
behavior, definition instruction lists, sprite/sound allocation, and the dud
sprite's full-WRAM player-projectile position alias indexed by enemy slot.
Use even enemy projectile slots 0..34, player projectile slots 0..8, actual
ROM definitions and valid sound cursors. Compare compiled actual-C full WRAM
and raw returns and add independent semantic tests. The assert(0) branch in
SamusProjectileInteractionHandler is outside this ticket.

## Comments

Claimed by projectile_contacts subagent. Integration owner handles lib/shared
dispatcher hooks, final source freeze, cleanup and authentication.

## Completion evidence

Implemented all four native A0 routines in
`crates/port/src/enemy_projectile_contacts.rs`. Native overlap uses signed C
integer promotion after abs16, including zero axis radii inside Samus and
the abs16 signed-minimum result. Detection checks invincibility only once,
continues descending through lower enemy slots after damage, and publishes
the final collision index/knockback direction. Suit division precedes the
Samus_DealDamage exact-300 sentinel and frozen-health gate; health subtraction
wraps and clamps its signed result. Hit/shot instruction lists come from
actual bank86 definitions. Player projectile detection uses 32-pixel cell
equality, native shot filters, ascending order and flag-two suppression;
cleared enemy properties do not stop subsequent shots in the same inner
loop. Dud sprites intentionally read player projectile positions at enemy
slot k, including full-WRAM aliases beyond nominal ten player slots. Sprite
allocation and sound quotas use existing shared translated helpers.
Normal shots set native null preinstruction 84FB, preserve the low12 damage
bits and copy the actual player projectile type into G. The unrelated
SamusProjectileInteractionHandler assert(0) path remains outside this scope.

Reproduce manifests with `python3 tools/port_eproj_contacts_fixtures.py`, then
run each using `python3 tools/port_family.py verify --manifest
tools/port/<family>.json --source deps/snesrev-sm --output out/port/<family>`.
Four bounded families total 8,070 actual-C full-WRAM/raw-return cases:

- Native Samus handler, 1,728: enemy-projectile-hit-samus.
- Descending Samus detection, 2,310: enemy-projectile-detect-samus.
- Native shot handler plus dud pool pressure/audio gates, 1,992:
  enemy-projectile-hit-projectile.
- Cell filters/flag suppression/ascending order, 2,040:
  enemy-projectile-detect-projectiles.

Initial smoke runs all pass, but these are not final acceptance because
shared integration changes remain pending:
`out/port/enemy-projectile-hit-samus/20261011T122153.625954Z/verdict.json`,
`out/port/enemy-projectile-detect-samus/20261011T122235.897340Z/verdict.json`,
`out/port/enemy-projectile-hit-projectile/20261011T122403.505490Z/verdict.json`,
`out/port/enemy-projectile-detect-projectiles/20261011T122237.066063Z/verdict.json`.
The earlier 1,800-case shot manifest smoke also passed at
`out/port/enemy-projectile-hit-projectile/20261011T122234.704758Z/verdict.json`.

All seven independent semantic tests in
`crates/port/tests/enemy_projectile_contacts.rs` pass through a temporary
rustc harness exposing the module; ordinary Cargo testing awaits integration
owner lib registration. Both new Rust files pass rustfmt, and the fixture
generator passes py_compile. Initial smoke cleanup completed across all four
finished family roots using `tools/port_cleanup.py --root <root> --apply`
followed by `--deduplicate --apply`. Ordinary cleanup reclaimed zero bytes.
Dedup reclaimed 226,492,416 Samus-handler bytes, 302,776,320 Samus-detector
bytes, 968,884,224 shot-handler bytes across its two runs, and 267,386,880
shot-detector bytes: 1,765,539,840 bytes total. Available disk afterward was
275,640,168,448 bytes. No run writer remains; required sealed artifacts and
dependency source snapshots are retained. Temporary direct-test harness
files were removed. No implementer worktree was created. Final source-frozen
reruns, Cargo verification, repeat final-run cleanup and source-current
evidence authentication remain explicitly pending with the integration owner.
Post-cleanup authentication succeeds for the expanded shot-handler smoke.
The other three smoke verdicts reject source-current acceptance because the
concurrently edited `enemy_projectile_sparks.rs` dependency changed; this is
why final reruns after all agents freeze are required.

Contact integration handoff (2026-10-11): exported
`enemy_projectile_contacts` from `crates/port/src/lib.rs`. Ordinary
`cargo test -p port --test enemy_projectile_contacts` now passes all seven
independent semantic tests. Targeted `rustfmt --check --edition 2024` passes
for the module and test. Review against native `sm_a0.c` A09894/A09923/A0996C/
A099F9, A0A45E suit division, 91DF51 damage, and shared sprite/audio helpers
found no contact-only mismatch. Module, tests and export are frozen for the
integration owner's final actual-C families and live replay. There is no
central `run_rom` dispatcher in the current worktree; ordinary ROM manifests
generate synchronous ABI wrappers around existing `dispatch_rom`, with
native signatures `(void)`, `(uint16 k)`, `(void)`, `(uint16 k,uint16 j)` and
void returns for the four routines in that order.

The Cargo test process finished before cleanup. Its unsupported run format
has no sealed artifacts; removed only the identified contact test binary and
dependency file beneath `target/debug/build/port/5508750a98d33221/out/`,
reclaiming 8,979,460 bytes. Available disk after cleanup:
274,529,824,768 bytes. No implementer worktree was created. Final
source-current actual-C verification, live admission, family cleanup and
authentication remain with the integration owner.


Final frozen integration verification (2026-10-11): all 8,070 actual-C
full-WRAM/raw-return cases pass under
`out/port/projectile-contact-render-final/`. Exact verdict paths, routine
coverage and post-cleanup source authentication are recorded in
`.scratch/decomp-port/contact-render-authentication.json`. The ordinary
rendering/contact manifests use `tools/port_family.py`; packed direct rendering
adapters use `tools/port_plm_instructions.py` and call the exact native C
bank81 functions without retyping their algorithms.

Combined port suite: 280 tests pass; strict all-target Clippy passes. New
rendering/contact/sparks modules and tests pass targeted rustfmt. The broad
workspace formatting check reports existing discrepancies in projectile, PLM
and room source from earlier increments; this run does not claim that check
passed. Python syntax checks pass. The live harness active-pool capture control
passes; full Python harness results are recorded separately when complete.

Finished-run cleanup used `python3 tools/port_cleanup.py --root <family-root>
--apply` followed by `--deduplicate --apply` for each of this ticket's final
roots. Ordinary cleanup reclaimed zero bytes; immutable sharing reclaimed
1,057,751,040 bytes. Commands/logs and remaining space (270,406,627,328 bytes at
cleanup) are in `.scratch/decomp-port/contact-render-cleanup.json`. No
implementer worktree was created. Sealed source snapshots, native ROM, binaries,
manifests and full before/after WRAM remain retained. Evidence was authenticated
after cleanup. Live admission proceeds separately under ticket136; original-ROM
parity and full Rust gameplay ownership remain unproven.

Full Python harness validation: `python3 -m unittest discover -s tools -p
'test_port_*.py'` passes 44 tests with one opt-in live control skipped; the
new active-capture test is also included in the passing 17-test live-tool suite.
Root target cleanup utility and dedup each reclaimed zero bytes (unsupported
root build cache is retained for continued work); no integrated worktree remains.

Batch integration registers these ordinary families in `tools/port/batch.json`;
`projectile-render-contacts` and `projectile-render-hits` profiles preserve the
separate detector/handler execution measurements. Explicit frozen batch record
`out/port/projectile-contact-render-final/batch.json` points at final evidence;
both replay profiles construct three authenticated live contracts. Five batch
orchestration tests pass. A final aggregate cleanup of the finished offline
batch reclaimed zero ordinary bytes and 524,288 additional shared snapshot
bytes; file hashes and retained dependencies are unchanged.
