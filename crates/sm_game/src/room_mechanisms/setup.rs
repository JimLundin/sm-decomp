//! Semantic setup actions and exact encoded dispatch identity.
#![forbid(unsafe_code)]
use super::addresses;
use super::rules::gates::{Color, Direction, Side};
#[derive(Clone, Copy, Debug, PartialEq)]
enum Contact {
    Physics(super::rules::contact_physics::Setup),
    Hand(super::rules::hands::Hand),
}
#[derive(Clone, Copy, Debug, PartialEq)]
enum Diagnostic {
    UnsupportedB3E3,
}
#[derive(Clone, Copy, Debug, PartialEq)]
enum Setup<X> {
    Return(u8),
    Diagnostic(Diagnostic),
    Door(super::rules::doors::Setup),
    Terrain(super::rules::terrain_setup::Setup),
    Gate(super::rules::gates::Setup),
    Station(super::rules::stations::Setup),
    Scroll(super::rules::scroll_setup::Setup),
    Item(super::rules::item_setup::Setup),
    Cannon(super::rules::cannon_setup::Setup),
    Plant(super::rules::plants::Surface),
    EyeDoor(super::rules::eye_door::Part),
    MetroidQuota,
    SoundDelay,
    MotherBrainGlass,
    TorizoStatue,
    NoobTube,
    TubeTerrain(super::rules::noob_tube::Setup),
    CrumblingWall,
    Treadmill,
    PeriodicDamage,
    HandTrigger,
    BoosterEscape,
    Shaktool,
    AnimalRescue(super::rules::animal_rescue::Setup),
    TourianWall(super::rules::tourian_wall::Setup),
    Contact(Contact),
    External(X),
}
fn decode(original: u32) -> Setup<u32> {
    match original {
        addresses::setup::SETUP_ITEM_EE4D => {
            Setup::Item(super::rules::item_setup::Setup::Initialize(
                super::rules::item_setup::Container::Visible,
                Some(super::rules::item_setup::Graphic::EnergyTank),
            ))
        }
        addresses::setup::SETUP_ITEM_EE52 => {
            Setup::Item(super::rules::item_setup::Setup::Initialize(
                super::rules::item_setup::Container::Visible,
                Some(super::rules::item_setup::Graphic::MissileTank),
            ))
        }
        addresses::setup::SETUP_ITEM_EE57 => {
            Setup::Item(super::rules::item_setup::Setup::Initialize(
                super::rules::item_setup::Container::Visible,
                Some(super::rules::item_setup::Graphic::SuperMissileTank),
            ))
        }
        addresses::setup::SETUP_ITEM_EE5C => {
            Setup::Item(super::rules::item_setup::Setup::Initialize(
                super::rules::item_setup::Container::Visible,
                Some(super::rules::item_setup::Graphic::PowerBombTank),
            ))
        }
        addresses::setup::SETUP_ITEM_EE64 => {
            Setup::Item(super::rules::item_setup::Setup::Initialize(
                super::rules::item_setup::Container::Visible,
                None,
            ))
        }
        addresses::setup::SETUP_ITEM_EE77 => {
            Setup::Item(super::rules::item_setup::Setup::Initialize(
                super::rules::item_setup::Container::ShotBlock,
                Some(super::rules::item_setup::Graphic::EnergyTank),
            ))
        }
        addresses::setup::SETUP_ITEM_EE7C => {
            Setup::Item(super::rules::item_setup::Setup::Initialize(
                super::rules::item_setup::Container::ShotBlock,
                Some(super::rules::item_setup::Graphic::MissileTank),
            ))
        }
        addresses::setup::SETUP_ITEM_EE81 => {
            Setup::Item(super::rules::item_setup::Setup::Initialize(
                super::rules::item_setup::Container::ShotBlock,
                Some(super::rules::item_setup::Graphic::SuperMissileTank),
            ))
        }
        addresses::setup::SETUP_ITEM_EE86 => {
            Setup::Item(super::rules::item_setup::Setup::Initialize(
                super::rules::item_setup::Container::ShotBlock,
                Some(super::rules::item_setup::Graphic::PowerBombTank),
            ))
        }
        addresses::setup::SETUP_ITEM_EE8E => {
            Setup::Item(super::rules::item_setup::Setup::Initialize(
                super::rules::item_setup::Container::ShotBlock,
                None,
            ))
        }
        addresses::setup::ITEM_TRIGGER => Setup::Item(super::rules::item_setup::Setup::Trigger),
        addresses::setup::SETUP_SCROLL_B33A => Setup::Scroll(
            super::rules::scroll_setup::Setup::Place(super::rules::scroll_setup::Tile::Extension(
                super::rules::scroll_setup::Direction::Right,
            )),
        ),
        addresses::setup::SETUP_SCROLL_B345 => Setup::Scroll(
            super::rules::scroll_setup::Setup::Place(super::rules::scroll_setup::Tile::Extension(
                super::rules::scroll_setup::Direction::Left,
            )),
        ),
        addresses::setup::SETUP_SCROLL_B350 => Setup::Scroll(
            super::rules::scroll_setup::Setup::Place(super::rules::scroll_setup::Tile::Extension(
                super::rules::scroll_setup::Direction::Down,
            )),
        ),
        addresses::setup::SETUP_SCROLL_B35B => Setup::Scroll(
            super::rules::scroll_setup::Setup::Place(super::rules::scroll_setup::Tile::Extension(
                super::rules::scroll_setup::Direction::Up,
            )),
        ),
        addresses::setup::SETUP_SCROLL_B371 => {
            Setup::Scroll(super::rules::scroll_setup::Setup::Place(
                super::rules::scroll_setup::Tile::Scroll(super::rules::scroll_setup::Solidity::Air),
            ))
        }
        addresses::setup::SETUP_SCROLL_B382 => Setup::Scroll(
            super::rules::scroll_setup::Setup::Place(super::rules::scroll_setup::Tile::Scroll(
                super::rules::scroll_setup::Solidity::Solid,
            )),
        ),
        addresses::setup::SCROLL_TOUCH => Setup::Scroll(super::rules::scroll_setup::Setup::Touch),
        addresses::setup::SETUP_STATION_B1C8 => {
            Setup::Station(super::rules::stations::Setup::Access(
                super::rules::stations::Service::Map,
                super::rules::stations::Side::Right,
            ))
        }
        addresses::setup::SETUP_STATION_B1F0 => {
            Setup::Station(super::rules::stations::Setup::Access(
                super::rules::stations::Service::Map,
                super::rules::stations::Side::Left,
            ))
        }
        addresses::setup::SETUP_STATION_B26D => {
            Setup::Station(super::rules::stations::Setup::Access(
                super::rules::stations::Service::Energy,
                super::rules::stations::Side::Right,
            ))
        }
        addresses::setup::SETUP_STATION_B29D => {
            Setup::Station(super::rules::stations::Setup::Access(
                super::rules::stations::Service::Energy,
                super::rules::stations::Side::Left,
            ))
        }
        addresses::setup::SETUP_STATION_B2D0 => {
            Setup::Station(super::rules::stations::Setup::Access(
                super::rules::stations::Service::Missile,
                super::rules::stations::Side::Right,
            ))
        }
        addresses::setup::SETUP_STATION_B300 => {
            Setup::Station(super::rules::stations::Setup::Access(
                super::rules::stations::Service::Missile,
                super::rules::stations::Side::Left,
            ))
        }
        addresses::setup::STATION_SAVE_TRIGGER => {
            Setup::Station(super::rules::stations::Setup::SaveTrigger)
        }
        addresses::setup::STATION_SAVE => Setup::Station(super::rules::stations::Setup::Save),
        addresses::setup::SETUP_STATION_B18B => Setup::Station(
            super::rules::stations::Setup::Place(super::rules::stations::Service::Map),
        ),
        addresses::setup::SETUP_STATION_B21D => Setup::Station(
            super::rules::stations::Setup::Place(super::rules::stations::Service::Energy),
        ),
        addresses::setup::SETUP_STATION_B245 => Setup::Station(
            super::rules::stations::Setup::Place(super::rules::stations::Service::Missile),
        ),
        addresses::setup::CONTACT_ICE => {
            Setup::Contact(Contact::Physics(super::rules::contact_physics::Setup::Ice))
        }
        addresses::setup::SETUP_CONTACT_B408 => Setup::Contact(Contact::Physics(
            super::rules::contact_physics::Setup::Surface,
        )),
        addresses::setup::SETUP_CONTACT_B497 => Setup::Contact(Contact::Physics(
            super::rules::contact_physics::Setup::Submerge,
        )),
        addresses::setup::SETUP_CONTACT_B4A8 => Setup::Contact(Contact::Physics(
            super::rules::contact_physics::Setup::SlowFall,
        )),
        addresses::setup::SETUP_CONTACT_B4B6 => Setup::Contact(Contact::Physics(
            super::rules::contact_physics::Setup::FastFall,
        )),
        addresses::setup::SETUP_CONTACT_B541 => Setup::Contact(Contact::Physics(
            super::rules::contact_physics::Setup::StopVertical,
        )),
        addresses::setup::SETUP_CONTACT_B4C4 => Setup::Contact(Contact::Physics(
            super::rules::contact_physics::Setup::CollisionSurface,
        )),
        addresses::setup::CONTACT_HANDS_LOWER_NORFAIR => {
            Setup::Contact(Contact::Hand(super::rules::hands::Hand::LowerNorfair))
        }
        addresses::setup::CONTACT_HANDS_WRECKED_SHIP => {
            Setup::Contact(Contact::Hand(super::rules::hands::Hand::WreckedShip))
        }
        addresses::setup::DOOR_MOTHER_BRAIN_ESCAPE => {
            Setup::Door(super::rules::doors::Setup::MotherBrainEscape)
        }
        addresses::setup::DOOR_CERES_ESCAPE => Setup::Door(super::rules::doors::Setup::CeresEscape),
        addresses::setup::DOOR_GREY => Setup::Door(super::rules::doors::Setup::Grey),
        addresses::setup::DOOR_BLUE => Setup::Door(super::rules::doors::Setup::Blue),
        addresses::setup::DOOR_COLORED => Setup::Door(super::rules::doors::Setup::Colored),
        addresses::setup::DOOR_SHOT_SIGNAL => Setup::Door(super::rules::doors::Setup::ShotSignal),
        addresses::setup::SETUP_GATE_C54D => Setup::Gate(super::rules::gates::Setup::Trigger(
            Side::Left,
            Color::Green,
        )),
        addresses::setup::SETUP_GATE_C56C => Setup::Gate(super::rules::gates::Setup::Trigger(
            Side::Right,
            Color::Green,
        )),
        addresses::setup::GATE_TRIGGER_LEFT_RED => {
            Setup::Gate(super::rules::gates::Setup::Trigger(Side::Left, Color::Red))
        }
        addresses::setup::GATE_TRIGGER_RIGHT_RED => {
            Setup::Gate(super::rules::gates::Setup::Trigger(Side::Right, Color::Red))
        }
        addresses::setup::SETUP_GATE_C5D3 => Setup::Gate(super::rules::gates::Setup::Trigger(
            Side::Left,
            Color::Yellow,
        )),
        addresses::setup::SETUP_GATE_C5F1 => Setup::Gate(super::rules::gates::Setup::Trigger(
            Side::Right,
            Color::Yellow,
        )),
        addresses::setup::GATE_TRIGGER_LEFT_BLUE => {
            Setup::Gate(super::rules::gates::Setup::Trigger(Side::Left, Color::Blue))
        }
        addresses::setup::SETUP_GATE_C627 => Setup::Gate(super::rules::gates::Setup::Trigger(
            Side::Right,
            Color::Blue,
        )),
        addresses::setup::GATE_PLACEMENT_DOWN_SOLID => {
            Setup::Gate(super::rules::gates::Setup::Placement(Direction::Down, true))
        }
        addresses::setup::GATE_PLACEMENT_UP_SOLID => {
            Setup::Gate(super::rules::gates::Setup::Placement(Direction::Up, true))
        }
        addresses::setup::SETUP_GATE_C6D8 => Setup::Gate(super::rules::gates::Setup::Placement(
            Direction::Down,
            false,
        )),
        addresses::setup::GATE_PLACEMENT_UP_OPEN => {
            Setup::Gate(super::rules::gates::Setup::Placement(Direction::Up, false))
        }
        addresses::setup::GATE_SHOOTBLOCK_DOWN => {
            Setup::Gate(super::rules::gates::Setup::Shootblock(Direction::Down))
        }
        addresses::setup::GATE_SHOOTBLOCK_UP => {
            Setup::Gate(super::rules::gates::Setup::Shootblock(Direction::Up))
        }
        addresses::setup::TERRAIN_SCREW_ATTACK => {
            Setup::Terrain(super::rules::terrain_setup::Setup::ScrewAttack)
        }
        addresses::setup::TERRAIN_SPEED_BOOST => {
            Setup::Terrain(super::rules::terrain_setup::Setup::SpeedBoost)
        }
        addresses::setup::TERRAIN_CRUMBLE => {
            Setup::Terrain(super::rules::terrain_setup::Setup::Crumble)
        }
        addresses::setup::TERRAIN_SHOT => Setup::Terrain(super::rules::terrain_setup::Setup::Shot),
        addresses::setup::TERRAIN_BOMB_CONTACT => {
            Setup::Terrain(super::rules::terrain_setup::Setup::BombContact)
        }
        addresses::setup::TERRAIN_BOMB_SHOT => {
            Setup::Terrain(super::rules::terrain_setup::Setup::BombShot)
        }
        addresses::setup::TERRAIN_POWER_BOMB_SHOT => {
            Setup::Terrain(super::rules::terrain_setup::Setup::PowerBombShot)
        }
        addresses::setup::TERRAIN_SUPER_MISSILE_SHOT => {
            Setup::Terrain(super::rules::terrain_setup::Setup::SuperMissileShot)
        }
        addresses::setup::TERRAIN_POWER_BOMB_TRIGGER => {
            Setup::Terrain(super::rules::terrain_setup::Setup::PowerBombTrigger)
        }
        addresses::setup::TERRAIN_CRITTERS_ESCAPE => {
            Setup::Terrain(super::rules::terrain_setup::Setup::CrittersEscape)
        }
        addresses::setup::TERRAIN_GRAPPLE => {
            Setup::Terrain(super::rules::terrain_setup::Setup::Grapple)
        }
        addresses::setup::TERRAIN_DEACTIVATE => {
            Setup::Terrain(super::rules::terrain_setup::Setup::Deactivate)
        }
        addresses::setup::TERRAIN_ENEMY_BREAKABLE => {
            Setup::Terrain(super::rules::terrain_setup::Setup::EnemyBreakable)
        }
        addresses::setup::CANNON_RIGHT => Setup::Cannon(super::rules::cannon_setup::Setup::Right),
        addresses::setup::CANNON_DIAGONAL_RIGHT => {
            Setup::Cannon(super::rules::cannon_setup::Setup::DiagonalRight)
        }
        addresses::setup::CANNON_LEFT => Setup::Cannon(super::rules::cannon_setup::Setup::Left),
        addresses::setup::CANNON_DIAGONAL_LEFT => {
            Setup::Cannon(super::rules::cannon_setup::Setup::DiagonalLeft)
        }
        addresses::setup::CANNON_UNSHIELDED => {
            Setup::Cannon(super::rules::cannon_setup::Setup::Unshielded)
        }
        addresses::setup::PLANT_PLANTS_FLOOR => Setup::Plant(super::rules::plants::Surface::Floor),
        addresses::setup::PLANT_PLANTS_CEILING => {
            Setup::Plant(super::rules::plants::Surface::Ceiling)
        }
        addresses::setup::SOUND_DELAY => Setup::SoundDelay,
        addresses::setup::BOOSTER_ESCAPE => Setup::BoosterEscape,
        addresses::setup::TUBE_TERRAIN_AIR => {
            Setup::TubeTerrain(super::rules::noob_tube::Setup::Air)
        }
        addresses::setup::TUBE_TERRAIN_SOLID => {
            Setup::TubeTerrain(super::rules::noob_tube::Setup::Solid)
        }
        addresses::setup::TUBE_TERRAIN_FOUR_BLOCKS => {
            Setup::TubeTerrain(super::rules::noob_tube::Setup::FourBlocks)
        }
        addresses::setup::NOOB_TUBE => Setup::NoobTube,
        addresses::setup::TORIZO_STATUE => Setup::TorizoStatue,
        addresses::setup::MOTHER_BRAIN_GLASS => Setup::MotherBrainGlass,
        addresses::setup::CRUMBLING_WALL => Setup::CrumblingWall,
        addresses::setup::TREADMILL => Setup::Treadmill,
        addresses::setup::PERIODIC_DAMAGE => Setup::PeriodicDamage,
        addresses::setup::TERRAIN_BLANK => {
            Setup::Terrain(super::rules::terrain_setup::Setup::Blank)
        }
        addresses::setup::TERRAIN_SHOT_BLOCK => {
            Setup::Terrain(super::rules::terrain_setup::Setup::ShotBlock)
        }
        addresses::setup::TERRAIN_GRAPPLE_BLOCK => {
            Setup::Terrain(super::rules::terrain_setup::Setup::GrappleBlock)
        }
        addresses::setup::HAND_TRIGGER => Setup::HandTrigger,
        addresses::setup::SHAKTOOL => Setup::Shaktool,
        addresses::setup::TOURIAN_WALL_OPEN => {
            Setup::TourianWall(super::rules::tourian_wall::Setup::Open)
        }
        addresses::setup::TOURIAN_WALL_CLOSE => {
            Setup::TourianWall(super::rules::tourian_wall::Setup::Close)
        }
        addresses::setup::ANIMAL_RESCUE_RESCUE_WALL => {
            Setup::AnimalRescue(super::rules::animal_rescue::Setup::RescueWall)
        }
        addresses::setup::ANIMAL_RESCUE_MAIN_STREET_PASSAGE => {
            Setup::AnimalRescue(super::rules::animal_rescue::Setup::MainStreetPassage)
        }
        addresses::setup::METROID_QUOTA => Setup::MetroidQuota,
        addresses::setup::EYE_DOOR_EYE_DOOR_EYE => {
            Setup::EyeDoor(super::rules::eye_door::Part::Eye)
        }
        addresses::setup::EYE_DOOR_EYE_DOOR_BODY => {
            Setup::EyeDoor(super::rules::eye_door::Part::Body)
        }
        addresses::setup::CLEAR_BOTWOON_WALL
        | addresses::setup::RETURN_0_B3CF
        | addresses::setup::MARIDIA_ELEVATUBE
        | addresses::setup::WRECKED_SHIP_ATTIC
        | addresses::setup::WRECKED_SHIP_CHOZO_SLOPE_ACCESS
        | addresses::setup::RETURN_0_D6F7
        | addresses::setup::LOWER_NORFAIR_CHOZO_HAND
        | addresses::setup::RETURN_0_B3D0
        | addresses::setup::MARIDIA_SAND_FALLS
        | addresses::setup::GATE_BLOCK => Setup::Return(0),
        addresses::setup::RETURN_1_B3D2 | addresses::setup::RETURN_1_CFD1 => Setup::Return(1),
        addresses::setup::RETURN_0_X41 => Setup::Return(0x41),
        addresses::setup::DIAGNOSTIC_UNSUPPORTED_B3_E3 => {
            Setup::Diagnostic(Diagnostic::UnsupportedB3E3)
        }
        a => Setup::External(a),
    }
}

