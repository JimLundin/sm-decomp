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
