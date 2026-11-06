//! The NMI's video commit points (D16, K4): the only places video memory and the committed
//! registers change. Each copies a part of the batch the main strand built during the frame.

use data::dma::{OAM_BUFFER, PALETTE_BUFFER};
use data::pose::tiles::{Graphics, GraphicsRef, SamusTiles, TileDefinitionId};
use data::video::OAM_BYTES;

use crate::state::video::{
    BusAddress, CGRAM_COLOURS, DmaChannel, Mode7, SCROLL_BITS, Scroll, VRAM_WORDS, Video,
    VramSource, VramStep, VramTransfer, VramWord,
};

/// VRAM's word address is 15 bits; the PPU ignores the top bit.
const VRAM_ADDRESS_MASK: usize = VRAM_WORDS - 1;

/// Samus's tiles' VRAM destinations: each half's part 1, then its part 2 (`$80:9376`).
const SAMUS_TOP_TILES: [VramWord; 2] = [VramWord(0x6000), VramWord(0x6100)];
const SAMUS_BOTTOM_TILES: [VramWord; 2] = [VramWord(0x6080), VramWord(0x6180)];
/// The DMA channel OAM goes through (`$80:933A`).
const OAM_CHANNEL: usize = 0;
/// The DMA channel CGRAM, Samus's tiles and the VRAM write table go through (`$80:933A`,
/// `$80:9376`, `$80:8C83`).
const VRAM_CHANNEL: usize = 1;
/// DMAP for byte writes to one register, the A-bus address incrementing.
const DMA_ONE_REGISTER: u8 = 0x00;
/// DMAP for word writes to a register pair, the A-bus address incrementing.
const DMA_TWO_REGISTERS: u8 = 0x01;
/// BBAD for OAMDATA, `$2104`.
const DMA_TO_OAM: u8 = 0x04;
/// BBAD for VMDATAL, `$2118`.
const DMA_TO_VRAM: u8 = 0x18;
/// BBAD for CGDATA, `$2122`.
const DMA_TO_CGRAM: u8 = 0x22;
/// The bytes a DMA with a zero count transfers.
const FULL_COUNT: usize = 0x1_0000;

impl Video {
    /// Bank `$80` log, update OAM and CGRAM `$80:933A`: DMA 0 copies the OAM buffer to OAM
    /// and DMA 1 the whole palette buffer to CGRAM, both one register a byte.
    pub fn upload_oam_and_cgram(&mut self) {
        self.memory.oam = self.batch.oam;
        self.memory.cgram = self.batch.cgram;
        let uploads = [
            (OAM_CHANNEL, DMA_TO_OAM, OAM_BUFFER, OAM_BYTES),
            (
                VRAM_CHANNEL,
                DMA_TO_CGRAM,
                PALETTE_BUFFER,
                CGRAM_COLOURS * 2,
            ),
        ];
        for (channel, destination, from, bytes) in uploads {
            if let Some(channel) = self.dma.get_mut(channel) {
                channel.control = DMA_ONE_REGISTER;
                channel.destination = destination;
                channel.transferred(from, bytes);
            }
        }
    }

    /// Bank `$80` log, update IO registers `$80:91EE`: the register mirrors to the PPU.
    ///
    /// Each scroll register keeps the ten bits the PPU has of the mirror's word (the record's
    /// BG1HOFS holds 0 where the mirror holds 400h), the three COLDATA writes each set the
    /// components they select, and the mode 7 registers are written only in mode 7.
    // open: `$80:91EE` also tests `REG_BGMODE_FAKE`'s mode for the mode 7 registers.
    pub fn commit_registers(&mut self) {
        let mirrors = self.batch.registers;
        let fixed = self
            .batch
            .fixed_colour
            .iter()
            .fold(self.committed.colour_math.fixed, |colour, &write| {
                colour.with(write)
            });
        let mode7 = if mirrors.mode.is_mode7() {
            mirrors.mode7
        } else {
            Mode7 {
                select: mirrors.mode7.select,
                ..self.committed.mode7
            }
        };
        let mut committed = mirrors;
        committed.scroll = mirrors.scroll.map(|Scroll { x, y }| Scroll {
            x: x & SCROLL_BITS,
            y: y & SCROLL_BITS,
        });
        committed.colour_math.fixed = fixed;
        committed.mode7 = mode7;
        self.committed = committed;
    }

