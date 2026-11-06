//! The `ppu` stream's encoder: the engine's committed registers and DMA channels as the
//! recorder's register struct, so the fold digests them with the recorder's own rule
//! ([`crate::cache::ppu_digest`]).
//!
//! It lives in `verify`, not `data`: the struct and its digest rule are the oracle record's
//! layout, which the engine never needs (D6: translation to the oracle's form lives in
//! verification).

use game::state::video::{DmaChannel, Registers, Video};
use oracle_record::PpuRegisters;

use crate::cache::ppu_digest;

/// The record's register struct for `video`'s committed registers and DMA channels.
///
/// The registers the recorder's rule leaves out (`nmitimen`, `htime`, `vtime`, `hdmaen`,
/// `memsel`) are 0: the State has no value for them and the digest doesn't read them.
#[must_use]
pub fn encode(video: &Video) -> PpuRegisters {
    let Registers {
        forced_blank,
        brightness,
        objects,
        mode,
        mosaic,
        tilemaps,
        tile_bases: [bg12, bg34],
        scroll,
        mode7,
        windows,
        main_screen,
        sub_screen,
        main_window_mask,
        sub_window_mask,
        colour_math,
        settings,
    } = video.committed;
    let [m7a, m7b, m7c, m7d] = mode7.matrix;
    let [m7x, m7y] = mode7.centre;
    let screen = |layers: game::state::video::Layers| u8::try_from(layers.bits).unwrap_or(u8::MAX);
    let fixed = colour_math.fixed;
    PpuRegisters {
        inidisp: (u8::from(forced_blank) << 7) | brightness.0,
        obsel: objects.0,
        bgmode: mode.0,
        mosaic: mosaic.0,
        bg_sc: tilemaps.map(|tilemap| tilemap.0),
        bg12nba: bg12.0,
        bg34nba: bg34.0,
        bg_scroll: scroll.map(|layer| [layer.x, layer.y]),
        m7sel: mode7.select.0,
        m7: [m7a, m7b, m7c, m7d, m7x, m7y],
        window_select: windows.select.map(|settings| settings.0),
        window_positions: windows.edges,
        wbglog: windows.background_logic.0,
        wobjlog: windows.object_logic.0,
        screens: [
            screen(main_screen),
            screen(sub_screen),
            screen(main_window_mask),
            screen(sub_window_mask),
        ],
        cgwsel: colour_math.select.0,
        cgadsub: colour_math.layers.0,
        fixed_color: [fixed.red, fixed.green, fixed.blue],
        setini: settings.0,
        nmitimen: 0,
        htime: 0,
        vtime: 0,
        hdmaen: 0,
        memsel: 0,
        dma: video.dma.map(channel_bytes),
    }
}

/// A DMA channel as its eleven registers, `$43x0` to `$43xA`.
const fn channel_bytes(channel: DmaChannel) -> [u8; 11] {
    let [source_low, source_high] = channel.source.to_le_bytes();
    let [count_low, count_high] = channel.count.to_le_bytes();
    let [table_low, table_high] = channel.table.to_le_bytes();
    [
        channel.control.to_byte(),
        channel.destination.to_byte(),
        source_low,
        source_high,
        channel.source_bank,
        count_low,
        count_high,
        channel.indirect_bank,
        table_low,
        table_high,
        channel.line_counter,
    ]
}

/// The `ppu` stream's digest of `video`, for [`crate::fold::EngineFrame::ppu`].
#[must_use]
pub fn digest(video: &Video) -> u64 {
    ppu_digest(&encode(video))
}

#[cfg(test)]
mod tests {
    use game::state::video::{Brightness, DmaChannel, DmaControl, Scroll, Video};

    use super::{digest, encode};
    use crate::cache::ppu_digest;

    #[test]
    fn the_engine_registers_encode_as_the_records_struct() {
        let mut video = Video::default();
        video.committed.forced_blank = true;
        video.committed.brightness = Brightness(15);
        video.committed.scroll[1] = Scroll { x: 0x2C0, y: 0xE0 };
        video.dma[7] = DmaChannel {
            control: DmaControl::from_byte(0x43),
            source: 0xD86F,
            table: 0xD8D1,
            ..DmaChannel::default()
        };
        let ppu = encode(&video);
        assert_eq!(ppu.inidisp, 0x8F);
        assert_eq!(ppu.bg_scroll[1], [0x2C0, 0xE0]);
        assert_eq!(ppu.dma[7], [0x43, 0, 0x6F, 0xD8, 0, 0, 0, 0, 0xD1, 0xD8, 0]);
        assert_eq!(digest(&video), ppu_digest(&ppu));
    }
}
