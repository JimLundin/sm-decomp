//! Player projectile definitions, instruction playback and drawing.
use crate::game::samus_animation::Samus_DrawShinesparkCrashEchoProjectiles;
use crate::game::sm_80::{InvalidInterrupt_Crash, QueueSfx2_Max6};
use crate::game::sm_81::DrawProjectileSpritemap;
use crate::game::sm_8b::CalcExplosion_Mode7;
use crate::game::sm_90::ClearProjectile;
use crate::game_state::Game;
use crate::ram_map::{checked_word as word, word_arrays};
use crate::rom::RomRecord;
use crate::rtl::Unreachable;

const record: unsafe fn(u16, &mut Game) -> RomRecord = crate::rtl::rom_bank_record::<0x93>;

#[forbid(unsafe_code)]
fn table(address: u32, index: isize, game: &Game) -> u16 {
    crate::rtl::rom_table_word!(game, address, index)
}

#[forbid(unsafe_code)]
fn aimed_program(data: &RomRecord, direction: usize) -> u16 {
    // Preserve the ten-entry field's bounds check before reading the selected word.
    data.word([2, 4, 6, 8, 10, 12, 14, 16, 18, 20][direction])
}

unsafe fn initialize_fixed(k: u16, address: u32, type_shift: u32, game: &mut Game) {
    let slot = usize::from(k >> 1);
    let selector = (word!(game.projectile_type[slot]) >> type_shift) & 0xf;
    let data = record(table(address, selector as isize, game), game);
    let damage = data.word(0);
    word!(game.projectile_damage[slot] = damage);
    if damage & 0x8000 != 0 {
        InvalidInterrupt_Crash(game);
    }
    word!(game.projectile_bomb_instruction_ptr[slot] = data.word(2));
    word!(game.projectile_bomb_instruction_timers[slot] = 1);
}
pub unsafe fn initialize_super_missile(k: u16, game: &mut Game) {
    initialize_fixed(k, 0x93842b, 8, game);
}
pub unsafe fn initialize_bomb(k: u16, game: &mut Game) {
    initialize_fixed(k, 0x9383f1, 8, game);
}
pub unsafe fn initialize_sba(k: u16, game: &mut Game) {
    initialize_fixed(k, 0x938413, 0, game);
}

pub unsafe fn initialize_projectile(k: u16, game: &mut Game) {
    let slot = usize::from(k >> 1);
    let direction = usize::from(word!(game.projectile_dir[slot]) & 0xf);
    let kind = word!(game.projectile_type[slot]);
    let (address, selector) = if kind & 0xf00 != 0 {
        (0x9383f1, isize::from(kind.to_ne_bytes()[1] & 0xf))
    } else {
        (
            if kind & 0x10 != 0 { 0x9383d9 } else { 0x9383c1 },
            (word!(game.projectile_type[slot]) & 0xf) as isize,
        )
    };
    let data = record(table(address, selector, game), game);
    if data.word(0) & 0x8000 != 0 {
        InvalidInterrupt_Crash(game);
    }
    // Diagnostics can mutate the captured allocation; retain the live reread.
    word!(game.projectile_damage[slot] = data.word(0));
    let program = aimed_program(&data, direction);
    word!(game.projectile_bomb_instruction_ptr[slot] = program);
    let frame = record(program, game);
    word!(game.projectile_x_radius[slot] = u16::from(frame.byte(4)));
    word!(game.projectile_y_radius[slot] = u16::from(frame.byte(5)));
    word!(game.projectile_bomb_instruction_timers[slot] = 1);
}

pub unsafe fn initialize_echo(k: u16, game: &mut Game) {
    let slot = usize::from(k >> 1);
    let direction = usize::from(word!(game.projectile_dir[slot]) & 0xf);
    let selector = isize::from(game.g_ram[word_arrays::projectile_type + 2 * slot]) - 34;
    let data = record(table(0x938403, selector, game), game);
    word!(game.projectile_damage[slot] = data.word(0));
    if data.word(0) & 0x8000 != 0 {
        InvalidInterrupt_Crash(game);
    }
    word!(game.projectile_bomb_instruction_ptr[slot] = aimed_program(&data, direction));
    word!(game.projectile_bomb_instruction_timers[slot] = 1);
}

pub unsafe fn start_impact(k: u16, game: &mut Game) {
    let slot = usize::from(k >> 1);
    if word!(game.projectile_type[slot]) & 0xf00 != 0 {
        if word!(game.cinematic_function) == 0 {
            QueueSfx2_Max6(7, game);
        }
        let kind = word!(game.projectile_type[slot]);
        word!(game.projectile_type[slot] = kind & 0xf0ff | 0x800);
        if kind & 0x200 != 0 {
            word!(game.projectile_bomb_instruction_ptr[slot] = table(0x938693, 0, game));
            word!(game.earthquake_type = 20);
            word!(game.earthquake_timer = 30);
        } else {
            word!(game.projectile_bomb_instruction_ptr[slot] = table(0x93867f, 0, game));
        }
        if word!(game.cooldown_timer) as i32 - 21 & 0x8000 == 0 {
            word!(game.cooldown_timer = 20);
        }
    } else {
        word!(game.projectile_type[slot] = word!(game.projectile_type[slot]) & 0xf0ff | 0x700);
        word!(game.projectile_bomb_instruction_ptr[slot] = table(0x93867b, 0, game));
        QueueSfx2_Max6(0xc, game);
    }
    word!(game.projectile_bomb_instruction_timers[slot] = 1);
    word!(game.projectile_damage[slot] = 8);
}

