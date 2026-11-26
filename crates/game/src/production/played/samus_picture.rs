//! Connect the ordinary fight's body admission and art to the room picture.

use boss_ceres_ridley::Stage;
use samus_appearance::OrdinaryBodyDraw;
use samus_health::{PlayedFrame, ProtectionFrames};
use types::{Parity, Position, PushFrames};

use crate::intent::SpriteDraw;

use super::{Content, Fault, State, samus::Her};

impl State {
    pub(super) fn ordinary_fight_body(
        &self,
        her: &Her,
        health: PlayedFrame,
        camera: Position,
        draw_at: Option<Parity>,
        content: &Content,
    ) -> Result<Option<SpriteDraw>, Fault> {
        if self.station.stage != Stage::Calm || self.station.ridley.is_none() {
            return Ok(None);
        }
        let (Some(art), Some(push), Some(parity)) =
            (content.samus_body_art.as_ref(), her.push_at_draw, draw_at)
        else {
            return Ok(None);
        };
        let chance = OrdinaryBodyDraw {
            protected_while_drawn: health.protection_drawn != ProtectionFrames::ZERO,
            push_time_left: push != PushFrames::NONE,
            timed_special_colour: false,
            finished_frame_parity: parity,
        };
        if !chance.body_drawn() {
            return Ok(Some(SpriteDraw::default()));
        }
        let view = self.passage.room.entry.view;
        her.body_picture(art, camera, view.affine.then_some(view.transform))
            .map(Some)
    }

    pub(super) fn write_ordinary_fight_body(
        &mut self,
        body: &SpriteDraw,
    ) -> core::ops::Range<usize> {
        self.room_video.begin_ordinary_sprites();
        self.room_video.append_ordinary_body(body)
    }
}
