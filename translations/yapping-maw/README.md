# Yapping Maw translated source

Complete32-definition scope: YappingMaw_Instr_1..7, Init/Main, Func_1..19,
Touch/Shot/Frozen and addressless CallYappingMawFunc. Dispatcher contains exactly
A8:A235/A28C/A445/A68A. Status: translated/unverified; library registration,
production runtime/native adapter and behavioral acceptance remain pending.
Independent whole32-body review accepted the corrected bounded source; five
authored literal model tests have not executed.

Recovered inventory revision578f90b3cc49557bb70060ad033bb90b8cf8ac50,
baseline4aa9eb660e7b02dfc176c3d84ab027161630e11d. Actual sm_a8.c SHA256
c2a0b15844752c61d56daa61093781339bd796edeb2c9fc3460b3c173ffada4f.
Exact Rust SHA256 cc9d4537bae17e8a2b70a5cf968efa09114ce0caaffcf2806904ec52aa50faf5;
Cargo e0f63b837f65f9456b9f7e1bacd567bcf6f03c9e7e8c99c99faf654ac8d7cd00.
Private input/dependency inventories and the rejected original source are excluded.

Safe std-only execute uses exact128KiB WRAM, aligned64-byte slots below800,
actor/three extended overlays, bounded even four-body projectile indices and
mouth sprite. Immutable runtime ROM domains are A8:A097/A0A7 instruction and
offset tables, A0:B1C3 signed sine and A2:838F quadratic curve. Quadratic D>>1
indexing, wrapping32-bit extension and wrapping16-bit acceleration are explicit.
Native service admission must cover complete linked objects and future copybacks.

Typed services retain body projectile spawning, sprite creation, screen/geometry,
Samus input/capture, shot/frozen damage, audio and multiplier MMIO. Shared
EprojInit_YappingMawsBody remains foreign and uncredited. Init spawns3/2/1/0,
then the mouth, preserving actor44 loop-controller feedback; a changed controller
terminates rather than changing scope. Full WRAM publication, slot/controller,
alias/allocation/lifetime/reentry/generation and ordered effects are obligations
of the eventual host. Refusal leaves state unchanged; accepted errors retain
effects and prohibit native fallback.

Review corrections distinguish VRAM tile offset20 from layer22 and frozen timer
26 from VRAM20 throughout spawn and frozen behavior. Literal test inputs use
different field values to detect regression. Capture/release E91D exceptions,
four linked death/frozen palettes, asymmetric geometry/angle limits, signed
sine product rounding and unconditional WRMPYA write (even multiplier0) remain
explicit source behavior. Corrected whole-source review is not actual-C parity.

Pending gates: compiled closure/Clippy, execution of model tests, full32-entry
actual-C WRAM/effect chains and sign/carry/table/capture/spawn/death/freeze
boundaries, refusal/terminal/reentry controls, library/runtime registration,
native adapter and separately live replay/original-ROM acceptance. Root MIT
and retained upstream notices govern their respective contributions.
