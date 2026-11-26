//! Append Ridley's ordinary picture around Samus at the room's shared OAM cursor.

use core::ops::Range;

use boss_ceres_ridley::{BabyLook, Drawn, FightColours, FightParts, RidleyLook, RoomLayerOrder};
use data::assets::ridley_picture::Art;
use types::{Pair, Position, Px};

use super::{Content, Fault, State, station::Enemies};
use crate::exact::{ridley_door, ridley_picture};
use crate::intent::SpriteDraw;
use crate::state::video::NmiBatch;

pub(super) struct Picture<'a> {
    art: &'a Art,
    camera: Position,
    parts: Option<FightParts>,
    body: Option<Drawn<RidleyLook>>,
    baby: Option<Drawn<BabyLook>>,
    door: Option<(Pair<Px>, i16)>,
    attachment: RoomLayerOrder,
}

pub(super) fn picture<'a>(
    content: &'a Content,
    enemies: &Enemies,
    camera: Position,
) -> Option<Picture<'a>> {
    let art = content.ridley_picture.as_ref()?;
    let run = enemies.ridley?;
    let door = enemies.doors.iter().find_map(|run| {
        let door = run.door;
        door.phase.drawn_with_ridley().then_some(door.position)
    });
    let order = run.ridley.picture_order(door.is_some());
    let drawing = run.ridley.drawing();
    Some(Picture {
        art,
        camera,
        parts: run.parts,
        body: drawing.ridley,
        baby: drawing.baby.filter(|_| order.baby),
        door: door
            .filter(|_| order.door)
            .map(|position| (position, door_shake(enemies.quake))),
        attachment: order.opportunity,
    })
}

pub(super) struct Spans {
    pub(super) samus: Option<Range<usize>>,
    pub(super) door: Option<Range<usize>>,
}

impl State {
    pub(super) fn write_ordinary_sprites(
        &mut self,
        samus: Option<&SpriteDraw>,
        picture: Option<Picture<'_>>,
    ) -> Result<Spans, Fault> {
        let Some(picture) = picture else {
            return Ok(Spans {
                samus: samus.map(|body| self.write_ordinary_fight_body(body)),
                door: None,
            });
        };

        self.room_video.begin_ordinary_sprites();
        let batch = &mut self.room_video.video.batch;
        picture.parts(batch)?;
        let early_door = if picture.attachment == RoomLayerOrder::Before {
            picture.append_attachment(batch)
        } else {
            None
        };

        let samus = samus.map(|body| self.room_video.append_ordinary_body(body));
        let batch = &mut self.room_video.video.batch;
        if let Some(body) = picture.body {
            let origin = ridley_picture::relative(body.position, picture.camera);
            ridley_picture::body(
                batch,
                &picture.art.left,
                body.look,
                origin,
                picture.palette(),
            );
        }
        let door = if picture.attachment == RoomLayerOrder::After {
            picture.append_attachment(batch)
        } else {
            early_door
        };
        Ok(Spans { samus, door })
    }
}

impl Picture<'_> {
    fn palette(&self) -> u8 {
        match self.parts.map(|parts| parts.colours).unwrap_or_default() {
            FightColours::Normal => 7,
            FightColours::Hurt => 0,
        }
    }

    fn parts(&self, batch: &mut NmiBatch) -> Result<(), Fault> {
        let Some(parts) = self.parts else {
            return Ok(());
        };
        let palette = self.palette();
        if let Some(tail) = parts.tail {
            ridley_picture::tail(
                batch,
                self.art,
                usize::from(tail.tip.get()),
                tail.anchors,
                self.camera,
                palette,
            )
            .ok_or(Fault::RidleyPicture)?;
        }
        if let Some(wing) = parts.wing {
            let origin = ridley_picture::relative(wing.anchor, self.camera);
            ridley_picture::wings(
                batch,
                self.art,
                wing.facing,
                usize::from(wing.beat.get()),
                origin,
                palette,
            )
            .ok_or(Fault::RidleyPicture)?;
        }
        Ok(())
    }

    fn append_attachment(&self, batch: &mut NmiBatch) -> Option<Range<usize>> {
        if let Some(baby) = self.baby {
            ridley_picture::baby(batch, self.art, baby, self.camera);
        }
        let (position, shake) = self.door?;
        let start = usize::from(batch.sprites.0);
        ridley_door::append(batch, &self.art.attached_door, position, shake);
        Some(start..usize::from(batch.sprites.0))
    }
}

