//! T1 timing observed at the Ceres return's darkening turn boundary.

use data::assets::world::doorways::RIDLEY_TO_ESCAPE_58;
use data::id::DoorwayId;
use oracle_record::FrameRecord;
use world_doors::{Doors, Passage, Phase};

/// A fade turn can be ready before NMI publishes it.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub(super) struct ReturnTiming {
    awaiting_publication: bool,
}

pub(super) struct CutTiming {
    pub(super) ready: bool,
    pub(super) publishes: bool,
}

impl ReturnTiming {
    pub(super) fn cut(
        &mut self,
        door: Option<DoorwayId>,
        doors: Doors,
        record: &FrameRecord,
    ) -> CutTiming {
        let publishes = super::finished(record);
        let ready = self.ready(
            door,
            doors,
            publishes,
            record.nmi().map(|nmi| nmi.interrupted_pc),
        );
        CutTiming { ready, publishes }
    }

    /// Read the native platform timing for this cut. The caller supplies the
    /// carried doorway and phase. The interrupted site has reached downstream
    /// actor, drawing or audio work on the admitted overruns after interpolation.
    pub(super) fn ready(
        &mut self,
        door: Option<DoorwayId>,
        doors: Doors,
        published: bool,
        interrupted_pc: Option<u32>,
    ) -> bool {
        let darkening = door == Some(RIDLEY_TO_ESCAPE_58)
            && matches!(
                doors,
                Doors::Passing(Passage {
                    phase: Phase::Darkening(_),
                    ..
                })
            );
        let early = darkening
            && matches!(
                interrupted_pc,
                Some(0x0090_c5df | 0x0081_89af | 0x0081_8a2d | 0x00a0_8888 | 0x0080_9fa6)
            );
        let ready = !self.awaiting_publication && (published || early);
        self.awaiting_publication = !published && (self.awaiting_publication || ready);
        ready
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use data::assets::world::doorways::ESCAPE_58_TO_RIDLEY;
    use world_doors::FadeTime;

    #[test]
    fn return_work_precedes_publication_once() {
        let mut timing = ReturnTiming::default();
        let darkening = Doors::Passing(Passage {
            phase: Phase::Darkening(FadeTime::START),
            ..Passage::START
        });
        let site = Some(0x0090_c5df);
        assert!(!timing.ready(
            Some(RIDLEY_TO_ESCAPE_58),
            Doors::Passing(Passage::START),
            false,
            site
        ));
        assert!(!timing.ready(Some(ESCAPE_58_TO_RIDLEY), darkening, false, site));
        assert!(!timing.ready(
            Some(RIDLEY_TO_ESCAPE_58),
            darkening,
            false,
            Some(0x0082_da3d)
        ));
        assert!(timing.ready(Some(RIDLEY_TO_ESCAPE_58), darkening, false, site));
        let checkpoint = timing;
        assert!(!timing.ready(Some(RIDLEY_TO_ESCAPE_58), darkening, false, site));
        assert!(!timing.ready(Some(RIDLEY_TO_ESCAPE_58), darkening, true, None));
        assert!(timing.ready(Some(RIDLEY_TO_ESCAPE_58), darkening, true, None));
        let mut restored = checkpoint;
        assert!(!restored.ready(Some(RIDLEY_TO_ESCAPE_58), darkening, true, None));
        assert!(restored.ready(Some(RIDLEY_TO_ESCAPE_58), darkening, true, None));
    }
}
