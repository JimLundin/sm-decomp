//! Draygon's hurt palettes, reactions and death sequence (bank
//! $A5:9185-$A5:9701).
//! Boss words remain native compatibility state; player and sprites use their owners.
use crate::actors::boss_interaction as interaction;
use crate::actors::boss_lease::{BossState, steps};
use crate::actors::draygon_flight::{facing_animation, foam_at_mouth};
use crate::actors::draygon_host::Lease;
use crate::actors::draygon_host::{
    ARMS, BANK, BODY, DELETE, Draygon, DraygonEdit, EYE, GRAPPLE_DROPPED, SIGN, TAIL, abs16,
    function, instruction_list as list,
};
use crate::actors::host::{ActorLease, Death, Shot};
use crate::actors::queries::angle_from_delta;

use crate::actors::slot_view::store_byte;
use crate::enemy_projectiles::drops::DropSource;
use crate::enemy_script::EnemyScript;
use crate::gameplay_flag_host::BossWordOrder;
use crate::menus::prelude::{battle, owned};
use crate::sprite_effect_state::{LegacySpriteDefinition, SpriteSpawn};

/// Colours 9-12 of BG1/2 palette 5, as palette words and in RAM at $7E:C000.
const HEALTH_COLOURS: core::ops::Range<usize> = 89..93;
/// The hurt AI's palettes: BG1/2 palette 5 and sprite palette 7, which flash
/// to $A5:A297.
const BG_PALETTE_5: u16 = 0xa277;
const SPRITE_PALETTE_7: u16 = 0xa1f7;
const FLASH_PALETTE: u16 = 0xa297;
/// Their first colours in palette RAM ($7E:C0A0, $7E:C1E0).
const BG_PALETTE_5_COLOUR: usize = 80;
const SPRITE_PALETTE_7_COLOUR: usize = 240;
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
/// palette buffer.
pub fn write_health_palette<L: Lease>(native: bool, game: &mut L) {
    let mut cursor = Draygon::of(game, BODY).health_palette().wrapping_mul(4);
    for index in HEALTH_COLOURS {
        let value =
            crate::actor_tables::word(game, |t| &t.draygon_health_palettes, (cursor >> 1) as isize);
        if native {
            game.session_mut().presentation.visual_mut().colors[index] =
                u16::from_le_bytes(value.to_ne_bytes());
        } else {
            interaction::set_at(game, interaction::Array::PaletteBuffer, index, value);
        }
        cursor = cursor.wrapping_add(2);
    }
}
fn copy_hurt_palette<L: Lease>(default: u16, target: usize, game: &mut L) {
    let k = game.current_enemy();
    let palette = if Draygon::of(game, k).actor().flash & FLASHING != 0 {
        FLASH_PALETTE
    } else {
        default
    };
    let record = crate::cartridge::native_record(BANK, palette, game);
    for offset in 0..PALETTE_BYTES {
        let byte = record.byte(offset);
        let colours = &mut game.session_mut().presentation.visual_mut().colors;
        store_byte(&mut colours[target + offset / 2], offset % 2, byte);
    }
}
/// Hurt AI - Draygon ($A5:954D): flash the palettes and, while grappled,
/// drain health.
pub fn hurt_ai<L: Lease>(game: &mut L) {
    copy_hurt_palette(BG_PALETTE_5, BG_PALETTE_5_COLOUR, game);
    let k = game.current_enemy();
    if Draygon::of(game, k).actor().flash & FLASHING == 0 {
        write_health_palette(false, game);
    }
    copy_hurt_palette(SPRITE_PALETTE_7, SPRITE_PALETTE_7_COLOUR, game);
    // $A5:9564/$9572, $A5:95B3/$95C1: both copies count their colours down
    // in $060B.
    game.session_mut()
        .enemy_projectiles
        .spawn
        .store_shared_counter(0);
    if battle!(game.SamusGrappleFlags) & GRAPPLE_FLAG != 0
        && Draygon::of(game, BODY).actor().frame & GRAPPLE_FRAMES == 0
    {
        let health = (Draygon::of(game, BODY).actor().health as i16)
            .wrapping_sub(GRAPPLE_DRAIN)
            .max(0);
        DraygonEdit::of(game, BODY).actor().health = health as u16;
        reaction(game);
    }
}
/// Draygon health-based palette handling ($A5:9701).
fn update_health_palette<L: Lease>(game: &mut L) {
    let mut cursor: i32 = 0;
    while (Draygon::of(game, BODY).actor().health as i16).wrapping_sub(crate::actor_tables::word(
        game,
        |t| &t.draygon_health_thresholds,
        (cursor >> 1) as isize,
    ) as i16)
        < 0
    {
        cursor += 2;
    }
    if cursor != i32::from(Draygon::of(game, BODY).health_palette()) {
        *DraygonEdit::of(game, BODY).health_palette() = cursor as u16;
        write_health_palette(true, game);
    }
}
/// Enemy shot - Draygon ($A5:95F0).
pub fn shot<L: Lease>(game: &mut L) {
    if Draygon::of(game, BODY)
        .swoop_y_acceleration()
        .wrapping_sub(MAX_ACCELERATION)
        & SIGN
        != 0
    {
        {
            let mut draygon = DraygonEdit::of(game, BODY);
            let old = draygon.swoop_y_acceleration();
            *old = old.wrapping_add(ACCELERATION_STEP);
        }
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
/// Draygon reaction ($A5:960D): update the health palette, or on death drop the
/// grapple, let go of Samus and drift to the death spot.
pub fn reaction<L: Lease>(game: &mut L) {
    let k = game.current_enemy();
    if Draygon::of(game, k).actor().health != 0 {
        update_health_palette(game);
        return;
    }
    battle!(game.GrappleBeamFunction = GRAPPLE_DROPPED);
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
    *DraygonEdit::of(game, EYE).function() = function::NONE;
    *DraygonEdit::of(game, BODY).function() = function::DEATH_DRIFT_TO_DEATH_SPOT;
    game.owned_step(crate::player_movement::forced::release_from_draygon);
    battle!(game.SamusGrappleFlags = 0);
    let dx = DEATH_X.wrapping_sub(Draygon::of(game, BODY).actor().x.whole);
    let dy = DEATH_Y.wrapping_sub(Draygon::of(game, BODY).actor().y.whole);
    *DraygonEdit::of(game, BODY).death_angle() =
        u16::from(QUARTER_TURN.wrapping_sub(angle_from_delta(dx, dy) as u8));
    let vector = {
        let angle = u16::from(Draygon::of(game, BODY).death_angle().to_le_bytes()[0]);
        game.direction_step(angle, 1)
    };
    {
        let mut draygon = DraygonEdit::of(game, BODY);
        *draygon.death_x_speed() = (vector.x >> 16) as u16;
        *draygon.death_x_subspeed() = vector.x as u16;
        *draygon.death_y_speed() = (vector.y >> 16) as u16;
        *draygon.death_y_subspeed() = vector.y as u16;
    }
}
/// Draygon body function - death sequence - drift to death spot ($A5:9185).
pub fn drift_to_death_spot<L: Lease>(game: &mut L) {
    let k = game.current_enemy();
    foam_at_mouth(k, FOAM_FRAMES, game);
    // The angle is from the position rounded to its 4x4 pixel block.
    let dx = (i32::from(DEATH_X >> 2) - (i32::from(Draygon::of(game, BODY).actor().x.whole) >> 2))
        as u16;
    let dy = (i32::from(DEATH_Y >> 2) - (i32::from(Draygon::of(game, BODY).actor().y.whole) >> 2))
        as u16;
    *DraygonEdit::of(game, BODY).death_angle() =
        u16::from(QUARTER_TURN.wrapping_sub(angle_from_delta(dx, dy) as u8));
    let vector = {
        let angle = u16::from(Draygon::of(game, BODY).death_angle().to_le_bytes()[0]);
        game.direction_step(angle, 1)
    };
    crate::actors::draygon_grab::store_speeds(vector, game);
    let scratch = u16::from(Draygon::of(game, BODY).death_angle().to_le_bytes()[0]);
    game.set_step_scratch(scratch);
    let angle = u16::from(Draygon::of(game, BODY).death_angle().to_le_bytes()[0]);
    game.step_along(angle, vector);
    if abs16(
        Draygon::of(game, BODY)
            .actor()
            .x
            .whole
            .wrapping_sub(DEATH_X),
    )
    .wrapping_sub(DEATH_SPOT_RANGE)
        & SIGN
        == 0
    {
        return;
    }
    if abs16(
        Draygon::of(game, BODY)
            .actor()
            .y
            .whole
            .wrapping_sub(DEATH_Y),
    )
    .wrapping_sub(DEATH_SPOT_RANGE)
        & SIGN
        == 0
    {
        return;
    }
    game.owned_step(crate::sprite_effect_controllers::draygon_spawn_ring);
    *DraygonEdit::of(game, BODY).function() = function::DEATH_WAIT_FOR_EVIRS;
    crate::enemy_projectiles::host::Effects::queue_music(game, ELEVATOR_MUSIC);
    *DraygonEdit::of(game, BODY).function_timer() = EVIR_FRAMES;
    {
        let mut draygon = DraygonEdit::of(game, BODY);
        let actor = draygon.actor();
        actor.instruction = list::SLEEP;
        actor.instruction_delay = 1;
    }
    let properties = Draygon::of(game, BODY).actor().properties | DELETE;
    DraygonEdit::of(game, TAIL).actor().properties = properties;
    DraygonEdit::of(game, ARMS).actor().properties = properties;
    facing_animation(BODY, EYE, [list::EYE_LEFT_DEAD, list::EYE_RIGHT_DEAD], game);
    *DraygonEdit::of(game, EYE).function() = function::NONE;
}
/// Handle dying Draygon smoke ($A5:92EA): every 8 frames, a big dust cloud low in the room.
fn dying_smoke<L: Lease>(game: &mut L) {
    if game.frame() & 7 == 0 {
        let random = game.roll_random();
        let spawn = SpriteSpawn {
            position: crate::geometry::Point {
                x: (random & 0x7f) + 192,
                y: ((random >> 8) & 0x3f) + 400,
            },
            definition: LegacySpriteDefinition(21),
            appearance: 0,
        };
        crate::sprite_objects::spawn(game, spawn);
    }
}
/// Instruction - spawn dying Draygon sprite object `kind` within 40h px ($A5:973F-$A5:9778).
pub fn spawn_dying_sprite<L: Lease>(kind: u16, script: EnemyScript, game: &mut L) -> EnemyScript {
    let random = game.roll_random();
    let [dx, dy] = [random, random >> 8].map(|value| (value & 0x7f).wrapping_sub(64));
    let x = Draygon::of(game, BODY).actor().x.whole.wrapping_add(dx);
    let y = Draygon::of(game, BODY).actor().y.whole.wrapping_add(dy);
    steps::create_sprite(game, x, y, kind);
    script
}
/// Instruction - spawn Draygon goop - leftwards ($A5:9F7C) or rightwards ($A5:9FAE).
pub fn spawn_goop<L: Lease + ActorLease>(
    right: bool,
    script: EnemyScript,
    game: &mut L,
) -> EnemyScript {
    let dx = if right { 24 } else { 0xffe4 };
    let x = Draygon::of(game, BODY).actor().x.whole.wrapping_add(dx);
    let y = Draygon::of(game, BODY).actor().y.whole.wrapping_sub(16);
    game.place_projectile_spawn(x, y);
    let angle = (game.roll_random() & 0x3f) + if right { 192 } else { 128 };
    game.session_mut().enemy_projectiles.spawn.secondary = angle;
    let enemy = game.current_enemy();
    game.spawn_actor_eproj(2, enemy, 0x8e50);
    script
}
/// Draygon body function - death sequence - wait for evirs ($A5:9294).
pub fn wait_for_evirs<L: Lease>(game: &mut L) {
    game.owned_step(dying_smoke);
    {
        let mut draygon = DraygonEdit::of(game, BODY);
        let old = draygon.function_timer();
        *old = old.wrapping_sub(1);
    }
    if Draygon::of(game, BODY).function_timer() != 0 {
        // Moving left, the evirs also take $0E24, whatever was left there ($A5:A01C).
        let bias = game.session().actors.compatibility.scratch[1];
        game.owned_step(|game| crate::sprite_effect_controllers::draygon_motion(game, bias));
    } else {
        *DraygonEdit::of(game, BODY).function() = function::DEATH_BURIED_BY_EVIRS;
    }
}
/// Draygon body function - death sequence - buried by evirs ($A5:92AB): sink
/// until buried, then delete Draygon, set its boss bit and drop its rewards.
pub fn buried_by_evirs<L: Lease>(game: &mut L) {
    game.owned_step(dying_smoke);
    let bias = game.session().actors.compatibility.scratch[1];
    game.owned_step(|game| crate::sprite_effect_controllers::draygon_motion(game, bias));
    {
        let mut draygon = DraygonEdit::of(game, BODY);
        let old = &mut draygon.actor().y.whole;
        *old = old.wrapping_add(1);
    }
    if Draygon::of(game, BODY)
        .actor()
        .y
        .whole
        .wrapping_sub(BURIED_Y)
        & SIGN
        == 0
    {
        {
            let mut draygon = DraygonEdit::of(game, BODY);
            let actor = draygon.actor();
            actor.instruction = list::DELETE;
            actor.instruction_delay = 1;
        }
        let properties = Draygon::of(game, BODY).actor().properties | DELETE;
        DraygonEdit::of(game, BODY).actor().properties = properties;
        DraygonEdit::of(game, EYE).actor().properties = properties;
        let drop_index = BossState::area(game);
        let area = BossState::area(game);
        let bits = steps::boss_word(game, area, BossWordOrder::Little);
        game.owned_step(|game| {
            crate::progress::set_boss_word_le(game, area, bits | DRAYGON_DEAD);
            crate::enemy_projectiles::drops::scatter(DropSource::Draygon, drop_index, game);
            crate::sprite_objects::retire_all(game.session_mut());
        });
    }
}
