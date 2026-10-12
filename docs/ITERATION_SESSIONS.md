# Bounded recompilation sessions

`just recomp-iteration` requests one accepted increment. `--count N` requests a
positive number; `--target-generated N` instead stops once the verified same-domain
generated-instruction total reaches that target. These forms are mutually exclusive.
Readiness remains its own bounded command. Direct `validate-iteration` requires
`--preparation RUN` pointing to a verified sealed preparation. A target already met performs no
candidate changes.

The session coordinates separate exact-source child runs for baseline readiness,
measured selection, preparation, candidate validation and experiments. Its mutable
`state.toml` is a control log, not candidate validation evidence. Every operation
also has an immutable numbered event. Validation allocates and records its child
before applying the prepared configuration. Resume verifies the source, child,
event history and publication, and counts each authenticated publication once.
Failed/uncovered/error/incomplete candidates
and original user work remain preserved; only published acceptance reverified from
the current pointer and this session's candidate counts as progress. A baseline
refresh or competing/stale acceptance does not count.

Absent acceptance runs the complete readiness gates. Verified historical acceptance
with current source changes refreshes the required exact gates before selection.
An invalid pointer is copied byte for byte into sealed recovery evidence; fresh
readiness gets no parent or reuse from rejected lineage. Publication verifies all
new gates and requires the rejected bytes still match under the acceptance lock,
retains rejected-pointer evidence, and atomically replaces the pointer. Concurrent
pointer changes reject recovery; invalid evidence is never deleted to make a run
appear initial. Missing inputs produce concrete retained setup/gate reports.

Supported measured routines take priority. The session prepares the one exact
observed slice, verifies the current accepted predecessor immediately before
mutation, and runs fresh full generated parity and all applicable gates. With no
useful supported routine, observed prospective LDA absolute can take the independently
proved pattern route. Unimplemented observed patterns or no remaining covered work
produce concrete decisions; execution outside input indices 0..17999 needs separate
independent domain validation.

`just recomp-iteration --resume SESSION_ID` preserves the original request, accepted
count and unresolved increment ID. A failed candidate resumes through its verified
ordered diagnostic history. New source after an agent's bounded repair gets a fresh
validation snapshot; unchanged uncovered new keys can take verified actual table
pruning. Unchanged semantics failure asks the agent for the next concrete experiment,
without resetting the history or pretending to synthesize a repair. Three successive
experiments without new discriminating evidence require a decision. Shared hardware
repairs require refreshed readiness and separate scope. User stop halts the current
process after retaining its interrupted logs and preserves all counts/evidence.

The brief records accepted/requested progress, exact domain, source changes,
selection rationale, compiled/interpreted totals and fraction, per-key counts,
validated distinct keys, declared pattern scope, deferred/unsupported inventory,
reused evidence, gate/build/corpus/cache/cold/warm durations and preserved diagnosis.
Warm replay targets 60 seconds and each cold replay 1200 seconds; completed overruns
are reported, while actual timeouts remain incomplete. Source diffs compare the
candidate against preserved accepted source, including preexisting user work.

Session fixture operations are an in-process public boundary for testing request,
continuation and stop behavior with actual temporary evidence files. Production
commands expose no alternative gate runner or acceptance bypass. The completed
[real integration session](ITERATION_DEMONSTRATIONS.md#completed-real-integration)
accepted exactly one increment after full 18,000-frame validation and stopped,
with preserved retries, source diffs, per-key progress and separate timings.
