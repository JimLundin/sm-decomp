# External review

Living third-party review, commissioned by the user. A reviewer checks in
periodically, updates this file and, when needed, the Review directives
section of [PLAN.md](PLAN.md).
The user directs implementation agents to adjust to it. Git history keeps
earlier versions; this file holds only current findings.

## Milestone plan (2026-10-24)

**Active oversight directive (2026-10-27 09:50 CET).** The user assigns this
session independent, proactive oversight alongside the running implementer.
This directive supersedes older work-order and pacing advice below; PLAN's
Fast loop, resource limits and landing rules remain in force. Existing source
owners and the sole Windows integrator retain their assignments.

Measured main `6ad82e25`: production 111,924, legacy 17,353, unsafe-containing
lines 2,712, verification 40,385 (`progress.sh`, independently run). Between
`2adc3ae3` at 02:24 and main at 07:07, production fell 181 (~38/hour), legacy
12 and unsafe-containing lines 2. Reported 44/44 is retained scenario evidence,
not a fresh run or complete-engine proof. The joint WRAM/Execution change is
unlanded; its larger draft savings cannot be added to main's progress.
Mailbox compiler failures show incomplete macro-consumer closure. Resources
were sufficient in the implementer's latest report: failure is source/integration.

Next work, in order:

1. **Close and land the current WRAM/Execution integration.** Current owners
   correct the complete A6/A7/A9 diagnostic groups, including macro definitions,
   affected expansions, import scopes and width semantics. Use the compiler
   result from the full composition to find remaining groups; do not repair
   one emitted error at a time. Preserve byte/failure/effect order and transitive
   unsafe contracts. Freeze the reviewed composition, qualify its actual
   codec with genuine captures and required replay rows, reconcile main and
   land promptly. Keep the existing oracle and identity guards.
2. **Overlap useful disjoint source work with qualification.** Continue the
   already assigned sm_8b and Linux sm_aa family work outside the frozen join.
   Oversight prepares independent source findings and the next complete scope;
   it must not create another shared-file owner or duplicate the donor reviews.
3. **After landing, select the next complete responsibility from actual main.**
   Choose the largest removable unsafe dependency or duplicated family whose
   callers and required rows are understood. State the expected net benefit
   and removal condition before implementation. Several matching functions can
   share one coherent commit and one affected-row campaign. Tiny cleanups are
   useful when they unblock this work; they should not consume the integrator's
   qualification path as the primary output.

Apply review effort to risk. For an unchanged private numerical helper, review
its callers and run PLAN's affected rows plus smokes. Ownership, macros, callbacks,
reset/load and codec changes need explicit source reasoning for those boundaries
and their required execution evidence. Transfer exact unchanged leaf proofs only
when the shared API/dependency relationship still matches. Repeat checks only for
changed source, failed results or a concrete unresolved concern; retain scoped
mutation before bank deletion and full-corpus waypoint policy.

At the next completed qualification cycle, report landed net deltas and time in
source/review/build/capture/replay/landing; use that to choose the next bottleneck
correction. If integration still has no compiler pass after one hour, give the
exact remaining group and recovery action, following PLAN's stall rule. Estimates
must name the next observable result, separately from finished qualification.
Do not invent a new deadline from unsafe line counts: these include attributes
and scopes and can fall without removing raw operations. Keep raw-operation
removal and closed ownership dependencies explicit beside the existing Score.
Bank deletion, safe wrappers and replay passes remain intermediate achievements;
owned Session, independent media and Map Rando acceptance remain the destination.

**Size feasibility, final reviewed ce8b.** Independent `progress.sh` and Linux's
final review agree: production 114,244, legacy 18,459, unsafe-containing 2,637
and verification 41,828. Versus main `6ad82e25`: +2,320 production, +1,106 legacy,
-75 unsafe-containing and +1,443 verification. Source/format/library/DLL pass;
at 10:08 CET all14 genuine captures pass, while full22 restore/final10 rows and
landing remain unfinished. Reaching 100,000 needs 14,244 net production lines
removed. If only banks changed, their replacements could total at most 4,215
lines (22.8%); at 80% replacement, another ~10,552 lines must disappear elsewhere.
Bank moves alone cannot substantiate the existing size promise. After this cut,
select whole-domain duplication in the existing enemy-projectile and actor
modules alongside bank closures; report concrete expected savings before a
campaign. Include future adapters/media cost and remeasure actual main; do not
trade behavioral fidelity or source readability for an arithmetic target.

**Prepared next primitive: fixed DMA-register writes for HDMA allocation.**
After the current join lands, investigate the existing owned DMA writer as the
leaf for `visual_runtime::{initialize_hdma,spawn_hdma,spawn_hdma_last,
spawn_hdma_words}` and their fixed initializers. Allocation only selects slots
0..5; control/target writes therefore use 0x4320/21 through 0x4370/71. The
current generic `WriteRegWord` makes these wrappers unsafe although the joined
backend's DMA-register mutation is safe. This is a scoped opportunity, not an
approved source change or a new claimed reduction.

Preserve the entire register-write protocol: low byte, post-write observer,
high byte, post-write observer; original missing-backend failure point; original
definition read(1), publication and later read(0); u16::MAX on exhausted slots.
Do not directly substitute Game's seed writer: it omits the generic write
observer. Generic bus writes also call the cartridge after register dispatch;
prove that tail inert for these exact bank-0 addresses and all supported mapper
types. Reuse the existing observer, including its ROM mutation/failed-observation
behavior. HDMA interpreter channel bytes can select outside this range and must
retain the general path. Qualify the complete initializer cohort once with its
named Ceres/intro/Morph/boss/X-ray rows plus PLAN smokes. Count actual leaf and
caller removal; keep unrelated bus, HDMA execution, reset and renderer contracts.

