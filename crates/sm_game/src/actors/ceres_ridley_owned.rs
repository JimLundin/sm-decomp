//! Ceres Ridley fight, intro, getaway and escape sequence over owned state.
#![forbid(unsafe_code)]
use super::{
    ESCAPE_STARTED, ESCAPE_TEXT_TILES, MODE7_TRANSPARENT_OVERFLOW, RIDLEY_ESCAPING, ZOOM_PALETTES,
    ceres_data, ceres_function, ceres_instruction_list,
};
use crate::actors::host::{abs16, below};
use crate::actors::ridley::{
    self, FACING_RIGHT, NORFAIR, RIDLEY, Rect, Ridley, function, instruction_list,
    owned as ridley_owned, tail,
};
use crate::actors::session_slots::session_slot;
use crate::constants::enemy_properties::{
    K_ENEMY_PROPS_BLOCK_PLASMA_BEAM, K_ENEMY_PROPS_DELETED, K_ENEMY_PROPS_INVISIBLE,
    K_ENEMY_PROPS_TANGIBLE,
};
use crate::projectile_geometry::SINE_COSINE;
pub trait Host: crate::actors::ridley::owned::Host {
    fn boss_byte(&mut self, area: u16) -> u8;
    fn disable_minimap_for_boss(&mut self);
    fn copy_palette(&mut self, destination: u16, bank: u8, address: u16, colours: u16);
    fn copy_target_palette(&mut self, bank: u8, destination: u16, address: u16, colours: u16);
    fn clear_initial_palettes(&mut self, norfair: bool);
    fn colour(&self, index: usize) -> u16;
    fn set_colour(&mut self, index: usize, value: u16);
    fn eye_fade_step(&mut self, index: u16) -> u8;
    fn ceres_eye_fade(&mut self, index: isize) -> u16;
    fn ceres_body_fade(&mut self, index: isize) -> u16;
    fn ceres_background_fade(&mut self, index: isize) -> u16;
    fn ceres_attack(&mut self, index: isize) -> u16;
    fn ceres_getaway_zoom(&mut self, index: isize) -> u16;
    fn ceres_getaway_velocity(&mut self, index: isize) -> sm_data::ceres_ridley::GetawayVelocity;
    fn ceres_baby_metroid_frame(&mut self, index: isize) -> u16;
    fn ceres_wing_frame(&mut self, index: isize) -> u16;
    fn ceres_health_palette(&mut self);
    fn emergency_text_glow(&mut self);
    fn set_mode7_irq(&mut self, value: u16);
    fn set_affine(&mut self, index: usize, value: u16);
    fn set_bg1_scroll(&mut self, horizontal: bool, value: u16);
    fn set_bg_mode(&mut self, value: u8);
    fn set_mode7_settings(&mut self, value: u8);
    fn begin_ceres_push(&mut self);
    fn queue_mode7_transfer(&mut self, bank: u8, list: u16);
    fn queue_escape_timer_tiles(&mut self) -> bool;
    fn queue_getaway_tiles(&mut self);
    fn queue_escape_tile_table(&mut self);
    fn type_text(&mut self, tiles: u16) -> bool;
}
mod bg2_staging {
    use super::Host;
    pub fn set<H: Host + ?Sized>(h: &mut H, index: usize, value: u16) {
        let [low, high] = value.to_le_bytes();
        h.set_tail_byte(2 * index, low);
        h.set_tail_byte(2 * index + 1, high);
    }
}
pub fn init<L: Host + ?Sized>(game: &mut L) {
    let area = game.session().room.area;
    if (game.boss_byte(area) & 1) != 0 {
        session_slot!(game, Ridley[RIDLEY].base.properties, |old| old
            | K_ENEMY_PROPS_TANGIBLE as u16
            | K_ENEMY_PROPS_DELETED as u16
            | K_ENEMY_PROPS_INVISIBLE as u16);
        return;
    }
    for word in game
        .session_mut()
        .actors
        .extended
        .tilemaps
        .iter_mut()
        .rev()
        .flat_map(|block| block.iter_mut().rev())
    {
        *word = 0;
    }
    game.disable_minimap_for_boss();
    session_slot!(game, Ridley[RIDLEY]; parameter_1 = 0, parameter_2 = 0);
    ridley_owned::set_instruction_list(ceres_instruction_list::INITIAL, game);
    session_slot!(game, Ridley[RIDLEY].base.palette_index = 3584);
    session_slot!(game, Ridley[RIDLEY].palette = 3584);
    session_slot!(game, Ridley[RIDLEY].base.extra_properties, |old| old | 4);
    session_slot!(game, Ridley[RIDLEY]; fight_mode = 0, hit_counter = 0);
    {
        game.session_mut().feedback.earthquake_kind = 0;
        game.session_mut().feedback.earthquake_timer = 0;
    };
    ridley_owned::initialise_tail(game);
    ridley_owned::initialise_ribs(game);
    session_slot!(game, Ridley[RIDLEY].base.properties, |old| old
        | K_ENEMY_PROPS_BLOCK_PLASMA_BEAM as u16
        | K_ENEMY_PROPS_TANGIBLE as u16);
    if game.session().room.area == NORFAIR {
        session_slot!(game, Ridley[RIDLEY].base; x_pos = 96, y_pos = 394);
        session_slot!(game, Ridley[RIDLEY].function = ceres_function::INITIAL);
        session_slot!(game, Ridley[RIDLEY]; x_velocity = 0, y_velocity = 0);
        session_slot!(game, Ridley[RIDLEY].base.layer = 5);
        session_slot!(game, Ridley[RIDLEY]; fight_mode = 0, movement_enabled = 1, min_y = 64, max_y = 416,
            min_x = 64, max_x = 224, facing = FACING_RIGHT, tail_damage = 120, );
        game.copy_target_palette(0xa6, 0x140, ceres_data::NORFAIR_PALETTES, 0x20);
        game.clear_initial_palettes(true);
    } else {
        session_slot!(game, Ridley[RIDLEY].base; x_pos = 186, y_pos = 169);
        game.session_mut().room.ceres_status = 0;
        session_slot!(game, Ridley[RIDLEY]; facing = ridley::FACING_LEFT, movement_enabled = 0);
        bg2_staging::set(game, tail::IDLE_WHIP, 1);
        session_slot!(game, Ridley[RIDLEY]; tail_damage = 15, function = ceres_function::INITIAL);
        session_slot!(game, Ridley[RIDLEY]; x_velocity = 0, y_velocity = 0);
        session_slot!(game, Ridley[RIDLEY]; min_y = -32i32 as u16, max_y = 176, min_x = 40,
            max_x = 224);
        game.copy_target_palette(0xa6, 0x140, ceres_data::CERES_PALETTES, 0x20);
        game.clear_initial_palettes(false);
        session_slot!(
            game,
            Ridley[RIDLEY].baby_metroid_instruction = ceres_instruction_list::BABY_METROID
        );
        session_slot!(game, Ridley[RIDLEY].baby_metroid_instruction_timer = 1);
        session_slot!(
            game,
            Ridley[RIDLEY].baby_metroid_function = ridley::baby_metroid_function::HELD
        );
        session_slot!(game, Ridley[RIDLEY].wings_frame = 5);
        game.queue_music_delayed(0);
    }
}

