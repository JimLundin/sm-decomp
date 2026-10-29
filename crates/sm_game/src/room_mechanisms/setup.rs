//! Semantic setup actions and exact encoded dispatch identity.
#![forbid(unsafe_code)]
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
    #[allow(dead_code)]
    Noop,
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
/// Captured at the encoded setup-word read, independently of the semantic action.
#[derive(Clone, Copy, Debug, PartialEq)]
struct Decoded {
    original: u32,
    action: Setup<u32>,
}
impl Decoded {
    #[allow(dead_code)]
    fn original(self) -> u32 {
        self.original
    }
    fn action(self) -> Setup<u32> {
        self.action
    }
}
impl From<Decoded> for Setup<u32> {
    fn from(value: Decoded) -> Self {
        value.action
    }
}
fn decode(original: u32) -> Decoded {
    let action = match original {
        8711757 => Setup::Item(super::rules::item_setup::Setup::Initialize(
            super::rules::item_setup::Container::Visible,
            Some(super::rules::item_setup::Graphic::EnergyTank),
        )),
        8711762 => Setup::Item(super::rules::item_setup::Setup::Initialize(
            super::rules::item_setup::Container::Visible,
            Some(super::rules::item_setup::Graphic::MissileTank),
        )),
        8711767 => Setup::Item(super::rules::item_setup::Setup::Initialize(
            super::rules::item_setup::Container::Visible,
            Some(super::rules::item_setup::Graphic::SuperMissileTank),
        )),
        8711772 => Setup::Item(super::rules::item_setup::Setup::Initialize(
            super::rules::item_setup::Container::Visible,
            Some(super::rules::item_setup::Graphic::PowerBombTank),
        )),
        8711780 => Setup::Item(super::rules::item_setup::Setup::Initialize(
            super::rules::item_setup::Container::Visible,
            None,
        )),
        8711799 => Setup::Item(super::rules::item_setup::Setup::Initialize(
            super::rules::item_setup::Container::ShotBlock,
            Some(super::rules::item_setup::Graphic::EnergyTank),
        )),
        8711804 => Setup::Item(super::rules::item_setup::Setup::Initialize(
            super::rules::item_setup::Container::ShotBlock,
            Some(super::rules::item_setup::Graphic::MissileTank),
        )),
        8711809 => Setup::Item(super::rules::item_setup::Setup::Initialize(
            super::rules::item_setup::Container::ShotBlock,
            Some(super::rules::item_setup::Graphic::SuperMissileTank),
        )),
        8711814 => Setup::Item(super::rules::item_setup::Setup::Initialize(
            super::rules::item_setup::Container::ShotBlock,
            Some(super::rules::item_setup::Graphic::PowerBombTank),
        )),
        8711822 => Setup::Item(super::rules::item_setup::Setup::Initialize(
            super::rules::item_setup::Container::ShotBlock,
            None,
        )),
        8711851 => Setup::Item(super::rules::item_setup::Setup::Trigger),
        8696634 => Setup::Scroll(super::rules::scroll_setup::Setup::Place(
            super::rules::scroll_setup::Tile::Extension(
                super::rules::scroll_setup::Direction::Right,
            ),
        )),
        8696645 => Setup::Scroll(super::rules::scroll_setup::Setup::Place(
            super::rules::scroll_setup::Tile::Extension(
                super::rules::scroll_setup::Direction::Left,
            ),
        )),
        8696656 => Setup::Scroll(super::rules::scroll_setup::Setup::Place(
            super::rules::scroll_setup::Tile::Extension(
                super::rules::scroll_setup::Direction::Down,
            ),
        )),
        8696667 => Setup::Scroll(super::rules::scroll_setup::Setup::Place(
            super::rules::scroll_setup::Tile::Extension(super::rules::scroll_setup::Direction::Up),
        )),
        8696689 => Setup::Scroll(super::rules::scroll_setup::Setup::Place(
            super::rules::scroll_setup::Tile::Scroll(super::rules::scroll_setup::Solidity::Air),
        )),
        8696706 => Setup::Scroll(super::rules::scroll_setup::Setup::Place(
            super::rules::scroll_setup::Tile::Scroll(super::rules::scroll_setup::Solidity::Solid),
        )),
        8696723 => Setup::Scroll(super::rules::scroll_setup::Setup::Touch),
        8696264 => Setup::Station(super::rules::stations::Setup::Access(
            super::rules::stations::Service::Map,
            super::rules::stations::Side::Right,
        )),
        8696304 => Setup::Station(super::rules::stations::Setup::Access(
            super::rules::stations::Service::Map,
            super::rules::stations::Side::Left,
        )),
        8696429 => Setup::Station(super::rules::stations::Setup::Access(
            super::rules::stations::Service::Energy,
            super::rules::stations::Side::Right,
        )),
        8696477 => Setup::Station(super::rules::stations::Setup::Access(
            super::rules::stations::Service::Energy,
            super::rules::stations::Side::Left,
        )),
        8696528 => Setup::Station(super::rules::stations::Setup::Access(
            super::rules::stations::Service::Missile,
            super::rules::stations::Side::Right,
        )),
        8696576 => Setup::Station(super::rules::stations::Setup::Access(
            super::rules::stations::Service::Missile,
            super::rules::stations::Side::Left,
        )),
        8697232 => Setup::Station(super::rules::stations::Setup::SaveTrigger),
        8697326 => Setup::Station(super::rules::stations::Setup::Save),
        8696203 => Setup::Station(super::rules::stations::Setup::Place(
            super::rules::stations::Service::Map,
        )),
        8696349 => Setup::Station(super::rules::stations::Setup::Place(
            super::rules::stations::Service::Energy,
        )),
        8696389 => Setup::Station(super::rules::stations::Setup::Place(
            super::rules::stations::Service::Missile,
        )),
        8696811 => Setup::Contact(Contact::Physics(super::rules::contact_physics::Setup::Ice)),
        8696840 => Setup::Contact(Contact::Physics(
            super::rules::contact_physics::Setup::Surface,
        )),
        8696983 => Setup::Contact(Contact::Physics(
            super::rules::contact_physics::Setup::Submerge,
        )),
        8697000 => Setup::Contact(Contact::Physics(
            super::rules::contact_physics::Setup::SlowFall,
        )),
        8697014 => Setup::Contact(Contact::Physics(
            super::rules::contact_physics::Setup::FastFall,
        )),
        8697153 => Setup::Contact(Contact::Physics(
            super::rules::contact_physics::Setup::StopVertical,
        )),
        8697028 => Setup::Contact(Contact::Physics(
            super::rules::contact_physics::Setup::CollisionSurface,
        )),
        8704399 => Setup::Contact(Contact::Hand(super::rules::hands::Hand::LowerNorfair)),
        8705568 => Setup::Contact(Contact::Hand(super::rules::hands::Hand::WreckedShip)),
        8697336 => Setup::Door(super::rules::doors::Setup::MotherBrainEscape),
        8698353 => Setup::Door(super::rules::doors::Setup::CeresEscape),
        8701844 => Setup::Door(super::rules::doors::Setup::Grey),
        8701883 => Setup::Door(super::rules::doors::Setup::Blue),
        8701873 => Setup::Door(super::rules::doors::Setup::Colored),
        8701922 => Setup::Door(super::rules::doors::Setup::ShotSignal),
        8701261 => Setup::Gate(super::rules::gates::Setup::Trigger(
            Side::Left,
            Color::Green,
        )),
        8701292 => Setup::Gate(super::rules::gates::Setup::Trigger(
            Side::Right,
            Color::Green,
        )),
        8701323 => Setup::Gate(super::rules::gates::Setup::Trigger(Side::Left, Color::Red)),
        8701359 => Setup::Gate(super::rules::gates::Setup::Trigger(Side::Right, Color::Red)),
        8701395 => Setup::Gate(super::rules::gates::Setup::Trigger(
            Side::Left,
            Color::Yellow,
        )),
        8701425 => Setup::Gate(super::rules::gates::Setup::Trigger(
            Side::Right,
            Color::Yellow,
        )),
        8701456 => Setup::Gate(super::rules::gates::Setup::Trigger(Side::Left, Color::Blue)),
        8701479 => Setup::Gate(super::rules::gates::Setup::Trigger(
            Side::Right,
            Color::Blue,
        )),
        8701630 => Setup::Gate(super::rules::gates::Setup::Placement(Direction::Down, true)),
        8701643 => Setup::Gate(super::rules::gates::Setup::Placement(Direction::Up, true)),
        8701656 => Setup::Gate(super::rules::gates::Setup::Placement(
            Direction::Down,
            false,
        )),
        8701660 => Setup::Gate(super::rules::gates::Setup::Placement(Direction::Up, false)),
        8701664 => Setup::Gate(super::rules::gates::Setup::Shootblock(Direction::Down)),
        8701754 => Setup::Gate(super::rules::gates::Setup::Shootblock(Direction::Up)),
        8703426 => Setup::Terrain(super::rules::terrain_setup::Setup::ScrewAttack),
        8703466 => Setup::Terrain(super::rules::terrain_setup::Setup::SpeedBoost),
        8703543 => Setup::Terrain(super::rules::terrain_setup::Setup::Crumble),
        8703595 => Setup::Terrain(super::rules::terrain_setup::Setup::Shot),
        8703619 => Setup::Terrain(super::rules::terrain_setup::Setup::BombContact),
        8703706 => Setup::Terrain(super::rules::terrain_setup::Setup::BombShot),
        8703790 => Setup::Terrain(super::rules::terrain_setup::Setup::PowerBombShot),
        8703847 => Setup::Terrain(super::rules::terrain_setup::Setup::SuperMissileShot),
        8703904 => Setup::Terrain(super::rules::terrain_setup::Setup::PowerBombTrigger),
        8698232 => Setup::Terrain(super::rules::terrain_setup::Setup::CrittersEscape),
        8703925 => Setup::Terrain(super::rules::terrain_setup::Setup::Grapple),
        8696769 => Setup::Terrain(super::rules::terrain_setup::Setup::Deactivate),
        8696788 => Setup::Terrain(super::rules::terrain_setup::Setup::EnemyBreakable),
        8707732 => Setup::Cannon(super::rules::cannon_setup::Setup::Right),
        8707769 => Setup::Cannon(super::rules::cannon_setup::Setup::DiagonalRight),
        8707824 => Setup::Cannon(super::rules::cannon_setup::Setup::Left),
        8707861 => Setup::Cannon(super::rules::cannon_setup::Setup::DiagonalLeft),
        8707916 => Setup::Cannon(super::rules::cannon_setup::Setup::Unshielded),
        8696028 => Setup::Plant(super::rules::plants::Surface::Floor),
        8696083 => Setup::Plant(super::rules::plants::Surface::Ceiling),
        8697795 => Setup::SoundDelay,
        8698012 => Setup::BoosterEscape,
        8705663 => Setup::TubeTerrain(super::rules::noob_tube::Setup::Air),
        8705673 => Setup::TubeTerrain(super::rules::noob_tube::Setup::Solid),
        8705683 => Setup::TubeTerrain(super::rules::noob_tube::Setup::FourBlocks),
        8705740 => Setup::NoobTube,
        8705542 => Setup::TorizoStatue,
        8705526 => Setup::MotherBrainGlass,
        8694568 => Setup::CrumblingWall,
        8695882 => Setup::Treadmill,
        8703957 => Setup::PeriodicDamage,
        8704264 => Setup::Terrain(super::rules::terrain_setup::Setup::Blank),
        8704279 => Setup::Terrain(super::rules::terrain_setup::Setup::ShotBlock),
        8704299 => Setup::Terrain(super::rules::terrain_setup::Setup::GrappleBlock),
        8705558 => Setup::HandTrigger,
        8698076 => Setup::Shaktool,
        8697169 => Setup::TourianWall(super::rules::tourian_wall::Setup::Open),
        8697199 => Setup::TourianWall(super::rules::tourian_wall::Setup::Close),
        8698309 => Setup::AnimalRescue(super::rules::animal_rescue::Setup::RescueWall),
        8698633 => Setup::AnimalRescue(super::rules::animal_rescue::Setup::MainStreetPassage),
        8706846 => Setup::MetroidQuota,
        8706700 => Setup::EyeDoor(super::rules::eye_door::Part::Eye),
        8706745 => Setup::EyeDoor(super::rules::eye_door::Part::Body),
        8694567 | 8696783 | 8698095 | 8698618 | 8705782 | 8705783 | 8704398 | 8696784 | 8697167
        | 8698220 => Setup::Return(0),
        8696786 | 8703953 => Setup::Return(1),
        8703949 => Setup::Return(0x41),
        8696803 => Setup::Diagnostic(Diagnostic::UnsupportedB3E3),
        a => Setup::External(a),
    };
    Decoded { original, action }
}

use super::host::{Contact as HostContact, Host};
pub fn setup(
    c: &mut impl Host,
    original: u32,
    slot: usize,
    displacement: &mut HostContact<'_>,
    accumulator: u16,
) -> u8 {
    match decode(original).action() {
        Setup::Noop => 0,
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
