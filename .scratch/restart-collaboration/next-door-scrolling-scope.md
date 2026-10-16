# Successor scope: directional door scrolling and IRQ completion

Status: actionable read-only successor recommendation complete; root chooses after ticket09
Date: 2026-10-16
Research owners: `/root/transition_boundary` (pinned-C closure), `/root` (Rust reuse, host composition and assignment)
Writable ownership for this packet: this file only

Recommend one **13-routine directional door-scrolling scope** with three
external entry points: `FixDoorsMovingUp`, `DoorTransitionScrollingSetup` and
`Irq_FollowDoorTransition`. Own all four directional setups and movement paths,
plus their two local bookkeeping helpers. Keep the generic IRQ dispatch,
register writes and scheduling in C, and call the existing scrolling subsystem
at its current nested boundary.

This is the complete directional boundary intentionally left outside ticket
09's room-transition coordinator. It contains no coroutine of its own. The
coordinator, collision entry, room loading and background interpreter remain in
their current ownership. This packet reuses
[the transition packet](next-transition-scope.md) and does not repeat its
27-routine audit. No implementation, build, test or gameplay run was performed.

## Exact source

Recovered C revision: `578f90b3cc49557bb70060ad033bb90b8cf8ac50`.
Every C/header citation below resolves under:

`/home/jim/sm-decomp-restart/restart/out/scroll/native/2c9c9a3cf51a059220afac1ee04727c956e45d067e10e17405bf3adbd268fc7f/source/src`

[sm_80.c](/home/jim/sm-decomp-restart/restart/out/scroll/native/2c9c9a3cf51a059220afac1ee04727c956e45d067e10e17405bf3adbd268fc7f/source/src/sm_80.c)
SHA-256: `171d73158e0e7323118b4bb142d4e7c79f3d74cdc4129a667aa8873d7eb785b7`.
This retained source already contains the prior scrolling hooks. Line numbers
are for these exact bytes, not the active ticket-09 generated source.

The root glossary/domain instructions were read for the preceding packet;
the main checkout has no `docs/adr/` directory. Terminology and source/ROM
identity from that packet are retained here.

## The complete external boundary

Source-wide call-site inspection finds only these callers outside the proposed
13-routine set:

| Root | External caller | Exact placement |
| --- | --- | --- |
| `FixDoorsMovingUp` | `DoorTransitionFunction_FixDoorsMovingUp` | `sm_82.c:3932–3936`: called only when `(door_direction & 3) == 3`, before the new-room setup phase. |
| `DoorTransitionScrollingSetup` | `DoorTransitionFunction_SetupScrolling` | `sm_82.c:3950–3960`: called after BG2 offsets/reset fields and the direction-dependent counter preparation. |
| `Irq_FollowDoorTransition` | `IrqHandler_18_VerticalDoor_EndHud` | `sm_80.c:1180–1192`: called after TM/color-math writes, only while completion bit `0x8000` is clear, and before scheduling IRQ phase 20. |
| `Irq_FollowDoorTransition` | `IrqHandler_24_HorizDoor_EndHud` | `sm_80.c:1207–1220`: same completion-bit guard, after TM/color-math writes and before scheduling IRQ phase 26. |

All three roots are synchronous `void` routines (`funcs.h:48`, `61–62`). The
four movement routines return `uint8` (`funcs.h:11–14`), but only the private
IRQ dispatcher consumes that completion result. Setup and upward correction
deliberately ignore it. Keep movement completion distinct from native-hook
ownership/acceptance; the public IRQ root itself has no return-value protocol.

Concrete hook placement: wrap each of the three root **function bodies before
its first mutation** (`sm_80.c:2290`, `2306`, `2389`), and retain a C-reference
body for the unowned/reference mode. Existing coordinator service calls then
reach the same setup/correction symbol, and both existing IRQ wrappers reach
the same follow symbol. No new call belongs in the host-step loop or the
coordinator's coroutine wait. The four direction tables/private calls should
be implemented inside the scope rather than independently intercepted as
unrelated public routines.

`Irq_FollowDoorTransition` has **no internal completed-bit guard**. That guard
belongs to its two existing IRQ callers. Preserve the exact root body and
caller guards instead of making the root silently idempotent after completion
(`sm_80.c:1189`, `1217`, `2389–2394`).

## Exact source ownership set

