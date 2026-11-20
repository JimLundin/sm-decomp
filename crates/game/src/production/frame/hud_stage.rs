//! Actual HUD observation choices and terminal producer publication.

use super::hud::{Hud, TilemapFault, WorkFault, tilemap};
use canonical::model::Pinned;
use canonical::{Canonical, model::Model};
use interface_hud::{
    DisplayInputs, Dropped, FrameCount, HerMap, HudEvent, HudPictures, ItemControlInputs,
    MiniMapPower, Mute, Observation, Room, Rules, SelectableItem, Selection, Status,
};
use serde::{Deserialize, Serialize};

/// Current non-movement reasons that suppress the switch click.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct MutePremises {
    /// The current time owner has stopped time.
    pub time_stopped: bool,
    /// The current weapons owner has the grapple beam out.
    pub grapple_out: bool,
}

impl MutePremises {
    /// Bridge the current selected movement action to the HUD's named click premise.
    #[must_use]
    pub const fn muted_by(self, action: types::Action) -> Option<Mute> {
        if self.time_stopped {
            return Some(Mute::TimeStopped);
        }
        if self.grapple_out {
            return Some(Mute::Grapple);
        }
        match action {
            types::Action::Spinning => Some(Mute::SpinJump),
            types::Action::WallJumping => Some(Mute::WallJump),
            types::Action::AgainstWall
            | types::Action::Crouching
            | types::Action::Falling
            | types::Action::Jumping
            | types::Action::Landing(_)
            | types::Action::Moonwalking
            | types::Action::Morphing
            | types::Action::Rolling
            | types::Action::Running
            | types::Action::StandingUp
            | types::Action::Still
            | types::Action::TakingOff
            | types::Action::Turning(_)
            | types::Action::Unmorphing => None,
        }
    }
}

/// Current event-producer receipts consumed by HUD refresh.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct RefreshEvents {
    /// Actual drop preceding the status bar.
    pub before_status_bar: Dropped,
    /// Actual drop following the status bar.
    pub after_status_bar: Dropped,
    /// Actual pickup publication.
    pub collected: Option<SelectableItem>,
    /// Current Mother Brain fall event.
    pub mother_brain_falls: bool,
    /// Current mini-map turn-on event.
    pub turned_on: bool,
}

/// Current external refresh premises, separate from movement and status owners.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct DisplayPremises {
    /// Actual completed-frame count read by this refresh.
    pub frame: FrameCount,
    /// Actual non-movement click suppression premises.
    pub mute: MutePremises,
    /// Actual gameplay disposition permits the mini-map to run.
    pub runs: bool,
    /// Current event-producer receipts.
    pub events: RefreshEvents,
    /// Current room on its map.
    pub room: Room,
    /// Complete checked current area map.
    pub map: HerMap,
    /// Current progress owner has collected this map.
    pub map_collected: bool,
}

/// Complete independently qualified borrowed refresh dependency.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct DisplayReceipt {
    /// Admitted external current owner facts and checked map content.
    pub premises: DisplayPremises,
    /// Admitted current status owner, requiring fresh revalidation at refresh.
    pub status: Status,
}

/// Complete current external refresh owner, excluding future movement and status inputs.
pub struct DisplayPremisesPin;
impl Pinned for DisplayPremisesPin {
    type State = Box<DisplayPremises>;
    const NAME: &'static str = "production-hud-current-display-premises";
    const SHAPE: u64 = 0x0283_0E79_9FA1_BC1B;
}

/// Complete independently borrowed refresh dependency and its current status premise.
pub struct DisplayReceiptPin;
impl Pinned for DisplayReceiptPin {
    type State = Box<DisplayReceipt>;
    const NAME: &'static str = "production-hud-admitted-display-receipt";
    const SHAPE: u64 = 0x78D6_8934_1237_C57C;
}

/// Complete chosen control inputs and the answer actually consumed before movement.
pub struct ControlAnswerPin;
impl Pinned for ControlAnswerPin {
    type State = interface_hud::ControlAnswer;
    const NAME: &'static str = "production-hud-completed-controls";
    const SHAPE: u64 = 0x6DED_6660_B8D0_EC62;
}

/// Actual early control observation consumed after its physical owner completes.
#[derive(Canonical, Clone, Debug, PartialEq, Eq)]
pub struct ControlsCompletion {
    /// Actual completed physical owner identity.
    pub work: crate::production::timing::WorkId,
    /// Full actual inputs, selection and ordered switch answer.
    pub answer: Model<ControlAnswerPin>,
}

