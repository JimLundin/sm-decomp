//! The names the vector file gives each value, as the spec writes them.

use clean_model_progress::{MapStatus, ReserveMode, Supply};
use clean_types::{Area, Beam, BossKind, Event, Item};

/// The value `name` stands for in `table`, if any.
pub fn lookup<T: Copy>(table: &[(&str, T)], name: &str) -> Option<T> {
    table
        .iter()
        .find(|(n, _)| *n == name)
        .map(|&(_, value)| value)
}

/// The value `name` stands for in `table`; any other name stops the test.
pub fn named<T: Copy>(table: &[(&str, T)], name: &str) -> T {
    lookup(table, name).unwrap_or_else(|| panic!("unknown name {name:?}"))
}

pub const AREAS: [(&str, Area); 7] = [
    ("Brinstar", Area::Brinstar),
    ("Ceres", Area::Ceres),
    ("Crateria", Area::Crateria),
    ("Maridia", Area::Maridia),
    ("Norfair", Area::Norfair),
    ("Tourian", Area::Tourian),
    ("WreckedShip", Area::WreckedShip),
];

pub const ITEMS: [(&str, Item); 11] = [
    ("Bombs", Item::Bombs),
    ("GrappleBeam", Item::GrappleBeam),
    ("GravitySuit", Item::GravitySuit),
    ("HiJumpBoots", Item::HiJumpBoots),
    ("MorphBall", Item::MorphBall),
    ("ScrewAttack", Item::ScrewAttack),
    ("SpaceJump", Item::SpaceJump),
    ("SpeedBooster", Item::SpeedBooster),
    ("SpringBall", Item::SpringBall),
    ("VariaSuit", Item::VariaSuit),
    ("XRayScope", Item::XRayScope),
];

pub const BEAMS: [(&str, Beam); 5] = [
    ("Charge", Beam::Charge),
    ("Ice", Beam::Ice),
    ("Plasma", Beam::Plasma),
    ("Spazer", Beam::Spazer),
    ("Wave", Beam::Wave),
];

pub const TANKS: [(&str, Supply); 5] = [
    ("EnergyTank", Supply::Energy),
    ("ReserveTank", Supply::Reserve),
    ("MissileTank", Supply::Missiles),
    ("SuperMissileTank", Supply::SuperMissiles),
    ("PowerBombTank", Supply::PowerBombs),
];

pub const SUPPLIES: [(&str, Supply); 5] = [
    ("energy", Supply::Energy),
    ("reserve", Supply::Reserve),
    ("missiles", Supply::Missiles),
    ("super_missiles", Supply::SuperMissiles),
    ("power_bombs", Supply::PowerBombs),
];

pub const HELD: [(&str, Supply); 5] = [
    ("energy", Supply::Energy),
    ("reserve_energy", Supply::Reserve),
    ("missiles", Supply::Missiles),
    ("super_missiles", Supply::SuperMissiles),
    ("power_bombs", Supply::PowerBombs),
];

pub const EVENTS: [(&str, Event); 18] = [
    ("AcidReceded", Event::AcidReceded),
    ("AnimalsFreed", Event::AnimalsFreed),
    ("DraygonStatueGrey", Event::DraygonStatueGrey),
    ("GlassTunnelBroken", Event::GlassTunnelBroken),
    ("KraidStatueGrey", Event::KraidStatueGrey),
    ("LavaQuakeOutrun", Event::LavaQuakeOutrun),
    ("MetroidRoom1Empty", Event::MetroidRoom1Empty),
    ("MetroidRoom2Empty", Event::MetroidRoom2Empty),
    ("MetroidRoom3Empty", Event::MetroidRoom3Empty),
    ("MetroidRoom4Empty", Event::MetroidRoom4Empty),
    ("MotherBrainTankBroken", Event::MotherBrainTankBroken),
    ("PhantoonStatueGrey", Event::PhantoonStatueGrey),
    ("PlanetAwake", Event::PlanetAwake),
    ("RidleyStatueGrey", Event::RidleyStatueGrey),
    ("ShaktoolPath", Event::ShaktoolPath),
    ("TimeBombSet", Event::TimeBombSet),
    ("TourianOpen", Event::TourianOpen),
    ("TourianRoomsUnset", Event::TourianRoomsUnset),
];

pub const BOSS_KINDS: [(&str, BossKind); 3] = [
    ("MainBoss", BossKind::MainBoss),
    ("MiniBoss", BossKind::MiniBoss),
    ("Torizo", BossKind::Torizo),
];

pub const RESERVE_MODES: [(&str, ReserveMode); 3] = [
    ("None", ReserveMode::NoTank),
    ("Automatic", ReserveMode::Automatic),
    ("Manual", ReserveMode::Manual),
];

pub const MAPS: [(&str, MapStatus); 2] = [
    ("Downloaded", MapStatus::Downloaded),
    ("OnArrival", MapStatus::OnArrival),
];
