# Ridley fireball host-aware impact scheduling
Status: resolved
Blocked by: 80
Triage: ready-for-agent

Thread explicit mutable ProjectileHost through preinstruction/frame dispatch.
Implement940E terrain impact, hostpoint writes,947F explosion allocation and
sound. Legacy preinstruction and enabled frame APIs refuse known hostrequired
940E before any WRAM writes. Actual-C fullWRAM/rawreturns/hoststate, independent
checks, cleanup/authentication required. No broad live ownership claim.

## Implementation

940E now checks horizontal terrain before vertical, deletes impacted fireball,
updates external hostpoint only when Fzero, allocates965E/9650 stationary947F
explosion and queues2B sound. Fulloccupied bank still has the just-deleted slot
available; fixtures cover reuse, not nonexistent fullslot refusal. Hostaware
preinstruction/frame APIs retain context through direct calls and descending
scheduler. Legacy enabledframes preflight allactive940E before anyglobal/slot
writes; direct legacy940E refuses beforecollision. Other instruction opcodes
retain existing context behavior; no broad instructioncontext claim.

ActualC513cases passed. First compile rejected duplicatedtypedcontext because
standalone offlinefamily reloadedallocator; hostwrapper now aliases sharedmemory
allocator module, retaining context updates instead of copying/discarding them.
Allfullcrate checks and4independenthosttests pass. Finalorthogonalcases pending.

## Completion evidence

Final585actual-C cases pass (292preinstruction,292frame,1control), fullWRAM,
rawreturns and16byteexternalhoststate at
`out/port/ridley-fireball-host-impact/20261011T064404.016491Z/verdict.json`.
Includes horiz/vert/bothimpactpriority, negativevelocities, Fsuppression, fullbank
reuse and descendingnewspawnordering plus accepted/fullquota/state/frozen/debug
soundgates. Both513initial and585final verdicts authenticate aftercleanup.
Fullport tests/fmt/strictClippy pass;4independenthosttests andhosttamperregression
pass. Cleanupallthree finishedruns applied/deduplicated: diagnostic0bytes,
initial189,661,184 andfinal204,603,392snapshotbytes reclaimed, total394,264,576
bytes (~376MiB),0unsealed.17GiBdiskfree. Noimplementerworktree/disposablebuild.
Explicitcontext caller path now native offline; liveadmission remains open.
