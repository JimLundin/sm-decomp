//! Checked setup execution shared by encoded callers; effects remain live in RAM.
#![forbid(unsafe_code)]
use crate::game::mechanism_setup::{Contact, Diagnostic, Setup};
use crate::game_state::Game;
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum Boundary<X> {
    Contact(Contact),
    Diagnostic(Diagnostic),
    External(X),
}
/// A boundary return precedes every Game access and leaves the caller's state intact.
pub fn execute<X>(
    action: Setup<X>,
    slot: usize,
    accumulator: u16,
    game: &mut Game,
) -> Result<u8, Boundary<X>> {
    let result = match action {
        Setup::Contact(contact) => return Err(Boundary::Contact(contact)),
        Setup::Diagnostic(kind) => return Err(Boundary::Diagnostic(kind)),
        Setup::External(code) => return Err(Boundary::External(code)),
        Setup::Noop => 0,
        Setup::Return(value) => value,
        Setup::Door(kind) => {
            crate::game::mechanism_doors::setup(
                &mut crate::game::mechanism_doors_ram::DoorFields(game),
                slot,
                kind,
            );
            0
        }
        Setup::Terrain(kind) => crate::game::mechanism_terrain_setup::setup(
            &mut crate::game::mechanism_terrain_setup_ram::TerrainFields(game),
            slot,
            kind,
        ),
        Setup::Gate(kind) => crate::game::mechanism_gates::setup(
            &mut crate::game::mechanism_gates_ram::GateFields(game),
            slot,
            kind,
            accumulator,
        ),
        Setup::Station(kind) => crate::game::mechanism_stations::setup(
            &mut crate::game::mechanism_stations_ram::StationFields {
                game,
                effects: crate::game::mechanism_stations_ram::RuntimeEffects,
            },
            slot,
            kind,
        ),
        Setup::Scroll(kind) => crate::game::mechanism_scroll_setup::setup(
            &mut crate::game::mechanism_scroll_setup_ram::ScrollFields(game),
            slot,
            kind,
        ),
        Setup::Item(kind) => crate::game::mechanism_item_setup::setup(
            &mut crate::game::mechanism_item_setup_ram::ItemFields(game),
            slot,
            kind,
        ),
        Setup::Cannon(kind) => crate::game::mechanism_cannon_setup::setup(
            &mut crate::game::mechanism_cannon_setup_ram::CannonFields(game),
            slot,
            kind,
        ),
        Setup::Plant(kind) => crate::game::mechanism_plants::begin(
            &mut crate::game::mechanism_plants_ram::PlantFields(game),
            slot,
            kind,
        ),
        Setup::TourianWall(kind) => crate::game::mechanism_tourian_wall::setup(
            &mut crate::game::mechanism_tourian_wall_ram::WallFields(game),
            slot,
            kind,
        ),
        Setup::AnimalRescue(kind) => crate::game::mechanism_animal_rescue::setup(
            &mut crate::game::mechanism_animal_rescue_ram::RescueFields(game),
            slot,
            kind,
        ),
        Setup::TubeTerrain(kind) => crate::game::mechanism_noob_tube::setup(
            &mut crate::game::mechanism_noob_tube_ram::TubeFields(game),
            slot,
            kind,
        ),
        Setup::TorizoStatue => crate::game::mechanism_torizo_statue::setup(
            &mut crate::game::mechanism_torizo_statue_ram::StatueFields(game),
            slot,
        ),
        Setup::MotherBrainGlass => crate::game::mechanism_mother_brain_glass::setup(
            &mut crate::game::mechanism_mother_brain_glass_ram::GlassFields(game),
            slot,
        ),
        Setup::CrumblingWall => crate::game::mechanism_crumbling_wall::setup(
            &mut crate::game::mechanism_crumbling_wall_ram::WallFields(game),
            slot,
        ),
        Setup::HandTrigger => crate::game::mechanism_hands_ram::prepare_ship_trigger(
            (slot as u16).wrapping_mul(2),
            game,
        ),
        Setup::PeriodicDamage => crate::game::samus_periodic_damage::grapple_contact(
            &mut crate::game::samus_periodic_damage_ram::DamageFields(game),
        ),
        Setup::Treadmill => crate::game::mechanism_treadmill::setup(
            &mut crate::game::mechanism_treadmill_ram::TreadmillFields(game),
            slot,
        ),
        Setup::Shaktool => crate::game::mechanism_shaktool::begin(
            &mut crate::game::mechanism_shaktool_ram::ShaktoolFields(game),
        ),
        Setup::BoosterEscape => crate::game::mechanism_booster_escape::begin(
            &mut crate::game::mechanism_booster_escape_ram::EscapeFields(game),
            slot,
        ),
        Setup::SoundDelay => crate::game::mechanism_sound_delay::begin(
            &mut crate::game::mechanism_sound_delay_ram::SoundDelayFields(game),
            slot,
        ),
        Setup::MetroidQuota => crate::game::mechanism_metroid_quota::begin(
            &mut crate::game::mechanism_metroid_quota_ram::QuotaFields(game),
            slot,
        ),
        Setup::EyeDoor(kind) => crate::game::mechanism_eye_door::begin(
            &mut crate::game::mechanism_eye_door_ram::EyeDoorFields(game),
            slot,
            kind,
        ),
        Setup::NoobTube => {
            return execute(
                Setup::<X>::TubeTerrain(crate::game::mechanism_noob_tube::Setup::Solid),
                slot,
                accumulator,
                game,
            );
        }
    };
    Ok(result)
}
