# Vertical terrain service: bounded dependency findings

Prepared source research only. This supplements the retained
`.scratch/restart-verification/next-player-motion-contract.md`; its seven owned
roots are unchanged. No implementation claim or verification acceptance.

All source citations below are relative to the pinned recovered source directory
`/home/jim/sm-decomp-restart/restart/out/scroll/native/2c9c9a3cf51a059220afac1ee04727c956e45d067e10e17405bf3adbd268fc7f/source/src/`.
Root owns adapter/registration decisions separately.

## Decisive synchronous-state requirement

`Samus_MoveDown_NoSolidColl` (`sm_94.c:1078`, 94:9763) is suitable as one native
synchronous service, provided the entire collision/PLM continuation stays on the
same native stack and current state. Full WRAM alone does not encode that stack:

- `cur_coll_amt32` is a process-global `int32 *` (`sm_94.c:14`, declaration in
  `variables.h:21`). Each vertical block check sets it to the local
  `CollInfo.ci_r18_r20`, executes the entire reaction/extension chain, then clears
  it to NULL (`sm_94.c:965–973`). `CollInfo` belongs to the traversal stack frame
  (`sm_94.c:1009,1030`). The same pointer is used by horizontal checks
  (`sm_94.c:954–962`). It is neither a WRAM address nor a durable service result.
- `PlmSetup_QuicksandSurfaceB` (84:B4C4 in `funcs.h:4490`, body
  `sm_84.c:1462–1495`) dereferences that pointer. Downward stationary/falling
  quicksand clamps bits 8..23 of the displacement to 0x0030 when their unsigned
  value exceeds 0x30, preserving the low byte and high byte; it increments the
  quicksand flag. Contact-damage index 1 can instead zero the displacement and
  return carry. A NULL pointer on the vertical path calls `Unreachable`.
- The C code does not save/restore an outer collision pointer. Reentrant
  collision replaces it and clears it on inner return, invalidating the outer
  continuation. Serialize calls using the shared native globals; require NULL on
  top-level service entry and normal exit, forbid callback-driven collision
  reentry, and never yield, retain the pointer, or copy its numeric address into
  canonical state. A future reentrant implementation needs an explicit nested
  state contract; merely adding pointer save/restore does not establish one.
- Preserve native stack locals as well as WRAM across any nested Rust hook.
  Copy current canonical WRAM into native state before the call and copy all
  131072 bytes back before the vertical root resumes feedback/displacement work.
  A service error after effects begin is fatal, without replay/fallback.

For the bounded non-PLM branches inspected, the other external dependency is the
immutable pinned `g_rom` mapping (`sm_rtl.h:42–45`, `sm_rtl.c:518–522`). Invalid
low-half `RomPtr` reads set native `g_fail`; this is outside WRAM. `Mult8x8` and
`SnesModulus` are pure functions, with byte-width inputs and defined zero-divisor
behavior (`sm_rtl.c:530–539`), not hardware register operations. The compiled
constant tables/function dispatch tables are also part of source identity.

The supplied source has a real effect hook already: `SpawnPLM` emits event 3
with header/current-block and event 4 with raw result around the original body
(`sm_84.c:194–221`). Preserve those ordered events, the effect sink/failure state,
and any other native-host words required by the existing bridge. Do not replace
them with a synthetic post-return summary. No exhaustive non-WRAM closure is
claimed for arbitrary PLM setups.

## Exact nested continuations

Traversal uses even-frame left-to-right or odd-frame right-to-left checks and
stops at the first carry (`sm_94.c:997–1035,1082`). Extension reactions alter
`cur_block_index` and redispatch in place until bit 7 of the reaction result is
clear (`sm_94.c:888–901,965–973`). The outer traversal's next cell is determined
by its own loop variable, not the terminal extension target. Preserve this order:
an early PLM mutation can change a later block's reaction in the same call.