| Role | Symbol | Source / address when stated by C |
| --- | --- | --- |
| Root | `FixDoorsMovingUp` | `sm_80.c:2290`, `80:AD1D` |
| Root / private setup dispatcher | `DoorTransitionScrollingSetup` | `sm_80.c:2306`; direction table at `2299` |
| Private setup | `DoorTransitionScrollingSetup_Right` | `sm_80.c:2312`, `80:AD4A` |
| Private setup | `DoorTransitionScrollingSetup_Left` | `sm_80.c:2325`, `80:AD74` |
| Private setup | `DoorTransitionScrollingSetup_Down` | `sm_80.c:2338`, `80:AD9E` |
| Private setup | `DoorTransitionScrollingSetup_Up` | `sm_80.c:2351`, `80:ADC8` |
| Private bookkeeping | `UpdatePreviousLayerBlocks` | `sm_80.c:2368`, `80:AE10` |
| Private bookkeeping | `UpdateBgScrollOffsets` | `sm_80.c:2375`, `80:AE29` |
| Root / private movement dispatcher | `Irq_FollowDoorTransition` | `sm_80.c:2389`; direction table at `2382` |
| Private movement | `DoorTransition_Right` | `sm_80.c:2397`, `80:AE7E` |
| Private movement | `DoorTransition_Left` | `sm_80.c:2411`, `80:AEC2` |
| Private movement | `DoorTransition_Down` | `sm_80.c:2422`, `80:AF02` |
| Private movement | `DoorTransition_Up` | `sm_80.c:2456`, `80:AF89` |

The two bookkeeping helpers have no source callers outside this set. All four
setups are selected by `door_direction & 3`; they cannot be dropped merely
because the retained Ceres route selects right. `AddToHiLo` is a shared inline
arithmetic primitive (`variables.h:10–14`), not a fourteenth gameplay routine
or a reason to claim its many unrelated callers.

## Setup semantics and the upward prepass

`DoorTransitionScrollingSetup` first assigns both layer-1 coordinates from the
door destination, then dispatches direction (`sm_80.c:2306–2309`). Each setup
finishes by invoking its movement routine immediately; the first movement
therefore happens in the coordinator's setup service, before later IRQ calls.

| Direction | Setup sequence that must remain ordered |
| --- | --- |
| Right | Calculate layer-2 X, subtract 256 from layer-2 X, calculate layer-2 Y, subtract 256 from layer-1 X; derive offsets/blocks and previous blocks; decrement both previous X blocks; call right movement (`sm_80.c:2312–2323`). |
| Left | Corresponding positive 256 shifts and previous-X increments; call left movement (`sm_80.c:2325–2336`). |
| Down | Calculate layer-2 X/Y; subtract 224 from both layer Y positions; derive offsets/blocks and previous blocks; decrement previous Y blocks; call down movement (`sm_80.c:2338–2349`). |
| Up | Calculate layer-2 X; temporarily add 31 to layer-1 Y for layer-2 Y calculation; add 224 to layer-2 Y and set layer-1 Y to original plus 256; derive offsets; add 32 to destination Y; derive blocks/previous blocks and increment previous Y blocks; decrement layer-1 Y once; call up movement (`sm_80.c:2351–2366`). |

`UpdateBgScrollOffsets` computes both BG2 offsets against **layer 1**, not
layer 2 (`sm_80.c:2375–2380`). Keep this exact expression despite the names.
`UpdatePreviousLayerBlocks` copies the four current layer block fields before
each setup deliberately adjusts the axis it must upload (`2368–2373`).

Upward doors require the earlier `FixDoorsMovingUp` root. It resets the counter
to zero, calculates/copies blocks, increments both previous Y blocks and calls
`DoorTransition_Up` at counter zero (`sm_80.c:2290–2297`). That call increments
the counter to one. The coordinator performs this **before** room loading
(`sm_82.c:3932–3947`) and later resets the counter only for directions other
than up (`3954–3958`). Thus normal upward setup enters its movement routine
with counter one and must not reset it. Translating only setup plus IRQ motion
would miss this required scheduled prepass.

The coordinator also adds one to `reg_BG1VOFS` for down before setup, rebases
Samus and clears the completion bit in its later tile-loading phase, and
chooses the vertical/horizontal IRQ family (`sm_82.c:3954–3955`,
`3966–3976`, `3990–3991`). Those are fixed caller responsibilities in ticket
09; copying them into these roots would perform them twice.

