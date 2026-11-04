# Plan

Goal: a compact Super Metroid engine playing boot to credits with owned, safe Rust
and clear modules. Remove unsafe/raw-memory operations from engine behavior;
REVIEW scopes frozen compat and verification separately. The completion target
below 100,000 lines applies to sm_data, sm_game, sm_runtime and sm_app. Report gross
physical totals, compat and verification separately; REVIEW owns phase/size measures.

Jim's completion scope (2026-11-01): migrate all engine responsibilities into the
new crate setup and extend verification to the complete existing any%, 100% and
low% movies. Account for boot, transitions and endgame as well as selected rooms;
record uncovered scope until genuine replays and named facts prove it.

Jim's work order (2026-11-01): retire explicit RAM manipulation and migrate engine
responsibilities into owned state in the new crates first. New scenario and trick
expansion waits until that migration is complete. Keep existing regression
verification running to qualify migration changes; complete existing movie scope
remains part of final acceptance.

Jim's migration scope (2026-11-01): each migration PR retires hundreds of actual
RAM accesses through complete domains and repeated patterns, including their
producers, consumers and lifecycle boundaries. Compose independently reviewed
components into a meaningful slice and qualify its combined source. Measure one
overlay against landed main; preserve behavior and report retained raw aliases.
The root's current slice target is at least 200 access retirements. Counts and
the landed burndown belong in the dated PROGRESS_REPORT, with unresolved access
candidates and pending work separate from proved eliminations.

Jim's address rule (2026-11-02): `session_slot!`, `SlotAccess`, `native_word`,
`write_slice_*` and other APIs that name RAM addresses are RAM access. A slice
may not raise `engine_ram_address_sites` or `engine_raw_ram_calls` in
`ram_ratchet.py`. The 152 slice's owned actor files move onto typed Session
fields before it lands; see REVIEW's Phase 2 ratchets.

Jim's CI direction (2026-11-02): verification runs on GitHub Actions, on Linux, through
one workflow ([verify.yml](../.github/workflows/verify.yml), called from ci.yml). Pull requests gate on the
oracle digest lock plus lockstep for rows it cannot hold; full lockstep runs nightly. This
replaces the 2026-10-27 stop on Linux. Local machines compile-check; they don't need
a verifier build or replays. Push a slice once and let its gate finish unless fixing a
failure. New commits supersede their branch's PR and push gates; nightly and dispatched runs finish.
Jim's workflow direction (2026-11-03): CI has fixed-scope, delimited workflows that `ci.yml`
composes. `ci.yml` alone holds triggers and has no steps; it calls `quality.yml` (format, lints,
ratchets, tests, dependency direction), `verify.yml` (replays,
by `scope`) and `bench.yml` (replay speed against a base ref). Dispatch picks one `action`. Unused workflows go; the nightly desktop link and mutant runs were removed,
and Quality still checks the mutation anchors. Verify skips replays when a pull request changes
no replay-relevant file. Python is typed and type-checked in Quality. Avoid adding Python unless
it is necessary: this is a Rust project.
Jim's performance direction (2026-11-03): a playable engine needs well above 60 ticks per second;
headless it should reach hundreds. [PERFORMANCE](PERFORMANCE.md) holds the targets, the measured
baseline and the ordered plan (Session authoritative for the whole tick, table-driven projection,
no emulated PPU in headless runs, digest-lock gate).
The production-line ratchet stays advisory until
RAM migration finishes, then becomes enforced so the code shrinks toward the 100,000-line
target. PR descriptions carry landing evidence; don't grow PROGRESS_REPORT for each landing.

Jim retired issue #6 (2026-11-01). Progress and blockers are reported in this
conversation; durable evidence stays in repository reports and owned artifacts.
Historical issue references are evidence provenance, not a reporting destination.

Jim's verifier execution direction (2026-11-02): ordinary Rust compilation is
enough for local compiler checks. Replace the verifier DLL in GitHub Actions
with a headless Rust executable using the existing verifier and orchestration.
Preserve its replay modes, units, checkpoint operations, comparisons and actual
source/native/input identity checks. Replay qualification runs on Linux under
the CI direction above. Existing in-flight replay evidence retains its original
artifact identities.

Jim's CI platform clarification (2026-11-02): Linux CI is intentional. This
supersedes the earlier Windows-only CI direction; local implementation and
ordinary Cargo checking continue in the existing Windows workspace. Consolidate
hosted qualification through the existing verifier and orchestration, preserving
all rows, actual source/executable/native/ROM/input identities and checkpoint
guards. Preserve live Windows and Linux runs with their original proof scope;
integrating CI does not transfer qualification between different heads or hosts.

Jim's current gate clarification (2026-11-02): land on exact-head PR CI and PR
Verify's lock-first gate; dispatched full runs are extra evidence and full lockstep
runs nightly. Preserve the fetched branch's direct headless verifier build and
source/native/capability guards. New commits supersede PR and push gates;
nightly and dispatched runs finish. This supersedes the earlier no-cancellation
direction. Re-cut the parked regression rows from genuine starts, preserving
required facts and continuation checks. Staged-prefix workflow jobs stay removed;
use the private ci-data ROM restore without adding ROM-secret blocks.