**Ready support handoff: fail before compiling an incomplete source inventory.**
The isolated `tools/verify_dll.py` draft in [mailbox6437909267](https://github.com/JimLundin/sm-decomp/issues/2#issuecomment-6437909267)
reuses the loaded-library guard's exact expected-file classification and checks
literal FILES bindings before compiler startup. Actual569 bindings pass; missing,
duplicate, wrong-path and inactive/conditional entries fail before the compiler.
The runtime source/native guards remain unchanged. Cost +42 support Python lines,
+1,904 raw bytes; production/legacy/unsafe/verification Rust all0. Warm preflight
median130ms is measured, not a claimed build-time reduction. Sole integrator may
adopt the lossless exact-parent patch beside the next code change; it must not
alter or delay current ce8b qualification/landing.

**Prepared architecture closure: owned continuation and checkpoint locality.**
This is design ahead of the existing queue, not a second work order. Keep the
accepted Session seam: `open`, `step`, `checkpoint`. A sealed owned Checkpoint
describes a complete committed Tick; validated NewGame/PlayerSave/Scenario starts
serve tools and Map Rando without arbitrary mutable field access. Our gameplay
Checkpoint and the existing full verifier `.smvchk` have distinct purposes: the
latter still needs both PPUs, CPU/DMA/APU/SPC/DSP/native player, recorder, SRAM,
Actor lineage and allocation-equivalence state.

Current `verify/checkpoint.rs::machine` reaches Game's scratch/recorder/tracing
storage, WorkRam, devices and post-load ROM patch behavior. New `Execution` only
borrows a `HostContext`; it does not own the persisted fields. Prepare one complete
storage cohort with its initializers and actual reset/load/Runtime consumers,
remove the original fields, then move private capture/preflight/ordered adoption
to that owner and the existing concrete backend adapters. Preserve each store,
partial-failure publication, fresh selector and allocation identity. Do not
replace the traversal with a participant registry, raw struct dump or mirrored
state. Classify new persistent fields deliberately as encoded, reconstructed
or unnecessary. No first scalar extraction earns removal of whole Game from
codec compatibility; initialization, restore, RAM/ROM and backend dependencies
must close before a contributor can disappear. Prepare the coherent closure
before one codec freeze/capture campaign, rather than recapturing per extraction.

The existing runner already reuses complete DLLs and exact-source prefixes.
Its56-byte checkpoint header lacks capture-source/native/build-profile context;
switching caches to codec-only or scanning opaque old directories would lose a
guard. Future genuinely captured envelopes can own immutable provenance, including
where the current producer's executed interval began. Inherited prefix history
must remain distinct from current execution. Keep configured setup gaps, both
independent Kago14350 approaches, full input/prefix/origin checks and source/native
build guards. No sidecar registry, backfill or restamp. Cross-source development
reuse needs that context and equal validated codec/native/profile; target gameplay
Checkpoints still require the exact gameplay fingerprint, including topology/options.

Removal evidence: the schema and every initialize/capture/preflight/adopt path no
longer call an excluded implementation. Then use the existing full original
Morph1200 continuation with one unrelated private numerical edit: changed checked
build source, unchanged genuinely captured codec, identical end machine/Actor
payload and zero failures. This is a deferred falsifier, not a run or present speed
claim. Rust changes still compile; prepared-content changes can avoid compilation
only after those definitions are actual data. Current process/source owners and
PLAN's Fast loop, deletion and waypoint checks retain their authority.

At each item's start, check the user-requested Windows/Linux
[coordination mailbox](https://github.com/JimLundin/sm-decomp/issues/2) for ownership and handoffs.

Set by the reviewer at the user's request so progress is countable. It replaces
"consolidate exercised subsystems" with one unit of work: **retire a legacy bank**.
Baseline at 5096059: 24/44 checks; `game/sm_*.rs` 23 files, **55,342 lines, 3,160
unsafe lines** (of 116,960 production and 4,965 unsafe total).

**Scoreboard** — last line of every agent commit body, measured locally:
`Score: checks 24/44 · legacy 55342 · production 116960 · unsafe 4965`.
`legacy` = `cat sm/src/game/sm_*.rs | wc -l`. From 2026-10-25 append
`· verification N` (verification Rust lines, as `progress.sh` reports).
Priority of the numbers (from 2026-10-25 16:00): **unsafe** and checks first, then
legacy, then size. Legacy alone can fall by moving code; unsafe cannot.

**Size goal (user aspiration, 2026-10-24).** Lines should fall substantially:
owned code uses native types and data instead of re-expressing 65816 routines.
Reference: snesrev's routine-by-routine C is ~76,500 game-logic lines. Budgets,
total production: ≤110,000 by 10-26 (layering check), ≤100,000 when all banks
are retired, **aspiration ~75,000** for the finished engine (game logic ~60,000,
platform/renderer/audio ~15,000). Each retired bank's replacement code should be
at most ~70% of the bank lines it removes; report the ratio per bank. Do not
compress formatting, delete comments or move code into verification paths to
meet a budget; behaviour and quirks still win over size.

**Verification policy (user direction, 2026-10-24).** Verification means movie
slices turned into focused tests, kept out of production code:

- A test is data: a `movies/segments.toml` row (movie, frame window from a
  checkpoint, per-frame oracle comparison, expectations) run by the one existing
  generic runner. New evidence for a change is a new or named row, not new Rust.
- Every frame-comparison failure becomes a regression row: the shortest window
  from a checkpoint before the first differing frame through a few frames after,
  committed with its fix.
- No new hand-written fixture modules (the `verification/*_cases.rs`,
  `*_checks.rs` style that pokes RAM/ROM words into a constructed world). Delete
  an existing fixture once a segment exercises the same code; that deletion is
  progress. A constructed test is allowed only for behaviour no movie reaches,
  under ~100 lines, with the reason in its first line.
- Production code gets no new probes, observers or `cfg(sm_verify_dll)` blocks
  (589 production lines in 91 files mention verify/probe/observer today); remove
  them as banks retire. After A1.3, lane A tries instrumentation coverage
  (`cargo llvm-cov` on a segment run) to prove which code a segment executed,
  replacing hand-placed probes.
- Verification Rust (38,993 now, 33% of production) must fall from here; target
  under ~15,000 once the movies drive coverage.


**Reset (2026-10-25 12:30, user grant to change goals and process).** Diagnosis
after 18 hours: the code moves, but slowly. (1) Each commit carried a heavy proof
ritual (mutation kills, independent controls, optimized + frontend builds, clean
reruns), so a 50-line bank family took 30-60 minutes on a machine limited to three
compilers; PLAN's **Fast loop** now replaces it. (2) Lane A spent ~10 hours aligning
one 129,712-frame movie end to end and is stuck on timing drift at the first Ceres
audio upload; hand routes give ~1 check/hour and the 18 pending rows are late-game.
(3) Lane B ran one or two agents on two banks while 30,000 of the 51,800 legacy lines
are enemy AI gated behind coverage. The changes below remove those three limits.

**Throughput reset 2 (2026-10-25 16:00, user direction: finish within days, use
hosted CI).** Measured by the reviewer before this change:
- Pace: 36 lane commits in the 24h to 14:00, ~150 legacy lines and ~10 unsafe lines
  each. At that rate 4,600 unsafe lines take weeks. Hand-moving one function per
  commit cannot meet the goal; most unsafe comes from a few access primitives.
- Primitive experiment (compile-only, scripted, ~10 min): make `word!` bounds-checked
  `g_ram` indexing, then strip `unsafe` from every fn the compiler accepts (fixpoint):
  269 fns become safe, unsafe lines 4,603 -> 4,170, plus 224 now-unneeded `unsafe`
  blocks for `cargo fix`. On the oracle: 40 of 40 runnable rows pass (Segments run
  40203144686, branch `ci/safe-word`).
- What keeps the remaining ~1,100 bank fns unsafe (by fn): enemy raw pointers
  (`Get_X(k)` / `(*E).field`) ~450, `ram_ptr!` ~360, only calls to unsafe fns ~280
  (fall once callees are safe), ROM pointers ~110, hardware/IO ~50.
- Hosted oracle: `.github/workflows/segments.yml` runs every route-generated row
  (40 rows, 14 Windows shards) in 32 min with the ROM from secrets; 0 failures on
  b06f9ca (run 40199817511). Standard runners are free on this public repo.
- `ci.yml` had been red on every push since 10-24 11:53 (`--example sm_verify` and
  the detached build), unseen because the Fast loop builds neither.

**Throughput reset 3 (2026-10-25 23:25, user request; supersedes conflicting notes below).**
Measured: from 17:37 to 23:07 main moved legacy -1,215 (~220/h), unsafe -151 (~27/h),
production +200. The last hour *wrote* ~4,400 legacy lines of B work (five bank files), but
no B commit has landed since 18:26. **Writing is no longer the bottleneck; landing is.**
Causes: (1) 45 of 82 rows still replay from boot: route shards take 12-30 min, seeded
shards under 1 min, so a branch run takes 30-45 min at six shards; (2) stacked branches
re-ran the same rows and were cancelled; (3) shared files (`lib.rs`, `verify_source.rs`,
`sm_82.rs`) make auto-land rebases fail. Coverage is done (44/44, 6c366e5), which unlocks
every boss bank (~20k lines), and those banks hardly touch each other. Changes:
- **Landing (user, 23:25):** Windows agents land on local green (PLAN "Landing"). Hosted
  main is the backstop; a red main is fixed or reverted before anyone else lands.
- **Lane A -> CI speed (A2), then bank work.** A2.1: move all 45 boot-replay rows onto
  seeds (several per commit) and delete their recipes from `tools/movie_inputs.py`
  (1,796 lines). A2.2: pack seed rows into a few shards (~40 s setup each now, for 3 s of
  rows). Target: a branch run in <=10 min by 10-26 06:00. Then A takes the B4 queue.
- **Lane M: no more in-place M2 codemods on bank files.** Converting a bank in place and
  then moving it again is double work, and a retirement removes more raw dereferences
  (sm_b2: -291) than a codemod does (sm_a8: -27 unsafe for +149 lines). The cloud M agent
  lands `ci/m2-sm-a8` and then works on unsafe outside the banks: M3 `ram_ptr!`, M4 ROM
  pointers, `main.rs`/`rtl.rs`/platform (~400 lines). The Windows M agent becomes B4.
- **Lane B: six queues, one bank file each.** Boss and enemy banks are open now (rows pass).
  B1 `sm_80` -> `sm_82`; B2 `sm_8b`; B3 `sm_a2` -> `sm_a3`; B4 (Windows M agent) `sm_a8`
  after `ci/m2-sm-a8` lands -> `sm_b3`; B5/B6 (cloud workers, hosted mode; scheduled tasks "sm-decomp cloud worker B5/B6",
  hourly at :05 and :20 from 00:05 on 10-26; boss rows are seeded, so they need not wait for A2)
  `sm_a9` -> `sm_aa` and `sm_a6` -> `sm_a7`; whoever frees up first takes `sm_a5`, `sm_9b`,
  then `sm_a4`. Retire into owned safe modules (sm_b2 style). Shared registry edits
  (`lib.rs`, `verify_source.rs`) stay one line each so rebases merge cleanly.
- **Size:** moves run ~95%. Moving code into safe modules comes first for now (unsafe,
  then legacy); the <=80% target applies again to `sm_80`, `sm_82`, `sm_8b`, the non-enemy
  banks, where shared owners already exist.

**Landed (10-26 01:00, reviewer).** The 00:00 order worked: 27 B commits reached main in the
hour and **five bank files are gone** (`sm_88`, `sm_a0`, `sm_81`, `sm_8f`, `sm_b2`); legacy
47,590 -> 40,331. Main green (runs 100-101); full corpus run 95 in progress. Next, in order:
1. **Unsafe now moves instead of falling.** Bank-file unsafe fell 2,372 -> 1,862, but total only
   4,188 -> 3,905: +227 unsafe lines landed in owned modules (`map_menu.rs` alone has 39, its
   `unsafe fn advance/prepare/fade_palette` copied from `sm_81`). From now, a retirement may not
   add `unsafe` to a non-bank file: convert the moved code to checked access (`word!`,
   `ActorAccess`, `slot`) in the same commit, or leave that fn in the bank. Only hardware/IO
   register fns may move unsafe, into the existing platform owner, named in the body. New owned
   modules start `#![forbid(unsafe_code)]` (B5/B6 already do).
2. **Production is the lagging waypoint** (114,607 vs <=114,000 at 12:00). Moves ran 82-97%
   (`sm_81` 97%, `sm_b2` 96%, `sm_8f` 93%); `sm_a2` Goofball (67%) and B5 (85%) show it can be
   done. B6's 183% (bbefa6a, +54 production) is not acceptable: B6's next commit must lower
   production. Every B commit body states its ratio; above 90% needs the one-line reason.
3. **Lane A:** 55/82 seeded, but `tools/movie_inputs.py` is still 1,766 lines: seeding rows
   without deleting their recipes is half of A2. The next A commit deletes every recipe no
   unseeded row uses (name the line count), then seeds the remaining 27 rows, 6+ per commit.
4. **Queues now:** B1 `sm_80` (714 left: delete it before more `sm_82`), B2 `sm_8b` (3,420),
   B3 `sm_a2` -> `sm_a3`, B4 (Windows M agent) `sm_a8` families -> `sm_b3` (M2 sm_a8 landed in
   090d163; no more `M:` codemods on banks), B5 `sm_a9`, B6 `sm_a6`. Cloud M: M3 (`ram_ptr!`)
   outside banks on `ci/m3-ram-bytes`; delete the obsolete `ci/m2-sm-a8`.

**Lane M — mechanical safety (one agent, new: start a fourth agent if the machine
allows, else it takes the B3 slot; M's local work is `cargo check` only).**
Unit of work: one primitive made safe across the whole crate by a script, then the
fixpoint strip, in one commit. Push it to a `ci/<topic>` branch first; land on main
when Segments is green there (rebase and rerun the script, never hand-merge a
codemod). Body: the script (or its path), unsafe before/after, the Segments run.
- **M1** safe `word!` + fixpoint + `cargo fix` for unused `unsafe`. Prototype green:
  `ci/safe-word` (42c59cb on b06f9ca) passed all 40 runnable rows on Segments run
  40203144686. Land it: rebase onto main, apply the `ram_map.rs` change, rerun
  `tools/codemod/strip_unsafe.py` (on that branch), `cargo fix`, push to `ci/m1`,
  land when green. Target unsafe <= 4,000.
- **M1 revised (16:55 check-in; lane M stalled two check-ins, 0 commits, no `ci/m1`).**
  B2's finding (PLAN item 2) is right: 42c59cb's whole-RAM `&mut` can alias live
  enemy `*mut` pointers, and its strip made raw-pointer `Load24` safe. Do not land it.
  Sound first deliverable, **M1a, due 18:00 as a `ci/m1a` push**: the fixpoint strip
  alone, `word!` unchanged, stripping `unsafe` only from fns whose signature has no raw
  pointer and whose body then compiles with no `unsafe {}` block; then `cargo fix` for
  unused blocks. Then M2 (removes retained enemy pointers), then safe `word!` (M1b).
  Owner: the B3 agent (sm_a4 quiet this hour) until a fourth agent starts.
- **M1 done (17:55 check-in), sound form:** d0fffa6 made `word!` checked caller-side
  byte access (no whole-RAM borrow, so no alias with live enemy pointers) and cleared 123
  blocks; all 59 rows passed locally. Lane M then hand-audited ~10 fns per commit
  (62e33d9, 4a515c9) and started M2 (92ac60e Ripper). **Next M commit: the scripted
  fixpoint strip (M1a)** now that `word!` is checked: strip `unsafe` from every fn with
  no raw pointer in its signature whose body compiles without an `unsafe {}` block,
  iterate to a fixpoint, `cargo fix`; one commit, script path in the body. Target
  unsafe <= 3,900 (10-26 12:00) from that commit alone; hand audits only after it.
- **M2** enemy records by slot: replace `Get_X(k) -> *mut` and `(*E).field` with a
  safe slot accessor that re-borrows `game.g_ram` per access; codemod per bank,
  common enemies first (`sm_a2`, `sm_a3`, `sm_a8`, `sm_b3`), then bosses; fixpoint
  after each. Report unsafe per bank.
- **M3** `ram_ptr!` -> checked byte/word/slice views. **M4** ROM pointers ->
  `&'static` slices or typed table reads; hardware register fns safe.
After M4, re-measure and set the next primitive from what still holds unsafe.

**Lane C — CI as the verification pipeline (user decisions 2026-10-25 18:05, live).**
Why: at ~760 legacy lines/hour, 48,715 lines is ~64 agent-hours, about a week. The one
Windows machine is the limit, and it only matters because rows can run nowhere else:
the code already compiles on Linux (reviewer ran `cargo check` in a cloud session: 302
warnings, same as Windows). The user approved four changes: lift the local-only rule,
encrypted seeds in CI, cloud agents with auto-land, a Linux verifier spike. Order:
- **C0b (lane A, next commit; unblocked by the lifted CI rule):** gate the two mutation
  steps in `ci.yml` (and the ROM-prep/report steps) behind `workflow_dispatch`; scripts
  stay for local capture callers. `ci.yml` green.
- **C3 status (18:52):** `ci/c3-pipeline` (8520ac1, based on main 22013c1) ran its rows
  green in Segments run 40217351891; `land` refused by design (workflow edits cannot be
  pushed with `GITHUB_TOKEN`). **The C agent fast-forwards main to it by hand now**, then
  rebases `ci/l1-linux`. Workflow-only commits after C3 land by hand the same way.
- **C0b done (0fd90cf); `ci.yml` green on main since; the 18:00 CI miss is corrected. C2
  landed early (8e2e9a0).** Lane A's next code: Mother Brain phase seeds (A1.3 (1)).
- **C1 (done):** `segments.yml` on main and `ci/**`; 3 green runs on main, ~45 min each.
- **C2 (lane A, due 10-26 06:00):** seeds and lag logs in CI. Pack `target/seeds/` plus
  lag logs, encrypt (`openssl enc -aes-256-cbc -pbkdf2`) with a key derived from the
  ROM itself (`sha256("sm-decomp-seeds-v1" + ROM bytes)`): no new secret, since CI
  already decrypts the ROM and only ROM holders can read the seeds. Upload as a release asset on a `ci-data` tag (ciphertext only),
  decrypt on the runner. `segments.yml` then runs seed rows, one shard per seed: no
  shard replays from boot. Target: the whole corpus under 15 min.
- **C3 (first cloud agent, due 10-26 06:00):** (a) on `ci/**`, `segments.yml` runs only
  the head commit's `Rows:` rows plus the smoke set (all rows if none named); (b) an
  auto-land job: on green, rebase onto main and fast-forward main; conflict = fail and
  leave the branch. Main keeps the full corpus, with only the newest push queued.
  Then every lane switches to PLAN's hosted mode.
- **L1 (same cloud agent after C3, ~3h spike):** build the verifier as a Linux shared
  library (`verify_dll.py` loads it with `ctypes.CDLL`; the oracle core is portable C).
  Go: the smoke set passes on `ubuntu-24.04` with byte-identical results to Windows.
  Then shards move to Linux (faster, more runners). No-go: write down the blocker.
- **Runner budget:** the free plan allows ~20 concurrent jobs; one full run uses 14. If
  branch runs queue over 15 min after C3, the reviewer asks the user about a paid plan
  or larger runners before adding agents.
- **Runner budget (user decision 20:05: no paid plan, no larger runners; free tier only).**
  Free public-repo runners cost nothing but cap at ~20 concurrent jobs. At 19:52 every
  land dispatched the full corpus (33 shards) on main, so main alone held the pool and
  branch runs queued; a TASVideos 503 also failed `ci/windows-file-menu` before any frame.
  C agent's next commit (workflow, hand-landed), in order: (1) hand-land
  `ci/movie-download-retry` (rows green, land refused only as a workflow edit); (2) the
  land job stops dispatching the full corpus; main runs it on a schedule every 3h, on
  bank-file deletion and at waypoints, matrix `max-parallel: 6`; (3) the plan job runs the
  union of `Rows:` over `origin/main..HEAD`; (4) cache downloaded movies (`actions/cache`
  keyed by hash). Agents: push only the tip of a stack, one live branch each; a bank
  retirement's full run waits until no other full run is queued. No new agents until
  branch runs start within 10 min.

**Landing jam (20:52).** Branch work is no longer verified faster than it is written: at
20:52, 7 `ci/windows-*` branches plus M1a and Spore Spawn were unlanded, both bank
deletions among them. PLAN's landing jam rule applies: one live rebased branch per agent,
retirement branches run their rows (not the full corpus), main's full run follows each
deletion. Land order: M1a, Spore Spawn, `sm_88` deletion, `sm_a0` deletion, then sm_81.
**B landing (22:52, updated 23:25):** five bank files (`sm_88`, `sm_a0`, `sm_81`, `sm_8f`,
`sm_b2`) sit on `ci/b-integrate`, `ci/b1-retirements` and `ci/b3-retirements`. Under reset 3, B1
lands b-integrate + b1 on local green now (rebase, run the union of their `Rows:`, push to
main); B3 then rebases b3 and does the same. Delete the `ci/b*` branches after landing.
**Ratios:** `sm_8f` (94%) and `sm_b2` (96%) moved nearly line for line into owned modules: the
new code is safe and well named, so accept them, but `sm_80`/`sm_82` must aim <=80%.
**Lane A (22:52): credits flipped, 43/44** (01b3036, 13 rows, 51,276 frames). Left: Golden
Torizo: add the three PPU fields (PLAN item 1) to the RAM-to-Session converter, then the row.
After it, A moves authored rows onto seeds and deletes recipes from `tools/movie_inputs.py`.
**M2 `sm_a4` (`ci/m2-sm-a4`):** -215 raw dereferences, but production +121, legacy +43, unsafe
-5. `slot!` is a one-time cost; later M2 banks must not grow production.
**M2 production (22:52, not followed):** `ci/m2-sm-a8` (pushed 22:05, after the 21:52 rule)
is legacy +126, production +149 for unsafe -27; one `slot!` per access spells every field
access out. Land it (unsafe first), but the next M2 commit changes `enemy_slots.py` to bind
one typed slot view per fn (`let e = slot_view!(k)`) so a bank's lines do not grow; rerun on
`sm_a4`/`sm_a8` in the same commit. M2 commits that grow production need their reason.

**Cloud agents (user approved 18:05).** Linux sessions without the ROM, hosted mode
only, one lane item each, own `ci/<lane>-<topic>` branches. Start with two: **C3+L1**
(above), then **lane M** (M1a fixpoint strip, then M2 per enemy bank; compile-checked
codemods suit agents that cannot run rows locally). Add enemy-bank B agents once C2+C3
run seeded rows in CI. Windows agents keep A, B1, B2, B3. **Launched 18:35 CET** as one-off cloud sessions: `C:` agent (C3 then L1) and
`M:` cloud agent (M1a fixpoint, then M2 on banks the Windows M agent is not touching).
The Windows lane-M agent keeps its M2 hand moves; it does not run M1a.

**Lane A — coverage (one agent): room-seeded TAS segments (A1).** Stop aligning the
whole movie. Restart alignment at every room instead: the TAS emulator gives the true
state at each room entry, so drift never accumulates past one room. Both the oracle
and the native branch start each frame from the same state, so an imperfect seed
can only lose route fidelity, never fake a pass.
- **A1.1 Dump (≤2h).** Extend `tools/lsnes_lag_log.lua`: at the first gameplay
  frame after each room change ($079B changes, $0998 back to gameplay) write WRAM,
  SRAM, VRAM, CGRAM, OAM and APU RAM (lsnes memory-region reads) plus the movie index to
  `target/seeds/` (local, ROM-derived, never committed; hashes only). Run on any% and
  100%. Commit the Lua and the room/index list it prints.
- **A1.2 Seed (≤3h).** Add `--seed PATH` to the verifier (verification side, not
  production): boot as now, load the seed's memories, rebuild owned state from RAM
  through the existing codecs/checkpoint loader, then run that room's inputs from the
  existing lag-aware importer. **Go:** one seed runs ≥600 frames with 0 oracle
  failures and reaches the same next room as the capture. **No-go:** name the state
  that could not be derived from RAM and fix that converter: RAM-to-Session loading
  is on the long-term path (save states, Map Rando), not throwaway.
- **A1.3 Flip rows.** `segments.toml` gains `seed = "<movie>:<room index>"` in place
  of an authored prefix. Point the 18 pending rows (bosses, late items, Wrecked Ship,
  Maridia, Tourian, credits) at 100%/any% seeds, several per commit, each named. Then
  move existing authored rows onto seeds and delete their recipes from
  `tools/movie_inputs.py` (credit: support code falls).
- **A1.1 done for any%** (9290c2d: 197 room seeds, hashes only in
  `movies/room-seeds-any.csv`); the 100% capture rides along with A1.2. Crocomire's
  hand route is done (75fa0b1). No more hand routes: lane A's next code is A1.2
  `--seed` (first deliverable by 15:52: load one seed and report the first frame
  that diverges, even if no-go). A0 stays parked at
  10,555 with its finding in PLAN item 1; A1 does not need it.
- **A1.2 go reported, not landed (14:52 check-in).** PLAN item 1 (written in 4943d38)
  says a seeded trial passed 600 frames and reached the captured next room, with
  "integration after the queued bank changes", and the 100% capture (331 seeds) is
  done; none of it is committed. That is held work, a stall. Land it now as its own
  `A:` commit: `--seed`, the 100% index CSV and one `seed =` row naming the room and
  frames. It is verification-side, so it waits for no bank queue. The lane-A agent
  does no bank work until it lands. Then A1.3: flip one pending row per commit,
  starting with the cheapest, to reach ≥28/44 by 18:00.

- **A1.3 status (17:55 check-in): 40/44.** Gravity, Grapple (100% seeds), Botwoon (low%
  seed 122) flipped this hour; MB purple-breath regression added. Pending: Spore Spawn
  (absent from any%/100%), Golden Torizo (skipped in 100%), Mother Brain (100% 10,000-frame
  trial: 0 oracle differences, but misses the capture's defeat), credits. Next, in order:
  (1) Mother Brain: 0 differences means the seed drifts from the capture, not a native
  bug; capture extra seeds inside the MB room at each phase change and chain rows from
  them; credits follows from the last MB seed. (2) Spore Spawn and Golden Torizo: index
  one published movie that fights each (as low% did for Botwoon) and seed from it.
  No hand routes.
- **A1.3 status (20:52): 41/44** (f2a7264 Mother Brain fresh defeat from a phase seed);
  Spore Spawn 42/44 on `ci/windows-spore-spawn`. Left: Golden Torizo, credits (chain from
  the MB phase seeds), then move authored rows onto seeds and delete their recipes.

**Lane B — bank retirement (up to three agents, one bank each).** Unchanged method:
for each function, delete it if unreachable or duplicated, otherwise move it onto safe
Session state with native types (enums/fn values, named views, range tests), no new
Host/effects/adapter layer; a bank is done when its file is deleted, with the full
corpus and scoped mutants run once before deletion. Report the replacement ratio per
commit; aim ≤70% (bd76e2a ran 154%: production must fall with each move; a
commit that grows production needs a one-line reason no smaller form exists). Each commit also deletes what the move makes dead (verify/probe
hooks: 607 lines/94 files, duplicated owned paths). Queues:
- **B1:** `sm_90` (finish) → `sm_81` → `sm_80` → `sm_82`.
- **B2:** `sm_88` → `sm_8b`.
- **B3 (new):** finish `sm_89` (48 lines left, the 18:00 deletion) before more `sm_a0`;
  small banks `sm_89` (182) → `sm_a0` (458) → `sm_8f` (421) → `sm_b2`
  (880), then enemy families already met in a passing row (name the row and the enemy;
  e.g. Ceres/Crateria/Brinstar/Norfair common enemies in `sm_a2`, `sm_a3`, `sm_a8`,
  `sm_b3`). `sm_a4` Crocomire (1,630) is unlocked by `boss-crocomire` and follows
  `sm_89` in B3. Other boss banks wait for their A1.3 row: `sm_a5` Draygon,
  `sm_a6` Ridley, `sm_a7` Kraid/Phantoon, `sm_aa` Torizos, `sm_a9` Mother Brain,
  `sm_9b` Grapple.
If only two lane-B agents run, B3 waits; never two agents on one bank.
**18:52: B3 moves to `sm_a0` now** (330 lines, the 10-26 12:00 deletion, unowned): delete
it, then `sm_8f`. `sm_a4` grew again (17f04a1, 153%) after the 17:55 directive; converting
its raw getters to checked actors is M2 work, so it goes to lane M (cloud codemod), not B3.
Every B commit must now lower **production**, not just legacy: 10-26 12:00 needs -1,145 and
the last hour gave -32. Delete dead hooks (607/93) with each move.
PLAN.md is 241 lines (limit ~200): the next B1 commit trims work-order item 2 to <=15 lines.
**17:55:** `sm_a4` ran 108% and 146% this hour (legacy +68): checked-Actor closures grow
the bank. Next `sm_a4` commits must lower legacy (delete or move out whole families);
a growing commit needs the one-line reason. `sm_88` is at 442 lines: finish and delete
it before the 10-26 18:00 waypoint.
Commit unit (from 16:00): a whole function family or small bank, several hundred lines
is fine; run locally only the rows the change reaches, Segments on main runs the rest.
After each M step lands, rebase onto it before continuing; do not re-add `unsafe`.

History: A0.1-A0.2 done (7867e91, cec060e); A0.3 aligned 9,266 steps, then an
elapsed-NMI experiment reached 10,555 (parked 11:52). `sm_94` deleted (ca42bc5, ratio
~75%), `sm_91` deleted (52ba836, ~79%).

**Waypoints** (Europe/Stockholm; reset 2026-10-25 12:30; a miss gets a diagnosis and one corrective step):

| By | Checks | Unsafe lines | Banks deleted (cumulative) | Legacy | Production | Result |
| --- | ---: | ---: | --- | ---: | ---: | --- |
| 10-25 06:00 | A0.4 any% to credits | | sm_94 | <=54,600 | <=116,500 | **MISS** |
| 10-25 18:00 | >=28/44; A1.2 go/no-go recorded | CI green; M1 on a branch | + sm_91, sm_90, sm_89 | <=50,300 | <=115,300 | **HIT except CI**: 40/44, A1.2 go, M1 landed (d0fffa6), banks deleted, legacy 48,715, production 115,177; `ci.yml` red until 0fd90cf (19:18) |
| 10-26 12:00 | >=42/44 (**44/44 met 23:07**, 6c366e5); branch run <=10 min (A2) | <=3,900 (M1 landed) | + sm_a0 | <=47,000 | <=114,000 | |
| 10-26 18:00 | 44/44 (raised 17:55) | <=3,300 (M2 common enemies) | + sm_88, sm_8f, sm_81 | <=44,000 | <=112,500 | |
| 10-27 18:00 | 44/44 | <=2,300 (M2 bosses, M3) | + sm_80, sm_b2, first boss bank | <=36,000 | <=109,000 | |
| 10-28 18:00 | 44/44 + 100% seeds | <=1,200 (M4) | + sm_82, sm_8b | <=28,000 | <=105,000 | |

Unsafe targets after M1 are estimates from the fn counts above; the reviewer
re-baselines them after M2's first bank.

The previous 18:00 and 10-26 production lines (≤114,000, ≤110,000) assumed a 70%
ratio and missed three hours running at ~80%; the reset lines assume 75% plus hook
deletion. End-state budgets in the size goal stand (≤100,000 with all banks retired,
~75,000 aspiration).

**18:00 CI miss (recorded 17:55 CET):** `ci.yml` is red only in its two mutation steps
(`player_detached/mutations.py`, `room_mutations.py`; run 40211656694). C0b was not done:
PLAN's "Local validation" note records a direct user instruction to the agents not to use
or observe CI, and the agents' audit (PLAN item 2) keeps the drivers for local capture
callers. The user lifted the local-only rule at 18:05. Corrective step: C0b, the next lane-A
commit, gates those steps behind `workflow_dispatch`, keeping the scripts.

**06:00 miss (recorded 05:52 CET):** no agent committed 23:48-05:50; the plan was not
the blocker. Corrective step done: agents restarted ~05:50; sm_94 deleted 06:51 CET.

Progress picture: `tools/treemap.py`, auto-published; no other dashboards.

**Progress feed (user request, 2026-10-25).** Every push to main posts the measured
line deltas, lane, ratio and flags to issue #1 (`progress-feed.yml`). It reads the
`Score:` line for checks, so keep that line exact; agents do not maintain the feed.

## Check-ins

- **2026-10-26 01:00 CET (e3e5237), no stall; B landing cleared:** agent commits since c1be617:
  A 4 (seeds 41 -> 55/82, shard grouping), M 3 (sm_a8 M2 landed, fixpoint rerun, harness fix),
  B 27: B1 10 (`sm_88`, `sm_a0`, `sm_81` deleted; `sm_80` -1,791), B2 12 (`sm_8b` -1,508), B3 3
  (`sm_8f`, `sm_b2` deleted; `sm_a2` Goofball), B5 1, B6 1 (both auto-landed, ~5 min runs). All
  `Score:`, 0 docs-only. Score `44/44 · 40331 · 114607 · 3905 · 39514` (last 44 · 47590 · 115354 ·
  4188 · 39463); warnings 255 unique (491 with duplicates); hooks 607/95. 10-26 12:00: legacy and
  banks **ahead** (18:00 targets met), unsafe on (5 over), production **behind** (607 over),
  recipes behind (1,766 unchanged). Verification +51, non-bank unsafe +227: see Landed (01:00).
- **2026-10-26 00:00 CET (c1be617), B landing stall, 3rd of last 4 check-ins:** agent
  commits since 853aa47: A 4 (6c366e5 Golden Torizo quirk fix + 2 regression rows; 3 A2 seed
  rows), M 0 on main (`ci/m2-sm-a8` rebased 23:44, run queued), **B 0 written, 0 landed**
  (B1-B3 silent since 22:51; last B on main 19:27). 1 user chore (6c00ef1). All `Score:`, 0
  docs-only. Score `44/44 · 47590 · 115354 · 4188 · 39463` (last 43 · 47589 · 115353 · 4188
  · 39401); warnings 296; hooks 607/93. A2: 41/82 seeded (37), recipes 1,766 (1,796), seeded
  branch run 6.5 min. 10-26 12:00: checks met; A2 on pace if batched; legacy, sm_a0, unsafe,
  production **behind** until `ci/b1-retirements` lands (rows green, run 40237511884). Treemap OK.
- **2026-10-25 22:52 CET (853aa47), no stall, B landing pending:** 4 non-review commits on
  main since d83432d: A 1 (01b3036 **credits flipped, 43/44**), M 2 (sm_a4 M2 +121 production;
  common enemies -429 legacy, 110%), 2 user-requested docs cleanups (Jim; not counted). On `ci/`:
  B 9 new code commits (B1 `sm_81` deleted + 4 `sm_80`; B3 `sm_8f`, `sm_b2` deleted; b-integrate
  rebased with `sm_88`, `sm_a0` deletions), M `sm_a8` (+149 production). All `Score:`. Score
  `43/44 · 47589 · 115353 · 4188 · 39401` (last 42 · 47983 · 115197 · 4197 · 39353); warnings
  296; hooks 607/93. 10-26 12:00: checks met; legacy, sm_a0 ahead once b1 lands (b1 tip 43,213);
  unsafe (b1 tip 4,046 vs 3,900) and production (114,812 vs 114,000) behind. Treemap OK.
- **2026-10-25 21:52 CET (6ea29c9), B landing stall, second in a row:** 3 agent commits on
  main since 648afab: A 2 (0086292 **Spore Spawn, 42/44**, regression row; 876fbfb branch rows
  only), M 1 (1225f54 M1a fixpoint landed, -95 unsafe), B 0 (third check-in). Unlanded:
  `ci/b-integrate` (6 B, conflicts with M1a, see Milestone plan), credits (43/44), M2 sm_a4,
  common enemies (-429 legacy, 110%). All `Score:`, 0 docs-only. Score `42/44 · 47983 · 115197
  · 4197 · 39353` (last 41 · 47998 · 115215 · 4292 · 39353); warnings 297; hooks 607/93.
  10-26 12:00: checks met; legacy and sm_a0 on if b-integrate lands; unsafe, production behind.
  Treemap OK (200, 6ea29c9); its "Legacy 62.2k" counts unsafe files too, not Score's legacy.
- **2026-10-25 20:52 CET (648afab), landing stall in B1/B3 (first):** 8 agent commits
  landed since 6bd23aa: A 3 (f2a7264 **Mother Brain defeat, 41/44**; download retry; runner
  budget), M 4 (Rio, Norfair Rio, Flies: -392 legacy), B2 `sm_88` 1; **B1 `sm_81` and B3
  `sm_a0` 0 on main two check-ins running** while `sm_88`/`sm_a0` deletions, sm_81 stacks,
  M1a (-95 unsafe) and Spore Spawn (42/44) wait on `ci/` (runs cancelled, branches 10
  behind). All `Score:`, 0 docs-only. Score `41/44 · 47998 · 115215 · 4292 · 39353` (last
  40 · 48466 · 115145 · 4307 · 39347); warnings 297; hooks 607/93. 10-26 12:00: checks and
  legacy on pace if held work lands; unsafe, sm_a0, production **behind** (production +70:
  b782ca4 grew it 69). Changed: PLAN landing jam rule; M2 must not grow production.
- **2026-10-25 19:52 CET (6bd23aa), no stall, CI queue is the bottleneck:** 6 agent
  commits on main since 3d80523: A 2 (0fd90cf **C0b done, `ci.yml` green**; 8e2e9a0 **C2
  landed early**, 63 runnable rows), C 3 (C3 live, L1 go), M 1 docs-only (M1A-REVIEW, 17%).
  B landed 0, but 10 commits sit on `ci/windows-*`, `ci/m1a-fixpoint` (all <30 min old;
  sm_88 retirement, sm_81 -823, sm_a0, Rio, M1a -95 unsafe) waiting on runners: main's
  build queued 15 min, 33 shards queued. Score unchanged `40/44 · 48466 · 115145 · 4307 ·
  39347` (last 40 · 48466 · 115145 · 4304 · 39336); warnings 301; hooks 607/93. 10-26 12:00:
  legacy on pace if the queue lands; production, unsafe, checks behind. Changed: CI budget (20:05: user rules out paid runners; free-tier plan).
- **2026-10-25 18:52 CET (3d80523), no stall, half hour:** 7 agent commits since 9fff566, all
  18:17-18:25: A 1 (67fb989 checkpoint tooling; no flip, **no C0b, 2nd miss**), M 2 (bc07cde
  Gripper M2; 0924e3b disk tooling), B1 `sm_81` 2, B2 `sm_88` 1, B3 `sm_a4` 1 (+36 legacy,
  153%, against the 17:55 directive); cloud C 3 on `ci/` (C3 rows green, land refused: workflow
  edit). All `Score:`, 0 docs-only. Score `40/44 · 48466 · 115145 · 4304 · 39336` (last 40 ·
  48715 · 115177 · 4327 · 39336); warnings 301; hooks 607/93. 10-26 12:00: legacy on, checks,
  unsafe (M1a not pushed), sm_a0 and production **behind**. Changed: C3 hand-land, B3 to sm_a0.
- **2026-10-25 17:55 CET (9fff566), no stall, best hour so far:** 19 agent commits since
  3ea69a0: A 5 (Gravity, Grapple, Botwoon flipped; MB regression), M 5 (d0fffa6 checked
  `word!`, -124 unsafe; Ripper M2), B1 `sm_81` 3, B2 `sm_88` 3, B3 `sm_a4` 3; all `Score:`,
  0 docs-only. Score `checks 40/44 · legacy 48715 · production 115177 · unsafe 4327 ·
  verification 39336` (last 37 · 49512 · 115340 · 4546 · 39240); warnings 302 (357).
  18:00 hit except `ci.yml` (user decision asked). Changed: M1 done, M1a fixpoint next;
  A1.3 next steps; checks waypoints raised; `sm_a4` growth flagged.
- **2026-10-25 16:55 CET (3ea69a0), stall in lane M (2nd check-in, 0 commits):** 7 agent
  commits since 9b062a7: A 4 (Draygon, Space Jump, MB spawn regression, C0 compile),
  B1 `sm_81` 1, B2 `sm_88` 2 (unsafe -31 then +33: backend contracts restored, sound),
  B3 0, M 0; all `Score:`, 0 docs-only. Score `checks 37/44 · legacy 49512 · production
  115340 · unsafe 4546 · verification 39240` (last 35 · 49698 · 115375 · 4554 · 39232);
  warnings 357. 18:00: checks/legacy/banks met, production 40 over, `ci.yml` red,
  M1 unsound. Changed: M1a (sound strip) to B3 agent; C0b drops mutation steps.
- **2026-10-25 15:55 CET (9b062a7), no stall, short hour (15 min after reset 2):**
  5 agent commits since f2d3a05: A 2 (6f579a3 Ridley, f207b36 Phantoon: checks 33 -> 35),
  B1 `sm_81` 1 (a2088b5), B2 `sm_88` 1 (89%), B3 `sm_a4` 1 (44%); M 0 (lane opened at
  15:40); all labelled with `Score:`, bodies <=8 lines naming rows. Score `checks 35/44 ·
  legacy 49698 · production 115375 · unsafe 4554 · verification 39232` (last 33 · 49901 ·
  115411 · 4567 · 39228); warnings 357 (362). 18:00: checks, legacy, banks met;
  production 75 over (on pace); unsafe/M1 branch and green `ci.yml` still open. No changes.

## Lane watch (2026-10-24 23:52)

- d0d1961 is a sound bank-retirement step (−90 engine, adapter and `CollInfo`
  record deleted, `Definitions` implemented directly on `Game`), but it moved a
  `printf` `unsafe` block into an impl that was `#[forbid(unsafe_code)]`. Route
  ROM-pointer warnings through a safe logger when the bank's last family goes.
- Commit bodies are compressed to the point of code ("no-default376, source1f25").
  Keep ~8 lines but write words: which checks ran, which segments, result.
- 00:55: lane A is executing A0 as written (logger read-only, no frames dropped,
  hashes only, no fixture Rust). Label map/report commits `A:` only beside a lane
  A code or tool commit; a427ba4 alone counted as no lane progress.

## Corpus watch (2026-10-24 22:52)

- Consolidation is clean (no unsafe added, verify cfg kept behind diagnostic
  calls, coverage named per commit: Crateria 800, Ceres 8340–8780, boot 77381+),
  but small: −144 production/hour puts 110,000 ~48 hours out. 98aafd4 is +5
  production for 7 unsafe functions retired; acceptable, stated.
- Verification grew 7× faster than production fell, mostly hand-written fixture
  checks per adoption. Directive now caps verification added at production
  deleted per adoption; checkpoint segments are the preferred evidence.
- Coverage produced no commit this hour. If the Power Bomb route is not a checked
  segment by next check-in, that is a coverage stall for the third time today.

## Progress picture (user request, 2026-10-25 21:30)

The interactive codebase map is retired at the user's request. The one progress view
is `tools/treemap.py`: production Rust by area, banded owned / partly owned / legacy,
with four headline numbers (required checks, legacy, unsafe, production lines) and
their 24-hour change. Pages redraws it on every main push; README embeds it (PROGRESS.md
was removed at the user's request, f2f6178).

## Initial review (2026-10-24, a408c8f)

## Baseline measured at a408c8f

| Measure | Value |
| --- | --- |
| Production Rust in `sm/src` (excluding `verification`/`verify` paths) | 94,285 on 2026-10-21 → **117,986** now |
| Verification Rust | 1,316 on 2026-10-21 → **35,653** now |
| Lines containing `unsafe` in `sm/src` | 8,537 on 2026-10-17 → **5,042** now |
| Files with `forbid(unsafe_code)` | 289 |
| Commits since 2026-10-21 touching only `docs/`, `PROGRESS.md`, `README.md`, `AGENTS.md` | **226 of 412 (55%)** |
| `docs/PLAN.md` | 1,642 lines (mandatory pre-read) |
| Markdown in `docs/` + `PROGRESS.md` | ~15,000 lines, plus 875 KB of HTML reports |
| `cfg(sm_verify_dll)` sites in production code | 183 |
| `.probe("…")` calls in production code | 114 |
| `cargo check --no-default-features --lib` warnings | 2,426 (1,084 unnecessary parentheses, 228 unused variables, 83 unused imports) |
| `#[test]` functions | 39 |
| Exercised normal gameplay for the adopted player work | ~8,577 frames, one gameplay room; any% replay reaches six rooms, no credits |

## What is working

- The per-frame ROM oracle is the project's strongest asset.
- Owned modules such as `game/player_movement/` are good Rust: forbidden
  unsafe, typed state, real enums (`LiquidAtHeight`), no RAM pointers.
- Unsafe lines fell by 41% and the library compiles.

## Findings and directives

### 1. Coverage is the bottleneck (highest priority)

Adoption claims rest on very little exercised gameplay. One room of normal
play cannot establish fidelity for locomotion, collision or transitions, and
targeted mutations only test what the replay already reaches.

Directive: build an oracle checkpoint corpus before further adoption.
- Keep the replay in sync with the TAS (input/lag alignment) as far as it can
  go, and/or capture `.smvchk` checkpoints from the oracle side covering every
  area (Crateria, Brinstar, Norfair, Wrecked Ship, Maridia, Tourian), each
  boss, the main item pickups, doors/elevators, save/load, death, and credits.
- Checkpoints are ROM-derived: generate them locally, never commit them. Commit
  only the recipe (movie, frame ranges, required probes) so any agent can
  regenerate the corpus.
- Done when one command replays every segment and reports per-segment
  pass/fail and rooms visited. Every later adoption must name which segments
  exercise it.

### 2. Production size is growing against the goal

The goal is below 100,000 production lines; it went from 94k to 118k in three
days. Adoptions add Host/Effects/Content layers and adapters while the bank
files stay large (`sm_a9.rs` 5.7k, `sm_8b.rs` 4.9k, `sm_a6.rs` 4.9k).

Directive:
- Every adoption commit reports production lines added and bank/legacy lines
  deleted in the same commit. Net production growth needs a one-line reason.
- Prefer deleting legacy bank code over adding new layers. Do not introduce a
  new Host trait, effects/content split or adapter unless an existing one
  cannot serve.
- Next production target: back under 110,000 before starting a new subsystem.

### 3. Process outweighs product

Over half of recent commits change only reports. `AGENTS.md` already forbids
dashboards and evidence stores, yet PROGRESS.md, the atlas and the
architecture report are maintained by hand on most commits.

Directive:
- Rewrite `docs/PLAN.md` to at most ~200 lines: goal, rules, validation
  strategy, the current item and the next three. Move dated history and
  completed-item narratives to [the historical archive](https://github.com/JimLundin/sm-decomp/blob/01b30366c1b798463c2726db69f31afd4abbdcab/docs/HISTORY.md), which agents need not read.
- `PROGRESS.md` becomes a short table generated by `tools/progress.sh`
  (production lines, verification lines, unsafe lines, warnings, checkpoint
  segments passing). Update it at most once per adopted milestone, in the
  same commit as the milestone.
- Stop refreshing `docs/codebase-map`, `docs/architecture.html` and
  [the former architecture report](https://github.com/JimLundin/sm-decomp/blob/01b30366c1b798463c2726db69f31afd4abbdcab/docs/target-architecture/report.html) per commit. Refresh only on user request.
- Target: under 20% of commits are docs-only.

### 4. Verification hooks have leaked into production code

183 `cfg(sm_verify_dll)` blocks and 114 probe strings sit in gameplay code.
Mutation tests patch source text from Python, so any reformat breaks them.

Directive:
- Route probes through one trait method on the host with a no-op production
  implementation; remove inline `cfg(sm_verify_dll)` from gameplay functions
  where a host method can carry the difference.
- Evaluate `cargo-mutants` (scoped to owned modules) to replace text-patch
  mutation recipes. Keep the Python recipes only where cargo-mutants cannot
  reach, and do not add new ones without trying it first.
- Do not grow the verification tree for each adoption; reuse the checkpoint
  corpus from item 1.

### 5. Hygiene

- One mechanical commit: `cargo fix` and `cargo clippy --fix` for the c2rust
  leftovers (parentheses, unused imports/variables), verified by a full
  replay. Then keep the warning count from rising; record it in progress.
- Explicitly `#[allow(improper_ctypes)]` the eight FFI declarations whose
  layout is already guarded by `offset_of!` asserts (e.g. `SpcPlayer`, `Dsp`),
  with a comment, so real FFI warnings stand out.
- Add plain `#[test]` unit tests for pure owned logic (speed tables, liquid
  physics, pose resolution). They are fast and catch errors before the oracle.

### 6. CI on main

Many agents commit directly to `main`. The 2026-10-24 local validation policy
stays: CI does not gate an individual adoption. But a red `main` is a stop
signal: the next adoption does not start until main is green again or the
failing commit is reverted.

## Order of work

1. Hygiene commit (item 5, first two bullets): small, mechanical, unblocks
   readable warnings.
2. PLAN/PROGRESS reduction (item 3).
3. Checkpoint corpus (item 1).
4. Resume subsystem adoption under the size rule (item 2), using the corpus,
   with probe consolidation (item 4) done inside the subsystems being touched.
