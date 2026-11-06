//! The span fold of the format-2 lock (RELOCK §2, §6).
//!
//! Which streams fold at which cuts, how the APU port log is joined over a busy span,
//! and where a span's mark closes. The oracle's recorder and the engine's verifier both
//! fold through [`SpanFold`], so the engine's marks are computed by exactly the
//! recorder's rules.
//!
//! A cut is quiescent when it is the return of a requested NMI ([`Cut::Requested`]):
//! the NMI found the main strand waiting. The memory streams, `ppu`, `wram` and `apu`
//! fold at quiescent cuts only; `video` and `timing` fold every frame. Each stream's
//! mark is the fold of its digests over [`MARK_FRAMES`] frames from the empty digest,
//! the last span partial.

use crate::digest::{Digest, fold};
use core::hash::{Hash, Hasher};

/// Video frames per span mark.
pub const MARK_FRAMES: u64 = 1000;

/// A machine stream of the format-2 lock.
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum Stream {
    /// Video RAM.
    Vram,
    /// Palette RAM.
    Cgram,
    /// Sprite attribute memory.
    Oam,
    /// Cartridge save RAM.
    Sram,
    /// The architectural PPU registers both sides can see.
    Ppu,
    /// The CPU's APU port log over a busy span.
    Apu,
    /// The picture as shown.
    Video,
    /// The cut kind, NMI kind and overrun flag.
    Timing,
    /// Work RAM.
    Wram,
}

/// When a stream folds a frame.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Folding {
    /// At quiescent cuts only.
    Quiescent,
    /// At every frame's cut.
    EveryFrame,
}

impl Stream {
    /// Every stream, in the order an entry lists them.
    pub const ALL: [Self; 9] = [
        Self::Vram,
        Self::Cgram,
        Self::Oam,
        Self::Sram,
        Self::Ppu,
        Self::Apu,
        Self::Video,
        Self::Timing,
        Self::Wram,
    ];

    /// The name an entry's `streams.NAME` key uses.
    #[must_use]
    pub const fn name(self) -> &'static str {
        match self {
            Self::Vram => "vram",
            Self::Cgram => "cgram",
            Self::Oam => "oam",
            Self::Sram => "sram",
            Self::Ppu => "ppu",
            Self::Apu => "apu",
            Self::Video => "video",
            Self::Timing => "timing",
            Self::Wram => "wram",
        }
    }

    /// When the stream folds.
    #[must_use]
    pub const fn folding(self) -> Folding {
        match self {
            Self::Vram
            | Self::Cgram
            | Self::Oam
            | Self::Sram
            | Self::Ppu
            | Self::Apu
            | Self::Wram => Folding::Quiescent,
            Self::Video | Self::Timing => Folding::EveryFrame,
        }
    }

    /// Whether the stream folds at a cut that is `quiescent` or not.
    #[must_use]
    pub fn folds_at(self, quiescent: bool) -> bool {
        quiescent || self.folding() == Folding::EveryFrame
    }
}

/// How a frame's interval ended, as the fold sees it.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Cut {
    /// The return of an NMI that found the main strand waiting: quiescent.
    Requested,
    /// The return of an NMI that found the main strand mid-frame: a lag frame.
    Overrun,
    /// Vblank began with NMI disabled.
    VblankWithoutNmi,
    /// Vblank ended with the NMI handler still running.
    NmiStillRunning,
}

impl Cut {
    /// Whether the memory streams fold at this cut.
    #[must_use]
    pub const fn quiescent(self) -> bool {
        matches!(self, Self::Requested)
    }

    /// The `timing` stream's frame: the cut kind (0 an NMI's return, 1 vblank without
    /// NMI, 2 still running), the NMI kind (0 none, 1 requested, 2 overrun) and the
    /// overrun flag.
    #[must_use]
    pub const fn timing(self) -> [u8; 3] {
        match self {
            Self::Requested => [0, 1, 0],
            Self::Overrun => [0, 2, 1],
            Self::VblankWithoutNmi => [1, 0, 0],
            Self::NmiStillRunning => [2, 0, 0],
        }
    }
}

