//! The quiet early mini-map, at her position after her moves.

use super::Fault;
use crate::production::hud::Hud;
use canonical::Canonical;
use interface_hud::{
    Ammunition, CancelButton, Equipped, FrameCount, HerMap, HudOpportunity, HudPictures,
    ItemControlInputs, MiniMapPower, MovementInputs, Room, Rules, SelectButton, Selection,
};
use types::Parts;

/// Current room, activity and frame facts for the selected early map.
#[derive(Clone, Copy)]
pub struct Inputs<'a> {
    /// The current area map imported from checked content.
    pub map: &'a HerMap,
    /// The current room and its dimensions.
    pub room: Room,
    /// The ordinary opportunity's HUD clock.
    pub frame: FrameCount,
    /// Actual admitted item control dispositions.
    pub controls: (SelectButton, CancelButton),
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
    pub(super) hud: Hud,
    pub(super) position: Option<[u32; 2]>,
}

impl State {
    pub(super) fn begin(&mut self, input: Inputs<'_>) -> Result<(), Fault> {
        if input.controls != (SelectButton::NotPressed, CancelButton::NotPressed)
            || self.hud.state().selection != Selection::None
            || (self.hud.state().mini_map == MiniMapPower::On) != input.enabled
        {
            return Err(Fault::HudMap);
        }
        let mut hud = Hud::new(self.hud.state().clone());
        // This arm has neither Select nor Cancel. No availability is inspected.
        let controls = ItemControlInputs {
            select: input.controls.0,
            cancel: input.controls.1,
            can_switch: true,
            ammunition: Ammunition::new(enum_map::EnumMap::default()).ok_or(Fault::HudMap)?,
            equipped: Equipped::default(),
        };
        if !hud.begin_staged(input.frame) || !hud.choose_controls(controls) || !hud.apply() {
            return Err(Fault::HudMap);
        }
        self.hud = hud;
        self.position = None;
        Ok(())
    }

    pub(super) fn publish(
        &mut self,
        position: types::Position,
        input: Inputs<'_>,
    ) -> Result<(), Fault> {
        if self.position.is_some()
            || self.hud.opportunity() != Some(HudOpportunity::MiniMapUpdate)
            || !self.hud.choose_map(
                input.rules,
                input.pictures,
                MovementInputs {
                    runs: true,
                    turned_on: false,
                    mother_brain_falls: false,
                    position: interface_hud::Pair {
                        x: position.x.pixel().0,
                        y: position.y.pixel().0,
                    },
                    room: input.room,
                    map: input.map.clone(),
                    map_collected: input.map_collected,
                },
            )
            || !self.hud.apply()
        {
            return Err(Fault::HudMap);
        }
        self.position = Some([position.x.0, position.y.0]);
        Ok(())
    }
}
