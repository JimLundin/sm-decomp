//! Room mechanisms: owned execution, creation, rules, scripts and presentation.
mod commands;
mod creation;
mod effects;
mod fields;
mod fields_runtime;
mod graphics;
mod native_content;
mod port;
mod pre;
mod presentation;
mod scheduler;
mod setup;
mod state;
mod rules {
    pub(super) mod hands;
    pub(super) mod stations;
    pub(super) mod torizo_statue;

    pub(super) mod eye_door;
    pub(super) mod gates;
    pub(super) mod mother_brain_glass;
    pub(super) mod noob_tube;
    pub(super) mod treadmill;

    pub(super) mod animal_rescue;
    pub(super) mod booster_escape;
    pub(super) mod crumbling_wall;
    pub(super) mod metroid_quota;
    pub(super) mod shaktool;
    pub(super) mod sound_delay;

    pub(super) mod cannon_setup;
    pub(super) mod contact_physics;
    pub(super) mod doors;
    pub(super) mod item_setup;
    pub(super) mod plants;
    pub(super) mod scroll_setup;
    pub(super) mod terrain_setup;
    pub(super) mod tourian_wall;
}

use crate::game::mechanism_displacement::Displacement;
use crate::game_state::Game;
use crate::types::SpawnHardcodedPlmArgs;
use fields::{Fields, Global};
use port::Execution;
pub(crate) use rules::treadmill::Direction;
use state::Column;

