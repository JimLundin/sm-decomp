//! Current owner join for the scoped quiet map opportunity.

use super::{
    Caller, State,
    borrowed_gameplay::{GameplayRoute, SceneController},
    borrowed_player_preparation::{ActorContentDisposition, ActorScene, InstructionDisposition},
    hud::Hud,
    hud_stage::{DisplayPremises, DisplayReceipt, MutePremises, RefreshEvents},
};
use interface_hud::{
    Dropped, Energy, FrameCount, HerMap, Place, Reserve, ReserveMode, Room, Status,
};

use types::Parts;

/// Current event facts read by the early mini-map opportunity.
#[derive(Clone, Copy, Debug, PartialEq, Eq, serde::Deserialize)]
#[serde(deny_unknown_fields)]
pub struct MapEvents {
    /// The current actor route reports Mother Brain's fall.
    pub mother_brain_falls: bool,
    /// The current route turns the mini-map on this opportunity.
    pub turned_on: bool,
}

/// Carried external facts qualified from admission through the adjacent quiet map opportunity.
/// The centre and ordered explored window qualify the physical bill without supplying any output.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct RecurringDisplay {
    /// First adjacent opportunity in the independently qualified no-writer interval.
    pub frame: FrameCount,
    /// The carried quiet-panel source qualifies later first-half blink opportunities.
    pub retain_first_half: bool,
    /// Admitted stable room on the checked area map.
    pub room: Room,
    /// Checked centre of the bounded ordinary one-page arm.
    pub centre: Place,
    /// Row-major exploration at the checked centre after marking the centre itself.
    pub explored: [bool; 15],
    /// Stable energy carrying capacity.
    pub energy_capacity: u16,
    /// Stable manual, empty reserve.
    pub reserve: Reserve,
    /// Current non-movement silence premises in the no-writer interval.
    pub mute: MutePremises,
    /// Qualified current map events from the ordinary in-play Ceres route.
    pub map_events: MapEvents,
    /// Independently preserved disposition to draw the map in ordinary gameplay.
    pub runs: bool,
    /// Stable map collection fact.
    pub map_collected: bool,
}

impl RecurringDisplay {
    pub(super) fn current(
        &self,
        state: &State,
        before: &Hud,
        rules: &interface_hud::Rules,
        map: &HerMap,
    ) -> Option<DisplayReceipt> {
        let Caller::PlayerContinuation {
            prepared,
            remaining,
            ..
        } = &state.caller
        else {
            return None;
        };
        let gameplay = prepared.gameplay.as_ref()?;
        let actors = remaining.0.actors.as_ref()?;
        let interaction = &remaining.0.interaction.as_ref()?.terminal;
        let projectiles = &remaining.0.projectiles.as_ref()?.terminal;
        let frame = FrameCount(state.hud_frame_count?);
        if (frame != self.frame && !self.retain_first_half)
            || before.frame_count() != Some(frame)
            || !rules.her_blink.first_half(frame)
            || gameplay.scene != SceneController::MainGameplay
            || gameplay.route != GameplayRoute::MainGameplay
            || actors.dependency.scene != ActorScene::PreEscape
            || actors
                .actors
                .iter()
                .any(|actor| actor.content != ActorContentDisposition::OrdinaryCeresDoor)
            || interaction.activity.projectiles != 0
            || interaction.activity.bombs != 0
            || projectiles
                .slots
                .iter()
                .any(|slot| slot.instruction != InstructionDisposition::Empty)
            || state.movement_inputs.as_ref()?.0.elevator_moving
            || !self.quiet()
            || !before.events().is_empty()
            || !self.window_holds(state, before)
        {
            return None;
        }
        let health = state.health.as_ref()?;
        health.validate_current().ok()?;
        let inventory = state.hud_inventory?;
        Some(DisplayReceipt {
            premises: DisplayPremises {
                frame,
                mute: self.mute,
                runs: self.runs,
                // The staged mini-map needs only the two map events. Late status
                // requires its own pickup and drop opportunities after this join.
                events: RefreshEvents {
                    before_status_bar: Dropped::Nothing,
                    after_status_bar: Dropped::Nothing,
                    collected: None,
                    mother_brain_falls: self.map_events.mother_brain_falls,
                    turned_on: self.map_events.turned_on,
                },
                room: self.room,
                map: map.clone(),
                map_collected: self.map_collected,
            },
            status: Status {
                energy: Energy {
                    now: health.state().energy.whole.get(),
                    capacity: self.energy_capacity,
                },
                reserve: self.reserve,
                ammunition: inventory.ammunition().ok()?,
                equipped: inventory.equipped(),
            },
        })
    }

    const fn quiet(&self) -> bool {
        self.runs
            && !self.map_collected
            && !self.mute.time_stopped
            && !self.mute.grapple_out
            && matches!(self.reserve.mode, ReserveMode::Manual)
            && self.reserve.energy == 0
            && !self.map_events.mother_brain_falls
            && !self.map_events.turned_on
    }

    fn window_holds(&self, state: &State, before: &Hud) -> bool {
        let position = state.movement.state().position;
        let (x, y) = (position.x.pixel().0, position.y.pixel().0);
        let centre = Place {
            x: self
                .room
                .cell
                .x
                .saturating_add(i32::from(x / 256))
                .rem_euclid(64),
            y: self.room.cell.y.saturating_add(i32::from(y / 256)),
        };
        if x / 16 >= self.room.blocks.x || y / 16 >= self.room.blocks.y || centre != self.centre {
            return false;
        }
        (-1_i32..=1)
            .flat_map(|dy| (-2_i32..=2).map(move |dx| (dx, dy)))
            .map(|(dx, dy)| {
                (dx, dy) == (0, 0)
                    || before.state().explored.contains(Place {
                        x: centre.x.saturating_add(dx),
                        y: centre.y.saturating_add(dy),
                    })
            })
            .eq(self.explored)
    }
}
