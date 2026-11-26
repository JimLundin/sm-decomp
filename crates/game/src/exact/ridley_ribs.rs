//! Queue Ridley's ordinary ribs object strips in room drawing order.

use crate::state::video::{NmiBatch, VramSource, VramStep, VramTransfer, VramWord};
use data::assets::ridley_ribs::Strips;

/// Enqueue both rows for the next requested display upload.
pub fn queue(batch: &mut NmiBatch, strips: &Strips) {
    let destinations = [VramWord(0x7220), VramWord(0x7320)];
    for (to, strip) in destinations.into_iter().zip(strips) {
        batch.vram.push(VramTransfer {
            to,
            step: VramStep::Row,
            source: VramSource::Words {
                from: strip.source,
                words: strip.words.to_vec(),
            },
        });
    }
}

#[cfg(test)]
mod tests {
    use super::queue;
    use crate::state::video::Video;
    use data::assets::ridley_ribs::{STRIP_WORDS, Strip};
    use data::dma::BusAddress;
    use data::pose::tiles::Graphics;

    #[test]
    fn restored_ribs_queue_publishes_two_complete_rows() {
        let [upper, lower] = [0x1234, 0x5678].map(|word| [word; STRIP_WORDS]);
        let strips = [upper, lower].map(|words| Strip {
            source: BusAddress::default(),
            words,
        });
        let mut video = Video::default();
        queue(&mut video.batch, &strips);
        let mut restored: Video = canonical::from_bytes(&canonical::to_bytes(&video)).unwrap();
        for start in [0x7220, 0x7320] {
            assert_eq!(
                &restored.memory.vram[start..start + STRIP_WORDS],
                &[0; STRIP_WORDS]
            );
        }
        let serviced = restored.process_vram_writes(&Graphics::default());
        assert_eq!(
            serviced.iter().map(|write| write.to.0).collect::<Vec<_>>(),
            [0x7220, 0x7320]
        );
        assert_eq!(&restored.memory.vram[0x7220..0x7240], &upper);
        assert_eq!(&restored.memory.vram[0x7320..0x7340], &lower);
        assert_eq!(restored.batch.vram.len(), 0);
    }
}
