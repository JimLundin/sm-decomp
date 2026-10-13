# Fresh final independent-session concurrency drill

Status: actual final execution and cleanup completed; root validation pending.
Owner: fresh_drill_prepare. Production authority remains pending.

The immutable previous drill and isolated repository remain untouched. The new
Rust harness owns only `/tmp/sm-decomp-concurrency-drill-final-20261013`, its
queue-returned `/tmp/sm-concurrency-drill-final-JOB-ATTEMPT` worktrees, and this
store. Its effort is `concurrency-drill-final`. Two actual independent
`codex exec --json -s danger-full-access -C ISOLATED -` sessions inherit the
user's configured model. Launch removes inherited CODEX_THREAD_ID so Codex
provides its real UUID. No made-up identity or model override is used.

1. Freeze exact corrected engine source, commit, fingerprint and retained binary
   under root's build provenance; setup seals those independently of the tiny
   Rust fixture baseline. Build both Rust helpers locked/offline.
2. Launch A/B simultaneously using prepared prompts. A prepares isolated
   integrator authority and distinct real dispatcher identities. Both race one
   reservation and launch real workers; those dispatch real collaboration
   subagents restricted to one document. Maintained Start/checkpoint preserve
   exact UUIDs, WIP and handoffs. Capacity3 defers a fourth reservation.
3. Interrupt only A controller. B checks the same stopped parent, same live
   worker process and exact held claim. Transfer dispatcher generation, reject
   resume until adoption, then adopt/resume live child's unchanged handoff.
   Preserve scoped WIP, stop worker with checked observation, reserve attempt2
   with new workspace, recover scoped commit and dispatch actual replacement.
   Original A generation and superseded attempt reject.
4. Publish A/B disjoint submissions and retry exact updates. Zero-slot next
   controls prove writable overlap and a separate disjoint duplicate ticket
   defer. Five real lease subprocesses contend for budget4; snapshot four
   simultaneous owners plus waiter and retain all released receipts.
5. Resume A integrator, reserve first integration, launch independent real
   integration child and maintained Start/resume. A second integration defers.
   Interrupt A only; B inspects exact live child, transfers integrator generation
   and adopts/resumes same claim. Old generation rejects. Stop all implementer
   writers and release child before cleanup/integration.
6. Authenticate first A integration, then reserve/start/resume/authenticate B
   sequentially against accumulated merged source. Both utility cleanup passes
   and retired worktrees are mandatory. Retry final B exact integrated request
   after retirement at unchanged HEAD. An A retry after B changes merged source
   rejects the stale cached source; retain that fence instead of rolling back.
7. Final process observations, cleanup, retained source and manifests attest all
   11 existing controls and exactly two authenticated integrations. Root's
   read-only preparation validator is the final review boundary.

Reproducible build commands:

```sh
cargo build --locked --offline --manifest-path /home/jim/sm-decomp/.scratch/decomp-parallel/drill-final/Cargo.toml --target-dir /tmp/sm-decomp-drill-final-harness-build
cargo build --locked --offline --manifest-path /home/jim/sm-decomp/.scratch/decomp-parallel/drill-final/evidence-tools/Cargo.toml --target-dir /tmp/sm-decomp-drill-final-harness-build
```

After root supplies its exact frozen binary/source, run `decomp-drill setup
ENGINE SOURCE`, then `launch a PROMPTS/session-a.md` and `launch b
PROMPTS/session-b.md` concurrently. Every command uses the absolute helper
path `/tmp/sm-decomp-drill-final-harness-build/debug/decomp-drill`. Store
actual launch JSONL/process identities and root-controlled barrier transitions.
Unexpected rejection retains diagnostics and stops dependent work.

Completed-run receipt inspection uses the retained `helpers/decomp-drill` and
`helpers/decomp-drill-evidence`; original build paths are retired. Reproduction
also creates the reviewed explicit issue files described in setup-repair.md
before next, seeds a same-id disjoint duplicate candidate under an open ticket
before switching to the held B ticket, and uses versioned
`helpers/decomp-drill-evidence-v2 start-request CLAIM` to add exact submitted
commit from durable ledger to an inherited Start request. Frozen main.rs stays
sealed. All approved mutations remain maintained Rust commands.

`helpers/decomp-drill-evidence-replay seed-ticket` and `switch-ticket` reproduce
the reviewed duplicate-ticket artifact repair entirely in Rust. The replay
helper built locked/offline; switch-ticket validated the exact never-launched
ready candidate and restored the already-sealed original scope without changing
claim/source. It supplements the retained actual execution log, which records
the transient repair used during that session.
