//! The passage's shaft view and the room setup's four-cell closing placement.

use crate::state::video::{BgMode, Mode7};
use canonical::Canonical;
use data::assets::world::{DoorRule, RoomSetup};
use data::level::{Block, BlockKind, Bts};

/// Fixed entries installed by the room loader.
pub const CATALOGUE: [&str; 2] = [
    "world-shaft-entry/ShaftViewStarts",
    "world-shaft-entry/ShaftDoorSeals",
];

/// Which haze program the room placed. Its later intensity belongs to world haze.
#[derive(Canonical, Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum Haze {
    /// The room has not placed a haze program.
    #[default]
    Absent,
    /// The ordinary Ceres haze program.
    Ordinary,
    /// The red escape haze program.
    Red,
}

/// What the incoming doorway and the selected room version install.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Setup {
    /// The incoming doorway's display rule.
    pub door: DoorRule,
    /// The selected version's setup.
    pub room: RoomSetup,
    /// Ceres Ridley is defeated when the setup selects its haze.
    pub ridley_beaten: bool,
}

/// Opportunities supplied by the passage.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Opportunity {
    /// The early contents setup.
    Fetch,
    /// The block scripts take their first turn.
    Blocks,
    /// Neither owner runs at this cut.
    Hold,
}

/// The shaft rotation's initial table position, direction and wait.
#[derive(Canonical, Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct Rotation {
    /// Signed distance from the centred sample.
    pub sample: i16,
    /// The next sample is toward counterclockwise rotation.
    pub reverse: bool,
    /// Turns held before sampling.
    pub hold: u16,
}

/// The display and rotation state the shaft passes to its consumers.
#[derive(Canonical, Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct View {
    /// The display's usual HUD mode, preserved across setup.
    pub hud: BgMode,
    /// Requested mode below the HUD.
    pub room: BgMode,
    /// Number of HUD lines before switching to the room mode, if installed.
    pub hud_lines: Option<u8>,
    /// Whether the room publishes its affine registers.
    pub affine: bool,
    /// The transform, including the display's retained outside and mirror controls.
    pub transform: Mode7,
    /// The room's seed for the later rotation owner.
    pub rotation: Rotation,
}

/// The built level and entry effects, carried through the passage.
#[derive(Canonical, Clone, Debug, Default, PartialEq, Eq)]
pub struct State {
    /// Width of the built level in cells.
    pub width: u16,
    /// Layer one, row major, including each tile, flip, collision kind and mark.
    pub cells: Vec<Block>,
    /// The pending block-script placement's anchor.
    pub placement: Option<[u16; 2]>,
    /// The room's placed haze program.
    pub haze: Haze,
    /// The affine registers published to the display; before entry they can retain an older transform.
    pub displayed: Mode7,
    /// The room's display request and rotation seed.
    pub view: View,
}

impl State {
    /// The transient closing placement deletes itself on its first block-script turn.
    pub const fn run_blocks(&mut self) {
        self.placement = None;
    }

    /// Apply a passage opportunity. A misplaced closing anchor leaves the state intact.
    ///
    /// # Errors
    /// The four-cell placement must fit within the destination level.
    pub fn advance(
        &mut self,
        setup: Setup,
        opportunity: Opportunity,
    ) -> Result<(), super::room_loading::LoadFault> {
        match opportunity {
            Opportunity::Blocks => self.run_blocks(),
            Opportunity::Hold => {}
            Opportunity::Fetch => {
                if let Some([x, y]) = setup.room.closing_anchor() {
                    let width = usize::from(self.width);
                    let first = usize::from(y)
                        .saturating_mul(width)
                        .saturating_add(usize::from(x));
                    let last = first.saturating_add(width.saturating_mul(3));
                    if x >= self.width {
                        return Err(super::room_loading::LoadFault::Geometry);
                    }
                    let span = self
                        .cells
                        .get_mut(first..=last)
                        .ok_or(super::room_loading::LoadFault::Geometry)?;
                    for cell in span.iter_mut().step_by(width) {
                        cell.kind = BlockKind::Solid(Bts(cell.kind.bts()));
                    }
                    self.placement = Some([x, y]);
                }
                self.haze = match setup.room {
                    RoomSetup::Nothing => Haze::Absent,
                    RoomSetup::CeresHaze
                    | RoomSetup::CeresRidleyHaze
                    | RoomSetup::CeresHazeDoorShut { .. } => {
                        if setup.ridley_beaten {
                            Haze::Red
                        } else {
                            Haze::Ordinary
                        }
                    }
                };
                match setup.door {
                    DoorRule::ToCeresElevatorShaft => {
                        self.view.affine = true;
                        self.view.room = BgMode(7);
                        self.view.hud_lines = Some(31);
                        self.view.transform.matrix = [256, 0, 0, 256];
                        self.view.transform.centre = [128, 1008];
                        self.displayed.matrix = self.view.transform.matrix;
                        self.displayed.centre = self.view.transform.centre;
                        self.view.rotation = Rotation {
                            sample: 0,
                            reverse: false,
                            hold: 60,
                        };
                    }
                    DoorRule::FromCeresElevatorShaft => {
                        self.view.room = self.view.hud;
                        self.view.hud_lines = None;
                        self.view.affine = false;
                    }
                    DoorRule::Nothing => {}
                }
            }
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::state::video::Mode7Select;

    #[test]
    fn shaft_setup_preserves_each_look_mark_and_display_control() {
        let block = Block::new(0x9d12, 19);
        let entry = State {
            width: 3,
            cells: vec![block; 18],
            view: View {
                hud: BgMode(5),
                transform: Mode7 {
                    select: Mode7Select(65),
                    ..Mode7::default()
                },
                rotation: Rotation {
                    sample: -1,
                    reverse: true,
                    hold: 12,
                },
                ..View::default()
            },
            ..State::default()
        };
        let setup = Setup {
            door: DoorRule::ToCeresElevatorShaft,
            room: RoomSetup::CeresHazeDoorShut { top: [2, 1] },
            ridley_beaten: true,
        };
        let mut room = entry.clone();
        room.advance(setup, Opportunity::Fetch).unwrap();
        assert_eq!(room.view.hud, entry.view.hud);
        assert_eq!(room.view.transform.select, entry.view.transform.select);
        assert_eq!(
            room.view.rotation,
            Rotation {
                sample: 0,
                reverse: false,
                hold: 60
            }
        );
        assert_eq!(room.displayed.centre, [128, 1008]);
        assert_eq!(room.haze, Haze::Red);
        assert_eq!(room.placement, Some([2, 1]));
        room.run_blocks();
        assert_eq!(room.placement, None);
        for (index, actual) in room.cells.iter().enumerate() {
            assert_eq!(actual.tile, block.tile);
            assert_eq!(
                actual.kind,
                if [5, 8, 11, 14].contains(&index) {
                    BlockKind::Solid(Bts(19))
                } else {
                    block.kind
                }
            );
        }
        let invalid = Setup {
            room: RoomSetup::CeresHazeDoorShut { top: [2, 3] },
            ..setup
        };
        let mut attempt = entry.clone();
        assert!(attempt.advance(invalid, Opportunity::Fetch).is_err());
        assert_eq!(attempt, entry);
        attempt
            .advance(
                Setup {
                    room: RoomSetup::CeresHaze,
                    ridley_beaten: false,
                    ..setup
                },
                Opportunity::Fetch,
            )
            .unwrap();
        assert_eq!(attempt.haze, Haze::Ordinary);
        assert_eq!(attempt.cells, entry.cells);
        assert_eq!(attempt.placement, None);
    }
}
