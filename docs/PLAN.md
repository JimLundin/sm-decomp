# Plan

Goal: a compact Super Metroid game engine with descriptive gameplay routines,
playing from boot to credits. SNES emulation remains the current video/audio
backend and verification oracle; it is not the long-term engine architecture.
Map Rando support comes after vanilla.

Public progress: keep [PROGRESS.md](../PROGRESS.md) current and push it with
verified milestone commits or material status changes, as requested by the user
on 2026-10-23. Distinguish committed results from work still under validation.
The report is graphical: show module-level ownership, callers and individual
validation steps. Update affected node colors on every implementation commit;
outline nodes changed since the previous report. Keep prose and accounting in
linked technical documents, and distinguish checked drafts from committed work.
Use individual file nodes for the active responsibility, with separate nodes for
each verification step. Highlight touched files even when their status color
stays the same. Keep adopted folder maps expandable and the active work visible;
show failed checks in red. No percentage based on file or node counts.

This is the active plan. Read the current work order and validation strategy
below. For the step-by-step refactor process, task boundaries and parallel work,
use [the iteration guide](ITERATION.md). Consult reference documents only for
the responsibility being changed; historical experiments are not a work queue.

Git concurrency (user direction, 2026-10-19): agents must not manually
reserve or hold Git locks, including index.lock and branch ref locks.
Use Git's normal short-lived locks only for the actual Git write. Builds,
verification, reasoning, reviews and waits must run without a held lock.
Use isolated snapshots when needed; recheck HEAD and reconcile changes
before committing instead of preventing other agents from committing.
An agent holding a reservation must release its own reservation immediately,
preserving source changes and verification work. Do not delete another
process's lock; tell its owner to release it.

## Accepted target architecture (2026-10-22)

The destination is the [owned Rust architecture](target-architecture/README.md),
with small platform libraries for device access. Read its
[migration handoff](target-architecture/MIGRATION.md) before changing subsystem
boundaries, state ownership or backend interfaces. Windows workers may keep
this emulated backend and built-in oracle during staged replacement. Building
an independent game slice is a target-architecture completion condition, not a
prerequisite to current refactors.

Gameplay should use coordinated state machines with owned state, named inputs,
transitions and outcomes. Preserve observation and suspension points. Migrate
complete producer/consumer interactions before removing their RAM communication.
Checked RAM access is transitional; completed domain behavior should need no
Game, RAM, raw pointers or unsafe calls. Keep byte-compatible encoding for the
current comparisons. Never retain two authoritative mutable copies through a
RAM-authoritative legacy interval.

Read the relevant interaction in [engine design](ENGINE_DESIGN.md) and
[RAM ownership](RAM_OWNERSHIP.md) when changing its contract. For reset/load,
replay, inline CPU/physical writes or oracle rewind/adoption, read the relevant
section of [state migration](ENGINE_STATE_MIGRATION.md). Moving bank files into
new packages alone establishes neither ownership nor backend independence.

## Active direction (2026-10-22)

User correction (2026-10-23): sharing more individual rules while growing adapters
is not sufficient progress toward a clear engine structure. Finish the already
verified charge-palette change, then prioritize production consolidation of a
complete owned subsystem. The next proposal must name its public API and state
owner, migrate actual main callers, and identify the bank orchestration and
per-operation RAM adapters retired in the same adoption. Price the complete
adopted subsystem and deletions together, including the currently local candidate.
Do not substitute another isolated shared consumer, a package move, or adoption of
outer wiring alone. If a complete boundary cannot yet be adopted, identify its
specific blockers and the minimum work that closes them before further coding.
Preparation and palette together add 256 main-source lines; their algorithm
sharing and safety gains do not establish net simplification or owned-engine
adoption. The next milestone must address that structural gap explicitly.