/// Actual current owners joined at the qualified refresh opportunity.
#[derive(Clone, Copy)]
pub struct CurrentDisplay<'a> {
    /// Actual current external refresh premises.
    pub premises: &'a DisplayPremises,
    /// Actual current status owner.
    pub status: Status,
    /// Actual movement position after movement and before pose placement.
    pub position: types::Pair<u16>,
    /// Actual selected movement action at this opportunity.
    pub action: types::Action,
    /// Actual current event-owner identity and complete outputs.
    pub event_owner: Option<&'a RefreshEvents>,
}

impl DisplayPremises {
    /// Capture fresh display inputs from actual movement and status owners at use.
    /// No position or health is retained by the external premise descriptor.
    #[must_use]
    pub fn inputs(
        &self,
        status: Status,
        position: types::Pair<u16>,
        action: types::Action,
    ) -> DisplayInputs {
        DisplayInputs {
            frame: self.frame,
            status,
            muted_by: self.mute.muted_by(action),
            dropped_before_status_bar: self.events.before_status_bar,
            dropped_after_status_bar: self.events.after_status_bar,
            collected: self.events.collected,
            mother_brain_falls: self.events.mother_brain_falls,
            runs: self.runs,
            turned_on: self.events.turned_on,
            position,
            room: self.room,
            map: self.map.clone(),
            map_collected: self.map_collected,
        }
    }
}

/// Ordered requests and complete tile bytes from the actual terminal HUD producer.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct DisplayPublication {
    /// Every actual HUD tile in native row order.
    pub tilemap: Vec<u8>,
    /// Complete ordered HUD requests, including the earlier controls switch.
    pub events: Vec<HudEvent>,
}

/// Invalid HUD caller order rejects before changing the actual owner.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum StageFault {
    /// The actual observation has already been chosen or consumed, or is not offered.
    Order,
    /// A new update would replace an unfinished actual update.
    Work(WorkFault),
    /// The completed actual grid cannot be encoded in full.
    Tilemap(TilemapFault),
    /// An admitted complete current dependency has changed before use.
    StaleReceipt,
    /// The actual current event owner is absent.
    MissingEventOwner,
    /// The source's quiet-event equivalence requires an unselected powered-on HUD.
    QuietEquivalence,
}

/// Begin this update and retain only current controls before movement.
/// # Errors
/// Rejects unfinished prior work or an unavailable controls observation atomically.
pub fn choose_controls(hud: &mut Hud, inputs: ItemControlInputs) -> Result<(), StageFault> {
    let mut candidate = hud.clone();
    if candidate.opportunity() != Some(interface_hud::HudOpportunity::ItemControls) {
        candidate.begin().map_err(StageFault::Work)?;
    }
    if !candidate.choose_controls(inputs) {
        return Err(StageFault::Order);
    }
    *hud = candidate;
    Ok(())
}

/// Consume the retained actual controls answer once before its dependent readers.
/// # Errors
/// Missing, unread or completed controls reject without applying another observation.
pub fn complete_controls(hud: &mut Hud) -> Result<Selection, StageFault> {
    if !matches!(hud.controls(), Some(Observation::Chosen(_))) {
        return Err(StageFault::Order);
    }
    let mut candidate = hud.clone();
    if !candidate.apply() {
        return Err(StageFault::Order);
    }
    let selection = candidate.state().selection;
    *hud = candidate;
    Ok(selection)
}

/// Consume controls and retain their full actual completed answer for later owners.
/// The caller supplies the identity returned by its qualified physical operation.
/// # Errors
/// Missing, repeated or inconsistent controls leave the current HUD unchanged.
pub fn complete_controls_receipt(
    hud: &mut Hud,
    work: crate::production::timing::WorkId,
) -> Result<ControlsCompletion, StageFault> {
    let mut candidate = hud.clone();
    let selection = complete_controls(&mut candidate)?;
    let Some(Observation::Completed(answer)) = candidate.controls() else {
        return Err(StageFault::Order);
    };
    if selection != answer.selection {
        return Err(StageFault::Order);
    }
    let result = ControlsCompletion {
        work,
        answer: Model(answer.clone()),
    };
    *hud = candidate;
    Ok(result)
}

/// Retain fresh current display inputs after movement, with current content and rules.
/// # Errors
/// Early, repeated or completed display choices reject without changing the owner.
pub fn choose_display(
    hud: &mut Hud,
    rules: &Rules,
    pictures: &HudPictures,
    inputs: DisplayInputs,
) -> Result<(), StageFault> {
    let mut candidate = hud.clone();
    if !candidate.choose_display(rules, pictures, inputs) {
        return Err(StageFault::Order);
    }
    *hud = candidate;
    Ok(())
}