pub fn main<L: Host + ?Sized>(game: &mut L) {
    // The state steps, the function, the tail, drawing, ribs and baby metroid
    // run on the lease.
    session_slot!(game, Ridley[RIDLEY].base.health = 0x7fff);
    let function = session_slot!(game, Ridley[RIDLEY].function);
    run_function(game, function);
    if game.session().room.ceres_status == 0 {
        let moving = session_slot!(game, Ridley[RIDLEY].movement_enabled) != 0;
        if moving {
            game.hurt_flash();
            ridley_owned::move_ridley(game);
            ridley_owned::update_wings(game);
            randomize_tail_curliness(game);
        }
        if moving {
            ridley_owned::handle_tail(game);
            game.ceres_health_palette();
            ridley_owned::handle_ribs(game);
            game.draw_tail();
            game.draw_wings();
        }
        ridley_owned::handle_baby_metroid(game);
    }
}

pub fn run_function<L: Host + ?Sized>(game: &mut L, function: u16) {
    if !run_ceres_function(game, function) && !ridley_owned::run_ridley_function(game, function) {
        game.unreachable();
    }
}

pub(crate) fn run_ceres_function<L: Host + ?Sized>(game: &mut L, function: u16) -> bool {
    use ceres_function as f;
    match function {
        f::TRANSITION_TO_FLYING => transition_to_flying(game),
        f::INITIAL => initial(game),
        f::WAIT_TO_FADE_IN => wait_to_fade_in(game),
        f::FADE_IN_EYE => fade_in_eye(game),
        f::FADE_IN_BODY => fade_in_body(game),
        f::WAIT_TO_ROAR => wait_to_roar(game),
        f::ROAR_BEFORE_FLYING => roar_before_flying(game),
        ridley::function::CERES_LIFTOFF_ACCELERATING => liftoff_accelerating(game),
        f::LIFTOFF_DECELERATING => liftoff_decelerating(game),
        f::HOVERING => hovering(game),
        f::FIREBALLS_MOVE_TO_POSITION => fireballs_move_to_position(game),
        f::FIREBALLS_SHOOTING => fireballs_shooting(game),
        f::LUNGE_SETUP => lunge_setup(game),
        f::LUNGE => lunge(game),
        f::SWOOP_SETUP => swoop_setup(game),
        f::SWOOP_MOVE_TO_POSITION => swoop_move_to_position(game),
        f::SWOOP_AIMING_DOWN => swoop_aiming_down(game),
        f::SWOOP_AIMING_LEFT => swoop_aiming_left(game),
        f::SWOOP_AIMING_UP => swoop_aiming_up(game),
        f::SWOOP_AIMING_UP_FASTER => swoop_aiming_up_faster(game),
        ridley::function::CERES_REAL_RETREAT_RISING => real_retreat_rising(game),
        f::WRITE_PALETTES_AND_SPAWN_WALLS => write_palettes_and_spawn_walls(game),
        f::SET_UP_GETAWAY => set_up_getaway(game),
        f::GETAWAY => {}
        f::SELF_DESTRUCT_ACTIVATED => self_destruct_activated(game),
        f::SELF_DESTRUCT_ACTIVATING => self_destruct_activating(game),
        _ => return false,
    }
    true
}