use super::host::{Contact as HostContact, Host};
pub fn setup(
    c: &mut impl Host,
    original: u32,
    slot: usize,
    displacement: &mut HostContact<'_>,
    accumulator: u16,
) -> u8 {
    match decode(original) {
        Setup::Return(value) => value,
        Setup::Diagnostic(_) => c.diagnostic(super::host::Diagnostic::UnsupportedSetup),
        Setup::External(_) => c.diagnostic(super::host::Diagnostic::Unreachable),
        Setup::Contact(Contact::Physics(kind)) => {
            super::rules::contact_physics::setup(c, displacement, kind)
        }
        Setup::Contact(Contact::Hand(kind)) => {
            super::rules::hands::setup(c, slot, kind, displacement)
        }
        Setup::Door(kind) => {
            super::rules::doors::setup(c, slot, kind);
            0
        }
        Setup::Gate(kind) => super::rules::gates::setup(c, slot, kind, accumulator),
        Setup::HandTrigger => super::rules::hands::prepare_ship_trigger(c, slot),
        Setup::PeriodicDamage => {
            let damage = c.session().player.periodic_damage.wrapping_add(1);
            c.session_mut().player.periodic_damage = damage;
            0x41
        }
        Setup::Shaktool => super::rules::shaktool::begin(c),
        Setup::NoobTube => {
            super::rules::noob_tube::setup(c, slot, super::rules::noob_tube::Setup::Solid)
        }
        Setup::Terrain(kind) => super::rules::terrain_setup::setup(c, slot, kind),
        Setup::Station(kind) => super::rules::stations::setup(c, slot, kind),
        Setup::Scroll(kind) => super::rules::scroll_setup::setup(c, slot, kind),
        Setup::Item(kind) => super::rules::item_setup::setup(c, slot, kind),
        Setup::Cannon(kind) => super::rules::cannon_setup::setup(c, slot, kind),
        Setup::Plant(kind) => super::rules::plants::begin(c, slot, kind),
        Setup::TourianWall(kind) => super::rules::tourian_wall::setup(c, slot, kind),
        Setup::AnimalRescue(kind) => super::rules::animal_rescue::setup(c, slot, kind),
        Setup::TubeTerrain(kind) => super::rules::noob_tube::setup(c, slot, kind),
        Setup::TorizoStatue => super::rules::torizo_statue::setup(c, slot),
        Setup::MotherBrainGlass => super::rules::mother_brain_glass::setup(c, slot),
        Setup::CrumblingWall => super::rules::crumbling_wall::setup(c, slot),
        Setup::Treadmill => super::rules::treadmill::setup(c, slot),
        Setup::BoosterEscape => super::rules::booster_escape::begin(c, slot),
        Setup::SoundDelay => super::rules::sound_delay::begin(c, slot),
        Setup::MetroidQuota => super::rules::metroid_quota::begin(c, slot),
        Setup::EyeDoor(kind) => super::rules::eye_door::begin(c, slot, kind),
    }
}
