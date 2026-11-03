# Owned display registers: survey and blocker (2026-11-03, lane `disp`)

The `register_writes!`/`write_register` form is 85 of `sm_game_ram_sites`: display_refresh 27,
message_boxes 13, cinematic_display 10, display/sequence 9, display/transfer 8, map_menu 4,
hdma_objects 4, gameplay_frame 3, hud 2, and one each in liquid_effects, cinematic_states,
room_state, ending_sequence and menu_window. None was retired. This note explains why.

## What the writes reach

- `display::sequence::Registers` is implemented by `Game` (`sm/src/game_state.rs`), as
  `rtl::write_reg` into the emulated backend (`Execution::write`). It is also implemented by
  `Runtime` (`sm/src/game/session_runtime.rs`), with one `foreign` call per write.
- The writes go to hardware, not to WRAM:
  - PPU $2100..$2133: display controls, scroll and Mode 7 latches, and the VRAM/CGRAM/OAM ports.
  - DMA $43x0..$43xA. Writing MDMAEN runs the DMA at once, reading WRAM or ROM and writing
    VRAM, CGRAM, OAM or WRAM.
  - CPU I/O $4200..$420C: NMITIMEN, WRIO, the multiplier and divider, H/VTIME, HDMAEN, MEMSEL.
  - The WRAM port $2181..$2183.
- `mirror`/`show_mirror` also store the WRAM shadow ($51..$86) through
  `WorkRamAccess::ram().write_byte`. Only `Game` implements `WorkRamAccess`, so every
  mirrored write runs on Game.
- Readers:
  - the emulated PPU's line renderer (`sm/src/snes/ppu.rs`; PR #176's `LineConfig` is built
    from its fields);
  - the DMA and HDMA engines;
  - the CPU's NMI and IRQ timing;
  - in verify builds, `room_background_observer` (on every write) and the native shadow PPU.
    `sm_runtime::rendering::{Registers, Ports, OwnedMemory}` runs only under
    `SM_NATIVE_LINES` and is resynced from the emulated PPU when they differ.

## Typed owners that already exist

- The Owned presentation's `OwnedDisplay`/`SessionRefresh` implement `scene::commit::RefreshHost`
  over `scene::state` (`controls`, `effective`, `raster`, `hdma`). The Legacy `Refresh<H>`
  adapter in display_refresh.rs implements the same trait with register writes.
- `display::video::Upload` and `Video::produced` describe uploads as typed values. They
  bracket the register writes and do not replace them, because the emulated PPU still needs
  the port writes.
- The display mirrors are typed `preparation::Field`s (SMDISPL).

Replays always run the Legacy presentation, where the emulated PPU is the authority.

## Designs considered

1. **Session fields.** Not possible yet. The renderer, DMA and timers read the backend's own
   state, so a Session copy is a cache unless the renderer reads it.
2. **A deferred display-command queue.** Not exact. MDMAEN, VMDATA, INIDISP and the scroll
   latches take effect at once. Writes made mid-frame (HDMAEN, the IRQ outputs, message boxes)
   change the lines drawn after them. The VRAM readbacks read the result of the DMA straight
   away.
3. **A synchronous typed command** (`Hardware::apply(Command)`, encoded by the host). This
   would be exact, but almost every site runs on `Game`. The same `write_reg` sequence would
   move from sm_game into sm/, the compatibility crate we are moving off. That is a rename
   under the brief's rule, not a typed owner.

## What retires them

1. Make the Owned presentation (or the native renderer, step 5 after `native_line`) the
   authority on the normal replay path. This is already blocker 2 in typed-slot-views.md.
   - The NMI refresh then runs `OwnedDisplay`.
   - The Legacy `Refresh` adapter becomes the emulated backend's encoder and moves to sm/ with
     that backend. That retires display_refresh's NMI writes, hdma_objects and the `irq`
     outputs.
2. The setup producers write hardware immediately: graphics loads, `copy_to_vram_now`,
   `setup_transfer`, HUD init, message boxes, the map menu window, and the cinematic BG and
   Mode 7 setup. First give each of them a typed record alongside its writes, as `Upload` does
   (`Upload::Tiles` for the channel-1 VRAM DMAs; a typed window/colour-math control record for
   the menus and message boxes). Then check the records against the emulated PPU under
   `SM_NATIVE_LINES`. Once the native renderer is the authority, the writes become backend
   encoding and leave sm_game.
3. `display/sequence.rs` (the trait and macro, 9) goes when its last caller does.

