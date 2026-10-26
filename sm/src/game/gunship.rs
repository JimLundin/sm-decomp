//! Gunship landing, boarding, refilling, saving and escape.
//! Part addresses stay widened; Runtime projects their bytes through the existing owners.
#![forbid(unsafe_code)]
use crate::game::enemy_projectiles::{SpawnEprojWithRoomGfx, host::Effects};
use crate::game::item_pickups::Resource;
use crate::game::session::Position;
use crate::game::session_runtime::{Runtime, player_command, player_face_forward};
use crate::game::{
    gameplay_flag_runtime, item_pickups_runtime, message_boxes, palette_effects, save_slots,
};
use crate::game_state::Game;
use crate::ram_map::words as ram;

const X: usize = 2;
const Y: usize = 6;
const PROPERTIES: usize = 0xe;
const TIMER: usize = 0x18;
const INSTRUCTION: usize = 0x1a;
const DELAY: usize = 0x1c;
const PALETTE: usize = 0x1e;
const TILE: usize = 0x20;
const WAIT: usize = 0x30;
const BOB_FRAME: usize = 0x34;
const BOB_TIMER: usize = 0x36;
const REST_Y: usize = 0x38;
const PHASE: usize = 0x3a;
const PARAMETER_2: usize = 0x3e;

// A physical part view, not a new actor identity or a detached state copy.
#[derive(Clone, Copy)]
struct Part(usize);
impl Part {
    fn new(index: u16, part: usize) -> Self {
        Self(0xf78 + usize::from(index) + 64 * part)
    }
    fn read(self, rt: &Runtime<'_>, field: usize) -> u16 {
        rt.native_word(self.0 + field)
    }
    fn write(self, field: usize, value: u16, rt: &mut Runtime<'_>) {
        rt.write_slice_native(self.0 + field, value);
    }
    fn add(self, rt: &mut Runtime<'_>, field: usize, amount: u16) -> u16 {
        let value = self.read(rt, field).wrapping_add(amount);
        self.write(field, value, rt);
        value
    }
    fn instruction(self, rt: &mut Runtime<'_>, address: u16) {
        self.write(DELAY, 1, rt);
        self.write(INSTRUCTION, address, rt);
    }
}

fn expired(rt: &mut Runtime<'_>, part: Part, field: usize) -> bool {
    let before = part.read(rt, field);
    let after = before.wrapping_sub(1);
    part.write(field, after, rt);
    before == 1 || after & 0x8000 != 0
}

fn table(rt: &Runtime<'_>, address: u32, index: usize) -> u16 {
    rt.loaded_image()
        .expect("ROM must be loaded")
        .table_word(crate::rtl::lorom_offset(address) as usize, index as isize)
}

fn advance_native(rt: &mut Runtime<'_>, high: usize, low: usize, amount: i32) {
    // AddToHiLo reads low then high and writes low then high, in native order.
    let fraction = rt.native_word(low);
    let mut position = Position {
        whole: rt.native_word(high),
        fraction,
    };
    position.advance(amount);
    rt.write_slice_native(low, position.fraction);
    rt.write_slice_native(high, position.whole);
}

fn hatch(rt: &mut Runtime<'_>, index: u16, opening: bool) {
    Part::new(index, 2).instruction(rt, if opening { 0xa5be } else { 0xa5ee });
    Part::new(0, 0).write(WAIT, 144, rt);
    rt.queue_sound(3, 6, if opening { 0x14 } else { 0x15 });
}

fn save(rt: &mut Runtime<'_>) {
    // The legacy unaligned-word helper reads and writes this save flag LE.
    let flags = rt.word(0xd8f8) | 1;
    rt.write_slice_le(0xd8f8, flags);
    rt.write_slice_le(ram::load_station_index, 0);
    let slot = rt.word(ram::selected_save_slot);
    rt.foreign(|game| save_slots::save(slot, game));
}

fn resume_player(rt: &mut Runtime<'_>) {
    rt.write_slice_le(ram::frame_handler_alfa, 0xe695);
    rt.write_slice_le(ram::frame_handler_beta, 0xe725);
}

