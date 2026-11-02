//! Immediate finite hardware effects on the current Session display owner.
#![forbid(unsafe_code)]
use super::state::{DisplayError, DisplayState};

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum DmaField {
    Control,
    Target,
}

pub fn write_dma_channel(
    state: &mut DisplayState,
    channel: usize,
    field: DmaField,
    value: u8,
) -> Result<(), DisplayError> {
    let channel = state
        .hdma
        .channels
        .get_mut(channel)
        .ok_or(DisplayError::Geometry)?;
    match field {
        DmaField::Control => {
            channel.mode = value & 7;
            channel.fixed = value & 8 != 0;
            channel.decrement = value & 0x10 != 0;
            channel.unused_bit = value & 0x20 != 0;
            channel.indirect = value & 0x40 != 0;
            channel.from_destination = value & 0x80 != 0;
        }
        DmaField::Target => channel.destination = value,
    }
    Ok(())
}

/// MPYM then MPYH: each physical byte read updates PPU1's open-bus latch.
pub fn read_product_word(state: &mut DisplayState) -> u16 {
    let product = sm_data::hardware::affine_product(&state.effective.affine.matrix);
    state.latches.ppu_open_bus[0] = product[1];
    let low = state.latches.ppu_open_bus[0];
    state.latches.ppu_open_bus[0] = product[2];
    u16::from_le_bytes([low, state.latches.ppu_open_bus[0]])
}

pub fn write_affine(state: &mut DisplayState, index: usize, value: u8) {
    state
        .effective
        .affine
        .write(&mut state.latches.affine_previous, index, value);
}

pub fn write_scroll(state: &mut DisplayState, index: usize, value: u8) {
    sm_data::table::check_index(index, 8);
    if index < 2 {
        write_affine(state, 6 + index, value);
    }
    let high = u16::from(value) << 8;
    let low = if index & 1 == 0 {
        let low =
            (state.latches.scroll_previous & 0xf8) | (state.latches.scroll_previous_second & 7);
        state.latches.scroll_previous_second = value;
        low
    } else {
        state.latches.scroll_previous
    };
    state.effective.scroll[index] = (high | u16::from(low)) & 0x3ff;
    state.latches.scroll_previous = value;
}

/// The original control upload's scroll and affine tail, on this same owner.
pub fn refresh_latches(state: &mut DisplayState) {
    state.effective.mode_byte = state.controls.mode;
    state
        .effective
        .affine
        .select(state.controls.affine_selection);
    state.effective.window_selection = state.controls.window_selection;
    let controls = state.controls;
    for (address, value) in [
        (0x00, controls.brightness),
        (0x01, controls.object_layout),
        (0x06, controls.mosaic),
        (0x07, controls.screens[0]),
        (0x08, controls.screens[1]),
        (0x0a, controls.screens[3]),
        (0x0b, controls.characters[0]),
        (0x0c, controls.characters[1]),
        (0x2a, controls.window_logic[0]),
        (0x2b, controls.window_logic[1]),
        (0x2d, controls.sub_layers),
        (0x2e, controls.main_window),
        (0x2f, controls.sub_window),
        (0x32, controls.fixed_color[0]),
        (0x32, controls.fixed_color[1]),
        (0x32, controls.fixed_color[2]),
        (0x33, controls.geometry),
    ] {
        write_raster(state, address, value);
    }
    for (index, value) in controls.window_positions.into_iter().enumerate() {
        write_raster(
            state,
            0x26 + u8::try_from(index).expect("four window bytes"),
            value,
        );
    }
    for (index, value) in controls.window_selection.into_iter().enumerate() {
        write_raster(
            state,
            0x23 + u8::try_from(index).expect("three selector bytes"),
            value,
        );
    }
    for index in 0..8 {
        let low = state.controls.scroll_words[index].to_le_bytes()[0];
        write_scroll(state, index, low);
        let high = state.controls.scroll_words[index].to_le_bytes()[1];
        write_scroll(state, index, high);
    }
    state.controls.committed_fake_mode = state.controls.fake_mode;
    if state.controls.mode & 7 == 7 || state.controls.fake_mode & 7 == 7 {
        for index in 0..6 {
            let low = state.controls.affine_words[index].to_le_bytes()[0];
            write_affine(state, index, low);
            let high = state.controls.affine_words[index].to_le_bytes()[1];
            write_affine(state, index, high);
        }
    }
}

/// The second M7A byte is the original native byte one, not a shifted result.
pub fn sine_times(state: &mut DisplayState, length: u16, angle: u16) -> u16 {
    let value = crate::projectile_geometry::SINE_COSINE[usize::from(angle & 0xff) + 64];
    let bytes = value.to_ne_bytes();
    write_affine(state, 0, value.to_le_bytes()[0]);
    write_affine(state, 0, bytes[1]);
    write_affine(state, 1, length.to_le_bytes()[0]);
    read_product_word(state)
}

