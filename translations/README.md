# Translated actor source candidates

Eight complete actor scopes, 159 definitions, are translated and unverified.
Six existing sources are registered as restart library targets; Mini Crocomire
and MiniDraygon/Evir remain source-only pending wiring. No production runtime
registration or native adapter is provided. The six Cargo manifests are preserved exactly;
Skree and Rinka are standalone std-only modules without Cargo manifests.
No compilation, test or native execution verdict is asserted here.

Sources were frozen from baseline 4aa9eb660e7b02dfc176c3d84ab027161630e11d
against recovered snesrev/sm revision
578f90b3cc49557bb70060ad033bb90b8cf8ac50. A2 source SHA256:
d62422845f0fee20c929ed7a635a0eb397d376a6b2d62b9c0aab21e3c18a315e.
The public root inventory binds each admitted Rust/Cargo file; private dependency
inventories, rejected candidates, captures and upstream downloaded trees are
excluded. Applicable upstream MIT/Opus notices elsewhere in this repository
are preserved; original contribution licensing does not replace them.

| Actor | Complete recovered scope | Source |
| --- | --- | --- |
| Skree (10) | Instr_1, Init, CallSkreeFunc, Main, Func_1..5, Shot | skree/enemy_skree.rs |
| Beetom (43) | Instr_1, Init, Main, Touch, Shot, CallBeetomFunc, Func_1..37 | beetom/lib.rs |
| Rising/Falling Platform (19) | Init, Func_1..13, CallRisingFallingPlatformFunc, Main, Touch, Shot, Powerbomb | platform/lib.rs |
| Rinka (26) | Init/Init2/Init3, Rinka_1/2, CallRinkaFunc, Main, Rinka_3/4, B85B/B865, Rinka_6..10, Frozen/Touch/Shot/Powerbomb, B960, Instr_B9A2/B9B3/B9BD/B9C7 | rinka/enemy_rinka.rs |
| Fireflea (12) | Init, Func_1..6, Main, Touch, Powerbomb, Shot, Common | fireflea/src/lib.rs |
| Bouncing Goofball (10) | Init, Func1, Main, State0/State1, SamusCloseX, Func2/Func3, Instr_88C5/88C6 | bouncing-goofball/src/lib.rs |
| Mini Crocomire (14) | Instr_897E/8990/899D, Init, Main, Func1..4, PreInstr5/6, Func7..9 | mini-crocomire/src/lib.rs |
| MiniDraygon/Evir (25) | Instr_1..4, Init, Main, Func_1..13, CallMiniDraygonFunc, Touch/Powerbomb/Shot, EvirProjectile_Init/Main | minidraygon/src/lib.rs |

Each execute seam uses exact 128KiB canonical WRAM, explicit little-endian
words and aligned enemy slots, immutable caller-authenticated ROM and typed
synchronous services. Preownership refusal leaves state unchanged; accepted
errors are terminal and retain prior effects. Native fallback after ownership
is prohibited. Real adapters must establish whole-entry admission, callback
copyback, stable/global CURRENT distinctions, allocation/generation/reentry,
ROM domains and ordered foreign effects. No permissive service default exists.

Skree retains terrain, audio, particle allocation, shot and death as foreign
services; its A3:C69C animation words and captured/current k distinctions require
review. Existing Skree particles are outside this scope. Beetom retains
collision, random, proximity, audio and A8 damage services; search, signed
timers, crawl/jump and separate ROM table halves remain acceptance targets.
Only seven explicit-k helpers accept a slot different from CURRENT.

Platform preserves all fourteen shared dispatcher branches, including seven
foreign HorizontalShootableShutter requests. That neighboring actor is not
claimed. Linear speed ROM rows, player carriage, overlap, screen and sound
services require admitted native behavior. Rinka requires eleven A2:B75B
position triples, shared slot-zero count/death and occupancy overlay aliasing;
trig, movement, damage, deletion and external projectile spawn-point assignment
remain services with exact callback and spawn order.

Fireflea uses speed A2:8187 (65 rows) and amplitude A3:8D1D (eight words), with
native trig and damage/death dependencies, signed/wrapping motion and global
darkness effects. Goofball uses corrected A2:86DF/86EF/8701 runtime offsets
1106DF/1106EF/110701. Independent review rejected the previous A1-bank offsets;
those rejected bytes are excluded. The corrected source includes a literal
A1/A2 discriminator test, but that test has not been executed. Proximity/audio,
calibration termination, instruction effects and foreign state remain gates.

Formatting/source inspection does not prove recovered behavior. Required next
gates are independent full-body review, compiled source/dependency closure,
formatting/Clippy, actual-C pairs/chains across every entry, full WRAM and ordered
effects, sign/carry/table/state boundaries, refusal/terminal/reentry controls,
then separately live replay and original-ROM acceptance. Integration and shared
progress stages require the production owner's reviewed registration.