pub fn randomize_tail_curliness<L: Host + ?Sized>(game: &mut L) {
    if game.session().timing.random_number >= 0xff00 {
        bg2_staging::set(
            game,
            tail::IDEAL_SEGMENT_ANGLE,
            (game.session().timing.random_number & 0xf) + 8,
        );
    }
}

pub fn hurt<L: Host + ?Sized>(game: &mut L) {
    game.hurt_flash();
    game.draw_tail();
    game.draw_wings();
    ridley_owned::handle_baby_metroid(game);
}

pub fn transition_to_flying<L: Host + ?Sized>(game: &mut L) {
    session_slot!(game, Ridley[RIDLEY]; x_velocity = 0, y_velocity = 0);
}

pub fn initial<L: Host + ?Sized>(game: &mut L) {
    if game.session().room.entry.enemy_interlock == 0 {
        session_slot!(
            game,
            Ridley[RIDLEY].function = ceres_function::WAIT_TO_FADE_IN
        );
        let wait = if game.session().room.area == NORFAIR {
            170
        } else {
            512
        };
        session_slot!(game, Ridley[RIDLEY].function_timer = wait);
        wait_to_fade_in(game);
    }
}

pub(super) fn timer_expired<L: Host + ?Sized>(game: &mut L) -> bool {
    session_slot!(game, Ridley[RIDLEY].function_timer, |function_timer| {
        function_timer.wrapping_sub(1)
    });
    (session_slot!(game, Ridley[RIDLEY].function_timer) & 0x8000) != 0
}

pub fn wait_to_fade_in<L: Host + ?Sized>(game: &mut L) {
    if timer_expired(game) {
        session_slot!(game, Ridley[RIDLEY].function = ceres_function::FADE_IN_EYE);
        session_slot!(game, Ridley[RIDLEY]; fade_index = 0, function_timer = 0);
    }
}

pub fn fade_in_eye<L: Host + ?Sized>(game: &mut L) {
    if (session_slot!(game, Ridley[RIDLEY].fade_index) & 0x8000) != 0 {
        return;
    }
    let index = session_slot!(game, Ridley[RIDLEY].fade_index);
    session_slot!(game, Ridley[RIDLEY].function_timer, |function_timer| {
        function_timer.wrapping_add(1)
    });
    if session_slot!(game, Ridley[RIDLEY].function_timer) == 0 {
        return;
    }
    session_slot!(game, Ridley[RIDLEY].function_timer = 0);
    let step = game.eye_fade_step(index);
    if step == 0xff {
        session_slot!(game, Ridley[RIDLEY]; fade_index = 0, function = ceres_function::FADE_IN_BODY,
            movement_enabled = 1);
    } else {
        session_slot!(game, Ridley[RIDLEY].fade_index = index.wrapping_add(1));
        let colour = 3 * isize::from(step);
        {
            let value = game.ceres_eye_fade(colour);
            game.set_colour(252, value);
            let value = game.ceres_eye_fade(colour + 1);
            game.set_colour(253, value);
            let value = game.ceres_eye_fade(colour + 2);
            game.set_colour(254, value);
        }
    }
}

pub fn fade_in_body<L: Host + ?Sized>(game: &mut L) {
    session_slot!(game, Ridley[RIDLEY].function_timer, |function_timer| {
        function_timer.wrapping_add(1)
    });
    if session_slot!(game, Ridley[RIDLEY].function_timer) < 2 {
        return;
    }
    session_slot!(game, Ridley[RIDLEY].function_timer = 0);
    let mut index = session_slot!(game, Ridley[RIDLEY].fade_index);
    let (mut first, mut second) = (145, 241);
    for _ in 0..11 {
        let colour = game.ceres_body_fade((index >> 1) as isize);
        {
            game.set_colour(first, colour);
            game.set_colour(second, colour);
        }
        index = index.wrapping_add(2);
        first += 1;
        second += 1;
    }
    if index >= 0x160 {
        if game.session().room.area == NORFAIR {
            session_slot!(game, Ridley[RIDLEY].base.layer = 2);
        }
        session_slot!(game, Ridley[RIDLEY].base.properties, |old| old
            & !(K_ENEMY_PROPS_TANGIBLE as u16));
        session_slot!(game, Ridley[RIDLEY]; fade_index = 0, function = ceres_function::WAIT_TO_ROAR,
            function_timer = 4);
        game.queue_music_delayed(5);
    } else {
        session_slot!(game, Ridley[RIDLEY].fade_index = index);
    }
}