/// One physical device image, explicitly selected outside the owned interval.
/// It carries effective state independently of staged gameplay control words.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct DeviceImage {
    pub arithmetic: sm_data::hardware::Arithmetic,
    pub effective: super::state::EffectiveDisplayDescription,
    pub affine: sm_data::hardware::Affine,
    pub affine_previous: u8,
    pub ppu_open_bus: [u8; 2],
    pub mode_byte: u8,
    pub scroll: [u16; 8],
    pub window_selection: [u8; 3],
    pub scroll_previous: u8,
    pub scroll_previous_second: u8,
}
pub fn export(state: &DisplayState) -> DeviceImage {
    DeviceImage {
        arithmetic: state.arithmetic,
        effective: state.effective,
        affine: state.effective.affine,
        affine_previous: state.latches.affine_previous,
        ppu_open_bus: state.latches.ppu_open_bus,
        mode_byte: state.effective.mode_byte,
        scroll: state.effective.scroll,
        window_selection: state.effective.window_selection,
        scroll_previous: state.latches.scroll_previous,
        scroll_previous_second: state.latches.scroll_previous_second,
    }
}
/// Adopt the selected normalized image after an external writer, including failure.
pub fn import(state: &mut DisplayState, image: DeviceImage) {
    state.arithmetic = image.arithmetic;
    state.effective = image.effective;
    state.effective.affine = image.affine;
    state.latches.affine_previous = image.affine_previous;
    state.latches.ppu_open_bus = image.ppu_open_bus;
    state.effective.mode_byte = image.mode_byte;
    state.effective.scroll = image.scroll;
    state.effective.window_selection = image.window_selection;
    state.latches.scroll_previous = image.scroll_previous;
    state.latches.scroll_previous_second = image.scroll_previous_second;
}

/// Extract this visible row from the current owner. The admitted declaration
/// retains actual atlas/map/window-extent/OBJSEL/EXTBG/color/mosaic inputs; none
/// of those identities or missing native fields is inferred from a numeric word.
pub fn sample_line(
    state: &DisplayState,
    mut line: super::records::SceneLine,
) -> Result<super::records::SceneLine, DisplayError> {
    use super::records::{SceneAffine, SceneMode, ScreenPosition};
    match (state.effective.mode_byte & 7, line.mode) {
        (1, SceneMode::TilePlanes) | (7, SceneMode::AffinePlane { .. }) => {}
        _ => return Err(DisplayError::Scene),
    }
    let effective = &state.effective;
    let inputs = &effective.raster_inputs;
    let selected = u32::from_le_bytes([
        effective.window_selection[0],
        effective.window_selection[1],
        effective.window_selection[2],
        0,
    ]);
    if inputs.window_native_flags & 0x00ff_ffff != selected
        || inputs.pseudo_hires
        || effective.direct_color
        || effective.mode_byte & 0xf0 != 0
    {
        return Err(DisplayError::Scene);
    }
    line.blank = effective.forced_blank;
    if let SceneMode::AffinePlane { ref mut extended } = line.mode {
        *extended = inputs.extended_background;
    }
    // Current hardware inputs, rather than a caller's future-row template.
    line.object_selection.size_set = inputs.object_size;
    line.object_selection.first_order = inputs
        .object_priority
        .then(|| ((state.latches.object_address & 0xff) >> 1).to_le_bytes()[0]);
    line.object_selection.interlaced = inputs.object_interlaced;
    line.pixelation.enabled = core::array::from_fn(|index| inputs.mosaic_enabled[index]);
    line.pixelation.size = inputs.mosaic_size;
    line.pixelation.origin = state.raster.mosaic_origin;
    line.pixelation.modulo = inputs.mosaic_modulo;
    let condition = |value| match value {
        0 => Ok(super::records::WindowCondition::Never),
        1 => Ok(super::records::WindowCondition::Outside),
        2 => Ok(super::records::WindowCondition::Inside),
        3 => Ok(super::records::WindowCondition::Always),
        _ => Err(DisplayError::Geometry),
    };
    line.color = super::records::SceneColor {
        clip: condition(effective.color_clip)?,
        prevent_math: condition(effective.color_prevent_math)?,
        enabled: effective.color_math_enabled,
        use_subscreen: effective.color_add_subscreen,
        subtract: effective.color_subtract,
        half: effective.color_half,
        fixed: inputs.fixed_color,
        brightness: inputs.brightness,
    };
    for (index, routing) in line.routing.iter_mut().enumerate() {
        let enabled = if index == 0 {
            effective.main_layers
        } else {
            inputs.sub_layers
        };
        routing.visible = core::array::from_fn(|layer| enabled & (1 << layer) != 0);
        routing.masked =
            core::array::from_fn(|layer| inputs.screen_windowed[index] & (1 << layer) != 0);
    }
    for (index, extent) in line.windows.extents.iter_mut().enumerate() {
        extent.first = inputs.window_positions[index * 2];
        extent.last = inputs.window_positions[index * 2 + 1];
    }
    for (index, plane) in line.planes.iter_mut().enumerate() {
        plane.scroll = ScreenPosition {
            x: i16::from_le_bytes(state.effective.scroll[index * 2].to_le_bytes()),
            y: i16::from_le_bytes(state.effective.scroll[index * 2 + 1].to_le_bytes()),
        };
    }
    let affine = state.effective.affine;
    line.affine = SceneAffine {
        matrix: [
            affine.matrix[0],
            affine.matrix[1],
            affine.matrix[2],
            affine.matrix[3],
        ],
        center: ScreenPosition {
            x: affine.matrix[4],
            y: affine.matrix[5],
        },
        scroll: ScreenPosition {
            x: affine.matrix[6],
            y: affine.matrix[7],
        },
        flip_horizontal: affine.horizontal_flip,
        flip_vertical: affine.vertical_flip,
        bounded_field: affine.large_field,
        fill_outside: affine.character_fill,
    };
    for (index, window) in line.windows.layers.iter_mut().enumerate() {
        let flags = state.effective.window_selection[index / 2] >> ((index & 1) * 4);
        window.first_inverted = flags & 1 != 0;
        window.first_enabled = flags & 2 != 0;
        window.second_inverted = flags & 4 != 0;
        window.second_enabled = flags & 8 != 0;
        window.combination = match inputs.window_logic[index] {
            0 => super::records::WindowCombination::Union,
            1 => super::records::WindowCombination::Intersection,
            2 => super::records::WindowCombination::Exclusive,
            3 => super::records::WindowCombination::Equal,
            _ => return Err(DisplayError::Geometry),
        };
    }
    line.object_selection.even_frame = state.raster.even_frame;
    Ok(line)
}

