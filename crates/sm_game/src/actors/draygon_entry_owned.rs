//! Draygon initialization, intro and main orchestration over owned state.
#![forbid(unsafe_code)]
use super::{
    BG2_TILEMAP_SIZE, BG2_X_OFFSET, BG2_Y_OFFSET, INITIAL_SWOOP_Y_ACCELERATION, INTRO_DANCE,
    INTRO_DELAY, PALETTE_7, RIGHT_RESET_OFFSET,
};
use crate::actors::draygon_flight::owned::fire_wall_turret;
use crate::actors::draygon_host::{
    ARMS, BANK, BODY, Draygon, EYE, SIGN, TAIL, function, instruction_list as list,
};
use crate::actors::session_slots::session_slot;
pub trait Host:
    crate::actors::draygon_sweep::owned::Host + crate::actors::draygon_death::owned::Host
{
    fn queue_intro_transfer(&mut self);
    fn spawn_intro_trails(&mut self);
    fn step_intro_trajectory(&mut self);
    fn install_initial_palette(&mut self);
    fn install_bg2_scroll_hook(&mut self);
    fn install_layers(&mut self);
    fn disable_minimap_for_boss(&mut self);
    fn dispatch_draygon_other(&mut self, address: u32, key: u16);
    fn set_bg2_scroll(&mut self, horizontal: bool, value: u16);
}
pub fn fight_intro<L: Host>(first: bool, game: &mut L) {
    fire_wall_turret(game);
    let limit = if first { INTRO_DELAY } else { INTRO_DANCE };
    if session_slot!(game, Draygon[BODY].function_timer).wrapping_sub(limit) & SIGN != 0 {
        if first {
            if session_slot!(game, Draygon[BODY].function_timer) == 0 {
                game.queue_intro_transfer();
                game.spawn_intro_trails();
            }
        } else {
            game.step_intro_trajectory();
        }
        session_slot!(game, Draygon[BODY].function_timer, |old| old
            .wrapping_add(1));
    } else {
        session_slot!(
            game,
            Draygon[BODY].function = if first {
                function::FIGHT_INTRO_DANCE
            } else {
                function::SWOOP_RIGHT_SETUP
            }
        );
        session_slot!(game, Draygon[BODY].function_timer = 0);
        if first {
            let x = session_slot!(game, Draygon[BODY].base.x_pos);
            session_slot!(game, Draygon[BODY].left_reset_x = x);
            session_slot!(
                game,
                Draygon[BODY].right_reset_x = x.wrapping_add(RIGHT_RESET_OFFSET)
            );
            let copied = session_slot!(game, Draygon[BODY].base.y_pos);
            session_slot!(game, Draygon[BODY].reset_y = copied);
            let copied = session_slot!(game, Draygon[BODY].left_reset_x);
            session_slot!(game, Draygon[BODY].base.x_pos = copied);
            let copied = session_slot!(game, Draygon[BODY].reset_y);
            session_slot!(game, Draygon[BODY].base.y_pos = copied);
            session_slot!(
                game,
                Draygon[BODY].swoop_y_acceleration = INITIAL_SWOOP_Y_ACCELERATION
            );
        }
    }
}
pub fn prepare_initialization<H: Host>(game: &mut H) {
    game.install_initial_palette();
    for word in game
        .session_mut()
        .actors
        .extended
        .tilemaps
        .iter_mut()
        .rev()
        .flat_map(|block| block.iter_mut().rev())
    {
        *word = 824;
    }
    session_slot!(
        game,
        Draygon[game.session().actors.controls.selected as i32]
            .base
            .palette_index = PALETTE_7
    );
    // The native bank word also clears byte178F before the address word is stored.
    game.install_bg2_scroll_hook();
    game.session_mut().actors.controls.process_all = 0xffff;
    for (k, instruction) in [
        (BODY, list::BODY_LEFT_IDLE),
        (EYE, list::EYE_LEFT_IDLE),
        (TAIL, list::TAIL_LEFT_FAKE_TAIL_WHIP),
        (ARMS, list::ARMS_LEFT_NEAR_SWOOP_APEX),
    ] {
        session_slot!(game, Draygon[k].base.current_instruction = instruction);
    }
    game.session_mut().actors.drawing.tilemap_size = BG2_TILEMAP_SIZE;
    session_slot!(
        game,
        Draygon[game.session().actors.controls.selected as i32].function =
            function::FIGHT_INTRO_INITIAL_DELAY
    );
}
pub fn complete_initialization<H: Host>(game: &mut H) {
    session_slot!(game, Draygon[BODY].turret_5_destroyed = 1);
    game.disable_minimap_for_boss();
}
pub fn initialize<H: Host>(game: &mut H) {
    prepare_initialization(game);
    game.install_layers();
    complete_initialization(game);
}
pub fn main_ai<H: Host>(body: bool, game: &mut H) {
    let selected = game.session().actors.controls.selected;
    let function = session_slot!(game, Draygon[i32::from(selected)].function);
    let key = if body {
        selected
    } else {
        game.session().actors.controls.selected
    };
    run_function_on(game, function, key);
    if !body {
        return;
    }
    let x = session_slot!(game, Draygon[BODY].base.x_pos);
    for key in [EYE, TAIL, ARMS] {
        session_slot!(game, Draygon[key].base.x_pos = x);
    }
    let y = session_slot!(game, Draygon[BODY].base.y_pos);
    for key in [EYE, TAIL, ARMS] {
        session_slot!(game, Draygon[key].base.y_pos = y);
    }
}
pub fn run_function_on<L: Host>(game: &mut L, function: u16, k: u16) {
    use crate::actors::{draygon_grab::owned as grab, draygon_sweep::owned as sweep};
    match function {
        function::FIGHT_INTRO_INITIAL_DELAY => fight_intro(true, game),
        function::FIGHT_INTRO_DANCE => fight_intro(false, game),
        function::SWOOP_RIGHT_SETUP => sweep::swoop_setup(true, game),
        function::SWOOP_RIGHT_DESCENDING => sweep::swoop(true, false, game),
        function::SWOOP_RIGHT_APEX => sweep::swoop_apex(true, game),
        function::SWOOP_RIGHT_ASCENDING => sweep::swoop(true, true, game),
        function::SWOOP_LEFT_DESCENDING => sweep::swoop(false, false, game),
        function::SWOOP_LEFT_APEX => sweep::swoop_apex(false, game),
        function::SWOOP_LEFT_ASCENDING => sweep::swoop(false, true, game),
        function::GOOP_RIGHT_SETUP => crate::actors::draygon_flight::owned::goop_setup(true, game),
        function::GOOP_LEFT_SETUP => crate::actors::draygon_flight::owned::goop_setup(false, game),
        function::GOOP_RIGHT_MOVE_UNTIL_IN_RANGE => {
            crate::actors::draygon_flight::owned::goop_move_until_in_range(true, game)
        }
        function::GOOP_LEFT_MOVE_UNTIL_IN_RANGE => {
            crate::actors::draygon_flight::owned::goop_move_until_in_range(false, game)
        }
        function::GOOP_RIGHT_FIRING => {
            crate::actors::draygon_flight::owned::goop_firing(true, game)
        }
        function::GOOP_LEFT_FIRING => {
            crate::actors::draygon_flight::owned::goop_firing(false, game)
        }
        function::GOOP_RIGHT_MOVE_UNTIL_OFF_SCREEN => {
            crate::actors::draygon_flight::owned::goop_move_until_off_screen(true, game)
        }
        function::GOOP_LEFT_MOVE_UNTIL_OFF_SCREEN => {
            crate::actors::draygon_flight::owned::goop_move_until_off_screen(false, game)
        }
        function::CHASE_SAMUS => grab::chase_samus(game),
        function::REPELLED_BY_GRAPPLE => grab::repelled_by_grapple(game),
        function::GRABBED_MOVING_TO_TARGET => grab::move_to_target(k, game),
        function::GRABBED_RISING_SPIRAL => grab::rising_spiral(k, game),
        function::GRABBED_TAIL_WHIP => grab::tail_whip(k, game),
        function::GRABBED_FINAL_TAIL_WHIPS_START => grab::final_tail_whips(k, game),
        function::GRABBED_FINAL_TAIL_WHIPS_ONGOING => grab::move_samus_with_draygon(k, game),
        function::FLAIL_TAIL_AND_FLY_STRAIGHT_UP => grab::flail_tail_and_fly_straight_up(game),
        function::FLY_STRAIGHT_UP => grab::fly_straight_up(game),
        function::NONE => {}
        // The left swoop setup's diagnostic, the death and anything else.
        function::SWOOP_LEFT_SETUP => {
            fire_wall_turret(game);
            sweep::swoop_setup(false, game);
        }
        function::DEATH_DRIFT_TO_DEATH_SPOT => {
            crate::actors::draygon_death::owned::drift_to_death_spot(game)
        }
        function::DEATH_WAIT_FOR_EVIRS => crate::actors::draygon_death::owned::wait_for_evirs(game),
        function::DEATH_BURIED_BY_EVIRS => {
            crate::actors::draygon_death::owned::buried_by_evirs(game)
        }
        _ => game.dispatch_draygon_other((u32::from(BANK) << 16) | u32::from(function), k),
    }
}
pub fn set_bg2_scroll<S: Host>(game: &mut S) {
    let x = i32::from(session_slot!(game, Draygon[BODY].graphics_x_displacement))
        + i32::from(game.session().room.camera.layer1[0])
        - i32::from(session_slot!(game, Draygon[BODY].base.x_pos))
        - BG2_X_OFFSET;
    game.set_bg2_scroll(true, x as u16);
    let y = i32::from(session_slot!(game, Draygon[BODY].graphics_y_displacement))
        + i32::from(game.session().room.camera.layer1[1])
        - i32::from(session_slot!(game, Draygon[BODY].base.y_pos))
        - BG2_Y_OFFSET;
    game.set_bg2_scroll(false, y as u16);
}