pub fn wait_to_roar<L: Host + ?Sized>(game: &mut L) {
    if timer_expired(game) {
        ridley_owned::set_instruction_list(ceres_instruction_list::FIGHT_INTRO_ROAR, game);
        session_slot!(game, Ridley[RIDLEY].function_timer = 0);
        session_slot!(
            game,
            Ridley[RIDLEY].function = ceres_function::ROAR_BEFORE_FLYING
        );
        if game.session().room.area != NORFAIR {
            session_slot!(game, Ridley[RIDLEY].function_timer = 252);
        }
    }
}

pub fn roar_before_flying<L: Host + ?Sized>(game: &mut L) {
    if !timer_expired(game) {
        return;
    }
    if game.session().room.area == NORFAIR {
        session_slot!(game, Ridley[RIDLEY].function_timer = 2);
        let step = session_slot!(game, Ridley[RIDLEY].fade_index).wrapping_add(1);
        session_slot!(game, Ridley[RIDLEY].fade_index = step);
        if !fade_in_background(step.wrapping_sub(1), game) {
            return;
        }
        game.move_liquid(440, -96i32 as u16, 32);
    }
    session_slot!(game, Ridley[RIDLEY].fade_index = 0);
    ridley_owned::set_instruction_list(ceres_instruction_list::TRANSITION_TO_FLYING, game);
    session_slot!(game, Ridley[RIDLEY]; wings_timer_delta = 8, wings_timer = 8);
    ridley_owned::activate_tail_segments(game);
    bg2_staging::set(game, tail::FUNCTION, 1);
    session_slot!(
        game,
        Ridley[RIDLEY].function = ceres_function::TRANSITION_TO_FLYING
    );
}

pub fn fade_in_background<L: Host + ?Sized>(step: u16, game: &mut L) -> bool {
    let colours = game.ceres_background_fade(step as isize);
    if colours == 0 {
        return true;
    }
    game.copy_palette(0xe2, 0xa6, colours, 0xe);
    false
}

pub fn liftoff_accelerating<L: Host + ?Sized>(game: &mut L) {
    session_slot!(game, Ridley[RIDLEY].y_velocity, |old| old.wrapping_sub(16));
    if below(session_slot!(game, Ridley[RIDLEY].base.y_pos), 112) {
        session_slot!(
            game,
            Ridley[RIDLEY].function = ceres_function::LIFTOFF_DECELERATING
        );
        liftoff_decelerating(game);
    }
}

pub fn liftoff_decelerating<L: Host + ?Sized>(game: &mut L) {
    session_slot!(game, Ridley[RIDLEY].y_velocity, |old| old.wrapping_add(20));
    if below(session_slot!(game, Ridley[RIDLEY].base.y_pos), 80) {
        session_slot!(game, Ridley[RIDLEY]; function = ceres_function::HOVERING, fight_mode = 1);
    }
}

pub fn hovering<L: Host + ?Sized>(game: &mut L) {
    if session_slot!(game, Ridley[RIDLEY].hit_counter) >= 0x64 {
        session_slot!(game, Ridley[RIDLEY].fight_mode = 0);
        session_slot!(
            game,
            Ridley[RIDLEY].function = function::FAKE_RETREAT_MOVE_TO_POSITION
        );
        ridley_owned::fake_retreat_move_to_position(game);
    } else if below(game.session().inventory.health.current, 30) {
        session_slot!(game, Ridley[RIDLEY].fight_mode = 0);
        session_slot!(
            game,
            Ridley[RIDLEY].function = function::CERES_REAL_RETREAT_RISING
        );
        real_retreat_rising(game);
    } else if !hovering_movement(game) || {
        let timer = session_slot!(game, Ridley[RIDLEY].timer).wrapping_add(1);
        session_slot!(game, Ridley[RIDLEY].timer = timer);
        timer >= 0x7c
    } {
        let attack = game.ceres_attack((game.session().timing.random_number & 0xf) as isize);
        session_slot!(game, Ridley[RIDLEY]; function = attack, timer = 0);
    }
}

pub fn hovering_movement<L: Host + ?Sized>(game: &mut L) -> bool {
    let rect = Rect {
        x: 192,
        y: 100,
        w: 8,
        h: 8,
    };
    ridley_owned::ceres_accelerate(0, 0, rect.x, rect.y, game);
    ridley_owned::separated_from_rect(0, rect, game)
}

