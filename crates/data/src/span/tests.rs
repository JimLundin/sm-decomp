use super::*;
use crate::digest::fold_all;

/// A synthetic frame: every memory's digest is its frame number and stream position.
struct Synthetic {
    index: u64,
    cut: Cut,
    ports: Vec<PortAccess>,
    video: bool,
    sram: bool,
}

impl Synthetic {
    fn new(index: u64, cut: Cut) -> Self {
        Self {
            index,
            cut,
            ports: Vec::new(),
            video: true,
            sram: true,
        }
    }

    fn memory_digest(index: u64, stream: Stream) -> u64 {
        let position = Stream::ALL.iter().position(|each| *each == stream).unwrap();
        (index << 8) | u64::try_from(position).unwrap()
    }
}

impl Frame for Synthetic {
    fn cut(&self) -> Cut {
        self.cut
    }
    fn memory(&self, stream: Stream) -> Option<u64> {
        (stream != Stream::Sram || self.sram).then(|| Self::memory_digest(self.index, stream))
    }
    fn ports(&self) -> impl Iterator<Item = PortAccess> {
        self.ports.iter().copied()
    }
    fn video(&self) -> Option<u64> {
        self.video.then_some(self.index ^ 0x5a5a)
    }
}

fn write(port: u8, value: u8) -> PortAccess {
    PortAccess {
        direction: PortDirection::Write,
        port,
        value,
    }
}

fn marks_of(fold: SpanFold, stream: Stream) -> Option<Vec<u64>> {
    fold.finish()
        .into_iter()
        .find(|(name, _)| *name == stream)
        .and_then(|(_, marks)| marks)
}

#[test]
fn spans_close_every_mark_frames_and_the_last_is_partial() {
    let mut fold = SpanFold::default();
    let frames = MARK_FRAMES * 2 + 5;
    let closed: Vec<u64> = (0..frames)
        .filter(|&index| fold.fold(&Synthetic::new(index, Cut::Requested)).closed)
        .collect();
    assert_eq!(closed, [MARK_FRAMES - 1, MARK_FRAMES * 2 - 1]);
    assert!(fold.partial());
    let video = marks_of(fold, Stream::Video).unwrap();
    let span = |range: core::ops::Range<u64>| fold_all(range.map(|index| index ^ 0x5a5a));
    assert_eq!(
        video,
        [
            span(0..MARK_FRAMES),
            span(MARK_FRAMES..MARK_FRAMES * 2),
            span(MARK_FRAMES * 2..frames)
        ]
    );
}

#[test]
fn a_row_of_whole_spans_has_no_extra_mark() {
    let mut fold = SpanFold::default();
    (0..MARK_FRAMES).for_each(|index| {
        fold.fold(&Synthetic::new(index, Cut::Overrun));
    });
    assert!(!fold.partial());
    let marks = fold.finish();
    assert!(
        marks
            .iter()
            .all(|(_, marks)| marks.as_ref().unwrap().len() == 1)
    );
}

#[test]
fn memories_fold_at_quiescent_cuts_only() {
    let cuts = [
        Cut::Requested,
        Cut::Overrun,
        Cut::VblankWithoutNmi,
        Cut::NmiStillRunning,
        Cut::Requested,
    ];
    let mut fold = SpanFold::default();
    let folds: Vec<FrameFold> = cuts
        .iter()
        .zip(0..)
        .map(|(&cut, index)| fold.fold(&Synthetic::new(index, cut)))
        .collect();
    let quiescent: Vec<bool> = folds.iter().map(|frame| frame.quiescent).collect();
    assert_eq!(quiescent, [true, false, false, false, true]);
    assert!(
        folds
            .iter()
            .all(|frame| matches!(frame.streams[6..8], [Folded::Digest(_), Folded::Digest(_)]))
    );
    assert!(
        folds[1..4]
            .iter()
            .all(|frame| frame.streams[0] == Folded::Skipped)
    );
    let vram = marks_of(fold.clone(), Stream::Vram).unwrap();
    assert_eq!(
        vram,
        [fold_all([
            Synthetic::memory_digest(0, Stream::Vram),
            Synthetic::memory_digest(4, Stream::Vram)
        ])]
    );
    let timing = marks_of(fold, Stream::Timing).unwrap();
    assert_eq!(
        timing,
        [fold_all(cuts.map(|cut| Digest::of(&cut.timing())))]
    );
}

#[test]
fn apu_folds_the_busy_span_at_its_closing_quiescent_cut() {
    let mut fold = SpanFold::default();
    let mut frame = |index, cut, ports: Vec<PortAccess>| {
        let mut frame = Synthetic::new(index, cut);
        frame.ports = ports;
        fold.fold(&frame).streams[5]
    };
    let read = PortAccess {
        direction: PortDirection::Read,
        port: 2,
        value: 0x99,
    };
    assert_eq!(
        frame(0, Cut::Requested, vec![write(0, 1)]),
        Folded::Digest(Digest::of(&[0, 0, 1]))
    );
    assert_eq!(frame(1, Cut::Overrun, vec![write(1, 2)]), Folded::Skipped);
    assert_eq!(frame(2, Cut::Overrun, vec![read]), Folded::Skipped);
    let busy = Digest::of(&[0, 1, 2, 1, 2, 0x99, 0, 3, 4]);
    assert_eq!(
        frame(3, Cut::Requested, vec![write(3, 4)]),
        Folded::Digest(busy)
    );
    assert_eq!(
        frame(4, Cut::Requested, Vec::new()),
        Folded::Digest(Digest::of(&[]))
    );
    // Accesses after the last quiescent cut are never folded.
    assert_eq!(frame(5, Cut::Overrun, vec![write(0, 7)]), Folded::Skipped);
    let apu = marks_of(fold, Stream::Apu).unwrap();
    assert_eq!(
        apu,
        [fold_all([Digest::of(&[0, 0, 1]), busy, Digest::of(&[])])]
    );
}

#[test]
fn a_stream_one_frame_lacks_is_absent_and_the_others_are_not() {
    let mut fold = SpanFold::default();
    for index in 0..10 {
        let mut frame = Synthetic::new(index, Cut::Requested);
        frame.video = index != 3;
        frame.sram = index != 7;
        let folded = fold.fold(&frame);
        assert_eq!(folded.streams[6] == Folded::Absent, index == 3);
    }
    // A quiescent stream a non-quiescent frame lacks isn't asked for, so stays present.
    let mut frame = Synthetic::new(10, Cut::Overrun);
    frame.sram = false;
    fold.fold(&frame);
    let marks = fold.finish();
    let absent: Vec<Stream> = marks
        .iter()
        .filter(|(_, marks)| marks.is_none())
        .map(|(stream, _)| *stream)
        .collect();
    assert_eq!(absent, [Stream::Sram, Stream::Video]);
}

#[test]
fn the_memory_digests_hash_as_the_arrays_did() {
    let words = [7_u16; 0x100];
    assert_eq!(cgram_digest(&words), hashed(&words));
    assert_eq!(vram_digest(&words[..3]), hashed(&[7_u16; 3]));
}
