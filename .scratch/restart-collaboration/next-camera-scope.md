# Successor: normal-camera scheduled frame

Status: bounded research complete; advisory scope for integration owner
Date: 2026-10-16
Owner: `/root/transition_boundary`
Writable ownership: this Markdown file only; released at handoff

Recommend the complete `MainScrollingRoutine` boundary and its thirteen-routine
gameplay closure. It is a synchronous, pure-WRAM operation with substantial
existing Rust reuse. Graphics preparation, IRQ movement and the room-transition
coordinator remain separate established boundaries. No implementation is assigned
by this packet.

## Source identity

Recovered C pin: `578f90b3cc49557bb70060ad033bb90b8cf8ac50`. All C citations
below resolve beneath:

`/home/jim/sm-decomp-restart/restart/out/scroll/native/2c9c9a3cf51a059220afac1ee04727c956e45d067e10e17405bf3adbd268fc7f/source/src`

The retained `sm_90.c` SHA-256 is
`38a661917219e6647826d25ea0bc45039c36bd54924f8cf968facb3f463389fa`.
Main Rust was inspected at checkout commit
`193cad941713b3539d23e08238a79e7fb09dde61`; the files named below were clean.

## Exact scheduled boundary and closure

The sole source caller of `MainScrollingRoutine` is
`GameState_8_MainGameplay`, at `sm_82.c:874`: after movement, PLMs and collision
work, before drawing. The separate graphics entry
`CalculateLayer2PosAndScrollsWhenScrolling` runs later at `sm_82.c:882`.

**Hook the actual routine body, not a host `game_state == 8` condition.** Movement
can already have published state 9 on a door hit (`sm_94.c:858–870`), while the
latched state-8 invocation continues through the camera call
(`sm_82.c:657–686`, `864–887`). A state-only gate would skip that scheduled call.

| Routine | Exact C location |
| --- | --- |
| `MainScrollingRoutine` | `sm_90.c:1269`, `90:94EC` |
| `CallScrollingFinishedHook` | `sm_90.c:1262` |
| `Samus_ScrollFinishedHook_SporeSpawnFight` | `sm_90.c:1315`, `90:9589` |
| `Samus_HandleScroll_X` | `sm_90.c:1320`, `90:95A0` |
| `Samus_HandleScroll_Y` | `sm_90.c:1343`, `90:964F` |
| `Samus_CalcDistanceMoved_X` | `sm_90.c:1364`, `90:96C0` |
| `Samus_CalcDistanceMoved_Y` | `sm_90.c:1374`, `90:96FF` |
| `HandleAutoscrolling_X` | `sm_80.c:1889`, `80:A528` |
| `HandleScrollingWhenTriggeringScrollRight` | `sm_80.c:1937`, `80:A641` |
| `HandleScrollingWhenTriggeringScrollLeft` | `sm_80.c:1961`, `80:A6BB` |
| `HandleAutoscrolling_Y` | `sm_80.c:1984`, `80:A731` |
| `HandleScrollingWhenTriggeringScrollDown` | `sm_80.c:2048`, `80:A893` |
| `HandleScrollingWhenTriggeringScrollUp` | `sm_80.c:2073`, `80:A936` |

This count includes the private finish-hook dispatcher. Remaining numeric
primitives are `AddToHiLo`/`SetHiLo` (`variables.h:10–19`), pair construction
(`types.h:155–156`), `swap16`, and pure `Mult8x8` (`sm_rtl.c:530–532`). There
are no ROM reads, hardware writes, foreign gameplay calls or coroutine waits
inside this closure.

## Rust reuse and ABI

