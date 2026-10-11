# Pirate rectangle and laser host allocation
Status: resolved
Blocked by: 84
Triage: ready-for-agent

PortA009hostpoint/r22laser andA098rectangleclaw initialization through existing
nativeflight/return scripts. Preserve currentenemyROM properties/parameter/gfx,
rectwrapping andsoundrefusals. VerifyactualC WRAM/returns/hostsidecars andreal
scriptpointers withindependenttests/fullchecks/cleanup. No liveclaim.

## Implementation

A009 now selects9F41/9F7D script by nonzerohostr22, copies hostpoint, reads actual
currentenemyA0header+6 properties with1000flag, setsEparameter1 and queues67sound.
A098 copies rect x+width/y+height withu16wrapping, selects9FB9/9FE1 bynonzeroparam,
setsE2048/F1. ExistingnativeA05Bnoop/A050/8161scriptcontrol andleft/right laser/claw
flight execute. Currentenemy validation/ROMextent appliesonlylaser; claw needs
noenemy state. Bothinitializers requirehost beforewrites (fullslotparam rule
unchanged). Explicitwithgfxapi remainsfuturecallerwork; thisscope usesroomgfx
entry andcanonicalinitializerbehavior only.

Initial513cases pass; duplicateA05Bpattern addition removed (it was alreadycovered),
source stablefullchecks pass. Final545cases include32additionalnativeflight/
returnframes. Independentclawtest verifies rectangle wrapping andinvalidcurrent
indexirrelevance. No liveclaim or crossbankpiratecaller claim.

## Completion evidence

Preceding545actual-C cases (256allocations,288independently seeded script-start/flight
frames,1control) pass fullWRAM/rawreturns/all8hostwords:
`out/port/pirate-host-context/20261011T070812.146131Z/verdict.json`.
This verifies allocation and independently seeded native scripts/flight; no
multiframeallocation-to-flight trace claim. Rect/r22context fields are immutable
in everycomparison, notonlypointfields. Allocation soundnormal/state/frozenrefusal
cases covered in preceding545cases; final549extension adds explicitquota/fullring
andpoisonedsoundstate. Fullporttests/fmt/strictClippy pass;10host independent
tests include newrectanglewrapping/noenemydependence example.

Bothcompletedruns cleaned utilityapply/deduplicate:0unsealed;130,154,496 and
141,688,832 snapshotbytes reclaimed, total271,843,328 (~259MiB). Final545verdict
authenticated aftercleanup; earlier513run historicalsourcehash. Noimplementer
worktree/disposablebuild. Rootrequested review-ready handoff withoutcommit.

## Final review handoff

Final549actual-C cases pass at
`out/port/pirate-host-context/20261011T070943.666604Z/verdict.json`.
Adds fourtargeted soundquota/fullring/wrappedring cases with poisonedquota/queue,
plus explicitflightfixturehostvalues. Finalverdict authenticates aftercleanup.
Finalrun apply/deduplicate reclaimed0unsealed and142,213,120snapshotbytes; total
allthreecompletedruns414,056,448bytes (~395MiB).8.7GiBdiskfree. Sourceunchanged
sincefullcratechecks; noadditionalbroadtestsneeded. Review-ready ownedfiles:
enemy_projectiles.rs, projectile_host.rs, fixturebuilder/manifest andticket85.
Status resolved afterverification/cleanup; rootwillintegratecommit perinstruction.

Integrated on decomp-port-integration. Root independently reauthenticated the
final 549-case verdict after cleanup. Allocation, script starts and independently
seeded flight frames are verified; live pirate caller ownership remains open.
