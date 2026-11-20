//! Spec: ItemControls, MiniMapUpdate, StatusRefresh, RetainedHudWork.

use serde::{Deserialize, Serialize};
use types::Pair;

use crate::events::HudEvent;
use crate::inputs::{
    Dropped, FrameCount, FrameInputs, ItemControlInputs, MiniMapInputs, Mute, Room, Status,
    StatusInputs,
};
use crate::item::SelectableItem;
use crate::map::HerMap;
use crate::pictures::HudPictures;
use crate::rules::Rules;
use crate::state::{HudState, Selection};

/// Spec: RetainedHudWork. An offered observation has no answer; a chosen one has not yet
/// applied its effects; a completed one retains its consumed answer.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum Observation<T> {
    /// Receive current inputs when choosing, not when inspecting this offer.
    Unread,
    /// The actual answer, held until applied once.
    Chosen(T),
    /// The consumed answer; effects are already in the current State and events.
    Completed(T),
}

/// Spec: RetainedHudWork. Which semantic observation can next be chosen or applied.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum HudOpportunity {
    /// Controls before movement; no map or grid changes.
    ItemControls,
    /// Movement position before later pose placement.
    MiniMapUpdate,
    /// Current status quantities after intervening play.
    StatusRefresh,
    /// Compatibility offer for callers supplying both drawing observations together.
    DisplayRefresh,
}

/// Spec: ItemControls. Actual chosen inputs and the selection/event they produced, before effects.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct ControlAnswer {
    /// Controls and availability actually read.
    pub inputs: ItemControlInputs,
    /// The selected and one-shot item to apply.
    pub selection: Selection,
    /// The ordered switch, including switching to the same item or none.
    pub switched: Option<HudEvent>,
}

/// Bundled compatibility inputs for MiniMapUpdate and StatusRefresh; no controls are read.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[expect(
    clippy::struct_excessive_bools,
    reason = "independent current input facts and events, not continuation state flags"
)]
pub struct DisplayInputs {
    /// The frame count used for flash and blink.
    pub frame: FrameCount,
    /// Current counts, capacities and reserve mode.
    pub status: Status,
    /// Current click suppression reason.
    pub muted_by: Option<Mute>,
    /// The drop before drawing.
    pub dropped_before_status_bar: Dropped,
    /// The drop after drawing.
    pub dropped_after_status_bar: Dropped,
    /// Current pickup event.
    pub collected: Option<SelectableItem>,
    /// Current Mother Brain fall event.
    pub mother_brain_falls: bool,
    /// Whether the mini-map runs.
    pub runs: bool,
    /// Current turn-on event, preceding the fall.
    pub turned_on: bool,
    /// Position after movement, before pose placement.
    pub position: Pair<u16>,
    /// The current room.
    pub room: Room,
    /// Full map pictures, beyond pictures and charted cells.
    pub map: HerMap,
    /// Current collected-map fact.
    pub map_collected: bool,
}

impl DisplayInputs {
    /// Capture both drawing observations of an already complete frame input.
    #[must_use]
    pub fn from_frame(inputs: &FrameInputs<'_>) -> Self {
        Self {
            frame: inputs.frame,
            status: inputs.status,
            muted_by: inputs.controls.muted_by,
            dropped_before_status_bar: inputs.dropped_before_status_bar,
            dropped_after_status_bar: inputs.dropped_after_status_bar,
            collected: inputs.collected,
            mother_brain_falls: inputs.mother_brain_falls,
            runs: inputs.mini_map.runs,
            turned_on: inputs.mini_map.turned_on,
            position: inputs.mini_map.position,
            room: inputs.mini_map.room,
            map: inputs.mini_map.map.clone(),
            map_collected: inputs.mini_map.map_collected,
        }
    }

    /// Spec: StatusRefresh. Extract the late quantities from a bundled observation.
    #[must_use]
    pub const fn status_inputs(&self) -> StatusInputs {
        StatusInputs {
            energy: self.status.energy,
            reserve: self.status.reserve,
            ammunition: self.status.ammunition,
            muted_by: self.muted_by,
            collected: self.collected,
            dropped_before_status_bar: self.dropped_before_status_bar,
            dropped_after_status_bar: self.dropped_after_status_bar,
        }
    }

    /// Spec: MiniMapUpdate. Borrow the retained map for the existing map rules.
    #[must_use]
    pub const fn mini_map(&self) -> MiniMapInputs<'_> {
        MiniMapInputs {
            runs: self.runs,
            turned_on: self.turned_on,
            position: self.position,
            room: self.room,
            map: &self.map,
            map_collected: self.map_collected,
        }
    }
}

