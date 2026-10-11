# Host-aware spore spawning and movement closure
Status: resolved
Blocked by: 82
Triage: ready-for-agent

Port DC77 hostpoint-writing spawn opcode and DC8D spore initializer, connecting
existing DCEE movement/DD46 spawner countdown through host-aware scheduling.
Use actual script DC06 and mandatoryhostsidecars/fullWRAM/rawreturns, preserve
same-frame descending allocation ordering, refusal and exact originalaliases.

## Implementation

DC77 writes suppliedprojectile coordinates into explicit hostpoint then allocates
DE7A/DC8D. Initializer writes sourceX intoF, coordinates andgfx512. ExistingDCEE
phase movement honors Fbit80, signedtablebytes andY768 deletion. ExistingDD46
countdown plus actualscriptDC0E now spawns through hostawareinstruction/frame
execution. LegacyDC77refusesbeforewrites and DC8Drequirescontext beforeallocation;
fullslot keeps hostpointupdates andparameterwrite as C.

Generator asserts actualDC0Eopcodebytes. Schedulerfixtures use actualDE88spawner
ID/DD46preinstr with current countdown5; phasefixtures cover all256lowphasebytes,
X127/128/255 andsigned/wrappedYboundaries. Initial768cases passed; activeID corrected
from arbitraryDE96toDE88 for finalscope. Independenttest verifies sourceX/F/gfx and
hostpoint preservation. Fullcratechecks pass; no liveclaim.

## Completion evidence

Final768actual-C cases (256spawnopcode,256frame,256phasepreinstr) pass fullWRAM,
rawreturns andhostsidecars at
`out/port/host-spore-spawning/20261011T065229.867668Z/verdict.json`.
ActualspawnerIDDE88 andscriptDC0E verified; newlyallocatedlower/higher/fullbanks
andnativeDCEEmovement included. Bothretained768case verdicts authenticate after
cleanup. Fullporttests/fmt/strictClippy and7independenthosttests pass. Bothfinished
runs cleaned apply/deduplicate:0unsealed,109,051,904bytes each, total218,103,808bytes
(~208MiB)reclaimed.15GiBdiskfree; noimplementerworktree/disposablebuild. Scope
remains offlinehostawareclosure; liveownership unproven.
