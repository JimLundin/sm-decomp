//! Statue behavior and carries over canonical actors/player/terrain/scrolls.
#![forbid(unsafe_code)]
use super::{
    BASE_DECORATION, CLEAR_SPIKE_PLM, Decoration, FOOTSTEP_SOUND, GRAB_SOUND,
    LOWER_NORFAIR_SLOPE_PLM, PHANTOON, RIDLEY, SPIKE_BLOCK, SPIKE_FOOTSTEPS, Statue, UNLOCK_SAMUS,
    WRECKED_SHIP_SLOPE_PLM, WRECKED_SHIP_SPIKES_PLM, function, instruction_list as list,
};
use crate::actors::host::{Slopes, abs16};
use crate::constants::enemy_properties::{
    K_ENEMY_PROPS_DISABLE_SAMUS_COLL, K_ENEMY_PROPS_PROCESSED_OFFSCREEN,
};
use crate::enemy_script::EnemyScript;
use crate::room_mechanisms::host::ByteOrder;
use crate::room_scrolls::CameraHost;
use crate::session_slot;

pub trait Host:
    crate::actors::access::ActorAccess
    + crate::actors::session_slots::SessionSlots
    + crate::actors::host::ActorMotion
    + CameraHost
    + crate::effects::SoundEffects
{
    fn tourian_statue(&self, index: isize) -> sm_data::statues::Statue;
    fn chozo_statue(&self, index: isize) -> sm_data::statues::Statue;
    fn statue_movement(&self, index: isize) -> sm_data::statues::Movement;
    fn install_tourian_palettes(&mut self);
    fn install_tube_palette(&mut self);
    fn install_chozo_palettes(&mut self, lower_norfair: bool);
    fn boss_word_native(&mut self, area: u16) -> u16;
    fn start_lowering_acid(&mut self);
    fn lower_acid(&mut self);
    fn select_block_at(&mut self, x: u16, y: u16);
    fn spawn_collision_plm(&mut self, header: u16);
    fn spawn_room_eproj(&mut self, definition: u16, parameter: u16) -> u16;
    fn spawn_plm(&mut self, x: u16, y: u16, header: u16);
    fn command_player(&mut self, command: u16);
    fn invalid_function(&mut self, address: u16) -> !;
}

pub fn prepare_tourian_statue<L: Host>(game: &mut L) {
    let me = i32::from(game.session().actors.controls.selected);
    session_slot!(game, Decoration[me].base; palette_index = 0, instruction_timer = 1, timer = 0);
    let list = game
        .tourian_statue((session_slot!(game, Decoration[me].kind) >> 1) as isize)
        .instruction_list;
    session_slot!(game, Decoration[me].base.current_instruction = list);
    if session_slot!(game, Decoration[me].kind) == 0 {
        // The spawns take their initialisation parameter from A: the
        // instruction list just stored, then each spawn's returned slot.
        let base = game.spawn_room_eproj(BASE_DECORATION, list);
        let ridley = game.spawn_room_eproj(RIDLEY, base);
        game.spawn_room_eproj(PHANTOON, ridley);
    }
}

pub fn prepare_chozo_statue<L: Host>(game: &mut L) -> u16 {
    let me = i32::from(game.session().actors.controls.selected);
    session_slot!(
        game,
        Statue[me].base.properties = (session_slot!(game, Statue[me].base.properties) as i32
            | (K_ENEMY_PROPS_DISABLE_SAMUS_COLL as i32
                | K_ENEMY_PROPS_PROCESSED_OFFSCREEN as i32
                | 0x8000)) as u16
    );
    session_slot!(
        game,
        Statue[me].base.spritemap_pointer = list::NOTHING_SPRITEMAP
    );
    session_slot!(game, Statue[me].base; instruction_timer = 1, timer = 0);
    session_slot!(game, Statue[me]; function = function::NOTHING, activated = 0);
    session_slot!(game, Statue[me].base.palette_index = 0);
    // The original clears slot 0's layer, not the statue's.
    session_slot!(game, Statue[0].base.layer = 0);
    let variant: u16 = session_slot!(game, Statue[me].variant);
    session_slot!(
        game,
        Statue[me].base.current_instruction =
            game.chozo_statue((variant >> 1) as isize).instruction_list
    );
    variant
}

pub fn main_ai<L: Host>(game: &mut L) {
    let me = i32::from(game.session().actors.controls.selected);
    let k = game.session().actors.controls.selected;
    let address = session_slot!(game, Statue[me].function);
    match address {
        function::LOWER_NORFAIR => lower_norfair(game, k),
        function::WRECKED_SHIP => wrecked_ship(game, k),
        function::NOTHING | function::CLEARED | function::SCRIPTED => {}
        _ => game.invalid_function(address),
    }
}