pub(crate) fn initialize_top(game: &mut Game) {
    let mut rt = Runtime::new(game);
    let top = Part::new(rt.word(ram::cur_enemy_index), 0);
    top.write(PROPERTIES, top.read(&rt, PROPERTIES) | 0x2400, &mut rt);
    top.write(DELAY, 1, &mut rt);
    top.write(TIMER, 0, &mut rt);
    top.write(INSTRUCTION, 0xa616, &mut rt);
    top.write(PALETTE, 3584, &mut rt);
    if rt.word(ram::game_state) != 40 || rt.word(ram::demo_set) != 0 {
        if rt.word(ram::loading_game_state) == 34 {
            top.write(Y, rt.word(ram::samus_y_pos).wrapping_sub(17), &mut rt);
            top.write(PHASE, 0xa80c, &mut rt);
        } else {
            let y = top.read(&rt, Y).wrapping_sub(25);
            top.write(Y, y, &mut rt);
            top.write(REST_Y, y, &mut rt);
            top.write(PHASE, 0xa9bd, &mut rt);
        }
    } else {
        rt.write_slice_le(ram::samus_y_pos, 1138);
        let y = top.read(&rt, Y).wrapping_sub(25);
        top.write(Y, y, &mut rt);
        top.write(REST_Y, y, &mut rt);
        top.write(PHASE, 0xab60, &mut rt);
        Part::new(0, 0).write(WAIT, 144, &mut rt);
    }
    rt.foreign(|game| palette_effects::spawn(0xe1c0, game));
    Part::new(rt.word(ram::cur_enemy_index), 0).write(BOB_TIMER, 1, &mut rt);
    Part::new(0, 0).write(BOB_FRAME, 0, &mut rt);
}

pub(crate) fn initialize_bottom(game: &mut Game) {
    let mut rt = Runtime::new(game);
    let bottom = Part::new(rt.word(ram::cur_enemy_index), 0);
    bottom.write(PROPERTIES, bottom.read(&rt, PROPERTIES) | 0x2400, &mut rt);
    bottom.write(DELAY, 1, &mut rt);
    bottom.write(TIMER, 0, &mut rt);
    let instruction = if bottom.read(&rt, PARAMETER_2) != 0 {
        0xa60e
    } else {
        0xa61c
    };
    bottom.write(INSTRUCTION, instruction, &mut rt);
    let drawing = (usize::from(rt.word(ram::cur_enemy_index)) >> 1) * 2
        + crate::ram_map::word_arrays::enemy_drawing_queue;
    bottom.write(TILE, rt.word(drawing + 212), &mut rt);
    bottom.write(PALETTE, 3584, &mut rt);
    if bottom.read(&rt, PARAMETER_2) != 0 {
        bottom.write(Y, rt.word(drawing + 122).wrapping_sub(1), &mut rt);
        if rt.word(ram::game_state) == 40 && rt.word(ram::demo_set) == 0 {
            bottom.instruction(&mut rt, 0xa5be);
        }
    } else if rt.word(ram::loading_game_state) == 34 {
        bottom.write(Y, rt.word(ram::samus_y_pos).wrapping_add(23), &mut rt);
    } else {
        bottom.add(&mut rt, Y, 15);
        bottom.write(BOB_TIMER, 71, &mut rt);
    }
    bottom.write(PHASE, 0xa7d7, &mut rt);
}

fn bob(rt: &mut Runtime<'_>, index: u16) {
    let top = Part::new(index, 0);
    if expired(rt, top, BOB_TIMER) {
        let index_in_table = usize::from(top.read(rt, BOB_FRAME).wrapping_mul(2));
        let base = crate::rtl::lorom_offset(0xa2a7cf) as usize;
        let delay = u16::from(rt.native_image().read(base + index_in_table));
        top.write(BOB_TIMER, delay, rt);
        let amount = rt.native_image().read(base + index_in_table + 1) as i8 as u16;
        top.add(rt, Y, amount);
        Part::new(index, 1).add(rt, Y, amount);
        Part::new(index, 2).add(rt, Y, amount);
        top.write(BOB_FRAME, top.read(rt, BOB_FRAME).wrapping_add(1) & 3, rt);
    }
}