    /// Bank `$80` log, process VRAM write table `$80:8C83`: each queued upload DMAs to VRAM
    /// through channel 1 in queue order, and the queue empties. The channel keeps the last
    /// upload's registers; an empty queue leaves it as it was.
    pub fn process_vram_writes(&mut self, graphics: &Graphics) {
        for transfer in std::mem::take(&mut self.batch.vram) {
            let (from, bytes) = write_vram(&mut self.memory.vram, &transfer, graphics);
            if let Some(channel) = self.dma.get_mut(VRAM_CHANNEL) {
                channel.control = DMA_TWO_REGISTERS;
                channel.destination = DMA_TO_VRAM;
                channel.transferred(from, bytes);
            }
        }
    }

    /// Bank `$80` log, transfer Samus tiles to VRAM `$80:9376`: for each half her drawing
    /// queued, DMA 1 writes its definition's part 1 to `$6000` (the top) or `$6080` (the
    /// bottom) and, unless part 2's size is 0, part 2 from the bytes after it to `$6100` or
    /// `$6180`. The queue stays: the main loop clears it (`$82:8948`).
    pub fn transfer_samus_tiles(&mut self, tiles: &SamusTiles, graphics: &Graphics) {
        let queued = self.batch.samus_tiles;
        for (definition, destinations) in [
            (queued.top, SAMUS_TOP_TILES),
            (queued.bottom, SAMUS_BOTTOM_TILES),
        ] {
            if let Some(definition) = definition {
                self.transfer_samus_half(tiles, graphics, definition, destinations);
            }
        }
    }

    /// One half's DMAs. The channel keeps what the last leaves: the control, VMDATA, the
    /// source past the bytes it read and a spent count.
    fn transfer_samus_half(
        &mut self,
        tiles: &SamusTiles,
        graphics: &Graphics,
        definition: TileDefinitionId,
        [first, second]: [VramWord; 2],
    ) {
        let Some(definition) = tiles.definition(definition) else {
            return;
        };
        let Some(channel) = self.dma.get_mut(VRAM_CHANNEL) else {
            return;
        };
        channel.control = DMA_TWO_REGISTERS;
        channel.destination = DMA_TO_VRAM;
        let part_one = match definition.first {
            0 => FULL_COUNT,
            size => usize::from(size),
        };
        let vram = &mut self.memory.vram;
        dma_to_vram(
            vram,
            channel,
            graphics,
            (definition.source, part_one),
            first,
        );
        if definition.second != 0 {
            let part_two = (definition.second_source(), usize::from(definition.second));
            dma_to_vram(vram, channel, graphics, part_two, second);
        }
    }
}

/// `bytes` of ROM graphics as little-endian words, or `None` where the content has none.
fn rom_words(
    graphics: &Graphics,
    from: GraphicsRef,
    bytes: usize,
) -> Option<impl Iterator<Item = u16>> {
    let source = graphics.bytes(from, bytes)?;
    Some(
        source
            .as_chunks::<2>()
            .0
            .iter()
            .map(|&pair| u16::from_le_bytes(pair)),
    )
}

impl DmaChannel {
    /// The registers a finished transfer of `bytes` from `from` leaves: the source on by the
    /// bytes, within its bank, and the count spent.
    const fn transferred(&mut self, from: BusAddress, bytes: usize) {
        // The A-bus address counts 16 bits: a 64 KiB transfer comes back to its start.
        let [low, high, ..] = bytes.to_le_bytes();
        let past = from.advanced(u16::from_le_bytes([low, high]));
        self.source = past.offset;
        self.source_bank = past.bank;
        self.count = 0;
    }
}

/// One DMA of `bytes` from `from` to VRAM at `to`, leaving `channel` as the hardware does.
fn dma_to_vram(
    vram: &mut [u16],
    channel: &mut DmaChannel,
    graphics: &Graphics,
    (from, bytes): (GraphicsRef, usize),
    to: VramWord,
) {
    if let Some(words) = rom_words(graphics, from, bytes) {
        write_words(vram, to, 1, words);
    }
    channel.transferred(from.into(), bytes);
}

/// One upload, and where its DMA read from and how many bytes. ROM graphics the content
/// doesn't hold write nothing.
fn write_vram(
    vram: &mut [u16],
    transfer: &VramTransfer,
    graphics: &Graphics,
) -> (BusAddress, usize) {
    let stride = match transfer.step {
        VramStep::Row => 1,
        VramStep::Column => 32,
    };
    match &transfer.source {
        VramSource::Words { from, words } => {
            write_words(vram, transfer.to, stride, words.iter().copied());
            (*from, words.len().wrapping_mul(2))
        }
        VramSource::Content(rom) => {
            let bytes = usize::from(rom.words).wrapping_mul(2);
            if let Some(words) = rom_words(graphics, rom.from, bytes) {
                write_words(vram, transfer.to, stride, words);
            }
            (rom.from.into(), bytes)
        }
    }
}