pub fn lower_norfair<L: Host>(game: &mut L, k: u16) {
    let me = k as i32;
    if session_slot!(game, Statue[me].activated) != 0 {
        session_slot!(
            game,
            Statue[me].base.current_instruction = list::LOWER_NORFAIR_ACTIVATED
        );
        session_slot!(game, Statue[me].base.instruction_timer = 1);
    }
}

pub fn wrecked_ship<L: Host>(game: &mut L, k: u16) {
    let area = game.session().room.area;
    if (game.boss_word_native(area) & 1) != 0 {
        let me = k as i32;
        if session_slot!(game, Statue[me].activated) != 0 {
            session_slot!(
                game,
                Statue[me].base.current_instruction = list::WRECKED_SHIP_ACTIVATED
            );
            session_slot!(game, Statue[me].base.instruction_timer = 1);
            // Never read.
            session_slot!(game, Statue[me]; var_a = 0xff00, var_b = 256);
        }
    }
}

pub fn start_lowering_acid<L: Host>(game: &mut L) {
    game.start_lowering_acid();
}

pub fn set_lowered_acid<L: Host>(game: &mut L) {
    game.lower_acid();
}

pub fn unlock_samus<L: Host>(game: &mut L) {
    game.command_player(UNLOCK_SAMUS);
}

pub fn grab_sound<L: Host>(game: &mut L) {
    game.queue_sound(1, GRAB_SOUND, 6);
}

pub fn footstep_sound<L: Host>(game: &mut L) {
    game.queue_sound(1, FOOTSTEP_SOUND, 6);
}

pub fn spike_clearing_footstep<L: Host>(game: &mut L, k: u16, jp: EnemyScript) -> EnemyScript {
    let me = k as i32;
    let offset: u16 = jp.word(0);
    let x = (session_slot!(game, Statue[me].base.x_pos)).wrapping_add(offset);
    let y = (session_slot!(game, Statue[me].base.y_pos)).wrapping_add(28);
    game.select_block_at(x, y);
    // The block below the selected one.
    let index =
        i32::from(game.session().room.width) + i32::from(game.session().room.collision_cursor);
    let block = crate::room_terrain::level_word(game, 2 * index as usize, ByteOrder::Native);
    if (block & 0xf000) == SPIKE_BLOCK {
        game.spawn_collision_plm(CLEAR_SPIKE_PLM);
        game.spawn_room_eproj(SPIKE_FOOTSTEPS, offset);
    }
    jp.offset(1)
}

pub fn movement<L: Host>(game: &mut L, k: u16, jp: EnemyScript) -> EnemyScript {
    let me = k as i32;
    session_slot!(game, Statue[me].movement = jp.word(0));
    let step = game.statue_movement((session_slot!(game, Statue[me].movement) >> 1) as isize);
    game.move_horizontal(k, (step.x_velocity as i16 as i32) << 8, Slopes::Ignore);
    let step = game.statue_movement((session_slot!(game, Statue[me].movement) >> 1) as isize);
    game.move_vertical(k, (abs16(step.x_velocity) as i16 as i32) << 8);
    game.snap_to_slopes(k);
    let index = session_slot!(game, Statue[me].movement) as i32 >> 1;
    let step = game.statue_movement(index as isize);
    game.session_mut().player.x =
        (step.samus_x_offset).wrapping_add(session_slot!(game, Statue[me].base.x_pos));
    game.session_mut().player.y =
        (step.samus_y_offset).wrapping_add(session_slot!(game, Statue[me].base.y_pos));
    jp.offset(1)
}

pub fn release_samus<L: Host>(game: &mut L) {
    game.command_player(UNLOCK_SAMUS);
    for (offset, value) in [(0, 0), (2, 0), (3, 0), (7, 1)] {
        let bytes = u16::to_ne_bytes(value);
        game.set_camera_scroll_byte(6 + offset, bytes[0]);
        game.set_camera_scroll_byte(7 + offset, bytes[1]);
    }
    game.spawn_plm(0x17, 0x1d, WRECKED_SHIP_SLOPE_PLM);
}

pub fn initialize_tourian_statue<H: Host>(game: &mut H) {
    prepare_tourian_statue(game);
    game.install_tourian_palettes();
}
pub fn initialize_tube_cracks<H: Host>(game: &mut H) {
    game.install_tube_palette();
}
pub fn initialize_chozo_statue<H: Host>(game: &mut H) {
    let variant = prepare_chozo_statue(game);
    game.install_chozo_palettes(variant != 0);
    if variant != 0 {
        game.spawn_plm(0xc, 0x1d, LOWER_NORFAIR_SLOPE_PLM);
    } else {
        game.spawn_plm(0x4a, 0x17, WRECKED_SHIP_SPIKES_PLM);
        game.spawn_plm(0x17, 0x1d, WRECKED_SHIP_SLOPE_PLM);
    }
}
