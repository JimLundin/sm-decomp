//! Complete HUD quantities and actual ordered publications retained across staged HUD work.

use canonical::Canonical;
use canonical::model::{Model, Pinned};
use interface_hud::{HudPictures, HudState, Place, Rules};

/// The actual complete HUD drawing cannot supply an encoded cell.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum TilemapFault {
    /// A cell within the complete HUD extent is absent.
    MissingCell,
}

/// Encode every actual HUD cell in native row order, including its flips and colour set.
/// HUD cells always have foreground priority in this output format.
///
/// # Errors
/// An absent cell rejects the complete output instead of filling or omitting it.
pub fn tilemap(state: &HudState) -> Result<Vec<u8>, TilemapFault> {
    let mut bytes = Vec::with_capacity(192);
    for y in 0_i32..3 {
        for x in 0_i32..32 {
            let tile = state
                .grid
                .get(Place { x, y })
                .ok_or(TilemapFault::MissingCell)?;
            let flips = match tile.look.flip {
                interface_hud::Flip::None => 0,
                interface_hud::Flip::Across => 0x4000,
                interface_hud::Flip::Down => 0x8000,
                interface_hud::Flip::Both => 0xc000,
            };
            let value = tile.look.picture.index()
                | (u16::from(tile.colours.number()) << 10)
                | 0x2000
                | flips;
            bytes.extend_from_slice(&value.to_le_bytes());
        }
    }
    Ok(bytes)
}

/// Complete initial HUD state before staged work begins.
pub struct HudPin;
impl Pinned for HudPin {
    type State = HudState;
    const NAME: &'static str = "production-frame-hud";
    const SHAPE: u64 = 0xCEC1_0497_AE70_9FF5;
}

/// Complete actual HUD work, including chosen answers and ordered produced events.
pub struct WorkPin;
impl Pinned for WorkPin {
    type State = interface_hud::HudWork;
    const NAME: &'static str = "production-frame-hud-work";
    const SHAPE: u64 = 17_383_438_775_381_115_216;
}

/// One current HUD authority; staged work retains its complete state and publications.
#[derive(Canonical, Clone, Debug, PartialEq, Eq)]
pub enum Hud {
    /// Independently admitted complete quantities, before ordinary work begins.
    Initial(Box<Model<HudPin>>),
    /// Current complete quantities, consumed and pending answers, and actual events.
    Work(Box<Model<WorkPin>>),
}

/// Invalid ordinary HUD scheduling leaves the retained owner unchanged.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum WorkFault {
    /// An unfinished ordinary update cannot be replaced by a new one.
    Busy,
}

impl Hud {
    /// Retain the current complete HUD without observing controls or display inputs.
    #[must_use]
    pub fn new(state: HudState) -> Self {
        Self::Initial(Box::new(Model(state)))
    }

    /// Current complete HUD quantities, including effects completed during staged work.
    #[must_use]
    pub fn state(&self) -> &HudState {
        match self {
            Self::Initial(state) => &state.0,
            Self::Work(work) => work.0.state(),
        }
    }

    /// Actual cumulative events; inspection is not external delivery.
    #[must_use]
    pub fn events(&self) -> &[interface_hud::HudEvent] {
        match self {
            Self::Initial(_) => &[],
            Self::Work(work) => work.0.events(),
        }
    }

    /// Begin ordinary work without reading future controls, content or display inputs.
    ///
    /// # Errors
    /// An unfinished update rejects replacement before changing the owner.
    pub fn begin(&mut self) -> Result<(), WorkFault> {
        if matches!(self, Self::Work(work) if work.0.publication().is_none()) {
            return Err(WorkFault::Busy);
        }
        *self = Self::Work(Box::new(Model(interface_hud::HudWork::new(
            self.state().clone(),
        ))));
        Ok(())
    }

    /// Begin the actual separated HUD frame using its independently admitted frame clock.
    pub fn begin_staged(&mut self, frame: interface_hud::FrameCount) -> bool {
        let Self::Initial(state) = self else {
            return false;
        };
        *self = Self::Work(Box::new(Model(interface_hud::HudWork::begin(
            state.0.clone(),
            frame,
        ))));
        true
    }