/// `words` into VRAM from `to`, `stride` words apart. The address wraps within VRAM as the
/// PPU's does.
fn write_words(vram: &mut [u16], to: VramWord, stride: usize, words: impl Iterator<Item = u16>) {
    let start = usize::from(to.0);
    for (index, word) in words.enumerate() {
        let address = start.wrapping_add(index.wrapping_mul(stride)) & VRAM_ADDRESS_MASK;
        if let Some(slot) = vram.get_mut(address) {
            *slot = word;
        }
    }
}

#[cfg(test)]
mod tests {
    use data::pose::tiles::{
        FrameTiles, Graphics, GraphicsRef, SamusTiles, TileDefinition, TileDefinitionId,
    };

    use crate::state::video::{
        Brightness, BusAddress, Colour, ColourWrite, DmaChannel, FixedColour, Registers, RomWords,
        SamusTileUploads, Scroll, VRAM_WORDS, Video, VramSource, VramStep, VramTransfer, VramWord,
    };

    /// Words from a buffer at $7E:2000.
    fn words(words: Vec<u16>) -> VramSource {
        VramSource::Words {
            from: BusAddress {
                bank: 0x7E,
                offset: 0x2000,
            },
            words,
        }
    }

    fn transfer(to: u16, step: VramStep, source: VramSource) -> VramTransfer {
        VramTransfer {
            to: VramWord(to),
            step,
            source,
        }
    }

    /// Bank $9E's first eight bytes as graphics: words 0201h, 0403h, 0605h, 0807h.
    fn graphics() -> Graphics {
        Graphics::new(vec![(0x9E, vec![1, 2, 3, 4, 5, 6, 7, 8])])
    }

    const fn at(offset: u16) -> GraphicsRef {
        GraphicsRef { bank: 0x9E, offset }
    }

    #[test]
    fn the_queue_writes_in_order_and_empties() {
        let mut video = Video::default();
        video.batch.vram = vec![
            transfer(0x10, VramStep::Row, words(vec![1, 2, 3])),
            transfer(0x11, VramStep::Row, words(vec![9])),
        ];
        video.process_vram_writes(&Graphics::default());
        assert_eq!(video.memory.vram[0x10..0x13], [1, 9, 3]);
        assert_eq!(video.batch.vram, []);
        // The last upload's registers: two bytes from $7E:2000.
        assert_eq!(
            video.dma[1],
            DmaChannel {
                control: 0x01,
                destination: 0x18,
                source: 0x2002,
                source_bank: 0x7E,
                count: 0,
                ..DmaChannel::default()
            }
        );
    }

    #[test]
    fn an_empty_queue_leaves_channel_1_as_it_was() {
        let mut video = Video::default();
        video.dma[1].source = 0x1234;
        let before = video.dma[1];
        video.process_vram_writes(&Graphics::default());
        assert_eq!(video.dma[1], before);
    }

    #[test]
    fn oam_and_cgram_leave_channels_0_and_1_past_their_buffers() {
        let mut video = Video::default();
        video.dma[1].table = 0x1234;
        video.upload_oam_and_cgram();
        assert_eq!(
            video.dma[0],
            DmaChannel {
                control: 0x00,
                destination: 0x04,
                source: 0x0590,
                source_bank: 0x00,
                ..DmaChannel::default()
            }
        );
        assert_eq!(
            video.dma[1],
            DmaChannel {
                control: 0x00,
                destination: 0x22,
                source: 0xC200,
                source_bank: 0x7E,
                table: 0x1234,
                ..DmaChannel::default()
            }
        );
    }

    #[test]
    fn a_rom_upload_leaves_its_bank_and_the_bytes_past_it() {
        let mut video = Video::default();
        video.batch.vram = vec![transfer(
            0x20,
            VramStep::Row,
            VramSource::Content(RomWords {
                from: at(0x8002),
                words: 2,
            }),
        )];
        video.process_vram_writes(&graphics());
        assert_eq!(
            (video.dma[1].source_bank, video.dma[1].source),
            (0x9E, 0x8006)
        );
    }

