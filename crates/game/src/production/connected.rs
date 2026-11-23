//! One carried ordinary running frame through movement, appearance and camera.
//!
//! Each model is stepped once per frame through its whole-frame entry, in the played owner's
//! order: her clip's look ahead, her moves, her animation (with her stride and pose selection),
//! then the camera following her. With the early mini-map selected, the map reads her position
//! after her moves from movement's outcome. The frame commits every owner together or none.

pub mod early_map;
pub mod inputs;
mod world;

use super::animation::{self, Parts};
use crate::production::{AppearanceContent, CameraRules, MovementContent, MovementInputs};
use crate::state::owned::{AppearancePin, CameraPin, MovementPin};
use canonical::{
    Canonical,
    model::{Model, Pinned},
};
use samus_appearance::{Figure, Selection, Stride};
use types::{Action, Facing, Sprite};

/// Complete content for the connected selected owners.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Content {
    /// Checked body/cannon content.
    pub movement: MovementContent,
    /// Checked clips and running paces.
    pub appearance: AppearanceContent,
}

/// Complete selected tuning bound to the saved context.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Rules {
    /// Movement parameters.
    pub movement: samus_movement::Rules,
    /// Appearance parameters.
    pub appearance: samus_appearance::Rules,
    /// Camera parameters.
    pub camera: CameraRules,
}

/// The figure she is drawn in as the frame starts.
pub struct PlayingPin;
impl Pinned for PlayingPin {
    type State = Figure;
    const NAME: &'static str = "production-connected-playing";
    const SHAPE: u64 = 0x4301_5F68_4F3E_58BA;
}

/// The selected owners, carried frame to frame.
#[derive(Canonical, Clone, Debug, PartialEq, Eq)]
pub struct State {
    /// Her movement.
    pub movement: Model<MovementPin>,
    /// Her animation's playhead and boost.
    pub appearance: Model<AppearancePin>,
    /// The figure she is drawn in.
    pub playing: Model<PlayingPin>,
    /// The camera.
    pub camera: Model<CameraPin>,
    completed: u64,
    early_map: Option<Box<early_map::State>>,
}

/// What one frame published.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Outputs {
    /// Her appearance's sounds, in the order asked.
    pub sounds: Vec<samus_appearance::FrameSound>,
    /// Her stride's graphical contact, even when its sound is quieted.
    pub contact: Option<samus_appearance::GraphicalContact>,
    /// Movement's complete outcome.
    pub movement: samus_movement::Outcome,
    /// The camera as the frame leaves it.
    pub camera: world_camera::Camera,
}

/// Why a frame was not played. State is unchanged.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Fault {
    /// The room's geometry is malformed.
    Inputs,
    /// An action, activity, liquid, booster, terrain or reaction outside the running family.
    Family,
    /// Movement has no situation or content for her pose.
    Movement,
    /// A figure she shows is outside the running family or has no imported clip.
    Figure,
    /// The camera's room is malformed, or it did not last see her where she was.
    Camera,
    /// The early map owner is missing, unexpected or refused its inputs.
    HudMap,
}

impl State {
    /// Admit the selected owners' first State.
    /// # Errors
    /// A figure or pose outside the running family.
    pub fn new(
        movement: samus_movement::Samus,
        appearance: samus_appearance::State,
        playing: Figure,
        camera: world_camera::Camera,
    ) -> Result<Self, Fault> {
        if playing.pose != movement.pose
            || playing.sprite != Sprite::Plain
            || movement.pose.action != Action::Running
        {
            return Err(Fault::Figure);
        }
        Ok(Self {
            movement: Model(movement),
            appearance: Model(appearance),
            playing: Model(playing),
            camera: Model(camera),
            completed: 0,
            early_map: None,
        })
    }

    /// Select the early map owner before the first frame.
    /// # Errors
    /// A frame already played, or an owner already selected.
    pub fn with_early_map(mut self, hud: interface_hud::HudState) -> Result<Self, Fault> {
        if self.completed != 0 || self.early_map.is_some() {
            return Err(Fault::HudMap);
        }
        self.early_map = Some(Box::new(early_map::State {
            hud: crate::production::hud::Hud::new(hud),
            position: None,
        }));
        Ok(self)
    }

    /// The early map's HUD and the position it last read, once a frame has played.
    #[must_use]
    pub fn early_map(&self) -> Option<(&interface_hud::HudState, Option<[u32; 2]>)> {
        self.early_map
            .as_ref()
            .map(|map| (map.hud.state(), map.position))
    }

    /// The frames played.
    #[must_use]
    pub const fn completed(&self) -> u64 {
        self.completed
    }

    /// Check the owners against the running family and each other.
    /// # Errors
    /// A figure or pose outside the family, or an early map whose reading disagrees with the
    /// frames played.
    pub fn validate(&self, content: &Content) -> Result<(), Fault> {
        drawn(self.playing.0, &content.appearance)?;
        if let Some(map) = &self.early_map
            && (map.position.is_some() != (self.completed > 0)
                || map.position.is_some() != map.hud.map_ready())
        {
            return Err(Fault::HudMap);
        }
        Ok(())
    }