Mirror stores (lane `mirrors`): `write_register_and_mirror` and the `mirror`/`show_mirror`
arms (sm_game's and rtl's) now take a `preparation::Field` and store it with
`store_display_control_byte(.., Store::Named)`, the same RAM byte on Game. The ratchet counts
any call whose mirror is still a RAM offset ("raw display mirror stores"); there are none.
The register half of each call is still a register write.

## Typed display commands (lane `owned-pres`, 2026-11-03)

### How close the native renderer is

- `sm_runtime::rendering` (the `native_line` check in `sm/src/snes/ppu.rs`) redraws every line
  from its own `Registers`/`Ports`/`OwnedMemory` and compares it with the emulated PPU's line.
  CI runs it on every room job of the nightly full route (`replay_job.py` sets
  `SM_NATIVE_LINES=1` on `schedule`). Run 41424604620 (main 558131e72, 2026-11-02) passed with
  no `native-lines.csv`: no differing line, no frame-start resync and no typed-producer
  mismatch on the full route.
- So the native renderer is byte-exact on replays today, but it is still fed by the same
  `$21xx` port writes the emulated PPU takes (`native_line::write`), except for the typed
  producers (Samus tiles, tilemaps, animated tiles), which feed it `Video::produced` uploads.
- It is not yet the authority: the emulated PPU draws the frame, and DMA, HDMA and the IRQ
  timing still read the emulated backend. Making it the authority is the step-5 work in
  `v2-native-video.md`, and it is not one landable slice.

### The slice: sm_game says what, the backend's encoder says how

The target architecture (CONTRACTS "Original hardware register operations are expressed as
the required gameplay or presentation behavior"; ARCHITECTURE "Focused GPU renderer": Scene
records are "not SNES registers") puts register encoding outside the game. The migration
policy keeps the emulated backend as the comparison path until replacement evidence exists.
That allows this step, which keeps the emulated PPU as the authority:

- sm_game names a display change as a typed value, `display::video::Command`, and hands it to
  the `Video` host at the point the writes were made (`Video::apply`).
- The emulated backend's encoder (`sm/src/game/display_commands.rs`, on `Game`) performs the
  original's writes for that value synchronously, in the original's order and widths. Writes
  are byte-identical and at the same time, so this is `[exact]`.
- Where the command is an upload, the encoder brackets its writes with
  `producing`/`produced`, so under `SM_NATIVE_LINES` the native PPU takes the typed upload in
  place of the port writes, and a mismatch is logged as `typed,<producer>,<part>`. That is
  retirement step 2 (a typed record checked against the emulated PPU) for the producer.
- The commands reach `Game` through the existing `Video` impl, so no Game host impl, foreign
  call or address API is added.

This is not design 3's rename. The command is the record the native renderer consumes and the
Owned presentation would apply; the register addresses, port widths and DMA channel numbers
leave sm_game. Once the native renderer is the authority, the encoder becomes the emulated
backend's adapter and goes with it.

First commands:

| Command | Producer | Encoding (in order) |
| --- | --- | --- |
| `TilesNow { address, increment, source, bytes }` | HUD initialisation (hud.rs) | VMADD word, VMAIN word (mode, address low byte), channel 1: DMAP word $1801, A1T word, A1B, DAS word; MDMAEN 2 |
| `FixedColor(FixedColor)` | ending space view (ending_sequence.rs) | COLDATA |
| `WindowHdma(WindowHdma)` | map-menu window (menu_window.rs) | channel $43x0..$43xA: mode $40, B-bus port, table word and bank, indirect bank, then the cursor bytes cleared |

| `Control(Control)` with `apply_mirrored` | map-menu area window (map_menu.rs) | the control's register (TM, TS, TMW, TSW, W12SEL, W34SEL, WOBJSEL, CGWSEL, CGADSUB, HDMAEN), with its display mirror stored by sm_game before or after as before |

A control's mirror field is named at the call, not implied by the control: the map menu
mirrors TS into the sub window byte and TM into the main window byte, as the original does.

### Left for the next slices

- display/transfer.rs (`setup_transfer`, `copy_to_vram_now`): `copy_to_vram_now` writes VMAIN
  as a byte after the channel and starts through `start_room_dma`, which may read prepared
  room assets. It needs its own command, with the prepared-asset source in the encoder.
- message_boxes, cinematic_display, the remaining controls in gameplay_frame, liquid_effects,
  cinematic_states and room_state: the same `Control`/`FixedColor` commands cover most of
  their writes once the enum gains their registers (BGMODE, Mode 7, scroll latches).
- display_refresh's NMI writes and hdma_objects: still retirement step 1 (the Owned
  presentation or native renderer as the authority), since the `Refresh` adapter is itself the
  encoder of `scene::commit::RefreshHost`.
- display/sequence.rs goes with its last caller.
