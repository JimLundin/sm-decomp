//! The gunship, Samus's ship: it lands on Zebes, lets Samus enter to refill and save,
//! and lifts off in the escape (bank $A2:A622-$A2:AD80). Its top, bottom and entrance
//! pad are three consecutive slots.
#![forbid(unsafe_code)]
use crate::actors::combat;
use crate::actors::host::{ActorLease, ActorRuntime as _, BossEffects as _, sign_extend8};
use crate::actors::layouts::part_view;
use crate::display::transfer::{Source, TilemapUpload, queue_tilemap};
use crate::enemy_projectiles::host::Effects as _;
use crate::geometry::Position;
use crate::item_pickups::Resource;
use crate::player_position::{PlayerPosition as _, PlayerWord};
use crate::player_projectiles::content::{self, table};
use crate::ram::words as W;
use crate::session_slot;

part_view! {
    /// A gunship part's slot. Slot 0's `entrance_timer` times the entrance's opening and
    /// closing. `step` is the top's brake row or rest Y, and the bottom's liftoff counter;
    /// `function` is the top's gunship function and the bottom's liftoff speed. `pad`
    /// (parameter 2) marks the entrance pad.
    Gunship: EnemyLayout8 {
        words: [entrance_timer, var_b, hover_frame, hover_timer, step, function, parameter_1, pad],
    }
}

/// What the gunship reaches outside its lease: palette effects, Samus, events, the
/// inventory, the save confirmation and the save itself.
pub trait Host {
    fn spawn_palette_effect(&mut self, definition: u16);
    fn face_player_forward(&mut self);
    fn event_happened(&mut self, event: u16) -> bool;
    fn restore(&mut self, resource: Resource, amount: u16);
    /// The message box `kind`'s response, negative until it closes.
    fn message_box_response(&mut self, kind: u16) -> i32;
    fn save_game(&mut self, slot: u16);
    /// A stored function outside the gunship table retains the original dispatch.
    fn run_other_pre_instruction(&mut self, address: u32);
}

/// Gunship functions ($A2:A80C-$A2:AD2D).
const DESCENDING: u16 = 0xa80c;
const APPLY_BRAKES: u16 = 0xa8d0;
const WAIT_TO_OPEN_FOR_EJECT: u16 = 0xa942;
const EJECT_SAMUS: u16 = 0xa950;
const WAIT_TO_CLOSE_AND_SAVE: u16 = 0xa987;
const IDLE: u16 = 0xa9bd;
const WAIT_TO_OPEN_FOR_ENTRY: u16 = 0xaa4f;
const LOWER_SAMUS: u16 = 0xaa5d;
const WAIT_TO_CLOSE_FOR_ENTRY: u16 = 0xaa94;
const ENTERED: u16 = 0xaaa2;
const SAVE_CONFIRMATION: u16 = 0xab1f;
const WAIT_TO_OPEN_FOR_EXIT: u16 = 0xab60;
const RAISE_SAMUS: u16 = 0xab6e;
const WAIT_TO_CLOSE_FOR_EXIT: u16 = 0xaba5;
const LOAD_DUST_CLOUD_TILES: u16 = 0xabc7;
const FIRE_UP_ENGINES: u16 = 0xac1b;
const STEADY_RISE: u16 = 0xacd7;
const ACCELERATING_SET_GAME_STATE: u16 = 0xad0e;
const ACCELERATING: u16 = 0xad2d;
/// The bottom's RTL function.
const BOTTOM: u16 = 0xa7d7;
/// Gunship brakes movement data ($A2:A622), hover data ($A2:A7CF) and the dust cloud
/// tiles' sources ($A2:AC07) and VRAM addresses ($A2:AC11).
const BRAKES: u32 = 0xa2_a622;
const HOVER: u32 = 0xa2_a7cf;
const DUST_SOURCES: u32 = 0xa2_ac07;
const DUST_DESTINATIONS: u32 = 0xa2_ac11;
/// The save station flags word, read and written little-endian.
const USED_SAVE_STATIONS: usize = 0xd8f8;
/// Native presentation aliases cleared on liftoff: suit colour and crystal flash delay.
const UNREAD: [usize; 2] = [0xdf0, 0xdf2];
/// The gunship's top, bottom and entrance pad, from the current slot.
const TOP: i32 = 0;
const BOTTOM_PART: i32 = 64;
const PAD: i32 = 128;

