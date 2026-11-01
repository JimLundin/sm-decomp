# Verification sharding investigation, 2026-11-01

The published room chains already use the existing scenario runner and balanced
shards. Keep that subdivision. Reduce the three replay matrices from eight to six
concurrent jobs each as a bounded scheduling experiment, preserving their rows,
comparisons, locks and gates. Complete movies need genuine checkpoint-backed
scenarios and cadence repair before they can use the same scheduling machinery.

## Evidence and scope

Source inspected: `260d11ef6406eb2a4ea053a7c633877f29a0b74e`, the root's actual
`collab-8b` landing checkout. Read-only source digests are in the lane's
`source-pins.csv`; source was not copied wholesale or changed. The successful
PR144 runs below use `49656229aed66d6c78d95b2b2c22a99cd2a0e6ad`.
PR145 was open at `534c5be320c20856e3ef5f710ea35f639579eea3` when sampled.
Its corpus [41318110135](https://github.com/JimLundin/sm-decomp/actions/runs/41318110135)
was incomplete in the snapshot: 46 jobs, 11 completed. It supplies no final timing.

| Successful PR144 run | Jobs / replay jobs | Workflow elapsed | Windows runner minutes | Replay runner minutes | Replay job service range | Created-to-start range / median |
| --- | --- | --- | --- | --- | --- | --- |
| [CI 41310808761](https://github.com/JimLundin/sm-decomp/actions/runs/41310808761) | 4 / 0 | 4.85 min | 10.92 | n/a | n/a | n/a |
| [Corpus 41310834695](https://github.com/JimLundin/sm-decomp/actions/runs/41310834695) | 46 / 44 | 31.28 min | 112.63 | 108.92 | 0.38-8.42 min | 0.03-26.75 / 20.88 min |
| [Route 41310808744](https://github.com/JimLundin/sm-decomp/actions/runs/41310808744) | 19 / 16 | 31.13 min | 128.28 | 123.38 | 6.38-9.08 min | 2.20-18.05 / 8.39 min |
| [Lock 41310808806](https://github.com/JimLundin/sm-decomp/actions/runs/41310808806) | 30 / 27 | 36.75 min | 140.47 | 135.57 | 2.32-7.22 min | 0.42-27.47 / 18.18 min |

Times come from the [GitHub workflow jobs API](https://docs.github.com/en/rest/actions/workflow-jobs#list-jobs-for-a-workflow-run),
`created_at`, `started_at`, `completed_at`, retained as IDs/timestamps in
`jobs.csv` and `runs.csv`. Workflow elapsed is first job creation to last job
completion, rather than GitHub's later run update timestamp. Job service includes
checkout, artifact placement, data restoration, replay and cleanup. Waiting includes
the workflow's own matrix limit and hosted scheduling; this sample cannot assign
the cause of each delay. Peak overlapping Windows job timestamp intervals across
these four runs was 17. That is an observed interval count, not proof of the
account's configured limit or an idle-runner guarantee.

## Existing subdivision

The metadata-only dry run passed `scenario.check`: 693 scenarios, 693 pins,
33 bundles, 634 locked and 41 explicitly unlocked. The corpus planner assigns
all 164 runnable non-chain rows exactly once to 44 shards. Pending rows remain
excluded and reported. Both route plans retain 331 100% rooms / 140,106 selected
ticks and 197 any% rooms / 106,110 selected ticks, in nine and seven jobs.

[`full_route.py`](../.github/scripts/full_route.py) already sorts independent
room scenarios by descending tick count and assigns each to the lightest bin.
Each row starts from its own original seed; a bin need not contain adjacent rooms.
[`plan_corpus.py`](../.github/scripts/plan_corpus.py) retains input/bundle identity,
splits checkpoints individually, chunks other rows, and starts expensive shards
first. [`lock_gate.py`](../.github/scripts/lock_gate.py) reuses both plans and
packs locked corpus shards by their cost. Reuse these paths, without another
framework, registry or evidence store.

Observed route service is reasonably balanced: 6.38-9.08 minutes including setup.
There is no measured need to subdivide those 16 jobs further. The corpus has much
smaller jobs, and three distinct assets currently produce `seed-coverage-0` names;
job names alone therefore cannot reliably attribute timings to individual rows.
The existing cost includes a checkpoint's full cold prefix at an estimated 5x
replay speed. That estimate remains conservative scheduling guidance, not a measured
current-checkpoint speed. Keep full prefix rejection/regeneration and identity guards.

## Capacity and proposed change

At the inspected source, corpus, route and lock matrices each request up to eight
jobs: aggregate configured replay demand 24. CI adds three Windows jobs and one
Ubuntu job, so the stale workflow comments about two eight-job families plus CI
omit the lock family. Coverage is manual and separately requests six corpus plus
six route jobs when enabled; its build and report can also compete. Additional
scheduled or branch runs can overlap. Matrix caps do not reserve account capacity.

Official [Actions limits](https://docs.github.com/en/actions/reference/limits)
list 20 concurrent standard hosted jobs for Free, with higher limits on other
plans, and no separate Windows column. Jim's actual account plan/other-repository
activity was not queried. The repository's six-shard Windows policy comes from
[PLAN](PLAN.md), not from a claimed GitHub Windows cap. Standard hosted runners
are [free in public repositories](https://docs.github.com/en/billing/concepts/product-billing/github-actions);
larger paid runners are outside this proposal.

The accompanying patch changes `max-parallel` to six in `segments.yml`,
`full-route.yml` and `lock-gate.yml`, and corrects planner comments/constants to
separate the unchanged 16-job target from concurrent capacity. Aggregate configured
replay demand becomes 18. CI can still take aggregate demand above 20 temporarily;
coverage and unrelated runs can add more. No job, gate, scenario, fact, comparison,
timeout, lock, input identity, data restore or cancellation group changes.

[`max-parallel`](https://docs.github.com/en/actions/using-jobs/using-a-matrix-for-your-jobs#defining-the-maximum-number-of-concurrent-jobs)
caps one matrix. This patch cannot promise queue elimination. Existing
[`concurrency`](https://docs.github.com/en/actions/using-workflows/workflow-syntax-for-github-actions#concurrency)
keeps meaningful corpus and route runs running; lock-gate cancels superseded heads.
Do not combine these into a shared cancellation group. Keep scheduled corpus at
minute 17: official [schedule guidance](https://docs.github.com/en/actions/reference/events-that-trigger-workflows#schedule)
describes delays/dropped jobs under high load, especially at the hour boundary.
No stale-run cancellation or dispatch was performed in this investigation.

## Timing estimate and qualification

Holding observed replay job service times fixed, six runners give mathematical
isolated completion floors of 18.15 corpus, 20.56 route and 22.59 lock minutes,
before builds/plan/result and provider waiting. They are lower bounds, not forecasts.
The corresponding sums remain 108.92, 123.38 and 135.57 runner minutes; caps alone
do not save setup or replay work. Builds cost 3.72 / 4.90 / 4.90 Windows minutes
in these corpus / route / lock runs.

The lane's executable dry run models FIFO admission at observed replay creation
times, original API job order, fixed observed service times and a shared pool.
It excludes builds already represented by creation dependencies and reserves
capacity by varying the replay pool. GitHub does not promise this FIFO order.

| Replay pool | Matrix caps corpus / route / lock | Modeled completion after each family's replay creation: corpus / route / lock |
| --- | --- | --- |
| 20 | 8 / 8 / 8 | 14.10 / 17.30 / 25.05 min |
| 20 | 6 / 6 / 6 | 18.45 / 23.20 / 23.97 min |
| 16, reserving four slots | 8 / 8 / 8 | 14.10 / 17.30 / 31.90 min |
| 16, reserving four slots | 6 / 6 / 6 | 18.45 / 23.20 / 29.28 min |

Six slows isolated corpus/route completion and can shorten the competing lock
tail. The model does not reproduce the actual hosted queues, so an unconditional
wall-time improvement is unproved. After root review and landing, compare the
next required complete run on its exact head with these API durations; do not
repeat an accepted campaign just for timing. Required CI qualification retains
all existing plan/build/result checks, full corpus including scheduled faithful
sampling when applicable, both room chains and every committed lock. Timeout
continues to fail. If six produces unacceptable tails, tune the existing bins
using measured per-row service and setup costs before considering more capacity.

## Complete movie split design

The 528 passing room-chain scenarios prove selected gameplay from original
seeds. `full_route.py` expressly stops some rows at no-update waits and records
`state:8` instead of their next-room fact. These selected rows do not qualify
boot, all transitions, waits, endgame or uninterrupted movie continuation.
The three existing movies contain 129,712 any%, 222,788 100% and 167,797 low%
original video frames ([movie identities](../movies/manifest.toml)). Video frames,
native reads and scenario Ticks remain different coordinates.

The authentic any% map has 722,077 events, 106,967 native reads and 129,712 video
frames. It supports only the first 9,266 steps, retaining five earlier no-poll
vblank steps. There are 984 requested-NMI-work gap rows in 27 ranges, 117,159
blocked rows and an unsettled EOF. The first unsupported native read is 9,261;
later supported-looking ranges cannot qualify continuation through that gap.
The complete map is explicitly `complete = false`. Pins:

- Map: `b2413c7391902782a0812a5bead4bf01bd701702bf280f0fa86ee05af3af18a5`.
- Manifest: `3f4aef3b5e0c79a566ef8a85d9abcee014b8870232a5953ebc3df33899abadd1`.
- Canonical supported input prefix: `ccd67ec9329ba43256552e73f2fdfd3d6791ecae85a6c6834504b3fb7633c001`.

These are the original map owner's `full-movie-map-artifact-run/HANDOFF.md`,
`PROOF.md`, `GAPS.tsv`, `RANGES.tsv` and the exact manifest at
`full-movie-map-pack-qualification/artifacts/movies/any-original.map.csv.toml`
under the root's existing temporary collab directory. The manifest was read;
the map/private inputs were not copied or replayed. Its movie/native/native-source,
ROM, video, poll, event and importer pins remain separate. The owner's full-any
115,542-step attempt had zero comparison failures but missed 121 gameplay facts:
it remains a failure. 100% and low% complete movies remain unproved.

Use the current scenario format (`start`, `prefix`, `ticks`, `facts`) and pin
fields (`inputs`, `input_sha256`, `movie_map`, `movie_map_sha256`, checkpoints).
Once a parent is genuinely qualified, a bounded initial candidate subdivision is
input indices `[0,3000)`, `[3000,6000)`, `[6000,9266)`: boot then two genuine
continuation starts. Input index i commits Tick(i+1); initial observation is Tick0.
Rows use `prefix = B`, `ticks = E-B`, so every input and every committed Tick in
the supported prefix occurs exactly once in the selected windows. This is a
candidate within authentic mapping support, not a claim that the engine passes it.

For B>0, the existing `scenario cut PARENT --at B+8 --lead 8 --window E-B-8`
expresses a window without new machinery. Give each resulting row named facts
covering its boot/menu/gameplay/transition/event/endgame scope. The current cut
implementation preserves an inherited mapped input's entire authentic prefix and
map pins and validates the window before writing; it generates real patched and,
where the prefix passes, faithful checkpoints. Missing faithful continuation
remains an explicit uncovered/unlocked case. First boot window uses a normal boot
scenario; cut deliberately cannot generate a zero-prefix start.

Current `scenario.lockable` excludes rows with a prefix or checkpoint. Those
continuation windows therefore stay on the existing live comparison paths;
adding cuts does not automatically include them in the engine-only lock gate.
To make complete-movie windows part of that frequent gate, the existing lock
model must first qualify oracle identity from boot/seed plus the full input
prefix independently of the engine checkpoint, under REVIEW condition 8.

Before enabling any candidate, require original-read/Tick map integrity across
its full prefix, complete source/native/ROM/full-input/loader pins, genuine initialized
checkpoint payload/continuation, both live oracles, free run, named facts, and
record/check of eligible faithful locks. Keep original parent facts: a cut's local
fact alone cannot establish complete-movie scope. Preserve all rejection controls,
including input-prefix mismatch and extra bytes, and encrypted bundle restore
checks. Never restamp checkpoint headers or create post-gap continuations from
unsupported mappings. Later any% windows require requested-NMI cadence repair;
100% and low% require their own authentic event/read maps and qualified parents.

After measuring a few real windows, target roughly 6-9 minutes of replay service,
the observed route job scale. Weight each existing shard by measured continuation
time plus setup/restore and full cold prefix cost. With a compatible checkpoint,
setup can be shared at boundaries; cold regeneration must retain complete prefixes.
If K windows independently rebuild prefixes B1...BK, extra cold work is proportional
to their sum, not just the movie length. Capture matching manifest starts during
one original prefix traversal under a separate root resource grant; do not launch
independent full-prefix captures for every cut. Setup savings and window runtime
for these complete movies are unmeasured, so no runner-minute reduction is claimed.

## Artifact checks

The lane's `analyze.py` completes the metadata-only assignment/model check in
under four seconds using the existing embedded Python. The candidate planner
check compares every corpus shard and every route row assignment before/after,
and verifies workflow changes are limited to caps/comments. No compiler, DLL,
native capture, replay, private-data copy, Git operation or upload ran. Root must
apply/reconcile the patch and run required CI; these source checks are not replay
qualification. The report is a dated investigation, not a standing policy.
