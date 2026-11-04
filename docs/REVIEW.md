# Milestones and decisions

Jim's decisions and the milestones they set. [PLAN](PLAN.md) holds the standing rules,
and [#177](https://github.com/JimLundin/sm-decomp/issues/177) holds live status. Git
history keeps everything retired from here; the
[last full edition](https://github.com/JimLundin/sm-decomp/blob/e281ce9153a8bf35a48c5d7aca2a48533df49cac/docs/REVIEW.md) and the
[frozen progress report](https://github.com/JimLundin/sm-decomp/blob/e281ce9153a8bf35a48c5d7aca2a48533df49cac/docs/PROGRESS_REPORT.md) are starting points.

### Direction (Jim, 2026-10-29)

We are building the Super Metroid engine: a purpose-built, compact Rust engine
that plays the original game faithfully, with modularity wherever it makes
sense: room/door graphs, textures, tilesets, palettes, sprites, sounds and
gameplay options replaceable through `sm_data` and explicit profiles. Map Rando
is the first use case and the acceptance bar: success means enough modularity
to support it. Further modularity is welcome where it keeps the engine clear;
it is still a Super Metroid engine, not a general-purpose one.
Judge every change by whether it moves the code toward that engine and makes
the code base smaller and clearer.

The external review runs at 08:34 and 16:34 Stockholm time.
Visible progress on main is expected between any two reviews; a review with no
new commits, or none serving the current phase, is treated as a stall and
investigated. Report blockers to Jim in the conversation before the next
review rather than waiting.

**Progress means behaviour crossing into a target package** (`sm_data`,
`sm_game`, `sm_runtime`, `sm_app`, `sm_tools`) that the compiler keeps separate
from the emulator. Code that exists only to keep the original runtime and oracle
working is **compat**: it shrinks and gets no new features. Owners built inside
the RAM-compatible runtime stay inside the backend cycle and are not the goal.

### External review work list (#254) decisions (Jim, 2026-11-04)

Jim accepted the coordinator's recommendations on the review's seven open decisions,
on condition that progress is reported in concrete measures (the dashboard in #177).

1. **Shell (#248).** The target is winit, wgpu and cpal. Gate 1 runs on whatever shell
   exists, and the shell swap is a separate lane.
2. **Gate 1 first (#248).** The Morph room slice with no `sm` dependency comes before
   #231's size lanes.
3. **Retiring the whole-RAM comparison (#246).** The nightly whole-RAM lockstep stays
   as a report until #249's any%, 100% and low% movies pass and the nightly report of
   unowned-byte differences has been quiet for two weeks.
4. **Independent reference (#250).** Mesen is the second reference, with spot checks on
   real hardware for disputed behaviour. The unpatched real game stays the authority.
5. **Goals and process (#252).** Only text Jim has approved, landed on main, changes
   goals or process.
6. **Game data (#253).** Values the ROM provides are read from the ROM at load time.
   Source holds structure (types and tables of meaning) and behaviour.
7. **Size floor (#231).** There is no fixed floor. The target for this phase is about
   125k engine lines, revisited after #251's real types.

**Order:** finish the in-flight decoupling (Session compare, then deleting `foreign`
and the RAM export). In parallel, fix the safety net (#239, #240) and put in place the
advisory measures and nightly reports (#245, #246). Then Gate 1 (#248), then the size
work and #251. Landing is one PR at a time through `tools/lane/land.sh` (#240).

### End state: the engine alone, against the emulator oracle (Jim, 2026-11-04)

Jim confirmed the target. Only the new engine runs the game, and the emulator is
used solely as the oracle. Once `sm_game_ram_sites` reached 0, the work runs in
this order:

1. **Compare in Session space.** Verification decodes the oracle emulator's RAM
   into a Session and compares it with the engine's Session (D6). The engine then
   no longer needs to export RAM for comparison.
2. **The runtime owns the tick.** One lease per tick, with the NMI inside the
   tick. After that, `foreign`, the leases and the Session ↔ RAM export are
   deleted, and `Game` implements no `sm_game` host trait.
3. **The native renderer is the replay output (D9).** The emulated PPU remains
   only on the oracle side.
4. **The old `sm/` game code is deleted.** What stays is the emulator, used as the
   oracle, plus the RAM-to-Session translation in verification.

Strict ratchet measures for this phase:

- `game_host_impls` must reach 0.
- `session_entries` (leases opened from the engine) must reach 0.
- The RAM export/decode lines in the engine must reach 0.

`sm_game_ram_sites` and `foreign_calls` stay strict as well.

### Session-authoritative tick decisions (Jim, 2026-11-04)

These answer the decisions D6, D7 and D9 in
[the tick handoff](https://github.com/JimLundin/sm-decomp/blob/e281ce9153a8bf35a48c5d7aca2a48533df49cac/docs/handoff/session-authoritative-tick.md).

- **Goal.** The old `sm/` game is the first attempt, and the aim is to remove it.
  Verification may still use pieces of it for now, but it must not shape or hold
  back the new engine (`sm_game`, `sm_runtime`).
- **D9: the owned renderer is the authority.** Our own renderer, not the emulated
  PPU, is the authority for the new engine. Register writes leave `sm_game` as typed
  display commands. If a RAM or register compatibility shim is still needed, it
  lives outside the engine.
- **D6: RAM translation belongs to verification.** RAM compatibility shims such as
  `CompatibilityRam` and the RAM ↔ Session translation may live in `sm/`, preferably
  under verification. Verification is responsible for translating RAM into the
  Session format for comparison. Moving such a shim out of `sm_game` into
  verification counts as retiring it from the engine.
- **D7: verification foreign calls go with `foreign`.** The verification-only
  `foreign` calls are retired together when `foreign` itself is removed. They are
  not ported one by one.

### After the decoupling: as few lines as possible (Jim, 2026-11-04)

The plan is issue #231. Jim's ruling:

- **Get the engine as small as it can go.** The aim is no bloat, not a fixed
  number. No floor is accepted in advance: neither 100k nor 125k is the stopping
  point.
- **Architecture first.** Lines come off by better structure, not by hacks.
  Joining lines, cramming expressions, or macros that turn code into an
  unreadable generator don't count, and the Rust style rules in AGENTS.md still
  apply.
- **Data-driven structure is valid and preferred.** Moving behaviour into data
  (tables, state machines described as data, importer-resolved hooks) is a valid
  way to cut code, and that includes redesigning the bosses as data-driven state
  machines. The data has to be typed and stored in `.rs` or `.py` modules, not
  JSON. Moving import code into `sm_tools` counts only when it replaces engine
  logic with data; moving code between packages unchanged doesn't count.
- **Measure:** the [code volume](VERIFY_PIPELINE.md#code-volume) counts logic
  and data tokens and structure (traits, blanket impls), which joining lines
  doesn't move; it is advisory beside `production_lines`.

### Gate 1 slice

Morph room 9E9F: `Session::open` from prepared content, the full Morph1200 inputs, the
`sm_runtime` renderer and sequencer, a minimal `sm_app` and headless replay in `sm_tools`,
compared to the oracle in free run. *Exit:* [VALIDATION](target-architecture/VALIDATION.md)
Gate 1 evidence. Then Gates 2 to 5, and Map Rando import through the same `sm_data` IDs.

Accepted interface decisions for the slice:

1. Tools construct the complete sealed validated start; `sm_data` owns the immutable
   envelope and IDs. Session never runs CPU normalization, opens a RAM dump or guesses fields.
2. The initial observation is Tick 0; input index i commits Tick(i+1).
3. `sm_game` checkpoints semantic audio intent and delays; the mixer, voices and echo stay
   outside. Opening never resends an accepted fanfare batch.
4. The portable, default and headless build edges exclude the old game, the backend, native
   code, reference code and devices.

### Ratchets

`tools/ram_ratchet.py` holds them, and CI fails on a rise. A deliberate rise raises its
baseline by hand in the same PR, and the PR says why. Macros and APIs that name SNES RAM
addresses count as RAM access even when typed Session storage answers them (Jim,
2026-11-02). Each landing's PR description states its ratchet deltas. A landing carries at
most about 25 commits; when its gate fails, land the green prefix.

**Independent pixels.** Both sides of a lockstep frame draw through the shared emulated
PPU, so their agreement alone doesn't establish fidelity. See #250.

SPC700 CPU and APU bus emulation are frozen compat, and unsafe is allowed there.

### Scenario harness, lane S (Jim, 2026-10-31)

Jim approved the scenario harness in #6 on 2026-10-31; the design is
`docs/SCENARIO_HARNESS.md`, landing with lane S's first commit. It replaces
PLAN's former rule against new process frameworks. It is the comparison Phase 3
needs ("compared to the oracle through semantic projections") and the projection
path that VALIDATION's *Verification strategy* already describes. Lane S runs
beside M and B under the landing rule and ratchets (1) to (5).

**Jim's decisions.** (a) No tolerances: every divergence is fixed or listed in
`docs/LEGACY.md`. For this harness that overrides "documented tolerances" in
VALIDATION Gate 2. (b) The original step-1 lane decision and its retained
status are recorded in the [harness snapshot](https://github.com/JimLundin/sm-decomp/blob/e281ce9153a8bf35a48c5d7aca2a48533df49cac/docs/PROGRESS_REPORT.md#harness-snapshot).
Current harness work supports migration under PLAN's work order.

**Review conditions (reviewer, 2026-10-31).**

1. **It is verification, and it pays for itself.** Harness code counts as
   verification, reported separately and outside the 100,000-line target. Step 1
   adds at most ~2,000 lines. Every later landing in which a projection takes over
   a domain deletes what it supersedes in the same change: that domain's
   `normalize_snapshot_ram` ranges and `LEGACY(verifier)` rows, and the checks it
   replaces. By Phase 3's exit, verification is no larger than at `e0b47fe`
   (57,802 lines).
2. **One runner.** The harness grows inside the current verifier and then
   replaces it; it never runs beside it as a second framework. Whole-RAM
   comparison stays authoritative for a domain until a projection covers it.
3. **Placement.** The scenario model, observation types, comparison and report
   live in a new `crates/sm_tools` (forbids unsafe; depends only on `sm_data`,
   `sm_game` and `sm_runtime`; the CI guard rejects any target package depending
   on it, and it depending on `sm`). The oracle projection decodes RAM through
   `ram_map.rs` and stays in `sm` as oracle tooling.
4. **The engine side reads typed state only.** A domain is projected from the
   engine only where the Session owns it as typed fields, never through
   `SessionRam`, `RawRam` or `CompatibilityRam`. Untyped domains stay RAM-compared,
   and each report lists coverage per domain. Projection coverage therefore
   follows Phase 2(c); it does not replace it.
5. **Data and secrecy.** Committed scenario files hold IDs, start references by
   digest, facts and domains. Input streams, seeds and checkpoints stay in the
   encrypted ci-data assets (PLAN: never commit movies or derived data), and
   `scenario cut`/`lock` update them through that route. CI logs and committed
   files carry tick, domain, field names and hashes, never ROM-derived values.
6. **Order, each closing on retained evidence in repository reports and owned receipts.**
   Step 1: typed observations and both projections next to the unchanged RAM
   comparison; structured divergence report; no new `sm_game_ram` sites.
   *Exit:* chain rooms 1, 6, 100, 200 and 320 pass, and a one-field mutant is
   reported at the right tick, domain and field.
   Step 2: free-run and determinism modes, and the oracle digest lock
   (condition 8), measured on those five rooms.
   Step 3: free run against the lock becomes the frequent gate (condition 8's
   switch), plus the landing replay job. Step 4: scenarios as data with
   `cut`/`lock`; PLAN's regression rows become scenarios. Step 5 onward:
   retire whole-RAM comparison domain by domain, as in condition 1.
7. **Nothing else waits.** The Cue-trace answer (ratchet 5) is still due before
   the next landing; the harness's Cue domain replaces that one-off run later.
   M does not wait for lane S, and lane S does not outrank M.
8. **Oracle digest lock (Jim, 2026-10-31 evening).** The oracle plays each
   scenario once, start to end, and the result is committed as a few digests;
   CI then runs only the engine, in free run, and compares.
   - *What is committed.* Per scenario and oracle mode: the tick count, one
     running digest per observation domain, folded over every tick (WRAM, SRAM,
     VRAM, CGRAM, OAM, PPU registers, video, APU ports, and each typed domain as
     its projection lands, with the `normalize_snapshot_ram` ranges masked), and
     the combined running digest every 1,000 ticks. That is a few hundred bytes
     per scenario. Running digests, not end state: a one-frame video glitch, a
     wrong sound or a transient value never shows in the final RAM.
   - *What is not.* Start and end machine states are ROM-derived; they stay in
     the encrypted ci-data assets (PLAN: never commit derived game data). The
     digests reveal nothing.
   - *On a mismatch,* the differing domain and the last matching 1,000-tick mark
     are already known. CI reruns that span live in lockstep, which names the
     first tick and field, and confirms the failure.
   - *Oracle independence (Jim, 2026-10-31).* Engine work never changes the lock.
     (i) Lock against the **faithful** oracle, the real game. The patched oracle
     applies entries the engine reproduces; its output moves with engine work
     as native fixes flip entries. Read the [oracle snapshot](https://github.com/JimLundin/sm-decomp/blob/e281ce9153a8bf35a48c5d7aca2a48533df49cac/docs/PROGRESS_REPORT.md#oracle-and-trick-source-snapshot)
     for the retained inventory. A row
     whose engine still reproduces a gameplay-visible patch is listed in LEGACY
     and stays on live patched lockstep until that entry `follows`; the list only
     shrinks. (ii) Key each scenario on the oracle's start (seed digest, or boot
     plus input-prefix digest), never on a checkpoint file: checkpoints serialize
     engine State and are regenerated when its layout changes. (iii) *Decided
     (review, 2026-11-01): no standalone recorder now.* Extracting it means
     refactoring the oracle frame off `Game`, which is compat work
     ([retained inventory](https://github.com/JimLundin/sm-decomp/blob/e281ce9153a8bf35a48c5d7aca2a48533df49cac/docs/PROGRESS_REPORT.md#oracle-and-trick-source-snapshot)). Prove independence by evidence
     instead: the scheduled corpus re-records a rotating sample of at least 30
     locked scenarios on the faithful oracle from the current main and fails if
     any digest differs from the committed lock without a lock change in the
     same commit. The standalone recorder becomes due when Phase 2 has moved
     the oracle frame's remaining `Game` uses out of `sm`, or at the first
     unexplained lock drift.
     (iv) Digests are per domain, so adding a projection adds digests and
     retiring a mask refreshes only that domain.
   - *Maintenance.* `scenario lock` writes the file and records that identity
     plus the digest format version. Only a change to the oracle side, the
     scenario's inputs, or a deliberate verifier change (a mask or projection)
     updates the lock, in that same commit. CI fails on a stale lock rather than
     re-recording.
   - *Landing replay.* A commit costs one incremental build plus engine-only
     frames, so one job replays each commit of a landing (at most 25) in order
     and names the first red commit: ratchet (4)'s green prefix.
   - *Adopted (Jim, 2026-10-31).* Engine-only free run against the lock is the
     frequent verifier. The [adoption measurement](https://github.com/JimLundin/sm-decomp/blob/e281ce9153a8bf35a48c5d7aca2a48533df49cac/docs/PROGRESS_REPORT.md#harness-snapshot)
     retains the step-1 comparison. Jim replaced the former 2x adoption bar: a hash per
     scenario is deterministic and small, coverage grows by adding scenarios,
     and engine optimisation widens the gap. Running lockstep across everything
     on every change is too costly. Lockstep remains, to locate a fault and
     confirm a failure.
   - *Running the gate.* Check-lock runs on every PR and push over every locked
     scenario. A mismatch reruns its 1,000-tick span live in lockstep in the
     same job. A row that is not yet green against the faithful lock is listed
     in LEGACY and stays on live patched lockstep until fixed (8(i)). Free run
     is stricter than lockstep: masked scratch that later changes behaviour
     shows up. The scheduled corpus keeps one full live lockstep run until
     projections retire the RAM comparison.
9. **Scenario sources: tricks for Map Rando (Jim, 2026-10-31).** New scenario
   and trick expansion is deferred under PLAN's migration-first work order.
   The following design remains the destination after migration. Map Rando's
   point is the vanilla trick library, so scenarios should cover it. Its own
   patches are mostly QoL, so its few gameplay changes (suit damage, door
   timing) wait for a later fidelity profile. Not every strat gets a scenario.
   - *sm-json-data translation.* An `sm_tools` importer reads a pinned
     [sm-json-data](https://github.com/vg-json-data/sm-json-data) commit
     (CC-BY-4.0; credit in `THIRD_PARTY_NOTICES.md`; the data is not copied
     into the repo) and turns a chosen strat into a scenario skeleton: its
     stable ID (room, from node, to node, name), the entrance and exit
     conditions, and the equipment and end node as named facts. See the
     [source inventory](https://github.com/JimLundin/sm-decomp/blob/e281ce9153a8bf35a48c5d7aca2a48533df49cac/docs/PROGRESS_REPORT.md#oracle-and-trick-source-snapshot) for the
     retained upstream snapshot. Choose by tech frequency and
     by door conditions first (G-mode, shinecharged, runway, spark), because
     carrying state into rooms that were never neighbours is what Map Rando
     adds.
   - *Inputs.* Each scenario needs inputs: authored with
     `tools/authored_inputs.py`, or extracted from video (next item). A
     scenario is accepted only when the faithful oracle reaches its end node
     with its facts; then it is locked (condition 8).
   - *Video-to-movie trial (after step 4).* Map Rando's strat videos
     are CC0, captured as uncompressed AVI from accurate emulators with the
     Practice ROM's input display, and mapped to strats in a public database
     dump. Trial on 10 strats: read per-frame buttons from the input display;
     reconstruct the start (room, entry, equipment, position) as a documented
     constructed start; replay on the faithful oracle. *Success:* the oracle
     reaches the end node, and frames outside the HUD match the video. Report
     the hit rate and per-strat cost in its PR, then
     decide whether to scale it.
     Videos are fetched by pinned digest and never committed.