/// Revalidate the complete qualified current-owner join before choosing fresh display.
/// The source's event equivalence is conditional on current HUD state, never a pad gate.
/// # Errors
/// Missing or changed owners and invalid HUD/event predicates reject before any choice.
pub fn choose_admitted_display(
    hud: &mut Hud,
    rules: &Rules,
    pictures: &HudPictures,
    current: CurrentDisplay<'_>,
    receipt: &DisplayReceipt,
) -> Result<(), StageFault> {
    let events = current.event_owner.ok_or(StageFault::MissingEventOwner)?;
    if current.premises != &receipt.premises
        || current.status != receipt.status
        || events != &current.premises.events
    {
        return Err(StageFault::StaleReceipt);
    }
    if hud.state().selection != Selection::None
        || hud.state().mini_map != MiniMapPower::On
        || events.before_status_bar != Dropped::Nothing
        || events.after_status_bar != Dropped::Nothing
        || events.collected.is_some()
        || events.mother_brain_falls
        || events.turned_on
    {
        return Err(StageFault::QuietEquivalence);
    }
    choose_display(
        hud,
        rules,
        pictures,
        current
            .premises
            .inputs(current.status, current.position, current.action),
    )
}

/// Publish full tile bytes and ordered requests at actual terminal refresh completion.
/// The caller retains and charges the physical return tail after this producer is Ready.
/// # Errors
/// Missing, unread or completed refresh and incomplete grids reject the entire publication.
pub fn complete_display(hud: &mut Hud) -> Result<DisplayPublication, StageFault> {
    if !matches!(hud.display(), Some(Observation::Chosen(_))) {
        return Err(StageFault::Order);
    }
    let mut candidate = hud.clone();
    if !candidate.apply() {
        return Err(StageFault::Order);
    }
    let (state, events) = candidate.publication().ok_or(StageFault::Order)?;
    let publication = DisplayPublication {
        tilemap: tilemap(state).map_err(StageFault::Tilemap)?,
        events: events.to_vec(),
    };
    *hud = candidate;
    Ok(publication)
}

#[cfg(test)]
#[path = "hud_stage/controls_tests.rs"]
mod controls_tests;

#[cfg(test)]
mod tests {
    use super::*;
    use interface_hud::{
        Ammunition, CancelButton, Dropped, Energy, Equipped, FrameCount, HerMap, HudGrid, HudState,
        MapCellSet, MiniMapPower, Place, Reserve, ReserveMode, Room, SelectButton, Shown, Status,
    };

    #[test]
    fn complete_current_display_shapes_remain_pinned() {
        assert_eq!(canonical::model::pin_holds::<DisplayPremisesPin>(), Ok(()));
        assert_eq!(canonical::model::pin_holds::<DisplayReceiptPin>(), Ok(()));
    }

    #[test]
    fn current_action_and_independent_mute_owners_are_joined_at_use() {
        let clear = MutePremises {
            time_stopped: false,
            grapple_out: false,
        };
        assert_eq!(clear.muted_by(types::Action::Running), None);
        assert_eq!(
            clear.muted_by(types::Action::Spinning),
            Some(Mute::SpinJump)
        );
        assert_eq!(
            clear.muted_by(types::Action::WallJumping),
            Some(Mute::WallJump)
        );
        assert_eq!(
            MutePremises {
                grapple_out: true,
                ..clear
            }
            .muted_by(types::Action::Running),
            Some(Mute::Grapple)
        );
        assert_eq!(
            MutePremises {
                time_stopped: true,
                ..clear
            }
            .muted_by(types::Action::Running),
            Some(Mute::TimeStopped)
        );
    }

    fn controls() -> ItemControlInputs {
        ItemControlInputs {
            select: SelectButton::Pressed,
            cancel: CancelButton::NotPressed,
            can_switch: true,
            ammunition: Ammunition::new(enum_map::EnumMap::default()).unwrap(),
            equipped: Equipped::default(),
        }
    }