/// Slot `part` past the gunship at `index`.
fn part(index: u16, part: i32) -> i32 {
    i32::from(index) + part
}
/// A timer `before` counted down; whether it ran out, reaching zero or going negative.
fn tick(before: u16) -> (u16, bool) {
    let after = before.wrapping_sub(1);
    (after, before == 1 || after & 0x8000 != 0)
}
fn hover_expired(rt: &mut impl ActorLease, slot: i32) -> bool {
    let (after, expired) = tick(session_slot!(rt, Gunship[slot].hover_timer));
    session_slot!(rt, Gunship[slot].hover_timer = after);
    expired
}
fn set_instruction(rt: &mut impl ActorLease, slot: i32, list: u16) {
    session_slot!(rt, Gunship[slot].base; instruction_timer = 1, current_instruction = list);
}
fn add_y(rt: &mut impl ActorLease, slot: i32, amount: u16) {
    session_slot!(rt, Gunship[slot].base.y_pos, |y| y.wrapping_add(amount));
}
/// Advance aligned owned parts directly. Odd, widened off-table and non-LE
/// native identities retain the original fraction-first compatibility accesses.
fn advance_part(rt: &mut impl ActorLease, slot: i32, amount: i32) {
    if cfg!(target_endian = "little") && (0..2048).contains(&slot) && slot % 64 == 0 {
        let actor = &mut rt.session_mut().actors.slots[usize::try_from(slot / 64).unwrap()];
        actor.y.advance(amount);
        return;
    }
    let high = crate::ram_slot!(@at Gunship, slot, base.y_pos);
    let low = high + 2;
    let fraction = rt.lease_native_word(low);
    let mut position = Position {
        whole: rt.lease_native_word(high),
        fraction,
    };
    position.advance(amount);
    rt.write_slice_native(low, position.fraction);
    rt.write_slice_native(high, position.whole);
}
/// Add a 16.16 `amount` to Samus's Y position, reading and storing the fraction first.
fn advance_player(rt: &mut impl ActorLease, amount: i32) {
    let fraction = rt.player_y_fraction();
    let mut position = Position {
        whole: rt.player_y(),
        fraction,
    };
    position.advance(amount);
    rt.set_player_word(PlayerWord::YFraction, position.fraction);
    rt.set_player_y(position.whole);
}
/// Open or close the entrance pad.
fn entrance(rt: &mut impl ActorLease, index: u16, opening: bool) {
    set_instruction(rt, part(index, PAD), if opening { 0xa5be } else { 0xa5ee });
    session_slot!(rt, Gunship[0].entrance_timer = 144);
    rt.queue_sound(3, 6, if opening { 0x14 } else { 0x15 });
}
fn save(rt: &mut impl ActorLease) {
    crate::probe::hit("gunship.save");
    let flags = rt.word(USED_SAVE_STATIONS) | 1;
    rt.write_slice_le(USED_SAVE_STATIONS, flags);
    rt.session_mut().menu_screens.save_selection.load_station = 0;
    let slot = rt.session().menu_screens.save_selection.selected_slot;
    rt.save_game(slot);
}
fn unlock_player(rt: &mut impl ActorLease) {
    rt.session_mut().player.transitions.frame_alpha =
        crate::player_transitions::state::FrameAlphaMode::from_raw(0xe695);
    rt.session_mut().player.transitions.frame_beta =
        crate::player_transitions::state::FrameBetaMode::Gameplay;
}