## Movement arithmetic and direction-specific endings

For Samus motion, `AddToHiLo` combines the position and subposition into a
32-bit word, adds the supplied displacement, then splits it back
(`variables.h:10–14`). Right/down use the speed/subspeed pair; left/up negate
that pair (`types.h:155–156`; movement call sites `sm_80.c:2399`, `2413`,
`2426`, `2459`). Implement the bit-level result with wrapping 32-bit
addition/subtraction and preserve carry/borrow into the 16-bit position.
Layer positions, previous positions and counter increments also wrap to 16
bits. These are not saturating pixel moves or signed host-width arithmetic.

Each routine snapshots the old frame counter before any child call and later
writes `old + 1`. Preserve that local value across synchronous child state
exchange. The precise differences are:

| Direction | Movement, child call and completion |
| --- | --- |
| Right | Add speed to Samus X, copy previous X, add 4 to both layer X positions, call `UpdateScrollVarsUpdateMap`, increment counter; if **old counter equals 63**, call that child a second time and return 1 (`sm_80.c:2397–2408`). The second child sees counter 64. |
| Left | Subtract speed from Samus X, copy previous X, subtract 4 from both layer X positions, call the child, increment counter, return **old counter equals 63**. There is no second terminal child call (`sm_80.c:2411–2420`). |
| Down | Counter zero performs the transient row preparation below. Nonzero counters below `0x39` move Samus Y and both layers by +4 and call the child. Increment counter; if the **wrapped new counter is at least `0x39`**, call the child again and return 1 (`sm_80.c:2422–2454`). Preserve that exact predicate; it differs from the equality tests in the other directions. |
| Up | Counter zero performs transient row preparation. Every nonzero counter subtracts speed from Samus Y and 4 from both layers. Counters 1–4 update the four WRAM BG scroll shadows directly without the child; counters at least 5 call the child. Increment counter and return **old counter equals 56** (`sm_80.c:2456–2490`). |

When a movement routine returns 1, `Irq_FollowDoorTransition` snaps **only
layer 1 X/Y** to the door destination and ORs `0x8000` into
`door_transition_flag` (`sm_80.c:2389–2394`). It does not snap layer 2, rewrite
BG register shadows or call the scrolling child after that snap. Do not add a
convenient final synchronization pass: it would alter the reference's ordered
effects.

For normal coordinator initialization, the source predicts the following
counts. These are derivations of the bodies above, **not newly observed live
counts**:

| Direction | Movement invocations through completion | Of those, IRQ-follow invocations | Nested `UpdateScrollVarsUpdateMap` calls inside this scope |
| --- | --- | --- | --- |
| Right | 64: setup at old 0, then old 1–63 | 63 | 65 |
| Left | 64: setup at old 0, then old 1–63 | 63 | 64 |
| Down | 57: setup at old 0, then old 1–56 | 56 | 58 |
| Up | 57: prepass at old 0, setup at old 1, then old 2–56 | 55 | 53 |

Do not encode these counts as the implementation's completion rule. Noncanonical
input counters and wraparound follow the predicates in C, and the IRQ caller's
completion-bit guard controls whether another root invocation occurs.

## Transient vertical state is visible to the scrolling child

At counter zero, down saves layer-1/2 Y and the two BG vertical scroll shadows,
subtracts 15 from both layer Ys, calculates/copies block positions, decrements
the previous Y blocks, and calls `UpdateScrollVarsUpdateMap`. It then restores
only the saved layer Ys and vertical register shadows (`sm_80.c:2433–2447`).
Up does the same with subtraction 16 and previous-Y **increments**
(`sm_80.c:2472–2486`).

These temporary coordinates must be published before the child runs, and the
child's block/queue/transfer changes must remain after the selective restore.
Restoring a whole pre-call snapshot, or deferring the child until after the
coordinate restore, loses the purpose of these paths. Keep the saved scalar
locals and restore exactly the four C fields in the C order. These cases are
especially useful focused differential inputs because the retained route is
rightward and cannot live-exercise them.

## Existing scroll boundary and remaining dependencies

The 13 routines have only four non-inline callees outside the set:

| Dependency | Exact source / use |
| --- | --- |
| `CalculateLayer2Xpos` | `sm_80.c:1748–1760`; all four setups. Policy 0 copies layer 1; policy 1 preserves layer 2 and returns 1; other policies use `(policy & FE) * position / 256`. Setups ignore its return but need its writes. |
| `CalculateLayer2Ypos` | `sm_80.c:1762–1774`; all four setups, including up's temporary layer-1 Y adjustment. Same policy behavior for Y. |
| `CalculateBgScrollAndLayerPositionBlocks` | `sm_80.c:1866–1887`; setup, upward prepass and transient vertical preparation. Layer blocks use a sign-extended 16-bit position shifted by 4, while BG blocks use logical register-shadow shifts. |
| `UpdateScrollVarsUpdateMap` | Existing installed hook at `sm_80.c:1783–1787`; every movement direction reaches it according to the table above. |

The parent is checking reuse of these existing Rust operations. Do not assign
a second translation of the existing map/scroll engine to this worker merely
to close lexical call graphs. The 13-routine source inventory excludes those
already separate dependencies.

In the retained C, `UpdateScrollVarsUpdateMap` invokes
`sm_native_scroll_dispatch(1)` and runs its C body only if unowned, then calls
`sm_native_scroll_entry_done(1)` (`sm_80.c:1783–1791`). Its C body calculates
BG scroll shadows and updates graphics; graphics compares current/previous
blocks and prepares foreground/background columns and rows
(`1776–1791`, `1810–1864`). This existing wrapper is the required nested
ownership boundary. A C service path must call the wrapper, not
`NativeOriginal_UpdateScrollVarsUpdateMap`. A direct Rust composition must
preserve equivalent existing scroll ownership and observations.

Two nesting contexts follow from source:

- Coordinator setup/correction → directional root → existing scroll entry 1.
- C IRQ phase 18 or 24 → directional root → existing scroll entry 1.

In both contexts the child must see the directional routine's preceding WRAM
writes; the directional routine must see the child's committed changes before
counter updates, final snap or selective vertical restoration. Right's two
terminal child calls remain separate and ordered. The parent owns the concrete
host nesting contract; these are the source-required edges, not a proposal for
a general callback framework.

The coordinator's alignment phase separately calls scrolling entry 2,
`CalculateLayer2PosAndScrollsWhenScrolling` (`sm_82.c:3924`, `3927`;
wrapper `sm_80.c:1793–1797`). That call belongs to ticket 09, not to these
directional roots. Initial `DisplayViewablePartOfRoom` also remains outside.

## Why generic IRQ handling should remain foreign

The scope's roots perform arithmetic and WRAM/shadow updates; they do not call
`WriteReg`, `WriteRegWord`, `Vector_IRQ` or `WaitForNMI_Async`. The generic
IRQ wrappers are **callers** around the directional work, not foreign services
that these new Rust roots need to invoke.

Keep `Vector_IRQ` and `kIrqHandlers` (`sm_80.c:1245–1263`),
`IrqHandler_SetResult` (`1100–1105`), the HUD/register phases and IRQ enable
helpers in C. That leaves the timing/register schedule intact while owning
all door-motion decisions:

| IRQ family | Existing schedule and foreign work |
| --- | --- |
| Start of door | Phases 8/10 write HUD/background/color-math registers and select the next handler from `irqhandler_next_handler` (`sm_80.c:1137–1155`). No movement call. |
| Vertical | 16 begins HUD, selects 18 at V=31/H=152; 18 writes TM/color math, conditionally calls the directional root, selects 20 at V=216/H=152; 20 consumes the requested next handler or returns to 16 (`sm_80.c:1173–1198`). |
| Horizontal | 22 begins HUD, selects 24 at V=31/H=152; 24 writes TM/color math, conditionally calls the directional root, selects 26 at V=160/H=152; 26 consumes the requested next handler or returns to 22 (`sm_80.c:1200–1225`). |

Both end-HUD phases choose TM 16 when
`(previous_cre_bitset | cre_bitset) & 1`, otherwise TM 17. Their hardware writes
occur before the directional root and their next-IRQ scheduling occurs after
it. Hooking `Vector_IRQ` as a whole would claim these unrelated timing/register
effects and expand the scope; hooking the two call sites separately would
duplicate an already unified root. Retain the single `Irq_FollowDoorTransition`
boundary and the outer call order.