/// The bundled compatibility observation, including its content and rules.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct DisplayAnswer {
    /// Current inputs at choice, retained unchanged until application.
    pub inputs: DisplayInputs,
    /// Content at this choice, not at frame construction.
    pub pictures: HudPictures,
    /// Rules at this choice.
    pub rules: Rules,
}

/// Spec: MiniMapUpdate. The owned movement observation and its pending or applied result.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct MapAnswer {
    /// The actual movement inputs.
    pub inputs: MovementInputs,
    /// The rules at movement choice.
    pub rules: Rules,
    /// The HUD pictures at movement choice.
    pub pictures: HudPictures,
    /// Resulting carried HUD, including the unchanged status bar and caches.
    pub after: HudState,
}

/// Spec: MiniMapUpdate. The movement inputs, owned so a chosen map survives replacement.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[expect(
    clippy::struct_excessive_bools,
    reason = "independent map conditions and events"
)]
pub struct MovementInputs {
    /// Whether the caller allows the map to update.
    pub runs: bool,
    /// Turn-on event, applied before any fall.
    pub turned_on: bool,
    /// Mother Brain's first form falls during this frame.
    pub mother_brain_falls: bool,
    /// Whole pixels after movement, before pose placement.
    pub position: Pair<u16>,
    /// Current room and its dimensions.
    pub room: Room,
    /// The current area's pictures and charted cells.
    pub map: HerMap,
    /// Whether charted cells are visible even if unexplored.
    pub map_collected: bool,
}

impl MovementInputs {
    /// Capture current movement inputs from the caller's map and movement observation.
    #[must_use]
    pub fn capture(inputs: &MiniMapInputs<'_>, mother_brain_falls: bool) -> Self {
        Self {
            runs: inputs.runs,
            turned_on: inputs.turned_on,
            mother_brain_falls,
            position: inputs.position,
            room: inputs.room,
            map: inputs.map.clone(),
            map_collected: inputs.map_collected,
        }
    }

    /// Borrow this observation as inputs to the mini-map rules.
    #[must_use]
    pub const fn as_mini_map(&self) -> MiniMapInputs<'_> {
        MiniMapInputs {
            runs: self.runs,
            turned_on: self.turned_on,
            position: self.position,
            room: self.room,
            map: &self.map,
            map_collected: self.map_collected,
        }
    }
}

/// Spec: StatusRefresh. The actual late observation, its resulting HUD, and any click request.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct StatusAnswer {
    /// The actual late status quantities and events.
    pub inputs: StatusInputs,
    /// The rules at status choice.
    pub rules: Rules,
    /// The HUD pictures at status choice.
    pub pictures: HudPictures,
    /// Resulting complete HUD, including the already applied mini-map.
    pub after: HudState,
    /// The late request, separate from any earlier switch.
    pub click: Option<HudEvent>,
}

/// Spec: RetainedHudWork. Caller-owned position in one frame's ordered event history.
///
/// Keep a separate cursor for each independent reader, or pass the same cursor from the early
/// reader to the late reader. Start a new cursor for each frame. Save it alongside the work if
/// delivery must resume after restoration.
#[derive(Clone, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct EventCursor {
    delivered: usize,
}

impl EventCursor {
    /// Take the next available request and advance this reader only; no request is removed
    /// from the frame's history. An empty read leaves the cursor unchanged.
    pub fn next_event(&mut self, work: &HudWork) -> Option<HudEvent> {
        let event = *work.events().get(self.delivered)?;
        self.delivered = self.delivered.saturating_add(1);
        Some(event)
    }
}