pub(crate) fn advance(game: &mut Game) -> u32 {
    let mut rt = Runtime::new(game);
    let bottom = Part::new(rt.word(ram::cur_enemy_index), 1);
    if expired(&mut rt, bottom, BOB_TIMER) {
        rt.queue_sound(2, 6, 0x4d);
        bottom.write(BOB_TIMER, 70, &mut rt);
    }
    let top = Part::new(rt.word(ram::cur_enemy_index), 0);
    if top.read(&rt, PHASE).wrapping_add(0x56be) & 0x8000 == 0
        && top.read(&rt, PHASE).wrapping_add(0x53e5) & 0x8000 != 0
    {
        let index = rt.word(ram::cur_enemy_index);
        bob(&mut rt, index);
    }
    0xa20000 | u32::from(top.read(&rt, PHASE))
}

fn descend(rt: &mut Runtime<'_>, index: u16) {
    let top = Part::new(index, 0);
    let slow = top.read(rt, Y).wrapping_sub(768) & 0x8000 == 0;
    let amount = if slow { 0x28000 } else { 0x48000 };
    advance_native(rt, ram::samus_y_pos, ram::samus_y_subpos, amount);
    for part in [2, 1, 0] {
        let at = Part::new(index, part).0 + Y;
        advance_native(rt, at, at + 2, amount);
    }
    if slow && top.read(rt, Y).wrapping_sub(1119) & 0x8000 == 0 {
        top.write(Y, 1119, rt);
        Part::new(index, 1).write(Y, 1159, rt);
        Part::new(index, 2).write(Y, top.read(rt, Y).wrapping_sub(1), rt);
        top.write(PHASE, 0xa8d0, rt);
        top.write(REST_Y, 0, rt);
    }
}

fn settle(rt: &mut Runtime<'_>, index: u16) {
    let top = Part::new(index, 0);
    let row = usize::from(top.read(rt, REST_Y));
    let y = rt
        .word(ram::samus_y_pos)
        .wrapping_add(table(rt, 0xa2a622, row));
    rt.write_slice_le(ram::samus_y_pos, y);
    for part in [2, 1, 0] {
        let part = Part::new(index, part);
        // Read the live position before each separate fallible table lookup.
        let y = part.read(rt, Y).wrapping_add(table(rt, 0xa2a622, row));
        part.write(Y, y, rt);
    }
    if top.add(rt, REST_Y, 1).wrapping_sub(17) & 0x8000 == 0 {
        top.write(PHASE, 0xa942, rt);
        top.write(REST_Y, top.read(rt, Y), rt);
        top.write(BOB_TIMER, 1, rt);
        Part::new(0, 0).write(BOB_FRAME, 0, rt);
        rt.write_slice_le(ram::samus_x_pos, top.read(rt, X).wrapping_add(1));
        rt.write_slice_le(ram::samus_prev_x_pos, rt.word(ram::samus_x_pos));
        hatch(rt, index, true);
    }
}

fn await_boarding(rt: &mut Runtime<'_>, index: u16) {
    if rt.word(ram::game_state) != 8 || rt.word(ram::frame_handler_alfa) != 0xe695 {
        return;
    }
    let top = Part::new(index, 0);
    if (top
        .read(rt, X)
        .wrapping_sub(8)
        .wrapping_sub(rt.word(ram::samus_x_pos)) as i16)
        < 0
        && (top
            .read(rt, X)
            .wrapping_add(8)
            .wrapping_sub(rt.word(ram::samus_x_pos)) as i16)
            >= 0
        && (top
            .read(rt, Y)
            .wrapping_sub(64)
            .wrapping_sub(rt.word(ram::samus_y_pos)) as i16)
            < 0
        && (top.read(rt, Y).wrapping_sub(rt.word(ram::samus_y_pos)) as i16) >= 0
        && rt.byte(0xa1f) == 0
        && rt.word(ram::joypad1_newkeys) & 0x400 != 0
    {
        top.write(PHASE, 0xaa4f, rt);
        if rt.word(ram::samus_x_pos) != 1152 {
            rt.write_slice_le(ram::samus_x_pos, top.read(rt, X));
            rt.write_slice_le(ram::samus_prev_x_pos, rt.word(ram::samus_x_pos));
        }
        rt.foreign(player_face_forward);
        rt.foreign(|game| player_command(0x1a, game));
        rt.write_slice_le(ram::elevator_status, 0);
        Part::new(index, 2).write(Y, top.read(rt, Y).wrapping_sub(1), rt);
        hatch(rt, index, true);
    }
}

