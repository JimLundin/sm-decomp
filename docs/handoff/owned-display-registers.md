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
