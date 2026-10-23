//! Semantic setup actions and exact encoded dispatch identity.
#![forbid(unsafe_code)]
use crate::game::mechanism_gates::{Color, Direction, Side};
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum Contact {
    Physics(crate::game::mechanism_contact_physics::Setup),
    Hand(crate::game::mechanism_hands::Hand),
}
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum Diagnostic {
    UnsupportedB3E3,
}
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum Setup<X> {
    Noop,
    Return(u8),
    Diagnostic(Diagnostic),
    Door(crate::game::mechanism_doors::Setup),
    Terrain(crate::game::mechanism_terrain_setup::Setup),
    Gate(crate::game::mechanism_gates::Setup),
    Station(crate::game::mechanism_stations::Setup),
    Scroll(crate::game::mechanism_scroll_setup::Setup),
    Item(crate::game::mechanism_item_setup::Setup),
    Cannon(crate::game::mechanism_cannon_setup::Setup),
    Plant(crate::game::mechanism_plants::Surface),
    EyeDoor(crate::game::mechanism_eye_door::Part),
    MetroidQuota,
    SoundDelay,
    MotherBrainGlass,
    TorizoStatue,
    NoobTube,
    TubeTerrain(crate::game::mechanism_noob_tube::Setup),
    CrumblingWall,
    Treadmill,
    PeriodicDamage,
    HandTrigger,
    BoosterEscape,
    Shaktool,
    AnimalRescue(crate::game::mechanism_animal_rescue::Setup),
    TourianWall(crate::game::mechanism_tourian_wall::Setup),
    Contact(Contact),
    External(X),
}
/// Captured at the encoded setup-word read, independently of the semantic action.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Decoded {
    original: u32,
    action: Setup<u32>,
}
impl Decoded {
    pub fn original(self) -> u32 {
        self.original
    }
    pub fn action(self) -> Setup<u32> {
        self.action
    }
}
impl From<Decoded> for Setup<u32> {
    fn from(value: Decoded) -> Self {
        value.action
    }
}
pub fn decode(original: u32) -> Decoded {
    let action = match original {
        8711757 => Setup::Item(crate::game::mechanism_item_setup::Setup::Initialize(
            crate::game::mechanism_item_setup::Container::Visible,
            Some(crate::game::mechanism_item_setup::Graphic::EnergyTank),
        )),
        8711762 => Setup::Item(crate::game::mechanism_item_setup::Setup::Initialize(
            crate::game::mechanism_item_setup::Container::Visible,
            Some(crate::game::mechanism_item_setup::Graphic::MissileTank),
        )),
        8711767 => Setup::Item(crate::game::mechanism_item_setup::Setup::Initialize(
            crate::game::mechanism_item_setup::Container::Visible,
            Some(crate::game::mechanism_item_setup::Graphic::SuperMissileTank),
        )),
        8711772 => Setup::Item(crate::game::mechanism_item_setup::Setup::Initialize(
            crate::game::mechanism_item_setup::Container::Visible,
            Some(crate::game::mechanism_item_setup::Graphic::PowerBombTank),
        )),
        8711780 => Setup::Item(crate::game::mechanism_item_setup::Setup::Initialize(
            crate::game::mechanism_item_setup::Container::Visible,
            None,
        )),
        8711799 => Setup::Item(crate::game::mechanism_item_setup::Setup::Initialize(
            crate::game::mechanism_item_setup::Container::ShotBlock,
            Some(crate::game::mechanism_item_setup::Graphic::EnergyTank),
        )),
        8711804 => Setup::Item(crate::game::mechanism_item_setup::Setup::Initialize(
            crate::game::mechanism_item_setup::Container::ShotBlock,
            Some(crate::game::mechanism_item_setup::Graphic::MissileTank),
        )),
        8711809 => Setup::Item(crate::game::mechanism_item_setup::Setup::Initialize(
            crate::game::mechanism_item_setup::Container::ShotBlock,
            Some(crate::game::mechanism_item_setup::Graphic::SuperMissileTank),
        )),
        8711814 => Setup::Item(crate::game::mechanism_item_setup::Setup::Initialize(
            crate::game::mechanism_item_setup::Container::ShotBlock,
            Some(crate::game::mechanism_item_setup::Graphic::PowerBombTank),
        )),
        8711822 => Setup::Item(crate::game::mechanism_item_setup::Setup::Initialize(
            crate::game::mechanism_item_setup::Container::ShotBlock,
            None,
        )),
        8711851 => Setup::Item(crate::game::mechanism_item_setup::Setup::Trigger),
        8696634 => Setup::Scroll(crate::game::mechanism_scroll_setup::Setup::Place(
            crate::game::mechanism_scroll_setup::Tile::Extension(
                crate::game::mechanism_scroll_setup::Direction::Right,
            ),
        )),
        8696645 => Setup::Scroll(crate::game::mechanism_scroll_setup::Setup::Place(
            crate::game::mechanism_scroll_setup::Tile::Extension(
                crate::game::mechanism_scroll_setup::Direction::Left,
            ),
        )),
        8696656 => Setup::Scroll(crate::game::mechanism_scroll_setup::Setup::Place(
            crate::game::mechanism_scroll_setup::Tile::Extension(
                crate::game::mechanism_scroll_setup::Direction::Down,
            ),
        )),
        8696667 => Setup::Scroll(crate::game::mechanism_scroll_setup::Setup::Place(
            crate::game::mechanism_scroll_setup::Tile::Extension(
                crate::game::mechanism_scroll_setup::Direction::Up,
            ),
        )),
        8696689 => Setup::Scroll(crate::game::mechanism_scroll_setup::Setup::Place(
            crate::game::mechanism_scroll_setup::Tile::Scroll(
                crate::game::mechanism_scroll_setup::Solidity::Air,
            ),
        )),
        8696706 => Setup::Scroll(crate::game::mechanism_scroll_setup::Setup::Place(
            crate::game::mechanism_scroll_setup::Tile::Scroll(
                crate::game::mechanism_scroll_setup::Solidity::Solid,
            ),
        )),
        8696723 => Setup::Scroll(crate::game::mechanism_scroll_setup::Setup::Touch),
        8696264 => Setup::Station(crate::game::mechanism_stations::Setup::Access(
            crate::game::mechanism_stations::Service::Map,
            crate::game::mechanism_stations::Side::Right,
        )),
        8696304 => Setup::Station(crate::game::mechanism_stations::Setup::Access(
            crate::game::mechanism_stations::Service::Map,
            crate::game::mechanism_stations::Side::Left,
        )),
        8696429 => Setup::Station(crate::game::mechanism_stations::Setup::Access(
            crate::game::mechanism_stations::Service::Energy,
            crate::game::mechanism_stations::Side::Right,
        )),
        8696477 => Setup::Station(crate::game::mechanism_stations::Setup::Access(
            crate::game::mechanism_stations::Service::Energy,
            crate::game::mechanism_stations::Side::Left,
        )),
        8696528 => Setup::Station(crate::game::mechanism_stations::Setup::Access(
            crate::game::mechanism_stations::Service::Missile,
            crate::game::mechanism_stations::Side::Right,
        )),
        8696576 => Setup::Station(crate::game::mechanism_stations::Setup::Access(
            crate::game::mechanism_stations::Service::Missile,
            crate::game::mechanism_stations::Side::Left,
        )),
        8697232 => Setup::Station(crate::game::mechanism_stations::Setup::SaveTrigger),
        8697326 => Setup::Station(crate::game::mechanism_stations::Setup::Save),
        8696203 => Setup::Station(crate::game::mechanism_stations::Setup::Place(
            crate::game::mechanism_stations::Service::Map,
        )),
        8696349 => Setup::Station(crate::game::mechanism_stations::Setup::Place(
            crate::game::mechanism_stations::Service::Energy,
        )),
        8696389 => Setup::Station(crate::game::mechanism_stations::Setup::Place(
            crate::game::mechanism_stations::Service::Missile,
        )),
        8696811 => Setup::Contact(Contact::Physics(
            crate::game::mechanism_contact_physics::Setup::Ice,
        )),
        8696840 => Setup::Contact(Contact::Physics(
            crate::game::mechanism_contact_physics::Setup::Surface,
        )),
        8696983 => Setup::Contact(Contact::Physics(
            crate::game::mechanism_contact_physics::Setup::Submerge,
        )),
        8697000 => Setup::Contact(Contact::Physics(
            crate::game::mechanism_contact_physics::Setup::SlowFall,
        )),
        8697014 => Setup::Contact(Contact::Physics(
            crate::game::mechanism_contact_physics::Setup::FastFall,
        )),
        8697153 => Setup::Contact(Contact::Physics(
            crate::game::mechanism_contact_physics::Setup::StopVertical,
        )),
        8697028 => Setup::Contact(Contact::Physics(
            crate::game::mechanism_contact_physics::Setup::CollisionSurface,
        )),
        8704399 => Setup::Contact(Contact::Hand(
            crate::game::mechanism_hands::Hand::LowerNorfair,
        )),
        8705568 => Setup::Contact(Contact::Hand(
            crate::game::mechanism_hands::Hand::WreckedShip,
        )),
        8697336 => Setup::Door(crate::game::mechanism_doors::Setup::MotherBrainEscape),
        8698353 => Setup::Door(crate::game::mechanism_doors::Setup::CeresEscape),
        8701844 => Setup::Door(crate::game::mechanism_doors::Setup::Grey),
        8701883 => Setup::Door(crate::game::mechanism_doors::Setup::Blue),
        8701873 => Setup::Door(crate::game::mechanism_doors::Setup::Colored),
        8701922 => Setup::Door(crate::game::mechanism_doors::Setup::ShotSignal),
        8701261 => Setup::Gate(crate::game::mechanism_gates::Setup::Trigger(
            Side::Left,
            Color::Green,
        )),
        8701292 => Setup::Gate(crate::game::mechanism_gates::Setup::Trigger(
            Side::Right,
            Color::Green,
        )),
        8701323 => Setup::Gate(crate::game::mechanism_gates::Setup::Trigger(
            Side::Left,
            Color::Red,
        )),
        8701359 => Setup::Gate(crate::game::mechanism_gates::Setup::Trigger(
            Side::Right,
            Color::Red,
        )),
        8701395 => Setup::Gate(crate::game::mechanism_gates::Setup::Trigger(
            Side::Left,
            Color::Yellow,
        )),
        8701425 => Setup::Gate(crate::game::mechanism_gates::Setup::Trigger(
            Side::Right,
            Color::Yellow,
        )),
        8701456 => Setup::Gate(crate::game::mechanism_gates::Setup::Trigger(
            Side::Left,
            Color::Blue,
        )),
        8701479 => Setup::Gate(crate::game::mechanism_gates::Setup::Trigger(
            Side::Right,
            Color::Blue,
        )),
        8701630 => Setup::Gate(crate::game::mechanism_gates::Setup::Placement(
            Direction::Down,
            true,
        )),
        8701643 => Setup::Gate(crate::game::mechanism_gates::Setup::Placement(
            Direction::Up,
            true,
        )),
        8701656 => Setup::Gate(crate::game::mechanism_gates::Setup::Placement(
            Direction::Down,
            false,
        )),
        8701660 => Setup::Gate(crate::game::mechanism_gates::Setup::Placement(
            Direction::Up,
            false,
        )),
        8701664 => Setup::Gate(crate::game::mechanism_gates::Setup::Shootblock(
            Direction::Down,
        )),
        8701754 => Setup::Gate(crate::game::mechanism_gates::Setup::Shootblock(
            Direction::Up,
        )),
        8703426 => Setup::Terrain(crate::game::mechanism_terrain_setup::Setup::ScrewAttack),
        8703466 => Setup::Terrain(crate::game::mechanism_terrain_setup::Setup::SpeedBoost),
        8703543 => Setup::Terrain(crate::game::mechanism_terrain_setup::Setup::Crumble),
        8703595 => Setup::Terrain(crate::game::mechanism_terrain_setup::Setup::Shot),
        8703619 => Setup::Terrain(crate::game::mechanism_terrain_setup::Setup::BombContact),
        8703706 => Setup::Terrain(crate::game::mechanism_terrain_setup::Setup::BombShot),
        8703790 => Setup::Terrain(crate::game::mechanism_terrain_setup::Setup::PowerBombShot),
        8703847 => Setup::Terrain(crate::game::mechanism_terrain_setup::Setup::SuperMissileShot),
        8703904 => Setup::Terrain(crate::game::mechanism_terrain_setup::Setup::PowerBombTrigger),
        8698232 => Setup::Terrain(crate::game::mechanism_terrain_setup::Setup::CrittersEscape),
        8703925 => Setup::Terrain(crate::game::mechanism_terrain_setup::Setup::Grapple),
        8696769 => Setup::Terrain(crate::game::mechanism_terrain_setup::Setup::Deactivate),
        8696788 => Setup::Terrain(crate::game::mechanism_terrain_setup::Setup::EnemyBreakable),
        8707732 => Setup::Cannon(crate::game::mechanism_cannon_setup::Setup::Right),
        8707769 => Setup::Cannon(crate::game::mechanism_cannon_setup::Setup::DiagonalRight),
        8707824 => Setup::Cannon(crate::game::mechanism_cannon_setup::Setup::Left),
        8707861 => Setup::Cannon(crate::game::mechanism_cannon_setup::Setup::DiagonalLeft),
        8707916 => Setup::Cannon(crate::game::mechanism_cannon_setup::Setup::Unshielded),
        8696028 => Setup::Plant(crate::game::mechanism_plants::Surface::Floor),
        8696083 => Setup::Plant(crate::game::mechanism_plants::Surface::Ceiling),
        8697795 => Setup::SoundDelay,
        8698012 => Setup::BoosterEscape,
        8705663 => Setup::TubeTerrain(crate::game::mechanism_noob_tube::Setup::Air),
        8705673 => Setup::TubeTerrain(crate::game::mechanism_noob_tube::Setup::Solid),
        8705683 => Setup::TubeTerrain(crate::game::mechanism_noob_tube::Setup::FourBlocks),
        8705740 => Setup::NoobTube,
        8705542 => Setup::TorizoStatue,
        8705526 => Setup::MotherBrainGlass,
        8694568 => Setup::CrumblingWall,
        8695882 => Setup::Treadmill,
        8703957 => Setup::PeriodicDamage,
        8704264 => Setup::Terrain(crate::game::mechanism_terrain_setup::Setup::Blank),
        8704279 => Setup::Terrain(crate::game::mechanism_terrain_setup::Setup::ShotBlock),
        8704299 => Setup::Terrain(crate::game::mechanism_terrain_setup::Setup::GrappleBlock),
        8705558 => Setup::HandTrigger,
        8698076 => Setup::Shaktool,
        8697169 => Setup::TourianWall(crate::game::mechanism_tourian_wall::Setup::Open),
        8697199 => Setup::TourianWall(crate::game::mechanism_tourian_wall::Setup::Close),
        8698309 => Setup::AnimalRescue(crate::game::mechanism_animal_rescue::Setup::RescueWall),
        8698633 => {
            Setup::AnimalRescue(crate::game::mechanism_animal_rescue::Setup::MainStreetPassage)
        }
        8706846 => Setup::MetroidQuota,
        8706700 => Setup::EyeDoor(crate::game::mechanism_eye_door::Part::Eye),
        8706745 => Setup::EyeDoor(crate::game::mechanism_eye_door::Part::Body),
        8694567 | 8696783 | 8698095 | 8698618 | 8705782 | 8705783 | 8704398 | 8696784 | 8697167
        | 8698220 => Setup::Return(0),
        8696786 | 8703953 => Setup::Return(1),
        8703949 => Setup::Return(0x41),
        8696803 => Setup::Diagnostic(Diagnostic::UnsupportedB3E3),
        a => Setup::External(a),
    };
    Decoded { original, action }
}