    pub(in crate::production) fn choose_map(
        &mut self,
        rules: &Rules,
        pictures: &HudPictures,
        inputs: interface_hud::MovementInputs,
    ) -> bool {
        let Self::Work(work) = self else {
            return false;
        };
        work.0.choose_mini_map(rules, pictures, inputs)
    }

    pub(in crate::production) fn map_ready(&self) -> bool {
        self.applied_map().is_some()
    }

    /// Completed current Map answer with the Movement position used at its choice.
    #[must_use]
    pub fn applied_map(&self) -> Option<&interface_hud::MapAnswer> {
        let Self::Work(work) = self else {
            return None;
        };
        if let interface_hud::Observation::Completed(answer) = work.0.mini_map() {
            Some(answer)
        } else {
            None
        }
    }

    pub(in crate::production) fn choose_ordinary_status(
        &mut self,
        rules: &Rules,
        pictures: &HudPictures,
        inputs: interface_hud::StatusInputs,
    ) -> bool {
        let Self::Work(work) = self else {
            return false;
        };
        work.0.choose_status(rules, pictures, inputs)
    }

    /// Next actual controls or display opportunity; no inputs are captured by inspection.
    #[must_use]
    pub fn opportunity(&self) -> Option<interface_hud::HudOpportunity> {
        match self {
            Self::Initial(_) => None,
            Self::Work(work) => work.0.opportunity(),
        }
    }

    /// Current control observation and any retained actual answer.
    #[must_use]
    pub fn controls(&self) -> Option<&interface_hud::Observation<interface_hud::ControlAnswer>> {
        match self {
            Self::Initial(_) => None,
            Self::Work(work) => Some(work.0.controls()),
        }
    }

    /// Current display observation and its retained inputs, rules and content.
    #[must_use]
    pub fn display(&self) -> Option<&interface_hud::Observation<interface_hud::DisplayAnswer>> {
        match self {
            Self::Initial(_) => None,
            Self::Work(work) => Some(work.0.display()),
        }
    }

    /// Choose actual current controls once, before applying their effects.
    pub fn choose_controls(&mut self, inputs: interface_hud::ItemControlInputs) -> bool {
        match self {
            Self::Initial(_) => false,
            Self::Work(work) => work.0.choose_controls(inputs),
        }
    }

    /// Choose fresh display inputs and content only at the actual display opportunity.
    pub fn choose_display(
        &mut self,
        rules: &Rules,
        pictures: &HudPictures,
        inputs: interface_hud::DisplayInputs,
    ) -> bool {
        match self {
            Self::Initial(_) => false,
            Self::Work(work) => work.0.choose_display(rules, pictures, inputs),
        }
    }

    /// Apply one chosen actual answer once; absent or consumed answers return false.
    pub fn apply(&mut self) -> bool {
        match self {
            Self::Initial(_) => false,
            Self::Work(work) => work.0.apply(),
        }
    }

