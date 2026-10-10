# Port system state family
Status: claimed
Blocked by: None
Triage: ready-for-agent

Port ReleaseButtonsFilter, PrepareBitAccess, boss/event set/clear/check,
Multiply16x16, HandleFadeIn/Out, ClearUnusedOam, ClearOamExt and three map-fill
routines plus WriteLotsOf0x1c2f. Own crates/port/src/system.rs and related tests
and tools/port/system.json. Differential verification depends on ticket 01.
Declare valid domains and preserve scratch writes, signed comparisons and aliasing.