- Preserve the complete game and exact defined behavior, quirks included. The
  oracle decides. `sm/src` is the source of truth; use `ram_map.rs` and the
  [bank logs](https://patrickjohnston.org/bank/) when meaning is unclear. There
  is no C source to regenerate gameplay from. Preserve byte-compatible `g_ram`,
  ROM formats, numeric dispatch, ordered effects and meaningful gameplay names.
- Reduce maintained Rust below **100,000 lines**, then reassess further reduction.
  **90,000** is a stretch planning checkpoint with no established remaining
  deletion budget. Prefer substantial repeated algorithms and complete
  responsibilities over further small spelling/clone-scan cohorts. Require a
  materially new scope before restarting a previously parked experiment.
- **Remove all unsafe calls and raw-memory operations** alongside reduction and
  content extraction. This includes ROM ownership, packed queues, callbacks,
  initialization, native interfaces and verification. Safe wrappers, macros or
  C adapters that hide unsafe operations do not complete this requirement.
  Enforce `forbid(unsafe_code)` in completed independent safe modules.
- Safety work may increase source size. The volume-only parking rules do not
  veto necessary safety work. Count and explain its complete cost. The shared
  `word!` interface cannot become a whole-RAM borrow while retained pointers
  remain live across calls: migrate offsets/handles, preserve live rereads and
  overlapping byte/word fields, and use checked access with scoped borrows.
- Design the complete replacement before splitting commits: setup, updates,
  transitions, exceptional paths, callers and retirement. A commit contains one
  reviewable idea and may touch many files. Small preparation is allowed with
  named consumers and an explicit deletion step. Redirect actual callers and
  remove retired bank facades; keep old entries only for real identity/caller
  contracts. Ordinary functions, enums or data should simplify the full behavior.
- User accounting clarification (2026-10-23): measure production engine code
  separately from verification harnesses and adjacent verification support.
  Exclude that verification code from the engine-reduction total, and keep it
  intentionally separate where practical. Report its count and the combined
  maintained total alongside production; do not disguise runtime adapters as
  verification. Existing unseparated historical totals remain historical.
- Price production helpers, tables, wrappers, adapters, callers, registration
  and maintained native source together. Report installers and verification
  support separately; disclose external dependencies separately too.
  Count overlaps once; independent draft savings are not additive.
  File moves, line packing, formatting compression and moving implementation
  outside `sm/src` are not reductions. Retain token/text checks against packing.
- A renderer replacement cannot retire backend code still used by gameplay or
  the oracle. Preserve shader/audio behavior and the oracle's hooks, state,
  patches and snapshots until actual consumers and equivalent checks migrate.

## Work order

**Coordinated iterations (2026-10-22): the user authorized ongoing worker
supervision, lifecycle/commit coordination and resource oversight.** The
beam launch, missile acceleration, extension creation, placement, initialization and playback are shared below;
remaining consumers and unsafe dependencies are next. User direction on
2026-10-23 expands useful parallelism: three paired lanes (six workers), each with
an implementer and independent caller/validation investigator, feed one serial
integration/compiler lane. Current lanes are animated tiles, complete enemy
projectiles, and a direct typed palette core as a Session seed. Use isolated
drafts and freeze interfaces before splitting more implementation; no concurrent
main edits or builds. Increase parallel source work when genuinely independent
work is ready, while preserving this machine's single heavy compiler limit.
Enemy-projectile setup and pre-action families now have four additional workers
on fixed private interfaces. Palette implementation/review is complete; animated
tiles and enemy projectiles continue through the single integration lane.

The Session target requires ordinary owned data structures, not a composition
of existing address-codec ports. New designs must put direct typed state in the
core, confine byte compatibility to explicit edges, and replace real production
paths. Do not add an always-live duplicate Session beside writable RAM. The
palette lane must demonstrate its core with backend modules unavailable while
preserving production partial writes and exclusive legacy transitions.
Build-pipeline preparation is complete; previous task queues and ownership
experiments are not automatically restarted. Preserve active caches and saved
references; measure build/test time and resource growth before changing the loop.

1. [x] **Test Samus domain alternatives.** Complete; the tested representations
   did not establish the proposed large volume payoff.
2. [x] **Compare the selected large subsystem replacements.** Complete for the
   recorded scopes; bounded discovery does not rule out new algorithm sharing.
3. [x] **Implement the strongest measured replacements and recount.** Landed.
   The earlier under-100,000 milestone is historical: edition 2024's explicit
   unsafe scopes put the current source above that threshold again.
4. [ ] **Retire bank ownership by gameplay responsibility.** The complete
   player-projectile and room-mechanism subsystems are adopted in production. Continue with a
   complete adjacent responsibility and a reviewed owner/caller/deletion proposal;
   do not restart the archived Session or resume isolated shared-consumer work.

   Enemy-projectile adoption is complete for the reviewed scope: full caller and
   bank retirement, owned native ROM,1382 comparisons/87 independent checks and
   18216 exercised frames pass (all seven probes, zero failures). Production
   grows3490lines; verification grows2160. Ownership remains interval-scoped;
   unsafe legacy effects and persistent state migration remain.

   Player-projectile typed core is adopted through36470e60: fifteen singleton
   layers retire and actual flare decisions move into ordinary owned state.
   Local release/frontend/format,106groups+56comparisons and11549frames with
   all12probes pass. GitHub run39998215217 passes independent composed execution,
   two targeted mutation checks, formatting and full build. The earlier failed
   fixture ignored the beam leading edge; corrected two-tick and original
   first-tick boundary checks both pass. Production-1937lines; verification
   Rust+1109lines; combined-828. Mutation Python/CI support is separate.

   Adopted: complete room-mechanism typed core composed with one shared player,
   inventory and terrain owner. Concrete interface v2 passed independent review
   and compilation; four isolated lanes implement shared ownership/runtime,
   creation/rules/contact, scheduler/messages/pickups and drawing/content.
   Separate workers cover actual shared-player consumers and mutation-sensitive
   suspension/drawing checks. Retire Fields/Port routing
   and migrate real callers in the same adoption. Existing interval owners are
   not persistent whole-game state. Shared declarations compile. Refreshed1020
   mechanism reference IDs from the verified player-build DLL:1017returning,
   two expected fatal and one expected nonreturn; no historical output differences.
   Extend suspension/shared-inventory/drawing checks and their discriminating
   mutations before adopting the next core. Case1020 is adopted verification:
   actual pre-action/message suspension preserves counter5507 and delay13 on
   completion; four adjacent cases retain exact outputs.
   Adopted through e216f0ef: cases1021..1032 for
   continuation timing, native provenance/rebinding, overlapping tile reads,
   full emission, transfer layouts and wide/live streams. Nineteen local cases
   match frozen outputs; release verifier and formatting pass. Hosted run
   40001171992 passes full build, both frozen baselines and all three actual
   scheduler mutations (repeated pre, repeated delay, early continuation publish).
   Contract v2.1 preserves lazy graphics failures and live scratch/odd offsets;
   actual aggregate codec/runtime source review passes. Complete candidate at
   room-session/candidate-e216f0ef has ten superseded files retired and upgraded
   composed checks. Build4 and all1037 mechanism comparisons now pass on source
   0b5ef619789d78be13116549d652aca87edb5596d393441444dde47f450f0a31.
   Comparison caught reversed D830/D870 command flag maps; all four branches
   are corrected. New reference cases1033..1036 distinguish both maps and both
   query outcomes; independent review and old-reference capture pass. The fourth
   mutation deliberately swaps the query map; hosted CI40005395335 passes all
   three frozen baselines and all four room mutants on main.
   Composed host compilation,106 player groups,56 player comparisons and both
   animation/palette runtime guards pass. Speed Booster replay passes28314frames,
   all19 mechanism/player probes, zero failures and120frames after message
   completion. Candidate movie94.81s versus same-route baseline88.18s (+7.5%
   in one paired sample, not a stable performance estimate). Investigate the
   aggregate decoding cost of compatibility getters before further cleanup.
   Hosted validation branch verify/shared-session-20261023 at7ba1e33a includes
   full formatting (two verification files needed import/layout formatting only).
   Production+890lines; verification Rust+673 after formatting; Python+34.
   Caller/ownership/retirement review and hosted composed execution pass.
   Hosted candidate full build and all eight mutants pass in40005712881.
   The shared core is adopted from reviewed7ba1e33a; ten superseded files retire,
   production+890lines, verification Rust+673, combined+1563, Python support+34.
   Unsafe declarations fall7 to2606; remaining legacy effects are still unsafe.
   Formatted source also passes all1037 comparisons (99.22s), no mismatches.
   This is shared interval ownership, not persistent whole-game Session.
   Local frontend build was blocked
   by Defender ASR01443614-CD74-433A-B99E-2ECDC07BFC25 executing khronos_api's
   build script (event1121); use the hosted build, preserve the block. Formatted
   local direct verifier build5 passes on source
   8a4ee812de0eaa7b25dcecd6877c2f08d6000aae282a8efbe257ba27d286d31d.
   Five compatibility getters now use checked live reads at their existing
   RAM-authoritative boundary, removing aggregate import/export per read.
   Independent review passes;54 direct queries and16 nested room/enemy cases
   match frozen outputs,106player groups pass on main's rebuilt verifier.
   Case1037 adds an exact frozen CI baseline; old case IDs and mutation kill
   criteria are unchanged. Optimized replay28314frames/19probes/zero failures
   takes96.14s: no measured speedup versus94.81s before. Production+7lines,
   verification Rust+93, Python+2; unsafe count unchanged. Nested EyeDoorProjectile
   argument callback is not covered; direct getter address/endian/live checks are.
   Next complete ownership scope: compose the18-slot enemy-projectile pool with
   this Session, redirect room/enemy interactions, remove enemy compat.rs and the
   last ownership_transport user, and replace ordinary per-field edges routing.
   Freeze full setup/pre/contact/drop/attachment/effects contract before splitting
   implementation. Reuse1382comparisons/87checks, extend shared-state mutations;
   no net deletion claim until complete replacement is counted.
   Design contract now independently accepted under next-enemy-session. Union
   audit:48998 existing +801enemy +20new scalar bytes =49819 disjoint WRAM bytes;
   native spawn16bytes separate. Remove the old cached statue-finished input when
   it becomes owned. Preserve18A8 invincibility,18AA knockback and18AC projectile
   protection as distinct fields. Concrete Rust interfaces and lane ownership
   are being prepared; no gameplay implementation split yet. Refreshed all
   1382captures/87checks from main1f0cb0a6: every case passes and all frozen
   captures match,133.85s on source
   b03f593e0eed142c9dc79a287ad269b33212881de84c05457a6844cd3438a451.
   Separate verification workers draft
   actual restoration boundary cases and shared/native ownership checks.


   Room mechanisms are now adopted as one production responsibility: creation,
   setup, pre-actions, all 110 command identities, scheduler, drawing/clipping,
   graphics, message suspension, lifecycle and encounter/contact/Xray callers.
   All 24 RAM adapters and their superseded rules, setup runtime, spawning and
   sm_84 are deleted (51 files). Only the unrelated Golden Torizo palette and
   two Samus sound routines relocate intact. One Fields interface consolidates
   rule state/effects; drawing shares emission and destination calculations.

   RoomMechanisms owns 40 slots and graphics/drawing/execution fields during
   subsystem intervals. Foreign calls publish and consume that owner, then
   import actual returned RAM. Full flags, stale fields, live selectors, aliases
   and context-bound continuation remain intact. This is not persistent ownership
   between operations. Native ROM/effect boundaries remain unsafe and qualified.

   Final source 88265b3ed879c62fb219b6829703f9560c56cc6995def849bb9c44e6102569cd
   passes release/frontend/verifier builds and the complete frozen baseline:
   1015 full RAM/return/native-trace outputs match byte for byte, two exact native
   exit diagnostics match, and one bounded nonreturn is observed. Two additional
   groups pass for ownership/unwind/aliases and the eight qualified commands.
   Six formerly unaligned algorithms use independent byte expectations plus
   existing aligned drawing references; two wrappers check production helpers
   and statically inspected dispatch/native identities. Their unchanged native
   pose intervals are not claimed as defined Rust equivalence. The nonreturn
   case observes a timeout, not its preceding clear dynamically.

   Speed Booster inputs 0..28313 pass (28314 frames, zero failures). All three
   spawn sources, terrain/drawing, gameplay and message completion are required
   hits. Message completion occurs at 28193, followed by 120 frames. Coverage:
   12 rooms, nine enemy definitions, no credits. Existing 106 projectile groups
   also pass. Independent implementation/caller/check review passes.

   Integration delta: -1154 lines / -12222 tokens / -39458 LF bytes. Include the
   already-landed 926-line baseline: complete mechanism effort is -228 lines /
   -4214 tokens / -6464 LF bytes. Current source: 108682 Rust lines, 2732 unsafe
   declarations (-46 from frozen baseline). The unrelated room-copy correction
   below is excluded from mechanism savings. No new maintained native/tooling
   source or dependencies. Local frozen references and integrated-2 artifacts
   remain in Local/sm-decomp-experiments/20261023/room-mechanisms.

   Ownership audit found a concrete remaining alias writer: PalFx_ProcessOne
   writes palette_buffer + (color & !1); color 06C8 reaches mechanism tilemap C6C8
   and color 1E1C reaches instruction timers DE1C. Both are defined in-bounds paths.
   Whole-RAM restore/reset/load, replay patches, CPU/DMA and the shared 613
   continuation also prevent persistent ownership. Do not clamp these offsets
   or claim persistence from adding a long-lived field or two bus hooks.

   Shared ownership transport is adopted: projectile Transaction and mechanism
   RoomPort use one private maintained implementation, registered privately under
   each namespace. The separate semantic codecs remain independent. Duplicate
   acquire/publish/release/foreign/drop and checked-write implementations are
   removed; the constructor exposes no new game-wide byte or state interface.
   Strengthened existing checks cover optional ROM absence, value-before-index
   evaluation and nested calls between the two actual owners. Final delta including
   registration/checks is -8 lines/-5 tokens/-133 LF bytes. Current Rust 108674;
   unsafe/cast/native counters unchanged. This is modest deduplication, not a
   persistent owner or backend replacement.

   Source 06c88508786a98d3bda75f0bfbb1b2a33d65398ac3783f7bffb81f8f2096667d passes
   release/frontend/verifier build, 106 projectile groups, 104 fresh projectile RAM
   comparisons, all 1018 mechanism baseline outcomes and groups 1018/1019. The
   28314-frame Speed Booster route again passes with all seven required probes
   and 120 frames after message completion. No credits claim. Independent review
   passes. Evidence is local under 20261023/shared-ownership.

   PaletteEffects is adopted: one eight-slot owner and set_enabled/clear/spawn/
   advance operations replace sm_8d and its internal palette_ram adapter. All 13
   caller files migrate in the same change. The private shared transport is reused;
   color writes retain their broad byte offsets and immediate ordering. Ownership
   is interval-scoped; this does not close the persistence blockers above.

   Source 6e224f28e382b7caf9c9527c099ef067633ce098c9d83434cfea6a149fb1c171
   passes release/frontend/verifier builds, all 512 frozen old-output comparisons,
   and all three shared comparison exports' buffer/known-output checks. Any% boot
   inputs 0..271 pass 272/0 with all seven palette probes and 120 following frames.
   This is opening-sequence coverage, not gameplay rooms or credits. Constructed
   cases cover remaining pre-action/command identities, aliases, wrapping,
   diagnostic/write prefixes and real music callback state mutations.

   Production delta: +126 lines/+1388 tokens/+2897 LF bytes. Separate verification
   support: +370/+3683/+14276, including a dedicated verification/palette_effects.rs
   carrier. Combined Rust is 109399; unsafe declarations 2728. Four fewer unsafe
   declarations overall, but native diagnostics remain explicit. This adoption
   improves ownership/safety and retires bank orchestration; it is not net code
   reduction. Independent final implementation/caller/fixture review passes.

   Parallel next work: complete animated tiles including its actual NMI consumer
   and typed per-upload/acknowledgement contract; complete enemy-projectile state,
   drops/contact/attached-body producer closure; direct-field palette lifecycle/
   interpreter with semantic effects and an independent Session seed. These are
   isolated drafts/designs, not adopted runtime. The source audits did not establish
   large duplication savings for animated tiles or enemy projectiles. Do not claim
   generic interpreter sharing: their cursor capture, selector and failure rules
   differ. Safety/architecture work may grow production; report its cost honestly.
   The explicit upload and backend-free typed-core scopes must advance the actual
   destination, rather than add more memory-access wrappers. Price production and
   verification separately. Persistent whole-runtime ownership remains unproven.
   Actual-old baselines are now frozen on source
   108fa09ab7eb1a229a7b5a041d653eebf5855cbf1d7faf254db94b2723665601:
   438 animated-tile cases and 354 enemy-projectile core cases pass. Animated
   cases cover all 24 commands, near/broad aliases and actual DMA/partial transfer;
   enemy core cases are not full callback/drop/contact/attachment coverage.
   Their carriers/registration add 834 verification-support lines, no production
   algorithm changes. The direct-field palette core is now adopted at 601f8da3.
   GitHub Windows run 39986571000 passes independent core execution, frontend
   build and formatting. Local checks pass 512 frozen observations, runtime
   guard checks, 15 animated-tile callers and 272 opening frames with zero failures.
   Production adds 227 lines; verification adds 328; CI/Cargo adds 13. This is
   architectural progress, not reduction or persistent whole-game ownership.
   Local Defender ASR 1121 / 01443614-CD74-433A-B99E-2ECDC07BFC25 still blocks
   the standalone executable; preserve that block. Enemy-projectile caller
   closure continues in isolated drafts.

   Animated tiles are now adopted: six typed slots, all 24 commands, explicit
   upload/acknowledgement and 21 call sites across eight files replace sm_87 and
   the old NMI upload body. Source
   23fd5fb6a109c2b6bc6105e52523a1ebb579200abb14f0dff6f61935844fccb2
   passes release/frontend/verifier builds, full formatting and all 438 frozen
   comparisons. Speed Booster from boot passes 20947 frames, zero failures:
   spawn first at 20857, upload at 20894, command at 20944, then two follow-up
   frames. Earlier frames are setup, not animation coverage. Production adds
   199 lines; verification/support adds 35. Rust total 111035; unsafe declarations
   2725. Independent review passes. Execution still uses the compatibility Port;
   the next isolated animation/palette composition removes that dependency.
   This is not persistent Session ownership or independent backend completion.
   Follow-on composed core is adopted at354f1fb5. It removes Port/Game from
   animation execution and composes the actual animation/palette cores with one
   color owner. Full GitHub run39989858734 passes independent-host execution,
   frontend build and formatting. Local438comparisons, both runtime guards and
   20947frames pass. Production+124lines, verification+587, CI/Cargo+13; no net
   reduction. The independent host is a bounded prepared slice; production
   ownership remains interval-scoped. Enemy dual594cases pass; old1362references
   are now being captured. Full caller adoption, deduplication and native-ROM
   ownership changes remain isolated pending validation.

   Baseline route failure at input 20857 is fixed: the room-content loader introduced
   at f116c8a6 copied overlapping layer2/BTS buffers forward. The actual room size
   is 5A00. Valid in-bounds overlaps now preserve entry source contents; nonoverlap
   and invalid-extent partial writes retain their prior behavior. This agrees with
   native descending-word copies for normal room layouts, not malformed odd sizes.
   The actual decoder/split regression fails before the fix and passes afterward;
   boundary and partial-failure cases pass on both. Final source
   2e9cd73487ea202c297ea4b07e622bca04c2208eaf4b6f24d8bd7addbdeaea86 builds and
   verifies inputs 0..27683 with no failures (fresh 20857-frame prefix + 6827-frame suffix), both
   corrected-copy probes, completed loading, gameplay and message execution plus
   120 follow-up frames. No credits coverage. Main +89 lines / +951 tokens / +3581 LF bytes;
   local diagnostic +10 / +282 / +858; total 99 / 1233 / 4439. Current main: 109227 Rust lines.
   The executed observer is preserved in debug-room-copy-20857; its reconstructed
   source hash matches the original probe build. No debug observer remains in main.
   New test exports invalidated the earlier checkpoint; final prefix was regenerated.

   `game/player_projectiles` owns ten shots, eighteen trail pairs, weapon/charge
   controls and launch history during each complete subsystem interval. One
   391-word / 782-byte compatibility map handles canonical, broad, odd and signed
   access through the same algorithms. Foreign calls publish and relinquish the
   owner, then reacquire their actual returned RAM. No owner persists across native
   gameplay, reset/load or oracle intervals. Unwind preserves successful writes;
   this is not a claim about pre-panic-hook or abort publication.

   Fourteen whole adapters are retired, along with old bank dispatch/reset/update/
   reflection/flare and sound orchestration. Actual player, door, enemy, cinematic,
   scene, message and beta callers are migrated. Both formerly missing motions
   90D4D2 and 90EFD3 execute their real behavior. Native output/content remains
   explicit: flare tables use loaded_rom while sequences retain captured g_rom.
   The unchecked native sequence boundary is still unsafe. Persistent ownership,
   backend independence, complete-game coverage and the reduction goal remain open.

   Final source `24bd3d424dea880173b5ae8ebaa9b25dd4fa9f1d0e57696deaac1621bda3e8b7`:
   release library/frontend/verifier build passes; 106 regression groups pass.
   All 104 synthetic scenarios also compare the entire 131072-byte RAM image
   directly against independently executed old entries, with zero differences.
   The two ownership groups cover every typed field/lane, broad/odd/signed aliases,
   partial versus atomic writes and foreign returned-RAM/unwind behavior.
   Producer assertions establish actual beam/missile/bomb/SBA/spread creation and
   motion, playback and history tails. Expiry/contact cases include native Power
   Bomb DMA setup and channel assertions. No ROM-derived fixtures are committed.

   Final oracle replay: 8780 frames, zero failures, exit0, 24.70s replay/25.0235s
   invocation. Seven required probes execute with 120 follow-up frames: weapons
   1649; history 1900; flight 85; playback 42; projectile draw path 135; explosion
   draw path 22; trails 3604. Last first hit is input8659, final input8779. Coverage
   is 427 gameplay frames, room DF45, enemy E23F, no credits. Synthetic cases are
   not exhaustive; family cases with no-op motion do not establish flight coverage.

   Main: 108821 Rust lines / 226 files, +1389 lines / +11644 lexical tokens /
   +43988 LF bytes. Runtime plus probes accounts for +596 lines; maintained
   verification adds 793. Counted unsafe declarations 2767 (-22), cast lines5892
   (-26). Two verifier exports account for no_mangle growth to67; C ABI metric7.
   Retained external reference/fixture source adds136/4316/11355; the initial
   ownership experiment adds478/5029/20068. Complete incremental authored source:
   +2003 lines / +20989 tokens / +75411 LF bytes. Staging copies are counted once;
   archiving the old experimental candidate earns no production retirement credit.
   Independent source, caller, outcome, provenance and price review approved.

   The previous 60-suite/59-vector palette result and nine fatal pairs remain
   unchanged historical baseline evidence, not new integration tests. All compiled
   runtime284/harness338 inputs and recorded artifacts match. The 341-file
   historical composition is restored exactly in consolidation-palette-reference-
   sm-src; the original archive is preserved. See HANDOFF for commands and local
   evidence. The next proposal must choose a complete responsibility from current
   callers and price its adoption/deletion; another wrapper layer is insufficient.

   Next dependencies, in order:

   - [x] Resolve the preserved beam-launch draft, synchronize the carrier and
     retain independent old decoding. The original draft remains in `launch-draft/`;
     [the handoff](HANDOFF.md#paused-launch-draft) describes the verified continuation.
   - [x] Share missile acceleration and retire both duplicate bodies with
     canonical owned, broad RAM, failure-prefix and consumer comparisons.
   - [x] Share extension creation, migrate both callers and delete duplicate
     bodies with canonical/broad allocation, failure-prefix and consumer checks.
   - [x] Share complete placement, migrate beam/missile/extension callers and
     remove bank/owned/broad duplicates with independent failure-prefix checks.
     Ordinary flight ordering and extension contact are already shared.
   - [x] Share all five initializer variants, migrate actual producers and remove
     duplicate sequencing/table selection with independent failure-prefix checks.
   - [x] Share playback and trail selection, migrate HandleProjectile/trail-place
     consumers, retire old bodies/helpers and compare scheduler/continuation tails.
   - [x] Share complete trail emission/placement/side playback and native OAM
     output, migrate real flight/special callers and retire duplicate owned bodies.
   - [x] Share projectile drawing, sprite output and echo/trail tails; migrate
     actual callers and remove duplicate bodies with independent comparisons.
   - [x] Share block reactions, slopes and trigger selection; migrate actual
     shot/bomb/grapple/spread consumers and compare native setup/resumed tails.
   - [x] Share complete all-source mechanism allocation/initialization, migrate
     real producers and delete duplicate source sequencing with independent checks.
   - [x] Share all four ordinary door setups, retire duplicate bank/owned rules,
     and compare Grey/shot consumer tails against independent frozen bodies.
   - [x] Share all 13 break/reveal/respawn/retained terrain setups and retire
     duplicate bodies with independent setup and actual caller checks; the 18
     historical visible-render cases are qualified above.
   - [x] Share all 14 gate trigger/placement/shootblock setups, retire helpers
     and compare actual signal, projectile-init/wake and shot-scan tails.
   - [x] Share all seven contact-physics setups and compare actual movement,
     inside-sampling, aligned displacement aliases and fatal null handling.
   - [x] Share both Chozo hand contacts, preserve ordered effects and captured
     release, and compare defined paths separately from checked-only old hazards.
   - [x] Share all 11 station setups and their activation helper, preserving
     lazy guards, live aliases and association order; compare actual consumers.
   - [x] Share all seven extension/scroll setups, preserve missing-target
     nonreturn and ordered aliases, and compare actual redirect/scroll consumers.
   - [x] Share all 11 item initialization/wakeup setups, preserve live aliases
     and missing association, and compare defined setup and qualified consumers.
   - [x] Share all five cannon setups, preserve damage linkage and ordered
     geometry, and compare shot-count/damage consumers with qualified down-right.
   - [x] Share plant capture/hold/damage/protection, retire five bodies and
     bypass native candidate callbacks; compare complete nonvisual release tails.
   - [x] Share eye-door placement/open wakeup/blue conversion, retire five
     bodies and helper, and compare actual progress and collision/deletion tails.
   - [x] Share Metroid callback selection and quota-event polling, retire five
     bodies, and compare actual death/persistence/room-selection interactions.
   - [x] Share deferred sound setup/countdown and retire two bodies; compare
     actual creation, scheduler expiry and queue rejection/acceptance.
   - [x] Share Speed Booster escape setup/three-stage controller, retire four
     bodies and compare actual collection, motion and event persistence tails.
   - [x] Share Shaktool room scroll/event control and retire two bodies; compare
     actual room setup, bomb fuse, camera and persistent event consumers.
   - [x] Share animal-rescue wall/event/passage/door-gate control and retire five
     bodies; compare actual script, terrain, collision and event consumers.
   - [x] Share paired Tourian invisible-wall controls and retire both bodies;
     compare actual enemy producers, deletion and material collision effects.
   - [x] Share Mother Brain escape-door setup through the existing door core;
     compare actual debris/door producer, player transition and offscreen script.
   - [x] Share Mother Brain glass setup/hits/threshold/burst and both count
     producers; compare actual shots/shards and guarded full script/event tails.
   - [x] Share Botwoon delay/scroll and both crumbling-row commands; compare
     death/reload, Tourian producer and complete guarded terrain/collision tails.
   - [x] Share Wrecked Ship treadmill clear/activation/carry; compare actual
     door producers, boss/reentry gates, inside movement and animation cycles.
   - [x] Share Ceres escape-door setup through the existing door core; compare
     actual room-state selection, allocation, deletion and collision/reentry tails.
   - [x] Share broken-turret grapple damage and periodic health application;
     compare allocation/held contact, diagnostic entry and environmental producers.
   - [x] Share Chozo plug/trigger/watcher/acid behavior; compare both producers,
     qualified watcher prefixes/completions, nested allocation and passage tails.
   - [x] Share unused shot/grapple installation and the exact legacy helper;
     compare constructed scripts, actual collision consumers and respawn tails.
   - [x] Share Bomb Torizo statue setup/wake/debris/music and actor wait;
     compare actual room selection, guarded full script and death/reentry tails.
   - [x] Share Noob Tube rupture and D67F/D689/D693 installations/live D6CC
     alias; compare real room selection, supported sibling prefixes and full
     rupture/event/duplicate tails with explicit projectile-class injection.
   - [x] Close the 14 residual setup identities and retire native bank setup
     dispatch. Share full-u32 typed decoding and ordinary execution, preserving
     aliases, synchronous writes, nested displacement and fatal diagnostics.
   - [x] Share the complete operation-owned Samus movement/contact scan and
     retire raw displacement forwarding through creation/setup/nested hands;
     validate all four movement/pose entries, branches and resume/failure tails.
   - [x] Share complete bomb/power-bomb/spread lifecycle and terrain contact;
     execute owned terrain directly and compare actual HDMA growth into Noob Tube.
   - [x] Share complete special-beam creation and all six motion states; retire
     duplicate rules and compare actual input through natural retirement.
   - [x] Share complete beam charge/firing and direct initial scans; retire
     duplicate rules and compare actual input through natural retirement.
   - [x] Share complete missile/super firing; retire duplicated producers and
     compare actual input through extension, impact and natural retirement.
   - [x] Share complete morph bomb/power-bomb/spread creation; retire duplicate
     producers and compare actual input through fuse, all-five spread retirement
     and real power-bomb HDMA/tube activation.
   - [x] Share complete HUD switching/cooldown/weapon coordination; retire duplicate
     algorithms and no-op transfer, preserve in-memory inventory refresh, and
     compare actual selection through lifetimes and grapple/Xray scheduling.
   - [x] Share complete player preparation/setup/finalization; retire duplicate
     phase bodies and validate real demo/beta restoration, caller suspension,
     intro gating, aliases and failure prefixes.
   - [x] Share complete charged-shot palette feedback and output resolution;
     validate actual producer/coordinator/beta/Misc and next-feedback chains.
   - [x] Adopt the complete owned projectile subsystem with real callers and
     retire its bank/operation adapters; validate actual old/new entries and oracle.
   - [ ] Select the next complete ownership boundary from current production
     callers; specify state, foreign interactions, deletions and validation before
     implementation. Keep persistent ownership and backend migration explicit.

   Continue the durable candidate at
   `C:/Users/Jim.Lundin/AppData/Local/sm-decomp-experiments/20261022/player-projectiles/`.
   `runtime-impact-working/` contains the verified consolidation and historical palette evidence, with
   `CHARGE_PALETTE_HANDOFF.md`, `charge-palette-price.json`,
   `charge-palette-results.json` and the charge-palette fatal JSON files;
   `charge-palette-parent/` preserves frozen inputs. Prior preparation evidence
   remains in `PLAYER_PREPARATION_HANDOFF.md` and `player-preparation-parent/`. Prior HUD evidence remains
   in `HUD_COORDINATOR_HANDOFF.md` and `hud-coordinator-parent/`. Prior morph-bomb evidence:
   `MORPH_BOMB_TRANSACTION_HANDOFF.md`, `morph-bomb-transaction-price.json`,
   `morph-bomb-transaction-results.json` and `morph-bomb-transaction-fatal.json`;
   `morph-bomb-transaction-parent/` preserves frozen inputs. Prior missile evidence:
   `MISSILE_TRANSACTION_HANDOFF.md`, `missile-transaction-price.json`,
   `missile-transaction-results.json` and `missile-transaction-fatal.json`;
   `missile-transaction-parent/` preserves frozen inputs. Charge, special and bomb
   evidence remain in their corresponding handoffs, result/price/fatal files and
   parent snapshots. Use PYTHONUTF8=1 for local Python source comparisons/pricing.
   Samus contact evidence remains in `SAMUS_CONTACT_HANDOFF.md`,
   `samus-contact-price.json`, `samus-contact-results.json` and
   `samus-contact-parent/`.
   Setup-dispatch evidence remains
   with `SETUP_DISPATCH_HANDOFF.md`, `setup-dispatch-price.json`,
   `setup-dispatch-results.json` and supplemental `setup-dispatch-gate-results.json`.
   `setup-dispatch-parent/` preserves frozen inputs. Noob Tube evidence remains
   with `NOOB_TUBE_HANDOFF.md`, `noob-tube-price.json` and
   `noob-tube-check-2.log`. `noob-tube-parent/` preserves prior
   main/carrier/local/composition inputs. Torizo evidence remains
   with `TORIZO_STATUE_HANDOFF.md`, `torizo-statue-price.json` and
   `torizo-statue-check-1.log`. `torizo-statue-parent/` preserves prior
   main/carrier/local/composition inputs. Block-reaction evidence remains
   with `BLOCK_REACTION_HANDOFF.md`, `block-reaction-price.json` and
   `block-reaction-check-1.log`. `block-reaction-parent/` preserves prior
   main/carrier/local/composition inputs. Chozo-terrain evidence remains
   with `CHOZO_TERRAIN_HANDOFF.md`, `chozo-terrain-price.json` and
   `chozo-terrain-check-1.log`. `chozo-terrain-parent/` preserves prior
   main/carrier/local/composition inputs. Periodic-damage evidence remains
   with `PERIODIC_DAMAGE_HANDOFF.md`, `periodic-damage-price.json` and
   `periodic-damage-check-1.log`. `periodic-damage-parent/` preserves prior
   main/carrier/local/composition inputs. Ceres evidence remains
   with `CERES_DOOR_HANDOFF.md`, `ceres-door-price.json` and
   `ceres-door-check.log`. `ceres-door-parent/` preserves prior
   main/carrier/local/composition inputs. Treadmill evidence remains
   with `TREADMILL_HANDOFF.md`, `treadmill-price.json` and
   `treadmill-check-attempt1.log`. `treadmill-parent/` preserves prior
   main/carrier/local/composition inputs. Crumbling-wall evidence remains in `CRUMBLING_WALL_HANDOFF.md`, `crumbling-wall-price.json` and
   `crumbling-wall-check-attempt2.log`. `crumbling-wall-parent/` preserves prior
   main/carrier/local/composition inputs. Glass evidence remains in `MOTHER_BRAIN_GLASS_HANDOFF.md`, `mother-brain-glass-price.json` and
   `mother-brain-glass-check-attempt-1.log`. `mother-brain-glass-parent/` preserves
   prior main/carrier/local/composition inputs. Escape-door evidence remains in `MOTHER_BRAIN_DOOR_HANDOFF.md`, `mother-brain-door-price.json` and
   `mother-brain-door-check-attempt-1.log`. `mother-brain-door-parent/` preserves
   prior main/carrier/local/composition inputs. Tourian evidence remains in
   `TOURIAN_WALL_HANDOFF.md`, `tourian-wall-price.json` and
   `tourian-wall-check-attempt-1.log`. `tourian-wall-parent/` preserves prior
   main/carrier/local/composition inputs. Animal-rescue evidence remains in
   `ANIMAL_RESCUE_HANDOFF.md`, `animal-rescue-price.json` and
   `animal-rescue-check-attempt-2.log`. `animal-rescue-parent/` preserves prior
   main/carrier/local/composition inputs. Shaktool evidence remains in
   `SHAKTOOL_HANDOFF.md`, `shaktool-price.json` and `shaktool-check-attempt-1.log`.
   `shaktool-parent/` preserves prior main/carrier/local/composition inputs.
   Booster-escape evidence remains in
   `BOOSTER_ESCAPE_HANDOFF.md`, `booster-escape-price.json` and
   `booster-escape-check-attempt-1.log`. `booster-escape-parent/` preserves prior
   main/carrier/local/composition inputs. Sound evidence remains in
   `SOUND_DELAY_HANDOFF.md`, `sound-delay-price.json` and
   `sound-delay-check-attempt-1.log`. `sound-delay-parent/` preserves prior
   main/carrier/local/composition inputs. Quota evidence remains in
   `METROID_QUOTA_HANDOFF.md`, `metroid-quota-price.json` and
   `metroid-quota-check-attempt-1.log`. `metroid-quota-parent/` preserves prior
   main/carrier/local/composition inputs. Eye-door evidence remains in
   `EYE_DOOR_HANDOFF.md`, `eye-door-price.json` and
   `eye-door-check-attempt-1.log`. `eye-door-parent/` preserves prior
   main/carrier/local/composition inputs. Plants evidence remains in
   `PLANTS_HANDOFF.md`, `plants-price.json` and `plants-check-attempt-1.log`.
   `plants-parent/` preserves prior main/carrier/local/composition inputs.
   Cannon evidence remains in
   `CANNON_SETUP_HANDOFF.md`, `cannon-setup-price.json` and
   `cannon-setup-check-attempt-1.log`. `cannon-setup-parent/` preserves prior
   main/carrier/local/composition inputs. Item evidence remains in
   `ITEM_SETUP_HANDOFF.md`, `item-setup-price.json` and
   `item-setup-check-attempt-1.log`. `item-setup-parent/` preserves prior
   main/carrier/local/composition inputs. Scroll evidence remains in
   `SCROLL_SETUP_HANDOFF.md`, `scroll-setup-price.json`,
   `scroll-setup-check-attempt-2.log` and three `scroll-missing-*.log` files.
   `scroll-setup-parent/` preserves prior main/carrier/local/composition inputs.
   Station evidence remains in `STATION_SETUP_HANDOFF.md`,
   `station-setup-price.json` and `station-setup-check-attempt-1.log`;
   `station-setup-parent/` preserves its prior inputs. Chozo evidence remains in
   `CHOZO_CONTACT_HANDOFF.md`, `chozo-contact-price.json` and
   `chozo-contact-check-attempt-5.log`; `chozo-contact-parent/` preserves its
   prior inputs. Physics evidence remains in
   `CONTACT_PHYSICS_HANDOFF.md`, `contact-physics-price.json` and
   `contact-physics-final.log`; `contact-physics-parent/` preserves prior inputs.
   Gate results remain in `GATE_SETUP_HANDOFF.md`, `gate-setup-price.json`
   and `gate-setup-final.log`.
   `gate-setup-parent/` preserves prior main/carrier/local/composition sources.
   Terrain results remain in `TERRAIN_SETUP_HANDOFF.md`, `terrain-setup-price.json`
   and `terrain-setup-final.log`; `terrain-setup-parent/` preserves prior inputs.
   Door results remain in `DOOR_SETUP_HANDOFF.md`, `door-setup-price.json`
   and `door-setup-attempt-1.log`.
   `door-setup-parent/` preserves prior main/carrier/local/composition inputs.
   Creation results remain in `MECHANISM_CREATION_HANDOFF.md`,
   `mechanism-creation-price.json` and `mechanism-creation-final.log`.
   `mechanism-creation-parent/` preserves prior
   main/carrier/local/composition sources. Block results remain in
   `BLOCK_HANDOFF.md`, `block-price.json` and `block-final-attempt-2.log`.
   `block-parent/` preserves prior main/carrier/local sources and composition.
   Drawing inputs/results remain in `drawing-parent/`, `drawing-composition-parent/`
   and `DRAWING_HANDOFF.md`, including its unresolved inherited Intro mismatch.
   The trail handoff/results also remain available.
   `trail-parent/` and `trail-composition-parent/` preserve prior inputs.
   `playback-parent/` and `playback-composition-parent/` preserve prior inputs.
   Initializer references remain in `initialization-parent/`; placement, creation,
   acceleration, extension and independent flight snapshots/caches remain preserved.
   `runtime-impact-stage/` preserves the previous terminal stage; the independent
   flight reference remains in `flight-parent/`; `extension-parent/` saves the
   immediately preceding source and composition changes.
   Earlier room checks remain in `runtime-room-stage/`. Read the relevant commands,
   filenames and capture limitations in [HANDOFF.md](HANDOFF.md). Do not restart
   or overwrite landed changes. The inherited comparison carrier is edition
   2021; current production is edition 2024. Check final-source correspondence
   explicitly.

   If stuck for more than an hour on one item, record the findings here and stop.

## Toolchain and Windows constraints

Use Rust edition 2024. The requested latest nightly is installed as
`nightly-2026-10-22` (upstream version
`1.101.0-nightly (1a2c4797e 2026-10-21)`), but Defender ASR event 1121, rule
`01443614-CD74-433A-B99E-2ECDC07BFC25`, blocks its compiler. Its version is not
runtime-verified. Keep the usable stable default,
`1.99.1 (58cf8f9c3 2026-10-13)`, target `x86_64-pc-windows-gnu`, until the
administrator authorizes that compiler. The nightly switch/checks remain open;
do not bypass the block or change security settings.

Edition migration and stable release/compiler/source checks are complete for
their recorded scope, not new gameplay coverage or unsafe elimination. The
three crate roots deny `unsafe_op_in_unsafe_fn`. Repository-wide formatting
already fails on the baseline; format changed files with edition 2024 and
`skip_children=true,reorder_modules=false,reorder_imports=false`.

The existing DLL/ctypes comparison workflow is permitted. Denied executable or
build-script attempts remain failed/blocked, not runtime passes. Use the
[Windows commands](../movies/README.md) and the installed tool paths in the
[iteration guide](ITERATION.md#windows-shell-setup).


## Validation strategy (2026-10-21)

User direction (2026-10-23): include mutation testing of the verification
harness. For each migrated responsibility, choose meaningful faults in its
production behavior (boundaries, state updates, ordering or ownership), confirm
the unmodified checks pass, then confirm those checks reject each compiling
mutant. Use isolated source copies; never adopt mutant code. Compile failures,
unrelated crashes and unexecuted checks do not count as detected mutations.
Investigate surviving mutants and improve coverage or explain equivalence.
Keep mutation support with verification code and report its scope separately;
a few detected mutants do not establish comprehensive mutation coverage.
Run standalone mutation executables on hosted Windows CI under the existing
local Defender restriction. Start with projectile leading-edge and retirement
faults; extend the scope with each subsequent subsystem conversion.

User direction: test movie segments that actually execute the changed code
and its relevant downstream effects. Unrelated frames add no coverage. This
replaces the former full-replay cadence, broader-regression defaults and
historical per-commit recipes; subsystem completion alone requires no full run.

1. **Select before running.** Identify changed routines, relevant branches,
   callers, shared data and downstream effects. Choose the smallest scenario
   that exercises them. Documentation/comment/format-only edits need applicable
   static checks; removing demonstrably unused private code needs a reference
   search and compiler checks. Neither needs a gameplay replay. Frontend-only
   changes need frontend checks, since the headless movie does not call the UI.
   Before every Rust commit, run `cargo fmt` and `cargo fmt --check` from the
   repository root, using the repository toolchain and default configuration.
   CI runs that exact check. Formatting selected files with module/import
   reordering disabled is not a substitute for the full check.
2. **Focused behavior changes.** Run formatting, a release build and the shortest
   exercised oracle segment on the final source. Require explicit routine and
   changed-branch probes with `--require-hit`; a missing hit fails verification.
   Use `--frames` for a known endpoint, or `--after-hits N` to stop after every
   required probe first executes plus N follow-up frames. Choose N to observe
   the actual effects, including subsequent callbacks, frames, transitions or
   persistent writes. Require a probe at the later effect too when available.
   Entry hits alone do not prove branch coverage or correct behavior. Keep all
   normal oracle comparisons, rendering and audio advancement enabled.
3. **Skip only a justified prefix.** A compatible local checkpoint must precede
   affected setup and calls. Inspect earlier callers and shared-state effects
   before reusing it after an edit: runtime compatibility alone does not prove
   the changed build reaches that state. If a checkpoint must first be built,
   identify that prefix as setup, cache it before affected setup/calls, and
   keep setup frames separate from changed-code coverage. An uncovered routine
   needs another route or a focused component/oracle test of its real implementation;
   repeatedly running an unrelated green movie adds no coverage. Constructed
   tests must state their limits and do not establish movie execution. Choose
   arithmetic/aliasing cases for plausible failure modes rather than defaulting
   to large unrelated sweeps. If relevant context remains uncovered, report it
   and hold further cleanup of that behavior.
4. **A full movie requires demonstrated coverage.** Before starting one, show
   why the changed code or its measured downstream effects require that whole
   route rather than bounded exercised segments. Name the required probes and
   expected frame ranges first. Code size, broad subsystem labels, elapsed time,
   commit count and batch completion never justify a full replay by themselves.
   If coverage is missing, select another exercised segment or a component/oracle
   case; do not lengthen an unrelated run. Stop a redundant run once its lack of
   relevance is established. Reuse source-checked builds and compatible checkpoints.

Record selected scenarios, required hits, executed frame range and result in
the commit message, with the reason for a full replay or static-only check.
Use source-checked builds and `sh tools/progress.sh` as usual. No new test
registry, evidence store or coverage dashboard is needed. Commands and
checkpoint constraints are in [movies/README.md](../movies/README.md).

## The oracle

Select changed behavior using the validation strategy above and
[movie commands](../movies/README.md). The historical any% route visits six
rooms; Speed Booster visits 55 rooms and 53 enemy definitions, reaching
Kraid/Varia. Neither establishes credits coverage. Later bosses/endgame and
unvisited branches need relevant checks. The local Charge Beam route includes
Morph Ball eye-beam inputs 28200..28550; a route's existence is not coverage of
an arbitrary edit.

Inspect `cpu_infra.rs` comparison masks, `PatchBugs`, hardware/audio limits and
persistence behavior when a change touches them. Keep normal oracle comparisons,
rendering and audio advancement enabled. Constructed components may check
unvisited actual implementations with explicit limits; source stamps prove
identity, not behavior. Required probe misses fail even with zero frame diffs.

The Windows launcher checks source bytes embedded in the compiled library.
Register new library files in `sm/src/verify_source.rs`. Keep source and inputs
fixed during checks; the launcher rejects stale builds and source changes.
Checkpoints must precede affected history. Removing temporary probes requires
rebuilding and repeating the same measured segment on clean source, as described
in movies/README.md. Frontend-only behavior needs frontend checks.


## Definition of done for any change

1. Applicable checks from the validation strategy pass on the final source.
   Documentation-only changes need static checks; code changes need formatting
   and appropriate compiler checks, with release builds for behavior changes.
2. Changed behavior executes in the selected checks, with no new failing frames.
   Static equivalence must establish unchanged operations; an unexercised green
   replay or source inspection alone does not validate a behavior rewrite.
3. Run `sh tools/progress.sh`; explain counter increases. Report complete
   replacement savings and remaining budget only when actually measured.
4. Recheck HEAD and other work before committing to main. Commit one coherent
   idea with the verify result. Update this plan's active item when status or
   direction changes; routine check details belong in the commit message.

## Workspace discipline

Keep this file an active plan, not a per-commit journal. Replace current status
in place when it changes; routine check details belong in commit messages.
Use the [iteration guide](ITERATION.md) to divide one responsibility into
independent investigation/implementation and a single integration lane. This
recipe does not itself start or require subagents.

After the recorded resource exhaustion, run **one heavy compiler job at a time**
across the machine. Coordinate with active owners; `-j 1` only limits one Cargo
process. During compilation, other workers can trace callers, prepare cases and
review isolated source. Finish current cohorts before starting more experiments.
No Git locks may be held during builds, reasoning, verification or waits.

Keep game/host artifacts local to their checkout; seed registry dependencies,
not whole target trees. Reuse source-checked identical baselines and compatible
checkpoints. Use `CARGO_INCREMENTAL=0` for disposable Cargo snapshots, preserve
active working caches, and keep disposable output bounded. Stream compiler output
to files instead of returning full warning/HIR/THIR dumps. Do not rebuild a DLL
while another process has it loaded. Preserve unrelated source changes and other
workers' artifacts. Before removing disposable output, establish that it is no
longer needed and safely resolve its path.

Performance work targets the existing verification pipeline. Measure
edit/build/result time before optimizing it. The 2026-10-22 inspection and
next measurements are in ITERATION.md; historical timings vary with source and
load. Do not build process tooling, evidence stores, registries or dashboards.
If the loop is too slow or weak, record the specific problem here and improve
the verify run itself. Never commit ROMs, movies, save states or ROM-derived
content.

## Progress and references

Verified sound-delay stage: Rust 104,512; maintained C bridge 266 and headers
296 = **105,074 implementation lines**. Including 803 dependency-installer and
107 verifier-launcher lines gives **105,984**. The separate prospective owned
composition and local comparison additions are disclosed above.
Residue counters: raw offsets 21; C blocks 7; counted C exports 6; no_mangle 65;
unsafe declarations 2,885; cast-containing lines 6,033. These narrow counters do
not count all unsafe operations. Use the actual checkout's progress output for
new changes; uncommitted drafts are not landed savings.
The isolated launch draft is now synchronized and verified; its original source
bytes and patch remain preserved outside Git.

External source is disclosed separately: CPU 2,836, SPC 2,104, PPU 1,261, DSP 685,
music 1,659, stb 7,762, configuration/utilities 919, desktop 882, and shader
659 C / 99 header lines. Maintained adapters/installers still count. See the
relevant [CPU](CPU_DEPENDENCY.md), [SPC/music](SPC_DEPENDENCY.md),
[PNG](PNG_DEPENDENCY.md), [configuration](CONFIG_DEPENDENCY.md),
[desktop](DESKTOP_DEPENDENCY.md) or [shader](SHADER_DEPENDENCY.md) note when
changing that boundary.

Iteration change, 2026-10-22: front-load the active queue and validation, move
historical detail out of the required reading path, reuse selected checks, and
coordinate parallel work through one integration/compiler lane. The inspection
found repeated plan updates and multiple build paths. The existing launcher now
uses identical Cargo flags for normal/verification builds, automatically reuses
Cargo with SDL2, offers one release/frontend/verifier check command, streams logs,
and skips identical direct builds with source/compiler/native/output checks.
The active local component runner similarly reuses checked inputs and incremental
codegen. Measured unchanged-build timings and stale-input tests are in ITERATION.md;
no full-cycle speedup is claimed. Compare the next implementation cycles.

Detailed completed/parked experiments, accounting and historical check results
remain in Git at `de9bf05e:docs/PLAN.md`. Consult only the relevant portion when
reviving or depending on an earlier finding, for example:

```powershell
git show de9bf05e:docs/PLAN.md | Select-String 'subject' -Context 4,12
```

Earlier reset history remains at `ec45c573:docs/PLAN.md`; the pre-transpile
project is archived in `archive/pre-transpile-2026-10-17`. `.scratch/`, old
readiness gates, acceptance pointers and translation scopes are not active rules.
The lost TEMP `sm-tailored-next` and original-C/parser artifacts are unavailable;
historical results do not supply inspectable adoption proof. Reconstruct selected
current-Rust checks in durable Local experiment directories when needed, without
presenting a partial reconstruction as the old baseline. Avoid rerunning completed
scans or counting previously adopted savings again.