    /// Only completed ordinary work exposes full state and ordered events for publication.
    #[must_use]
    pub fn publication(&self) -> Option<(&HudState, &[interface_hud::HudEvent])> {
        match self {
            Self::Initial(_) => None,
            Self::Work(work) => work.0.publication(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use interface_hud::{
        Ammunition, CancelButton, DisplayInputs, Dropped, Energy, Equipped, FrameCount,
        HudOpportunity, ItemControlInputs, MiniMapPower, Reserve, ReserveMode, Room, SelectButton,
        SelectableItem, Selection, Shown, Status,
    };

    fn owner() -> Hud {
        let mut explored = interface_hud::MapCellSet::default();
        explored.insert(Place { x: 63, y: 31 });
        Hud::new(HudState {
            countdown: interface_hud::Countdown::Dormant,
            grid: interface_hud::HudGrid::filled(HudPictures::default().blank_cell()),
            shown: Shown {
                energy: 321,
                ..Shown::default()
            },
            selection: Selection::None,
            explored,
            mini_map: MiniMapPower::Off,
        })
    }

    #[test]
    fn initial_hud_shape_remains_pinned() {
        assert_eq!(canonical::model::pin_holds::<HudPin>(), Ok(()));
    }

    #[test]
    fn complete_work_shape_remains_pinned() {
        assert_eq!(canonical::model::pin_holds::<WorkPin>(), Ok(()));
    }

    #[test]
    fn fresh_restores_keep_chosen_controls_and_later_fresh_display_once() {
        restore_controls_then_display(
            Equipped {
                grapple_beam: true,
                x_ray_scope: false,
            },
            Selection::OneShot(SelectableItem::GrappleBeam),
            &[
                interface_hud::HudEvent::Switched {
                    item: Some(SelectableItem::GrappleBeam),
                },
                interface_hud::HudEvent::Click,
            ],
        );
        restore_controls_then_display(
            Equipped::default(),
            Selection::None,
            &[interface_hud::HudEvent::Switched { item: None }],
        );
    }

    fn restore_controls_then_display(
        equipped: Equipped,
        selection: Selection,
        expected_events: &[interface_hud::HudEvent],
    ) {
        let hud = owner();
        let bytes = canonical::to_bytes(&hud);
        drop(hud);
        let mut hud: Hud = canonical::from_bytes(&bytes).unwrap();
        let initial = hud.state().clone();
        hud.begin().unwrap();
        let controls = ItemControlInputs {
            select: SelectButton::Pressed,
            cancel: CancelButton::Held,
            can_switch: true,
            ammunition: Ammunition::new(enum_map::EnumMap::default()).unwrap(),
            equipped,
        };
        assert!(hud.choose_controls(controls));
        assert_eq!(hud.state(), &initial);
        assert_eq!(hud.events(), []);
        assert!(hud.publication().is_none());
        let bytes = canonical::to_bytes(&hud);
        drop(hud);
        let mut hud: Hud = canonical::from_bytes(&bytes).unwrap();
        assert!(hud.apply());
        assert_eq!(hud.state().grid, initial.grid);
        assert_eq!(hud.state().explored, initial.explored);
        assert_eq!(hud.state().selection, selection);
        assert_eq!(hud.opportunity(), Some(HudOpportunity::DisplayRefresh));
        assert!(!hud.choose_controls(controls));
        assert_eq!(hud.begin(), Err(WorkFault::Busy));
        let bytes = canonical::to_bytes(&hud);
        drop(hud);
        let mut hud: Hud = canonical::from_bytes(&bytes).unwrap();
        let display = DisplayInputs {
            frame: FrameCount(0),
            status: Status {
                energy: Energy {
                    now: 123,
                    capacity: 399,
                },
                reserve: Reserve {
                    energy: 0,
                    mode: ReserveMode::Manual,
                },
                ammunition: Ammunition::new(enum_map::EnumMap::default()).unwrap(),
                equipped: Equipped::default(),
            },
            muted_by: None,
            dropped_before_status_bar: Dropped::Nothing,
            dropped_after_status_bar: Dropped::Nothing,
            collected: None,
            mother_brain_falls: false,
            runs: true,
            turned_on: true,
            position: types::Pair { x: 384, y: 128 },
            room: Room {
                cell: Place { x: 2, y: 2 },
                blocks: types::Pair { x: 32, y: 16 },
            },
            map: interface_hud::HerMap::default(),
            map_collected: false,
        };
        assert!(hud.choose_display(&Rules::reference(), &HudPictures::default(), display));
        assert_eq!(hud.state().shown.energy, 321);
        let bytes = canonical::to_bytes(&hud);
        drop(hud);
        let mut hud: Hud = canonical::from_bytes(&bytes).unwrap();
        assert!(hud.apply());
        let (state, events) = hud.publication().unwrap();
        assert_eq!(state.shown.energy, 123);
        assert_eq!(state.mini_map, MiniMapPower::On);
        assert!(state.explored.contains(Place { x: 63, y: 31 }));
        assert!(state.explored.contains(Place { x: 3, y: 2 }));
        assert_eq!(events, expected_events);
        let finished = hud.clone();
        let bytes = canonical::to_bytes(&hud);
        drop(hud);
        let mut hud: Hud = canonical::from_bytes(&bytes).unwrap();
        assert_eq!(hud, finished);
        assert!(!hud.apply());
        assert_eq!(hud, finished);
    }
}