    fn display() -> DisplayInputs {
        DisplayInputs {
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
                ammunition: controls().ammunition,
                equipped: Equipped::default(),
            },
            muted_by: None,
            dropped_before_status_bar: Dropped::Nothing,
            dropped_after_status_bar: Dropped::Nothing,
            collected: None,
            mother_brain_falls: false,
            runs: true,
            turned_on: false,
            position: types::Pair { x: 384, y: 128 },
            room: Room {
                cell: Place { x: 2, y: 2 },
                blocks: types::Pair { x: 32, y: 16 },
            },
            map: HerMap::default(),
            map_collected: false,
        }
    }

    #[test]
    fn phase_admission_and_terminal_publication_are_atomic_and_exact_once() {
        let pictures = HudPictures::default();
        let mut hud = Hud::new(HudState {
            countdown: interface_hud::Countdown::Dormant,
            grid: HudGrid::filled(pictures.blank_cell()),
            shown: Shown {
                energy: 321,
                ..Shown::default()
            },
            selection: Selection::None,
            explored: MapCellSet::default(),
            mini_map: MiniMapPower::On,
        });
        let initial = hud.clone();
        assert_eq!(complete_controls(&mut hud), Err(StageFault::Order));
        assert_eq!(
            choose_display(&mut hud, &Rules::reference(), &pictures, display()),
            Err(StageFault::Order)
        );
        assert_eq!(hud, initial);
        choose_controls(&mut hud, controls()).unwrap();
        let chosen = hud.clone();
        assert_eq!(complete_display(&mut hud), Err(StageFault::Order));
        assert_eq!(hud, chosen);
        let bytes = canonical::to_bytes(&hud);
        drop(hud);
        let mut hud: Hud = canonical::from_bytes(&bytes).unwrap();
        assert_eq!(complete_controls(&mut hud), Ok(Selection::None));
        assert_eq!(complete_controls(&mut hud), Err(StageFault::Order));
        assert_eq!(hud.state().shown.energy, 321);
        assert_eq!(hud.events(), [HudEvent::Switched { item: None }]);
        let bytes = canonical::to_bytes(&hud);
        drop(hud);
        let mut hud: Hud = canonical::from_bytes(&bytes).unwrap();
        let external = display();
        let premises = DisplayPremises {
            frame: external.frame,
            mute: MutePremises {
                time_stopped: false,
                grapple_out: false,
            },
            runs: external.runs,
            events: RefreshEvents {
                before_status_bar: external.dropped_before_status_bar,
                after_status_bar: external.dropped_after_status_bar,
                collected: external.collected,
                mother_brain_falls: external.mother_brain_falls,
                turned_on: external.turned_on,
            },
            room: external.room,
            map: external.map,
            map_collected: external.map_collected,
        };
        let mut current_status = external.status;
        current_status.energy.now = 117;
        let current_position = types::Pair { x: 400, y: 128 };
        choose_fresh_display(
            &mut hud,
            &pictures,
            &premises,
            current_status,
            current_position,
        );
        assert_eq!(hud.state().shown.energy, 321);
        let bytes = canonical::to_bytes(&hud);
        drop(hud);
        let mut hud: Hud = canonical::from_bytes(&bytes).unwrap();
        let publication = complete_display(&mut hud).unwrap();
        assert_eq!(publication.tilemap.len(), 192);
        assert_eq!(publication.events, [HudEvent::Switched { item: None }]);
        assert_eq!(hud.state().shown.energy, 117);
        let completed = hud.clone();
        assert_eq!(complete_display(&mut hud), Err(StageFault::Order));
        assert_eq!(hud, completed);
    }

    fn choose_fresh_display(
        hud: &mut Hud,
        pictures: &HudPictures,
        premises: &DisplayPremises,
        current_status: Status,
        current_position: types::Pair<u16>,
    ) {
        let event_owner = premises.events;
        let receipt = DisplayReceipt {
            premises: premises.clone(),
            status: current_status,
        };
        let before = hud.clone();
        assert_eq!(
            choose_admitted_display(
                hud,
                &Rules::reference(),
                pictures,
                CurrentDisplay {
                    premises,
                    status: current_status,
                    position: current_position,
                    action: types::Action::Running,
                    event_owner: None
                },
                &receipt,
            ),
            Err(StageFault::MissingEventOwner)
        );
        let mut stale_status = current_status;
        stale_status.energy.now = 99;
        assert_eq!(
            choose_admitted_display(
                hud,
                &Rules::reference(),
                pictures,
                CurrentDisplay {
                    premises,
                    status: stale_status,
                    position: current_position,
                    action: types::Action::Running,
                    event_owner: Some(&event_owner)
                },
                &receipt,
            ),
            Err(StageFault::StaleReceipt)
        );
        assert_eq!(*hud, before);
        choose_admitted_display(
            hud,
            &Rules::reference(),
            pictures,
            CurrentDisplay {
                premises,
                status: current_status,
                position: current_position,
                action: types::Action::Running,
                event_owner: Some(&event_owner),
            },
            &receipt,
        )
        .unwrap();
    }
}
