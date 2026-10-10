# Readiness commands

`just readiness` allocates one immutable run and executes the fixed readiness
contract for Japan/USA any%, clean power-on, seed 0, P1, input indices 0..17999.
It prints the run directory and brief immediately. Doctor, check, lint, every
70,011 enabled native instruction case, independent timing, cold fallback and
current generated parity, actual key coverage, independent synchronization,
focused generated execution and paired cache fidelity remain mandatory.
Feedback demonstrations are also mandatory; absent or failed evidence prevents
acceptance. `just readiness-feedback` exercises the nine isolated synthetic
controls separately; see [their contract](FEEDBACK_DEMONSTRATIONS.md).

`just readiness --original-run out/runs/RUN` can consume a separately verified
original recording-core capture. Initial candidate synchronization always runs
fresh. `just validate-iteration` consumes the verified current accepted pointer;
`--predecessor RUN` selects its preserved accepted lineage explicitly. Dependency
reuse verifies complete accepted origins and unchanged shared inputs. Generated
parity, changed-entry coverage, focused execution and cache validation remain
candidate-specific. Initial preexisting unexecuted keys remain unvalidated debt;
every newly added or changed candidate key must execute before acceptance.

`just replay --start-input N --start-instruction N --instructions N --cache PATH`
records an explicit diagnostic interval without advancing acceptance. Focused
readiness intervals come from actual generated dispatch discovery. Overlapping
intervals are merged and retain all requested generated keys. Complete paired
cache creation is timed separately from warm replay. Default time targets are
1200 seconds for cold replay and 60 for warm focus; `--timeout` sets an explicit
hard budget and a timeout is incomplete.

The command preserves all mandatory outcomes, including multiple failures, and
returns 0 for complete validation, 1 for divergence/uncovered execution and 2
for infrastructure/incomplete failure. Each corpus batch retains the exact
executed executable, generated body, prepared expectations, ordered ledger,
process commands, return codes and logs. Independent timing has a separate
process report, so result data cannot overwrite its execution provenance.

Acceptance rechecks every gate's actual origin and exact scope, sealed evidence,
source snapshot, current material identities and predecessor under an acceptance
lock. It then atomically replaces `out/accepted.json` and retains prior pointers
in `out/accepted-history/`. Acceptance does not imply a Git commit. The immutable
brief says validation passed with acceptance pending; `acceptance-result.json`
records publication success or failure. A standalone scoped pass cannot publish.

`just readiness-restore RUN NEW_DIRECTORY` verifies and restores a selected
terminal source/config snapshot into a newly initialized Git directory.
`--accepted` additionally verifies complete accepted evidence and pointer lineage.
Existing destinations are rejected before writing, preserving unrelated edits.
ROMs, downloads and build output are not copied into recovered source.
