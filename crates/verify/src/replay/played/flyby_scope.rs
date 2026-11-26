//! Ordinary Ceres through the stopped backdrop, then its flyby and room consumers.

use super::{
    Run, Stop, ceres_doors, compare, comparison_failure, finished, frame, played, quake, ridley,
    rows::Rows, seed_controller,
};
use crate::replay::{compare::Observation, data_failure, result::Outcome};
use oracle_record::FrameRecord;

const STOPPED_RETREAT: u64 = 1157;

/// First comparison after ordinary play reaches the stopped retreat.
pub(in crate::replay) const fn first_cut(start: u64) -> Option<u64> {
    let ready = if start < STOPPED_RETREAT {
        STOPPED_RETREAT
    } else {
        start
    };
    ready.checked_add(1)
}

impl Run<'_> {
    pub(super) fn flyby_scope(
        &mut self,
        rows: &mut Rows<'_>,
        mut previous: FrameRecord,
        start: u64,
        end: u64,
        mut record: impl FnMut(u64, Vec<Observation>) -> Result<(), Outcome>,
    ) -> Result<(), Outcome> {
        let first = first_cut(start).ok_or_else(|| data_failure(Some(start), "flyby origin"))?;
        if start < STOPPED_RETREAT {
            let controller = seed_controller(&previous)?;
            let (held, stop) = self.through(
                rows,
                previous,
                controller,
                start.saturating_add(1)..=STOPPED_RETREAT,
            )?;
            match stop {
                Stop::End if held == STOPPED_RETREAT.saturating_sub(start) => {}
                Stop::At(outcome) => return Err(outcome),
                Stop::End => return Err(data_failure(Some(start), "stopped flyby origin")),
            }
            previous = rows.record(STOPPED_RETREAT)?;
        }
        for cut in first..=end {
            let current = rows.record(cut)?;
            if !finished(&previous)
                || !finished(&current)
                || current.video_frame != previous.video_frame.wrapping_add(1)
            {
                return Err(data_failure(Some(cut), "complete flyby turn"));
            }
            let expected = self.flyby_scoped_turn(&previous, &current, cut)?;
            record(cut, expected)?;
            previous = current;
        }
        Ok(())
    }

    fn flyby_scoped_turn(
        &mut self,
        previous: &FrameRecord,
        current: &FrameRecord,
        cut: u64,
    ) -> Result<Vec<Observation>, Outcome> {
        let passing = played::passage::Passing {
            work_begun: true,
            work_done: true,
            publishes: true,
            actor_fetch: super::platform::actor_fetch(current),
            shaft_background: played::room_tiles::ShaftBackground::Pending,
            shaft_raster: played::passage::ShaftRaster::Pending,
            audio: played::passage::PassageAudio {
                shaft_reply: None,
                sounds_sent: true,
                music_ready: true,
            },
            random: None,
            start: played::arrival::Readiness::NotYet,
        };
        let effects = self
            .state
            .step_flyby(&passing, &self.played, &self.rules)
            .map_err(|fault| Outcome::Unsupported {
                cut: Some(cut),
                dependency: format!("played flyby frame {fault:?}"),
            })?;
        let mut expected = ridley::flyby::expected(previous, current)?;
        expected.extend(ridley::expected(&current.wram)?);
        expected.extend(ceres_doors::expected(&current.wram)?);
        expected.extend(quake::expected(previous, current, true)?);
        expected.extend(frame::expected(&current.wram)?);

        let mut actual = self.drawing.actual(&self.state, effects.picture)?;
        actual.extend(ridley::actual(&self.state.station));
        actual.extend(ceres_doors::actual(&self.state.station));
        actual.extend(quake::actual(&self.state, Some(effects.quake)));
        actual.extend(frame::actual(self.state.frame));
        compare(&expected, &actual).map_err(|fault| comparison_failure(cut, fault))?;
        Ok(expected)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::replay::{appearance, connected, movement};
    use types::ceres_getaway::GetawayState;

    #[test]
    fn comparison_starts_after_the_prelude_or_after_a_flyby_seed() {
        assert_eq!(first_cut(1155), Some(1158));
        assert_eq!(first_cut(1156), Some(1158));
        assert_eq!(first_cut(1157), Some(1158));
        assert_eq!(first_cut(1170), Some(1171));
        assert_eq!(first_cut(u64::MAX), None);
    }

    #[test]
    #[ignore = "pinned record cache and player ROM"]
    fn flyby_restores_both_native_paths_through_the_warning_handoff() {
        let sources = super::super::tests::sources();
        let lock = super::super::read_lock(&sources).unwrap();
        let content = connected::Content {
            movement: movement::Content::load(&sources.rom).unwrap(),
            appearance: appearance::Content::load(&sources.rom).unwrap(),
        };
        for row in ["chain-any-0006-E0B5", "samus-ceres-push"] {
            let mut rows = super::super::open_rows(&sources, &lock, row).unwrap();
            let seed = rows.record(1156).unwrap();
            let mut run = Run::admit(&seed, &content, &sources.rom, row).unwrap();
            run.flyby_scope(&mut rows, seed, 1156, 1270, |_, _| Ok(()))
                .unwrap();
            let settled = run.state.clone();
            let settled_drawing = run.drawing.retained().unwrap();
            assert_eq!(
                settled.station.ridley.unwrap().phase,
                boss_ceres_ridley::RidleyPhase::Getaway(GetawayState::WarningReady)
            );
            for cut in [1157, 1170, 1261, 1262, 1269, 1270] {
                let record = rows.record(cut).unwrap();
                let seed = rows.record(1156).unwrap();
                let mut resumed = Run::admit(&seed, &content, &sources.rom, row).unwrap();
                resumed
                    .flyby_scope(&mut rows, seed, 1156, cut, |_, _| Ok(()))
                    .unwrap();
                resumed.restore_flyby(&record).unwrap();
                resumed
                    .flyby_scope(&mut rows, record, cut, 1270, |_, _| Ok(()))
                    .unwrap();
                assert!(
                    resumed.state == settled,
                    "{row} restored flyby state at {cut}"
                );
                assert!(
                    resumed.drawing.retained().unwrap() == settled_drawing,
                    "{row} restored flyby drawing at {cut}"
                );
            }

            // Carry Samus and the rest of the room in ordinary play before crossing
            // the handoff. The scoped flyby deliberately holds Samus in place.
            let seed = rows.record(1156).unwrap();
            let mut live = Run::admit(&seed, &content, &sources.rom, row).unwrap();
            let controller = seed_controller(&seed).unwrap();
            assert_eq!(
                live.through(&mut rows, seed, controller, 1157..=1270)
                    .unwrap(),
                (114, Stop::End),
                "{row} live flyby"
            );
            let passed = rows.record(1270).unwrap();
            let mut restored = live.clone();
            restored.restore_flyby(&passed).unwrap();
            let controller = seed_controller(&passed).unwrap();
            for candidate in [&mut live, &mut restored] {
                assert_eq!(
                    candidate
                        .through(&mut rows, passed.clone(), controller, 1271..=1271)
                        .unwrap(),
                    (1, Stop::End),
                    "{row} first warning turn"
                );
            }
            assert!(restored.state == live.state, "{row} restored warning state");
            assert!(
                restored.warning_picture == live.warning_picture,
                "{row} restored warning picture"
            );
        }
    }

    impl Run<'_> {
        fn restore_flyby(&mut self, record: &FrameRecord) -> Result<(), Outcome> {
            use super::super::station;
            let wram = &record.wram;
            self.state.station =
                station::seed(wram, ceres_doors::seed(wram)?, self.state.station.stage)?;
            self.state.frame = frame::seed(wram)?;
            self.state.quake = canonical::model::Model(quake::seed(wram)?);
            self.drawing = ridley::flyby::Drawing::new(self.drawing.art.clone(), record);
            Ok(())
        }
    }
}
