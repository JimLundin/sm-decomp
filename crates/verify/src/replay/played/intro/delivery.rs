//! Observations for page 2's press through the beginning of the examination.

use super::{
    BABY_COUNT, BABY_LIST, BABY_TIMER, BABY_X, BABY_Y, Baby, BabyScript, IntroStage, Outcome, Pair,
    Pan, Progress, Reading, ScriptPlace, Wrapping, data_failure, source, step_at, word, wram,
};

const SCENE_TILEMAP: super::WramWord = wram("CINEMATIC_SCENE_TILEMAP", 0x58);
const HORIZONTAL: super::WramWord = wram("CINEMATIC_BG1_X", 0x1993);

pub(super) fn stage(wram: &[u8]) -> Result<IntroStage, Outcome> {
    use IntroStage::{
        Delivery, Discovery, IntoDelivery, IntoDiscovery, OutOfDelivery, OutOfDiscovery,
        PageFourPending, PageFourWaits, PageThreePending, PageThreeWaits, PageTwoWaits, Text,
        ToScene,
    };
    let background = word(wram, SCENE_TILEMAP)? & 0xff;
    Ok(match word(wram, source::CINEMATIC_ACTIVE)? {
        0xaf6c => PageTwoWaits,
        0xb250 => IntoDiscovery,
        0xb370 => PageThreePending,
        0xb3f4 => OutOfDiscovery,
        0xb0f2 => PageThreeWaits,
        0xb381 => PageFourPending,
        0xb458 => OutOfDelivery,
        0xb123 => PageFourWaits,
        0xb2d2 if background == 0x58 => IntoDelivery,
        0xb2d2 => ToScene,
        0xa391 if word(wram, super::TO_TEXT_COUNT)? == u16::MAX => Text,
        0xa391 if background == 0x58 => Delivery,
        0xa391 => Discovery,
        _ => {
            return Err(data_failure(
                None,
                "played intro discovery and delivery stage",
            ));
        }
    })
}

pub(super) const fn return_count(stage: IntroStage) -> bool {
    matches!(
        stage,
        IntroStage::PageTwoWaits
            | IntroStage::OutOfDiscovery
            | IntroStage::PageThreeWaits
            | IntroStage::OutOfDelivery
            | IntroStage::PageFourWaits
            | IntroStage::Text
    )
}

pub(super) fn pan(wram: &[u8]) -> Result<Pan, Outcome> {
    Pan::new(word(wram, HORIZONTAL)?).ok_or_else(|| data_failure(None, "played intro pan"))
}

pub(super) fn baby(
    reading: &Reading,
    wram: &[u8],
    stage: IntroStage,
) -> Result<Option<Baby>, Outcome> {
    if !matches!(
        stage,
        IntroStage::IntoDelivery
            | IntroStage::Delivery
            | IntroStage::PageFourPending
            | IntroStage::OutOfDelivery
            | IntroStage::ToScene
    ) {
        return Ok(None);
    }
    let list = word(wram, BABY_LIST)?;
    if list == 0 {
        return Ok(None);
    }
    let (script, places) = if stage == IntroStage::ToScene {
        (BabyScript::Examination, &reading.places.baby)
    } else {
        (BabyScript::Delivery, &reading.places.delivery)
    };
    let count = word(wram, BABY_COUNT)?;
    Ok(Some(Baby {
        place: Pair {
            x: Wrapping(word(wram, BABY_X)?),
            y: Wrapping(word(wram, BABY_Y)?),
        },
        script: ScriptPlace {
            script,
            progress: Progress {
                step: step_at(places, list)?,
                hold: word(wram, BABY_TIMER)?,
                repeat_left: (count > 0 && count < 10).then_some(count),
            },
        },
    }))
}

use super::{
    Controller, FrameEnd, Part, Scenes, View, brightness, brightness_of, event_observation,
    finished, frame_input, intro, observations, polled, published_palette, shown,
};
use crate::replay::compare::Observation;
use crate::replay::index::{Case, Start};
use crate::replay::played::{open_rows, read_lock};
use crate::replay::readers::ExpectedCut;
use crate::replay::result::CaseResult;
use crate::replay::{Sources, compared, runner::record_cut};
use game::production::played::intro::{EncounterReplies, FightInput};
use oracle_record::FrameRecord;
use screens_intro::{
    DiscoveryReply, DiscoveryRequest, IntroEvent, Language, PictureLayout, Report, ScenePicture,
    TextTiles,
};