/// Initialisation AI - gunship top ($A2:A644).
pub fn initialize_top(rt: &mut impl ActorLease) {
    let top = combat::current_slot(rt);
    session_slot!(rt, Gunship[top].base.properties, |properties| properties
        | 0x2400);
    session_slot!(rt, Gunship[top].base; instruction_timer = 1, timer = 0, current_instruction = 0xa616);
    session_slot!(rt, Gunship[top].base.palette_index = 3584);
    if rt.word(W::GAME_STATE) != 40 || rt.session().flow.demo_set != 0 {
        if rt.session().menu_screens.save_selection.loading_game_state == 34 {
            let y = rt.player_y().wrapping_sub(17);
            session_slot!(rt, Gunship[top].base.y_pos = y);
            session_slot!(rt, Gunship[top].function = DESCENDING);
        } else {
            let y = session_slot!(rt, Gunship[top].base.y_pos).wrapping_sub(25);
            session_slot!(rt, Gunship[top].base.y_pos = y);
            session_slot!(rt, Gunship[top]; step = y, function = IDLE);
        }
    } else {
        rt.set_player_y(1138);
        let y = session_slot!(rt, Gunship[top].base.y_pos).wrapping_sub(25);
        session_slot!(rt, Gunship[top].base.y_pos = y);
        session_slot!(rt, Gunship[top]; step = y, function = WAIT_TO_OPEN_FOR_EXIT);
        session_slot!(rt, Gunship[0].entrance_timer = 144);
    }
    rt.spawn_palette_effect(0xe1c0);
    let top = combat::current_slot(rt);
    session_slot!(rt, Gunship[top].hover_timer = 1);
    session_slot!(rt, Gunship[0].hover_frame = 0);
}

/// Initialisation AI - gunship bottom / entrance pad ($A2:A6D2).
pub fn initialize_bottom(rt: &mut impl ActorLease) {
    let bottom = combat::current_slot(rt);
    session_slot!(rt, Gunship[bottom].base.properties, |properties| properties
        | 0x2400);
    session_slot!(rt, Gunship[bottom].base; instruction_timer = 1, timer = 0);
    let pad = session_slot!(rt, Gunship[bottom].pad) != 0;
    session_slot!(
        rt,
        Gunship[bottom].base.current_instruction = if pad { 0xa60e } else { 0xa61c }
    );
    let previous = bottom - BOTTOM_PART;
    let tiles = session_slot!(rt, Gunship[previous].base.vram_tiles_index);
    session_slot!(rt, Gunship[bottom].base; vram_tiles_index = tiles, palette_index = 3584);
    if session_slot!(rt, Gunship[bottom].pad) != 0 {
        // The entrance pad sits below the top, two slots before it.
        let y = session_slot!(rt, Gunship[bottom - PAD].base.y_pos);
        session_slot!(rt, Gunship[bottom].base.y_pos = y.wrapping_sub(1));
        if rt.word(W::GAME_STATE) == 40 && rt.session().flow.demo_set == 0 {
            set_instruction(rt, bottom, 0xa5be);
        }
    } else if rt.session().menu_screens.save_selection.loading_game_state == 34 {
        let y = rt.player_y().wrapping_add(23);
        session_slot!(rt, Gunship[bottom].base.y_pos = y);
    } else {
        add_y(rt, bottom, 15);
        session_slot!(rt, Gunship[bottom].hover_timer = 71);
    }
    session_slot!(rt, Gunship[bottom].function = BOTTOM);
}