| Reaction | Real synchronous behavior and scope |
| --- | --- |
| Empty/type 2 spike air | Types 0/2/4/6 return no collision; vertical spike air has no damage side effect (`sm_94.c:731–733,935–951`). Inside-block spike logic is a different entry. |
| Solid/types 8/C/E | Clips amount and sets Y subposition 0 for upward, FFFF for downward (`sm_94.c:693–708`). Root applies the returned amount afterward. |
| Slope/type 1 | BTS low 5 bits select square or nonsquare; square downward contact sets slope-adjusted flag and subposition FFFF; nonsquare publishes `current_slope_bts` and uses center-column/modulus and orientation checks (`sm_94.c:479–515,590–625,719–724`). Tables are compiled constants at lines 93–126 and 133–136. |
| Spike block/type A | Calls one of 16 handlers indexed by the **unmasked BTS byte**, then solid clipping (`sm_94.c:738–764`). BTS 0 can add 60 periodic damage with area-3 boss-bit gate; BTS 1/3 add 16; invincibility blocks repeated damage. Sets invincibility 60, knockback timer 10, facing-derived knockback direction (`sm_94.c:640–664`). `CheckBossBitForCurArea` only reads WRAM (`sm_80.c:66–68`). No immediate health deduction or hardware write here. |
| Door/type 9 | Sets door transition function first, reads bank8F door-list word and bank83 DoorDef. Non-room destination clips as solid and sets elevator flag when pose <9; room destination sets `door_def_ptr`, game state 9 and returns clear (`sm_94.c:873–885`; `ida_types.h:2555`). It schedules transition state, not room-loading execution. |
| Special air/block, types 3/B | Resolve common header table 94:9139 or area table 94:92D9/92E9 then bank84 header. Common special air discards SpawnPLM carry; area special air returns it; special block clips as solid only on carry (`sm_94.c:778–818`). These distinctions matter even with identical setup effects. |
| Bombable air/block, types 7/F | Header table 94:936B when BTS high bit is clear. Air ignores carry; block clips on carry. BTS high bit suppresses spawn: air clear, block solid (`sm_94.c:829–855`). |

`SpawnPLM` searches slots 39 down to 0, returns clear with no setup when full,
otherwise initializes the slot, publishes `plm_id`, and immediately invokes the
header setup (`sm_84.c:202–221`). The setup result is a bit field, not a bool;
terrain selects bit 0. It does **not** execute the PLM instruction list or
`PlmHandler_Async` (`sm_84.c:224`); do not run that later scheduler early.

Generic `CallPlmHeaderFunc` is a large switch with default `Unreachable`
(`sm_84.c:2908–3026`), not proof that every input is admitted. For example closed
gate setup directly spawns enemy projectiles and writes a five-row BTS extent
(`sm_84.c:2065–2073`); door setups can use projectile indexes (`sm_84.c:2124–2147`).
These branches need their own assets, index bounds and host continuations.

## Concrete minimum admission rules

1. Bind exact native source/ROM and service selector 94:9763; retain scalar
   signed displacement and complete WRAM input/output. Admit zero displacement
   separately: it performs no terrain scan, preserves existing quicksand/slope
   flags, derives collision from preexisting quicksand, and adds zero
   (`sm_94.c:1078–1092`). Nonzero clears quicksand/slope flags before traversal.
2. Before the Rust root's first mutation, prove bounds for the candidate row and
   every visited extension target using the C arithmetic widths, including
   `Mult8x8` truncation and 16-bit `cur_block_index`. Level and BTS indexes must
   stay below 0x3200. Reject extension cycles/nontermination unchanged; extension
   BTS is signed and offsets by one block or room width. Do not approximate
   lookup with an unbounded mathematical room grid. A conservative initial
   geometric subset may require width 1..255 and a nonwrapping in-room span.
3. Admit the reaction types in the table only with their specific requirements:
   spike-block BTS <16; door-list selected word and bank83 structure readable;
   slope tables use their native masks/widths. The generic whole-ROM mapping is
   not proof of a valid table selector. Common/area/bomb PLM tables have no
   declared lengths in this C; freeze supported index-to-header mappings from
   pinned ROM before claiming them.
4. Freeze a small PLM setup allowlist first, rather than admitting all switch
   cases. Good bounded candidates are 84:B4C4 quicksand, 84:CE37 respawning
   crumble, and 84:CDEA speed-boost block: each inspected body writes WRAM and/or
   the active displacement without nested hardware or projectile work
   (`sm_84.c:1462–1495,2169–2200`). Validate actual selected bank84 header/setup,
   exact reachable state, and their target block bounds. These are proposed
   allowed **setup functions**, not an asserted completed ROM header map.