/// Spawn a mechanism at a hardcoded room block. Native setup effects require initialized runtime state.
pub(crate) unsafe fn spawn_hardcoded(args: SpawnHardcodedPlmArgs, game: &mut Game) {
    Execution::new(game).spawn(
        creation::Source::Hardcoded {
            x: args.block_x,
            y: args.block_y,
            header: args.header,
        },
        &mut Displacement::Absent,
    );
}
pub(crate) unsafe fn spawn_room(address: u16, game: &mut Game) {
    Execution::new(game).spawn(
        creation::Source::Room { address },
        &mut Displacement::Absent,
    );
}
pub(crate) unsafe fn spawn_collision(header: u16, game: &mut Game) -> u8 {
    Execution::new(game).spawn(
        creation::Source::Collision { header },
        &mut Displacement::Absent,
    )
}
pub(crate) unsafe fn spawn_contact(
    header: u16,
    displacement: &mut Displacement<'_>,
    game: &mut Game,
) -> u8 {
    Execution::new(game).spawn(creation::Source::Collision { header }, displacement)
}
pub(crate) unsafe fn close_door(game: &mut Game) {
    Execution::new(game).close_door();
}
pub(crate) unsafe fn reload_graphics(game: &mut Game) {
    Execution::new(game).reload_graphics();
}
pub(crate) unsafe fn run(game: &mut Game) -> u8 {
    unsafe { Execution::new(game).run() }
}
pub(crate) fn set_enabled(enabled: bool, game: &mut Game) {
    let mut e = Execution::new(game);
    let flags = e.port().word(0x1c23);
    e.port_mut().put_word(
        0x1c23,
        if enabled {
            flags | 0x8000
        } else {
            flags & !0x8000
        },
    );
}
pub(crate) fn clear(game: &mut Game) {
    let mut e = Execution::new(game);
    for slot in (0..40).rev() {
        e.release(slot);
    }
    e.port_mut().put_word(0x1c2d, 0);
}
/// Publish the coordinate side effect before the caller's later field reads.
pub(crate) fn calculate_coordinates(k: u16, game: &mut Game) {
    Execution::new(game).coordinates(usize::from(k >> 1));
}
pub(crate) fn selected(game: &mut Game) -> u16 {
    Execution::new(game).global(Global::Selected)
}
pub(crate) fn block_index(k: u16, game: &mut Game) -> u16 {
    Execution::new(game).slot(Column::Block, usize::from(k >> 1))
}
pub(crate) fn projectile_argument(k: u16, game: &mut Game) -> u16 {
    Execution::new(game)
        .port()
        .native_word(0x1dc7 + usize::from(k >> 1) * 2)
}
pub(crate) fn block_x(game: &mut Game) -> u16 {
    Execution::new(game).global(Global::BlockX)
}
pub(crate) fn block_y(game: &mut Game) -> u16 {
    Execution::new(game).global(Global::BlockY)
}
pub(crate) fn carry(direction: Direction, game: &mut Game) {
    rules::treadmill::carry(&mut Execution::new(game), direction);
}
pub(crate) fn add_glass_hits(slot: usize, amount: u16, game: &mut Game) {
    rules::mother_brain_glass::add_hits(&mut Execution::new(game), slot, amount);
}
pub(crate) fn lower_acid(game: &mut Game) {
    rules::hands::lower_acid(&mut Execution::new(game));
}
pub(crate) fn open_passage(game: &mut Game) {
    rules::crumbling_wall::open_passage(&mut Execution::new(game));
}
pub(crate) fn statue_present(game: &mut Game) -> bool {
    rules::torizo_statue::present(&Execution::new(game))
}
/// The actor becomes tangible before scanning, and music precedes its live resume reads.
pub(crate) fn wait_statue(index: u16, game: &mut Game) {
    let mut e = Execution::new(game);
    let actor = 0xf78 + usize::from(index);
    let properties = actor + core::mem::offset_of!(crate::types::EnemyData, properties);
    let value = e.port().native_word(properties) | 0x400;
    e.port_mut().put_native_word(properties, value);
    if rules::torizo_statue::present(&e) {
        return;
    }
    e.music(6);
    let value = e.port().native_word(properties) & !0x400;
    e.port_mut().put_native_word(properties, value);
    let instruction = actor + core::mem::offset_of!(crate::types::EnemyData, current_instruction);
    let value = e.port().native_word(instruction).wrapping_add(2);
    e.port_mut().put_native_word(instruction, value);
    e.port_mut().put_native_word(
        actor + core::mem::offset_of!(crate::types::EnemyData, instruction_timer),
        1,
    );
}
/// Reveal only the mechanism items. Room-special-case reveal stays with Xray.
/// Native item draw records require the existing initialized ROM allocation.
pub(crate) unsafe fn reveal_items(game: &mut Game) {
    let mut e = Execution::new(game);
    for slot in (0..40).rev() {
        if e.slot(Column::Header, slot) < 0xdf89 {
            continue;
        }
        let argument = e.slot(Column::RoomArgument, slot);
        let index = e.foreign(|g| crate::game::progress_flags::PrepareBitAccess(argument, g));
        let item = u16::from(e.port().byte(0xd870 + usize::from(index)));
        if e.port().word(0x5e7) & item != 0 {
            continue;
        }
        e.coordinates(slot);
        let graphic = e.port().native_word(0xdf0c + slot * 2) >> 1;
        let address = e.port().rom_image().table_word(
            crate::rtl::lorom_offset(0x84839d) as usize,
            isize::try_from(graphic).unwrap(),
        );
        let sequence = unsafe { native_content::sequence(&mut e, 0x84, address) };
        let block = unsafe { sequence.word(2) } & 0xfff;
        let x = e.global(Global::BlockX);
        let y = e.global(Global::BlockY);
        e.foreign(|g| unsafe { crate::game::xray_background::reveal_room_block(block, x, y, g) });
    }
}
/// Numeric identities used by the existing verification carrier.
pub(crate) mod checks {
    use super::*;
    use crate::game::enemy_script::EnemyScript;
    pub(crate) fn setup_entry(
        original: u32,
        k: u16,
        displacement: &mut Displacement<'_>,
        accumulator: u16,
        game: &mut Game,
    ) -> u8 {
        Execution::new(game).setup(original, usize::from(k >> 1), displacement, accumulator)
    }
    pub(crate) fn pre(ea: u32, k: u16, game: &mut Game) {
        Execution::new(game).pre(ea, k);
    }
    pub(crate) unsafe fn command(ea: u32, p: EnemyScript, k: u16, game: &mut Game) -> EnemyScript {
        unsafe { Execution::new(game).command(ea, p, k) }
    }
    pub(crate) fn process_draw(k: u16, game: &mut Game) {
        Execution::new(game).process_draw(k);
    }
    pub(crate) unsafe fn draw(k: u16, game: &mut Game) {
        unsafe {
            Execution::new(game).draw(k);
        }
    }
}

mod ownership_checks;
pub(crate) fn ownership_checks(game: &mut Game) -> bool {
    ownership_checks::run(game)
}

mod qualified_checks;
pub(crate) fn qualified_checks(game: &mut Game) -> bool {
    qualified_checks::run(game)
}
