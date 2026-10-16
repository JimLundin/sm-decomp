# Advisory: first playable Parlor arrival

Status: research complete; released to the parent integrator for scope selection.
Ownership: this Markdown packet only. No implementation, runs, builds, tests,
captures, authentication, other-agent monitoring, or shared-index changes.

## One recommendation

Continue the exact native Ceres input prefix through Landing Site, then author
one bounded ordinary-input tail to **Parlor and Alcatraz, area 0 room `0x92FD`**.
Finish after normal gameplay resumes there and deliberate input visibly moves
Samus within the room. This adds one westward room crossing along early vanilla
progression. It does not require a save/load lifecycle or an item milestone.

The room name is corroborated by the locally retained community source
[`RoomHeaderPointers.cs:6–10`](/home/jim/sm-decomp/out/research/community/SuperMetroidDecomp/csharp/src/SuperMetroid.Core/Rooms/RoomHeaderPointers.cs:6).
That source supplies the conventional name only; route observations below own
the behavioral evidence.

## Retained route and evidence limits

- The reusable native prefix is
  [`restart/out/runs/ceres-06/inputs.txt`](/home/jim/sm-decomp-restart/restart/out/runs/ceres-06/inputs.txt),
  exactly 16,991 host-step pads. Its recorded replay matched observations, WRAM,
  inputs and final PNG; see
  [`04-runnable-baseline.md:69–77`](/home/jim/sm-decomp-restart/.scratch/restart-pipeline/issues/04-runnable-baseline.md:69).
  The last observation is state 8, area 0, room `91F8`, x=1152, y=263, health=99,
  at step 16,991, with 16,068 polls and updates:
  [`observations.csv:16992`](/home/jim/sm-decomp-restart/restart/out/runs/ceres-06/observations.csv:16992).
  This is the beginning of normal Landing Site gameplay; do not assume Samus
  has already reached the ground or that every subsequent requested pad is read.
- Mixed Rust gameplay already reuses that prefix and matches the retained C
  observations, final WRAM and PNG:
  [`FULL_CERES.md:3–13`](/home/jim/sm-decomp-restart/restart/FULL_CERES.md:3).
  These are sampled gameplay observations, not per-step full-state/effect parity.
- No post-Landing native pad route was identified in the inspected restart route
  docs and retained `restart/out/**/inputs.txt`/observation inventories.
  Main's long TAS recordings are available, but imported movie frames are not
  native host-step pads; synchronization is explicitly limited in
  [`TAS_MOVIES.md:78–101`](/home/jim/sm-decomp/docs/TAS_MOVIES.md:78).
- There is useful legacy route guidance: the retained original-core trace
  changes directly from `91F8` to `92FD` during state `000B` at
  [`full-any-lsnes.csv:14015–14016`](/home/jim/sm-decomp/out/sync/full-any-lsnes.csv:14015),
  then reaches state `0008` at frame 16,460 and moves x=`04D8 → 04D2 → 04CC` at
  [`:14040–14042`](/home/jim/sm-decomp/out/sync/full-any-lsnes.csv:14040).
  This confirms the immediate checkpoint and westward traversal. It does not
  authorize splicing its frame numbers into the native prefix. The capture lacks
  current required provenance and remains diagnostic evidence, as recorded in
  [`original-feasibility.md:80–83`](/home/jim/sm-decomp/.scratch/restart-collaboration/original-feasibility.md:80).

## Bounded authoring and exact proposed postcondition

Proposed scenario name: `parlor`. Preserve all first 16,991 pads byte-for-byte
and continue the same cold-booted `Game`. Permit at most 6,000 additional host
steps (22,991 total) and a 120-second headless deadline per bounded attempt.
These are advisory bounds for selection, not measurements of a completed tail.

Use ordinary directional, jump/run and beam inputs as needed to descend from
the ship, traverse west, open the door and enter Parlor. Adapt only the tail
using observed state/presentation. Record each supplied pad before stepping.
Freeze a successful tail and replay the complete recorded sequence from reset.
No WRAM injection, checkpoint loading or original-movie retiming is needed.

Pass only after all these conditions hold in order:

1. The retained prefix completes with the Ceres escape sequence and exact
   Landing Site state-8/area-0/room-`91F8` boundary observed.
2. The tail traverses the door into area 0, room `92FD`; reaching the room pointer
   during loading/state `000B` is insufficient.
3. State 8, area 0, room `92FD` and positive health persist for at least 32
   **completed updates** after arrival. During this interval polls advance and
   an ordinary horizontal input produces at least eight pixels of same-room
   horizontal displacement. Capture the input interval and before/after state.
4. Stop before a second room exit. EOF, window close, deadline, death, a native
   error, unexpected room progression or a step limit without the checkpoint is
   a non-pass. Retain the last completed observation and attempted pad.

The useful material result is a completed non-Ceres door transition followed by
working in-room control and presentation. Retain the exact input file, runtime
and native identities, observations, final WRAM/PNG and exercised Rust/refusal/C
and CPU-fallback counters. Compare recovered-C and mixed-Rust executions of the
same frozen pads under the existing contracts; report encountered differences
and remaining C ownership. Original-ROM parity and broad vanilla completion
remain separate claims. A checkpoint alone does not prove either.

## Existing tool boundary and concurrent implementation

The following CLI findings refer to **pre-Parlor runtime source at commit
`6b5cec032565386f06af226584df1936cc0150fc`**, not a review of the active successor:

- `restart/src/runtime_cli.rs:239–241` admits only `boot|ceres|none`;
  `:318–322` ends noninteractive `ceres` at area-0 arrival; `:337–339` treats
  `none` reaching its limit as success. Neither is the proposed Parlor predicate.
- `:278–281` reads the initial file pad, but `:341–358` then prioritizes interactive
  keyboard input over the file. Existing interactive mode therefore cannot
  consume the full Ceres prefix before handing input to the keyboard.
- `:533–538` rejects EOF and accepts only 12-bit native hexadecimal pads;
  `:366–377` records/flushes input before the next host step.
- `restart/src/route.rs:59–60` marks arrival using state 8 and area 0, without
  requiring `91F8`; `:64–80` and the subsequent room controls describe the
  existing Ceres-oriented policy.

Use a small policy in the existing Game loop to consume the prefix before the
bounded tail, with one explicit checkpoint predicate. The existing interfaces
already provide the needed operations:
[`Game::host_step`, `State` and read-only WRAM, `native-host/src/lib.rs:231–291`](/home/jim/sm-decomp-restart/restart/native-host/src/lib.rs:231),
plus [`Window::poll` and `Window::present`, `presentation/src/lib.rs:154–193`](/home/jim/sm-decomp-restart/restart/presentation/src/lib.rs:154).
No additional runtime framework is indicated.

During final citation checks, the integrator advanced restart HEAD to
`390db3217a78413f704d3aef3e190f40a2b3cee2`, whose CLI includes Parlor. This packet
does not review that implementation or duplicate its ownership. The parent owns
selection and reconciliation with the active scope; the proposed numeric bounds
above are advisory if the implementation has already adopted a contract.

## Cleanup and handoff

Only this packet was written. No verification runs or intermediates were created;
all read-only searches finished. Reclaimed bytes: 0. Filesystem available space
at handoff check: 103,788,433,408 bytes. Existing evidence and active work remain
untouched. Draft ownership is released to the parent integrator.