/// Process gunship hover ($A2:A784): the three parts bob together.
fn hover(rt: &mut impl ActorLease, index: u16) {
    let top = part(index, TOP);
    if !hover_expired(rt, top) {
        return;
    }
    let row = usize::from(session_slot!(rt, Gunship[top].hover_frame).wrapping_mul(2));
    let image = rt.native_image();
    let delay = u16::from(image.read(content::offset(HOVER) + row));
    session_slot!(rt, Gunship[top].hover_timer = delay);
    let amount = sign_extend8(u16::from(
        rt.native_image().read(content::offset(HOVER) + row + 1),
    ));
    for slot in [TOP, BOTTOM_PART, PAD] {
        add_y(rt, part(index, slot), amount);
    }
    session_slot!(rt, Gunship[top].hover_frame, |frame| frame.wrapping_add(1)
        & 3);
}

/// Main AI - gunship top ($A2:A759): the engine hum and the hover; its function, run
/// next through the pre-instruction table.
pub fn main(rt: &mut impl ActorLease) -> u32 {
    let bottom = combat::current_slot(rt) + BOTTOM_PART;
    if hover_expired(rt, bottom) {
        rt.queue_sound(2, 6, 0x4d);
        session_slot!(rt, Gunship[bottom].hover_timer = 70);
    }
    let top = combat::current_slot(rt);
    let function = session_slot!(rt, Gunship[top].function);
    if function.wrapping_add(0x56be) & 0x8000 == 0 && function.wrapping_add(0x53e5) & 0x8000 != 0 {
        let index = rt.session().actors.controls.selected;
        hover(rt, index);
    }
    0xa2_0000 | u32::from(session_slot!(rt, Gunship[top].function))
}

/// Landing on Zebes - descending ($A2:A80C).
fn descend(rt: &mut impl ActorLease, index: u16) {
    let top = part(index, TOP);
    let slow = session_slot!(rt, Gunship[top].base.y_pos).wrapping_sub(768) & 0x8000 == 0;
    let amount = if slow { 0x28000 } else { 0x48000 };
    advance_player(rt, amount);
    for slot in [PAD, BOTTOM_PART, TOP] {
        advance_part(rt, part(index, slot), amount);
    }
    if slow && session_slot!(rt, Gunship[top].base.y_pos).wrapping_sub(1119) & 0x8000 == 0 {
        session_slot!(rt, Gunship[top].base.y_pos = 1119);
        session_slot!(rt, Gunship[part(index, BOTTOM_PART)].base.y_pos = 1159);
        let y = session_slot!(rt, Gunship[top].base.y_pos).wrapping_sub(1);
        session_slot!(rt, Gunship[part(index, PAD)].base.y_pos = y);
        session_slot!(rt, Gunship[top]; function = APPLY_BRAKES, step = 0);
    }
}

/// Landing on Zebes - apply brakes ($A2:A8D0).
fn brake(rt: &mut impl ActorLease, index: u16) {
    let top = part(index, TOP);
    let row = usize::from(session_slot!(rt, Gunship[top].step)).cast_signed();
    let y = rt.player_y().wrapping_add(table(rt, BRAKES, row));
    rt.set_player_y(y);
    for slot in [PAD, BOTTOM_PART, TOP] {
        // The live position is read before each separately failing table read.
        let slot = part(index, slot);
        let y = session_slot!(rt, Gunship[slot].base.y_pos).wrapping_add(table(rt, BRAKES, row));
        session_slot!(rt, Gunship[slot].base.y_pos = y);
    }
    let step = session_slot!(rt, Gunship[top].step).wrapping_add(1);
    session_slot!(rt, Gunship[top].step = step);
    if step.wrapping_sub(17) & 0x8000 == 0 {
        let y = session_slot!(rt, Gunship[top].base.y_pos);
        session_slot!(rt, Gunship[top]; function = WAIT_TO_OPEN_FOR_EJECT, step = y, hover_timer = 1);
        session_slot!(rt, Gunship[0].hover_frame = 0);
        let x = session_slot!(rt, Gunship[top].base.x_pos).wrapping_add(1);
        rt.set_player_x(x);
        rt.set_previous_player_x(x);
        entrance(rt, index, true);
    }
}

