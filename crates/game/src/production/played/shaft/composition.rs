//! Assemble one pending room sprite picture from the actual shaft actors.

use super::actor_picture::{ActorPictures, room_point};
use crate::intent::SpriteDraw;
use crate::production::blackout::StandingFrame;
use crate::production::played::{
    Content, Fault, State,
    ridley_picture::{self, Spans},
    room_video::ShaftSprites,
    samus::Her,
    station::Enemies,
};
use enemy_interactions::picture::{DrawView, PreScrollView};
use interface_hud::CountdownPicture;
use types::{Pair, Parity, Parts, Position};

pub(in crate::production::played) struct PictureFrame<'a> {
    pub(in crate::production::played) view: Position,
    pub(in crate::production::played) shake: world_quake::Offset,
    pub(in crate::production::played) refresh: bool,
    pub(in crate::production::played) timer: CountdownPicture,
    pub(in crate::production::played) body: Option<SpriteDraw>,
    pub(in crate::production::played) held_body: Option<SpriteDraw>,
    pub(in crate::production::played) samus_turn: Option<&'a mut Her>,
}

impl State {
    pub(in crate::production::played) fn assemble_play_picture(
        &mut self,
        enemies: &mut Enemies,
        hud: (&interface_hud::HudState, &interface_hud::FrameOutputs),
        body: (Position, Option<SpriteDraw>),
        samus_turn: Option<&mut Her>,
        quake: (world_quake::Offset, bool),
        content: &Content,
    ) -> Result<Spans, Fault> {
        let (view, body) = body;
        let (shake, refresh) = quake;
        let ordinary = if enemies.shaft.is_some() {
            if samus_turn.is_some() {
                self.prepare_shaft_picture(
                    enemies,
                    PictureFrame {
                        view,
                        shake,
                        refresh,
                        timer: hud.1.countdown.picture,
                        body,
                        held_body: None,
                        samus_turn,
                    },
                    content,
                )?;
            }
            Spans {
                samus: None,
                door: None,
            }
        } else {
            let picture = ridley_picture::picture(content, enemies, view);
            self.write_ordinary_sprites(body.as_ref(), picture)?
        };
        self.room_video.prepare_play_hud(hud.0)?;
        Ok(ordinary)
    }

    pub(in crate::production::played) fn prepare_passage_shaft_picture(
        &mut self,
        enemies: &mut Enemies,
        view: Position,
        samus: samus_movement::Samus,
        parity: Parity,
        content: &Content,
    ) -> Result<(), Fault> {
        let (body, held_body) = if enemies.shaft_actors.is_some() {
            self.passage_shaft_body(samus, view, parity, content)?
        } else {
            (None, None)
        };
        self.prepare_shaft_picture(
            enemies,
            PictureFrame {
                view,
                shake: world_quake::drawn_offset(enemies.quake, world_quake::Time::Running),
                refresh: false,
                timer: CountdownPicture::Absent,
                body,
                held_body,
                samus_turn: None,
            },
            content,
        )
    }

    pub(in crate::production::played) fn prepare_shaft_picture(
        &mut self,
        enemies: &mut Enemies,
        frame: PictureFrame<'_>,
        content: &Content,
    ) -> Result<(), Fault> {
        let (Some(shaft), Some(turn)) = (&mut enemies.shaft, enemies.shaft_actors) else {
            return Ok(());
        };
        let art = content.shaft_overlay_art.as_ref().ok_or(Fault::Escape)?;
        let effects = content.shaft_burst_art.as_ref().ok_or(Fault::Escape)?;
        let view = Pair {
            x: frame.view.x.pixel(),
            y: frame.view.y.pixel(),
        };
        let pictures = ActorPictures::resolve(
            &mut shaft.actor_clocks,
            turn,
            art.overlay_radius,
            shaft
                .vents
                .iter()
                .copied()
                .zip(enemies.shaft_vent_turns.iter().copied()),
            (
                PreScrollView(room_point(self.vent_camera())),
                DrawView(room_point(view)),
            ),
            frame.refresh,
        );
        let dust = super::dust::advance(frame.samus_turn, view, content)?;
        let standing = self.elevator_locked().then(|| {
            StandingFrame::from_locked(
                &self.samus,
                frame.view,
                self.passage.room.entry.view.transform,
            )
        });
        self.room_video.prepare_shaft_picture(
            turn,
            ShaftSprites {
                effects: shaft.effect_pictures(effects, view, frame.shake)?,
                actors: pictures,
                standing,
                dust,
                body: frame.body,
                held_body: frame.held_body,
                timer: frame.timer,
            },
            content,
        )
    }
}