The coordinator waits on the flag written here (`sm_82.c:4037–4040`). This
directional scope must neither clear/complete the coordinator coroutine nor
change its pending return. Publishing the bit through the actual IRQ call is
the connection between the two coherent subsystems.

## Shared state contract

The root inputs/outputs remain live WRAM. Essential offsets are:

| State | WRAM offsets / declaration |
| --- | --- |
| BG1/BG2 H/V scroll shadows | `00B1`, `00B3`, `00B5`, `00B7`; `variables.h:90–93` |
| Direction | `0791`; `variables.h:276` |
| Layer and previous-layer block coordinates; BG block coordinates | `08F7`–`090D`; `variables.h:314–325` |
| Layer-1 X/Y, layer-2 X/Y | `0911`, `0915`, `0917`, `0919`; `variables.h:327–331` |
| Layer-2 X/Y policy bytes | `091B`, `091C`; `variables.h:332–333` |
| BG offsets | `091D`, `091F`, `0921`, `0923`; `variables.h:334–337` |
| Transition counter, destination X/Y, subspeed/speed, completion flag | `0925`, `0927`, `0929`, `092B`, `092D`, `0931`; `variables.h:338–344` |
| Samus position/subposition X/Y | `0AF6`, `0AF8`, `0AFA`, `0AFC`; `variables.h:542–545` |
| Samus previous X/Y | `0B10`, `0B14`; `variables.h:553`, `555` |

The existing scroll child additionally changes block comparison fields and
upload buffers/queues. Their effects cannot be reduced to the motion fields
above. This table identifies the directional interface; it is not an
allow-list that permits discarding child state.

## Material available from the same Ceres route

Reuse the prior packet's source/ROM findings and retained native observations.
No new route is needed to start this successor:

| Existing crossing | Door / direction | Retained state milestones |
| --- | --- | --- |
| DF45 → DF8D | `83:AB4C`, right / 0, destination `(0,0)` | State 9 at 8726, state 11 at 8727, destination pointer at 8744, state 8 at 8831. |
| DF8D → DFD7 | `83:AB64`, right / 0, destination `(0,0)` | State 9 at 9030, state 11 at 9031, destination pointer at 9053, state 8 at 9140. |

These host-step observations and the 63 additional explicit waits per crossing
are already recorded in [the prior packet](next-transition-scope.md), backed by
`/home/jim/sm-decomp-restart/restart/out/scroll/runs/reference-02/lifecycle.csv`
and `/home/jim/sm-decomp-restart/restart/out/runs/ceres-06/observations.csv`.
They are not original-ROM frame numbers or a reason to hard-code 63 waits.
The existing accepted prefix ends at step 9380 and keeps later stairwell
scrolling/projectile evidence.

Both doors use negative/default distance, so `LoadDoorHeader` chooses 200 and
writes combined speed `0x0000C800` (`sm_82.c:3619–3628`; door bytes/ROM
addresses in the prior packet). Both new rooms have Mode7 cleared before
directional setup (`sm_82.c:3944`; `sm_88.c:377`), so the existing column
preparation guard is open (`sm_80.c:2116–2118`). Their layer-2 policy bytes
are DF8D `(C1,01)` and DFD7 `(01,41)`, so both axes suppress background
scrolling uploads via the odd-policy checks (`sm_80.c:1824`, `1850`).
Foreground preparation and actual directional motion remain material.

For this route, the new source ownership observations should distinguish setup
from IRQ continuation, identify `door_direction`, capture counter progression
and completion-bit publication, and attribute existing scroll-child calls to
the directional parent. The source predicts **two setup-root invocations,
126 IRQ-follow invocations, 128 right-movement invocations and 130 nested
entry-1 calls inside this scope** across the two ordinary crossings. These
predicted counts require confirmation in the new source-frozen reference;
existing global scrolling counts alone do not establish them. No upward
correction or left/down/up live claim follows from this route.

Focused actual-C cases for implementation can cover the remaining distinctive
behavior: all four setup directions; carry/borrow and counter wrap; upward
prepass plus preserved counter; vertical temporary-state visibility/restore;
up counters 4→5 and 5→6; right terminal double-call ordering; and completion
snap that leaves layer 2 and BG shadows as C does. Use the existing verification
mechanism selected by the integrator, not a new runner. This paragraph defines
source-derived concerns, not additional checks run by the research agent.

## Main and restart Rust reuse

