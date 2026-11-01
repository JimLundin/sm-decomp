# External review

Living independent review, commissioned by the user. This file holds current
direction; [frozen qualification history](https://github.com/JimLundin/sm-decomp/blob/170bc5351c64f0846bbcb7b4714064a7230db153/docs/REVIEW.md)
retains earlier results and contracts. Retired issue receipts remain in the
primary's owned archive, located by their original comment IDs in that history.

## Milestone plan (2026-10-29)

Approved by Jim on 2026-10-29; it supersedes the 2026-10-27 plan and its pacing
advice. PLAN remains authoritative for the Fast loop, Git, resources and commit
rules; the accepted architecture, ADRs and ENGINE_STATE_MIGRATION are unchanged.
Primary (Jim, 2026-10-29 evening): the collaborator session is the sole Git
writer for main; root releases main and hands in-flight work over as patches.
Linux stays stopped.

Read PLAN's 2026-11-01 work order and migration scope before assigning phase work.
PLAN owns current pacing and reporting; the compiler/fidelity gates and full
completion scope below remain required.

**Landing rule (Jim, 2026-10-29).** A change lands when the local compiler gate
(fmt; `rustc --emit=metadata` of `sm` with and without `sm_verify_dll`;
`player_detached`; source inventory), PR CI and a full corpus on its exact head
are green. The PR description carries the evidence; no separate qualification
records, timing tables or resource-ledger prose are required.

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

The [review baseline](PROGRESS_REPORT.md#review-baseline) records the measured
backend cycle, coupling and checkpoint invalidations that motivated these phases.

### Phases

Each phase closes on evidence, not a date. Lanes: A tooling/verification,
M seams and cycle cuts, B bank retirement.

0. **Verification loop (A; prerequisite for later phases).** Fingerprint the
   Checkpoint's serialised layout (`State` field names and types) instead of
   source text; keep the round-trip/extra-bytes/input-prefix rejections. Refresh
   the `f158` seed asset; split the checkpoint shard to at most ~60 minutes. Run
   `tools/cycle_report.py` in CI. *Exit:* two consecutive green full-corpus runs
   on main, scheduled or dispatched (Jim, 2026-10-29: the schedule fired only
   every 4–9 h, so dispatched runs count). The retained exit evidence is in
   [Phase 0 qualification](PROGRESS_REPORT.md#phase-0-qualification).
1. **Package graph and seams (M).** Add `sm_data`, `sm_game` and `sm_runtime`
   to the workspace; `sm` becomes compat plus oracle tools. Put the CONTRACTS
   types in place: `TickOutput`, a coarse `Scene`, a `Cue` record in `sm_data`.
   Route `game/`'s audio backend calls (`write_sound_port`, `upload_bank`)
   through Cues; a compat adapter maps each Cue back to `ApuQueue` port bytes so
   oracle input is unchanged. Move the cycle-free files from the
   [review baseline](PROGRESS_REPORT.md#review-baseline) into `sm_game`; CI
   fails if `sm_data` or `sm_game` depends on compat. *Exit:* both build headless
   without compat, the corpus is green, and the Cue trace reproduces the port
   trace on every corpus row.
2. **Cut the backend cycle (M and B, rolling).** Cheapest coupling first:
   (a) ROM reads become `sm_data` tables built by an `sm_tools` importer, checked
   byte-equal to the ROM at load and keyed by stable room/door IDs, which is what
   Map Rando's room graph needs; (b) PPU register writes pass through one
   display-state type feeding `sm_game::extraction` and the Scene; (c) RAM state
   moves to typed domain modules in CONTRACTS order (progress, rooms, player,
   actors) under the existing interval rules; (d) B resumes sm_8b, sm_a3,
   sm_a5..sm_a9 and sm_aa, retiring straight into `sm_game` domain modules.
   *Exit per domain:* it compiles in `sm_game` and its corpus rows pass.

   **Phase 2 ratchets (review, 2026-10-30; flagged in two consecutive reviews).**
   The [ratchet review snapshot](PROGRESS_REPORT.md#ratchet-review-snapshot)
   records the increases that prompted these rules.
   From now: (1) no landing may raise `sm_game_ram` total sites or coupled
   files; a move that brings RAM-addressed code must convert at least as many
   sites to typed `Session` fields in the same landing; make `cycle_report.py`
   fail CI on an increase over a checked-in baseline. (2) A move is line-neutral
   or smaller in production lines; host/lease adapters added to make it compile
   count against it. (3) Each landing's PR description states both deltas.

   **Landing size and reporting (review, 2026-10-31; flagged in three reviews).**
   (4) A landing PR carries at most ~25 commits; when its CI or corpus fails,
   land the green prefix rather than holding everything. The original
   [landing stall](PROGRESS_REPORT.md#landing-stall-snapshot) remains recorded. (5) The Phase 1 exit item, the Cue trace matching
   the port trace on every corpus row, is still open (`sm/src/verify.rs:1184`
   requires the physical-audio oracle). Before the next landing, either run it or
   state the blocker and next action to Jim in the conversation. Keep measured
   cycle and ratchet figures in the dated PROGRESS_REPORT.
3. **Gate 1 vertical slice.** Morph room 9E9F: `Session::open` from prepared
   content, the full Morph1200 inputs, `sm_runtime` renderer and sequencer, a
   minimal `sm_app`, headless replay in `sm_tools`, compared to the oracle
   through the scenario harness's projections (lane S, below) in free-run mode.
   *Exit:* VALIDATION Gate 1 evidence.
4. **Gates 2–5**, then Map Rando import through the same `sm_data` IDs.

SPC700 CPU and APU bus emulation are frozen compat (unsafe permitted, outside
the unsafe goal). The N-SPC sequencer (`spc_player.rs`) and S-DSP are the seed
of `sm_runtime` audio; continue them only once separated from emulator types,
with verification growth no larger than the production change it covers.

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
status are recorded in the [harness snapshot](PROGRESS_REPORT.md#harness-snapshot).
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
     as native fixes flip entries. Read the [oracle snapshot](PROGRESS_REPORT.md#oracle-and-trick-source-snapshot)
     for the retained inventory. A row
     whose engine still reproduces a gameplay-visible patch is listed in LEGACY
     and stays on live patched lockstep until that entry `follows`; the list only
     shrinks. (ii) Key each scenario on the oracle's start (seed digest, or boot
     plus input-prefix digest), never on a checkpoint file: checkpoints serialize
     engine State and are regenerated when its layout changes. (iii) *Decided
     (review, 2026-11-01): no standalone recorder now.* Extracting it means
     refactoring the oracle frame off `Game`, which is compat work
     ([retained inventory](PROGRESS_REPORT.md#oracle-and-trick-source-snapshot)). Prove independence by evidence
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
     frequent verifier. The [adoption measurement](PROGRESS_REPORT.md#harness-snapshot)
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
     [source inventory](PROGRESS_REPORT.md#oracle-and-trick-source-snapshot) for the
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
     the hit rate and per-strat cost in a dated PROGRESS_REPORT entry, then
     decide whether to scale it.
     Videos are fetched by pinned digest and never committed.

### Retained publication and acceptance

Read the dated [Phase 0 qualification](PROGRESS_REPORT.md#phase-0-qualification)
and [cohort proofs](PROGRESS_REPORT.md#retained-cohort-proofs) before changing an
accepted owner. Frozen reports retain full hashes, native/cache/compiler/profile/
source/codec/input/loader pins, donor/caller costs, exact deltas and failed or
unexecuted variants. Consult the relevant original owned receipt.

Catalog/schema/validation/adapters and verification growth all count; moves and
formatter joins earn no savings. Windows selected ROM/search paths retain ASCII
policy; pack/verifier paths retain Rust handling. Absolute ASCII ROM paths work
independently of cwd; --config skips relative-directory search; no-room-pack flow
adds no guard. Native dependencies stay disclosed until actual consumers retire.

### Shared renderer contracts

Preserve these interfaces and source contracts:

- BG wraps its base before column/plane additions. Reads reach801e; retain 31
  native tail words, port/padding/cgram aliases. No final-address mask or zeros.
- Sprite evaluation writes directly through a narrow mutable 256-word native
  workspace. Completed pixels survive explicit errors and Rust unwinding.
  Success publishes sticky overflow before lineHasSprites; failure keeps prior
  flags/lineHasSprites. No production panic wrapper or whole-Ppu borrow.
- Keep brightness/backdrop/forcedblank/line0 mutation order. C acquires gated
  signed sprite aliases from surrounding native storage. Per-pixel borrows end
  before the original four-byte native commit. Preserve failures and aliases.
- Pure color begin/needs-subscreen/finish stages retain native/reference policy,
  including pseudo-hires/modes5/6 subpixels without math. EXTBG/mosaic fallback,
  register/latch/snapshot/reference/device dependencies remain disclosed.
- Keep whole ppu.rs in checkpoint contributors and add all three rendering
  leaves, plus actual library/literal registrations, through the sole integrator.
  New codec requires genuine initialized/required starts/continuation and complete
  source/native/ROM/input guards. Never restamp 05 checkpoints.

The safe core and actual prepared consumer must compile without oldsm/SNES/native/
device/ROM dependencies. The direct metadata entry reaches only the three frozen
leaves and std/core; whole sm --no-default-features is not that proof. Retain the
original Linux detached proof; no new Linux job. Source authors remain source-only.
Release original handles before changed-entry reuse.

**Independent pixels are required.** Oracle and mine movie branches both use the
shared renderer. Their pixel/RAM agreement alone cannot establish fidelity.
The existing verifier now supplies independent Mode1/Mode7 pixels, priority/masks/
color/scanline/affine, alias/failure and actual EXTBG/mosaic boundary witnesses.
Broader variants and checked-overflow mutation sensitivity remain scoped/unproven.
PLAN affected rows/four smokes/full Morph1200 retain their original proofs.

### Message composition contracts

Reuse the unchanged dialogue runner and frozen V03 renderer
([retained slice evidence](PROGRESS_REPORT.md#owned-slice-snapshot)). One pure
message-composition implementation serves the native writer and an actual prepared
display consumer. Remove replaced borders/text/button/selection/radius/scanline
bodies in the same cohort. Keep lazy draw/drawing/fresh modify reads, scratch/text/
DAS/word/bound failure order, do-until zero/odd/wrapped behavior, letter priority,
HDMA repeat/count and palette/saved-display restore order. Preserve live request
clearing, parent/child suspension, input filtering, reset/load and checkpoints.

Prepared starts require complete validated saved display, message/font/media IDs
and revisions; missing data rejects. Do not infer a message or initial display
from 9E9F. Native writable RAM and separately owned prepared display do not become
competing authorities. display_refresh::wait's0782 request/acknowledgment structure
is retained; no new clock, loop-to-completion or invented 60Hz policy. Audio/device
delivery and full Session/Scene extraction remain explicit dependencies.

### Full owned playable slice remains open

The four accepted interface decisions below refine the full Morph9E9F proposal.
The retained production estimate is not blanket authorization
([snapshot](PROGRESS_REPORT.md#owned-slice-snapshot)).
Preserve all eight full-slice proof groups and original full 1,200-frame Morph.

1. Tools construct G's complete sealed validated start; D owns immutable envelope/
   IDs. Session never runs CPU normalization, opens a RAM dump or guesses fields.
2. Initial observation is Tick 0; input index i commits Tick(i+1). Preserve actual
   normal/dialogue suspension, source sampling and late NMI/HDMA/IRQ settlement.
3. G checkpoints semantic audio intent/delays; A mixer/voices/echo and C epoch/
   acceptance/outbox stay outside. Opening never resends an accepted fanfare batch.
4. Actual portable/default/headless edges exclude oldsm/backend/native/reference/
   devices. Reuse actual pure leaves once; package moves alone do not establish it.

The [retained seed/trace facts](https://github.com/JimLundin/sm-decomp/blob/170bc5351c64f0846bbcb7b4714064a7230db153/docs/REVIEW.md#full-owned-playable-slice-20-remains-open)
use normalized WRAM06b4d253, not unavailable raw89960823. Room 9E9F / state 9EB1,
equipment 0 and next input 18109 are observed. The original full 1,200 trace stays
in 9E9F; award 590 / completion 977 retain source eba0 / DLL a1c0 / codec 3ac provenance.
These facts do not establish complete initial Scene/audio or a committed Scenario.
Exact cadence/reference policy and G's complete field schema still need evidence.
Use the completed original reader; do not start duplicate artifact surveys.

Scene extraction still needs immediate atmospheric/OAM/Samus aliases, NMI544-byte
display commitment and later effects. Terrain/observation/gamma surveys remain
scoped constraints; no standalone cache, dispatch facade or getter campaign is
released. ENGINE_STATE_MIGRATION and retained qualified history hold their actual
foreign-transfer/reset/adoption/failure contracts.

### Pace, readable handoffs and publication

Keep useful Windows owners supplied with bounded core work and a concrete ready
follow-up: exact owner/files/source/budget/rows/handles and next recipient. Prepare
successor commands and pinned inputs before upstream READY; collect/release the
prior family, sample shared resources and immediately launch pregranted successors.
Wake completed workers with followup_task; send_message does not activate them.
Preserve live jobs/frozen sources and send minimal actual results before prose.
Drain qualified code to main at the next safe boundary; documentation ACKs and
CI observation never gate publication or independent implementation.

Follow PLAN's current reporting rules. Mark pending checks honestly and keep
exact proof in owned records. Direct handoffs continue during CI; historical
issue receipts remain evidence provenance.

Measure each cycle's actual source/review/build/capture/replay/publication time,
queue/handoff gaps and net deltas. Preserve original handles and exact attribution;
cached refs/mailbox observations do not prove push time. Correct measured stalls;
do not repeat accepted campaigns for timing or claim unmeasured speedups.
Pages, progress images, treemaps and the codebase map are retired. Keep commit
messages concise under PLAN; retain exact metrics/proofs separately. Docs updates
travel with code. Current qualification requires compiler warnings as errors on
every real Rust path; historical warning-bearing results are not strict-lint proof.
Preserve other workers' source/caches/locks and all fidelity gates.

### Size, metrics and final acceptance

The engine should end compact: the 100,000-line goal applies to the target
packages (`sm_data`, `sm_game`, `sm_runtime`, `sm_app`) at completion. Compat
and verification are reported separately and do not count against it. Moving
code into a package is neutral, not growth; moves, formatting and relocation to
verification are still not savings, and every adapter counts once.

The dated PROGRESS_REPORT carries these measures and each observation's source
and qualification scope. Its historical
[baseline](PROGRESS_REPORT.md#daily-report-baseline) is retained separately:

| Measure | Direction |
| --- | --- |
| Lines in target packages | up |
| Lines outside the backend cycle (`tools/cycle_report.py`) | up |
| Legacy bank lines | 0 |
| Compat lines | down |
| Unsafe fns/blocks/impls in `game/` (excluding compat) | 0 |
| Scheduled full corpus | green |
| Validation gates passed | Gate 1 at phase 3 |

Full owned Session, independent renderer/audio, boot-to-credits fidelity and Map
Rando remain open; the accepted architecture and VALIDATION gates define them.
