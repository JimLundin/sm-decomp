//! The engine's side of the span fold (RELOCK §8, step g).
//!
//! An engine run's per-frame outputs fold into an [`EngineRow`] through
//! `data::span::SpanFold`, the fold the oracle's recorder uses, so both rows' marks come
//! from one set of rules.
//!
//! The engine doesn't emit frames yet; [`EngineFrame`] is the shape it will hand over
//! per Tick: the cut its `Recorded` timing source gave, its video memory, its save RAM, the
//! PPU registers' digest from [`crate::ppu::digest`], its APU port accesses and
//! the shown picture. A stream the engine doesn't give at a frame where it folds is
//! absent from the row, which the comparison reports as `Absent(Side::Engine)`.

use crate::lock::{EngineRow, Stream};
use data::digest::Digest;
use data::span::{self, Cut, PortAccess, SpanFold};
use game::state::sram::SAVE_RAM_BYTES;
use game::state::video::{CGRAM_COLOURS, OAM_HIGH_BYTES, OamEntry, VideoMemory};

/// One engine frame, as the engine will emit it per Tick.
#[derive(Clone, Copy, Debug)]
pub struct EngineFrame<'a> {
    /// How the frame ended, from the timing source.
    pub cut: Cut,
    /// The PPU's memories after the frame.
    pub video: &'a VideoMemory,
    /// The cartridge's save RAM as its battery image, `None` when a run doesn't give it.
    pub sram: Option<&'a [u8; SAVE_RAM_BYTES]>,
    /// The `ppu` stream's digest, [`crate::ppu::digest`] of the State's video.
    pub ppu: Option<u64>,
    /// The CPU's APU port accesses in the frame, in order.
    pub ports: &'a [PortAccess],
    /// The shown picture's RGB bytes, `None` for a headless run.
    pub shown: Option<&'a [u8]>,
}

/// OAM's low table as the 256 little-endian words the PPU holds.
fn oam_words(low: &[OamEntry]) -> [u16; 256] {
    let mut words = [0; 256];
    for ([position, tile], entry) in words.as_chunks_mut::<2>().0.iter_mut().zip(low) {
        *position = u16::from_le_bytes([entry.x, entry.y]);
        *tile = u16::from_le_bytes([entry.tile, entry.attributes]);
    }
    words
}

impl span::Frame for EngineFrame<'_> {
    fn cut(&self) -> Cut {
        self.cut
    }

    fn memory(&self, stream: span::Stream) -> Option<u64> {
        match stream {
            span::Stream::Vram => Some(span::vram_digest(&self.video.vram)),
            span::Stream::Cgram => {
                let colours: [u16; CGRAM_COLOURS] = self.video.cgram.map(|colour| colour.0);
                Some(span::cgram_digest(&colours))
            }
            span::Stream::Oam => {
                let high: &[u8; OAM_HIGH_BYTES] = &self.video.oam.high;
                Some(span::oam_digest(&oam_words(&self.video.oam.low), high))
            }
            span::Stream::Sram => self.sram.map(|image| span::bytes_digest(image)),
            span::Stream::Ppu => self.ppu,
            // Work RAM isn't the engine's: the State replaces it (D5), and `wram` decides
            // nothing (D22).
            span::Stream::Wram | span::Stream::Apu | span::Stream::Video | span::Stream::Timing => {
                None
            }
        }
    }

    fn ports(&self) -> impl Iterator<Item = PortAccess> {
        self.ports.iter().copied()
    }

    fn video(&self) -> Option<u64> {
        self.shown.map(span::bytes_digest)
    }
}

impl From<span::Stream> for Stream {
    fn from(stream: span::Stream) -> Self {
        match stream {
            span::Stream::Vram => Self::Vram,
            span::Stream::Cgram => Self::Cgram,
            span::Stream::Oam => Self::Oam,
            span::Stream::Sram => Self::Sram,
            span::Stream::Ppu => Self::Ppu,
            span::Stream::Apu => Self::Apu,
            span::Stream::Video => Self::Video,
            span::Stream::Timing => Self::Timing,
            span::Stream::Wram => Self::Wram,
        }
    }
}

/// An engine run's fold, fed every frame in order.
#[derive(Clone, Debug, Default)]
pub struct EngineFold {
    spans: SpanFold,
}