[camera_scroll.rs](/home/jim/sm-decomp/crates/port/src/camera_scroll.rs)
was introduced by `7512b6479afe55da270931a9f3ae26c9d558e5a5`; its current digest
is `5700c005e1306ee61771b6c7e603621f0ec15eb9568c5aa3c215dae17bd789f3`.
It implements the frame, axes and finish hook (`100–198`), calling existing Rust
constraint helpers in [room.rs](/home/jim/sm-decomp/crates/port/src/room.rs:142)
(`142–305`, dispatch `606–611`) and distance arithmetic in
[samus.rs](/home/jim/sm-decomp/crates/port/src/samus.rs:237) (`237–250`, dispatch
`266`). Reuse this complete closure; avoid importing unrelated room/physics
dispatch branches merely for those helpers.

Recommend one no-argument native root hook with a mutable WRAM transaction and
the existing handled/refused/error convention. No ROM argument, service table
or coroutine return is required by gameplay. Wrap `MainScrollingRoutine`
before its first mutation; keep private routines within the Rust closure.
Retain separate root ownership counts and private routine/branch observations,
so root ownership is distinguishable from partial helper execution.

The existing Rust `prepare` computes privately, copies thirty input words and
256 scroll bytes, and publishes fourteen output words only after success
(`camera_scroll.rs:38–85`, `200–251`). Preserve atomic refusal. Both `supports`
and `dispatch_rom` currently call that complete preparation; a restart adapter
should prepare once and commit once rather than executing it twice per frame.
The current preparation allocates a full WRAM-sized buffer; adapting its storage
is an implementation choice, not a reason to rewrite the camera rules.

## Arithmetic and admission limits

Normal ordering is distance X → camera X → distance Y → camera Y → optional
finish hook → publish all previous Samus position/fraction words
(`sm_90.c:1302–1312`). Y constraints must see X's same-call result. Distances
are the direction-selected 32-bit difference **plus one pixel**, using the
signed 16-bit integer-position difference to select direction
(`1364–1381`). Preserve fractional carry, wrapping arithmetic and integer-only
stationary-axis tests (`1323`, `1344`).

X uses the exact facing/knockback/movement/acceleration XOR and four-entry
distance tables indexed by `camera_distance_index >> 1` (`1320–1338`). Odd
indices are valid; an unused invalid table index need not refuse a stationary
axis. Slow-grapple mode uses separate unsigned windows and three-pixel moves;
signed-high X skips both manual axes (`1269–1300`). Frozen-state guards exist
in the two autoscrollers, not around the whole frame. The only admitted finish
hook is zero or `9589`; that hook clamps integer Y without changing its fraction.

Existing Rust admission requires room dimensions 1–255 with product at most
256, validates actual scroll lookups below 256, and rejects unsupported hooks
or accessed camera-table entries (`camera_scroll.rs:110–113`, `200–237`;
`room.rs:142–150`). These are bounded Rust domains, not proof of arbitrary C
memory inputs. Existing tests cover fractional movement, slow scrolling, XOR,
same-frame axis dependence and full-state refusal
(`crates/port/tests/camera_scroll.rs:31–185`); they were inspected, not run.

## Existing route evidence and remaining proof

Retained `restart/out/scroll/runs/reference-02/result.toml:48–52` records a
passing 9,380-step reference. Its camera ranges are corridor X 0–256
(`68–69`), shaft Y 0–512 and stairs Y 0–256 (`109–112`); corresponding normal
gameplay consumes corridor columns and stairwell rows (`70–74`, `113–117`).
These observed ranges make the same route a useful camera scenario.

They do not establish Rust camera ownership or exact camera branch counts.
The existing 936 calls belong to the later graphics-scrolling entry, which
also has coordinator alignment callers. Predict ordinary axis following and
stationary-axis constraints; measure their actual execution in the successor's
frozen reference. Slow-grapple, Spore Spawn, knockback and all table variants
remain unproved by this route. Preserve the existing combined outcomes and add
root/routine ownership and actual layer-position changes; no broader replay is
required by these findings. Current registry evidence labels the main camera
mapping translated only (`registry.json:91745–91771`).

Cleanup: no builds, tests, replays, authentication or generated intermediates.
Reclaimed bytes: 0. Only this draft was written; other agents' work was preserved.
