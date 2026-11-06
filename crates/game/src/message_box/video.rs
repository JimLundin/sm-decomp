//! The message box's video work: the PPU writes, DMAs and HDMA set-up the box makes itself,
//! straight to the PPU while the NMI lags, rather than through the NMI's batch.

use data::dma::{
    MESSAGE_BOX_BACKUP, MESSAGE_BOX_BLANK, MESSAGE_BOX_ROWS, MESSAGE_BOX_SCROLL_TABLE,
};

use crate::commit::{DMA_TO_VRAM, DMA_TWO_REGISTERS, VRAM_CHANNEL, write_words};
use crate::state::video::{
    ColourWrite, DmaChannel, Layer, Layers, MESSAGE_BOX_BACKUP_WORDS, Playfield,
    PlayfieldColourMath, Scroll, TilemapBase, Video, VramWord,
};

/// The BG3 tilemap rows the box covers: from `$5880`, 700h bytes (`$85:81B7`).
const BOX_TILEMAP: VramWord = VramWord(0x5880);
const BOX_TILEMAP_BYTES: u16 = 0x0700;
/// The box's BG3 tilemap rows in words.
const BOX_TILEMAP_WORDS: usize = MESSAGE_BOX_BACKUP_WORDS;
/// BG3's tilemap base while the box is up, which its rows are placed from (`$85:8326`).
const MESSAGE_BOX_TILEMAP_BASE: VramWord = VramWord(0x5800);
/// The blank box tilemap's word (`$85:823F`).
const BLANK_TILE: u16 = 0x000E;

/// The text colours the box writes to CGRAM, from BG3 palette 6's colour 1: the Y button's
/// green and the save arrow's red (`$85:8150`).
const TEXT_COLOURS_START: usize = 0x19;
const TEXT_COLOURS: [u16; 2] = [0x0BB1, 0x001F];
/// Where in CGRAM the text colours are.
const TEXT_COLOUR_RANGE: std::ops::Range<usize> =
    TEXT_COLOURS_START..TEXT_COLOURS_START + TEXT_COLOURS.len();

/// The playfield the box shows over: its BG3 tilemap at `$5800`, 32x32, BG1, BG2, BG3 and
/// sprites on the main screen, no colour math (`$85:8175`).
const BOX_BG3_TILEMAP: TilemapBase = TilemapBase(0x58);
const BOX_MAIN_SCREEN: [Layer; 4] = [Layer::Bg1, Layer::Bg2, Layer::Bg3, Layer::Objects];

/// The three COLDATA writes that make the sub screen's backdrop black (`$85:8181`).
const BLACK_BACKDROP: [ColourWrite; 3] = [ColourWrite(0x20), ColourWrite(0x40), ColourWrite(0x80)];

/// DMAP for a VRAM read: B bus to A bus, a register pair (`$85:81C0`).
const DMA_FROM_TWO_REGISTERS: u8 = 0x81;
/// BBAD for VMDATALREAD, `$2139`.
const DMA_FROM_VRAM: u8 = 0x39;

/// HDMA channel 6, the box's BG3 Y scroll (`$85:838F`).
const SCROLL_CHANNEL: usize = 6;
/// DMAP for indirect HDMA writing one register twice.
const HDMA_INDIRECT_WRITE_TWICE: u8 = 0x42;
/// BBAD for BG3VOFS, `$2112`.
const DMA_TO_BG3_Y_SCROLL: u8 = 0x12;

/// BG3's index among the backgrounds.
const BG3: usize = 2;