fn move_player(rt: &mut Runtime<'_>, index: u16, phase: u16) {
    let top = Part::new(index, 0);
    let entering = phase == 0xaa5d;
    let target = top
        .read(rt, REST_Y)
        .wrapping_add(if entering { 18 } else { -30i16 as u16 });
    let delta = if entering {
        2
    } else if phase == 0xa950 {
        -1i16 as u16
    } else {
        -2i16 as u16
    };
    rt.write_slice_le(
        ram::samus_y_pos,
        rt.word(ram::samus_y_pos).wrapping_add(delta),
    );
    if (rt.word(ram::samus_y_pos).wrapping_sub(target) & 0x8000 == 0) == entering {
        top.write(
            PHASE,
            match phase {
                0xa950 => 0xa987,
                0xaa5d => 0xaa94,
                _ => 0xaba5,
            },
            rt,
        );
        hatch(rt, index, false);
    }
}

fn replenish(rt: &mut Runtime<'_>, index: u16) {
    let top = Part::new(index, 0);
    if rt.foreign(|game| gameplay_flag_runtime::CheckEventHappened(0xe, game)) != 0 {
        top.write(PHASE, 0xabc7, rt);
        Part::new(index, 1).write(PHASE, 0, rt);
        Part::new(index, 1).write(REST_Y, 0, rt);
        rt.write_slice_le(ram::substate, 0);
        rt.write_slice_le(ram::suit_pickup_light_beam_pos, 0);
        rt.write_slice_native(0xdf0, 0);
        rt.write_slice_native(0xdf2, 0);
        rt.foreign(|game| player_command(0xa, game));
    } else {
        for resource in [
            Resource::Health,
            Resource::Missiles,
            Resource::SuperMissiles,
            Resource::PowerBombs,
        ] {
            rt.foreign(|game| item_pickups_runtime::restore(resource, 2, game));
        }
        let full = [
            (ram::samus_reserve_health, ram::samus_max_reserve_health),
            (ram::samus_health, ram::samus_max_health),
            (ram::samus_missiles, ram::samus_max_missiles),
            (ram::samus_super_missiles, ram::samus_max_super_missiles),
            (ram::samus_power_bombs, ram::samus_max_power_bombs),
        ]
        .into_iter()
        .all(|(current, maximum)| (rt.word(current).wrapping_sub(rt.word(maximum)) as i16) >= 0);
        if full {
            top.write(PHASE, 0xab1f, rt);
        }
    }
}

fn upload_escape(rt: &mut Runtime<'_>, index: u16) {
    let row = usize::from(rt.word(ram::substate)) >> 1;
    let mut tail = rt.word(ram::vram_write_queue_tail);
    rt.write_slice_native(0xd0 + usize::from(tail), 1024);
    tail = tail.wrapping_add(2);
    let source = table(rt, 0xa2ac07, row);
    rt.write_slice_native(0xd0 + usize::from(tail), source);
    tail = tail.wrapping_add(2);
    rt.set_byte(0xd0 + usize::from(tail), 0x94);
    tail = tail.wrapping_add(1);
    let destination = table(rt, 0xa2ac11, row);
    rt.write_slice_native(0xd0 + usize::from(tail), destination);
    rt.write_slice_le(ram::vram_write_queue_tail, tail.wrapping_add(2));
    rt.write_slice_le(ram::substate, rt.word(ram::substate).wrapping_add(2));
    if rt.word(ram::substate).wrapping_sub(10) & 0x8000 == 0 {
        Part::new(index, 0).write(PHASE, 0xac1b, rt);
        rt.write_slice_le(ram::substate, 0);
    }
}

fn follow_player(rt: &mut Runtime<'_>, index: u16) {
    let y = rt.word(ram::samus_y_pos).wrapping_sub(17);
    Part::new(index, 0).write(Y, rt.word(ram::samus_y_pos).wrapping_sub(17), rt);
    Part::new(index, 2).write(Y, y.wrapping_sub(1), rt);
    Part::new(index, 1).write(Y, rt.word(ram::samus_y_pos).wrapping_add(23), rt);
}

