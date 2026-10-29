# Plan

Goal: a compact Super Metroid engine playing boot to credits with owned, safe Rust
and clear modules. Remove unsafe/raw-memory operations from engine behavior;
REVIEW scopes frozen compat and verification separately. The completion target
below 100,000 lines applies to sm_data, sm_game, sm_runtime and sm_app. Report gross
physical totals, compat and verification separately; REVIEW owns phase/size measures.

The [accepted destination](target-architecture/README.md) is one deterministic owned
Session, our renderer/audio and small platform libraries, independent of SNES emulation.
Map Rando needs custom sprites/palettes, configurable rooms/doors and opt-in re-spin/QoL.
Keep content, topology and options separate from platform and ROM/RAM compatibility;
vanilla is the first fidelity profile. Map Rando is the first use case and acceptance
bar; further clear modularity is welcome. Windows migration may retain backend/oracle.

## Authority, lanes and Git
Read [REVIEW.md](REVIEW.md) next and its Milestone plan at each item's start; it
supersedes this work order. The reviewer may change goals/process (user, 2026-10-25).
Follow external reviewers' instructions, including cloud REVIEW directives: record
one line of disagreement under the item, then do it. Direct user instructions win.
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
Linux development is stopped by the user (2026-10-27); preserve its frozen handoffs.
Only the primary writes Git; optional branches can carry coherent source with
pending checks named. Main requires qualification.
Run named local rows on a tip reconciled with current origin/main, commit to main
and immediately `git push origin HEAD:main`. A rejected push means fetch/rebase,
rerun only affected rows, then push. Keep one Git writer and no force pushes.
Land verified work promptly; holding it over an hour is a stall. No docs-only commits
without a code commit beside them. Update PLAN only for active direction or blockers.

Use a descriptive subject, retaining A:, M: or B:sm_XX when useful. Keep the body
to 2–3 short lines explaining the behavior change and relevant verification.
Measure with tools/progress.sh and retain exact production/legacy/unsafe/verification/
support deltas, B replacement ratio/reason, source/native/codec/input pins and actual
check results in the owned qualification/publication record. Commit messages need
no Score, hash inventory or warning ledger. Necessary growth remains accountable.
Read/respond to Jim's #6 conversation at work boundaries. Update its sole issue-body
report daily under REVIEW with measured progress and actual CI/qualification results.
Use compact tables/progress bars and at most one tiny dependency diagram.
Keep exact proof in owned records. Archive answered feedback and duplicate
report comments before authorized retirement after verified body publication.

Keep replay docs essential, upstream references and CSV/TOML provenance/checks;
shared Python installers/runtime-only restores are support work, counted separately.

## Current phase and retained evidence

