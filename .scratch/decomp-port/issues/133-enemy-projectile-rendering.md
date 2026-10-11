# Enemy projectile rendering and earthquake offsets

Status: resolved
Blocked by: None
Triage: ready-for-agent

Translate bank86 8390/83B2 priority traversal, 83D6 camera clipping and 8427
screen-shake values, including bank81 8C0A/8C7F spritemap emission. Preserve
reverse pool order, OAM cursor wrapping and extension-word aliases, signed
vertical offset visibility, low-byte tile addition, ROM table reads and frozen
earthquake behavior. Compare compiled actual C against complete WRAM and raw
returns; ordinary priority-family manifest must support live replay.

## Comments

Claimed by projectile_rendering implementer. Own new rendering module, fixture
generator, fixtures and integration tests; root owns shared module/dispatch
wiring and final frozen-source oracle/live verification.

## Completion evidence

Implemented bank86 8390/83B2/83D6/8427 plus bank81 8C0A/8C7F in maintained
`crates/port/src/enemy_projectile_rendering.rs`. Rendering helper accepts bank,
map, position, base tile/attributes and clipping mode. Emission preserves C's
captured cursor, wrapping OAM writes, extension OR writes before Y/character,
odd cursor positions and cursor/extension metadata aliases. Direct k arguments
floor to even byte slot like C. Shake table multiplication wraps before reading;
negative screen offsets use modulo65536 arithmetic. No projectile-state mutation.

Reproducible `tools/port_eproj_render_fixtures.py` emits the ordinary priority
manifest (1,448 cases, ABI compatible with live wrapping) and a separate direct
manifest (2,468 cases with packed point/leaf C adapters). Both use exact native
ROM, full WRAM/raw-return comparison; direct quake cases supply independently
computed expected returns from native ROM. Coverage includes all eighteen pool
slots, odd direct arguments, signed/offscreen clipping boundaries, native maps
of 0/1/4 entries, low-byte base tile addition, existing OAM extension contents,
priority filtering/reverse order, cursors0/508/511/512/543/FFFF, quake phases,
frozen behavior and wrapped high quake table indices.

Preliminary ordinary actual-C oracle:
`out/port/enemy-projectile-rendering-preliminary/20261011T122235.614147Z/verdict.json`
passed all1,448 cases. Current-source authentication after cleanup reports the
expected sibling `enemy_projectile_sparks.rs` source change, so the integration
owner must replace this preliminary result with final frozen-source evidence.
Finished run cleanup0 bytes and dedup244,580,352 bytes reclaimed. Retain sealed
source snapshots/binaries/ROM and before/after artifacts at this path.

Six independent Rust integration tests pass via standalone actual-lib wrapper:
carry/sign/offscreen selection; cursor wrap/base-tile overflow/extensions;
cursor543 aliases and write order; reverse mixed priority traversal; clipping
short circuit/odd-slot floor; quake phase/freeze/wrapped table. Tests and owned
module pass rustfmt; generator passes Python compilation. Temporary standalone
unit wrapper/build cleaned with utility+dedup (both0), then identified unsealed
build7,248,702 bytes removed. No implementer worktree created. Preliminary free
disk274,721,107,968 bytes. Shared hooks, final frozen-source oracle, live replay
and full-suite checks remain the integration owner's pending actions.

Integration handoff (2026-10-11): rendering export is present in maintained
`crates/port/src/lib.rs`; ordinary `cargo test -p port --test
enemy_projectile_rendering` passes all six semantic tests. Targeted
`rustfmt --check --edition 2024` passes for the module and tests; Python AST
parsing passes for the rendering generator and shared dispatcher oracle tool.
Review against native bank86 8390/83B2/83D6/8427 and bank81 8C0A/8C7F found
no mismatch. Packed leaf C adapters in `tools/port_plm_instructions.py` are
present and use native `DrawEprojSpritemapWithBaseTile` and
`DrawEprojSpritemapWithBaseTileOffscreen`, fixed bank8D, lower/upper XY and
map/GFX halves, tile `GFX & 255`, attributes `GFX & 0xFF00`.

Rendering module/tests/export/shared C adapters are frozen for final actual-C
verification and ordinary-priority live admission. There is no central
`run_rom` dispatcher in current source; ordinary ROM manifests generate ABI
wrappers around the module's `dispatch_rom`. The Cargo test process finished;
the integration owner retains responsibility for combined-run cleanup,
final source-current evidence authentication and ticket resolution. No
implementer worktree was created.


Final frozen integration verification (2026-10-11): all 3,916 actual-C
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
940,179,456 bytes. Commands/logs and remaining space (270,406,627,328 bytes at
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
