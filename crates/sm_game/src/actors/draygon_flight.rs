//! Draygon's fight intro, wall turrets, setup, goop passes and the
//! instructions its body scripts call (bank $A5:8687-$A5:9E0A).
//! Extended motion words stay native compatibility state until the whole boss migrates.
use crate::actors::boss_lease::BossState;
use crate::actors::boss_lease::bg2_staging;
use crate::actors::draygon_host::Lease;
use crate::actors::draygon_host::{
    ARMS, BANK, BODY, Draygon, EYE, Host, INTANGIBLE, SIGN, TAIL, abs16, function,
    instruction_list as list,
};
use crate::actors::host::BossEffects as _;
use crate::actors::session_slots::session_slot;
use crate::display::transfer::{Source, TilemapUpload, queue_tilemap};
use crate::enemy_script::EnemyScript;
use crate::menus::prelude::{boss, display_scroll, owned};

/// Sprite object 3Dh: Draygon foaming at the mouth, 20h px ahead of and 10h
/// px above the body.
const FOAM: u16 = 0x3d;
const FOAM_X: u16 = 0x20;
const FOAM_Y: u16 = 0x10;
/// Sprite object 18h: a bubble.
const BUBBLE: u16 = 0x18;
const BUBBLE_OFFSET: u16 = 0x10;
const TURRET_PROJECTILE: u16 = 0x8e5e;
const TURRET_SPEED: u16 = 3;
const TURRET_FRAMES: u16 = 0x3f;
/// The fight intro waits 100h frames, then dances for 4D0h.
const INTRO_DELAY: u16 = 0x100;
const INTRO_DANCE: u16 = 0x4d0;
/// The intro's VRAM write: 600h bytes from $B1:9400 to VRAM $6D00.
const INTRO_TRANSFER: [u16; 4] = [0x600, 0x9400, 0xb1, 0x6d00];
/// The right side reset X is 2A0h right of the left.
const RIGHT_RESET_OFFSET: u16 = 0x2a0;
const INITIAL_SWOOP_Y_ACCELERATION: u16 = 0x18;
/// $A5:A217: the target sprite palettes 1 and 2.
const TARGET_PALETTES: u32 = 0xa5_a217;
const PALETTE_7: u16 = 0xe00;
/// $A5:9342: the enemy graphics drawn hook that sets the BG2 scroll.
const SET_BG2_SCROLL_HOOK: u16 = 0x9342;
const GRAPHICS_DRAWN_HOOK: usize = 0x178c;
const GRAPHICS_DRAWN_HOOK_BANK: usize = 0x178e;
const BG2_TILEMAP_SIZE: u16 = 0x400;
/// The BG2 scroll follows the body, 1C2h left and C0h up.
const BG2_X_OFFSET: i32 = 0x1c2;
const BG2_Y_OFFSET: i32 = 0xc0;
/// Goop passes fly at Y 180h, oscillating 20h, from X FFB0h, firing within
/// D0h of Samus 10h goops at 1 in 16 frames, and end past X 2A0h or -50h.
const GOOP_Y: u16 = 0x180;
const GOOP_Y_AMPLITUDE: u16 = 0x20;
const GOOP_START_X: u16 = 0xffb0;
const GOOP_RANGE: u16 = 0xd0;
const GOOPS: u16 = 0x10;
const GOOP_CHANCE: u16 = 0xf;
const OFF_SCREEN_RIGHT: u16 = 0x2a0;
const OFF_SCREEN_LEFT: u16 = 0x50;
const RIGHT_RESET_X: u16 = 0x250;
const RESET_Y: u16 = 0xffb0;

