# Host-aware projectile instruction drop spawning
Status: resolved
Blocked by: 81
Triage: ready-for-agent

Thread host context through instruction/scheduler execution. PortAB8A/9270/8C68/
DC61 point-writing callers through actual SpawnEnemyDrops->F337/EF29 chain,
including area-selectedheader, rawsourcegfxoffset and fullslots hostwrites.
Actual-C fullWRAM/rawreturns/hoststate, independentchecks and cleanup required.

## Implementation

AB8A/9270/8C68/DC61 write externalpoint, select actualenemyheader, retain hostheader
through SpawnEnemyDrops, and allocateF337/EF29 with rawsourcegfxbyteoffset as C.
Internal helper accepts this unalignedprojectileslot offset without loosening
existingpublicalignedenemy API. Hostaware scheduler now threads mutablecontext
through instructionexecution. Legacydirectinstructions reject beforewrites;
legacyframe retains partialunsupportederror semantics for hostinstructions met
after earlier slots mutate, so callers must choose explicitcontext API. No broad
atomicframe/refusal claim beyond prior940Epreflight.

Actual script addresses: AB8A@AB82 has headersEF3F/EFBF followeddelete;9270@901B,
8C68@8C60,DC61@DC54 have timedmaps followed sharedGOTO84FC/delete. FirstAB8Alookup
8C51 was codeoperand and is superseded by actualscript fixture. Intermediate
320/512cases passed comparison but final536real-script/edge scope follows.
Independentfullslot test verifies externalpoint/headerwrites, return and exact
unchangedWRAM; directabsence refusal preservesWRAM. Fullcrate checks pass.

## Completion evidence

Final536actual-C cases (268opcode,268same-frame scheduling) match fullWRAM,
rawreturn and externalhoststate:
`out/port/projectile-host-drop-instructions/20261011T064908.856459Z/verdict.json`.
Realopcode/headercontinuations, area choices, allocationexhaustion/lower/higher
slots, coordinatehighbit/wrap, fullinventory and zeroRNGretryseed99 covered.
Generator pins actualscriptaddresses and assertsopcodebytes. Finalverdict
verifiedwithverify_plm_instruction_evidence aftercleanup; fullcrate checks pass.
Sixindependenthosttests pass (twoadded). Allthreefinishedruns cleaned viautility
apply/deduplicate;0unsealed,50,331,648 +75,497,472 +81,788,928 =207,618,048snapshot
bytes reclaimed (~198MiB).16GiBdiskfree; noimplementerworktree. Historical
intermediates retained; finalreal-script currentevidence authenticates. No liveclaim.
