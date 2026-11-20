//! Checked importer identities for actual selected tile-upload outputs.

use data::{
    assets::samus::{TileUpload, TilesId},
    dma::BusAddress,
    pose::tiles::{Graphics, GraphicsRef, SamusTiles, TileDefinition, TileDefinitionId},
    rom::{Lorom, RomAddress},
    timing::ContentRegion,
};
use game::{
    production::frame::rendering_stage::NormalDrawingReceipt, samus_appearance_bridge::Clips,
};
use import::samus::ImportedAssets;

#[cfg(test)]
mod tests;

/// An unproved importer identity or source byte span rejects selection.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Fault {
    /// Missing or ambiguous definition-to-upload assignment.
    Binding,
    /// Actual selected figure/step or top/bottom identity does not match checked content.
    Selection,
    /// Missing native definition or inconsistent pointer/header roundtrip.
    Definition,
    /// Selected bytes do not match the actual bound checked graphics range.
    Bytes,
}

/// Complete immutable provenance and content for one actually selected upload.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct BoundUpload {
    /// Engine-owned selected upload identity.
    pub tiles: TilesId,
    /// Exact definition assigned to that upload by this actual import.
    pub definition: TileDefinitionId,
    /// Checked native descriptor identity, for exact publication at the edge.
    pub descriptor: u16,
    /// Exact encoded first length; zero requests 65,536 checked bytes.
    pub first_length: u16,
    /// Exact encoded second length; zero has no second transfer.
    pub second_length: u16,
    /// Complete first source region from actual imported selected bytes.
    pub first: ContentRegion,
    /// Complete second source region, absent only for an encoded zero second length.
    pub second: Option<ContentRegion>,
}

/// Actual ordered top/bottom outputs, not captured future prepared descriptors.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct BoundFrame {
    /// Actual selected top upload.
    pub top: BoundUpload,
    /// Actual selected bottom upload when present.
    pub bottom: Option<BoundUpload>,
}

/// Immutable content used to qualify selected outputs at the verification/import edge.
pub struct Content<'a> {
    /// Actual assets and retained pool-insertion identities from the same import.
    pub imported: &'a ImportedAssets,
    /// Checked native frame/definition/pointer content.
    pub tiles: &'a SamusTiles,
    /// Checked immutable source graphics.
    pub graphics: &'a Graphics,
    /// Existing checked clean-step/native-frame identities.
    pub clips: &'a Clips,
}

/// Exact native header from typed checked definition content, for source roundtrip proof.
#[must_use]
pub const fn header(definition: TileDefinition) -> [u8; 7] {
    let [low, high] = definition.source.offset.to_le_bytes();
    let [first_low, first_high] = definition.first.to_le_bytes();
    let [second_low, second_high] = definition.second.to_le_bytes();
    [
        low,
        high,
        definition.source.bank,
        first_low,
        first_high,
        second_low,
        second_high,
    ]
}

fn first_length(encoded: u16) -> usize {
    if encoded == 0 {
        65536
    } else {
        usize::from(encoded)
    }
}

fn region(source: GraphicsRef, bytes: &[u8]) -> Result<ContentRegion, Fault> {
    let length = u32::try_from(bytes.len()).map_err(|_| Fault::Bytes)?;
    if bytes.is_empty()
        || u32::from(source.offset)
            .checked_add(length)
            .is_none_or(|end| end > 65536)
    {
        return Err(Fault::Bytes);
    }
    Ok(ContentRegion {
        start: BusAddress {
            bank: source.bank,
            offset: source.offset,
        },
        bytes: bytes.to_vec(),
    })
}