Main was inspected at `193cad941713b3539d23e08238a79e7fb09dde61`.
The exact directional-root/movement names occur in the generated source
inventory but have no implementation in `crates/port`. The main
[room module](/home/jim/sm-decomp/crates/port/src/room.rs:1) explicitly leaves
hardware transition scheduling outside its boundary. Its existing material is
useful selectively:

- [room.rs:598](/home/jim/sm-decomp/crates/port/src/room.rs:598) contains the
  exact four-word previous-block copy and BG-offset expressions. Port those
  small algorithms into the new scope's two owned bookkeeping helpers; no old
  crate import or larger camera-family migration is needed.
- [room.rs:116](/home/jim/sm-decomp/crates/port/src/room.rs:116) and
  [room.rs:137](/home/jim/sm-decomp/crates/port/src/room.rs:137) contain parallax
  and block calculations. The current restart implementation already has those
  algorithms privately in [room-scroll/scroll.rs:238](/home/jim/sm-decomp-restart/restart/room-scroll/src/scroll.rs:238)
  and [scroll.rs:255](/home/jim/sm-decomp-restart/restart/room-scroll/src/scroll.rs:255).
  Do not duplicate the map/column/row engine in the new crate.
- [camera_scroll.rs:87](/home/jim/sm-decomp/crates/port/src/camera_scroll.rs:87)
  demonstrates the shared fixed-point wrapping-add/subtract idiom, but normal
  camera policy is a different subsystem. Reuse the arithmetic idea only;
  importing its movement policy would change this scope.
- Existing [room tests](/home/jim/sm-decomp/crates/port/tests/room.rs:199) cover
  manual parallax, signed blocks and negative-distance decoding. These are
  regression ideas for dependencies, not evidence that directional setup,
  temporary vertical state or IRQ completion is already implemented or verified.

The restart [room-scroll public boundary](/home/jim/sm-decomp-restart/restart/room-scroll/src/lib.rs:11)
currently has three entries and a synchronous VRAM service. Its
`UpdateScrollVarsUpdateMap` path is already the desired owned child. Keep that
boundary and its observable entry/completion events; do not call its unwrapped
C original or silently replace its execution with a copied implementation.

## Recommended host seam: three synchronous roots and four dependencies

This future interface is separate from ticket09's frozen ABI. Do not modify the
active coordinator or host while those workers are implementing it. Start from
the completed09 baseline and record the following changes under the next claim.

Use a new dependency-free `restart/door-scrolling/**` gameplay crate with three
entries matching the source roots and ordinary handled/refused ownership.
Unlike09, these roots have no pending-result protocol: private movement results
remain inside the crate and only `Irq_FollowDoorTransition` publishes the shared
completion bit. Keep the outer coordinator coroutine untouched.

For the smallest first implementation, retain the three unhooked setup-math
callees as named synchronous native dependencies and use the existing wrapper
for the fourth:

| Typed dependency | Native operation / actual return | Required behavior |
| --- | --- | --- |
| `calculate_layer2_x` | `CalculateLayer2Xpos`, `u8` | Copy current private WRAM in, execute, copy back. Preserve the actual byte even though the setup caller discards it. |
| `calculate_layer2_y` | `CalculateLayer2Ypos`, `u8` | Same; up's temporary Y must be visible. |
| `calculate_blocks` | `CalculateBgScrollAndLayerPositionBlocks`, unit | Preserve signed layer-coordinate shifts and logical register-shadow shifts. |
| `update_scroll_vars` | existing wrapped `UpdateScrollVarsUpdateMap`, unit | Execute installed Rust entry1 when required, preserving child ownership/effects and complete copyback. |

This owns the13-routine directional source set and leaves three small math
callees explicitly C-owned. It avoids creating a public math API or retranslating
the existing scroll implementation merely to close this packet. The integrator
may later expose narrow existing pure Rust math reuse, but must record that
additional ownership/interface decision; it is not necessary to start this
delivery. Neither generic IRQ handling nor DMA becomes a new service here.

The needed composition extension is fixed by the four external call sites:

```text
room-transition root
  -> existing named coordinator service11 (FixDoorsMovingUp) or18 (ScrollingSetup)
     -> new directional root
        -> directional update_scroll_vars service
           -> existing room-scroll entry1

generic C IRQ phase18 or24
  -> new directional IRQ-follow root
     -> directional update_scroll_vars service
        -> existing room-scroll entry1
```

