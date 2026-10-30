//! Draygon's hurt palettes, reactions and death sequence (bank
//! $A5:9185-$A5:9701).
//! Boss words remain native compatibility state; player and sprites use their owners.
use crate::actors::draygon_flight::{facing_animation, foam_at_mouth};
use crate::actors::draygon_host::Lease;
use crate::actors::draygon_host::{
    ARMS, BANK, BODY, DELETE, Draygon, EYE, GRAPPLE_DROPPED, Host, SIGN, TAIL, abs16, function,
    instruction_list as list,
};
use crate::actors::host::{BossEffects as _, Death, Shot};
use crate::actors::queries::angle_from_delta;
use crate::actors::session_slots::session_slot;
use crate::ram::word;

/// $A5:96AF: four colours of BG1/2 palette 5 for each health step, and
/// $A5:96EF: the health below which each step starts.
const HEALTH_PALETTES: u32 = 0xa5_96af;
const HEALTH_THRESHOLDS: u32 = 0xa5_96ef;
/// Colours 9-12 of BG1/2 palette 5, as palette words and in RAM at $7E:C000.
const HEALTH_COLOURS: core::ops::Range<usize> = 89..93;
const PALETTE_RAM: usize = 0xc000;
/// The hurt AI's palettes: BG1/2 palette 5 and sprite palette 7, which flash
/// to $A5:A297.
const BG_PALETTE_5: u16 = 0xa277;
const SPRITE_PALETTE_7: u16 = 0xa1f7;
const FLASH_PALETTE: u16 = 0xa297;
const BG_PALETTE_5_RAM: usize = 0xc0a0;
const SPRITE_PALETTE_7_RAM: usize = 0xc1e0;
const PALETTE_BYTES: usize = 32;
/// Flash timer bit 1 flashes.
const FLASHING: u16 = 2;
/// The grapple beam drains 100h health every 8 frames.
const GRAPPLE_FLAG: u16 = 1;
const GRAPPLE_DRAIN: i16 = 0x100;
const GRAPPLE_FRAMES: u16 = 7;
/// Shots raise the swoop Y acceleration by 8, up to 98h.
const ACCELERATION_STEP: u16 = 8;
const MAX_ACCELERATION: u16 = 0x98;
/// The death spot, (100h, 1E0h), reached within 4 px; the evirs bury Draygon
/// for 1A0h frames, until Y 240h.
const DEATH_X: u16 = 0x100;
const DEATH_Y: u16 = 0x1e0;
const DEATH_SPOT_RANGE: u16 = 4;
const EVIR_FRAMES: u16 = 0x1a0;
const BURIED_Y: u16 = 0x240;
const FOAM_FRAMES: u16 = 0xf;
/// The elevator music track, and boss bit 0: Draygon is dead.
const ELEVATOR_MUSIC: u16 = 3;
const DRAYGON_DEAD: u16 = 1;
/// Angles are measured from the top, clockwise; 40h less is the maths angle.
const QUARTER_TURN: u8 = 0x40;

