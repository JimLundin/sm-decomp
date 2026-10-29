//! Resident incarnation publication and post-raster immutable object history.
#![forbid(unsafe_code)]

use super::{
    records::{
        AtlasBinding, CompiledImage, MapBinding, MediaRef, ObjectContext, PaletteIncarnation,
        ProducedObject, SceneMedia,
    },
    state::{DisplayError, DisplayState},
};
use std::sync::Arc;

/// A short borrow of the actual resident owner, independent of visible history.
pub struct Resident<'a> {
    pub image: MediaRef,
    pub palette: MediaRef,
    pub words: &'a mut Vec<u16>,
    pub colors: &'a [u16; 256],
}
/// G supplies these authored identities/bindings; publication allocates no IDs.
pub struct Incarnation {
    pub image: MediaRef,
    pub palette: MediaRef,
    pub atlases: Arc<[AtlasBinding]>,
    pub maps: Arc<[MapBinding]>,
}
pub fn append(
    resident: Resident<'_>,
    media: &SceneMedia,
    incarnation: Incarnation,
    aliases: impl FnOnce() -> [u16; 5],
) -> Result<Arc<SceneMedia>, DisplayError> {
    if incarnation.image.asset != resident.image.asset
        || incarnation.palette.asset != resident.palette.asset
    {
        return Err(DisplayError::Identity);
    }
    if resident.words.len() < 0x801f {
        return Err(DisplayError::Incomplete);
    }
    resident
        .words
        .get_mut(0x8000..0x8005)
        .ok_or(DisplayError::Incomplete)?
        .copy_from_slice(&aliases());
    resident.words[0x8005..0x801f].copy_from_slice(&resident.colors[..26]);
    let mut palettes: Vec<_> = media.palette_incarnations().cloned().collect();
    palettes.push(PaletteIncarnation {
        reference: incarnation.palette,
        colors: *resident.colors,
    });
    let mut images: Vec<_> = media.compiled_images().cloned().collect();
    images.push(CompiledImage {
        reference: incarnation.image,
        palette: incarnation.palette,
        words: Arc::from(resident.words.clone()),
        atlases: incarnation.atlases,
        maps: incarnation.maps,
    });
    SceneMedia::new(
        media.revision(),
        media.assets().cloned().collect(),
        palettes,
        images,
    )
    .map(Arc::new)
    .map_err(|_| DisplayError::Scene)
}

/// The callback may update resident pixels and replace current media. Read every
/// row input afterward; the G raster algorithm and row identities stay explicit.
pub trait RasterHost {
    fn draw_row(
        &mut self,
        state: &mut DisplayState,
        row: u16,
    ) -> Result<ObjectContext, DisplayError>;
    fn media(&self) -> &SceneMedia;
    fn produced(&self) -> &[ProducedObject];
    fn objects(&self) -> &[u8; 544];
    fn publish_row(&mut self, records: Arc<[ProducedObject]>);
}
pub fn raster_row(
    host: &mut impl RasterHost,
    state: &mut DisplayState,
    row: u16,
) -> Result<(), DisplayError> {
    let context = host.draw_row(state, row)?;
    let records = host
        .produced()
        .iter()
        .map(|record| {
            record
                .for_line(host.media(), context, host.objects())
                .map_err(|_| DisplayError::Scene)
        })
        .collect::<Result<Vec<_>, _>>()?;
    host.publish_row(Arc::from(records));
    Ok(())
}