const EGG_SCRIPT: super::WramWord = wram("DISCOVERY_EGG_INSTRUCTION", 0x1b39);
const EGG_HOLD: super::WramWord = wram("DISCOVERY_EGG_HOLD", 0x1b79);

fn reply(began: &FrameRecord, current: &FrameRecord) -> Result<EncounterReplies, Outcome> {
    let mut replies = EncounterReplies::QUIET;
    if word(&began.wram, EGG_SCRIPT)? == 0xcb91
        && word(&began.wram, EGG_HOLD)? == 1
        && word(&current.wram, EGG_SCRIPT)? == 0xcb9b
    {
        replies.discovery = DiscoveryReply::ReadyForPageThree;
    }
    Ok(replies)
}

const fn discovery_start() -> [IntroEvent; 3] {
    [
        IntroEvent::PictureSelected(ScenePicture::Discovery),
        IntroEvent::Layout(PictureLayout::SplitBlend),
        IntroEvent::Discovery(DiscoveryRequest::StartDiscovery),
    ]
}

fn events(before: (IntroStage, u16), stage: IntroStage) -> (Vec<IntroEvent>, bool) {
    use IntroStage::{
        Delivery, Discovery, IntoDelivery, IntoDiscovery, OutOfDelivery, OutOfDiscovery,
        PageFourPending, PageFourWaits, PageThreePending, PageThreeWaits, PageTwoWaits, Text,
        ToScene,
    };
    use PictureLayout::{PlainScene, PlainText, SplitBlend};
    let (from, count) = before;
    let step_colour = matches!(
        from,
        IntoDiscovery | IntoDelivery | OutOfDiscovery | OutOfDelivery
    ) && count % 4 == 0;
    let events = match (from, stage) {
        (PageTwoWaits, IntoDiscovery) => discovery_start().to_vec(),
        (IntoDiscovery, Discovery) | (IntoDelivery, Delivery) => {
            vec![IntroEvent::TextCleared, IntroEvent::Layout(PlainScene)]
        }
        (Discovery, PageThreePending) | (Delivery, PageFourPending) => {
            vec![IntroEvent::TextUpdates(TextTiles::Paused)]
        }
        (PageThreePending, OutOfDiscovery) | (PageFourPending, OutOfDelivery) => vec![
            IntroEvent::Layout(SplitBlend),
            IntroEvent::TextUpdates(TextTiles::Resumed),
        ],
        (OutOfDiscovery, OutOfDiscovery) if count == 1 => {
            vec![IntroEvent::Discovery(DiscoveryRequest::FinishDiscovery)]
        }
        (OutOfDiscovery | OutOfDelivery, Text) => vec![IntroEvent::Layout(PlainText)],
        (PageThreeWaits, IntoDelivery) => vec![
            IntroEvent::PictureSelected(ScenePicture::Delivery),
            IntroEvent::Layout(SplitBlend),
        ],
        (PageFourWaits, ToScene) => vec![
            IntroEvent::PictureSelected(ScenePicture::Examination),
            IntroEvent::Layout(SplitBlend),
        ],
        _ => Vec::new(),
    };
    let publishes = step_colour
        || matches!(
            (from, stage),
            (PageTwoWaits, IntoDiscovery)
                | (PageThreePending, OutOfDiscovery)
                | (PageThreeWaits, IntoDelivery)
                | (PageFourPending, OutOfDelivery)
                | (PageFourWaits, ToScene)
        );
    (events, publishes)
}

/// The original layer, tilemap and text-write mirrors corroborate the event projection.
fn confirm_publications(events: &[IntroEvent], wram: &[u8]) -> Result<(), Outcome> {
    const MAIN: usize = 0x69;
    const SUB: usize = 0x6b;
    const TEXT_UPDATES: super::WramWord = super::wram("CINEMATIC_TEXT_UPDATE", 0x19f3);
    for event in events {
        let holds = match *event {
            IntroEvent::Layout(layout) => {
                let (main, sub) = match layout {
                    PictureLayout::SplitBlend => (6, 17),
                    PictureLayout::PlainScene | PictureLayout::PlainFight => (21, 0),
                    PictureLayout::PlainText => (22, 0),
                };
                (wram.get(MAIN), wram.get(SUB)) == (Some(&main), Some(&sub))
            }
            IntroEvent::PictureSelected(scene) => {
                let selected = match scene {
                    ScenePicture::Discovery => 0x54,
                    ScenePicture::Delivery => 0x58,
                    ScenePicture::Examination => 0x5c,
                };
                word(wram, SCENE_TILEMAP)? & 0xff == selected
            }
            IntroEvent::TextUpdates(tiles) => {
                (word(wram, TEXT_UPDATES)? & 0x8000 != 0) == (tiles == TextTiles::Resumed)
            }
            IntroEvent::Discovery(DiscoveryRequest::StartDiscovery) => {
                word(wram, super::FIGHT_VISIBILITY)? != 0
            }
            IntroEvent::Discovery(DiscoveryRequest::FinishDiscovery) => {
                word(wram, super::FIGHT_VISIBILITY)? == 0
            }
            IntroEvent::TextCleared
            | IntroEvent::Click
            | IntroEvent::EquipmentSet(_)
            | IntroEvent::Flashback(_)
            | IntroEvent::FlashingLightsBegin => true,
        };
        if !holds {
            return Err(data_failure(None, "delivery publication mirror"));
        }
    }
    Ok(())
}

