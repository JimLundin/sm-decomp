# Mother Brain glass-shattering family and PLM callers
Status: resolved
Blocked by: None
Triage: ready-for-agent

Own native CDC5 shard and CE6D sparkle allocation, CE9B fractional movement,
probabilistic sparkle creation, native script animation/deletion, and bank-84
D331/D30B PLM callers. Existing inline allocation/motion are migrated into a
readable module; expose previously absent native callable entry points. Verify
against pinned actual C using actual unmodified ROM, full WRAM/raw returns and
all eight host words, including descending scheduled frames, RNG boundaries,
allocation exhaustion and position aliasing. Restrict C inputs to valid even
projectile and PLM slots, shard parameters 0..5 and sparkle source parameters
0..35 (C floors parameter/2). Original-ROM/live ownership remains separate.

## Comments

Claimed by glass_shattering subagent. Root owns integration in shared allocator,
instructions, PLM instructions and lib.rs; this agent owns module, fixture tool,
manifests and tests. Current inline family already handles allocation/movement;
this increment consolidates that contract and adds direct native callable
coverage and complete tightly coupled caller/script verification.

## Completion evidence

Consolidated CDC5/CE6D initializers, CE9B fractional arc/probabilistic sparkle
allocation and D331/D30B caller ownership in `glass_shattering.rs`. Shared root
integration delegates the existing inline paths and exposes native named PLM
callers and direct `EprojInitializer` dispatcher. The reusable C shim invokes
actual static initialization without copying C behavior. Existing inline runtime
behavior was already present; this increment improves family ownership, native
callability, valid-input preflight and exhaustive actual-C verification. No new
live replacement claim.

Reproducible manifests from `python3 tools/port_glass_fixtures.py` cover native
script tables, all RNG phase bytes, wrapping/fractional carry, pool saturation,
source/destination aliasing, native low-byte width including zero divisor,
probability-mask branches, all eight shard animation loops, complete sparkle
script/deletion and descending same-frame scheduling. Five independent Rust
tests pass, including fixed-point cull with unchanged RNG, odd sparkle source
floor, six-draw partial four-shard allocation, native sparkle deletion and
malformed initializer rejection before mutation. The initial hand-calculated
six-draw RNG/audio offsets in one test were corrected and the tests rerun.

Preliminary motion/allocation/frames runs pass; preliminary caller run rejects
an invalid sound ring cursor in the fixture. Corrected fixture uses native
channel-3 read/write bytes 0x645/0x648. All preliminary runs have scoped cleanup
and retained immutable artifact authentication; dependency seals are historical
because another concurrently integrated family was formatted before source
freeze. Diagnostic refusal is preserved. Preliminary cleanup reclaimed
495,976,448 bytes (85,065,728 motion; 262,930,432 allocation; 147,980,288 frames;
zero caller bytes). Source freezing and final current-source checks follow.


All five frozen current-source actual-C full-WRAM/raw-return/eight-host-word
runs pass and authenticate after cleanup (3,869 cases total):

- Allocation 1,643: `out/port/mother-brain-glass-allocation-final/20261011T120733.263081Z/verdict.json`.
- Motion 649: `out/port/mother-brain-glass-motion-final/20261011T120731.822251Z/verdict.json`.
- Scheduled scripts/frames 1,069: `out/port/mother-brain-glass-frames-final/20261011T120734.107333Z/verdict.json`.
- Native PLM callers 379: `out/port/mother-brain-glass-callers-final/20261011T120735.363930Z/verdict.json`.
- Actual-C chained trajectory 129: `out/port/mother-brain-glass-sequence-final/20261011T121040.700517Z/verdict.json`.

Run manifests with `python3 tools/port_plm_instructions.py --manifest
tools/port/<family>.json --source deps/snesrev-sm --output out/port/<family>`.
Regenerate trajectory inputs with `python3 tools/port_glass_fixtures.py
--sequence-source out/port/mother-brain-glass-frames-final/20261011T120734.107333Z
--caller-source out/port/mother-brain-glass-callers-final/20261011T120735.363930Z`.
The declared offline routine sequence begins with actual D30B four-shard
allocation from the seeded caller fixture and then captures 128 calls to the
native C `EprojRunAll` scheduler. Before the first call, the generator writes
0x8000 to WRAM 0x198D to enable scheduling; inspection of retained states
confirms those two bytes are the only initial changes. The generator performs
no per-call NMI-counter update or other WRAM input writes. The seeded NMI
counter at 0x05B6 remains 0xA5A5, and every one of the 127 next before-states
equals its preceding actual C after-state byte for byte. External host words
are explicitly supplied unchanged on every call. Per-call captures check
candidate outputs. These are controlled offline scheduler calls without the
whole-game frame/NMI/hardware loop or movie inputs; they do not establish a
live unmodified movie trajectory. Call 0 has four shards/one sparkle, call 16
three shards/14 sparkles, call 64 zero shards/two sparkles, and by call 96 every effect
has deleted. Sparkles are observed on 65 calls. No host state changes occur.

Five independent Rust tests, module/test formatting, Python compilation and
whitespace checks pass. Root owns aggregate crate, Clippy and harness checks.
After every finished final root ran `python3 tools/port_cleanup.py --root
<root> --apply` and `--deduplicate --apply`. Ordinary cleanup zero bytes;
final dedup reclaimed 634,781,696 bytes: allocation 262,930,432; motion
85,065,728; frames 147,980,288; callers 80,347,136; sequence 58,458,112.
Combined preliminary/final reclaimed 1,130,758,144 bytes. No implementer Git
worktree created. Final free disk 281,092,874,240 bytes. All nine retained
run inventories authenticate; five final runs additionally pass current-source
full-state/return/host verification. Recorded in
`.scratch/decomp-port/glass-authentication.json`.
Retain ROM/binary/source snapshots, host sidecars and captured before/after
sequence inputs. Offline family equivalence is established; live replacement,
original-ROM movie parity and whole-game ownership remain open.

Integration owner ran the combined 263-test port suite and strict all-target
Clippy successfully. The Python family/PLM/live harness suite passed 29 tests
with one opt-in live control skipped; temporary test runs removed themselves.
New modules/tests pass rustfmt, and the combined diff passes whitespace checks.
Shared repository formatting debt remains outside the new family modules.