pub fn fireballs_move_to_position<L: Host + ?Sized>(game: &mut L) {
    crate::probe::hit("ceres_ridley.fireballs_move_to_position");
    let mut speed = abs16(session_slot!(game, Ridley[RIDLEY].y_velocity));
    if speed < 0x80 {
        speed = 128;
    }
    let y_velocity = if session_slot!(game, Ridley[RIDLEY].y_velocity) & 0x8000 != 0 {
        -(speed as i32)
    } else {
        speed as i32
    };
    session_slot!(game, Ridley[RIDLEY].y_velocity = y_velocity as u16);
    ridley_owned::ceres_accelerate(
        0,
        0,
        session_slot!(game, Ridley[RIDLEY].base.x_pos),
        88,
        game,
    );
    if below(session_slot!(game, Ridley[RIDLEY].base.y_pos), 80) {
        let timer = session_slot!(game, Ridley[RIDLEY].timer).wrapping_add(1);
        session_slot!(game, Ridley[RIDLEY].timer = timer);
        if timer >= 0x30 {
            session_slot!(game, Ridley[RIDLEY].function = ceres_function::SWOOP_SETUP);
        }
    } else if below(session_slot!(game, Ridley[RIDLEY].base.y_pos), 128) {
        let copied = session_slot!(game, Ridley[RIDLEY].base.x_pos);
        session_slot!(game, Ridley[RIDLEY].target_x = copied);
        let copied = session_slot!(game, Ridley[RIDLEY].base.y_pos);
        session_slot!(game, Ridley[RIDLEY].target_y = copied);
        ridley_owned::set_instruction_list(instruction_list::SHOOT_FIREBALLS, game);
        session_slot!(
            game,
            Ridley[RIDLEY].function = ceres_function::FIREBALLS_SHOOTING
        );
        session_slot!(game, Ridley[RIDLEY].timer = 224);
        fireballs_shooting(game);
    }
}

pub fn fireballs_shooting<L: Host + ?Sized>(game: &mut L) {
    let jitter = if game.session().timing.random_number & 0x8000 != 0 {
        -(game.session().timing.random_number as i32 & 7)
    } else {
        game.session().timing.random_number as i32 & 7
    } as u16;
    // The Y add's ADC carries in from the X add, so a negative jitter adds one more.
    let (x, carry) = crate::carry::adc(session_slot!(game, Ridley[RIDLEY].target_x), jitter, false);
    let y = crate::carry::adc(session_slot!(game, Ridley[RIDLEY].target_y), jitter, carry).0;
    ridley_owned::ceres_accelerate(0, 0, x, y, game);
    let timer = (session_slot!(game, Ridley[RIDLEY].timer) as i16).wrapping_sub(1);
    session_slot!(game, Ridley[RIDLEY].timer = timer as u16);
    if timer < 0 {
        session_slot!(game, Ridley[RIDLEY]; timer = 0, function = ceres_function::HOVERING);
    }
}

pub fn lunge_setup<L: Host + ?Sized>(game: &mut L) {
    ridley_owned::set_instruction_list(ceres_instruction_list::LUNGE, game);
    session_slot!(game, Ridley[RIDLEY]; function = ceres_function::LUNGE, function_timer = 64);
    lunge(game);
}

pub fn lunge<L: Host + ?Sized>(game: &mut L) {
    let x = game.session().player.x;
    let mut y = game.session().player.y.wrapping_sub(68);
    if below(game.session().player.y, 132) {
        y = 64;
    }
    ridley_owned::ceres_accelerate(0, 0xd, x, y, game);
    let rect = Rect { x, y, w: 2, h: 2 };
    if !ridley_owned::separated_from_rect(0, rect, game) || {
        let expired =
            (session_slot!(game, Ridley[RIDLEY].function_timer) as i16).wrapping_sub(1) < 0;
        session_slot!(game, Ridley[RIDLEY].function_timer, |function_timer| {
            function_timer.wrapping_sub(1)
        });
        expired
    } {
        session_slot!(game, Ridley[RIDLEY]; timer = 0, function = ceres_function::HOVERING);
    }
}

pub fn swoop_setup<L: Host + ?Sized>(game: &mut L) {
    session_slot!(
        game,
        Ridley[RIDLEY].function = ceres_function::SWOOP_MOVE_TO_POSITION
    );
    session_slot!(game, Ridley[RIDLEY]; function_timer = 10, swoop_angle = 0);
    bg2_staging::set(game, tail::IDLE_WHIP, 0);
    swoop_move_to_position(game);
}

pub fn swoop_move_to_position<L: Host + ?Sized>(game: &mut L) {
    ridley_owned::ceres_accelerate(0, 1, 192, 80, game);
    if below(session_slot!(game, Ridley[RIDLEY].base.y_pos), 96) {
        session_slot!(
            game,
            Ridley[RIDLEY].function = ceres_function::SWOOP_AIMING_DOWN
        );
        session_slot!(game, Ridley[RIDLEY]; function_timer = 10, swoop_angle = 0);
    }
}

