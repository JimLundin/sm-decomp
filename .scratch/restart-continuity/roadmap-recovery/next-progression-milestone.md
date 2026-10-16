# Next progression recommendation: Landing Site to Parlor

Status: recommendation consumed; integrator implementing ticket15
Blocked by: None
Triage: ready-for-agent
Date: 2026-10-16 13:41 UTC
Owner: recovery session `01a144f0-bd8c-7ec1-b86c-3a4065c33302`
Integrator: `01a144ca-27ba-7130-a9e2-a615c7c80e3a`

Claim one ticket15 milestone: preserve the exact successful 16,991-input native
Ceres prefix, append a bounded ordinary-input tail, cross west from Landing Site
(`0x91F8`) to Parlor (`0x92FD`), and demonstrate material controller response
after normal gameplay resumes. Reuse the existing Game loop, input logs,
observations, replay and presentation. The integrator owns claim, implementation,
execution and acceptance; this packet adds no separate approval or run gate.

## Concrete next action

Create/claim the single successor in the restart issue queue at clean `a8a8c97`.
Implement one named post-Landing checkpoint using existing runtime mechanisms
and author its input tail. Start with a finite tail budget (4,096 host steps is
a reasonable initial ceiling) and the established bounded runtime deadline;
stop on the first actual behavior gap if the checkpoint is not reached. Retain
that diagnostic and address the reached dependency rather than broadening to
unrelated routes. A tail-budget exhaustion records an incomplete attempt.

Success means observing the area0/Parlor room transition, state8 resumed
gameplay, and a short ordinary-input interval with corresponding Samus movement
or another material input effect. Record the input/state correspondence. Merely
reaching an input/step limit or seeing the room during loading is insufficient.
Keep the current mixed C/Rust ownership explicit. No whole-game or original-ROM
agreement follows from this new native progression checkpoint.

## Retained foundation

- Prefix: `/home/jim/sm-decomp-restart/restart/out/runs/ceres-06/inputs.txt`
  (16,991 lines, 67,964 bytes). Existing route reaches area0/room91F8/state8 at
  its final step; no post-Landing native input continuation is retained.
- The current accumulated Rust foundation is ticket13's
  `/home/jim/sm-decomp-restart/restart/out/projectile-escape/runs/dev-02/result.toml`,
  final gameplay source `5850265` and native package prefix `0eee`. Its full
  16,991-step per-step C/Rust comparison includes 706 high-bit projectile
  updates owned by Rust; draw refusals remain707 each. Reuse the retained C
  reference only where native/schema/input identities actually match.
- The older `restart/out/full-ceres-rust/runs/escape-01/result.toml` and
  [full Ceres packet](../../restart-collaboration/full-ceres-milestone.md)
  describe the prior five-consumer runtime with CSV/endpoint agreement. Keep
  that history distinct from the current ticket13 foundation.
- Tickets13/14 are complete. Ticket14 at `a8a8c97` used the retained 34 original
  Handle call pairs and reports 801-byte original projection plus full C/Rust
  equality in 7.134 seconds. Reuse its evidence; do not rerun or recapture it.

## Existing tool limits

The failed coordinator's retained findings identify `runtime_cli.rs` around
319 as the Ceres scenario stop on area0 arrival. The `none` scenario accepts
its input/step extent without a progression predicate. Interactive mode around
350 overrides the input file after its first mask, so `--inputs PREFIX` plus
interactive play does not provide prefix-then-keyboard handoff. Game boots with
zero SRAM and has no restore interface. A save/load facility or new framework
is not required to run this bounded continuation through the current Game loop.

These are retained research findings, not newly executed tests. Existing TAS
recordings have synchronization limits described in `docs/TAS_MOVIES.md` and
do not establish an already-native continuation.

## Explorer result received — 2026-10-16 13:48 UTC

Explorer `01a14452-ba4e-7852-ae9f-1f4719bf11ad` published and released
[its packet](../../restart-collaboration/next-progression-milestone.md).
RoomHeaderPointers.cs:6–10 in the retained community source names91F8 as Landing
Site and92FD as Parlor and Alcatraz. Legacy `out/sync/full-any-lsnes.csv` rows
14015–14016 show the91F8-to92FD transition during000B; rows14040–14042 resume
state0008 and move x04D8→04D2→04CC at y008B. This is diagnostic route guidance,
not a native input continuation or timing contract. Exact Landing west-door
geometry was not established in that packet. Continuity permits the material
implementer to inspect minimal required door data as implementation context.

The explorer's later6000-tail/120-second/32-update proposals remain advisory;
ticket15's already-selected4096-tail/90-second contract owns implementation.
The current ticket13 replay foundation above supersedes the explorer packet's
older FULL_CERES reference. Its packet introduces no additional gate or rerun.

The former coordinator's intended
`.scratch/restart-collaboration/next-progression-milestone.md` did not exist at
recovery, and the explorer's exact write ownership could not be established
from read_thread. This replacement packet therefore uses the recovery owner's
exclusive directory and leaves that potential explorer path untouched.

## Cleanup and ownership

Research/recovery only: no code, builds, gameplay runs, captures, verification,
worktrees or generated run intermediates. Reclaimed bytes0; no cleanup utility
applies. Available disk at recovery inspection: 104234938368 bytes. Other owners'
source, evidence, prepared libraries, warm caches and legacy production fences
are preserved. Routine integrator continuation/recovery watch remains active
under this packet's owner after delivery.

## Delivery checkpoint — 2026-10-16 13:43 UTC

The integrator was fresh-checked idle83 seconds at13:41:56 UTC and resumed in
the same tool operation. It accepted ticket15, retained sole native-run/index
ownership and delegated disjoint runtime checkpoint/input-policy work to its
existing workers. By13:43 UTC it committed CLI registration and the bounded
Parlor contract. The current ticket13 foundation correction was sent directly
and incorporated above. Explorer source findings were delivered directly
at13:48 UTC; no original capture, new run or duplicate geometry research was
performed by this owner.

13:52 UTC: the integrator's first bounded `parlor/author-01` reports success at
18072 total steps (1081 tail), area0/room92FD/state8, eight moving polls and29
leftward pixels after resume. Runtime16.825 seconds and zeroCPU fallback.
Saved-input replay, publication and cleanup remain with the active integrator;
this observation is a progress checkpoint, not ticket resolution.
