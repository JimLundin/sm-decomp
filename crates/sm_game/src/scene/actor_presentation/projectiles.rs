//! Actual projectile/trail/rope publication into the current object owner.
#![forbid(unsafe_code)]
use super::{Binding, DataRole, Diagnostics, DisplayError, Session, SpriteRequest, WordOrder};
use crate::{
    external_inputs::ExternalInputs,
    geometry::Point,
    player_projectiles::host::Drawing,
    scene::{objects::Clip, records::ObjectVariants, samus::ObjectPublication},
};
use std::sync::Arc;

pub struct EnemySpriteRequest {
    pub offscreen: bool,
    pub bank: u8,
    pub map: u16,
    pub position: Point,
    pub tile: u16,
    pub graphics: u16,
}

impl Binding {
    pub fn draw_enemy(
        &self,
        session: &mut Session,
        request: EnemySpriteRequest,
        diagnostics: &mut dyn Diagnostics,
    ) -> Result<(), DisplayError> {
        self.draw_sprite(
            session,
            SpriteRequest {
                bank: request.bank,
                map: request.map,
                position: request.position,
                palette: request.graphics,
                base_tile: request.tile,
                clip: if request.offscreen {
                    Clip::Outside
                } else {
                    Clip::Inside
                },
            },
            diagnostics,
        )
    }

    pub fn projectile_draw(
        &self,
        session: &mut Session,
        observations: &ExternalInputs,
        drawing: Drawing,
        content: &crate::scene::prepared_samus::PreparedSamusContent,
        diagnostics: &mut dyn Diagnostics,
    ) -> Result<(), DisplayError> {
        self.check(session)?;
        match drawing {
            Drawing::Projectile { slot, x, y } => {
                let at = 0xcb8 + 2 * usize::from(slot >> 1);
                let order = session.presentation.owner()?.state.objects.native_order;
                let map = read_word(session, diagnostics, at, order)?;
                self.draw_sprite(
                    session,
                    SpriteRequest {
                        bank: 0x93,
                        map,
                        position: Point { x, y },
                        palette: 0,
                        base_tile: 0,
                        clip: Clip::None,
                    },
                    diagnostics,
                )
            }
            Drawing::Beam { sprite, x, y } => {
                // Captured loaded table, full widened index; no low-window diagnostic.
                let source = self
                    .0
                    .data_sources
                    .get(&DataRole::BeamPointers)
                    .ok_or(DisplayError::Identity)?;
                if !source
                    .record
                    .image
                    .same_allocation(&diagnostics.loaded_image())
                {
                    return Err(DisplayError::Identity);
                }
                let map =
                    super::word(&source.record, 2 * usize::from(sprite), source.native_order)?;
                self.draw_sprite(
                    session,
                    SpriteRequest {
                        bank: 0x93,
                        map,
                        position: Point { x, y },
                        palette: 0,
                        base_tile: 0,
                        clip: Clip::None,
                    },
                    diagnostics,
                )
            }
            Drawing::CrashEchoes => {
                if session.presentation.owner()?.state.nmi.frame_word & 1 != 0 {
                    for echo in [3, 2] {
                        if session.projectiles.echoes.x_speed[echo] != 0 {
                            self.draw_echo(session, observations, echo, content, diagnostics)?;
                        }
                    }
                }
                Ok(())
            }
            Drawing::GrappleSegment { x, y, tile } => {
                self.draw_grapple(session, Some((x, y, tile)), diagnostics)
            }
            Drawing::GrappleEnd => self.draw_grapple(session, None, diagnostics),
        }
    }

    fn draw_echo(
        &self,
        session: &mut Session,
        observations: &ExternalInputs,
        echo: usize,
        content: &crate::scene::prepared_samus::PreparedSamusContent,
        diagnostics: &mut dyn Diagnostics,
    ) -> Result<(), DisplayError> {
        let y = session.projectiles.echoes.y[echo];
        let image = diagnostics.loaded_image();
        let source = self
            .0
            .data_sources
            .get(&DataRole::EchoPoseHeights)
            .ok_or(DisplayError::Identity)?;
        if !source.record.image.same_allocation(&image) {
            return Err(DisplayError::Identity);
        }
        let height = super::byte(&source.record, usize::from(session.player.pose) * 8 + 4)?;
        let y = y
            .wrapping_sub(u16::from(height))
            .wrapping_sub(session.room.camera.layer1[1]);
        if i16::from_ne_bytes(y.to_ne_bytes()) >= 0 && y.wrapping_sub(248) & 0x8000 != 0 {
            let index = session.player.top_sprite;
            let x = session.projectiles.echoes.x[echo].wrapping_sub(session.room.camera.layer1[0]);
            diagnostics.draw_echo_sprite(
                session,
                observations,
                content,
                echo,
                index,
                Point { x, y },
            )?;
            // The first draw can replace the selected state; both queries remain fresh.
            if session.player.bottom_sprite != 0 {
                let index = session.player.bottom_sprite;
                let x =
                    session.projectiles.echoes.x[echo].wrapping_sub(session.room.camera.layer1[0]);
                diagnostics.draw_echo_sprite(
                    session,
                    observations,
                    content,
                    echo,
                    index,
                    Point { x, y },
                )?;
            }
        }
        Ok(())
    }

