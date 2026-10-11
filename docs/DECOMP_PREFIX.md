# Bounded original-ROM prefix investigation

Run `python3 tools/decomp_prefix.py --frames 320` with local verified assets.
In a temporary worktree, add `--asset-root /home/jim/sm-decomp`. Evidence defaults
beneath `out/decomp/prefix`; choose `--output` to retain it elsewhere. Exit 2 means
the investigation completed with a **nonaccepting** result; exit 1 is an error.
This command cannot emit a passing movie verdict.

The read-only lsnes observer runs the unmodified original ROM in the recording
core (rr2-beta23 / bsnes v085 Compatibility). It records actual fetches at reset,
boot clearing, completed main dispatch, input entry/return, NMI entry, explicit
wait, upload entry and message entry/return. Applicable events are retained;
a 320-frame prefix does not reach a message box. Controller register callbacks
record actual $4218/$4219 read values, with registration addresses captured in
closures rather than relying on lsnes' memoryspace-global callback addresses.
Each completed input call must have one matching read pair. WRAM reads are
passive; the observer never debug-reads hardware registers or changes inputs.
Sequence numbers distinguish multiple events in one movie frame. Completed
frames, emulator log completion/core/region, ROM/movie hashes, Lua script and
binary hashes bind the capture; full WRAM is retained at completed dispatches
and boot clearing. The waiting-for-NMI field is read as a byte.

`timeline.json` retains every source event, actual held/pressed/released inputs,
source authored masks and host-step assignments. Every native controller return
becomes one diagnostic C host step. Frames without a host step are explicitly
classified as skipped with all their boundary evidence retained. Explicit waits
are retained as unproved coroutine mappings. No repeated-position heuristic
removes intentional waits, and no search for later matching state or corrective
state injection occurs. The authored pad lookup uses lsnes' source-frame index;
actual register-read values, rather than that lookup, drive replay.

This map is diagnostic and unaccepted. The first unsupported boundary is the
prelude controller poll at source frame 19: it occurs before the first native
main dispatch, while the C boot coroutine offers no equivalent poll boundary.
Replaying a poll through a C host iteration therefore does not prove that the
same input was consumed at the same gameplay boundary. Boot-clear and all
subsequent evidence remain available for a future adapter; the converter does
not silently discard the early polls. Unfinished terminal dispatches likewise
remain records, never inferred completions.

The C host includes the existing headless support with its main renamed and
adds read-only linker observations at controller calls and before C NMI.
It verifies the frozen source archive and extracted files, runs RM_MINE from
one ordinary reset with zero SRAM, and executes all derived inputs in one fresh
process. Full completed WRAM, terminal WRAM/SRAM, ROM view, event records and
fallback effects remain in the evidence. Original completed dispatches are
paired by ordinal with C completed dispatches for diagnostic comparison only.
Game state, room, position, health, inventory, clock, RNG and controller words
are compared. Accepted/all NMI counters and the wait byte remain raw but are
excluded from this semantic diagnostic because the scheduler phases differ;
no equality claim or normalization is made for them. Upload state is compared.

A separate changed-gameplay process runs the same fixed timeline while a linker
wrapper changes the actual `NextRandom` result by one bit. Its completed
semantic states are compared directly to the unchanged C process. This control
cannot succeed merely because the original map already failed: it must show an
additional gameplay discrepancy between the two C subjects. No commercial ROM
or upstream source file is modified. C# replay design informed which boundaries
to observe; no C# implementation was copied.

## Retained result

The 320-frame capture at `out/decomp/ticket07/20261010T182149.168780Z` completed
with 160 native input polls and 10 completed original dispatches. The fresh C
subject ran 160 derived host steps with 10 completed dispatches and zero observed
fallback entries/instructions. The first compared mismatch is source frame 272,
original event 1025, against C host step 151: original held/pressed/previous input
words are zero, while the C words are $0080. Both subjects have game state 1 and
RNG 758 there. The changed-gameplay control changes that C RNG to 759 at the same
host step. These are diagnostic findings; original-ROM parity, adapted movie
completion and later gameplay acceptance remain unproven.

Public command tests cover missing-input error evidence and the complete native
capture/map/C replay/control path. ROM-derived artifacts stay ignored.

## Controller causality audit

`controller-causality.json` retains polls and dispatches in native event order.
Boot clearing ends the preceding-poll association; this is a diagnostic ledger,
not a claim of complete input ownership across every memory-clearing path.
The verdict's `future_poll_assignments` identifies where the existing poll-as-
host-step conversion feeds a sample observed after the original completion
being compared. In the bounded capture, completion1025 atframe272 precedes
poll1030, but C firstcompletion uses that poll. This explains the0080 mismatch
as future input attribution rather than proof of differing input arithmetic.

