//! Pure palette transfers while a compatibility scope owns the selected colors.
#![forbid(unsafe_code)]
use crate::scene::state::WordOrder;
use sm_data::cartridge::RomImage;

#[derive(Clone, Copy)]
pub enum Buffer {
    Current,
    Target,
}

#[derive(Clone, Copy)]
pub struct Span {
    pub buffer: Buffer,
    pub first: usize,
    pub words: usize,
    pub descending: bool,
    pub order: WordOrder,
}
impl Span {
    pub fn point(self, index: usize) -> (Buffer, usize) {
        let first = (match self.buffer {
            Buffer::Current => 0usize,
            Buffer::Target => 512usize,
        })
        .checked_add(self.first)
        .expect("palette position");
        let distance = index.checked_mul(2).expect("palette extent");
        let offset = if self.descending {
            first.checked_sub(distance)
        } else {
            first.checked_add(distance)
        }
        .expect("palette position");
        (Buffer::Current, offset)
    }
}
pub enum Source {
    Fill(u16),
    Colors(Span),
    Rom { image: RomImage, first: usize },
}
pub struct Transfer {
    pub destination: Span,
    pub source: Source,
}

/// Constructed only from the actual selected image at an ownership boundary.
pub struct Image {
    pub colors: [u16; 256],
    pub targets: [u16; 256],
}
impl Image {
    fn coordinate(buffer: Buffer, offset: usize) -> (bool, usize) {
        let offset = (match buffer {
            Buffer::Current => 0usize,
            Buffer::Target => 512usize,
        })
        .checked_add(offset)
        .expect("palette offset");
        if offset < 512 {
            (false, offset)
        } else {
            (true, offset - 512)
        }
    }
    pub fn byte(&self, buffer: Buffer, offset: usize) -> u8 {
        let (target, offset) = Self::coordinate(buffer, offset);
        let colors = if target { &self.targets } else { &self.colors };
        colors[offset / 2].to_le_bytes()[offset & 1]
    }
    fn word(&self, span: Span, index: usize) -> u16 {
        let (buffer, offset) = span.point(index);
        span.order
            .decode([self.byte(buffer, offset), self.byte(buffer, offset + 1)])
    }
    fn store_byte(&mut self, buffer: Buffer, offset: usize, byte: u8) {
        let (target, offset) = Self::coordinate(buffer, offset);
        let colors = if target {
            &mut self.targets
        } else {
            &mut self.colors
        };
        let color = &mut colors[offset / 2];
        let mut bytes = color.to_le_bytes();
        bytes[offset & 1] = byte;
        *color = u16::from_le_bytes(bytes);
    }
}
/// Publish the encoding of each completed typed mutation before the next read.
pub trait Publication {
    fn byte(&mut self, buffer: Buffer, offset: usize, byte: u8);
}
pub trait Host {
    fn palette_transfer(&mut self, transfer: Transfer);
}

/// `first` was selected at the original supplier query before ownership changed.
/// All later RAM source reads belong to `image`, including overlap propagation.
pub fn run(transfer: Transfer, first: u16, image: &mut Image, publish: &mut impl Publication) {
    for index in 0..transfer.destination.words {
        let value = if index == 0 {
            first
        } else {
            match &transfer.source {
                Source::Fill(value) => *value,
                Source::Colors(source) => image.word(*source, index),
                Source::Rom { image, first } => {
                    let offset = first
                        .checked_add(index.checked_mul(2).expect("palette source extent"))
                        .expect("palette source offset");
                    u16::from_ne_bytes([image.read(offset), image.read(offset + 1)])
                }
            }
        };
        let (buffer, offset) = transfer.destination.point(index);
        for (lane, byte) in transfer
            .destination
            .order
            .encode(value)
            .into_iter()
            .enumerate()
        {
            image.store_byte(buffer, offset + lane, byte);
            publish.byte(buffer, offset + lane, byte);
        }
    }
}