fn shake(rt: &mut Runtime<'_>, index: u16) {
    let bottom = Part::new(index, 1);
    let timer = bottom.read(rt, REST_Y);
    let amount = if timer.wrapping_sub(64) & 0x8000 != 0 {
        1u16
    } else {
        2
    };
    let y = rt.word(ram::samus_y_pos).wrapping_add(if timer & 1 != 0 {
        amount
    } else {
        amount.wrapping_neg()
    });
    rt.write_slice_le(ram::samus_y_pos, y);
    rt.write_slice_le(ram::samus_prev_y_pos, y);
    follow_player(rt, index);
    let timer = bottom.add(rt, REST_Y, 1);
    if timer.wrapping_sub(128) & 0x8000 != 0 {
        if timer == 64 {
            for parameter in (0..12).step_by(2) {
                rt.foreign(|game| SpawnEprojWithRoomGfx(0xa379, parameter, game));
            }
        }
    } else {
        Part::new(index, 0).write(PHASE, 0xacd7, rt);
        Part::new(0, 0).write(WAIT, 0, rt);
    }
}

fn accelerate(rt: &mut Runtime<'_>, index: u16) {
    let bottom = Part::new(index, 1);
    let velocity = bottom.add(rt, PHASE, 64);
    if (velocity & 0xff00).wrapping_sub(2560) & 0x8000 == 0 {
        bottom.write(PHASE, 2304, rt);
    }
    let mut position = Position {
        whole: rt.word(ram::samus_y_pos),
        fraction: rt.word(ram::samus_y_subpos),
    };
    position.advance(-(i32::from(bottom.read(rt, PHASE)) << 8));
    rt.write_slice_le(ram::samus_y_subpos, position.fraction);
    rt.write_slice_le(ram::samus_y_pos, position.whole);
    follow_player(rt, index);
}

pub(crate) fn phase(index: u16, phase: u16, game: &mut Game) {
    let mut rt = Runtime::new(game);
    let top = Part::new(index, 0);
    match phase {
        0xa80c => descend(&mut rt, index),
        0xa8d0 => settle(&mut rt, index),
        0xa942 | 0xa987 | 0xaa4f | 0xaa94 | 0xab60 | 0xaba5 => {
            if expired(&mut rt, Part::new(0, 0), WAIT) {
                let next = match phase {
                    0xa942 => 0xa950,
                    0xaa4f => 0xaa5d,
                    0xaa94 => 0xaaa2,
                    0xab60 => 0xab6e,
                    _ => 0xa9bd,
                };
                top.write(PHASE, next, &mut rt);
                if phase == 0xa987 {
                    resume_player(&mut rt);
                    rt.write_slice_le(ram::loading_game_state, 5);
                    save(&mut rt);
                } else if phase == 0xaba5 && rt.word(ram::game_state).wrapping_sub(40) & 0x8000 != 0
                {
                    resume_player(&mut rt);
                }
            }
        }
        0xa950 | 0xaa5d | 0xab6e => move_player(&mut rt, index, phase),
        0xa9bd => await_boarding(&mut rt, index),
        0xaaa2 => replenish(&mut rt, index),
        0xab1f => {
            let response = rt.foreign(|game| message_boxes::poll(0x1c, game));
            if response < 0 {
                return;
            }
            if response != 2 {
                save(&mut rt);
            }
            top.write(PHASE, 0xab60, &mut rt);
            hatch(&mut rt, index, true);
        }
        0xabc7 => upload_escape(&mut rt, index),
        0xac1b => shake(&mut rt, index),
        0xacd7 => {
            rt.write_slice_le(ram::samus_y_pos, rt.word(ram::samus_y_pos).wrapping_sub(2));
            follow_player(&mut rt, index);
            if top.read(&rt, Y).wrapping_sub(896) & 0x8000 != 0 {
                top.write(PHASE, 0xad0e, &mut rt);
                Part::new(index, 1).write(PHASE, 512, &mut rt);
            }
        }
        0xad0e | 0xad2d => {
            accelerate(&mut rt, index);
            if phase == 0xad0e && top.read(&rt, Y).wrapping_sub(256) & 0x8000 != 0 {
                top.write(PHASE, 0xad2d, &mut rt);
                rt.write_slice_le(ram::game_state, 38);
                rt.write_slice_le(ram::screen_fade_delay, 0);
                rt.write_slice_le(ram::screen_fade_counter, 0);
            }
        }
        _ => unreachable!("only registered gunship phases enter this module"),
    }
}
