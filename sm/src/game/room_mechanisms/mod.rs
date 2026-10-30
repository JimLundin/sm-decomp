//! Complete room mechanisms on the shared gameplay Session.
use sm_game::room_mechanisms::commands;
use sm_game::room_mechanisms::content;
mod content_runtime;
pub(crate) use sm_game::room_mechanisms::creation;
use sm_game::room_mechanisms::graphics;
mod graphics_runtime;
pub(crate) use sm_game::room_mechanisms::host;
use sm_game::room_mechanisms::pre;
use sm_game::room_mechanisms::presentation;
mod runtime;
use sm_game::room_mechanisms::setup;
mod rules {
    pub(super) use sm_game::room_mechanisms::rules::crumbling_wall;
    pub(super) use sm_game::room_mechanisms::rules::hands;
    pub(super) use sm_game::room_mechanisms::rules::mother_brain_glass;
    pub(super) use sm_game::room_mechanisms::rules::torizo_statue;
    pub(super) use sm_game::room_mechanisms::rules::treadmill;
}
use crate::game::{
    mechanism_displacement::Displacement,
    session_runtime::{self, Runtime},
};
use crate::{game_state::Game, types::SpawnHardcodedPlmArgs};
use host::{Contact, ContactIdentity, SpawnSource};
pub(crate) use rules::treadmill::{Direction, carry};

pub(crate) fn spawn_hardcoded(args: SpawnHardcodedPlmArgs, game: &mut Game) {
    creation::spawn(
        &mut Runtime::new(game),
        SpawnSource::Hardcoded {
            x: args.block_x,
            y: args.block_y,
            header: args.header,
        },
        &mut Contact::Absent,
    );
}
pub(crate) unsafe fn spawn_room(address: u16, game: &mut Game) {
    creation::spawn(
        &mut Runtime::new(game),
        SpawnSource::Room {
            entry: host::ProgramCursor(address),
        },
        &mut Contact::Absent,
    );
}
pub(crate) unsafe fn spawn_collision(header: u16, game: &mut Game) -> u8 {
    creation::spawn(
        &mut Runtime::new(game),
        SpawnSource::Collision { header },
        &mut Contact::Absent,
    )
}
fn contact<'a>(value: &'a mut Displacement<'_>) -> Contact<'a> {
    match value {
        Displacement::Absent => Contact::Absent,
        Displacement::Local(value) => Contact::Local(&mut **value),
        Displacement::RamOffset(at) => Contact::Imported(ContactIdentity(*at)),
    }
}
pub(crate) unsafe fn close_door(game: &mut Game) {
    creation::close_door(&mut Runtime::new(game));
}
pub(crate) unsafe fn reload_graphics(game: &mut Game) {
    graphics::reload_graphics(&mut Runtime::new(game));
}
pub(crate) unsafe fn run(game: &mut Game) -> u8 {
    session_runtime::run_mechanisms(game, host::CallerContext::Gameplay)
}
pub(crate) unsafe fn run_door(game: &mut Game) -> u8 {
    session_runtime::run_mechanisms(game, host::CallerContext::DoorTransition)
}
pub(crate) fn set_enabled(enabled: bool, game: &mut Game) {
    let mut h = Runtime::new(game);
    let flags = h.session().mechanisms.flags;
    h.session_mut().mechanisms.flags = if enabled {
        flags | 0x8000
    } else {
        flags & !0x8000
    };
}
pub(crate) fn clear(game: &mut Game) {
    clear_session(Runtime::new(game).session_mut());
}
pub(crate) use creation::clear_session;
pub(crate) fn calculate_coordinates(k: u16, game: &mut Game) {
    creation::coordinates(&mut Runtime::new(game), usize::from(k >> 1));
}
pub(crate) fn add_glass_hits(slot: usize, amount: u16, game: &mut Game) {
    rules::mother_brain_glass::add_hits(&mut Runtime::new(game), slot, amount);
}
pub(crate) fn lower_acid(game: &mut Game) {
    rules::hands::lower_acid(&mut Runtime::new(game));
}
pub(crate) fn open_passage(game: &mut Game) {
    rules::crumbling_wall::open_passage(&mut Runtime::new(game));
}
pub(crate) fn wait_statue(index: u16, game: &mut Game) {
    use host::Effects;
    let mut h = Runtime::new(game);
    h.statue_wait_actor(index, true);
    if rules::torizo_statue::present(&h) {
        return;
    }
    h.music(6);
    h.statue_wait_actor(index, false);
}
pub(crate) unsafe fn reveal_items(game: &mut Game) {
    graphics::reveal_items(&mut Runtime::new(game));
}

/// Numeric identities retained only for the actual-old verification carrier.
pub(crate) mod checks {
    use super::*;
    use crate::{
        game::enemy_script::{EnemyScript, bank_base},
        rom::RomRecord,
    };
    pub(crate) fn setup_entry(
        original: u32,
        k: u16,
        displacement: &mut Displacement<'_>,
        accumulator: u16,
        game: &mut Game,
    ) -> u8 {
        setup::setup(
            &mut Runtime::new(game),
            original,
            usize::from(k >> 1),
            &mut contact(displacement),
            accumulator,
        )
    }
    pub(crate) fn pre(ea: u32, k: u16, game: &mut Game) {
        super::pre::run(&mut Runtime::new(game), ea, k);
    }
    /// # Safety
    /// The native runtime and ROM binding must be initialized for legacy effects
    /// reached by this command; owned script bounds alone do not establish that.
    pub(crate) unsafe fn command(ea: u32, p: EnemyScript, k: u16, game: &mut Game) -> EnemyScript {
        let EnemyScript::At { image, position } = p else {
            panic!("mechanism command needs a readable script")
        };
        let anchor = usize::try_from(bank_base(0x84) + 0x8000).unwrap();
        let displacement = isize::try_from(position).unwrap() - isize::try_from(anchor).unwrap();
        let p = content::Script::new(
            RomRecord {
                image,
                base: anchor,
            },
            0x8000,
        )
        .advance(displacement);
        match commands::run(&mut Runtime::new(game), ea, p, k) {
            None => EnemyScript::Stop,
            Some(next) => {
                if let Some(target) = next.jump_target() {
                    return EnemyScript::Jump(target);
                }
                let (record, offset) = next.into_parts();
                EnemyScript::at(
                    record.image,
                    record
                        .base
                        .checked_add_signed(offset)
                        .expect("mechanism script offset overflow"),
                )
            }
        }
    }
    pub(crate) fn process_draw(k: u16, game: &mut Game) {
        presentation::process_draw(&mut Runtime::new(game), k);
    }
    pub(crate) unsafe fn draw(k: u16, game: &mut Game) {
        presentation::draw(&mut Runtime::new(game), k);
    }
}
#[path = "../../verification/room_mechanism_ownership.rs"]
mod ownership_checks;
pub(crate) fn ownership_checks(game: &mut Game) -> bool {
    ownership_checks::run(game)
}
#[path = "../../verification/room_mechanism_qualified.rs"]
mod qualified_checks;
pub(crate) fn qualified_checks(game: &mut Game) -> bool {
    qualified_checks::run(game)
}
