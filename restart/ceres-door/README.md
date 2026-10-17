# Ceres door actor

Dependency-free Rust translation of the complete `Enemy_CeresDoor` E23F actor,
parameters 0–6, and BA48's one-shot solid-block setup. Mutable gameplay state
lives in the supplied full WRAM copy. No generic enemy or PLM interpreter is
introduced here.

`run(entry, index, instruction, ram, rom, host)` returns `Outcome::Handled`
with `Output::Done`, `InstructionOffset(u16)` or `SetupStatus(u8)` as appropriate.
The thirteen entries, in native registration order, are Init, Main,
Instructions 1–8, SetTangible, SetInvisible and SolidSetup. SetTangible and
SetInvisible are the two shared CeresSteam instructions admitted only for E23F.
Generic enemy dispatch and instruction stepping remain native. Private calls
between these routines do not create additional scheduled invocations.

The enemy index is a 64-byte-aligned slot offset below `$800`; Init/Main also
require it to match `cur_enemy_index`. The shared callbacks require actor E23F,
bank A6 and parameter 0–6. SolidSetup takes an even PLM byte index below 80 and
requires header BA48. Instruction inputs/results are bank-A6 offsets, never
native pointers. Incorrect actor/header/parameter identity refuses without
writing; malformed admitted state returns an owned error. All locally
detectable input errors precede mutation. The native transaction owns commit,
unchanged refusal and fatal error handling; this crate does not copy full WRAM.

The complete main dispatch includes F76B, F770, F7A5, F7BD, F7DC and F850.
Palette copies, ordinary VRAM descriptors, Mode7 descriptors, quake state,
random advancement and sound queues are private Rust implementation. Generic
queue consumption, DMA, audio playback, rendering, collision and room loading
remain native. BA48 setup changes four vertical block types to `$8000`,
preserving tile bits and BTS; it has no persistent door handler.

The sole foreign gameplay operation is
`Host::spawn_eproj_with_room_gfx(ram, 0xE509, 3|12, point)` during escape puffs.
The host must publish the complete private WRAM and the supplied `[x,y]` into
the actual external `eproj_spawn_pt`, synchronously execute the recovered C
routine, and copy WRAM back before returning. The host counts requests before
execution and completions after return. Failure never permits C retry. Slot 0's
E/F counters are intentionally shared even when another slot owns the actor.
The updated random value and counters reach the foreign call before the sound
request; sound is queued only after successful foreign return.

## Source identity

Recovered C: `snesrev/sm` revision
`578f90b3cc49557bb70060ad033bb90b8cf8ac50`, archive SHA-256
`69c78831351112887575f852dadf0281dd5c23cef890b07a268ae6cf60aa9193`.

Source inspected under the retained prepared library directory
`restart/out/scroll/native/2c9c9a3cf51a059220afac1ee04727c956e45d067e10e17405bf3adbd268fc7f/source/`
in the restart integration worktree. These are exact retained-source hashes;
some files include the existing host's unrelated observational patches.

| Source | Responsibility | SHA-256 |
| --- | --- | --- |
| `src/sm_a6.c:4580` | Eight door instructions; shared Steam4/5; Init and all private main states/helpers | `36ce5f36896e6a28555fba833be68fc189bf1a35b17a64cab04cd3a4e34e6b96` |
| `src/sm_84.c:1733` | `sub_84B9F1` solid setup | `7a7da7a5c1a35033c8cac6d608cc9a3c043c52347504ab0f8609ac2c6acc095a` |
| `src/sm_80.c:22` | `NextRandom`, `QueueMode7Transfers`, SFX2/3 Max6/internal queue semantics | `171d73158e0e7323118b4bb142d4e7c79f3d74cdc4129a667aa8873d7eb785b7` |
| `src/sm_a9.c:4256` | Active/target palette copies | `82b84aaa333cebca6f6913684fd41b86bfa36a4968aea8ec60b5d6dc9ecc074f` |
| `src/enemy_types.h:320` | CeresDoor actor layout | `251af063d550e9e9ade1de9840df7617c8cb3648bbe241587a654b3ea3085f2d` |
| `src/ida_types.h:147` | EnemyBase layout and named constants | `12974c5708740164692dac151104020b53e8316e7378521c974ddf602d1f28ff` |
| `src/variables.h` | WRAM locations | `e2ce28d07ceff91129e93417099f33f20f161c37e95e745c575d0e30b3fd5585` |

ROM lookup tables remain in the supplied authenticated native ROM view: A6:F52C,
F72B, F840 and F900; initial palettes F4EE/F50E; animated palette words start
at the wrapped, potentially odd offset `2*(frame & 0x38)-0x078F`. Init's
parameter-2 upload is exactly 1,024 bytes from B0:C400 to VRAM 7000.

The upstream license is retained in `LICENSE.upstream.txt`. See
`VERIFICATION.md` for the checks and their limits.