impl Video {
    /// Bank `$85` log, initialise PPU for message boxes `$85:814B`, after its first wait:
    /// BG3 palette 6's text colours straight to CGRAM, the playfield the box shows over, a
    /// black sub screen backdrop and BG3's scroll at 0.
    // open: HDMA's enable ($420C cleared), the saved HDMA channels the restore reads
    // ($7E:33EA), and the cleared scroll table ($7E:3000) join with the HDMA channels.
    pub(crate) fn prepare_message_box(&mut self) {
        self.message_box.bg3_tilemap = self.playfield.bg3_tilemap;
        if let Some(colours) = self.memory.cgram.get_mut(TEXT_COLOUR_RANGE) {
            for (colour, &value) in colours.iter_mut().zip(&TEXT_COLOURS) {
                colour.0 = value;
            }
        }
        let mut main_screen = Layers::default();
        for layer in BOX_MAIN_SCREEN {
            main_screen.insert(layer);
        }
        self.playfield = Playfield {
            main_screen,
            bg3_tilemap: BOX_BG3_TILEMAP,
            colour_math: PlayfieldColourMath::default(),
        };
        let committed = &mut self.committed;
        committed.colour_math.fixed = BLACK_BACKDROP
            .iter()
            .fold(committed.colour_math.fixed, |colour, &write| {
                colour.with(write)
            });
        // Each scroll register takes two writes of 0: the low then the high byte.
        if let Some(scroll) = committed.scroll.get_mut(BG3) {
            *scroll = Scroll::default();
        }
    }

    /// Bank `$85` log, `$85:81B5`: DMA 1 reads the BG3 tilemap the box covers from VRAM into
    /// its backup.
    pub(crate) fn back_up_message_box_tilemap(&mut self) {
        let from = usize::from(BOX_TILEMAP.0);
        if let Some(rows) = self
            .memory
            .vram
            .get(from..from.saturating_add(BOX_TILEMAP_WORDS))
        {
            self.message_box.tilemap.clear();
            self.message_box.tilemap.extend_from_slice(rows);
        }
        if let Some(channel) = self.dma.get_mut(VRAM_CHANNEL) {
            channel.control = DMA_FROM_TWO_REGISTERS;
            channel.destination = DMA_FROM_VRAM;
            channel.box_transfer(MESSAGE_BOX_BACKUP, BOX_TILEMAP_BYTES);
        }
    }

    /// Bank `$85` log, clear message box BG3 tilemap `$85:8206`: DMA 1 writes the blank tilemap
    /// `$85:81F3` filled over the box's rows.
    pub(crate) fn upload_blank_message_box(&mut self) {
        let words = usize::from(BOX_TILEMAP_BYTES / 2);
        write_words(
            &mut self.memory.vram,
            BOX_TILEMAP,
            1,
            std::iter::repeat_n(BLANK_TILE, words),
        );
        if let Some(channel) = self.dma.get_mut(VRAM_CHANNEL) {
            channel.control = DMA_TWO_REGISTERS;
            channel.destination = DMA_TO_VRAM;
            channel.box_transfer(MESSAGE_BOX_BLANK, BOX_TILEMAP_BYTES);
        }
    }

    /// Bank `$85` log, `$85:8324`: DMA 1 writes the box's `rows`, a border, the message and a
    /// border, to BG3's tilemap `rows_at` words past `$5800`.
    pub(crate) fn upload_message_box(&mut self, rows: &[u16], rows_at: u16) {
        write_words(
            &mut self.memory.vram,
            VramWord(MESSAGE_BOX_TILEMAP_BASE.0.wrapping_add(rows_at)),
            1,
            rows.iter().copied(),
        );
        if let Some(channel) = self.dma.get_mut(VRAM_CHANNEL) {
            channel.control = DMA_TWO_REGISTERS;
            channel.destination = DMA_TO_VRAM;
            let bytes = u16::try_from(rows.len().saturating_mul(2)).unwrap_or(u16::MAX);
            channel.box_transfer(MESSAGE_BOX_ROWS, bytes);
        }
    }

    /// Bank `$85` log, `$85:838F`: HDMA channel 6, indirect, writes BG3's Y scroll twice a line
    /// from the scroll table, which is also where the indirect address starts. The table
    /// address and line counter clear; HDMA reloads them when the frame starts.
    pub(crate) fn set_up_message_box_scroll(&mut self) {
        if let Some(channel) = self.dma.get_mut(SCROLL_CHANNEL) {
            let table = MESSAGE_BOX_SCROLL_TABLE;
            *channel = DmaChannel {
                control: HDMA_INDIRECT_WRITE_TWICE,
                destination: DMA_TO_BG3_Y_SCROLL,
                source: table.offset,
                source_bank: table.bank,
                count: table.offset,
                indirect_bank: table.bank,
                table: 0,
                line_counter: 0,
            };
        }
    }
}