/// Spec: RetainedHudWork. Retain one frame across controls, movement and status.
///
/// Begin with [`Self::begin`] and the frame-start count. Inspect [`Self::opportunity`],
/// choose with current inputs and inspect `controls`, `mini_map` or
/// `status` for the actual answer before calling [`Self::apply`]. Controls expose selection
/// and the switch first. Choose movement after movement and before pose placement; applying it
/// changes the carried map but not status. Then choose status using the current quantities.
/// Unread opportunities capture nothing; a chosen answer cannot be overwritten or replayed.
///
/// [`Self::state`] and [`Self::events`] expose applied partial progress. [`Self::publication`]
/// exposes the complete HUD and ordered history together only after status; the caller owns
/// publication. [`EventCursor`] lets a reader take new requests without removing history.
/// Serialize the entire work and any delivery cursors, discard the originals, and deserialize
/// fresh values to continue. Restoring the HUD alone loses pending work and consumed progress.
/// [`Self::new`] and [`Self::choose_display`] remain the bundled compatibility path for callers
/// with both drawing observations ready together.
///
/// ```
/// # use interface_hud::{EventCursor, FrameCount, HudPictures, HudState, HudWork, ItemControlInputs, MovementInputs, Rules, StatusInputs};
/// # fn frame(hud: HudState, count: FrameCount, controls: ItemControlInputs,
/// #          movement: MovementInputs, status: StatusInputs, rules: &Rules, pictures: &HudPictures) {
/// let mut work = HudWork::begin(hud, count);
/// let mut delivery = EventCursor::default();
/// assert!(work.choose_controls(controls));
/// let _answer = work.controls();
/// assert!(work.apply());
/// let _switch = delivery.next_event(&work); // available before movement
/// // Obtain movement inputs after moving, before pose placement.
/// assert!(work.choose_mini_map(rules, pictures, movement));
/// let _answer = work.mini_map();
/// assert!(work.apply());
/// let _partial_hud = work.state(); // map visible; status still unchanged
/// // Obtain current status inputs after intervening play.
/// assert!(work.choose_status(rules, pictures, status));
/// let _answer = work.status();
/// assert!(work.apply());
/// let _click = delivery.next_event(&work); // only a new request, if one was raised
/// let (_hud, _ordered_history) = work.publication().expect("status complete");
/// # }
/// ```
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct HudWork {
    state: HudState,
    initial_highlight: Option<SelectableItem>,
    controls: Observation<ControlAnswer>,
    display: Observation<DisplayAnswer>,
    frame: Option<FrameCount>,
    mini_map: Observation<Box<MapAnswer>>,
    status: Observation<StatusAnswer>,
    events: Vec<HudEvent>,
}

impl HudWork {
    /// Begin the compatibility path; its bundled inputs supply the frame-start count.
    /// For separately chosen movement and status, use [`Self::begin`].
    #[must_use]
    pub const fn new(state: HudState) -> Self {
        Self {
            initial_highlight: state.shown.highlighted,
            state,
            controls: Observation::Unread,
            display: Observation::Unread,
            frame: None,
            mini_map: Observation::Unread,
            status: Observation::Unread,
            events: Vec::new(),
        }
    }

    /// Spec: RetainedHudWork. Latch the frame-start count for both blink and flash.
    #[must_use]
    pub const fn begin(state: HudState, frame: FrameCount) -> Self {
        let mut work = Self::new(state);
        work.frame = Some(frame);
        work
    }

    /// Frame-start count, or none on the compatibility path before display choice.
    #[must_use]
    pub const fn frame_count(&self) -> Option<FrameCount> {
        self.frame
    }

    /// Spec: RetainedHudWork. Inspect without capturing any inputs.
    #[must_use]
    pub const fn opportunity(&self) -> Option<HudOpportunity> {
        if !matches!(self.controls, Observation::Completed(_)) {
            Some(HudOpportunity::ItemControls)
        } else if !matches!(self.display, Observation::Unread) || self.frame.is_none() {
            if matches!(self.display, Observation::Completed(_)) {
                None
            } else {
                Some(HudOpportunity::DisplayRefresh)
            }
        } else if !matches!(self.mini_map, Observation::Completed(_)) {
            Some(HudOpportunity::MiniMapUpdate)
        } else if !matches!(self.status, Observation::Completed(_)) {
            Some(HudOpportunity::StatusRefresh)
        } else {
            None
        }
    }

    /// Spec: ItemControls. Choose from current inputs. Repeated or out-of-order choices return
    /// false and cannot reconsider an answer. This computes no drawing or map effects.
    pub fn choose_controls(&mut self, inputs: ItemControlInputs) -> bool {
        if !matches!(self.controls, Observation::Unread) {
            return false;
        }
        let mut selection = self.state.selection;
        let switched = selection.apply_item_controls(&inputs);
        self.controls = Observation::Chosen(ControlAnswer {
            inputs,
            selection,
            switched,
        });
        true
    }

    /// Choose both drawing observations together on the compatibility path, after controls.
    /// A chosen answer cannot be overwritten by a later placement or external input.
    pub fn choose_display(
        &mut self,
        rules: &Rules,
        pictures: &HudPictures,
        inputs: DisplayInputs,
    ) -> bool {
        if self.opportunity() != Some(HudOpportunity::DisplayRefresh)
            || !matches!(self.display, Observation::Unread)
        {
            return false;
        }
        self.frame = Some(inputs.frame);
        self.display = Observation::Chosen(DisplayAnswer {
            inputs,
            pictures: *pictures,
            rules: rules.clone(),
        });
        true
    }

