# Continuous process improvement

Status: active
Owner: `01a143c7-78d1-7033-8eff-a361c16993dd`
Authorized: 2026-10-16 — user explicitly assigned ongoing responsibility to
evaluate and improve execution speed.

The objective is more completed, playable Rust behavior per unit time. Improve
the limiting step while preserving clear ownership and honest evidence. Keep
this record small; existing commits, handoffs and run receipts supply timings.
No new approval layer, general scheduler or verification framework is required.

## Baseline observations

| Observation | Implication |
| --- | --- |
| Projectile warm loop 17.479 s; combined scrolling loop 16.622 s | These loops already provide useful short feedback. |
| First original-call reuse 0.88 s; completed-capture reuse 1.63 s | Reuse valid captured calls without repeating emulator work. |
| Three confirmed integrator socket timeouts; observed inactive-command windows included roughly 12 and 17 minutes | Session/handoff delays can dominate the actual checks. These are observed wall-clock gaps, not isolated model CPU measurements. |
| Several sessions initially watched the same producer; recovery notifications crossed | Give collaborators independent output-producing work and reduce duplicate messages. |
| Door scope assigned 10:34:31; gameplay integrated 10:47:32, host 10:48:42, combined runner 10:51:10 | Source preparation already overlaps; inspect frozen-build release and execution latency next. |

Sources: retained producer tickets 05–08, `updates.md`, the original watch's
completion/recovery observations, and restart Git timestamps. These are bounded
observations, not broad performance guarantees.

## Changes being evaluated

| Change | Owner / current action | How to evaluate |
| --- | --- | --- |
| Separate frozen verification execution from source integration | First experiment measured; adjust next short batch to integrator launch with independent verifier analysis | Keep execution delegation only when it overlaps useful work without a longer critical-path handoff. |
| Prepare one successor while current code is integrated/checked | Roadmap session produces `next-transition-scope.md` | Next scope has actionable roots/files/scenario before the current delivery ends. |
| Execute fixed check sequences without model deliberation between successful steps | Verifier uses the existing frozen argv and expected exits, stopping on unexpected results | Actual execution approaches command time; no added framework or weakened checks. |
| Check fresh status immediately before conditional recovery | This owner; announced roadmap fallback only when needed | Preserve surviving writers and avoid resuming an already-active or completed turn. |
| Event-driven coordination | Owners report readiness, first failure, outcome and handoff | Fewer repeated updates and interruptions, while blockers still get immediate attention. |

The integrator need not wait for monitor acknowledgment to release a ready
build. A verification failure returns its first concrete diagnostic to the
source owner. Changed-source reruns require a new frozen handoff; a passing
step is not repeated without a relevant change or an unresolved question.

## Evaluation cadence

At each real result or completed subsystem, record what delayed progress,
whether the latest change helped, and one next action. Prefer timing from
existing artifacts to new instrumentation. Separate implementation/research,
build/execution, queue/handoff, model failures and cleanup/publication.

Keep, adjust or remove each process change based on those observations. Warm
cache reuse, disjoint source ownership and a single integration owner remain
in place. Reorganize finished work only when an observed bottleneck justifies
it. Do not make function/test counts or extra paperwork the throughput target.

## Current checkpoint

Door code integrated at `ca4f209` at 10:51:10 UTC. The frozen handoff was
released at 10:52:43 UTC (file mtime), 93 seconds after integration. Its exact
commands, executable/native hashes and exclusive outputs are recorded in
`/home/jim/sm-decomp-restart/.scratch/restart-pipeline/ceres-door-verification-handoff.md`.
The frozen incremental build itself reported 0.05 s.

At 10:57 the four reserved output directories were still absent/empty and the
verifier's local note still said unreleased. Its thread was active, with its
last visible tool action preceding the release. This establishes at least a
four-minute release-to-execution gap, not its cause or a failed owner. Relayed
the authoritative release path and immediate next action to the existing
verifier, preserving its ownership and any subsequently started command. The
integrator continues source/evidence publication and awaits verifier process
completion before timing warm-dev. No duplicate command was launched.

