//! Selected sprite placement and retained exact OAM production.

use super::{borrowed::SpriteBuilderOrigin, inputs::FigurePin, movement::Movement};
use crate::{
    intent::SpriteDraw,
    production::appearance::Work,
    state::video::{NmiBatch, OAM_SPRITES, Oam, OamEntry, SpriteCount},
};
use canonical::{Canonical, model::Model};
use data::assets::samus::{
    self, SpriteSheet,
    key::{Dress, SpinSprite, Suit},
};
use samus_appearance::{AppearanceRead, Figure, Step, WorkStep};
use types::{Pair, Parts};

#[cfg(test)]
mod tests;

/// One immutable picture keyed by the clean clip's actual shown step.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Picture {
    /// Actual semantic figure.
    pub figure: Figure,
    /// Clean step, excluding native instruction operands.
    pub step: Step,
    /// Imported top/bottom maps, signed placement offset and tile selection.
    pub frame: samus::Frame,
}

/// Checked immutable picture bindings and retained sprite-sheet content.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Content {
    sheet: SpriteSheet,
    pictures: Vec<Picture>,
}

/// Current independently admitted normal-drawing dependencies.
#[derive(Canonical, Clone, Copy, Debug, PartialEq, Eq)]
pub struct Premises {
    /// Current completed camera viewport, in room pixels.
    pub camera: [u16; 2],
    /// Current admitted screen shake, in signed pixels.
    pub shake: [i16; 2],
    /// Actual normal visibility, with no rotation or invisibility mechanism required.
    pub normal_visible: bool,
    /// Actual cannon and echo owners add no selected pieces in this drawing opportunity.
    pub no_additional_selected_pieces: bool,
    /// Actual other drawing owners have completed with empty contributions.
    pub unselected_draws_empty: bool,
}

/// Actual selected model owners borrowed at the drawing opportunity.
pub struct Inputs<'a> {
    /// Current movement authority, including its actual centre and pose.
    pub movement: &'a Movement,
    /// Complete current appearance and completed drawing-read continuation.
    pub appearance: &'a Work,
    /// Persistent actual graphical-contact consumer progress.
    pub contacts: &'a Contacts,
    /// Actual retained playing figure, after completed selected-clip publication.
    pub figure: Figure,
    /// Fresh independently admitted current external drawing owners.
    pub premises: Premises,
}

/// Persistent downstream progress through the complete retained graphical output stream.
#[derive(Canonical, Clone, Debug, Default, PartialEq, Eq)]
pub struct Contacts {
    consumed: u64,
    pending: Option<Part2Work>,
}

#[derive(Canonical, Clone, Debug, PartialEq, Eq)]
struct Part2Work {
    work: crate::production::timing::WorkId,
    receipt: ContactReceipt,
}

impl Contacts {
    /// Complete graphical output extent already consumed by Part 2.
    #[must_use]
    pub const fn consumed(&self) -> u64 {
        self.consumed
    }

    /// Validate downstream progress against the complete retained selected output stream.
    /// # Errors
    /// Missing output prefix or inconsistent retained pending work.
    pub fn validate(&self, appearance: &Work) -> Result<(), Fault> {
        let end = u64::try_from(appearance.model.0.graphics.len())
            .map_err(|_| Fault::GraphicalContact)?;
        if self.consumed > end
            || self.pending.as_ref().is_some_and(|pending| {
                pending.receipt.first != self.consumed
                    || pending.receipt.end <= self.consumed
                    || pending.receipt.end > end
            })
        {
            return Err(Fault::GraphicalContact);
        }
        Ok(())
    }
}

/// Current independently admitted context of the graphical-contact Part 2 consumer.
#[derive(Canonical, Clone, Copy, Debug, PartialEq, Eq)]
pub struct Part2Premises {
    /// Actual current room and liquid owners select the bounded dry Ceres consumer.
    pub dry_ceres: bool,
}

/// Actual graphical opportunities consumed without changing sprites in this admitted arm.
#[derive(Canonical, Clone, Debug, PartialEq, Eq)]
pub struct ContactReceipt {
    /// First previously unconsumed graphical opportunity.
    pub first: u64,
    /// Actual completed output-stream extent.
    pub end: u64,
    /// Current room/liquid context read by Part 2.
    pub premises: Part2Premises,
}

