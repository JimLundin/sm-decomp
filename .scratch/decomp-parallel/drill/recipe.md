# Actual independent-session concurrency drill

Status: actual execution complete; final readiness validation pending
Owner: root preparation drill worker

The production queue stays untouched. The retained, hashed Rust `decomp-drill`
harness controls only `/tmp/sm-decomp-concurrency-drill-20261013` and its assigned
`/tmp/sm-concurrency-drill-JOB-ATTEMPT` worktrees. The isolated Git repository
contains a tiny, dependency-free Rust fixture and three disjoint documents.
Its reviewed gates execute real nonzero offline tests. The maintained pipeline
binary is built from the frozen full source commit; receipts separately name
that source/binary and the fixture baseline. No ROM or production source is
an input.

Two persistent `codex exec --json -s danger-full-access -C ISOLATED -` sessions
inherit the user's model configuration. The harness removes inherited
`CODEX_THREAD_ID` only at process launch; Codex supplies each new actual id to
its own commands. It retains `thread.started` JSONL, process PID/boot id/start
ticks/argv/cwd, prompts and exact request/result/error artifacts. Commands bind
managed authority to the actual caller identity. The harness never assigns a
made-up thread id or overrides models.

1. Start A and B concurrently. A prepares explicit integrator authority and
   registers A/B dispatcher tokens at generation 1 from actual launch records.
2. A/B request one assignment concurrently from the same queue. They launch
   actual persistent worker Codex subprocesses, each in its own process group.
   Those workers use collaboration subagents for scoped source/validation when
   the installed CLI supports the tools. A/B record exact returned worker ids
   and publish maintained typed handoffs. A/B fill capacity 3 and an extra
   request defers. Five real lease clients contend for the same worker budget
   of 4, with four simultaneous owners and one observed waiter.
3. Interrupt only A's controller process. Observe its exact stopped process
   and separately observe its worker thread/process still running and writing
   scoped evidence. B inspects shared queue and confirms no claim release or
   duplicate attempt. A's worker finishes bounded edits; preserve its WIP.
4. Stop the actual worker, retain exact source and evidence, then seal its
   stopped observation and A's role transfer, accounting for every held child.
   B becomes the same dispatcher token's generation 2 owner, adopts the typed
   handoff and reads its reviewed next command. A's generation 1 mutation and
   result retry reject. Only a checked worker-stop update releases its running
   attempt; the new attempt receives a new returned worktree and recovers the
   retained source through its exact scoped commit.
5. Worker B completes a scoped checkpoint and submission. After durable result
   publication, interrupt the dispatcher command caller; a resumed real session
   retries the identical typed update. No changed request is accepted. Test an
   old attempt after replacement and a pending revision result as rejections.
6. Integrator A is observed stopped. B requests the next integrator generation,
   including held integration/submission children and observed processes, then
   adopts and resumes the exact stored handoff. Old integrator mutations reject;
   a second integration reservation defers while the first remains running.
   Retain submitted ownership throughout, merge only the exact scoped source,
   freeze merged fixture HEAD, and run maintained integration gates.
7. After all writers stop, both scoped cleanup passes run. Preserve source
   snapshots and receipt dependencies, remove integrated worktrees/builds, and
   reauthenticate hashes and maintained authentication receipts. Record measured
   bytes and remaining disk space. The root owns final production cutover
   documentation only after it reviews the observed acceptance controls.

The frozen harness implements the actual launch, preparation, registration,
assignment, start/checkpoint, publication/retry, resume/adopt, checked transfer,
stop, integration and lease routes. The small separate Rust `evidence-tools`
helper added one reviewed pure duplicate-ticket request and builds/reauthenticates
retained manifests; it does not change the frozen engine or harness.

Observed outcomes are retained in `execution.toml`, `concurrency-drill.toml`,
`retained-authentication.toml`, `cleanup.md` and `integration-limitations.md`.
All 11 typed controls passed independent review with one authenticated integration.
The pending-revision extension in the original recipe was not run. A subsequent
sequential lane B integration Start exposed a frozen scoped-handoff limitation;
its honest diagnostic and submitted source remain retained. The production
read-only validator is separately versioned; production authority stays inactive.
