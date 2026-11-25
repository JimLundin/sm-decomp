//! The quiet early HUD frame, its mini-map read at her position after her moves.

use super::Fault;
use crate::production::hud::HudPin;
use canonical::{Canonical, model::Model};
use interface_hud::{
    CancelButton, Controls, CountdownInputs, Dropped, FrameCount, FrameInputs, HerMap, HudPictures,
    MiniMapInputs, MiniMapPower, Mute, Room, Rules, SelectButton, Selection, Status, StockAtStart,
};
use types::Parts;

/// The frame's facts from the HUD's neighbours, beside the position Movement supplies.
#[derive(Clone, Copy)]
pub struct Inputs<'a> {
    /// The current area map imported from checked content.
    pub map: &'a HerMap,
    /// The current room and its dimensions.
    pub room: Room,
    /// The frame count as the frame begins.
    pub frame: FrameCount,
    /// Item select and item cancel this frame.
    pub controls: (SelectButton, CancelButton),
    /// Whether the controls may switch the item.
    pub can_switch: bool,
    /// Her stock as the frame begins.
    pub stock_at_start: StockAtStart,
    /// What the status bar draws, as the frame ends.
    pub status: Status,
    /// Why a switch would make no click.
    pub muted_by: Option<Mute>,
    /// What the escape countdown reads this frame: whether the scene runs it, its start and the
    /// count of finished frames.
    pub countdown: CountdownInputs,
    /// The currently observed map power.
    pub enabled: bool,
    /// Whether charted cells are always shown.
    pub map_collected: bool,
    /// Current map pictures.
    pub pictures: &'a HudPictures,
    /// Current map rules.
    pub rules: &'a Rules,
}

#[derive(Canonical, Clone, Debug, PartialEq, Eq)]
pub(super) struct State {
    pub(super) hud: Model<HudPin>,
    pub(super) position: Option<[u32; 2]>,
}

impl State {
    /// Play the HUD's whole frame once her moves are known. The family has no item controls
    /// and nothing that hears the HUD's events, so a frame that would switch or click is
    /// refused.
    pub(super) fn play(
        &mut self,
        position: types::Position,
        input: Inputs<'_>,
    ) -> Result<(), Fault> {
        let state = &self.hud.0;
        if input.controls != (SelectButton::NotPressed, CancelButton::NotPressed)
            || state.selection != Selection::None
            || (state.mini_map == MiniMapPower::On) != input.enabled
        {
            return Err(Fault::HudMap);
        }
        let mut hud = state.clone();
        let outputs = hud
            .frame_of_play(
                input.rules,
                input.pictures,
                &FrameInputs {
                    frame: input.frame,
                    stock_at_start: input.stock_at_start,
                    controls: Controls {
                        select: input.controls.0,
                        cancel: input.controls.1,
                        can_switch: input.can_switch,
                    },
                    mini_map: MiniMapInputs {
                        runs: true,
                        turned_on: false,
                        position: interface_hud::Pair {
                            x: position.x.pixel().0,
                            y: position.y.pixel().0,
                        },
                        room: input.room,
                        map: input.map,
                        map_collected: input.map_collected,
                    },
                    mother_brain_falls: false,
                    status: input.status,
                    collected: None,
                    muted_by: input.muted_by,
                    dropped_before_status_bar: Dropped::Nothing,
                    dropped_after_status_bar: Dropped::Nothing,
                    countdown: input.countdown,
                },
            )
            .map_err(|_| Fault::HudMap)?;
        if !outputs.events.is_empty() {
            return Err(Fault::HudMap);
        }
        self.hud = Model(hud);
        self.position = Some([position.x.0, position.y.0]);
        Ok(())
    }
}