/// Write the health-based colours: natively into palette RAM, or into the
/// palette buffer. Was `damage_palette`.
pub fn write_health_palette<L: Lease>(native: bool, game: &mut L) {
    let mut cursor = session_slot!(game, Draygon[BODY].health_palette).wrapping_mul(4);
    for index in HEALTH_COLOURS {
        let value =
            crate::cartridge::rom_table_word!(game, HEALTH_PALETTES, (cursor >> 1) as isize);
        if native {
            let bytes = value.to_ne_bytes();
            let at = PALETTE_RAM + index * 2;
            game.set_slot_byte(at, bytes[0]);
            game.set_slot_byte(at + 1, bytes[1]);
        } else {
            word!(game.PALETTE_BUFFER[index] = value);
        }
        cursor = cursor.wrapping_add(2);
    }
}
fn copy_hurt_palette<L: Lease>(default: u16, target: usize, game: &mut L) {
    let k = game.current_enemy() as i32;
    let palette = if session_slot!(game, Draygon[k].base.flash_timer) & FLASHING != 0 {
        FLASH_PALETTE
    } else {
        default
    };
    let record = crate::cartridge::native_record(BANK, palette, game);
    for offset in 0..PALETTE_BYTES {
        let byte = record.byte(offset);
        game.set_slot_byte(target + offset, byte);
    }
}
/// Hurt AI - Draygon ($A5:954D): flash the palettes and, while grappled,
/// drain health. Was `hurt`.
pub fn hurt_ai<L: Lease>(game: &mut L) {
    copy_hurt_palette(BG_PALETTE_5, BG_PALETTE_5_RAM, game);
    let k = game.current_enemy() as i32;
    if session_slot!(game, Draygon[k].base.flash_timer) & FLASHING == 0 {
        write_health_palette(false, game);
    }
    copy_hurt_palette(SPRITE_PALETTE_7, SPRITE_PALETTE_7_RAM, game);
    if word!(game.SAMUS_GRAPPLE_FLAGS) & GRAPPLE_FLAG != 0
        && session_slot!(game, Draygon[BODY].base.frame_counter) & GRAPPLE_FRAMES == 0
    {
        let health = (session_slot!(game, Draygon[BODY].base.health) as i16)
            .wrapping_sub(GRAPPLE_DRAIN)
            .max(0);
        session_slot!(game, Draygon[BODY].base.health = health as u16);
        reaction(game);
    }
}
/// Draygon health-based palette handling ($A5:9701).
fn update_health_palette<L: Lease>(game: &mut L) {
    let mut cursor: i32 = 0;
    while (session_slot!(game, Draygon[BODY].base.health) as i16).wrapping_sub(
        crate::cartridge::rom_table_word!(game, HEALTH_THRESHOLDS, (cursor >> 1) as isize) as i16,
    ) < 0
    {
        cursor += 2;
    }
    if cursor != i32::from(session_slot!(game, Draygon[BODY].health_palette)) {
        session_slot!(game, Draygon[BODY].health_palette = cursor as u16);
        write_health_palette(true, game);
    }
}
/// Enemy shot - Draygon ($A5:95F0).
pub fn shot<L: Lease>(game: &mut L) {
    if session_slot!(game, Draygon[BODY].swoop_y_acceleration).wrapping_sub(MAX_ACCELERATION) & SIGN
        != 0
    {
        session_slot!(game, Draygon[BODY].swoop_y_acceleration, |old| old
            .wrapping_add(ACCELERATION_STEP));
    }
    game.shot_reaction(Shot::ReactionOnly);
    reaction(game);
}
/// Enemy touch - Draygon ($A5:95EA).
pub fn touch<L: Lease>(game: &mut L) {
    game.contact_reaction(Death::LeavePending);
    reaction(game);
}
/// Power bomb reaction - Draygon ($A5:9607).
pub fn power_bomb<L: Lease>(game: &mut L) {
    game.power_bomb_reaction(Death::LeavePending);
    reaction(game);
}
/// Draygon reaction ($A5:960D): update the health palette, or on death drop
/// the grapple, let go of Samus and drift to the death spot. Was `after_damage`.
pub fn reaction<L: Lease>(game: &mut L) {
    let k = game.current_enemy() as i32;
    if session_slot!(game, Draygon[k].base.health) != 0 {
        update_health_palette(game);
        return;
    }
    word!(game.GRAPPLE_BEAM_FUNCTION = GRAPPLE_DROPPED);
    facing_animation(
        BODY,
        BODY,
        [list::BODY_LEFT_DYING, list::BODY_RIGHT_DYING],
        game,
    );
    facing_animation(
        BODY,
        EYE,
        [list::EYE_LEFT_DYING, list::EYE_RIGHT_DYING],
        game,
    );
    session_slot!(game, Draygon[EYE].function = function::NONE);
    session_slot!(
        game,
        Draygon[BODY].function = function::DEATH_DRIFT_TO_DEATH_SPOT
    );
    game.foreign(|game| game.release_samus());
    word!(game.SAMUS_GRAPPLE_FLAGS = 0);
    let dx = DEATH_X.wrapping_sub(session_slot!(game, Draygon[BODY].base.x_pos));
    let dy = DEATH_Y.wrapping_sub(session_slot!(game, Draygon[BODY].base.y_pos));
    session_slot!(
        game,
        Draygon[BODY].death_angle =
            u16::from(QUARTER_TURN.wrapping_sub(angle_from_delta(dx, dy) as u8))
    );
    let vector = {
        let angle = u16::from(session_slot!(byte game, Draygon[BODY].death_angle[0]));
        game.direction_step(angle, 1)
    };
    session_slot!(game, Draygon[BODY]; death_x_speed = (vector.x >> 16) as u16,
        death_x_subspeed = vector.x as u16, death_y_speed = (vector.y >> 16) as u16,
        death_y_subspeed = vector.y as u16);
}
/// Draygon body function - death sequence - drift to death spot ($A5:9185).
/// Was `approach`.
pub fn drift_to_death_spot<L: Lease>(game: &mut L) {
    let k = game.current_enemy() as i32;
    foam_at_mouth(k as u16, FOAM_FRAMES, game);
    // The angle is from the position rounded to its 4x4 pixel block.
    let dx = (i32::from(DEATH_X >> 2)
        - (i32::from(session_slot!(game, Draygon[BODY].base.x_pos)) >> 2)) as u16;
    let dy = (i32::from(DEATH_Y >> 2)
        - (i32::from(session_slot!(game, Draygon[BODY].base.y_pos)) >> 2)) as u16;
    session_slot!(
        game,
        Draygon[BODY].death_angle =
            u16::from(QUARTER_TURN.wrapping_sub(angle_from_delta(dx, dy) as u8))
    );
    let vector = {
        let angle = u16::from(session_slot!(byte game, Draygon[BODY].death_angle[0]));
        game.direction_step(angle, 1)
    };
    let scratch = u16::from(session_slot!(byte game, Draygon[BODY].death_angle[0]));
    game.set_step_scratch(scratch);
    let angle = u16::from(session_slot!(byte game, Draygon[BODY].death_angle[0]));
    game.step_along(angle, vector);
    if abs16(session_slot!(game, Draygon[BODY].base.x_pos).wrapping_sub(DEATH_X))
        .wrapping_sub(DEATH_SPOT_RANGE)
        & SIGN
        == 0
    {
        return;
    }
    if abs16(session_slot!(game, Draygon[BODY].base.y_pos).wrapping_sub(DEATH_Y))
        .wrapping_sub(DEATH_SPOT_RANGE)
        & SIGN
        == 0
    {
        return;
    }
    game.foreign(|game| game.draygon_spawn_ring());
    session_slot!(
        game,
        Draygon[BODY].function = function::DEATH_WAIT_FOR_EVIRS
    );
    game.foreign(|game| game.queue_music_delayed(ELEVATOR_MUSIC));
    session_slot!(game, Draygon[BODY].function_timer = EVIR_FRAMES);
    session_slot!(game, Draygon[BODY].base; current_instruction = list::SLEEP, instruction_timer = 1);
    let properties = session_slot!(game, Draygon[BODY].base.properties) | DELETE;
    session_slot!(game, Draygon[TAIL].base.properties = properties);
    session_slot!(game, Draygon[ARMS].base.properties = properties);
    facing_animation(BODY, EYE, [list::EYE_LEFT_DEAD, list::EYE_RIGHT_DEAD], game);
    session_slot!(game, Draygon[EYE].function = function::NONE);
}
/// Draygon body function - death sequence - wait for evirs ($A5:9294). Was
/// `ring`.
pub fn wait_for_evirs<L: Lease>(game: &mut L) {
    game.foreign(|game| game.draygon_death_bubble());
    session_slot!(game, Draygon[BODY].function_timer, |old| old
        .wrapping_sub(1));
    if session_slot!(game, Draygon[BODY].function_timer) != 0 {
        game.foreign(|game| game.draygon_motion(0));
    } else {
        session_slot!(
            game,
            Draygon[BODY].function = function::DEATH_BURIED_BY_EVIRS
        );
    }
}
/// Draygon body function - death sequence - buried by evirs ($A5:92AB): sink
/// until buried, then delete Draygon, set its boss bit and drop its rewards.
/// Was `sink`.
pub fn buried_by_evirs<L: Lease>(game: &mut L) {
    game.foreign(|game| {
        game.draygon_death_bubble();
        game.draygon_motion(0);
    });
    session_slot!(game, Draygon[BODY].base.y_pos, |old| old.wrapping_add(1));
    if session_slot!(game, Draygon[BODY].base.y_pos).wrapping_sub(BURIED_Y) & SIGN == 0 {
        session_slot!(game, Draygon[BODY].base; current_instruction = list::DELETE, instruction_timer = 1);
        let properties = session_slot!(game, Draygon[BODY].base.properties) | DELETE;
        session_slot!(game, Draygon[BODY].base.properties = properties);
        session_slot!(game, Draygon[EYE].base.properties = properties);
        let drop_index = word!(game.AREA_INDEX);
        let area = word!(game.AREA_INDEX);
        let bits = game.foreign(|game| game.boss_word(area));
        game.foreign(|game| {
            game.set_boss_word(area, bits | DRAYGON_DEAD);
            game.scatter_draygon_drop(drop_index);
            game.retire_all_sprites();
        });
    }
}
