//! Lossless committed display fields; decoding never executes producers or delivery.

use super::commit::ResidentDisplayDescription;
use super::component::DisplayStartMetadata;
use super::{atmosphere as a, hdma_sources as h, records as r, samus as s, start as t, state as d};
use crate::actors::snapshot::CheckpointReader;
use crate::visual_state as v;
use std::{collections::BTreeMap, sync::Arc};

// An explicit field codec, independent of Rust layout and compatibility addresses.
// Each vector count is bounded by the remaining encoded bytes before allocation.
trait Field: Sized {
    fn put(&self, bytes: &mut Vec<u8>);
    fn get(read: &mut CheckpointReader<'_>) -> Result<Self, String>;
}
macro_rules! scalar {
    ($($kind:ty),+ $(,)?) => { $(
        impl Field for $kind {
            fn put(&self, bytes: &mut Vec<u8>) { bytes.extend_from_slice(&self.to_le_bytes()); }
            fn get(read: &mut CheckpointReader<'_>) -> Result<Self, String> {
                Ok(Self::from_le_bytes(read.bytes(core::mem::size_of::<Self>())?.try_into().unwrap()))
            }
        }
    )+ };
}
scalar!(u8, u16, u32, u64, i16, i32);
impl Field for usize {
    fn put(&self, bytes: &mut Vec<u8>) {
        (*self as u64).put(bytes);
    }
    fn get(read: &mut CheckpointReader<'_>) -> Result<Self, String> {
        usize::try_from(u64::get(read)?).map_err(|_| "display checkpoint size overflow".into())
    }
}
impl Field for bool {
    fn put(&self, bytes: &mut Vec<u8>) {
        u8::from(*self).put(bytes);
    }
    fn get(read: &mut CheckpointReader<'_>) -> Result<Self, String> {
        match u8::get(read)? {
            0 => Ok(false),
            1 => Ok(true),
            _ => Err("invalid display checkpoint boolean".into()),
        }
    }
}
impl<T: Field> Field for Option<T> {
    fn put(&self, bytes: &mut Vec<u8>) {
        self.is_some().put(bytes);
        if let Some(value) = self {
            value.put(bytes);
        }
    }
    fn get(read: &mut CheckpointReader<'_>) -> Result<Self, String> {
        if bool::get(read)? {
            Ok(Some(T::get(read)?))
        } else {
            Ok(None)
        }
    }
}
impl<T: Field, const N: usize> Field for [T; N] {
    fn put(&self, bytes: &mut Vec<u8>) {
        for value in self {
            value.put(bytes);
        }
    }
    fn get(read: &mut CheckpointReader<'_>) -> Result<Self, String> {
        let mut values = Vec::new();
        for _ in 0..N {
            values.push(T::get(read)?);
        }
        values
            .try_into()
            .map_err(|_| "display checkpoint array length".into())
    }
}
fn count(read: &mut CheckpointReader<'_>) -> Result<usize, String> {
    let count = usize::get(read)?;
    if count > read.0.len() {
        return Err("truncated display checkpoint sequence".into());
    }
    Ok(count)
}
impl<T: Field> Field for Vec<T> {
    fn put(&self, bytes: &mut Vec<u8>) {
        self.len().put(bytes);
        for value in self {
            value.put(bytes);
        }
    }
    fn get(read: &mut CheckpointReader<'_>) -> Result<Self, String> {
        let count = count(read)?;
        let mut values = Vec::new();
        for _ in 0..count {
            values.push(T::get(read)?);
        }
        Ok(values)
    }
}
impl<T: Field> Field for Arc<[T]> {
    fn put(&self, bytes: &mut Vec<u8>) {
        self.len().put(bytes);
        for value in self.iter() {
            value.put(bytes);
        }
    }
    fn get(read: &mut CheckpointReader<'_>) -> Result<Self, String> {
        Ok(Vec::<T>::get(read)?.into())
    }
}
impl<T: Field> Field for Arc<T> {
    fn put(&self, bytes: &mut Vec<u8>) {
        self.as_ref().put(bytes);
    }
    fn get(read: &mut CheckpointReader<'_>) -> Result<Self, String> {
        Ok(Arc::new(T::get(read)?))
    }
}
impl<K: Field + Ord, V: Field> Field for BTreeMap<K, V> {
    fn put(&self, bytes: &mut Vec<u8>) {
        self.len().put(bytes);
        for (key, value) in self {
            key.put(bytes);
            value.put(bytes);
        }
    }
    fn get(read: &mut CheckpointReader<'_>) -> Result<Self, String> {
        let count = count(read)?;
        let mut values = Self::new();
        for _ in 0..count {
            let key = K::get(read)?;
            if values
                .last_key_value()
                .is_some_and(|(last, _)| last >= &key)
            {
                return Err("unordered display checkpoint map".into());
            }
            values.insert(key, V::get(read)?);
        }
        Ok(values)
    }
}
macro_rules! record {
    ($kind:path { $($field:ident),+ $(,)? }) => {
        impl Field for $kind {
            fn put(&self, bytes: &mut Vec<u8>) { $(self.$field.put(bytes);)+ }
            fn get(read: &mut CheckpointReader<'_>) -> Result<Self, String> {
                Ok(Self { $($field: Field::get(read)?,)+ })
            }
        }
    };
}
macro_rules! identity {
    ($kind:path, $inner:ty) => {
        impl Field for $kind {
            fn put(&self, bytes: &mut Vec<u8>) {
                self.0.put(bytes);
            }
            fn get(read: &mut CheckpointReader<'_>) -> Result<Self, String> {
                Ok(Self(<$inner>::get(read)?))
            }
        }
    };
}
macro_rules! choices {
    ($kind:path { $($tag:literal => $variant:ident),+ $(,)? }) => {
        impl Field for $kind {
            fn put(&self, bytes: &mut Vec<u8>) {
                let tag: u8 = match self { $(Self::$variant => $tag,)+ };
                tag.put(bytes);
            }
            fn get(read: &mut CheckpointReader<'_>) -> Result<Self, String> {
                match u8::get(read)? { $($tag => Ok(Self::$variant),)+ _ => Err("unknown display checkpoint variant".into()) }
            }
        }
    };
}
identity!(r::AssetId, u32);
identity!(r::ProducerObjectId, u64);
identity!(r::Revision, [u8; 32]);
record!(r::AssetRef { id, revision });
record!(r::MediaRef { asset, incarnation });
record!(r::DisplayProducerIds { next });
record!(r::ScreenPosition { x, y });
record!(r::SceneView { room, camera });
choices!(r::ColorInterpretation { 0 => Bgr15Native });
choices!(r::SceneQuantization { 0 => Native, 1 => Reference });
record!(r::SceneGeometry {
    width,
    height,
    crop_origin,
    interlaced,
    even_frame,
    overscan,
    color,
    quantization
});
record!(r::DisplayStatus {
    range_overflow,
    time_overflow,
    has_objects,
    even_frame,
    interlaced,
    overscan
});
record!(r::ObjectOverflow { range, time });
choices!(r::TileEncoding { 0 => Planar2, 1 => Planar4, 2 => Affine8 });
record!(r::TileAtlas {
    reference,
    encoding,
    tile_count,
    data
});
record!(r::TileCell {
    tile,
    palette,
    priority,
    flip_horizontal,
    flip_vertical
});
record!(r::Tilemap {
    reference,
    atlas,
    palette,
    width,
    height,
    cells
});
record!(r::Palette { reference, colors });
choices!(r::ObjectSize { 0 => Small, 1 => Large });
record!(r::ObjectPartBinding {
    visual,
    frame,
    part
});
record!(r::ObjectSizeVariants { small, large });
record!(r::ObjectVariants { size_sets });
record!(r::VisualPart {
    atlas,
    material,
    tile,
    offset,
    size
});
record!(r::VisualFrame { parts });
record!(r::Visual { reference, frames });
record!(r::Font { reference, atlas });
impl Field for r::SceneAsset {
    fn put(&self, bytes: &mut Vec<u8>) {
        match self {
            Self::Atlas(value) => {
                0u8.put(bytes);
                value.put(bytes);
            }
            Self::Map(value) => {
                1u8.put(bytes);
                value.put(bytes);
            }
            Self::Palette(value) => {
                2u8.put(bytes);
                value.put(bytes);
            }
            Self::Visual(value) => {
                3u8.put(bytes);
                value.put(bytes);
            }
            Self::Font(value) => {
                4u8.put(bytes);
                value.put(bytes);
            }
            Self::Room(value) => {
                5u8.put(bytes);
                value.put(bytes);
            }
        }
    }
    fn get(read: &mut CheckpointReader<'_>) -> Result<Self, String> {
        match u8::get(read)? {
            0 => Ok(Self::Atlas(Field::get(read)?)),
            1 => Ok(Self::Map(Field::get(read)?)),
            2 => Ok(Self::Palette(Box::new(Field::get(read)?))),
            3 => Ok(Self::Visual(Field::get(read)?)),
            4 => Ok(Self::Font(Field::get(read)?)),
            5 => Ok(Self::Room(Field::get(read)?)),
            _ => Err("unknown display checkpoint asset".into()),
        }
    }
}
record!(r::PaletteIncarnation { reference, colors });
record!(r::AtlasBinding {
    atlas,
    first_word,
    resident_data
});
record!(r::MapBinding { map, first_word });
record!(r::CompiledImage {
    reference,
    palette,
    words,
    atlases,
    maps
});
impl Field for r::SceneMedia {
    fn put(&self, bytes: &mut Vec<u8>) {
        self.revision().put(bytes);
        self.assets().cloned().collect::<Vec<_>>().put(bytes);
        self.palette_incarnations()
            .cloned()
            .collect::<Vec<_>>()
            .put(bytes);
        self.compiled_images()
            .cloned()
            .collect::<Vec<_>>()
            .put(bytes);
    }
    fn get(read: &mut CheckpointReader<'_>) -> Result<Self, String> {
        Self::new(
            Field::get(read)?,
            Field::get(read)?,
            Field::get(read)?,
            Field::get(read)?,
        )
        .map_err(|error| format!("display checkpoint media: {error:?}"))
    }
}
choices!(r::PlaneRole { 0 => Main, 1 => Secondary, 2 => HudFx });
record!(r::ScenePlane {
    role,
    map,
    atlas,
    palette,
    width,
    height,
    cells,
    scroll
});
record!(r::SceneObject {
    order,
    visual,
    frame,
    part,
    atlas,
    material,
    tile,
    position,
    size,
    palette,
    priority,
    flip_horizontal,
    flip_vertical
});
record!(r::ProducedObject {
    producer,
    object,
    variants,
    context,
    native_word
});
record!(r::ObjectSelection {
    atlases,
    size_set,
    first_order,
    interlaced,
    even_frame
});
record!(r::ObjectContext { selection, palette });
record!(r::WindowExtent { first, last });
choices!(r::WindowCombination { 0 => Union, 1 => Intersection, 2 => Exclusive, 3 => Equal });
record!(r::LayerWindow {
    first_enabled,
    first_inverted,
    second_enabled,
    second_inverted,
    combination
});
record!(r::SceneWindows { extents, layers });
record!(r::ScreenRouting { visible, masked });
choices!(r::WindowCondition { 0 => Never, 1 => Inside, 2 => Outside, 3 => Always });
record!(r::SceneColor {
    clip,
    prevent_math,
    enabled,
    use_subscreen,
    subtract,
    half,
    fixed,
    brightness
});
record!(r::SceneAffine {
    matrix,
    center,
    scroll,
    flip_horizontal,
    flip_vertical,
    bounded_field,
    fill_outside
});
record!(r::ScenePixelation {
    enabled,
    size,
    origin,
    modulo
});
impl Field for r::SceneMode {
    fn put(&self, bytes: &mut Vec<u8>) {
        match self {
            Self::TilePlanes => 0u8.put(bytes),
            Self::AffinePlane { extended } => {
                1u8.put(bytes);
                extended.put(bytes);
            }
        }
    }
    fn get(read: &mut CheckpointReader<'_>) -> Result<Self, String> {
        match u8::get(read)? {
            0 => Ok(Self::TilePlanes),
            1 => Ok(Self::AffinePlane {
                extended: Field::get(read)?,
            }),
            _ => Err("unknown display checkpoint mode".into()),
        }
    }
}
record!(r::SignedObjectPixel { x, value });
record!(r::SceneLine {
    media,
    palette,
    mode,
    blank,
    planes,
    objects,
    object_selection,
    windows,
    routing,
    affine,
    pixelation,
    color,
    object_aliases
});
record!(r::SceneDescription {
    tick,
    gameplay,
    reference,
    revision,
    view,
    geometry,
    resident,
    lines,
    status,
    initial_object_overflow
});
record!(ResidentDisplayDescription {
    image,
    palette,
    words,
    colors,
    objects,
    produced
});

record!(v::FirefleaLighting {
    timer,
    flash_index,
    compatibility_word,
    darkness,
    compatibility_limit,
    compatibility_shade
});
record!(v::PaletteSlot {
    definition,
    color_offset,
    pre_action,
    program,
    delay,
    counter
});
record!(v::PaletteEffects {
    slots,
    flags,
    selected,
    heat,
    previous_heat
});
record!(v::AnimationSlot {
    definition,
    counter,
    program,
    delay,
    source,
    size,
    vram
});
record!(v::AnimatedTiles {
    slots,
    flags,
    selected,
    last_instruction
});
record!(v::PaletteFade {
    step,
    duration,
    cursor
});
record!(v::ScreenFade { delay, counter });
record!(v::Visuals {
    fireflea,
    colors,
    targets,
    palette_fade,
    screen_fade,
    palette_effects,
    animated_tiles
});
record!(v::PlayerPalette {
    hurt_flash,
    visor_timer,
    suit_index,
    xray_progress,
    timer,
    suit_beam_speed,
    suit_red_green,
    crystal_flash_delay
});
record!(a::AtmosphericSlot {
    remaining,
    x,
    y,
    frame_and_kind
});
record!(a::AtmosphericEffects { slots });
record!(a::AtmosphericProducers { slots });
record!(a::AtmosphericTile {
    native_word,
    variants
});
record!(a::AtmosphericKind {
    duration_pointer,
    durations,
    frame_count,
    tile_pointer,
    tiles
});
record!(a::AtmosphericSpritePart {
    x,
    y,
    native_word,
    variants
});
record!(a::AtmosphericSprite { present, parts });
record!(a::AtmosphereCatalog {
    revision,
    kinds,
    sprites
});
record!(s::SamusPart {
    x,
    y,
    native_word,
    variants
});
record!(s::SamusMap { count, parts });
record!(s::SamusCatalog {
    revision,
    pointers,
    maps
});

record!(h::DefinitionRef {
    id,
    gameplay,
    definition
});
record!(h::HdmaTableDefinition {
    reference,
    schema,
    bytes
});
record!(h::HdmaIndirectDefinition {
    reference,
    schema,
    bytes
});
record!(h::HdmaProgramDefinition {
    reference,
    schema,
    bytes
});
impl Field for h::HdmaDefinition {
    fn put(&self, bytes: &mut Vec<u8>) {
        match self {
            Self::Table(value) => {
                0u8.put(bytes);
                value.put(bytes);
            }
            Self::Indirect(value) => {
                1u8.put(bytes);
                value.put(bytes);
            }
            Self::Program(value) => {
                2u8.put(bytes);
                value.put(bytes);
            }
        }
    }
    fn get(read: &mut CheckpointReader<'_>) -> Result<Self, String> {
        match u8::get(read)? {
            0 => Ok(Self::Table(Field::get(read)?)),
            1 => Ok(Self::Indirect(Field::get(read)?)),
            2 => Ok(Self::Program(Field::get(read)?)),
            _ => Err("unknown display checkpoint HDMA definition".into()),
        }
    }
}
impl Field for h::HdmaCatalog {
    fn put(&self, bytes: &mut Vec<u8>) {
        self.gameplay().put(bytes);
        self.checkpoint_definitions().put(bytes);
    }
    fn get(read: &mut CheckpointReader<'_>) -> Result<Self, String> {
        let gameplay = Field::get(read)?;
        let values = <BTreeMap<r::AssetId, h::HdmaDefinition> as Field>::get(read)?;
        if values
            .iter()
            .any(|(key, value)| *key != value.reference().id)
        {
            return Err("display checkpoint HDMA definition key mismatch".into());
        }
        Self::new(gameplay, values.into_values().collect())
            .map_err(|error| format!("display checkpoint HDMA catalog: {error:?}"))
    }
}
record!(h::BankWindow {
    bank,
    address,
    offset,
    length
});
choices!(h::HdmaVariable { 0 => A, 1 => B, 2 => C, 3 => D, 4 => E, 5 => F });
impl Field for h::LiveOwner {
    fn put(&self, bytes: &mut Vec<u8>) {
        match self {
            Self::MessageDescriptor => 0u8.put(bytes),
            Self::MessageScanlines => 1u8.put(bytes),
            Self::HdmaTable1 => 2u8.put(bytes),
            Self::HdmaTable2 => 3u8.put(bytes),
            Self::WindowLeft => 4u8.put(bytes),
            Self::WindowRight => 5u8.put(bytes),
            Self::SkyBg2 => 6u8.put(bytes),
            Self::CrocomireBg2 => 7u8.put(bytes),
            Self::ObjectVariable { object, variable } => {
                8u8.put(bytes);
                object.put(bytes);
                variable.put(bytes);
            }
        }
    }
    fn get(read: &mut CheckpointReader<'_>) -> Result<Self, String> {
        match u8::get(read)? {
            0 => Ok(Self::MessageDescriptor),
            1 => Ok(Self::MessageScanlines),
            2 => Ok(Self::HdmaTable1),
            3 => Ok(Self::HdmaTable2),
            4 => Ok(Self::WindowLeft),
            5 => Ok(Self::WindowRight),
            6 => Ok(Self::SkyBg2),
            7 => Ok(Self::CrocomireBg2),
            8 => Ok(Self::ObjectVariable {
                object: Field::get(read)?,
                variable: Field::get(read)?,
            }),
            _ => Err("unknown display checkpoint live owner".into()),
        }
    }
}
impl Field for h::HdmaRegionSource {
    fn put(&self, bytes: &mut Vec<u8>) {
        match self {
            Self::Buffer { storage, offset } => {
                0u8.put(bytes);
                storage.put(bytes);
                offset.put(bytes);
            }
            Self::ObjectVariable { object, variable } => {
                1u8.put(bytes);
                object.put(bytes);
                variable.put(bytes);
            }
        }
    }
    fn get(read: &mut CheckpointReader<'_>) -> Result<Self, String> {
        match u8::get(read)? {
            0 => Ok(Self::Buffer {
                storage: Field::get(read)?,
                offset: Field::get(read)?,
            }),
            1 => Ok(Self::ObjectVariable {
                object: Field::get(read)?,
                variable: Field::get(read)?,
            }),
            _ => Err("unknown display checkpoint HDMA region".into()),
        }
    }
}
record!(h::HdmaLiveRegion {
    owner,
    source,
    length
});
impl Field for h::HdmaLiveDescription {
    fn put(&self, bytes: &mut Vec<u8>) {
        let (buffers, regions) = self.checkpoint_parts();
        buffers.put(bytes);
        regions.put(bytes);
    }
    fn get(read: &mut CheckpointReader<'_>) -> Result<Self, String> {
        let buffers = <BTreeMap<h::LiveOwner, Vec<u8>> as Field>::get(read)?;
        let regions = <BTreeMap<h::LiveOwner, h::HdmaLiveRegion> as Field>::get(read)?;
        if regions.iter().any(|(key, region)| *key != region.owner) {
            return Err("display checkpoint HDMA region key mismatch".into());
        }
        Self::new(
            buffers
                .into_iter()
                .map(|(owner, bytes)| h::HdmaOwnedBuffer { owner, bytes })
                .collect(),
            regions.into_values().collect(),
        )
        .map_err(|error| format!("display checkpoint HDMA live topology: {error:?}"))
    }
}
impl Field for h::SourceBinding {
    fn put(&self, bytes: &mut Vec<u8>) {
        match self {
            Self::Prepared { reference, windows } => {
                0u8.put(bytes);
                reference.put(bytes);
                windows.put(bytes);
            }
            Self::LiveOwner { owner, windows } => {
                1u8.put(bytes);
                owner.put(bytes);
                windows.put(bytes);
            }
            Self::Unbound => 2u8.put(bytes),
        }
    }
    fn get(read: &mut CheckpointReader<'_>) -> Result<Self, String> {
        match u8::get(read)? {
            0 => Ok(Self::Prepared {
                reference: Field::get(read)?,
                windows: Field::get(read)?,
            }),
            1 => Ok(Self::LiveOwner {
                owner: Field::get(read)?,
                windows: Field::get(read)?,
            }),
            2 => Ok(Self::Unbound),
            _ => Err("unknown display checkpoint source binding".into()),
        }
    }
}

choices!(d::WordOrder { 0 => Little, 1 => Big });
record!(d::SourceSpan {
    asset,
    bytes,
    offset
});
impl Field for d::TransferSource {
    fn put(&self, bytes: &mut Vec<u8>) {
        match self {
            Self::Asset(value) => {
                0u8.put(bytes);
                value.put(bytes);
            }
            Self::Hud { offset } => {
                1u8.put(bytes);
                offset.put(bytes);
            }
            Self::Objects { offset } => {
                2u8.put(bytes);
                offset.put(bytes);
            }
            Self::ActorTilemap { offset } => {
                3u8.put(bytes);
                offset.put(bytes);
            }
            Self::Scroll {
                slot,
                half,
                wrapped,
                offset,
            } => {
                4u8.put(bytes);
                slot.put(bytes);
                half.put(bytes);
                wrapped.put(bytes);
                offset.put(bytes);
            }
        }
    }
    fn get(read: &mut CheckpointReader<'_>) -> Result<Self, String> {
        match u8::get(read)? {
            0 => Ok(Self::Asset(Field::get(read)?)),
            1 => Ok(Self::Hud {
                offset: Field::get(read)?,
            }),
            2 => Ok(Self::Objects {
                offset: Field::get(read)?,
            }),
            3 => Ok(Self::ActorTilemap {
                offset: Field::get(read)?,
            }),
            4 => Ok(Self::Scroll {
                slot: Field::get(read)?,
                half: Field::get(read)?,
                wrapped: Field::get(read)?,
                offset: Field::get(read)?,
            }),
            _ => Err("unknown display checkpoint transfer source".into()),
        }
    }
}
choices!(d::ByteLanes { 0 => Both, 1 => Low, 2 => High });
record!(d::TileDestination {
    word,
    step,
    lanes,
    remap,
    increment_on_high
});
record!(d::TileTransfer {
    source,
    count,
    destination
});
record!(d::ObjectStaging {
    bytes,
    cursor,
    native_order,
    produced
});
record!(d::HudState {
    tiles,
    health,
    maximum_health,
    previous_health,
    ammunition,
    maximum_ammunition,
    previous_ammunition,
    selection,
    previous_selection,
    auto_cancel,
    reserve_mode,
    reserve_health,
    palette,
    media
});
record!(d::NmiState {
    pending,
    frame_byte,
    frame_word,
    including_lag,
    missed,
    max_missed,
    acknowledgment
});
record!(d::SamusHalf {
    pointer,
    enabled,
    descriptor,
    first,
    second
});
choices!(d::ScrollLayout { 0 => Column, 1 => Row });
record!(d::ScrollTransfer {
    enabled,
    layout,
    first,
    wrapped,
    wrapped_count,
    first_bytes,
    wrapped_bytes
});
record!(d::QueuedUpload { cursor, transfer });
impl Field for d::Mode7Target {
    fn put(&self, bytes: &mut Vec<u8>) {
        match self {
            Self::Tiles(value) => {
                0u8.put(bytes);
                value.put(bytes);
            }
            Self::Palette { first_color } => {
                1u8.put(bytes);
                first_color.put(bytes);
            }
        }
    }
    fn get(read: &mut CheckpointReader<'_>) -> Result<Self, String> {
        match u8::get(read)? {
            0 => Ok(Self::Tiles(Field::get(read)?)),
            1 => Ok(Self::Palette {
                first_color: Field::get(read)?,
            }),
            _ => Err("unknown display checkpoint mode7 target".into()),
        }
    }
}
record!(d::Mode7Upload {
    cursor,
    source,
    count,
    target,
    transfer_mode
});
record!(d::Readback {
    cursor,
    destination,
    first_word,
    count
});
record!(d::ActorDrawQueue {
    size,
    cursor,
    actors
});
record!(d::ActorTileStaging {
    pending,
    size,
    source,
    destination,
    transfer_source,
    transfer_destination,
    tilemap_bytes
});
record!(d::BackgroundState {
    tilemap,
    atlas,
    palette,
    camera,
    scroll,
    screen_layout,
    character_layout
});
record!(d::DisplayControls {
    brightness,
    object_layout,
    mode,
    fake_mode,
    mosaic,
    screens,
    previous_background3_map,
    characters,
    affine_selection,
    window_selection,
    window_positions,
    window_logic,
    main_layers,
    previous_main_layers,
    sub_layers,
    main_window,
    sub_window,
    next_color_selection,
    color_selection,
    previous_color_selection,
    next_color_math,
    color_math,
    previous_color_math,
    fixed_color,
    geometry,
    scroll_words,
    affine_words,
    nmi_enable,
    hdma_enable,
    committed_fake_mode
});
record!(d::HdmaBinding {
    enable,
    bank_slot,
    table
});
record!(d::HdmaObject {
    instruction,
    instruction_cursor,
    instruction_timer,
    pre_instruction,
    pre_instruction_bank,
    timer,
    variables
});
record!(d::HdmaChannel {
    source,
    table,
    active,
    terminated,
    do_transfer,
    repeat_count,
    source_address,
    table_cursor,
    indirect_cursor,
    mode,
    from_destination,
    indirect,
    fixed,
    decrement,
    source_bank,
    indirect_bank,
    destination,
    transfer_cursor,
    unused_bit,
    unused_byte,
    dma_active
});
record!(d::HdmaState {
    bindings,
    objects,
    channels,
    selected,
    flags,
    hdma_timer,
    dma_timer,
    dma_busy
});
record!(d::InterruptState {
    current,
    pending,
    vertical,
    horizontal,
    previous_cre,
    current_cre,
    transition_flag,
    transition_continuation
});
record!(d::DisplayDeliveryDescription {
    horizontal_position,
    vertical_position,
    beam_frames,
    horizontal_irq_enabled,
    vertical_irq_enabled,
    nmi_enabled,
    nmi_wanted,
    irq_wanted,
    in_nmi,
    in_irq,
    in_vblank,
    automatic_input_read,
    automatic_input_timer,
    counter_latch_gate,
    raster_suppressed
});
record!(d::EffectiveDisplayDescription {
    background3_map_words,
    background3_map_wider,
    background3_map_higher,
    color_clip,
    color_prevent_math,
    color_add_subscreen,
    direct_color,
    color_math_enabled,
    color_subtract,
    color_half,
    main_layers,
    forced_blank
});
record!(d::DisplayLatches {
    tile_pointer,
    tile_increment,
    tile_remap,
    tile_read_buffer,
    tile_increment_on_high,
    tile_padding,
    palette_pointer,
    palette_second,
    palette_buffer,
    object_written_address,
    object_written_high_table,
    object_address,
    object_high_table,
    object_second,
    object_buffer,
    scroll_previous,
    scroll_previous_second,
    affine_previous,
    latched_horizontal,
    latched_vertical,
    horizontal_second,
    vertical_second,
    counter_latched,
    open_bus,
    ppu_open_bus
});
record!(d::RasterState {
    range_overflow,
    time_overflow,
    has_objects,
    even_frame,
    interlaced,
    frame_interlaced,
    overscan,
    frame_overscan,
    mosaic_origin,
    mosaic_phase,
    visible
});
choices!(d::DisplayPhase { 0 => Committed, 1 => Producing, 2 => Refreshed, 3 => Settling, 4 => Failed });
// Preserve the original destination/payload result pair and its ordered outbox.
impl Field for (r::AssetRef, Vec<u8>) {
    fn put(&self, bytes: &mut Vec<u8>) {
        self.0.put(bytes);
        self.1.put(bytes);
    }
    fn get(read: &mut CheckpointReader<'_>) -> Result<Self, String> {
        Ok((r::AssetRef::get(read)?, Vec::<u8>::get(read)?))
    }
}

record!(d::DisplayDescription {
    media_revision,
    phase,
    objects,
    hud,
    visual,
    player_palette,
    atmospheric,
    atmosphere_catalog,
    producer_ids,
    atmospheric_producers,
    nmi,
    samus,
    backgrounds,
    scrolling,
    uploads,
    upload_tail,
    mode7_uploads,
    mode7_tail,
    readbacks,
    readback_tail,
    read_results,
    actor_queues,
    actor_tiles,
    controls,
    hdma,
    hdma_live,
    interrupt,
    delivery,
    effective,
    latches,
    raster,
    animated_sources
});

record!(t::StartIdentity {
    schema,
    gameplay,
    reference,
    capture,
    media,
    committed_tick,
    next_input_record
});
record!(t::SceneImagePolicy {
    width,
    height,
    crop_origin,
    color,
    quantization
});
record!(t::ReferenceProfile {
    fingerprint,
    release,
    capture,
    ticks_numerator,
    ticks_denominator,
    arithmetic_policy,
    input_policy,
    observation_policy,
    color_policy,
    slowdown_policy,
    glitch_policy,
    audio_timing_policy,
    image
});
record!(t::PortHistory {
    sampled,
    previous,
    pressed,
    repeated,
    repeat_counter
});
record!(t::ActionBindings {
    shoot,
    jump,
    run,
    cancel,
    switch,
    aim_down,
    aim_up
});
record!(t::InputHistory {
    capture,
    policy,
    ports,
    debug_second_port,
    repeat_first,
    repeat_next,
    released,
    timed_held,
    previous_timed_held,
    newly_held,
    held_timer,
    held_timer_reset,
    samus_held,
    samus_pressed,
    bindings,
    bug_fix_counter
});
choices!(t::CommittedPhase { 0 => InitialObservation, 1 => SettledInterval });
record!(t::ContinuationHistory {
    capture,
    observation_policy,
    phase,
    next_input_record,
    outer,
    primary,
    secondary,
    music,
    message,
    auxiliary,
    captured_mode
});
record!(t::AudioProgramDeclaration {
    program,
    bank,
    revision,
    definition
});
record!(t::ProgramAnchor {
    program,
    revision,
    source_interval,
    phase,
    timing_policy
});
impl Field for t::OngoingMusic {
    fn put(&self, bytes: &mut Vec<u8>) {
        match self {
            Self::Silent => 0u8.put(bytes),
            Self::Playing(value) => {
                1u8.put(bytes);
                value.put(bytes);
            }
        }
    }
    fn get(read: &mut CheckpointReader<'_>) -> Result<Self, String> {
        match u8::get(read)? {
            0 => Ok(Self::Silent),
            1 => Ok(Self::Playing(Field::get(read)?)),
            _ => Err("unknown display checkpoint music".into()),
        }
    }
}
record!(t::InitialAudioIntent {
    capture,
    revision,
    timing_policy,
    declarations,
    music,
    ambience
});
record!(DisplayStartMetadata {
    identity,
    reference,
    input_history,
    continuation,
    audio,
    samus_content
});
record!(t::DisplayStartRequirements {
    schema,
    gameplay,
    hdma_content,
    atmosphere_content,
    samus_content,
    capture,
    reference,
    media,
    audio_revision,
    audio_programs
});

// Compare independently supplied immutable declarations and complete current G
// metadata. A matching revision alone cannot admit changed catalog/Reference bytes.
fn binding(
    media: &r::SceneMedia,
    metadata: &DisplayStartMetadata,
    required: &t::DisplayStartRequirements,
) -> Result<Vec<u8>, String> {
    t::validate_control_metadata(metadata, required)
        .map_err(|error| format!("display checkpoint metadata: {error:?}"))?;
    media
        .validate()
        .map_err(|error| format!("display checkpoint media: {error:?}"))?;
    let mut bytes = Vec::new();
    metadata.put(&mut bytes);
    required.put(&mut bytes);
    media.put(&mut bytes);
    Ok(bytes)
}

// Seal independently supplied Session bindings, including allocation aliases.
pub fn session_declarations(
    content: &super::samus_content::ContentDescription,
    marker: &super::samus_body::MarkerPublication,
    projection: &[crate::session_owners::ActorQueueProjection; 8],
    publications: &crate::session_owners::SamusPublications,
) -> Vec<u8> {
    use super::samus_content::{Registration, Role};
    let mut bytes = Vec::new();
    content.gameplay.put(&mut bytes);
    content.revision.put(&mut bytes);
    content.allocations.len().put(&mut bytes);
    for (key, allocation) in &content.allocations {
        key.0.put(&mut bytes);
        allocation.identity.0.put(&mut bytes);
        allocation.revision.put(&mut bytes);
        allocation.extent.put(&mut bytes);
        allocation.bytes.put(&mut bytes);
    }
    content.roles.len().put(&mut bytes);
    for ((role, registration), declaration) in &content.roles {
        let role: u8 = match role {
            Role::BodyTop => 0,
            Role::BodyBottom => 1,
            Role::PositionFallback => 2,
            Role::Landing => 3,
            Role::Crouching => 4,
            Role::Drained => 5,
            Role::TilePose => 6,
            Role::TileDefinition => 7,
            Role::TileTop => 8,
            Role::TileBottom => 9,
            Role::UploadDescriptor => 10,
        };
        role.put(&mut bytes);
        let registration: u8 = match registration {
            Registration::Loaded => 0,
            Registration::Native => 1,
        };
        registration.put(&mut bytes);
        declaration.allocation.0.put(&mut bytes);
        declaration.mapped_base.put(&mut bytes);
        declaration.native_order.put(&mut bytes);
    }
    content.requests.len().put(&mut bytes);
    for binding in content.requests.iter() {
        let half: u8 = match binding.half {
            super::samus_body::Half::Top => 0,
            super::samus_body::Half::Bottom => 1,
        };
        half.put(&mut bytes);
        binding.request.pointer.put(&mut bytes);
        binding.request.half.put(&mut bytes);
    }
    marker.producer.put(&mut bytes);
    marker.variants.put(&mut bytes);
    for queue in projection {
        queue.first.put(&mut bytes);
        queue.words.put(&mut bytes);
    }
    publications.body.put(&mut bytes);
    publications.echoes.put(&mut bytes);
    publications.object_atlases.put(&mut bytes);
    publications.object_first_order.put(&mut bytes);
    bytes
}

pub fn encode(
    description: &d::DisplayDescription,
    resident: &ResidentDisplayDescription,
    committed: &r::Scene,
    media: &r::SceneMedia,
    metadata: &DisplayStartMetadata,
    required: &t::DisplayStartRequirements,
) -> Result<Vec<u8>, String> {
    let binding = binding(media, metadata, required)?;
    let mut bytes = Vec::from(b"SMDISPL\x03".as_slice());
    binding.put(&mut bytes);
    description.put(&mut bytes);
    resident.put(&mut bytes);
    committed.description().put(&mut bytes);
    committed.producer_history().to_vec().put(&mut bytes);
    Ok(bytes)
}

pub fn decode(
    bytes: &[u8],
    media: Arc<r::SceneMedia>,
    metadata: &DisplayStartMetadata,
    required: &t::DisplayStartRequirements,
) -> Result<t::ValidatedDisplayStart, String> {
    let expected = binding(&media, metadata, required)?;
    let mut read = CheckpointReader(bytes);
    if read.bytes(8)? != b"SMDISPL\x03" {
        return Err("unsupported display checkpoint schema".into());
    }
    let binding_length = usize::get(&mut read)?;
    if read.bytes(binding_length)? != expected {
        return Err("display checkpoint metadata or immutable content changed".into());
    }
    let description = d::DisplayDescription::get(&mut read)?;
    let resident = ResidentDisplayDescription::get(&mut read)?;
    let committed_description = r::SceneDescription::get(&mut read)?;
    let history = Vec::<Arc<[r::ProducedObject]>>::get(&mut read)?;
    if !read.0.is_empty() {
        return Err("trailing display checkpoint fields".into());
    }
    let committed = r::Scene::new(committed_description, media.clone())
        .and_then(|scene| scene.with_producer_history(history.into()))
        .map_err(|error| format!("display checkpoint committed Scene: {error:?}"))?;
    t::ValidatedDisplayStart::construct(
        t::DisplayStartDescription {
            identity: Some(metadata.identity),
            reference: Some(metadata.reference),
            input_history: Some(metadata.input_history),
            continuation: Some(metadata.continuation),
            audio: Some(metadata.audio.clone()),
            display: Some(description),
            resident: Some(resident),
            hdma_content: Some(required.hdma_content.clone()),
            samus_content: Some(metadata.samus_content.clone()),
            committed_scene: Some(committed),
        },
        media,
        required,
    )
    .map_err(|error| format!("display checkpoint start validation: {error:?}"))
}

#[cfg(sm_verify_dll)]
#[path = "../verification/display_checkpoint_rejections.rs"]
mod display_checkpoint_rejections;

#[cfg(sm_verify_dll)]
impl super::component::DisplayCheckpoint {
    pub fn assert_decoder_rejections(
        bytes: &[u8],
        media: Arc<r::SceneMedia>,
        metadata: &DisplayStartMetadata,
        required: &t::DisplayStartRequirements,
    ) {
        display_checkpoint_rejections::assert_display_checkpoint_rejections(
            bytes, media, metadata, required,
        );
    }
}