Ticket09's [ABI](/home/jim/sm-decomp-restart/.scratch/restart-pipeline/room-transition-abi.md)
allows one current child below a named coordinator service. This successor adds
one specifically admitted directional parent with one specifically admitted
scroll child; it does not permit arbitrary nested consumers. Each active parent
has its own copied transaction. Publish directional writes before the child,
apply the child commit before parent continuation, then refresh the outer
coordinator when its service returns. IRQ-root invocations occur after pending
coordinator roots have returned, with no active coordinator allocation.

Keep recursive directional entry, overlapping siblings, child calls outside
the declared directional service and arbitrary coordinator-service reentry
terminally rejected. Existing projectile/Ceres-door paths retain their guards.
The three math services are synchronous operations with no admitted child.
This is an implementation change for the next host owner, not a claim that
the active09 ABI already accepts this deeper chain.

The riskiest integration case is the terminal rightward call: the first child
must observe old counter63, the parent then writes64, and the second child must
observe64. A parent copyback or temporary-state mistake can preserve final
motion coordinates while losing the required block/queue updates. Likewise,
vertical restore must restore only the four named scalar fields, keeping child
descriptor/block changes. These are focused seam cases for the existing test
and comparison mechanisms, not justification for a second harness.

## Concrete assignment and first experiment

After09 completes, claim one whole directional scope and pin its new baseline:

| Owner | Exclusive source | Output |
| --- | --- | --- |
| Gameplay worker | new `restart/door-scrolling/**` | All13 routines, exact arithmetic and private dispatch, three entry interface, four typed dependencies, directional/transient-state focused cases. |
| Native-host worker | `restart/native-host/**` | Three root wrappers before first mutation; four named operations; explicit permitted directional/scroll child chain; ownership and private-path observations; recipe and bounded controls. |
| Root integrator | Cargo manifests/lock, gameplay/runtime registration, existing `sm-pilot`, new `sm-pilot/door_scrolling.rs`, ticket/docs/progress | Keep09 and earlier Rust consumers installed, own source freeze and actual shared reference/candidate/CLI/fault runs, timings and cleanup. |
| Read-only reviewer | one assigned Markdown report | Check the three root placements, source-derived counts and selective-state restoration, without another replay. |

Use the existing9380 inputs and the same two completed crossings. Preserve
ticket09's phase/background/continuation predicates and all earlier door,
projectile and row/column postconditions. Add narrow directional ownership and
parent-child attribution; the first source-frozen C reference confirms the
predicted2 setup/126 IRQ/128 right/130 child counts. Do not require unexercised
upward correction, left/down/up motion or background uploads as live conditions.
Counters are evidence, while actual final snap/completion and resumed gameplay
are the material postconditions.

Proposed first commands, only after the new module/scenario/recipe exists:

```sh
./target/debug/sm-pilot reference --scenario door-scrolling \
  --library NEW_PREPARED_LIBRARY --rom /home/jim/sm-decomp/rom/sm.sfc \
  --prefix out/runs/ceres-06/inputs.txt --out out/door-scrolling/runs/reference-01
./target/debug/sm-pilot dev --scenario door-scrolling \
  --library NEW_PREPARED_LIBRARY --rom /home/jim/sm-decomp/rom/sm.sfc \
  --reference out/door-scrolling/runs/reference-01 --out out/door-scrolling/runs/dev-01
```

Capture one new reference for the changed hook recipe, then reuse it for candidate
edits. A source-level terminal-motion/counter mutation should fail the existing
comparison; measure the ordinary warm loop and documentation reuse as before.
This packet assigns no implementation and establishes no new gameplay or
original-ROM parity claim. It is ready for root's choice after09 results.

## Handoff and cleanup

The source, reuse, host-composition and assignment recommendation is complete.
One worker should own the complete13-routine gameplay delivery; root-hook
registration and allowed child paths belong to the integrator/host owner after
ticket09 is frozen. No active ticket09 source was claimed or edited here.

Only read-only source/document inspection and this Markdown draft were created.
No verification runs, builds, worktrees or generated intermediates need cleanup.
Reclaimed bytes: 0. Available disk at the handoff check: 105,693,523,968 bytes.
Other agents' source, active runs, tickets and progress inventory were preserved.