fn publications(events: &[IntroEvent], palette: Option<&types::Palette>) -> Vec<Observation> {
    let mut result = vec![
        event_observation(events),
        Observation::quantity("intro.palette-published", i64::from(palette.is_some())),
    ];
    if let Some(palette) = palette {
        result.extend([
            published_palette(palette, false),
            published_palette(palette, true),
        ]);
    }
    result
}

/// Run the delivery scope with the same production adapter used by the normal intro caller.
/// D0 publishes only stage work. D1 consumes its pending room turn.
pub(in crate::replay) fn run(
    case: &Case,
    sources: &Sources,
    result: &mut CaseResult,
) -> Result<(), Outcome> {
    let Start::CacheCut { frame: start } = case.start else {
        return Err(data_failure(None, "delivery seed"));
    };
    let lock = read_lock(sources)?;
    let mut rows = open_rows(sources, &lock, &case.row)?;
    let mut previous = rows.record(start)?;
    let Scenes {
        content,
        mut reading,
    } = Scenes::import(&sources.rom, Part::Delivery)?;
    if !finished(&previous) || word(&previous.wram, super::LANGUAGE)? != 0 {
        return Err(data_failure(
            Some(start),
            "English intro seed at finished frame",
        ));
    }
    let seed = reading.state(&previous.wram)?;
    if seed.stage != IntroStage::PageTwoWaits {
        return Err(data_failure(Some(start), "intro seed waits after page 2"));
    }
    let state = intro::admit(Language::English, seed, shown(&previous.wram)?)
        .map_err(|_| data_failure(Some(start), "delivery admission"))?;
    let mut null = observations(&View {
        part: Part::Delivery,
        state: &state.intro,
        colours: &state.colours,
    });
    null.extend(publications(&[], None));
    let mut runner = SceneRun::Ready(state);
    let mut controller = Controller::seeded(&previous.wram)?;
    let mut began = previous.clone();
    let mut digest = Vec::new();
    let first = start.saturating_add(1);
    let outcome = (|| {
        for cut in first..=case.end.cut_inclusive {
            let current = rows.record(cut)?;
            if current.video_frame != previous.video_frame.wrapping_add(1) {
                return Err(data_failure(Some(cut), "delivery row join"));
            }
            let partial = cut == first;
            let resumed = matches!(runner, SceneRun::Pending(_));
            let (next, output) = runner
                .advance(&content, &began, &previous, &current, controller, partial)
                .map_err(|outcome| crate::replay::played::at(cut, outcome))?;
            runner = next;
            let (expected, actual) = project(
                &mut reading,
                &began,
                &current,
                runner.state(),
                output.as_ref(),
                partial,
                resumed,
            )
            .map_err(|outcome| crate::replay::played::at(cut, outcome))?;
            compared(cut, &expected.fields, &actual)?;
            record_cut(result, &mut digest, &expected, &null, cut)?;
            result.counts.pending_boundaries = result
                .counts
                .pending_boundaries
                .saturating_add(usize::from(partial));
            result.counts.pending_restorations = result
                .counts
                .pending_restorations
                .saturating_add(usize::from(resumed));
            controller = controller.read(current.pad.0, polled(&current));
            if finished(&current) {
                began = current.clone();
            }
            previous = current;
        }
        Ok(())
    })();
    if !digest.is_empty() {
        result.observation_sha256 = crate::lock::Sha256::of(&digest).to_string();
    }
    outcome
}

enum SceneRun {
    Ready(intro::State),
    Pending(intro::PendingRoom),
}

impl SceneRun {
    const fn state(&self) -> &intro::State {
        match self {
            Self::Ready(state) => state,
            Self::Pending(room) => room.state(),
        }
    }