/// Sample the attached door's horizontal displacement before the room spends its quake timer.
const fn door_shake(quake: world_quake::State) -> i16 {
    match quake.remaining().0 % 4 {
        2 => -4,
        3 => -1,
        _ => 0,
    }
}

#[cfg(test)]
mod tests {
    use super::{Picture, door_shake};
    use crate::intent::SpriteDraw;
    use crate::production::played::tests::state;
    use boss_ceres_ridley::{BabyLook, Drawn, RidleyLook, RoomLayerOrder};
    use data::assets::ridley_picture::{Art, BodyLooks};
    use data::spritemap::{
        ExtendedMap, ExtendedPiece, SpriteAttributes, Spritemap, SpritemapEntry,
    };
    use types::Pair;

    fn zero<T: Default>() -> Pair<T> {
        Pair {
            x: T::default(),
            y: T::default(),
        }
    }

    fn piece(tile: u16) -> SpritemapEntry {
        SpritemapEntry {
            attributes: SpriteAttributes {
                tile,
                ..SpriteAttributes::default()
            },
            ..SpritemapEntry::default()
        }
    }

    fn map(tile: u16) -> Spritemap {
        Spritemap {
            entries: vec![piece(tile)],
        }
    }

    fn art() -> Art {
        let body = BodyLooks {
            rest: vec![ExtendedMap {
                offset: zero(),
                pieces: vec![ExtendedPiece {
                    sprite: piece(2),
                    x_word: 0,
                }],
            }],
            open: Vec::new(),
            wide: Vec::new(),
            takeoff1: Vec::new(),
            takeoff2: Vec::new(),
        };
        Art {
            baby: [map(1), Spritemap::default(), Spritemap::default()],
            wings: core::array::from_fn(|_| core::array::from_fn(|_| Spritemap::default())),
            links: core::array::from_fn(|_| Spritemap::default()),
            tip: core::array::from_fn(|_| Spritemap::default()),
            left: body.clone(),
            right: body,
            front: Vec::new(),
            attached_door: map(3),
        }
    }

    #[test]
    fn the_attachment_crosses_both_room_layers_and_waits_for_publication() {
        let art = art();
        let mut samus = SpriteDraw::default();
        samus.lone(zero(), piece(4).attributes);
        for (attachment, samus_span, door_span, expected) in [
            (RoomLayerOrder::Before, 2..3, 1..2, [1, 3, 4, 2]),
            (RoomLayerOrder::After, 0..1, 3..4, [4, 2, 1, 3]),
        ] {
            let mut room = state(Vec::new());
            let before = room.room_video.video.memory.oam;
            let picture = Picture {
                art: &art,
                camera: zero(),
                parts: None,
                body: Some(Drawn {
                    look: RidleyLook::Rest,
                    position: zero(),
                }),
                baby: Some(Drawn {
                    look: BabyLook::First,
                    position: zero(),
                }),
                door: Some((zero(), door_shake(room.quake.0))),
                attachment,
            };
            let spans = room
                .write_ordinary_sprites(Some(&samus), Some(picture))
                .unwrap();
            assert_eq!(spans.samus, Some(samus_span));
            assert_eq!(spans.door, Some(door_span));
            let video = &room.room_video.video;
            assert!(
                video
                    .batch
                    .oam
                    .low
                    .iter()
                    .zip(expected)
                    .all(|(sprite, tile)| sprite.tile == tile)
            );
            assert_eq!(video.memory.oam, before);
            let stored = room.room_video.ordinary_picture_checkpoint();
            let mut restored = super::super::room_video::RoomVideo::default();
            restored.restore_ordinary_picture(&stored).unwrap();
            assert_eq!(*restored.video, **video);
            assert_eq!(
                canonical::model::pin_holds::<crate::state::owned::FightWingsPin>(),
                Ok(())
            );
            room.room_video.publish_ordinary_sprites();
            restored.publish_ordinary_sprites();
            let video = &room.room_video.video;
            assert_eq!(*restored.video, **video);
            assert_eq!(video.memory.oam, video.batch.oam);
            assert_eq!(video.memory.oam.low[4].y, crate::exact::oam::HIDDEN_Y);
        }
    }
}
