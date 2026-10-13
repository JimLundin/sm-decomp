# Decomp swarm launch status

Status: activated; handoff ready
Blocked by: setup turn completion for integrator transfer; gameplay milestones retain their gates
Updated: 2026-10-13

Production authority is active at [authority.toml](../authority.toml), with the
canonical durable [queue](../ledger.toml). The legacy Ceres ledger retains its
exact held work. [Activation](receipts/activate.stdout.toml) passed the reviewed
engine/source/drill/bridge checks. Preparation tickets 01–05 are complete.

The dedicated background task **Run decomp swarm** is active, real thread
`01a13592-d02b-7143-8489-a15b03f19da2`. Its dispatcher `decomp-run-20261013` is
registered at generation 1; see [registration](receipts/register-background.stdout.toml).
The setup/main Git/integrator role remains with real thread
`01a1356e-a8dd-7a80-88b0-1239a6a9b876`, token `ceres-20261013`, generation 1.
After observing this setup turn completed and the local checkpoint committed,
the background coordinator performs checked integrator transfer to generation 2
and continues the existing queue. [Start instructions](coordinator-start.md)
define that boundary; the setup dispatcher takes no new work after handoff.

Research tickets 13/14 completed through actual subagents and native result
admission. Exact claims, UUIDs and results are in the start/finish receipts and
manual handoffs. The [NMI Answer](research-13/answer.md) identifies eleven complete
bank80 routines; the [Samus Answer](research-14/Answer.md) selects beta separately
from alfa/input. Children 21–24 retain exercise, hardware and dependency gates.
This source research created no gameplay stage.

Host continuation passed 31 focused Rust tests and 10 current-source actual-C
fixtures. Both matrix cleanup passes succeeded and both verdicts authenticated
afterward. See the [gate audit](host-continuation/audit/gate-audit.md) and
[held host handoff](handoffs/held-host.toml). All affected Offline verification
and original host admission remain pending; controller integration follows.
The original baseline is `48656a284329d432291eca23a2ae0d1d7c0a3e48`. Host source
was already merged; controller submission `223f8a1` remains held.

Project [.codex/config.toml](../../../.codex/config.toml) enables up to three
spawned agents with researcher, worker and verifier roles, inheriting model and
permissions. Every session shares `SM_PIPELINE_ROOT=/home/jim/sm-decomp/out/pipeline`
and `SM_PIPELINE_WORKERS=4`; assignment capacity is 3. The existing
[progress viewer](http://127.0.0.1:4185/) remains available. Original-ROM, live
exercise and full-game milestones require their independent evidence.

[Cleanup](cleanup.md) is complete: utility reclamation 0 bytes, identified test
build removal 32,596,787 logical bytes, observed remaining space 221,151,543,296
bytes. TOML lint, Codex configuration loading and scoped source checks passed.
Retained research, archive, engine, drill and bridge hashes were rechecked.
The [setup checkpoint](checkpoint.md) records the durable launch and next action;
the [resume prompt](resume-prompt.md) continues this same activated workflow.