/// Choose actual selected contacts through the source-qualified dry unboosted Part 2 arm.
///
/// The graphical stream stays complete and available; consumption awaits actual completion.
/// Bind the existing charged Contact/Part 2 operation, without adding another physical stage.
/// # Errors
/// Missing/currently unsupported Part 2 context, incomplete receipts or repeated consumption.
pub fn choose_contacts(
    contacts: &mut Contacts,
    appearance: &Work,
    premises: Part2Premises,
    work: crate::production::timing::WorkId,
) -> Result<ContactReceipt, Fault> {
    if contacts.pending.is_some() {
        return Err(Fault::Order);
    }
    contacts.validate(appearance)?;
    appearance.validate().map_err(|_| Fault::GraphicalContact)?;
    let end =
        u64::try_from(appearance.model.0.graphics.len()).map_err(|_| Fault::GraphicalContact)?;
    if contacts.consumed >= end {
        return Err(Fault::Order);
    }
    if !premises.dry_ceres || appearance.model.0.state.stage == types::BoostStage::Boosting {
        return Err(Fault::GraphicalContact);
    }
    let mut seen = 0_u64;
    for completed in &appearance.model.0.completed {
        let samus_appearance::Receipt::Appearance { input, output, .. } = completed else {
            continue;
        };
        let Some(contact) = output.contact else {
            continue;
        };
        let AppearanceRead::Contact { stage, .. } = input else {
            return Err(Fault::GraphicalContact);
        };
        let index = usize::try_from(seen).map_err(|_| Fault::GraphicalContact)?;
        if appearance.model.0.graphics.get(index) != Some(&contact)
            || (seen >= contacts.consumed && *stage == types::BoostStage::Boosting)
        {
            return Err(Fault::GraphicalContact);
        }
        seen = seen.checked_add(1).ok_or(Fault::GraphicalContact)?;
    }
    if seen != end {
        return Err(Fault::GraphicalContact);
    }
    let receipt = ContactReceipt {
        first: contacts.consumed,
        end,
        premises,
    };
    contacts.pending = Some(Part2Work {
        work,
        receipt: receipt.clone(),
    });
    Ok(receipt)
}

/// Consume the retained admitted graphical prefix at actual charged operation completion.
/// # Errors
/// Missing/repeated contact work or a different completed hardware identity.
pub fn consume_contacts(
    contacts: &mut Contacts,
    completed: crate::production::timing::WorkId,
) -> Result<ContactReceipt, Fault> {
    let pending = contacts.pending.as_ref().ok_or(Fault::Order)?;
    if pending.work != completed {
        return Err(Fault::Completion);
    }
    let receipt = pending.receipt.clone();
    contacts.consumed = receipt.end;
    contacts.pending = None;
    Ok(receipt)
}

/// Clean shown-step identity, distinct from a native frame including operand bytes.
#[derive(Canonical, Clone, Copy, Debug, PartialEq, Eq)]
pub struct ShownStep(pub u8);

/// Actual selected picture and dependency receipt, independent of final expected OAM.
#[derive(Canonical, Clone, Debug, PartialEq, Eq)]
pub struct NormalDrawingReceipt {
    /// Actual playing figure used by this draw.
    pub figure: Model<FigurePin>,
    /// Actual clean step that selected immutable picture content.
    pub shown_step: ShownStep,
    /// Actual movement centre's whole pixels at drawing.
    pub centre: [u16; 2],
    /// Current camera, shake and admitted other drawing owners.
    pub premises: Premises,
    /// Byte cursor before this selected append.
    pub first_byte: u16,
    /// Actual imported header-derived selected piece count.
    pub pieces: u16,
}

/// Once-only producer progress; the backing stays in its sole `SpriteBuilderOrigin` owner.
#[derive(Canonical, Clone, Debug, Default, PartialEq, Eq)]
pub enum Stage {
    /// The actual reset has completed and selected drawing has not been chosen.
    #[default]
    Available,
    /// Selected exact writes are retained but not externally published.
    Chosen(NormalDrawingReceipt),
    /// Whole selected drawing work is physically pending.
    Drawing {
        /// Actual physical work identity.
        work: crate::production::timing::WorkId,
        /// Actual retained picture and dependencies.
        receipt: NormalDrawingReceipt,
    },
    /// Selected drawing completed; unused-slot hiding is still required.
    Drawn(NormalDrawingReceipt),
    /// Actual unused-slot hiding is retained pending whole finalization work.
    Finalizing {
        /// Actual physical work identity.
        work: crate::production::timing::WorkId,
        /// Actual selected draw already completed.
        receipt: NormalDrawingReceipt,
    },
    /// Complete producer backing may be materialized before its physical return tail.
    Ready(NormalDrawingReceipt),
}