    pub fn draw_trail(
        &self,
        session: &mut Session,
        pair: usize,
        right: bool,
        frozen: bool,
        diagnostics: &mut dyn Diagnostics,
    ) -> Result<(), DisplayError> {
        self.check(session)?;
        let saved = read_word(session, diagnostics, 0x590, WordOrder::Little)?;
        if i16::from_ne_bytes(saved.to_ne_bytes()).wrapping_sub(512) >= 0 {
            return Ok(());
        }
        let (timer, x_field, y_field, tile_field) = if right {
            (0xd67c, 0xd754, 0xd79c, 0xd70c)
        } else {
            (0xd658, 0xd730, 0xd778, 0xd6e8)
        };
        if frozen && read_word(session, diagnostics, timer + 2 * pair, WordOrder::Little)? == 0 {
            return Ok(());
        }
        let x = read_word(session, diagnostics, x_field + 2 * pair, WordOrder::Little)?
            .wrapping_sub(read_word(session, diagnostics, 0x911, WordOrder::Little)?);
        if x & 0xff00 == 0 {
            let entry = usize::from(read_word(session, diagnostics, 0x590, WordOrder::Little)?);
            let at = 0x370 + entry;
            let order = session.presentation.owner()?.state.objects.native_order;
            write_word(session, diagnostics, at, x, order)?;
            let y = read_word(session, diagnostics, y_field + 2 * pair, WordOrder::Little)?
                .wrapping_sub(read_word(session, diagnostics, 0x915, WordOrder::Little)?);
            if y & 0xff00 == 0 {
                write_word(session, diagnostics, at + 1, y, order)?;
                let tile = read_word(
                    session,
                    diagnostics,
                    tile_field + 2 * pair,
                    WordOrder::Little,
                )?;
                write_word(session, diagnostics, at + 2, tile, order)?;
                write_word(
                    session,
                    diagnostics,
                    0x590,
                    saved.wrapping_add(4),
                    WordOrder::Little,
                )?;
                // Authored frame identity follows the original stores, not a tile-byte match.
                let trail = session
                    .projectiles
                    .trails
                    .get(pair)
                    .ok_or(DisplayError::Source)?;
                let cursor = if right {
                    trail.right.cursor
                } else {
                    trail.left.cursor
                };
                let variants = self
                    .0
                    .trail_parts
                    .get(&(right, cursor))
                    .ok_or(DisplayError::Identity)?
                    .clone();
                publish(session, diagnostics, variants, entry, x, y, tile)?;
            }
        }
        Ok(())
    }

    fn draw_grapple(
        &self,
        session: &mut Session,
        segment: Option<(u16, u16, u16)>,
        diagnostics: &mut dyn Diagnostics,
    ) -> Result<(), DisplayError> {
        let saved = read_word(session, diagnostics, 0x590, WordOrder::Little)?;
        let entry = usize::from(saved);
        let at = 0x370 + entry;
        let order = session.presentation.owner()?.state.objects.native_order;
        let (x, y, tile, variants) = if let Some((x, y, tile)) = segment {
            super::write_byte(session, diagnostics, at, x.to_le_bytes()[0])?;
            super::write_byte(session, diagnostics, at + 1, y.to_le_bytes()[0])?;
            write_word(session, diagnostics, at + 2, tile, order)?;
            (x, y, tile, self.0.grapple_segment.clone())
        } else {
            let camera = session.room.camera.layer1;
            let x = session
                .player
                .grapple_endpoint
                .x
                .wrapping_sub(camera[0])
                .wrapping_sub(4);
            super::write_byte(session, diagnostics, at, x.to_le_bytes()[0])?;
            let y = session
                .player
                .grapple_endpoint
                .y
                .wrapping_sub(camera[1])
                .wrapping_sub(4);
            super::write_byte(session, diagnostics, at + 1, y.to_le_bytes()[0])?;
            write_word(session, diagnostics, at + 2, 14880, order)?;
            let extension_at = 0x570 + 2 * usize::from(saved >> 5);
            let extension = read_word(session, diagnostics, extension_at, WordOrder::Little)?;
            let bit = ((x & 0x100) >> 8) << (2 * ((saved >> 2) & 7));
            write_word(
                session,
                diagnostics,
                extension_at,
                extension | bit,
                WordOrder::Little,
            )?;
            (x, y, 14880, self.0.grapple_end.clone())
        };
        write_word(
            session,
            diagnostics,
            0x590,
            saved.wrapping_add(4),
            WordOrder::Little,
        )?;
        publish(session, diagnostics, variants, entry, x, y, tile)
    }
}

fn read_word(
    session: &Session,
    diagnostics: &dyn Diagnostics,
    address: usize,
    order: WordOrder,
) -> Result<u16, DisplayError> {
    Ok(order.decode([
        super::read_byte(session, diagnostics, address)?,
        super::read_byte(
            session,
            diagnostics,
            address.checked_add(1).ok_or(DisplayError::Source)?,
        )?,
    ]))
}
fn write_word(
    session: &mut Session,
    diagnostics: &mut dyn Diagnostics,
    address: usize,
    value: u16,
    order: WordOrder,
) -> Result<(), DisplayError> {
    if address.checked_add(2).is_none_or(|end| end > 0x20000) {
        return Err(DisplayError::Alias { offset: address });
    }
    for (lane, byte) in order.encode(value).into_iter().enumerate() {
        super::write_byte(session, diagnostics, address + lane, byte)?;
    }
    Ok(())
}
fn publish(
    session: &mut Session,
    diagnostics: &mut dyn Diagnostics,
    variants: Arc<ObjectVariants>,
    entry: usize,
    x: u16,
    y: u16,
    word: u16,
) -> Result<(), DisplayError> {
    let owner = session.presentation.owner_mut()?;
    let media = owner.media().clone();
    let context = diagnostics.object_context(owner)?;
    let producer = owner
        .state
        .producer_ids
        .allocate()
        .map_err(|_| DisplayError::Identity)?;
    crate::scene::samus::publish_object(
        &mut owner.state,
        &media,
        ObjectPublication {
            producer,
            variants,
            context,
            entry,
            position: (x, y),
            word,
        },
    )
}