/// The direction of a CPU access to an APU port.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum PortDirection {
    /// The CPU wrote the port.
    Write,
    /// The CPU read the port.
    Read,
}

/// One CPU access to `$2140-$2143`. DMA accesses aren't in the stream: lsnes's bus hook
/// can't see them, so neither side folds them.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct PortAccess {
    /// Read or write.
    pub direction: PortDirection,
    /// The port, 0-3.
    pub port: u8,
    /// The byte read or written.
    pub value: u8,
}

impl PortAccess {
    /// The access as the `apu` stream's bytes: `(kind, port, value)`, kind 0 a write.
    #[must_use]
    pub const fn bytes(self) -> [u8; 3] {
        let kind = match self.direction {
            PortDirection::Write => 0,
            PortDirection::Read => 1,
        };
        [kind, self.port, self.value]
    }
}

/// The digest of a value through `Hash`, as the memory streams hash their arrays.
#[must_use]
pub fn hashed(value: &impl Hash) -> u64 {
    let mut digest = Digest::default();
    value.hash(&mut digest);
    digest.finish()
}

/// The `vram` stream's digest: the 32K words as Rust hashes a word slice.
#[must_use]
pub fn vram_digest(words: &[u16]) -> u64 {
    hashed(&words)
}

/// The `cgram` stream's digest: the 256 colours as Rust hashes a word slice.
#[must_use]
pub fn cgram_digest(colours: &[u16]) -> u64 {
    hashed(&colours)
}

/// The `oam` stream's digest: the low table's words, then the high table.
#[must_use]
pub fn oam_digest(low: &[u16; 256], high: &[u8; 32]) -> u64 {
    let mut digest = Digest::default();
    low.hash(&mut digest);
    high.hash(&mut digest);
    digest.finish()
}

/// The digest of a byte stream: `sram`, `wram` and `video`'s shown picture.
#[must_use]
pub fn bytes_digest(bytes: &[u8]) -> u64 {
    Digest::of(bytes)
}

/// One frame as a side gives it to [`SpanFold::fold`]. Only the streams that fold at
/// the frame's cut are asked for, so a side digests its memories at quiescent cuts only.
pub trait Frame {
    /// How the frame's interval ended.
    fn cut(&self) -> Cut;
    /// A memory stream's digest (`vram`, `cgram`, `oam`, `sram`, `ppu`, `wram`), or
    /// `None` where this frame lacks it.
    fn memory(&self, stream: Stream) -> Option<u64>;
    /// The CPU's port accesses in the frame's interval, in order.
    fn ports(&self) -> impl Iterator<Item = PortAccess>;
    /// The shown picture's digest ([`bytes_digest`] of its RGB bytes), or `None` where
    /// this frame lacks it.
    fn video(&self) -> Option<u64>;
}

/// What a stream did at one frame.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Folded {
    /// The stream doesn't fold at this cut.
    Skipped,
    /// The frame lacked the stream; it is absent from the row.
    Absent,
    /// The digest folded.
    Digest(u64),
}

/// One stream's marks.
#[derive(Clone, Debug)]
pub struct Marks {
    current: u64,
    closed: Vec<u64>,
    absent: bool,
}

impl Default for Marks {
    fn default() -> Self {
        Self {
            current: Digest::default().finish(),
            closed: Vec::new(),
            absent: false,
        }
    }
}

impl Marks {
    /// Fold one cut's digest into the open span; `None` makes the stream absent.
    pub fn fold(&mut self, digest: Option<u64>) {
        match digest {
            Some(digest) => self.current = fold(self.current, digest),
            None => self.absent = true,
        }
    }

