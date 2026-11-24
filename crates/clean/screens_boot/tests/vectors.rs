//! Every vector of `screens-boot.toml` through the model: the State before, the frame end, and
//! the State after (or the title's frame). A key the file's header doesn't give fails the read.

#[cfg(test)]
mod vectors {
    use std::collections::BTreeSet;

    use enum_map::Enum;
    use screens_boot::{
        BehaviourId, DriverUpload, FadeStep, FinishedFrames, FrameEnd, FrameWork, HoldFrames,
        Level, Outcome, PowerOnStage, PowerOnState, Rules, ScreenLight, catalogue, step,
    };
    use serde::Deserialize;
    use types::Checked;

    const VECTORS: &str = include_str!("../../../../scenarios/vectors/screens-boot.toml");

    #[derive(Deserialize)]
    #[serde(deny_unknown_fields)]
    struct VectorFile {
        vector: Vec<Vector>,
    }

    #[derive(Deserialize, Debug)]
    #[serde(rename_all = "kebab-case")]
    enum Source {
        Recorded,
        Constructed,
    }

    #[derive(Deserialize)]
    #[serde(deny_unknown_fields)]
    struct Vector {
        id: String,
        source: Source,
        entries: Vec<String>,
        #[serde(default)]
        rules: RulesRecord,
        before: StateRecord,
        finished: bool,
        #[serde(default)]
        upload_ended: bool,
        after: AfterRecord,
    }

    #[derive(Deserialize, Default)]
    #[serde(deny_unknown_fields)]
    struct RulesRecord {
        fade_step: Option<u8>,
        hold: Option<u16>,
    }

    #[derive(Deserialize, Clone, Copy)]
    #[serde(rename_all = "kebab-case")]
    enum StageRecord {
        BeforeLogo,
        Brightening,
        Held,
        Dimming,
        SoundLoading,
        Settling,
    }

    #[derive(Deserialize, Clone, Copy)]
    #[serde(rename_all = "lowercase")]
    enum ScreenRecord {
        Blank,
        Lit,
    }

    #[derive(Deserialize)]
    #[serde(deny_unknown_fields)]
    struct StateRecord {
        stage: StageRecord,
        hold_left: Option<u16>,
        screen: (ScreenRecord, u8),
        frames: u16,
    }

    #[derive(Deserialize)]
    #[serde(deny_unknown_fields)]
    struct TitleRecord {
        unsupported: bool,
    }

    #[derive(Deserialize)]
    #[serde(untagged)]
    enum AfterRecord {
        Title(TitleRecord),
        State(StateRecord),
    }

    fn rules(record: &RulesRecord) -> Rules {
        let reference = Rules::REFERENCE;
        Rules {
            fade_step: record.fade_step.map_or(reference.fade_step, |levels| {
                FadeStep::new(levels).expect("a fade step of 1 to 15")
            }),
            hold: record.hold.map_or(reference.hold, HoldFrames),
        }
    }

    fn state(record: &StateRecord) -> PowerOnState {
        let stage = match (record.stage, record.hold_left) {
            (StageRecord::BeforeLogo, None) => PowerOnStage::BeforeLogo,
            (StageRecord::Brightening, None) => PowerOnStage::Brightening,
            (StageRecord::Held, Some(left)) => PowerOnStage::Held(HoldFrames(left)),
            (StageRecord::Dimming, None) => PowerOnStage::Dimming,
            (StageRecord::SoundLoading, None) => PowerOnStage::SoundLoading,
            (StageRecord::Settling, None) => PowerOnStage::Settling,
            _ => panic!("hold_left is given with \"held\" and only then"),
        };
        let level = Level::new(record.screen.1).expect("a level of 0 to 15");
        let screen = match record.screen.0 {
            ScreenRecord::Blank => ScreenLight::Blank(level),
            ScreenRecord::Lit => ScreenLight::Lit(level),
        };
        PowerOnState {
            stage,
            screen,
            frames: FinishedFrames(record.frames),
        }
    }

    fn expected(record: &AfterRecord) -> Outcome {
        match record {
            AfterRecord::Title(TitleRecord { unsupported: true }) => Outcome::TitleBegins,
            AfterRecord::Title(TitleRecord { unsupported: false }) => panic!("unsupported = false"),
            AfterRecord::State(after) => Outcome::PowerOn(state(after)),
        }
    }

    fn frame_end(vector: &Vector) -> FrameEnd {
        FrameEnd {
            work: if vector.finished {
                FrameWork::Finished
            } else {
                FrameWork::Unfinished
            },
            upload: if vector.upload_ended {
                DriverUpload::Ended
            } else {
                DriverUpload::Continuing
            },
        }
    }

    fn behaviour_id(name: &str) -> BehaviourId {
        catalogue()
            .into_iter()
            .map(|(id, _)| id)
            .find(|id| format!("{id:?}") == name)
            .unwrap_or_else(|| panic!("no spec entry {name}"))
    }

    fn vectors() -> Vec<Vector> {
        toml::from_str::<VectorFile>(VECTORS)
            .expect("the vector file reads")
            .vector
    }

    #[test]
    fn every_vector_passes() {
        let failures: Vec<String> = vectors()
            .iter()
            .filter_map(|vector| {
                let actual = step(
                    rules(&vector.rules),
                    state(&vector.before),
                    frame_end(vector),
                );
                let wanted = expected(&vector.after);
                (actual != wanted).then(|| {
                    format!(
                        "{} ({:?}): expected {wanted:?}, got {actual:?}",
                        vector.id, vector.source
                    )
                })
            })
            .collect();
        assert!(failures.is_empty(), "{}", failures.join("\n"));
    }

    #[test]
    fn every_entry_checked_by_vectors_has_one() {
        let exercised: BTreeSet<usize> = vectors()
            .iter()
            .flat_map(|vector| vector.entries.iter())
            .map(|name| behaviour_id(name).into_usize())
            .collect();
        for (id, behaviour) in catalogue() {
            if behaviour.checked == Checked::ByVectors {
                assert!(exercised.contains(&id.into_usize()), "no vector for {id:?}");
            }
        }
    }

    #[test]
    fn power_on_begins_before_the_logo() {
        assert_eq!(
            PowerOnState::AT_POWER_ON,
            PowerOnState {
                stage: PowerOnStage::BeforeLogo,
                screen: ScreenLight::Blank(Level::FULL),
                frames: FinishedFrames::ZERO,
            }
        );
    }
}