impl Video {
    /// Bank `$85` log, restore PPU `$85:861F`: DMA 1 writes the backup over the box's rows.
    pub(crate) fn restore_message_box_tilemap(&mut self) {
        write_words(
            &mut self.memory.vram,
            BOX_TILEMAP,
            1,
            self.message_box.tilemap.iter().copied(),
        );
        if let Some(channel) = self.dma.get_mut(VRAM_CHANNEL) {
            channel.control = DMA_TWO_REGISTERS;
            channel.destination = DMA_TO_VRAM;
            channel.box_transfer(MESSAGE_BOX_BACKUP, BOX_TILEMAP_BYTES);
        }
    }

    /// Bank `$85` log, `$85:8654`: the playfield's main screen and colour math back from the
    /// mirrors the NMI commits, and the text colours back from the palette buffer.
    // open: HDMA's enable comes back from what `$85:8169` saved ($7E:33EA), which joins with
    // the HDMA channels.
    pub(crate) fn restore_message_box_registers(&mut self) {
        self.playfield.bg3_tilemap = self.message_box.bg3_tilemap;
        self.playfield.main_screen = self.batch.registers.main_screen;
        self.playfield.colour_math = self.batch.playfield_colour_math;
        if let (Some(colours), Some(saved)) = (
            self.memory.cgram.get_mut(TEXT_COLOUR_RANGE),
            self.batch.cgram.get(TEXT_COLOUR_RANGE),
        ) {
            colours.copy_from_slice(saved);
        }
    }
}

impl DmaChannel {
    /// One of the box's DMAs of `bytes` with `buffer` on the A bus. The box's `STZ $43x7` and
    /// `STZ $43x9` also clear the indirect bank, the table address and the line counter.
    fn box_transfer(&mut self, buffer: data::dma::BusAddress, bytes: u16) {
        self.transferred(buffer, usize::from(bytes));
        self.indirect_bank = 0;
        self.table = 0;
        self.line_counter = 0;
    }
}

#[cfg(test)]
mod tests {
    use crate::state::video::{
        Colour, DmaChannel, FixedColour, Layer, Scroll, TilemapBase, VRAM_WORDS, Video,
    };

    #[test]
    fn preparing_writes_the_text_colours_and_the_box_playfield() {
        let mut video = Video::default();
        video.committed.colour_math.fixed = FixedColour {
            red: 6,
            green: 2,
            blue: 1,
        };
        video.committed.scroll[2] = Scroll { x: 5, y: 9 };
        video.playfield.colour_math.layers.0 = 0xB3;
        video.batch.cgram[0x19] = Colour(0x1234);
        video.prepare_message_box();
        assert_eq!(
            video.memory.cgram[0x19..0x1B],
            [Colour(0x0BB1), Colour(0x001F)]
        );
        // The palette buffer stays for the next requested NMI.
        assert_eq!(video.batch.cgram[0x19], Colour(0x1234));
        assert_eq!(video.playfield.bg3_tilemap, TilemapBase(0x58));
        assert_eq!(video.playfield.main_screen.bits, 0x17);
        assert!(!video.playfield.main_screen.contains(Layer::Bg4));
        assert_eq!(video.playfield.colour_math.layers.0, 0);
        assert_eq!(video.committed.colour_math.fixed, FixedColour::default());
        assert_eq!(video.committed.scroll[2], Scroll::default());
    }

    #[test]
    fn the_backup_reads_vram_through_dma_1() {
        let mut video = Video::default();
        video.dma[1].table = 0xFFFF;
        video.dma[1].line_counter = 0xFF;
        video.back_up_message_box_tilemap();
        assert_eq!(
            video.dma[1],
            DmaChannel {
                control: 0x81,
                destination: 0x39,
                source: 0x4800,
                source_bank: 0x7E,
                ..DmaChannel::default()
            }
        );
    }

