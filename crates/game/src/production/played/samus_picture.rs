//! Connect the ordinary body admission and art to the room picture.

use boss_ceres_ridley::Stage;
use samus_appearance::OrdinaryBodyDraw;
use samus_health::{PlayedFrame, ProtectionFrames};
use types::{Parity, Position, PushFrames};

use crate::intent::SpriteDraw;
use crate::production::room_loading;

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
        let fight = self.station.stage == Stage::Calm && self.station.ridley.is_some();
        let shaft = self
            .station
            .escape_shaft
            .as_ref()
            .is_some_and(|shaft| !shaft.locked);
        if !fight && !shaft {
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

    pub(super) fn passage_shaft_body(
        &self,
        samus: samus_movement::Samus,
        camera: Position,
        parity: Parity,
        content: &Content,
    ) -> Result<(Option<SpriteDraw>, Option<SpriteDraw>), Fault> {
        let Some(art) = content.samus_body_art.as_ref() else {
            return Ok((None, None));
        };
        let chance = OrdinaryBodyDraw {
            protected_while_drawn: self.health.0.protection != ProtectionFrames::ZERO,
            push_time_left: samus.knockback.time_left != PushFrames::NONE,
            timed_special_colour: false,
            finished_frame_parity: parity,
        };
        let mut her = self.samus.held();
        her.samus = samus;
        let view = self.passage.room.entry.view;
        let held = her.body_picture(art, camera, view.affine.then_some(view.transform))?;
        // The fade-in also visits the held pose after the ordinary actor layers.
        let ordinary = chance.body_drawn().then(|| held.clone());
        Ok((ordinary, Some(held)))
    }

    pub(super) fn write_ordinary_fight_body(
        &mut self,
        body: &SpriteDraw,
    ) -> core::ops::Range<usize> {
        self.room_video.begin_ordinary_sprites();
        self.room_video.append_ordinary_body(body)
    }
}

impl Her {
    pub(super) fn publish_suit_colours(&self, palette: &mut world_doors::Palette) {
        if let (Some(row), Some(colours)) =
            (room_loading::suit_row(&mut palette.screen), self.colours)
        {
            *row = colours.0;
        }
    }
}