/// Start `part`'s left or right instruction list, by `facing_key`'s facing.
pub fn facing_animation<S: BossState>(
    facing_key: i32,
    part_key: i32,
    instructions: [u16; 2],
    game: &mut S,
) {
    let instruction =
        instructions[usize::from(session_slot!(game, Draygon[facing_key].facing_right) != 0)];
    session_slot!(game, Draygon[part_key].base; current_instruction = instruction, instruction_timer = 1);
}
/// On frames matching `frame_mask`, Draygon foams at the mouth. Was
/// `shed_particle`.
pub fn foam_at_mouth<L: Lease>(k: u16, frame_mask: u16, game: &mut L) {
    if session_slot!(game, Draygon[k as i32].base.frame_counter) & frame_mask == 0 {
        let dx = if session_slot!(game, Draygon[BODY].facing_right) != 0 {
            FOAM_X
        } else {
            0u16.wrapping_sub(FOAM_X)
        };
        let x = session_slot!(game, Draygon[BODY].base.x_pos).wrapping_add(dx);
        let y = session_slot!(game, Draygon[BODY].base.y_pos).wrapping_sub(FOAM_Y);
        game.foreign(|game| game.create_sprite(x, y, FOAM, 0));
    }
}
/// Handle firing wall turret ($A5:87AA): every 40h frames, a random turret of
/// the last four fires unless destroyed. Returns the turret index it then
/// leaves in X, which the breath bubbles read after it. Was `turret`.
pub fn fire_wall_turret<S: BossState>(game: &mut S) -> Option<u16> {
    if game.frame() & TURRET_FRAMES != 0 {
        return None;
    }
    let index = 2 * ((game.roll_random() & 3) + 2);
    if session_slot!(game, Draygon[index as i32].turret_destroyed) == 0 {
        let x = crate::actor_tables::word(game, |t| &t.draygon_turret_positions, index as isize);
        let y =
            crate::actor_tables::word(game, |t| &t.draygon_turret_positions, index as isize + 1);
        game.place_projectile_spawn(x, y);
        game.spawn_actor_eproj(TURRET_SPEED, index, TURRET_PROJECTILE);
    }
    Some(index)
}
/// Draygon body function - fight intro: the initial delay ($A5:871B, with
/// `first`) or the dance ($A5:878B). Was `opening`.
pub fn fight_intro<L: Lease>(first: bool, game: &mut L) {
    fire_wall_turret(game);
    let limit = if first { INTRO_DELAY } else { INTRO_DANCE };
    if session_slot!(game, Draygon[BODY].function_timer).wrapping_sub(limit) & SIGN != 0 {
        if first {
            if session_slot!(game, Draygon[BODY].function_timer) == 0 {
                let [bytes, address, bank, vram_address] = INTRO_TRANSFER;
                let source = Source {
                    address,
                    bank: bank as u8,
                };
                queue_tilemap(
                    game,
                    TilemapUpload {
                        bytes,
                        source,
                        column: false,
                        vram_address,
                    },
                );
                game.foreign(|game| game.draygon_spawn_trails());
            }
        } else {
            game.foreign(|game| game.draygon_trajectory());
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
/// Initialisation AI - Draygon body ($A5:8687). Actor preparation and
/// completion surround the existing synchronous HDMA install seam.
pub fn initialize<L: Lease>(install_layers: impl FnOnce(&mut L), game: &mut L) {
    crate::word_transfers!(game, descending, 25; le(TARGET_PALETTES, 168) <- rom(TARGET_PALETTES, 24));
    bg2_staging::fill(game, (0..=2047).rev(), 824);
    session_slot!(
        game,
        Draygon[game.current_enemy() as i32].base.palette_index = PALETTE_7
    );
    // The native bank word also clears byte178F before the address word is stored.
    crate::word_transfers!(game, ascending, 1;
        native(GRAPHICS_DRAWN_HOOK_BANK) <- value(u16::from(BANK)),
        native(GRAPHICS_DRAWN_HOOK) <- value(SET_BG2_SCROLL_HOOK));
    boss!(game.FlagProcessAllEnemies = 0xffff);
    for (k, instruction) in [
        (BODY, list::BODY_LEFT_IDLE),
        (EYE, list::EYE_LEFT_IDLE),
        (TAIL, list::TAIL_LEFT_FAKE_TAIL_WHIP),
        (ARMS, list::ARMS_LEFT_NEAR_SWOOP_APEX),
    ] {
        session_slot!(game, Draygon[k].base.current_instruction = instruction);
    }
    boss!(game.EnemyTilemapSize = BG2_TILEMAP_SIZE);
    session_slot!(
        game,
        Draygon[game.current_enemy() as i32].function = function::FIGHT_INTRO_INITIAL_DELAY
    );
    install_layers(game);
    session_slot!(game, Draygon[BODY].turret_5_destroyed = 1);
    game.foreign(|game| game.disable_minimap_for_boss());
}
/// Initialisation AI - Draygon tail ($A5:C599) or, with `arms`, arms ($A5:C5AD).
/// Was `initialize_part`.
pub fn initialize_tail_or_arms<S: BossState>(arms: bool, game: &mut S) {
    let k = game.current_enemy() as i32;
    let instruction = if arms {
        list::ARMS_LEFT_IDLE
    } else {
        list::TAIL_LEFT_FAKE_TAIL_WHIP
    };
    session_slot!(game, Draygon[k].base; current_instruction = instruction, palette_index = PALETTE_7);
    if arms {
        session_slot!(game, Draygon[k].base.layer = 2);
    }
}
/// Main AI - Draygon body ($A5:86FC, with `body`) or eye ($A5:C486): run the
/// part's function, then the body carries the other parts with it. Was `update`.
pub fn main_ai<L: Lease>(body: bool, dispatch: impl FnOnce(u32, u16, &mut L::Game), game: &mut L) {
    if !body {
        return game.foreign(|game| {
            let k = game.current_enemy() as i32;
            let state =
                (u32::from(BANK) << 16) | u32::from(session_slot!(game, Draygon[k].function));
            dispatch(state, game.current_enemy(), game)
        });
    }
    // The body's function runs on the lease, the effects that need the game
    // in foreign calls.
    let k = game.current_enemy();
    let function = session_slot!(game, Draygon[k as i32].function);
    run_function_on(game, function, k, |game| {
        dispatch(u32::from(BANK) << 16 | u32::from(function), k, game)
    });
    let x = session_slot!(game, Draygon[BODY].base.x_pos);
    for k in [EYE, TAIL, ARMS] {
        session_slot!(game, Draygon[k].base.x_pos = x);
    }
    let y = session_slot!(game, Draygon[BODY].base.y_pos);
    for k in [EYE, TAIL, ARMS] {
        session_slot!(game, Draygon[k].base.y_pos = y);
    }
}

/// Run the bank $A5 body function `function` for part `k`, as the species
/// table's dispatch does: the flight, swoop and grab functions on this lease, the rest
/// through `fallback` on the game.
fn run_function_on<L: Lease>(
    game: &mut L,
    function: u16,
    k: u16,
    fallback: impl FnOnce(&mut L::Game),
) {
    use crate::actors::{draygon_grab as grab, draygon_sweep as sweep};
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
        function::GOOP_RIGHT_SETUP => goop_setup(true, game),
        function::GOOP_LEFT_SETUP => goop_setup(false, game),
        function::GOOP_RIGHT_MOVE_UNTIL_IN_RANGE => goop_move_until_in_range(true, game),
        function::GOOP_LEFT_MOVE_UNTIL_IN_RANGE => goop_move_until_in_range(false, game),
        function::GOOP_RIGHT_FIRING => goop_firing(true, game),
        function::GOOP_LEFT_FIRING => goop_firing(false, game),
        function::GOOP_RIGHT_MOVE_UNTIL_OFF_SCREEN => goop_move_until_off_screen(true, game),
        function::GOOP_LEFT_MOVE_UNTIL_OFF_SCREEN => goop_move_until_off_screen(false, game),
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
        _ => game.foreign(fallback),
    }
}
/// Enemy graphics drawn hook - Draygon - set BG2 X/Y scroll ($A5:9342). Was
/// `scroll_layer`.
pub fn set_bg2_scroll<S: BossState>(game: &mut S) {
    let x = i32::from(session_slot!(game, Draygon[BODY].graphics_x_displacement))
        + i32::from(game.camera().layer1[0])
        - i32::from(session_slot!(game, Draygon[BODY].base.x_pos))
        - BG2_X_OFFSET;
    display_scroll!(game.Bg2H = x as u16);
    let y = i32::from(session_slot!(game, Draygon[BODY].graphics_y_displacement))
        + i32::from(game.camera().layer1[1])
        - i32::from(session_slot!(game, Draygon[BODY].base.y_pos))
        - BG2_Y_OFFSET;
    display_scroll!(game.Bg2V = y as u16);
}
/// Oscillate around the goop height and move on.
fn goop_glide<S: BossState>(right: bool, game: &mut S) {
    let angle = session_slot!(game, Draygon[BODY].goop_y_angle);
    session_slot!(
        game,
        Draygon[BODY].base.y_pos = game
            .scaled_cosine(angle, GOOP_Y_AMPLITUDE)
            .wrapping_add(GOOP_Y)
    );
    session_slot!(
        game,
        Draygon[BODY].goop_y_angle,
        |old| (old as u8).wrapping_add(1) as u16
    );
    move_horizontally(right, game);
}
/// Move the body right, or left, by its X speed. Was `displace`.
pub fn move_horizontally<S: BossState>(right: bool, game: &mut S) {
    let high = session_slot!(game, Draygon[BODY].x_speed);
    let low = session_slot!(game, Draygon[BODY].x_subspeed);
    let speed = (u32::from(high) << 16) | u32::from(low);
    let delta = if right {
        speed
    } else {
        (-(speed as i32)) as u32
    };
    session_slot!(game, Draygon[BODY].base(x_pos, x_subpos) += delta);
}
fn beyond_right(x: u16) -> bool {
    x as i16 >= 0 && x.wrapping_sub(OFF_SCREEN_RIGHT) & SIGN == 0
}
/// Draygon body function - goop right ($A5:8B0A) or left ($A5:8C8E) - setup.
/// Was `begin`.
pub fn goop_setup<S: BossState>(right: bool, game: &mut S) {
    if right {
        fire_wall_turret(game);
    }
    session_slot!(
        game,
        Draygon[BODY].base.x_pos = if right {
            GOOP_START_X
        } else {
            session_slot!(game, Draygon[BODY].right_reset_x)
        }
    );
    session_slot!(game, Draygon[BODY].base.y_pos = GOOP_Y);
    session_slot!(game, Draygon[BODY]; x_subspeed = 0, x_speed = 1, goop_y_angle = 0);
    session_slot!(game, Draygon[ARMS].base; current_instruction = list::ARMS_RIGHT_NEAR_SWOOP_APEX, instruction_timer = 1);
    session_slot!(
        game,
        Draygon[BODY].function = if right {
            function::GOOP_RIGHT_MOVE_UNTIL_IN_RANGE
        } else {
            function::GOOP_LEFT_MOVE_UNTIL_IN_RANGE
        }
    );
    session_slot!(game, Draygon[BODY].base; current_instruction = if right { list::BODY_RIGHT_RESET } else { list::BODY_LEFT_RESET }, instruction_timer = 1);
    session_slot!(game, Draygon[BODY].facing_right = u16::from(right));
}
/// Every 128 frames of the current part, a bubble (sprite object 18h) 10h px
/// left of and above the body. The original reads the frame counter at X, so
/// after a turret pick it reads the body's words at that index ($A5:931C).
pub fn body_bubble<L: Lease>(turret: Option<u16>, game: &mut L) {
    let k = i32::from(turret.unwrap_or_else(|| game.current_enemy()));
    if session_slot!(game, Draygon[k].base.frame_counter) & 0x7f == 0 {
        let x = session_slot!(game, Draygon[BODY].base.x_pos).wrapping_sub(BUBBLE_OFFSET);
        let y = session_slot!(game, Draygon[BODY].base.y_pos).wrapping_sub(BUBBLE_OFFSET);
        game.foreign(|game| game.create_sprite(x, y, BUBBLE, 0));
    }
}
/// Draygon body function - goop right ($A5:8B52) or left ($A5:8CD4) - move
/// until Samus in range. Was `approach`.
pub fn goop_move_until_in_range<L: Lease>(right: bool, game: &mut L) {
    let turret = fire_wall_turret(game);
    body_bubble(turret, game);
    let distance =
        abs16(session_slot!(game, Draygon[BODY].base.x_pos).wrapping_sub(game.player_x()));
    if distance.wrapping_sub(GOOP_RANGE) & SIGN != 0 {
        session_slot!(game, Draygon[BODY]; function = if right { function::GOOP_RIGHT_FIRING } else { function::GOOP_LEFT_FIRING }, goop_counter = GOOPS);
    } else {
        goop_glide(right, game);
    }
}
/// Draygon body function - goop right ($A5:8BAE) or left ($A5:8D30) - firing
/// goops. Was `fire`.
pub fn goop_firing<S: BossState>(right: bool, game: &mut S) {
    if right {
        fire_wall_turret(game);
    }
    if boss!(game.SamusXSpeedDivisor) == 0 {
        let mut leave = false;
        if boss!(game.Random) & GOOP_CHANCE == 0 {
            let remaining = session_slot!(game, Draygon[BODY].goop_counter).wrapping_sub(1);
            session_slot!(game, Draygon[BODY].goop_counter = remaining);
            leave = remaining == 0;
            if !leave {
                session_slot!(game, Draygon[BODY].base; current_instruction = if right { list::BODY_RIGHT_FIRE_GOOP } else { list::BODY_LEFT_FIRE_GOOP }, instruction_timer = 1);
            }
        }
        if !leave {
            goop_glide(right, game);
            // The left pass tests unsigned x_pos < 0, so its left-edge test never
            // succeeds. The left exit below intentionally uses the signed comparison.
            let x = session_slot!(game, Draygon[BODY].base.x_pos);
            if !right || !beyond_right(x) {
                return;
            }
        }
        if boss!(game.SamusXSpeedDivisor) == 0 {
            session_slot!(game, Draygon[ARMS].base; current_instruction = if right { list::ARMS_RIGHT_IDLE } else { list::ARMS_LEFT_IDLE }, instruction_timer = 1);
            session_slot!(
                game,
                Draygon[BODY].function = if right {
                    function::GOOP_RIGHT_MOVE_UNTIL_OFF_SCREEN
                } else {
                    function::GOOP_LEFT_MOVE_UNTIL_OFF_SCREEN
                }
            );
            return;
        }
    }
    session_slot!(game, Draygon[BODY].function = function::CHASE_SAMUS);
}
/// Draygon body function - goop right ($A5:8C33) or left ($A5:8DB2) - move
/// until off-screen. Was `exit`.
pub fn goop_move_until_off_screen<L: Lease>(right: bool, game: &mut L) {
    body_bubble(None, game);
    if !right && boss!(game.SamusXSpeedDivisor) != 0 {
        session_slot!(game, Draygon[BODY].function = function::CHASE_SAMUS);
        return;
    }
    goop_glide(right, game);
    let x = session_slot!(game, Draygon[BODY].base.x_pos);
    let finished = if right {
        beyond_right(x)
    } else {
        (x as i16) < 0 && x.wrapping_add(OFF_SCREEN_LEFT) & SIGN != 0
    };
    if finished {
        session_slot!(
            game,
            Draygon[BODY].function = if right {
                function::SWOOP_LEFT_SETUP
            } else {
                function::SWOOP_RIGHT_SETUP
            }
        );
        session_slot!(game, Draygon[BODY].base; x_pos = if right { RIGHT_RESET_X } else { GOOP_START_X }, y_pos = RESET_Y);
        let copied = session_slot!(game, Draygon[BODY].swoop_duration);
        session_slot!(game, Draygon[BODY].function_timer = copied);
    }
}

/// Instruction - Draygon instructions lists = [[Y]], [[Y] + 2], [[Y] + 4],
/// [[Y] + 6] ($A5:94DD). Was `pose`.
pub fn set_instruction_lists<S: BossState>(script: EnemyScript, game: &mut S) -> EnemyScript {
    for (k, offset) in [(BODY, 0), (EYE, 2), (TAIL, 4), (ARMS, 6)] {
        session_slot!(
            game,
            Draygon[k].base.current_instruction = script.word(offset)
        );
    }
    for k in [BODY, EYE, TAIL, ARMS] {
        session_slot!(game, Draygon[k].base.instruction_timer = 1);
    }
    script.offset(4)
}
/// Instruction - displace Draygon body graphics [[Y]] px left, [[Y] + 2] px
/// down ($A5:9E0A). Was `layer_offsets`.
pub fn displace_graphics<S: BossState>(script: EnemyScript, game: &mut S) -> EnemyScript {
    session_slot!(game, Draygon[BODY]; graphics_x_displacement = script.word(0),
        graphics_y_displacement = script.word(2));
    script.offset(2)
}
/// Instruction - paralyse Draygon tail and arms ($A5:98D3). Was `hide_parts`.
pub fn paralyse_tail_and_arms<S: BossState>(script: EnemyScript, game: &mut S) -> EnemyScript {
    for k in [TAIL, ARMS] {
        session_slot!(game, Draygon[k].base; instruction_timer = 1, current_instruction = list::SLEEP);
    }
    script
}
/// Instruction - set Draygon body as intangible ($A5:98EF). Was `tangible`.
pub fn set_intangible<S: BossState>(script: EnemyScript, game: &mut S) -> EnemyScript {
    session_slot!(game, Draygon[BODY].base.properties, |old| old | INTANGIBLE);
    script
}