At 10:58:28 UTC the verifier launched the first reference, 345 seconds after
release. It stopped at 10:58:40 after 11.930 s, exit 1: required lifecycle or
ownership not observed. Subsequent candidate, CLI and fault runs were correctly
not started. These are the verifier timeline's own timestamps under
`restart/out/ceres-door/evidence/verifier/`; the existing failure evidence goes
to the integrator for one targeted diagnosis. Cause of the pre-execution delay
was not established; active thread status did not prove readiness.

**Decision: adjust.** Integrator agreed that the next short fixed batch starts
directly in the same execution turn as the source freeze. Delegate independent
result analysis and next-corpus/scope preparation. A 345-second handoff is not a
speedup for a 12-second command. Preserve current ownership and failure outputs;
do not race or repeat this reference. No general scheduler is needed.

The first failure also exposed a planning assumption: BA48 solidity setup was
required in a pre-escape scenario but the native owner's source diagnosis places
it in the escape-only room setup. The existing result also records opening for
parameter 1, while parameter 0 has no observed trigger/open step despite the
initial ticket promising both. Sent that exact scope discrepancy to the
integrator. Correct live claims and their justified predicates explicitly;
retain wider translated coverage separately. Do not turn a failed historical
result into a pass or repeat identical C execution if its complete trace can be
validly reused. The verifier independently checks proposed trace reuse.

**Next scope preparation:** anchor each promised material observation to the
selected room state and existing route evidence as early as possible. If fresh
instrumentation is necessary, use its first reference result to settle live
coverage before making speculative branch requirements into acceptance gates.
This is better placement of existing analysis, not an added audit or test suite.

**First observation after adjustment:** corrected coverage was checkpointed as
`bd4e28c` at 11:01:34 UTC; `dev-01`'s passing result/invocation were written at
11:01:56. Its measured complete loop was 21.419 s, including 0.908 s build and
one Rust game process. Thus checkpoint-to-result was about 22 s, with almost
all of it explained by the loop itself. This is an observed improvement in
handoff behavior, not a controlled benchmark or universal speedup claim.
The verifier checked the complete 9,380-step reference and retained hashes;
no second C replay was required. Root's ordinary CLI also passed afterward.
Fault control, documentation reuse and final cleanup/publication subsequently
completed; successor implementation assignment remains a separate next step.

Successor packet completed before door cleanup/publication: three-root,
15-state door-transition scheduler (19 routines including private dispatch;
recommended 27 with background command handlers), bounded Rust reuse, exact
host/coroutine constraint, existing crossing evidence, and disjoint proposed
files. Read the final host/reuse/scenario appendix in
`../restart-collaboration/next-transition-scope.md`. No new writer or run was
needed for that research.

**Decision: keep successor preparation in parallel.** It delivered an actionable
packet before the current milestone ended. Existing workers can settle its
small public ABI while current cleanup completes; the integrator chooses and
claims the scope at the final baseline. Next observation is time to an owned
successor, rather than more monitoring or speculative research.

Door ticket 08 is resolved with all comparisons and fault control complete,
documentation reuse 0.303 s / zero game processes, both workers retired, and
392 retained hashes authenticated after cleanup. Gross apparent bytes removed:
716,734,922; this is not net filesystem recovery. Final checkpoint `ca64dfc`
was committed at 11:10:23 UTC; clean restart checkout observed at 11:11. Its
final retained set has 466 authenticated files. Existing workers are preparing
the successor ABI; no new implementation claim is implied yet.

Next measured cost candidate: the observed inventory status invocation took
12.265 s and three stage updates together took 40.855 s (53.120 s utility time,
not whole closeout time). Requested a bounded read-only diagnosis by the now
available verification owner after its door closeout: locate repeated work or
an existing supported shortcut, without new runs, code, gates or production
mutation. This does not block the successor and does not justify a new platform.

At 11:10 the integrator reported every door requirement complete but its final
checkpoint waiting on the successor ABI note. Directed immediate door-only
checkpointing and a separate successor handoff. Completed milestone publication
must not depend on future design research; keep the next packet moving in
parallel without coupling its finish to already completed evidence.
The integrator followed this separation and checkpointed door delivery within
the same minute. Continued evaluation is tracked in
`issues/04-room-transition-process.md`.

