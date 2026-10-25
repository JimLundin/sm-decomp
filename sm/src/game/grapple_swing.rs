//! Grapple swing placement, rope-length changes and swept block contact.
use crate::game::prelude::*;
use crate::game::projectile_geometry::SINE_COSINE as kSinCosTable8bit_Sext;
use crate::game::projectile_room::Axis;
use crate::game::session_runtime::Runtime;
use crate::game::sm_80::Multiply16x16;

fn project_axis(origin: u16, sample: i16, length: u16) -> u16 {
    let distance = if sample == -256 || sample == 256 {
        length
    } else {
        (u16::from(length as u8) * u16::from(sample.unsigned_abs() as u8)) >> 8
    };
    if sample < 0 {
        origin.wrapping_sub(distance)
    } else {
        origin.wrapping_add(distance)
    }
}
unsafe fn place_sample(angle: u16, length: u16, game: &mut Game) {
    {
        let index = (angle >> 1) as usize;
        if word!(game.grapple_beam_flags) & 0x8000 == 0 {
            let offset = if kSinCosTable8bit_Sext[index + 64] < 0 {
                7
            } else {
                8
            };
            word!(
                game.grapple_beam_end_x_pos = word!(game.grapple_beam_end_x_pos) & 0xfff0 | offset
            );
            let offset = if kSinCosTable8bit_Sext[index] < 0 {
                7
            } else {
                8
            };
            word!(
                game.grapple_beam_end_y_pos = word!(game.grapple_beam_end_y_pos) & 0xfff0 | offset
            );
        }
        let x = project_axis(
            word!(game.grapple_beam_end_x_pos),
            kSinCosTable8bit_Sext[index + 64],
            length,
        );
        word!(game.grapple_beam_grapple_start_x = x);
        word!(game.grapple_beam_grapple_start_block_x = (x >> 4) as u8 as u16);
        let y = project_axis(
            word!(game.grapple_beam_end_y_pos),
            kSinCosTable8bit_Sext[index],
            length,
        );
        word!(game.grapple_beam_grapple_start_y = y);
        word!(game.grapple_beam_grapple_start_block_y = (y >> 4) as u8 as u16);
    }
}
unsafe fn contact(game: &mut Game) -> u8 {
    unsafe {
        let x = word!(game.grapple_beam_grapple_start_block_x);
        let row = Mult8x8(
            word!(game.grapple_beam_grapple_start_block_y) as u8,
            word!(game.room_width_in_blocks) as u8,
        );
        word!(game.cur_block_index = row.wrapping_add(x));
        game.react_to_block(|block_type, game| match block_type {
            0 | 3 | 4 | 6 | 7 => 0,
            1 | 8 | 9 | 11 | 12 | 14 | 15 => 1,
            2 => spike_contact(false, game),
            5 => crate::game::player_collision::grapple::redirect_block(
                &mut Runtime::new(game),
                Axis::X,
            ),
            10 => spike_contact(true, game),
            13 => crate::game::player_collision::grapple::redirect_block(
                &mut Runtime::new(game),
                Axis::Y,
            ),
            _ => unreachable!(),
        })
    }
}
static AIR_DAMAGE: [u16; 16] = [0, 0, 0x10, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0];
static SOLID_DAMAGE: [u16; 16] = [60, 16, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0];
unsafe fn spike_contact(solid: bool, game: &mut Game) -> u8 {
    unsafe {
        if word!(game.samus_invincibility_timer) == 0 {
            // The original reads a word spanning adjacent BTS bytes, not just one byte.
            let kind = *(ram_ptr!(game, u16; 0x16402, word!(game.cur_block_index) as isize)) as i16;
            if kind >= 0 {
                let damage = if solid {
                    SOLID_DAMAGE[kind as usize]
                } else {
                    AIR_DAMAGE[kind as usize]
                };
                let amount = ((damage as i16 as i32) << 16) as u32;
                if amount != 0 {
                    words!(
                        game.samus_periodic_damage,
                        samus_periodic_subdamage += amount
                    );
                    word! {game; samus_invincibility_timer = 60; samus_knockback_timer = 10; }
                }
            }
        }
        solid as u8
    }
}
unsafe fn check_clearance(angle: u16, game: &mut Game) -> u8 {
    unsafe {
        word!(game.g_word_7E0D98 = 6);
        let mut length = word!(game.grapple_beam_length).wrapping_add(8);
        loop {
            place_sample(angle, length, game);
            let blocked = contact(game) & 1;
            if blocked != 0 {
                return blocked;
            }
            length = length.wrapping_add(8);
            if word!(game.g_word_7E0D98 -= 1, reread) == 0 {
                return 0;
            }
        }
    }
}
pub unsafe fn place_rope_origin(game: &mut Game) {
    unsafe {
        let angle = 2 * *ram_ptr!(game; 0xcfb) as u16;
        let length = word!(game.grapple_beam_length);
        place_sample(angle, length, game);
        word!(game.x_pos_of_start_of_grapple_beam = word!(game.grapple_beam_grapple_start_x));
        word!(game.y_pos_of_start_of_grapple_beam = word!(game.grapple_beam_grapple_start_y));
    }
}
pub unsafe fn change_length(game: &mut Game) -> u8 {
    unsafe {
        if word!(game.grapple_beam_length_delta) == 0 {
            return 0;
        }
        let mut end =
            word!(game.grapple_beam_length_delta).wrapping_add(word!(game.grapple_beam_length));
        let shortening = word!(game.grapple_beam_length_delta) & 0x8000 != 0;
        if shortening && end < 8 {
            word!(game.grapple_beam_length_delta = 0);
            end = 8;
        } else if !shortening && end >= 63 {
            word!(game.grapple_beam_length_delta = 0);
            end = 63;
        }
        let angle = 2 * *ram_ptr!(game; 0xcfb) as u16;
        let mut length = word!(game.grapple_beam_length);
        while length != end {
            let sample_length = length.wrapping_add(if shortening { 7 } else { 57 });
            place_sample(angle, sample_length, game);
            if contact(game) != 0 {
                word!(game.grapple_beam_length = length);
                return 1;
            }
            length = if shortening {
                length.wrapping_sub(1)
            } else {
                length.wrapping_add(1)
            };
        }
        word!(game.grapple_beam_length = end);
        0
    }
}
pub unsafe fn advance_swing(game: &mut Game) -> u8 {
    unsafe {
        let scale = if word!(game.grapple_beam_flags) & 1 != 0 {
            160
        } else {
            256
        };
        let velocity =
            word!(game.grapple_beam_unkD2E).wrapping_add(word!(game.grapple_beam_unkD26));
        let backwards = velocity & 0x8000 != 0;
        let magnitude = if backwards {
            velocity.wrapping_neg()
        } else {
            velocity
        };
        let distance = (Multiply16x16(magnitude, scale, game) >> 8) as u16;
        if distance == 0 {
            return 0;
        }
        let delta = if backwards {
            distance.wrapping_neg()
        } else {
            distance
        };
        let target = 2 * (word!(game.grapple_beam_end_angle16).wrapping_add(delta) >> 8);
        let mut angle = 2 * *ram_ptr!(game; 0xcfb) as u16;
        while angle != target {
            let next = (if backwards {
                angle.wrapping_sub(2)
            } else {
                angle.wrapping_add(2)
            }) & 0x1ff;
            if check_clearance(next, game) & 1 != 0 {
                let stopped = ((angle >> 1) << 8) | 0x80;
                word!(game.grapple_beam_end_angle16 = stopped);
                word!(game.grapple_beam_end_angles_mirror = stopped);
                if (word!(game.g_word_7E0D98) == 6 || word!(game.g_word_7E0D98) == 5)
                    && word!(game.grapple_beam_length) == 8
                {
                    word!(game.grapple_beam_unkD36 |= 0x8000);
                    word! {game, 0; grapple_beam_unkD26, grapple_beam_unkD2E}
                } else {
                    word! {game;
                        grapple_beam_unkD30 = 16;
                        grapple_beam_unkD26 = -(word!(game.grapple_beam_unkD26) as i16 as i32 >> 1) as u16;
                        grapple_beam_unkD2E = -(word!(game.grapple_beam_unkD2E) as i16 as i32 >> 1) as u16;
                    }
                }
                return 1;
            }
            angle = next;
        }
        word!(game.grapple_beam_end_angle16 += delta);
        word!(game.grapple_beam_end_angles_mirror = word!(game.grapple_beam_end_angle16));
        word!(game.grapple_beam_unkD36 &= !0x8000);
        if word!(game.grapple_beam_unkD30 -= 1, reread) & 0x8000 != 0 {
            word!(game.grapple_beam_unkD30 = 0);
        }
        let impulse = word!(game.grapple_beam_unkD2E) as i16;
        let decayed = if impulse >= 0 {
            impulse.wrapping_sub(6).max(0)
        } else {
            impulse.wrapping_add(6).min(0)
        };
        word!(game.grapple_beam_unkD2E = decayed as u16);
        0
    }
}

pub(crate) fn react_beam(game: &mut Game) -> u8 {
    crate::game::player_collision::grapple::react_beam(&mut Runtime::new(game))
}
pub(crate) fn extend_beam(game: &mut Game) -> u8 {
    crate::game::player_collision::grapple::extend_beam(&mut Runtime::new(game))
}
pub(crate) fn start_rope_animation(game: &mut Game) {
    crate::game::grapple_rope::start(&mut Runtime::new(game));
}
pub(crate) fn draw_rope(game: &mut Game) {
    crate::game::grapple_rope::draw(&mut Runtime::new(game));
}