    /// Spec: MiniMapUpdate. Choose at the movement position. The answer includes the actual
    /// resulting window, exploration and map power; none is applied until `apply`.
    pub fn choose_mini_map(
        &mut self,
        rules: &Rules,
        pictures: &HudPictures,
        inputs: MovementInputs,
    ) -> bool {
        if self.opportunity() != Some(HudOpportunity::MiniMapUpdate)
            || !matches!(self.mini_map, Observation::Unread)
        {
            return false;
        }
        let Some(frame) = self.frame else {
            return false;
        };
        let mut after = self.state.clone();
        after.movement_map(
            rules,
            pictures,
            &inputs.as_mini_map(),
            inputs.mother_brain_falls,
            frame,
        );
        self.mini_map = Observation::Chosen(Box::new(MapAnswer {
            inputs,
            rules: rules.clone(),
            pictures: *pictures,
            after,
        }));
        true
    }

    /// Spec: StatusRefresh. Choose only after movement effects are visible on the carried HUD.
    pub fn choose_status(
        &mut self,
        rules: &Rules,
        pictures: &HudPictures,
        inputs: StatusInputs,
    ) -> bool {
        if self.opportunity() != Some(HudOpportunity::StatusRefresh)
            || !matches!(self.status, Observation::Unread)
        {
            return false;
        }
        let Some(frame) = self.frame else {
            return false;
        };
        let mut after = self.state.clone();
        let click = after.refresh_status(rules, pictures, &inputs, frame, self.initial_highlight);
        self.status = Observation::Chosen(StatusAnswer {
            inputs,
            rules: rules.clone(),
            pictures: *pictures,
            after,
            click,
        });
        true
    }

    /// Spec: MiniMapUpdate. Inspect unread, chosen or completed movement work.
    #[must_use]
    pub fn mini_map(&self) -> Observation<&MapAnswer> {
        match &self.mini_map {
            Observation::Unread => Observation::Unread,
            Observation::Chosen(answer) => Observation::Chosen(answer),
            Observation::Completed(answer) => Observation::Completed(answer),
        }
    }

    /// Spec: StatusRefresh. Inspect unread, chosen or completed status work.
    #[must_use]
    pub const fn status(&self) -> &Observation<StatusAnswer> {
        &self.status
    }

    /// Spec: RetainedHudWork. Apply the current chosen answer once; unread or complete work
    /// returns false. Controls expose their switch before refresh, without grid/map effects.
    pub fn apply(&mut self) -> bool {
        if let Observation::Chosen(answer) = &self.controls {
            self.state.selection = answer.selection;
            self.events.extend(answer.switched);
            self.controls = Observation::Completed(answer.clone());
            return true;
        }
        if let Observation::Chosen(answer) = &self.mini_map {
            self.state = answer.after.clone();
            self.mini_map = Observation::Completed(answer.clone());
            return true;
        }
        if let Observation::Chosen(answer) = &self.status {
            self.state = answer.after.clone();
            self.events.extend(answer.click);
            self.status = Observation::Completed(answer.clone());
            return true;
        }
        self.apply_display()
    }

    fn apply_display(&mut self) -> bool {
        let Observation::Chosen(answer) = &self.display else {
            return false;
        };
        let inputs = &answer.inputs;
        self.state.movement_map(
            &answer.rules,
            &answer.pictures,
            &inputs.mini_map(),
            inputs.mother_brain_falls,
            inputs.frame,
        );
        self.events.extend(self.state.refresh_status(
            &answer.rules,
            &answer.pictures,
            &inputs.status_inputs(),
            inputs.frame,
            self.initial_highlight,
        ));
        self.display = Observation::Completed(answer.clone());
        true
    }

    /// Spec: RetainedHudWork. Current complete State, possibly only partially updated.
    #[must_use]
    pub const fn state(&self) -> &HudState {
        &self.state
    }

    /// Spec: RetainedHudWork. Controls offer, actual answer or consumed answer.
    #[must_use]
    pub const fn controls(&self) -> &Observation<ControlAnswer> {
        &self.controls
    }

    /// Spec: RetainedHudWork. Refresh offer, actual answer or consumed answer.
    #[must_use]
    pub const fn display(&self) -> &Observation<DisplayAnswer> {
        &self.display
    }

    /// Spec: SwitchClick. The frame-start shown highlight, never inferred from selection/cells.
    #[must_use]
    pub const fn initial_highlight(&self) -> Option<SelectableItem> {
        self.initial_highlight
    }

    /// Spec: RetainedHudWork. Cumulative completed events, not delivery acknowledgements.
    #[must_use]
    pub fn events(&self) -> &[HudEvent] {
        &self.events
    }

    /// Spec: StatusRefresh. Once complete, return the full HUD and ordered event history.
    #[must_use]
    pub fn publication(&self) -> Option<(&HudState, &[HudEvent])> {
        self.opportunity()
            .is_none()
            .then_some((&self.state, &self.events))
    }
}