pub(super) fn swoop_stage<L: Host + ?Sized>(
    game: &mut L,
    angle_step: i32,
    target_angle: i32,
    next: u16,
    frames: u16,
) -> bool {
    ridley_owned::calculate_swoop_velocities(angle_step as u16, target_angle as u16, 768, game);
    if timer_expired(game) {
        session_slot!(game, Ridley[RIDLEY]; function = next, function_timer = frames);
        return true;
    }
    false
}

pub fn swoop_aiming_down<L: Host + ?Sized>(game: &mut L) {
    swoop_stage(game, -32, -1024, ceres_function::SWOOP_AIMING_LEFT, 36);
}

pub fn swoop_aiming_left<L: Host + ?Sized>(game: &mut L) {
    if swoop_stage(game, -512, -16384, ceres_function::SWOOP_AIMING_UP, 28) {
        bg2_staging::set(game, tail::WHIP_REQUEST, 1);
    }
}

pub fn swoop_aiming_up<L: Host + ?Sized>(game: &mut L) {
    swoop_stage(
        game,
        -512,
        -30720,
        ceres_function::SWOOP_AIMING_UP_FASTER,
        1,
    );
}

pub fn swoop_aiming_up_faster<L: Host + ?Sized>(game: &mut L) {
    ridley_owned::calculate_swoop_velocities(-768i32 as u16, -30720i32 as u16, 768, game);
    if timer_expired(game) {
        session_slot!(game, Ridley[RIDLEY]; function = ceres_function::HOVERING, timer = 0);
        bg2_staging::set(game, tail::IDLE_WHIP, 1);
    }
}

pub fn real_retreat_rising<L: Host + ?Sized>(game: &mut L) {
    session_slot!(game, Ridley[RIDLEY].min_y = -192i32 as u16);
    ridley_owned::ceres_accelerate(0, 1, 192, -128i32 as u16, game);
    if (session_slot!(game, Ridley[RIDLEY].base.y_pos) as i32 + 128) & 0x8000 != 0 {
        real_retreat_risen(game);
    }
}

pub fn real_retreat_risen<L: Host + ?Sized>(game: &mut L) {
    session_slot!(
        game,
        Ridley[RIDLEY].function = ceres_function::WRITE_PALETTES_AND_SPAWN_WALLS
    );
    session_slot!(game, Ridley[RIDLEY].function_timer = 64);
    write_palettes_and_spawn_walls(game);
}

pub fn write_palettes_and_spawn_walls<L: Host + ?Sized>(game: &mut L) {
    if timer_expired(game) {
        spawn_getaway_walls(game);
        session_slot!(game, Ridley[RIDLEY]; x_velocity = 0, y_velocity = 0);
        bg2_staging::set(game, tail::FUNCTION, 0);
        session_slot!(
            game,
            Ridley[RIDLEY].function = ceres_function::SET_UP_GETAWAY
        );
        {
            game.copy_palette(0xa2, 0xa6, ceres_data::RETREAT_SPRITE_PALETTE, 0xf);
            game.copy_palette(0x42, 0xa6, ceres_data::RETREAT_BACKGROUND_PALETTE, 8);
            game.copy_palette(0x1e2, 0xa6, ceres_data::RETREAT_BACKGROUND_PALETTE, 8);
        };
    }
}

pub fn set_up_getaway<L: Host + ?Sized>(game: &mut L) {
    session_slot!(game, Ridley[RIDLEY].function = ceres_function::GETAWAY);
    game.session_mut().room.ceres_status = RIDLEY_ESCAPING;
    set_up_mode7(game);
}

pub fn spawn_getaway_walls<L: Host + ?Sized>(game: &mut L) {
    for population in ceres_data::GETAWAY_WALLS {
        game.spawn_population(0xa6, population);
    }
}

pub fn self_destruct_activated<L: Host + ?Sized>(game: &mut L) {
    game.emergency_text_glow();
}

pub fn set_up_mode7<L: Host + ?Sized>(game: &mut L) {
    game.set_bg_mode(7);
    game.set_mode7_irq(1);
    game.set_mode7_settings(MODE7_TRANSPARENT_OVERFLOW);
    {
        game.set_affine(0, 256);
        game.set_affine(1, 256);
        game.set_affine(2, 256);
        game.set_affine(3, 256);
    }
    {
        game.set_affine(4, 64);
        game.set_affine(5, 64);
    }
    session_slot!(game, Ridley[RIDLEY]; getaway_finished = 0, getaway_index = 0);
    session_slot!(game, Ridley[RIDLEY]; swoop_angle = 1, mode7_x = -128i32 as u16);
    game.set_bg1_scroll(true, -128i32 as u16);
    session_slot!(game, Ridley[RIDLEY].mode7_y = 32);
    game.set_bg1_scroll(false, 32);
    session_slot!(game, Ridley[RIDLEY]; mode7_zoom = 2048, unread_8022 = 1024);
}

