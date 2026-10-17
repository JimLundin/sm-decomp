# MiniDraygon / Evir translated source

Complete25-definition scope: MiniDraygon_Instr_1..4, Init/Main, Func_1..13,
CallMiniDraygonFunc, Touch/Powerbomb/Shot and EvirProjectile_Init/Main.
Independent full-body source review accepted the conservative linked-slot/ROM/
service domain. Status remains translated/unverified/unregistered; no actual-C,
native execution, live replay or original-ROM verdict exists.

Recovered inventory revision578f90b3cc49557bb70060ad033bb90b8cf8ac50;
actual sm_a8.c bodies141..400 were individually reviewed. The inventory revision
is not inferred from a vendored dependency Git checkout. Exact Rust SHA256
e4627224f6cd349746671bd2062c6efb129dee5a045826fecc6f1c0c71600474;
Cargo ea92349061f6911133d389883b2b5973b833fade70f28949f280d02e1ebf3243.
Private dependency/candidate inventories and local evidence payloads are excluded.

Canonical128KiB WRAM, aligned32-slot enemy table and bounded linked children are
required. Projectile arming needs two preceding slots. Speed uses A2:8187/file
110187; Init reads an unaligned linked actor parameter high byte and halves it,
while reload uses full current parameter high byte. Trig services must preserve
independently negated high/low bit halves. Drawing/spawn aliases, captured slots
across callbacks, frozen projectile dispatch and linked death/freeze order remain
explicit. Angle/proximity/trig/offscreen/damage are foreign synchronous services
with full WRAM copyback; no replacement engine or permissive adapter is included.
Refusal is unchanged before ownership; accepted errors are terminal with prior
effects retained and no C fallback.

Formatting and metadata-only compilation were reported; no Cargo/Clippy/tests
or behavioral execution acceptance is claimed. Next gates: compiled closure,
all25-entry actual-C pairs/chains, dispatch/alias/timer/carry/trig boundaries,
ordered effects/foreign WRAM, linked lifecycle, refusal/terminal/reentry controls,
production registration and separate live/original-ROM verification. Root MIT
and preserved upstream notices govern their respective contributions.