impl EngineFold {
    /// Fold one frame.
    pub fn fold(&mut self, frame: &EngineFrame<'_>) {
        self.spans.fold(frame);
    }

    /// The frames folded so far.
    #[must_use]
    pub const fn frames(&self) -> u64 {
        self.spans.frames()
    }

    /// The run's marks as the comparison takes them, the partial last span closed; a
    /// stream some frame lacked is left out. Facts are evaluated elsewhere.
    #[must_use]
    pub fn finish(self) -> EngineRow {
        let streams = self
            .spans
            .finish()
            .into_iter()
            .filter_map(|(stream, marks)| {
                Some((
                    Stream::from(stream),
                    marks?.into_iter().map(Digest).collect(),
                ))
            })
            .collect();
        EngineRow {
            streams,
            facts: std::collections::BTreeMap::new(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use data::digest::fold_all;
    use data::span::Frame as _;
    use data::span::{MARK_FRAMES, PortDirection};
    use game::state::video::Colour;

    fn memory() -> VideoMemory {
        let mut video = VideoMemory::default();
        video
            .vram
            .iter_mut()
            .zip(0..)
            .for_each(|(word, value)| *word = value);
        video.cgram[3] = Colour(0x7fff);
        video.oam.low[1] = OamEntry {
            x: 1,
            y: 2,
            tile: 3,
            attributes: 4,
        };
        video.oam.high[0] = 0xaa;
        video
    }

    #[test]
    fn the_engine_memories_digest_as_the_records_bytes() {
        let video = memory();
        let frame = EngineFrame {
            cut: Cut::Requested,
            video: &video,
            sram: None,
            ppu: None,
            ports: &[],
            shown: None,
        };
        let mut low = [0_u16; 256];
        low[2] = 0x0201;
        low[3] = 0x0403;
        let mut high = [0_u8; 32];
        high[0] = 0xaa;
        let record_vram: Vec<u16> = (0..0x8000).collect();
        let mut record_cgram = [0_u16; 256];
        record_cgram[3] = 0x7fff;
        assert_eq!(
            frame.memory(span::Stream::Oam),
            Some(span::oam_digest(&low, &high))
        );
        assert_eq!(
            frame.memory(span::Stream::Vram),
            Some(span::vram_digest(&record_vram))
        );
        assert_eq!(
            frame.memory(span::Stream::Cgram),
            Some(span::cgram_digest(&record_cgram))
        );
    }

    #[test]
    fn an_engine_run_folds_to_an_engine_row() {
        let video = memory();
        let sram = [5_u8; SAVE_RAM_BYTES];
        let shown = [9_u8; 12];
        let ports = [PortAccess {
            direction: PortDirection::Write,
            port: 0,
            value: 1,
        }];
        let mut fold = EngineFold::default();
        let frames = MARK_FRAMES + 1;
        for index in 0..frames {
            let cut = if index % 2 == 0 {
                Cut::Requested
            } else {
                Cut::Overrun
            };
            fold.fold(&EngineFrame {
                cut,
                video: &video,
                sram: Some(&sram),
                ppu: Some(index),
                ports: &ports,
                shown: Some(&shown),
            });
        }
        assert_eq!(fold.frames(), frames);
        let row = fold.finish();
        let named: Vec<Stream> = row.streams.keys().copied().collect();
        // `wram` isn't the engine's, so it is absent and left out.
        assert_eq!(
            named,
            [
                Stream::Vram,
                Stream::Cgram,
                Stream::Oam,
                Stream::Sram,
                Stream::Ppu,
                Stream::Apu,
                Stream::Video,
                Stream::Timing
            ]
        );
        assert!(row.streams.values().all(|marks| marks.len() == 2));
        // The last, partial span is frame 1000 alone, a quiescent cut.
        assert_eq!(
            row.streams[&Stream::Ppu][1],
            Digest(fold_all([MARK_FRAMES]))
        );
        // Each quiescent cut's busy span holds its own access and the overrun frame's
        // before it; the first holds its own only.
        let busy = Digest::of(&[0, 0, 1, 0, 0, 1]);
        assert_eq!(row.streams[&Stream::Apu][1], Digest(fold_all([busy])));
    }
}