    #[test]
    fn the_blank_box_covers_its_rows_and_no_more() {
        let mut video = Video::default();
        video.memory.vram = vec![0x1111; VRAM_WORDS];
        video.upload_blank_message_box();
        assert_eq!(video.memory.vram[0x587F], 0x1111);
        assert!(
            video.memory.vram[0x5880..0x5C00]
                .iter()
                .all(|&word| word == 0x000E)
        );
        assert_eq!(video.memory.vram[0x5C00], 0x1111);
        assert_eq!(
            video.dma[1],
            DmaChannel {
                control: 0x01,
                destination: 0x18,
                source: 0x3F00,
                source_bank: 0x7E,
                ..DmaChannel::default()
            }
        );
    }

    #[test]
    fn restoring_takes_the_mirrors_and_the_palette_buffer_back() {
        let mut video = Video::default();
        video.batch.registers.main_screen.insert(Layer::Bg1);
        video.batch.playfield_colour_math.layers.0 = 0xB3;
        video.batch.cgram[0x19] = Colour(0x524A);
        video.batch.cgram[0x1B] = Colour(0x7FFF);
        video.prepare_message_box();
        video.restore_message_box_registers();
        assert_eq!(video.playfield.main_screen.bits, 0x01);
        assert_eq!(video.playfield.colour_math.layers.0, 0xB3);
        assert_eq!(
            video.memory.cgram[0x19..0x1C],
            [Colour(0x524A), Colour(0), Colour(0)]
        );
    }

    #[test]
    fn the_restore_dma_reads_the_backup() {
        let mut video = Video::default();
        video.restore_message_box_tilemap();
        assert_eq!(
            (
                video.dma[1].control,
                video.dma[1].destination,
                video.dma[1].source
            ),
            (0x01, 0x18, 0x4800)
        );
    }

    #[test]
    fn the_scroll_channel_points_at_its_table() {
        let mut video = Video::default();
        video.dma[6].table = 0x1234;
        video.set_up_message_box_scroll();
        assert_eq!(
            video.dma[6],
            DmaChannel {
                control: 0x42,
                destination: 0x12,
                source: 0x3380,
                source_bank: 0x7E,
                count: 0x3380,
                indirect_bank: 0x7E,
                table: 0,
                line_counter: 0,
            }
        );
    }

    #[test]
    fn the_backup_and_the_restore_round_trip_the_boxs_rows() {
        let mut video = Video::default();
        for (index, word) in video.memory.vram[0x5880..0x5C00].iter_mut().enumerate() {
            *word = 0x2000 | u16::try_from(index).unwrap_or(0);
        }
        video.playfield.bg3_tilemap = TilemapBase(0x5A);
        video.prepare_message_box();
        video.back_up_message_box_tilemap();
        assert_eq!(video.message_box.tilemap.len(), 0x380);
        assert_eq!(video.message_box.tilemap[0x37F], 0x237F);
        assert_eq!(video.playfield.bg3_tilemap, TilemapBase(0x58));
        video.upload_blank_message_box();
        assert_eq!(video.memory.vram[0x5880], 0x000E);
        video.restore_message_box_tilemap();
        video.restore_message_box_registers();
        assert_eq!(video.memory.vram[0x5880], 0x2000);
        assert_eq!(video.memory.vram[0x5BFF], 0x237F);
        assert_eq!(video.playfield.bg3_tilemap, TilemapBase(0x5A));
    }

    #[test]
    fn the_rows_upload_where_the_setup_puts_them() {
        let mut video = Video::default();
        video.upload_message_box(&[1, 2, 3], 0x1C0);
        assert_eq!(video.memory.vram[0x59C0..0x59C3], [1, 2, 3]);
        assert_eq!(video.memory.vram[0x59BF], 0);
        assert_eq!(video.memory.vram[0x59C3], 0);
        assert_eq!(
            video.dma[1],
            DmaChannel {
                control: 0x01,
                destination: 0x18,
                source: 0x3206,
                source_bank: 0x7E,
                ..DmaChannel::default()
            }
        );
    }
}