    /// Close the open span as a mark and open the next from the empty digest.
    pub fn close(&mut self) {
        self.closed.push(core::mem::replace(
            &mut self.current,
            Digest::default().finish(),
        ));
    }

    /// The closed marks, `None` where some frame lacked the stream.
    #[must_use]
    pub fn marks(&self) -> Option<&[u64]> {
        (!self.absent).then_some(&self.closed)
    }
}

/// One frame's fold: whether its cut was quiescent, whether it closed a span, and what
/// each stream (in [`Stream::ALL`] order) folded.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct FrameFold {
    /// The cut was quiescent.
    pub quiescent: bool,
    /// The frame was a span's last.
    pub closed: bool,
    /// Each stream's fold.
    pub streams: [Folded; 9],
}

/// One row's span fold, fed every frame in order.
#[derive(Clone, Debug, Default)]
pub struct SpanFold {
    frames: u64,
    spans: [Marks; 9],
    apu_busy: Vec<u8>,
}

impl SpanFold {
    /// Fold one frame. The port accesses join the busy span's log; at a quiescent cut
    /// `apu` folds the log since the previous quiescent cut, this frame's included, and
    /// starts a new one. Accesses after the row's last quiescent cut never fold.
    pub fn fold(&mut self, frame: &impl Frame) -> FrameFold {
        self.apu_busy
            .extend(frame.ports().flat_map(PortAccess::bytes));
        let cut = frame.cut();
        let apu = Digest::of(&self.apu_busy);
        let folded = self.fold_digests(cut, |stream| match stream {
            Stream::Apu => Some(apu),
            Stream::Video => frame.video(),
            Stream::Timing => Some(Digest::of(&cut.timing())),
            Stream::Vram
            | Stream::Cgram
            | Stream::Oam
            | Stream::Sram
            | Stream::Ppu
            | Stream::Wram => frame.memory(stream),
        });
        if folded.quiescent {
            self.apu_busy.clear();
        }
        folded
    }

    /// Fold one frame whose stream digests are already known: `digest` is asked only
    /// for the streams that fold at `cut`, and for `apu` must give the busy span's log
    /// digest. [`SpanFold::fold`] is this with the digests computed; this entry point
    /// re-folds a recorder's per-frame sidecar.
    pub fn fold_digests(
        &mut self,
        cut: Cut,
        mut digest: impl FnMut(Stream) -> Option<u64>,
    ) -> FrameFold {
        let quiescent = cut.quiescent();
        let mut streams = [Folded::Skipped; 9];
        for ((stream, spans), folded) in Stream::ALL.iter().zip(&mut self.spans).zip(&mut streams) {
            if stream.folds_at(quiescent) {
                let value = digest(*stream);
                spans.fold(value);
                *folded = value.map_or(Folded::Absent, Folded::Digest);
            }
        }
        self.frames = self.frames.saturating_add(1);
        let closed = self.frames.is_multiple_of(MARK_FRAMES);
        if closed {
            self.spans.iter_mut().for_each(Marks::close);
        }
        FrameFold {
            quiescent,
            closed,
            streams,
        }
    }

    /// The frames folded so far.
    #[must_use]
    pub const fn frames(&self) -> u64 {
        self.frames
    }

    /// Whether the row's last span is partial and still open.
    #[must_use]
    pub const fn partial(&self) -> bool {
        !self.frames.is_multiple_of(MARK_FRAMES)
    }

    /// The row's marks per stream in [`Stream::ALL`] order, the partial last span
    /// closed; `None` for a stream some frame lacked.
    #[must_use]
    pub fn finish(mut self) -> [(Stream, Option<Vec<u64>>); 9] {
        if self.partial() {
            self.spans.iter_mut().for_each(Marks::close);
        }
        let mut marks = self
            .spans
            .iter()
            .map(|spans| spans.marks().map(<[u64]>::to_vec));
        Stream::ALL.map(|stream| (stream, marks.next().flatten()))
    }
}

#[cfg(test)]
mod tests;
