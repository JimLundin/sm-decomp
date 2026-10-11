# Projectile pickup collection
Status: resolved
Blocked by: 77
Triage: ready-for-agent

Translate complete EFE0 pickup expiration/grapple/Samus collision and five pickup
callbacks. Preserve signed wrapping restore comparisons, reserves, sound ring
quota/refusal writes, exact proximity inequalities and expiration ordering.
Verify actual-C full WRAM cases and independent examples. No live claim.

## Implementation

Native EFE0 expires before collecting, preserves decremented-lifetime grapple
threshold and strict signed16 grapple/unsigned radius collision boundaries, and
executes five original pickup effects. Health/missile reserves and mode preserve
wrapping signed comparisons, including missile reserve cap99. Existing native
audio queue writes quota even when full/frozen/game-state/debug refusal rejects
sound. Pure SamusCode0D handler returns2, dispatcher returns1, with no writes.
Actual C harness links the full sm90/sm91/sm80 source implementations, no stubs.
Two independent tests verify health overflow/reserve mode with refused sound,
expiration precedence and delayed grapple collection. Full port tests/fmt/strict
Clippy pass. Initial2561 actual-C cases match; orthogonal edge extension follows.

## Completion evidence

Final2,804 full-WRAM/raw-return actual-C cases pass at
`out/port/projectile-pickup-collection/20261011T063149.014278Z/verdict.json`:
1,523 preinstruction cases (including243 orthogonal targeted cases),1,280 scheduler
frames and1 real-ROM sleep control. Targeted boundaries cover grapple lifetime
385/384 and15/16 distances on both axes, radius sum19/20, signed abs8000,
all pickup kinds at exact/over/under maximum, missile maxima98/99/100 and all sound
acceptance/refusal gates. Initial2,561-case run retained at20261011T063047.877371Z.
Both retained verdicts authenticated after cleanup with verify_plm_instruction_evidence.

Both finished runs cleaned with port_cleanup.py apply/deduplicate commands.
Unsealed reclamation zero; snapshot reclamation335,675,392 +368,312,320 =
703,987,712 bytes (about671MiB).18GiB disk free. No implementer worktree or
custom disposable build remains. Full port tests/fmt/strict Clippy pass.
Offline scope; live gameplay ownership remains unproven.
