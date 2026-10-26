//! Draygon's horizontal approach, firing pass and exit share one glide update.
//! Extended motion words stay native compatibility state until the whole boss migrates.
#![forbid(unsafe_code)]
use crate::game::enemy_layouts::EnemyLayout104 as Draygon;
use crate::game::enemy_projectiles::SpawnEprojWithGfx;
use crate::game::session_runtime::actor_cosine;
use crate::game::sprite_effect_runtime::draygon_body_bubble;
use crate::game_state::Game;
use crate::ram_map::{slot, word};
use crate::rtl::{abs16, words};
use crate::types::Point16U;

pub(crate) fn turret(game: &mut Game) {
    if word!(game.nmi_frame_counter_word) & 0x3f == 0 {
        let index = 2 * ((game.next_random() & 3) + 2);
        if slot!(game, Draygon[index as i32].var_40) == 0 {
            game.eproj_spawn_pt = Point16U {
                x: crate::rtl::rom_table_word!(game, 0xa587dc, index as isize),
                y: crate::rtl::rom_table_word!(game, 0xa587dc, index as isize + 1),
            };
            SpawnEprojWithGfx(3, index, 0x8e5e, game);
        }
    }
}
fn glide(right: bool, game: &mut Game) {
    let phase = slot!(game, Draygon[0].var_0D);
    slot!(
        game,
        Draygon[0].base.y_pos = actor_cosine(phase, 32, game).wrapping_add(384)
    );
    slot!(game, Draygon[0].var_0D, |old| (old as u8).wrapping_add(1)
        as u16);
    displace(right, game);
}
pub(super) fn displace(right: bool, game: &mut Game) {
    let speed = words!((slot!(game, Draygon[0].var_D), slot!(game, Draygon[0].var_E)));
    let delta = if right {
        speed
    } else {
        (-(speed as i32)) as u32
    };
    slot!(game, Draygon[0].base(x_pos, x_subpos) += delta);
}
fn beyond_right(x: u16) -> bool {
    x as i16 >= 0 && x.wrapping_sub(672) & 0x8000 == 0
}
pub(crate) fn begin(right: bool, game: &mut Game) {
    if right {
        turret(game);
    }
    slot!(
        game,
        Draygon[0].base.x_pos = if right {
            0xffb0
        } else {
            slot!(game, Draygon[0].var_02)
        }
    );
    slot!(game, Draygon[0].base.y_pos = 384);
    slot!(game, Draygon[0]; var_E = 0, var_D = 1, var_0D = 0);
    slot!(game, Draygon[0xc0].base; current_instruction = 0x9c06, instruction_timer = 1);
    slot!(game, Draygon[0].var_A = if right { 0x8b52 } else { 0x8cd4 });
    slot!(game, Draygon[0].base; current_instruction = if right { 0x97d1 } else { 0x97bb }, instruction_timer = 1);
    slot!(game, Draygon[0].var_20 = u16::from(right));
}
pub(crate) fn approach(right: bool, game: &mut Game) {
    turret(game);
    draygon_body_bubble(game);
    let distance = abs16(slot!(game, Draygon[0].base.x_pos).wrapping_sub(word!(game.samus_x_pos)));
    if distance.wrapping_sub(208) & 0x8000 != 0 {
        slot!(game, Draygon[0]; var_A = if right { 0x8bae } else { 0x8d30 }, var_03 = 16);
    } else {
        glide(right, game);
    }
}
pub(crate) fn fire(right: bool, game: &mut Game) {
    if right {
        turret(game);
    }
    if word!(game.samus_x_speed_divisor) == 0 {
        let mut leave = false;
        if word!(game.random_number) & 0xf == 0 {
            let remaining = slot!(game, Draygon[0].var_03).wrapping_sub(1);
            slot!(game, Draygon[0].var_03 = remaining);
            leave = remaining == 0;
            if !leave {
                slot!(game, Draygon[0].base; current_instruction = if right { 0x9c90 } else { 0x98fe }, instruction_timer = 1);
            }
        }
        if !leave {
            glide(right, game);
            // State 20 tests unsigned x_pos < 0, so its left-edge test never
            // succeeds. State 21 below intentionally uses the signed comparison.
            let x = slot!(game, Draygon[0].base.x_pos);
            if !right || !beyond_right(x) {
                return;
            }
        }
        if word!(game.samus_x_speed_divisor) == 0 {
            slot!(game, Draygon[0xc0].base; current_instruction = if right { 0x9bda } else { 0x97e7 }, instruction_timer = 1);
            slot!(game, Draygon[0].var_A = if right { 0x8c33 } else { 0x8db2 });
            return;
        }
    }
    slot!(game, Draygon[0].var_A = 0x8e19);
}
pub(crate) fn exit(right: bool, game: &mut Game) {
    draygon_body_bubble(game);
    if !right && word!(game.samus_x_speed_divisor) != 0 {
        slot!(game, Draygon[0].var_A = 0x8e19);
        return;
    }
    glide(right, game);
    let x = slot!(game, Draygon[0].base.x_pos);
    let finished = if right {
        beyond_right(x)
    } else {
        (x as i16) < 0 && x.wrapping_add(80) & 0x8000 != 0
    };
    if finished {
        slot!(game, Draygon[0].var_A = if right { 0x89b3 } else { 0x87f4 });
        slot!(game, Draygon[0].base; x_pos = if right { 592 } else { 0xffb0 }, y_pos = 0xffb0);
        slot!(game, Draygon[0].var_B = slot!(game, Draygon[0].var_C));
    }
}
