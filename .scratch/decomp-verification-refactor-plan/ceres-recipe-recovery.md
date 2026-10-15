# Ceres actual-C recipe recovery

Status: ordinary complete; final combined Native acceptance pending.
Owner: `/root`, session `01a13e7f-ab01-7413-a1c2-288b08a4b719`.
Source: `c4dc6895fc23e4e7373d2f5531c8867056dab994`.
Worktree: `/home/jim/sm-decomp-performance-ceres`.
Proof: `/home/jim/sm-decomp-performance-ceres-proof`.

The frozen Native matrix at `7b27ea1` failed to link Ceres before its 1,296
Fixture pairs. `EprojPreInstr_CeresElevatorPad` reaches `CallSomeSamusCode`,
whose function-pointer table retains broad real C dependencies. Adding only
`sm_90.c` cannot satisfy that closure. Two bounded actual compiler diagnostics
reproduced the failure in about three seconds each; linking the complete pinned
C unit set succeeded without an additional host shim.

`76edce2` adds the real registered-compiler regression and a clearly marked
diagnostic example. `c4dc689` adds the private `Recipe::CompleteC` variant for
ordinary and Native Ceres. It keeps the existing family ABI, full observation,
all required cases, and Native compiler slots. Real C supplies the globals;
there is no generated gameplay substitute or dispatcher shim. The registered
compiler control failed in 4.69 seconds before the correction and passed in
17.11 seconds afterward.

## Complete ordinary proof

An independent Git archive at `c4dc689/source` contains exact tracked bytes,
the copied pinned C reconstruction, private ROM, and pinned C archive. The first
attempt failed before compilation because the archive had been omitted from
that proof copy. Its rejected verdict and logs remain unchanged under
`c4dc689/ordinary`. The missing archive was copied independently from the
existing prerequisite and checked against the pinned SHA-256:
`69c78831351112887575f852dadf0281dd5c23cef890b07a268ae6cf60aa9193`.

A fresh attempt under `c4dc689/ordinary-02` passed all **1,296 pairs / one
routine** in **50.40 seconds**, using the shared four-worker budget and a
600-second experiment bound. The full-domain test requires fresh producers and
the exact declared count. Its verdict is:

`c4dc689/ordinary-02/evidence/1792088505911628-227327-0/verdict.toml`.

Retained exact reader: `c4dc689/tools/ceres-recipe-tests`, SHA-256
`a04d4f5198e692465b179cf836c45f21a13d24157e92b8676544f4bd7d69e52e`.
Execution logs: `c4dc689/logs/ordinary-02.{log,time}`.

Both cleanup passes passed for the ordinary proof and its temporary directory.
Cleanup reclaimed zero bytes; deduplication reclaimed **233,570,304 bytes**.
All sealed metadata hashes passed afterward. A fresh process, with runtime
tools unavailable on PATH, independently authenticated every required pair in
**8.20 seconds**. Logs: `ordinary-02-independent.{log,time}` and
`ordinary-02-after-admission.log`. This is ordinary acceptance; the final
combined Native recipe matrix remains required.

Both cleanup passes also passed for the first failed ordinary attempt and all
five diagnostic/link run directories. Each reported zero bytes. Every retained
file still matched its pre-cleanup hash. Failed evidence remains diagnostic.

## Remaining work

Integrate the recipe with final02/03/04/06, the optimized Native admission,
and07/08. Run the complete Native recipe matrix with the unchanged production
budgets and full producer attribution. Retire the implementer worktree/target
after preserving any exact evidence dependency, and authenticate the ordinary
proof again afterward. No installed tools, policies or gameplay claims changed.