impl Content<'_> {
    /// Bind a selected asset identity through the preserved actual importer map.
    /// Equal bytes never substitute for definition identity, and ambiguous maps fail.
    /// # Errors
    /// Unknown/ambiguous assignment, wrong selected definition or mismatched checked bytes.
    pub fn upload(
        &self,
        selected: TilesId,
        selected_definition: TileDefinitionId,
        actual: &TileUpload,
    ) -> Result<BoundUpload, Fault> {
        let mut assignments = self
            .imported
            .upload_bindings
            .iter()
            .filter(|(_, id)| **id == selected);
        let definition = *assignments.next().ok_or(Fault::Binding)?.0;
        if assignments.next().is_some() || definition != selected_definition {
            return Err(Fault::Binding);
        }
        let imported = self
            .imported
            .assets
            .sheet
            .upload(selected)
            .ok_or(Fault::Binding)?;
        if imported != actual {
            return Err(Fault::Bytes);
        }
        let decoded = *self.tiles.definition(definition).ok_or(Fault::Definition)?;
        let descriptor = self.tiles.pointer(definition).ok_or(Fault::Definition)?;
        if self.tiles.at_pointer(descriptor) != Some(definition)
            || TileDefinition::decode(header(decoded)) != decoded
        {
            return Err(Fault::Definition);
        }
        let first = self
            .graphics
            .bytes(decoded.source, first_length(decoded.first))
            .ok_or(Fault::Bytes)?;
        if actual.first != first {
            return Err(Fault::Bytes);
        }
        let second = if decoded.second == 0 {
            if !actual.second.is_empty() {
                return Err(Fault::Bytes);
            }
            None
        } else {
            let source = decoded.second_source();
            let bytes = self
                .graphics
                .bytes(source, usize::from(decoded.second))
                .ok_or(Fault::Bytes)?;
            if actual.second != bytes {
                return Err(Fault::Bytes);
            }
            Some(region(source, &actual.second)?)
        };
        Ok(BoundUpload {
            tiles: selected,
            definition,
            descriptor,
            first_length: decoded.first,
            second_length: decoded.second,
            first: region(decoded.source, &actual.first)?,
            second,
        })
    }

    /// Qualify the actual model-selected figure/step and its actual top/bottom upload outputs.
    /// # Errors
    /// Unavailable or ambiguous clean/native picture identity or mismatched actual uploads.
    pub fn bind(
        &self,
        receipt: &NormalDrawingReceipt,
        actual: &[(TilesId, &TileUpload)],
    ) -> Result<BoundFrame, Fault> {
        self.bind_figure(
            receipt.figure.0,
            samus_appearance::Step(receipt.shown_step.0),
            actual,
        )
    }

    pub(super) fn bind_figure(
        &self,
        figure: samus_appearance::Figure,
        step: samus_appearance::Step,
        actual: &[(TilesId, &TileUpload)],
    ) -> Result<BoundFrame, Fault> {
        let mut figures = self
            .clips
            .poses()
            .iter()
            .filter(|clip| clip.figure == figure);
        let imported = figures.next().ok_or(Fault::Selection)?;
        if figures.next().is_some() {
            return Err(Fault::Selection);
        }
        let frame = imported.frame_of(step).ok_or(Fault::Selection)?;
        let selected = self
            .tiles
            .frame(imported.id, frame)
            .ok_or(Fault::Selection)?;
        let mut outputs = actual.iter();
        let &(top, top_bytes) = outputs.next().ok_or(Fault::Selection)?;
        let top = self.upload(top, selected.top, top_bytes)?;
        let bottom = if let Some(definition) = selected.bottom {
            let &(bottom, bytes) = outputs.next().ok_or(Fault::Selection)?;
            Some(self.upload(bottom, definition, bytes)?)
        } else {
            None
        };
        if outputs.next().is_some() {
            return Err(Fault::Selection);
        }
        Ok(BoundFrame { top, bottom })
    }

    /// Prove the bound native descriptor's complete header against its immutable ROM source.
    /// # Errors
    /// Missing source bytes or a different descriptor/header identity.
    pub fn source(&self, bound: &BoundUpload, rom: Lorom<'_>) -> Result<(), Fault> {
        let definition = *self
            .tiles
            .definition(bound.definition)
            .ok_or(Fault::Definition)?;
        if self.tiles.pointer(bound.definition) != Some(bound.descriptor) {
            return Err(Fault::Definition);
        }
        let bytes = rom
            .read(RomAddress::new(0x92, bound.descriptor), 7)
            .map_err(|_| Fault::Definition)?;
        if bytes != header(definition) {
            return Err(Fault::Definition);
        }
        Ok(())
    }
}
