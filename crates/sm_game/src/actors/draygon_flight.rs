//! Draygon's fight intro, wall turrets, setup, goop passes and the
//! instructions its body scripts call (bank $A5:8687-$A5:9E0A).
//! Extended motion words stay native compatibility state until the whole boss migrates.
#![forbid(unsafe_code)]
use crate::actors::draygon_host::{
    ARMS, BANK, BODY, Draygon, EYE, Host, INTANGIBLE, SIGN, TAIL, abs16, function,
    instruction_list as list,
};
use crate::enemy_script::EnemyScript;
use crate::ram::RawRam as _;
use crate::ram::{slot, word};

/// Sprite object 3Dh: Draygon foaming at the mouth, 20h px ahead of and 10h
/// px above the body.
const FOAM: u16 = 0x3d;
const FOAM_X: u16 = 0x20;
const FOAM_Y: u16 = 0x10;
/// $A5:87DC: each wall turret's projectile spawn position; the turrets fire
/// Draygon's wall turret projectile ($86:8E5E) at 3 px a frame, every 40h
/// frames.
const TURRET_POSITIONS: u32 = 0xa5_87dc;
const TURRET_PROJECTILE: u16 = 0x8e5e;
const TURRET_SPEED: u16 = 3;
const TURRET_FRAMES: u16 = 0x3f;
/// The fight intro waits 100h frames, then dances for 4D0h.
const INTRO_DELAY: u16 = 0x100;
const INTRO_DANCE: u16 = 0x4d0;
/// The intro's VRAM write: 600h bytes from $B1:9400 to VRAM $6D00.
const INTRO_TRANSFER: [u16; 4] = [0x600, 0x9400, 0xb1, 0x6d00];
const VRAM_WRITE_QUEUE: usize = 0xd0;
const QUEUE_ENTRY_BYTES: u16 = 7;
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
pub fn facing_animation<H: Host + ?Sized>(
    facing_key: i32,
    part_key: i32,
    instructions: [u16; 2],
    game: &mut H,
) {
    let instruction = instructions[usize::from(slot!(game, Draygon[facing_key].facing_right) != 0)];
    slot!(game, Draygon[part_key].base; current_instruction = instruction, instruction_timer = 1);
}
/// On frames matching `frame_mask`, Draygon foams at the mouth. Was
/// `shed_particle`.
pub fn foam_at_mouth<H: Host + ?Sized>(k: u16, frame_mask: u16, game: &mut H) {
    if slot!(game, Draygon[k as i32].base.frame_counter) & frame_mask == 0 {
        let dx = if slot!(game, Draygon[BODY].facing_right) != 0 {
            FOAM_X
        } else {
            0u16.wrapping_sub(FOAM_X)
        };
        let x = slot!(game, Draygon[BODY].base.x_pos).wrapping_add(dx);
        let y = slot!(game, Draygon[BODY].base.y_pos).wrapping_sub(FOAM_Y);
        game.create_sprite(x, y, FOAM, 0);
    }
}
/// Handle firing wall turret ($A5:87AA): every 40h frames, a random turret of
/// the last four fires unless destroyed. Was `turret`.
pub fn fire_wall_turret<H: Host + ?Sized>(game: &mut H) {
    if word!(game.nmi_frame_counter_word) & TURRET_FRAMES == 0 {
        let index = 2 * ((game.next_random() & 3) + 2);
        if slot!(game, Draygon[index as i32].turret_destroyed) == 0 {
            let x = crate::cartridge::rom_table_word!(game, TURRET_POSITIONS, index as isize);
            let y = crate::cartridge::rom_table_word!(game, TURRET_POSITIONS, index as isize + 1);
            game.set_projectile_spawn_point(x, y);
            game.spawn_projectile(TURRET_SPEED, index, TURRET_PROJECTILE);
        }
    }
}
/// Draygon body function - fight intro: the initial delay ($A5:871B, with
/// `first`) or the dance ($A5:878B). Was `opening`.
pub fn fight_intro<H: Host + ?Sized>(first: bool, game: &mut H) {
    fire_wall_turret(game);
    let limit = if first { INTRO_DELAY } else { INTRO_DANCE };
    if slot!(game, Draygon[BODY].function_timer).wrapping_sub(limit) & SIGN != 0 {
        if first {
            if slot!(game, Draygon[BODY].function_timer) == 0 {
                let at = VRAM_WRITE_QUEUE + usize::from(word!(game.vram_write_queue_tail));
                let [size, source, bank, destination] = INTRO_TRANSFER;
                // The native bank word deliberately overlaps the destination's low byte.
                game.write_native_words(&[
                    (at, size),
                    (at + 2, source),
                    (at + 4, bank),
                    (at + 5, destination),
                ]);
                word!(game.vram_write_queue_tail += QUEUE_ENTRY_BYTES);
                game.draygon_spawn_trails();
            }
        } else {
            game.draygon_trajectory();
        }
        slot!(game, Draygon[BODY].function_timer, |old| old
            .wrapping_add(1));
    } else {
        slot!(
            game,
            Draygon[BODY].function = if first {
                function::FIGHT_INTRO_DANCE
            } else {
                function::SWOOP_RIGHT_SETUP
            }
        );
        slot!(game, Draygon[BODY].function_timer = 0);
        if first {
            let x = slot!(game, Draygon[BODY].base.x_pos);
            slot!(game, Draygon[BODY].left_reset_x = x);
            slot!(
                game,
                Draygon[BODY].right_reset_x = x.wrapping_add(RIGHT_RESET_OFFSET)
            );
            slot!(
                game,
                Draygon[BODY].reset_y = slot!(game, Draygon[BODY].base.y_pos)
            );
            slot!(
                game,
                Draygon[BODY].base.x_pos = slot!(game, Draygon[BODY].left_reset_x)
            );
            slot!(
                game,
                Draygon[BODY].base.y_pos = slot!(game, Draygon[BODY].reset_y)
            );
            slot!(
                game,
                Draygon[BODY].swoop_y_acceleration = INITIAL_SWOOP_Y_ACCELERATION
            );
        }
    }
}
/// Initialisation AI - Draygon body ($A5:8687). Actor preparation and
/// completion surround the existing synchronous HDMA install seam.
pub fn initialize<H: Host + ?Sized>(install_layers: impl FnOnce(&mut H), game: &mut H) {
    crate::word_transfers!(game, descending, 25; le(target_palettes, 168) <- rom(TARGET_PALETTES, 24));
    crate::word_transfers!(game, descending, 2048; le(tilemap_stuff, 2047) <- value(824));
    slot!(
        game,
        Draygon[word!(game.cur_enemy_index) as i32]
            .base
            .palette_index = PALETTE_7
    );
    // The native bank word also clears byte178F before the address word is stored.
    crate::word_transfers!(game, ascending, 1;
        native(GRAPHICS_DRAWN_HOOK_BANK) <- value(u16::from(BANK)),
        native(GRAPHICS_DRAWN_HOOK) <- value(SET_BG2_SCROLL_HOOK));
    word!(game.flag_process_all_enemies = 0xffff);
    for (k, instruction) in [
        (BODY, list::BODY_LEFT_IDLE),
        (EYE, list::EYE_LEFT_IDLE),
        (TAIL, list::TAIL_LEFT_FAKE_TAIL_WHIP),
        (ARMS, list::ARMS_LEFT_NEAR_SWOOP_APEX),
    ] {
        slot!(game, Draygon[k].base.current_instruction = instruction);
    }
    word!(game.enemy_bg2_tilemap_size = BG2_TILEMAP_SIZE);
    slot!(
        game,
        Draygon[word!(game.cur_enemy_index) as i32].function = function::FIGHT_INTRO_INITIAL_DELAY
    );
    install_layers(game);
    slot!(game, Draygon[BODY].turret_5_destroyed = 1);
    game.disable_minimap_for_boss();
}
/// Initialisation AI - Draygon tail ($A5:C599) or, with `arms`, arms ($A5:C5AD).
/// Was `initialize_part`.
pub fn initialize_tail_or_arms<H: Host + ?Sized>(arms: bool, game: &mut H) {
    let k = word!(game.cur_enemy_index) as i32;
    let instruction = if arms {
        list::ARMS_LEFT_IDLE
    } else {
        list::TAIL_LEFT_FAKE_TAIL_WHIP
    };
    slot!(game, Draygon[k].base; current_instruction = instruction, palette_index = PALETTE_7);
    if arms {
        slot!(game, Draygon[k].base.layer = 2);
    }
}
/// Main AI - Draygon body ($A5:86FC, with `body`) or eye ($A5:C486): run the
/// part's function, then the body carries the other parts with it. Was `update`.
pub fn main_ai<H: Host + ?Sized>(
    body: bool,
    dispatch: impl FnOnce(u32, u16, &mut H),
    game: &mut H,
) {
    let k = word!(game.cur_enemy_index) as i32;
    let state = u32::from(BANK) << 16 | u32::from(slot!(game, Draygon[k].function));
    dispatch(state, word!(game.cur_enemy_index), game);
    if !body {
        return;
    }
    let x = slot!(game, Draygon[BODY].base.x_pos);
    for k in [EYE, TAIL, ARMS] {
        slot!(game, Draygon[k].base.x_pos = x);
    }
    let y = slot!(game, Draygon[BODY].base.y_pos);
    for k in [EYE, TAIL, ARMS] {
        slot!(game, Draygon[k].base.y_pos = y);
    }
}
/// Enemy graphics drawn hook - Draygon - set BG2 X/Y scroll ($A5:9342). Was
/// `scroll_layer`.
pub fn set_bg2_scroll<H: Host + ?Sized>(game: &mut H) {
    let x = i32::from(slot!(game, Draygon[BODY].graphics_x_displacement))
        + i32::from(word!(game.layer1_x_pos))
        - i32::from(slot!(game, Draygon[BODY].base.x_pos))
        - BG2_X_OFFSET;
    word!(game.reg_BG2HOFS = x as u16);
    let y = i32::from(slot!(game, Draygon[BODY].graphics_y_displacement))
        + i32::from(word!(game.layer1_y_pos))
        - i32::from(slot!(game, Draygon[BODY].base.y_pos))
        - BG2_Y_OFFSET;
    word!(game.reg_BG2VOFS = y as u16);
}
/// Oscillate around the goop height and move on.
fn goop_glide<H: Host + ?Sized>(right: bool, game: &mut H) {
    let angle = slot!(game, Draygon[BODY].goop_y_angle);
    slot!(
        game,
        Draygon[BODY].base.y_pos = game
            .actor_cosine(angle, GOOP_Y_AMPLITUDE)
            .wrapping_add(GOOP_Y)
    );
    slot!(
        game,
        Draygon[BODY].goop_y_angle,
        |old| (old as u8).wrapping_add(1) as u16
    );
    move_horizontally(right, game);
}
/// Move the body right, or left, by its X speed. Was `displace`.
pub fn move_horizontally<H: Host + ?Sized>(right: bool, game: &mut H) {
    let high = slot!(game, Draygon[BODY].x_speed);
    let low = slot!(game, Draygon[BODY].x_subspeed);
    let speed = (u32::from(high) << 16) | u32::from(low);
    let delta = if right {
        speed
    } else {
        (-(speed as i32)) as u32
    };
    slot!(game, Draygon[BODY].base(x_pos, x_subpos) += delta);
}
fn beyond_right(x: u16) -> bool {
    x as i16 >= 0 && x.wrapping_sub(OFF_SCREEN_RIGHT) & SIGN == 0
}
/// Draygon body function - goop right ($A5:8B0A) or left ($A5:8C8E) - setup.
/// Was `begin`.
pub fn goop_setup<H: Host + ?Sized>(right: bool, game: &mut H) {
    if right {
        fire_wall_turret(game);
    }
    slot!(
        game,
        Draygon[BODY].base.x_pos = if right {
            GOOP_START_X
        } else {
            slot!(game, Draygon[BODY].right_reset_x)
        }
    );
    slot!(game, Draygon[BODY].base.y_pos = GOOP_Y);
    slot!(game, Draygon[BODY]; x_subspeed = 0, x_speed = 1, goop_y_angle = 0);
    slot!(game, Draygon[ARMS].base; current_instruction = list::ARMS_RIGHT_NEAR_SWOOP_APEX, instruction_timer = 1);
    slot!(
        game,
        Draygon[BODY].function = if right {
            function::GOOP_RIGHT_MOVE_UNTIL_IN_RANGE
        } else {
            function::GOOP_LEFT_MOVE_UNTIL_IN_RANGE
        }
    );
    slot!(game, Draygon[BODY].base; current_instruction = if right { list::BODY_RIGHT_RESET } else { list::BODY_LEFT_RESET }, instruction_timer = 1);
    slot!(game, Draygon[BODY].facing_right = u16::from(right));
}
/// Draygon body function - goop right ($A5:8B52) or left ($A5:8CD4) - move
/// until Samus in range. Was `approach`.
pub fn goop_move_until_in_range<H: Host + ?Sized>(right: bool, game: &mut H) {
    fire_wall_turret(game);
    game.draygon_body_bubble();
    let distance = abs16(slot!(game, Draygon[BODY].base.x_pos).wrapping_sub(game.player_x()));
    if distance.wrapping_sub(GOOP_RANGE) & SIGN != 0 {
        slot!(game, Draygon[BODY]; function = if right { function::GOOP_RIGHT_FIRING } else { function::GOOP_LEFT_FIRING }, goop_counter = GOOPS);
    } else {
        goop_glide(right, game);
    }
}
/// Draygon body function - goop right ($A5:8BAE) or left ($A5:8D30) - firing
/// goops. Was `fire`.
pub fn goop_firing<H: Host + ?Sized>(right: bool, game: &mut H) {
    if right {
        fire_wall_turret(game);
    }
    if word!(game.samus_x_speed_divisor) == 0 {
        let mut leave = false;
        if word!(game.random_number) & GOOP_CHANCE == 0 {
            let remaining = slot!(game, Draygon[BODY].goop_counter).wrapping_sub(1);
            slot!(game, Draygon[BODY].goop_counter = remaining);
            leave = remaining == 0;
            if !leave {
                slot!(game, Draygon[BODY].base; current_instruction = if right { list::BODY_RIGHT_FIRE_GOOP } else { list::BODY_LEFT_FIRE_GOOP }, instruction_timer = 1);
            }
        }
        if !leave {
            goop_glide(right, game);
            // The left pass tests unsigned x_pos < 0, so its left-edge test never
            // succeeds. The left exit below intentionally uses the signed comparison.
            let x = slot!(game, Draygon[BODY].base.x_pos);
            if !right || !beyond_right(x) {
                return;
            }
        }
        if word!(game.samus_x_speed_divisor) == 0 {
            slot!(game, Draygon[ARMS].base; current_instruction = if right { list::ARMS_RIGHT_IDLE } else { list::ARMS_LEFT_IDLE }, instruction_timer = 1);
            slot!(
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
    slot!(game, Draygon[BODY].function = function::CHASE_SAMUS);
}
/// Draygon body function - goop right ($A5:8C33) or left ($A5:8DB2) - move
/// until off-screen. Was `exit`.
pub fn goop_move_until_off_screen<H: Host + ?Sized>(right: bool, game: &mut H) {
    game.draygon_body_bubble();
    if !right && word!(game.samus_x_speed_divisor) != 0 {
        slot!(game, Draygon[BODY].function = function::CHASE_SAMUS);
        return;
    }
    goop_glide(right, game);
    let x = slot!(game, Draygon[BODY].base.x_pos);
    let finished = if right {
        beyond_right(x)
    } else {
        (x as i16) < 0 && x.wrapping_add(OFF_SCREEN_LEFT) & SIGN != 0
    };
    if finished {
        slot!(
            game,
            Draygon[BODY].function = if right {
                function::SWOOP_LEFT_SETUP
            } else {
                function::SWOOP_RIGHT_SETUP
            }
        );
        slot!(game, Draygon[BODY].base; x_pos = if right { RIGHT_RESET_X } else { GOOP_START_X }, y_pos = RESET_Y);
        slot!(
            game,
            Draygon[BODY].function_timer = slot!(game, Draygon[BODY].swoop_duration)
        );
    }
}

/// Instruction - Draygon instructions lists = [[Y]], [[Y] + 2], [[Y] + 4],
/// [[Y] + 6] ($A5:94DD). Was `pose`.
pub fn set_instruction_lists<H: Host + ?Sized>(script: EnemyScript, game: &mut H) -> EnemyScript {
    for (k, offset) in [(BODY, 0), (EYE, 2), (TAIL, 4), (ARMS, 6)] {
        slot!(
            game,
            Draygon[k].base.current_instruction = script.word(offset)
        );
    }
    for k in [BODY, EYE, TAIL, ARMS] {
        slot!(game, Draygon[k].base.instruction_timer = 1);
    }
    script.offset(4)
}
/// Instruction - displace Draygon body graphics [[Y]] px left, [[Y] + 2] px
/// down ($A5:9E0A). Was `layer_offsets`.
pub fn displace_graphics<H: Host + ?Sized>(script: EnemyScript, game: &mut H) -> EnemyScript {
    slot!(game, Draygon[BODY].graphics_x_displacement = script.word(0));
    slot!(game, Draygon[BODY].graphics_y_displacement = script.word(2));
    script.offset(2)
}
/// Instruction - paralyse Draygon tail and arms ($A5:98D3). Was `hide_parts`.
pub fn paralyse_tail_and_arms<H: Host + ?Sized>(script: EnemyScript, game: &mut H) -> EnemyScript {
    for k in [TAIL, ARMS] {
        slot!(game, Draygon[k].base; instruction_timer = 1, current_instruction = list::SLEEP);
    }
    script
}
/// Instruction - set Draygon body as intangible ($A5:98EF). Was `tangible`.
pub fn set_intangible<H: Host + ?Sized>(script: EnemyScript, game: &mut H) -> EnemyScript {
    slot!(game, Draygon[BODY].base.properties, |old| old | INTANGIBLE);
    script
}