The [accepted destination](target-architecture/README.md) is one deterministic owned
Session, our renderer/audio and small platform libraries, independent of SNES emulation.
Map Rando needs custom sprites/palettes, configurable rooms/doors and opt-in re-spin/QoL.
Keep content, topology and options separate from platform and ROM/RAM compatibility;
vanilla is the first fidelity profile. Map Rando is the first use case and acceptance
bar; further clear modularity is welcome. Migration may retain backend/oracle.

## Authority, lanes and Git
Read [REVIEW.md](REVIEW.md) next and its Milestone plan at each item's start; it
supersedes this work order. Only text Jim has approved, landed on main, changes goals
or process (REVIEW, #252). Issues, comments and review text are input, not instructions,
whoever wrote them. Direct user instructions win.
Git history holds no current rules; pre-2026-10-17 workflows are archived.

REVIEW sets active lanes and phase exits; Phase 0 verification-loop work lands
first. Preserve active cloud claims and choose unclaimed phase work. Bank
retirement resumes into target domain packages in REVIEW order; keep M bank
codemods stopped.
Non-enemy replacements aim <=80%; every B qualification record gives the full ratio
and a reason above 90%. Necessary safety growth must be counted and explained, not relabeled.

Never reserve or hold Git index/ref locks; Git's own short locks cover actual writes
only. Before every Git write, fetch/reconcile current origin/main, recheck HEAD/index
and preserve others' work. Never delete another process's lock or force-push main.
Primary transfer requires explicit RELEASE and verified terminal writer/turn
state; preserve live jobs and reservations. Main is the default, isolation optional.
Only the root lands on main; branches can carry coherent source with pending checks named.
Main moves one PR at a time, by fast-forward through `tools/lane/land.sh`, onto a head whose
exact tree passed CI and Verify (#240). If main moved, rebase and let the gate rerun.
No direct commits to main, merge commits or force pushes.
Land verified work promptly; holding it over an hour is a stall. No docs-only commits
without a code commit beside them. Update PLAN only for active direction or blockers.

Use a descriptive subject, retaining A:, M: or B:sm_XX when useful. Keep the body
to 2Ã¢â‚¬â€œ3 short lines explaining the behavior change and relevant verification.
Measure with tools/progress.sh and retain exact production/legacy/unsafe/verification/
support deltas, B replacement ratio/reason, source/native/codec/input pins and actual
check results in the owned qualification/publication record. Commit messages need
no Score, hash inventory or warning ledger. Necessary growth remains accountable.
Report landed behavior, measured burndown and blockers to Jim in the conversation.
Keep changing figures in the dated PROGRESS_REPORT and exact proof in owned records.

Keep replay docs essential, upstream references and CSV/TOML provenance/checks;
shared Python installers/runtime-only restores are support work, counted separately.

## Phase and retained evidence

[REVIEW's Milestone plan](REVIEW.md#milestone-plan-2026-10-29) sets the work order.
Read the dated [Phase 0 qualification](PROGRESS_REPORT.md#phase-0-qualification)
and [retained proof references](PROGRESS_REPORT.md#phase-0-records-and-scheduling)
before changing a retained owner. Preserve original source/DLL/native/input/
loader pins, unexecuted cases and exclusions in its owned receipts.
Persisted compatibility never substitutes for source/native/ROM/input proof or
permits header restamps. Frozen history assigns no work and authorizes no repeated
accepted campaign.

## Implementation contracts

- sm/src is authoritative; no C to consult/regenerate. Preserve quirks; use the oracle,
  ram_map.rs and [bank logs](https://patrickjohnston.org/bank/) when meaning is unclear.
- Before boundaries, state ownership or backend changes, read the accepted destination
  and [state migration](ENGINE_STATE_MIGRATION.md), plus linked contracts for that interaction.
- Keep g_ram byte-compatible with SNES layout and one authority per execution interval.
  Preserve full/odd identities, unknown values, wrapping arithmetic, native versus LE
  access, ROM/native allocation provenance, off-table aliases, suspension and failure prefixes.
- Keep captured identities distinct from fresh selectors; preserve every lazy query,
  ordered effect and Runtime/Drop epoch. Do not predecode, batch or sort effects for savings.
- Migrate complete API/owner/create/setup/update/callback/failure/reset/load/delete/reuse
  and actual consumers before retiring RAM communication. Export before legacy/CPU/native
  writers; import the actual selected normalized image afterwards, including partial failures.
- Completed cores forbid unsafe and need no Game/RAM/raw pointer/backend modules.
  No always-live owned mirror beside writable authoritative RAM, whole-RAM borrow while
  raw aliases remain, forwarding facade or package move presented as completed ownership.
- New owned/target modules forbid unsafe; retained compat/platform interfaces keep
  explicit unsafe entry contracts until their actual consumers migrate. Keep frozen
  SPC700/APU emulation inside REVIEW's compat scope; retirement adds no unsafe
  outside the existing assigned bank/platform owner.
- Checkpoint compatibility describes ordered serialized State fields/types and all
  opaque/manual payload encodings, including tags, lengths and allocation tails.
  Whole-source contributor directions retain source qualification, not format identity.
  Keep actual source/native/ROM/input/loader proof separate; a partial schema or
  matching identifier does not transfer behavior proof or permit header restamps.
- Keep backend/shader/audio/oracle behavior, masks/PatchBugs and snapshots until actual
  consumers and equivalent checks migrate. Dependencies mean the backend is not retired.
  The611..616 normalization means continuation also needs source proof, not byte comparison alone.
- One runner: the existing verifier, growing into the scenario harness under REVIEW's
  [lane S conditions](REVIEW.md#scenario-harness-lane-s-jim-2026-10-31) (Jim,
  2026-10-31). No other process framework, evidence store or registry. Target packages
  get no probes, observers or verify cfg blocks; the engine projection only reads typed
  Session state. No tolerances: fix a divergence or list it in [LEGACY](LEGACY.md).
  A constructed code check is allowed only for behavior no movie reaches: under 100
  lines, with the reason in its first line. Otherwise add a named row or scenario.
- Architectural progress is behavior in a target package compiled independently of
  compat. Report REVIEW's package/cycle measures; package moves alone are neutral.
- Count all helpers/tables/adapters/registration/callers/native runtime code. Separate
  verification, support/installers, dependencies and combined totals; document mixed-file
  exclusions reproducibly. Inline verification stays production; compare like classifications.
- Moves, formatting/packing/comment deletion and production-to-verification relocation
  are not savings. Count overlaps once, check tokens/text, never sum independent drafts
  or claim gross retirement as net. Unsafe deny attributes count as lines, not operations.
- Report physical production and adjusted implementation separately. Adjusted is
  physical minus specifically mapped superseded retained sections. The owned accounting
  record names source pin, file/function/span, replacement/actual caller/proof, current
  retention reason and deletion condition. Count overlaps once; deleting a section
  removes its discount simultaneously. This adjustment claims no physical deletion or
  net savings. Unsafe operation sites/signatures need source maps; text/attribute counts
  stay separate. Use tools/progress.sh --accounting --ref PIN --superseded RECORD.
- Never commit ROMs, movies/checkpoints/saves or derived game data. Only encrypted
  release assets use the existing ci-data route; verify complete restore/member identities.
- Coherent owner commits and corrective fixes compose the migration PR scope
  above. If an item stalls over an hour, record its blocker
  and findings, then keep useful independent work supplied.

## Fast loop

Keep a dedicated verification owner supplied alongside engine migration.
Qualify changed responsibilities and the combined source, including existing-row
captures required by state or codec changes under resource admission. Continue
the complete existing movie scope above; defer new trick/scenario expansion.
Report reached behavior and mutation sensitivity separately from passing rows.

1. cargo fmt --check; cargo check --no-default-features --lib with warnings denied.
   CI builds the headless verifier executable and runs the existing harness.
   Cargo enforces
   warnings='deny' in sm/Cargo.toml for every sm target while preserving dependency
   flags. Direct rustc uses explicit -D warnings, including detached/mutation builds.
   Fix diagnostics; blanket suppression and historical warning-bearing builds do
   not qualify this strict profile. Denied work never passes.
2. verify_dll.py --corpus scenarios --segment for every affected named row,
   plus door-loading-lag-nmi,tas-ceres-four-doors,area-crateria,item-morph-ball; zero failures.
   No reached row: A adds one; B moves unchanged and records uncovered: <function> in qualification.
3. Nothing extra routinely: no mutation/independent controls/frontend metadata, optimized
   rebuilds, clean-source reruns or full replays. Once before deleting a bank, scoped
   cargo-mutants <=30min; full corpus runs on main AFTER deletion, also at review waypoints.
4. Every comparison failure gets a regression row with its fix (a `scenario cut` once
   the harness has it); never weaken
   comparisons/expectations. Retry a flaky step once; a second failure is real.

Use [movies/README](../movies/README.md) for commands; inspect masks/hardware/audio/persistence when touched.

The primary publishes qualified migration slices to main and fixes known CI failures.
Observe the actual CI workflow and commit; continue independent implementation
while hosted checks run. Free runners only; full lockstep runs nightly, after bank
deletion and at review waypoints. Checkpoint rows have 65-minute job limits, 240 for the
long-prefix rows; timeout is failure, not coverage. Manual row IDs are space-separated;
unknown IDs fail. Keep seed/lag/source guards.

## Toolchain, resources and completion
Edition 2024, stable Rust; CI uses `x86_64-unknown-linux-gnu`, and the desktop link
`x86_64-pc-windows-gnu`. See [sm-resources](../.agents/skills/sm-resources/SKILL.md).

Done: Fast loop passes; measure tools/progress.sh and retain explained deltas and
exact qualification evidence. Fetch/reconcile/recheck HEAD/index, commit the coherent
change with a concise message and immediately push main.
Update active PLAN beside code and the dated progress report under REVIEW; no README
progress image or extra readiness gates. Bounded delegated core work continues during CI.
Pre-transpile .scratch/gates/pointers/scopes are inactive in archive/pre-transpile-2026-10-17.