/// Invalid selected content, missing producers or incorrect builder progress rejects atomically.
#[derive(Debug, PartialEq, Eq)]
pub enum Fault {
    /// Missing or conflicting figure/step, sprite-map or tile-upload content.
    Content,
    /// Actual selected Movement/Appearance/drawing dependencies are unavailable.
    Unsupported,
    /// A selected graphical contact has no admitted exact consumer here.
    GraphicalContact,
    /// Builder cursor is misaligned, outside capacity or would wrap.
    Capacity,
    /// An operation is absent, unfinished or repeated.
    Order,
    /// A different hardware operation completed.
    Completion,
}

const fn dress(sprite: types::Sprite) -> Dress {
    match sprite {
        types::Sprite::Plain => Dress {
            spin: SpinSprite::Plain,
            suit: Suit::First,
        },
        types::Sprite::OtherSuit => Dress {
            spin: SpinSprite::Plain,
            suit: Suit::VariaOrGravity,
        },
        types::Sprite::SpaceJump => Dress {
            spin: SpinSprite::SpaceJump,
            suit: Suit::First,
        },
        types::Sprite::ScrewAttack => Dress {
            spin: SpinSprite::ScrewAttack,
            suit: Suit::First,
        },
    }
}

impl Content {
    /// Validate immutable constructed or imported picture bindings.
    /// # Errors
    /// Conflicting keys or missing selected map/upload content.
    pub fn new(sheet: SpriteSheet, pictures: Vec<Picture>) -> Result<Self, Fault> {
        for (index, picture) in pictures.iter().enumerate() {
            if pictures
                .iter()
                .take(index)
                .any(|prior| prior.figure == picture.figure && prior.step == picture.step)
            {
                return Err(Fault::Content);
            }
            for id in [picture.frame.top, picture.frame.bottom]
                .into_iter()
                .flatten()
            {
                if sheet.sprite(id).is_none() {
                    return Err(Fault::Content);
                }
            }
            if let Some(tiles) = picture.frame.tiles {
                for id in [Some(tiles.top), tiles.bottom].into_iter().flatten() {
                    if sheet.upload(id).is_none() {
                        return Err(Fault::Content);
                    }
                }
            }
        }
        Ok(Self { sheet, pictures })
    }

    /// Bind clean steps through the existing checked native-frame content map.
    /// # Errors
    /// Missing imported pose, frame, map or upload, or conflicting bindings.
    pub fn import(
        assets: &samus::SamusAssets,
        clips: &crate::samus_appearance_bridge::Clips,
    ) -> Result<Self, Fault> {
        let mut pictures = Vec::new();
        for imported in clips.poses() {
            let clip = assets
                .clip(imported.figure.pose, dress(imported.figure.sprite))
                .ok_or(Fault::Content)?;
            for index in 0..imported.clip.instructions.len() {
                let step = Step(u8::try_from(index).map_err(|_| Fault::Content)?);
                let frame = imported.frame_of(step).ok_or(Fault::Content)?;
                let frame = *clip
                    .frames
                    .get(usize::from(frame.0))
                    .ok_or(Fault::Content)?;
                pictures.push(Picture {
                    figure: imported.figure,
                    step,
                    frame,
                });
            }
        }
        Self::new(assets.sheet.clone(), pictures)
    }

    fn picture(&self, figure: Figure, step: Step) -> Result<&Picture, Fault> {
        self.pictures
            .iter()
            .find(|picture| picture.figure == figure && picture.step == step)
            .ok_or(Fault::Content)
    }