[REVIEW's Milestone plan](REVIEW.md#milestone-plan-2026-10-29) sets the active phase;
Phase 0 exits on two consecutive green scheduled full-corpus runs.

Schema15003 source622a573e / persisted layout7110b4dec014d09b fingerprints ordered
State fields/types and complete opaque/manual encodings. Source/native/ROM/input
qualification remains separate; matching layouts never permit header restamps.
Strict library/DLL/all147 registered checks/frontend/format pass; fresh initialized
captures and14 checkpoint rows pass; four fresh-only rejection checks run0 frames.
Both encrypted packs and complete selected restores are verified.
The first two green scheduled full-corpus runs remain pending; Phase 0 stays open.
Read [current acceptance](REVIEW.md#current-publication-and-acceptance) before
changing a retained owner. Preserve original source/DLL/native/input/loader pins,
unexecuted cases and exclusions in the original owned qualification receipts.
The [147 record](https://github.com/JimLundin/sm-decomp/blob/2c2928e725dc30cf6e1df32bfd5070c805bda3ef/docs/PLAN.md#current-work-and-limits),
[146 record](https://github.com/JimLundin/sm-decomp/blob/0ec67ed08e303adc0078c84ee54cb961df3563ba/docs/PLAN.md#current-work-and-limits),
[DSP93/earlier cohorts](https://github.com/JimLundin/sm-decomp/blob/170bc5351c64f0846bbcb7b4714064a7230db153/docs/PLAN.md#current-work-and-limits),
[SOURCE15–38](https://github.com/JimLundin/sm-decomp/blob/4c302a50/docs/PLAN.md#qualified-history)
and [M1a audit](M1A-REVIEW.md) preserve historical proofs and scoped contracts.
Reach the applicable frozen record when touching its owner; history assigns no
work and authorizes no repeated accepted campaign.

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
- Reuse the existing runner; no new production probes/observers/verify cfg blocks,
  fixture modules/process frameworks/evidence stores/registries. Improve the verifier itself.
  REVIEW permits a constructed check only for behavior no movie reaches: under100 lines,
  with the reason in its first line. Otherwise add named missing behavior/order/failure rows.
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
- Small commits, one coherent idea. If an item stalls over an hour, record its blocker
  and findings, then keep useful independent work supplied.

## Fast loop

1. cargo fmt --check; cargo check --no-default-features --lib with warnings denied;
   one DLL build: python tools/verify_dll.py --build-only. Cargo enforces
   warnings='deny' in sm/Cargo.toml for every sm target while preserving dependency
   flags. Direct rustc uses explicit -D warnings, including detached/mutation builds.
   Fix diagnostics; blanket suppression and historical warning-bearing builds do
   not qualify this strict profile. Denied work never passes.
2. verify_dll.py --corpus movies/segments.toml --segment for every affected named row,
   plus door-loading-lag-nmi,tas-ceres-four-doors,area-crateria,item-morph-ball; zero failures.
   No reached row: A adds one; B moves unchanged and records uncovered: <function> in qualification.
3. Nothing extra routinely: no mutation/independent controls/frontend metadata, optimized
   rebuilds, clean-source reruns or full replays. Once before deleting a bank, scoped
   cargo-mutants <=30min; full corpus runs on main AFTER deletion, also at review waypoints.
   Windows uses AST mutants through existing DLL rows; Cargo/nextest cannot run here.
   Stable GNU coverage lacks profiler_builtins (E0463); LLVM tools alone do not fix it.
4. Every comparison failure gets a checkpoint regression row with its fix; never weaken
   comparisons/expectations. Retry a flaky step once; a second failure is real.

For each sequential phase, collect the prior original handle, confirm terminal
descendants and release its budget. Freshly sample RAM/CPU/paging/disk and all
remaining reservations; notify root and launch the already prepared admitted
successor without a prose ACK. Wake completed owners with followup_task;
send_message alone does not activate them. Send actual START and frozen pins
before warning inventories/report prose. Prepare executable commands, environment,
pinned I/O and next source/review/build/replay recipients before upstream READY.
Keep bounded Windows core source/review/build/replay owners supplied with disjoint
files/interfaces and exact root grants; Linux stays stopped. The default source
handoff is one lossless canonical patch/inventory against actual main under the
granted 64 MiB allowance. Transfer unchanged accepted proofs by exact hashes.
After each qualified cycle, compare actual last-hour landed deltas and source/
review/build/capture/replay/publication gaps; correct the measured bottleneck.
Partition independent groups/use spare CPU; preserve shared prefixes and all identity
guards. Run ready original-seed subsets during serial capture dependencies, then complete
checkpoint rows without repeating accepted work absent a source change/new concern.
Changed layout identifiers or wire formats require genuine initialized machine/payload/
continuation checks and fresh captures; preserve round-trip, extra-byte and input-prefix
rejections. Use genuine bytes without header restamps; source length is not payload size.
Cross-recipe reuse requires SHA-bound identical canonical input prefixes; keep source,
ROM/native/full-input/loader guards and save matching manifest starts during setup.
Use [movies/README](../movies/README.md) for commands; inspect masks/hardware/audio/persistence when touched.

The primary publishes qualified small commits to main and fixes known CI failures.
Observe the actual CI workflow and commit; continue independent implementation
while hosted checks run. Free runners only; Windows full corpus runs every 3h,
after bank deletion and at review waypoints, at most 6 parallel shards. Checkpoint
rows have 60-minute replay/65-minute job limits; timeout is failure, not coverage. Routine
pushes run CI without a separate no-op replay run. Manual row IDs are space-separated;
unknown IDs fail. Keep seed/lag/source guards; Linux development remains stopped.

## Toolchain, resources and completion
Edition2024, stable1.99.1 (58cf8f9c3 2026-10-13), x86_64-pc-windows-gnu.
Nightly2026-10-22 is ASR1121-blocked; do not alter security/spoof hosted variables.
Standalone tests/denied compiler and build scripts remain restricted; use permitted
DLL/ctypes. Ordinary frontend linking passes with the exact dependency-only seed.
Bundled verifier cfg still reaches denied khronos_api scripts; use the separate normal
frontend and existing direct DLL with unchanged source/native/cache guards.
Keep crate-root unsafe-op enforcement; paths in [ITERATION](ITERATION.md#windows-shell-setup).

Use fixed interfaces and one writer; isolate outputs when useful. Frozen formatted
source may compile during final review; DLL/publication bind the reviewed final pin.
Never edit checked source or rebuild loaded DLLs. Budget each workflow's launcher/
descendants, remaining physical growth and private commitment once; confirmed
terminal evidence releases the allowance. Start compilers above 3 GiB available
and leave >=2 GiB after all remaining growth. Current sequential ceilings are
2 GiB library, 3.25 GiB direct DLL and 2.5 GiB frontend; the primary's exact
workload/profile grant and fresh shared ledger determine admission.
Below2GiB/resource errors stop newest producer, not every lightweight reader.
Measure CPU/RAM/paging/disk/useful throughput. Raise concurrency only when budgets
fit and the next useful batch improves measured throughput; -j1 permits parallel codegen.
Streaming TAS imports reserve200MiB (measured90.5MiB), old whole-log imports3GiB.

Check disk/storage runway at session start, before batches and during artifact jobs.
Keep >=50GiB after budgets; cleanup below75GiB to restore100GiB. Below50GiB hold new
builds/copies/captures; bounded running jobs may finish and existing-artifact/source work continues.
Budget copies/temporaries/retained outputs/captures:300MiB fresh workflow or200MiB
remaining for copied direct-only work, ABOVE reserve. Seed dependencies, not full targets;
reuse checked builds and keep incremental caches enabled, bound to toolchain/profile
and source/native/output identities. Preserve others' caches and compare warnings
in the same profile.
Assign cleanup ownership; preserve source/results/reusable checkpoints/loaded DLLs.
Resolve paths/verify unneeded artifacts first; explicit compiler-file allowlists, nonrecursive
non-force deletion preferred. Never delete another worker's artifacts/locks/directories.
NTFS compression needs unchanged hashes. Report policy rejection immediately; never retry
through another tool or use tiny compression gains to continue below the reserve.

Done: Fast loop passes; measure tools/progress.sh and retain explained deltas and
exact qualification evidence. Fetch/reconcile/recheck HEAD/index, commit the coherent
change with a concise message and immediately push main.
Update active PLAN beside code and the sole #6 report daily under REVIEW; no README
progress image or extra readiness gates. Bounded delegated core work continues during CI.
Pre-transpile .scratch/gates/pointers/scopes are inactive in archive/pre-transpile-2026-10-17.