5. Immutable preflight must account for setup mutations affecting subsequent
   cells. Checking only initial block kinds is insufficient. Initially admitting
   a constrained row/extension topology and the three bounded setups above is
   viable; allowing arbitrary PLM requires a larger explicit continuation
   proof. Do not discover unsupported setup/state by executing then rolling back.
6. Full PLM pool is a genuine clear-return path (still ordered events 3/4).
   Admission may conservatively require a validated header mapping even when
   allocation would fail. No requirement to force a free slot. Setup sees the
   latest PLM list, damage/pose state, and active collision amount.
7. Enforce native-call serialization/nonreentry and preserve effect nesting and
   failure state. No stale-WRAM callback, deferred setup, pointer serialization,
   or blanket native C retry after accepted execution.

## Small set of materially distinct actual-C Fixture pairs

These cases are proposals, not executed evidence. Keep complete WRAM and ordered
native effects, not only Y and collision.

- **Empty/zero:** nonzero through air clears stale quicksand/slope; zero with
  preexisting quicksand sets collision 1 without scanning/resetting. Distinguish
  zero with no quicksand. Confirm native pointer remains NULL.
- **Floor versus ceiling/slope:** solid downward and upward fractional contact
  exercise FFFF/0000 subposition; one square-slope downward hit exercises slope
  flag; one nonsquare orientation/center-column mismatch remains clear.
- **Order/extension:** two distinct reactive cells under a spanning Samus,
  reverse frame parity, observe which carry/effects wins. Include an extension
  to a valid solid target and unchanged refusal of a cycle/out-of-range target.
- **Spikes:** BTS1 fresh hit versus invincible hit; BTS0 area3 boss-bit off/on;
  vertical type2 spike air stays harmless. BTS16 refuses before dispatch.
- **Doors:** valid room target changes game state/door pointer without collision;
  non-room target with pose<9 sets elevator flag and solid collision. Confirm
  transition function assignment in both cases.
- **Quicksand setup:** actual ROM-selected 84:B4C4 header with free slot,
  direction3/y-dir0 and an amount whose bits 8..23 exceed 0x30; assert exact
  amount masking, PLM slot writes, quicksand and final collision. Contact-damage
  index1 zeros amount/returns carry. Full pool skips setup and leaves amount
  unchanged; trace still contains events 3/4.
- **Breakable block:** actual selected speed-boost setup with qualifying and
  nonqualifying state changes level data versus frees the new slot/returns carry;
  crumble downward changes block/timer to4 then returns carry. Common-special-air
  versus special-block carry interpretation is a separate material distinction.

## Follow-up: finite original-ROM mappings and dispatch packet

Read directly with `xxd -g1/-g2 -s OFFSET -l LENGTH` from
`/home/jim/sm-decomp/rom/sm.sfc`. `sha256sum` returned
`12b77c4bc9c1832cee8881244659065ee1d84c70c3d29e6eaf92e6798cc2ca72`.
This is the pristine input ROM, not a dump of native prepared `g_rom`.
LoROM file offset is `((bank & 0x7f) << 15) | (address & 0x7fff)`.
All table/header values below are little-endian words.

| Sealed terrain selector | Table word(s), file offset, bytes | Header bytes (setup, instruction pointer) | Exact setup |
| --- | --- | --- | --- |
| Type B, BTS 00, any valid area | 94:9139 at 0A1139: `44 d0` | 84:D044 at 025044: `37 ce f9 c9` | 84:CE37 respawning crumble |
| Type B, BTS 62, any valid area | 94:91FD at 0A11FD: `30 d0` (9139 + 2*62) | 84:D030 at 025030: `ea cd 51 c9` | 84:CDEA respawning speed-boost block |
| Type 3, area 4, BTS 80 | 94:92E1 at 0A12E1: `59 92` (92D9 + 2*4); 94:9259 at 0A1259: `2b b7` (index 0) | 84:B72B at 02372B: `c4 b4 e3 aa` | 84:B4C4 quicksand surface B |
| Type B, area 4, BTS 80 | 94:92F1 at 0A12F1: `59 92` (92E9 + 2*4); same 94:9259/header | Same B72B header | Same 84:B4C4 setup, but carry invokes solid clipping |

