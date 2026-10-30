//! Brinstar Pipe Bug emergence, horizontal flight, reset and animation selection.
//! Native actor words remain authoritative across the existing query boundaries.
#![forbid(unsafe_code)]
use crate::actors::access::ActorKey as Key;
use crate::actors::host::LeaseActors;
use crate::actors::host::abs16;
use crate::actors::host::{ActorHost, ActorRuntime, Axis};
use crate::actors::layouts::EnemyLayout72 as Bug;
use crate::actors::queries;
use crate::cartridge::rom_table_word;
use crate::ram::RawRam as _;
use crate::ram::{slot, word};

pub fn initialize<H: ActorHost + ActorRuntime + LeaseActors + ?Sized>(game: &mut H) {
    let e = i32::from(word!(game.cur_enemy_index));
    slot!(game, Bug[e].var_B = slot!(game, Bug[e].base.x_pos));
    let y = slot!(game, Bug[e].base.y_pos);
    slot!(game, Bug[e]; var_C = y, var_00 = y.wrapping_sub(16), var_F = 0x8880, var_D = 48, var_E = 0, var_01 = 0);
    slot!(game, Bug[e].base.current_instruction = 0x87ab);
    if slot!(game, Bug[e].parameter_1) != 0 {
        slot!(game, Bug[e].base.current_instruction = 0x8a1d);
    }
}
pub fn state_address<H: ActorHost + ActorRuntime + LeaseActors + ?Sized>(game: &H) -> u32 {
    let e = i32::from(word!(game.cur_enemy_index));
    0xb30000 | u32::from(slot!(game, Bug[e].var_F))
}
fn update_animation<H: ActorHost + ActorRuntime + LeaseActors + ?Sized>(game: &mut H) {
    let e = i32::from(word!(game.cur_enemy_index));
    let phase = slot!(game, Bug[e].var_E);
    if phase != slot!(game, Bug[e].var_01) {
        slot!(game, Bug[e].var_01 = phase);
        let table = if slot!(game, Bug[e].parameter_1) != 0 {
            0xb38833
        } else {
            0xb3882b
        };
        slot!(game, Bug[e].base; current_instruction = rom_table_word!(game, table, phase as isize), instruction_timer = 1, timer = 0);
    }
}
pub fn await_view<H: ActorHost + ActorRuntime + LeaseActors + ?Sized>(_: u16, game: &mut H) {
    if !game.current_outside_screen() {
        let current = i32::from(word!(game.cur_enemy_index));
        slot!(game, Bug[current].var_F = 0x8890);
    }
}
pub fn await_player<H: ActorHost + ActorRuntime + LeaseActors + ?Sized>(k: u16, game: &mut H) {
    let e = i32::from(k);
    let dy = game.player_delta(k, Axis::Y) as i16;
    if dy < 0 && (i32::from(dy) + 96) & 0x8000 == 0 {
        let dx = game.player_delta(k, Axis::X);
        slot!(game, Bug[e].var_A, |direction| direction & 0x7fff
            | dx & 0x8000);
        if abs16(slot!(game, Bug[e].base.x_pos).wrapping_sub(word!(game.samus_x_pos))) < 0x40 {
            slot!(game, Bug[e].var_F = 0x88e3);
            slot!(game, Bug[e].base.properties, |properties| properties
                & !0x100);
            slot!(game, Bug[e].base.timer = 0);
            slot!(
                game,
                Bug[e].var_E = if slot!(game, Bug[e].var_A) & 0x8000 != 0 {
                    0
                } else {
                    2
                }
            );
            update_animation(game);
        }
    }
}
pub fn rise<H: ActorHost + ActorRuntime + LeaseActors + ?Sized>(k: u16, game: &mut H) {
    let e = i32::from(k);
    let previous = slot!(game, Bug[e].base.y_subpos);
    slot!(
        game,
        Bug[e].base.y_subpos = slot!(game, Bug[e].base.y_subpos).wrapping_sub(1)
    );
    slot!(
        game,
        Bug[e].base.y_pos =
            (i32::from(slot!(game, Bug[e].base.y_pos)) + i32::from(previous != 0) - 2) as u16
    );
    if (slot!(game, Bug[e].var_00) as i16).wrapping_sub(slot!(game, Bug[e].base.y_pos) as i16) >= 0
        && slot!(game, Bug[e].base.y_pos) < word!(game.samus_y_pos)
    {
        slot!(game, Bug[e].var_E, |phase| phase | 1);
        update_animation(game);
        slot!(game, Bug[e].var_F = 0x891c);
    }
}
pub fn fly<H: ActorHost + ActorRuntime + LeaseActors + ?Sized>(k: u16, game: &mut H) {
    let e = i32::from(k);
    let step = if slot!(game, Bug[e].var_A) & 0x8000 == 0 {
        2
    } else {
        0xfffe
    };
    slot!(
        game,
        Bug[e].base.x_subpos = slot!(game, Bug[e].base.x_subpos)
    );
    slot!(
        game,
        Bug[e].base.x_pos = slot!(game, Bug[e].base.x_pos).wrapping_add(step)
    );
    if queries::leaving_screen(&game.lease(), Key::from_legacy(k)) as u8 & 1 != 0 {
        slot!(game, Bug[e].base; x_pos = slot!(game, Bug[e].var_B), x_subpos = 0);
        let y = slot!(game, Bug[e].var_C);
        slot!(game, Bug[e].base; y_pos = y, y_subpos = y);
        slot!(game, Bug[e].var_E = 0);
        update_animation(game);
        slot!(game, Bug[e].base.properties, |properties| properties
            | 0x100);
        slot!(game, Bug[e]; var_D = 48, var_F = 0x897e);
    }
}
pub fn cooldown<H: ActorHost + ActorRuntime + LeaseActors + ?Sized>(k: u16, game: &mut H) {
    let e = i32::from(k);
    let previous = slot!(game, Bug[e].var_D);
    slot!(
        game,
        Bug[e].var_D = slot!(game, Bug[e].var_D).wrapping_sub(1)
    );
    if previous == 1 {
        slot!(game, Bug[e].var_F = 0x8890);
    }
}
