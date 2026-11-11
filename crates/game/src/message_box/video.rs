//! The message box's BG3 Y scroll HDMA channel, which the box points itself.
//!
//! Its overlay rows and PPU set-up are encoded in `exact` (`exact::tilemap::write_now`,
//! `exact::ppu`). The channel waits for the line program's HDMA channels (E3), with the box's
//! scroll table and HDMA's enable, which are not modelled yet.

use data::dma::MESSAGE_BOX_SCROLL_TABLE;

use crate::state::video::{BBusRegister, DmaChannel, DmaControl, Video};

/// HDMA channel 6, the box's BG3 Y scroll (`$85:838F`).
const SCROLL_CHANNEL: usize = 6;

impl Video {
    /// Bank `$85` log, `$85:838F`: HDMA channel 6, indirect, writes BG3's Y scroll twice a line
    /// from the scroll table, which is also where the indirect address starts. The table
    /// address and line counter clear; HDMA reloads them when the frame starts.
    pub(crate) fn set_up_message_box_scroll(&mut self) {
        if let Some(channel) = self.dma.get_mut(SCROLL_CHANNEL) {
            let table = MESSAGE_BOX_SCROLL_TABLE;
            *channel = DmaChannel {
                control: DmaControl::HDMA_INDIRECT_WRITE_TWICE,
                destination: BBusRegister::Bg3VerticalScroll,
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

#[cfg(test)]
mod tests {
    use crate::state::video::{BBusRegister, DmaChannel, DmaControl, Video};

    #[test]
    fn the_scroll_channel_points_at_its_table() {
        let mut video = Video::default();
        video.dma[6].table = 0x1234;
        video.set_up_message_box_scroll();
        assert_eq!(
            video.dma[6],
            DmaChannel {
                control: DmaControl::HDMA_INDIRECT_WRITE_TWICE,
                destination: BBusRegister::Bg3VerticalScroll,
                source: 0x3380,
                source_bank: 0x7E,
                count: 0x3380,
                indirect_bank: 0x7E,
                table: 0,
                line_counter: 0,
            }
        );
    }
}
