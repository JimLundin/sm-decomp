# Maridia N00b tube crack, shards and released bubbles

Status: resolved
Blocked by: None
Triage: ready-for-agent

Own the complete bank86 D6A5/D6C9/D774 tube allocation, D5E1/D5F2/D62A/D69A
script instructions and D7BF/D7DE/D7FD/D83D/D89F/D8DF callbacks in a maintained
family module. Preserve PLM coordinate calculation, shard script ROM table,
overlapping RNG read, fixed-point motion, frame parity, global projectile-index
aliasing and signed camera culling. Verify with compiled actual C, complete WRAM,
raw returns, unchanged external host words and native-ROM scheduled frames.

## Comments

Claimed by n00b_tube implementer. Existing inline implementations are migrated
through integration-owner wiring; meaningful new evidence must cover the whole
family rather than infer parity from existing translations.

## Completion evidence

Integration owner verifies the combined 263-test port suite and strict all-target
Clippy pass. The Python family/PLM/live harness suite passed 29 tests with one
opt-in live control skipped; temporary test directories removed themselves.
New modules/tests pass rustfmt and the combined diff passes whitespace checks.

Owned family now lives in `crates/port/src/n00b_tube.rs` with allocator and
instruction/preinstruction hooks in shared native dispatch. D6A5/D6C9/D774,
D5E1/D5F2/D62A/D69A and all six D7BF/D7DE/D7FD/D83D/D89F/D8DF callbacks preserve
odd/even frame projection, overlapping RNG read, signed 8.8-to-16.16 movement,
global projectile-index aliasing and signed camera culling. Initializers clear
E explicitly, matching direct actual-C calls with stale E rather than relying
on allocator reset. Missing native PLM D525 water-physics enable, D536 diagonal
earthquake and D543 ordered ten-shard/six-bubble allocation are implemented by
integration owner; existing D52C crack caller remains connected.

Final current-source actual-C evidence (all complete WRAM/raw-return parity;
projectile cases also compare all eight external host words):

- `out/port/n00b-tube-final/20261011T120623.657444Z/verdict.json`: 1,477 cases
  (300 allocations, 297 direct static initializers, 512 callbacks, 256 opcodes,
  112 scheduled native-script frames).
- `out/port/n00b-tube-callers-frozen/20261011T120836.264717Z/verdict.json`:
  152 native-ROM PLM caller cases, all pool free-slot counts 0–18 and distinct
  argument/global PLM slots. Earlier identical current-source final caller
  seal at `out/port/n00b-tube-callers-final/20261011T120633.445726Z/verdict.json`.
- `out/port/n00b-tube-sequence-final/20261011T120700.004624Z/verdict.json`:
  97 cases including 96 consecutive full-state captures from actual C's
  sixteen-effect PLM allocation under controlled offline scheduler inputs:
  before the first step WRAM 198D (projectile enable) is set to 8000 and 0915
  (camera Y) to 1024; before each step WRAM 05B6 (NMI frame counter word) is
  set to that step's frame ordinal 0–95. All other state flows from the prior
  actual-C output. This is an offline frame sequence with declared frame inputs,
  not an unmodified full-game trajectory. Both bubbles D8DF→D89F and shards D7FD→D83D
  execute, followed by bubble deletion and signed camera deletion; seven
  shards remain at frame 95. Independent C captures and native opcode control
  are checked against both newly compiled C and Rust.

Final authoritative coverage totals 1,726 cases: 1,477 tube + 152 frozen
PLM caller + 97 sequence cases. Earlier duplicate caller evidence is excluded
from this sum.

`tools/port_tube_fixtures.py` reproducibly emits all three manifests and the
chained C frame captures. Four independent Rust tests pass for separate phase
source/global destination slots, overlapping RNG scratch byte, fractional carry
at signed camera boundary and partial-pool spawn order/final parameter.
`cargo test -p port --test n00b_tube` and strict targeted Clippy pass. Root owns
full-suite checks and repository formatting; `cargo fmt --all --check` still
reports existing/shared formatting and no frozen-source mutation was made.

Finished-run cleanup applied with `tools/port_cleanup.py --root <run> --apply`
then `--deduplicate --apply` to every tube run. Ordinary cleanup reclaimed zero
bytes throughout. Snapshot deduplication reclaimed 314,966,016 bytes final
tube, 25,296,896 final sequence, 34,865,152 frozen caller, 34,865,152 earlier
final caller; preliminary tube/caller runs reclaimed 260,702,208 / 34,865,152
bytes. Total 705,560,576 bytes. Sequence inputs cleanup/dedup both zero; retain
25,178,144 bytes of source captures at `out/port/n00b-tube-sequence-inputs` for
manifest provenance along with sealed ROM/binaries/source snapshots and copied
before/after states. Every final verdict reauthenticated against current source
after cleanup via `verify_plm_instruction_evidence`; frozen caller separately
reauthenticated after its cleanup. No implementer worktree was created; existing
unrelated worktrees remain untouched. Free disk after cleanup: 281,710,186,496
bytes. Offline equivalence only; live caller ownership and original-ROM replay
remain separate open requirements. Integration branch `decomp-port-integration`;
no commit requested or made.