/// Deliver an actual register byte to the current raster owner. Returns false
/// only for a destination owned by a different capability; no unknown write is ignored.
pub fn write_raster(state: &mut DisplayState, address: u8, value: u8) -> bool {
    match address {
        0x00 => {
            state.effective.forced_blank = value & 0x80 != 0;
            state.effective.raster_inputs.brightness = value & 15;
        }
        0x01 => {
            let first = u16::from(value & 7) << 13;
            state.effective.raster_inputs.object_tile_words = [
                first,
                first.wrapping_add(u16::from((value & 0x18) + 8) << 9),
            ];
            state.effective.raster_inputs.object_size = value >> 5;
        }
        0x05 => state.effective.mode_byte = value,
        0x06 => {
            let inputs = &mut state.effective.raster_inputs;
            for (index, enabled) in inputs.mosaic_enabled.iter_mut().enumerate() {
                *enabled = value & (1 << index) != 0;
            }
            inputs.mosaic_size = (value >> 4) + 1;
            state.raster.mosaic_origin = 0;
        }
        0x07 | 0x08 | 0x0a => {
            let index = match address {
                0x07 => 0,
                0x08 => 1,
                _ => 2,
            };
            let inputs = &mut state.effective.raster_inputs;
            inputs.map_words[index] = u16::from(value & 0xfc) << 8;
            inputs.map_wider[index] = value & 1 != 0;
            inputs.map_higher[index] = value & 2 != 0;
        }
        0x09 => super::irq::deliver_output(state, super::irq::Output::Background3Map, value),
        0x0b | 0x0c => {
            let first = usize::from(address - 0x0b) * 2;
            state.effective.raster_inputs.tile_words[first] = u16::from(value & 15) << 12;
            state.effective.raster_inputs.tile_words[first + 1] = u16::from(value & 0xf0) << 8;
        }
        0x0d..=0x14 => write_scroll(state, usize::from(address - 0x0d), value),
        0x1a => state.effective.affine.select(value),
        0x1b..=0x20 => write_affine(state, usize::from(address - 0x1b), value),
        0x23..=0x25 => {
            state.effective.window_selection[usize::from(address - 0x23)] = value;
            let shift = u32::from(address - 0x23) * 8;
            let flags = &mut state.effective.raster_inputs.window_native_flags;
            *flags = (*flags & !(0xff << shift)) | (u32::from(value) << shift);
        }
        0x26..=0x29 => {
            state.effective.raster_inputs.window_positions[usize::from(address - 0x26)] = value
        }
        0x2a | 0x2b => {
            let first = usize::from(address - 0x2a) * 4;
            let count = if address == 0x2a { 4 } else { 2 };
            for (index, logic) in state.effective.raster_inputs.window_logic[first..first + count]
                .iter_mut()
                .enumerate()
            {
                *logic = (value >> (index * 2)) & 3;
            }
        }
        0x2c => super::irq::deliver_output(state, super::irq::Output::MainLayers, value),
        0x2d => state.effective.raster_inputs.sub_layers = value,
        0x2e | 0x2f => {
            state.effective.raster_inputs.screen_windowed[usize::from(address - 0x2e)] = value
        }
        0x30 => super::irq::deliver_output(state, super::irq::Output::ColorWindow, value),
        0x31 => super::irq::deliver_output(state, super::irq::Output::ColorArithmetic, value),
        0x32 => {
            for (index, channel) in state
                .effective
                .raster_inputs
                .fixed_color
                .iter_mut()
                .enumerate()
            {
                if value & (0x20 << index) != 0 {
                    *channel = value & 31;
                }
            }
        }
        0x33 => {
            state.raster.interlaced = value & 1 != 0;
            state.raster.overscan = value & 4 != 0;
            let inputs = &mut state.effective.raster_inputs;
            inputs.object_interlaced = value & 2 != 0;
            inputs.pseudo_hires = value & 8 != 0;
            inputs.extended_background = value & 0x40 != 0;
        }
        _ => return false,
    }
    true
}