    /// Actual selected immutable tile identities and bytes for the checked upload consumer.
    /// This does not invent native transfer flags, definitions or destination descriptors.
    /// # Errors
    /// Missing selected picture or tile content.
    pub fn uploads(
        &self,
        receipt: &NormalDrawingReceipt,
    ) -> Result<Vec<(samus::TilesId, &samus::TileUpload)>, Fault> {
        self.picture_uploads(receipt.figure.0, Step(receipt.shown_step.0))
    }

    /// Resolve checked tile content for an imported figure and step.
    /// # Errors
    /// Missing picture or tile content.
    pub fn picture_uploads(
        &self,
        figure: Figure,
        step: Step,
    ) -> Result<Vec<(samus::TilesId, &samus::TileUpload)>, Fault> {
        let picture = self.picture(figure, step)?;
        picture.frame.tiles.map_or_else(
            || Ok(Vec::new()),
            |tiles| {
                [Some(tiles.top), tiles.bottom]
                    .into_iter()
                    .flatten()
                    .map(|id| {
                        self.sheet
                            .upload(id)
                            .map(|upload| (id, upload))
                            .ok_or(Fault::Content)
                    })
                    .collect()
            },
        )
    }
}

fn batch(builder: &SpriteBuilderOrigin) -> Result<NmiBatch, Fault> {
    if !builder.cursor.is_multiple_of(4)
        || usize::from(builder.cursor) > OAM_SPRITES.saturating_mul(4)
    {
        return Err(Fault::Capacity);
    }
    let mut oam = Oam::default();
    for (entry, bytes) in oam.low.iter_mut().zip(builder.backing.as_chunks::<4>().0) {
        let [x, y, tile, attributes] = *bytes;
        *entry = OamEntry {
            x,
            y,
            tile,
            attributes,
        };
    }
    let high = builder
        .backing
        .get(OAM_SPRITES.saturating_mul(4)..)
        .ok_or(Fault::Capacity)?;
    oam.high.copy_from_slice(high);
    Ok(NmiBatch {
        oam,
        sprites: SpriteCount(builder.cursor / 4),
        ..NmiBatch::default()
    })
}

fn retain(builder: &mut SpriteBuilderOrigin, batch: &NmiBatch) {
    for (bytes, entry) in builder
        .backing
        .as_chunks_mut::<4>()
        .0
        .iter_mut()
        .zip(batch.oam.low)
    {
        *bytes = [entry.x, entry.y, entry.tile, entry.attributes];
    }
    if let Some(high) = builder.backing.get_mut(OAM_SPRITES.saturating_mul(4)..) {
        high.copy_from_slice(&batch.oam.high);
    }
    builder.cursor = batch.sprites.0.saturating_mul(4);
}

impl Stage {
    /// Encode the actual completed normal draw into the sole current builder.
    /// The caller qualifies the full dependency receipt before binding its whole profile.
    /// # Errors
    /// Missing actual draw/playing owners, unsupported extra pieces, bad content or capacity.
    pub fn choose(
        &mut self,
        builder: &mut SpriteBuilderOrigin,
        content: &Content,
        inputs: &Inputs<'_>,
    ) -> Result<&NormalDrawingReceipt, Fault> {
        let Inputs {
            movement,
            appearance,
            contacts,
            figure,
            premises,
        } = *inputs;
        if !matches!(self, Self::Available) {
            return Err(Fault::Order);
        }
        appearance.validate().map_err(|_| Fault::Unsupported)?;
        if contacts.pending.is_some()
            || u64::try_from(appearance.model.0.graphics.len()).ok() != Some(contacts.consumed)
        {
            return Err(Fault::GraphicalContact);
        }
        if !premises.normal_visible
            || !premises.no_additional_selected_pieces
            || !premises.unselected_draws_empty
            || figure.pose != movement.state().pose
            || appearance.model.0.pending.is_some()
            || appearance.model.0.consumed != appearance.model.0.steps.len()
            || !matches!(appearance.model.0.steps.last(), Some(WorkStep::Appearance(AppearanceRead::Draw { current, .. })) if *current == figure.pose.action)
        {
            return Err(Fault::Unsupported);
        }
        let step = appearance.model.0.state.playhead.step;
        let picture = content.picture(figure, step)?;
        if picture.frame.chest {
            return Err(Fault::Unsupported);
        }
        let centre = [
            movement.state().position.x.pixel().0,
            movement.state().position.y.pixel().0,
        ];
        let origin = Pair {
            x: crate::intent::on_screen(centre[0], premises.camera[0])
                .wrapping_add(i32::from(premises.shake[0])),
            y: crate::intent::on_screen(centre[1], premises.camera[1])
                .wrapping_add(i32::from(picture.frame.origin_y))
                .wrapping_add(i32::from(premises.shake[1])),
        };
        let mut draw = SpriteDraw::default();
        for id in [picture.frame.top, picture.frame.bottom]
            .into_iter()
            .flatten()
        {
            draw.place(content.sheet.sprite(id).ok_or(Fault::Content)?, origin);
        }
        let pieces = u16::try_from(draw.pieces.len()).map_err(|_| Fault::Capacity)?;
        let end = usize::from(builder.cursor / 4)
            .checked_add(draw.pieces.len())
            .ok_or(Fault::Capacity)?;
        if end >= OAM_SPRITES {
            return Err(Fault::Capacity);
        }
        let mut output = batch(builder)?;
        output.draw_sprites(&draw);
        let receipt = NormalDrawingReceipt {
            figure: Model(figure),
            shown_step: ShownStep(step.0),
            centre,
            premises,
            first_byte: builder.cursor,
            pieces,
        };
        retain(builder, &output);
        *self = Self::Chosen(receipt);
        let Self::Chosen(receipt) = self else {
            return Err(Fault::Order);
        };
        Ok(receipt)
    }

