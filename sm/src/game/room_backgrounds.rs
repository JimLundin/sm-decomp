//! Retained fixed clear/DMA leaves for room entry, on the lease; command
//! execution is owned.
use crate::game::display_commands::{Lease, Wire, byte, copy_to_vram_on_lease, word};
use crate::game::session_runtime::Runtime;
use crate::rtl::{A1B1, A1T1L, DAS1L, DMAP1, VMADDL, VMAIN};
use sm_game::display::transfer::Source;
use sm_game::presentation_workspace::{Access as _, Point, Role};
use sm_game::room_entry::content::LoadMode;

/// The tilemap staging buffer ($7E:4000), as the door clears' DMA source.
const STAGING: Source = Source {
    address: 0x4000,
    bank: 0x7e,
};

pub(crate) fn clear_fx(mode: LoadMode, runtime: &mut Runtime<'_>) {
    match mode {
        LoadMode::Initial => clear_tilemap_with_dma(0x5880, 0xa2f7, 0xa2f8, 0x780, runtime),
        LoadMode::Door => {
            // The 1920 words $7E:4000..$4F00, which the original fills
            // from the top down; nothing reads them between the stores.
            runtime.screen_fill(Point::new(Role::TilemapStaging, 0), 1920, 6222);
            copy_to_vram_on_lease(0x5880, STAGING, 0xf00, None, runtime);
        }
    }
}
pub(crate) fn clear_bg2(mode: LoadMode, runtime: &mut Runtime<'_>) {
    match mode {
        LoadMode::Initial => clear_tilemap_with_dma(0x4800, 0xa29a, 0xa29a, 0x800, runtime),
        LoadMode::Door => {
            // The 2048 words $7E:4000..$5000, filled top down in the original.
            runtime.screen_fill(Point::new(Role::TilemapStaging, 0), 2048, 824);
            copy_to_vram_on_lease(0x4800, STAGING, 0x1000, None, runtime);
        }
    }
}
pub(crate) fn clear_dual_bg(runtime: &mut Runtime<'_>) {
    // The 2048 words $7E:4000..$5000, filled top down in the original.
    runtime.screen_fill_native(Point::new(Role::TilemapStaging, 0), 2048, 824);
}

/// Fill the level words with air ($8000): all 12800 cells, for either load.
/// The original's store order (from the top word down, and for a door a word
/// of each of four runs at a time) reaches each cell's level word once, so
/// the typed store is the same.
pub(crate) fn clear_level(runtime: &mut Runtime<'_>) {
    for cell in runtime.session_mut().room.terrain.cells_mut() {
        cell.level = 0x8000;
    }
}

/// Fill VRAM's low bytes from `low`, then its high bytes from `high`, from the
/// cartridge's bank $80 (or the room's prepared image), through the lease's
/// transfer port. The DMAs read no work RAM, so the lease needs no publish.
fn clear_tilemap_with_dma(
    destination: u16,
    low: u16,
    high: u16,
    size: u16,
    runtime: &mut Runtime<'_>,
) {
    let assets = runtime.session().world.prepared_room_assets();
    let wire = &mut Lease(runtime);
    word(VMADDL, destination, wire);
    word(DMAP1, 0x1808, wire);
    word(A1T1L, low, wire);
    word(A1B1, 0x80, wire);
    word(DAS1L, size, wire);
    byte(VMAIN, 0, wire);
    wire.start_room_dma(2, assets.as_deref());
    word(VMADDL, destination, wire);
    word(DMAP1, 0x1908, wire);
    word(A1T1L, high, wire);
    word(A1B1, 0x80, wire);
    word(DAS1L, size, wire);
    byte(VMAIN, 0x80, wire);
    wire.start_room_dma(2, assets.as_deref());
}
