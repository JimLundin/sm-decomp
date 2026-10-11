# Mother Brain room turret host lifecycle
Status: resolved
Blocked by: 83
Triage: ready-for-agent

PortBFDF full roomturret preinstruction: exactscreenchecks, extraenemyvar1D
destruction, hostpoint/dustallocation, timers/rotation and bulletspawning. Connect
existing nativeallocation/rotation/bulletmovement via host-awareframes. Actual-C
fullWRAM/rawreturns/hoststate, independenttests/fullchecks/cleanup required.

## Completion evidence

BFDF now preserves exact signed screenpredicate, actualRam7800var1D offset783A,
destruction/deletion beforehostpoint/dustallocation, postdecrementtimers, RNG
rotation order and bulletallocation. Existing rotation/bulletmovement execute
through hostawareframe dispatch. LegacyBFDF refusesbeforewrites; enabledlegacy
frames preflightBFDF alongside940E. Otherhostrequiredscript errorsemantics remain
as previouslydocumented.

577actual-C cases (288preinstr,288frames,1control) pass fullWRAM/rawreturns/hoststate:
`out/port/mother-brain-turret-host-lifecycle/20261011T070149.116939Z/verdict.json`.
Covers signedcamera/positionedges, xscreen263/264/yscreen255/256, lifedestruction,
timerboth1/RNGrotation, fullbankreuse, newlower/higherbullets and dustspawn.
Generator assertsactualE509/E468dust andC18C/BF59bullet definitionbytes.
Twoindependenttests show offscreendead turretpreserveshostpoint and offscreenliving
turretpreservestimers/RNG. Fullporttests/fmt/strictClippy pass. Finalverdict
authenticated afterutilityapply/deduplicatecleanup:0unsealed,104,464,384snapshot
bytes reclaimed (~100MiB). Noimplementerworktree/disposablebuild. Offlineonly.
Remainingdiskspace12GiB. Rectangle/r22 consumers andliveverification remainopen.
