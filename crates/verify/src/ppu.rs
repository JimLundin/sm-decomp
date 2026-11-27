use game::state::video::Registers;
use oracle_record::PpuRegisters;

/// Reconstruct committed video registers from the native seed.
pub fn registers(ppu: &PpuRegisters) -> Registers {
    use game::state::video::{
        BgMode, Brightness, ColourMath, ColourMathLayers, ColourMathSelect, DisplaySettings,
        FixedColour, Layers, Mode7, Mode7Select, Mosaic, ObjectSelect, Scroll, TileBases,
        TilemapBase, WindowSettings, Windows,
    };
    let [tm, ts, tmw, tsw] = ppu.screens;
    let layer = |bits| Layers {
        bits: u32::from(bits),
        ..Layers::default()
    };
    let [aa, ab, ba, bb, centre_x, centre_y] = ppu.m7;
    let [red, green, blue] = ppu.fixed_color;
    Registers {
        forced_blank: ppu.inidisp & 0x80 != 0,
        brightness: Brightness(ppu.inidisp & 15),
        objects: ObjectSelect(ppu.obsel),
        mode: BgMode(ppu.bgmode),
        mosaic: Mosaic(ppu.mosaic),
        tilemaps: ppu.bg_sc.map(TilemapBase),
        tile_bases: [TileBases(ppu.bg12nba), TileBases(ppu.bg34nba)],
        scroll: ppu.bg_scroll.map(|[x, y]| Scroll { x, y }),
        mode7: Mode7 {
            select: Mode7Select(ppu.m7sel),
            matrix: [aa, ab, ba, bb],
            centre: [centre_x, centre_y],
        },
        windows: Windows {
            select: ppu.window_select.map(WindowSettings),
            edges: ppu.window_positions,
            background_logic: WindowSettings(ppu.wbglog),
            object_logic: WindowSettings(ppu.wobjlog),
        },
        main_screen: layer(tm),
        sub_screen: layer(ts),
        main_window_mask: layer(tmw),
        sub_window_mask: layer(tsw),
        colour_math: ColourMath {
            select: ColourMathSelect(ppu.cgwsel),
            layers: ColourMathLayers(ppu.cgadsub),
            fixed: FixedColour { red, green, blue },
        },
        settings: DisplaySettings(ppu.setini),
    }
}