/// Physical video values copied under an exclusive external-writer interval.
/// Authored identities are admitted separately; bytes never manufacture assets.
#[derive(Clone, Debug, PartialEq)]
pub struct PhysicalVideoImage {
    pub effective: super::state::EffectiveDisplayDescription,
    pub latches: super::state::DisplayLatches,
    pub raster: PhysicalRaster,
    pub words: std::sync::Arc<[u16]>,
    pub colors: [u16; 256],
    pub objects: [u8; 544],
    pub caches: PhysicalRenderCaches,
}
/// Native caches are capture data, never a second engine authority. Import keeps
/// the actual values, including a partly completed raster call.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct PhysicalRenderCaches {
    pub affine_start: [i32; 2],
    pub object_pixels: [u8; 256],
    pub object_priorities: [u8; 256],
    pub backgrounds: [[u16; 256]; 2],
    pub objects: [u16; 256],
    pub brightness_last: u8,
    pub brightness_normal: [u8; 63],
    pub brightness_half: [u8; 64],
    pub extra_left: u8,
    pub extra_right: u8,
    pub extra_left_right: u8,
}
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct PhysicalRaster {
    pub range_overflow: bool,
    pub time_overflow: bool,
    pub has_objects: bool,
    pub even_frame: bool,
    pub interlaced: bool,
    pub frame_interlaced: bool,
    pub overscan: bool,
    pub frame_overscan: bool,
    pub mosaic_origin: u8,
}
impl PhysicalVideoImage {
    /// Opening validates this physical image against the independently admitted
    /// media and producer definitions through the existing resident contract.
    pub fn resident(
        &self,
        image: super::records::MediaRef,
        palette: super::records::MediaRef,
        produced: std::sync::Arc<[super::records::ProducedObject]>,
    ) -> super::commit::ResidentDisplayDescription {
        super::commit::ResidentDisplayDescription {
            image,
            palette,
            words: std::sync::Arc::clone(&self.words),
            colors: self.colors,
            objects: self.objects,
            produced,
        }
    }
    /// Adopt physical fields, preserving the owned scheduler's phase/visibility.
    /// Those two fields have no corresponding physical native state.
    pub fn adopt_state(&self, state: &mut DisplayState) {
        state.effective = self.effective;
        state.latches = self.latches;
        let source = self.raster;
        let target = &mut state.raster;
        target.range_overflow = source.range_overflow;
        target.time_overflow = source.time_overflow;
        target.has_objects = source.has_objects;
        target.even_frame = source.even_frame;
        target.interlaced = source.interlaced;
        target.frame_interlaced = source.frame_interlaced;
        target.overscan = source.overscan;
        target.frame_overscan = source.frame_overscan;
        target.mosaic_origin = source.mosaic_origin;
    }
}
#[derive(Clone, Debug)]
pub struct PhysicalRasterRow {
    pub row: u16,
    pub before: PhysicalVideoImage,
    pub after: Option<PhysicalRaster>,
    pub after_caches: Option<PhysicalRenderCaches>,
}
#[derive(Clone, Debug)]
pub struct PhysicalRasterHistory {
    pub rows: Vec<PhysicalRasterRow>,
    pub terminal: PhysicalVideoImage,
}