    fn advance(
        self,
        content: &screens_intro::IntroContent,
        began: &FrameRecord,
        previous: &FrameRecord,
        current: &FrameRecord,
        controller: Controller,
        partial: bool,
    ) -> Result<(Self, Option<Report>), Outcome> {
        let fault = |fault| Outcome::Unsupported {
            cut: None,
            dependency: format!("intro frame {fault:?}"),
        };
        match self {
            Self::Ready(state) if partial => {
                if finished(current) {
                    return Err(data_failure(None, "discovery start interruption"));
                }
                let (pending, report) = intro::interrupt_work_frame(
                    state,
                    controller.pressed != 0,
                    screens_intro::TrackReply::NoneWaits,
                    content,
                    super::RULES,
                )
                .map_err(fault)?;
                Ok((Self::Pending(pending), Some(report)))
            }
            Self::Pending(room) => {
                if !finished(current) {
                    return Err(data_failure(None, "discovery room turn completion"));
                }
                let saved = toml::to_string(&room.checkpoint())
                    .map_err(|_| data_failure(None, "discovery checkpoint encoding"))?;
                let decoded = toml::from_str(&saved)
                    .map_err(|_| data_failure(None, "discovery checkpoint decoding"))?;
                let replies = reply(began, current)?;
                let moved = room.resume(content, super::RULES, replies);
                let restored =
                    intro::PendingRoom::restore(decoded).resume(content, super::RULES, replies);
                if restored != moved {
                    return Err(Outcome::CheckpointFailure { cut: 1298 });
                }
                let (next, report) = restored;
                Ok((Self::Ready(next), Some(report)))
            }
            Self::Ready(mut state) => {
                let frame = frame_input(Part::Delivery, controller, began, previous, current)?;
                let frame = if let FrameEnd::Finished { pressed, track, .. } = frame {
                    FrameEnd::Finished {
                        pressed,
                        track,
                        fight: FightInput::ScopedScene(reply(began, current)?),
                    }
                } else {
                    frame
                };
                let output =
                    intro::played(&mut state, frame, content, super::RULES).map_err(fault)?;
                Ok((Self::Ready(state), output))
            }
        }
    }
}

fn project(
    reading: &mut Reading,
    began: &FrameRecord,
    current: &FrameRecord,
    actual: &intro::State,
    report: Option<&Report>,
    partial: bool,
    resumed: bool,
) -> Result<(ExpectedCut, Vec<Observation>), Outcome> {
    if !partial && !finished(current) {
        return Ok((
            expected_cut(brightness(brightness_of(&current.wram)?), Vec::new()),
            brightness(actual.intro.brightness),
        ));
    }
    let prior = reading
        .previous_flashback
        .ok_or_else(|| data_failure(None, "delivery preceding stage"))?;
    let expected = reading.state(&current.wram)?;
    let palette = shown(&current.wram)?;
    let semantics = observations(&View {
        part: Part::Delivery,
        state: &expected,
        colours: &palette,
    });
    let mut actual_fields = observations(&View {
        part: Part::Delivery,
        state: &actual.intro,
        colours: &actual.colours,
    });
    let report = report.ok_or_else(|| data_failure(None, "delivery frame report"))?;
    let (events, palette_published) = if resumed {
        (Vec::new(), false)
    } else {
        events(prior, expected.stage)
    };
    confirm_publications(&events, &current.wram)?;
    let mut expected_outputs = publications(&events, palette_published.then_some(&palette));
    let mut actual_outputs = publications(&report.events, report.colours.as_ref());
    if matches!(
        expected.stage,
        IntroStage::OutOfDiscovery
            | IntroStage::PageThreeWaits
            | IntroStage::OutOfDelivery
            | IntroStage::PageFourWaits
            | IntroStage::Text
    ) {
        expected_outputs.push(Observation::quantity(
            "intro.clicks",
            i64::from(super::clicks(began, current)?),
        ));
        actual_outputs.push(Observation::quantity(
            "intro.clicks",
            super::count(report, IntroEvent::Click),
        ));
    }
    actual_fields.extend(actual_outputs);
    Ok((expected_cut(semantics, expected_outputs), actual_fields))
}

fn expected_cut(semantics: Vec<Observation>, outputs: Vec<Observation>) -> ExpectedCut {
    ExpectedCut {
        semantic_count: semantics.len(),
        output_count: outputs.len(),
        replies: 0,
        fields: semantics.iter().cloned().chain(outputs).collect(),
        semantics,
    }
}