/// Idle - handle letting Samus enter ($A2:A9BD): Samus standing on the pad presses
/// down.
fn idle(rt: &mut impl ActorLease, index: u16) {
    if rt.word(W::GAME_STATE) != 8 || rt.session().player.transitions.frame_alpha.to_raw() != 0xe695
    {
        return;
    }
    let top = part(index, TOP);
    let (x, y) = (
        session_slot!(rt, Gunship[top].base.x_pos),
        session_slot!(rt, Gunship[top].base.y_pos),
    );
    let (samus_x, samus_y) = (rt.player_x(), rt.player_y());
    let negative = |value: u16| value & 0x8000 != 0;
    if negative(x.wrapping_sub(8).wrapping_sub(samus_x))
        && !negative(x.wrapping_add(8).wrapping_sub(samus_x))
        && negative(y.wrapping_sub(64).wrapping_sub(samus_y))
        && !negative(y.wrapping_sub(samus_y))
        && rt.session().player.movement_and_facing.to_le_bytes()[1] == 0
        && rt.inputs().pressed & 0x400 != 0
    {
        session_slot!(rt, Gunship[top].function = WAIT_TO_OPEN_FOR_ENTRY);
        if rt.player_x() != 1152 {
            let x = session_slot!(rt, Gunship[top].base.x_pos);
            rt.set_player_x(x);
            rt.set_previous_player_x(x);
        }
        rt.face_player_forward();
        rt.foreign(|game| game.command_player(0x1a));
        rt.session_mut().room.entry.elevator_status = 0;
        let y = session_slot!(rt, Gunship[top].base.y_pos).wrapping_sub(1);
        session_slot!(rt, Gunship[part(index, PAD)].base.y_pos = y);
        entrance(rt, index, true);
    }
}

/// Eject Samus ($A2:A950), lower ($A2:AA5D) or raise ($A2:AB6E) her through the pad.
fn move_player(rt: &mut impl ActorLease, index: u16, function: u16) {
    let top = part(index, TOP);
    let entering = function == LOWER_SAMUS;
    let offset = if entering { 18 } else { 30u16.wrapping_neg() };
    let target = session_slot!(rt, Gunship[top].step).wrapping_add(offset);
    let delta = match function {
        LOWER_SAMUS => 2,
        EJECT_SAMUS => 1u16.wrapping_neg(),
        _ => 2u16.wrapping_neg(),
    };
    let y = rt.player_y().wrapping_add(delta);
    rt.set_player_y(y);
    if (rt.player_y().wrapping_sub(target) & 0x8000 == 0) == entering {
        let next = match function {
            EJECT_SAMUS => WAIT_TO_CLOSE_AND_SAVE,
            LOWER_SAMUS => WAIT_TO_CLOSE_FOR_ENTRY,
            _ => WAIT_TO_CLOSE_FOR_EXIT,
        };
        session_slot!(rt, Gunship[top].function = next);
        entrance(rt, index, false);
    }
}

/// Samus entered - go to liftoff or restore Samus health / ammo ($A2:AAA2).
fn refill(rt: &mut impl ActorLease, index: u16) {
    let top = part(index, TOP);
    if rt.event_happened(0xe) {
        session_slot!(rt, Gunship[top].function = LOAD_DUST_CLOUD_TILES);
        session_slot!(rt, Gunship[part(index, BOTTOM_PART)]; function = 0, step = 0);
        rt.write_slice_le(W::SUBSTATE, 0);
        rt.write_slice_le(W::SUIT_PICKUP_LIGHT_BEAM_POS, 0);
        for at in UNREAD {
            rt.write_slice_native(at, 0);
        }
        rt.foreign(|game| game.command_player(0xa));
        return;
    }
    for resource in [
        Resource::Health,
        Resource::Missiles,
        Resource::SuperMissiles,
        Resource::PowerBombs,
    ] {
        rt.restore(resource, 2);
    }
    let inventory = &rt.session().inventory;
    let at_capacity = |current: u16, maximum: u16| current.wrapping_sub(maximum) & 0x8000 == 0;
    let full = at_capacity(inventory.reserve_health, inventory.reserve_capacity)
        && at_capacity(inventory.health.current, inventory.health.maximum)
        && inventory
            .ammunition
            .iter()
            .all(|supply| at_capacity(supply.current, supply.maximum));
    if full {
        session_slot!(rt, Gunship[top].function = SAVE_CONFIRMATION);
    }
}