pub fn handle_getaway_if_active<L: Host + ?Sized>(game: &mut L) {
    if session_slot!(game, Ridley[RIDLEY].getaway_finished) == 0 {
        handle_getaway(game);
    }
}

pub fn handle_getaway<L: Host + ?Sized>(game: &mut L) {
    let index = session_slot!(game, Ridley[RIDLEY].getaway_index);
    session_slot!(game, Ridley[RIDLEY].getaway_index = index.wrapping_add(2));
    if index == 0 {
        game.queue_sound(1, 0x4e, 6);
    }
    if index == 208 {
        game.begin_ceres_push();
        {
            game.session_mut().feedback.earthquake_kind = 35;
            game.session_mut().feedback.earthquake_timer = 64;
        };
    }
    let step = (index >> 1) as isize;
    if game.ceres_getaway_zoom(step) == 0xffff {
        session_slot!(game, Ridley[RIDLEY].getaway_finished, |getaway_finished| {
            getaway_finished.wrapping_add(1)
        });
        game.set_bg_mode(9);
        game.set_mode7_settings(0);
        {
            game.set_affine(0, 0);
            game.set_affine(1, 0);
            game.set_affine(2, 0);
            game.set_affine(3, 0);
            game.set_affine(4, 0);
            game.set_affine(5, 0);
            game.set_bg1_scroll(true, 0);
            game.set_bg1_scroll(false, 0);
        }
        session_slot!(
            game,
            Ridley[RIDLEY].function = ceres_function::SELF_DESTRUCT_ACTIVATING
        );
        session_slot!(game, Ridley[RIDLEY].function_timer = 0);
    } else {
        session_slot!(
            game,
            Ridley[RIDLEY].mode7_zoom = game.ceres_getaway_zoom(step)
        );
        let velocity = game.ceres_getaway_velocity(step);
        let y = velocity
            .y
            .wrapping_add(session_slot!(game, Ridley[RIDLEY].mode7_y));
        session_slot!(game, Ridley[RIDLEY].mode7_y = y);
        game.set_bg1_scroll(false, y);
        let x = session_slot!(game, Ridley[RIDLEY].mode7_x).wrapping_sub(velocity.x);
        session_slot!(game, Ridley[RIDLEY].mode7_x = x);
        game.set_bg1_scroll(true, x);
        palette_by_zoom(session_slot!(game, Ridley[RIDLEY].mode7_zoom), game);
        session_slot!(game, Ridley[RIDLEY].swoop_angle, |swoop_angle| swoop_angle
            .wrapping_add(48));
        set_mode7_rotation(game);
        animate_mode7_baby_metroid(game);
        animate_mode7_wings(game);
    }
}

pub fn set_mode7_rotation<L: Host + ?Sized>(game: &mut L) {
    let angle = session_slot!(byte game, Ridley[RIDLEY].swoop_angle[1]) as u16;
    {
        game.set_affine(
            0,
            signed_scale(
                cosine_256(angle),
                session_slot!(game, Ridley[RIDLEY].mode7_zoom),
            ),
        );
        game.set_affine(
            1,
            signed_scale(
                sine_256(angle),
                session_slot!(game, Ridley[RIDLEY].mode7_zoom),
            ),
        );
        game.set_affine(
            2,
            signed_scale(
                -(sine_256(angle) as i32) as u16,
                session_slot!(game, Ridley[RIDLEY].mode7_zoom),
            ),
        );
        game.set_affine(
            3,
            signed_scale(
                cosine_256(angle),
                session_slot!(game, Ridley[RIDLEY].mode7_zoom),
            ),
        );
    }
}

pub(super) fn cosine_256(angle: u16) -> u16 {
    sine_multiply(angle.wrapping_add(64), 0x100) as u16
}

pub(super) fn sine_256(angle: u16) -> u16 {
    sine_multiply(angle, 0x100) as u16
}

pub fn sine_multiply(angle: u16, amplitude: i16) -> i16 {
    ((SINE_COSINE[((angle as i32 & 0xff) + 64) as usize] as i32 * amplitude as i32) >> 8) as i16
}

pub(super) fn signed_scale(a: u16, b: u16) -> u16 {
    let product = ((abs16(a) as i32 * b as i32) >> 8) as u16;
    (if a & 0x8000 != 0 {
        -(product as i32)
    } else {
        product as i32
    }) as u16
}

pub fn animate_mode7_baby_metroid<L: Host + ?Sized>(game: &mut L) {
    if (game.frame() & 3) == 0 {
        let frame = session_slot!(game, Ridley[RIDLEY].baby_metroid_frame).wrapping_add(1) & 3;
        session_slot!(game, Ridley[RIDLEY].baby_metroid_frame = frame);
        let list = game.ceres_baby_metroid_frame(frame as isize);
        game.queue_mode7_transfer(0xa6, list);
    }
}