    #[test]
    fn a_column_upload_steps_a_row_of_the_tilemap_and_wraps() {
        let mut video = Video::default();
        let last = u16::try_from(VRAM_WORDS - 32).unwrap();
        video.batch.vram = vec![transfer(last, VramStep::Column, words(vec![7, 8]))];
        video.process_vram_writes(&Graphics::default());
        assert_eq!(video.memory.vram[VRAM_WORDS - 32], 7);
        assert_eq!(video.memory.vram[0], 8);
    }

    #[test]
    fn content_uploads_write_the_rom_graphics_the_content_holds() {
        let mut video = Video::default();
        let rom = |offset, words| {
            VramSource::Content(RomWords {
                from: at(offset),
                words,
            })
        };
        video.batch.vram = vec![
            transfer(0x20, VramStep::Row, rom(0x8002, 2)),
            transfer(0x30, VramStep::Row, rom(0x8006, 2)),
        ];
        video.process_vram_writes(&graphics());
        assert_eq!(video.memory.vram[0x20..0x22], [0x0403, 0x0605]);
        // Past the bank's loaded bytes: nothing.
        assert_eq!(video.memory.vram[0x30..0x32], [0, 0]);
        assert_eq!(video.batch.vram, []);
    }

    /// A definition at $9E:8000: part 1 four bytes, part 2 `second` bytes after it.
    fn samus_tiles(second: u16) -> SamusTiles {
        SamusTiles::new(
            vec![vec![FrameTiles::default()]],
            vec![TileDefinition {
                source: at(0x8000),
                first: 4,
                second,
            }],
        )
    }

    #[test]
    fn samus_halves_upload_both_parts_through_dma_1() {
        let mut video = Video::default();
        video.dma[1].table = 0x1234;
        video.batch.samus_tiles = SamusTileUploads {
            top: Some(TileDefinitionId(0)),
            bottom: Some(TileDefinitionId(0)),
        };
        video.transfer_samus_tiles(&samus_tiles(4), &graphics());
        assert_eq!(video.memory.vram[0x6000..0x6002], [0x0201, 0x0403]);
        assert_eq!(video.memory.vram[0x6100..0x6102], [0x0605, 0x0807]);
        assert_eq!(video.memory.vram[0x6080..0x6082], [0x0201, 0x0403]);
        assert_eq!(video.memory.vram[0x6180..0x6182], [0x0605, 0x0807]);
        assert_eq!(
            video.dma[1],
            DmaChannel {
                control: 0x01,
                destination: 0x18,
                source: 0x8008,
                source_bank: 0x9E,
                count: 0,
                table: 0x1234,
                ..DmaChannel::default()
            }
        );
        // The queue stays for the main loop to clear.
        assert!(video.batch.samus_tiles.top.is_some());
    }

    #[test]
    fn a_part_2_of_size_0_and_an_empty_queue_transfer_nothing_more() {
        let mut video = Video::default();
        video.transfer_samus_tiles(&samus_tiles(0), &graphics());
        assert_eq!(video.dma[1], DmaChannel::default());
        video.batch.samus_tiles.top = Some(TileDefinitionId(0));
        video.transfer_samus_tiles(&samus_tiles(0), &graphics());
        assert_eq!(video.memory.vram[0x6000..0x6002], [0x0201, 0x0403]);
        assert_eq!(video.memory.vram[0x6100], 0);
        assert_eq!(video.dma[1].source, 0x8004);
    }

    #[test]
    fn the_palette_buffer_and_mirrors_commit() {
        let mut video = Video::default();
        video.batch.cgram[5] = Colour(0x7FFF);
        video.batch.registers = Registers {
            brightness: Brightness(15),
            ..Registers::default()
        };
        video.upload_oam_and_cgram();
        video.commit_registers();
        assert_eq!(video.memory.cgram[5], Colour(0x7FFF));
        assert_eq!(video.committed.brightness, Brightness(15));
    }

    #[test]
    fn the_scroll_keeps_ten_bits_and_coldata_sets_what_it_selects() {
        let mut video = Video::default();
        video.batch.registers.scroll[0] = Scroll {
            x: 0x0400,
            y: 0x1200,
        };
        video.batch.fixed_colour = [ColourWrite(0x26), ColourWrite(0x42), ColourWrite(0x00)];
        video.commit_registers();
        assert_eq!(video.committed.scroll[0], Scroll { x: 0, y: 0x200 });
        assert_eq!(
            video.committed.colour_math.fixed,
            FixedColour {
                red: 6,
                green: 2,
                blue: 0
            }
        );
    }
}