#[forbid(unsafe_code)]
pub fn start_bomb_explosion(k: u16, game: &mut Game) {
    let slot = usize::from(k >> 1);
    word!(game.projectile_bomb_instruction_ptr[slot] = table(0x938683, 0, game));
    word!(game.projectile_bomb_instruction_timers[slot] = 1);
}

pub unsafe fn trail_frame(k: u16, game: &mut Game) -> u16 {
    let slot = usize::from(k >> 1);
    let ip = word!(game.projectile_bomb_instruction_ptr[slot]);
    let delta = if word!(game.projectile_bomb_instruction_timers[slot]) == 1
        && record(ip, game).word(0) & 0x8000 == 0
    {
        0
    } else {
        8
    };
    record(ip.wrapping_sub(delta), game).word(6)
}

pub unsafe fn instruction_delete(k: u16, _: u16, game: &mut Game) -> u16 {
    ClearProjectile(k, game);
    0
}
pub unsafe fn instruction_goto(_: u16, j: u16, game: &mut Game) -> u16 {
    record(j, game).word(0)
}
pub unsafe fn dispatch_instruction(ea: u32, j: u16, k: u16, game: &mut Game) -> u16 {
    match ea {
        0x93822f => instruction_delete(j, k, game),
        0x938239 => instruction_goto(j, k, game),
        _ => Unreachable(game) as u16,
    }
}

pub unsafe fn advance(game: &mut Game) {
    let selected = word!(game.projectile_index);
    let slot = usize::from(selected >> 1);
    let previous = word!(game.projectile_bomb_instruction_timers[slot]);
    word!(game.projectile_bomb_instruction_timers[slot] = previous.wrapping_sub(1));
    if previous != 1 {
        return;
    }
    let mut cursor = word!(game.projectile_bomb_instruction_ptr[slot]);
    loop {
        let frame = record(cursor, game);
        if frame.word(0) & 0x8000 == 0 {
            word!(game.projectile_bomb_instruction_timers[slot] = frame.word(0));
            word!(game.projectile_spritemap_pointers[slot] = frame.word(2));
            word!(game.projectile_x_radius[slot] = u16::from(frame.byte(4)));
            word!(game.projectile_y_radius[slot] = u16::from(frame.byte(5)));
            word!(game.projectile_bomb_instruction_ptr[slot] = cursor.wrapping_add(8));
            return;
        }
        cursor = dispatch_instruction(
            0x930000 | u32::from(frame.word(0)),
            selected,
            cursor.wrapping_add(2),
            game,
        );
        if cursor == 0 {
            return;
        }
    }
}

unsafe fn draw_slots<const EXPLOSIONS: bool>(game: &mut Game) {
    let index = if EXPLOSIONS { 18 } else { 8 };
    word!(game.projectile_index = index);
    continue_drawing::<EXPLOSIONS>(index, 0, 0, game);
}

pub(crate) unsafe fn continue_drawing<const EXPLOSIONS: bool>(
    mut index: u16,
    mut x: u16,
    mut y: u16,
    game: &mut Game,
) {
    loop {
        let slot = (index >> 1) as usize;
        if word!(game.projectile_bomb_instruction_ptr[slot]) != 0 {
            let kind = word!(game.projectile_type[slot]);
            let visible = if EXPLOSIONS {
                kind & 0xf00 >= 0x300
                    && (kind & 0xf00 != 0x300 || word!(game.projectile_variables[slot]) != 0)
            } else if kind & 0xf10 != 0 {
                kind & 0xf00 < 0x300
            } else if kind & 0xc != 0 {
                (index & 2 != 0) == (word!(game.nmi_frame_counter_word) & 2 != 0)
            } else {
                (index & 2 == 0) == (word!(game.nmi_frame_counter_word) & 1 != 0)
            };
            if visible {
                'draw: {
                    let mode7 = word!(game.ceres_status) & 0x8000 != 0
                        && (!EXPLOSIONS || (kind & 0xf00 != 0x300 && kind & 0xf00 != 0x500));
                    let clip_y = if mode7 {
                        let point = CalcExplosion_Mode7(index, game);
                        // Ceres projectiles clip transformed Y but draw with the previous Y.
                        // Ceres explosions discard both transformed coordinates.
                        if EXPLOSIONS {
                            y
                        } else {
                            x = point.x;
                            point.y
                        }
                    } else {
                        x = word!(game.projectile_x_pos[slot])
                            .wrapping_sub(word!(game.layer1_x_pos));
                        if EXPLOSIONS
                            && (x as i32 - 304 & 0x8000 == 0 || x as i32 + 48 & 0x8000 != 0)
                        {
                            break 'draw;
                        }
                        y = word!(game.projectile_y_pos[slot])
                            .wrapping_sub(word!(game.layer1_y_pos));
                        y
                    };
                    if clip_y & 0xff00 == 0
                        && (EXPLOSIONS
                            || word!(game.projectile_spritemap_pointers[slot]) & 0x8000 != 0)
                    {
                        DrawProjectileSpritemap(index, x, y, game);
                    }
                    index = word!(game.projectile_index);
                }
            }
        }
        index = index.wrapping_sub(2);
        word!(game.projectile_index = index);
        if index & 0x8000 != 0 {
            break;
        }
    }
}

pub unsafe fn draw_projectiles(game: &mut Game) {
    draw_slots::<false>(game);
    Samus_DrawShinesparkCrashEchoProjectiles(game);
    crate::game::projectile_trails::update(game);
}
pub unsafe fn draw_explosions(game: &mut Game) {
    draw_slots::<true>(game);
}