    /// Play one frame from `input`.
    /// # Errors
    /// Inputs or results outside the running family leave State unchanged.
    pub fn advance(
        &mut self,
        input: &inputs::Inputs,
        content: &Content,
        rules: &Rules,
    ) -> Result<Outputs, Fault> {
        self.advance_with_map(input, content, rules, None)
    }

    pub(super) fn advance_with_map(
        &mut self,
        input: &inputs::Inputs,
        content: &Content,
        rules: &Rules,
        map: Option<early_map::Inputs<'_>>,
    ) -> Result<Outputs, Fault> {
        if self.early_map.is_some() != map.is_some() {
            return Err(Fault::HudMap);
        }
        let mut candidate = self.clone();
        candidate.validate(content)?;
        let outputs = candidate.frame(input, content, rules, map)?;
        candidate.validate(content)?;
        *self = candidate;
        Ok(outputs)
    }

    fn frame(
        &mut self,
        input: &inputs::Inputs,
        content: &Content,
        rules: &Rules,
        map: Option<early_map::Inputs<'_>>,
    ) -> Result<Outputs, Fault> {
        if input.activity != inputs::Activity::Inactive
            || input.items[types::Item::SpeedBooster]
            || input.liquid != samus_appearance::Liquid::NONE
        {
            return Err(Fault::Family);
        }
        input.geometry.validate().map_err(|_| Fault::Inputs)?;
        running(self.playing.0, &content.appearance)?;
        if self.playing.0.pose != self.movement.0.pose {
            return Err(Fault::Figure);
        }
        if let (Some(owner), Some(premises)) = (&mut self.early_map, map) {
            owner.begin(premises)?;
        }
        let playing = self.playing.0;
        let previous = self.movement.0;
        let clip = content.appearance.get(playing).ok_or(Fault::Figure)?;
        let ahead = super::appearance::ahead(&self.appearance.0, clip);

        let guard = world::Guard {
            world: &input.geometry,
            rejected: core::cell::Cell::new(false),
        };
        let mut samus = previous;
        let outcome = super::movement::advance_scoped(
            &mut samus,
            &content.movement,
            &rules.movement,
            &MovementInputs {
                sprite_before_movement: playing.sprite,
                cinematic_active: matches!(
                    input.cinematic,
                    samus_appearance::CinematicActivity::Active
                ),
                pad: input.held,
                pressed: input.pressed,
                items: input.items,
                spin_turned_far: ahead.turned_far,
                elevator_moving: false,
                parity: input.parity,
                animation_ends: ahead.end,
            },
            &guard,
        )
        .map_err(|_| Fault::Movement)?;
        if guard.rejected.get() {
            return Err(Fault::Family);
        }
        if let (Some(owner), Some(premises)) = (&mut self.early_map, map) {
            owner.publish(outcome.position_after_moves, premises)?;
        }

        let selected = Figure {
            pose: samus.pose,
            sprite: playing.sprite,
        };
        drawn(selected, &content.appearance)?;
        let mut appearance = self.appearance.0;
        let shown = animation::play(
            &mut appearance,
            &content.appearance,
            &rules.appearance,
            Parts {
                stride: Some(Stride {
                    figure: playing,
                    suppressors: input.contact,
                }),
                animation: Some(super::played::animation_inputs(
                    playing, selected, &outcome, input,
                )),
                selection: Some(Selection {
                    from: playing,
                    selected,
                    items: input.items,
                    submerged: input.liquid.submerged,
                    cinematic: input.cinematic,
                }),
            },
        )
        .map_err(|_| Fault::Figure)?;

        let mut camera = self.camera.0;
        super::camera_semantic::follow(
            &mut camera,
            &previous,
            &samus,
            &outcome,
            &input.camera,
            rules.camera.get(),
        )?;

        self.movement = Model(samus);
        self.appearance = Model(appearance);
        self.playing = Model(selected);
        self.camera = Model(camera);
        self.completed = self.completed.saturating_add(1);
        Ok(Outputs {
            sounds: shown.sounds,
            contact: shown.contact,
            movement: outcome,
            camera,
        })
    }
}

/// A frame starts in the running family: a plain running figure, facing left or right.
fn running(figure: Figure, content: &AppearanceContent) -> Result<(), Fault> {
    if figure.pose.action != Action::Running
        || !matches!(figure.pose.facing, Facing::Left | Facing::Right)
    {
        return Err(Fault::Figure);
    }
    drawn(figure, content)
}

/// A figure she may end a frame in: plain, with an imported clip. The pose she takes may
/// leave the running family; the next frame then refuses to start.
fn drawn(figure: Figure, content: &AppearanceContent) -> Result<(), Fault> {
    if figure.sprite != Sprite::Plain || content.get(figure).is_none() {
        return Err(Fault::Figure);
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn pins_keep_complete_inputs_and_playing_figure() {
        use canonical::model::{pin_holds, shape};
        assert_eq!(
            (
                shape::<inputs::Inputs>(),
                pin_holds::<PlayingPin>(),
                pin_holds::<AppearancePin>()
            ),
            (Ok(inputs::InputsPin::SHAPE), Ok(()), Ok(()))
        );
    }
}