/// Liftoff - load dust cloud tiles ($A2:ABC7), one transfer a frame.
fn load_dust_clouds(rt: &mut impl ActorLease, index: u16) {
    let row = usize::from(rt.word(W::SUBSTATE) >> 1).cast_signed();
    let address = table(rt, DUST_SOURCES, row);
    let vram_address = table(rt, DUST_DESTINATIONS, row);
    let source = Source {
        address,
        bank: 0x94,
    };
    let upload = TilemapUpload {
        bytes: 1024,
        source,
        column: vram_address & 0x8000 != 0,
        vram_address,
    };
    queue_tilemap(rt, upload);
    let substate = rt.word(W::SUBSTATE).wrapping_add(2);
    rt.write_slice_le(W::SUBSTATE, substate);
    if rt.word(W::SUBSTATE).wrapping_sub(10) & 0x8000 == 0 {
        session_slot!(rt, Gunship[part(index, TOP)].function = FIRE_UP_ENGINES);
        rt.write_slice_le(W::SUBSTATE, 0);
    }
}

/// The three parts follow Samus.
fn follow_player(rt: &mut impl ActorLease, index: u16) {
    let y = rt.player_y();
    session_slot!(
        rt,
        Gunship[part(index, TOP)].base.y_pos = y.wrapping_sub(17)
    );
    session_slot!(
        rt,
        Gunship[part(index, PAD)].base.y_pos = y.wrapping_sub(18)
    );
    session_slot!(
        rt,
        Gunship[part(index, BOTTOM_PART)].base.y_pos = y.wrapping_add(23)
    );
}

/// Liftoff - fire up engines and spawn dust clouds ($A2:AC1B): the ship shakes.
fn fire_up_engines(rt: &mut impl ActorLease, index: u16) {
    let bottom = part(index, BOTTOM_PART);
    let counter = session_slot!(rt, Gunship[bottom].step);
    let amount = if counter.wrapping_sub(64) & 0x8000 != 0 {
        1u16
    } else {
        2
    };
    let shake = if counter & 1 != 0 {
        amount
    } else {
        amount.wrapping_neg()
    };
    let y = rt.player_y().wrapping_add(shake);
    rt.set_player_y(y);
    rt.set_previous_player_y(y);
    follow_player(rt, index);
    let counter = session_slot!(rt, Gunship[bottom].step).wrapping_add(1);
    session_slot!(rt, Gunship[bottom].step = counter);
    if counter.wrapping_sub(128) & 0x8000 == 0 {
        session_slot!(rt, Gunship[part(index, TOP)].function = STEADY_RISE);
        session_slot!(rt, Gunship[0].entrance_timer = 0);
    } else if counter == 64 {
        for parameter in (0..12).step_by(2) {
            rt.foreign(|game| game.spawn_room_projectile(0xa379, parameter));
        }
    }
}

/// Liftoff - accelerating ($A2:AD2D): the bottom's speed grows to a cap.
fn accelerate(rt: &mut impl ActorLease, index: u16) {
    let bottom = part(index, BOTTOM_PART);
    let speed = session_slot!(rt, Gunship[bottom].function).wrapping_add(64);
    session_slot!(rt, Gunship[bottom].function = speed);
    if (speed & 0xff00).wrapping_sub(2560) & 0x8000 == 0 {
        session_slot!(rt, Gunship[bottom].function = 2304);
    }
    let speed = i32::from(session_slot!(rt, Gunship[bottom].function));
    advance_player(rt, -(speed << 8));
    follow_player(rt, index);
}

