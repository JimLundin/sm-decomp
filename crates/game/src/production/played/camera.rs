//! The camera following her through a frame of play, and moved by a passage.

use super::super::camera_semantic;
use super::super::connected::inputs::{CameraInputs, Lead, Screen};
use super::samus::Her;
use super::{Fault, Rules};
use core::num::Wrapping;
use types::{Pair, Parts, Px};
use world_camera::{Camera, RememberedPosition, ViewPosition};

impl super::State {
    /// The camera after following her through a frame of play in `room`: `her` frame, with
    /// `samus` as the frame leaves her.
    pub(super) fn followed(
        &self,
        room: &world_camera::Room,
        her: &Her,
        samus: &samus_movement::Samus,
        rules: &Rules,
    ) -> Camera {
        let mut camera = self.camera.0;
        camera_semantic::step(
            &mut camera,
            rules.camera.get(),
            &world_camera::Frame {
                samus: seen(her, samus),
                room,
                lead: lead(self.passage.room.camera.lead),
                time_stopped: false,
            },
        );
        camera
    }
}

/// The camera after a passage's frame moves the view to `view` and where it last saw her to the
/// pixels `seen`, keeping the subpixels it had.
pub(super) fn passed(camera: Camera, view: types::Position, seen: Pair<Px>) -> Camera {
    let kept = camera.seen();
    camera
        .with_view(ViewPosition(view))
        .with_remembered(RememberedPosition(Pair {
            x: types::fixed(seen.x, kept.x.sub()),
            y: types::fixed(seen.y, kept.y.sub()),
        }))
}

/// What the camera reads of her, at `samus` as the frame leaves her. While the pad holds her
/// nothing placed her, and she neither goes backwards nor rises.
fn seen(her: &Her, samus: &samus_movement::Samus) -> world_camera::Samus {
    her.outcome.as_ref().map_or(
        world_camera::Samus {
            position: samus.position,
            placed: None,
            facing: samus.pose.facing,
            backwards: false,
            rising: false,
        },
        |outcome| camera_semantic::completed_samus(samus, outcome),
    )
}

/// The camera's room, from her room's screens, size and scrollers.
pub(super) fn room(input: &CameraInputs) -> Result<world_camera::Room, Fault> {
    let [width, height] = input.size;
    let [normal, rising] = input.heights;
    world_camera::Room::new(
        input
            .screens
            .iter()
            .map(|screen| match screen {
                Screen::Open => types::ScreenKind::Open,
                Screen::Closed => types::ScreenKind::Closed,
                Screen::Short => types::ScreenKind::Short,
            })
            .collect(),
        types::Pair {
            x: usize::from(width),
            y: usize::from(height),
        },
        world_camera::Heights {
            while_not_rising: Wrapping(normal),
            while_rising: Wrapping(rising),
        },
    )
    .map_err(|_| Fault::CameraRoom)
}

const fn lead(lead: Lead) -> world_camera::LeadSetting {
    match lead {
        Lead::Ahead => world_camera::LeadSetting::Ahead,
        Lead::Left => world_camera::LeadSetting::KeepLeft,
        Lead::Right => world_camera::LeadSetting::KeepRight,
        Lead::Near => world_camera::LeadSetting::Near,
    }
}
