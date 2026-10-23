//! Qualified legacy effects; these are not counted as unsafe elimination.
use crate::{game_state::Game, rom::RomRecord};

pub(super) fn record(bank: u8, cursor: u16, game: &mut Game) -> RomRecord {
    // Retains the original low-window printf and failure flag before capture.
    unsafe { crate::rtl::rom_record(bank, cursor, game) }
}

pub(super) fn next_random(game: &mut Game) -> u16 {
    unsafe { crate::game::sm_80::NextRandom(game) }
}

pub(super) fn call_player_code(code: u16, game: &mut Game) -> u16 {
    unsafe { crate::game::sm_90::CallSomeSamusCode(code, game) }
}

pub(super) fn calculate_angle_from_xy(x: u16, y: u16, game: &mut Game) -> u16 {
    unsafe { crate::game::sm_a0::CalculateAngleFromXY(x, y, game) }
}

pub(super) fn damage_player(damage: u16, game: &mut Game) -> () {
    unsafe { crate::game::sm_91::Samus_DealDamage(damage, game) }
}

pub(super) fn create_sprite(x: u16, y: u16, id: u16, graphics: u16, game: &mut Game) -> u16 {
    unsafe { crate::game::sprite_objects::create_sprite(x, y, id, graphics, game) }
}

pub(super) fn ridley_offscreen(x: u16, y: u16, game: &mut Game) -> u8 {
    unsafe { crate::game::sm_a6::Ridley_Func_103(x, y, game) }
}

pub(super) fn unreachable(game: &mut Game) {
    unsafe {
        crate::rtl::Unreachable(game);
    }
}
pub(super) fn draw_spritemap(
    offscreen: bool,
    bank: u8,
    map: u16,
    x: u16,
    y: u16,
    tile: u16,
    graphics: u16,
    game: &mut Game,
) {
    unsafe {
        if offscreen {
            crate::game::sm_81::DrawEprojSpritemapWithBaseTileOffscreen(
                bank, map, x, y, tile, graphics, game,
            );
        } else {
            crate::game::sm_81::DrawEprojSpritemapWithBaseTile(
                bank, map, x, y, tile, graphics, game,
            );
        }
    }
}
/// Keep the old low-window diagnostic before capturing the native allocation.
/// All record reads themselves are safe and retain this exact owned source.
pub(super) fn raw_record(bank: u8, cursor: u16, game: &mut Game) -> super::content::NativeRecord {
    let address = (u32::from(bank) << 16) | u32::from(cursor);
    if address & 0x8000 == 0 {
        unsafe {
            crate::c_library::printf(c"RomPtr - Invalid access 0x%x!\n".as_ptr(), address);
        }
        game.g_fail = true;
    }
    super::content::NativeRecord::new(
        game.capture_native_rom(),
        crate::rtl::lorom_offset(address) as usize,
    )
}

pub(super) fn respawn_enemy(index: u16, game: &mut Game) {
    unsafe {
        crate::game::enemy_population_ram::respawn(index, game);
    }
}
