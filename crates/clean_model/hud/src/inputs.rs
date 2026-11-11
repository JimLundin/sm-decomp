//! What the HUD reads each frame (§1.3): the values other domains own, the buttons, and where she
//! is on her map.

use clean_types::{Pair, by_member};
use enum_map::EnumMap;

use crate::grid::Place;
use crate::item::{Ammo, SelectableItem};
use crate::map::HerMap;

/// The count of frames the game has finished, mod 256, as this frame begins; a lag frame isn't
/// one (§1.3).
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub struct FrameCount(pub u8);

/// Her energy now and her capacity (§1.3).
#[derive(Clone, Copy, Debug)]
pub struct Energy {
    /// Her energy now.
    pub now: u16,
    /// Her capacity.
    pub capacity: u16,
}

/// Whether the reserve tanks refill on their own.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum ReserveMode {
    /// They refill on their own (`auto`).
    Auto,
    /// She refills from them by hand.
    Manual,
}

/// Her reserve tanks: the energy in them and how they refill (§1.3).
#[derive(Clone, Copy, Debug)]
pub struct Reserve {
    /// The energy in the reserve tanks.
    pub energy: u16,
    /// How they refill.
    pub mode: ReserveMode,
}

/// The most of an ammunition, count or capacity, the HUD is defined on (§1.5).
struct AmmoDomain {
    /// The ammunition.
    ammo: Ammo,
    /// Its most.
    most: u16,
}

/// §1.5: missiles at most 999, super missiles and power bombs at most 99.
const AMMO_DOMAIN: [AmmoDomain; 3] = [
    AmmoDomain {
        ammo: Ammo::Missiles,
        most: 999,
    },
    AmmoDomain {
        ammo: Ammo::SuperMissiles,
        most: 99,
    },
    AmmoDomain {
        ammo: Ammo::PowerBombs,
        most: 99,
    },
];

/// A count of one ammunition and how many she can carry (§1.3).
#[derive(Clone, Copy, PartialEq, Eq, Debug, Default)]
pub struct Stock {
    /// How many she has.
    pub count: u16,
    /// How many she can carry.
    pub capacity: u16,
}

/// Her stock of each ammunition, each within the HUD's domain (§1.5).
#[derive(Clone, Copy, Debug)]
pub struct Ammunition(EnumMap<Ammo, Stock>);

impl Ammunition {
    /// Her stocks, or `None` if a count or capacity is past what the HUD is defined on (§1.5).
    #[must_use]
    pub fn new(stocks: EnumMap<Ammo, Stock>) -> Option<Self> {
        let domain = by_member(AMMO_DOMAIN, |row| row.ammo);
        stocks
            .iter()
            .all(|(ammo, stock)| stock.count.max(stock.capacity) <= domain[ammo].most)
            .then_some(Self(stocks))
    }

    /// Her stock of `ammo`.
    #[must_use]
    pub fn of(&self, ammo: Ammo) -> Stock {
        self.0[ammo]
    }
}

/// The items she has on that the HUD can select without a count (§1.3).
#[derive(Clone, Copy, Debug, Default)]
pub struct Equipped {
    /// She has the grapple beam on.
    pub grapple_beam: bool,
    /// She has the x-ray scope on.
    pub x_ray_scope: bool,
}

/// What other domains report about her that the status bar shows (§1.3).
#[derive(Clone, Copy, Debug)]
pub struct Status {
    /// Her energy.
    pub energy: Energy,
    /// Her reserve tanks.
    pub reserve: Reserve,
    /// Her ammunition.
    pub ammunition: Ammunition,
    /// The items she has on.
    pub equipped: Equipped,
}

impl Status {
    /// Spec: CyclingItems. Whether Select can land on `item`: an ammunition with a count above 0,
    /// or an item she has on.
    #[must_use]
    pub fn has(&self, item: SelectableItem) -> bool {
        match item {
            SelectableItem::Ammo(ammo) => self.ammunition.of(ammo).count != 0,
            SelectableItem::GrappleBeam => self.equipped.grapple_beam,
            SelectableItem::XRayScope => self.equipped.x_ray_scope,
        }
    }
}

/// Item select this frame (§1.3).
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum SelectButton {
    /// It didn't go down this frame.
    NotPressed,
    /// It went down this frame.
    Pressed,
}

/// Item cancel this frame (§1.3).
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum CancelButton {
    /// Neither pressed this frame nor held from before.
    NotPressed,
    /// It went down this frame.
    Pressed,
    /// Held from before.
    Held,
}

/// Why a switch makes no click (§1.3; spec: SwitchClick). Listed alphabetically; the order means
/// nothing.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum Mute {
    /// The grapple beam is out.
    Grapple,
    /// She spin jumps.
    SpinJump,
    /// Time is stopped.
    TimeStopped,
    /// She wall jumps.
    WallJump,
}

/// The buttons and what lets them act (§1.3).
#[derive(Clone, Copy, Debug)]
pub struct Controls {
    /// Item select.
    pub select: SelectButton,
    /// Item cancel.
    pub cancel: CancelButton,
    /// The player controls her and she doesn't face the screen.
    pub can_switch: bool,
    /// The weapons cancelled the selection after a shot this frame.
    pub shot_cancels: bool,
    /// Why a switch makes no click, or `None`.
    pub muted_by: Option<Mute>,
}

/// Her room on her map (§1.3).
#[derive(Clone, Copy, Debug)]
pub struct Room {
    /// Its top-left cell on her map.
    pub cell: Place,
    /// Its size in 16 px blocks, across and down.
    pub blocks: Pair<u16>,
}

/// What the mini-map reads (§1.3).
#[derive(Clone, Copy, Debug)]
pub struct MiniMapInputs<'map> {
    /// False while she is held at a station, drained, entering or leaving the gunship, or while
    /// the game's opening shows her.
    pub runs: bool,
    /// Something turned the mini-map on since its last turn, and no boss was met since; or,
    /// during this frame before the mini-map's turn, Crocomire fell into the acid.
    pub turned_on: bool,
    /// Where she is: whole pixels in her room, after this frame's moves and before a pose change
    /// places her.
    pub position: Pair<u16>,
    /// Her room.
    pub room: Room,
    /// Her map: its pictures and the cells it charts.
    pub map: &'map HerMap,
    /// She has collected her map, so the window shows the cells it charts.
    pub map_collected: bool,
}

/// Everything the HUD reads in one frame of play (§1.3).
#[derive(Clone, Copy, Debug)]
pub struct FrameInputs<'map> {
    /// The frame count as this frame begins.
    pub frame: FrameCount,
    /// What the status bar shows.
    pub status: Status,
    /// The buttons.
    pub controls: Controls,
    /// The item whose pickup this frame draws its icon, or `None`.
    pub collected: Option<SelectableItem>,
    /// Mother Brain's first form falls this frame, her brain in the tank destroyed.
    pub mother_brain_falls: bool,
    /// What the mini-map reads.
    pub mini_map: MiniMapInputs<'map>,
}

#[cfg(test)]
mod tests {
    use super::AMMO_DOMAIN;
    use crate::item::Ammo;
    use crate::tests::names_each_member_once;

    #[test]
    fn the_ammo_domain_names_each_ammo_once() {
        assert!(names_each_member_once::<Ammo, _>(&AMMO_DOMAIN, |row| row.ammo));
    }
}