At 11:16 the publication-cost report was available. It identifies repeated
whole-registry evidence/source-history observation, with deduplication only
inside each invocation; phase dominance and installed binary/source identity
remain unproved. **Adopt the existing supported shortcut:** use the published
registry for ordinary source-identity/status reading and status already printed
by an update, omitting redundant standalone status. Keep every required stage
publication and authentication. This avoids an observed 12.265 s operation when
its fresh validation is unnecessary; savings on the next milestone remain to
be measured. No supported generic multi-update exists, so no batching change is
authorized from this finding.

Timing provenance observed in integrator turn
`01a1443c-d9fb-79e3-a109-5f890c63ba86`: command
`exec-fcc732da-cca0-4c26-9fe4-f0075e321149` reported 12,265 ms for status;
`exec-d55cbfb8-1b03-4327-a227-7e0dbf43c1eb` reported 40,855 ms for three
sequential updates. These are command durations, not phase profiling.

At 11:22 both transition writers had begun source edits and root registration
was active. Successor research was still awaiting candidate acknowledgment.
The proposed directional door-scroll/IRQ scope is already a declared disjoint
dependency outside ticket 09. Released bounded read-only research to proceed
with that assumption while root retains final implementation choice. Requiring
an acknowledgment for this reversible advisory work had added an unnecessary
dependency; do not add that wait on future clearly disjoint successor research.

Ticket 09 completed at `cee31cf` (11:47:38 UTC), with gameplay `9ee6419`.
Corrected `dev-02` passes in 15.356 s, but its build was cached (0.082 s):
the preceding standalone compile was 0.61 s and the first forced 0.545 s build
belongs to the failed `dev-01`. Do not advertise 15.356 s as a fresh forced
rebuild loop. CLI 10.523 s, actual wait-fault rejection at step 8750, docs reuse
0.189 s / zero game processes; all 542 final retained hashes pass. Twenty-one
reached routines gain verified/live observations, all 27 integrated.

Two process regressions remain concrete:

- The supported status shortcut was not applied at closeout. Integrator command
  `exec-5c260bcb-19a0-4c6e-8a3e-cb851270779f` in turn
  `01a1446c-79d6-79d3-b21f-2121f4eceaaa` ran a redundant standalone status and
  took 22.608 s. Required subsequent updates still perform authentication.
  Verifier will check actual omission next time, not merely a stated intention.
- Integrator became idle after completion while ticket 10 was ready/unclaimed.
  Fresh status at 11:51:15 confirmed idle, so this owner resumed the same
  integrator in the same inspection operation. No source owner changed and no
  check was repeated. Our own observation interval had also stretched from
  11:39 to 11:51; its cause is not established by the available tools.

**Adjustment:** the otherwise-available roadmap coordinator provides a backup
continuation check. If the same integrator remains confirmed idle for over 60 s
with ready unclaimed restart work, it rechecks immediately and resumes that
existing owner, then reports here. It never creates competing writers or takes
over ambiguous ownership. This reduces dependence on any single monitoring
turn; it is not a new scheduler or an installed background service.

Ticket10 completed at `71cf03b` (12:13:48 UTC). First candidate passed in
15.689 s including a real same-byte-touch 0.601 s rebuild; reference creation
was 30.956 s and is a separate one-off cost. The warm loop remains within30s;
no new performance project is warranted. Verifier inspected actual command
receipts and found required updates without standalone status this time.
Keep the shortcut written in assignment/closeout instructions rather than
relying on a reminder alone. CLI9.797s, gameplay fault14.417s, docs0.190s with
zero game processes; cleanup retired both workers and preserved warm cache.

After five consumers on the same short route, recommended a small extension of
playable reach before another pre-escape translation: accumulated Rust through
the retained16,991-input full Ceres/area0 route. Roadmap confirmed that current
CLI already supports this with a90s bound and separate ownership inspection.
The producer created ticket11, now ready after10; this owner directed immediate
claim/execution at12:14. It adds no gate to10 and claims neither full per-step
C/Rust nor original parity. Normal-camera research remains independent.

Cleanup: process coordination has created no builds, tests, gameplay runs,
worktrees or run intermediates. Only local Markdown is owned; 0 bytes reclaimed.
