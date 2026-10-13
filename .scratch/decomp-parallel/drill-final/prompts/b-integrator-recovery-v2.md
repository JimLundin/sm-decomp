Resume actual B in disposable fixture only. All completed controls and A/B
submissions remain held. A independent integrator parent is stopped; actual
integration-worker remains same live child. Use original absolute decomp-drill
/tmp/sm-decomp-drill-final-harness-build/debug/decomp-drill. Never set identity,
models or use Python; all orchestration must use Rust helper commands.

Observe a and integration-worker. Transfer drill-integrator a 1 b, accounting
same live child and unrelated lane-c attempt2 reserved but never launched.
Release that lane-c via release-reserved lane-c drill-integrator 2 (checked no
launch). Resume integrate-lane-a drill-integrator2 reject, then adopt with
absolute .scratch/drill-control/drill-integrator-generation-1-transfer.toml,
then successful resume same claim. Use separated arguments. Mark
integration-adopted-ready. Wait integrations-go50 in bounded retries.

Observe integration-worker, worker-a-replacement, worker-b stopped/finished.
Integrate integrate-lane-a2 (separated args). It must authenticate actual gates,
cleanup both passes and retire A worktree. Reserve next-token drill-integrator2
once: require integrate-lane-b. Original start integrate-lane-b b
drill-integrator2 safely rejects omission of required submitted_commit; retain
that diagnostic. Then run versioned Rust evidence helper
/tmp/sm-decomp-drill-final-evidence-v2-build/debug/decomp-drill-evidence
start-request integrate-lane-b. Run original decomp-drill request decomp-session
absolute .scratch/drill-control/integrate-lane-b-corrected-start.request.toml,
then versioned helper remember integrate-lane-b. Resume integrate-lane-b
drill-integrator2, which must accept accumulated already integrated A source.
Integrate integrate-lane-b2 sequentially. Require authenticated gate/cleanup
and retired B worktree. Retire-attempts lane-a. Run integration-retry
integrate-lane-a2 then integration-retry integrate-lane-b2 after retirement.
Mark integrations-finished-ready and end with both exact outcomes. No further
claims, Python, production source/authority or source/engine/harness changes.
Unexpected failure preserves evidence and stops dependent mutations.
