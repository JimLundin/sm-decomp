//! The direct display selections and the NMI publications of the two cinematics.

use super::{explosion, zebes};
use crate::state::video::{
    BgMode, Brightness, Colour, ColourMath, ColourWrite, Layer, Layers, Mode7Select, Mosaic, Oam,
    Registers, Scroll, TileBases, TilemapBase, Video, Windows,
};
use types::zebes::FlightView;

/// Select the tile configuration before cinematic setup starts uploading art.
pub(super) fn select(registers: &mut Registers, view: FlightView) {
    let flat = view == FlightView::MosaicPage;
    registers.mode = BgMode(if flat { 1 } else { 7 });
    registers.mode7.select = Mode7Select(if flat { 0 } else { 0x80 });
    registers.tilemaps = [TilemapBase(if flat { 0x5c } else { 0 }), TilemapBase(0), TilemapBase(0), TilemapBase(0)];
    registers.tile_bases = [TileBases(if flat { 6 } else { 0 }), TileBases(0)];
    let mut main = Layers::default();
    main.insert(Layer::Objects);
    if view != FlightView::SpritesOnly { main.insert(Layer::Bg1); }
    registers.main_screen = main;
    registers.sub_screen = Layers::default();
}

pub(super) fn opening(video: &mut Video, view: FlightView) {
    select(&mut video.committed, view);
    select(&mut video.batch.registers, view);
}

fn light(registers: &mut Registers, shown: Option<u8>) {
    registers.forced_blank = shown.is_none();
    registers.brightness = Brightness(shown.unwrap_or(0));
}

fn affine(registers: &mut Registers, matrix: [i16;4], scroll: [i16;2], pivot: [i16;2]) {
    registers.mode7.matrix = matrix;
    registers.mode7.centre = pivot;
    if let Some(layer) = registers.scroll.first_mut() {
        *layer = Scroll { x: scroll[0].cast_unsigned(), y: scroll[1].cast_unsigned() };
    }
}

fn publish(video: &mut Video, palette: [Colour;256], oam: Oam) {
    video.batch.cgram = palette;
    video.batch.oam = oam;
    video.upload_oam_and_cgram();
    video.commit_registers();
}

pub(super) fn station(video: &mut Video, state: &explosion::State, out: &explosion::Output) {
    let Some((matrix, scroll, pivot)) = out.plane else { return };
    let regs = &mut video.batch.registers;
    light(regs, match state.scene.0.light {
        types::explosion::Light::Blanked => None,
        types::explosion::Light::Shown(level) => Some(level.get()),
    });
    affine(regs, [matrix.a,matrix.b,matrix.c,matrix.d],[scroll.x,scroll.y],[pivot.x,pivot.y]);
    publish(video, state.palette, state.oam);
}

pub(super) fn flight(video: &mut Video, state: &zebes::State, out: &zebes::Output) {
    let picture = &out.scene.picture;
    let regs = &mut video.batch.registers;
    select(regs, picture.view);
    light(regs, match picture.light {
        types::zebes::Brightness::Blanked => None,
        types::zebes::Brightness::Shown(level) => Some(level.get()),
    });
    let plane = picture.plane;
    let word = |value: i32| {
        let [low,high,..] = value.to_le_bytes();
        i16::from_le_bytes([low,high])
    };
    affine(regs, [plane.matrix.a,plane.matrix.b,plane.matrix.c,plane.matrix.d].map(word), [plane.scroll.x,plane.scroll.y], [plane.origin.x,plane.origin.y]);
    regs.mosaic = Mosaic(picture.mosaic.get().saturating_sub(1) << 4 | 1);
    if out.pages.is_some() {
        regs.windows = Windows::default();
        regs.colour_math = ColourMath::default();
        regs.sub_window_mask = Layers::default();
        video.batch.fixed_colour = [ColourWrite(0x20),ColourWrite(0x40),ColourWrite(0x80)];
        // Setup writes the affine registers directly, even as the flat page is selected.
        video.committed.mode7 = regs.mode7;
    }
    publish(video, state.palette, state.oam);
}
