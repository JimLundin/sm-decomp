//! Crocomire's phases and scripts run in `sm_game::actors::crocomire_reactions`;
//! its imported tables, BG2 alignment, boss word, projectiles and drop come from here.
#![forbid(unsafe_code)]
use crate::game::enemy_projectiles::{ClearEprojs, scatter_drop};
use crate::game::gameplay_flag_runtime;
use crate::game_state::Game;
use sm_data::crocomire::{
    BRIDGE_DEBRIS, BRIDGE_DEBRIS_ADDRESS, BridgeDebris, FLESH_TILEMAP_ADDRESS, MELT_TILEMAP_BYTES,
    MELT_TILES, MeltTilemap, SCROLL_SPRITEMAP_ADDRESS, SCROLL_SPRITEMAPS, SKELETON_TILEMAP_ADDRESS,
    SPIKE_WALL_COLUMNS, SPIKE_WALL_DESTINATION_ADDRESS, SPIKE_WALL_TILE_ADDRESS, ScrollSpritemaps,
    SpikeWallDestinations, SpikeWallTiles, import_words,
};
pub(crate) use sm_game::actors::crocomire_reactions::*;
use sm_game::imported::{Imported, word as table_word};

// The named bank $A4 instructions are the ones `session_runtime` dispatches on.
const _: () = {
    use sm_game::actors::crocomire_reactions::instruction as i;
    assert!(i::FIGHT_AI == 10782374 && i::MAYBE_START_PROJECTILE_ATTACK == 10782546);
    assert!(i::QUEUE_CRY == 10783995 && i::QUEUE_BIG_EXPLOSION == 10784007);
    assert!(i::QUEUE_SKELETON_COLLAPSES == 10784019 && i::SHAKE_SCREEN == 10784711);
    assert!(i::MOVE_LEFT_4PX == 10784735 && i::MOVE_LEFT_4PX_AND_SPAWN_DUST == 10784762);
    assert!(i::MOVE_LEFT_4PX_AND_SPAWN_DUST_2 == 10784767);
    assert!(i::MOVE_LEFT_4PX_SPAWN_DUST_AND_HANDLE_SPIKE_WALL == 10784797);
    assert!(i::MOVE_RIGHT_4PX_IF_ON_SCREEN == 10784859 && i::MOVE_RIGHT_4PX == 10784895);
    assert!(i::MOVE_RIGHT_4PX_IF_ON_SCREEN_AND_SPAWN_DUST == 10784911);
    assert!(i::MOVE_RIGHT_4PX_AND_SPAWN_DUST == 10784916);
    assert!(i::SPAWN_DUST_FIRST == 10787483 && i::SPAWN_DUST_LAST == 10787543);
};
use sm_game::enemy_projectiles::drops::DropSource;
use sm_game::gameplay_flag_host::BossWordOrder;

#[derive(Default)]
pub(crate) struct ImportedTables {
    scroll_spritemaps: Option<Imported<ScrollSpritemaps>>,
    bridge_debris: Option<Imported<BridgeDebris>>,
    spike_wall_tiles: Option<Imported<SpikeWallTiles>>,
    spike_wall_destinations: Option<Imported<SpikeWallDestinations>>,
    flesh_tilemap: Option<Imported<MeltTilemap>>,
    skeleton_tilemap: Option<Imported<MeltTilemap>>,
}

pub(crate) fn import(image: &crate::rom::RomImage) -> ImportedTables {
    ImportedTables {
        scroll_spritemaps: Imported::import(
            image,
            SCROLL_SPRITEMAP_ADDRESS,
            2 * SCROLL_SPRITEMAPS,
            import_words,
        ),
        bridge_debris: Imported::import(
            image,
            BRIDGE_DEBRIS_ADDRESS,
            2 * BRIDGE_DEBRIS,
            import_words,
        ),
        spike_wall_tiles: Imported::import(
            image,
            SPIKE_WALL_TILE_ADDRESS,
            2 * (SPIKE_WALL_COLUMNS + 1),
            import_words,
        ),
        spike_wall_destinations: Imported::import(
            image,
            SPIKE_WALL_DESTINATION_ADDRESS,
            2 * SPIKE_WALL_COLUMNS,
            import_words,
        ),
        flesh_tilemap: Imported::import(
            image,
            FLESH_TILEMAP_ADDRESS,
            MELT_TILEMAP_BYTES,
            MeltTilemap::import,
        ),
        skeleton_tilemap: Imported::import(
            image,
            SKELETON_TILEMAP_ADDRESS,
            MELT_TILEMAP_BYTES,
            MeltTilemap::import,
        ),
    }
}

impl Host for Game {
    fn crocomire_table_word(&self, address: u32, index: isize) -> u16 {
        let tables = &self.crocomire;
        match address {
            SCROLL_SPRITEMAP_ADDRESS => table_word(&tables.scroll_spritemaps, self, address, index),
            BRIDGE_DEBRIS_ADDRESS => table_word(&tables.bridge_debris, self, address, index),
            SPIKE_WALL_TILE_ADDRESS => table_word(&tables.spike_wall_tiles, self, address, index),
            SPIKE_WALL_DESTINATION_ADDRESS => {
                table_word(&tables.spike_wall_destinations, self, address, index)
            }
            _ => unreachable!("Crocomire table {address:#x}"),
        }
    }

    fn crocomire_melt_tilemap(&self, table: u32) -> Option<[u16; MELT_TILES]> {
        let rom = self.loaded_rom.as_ref().expect("ROM must be loaded");
        let imported = match table {
            FLESH_TILEMAP_ADDRESS => &self.crocomire.flesh_tilemap,
            SKELETON_TILEMAP_ADDRESS => &self.crocomire.skeleton_tilemap,
            _ => &None,
        };
        Imported::from(imported, rom).map(|tilemap| *tilemap.tiles())
    }

    fn align_crocomire_background(&mut self) {
        crate::game::visual_runtime::align_crocomire_background(self);
    }

    fn boss_word_native(&mut self, area: u16) -> u16 {
        gameplay_flag_runtime::boss_word_at(area, BossWordOrder::Native, self)
    }

    fn clear_enemy_projectiles(&mut self) {
        ClearEprojs(self);
    }

    fn scatter_crocomire_drop(&mut self, area: u16) {
        scatter_drop(DropSource::Crocomire, area, self);
    }
}