These exact entries suffice; no whole-table length assertion is needed. Nearby
bytes show area4 indexes 1/2 also B72B, but those are unnecessary for the finite
packet. For quicksand amount masking use **type3** first: area-special-air keeps
setup carry without the additional solid-clipping transformation. A typeB case
can separately exercise that transformation.

Concrete bounded fixture geometry, requiring no ROM edits: width16, height16,
Samus X=40, Y=40, X/Y radii8, zero subpositions, direction3, amount00010000.
The computed destination bottom is48, row3, horizontal span32..47 is one column
(column2), so either frame parity checks exactly **block50 (0x32)**. Put the
selected type/BTS at index50 and keep the surrounding room cells valid air.
The needed cell slices are level word WRAM10066..10067 and BTS byte16434; the
full level/BTS room spans remain indexes0..255. PLM slot39 is selected first if
its header is zero: it receives byte block offset0064. These three setups only
touch this block or ordinary bounded PLM/Samus fields; no extra terrain extent
or projectile record is required. Use area4 for quicksand, `samus_y_dir=0`,
contact damage0 for clamp to00003000; contact damage1 exercises zero/carry.
Crumble uses downward direction3; speed block uses counter0400 versus a
nonqualifying counter/pose. Preserve valid canonical unrelated fields.

For a finite parity case use X=48 with radius8: columns2/3 (blocks50/51), same
row3. Place a non-carry quicksand cell and a carry-producing crumble cell; the
frame parity determines whether quicksand setup happens before traversal stops.
An extension case may map block50 to51 with type5/BTS+1 and put the selected
terminal reaction at51. This is a sealed fixture topology, not a request to
build a second general bank94 validator. Supported input can be source-valid
canonical game state plus these sealed fixtures; arbitrary-memory fuzz is
outside this dispatch packet. Earlier admission rules describe the bounds that
must hold, not a mandate for a new independent validation framework.

**Original/prepared distinction:** native-host `src/lib.rs:130–137` authenticates
the pristine input, but `src/shim.c:39–42` boots `SnesInit`, whose pinned
`sm_cpu_infra.c` applies patches (for example lines855–865 and898–910). No native
boot or prepared-ROM dump was performed for this note, so runtime equality of
the listed slices is not asserted. The integrator can require those exact small
slices when freezing prepared ROM, preserving pristine and prepared identities
separately. No ROM mutation was made by this research. Original-ROM execution
parity is still separate from these original-ROM data facts.

**Existing scheduled Rust callbacks:** for these three selected setup functions,
terrain invokes synchronous original `SpawnPLM -> CallPlmHeaderFunc -> setup`
bodies and reaches no existing scheduled Rust gameplay callback. The current
restart file
`/home/jim/sm-decomp-restart/restart/native-host/src/item_instruction_prepare.rs:6–45`
patches `CallPlmInstr` and `CallPlmPreInstr`; it does not patch
`CallPlmHeaderFunc`. PLM allocation initializes an instruction pointer, but never
executes its list here. Current `prepare.rs:286–287` wraps SpawnPLM for ordered
effects only. A retained control explicitly distinguishes this setup boundary
from item scheduling at
`native-host/examples/item-instruction-host-control.rs:414–420`; that source is
supporting design evidence, not a newly executed check. Host effect observation
is not Rust gameplay ownership. This conclusion covers the chosen finite
set and bounded non-PLM branches, not every possible PLM header.

## Gaps and completion evidence

No native adapter selector/copyback/host-word implementation or admission
implementation is inspected here; root handles that mapping. Selected original
ROM entries are now established above; prepared-ROM slices remain to bind. Arbitrary
PLM setup/hardware/projectile closure, original-ROM parity, and live route
exercise are not established. No source change or build is implied by this note.

Read-only source research except this note. No builds, tests, game runs, claims,
processes or evidence captures were created. Run cleanup: no run directory,
no cleanup command applicable, reclaimed **0 bytes**. Remaining available space
observed with `df -B1 /home/jim/sm-decomp`: **100057858048 bytes**. No new retained
evidence dependencies; existing pinned source and retained contract remain.