A sound adapter still requires callable boot NMI/poll and reset-clear boundaries.
C omits the original NMI controller call; its first poll happens after boot
clearing inside the first resumed update. Feedingzero at that call might match
one checkpoint but would leave150nativeprelude polls unexecuted. The diagnostic
command therefore preserves every poll and remains nonaccepting.

## Callable controller primitive

The command also verifies a standalone controller polling seam. The original
observer passively captures full WRAM at each controller entry and return,
including frame19 before main dispatch. An independent offline C subject loads
that entry fixture, presents the actual recorded $4218/$4219 read pair through
the SNES input-register backing, and invokes the real linked `ReadJoypadInputs`
without advancing boot, gameplay or NMI. These fixture loads occur only in the
routine differential subject; the movie candidate still starts from ordinary
reset and receives no state injection. Captured second-controller reads are
required to be complete and zero when present; nonzero values reject this
bounded implementation rather than receiving invented samples.

All131072 WRAM bytes are compared and raw differences are retained. Native
execution may change one CPU-stack return-address byte from$94 to$16; the
per-case fixture identifies its exact offset only when it is the sole changed
stack byte and has that known transition. C must preserve$94 at that byte to
qualify for the architecture exclusion. Every other byte, including other stack
bytes, must match. A poll with no native stack change excludes nothing. The
verdict distinguishes `full_wram_match` from `gameplay_wram_match` and records
per-case raw stack differences. Sealed poll cases, binary, observer and native
entry/return snapshots bind this narrower claim.

Separate controls change the presented hardware A-button bit and corrupt the
room byte after the real primitive; both must fail the routine comparison.
This proves a callable polling foundation, including the prelude, but does not
supply boot/reset clearing or controller-to-dispatch scheduling. The overall
original-ROM prefix therefore remains nonaccepting, with its future-poll
assignment diagnosis intact.

## Bounded causal NMI polling execution

A separate `causal-polling` candidate restores the real controller primitive
after each C NMI and suppresses the poll relocated into C main-dispatch entry.
The source-order150prelude samples execute during the actual logo/reset
coroutine's150yielding NMI calls. `Vector_RESET_Async` then performs its real
64KiB clearing before gameplay begins. No snapshots are loaded into this
candidate and no coroutine aliases are changed. The first completed gameplay
boundary is observed before the next sample, sourceevent1030, is presented.
Every preceding poll's held word must match the native sequence in order;
matching counts alone do not pass.

The first completed gameplay dispatch at original frame272 matches all compared
semantic fields, including game_state1, RNG758 and held/pressed/previous0. A
separate changed-RNG process must introduce an additional discrepancy. This
removes the diagnosed future-poll assignment for that bounded completion by
executing the omitted NMI polling at its semantic seam. It proves neither the
original boot video-frame timing nor subsequent asynchronous wait scheduling;
the general diagnostic replay and overall verdict remain nonaccepting.

The causal subject now executes all160captured polls and compares all10completed
updates in the320frame capture, through original frame316. Each completion
requires its entire preceding poll sequence and count to equal the native
preceding sequence; repeated held values cannot hide a shifted poll ordinal.
All compared semantic fields match. The changed-RNG control is compared across
all10updates independently. Native explicit-wait events remain in the ledger
and the verdict explicitly leaves their timing unmapped. These source-ordered
completed-update comparisons do not prove full WRAM or hardware-frame parity.

## Extended APU-wait investigation

`--frames 640` retains extended comparisons as nonaccepting diagnostic evidence,
including completion-count mismatch and the first poll-provenance failure.
`matching_causal_prefix` stops at the earlier of the semantic or source-poll
failure; equal fields after a poll failure do not extend that claim. The strict
320frame contract remains checked separately.

Passive upload entry/return hooks and actual $2140..$2143 reads show that boot
upload occupies original frames168..254 and music upload316..355. Neither
interval contains an NMI or controller poll: these are hardware-only SPC
handshake waits. C's cross-unit `RtlApuUpload` wrapper observes real instantaneous
player uploads. Inserting gameplay or NMI iterations to mimic those elapsed
frames would change the original schedule. The host now uses its existing
soft-reset preparation before every replay frame, preventing the known sentinel
from reaching the dispatcher during longer diagnostic replay.

The640frame trace exposes the next incomplete controller boundary rather than
accepting later ordinal matches. The standalone controller arithmetic fixture
remains restricted to calls captured through frame320; a later native stack
return-address transition belongs to another callsite and remains unverified.