pub fn animate_mode7_wings<L: Host + ?Sized>(game: &mut L) {
    if (game.frame() & 7) == 0 {
        let frame = session_slot!(game, Ridley[RIDLEY].mode7_wings_frame).wrapping_add(1) & 1;
        session_slot!(game, Ridley[RIDLEY].mode7_wings_frame = frame);
        let list = game.ceres_wing_frame(frame as isize);
        game.queue_mode7_transfer(0xa6, list);
    }
}

pub fn palette_by_zoom<L: Host + ?Sized>(zoom: u16, game: &mut L) {
    let colours = ZOOM_PALETTES.wrapping_add(32 * u16::from(zoom.to_ne_bytes()[1]));
    game.copy_palette(0xa2, 0xa6, colours, 0xf);
}

pub fn self_destruct_activating<L: Host + ?Sized>(game: &mut L) {
    match session_slot!(game, Ridley[RIDLEY].function_timer) {
        0 => load_text_colours(game),
        2 => load_escape_timer_tiles(game),
        4 => load_escape_background_tiles(game),
        6 => emergency_text_appears(game),
        8 => japanese_text_appears(game),
        10 => type_self_destruct_text(game),
        12 => start_escape(game),
        _ => game.unreachable(),
    }
}

pub(super) fn next_step<L: Host + ?Sized>(game: &mut L) {
    for _ in 0..2 {
        session_slot!(game, Ridley[RIDLEY].function_timer, |function_timer| {
            function_timer.wrapping_add(1)
        });
    }
}

pub fn load_text_colours<L: Host + ?Sized>(game: &mut L) {
    game.set_colour(97, game.colour(1));
    game.set_colour(99, game.colour(3));
    game.set_colour(81, game.colour(17));
    game.set_colour(83, game.colour(19));
    // The escape timer sprite tile transfer entries ($A6:C4CB).
    session_slot!(game, Ridley[RIDLEY].fade_index = 0xc4cb);
    next_step(game);
    load_escape_timer_tiles(game);
}

pub fn load_escape_timer_tiles<L: Host + ?Sized>(game: &mut L) {
    if game.queue_escape_timer_tiles() {
        // The Ceres escape timer BG1/2 tile transfer entries ($A6:C4FE).
        session_slot!(game, Ridley[RIDLEY].fade_index = 0xc4fe);
        next_step(game);
        load_escape_background_tiles(game);
    }
}

pub fn load_escape_background_tiles<L: Host + ?Sized>(game: &mut L) {
    if game.queue_escape_timer_tiles() {
        next_step(game);
        game.queue_getaway_tiles();
        session_slot!(game, Ridley[RIDLEY].fade_index = 128);
        game.queue_music_delayed(7);
    }
}

pub fn emergency_text_appears<L: Host + ?Sized>(game: &mut L) {
    game.emergency_text_glow();
    let timer = session_slot!(game, Ridley[RIDLEY].fade_index);
    session_slot!(game, Ridley[RIDLEY].fade_index = timer.wrapping_sub(1));
    if timer == 1 {
        // Typewriter text - Ceres escape timer ($A6:C450).
        session_slot!(game, Ridley[RIDLEY].typewriter_list = 0xc450);
        session_slot!(game, Ridley[RIDLEY]; typewriter_tilemap = 0, typewriter_timer = 0,
            typewriter_timer_reset = 0, typewriter_stroke_timer = 0);
        session_slot!(game, Ridley[RIDLEY].fade_index = 32);
        if game.session().options.japanese_text == 0 {
            next_step(game);
        }
        next_step(game);
    }
}

pub fn japanese_text_appears<L: Host + ?Sized>(game: &mut L) {
    let timer = session_slot!(game, Ridley[RIDLEY].fade_index);
    session_slot!(game, Ridley[RIDLEY].fade_index = timer.wrapping_sub(1));
    if timer == 1 {
        next_step(game);
        game.queue_escape_tile_table();
    }
    type_self_destruct_text(game);
}

pub fn type_self_destruct_text<L: Host + ?Sized>(game: &mut L) {
    game.emergency_text_glow();
    if game.type_text(ESCAPE_TEXT_TILES) {
        next_step(game);
    }
}

pub fn start_escape<L: Host + ?Sized>(game: &mut L) {
    game.emergency_text_glow();
    transition_to_flying(game);
    session_slot!(game, Ridley[RIDLEY].function_timer = 0);
    session_slot!(
        game,
        Ridley[RIDLEY].function = ceres_function::SELF_DESTRUCT_ACTIVATED
    );
    {
        game.session_mut().timing.timer_status = 1;
        game.session_mut().room.ceres_status = ESCAPE_STARTED;
    };
    game.set_boss_bit(1);
}