    /// Bind actual qualified drawing work after its central hardware begin.
    /// # Errors
    /// Drawing is absent or already bound.
    pub fn bind(&mut self, work: crate::production::timing::WorkId) -> Result<(), Fault> {
        let Self::Chosen(receipt) = self else {
            return Err(Fault::Order);
        };
        *self = Self::Drawing {
            work,
            receipt: receipt.clone(),
        };
        Ok(())
    }

    /// Complete actual selected drawing dependencies retained across physical holds.
    #[must_use]
    pub const fn receipt(&self) -> Option<&NormalDrawingReceipt> {
        match self {
            Self::Available => None,
            Self::Chosen(receipt)
            | Self::Drawing { receipt, .. }
            | Self::Drawn(receipt)
            | Self::Finalizing { receipt, .. }
            | Self::Ready(receipt) => Some(receipt),
        }
    }

    /// Consume actual whole drawing completion once, without writing another sprite.
    /// # Errors
    /// Missing drawing or different hardware identity.
    pub fn complete(&mut self, completed: crate::production::timing::WorkId) -> Result<(), Fault> {
        let Self::Drawing { work, receipt } = self else {
            return Err(Fault::Order);
        };
        if *work != completed {
            return Err(Fault::Completion);
        }
        *self = Self::Drawn(receipt.clone());
        Ok(())
    }

    /// Hide unused entries using the retained exact encoder and bind finalization work.
    /// # Errors
    /// Drawing has not completed, finalization repeated or invalid current cursor.
    pub fn finalise(
        &mut self,
        builder: &mut SpriteBuilderOrigin,
        work: crate::production::timing::WorkId,
    ) -> Result<(), Fault> {
        let Self::Drawn(receipt) = self else {
            return Err(Fault::Order);
        };
        if receipt
            .first_byte
            .checked_add(receipt.pieces.saturating_mul(4))
            != Some(builder.cursor)
        {
            return Err(Fault::Completion);
        }
        let mut output = batch(builder)?;
        output.finalise_oam();
        retain(builder, &output);
        *self = Self::Finalizing {
            work,
            receipt: receipt.clone(),
        };
        Ok(())
    }

    /// Publish full actual backing only after actual finalization completion.
    /// # Errors
    /// Missing/repeated finalization or different hardware identity.
    pub fn ready(
        &mut self,
        completed: crate::production::timing::WorkId,
        builder: &SpriteBuilderOrigin,
    ) -> Result<Vec<u8>, Fault> {
        let Self::Finalizing { work, receipt } = self else {
            return Err(Fault::Order);
        };
        if *work != completed {
            return Err(Fault::Completion);
        }
        if builder.cursor != 0 {
            return Err(Fault::Completion);
        }
        let bytes = builder.backing.to_vec();
        *self = Self::Ready(receipt.clone());
        Ok(bytes)
    }
}
