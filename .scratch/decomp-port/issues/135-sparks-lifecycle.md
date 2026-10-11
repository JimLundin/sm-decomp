# Sparks allocation and collision/trail lifecycle

Status: resolved
Blocked by: None
Triage: ready-for-agent

Translate pinned bank86 F391 initializer and F3F0 preinstruction into the
maintained `enemy_projectile_sparks` family. Preserve current enemy position
and fractional copies, RNG and all eight ROM velocity pairs, full 16.16 gravity,
vertical terrain collision, quadrupled horizontal bounce velocity and sprite
allocation every fourth NMI frame. The last ROM pair intentionally extends
past the unused fourteen-word C local constant into following ROM instructions;
the actual ROM-backed upstream access is authoritative.

## Comments

Claimed by sparks implementer; integration owner handles shared allocator,
preinstruction/direct dispatch and final combined-source freeze. Verify compiled
actual C against full WRAM, raw returns and unchanged external host words with
bounded valid room geometry, native scripts, and a controlled chained sequence.
Offline parity does not establish live gameplay ownership.

## Completion evidence

Owned family is `crates/port/src/enemy_projectile_sparks.rs`; shared native
allocation, direct F391/F3F0 and scheduler hooks are present in lib.rs,
enemy_projectiles.rs and enemy_projectile_instructions.rs. The integration
owner's tool stream disconnected while work was ready, so sparks implementer
added the minimal previously agreed hooks and notified both siblings. No commits
or additional worktrees were created. Whole-game and live ownership remain open.

Actual-C comparisons exposed a wrong fixture definition (F345 is adjacent ROM
data; native `addr_kEproj_Sparks` is F498) and an implementation palette alias
(eproj_gfx_idx is WRAM 19BB; 19DF is timer). Both were corrected before accepted
evidence. Failures remain sealed under the three `sparks-*-preliminary` roots.

Current source full-WRAM/raw-return comparisons and all eight external host
words match in all 3,154 cases:

- `out/port/sparks-allocation-corrected/20261011T122702.268439Z/verdict.json`:
  801 cases, 152 native allocations covering every free-slot choice and saturation,
  648 direct initializer cases plus a native delete control. Enemy endpoints,
  coordinate/fraction wrapping, RNG boundaries and all eight velocity pairs are
  exercised. ROM pair seven deliberately reads DBBD/301A from F3F0 instructions.
- `out/port/sparks-update-corrected/20261011T122702.270959Z/verdict.json`:
  1,677 cases; signed upward movement, downward carry/speed boundaries, every
  slot, full-word horizontal quadrupling, collision return, all NMI trail phases,
  sprite saturation/free slots, slope variants and extension chains, and direct
  native F3F0 calls.
- `out/port/sparks-frames-corrected/20261011T122702.272796Z/verdict.json`:
  578 cases; native scripts and all slot/timer/trail/collision phases through the
  actual C descending scheduler, with a native allocation and delete control.
- `out/port/sparks-sequence-corrected/20261011T122736.367203Z/verdict.json`:
  98 cases; native allocation plus 96 chained actual-C frame states and delete
  control. Controlled offline inputs set initial projectile enable, enemy
  (128.FE37, 120.FE73), RNG seed zero, room width 64 and solid ground at y160;
  each frame changes only NMI byte 05B5 to the ordinal. The native script lands
  around frame14, deletes by frame30 and allocates seven sprite trails. All other
  state flows from the prior actual-C output. This is a declared controlled
  lifecycle sequence, not live gameplay or original-ROM parity.

`tools/port_sparks_fixtures.py` reproducibly emits all four manifests/captures.
Four independent semantic tests pass (`cargo test -p port --test
enemy_projectile_sparks`), covering fractional double movement, the ROM overread,
landing early return and byte-indexed final-position trail allocation.
`cargo clippy -p port --test enemy_projectile_sparks -- -D warnings` passes.
Combined full-suite checks remain the integration owner's responsibility.

Scoped cleanup ran `tools/port_cleanup.py --root <root> --apply` followed by
`--deduplicate --apply` for all seven completed sparks run roots. Ordinary
cleanup reclaimed zero; immutable sharing reclaimed 257,687,552 update bytes,
105,119,744 allocation bytes, 75,890,688 frame bytes and 34,865,152 sequence
bytes: total 473,563,136 bytes. Three diagnostic roots reclaimed zero bytes.
All binaries, exact candidate/dependency sources, manifest, pinned ROM, host
states, before/after WRAM and failure evidence remain retained; the small chained
inputs under `out/port/sparks-sequence-inputs` are required manifest dependencies.
Approximately 256 GiB remains free. No worktree/build tree needs removal.

After cleanup, `verify_plm_instruction_evidence` authenticates every final
verdict against the current source and every retained artifact. Results are
recorded in `.scratch/decomp-port/sparks-authentication.json`. Any later shared
source edits require refreshing final seals; these results authenticate the
current integrated sparks source, not future changes.

Combined contact/rendering integration refresh (2026-10-11): reran all four
checked-in manifests using `tools/port_plm_instructions.py --manifest
tools/port/sparks-<family>.json --source deps/snesrev-sm --output
out/port/projectile-contact-render-final/sparks-<family>` after the lib exports
and packed rendering C adapters froze. No source or manifest edits were needed.
All 3,154 actual-C full-WRAM/raw-return cases and external host words still
match:

- Allocation: `out/port/projectile-contact-render-final/sparks-allocation/20261011T125902.098529Z/verdict.json`, 801 cases.
- Update: `out/port/projectile-contact-render-final/sparks-update/20261011T125901.927889Z/verdict.json`, 1,677 cases.
- Frames: `out/port/projectile-contact-render-final/sparks-frames/20261011T125902.014607Z/verdict.json`, 578 cases.
- Sequence: `out/port/projectile-contact-render-final/sparks-sequence/20261011T125902.084738Z/verdict.json`, 98 cases.

All four writers finished before scoped `tools/port_cleanup.py --root
out/port/projectile-contact-render-final/sparks-<family> --apply` and
`--deduplicate --apply`. Ordinary cleanup reclaimed zero bytes;
deduplication reclaimed 105,119,744 allocation bytes, 257,687,552 update bytes,
75,890,688 frame bytes and 34,865,152 sequence bytes: 473,563,136 bytes total.
Available disk after cleanup was 268,490,203,136 bytes. Sealed artifacts,
source snapshots, binaries, pinned ROM, all host/WRAM outputs and required
`out/port/sparks-sequence-inputs` manifest dependencies remain retained.
No implementer worktree was created; no shared Cargo outputs were removed.

After cleanup, `verify_plm_instruction_evidence` authenticated all four final
verdicts against the current combined source and retained artifacts. Exact
results are recorded in
`.scratch/decomp-port/sparks-integration-authentication.json`. This replaces
the earlier seals for current-source acceptance; whole-game/live sparks
ownership remains open.