/// Gunship function `function` ($A2:A80C-$A2:AD2D) for the gunship at `index`.
pub fn run(rt: &mut impl ActorLease, index: u16, function: u16) {
    let top = part(index, TOP);
    match function {
        DESCENDING => descend(rt, index),
        APPLY_BRAKES => brake(rt, index),
        WAIT_TO_OPEN_FOR_EJECT
        | WAIT_TO_CLOSE_AND_SAVE
        | WAIT_TO_OPEN_FOR_ENTRY
        | WAIT_TO_CLOSE_FOR_ENTRY
        | WAIT_TO_OPEN_FOR_EXIT
        | WAIT_TO_CLOSE_FOR_EXIT => {
            let (after, expired) = tick(session_slot!(rt, Gunship[0].entrance_timer));
            session_slot!(rt, Gunship[0].entrance_timer = after);
            if !expired {
                return;
            }
            let next = match function {
                WAIT_TO_OPEN_FOR_EJECT => EJECT_SAMUS,
                WAIT_TO_OPEN_FOR_ENTRY => LOWER_SAMUS,
                WAIT_TO_CLOSE_FOR_ENTRY => ENTERED,
                WAIT_TO_OPEN_FOR_EXIT => RAISE_SAMUS,
                _ => IDLE,
            };
            session_slot!(rt, Gunship[top].function = next);
            if function == WAIT_TO_CLOSE_AND_SAVE {
                unlock_player(rt);
                rt.session_mut()
                    .menu_screens
                    .save_selection
                    .loading_game_state = 5;
                save(rt);
            } else if function == WAIT_TO_CLOSE_FOR_EXIT
                && rt.word(W::GAME_STATE).wrapping_sub(40) & 0x8000 != 0
            {
                unlock_player(rt);
            }
        }
        EJECT_SAMUS | LOWER_SAMUS | RAISE_SAMUS => move_player(rt, index, function),
        IDLE => idle(rt, index),
        ENTERED => refill(rt, index),
        SAVE_CONFIRMATION => {
            let response = rt.message_box_response(0x1c);
            if response < 0 {
                return;
            }
            if response != 2 {
                save(rt);
            }
            session_slot!(rt, Gunship[top].function = WAIT_TO_OPEN_FOR_EXIT);
            entrance(rt, index, true);
        }
        LOAD_DUST_CLOUD_TILES => load_dust_clouds(rt, index),
        FIRE_UP_ENGINES => fire_up_engines(rt, index),
        STEADY_RISE => {
            let y = rt.player_y().wrapping_sub(2);
            rt.set_player_y(y);
            follow_player(rt, index);
            if session_slot!(rt, Gunship[top].base.y_pos).wrapping_sub(896) & 0x8000 != 0 {
                session_slot!(rt, Gunship[top].function = ACCELERATING_SET_GAME_STATE);
                session_slot!(rt, Gunship[part(index, BOTTOM_PART)].function = 512);
            }
        }
        ACCELERATING_SET_GAME_STATE | ACCELERATING => {
            accelerate(rt, index);
            let rising = session_slot!(rt, Gunship[top].base.y_pos).wrapping_sub(256) & 0x8000 != 0;
            if function == ACCELERATING_SET_GAME_STATE && rising {
                session_slot!(rt, Gunship[top].function = ACCELERATING);
                rt.write_slice_le(W::GAME_STATE, 38);
                rt.write_slice_le(W::SCREEN_FADE_DELAY, 0);
                rt.write_slice_le(W::SCREEN_FADE_COUNTER, 0);
            }
        }
        _ => unreachable!("only registered gunship functions enter this module"),
    }
}
