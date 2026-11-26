//! The intro's work frame and the playable room it schedules between its stage and scripts.

use super::{
    ActorFile, Actors, ActorsContent, Answers, Content, Fault, Pressed, Screen, Sound, Stepped,
    Work,
};
use crate::production::played::intro::{self, State};
use screens_intro::{FightReply, FlashbackRequest, IntroEvent, Report, Rules, TrackReply};
use types::Parity;
use types::intro::IntroStage;
use types::options::Settings;

/// Progress of an interrupted intro frame, supplied by the platform timing owner.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum IntroTiming {
    /// No certified playable-room phase.
    #[default]
    Unspecified,
    /// The stage is setting up the room and its first palette, ahead of actor work.
    SettingUpFight,
    /// The last outbound dissolve is clearing text, ahead of actor work.
    ClearingText,
    /// The return stage is preparing its first palette, ahead of actor work.
    PreparingReturn,
    /// The engine is finalising sprite attributes after scene work and both weapon draws.
    FinalisingSprites,
}

/// The scene, live actor and work already begun at an interrupted cut.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct PlayingIntro {
    /// Scene state and displayed colours as the last completed frame left them.
    pub scene: State,
    /// The actor and the playable scenes.
    pub actors: Actors,
    pending: Option<Pending>,
}

#[derive(Clone, Debug, PartialEq, Eq)]
enum Pending {
    /// The opening installed the actor; the stage and sound publications complete after the load.
    Opening(Box<(State, Report)>),
    /// Room installation is visible, but its actor turn and the scene scripts have yet to finish.
    Fight { pressed: Pressed, track: TrackReply },
    /// Room and scene work is complete. The platform still owes this frame its commit.
    SpriteCommit,
    /// The frame was interrupted inside its stage, before any room turn.
    BeforeRoom { pressed: Pressed, track: TrackReply },
}

impl PlayingIntro {
    pub(super) fn begin(scene: State, file: ActorFile, settings: Settings) -> Self {
        Self {
            scene,
            actors: Actors::waiting(file, settings),
            pending: None,
        }
    }

    fn begin_opening(
        &mut self,
        pressed: Pressed,
        track: TrackReply,
        content: &Content,
        rules: Rules,
    ) -> Result<(), intro::Fault> {
        let mut scene = self.scene.clone();
        let report = intro::play_work_frame(
            &mut scene,
            pressed.values().any(|&down| down),
            track,
            &content.intro,
            rules,
            |_| Ok(FightReply::Fighting),
        )?;
        self.equipment(&report)?;
        self.pending = Some(Pending::Opening(Box::new((scene, report))));
        Ok(())
    }

    fn equipment(&mut self, report: &Report) -> Result<(), intro::Fault> {
        for event in &report.events {
            if let IntroEvent::EquipmentSet(equipment) = event {
                self.actors.equip(*equipment)?;
            }
        }
        Ok(())
    }

    fn interrupt(
        &mut self,
        pressed: Pressed,
        track: TrackReply,
        content: &Content,
        rules: Rules,
        timing: IntroTiming,
    ) -> Result<(), intro::Fault> {
        if self.pending.is_some() {
            return Ok(());
        }
        if self.scene.intro.stage == IntroStage::Begins {
            return self.begin_opening(pressed, track, content, rules);
        }
        if self.scene.intro.stage == IntroStage::PageOneWaits && pressed.values().any(|&down| down)
        {
            // Apply the room install before the NMI. Retain the frame inputs for the
            // scripts and the room turn when the same frame finishes.
            let mut preview = self.scene.intro.clone();
            let frame =
                screens_intro::begin_frame(&mut preview, true, track, &content.intro, rules);
            if frame
                .requests()
                .any(|request| request == FlashbackRequest::StartZebesFight)
            {
                if timing != IntroTiming::SettingUpFight {
                    return Err(intro::Fault::IntroTiming);
                }
                self.actors
                    .start(frame.state().height, actors_content(content)?)?;
                self.pending = Some(Pending::Fight { pressed, track });
            }
        }
        Ok(())
    }

    fn finish(
        &mut self,
        pressed: Pressed,
        track: TrackReply,
        count: u16,
        content: &Content,
        rules: Rules,
        waiting: samus_appearance::SoundsWaiting,
    ) -> Result<Report, intro::Fault> {
        let (pressed, track, installed) = match self.pending.take() {
            Some(Pending::Opening(completed)) => {
                let (scene, report) = *completed;
                self.scene = scene;
                return Ok(report);
            }
            Some(Pending::Fight { pressed, track }) => (pressed, track, true),
            Some(Pending::BeforeRoom { pressed, track }) => (pressed, track, false),
            Some(Pending::SpriteCommit) => return Err(intro::Fault::IntroTiming),
            None => (pressed, track, false),
        };
        let actors = &mut self.actors;
        let mut flash = None;
        let report = intro::play_work_frame(
            &mut self.scene,
            pressed.values().any(|&down| down),
            track,
            &content.intro,
            rules,
            |frame| {
                if !installed
                    && frame
                        .requests()
                        .any(|request| request == FlashbackRequest::StartZebesFight)
                {
                    actors.start(frame.state().height, actors_content(content)?)?;
                }
                if actors.battle.is_none() {
                    return Ok(FightReply::Fighting);
                }
                let parity = if count & 1 == 0 {
                    Parity::Even
                } else {
                    Parity::Odd
                };
                let (reply, colours) =
                    actors.turn(frame, parity, pressed, waiting, actors_content(content)?)?;
                flash = colours;
                Ok(reply)
            },
        )?;
        if let Some(flash) = flash {
            super::super::flashback::publish_flash(&mut self.scene.colours, flash);
        }
        self.equipment(&report)?;
        Ok(report)
    }
}

fn actors_content(content: &Content) -> Result<&ActorsContent, intro::Fault> {
    content.actors.as_ref().ok_or(intro::Fault::ZebesFightRoom)
}

pub(super) fn step(
    mut state: PlayingIntro,
    work: Work,
    count: u16,
    rules: Rules,
    content: &Content,
    answers: &impl Answers,
) -> Result<Stepped, Fault> {
    let mut drawn = work.finished();
    let report = match work {
        Work::Unfinished if state.pending.is_some() => None,
        Work::Unfinished if state.actors.battle.is_some() => {
            let pressed = answers.pending_buttons();
            let track = answers.track();
            match answers.intro_timing() {
                IntroTiming::FinalisingSprites => {
                    let report = state
                        .finish(
                            pressed,
                            track,
                            count,
                            content,
                            rules,
                            answers.sounds_waiting(),
                        )
                        .map_err(Fault::Intro)?;
                    state.pending = Some(Pending::SpriteCommit);
                    drawn = 1;
                    Some(report)
                }
                IntroTiming::ClearingText | IntroTiming::PreparingReturn => {
                    state.pending = Some(Pending::BeforeRoom { pressed, track });
                    drawn = 1;
                    None
                }
                IntroTiming::SettingUpFight | IntroTiming::Unspecified => {
                    return Err(Fault::Intro(intro::Fault::IntroTiming));
                }
            }
        }
        Work::Unfinished => {
            state
                .interrupt(
                    answers.pending_buttons(),
                    answers.track(),
                    content,
                    rules,
                    answers.intro_timing(),
                )
                .map_err(Fault::Intro)?;
            drawn = u16::from(state.pending.is_some());
            None
        }
        Work::Finished(_) if state.pending == Some(Pending::SpriteCommit) => {
            state.pending = None;
            drawn = 0;
            None
        }
        Work::Finished(pressed) => {
            drawn = u16::from(state.pending.is_none());
            Some(
                state
                    .finish(
                        pressed,
                        answers.track(),
                        count,
                        content,
                        rules,
                        answers.sounds_waiting(),
                    )
                    .map_err(Fault::Intro)?,
            )
        }
        Work::FinishedTwice(_) => return Err(Fault::TwoFinished),
    };
    let sounds = report.map_or_else(Vec::new, |report| {
        let clicks = report
            .events
            .iter()
            .filter(|&&event| event == IntroEvent::Click)
            .map(|_| Sound::IntroClick);
        report
            .music
            .iter()
            .map(|&request| Sound::IntroMusic(request))
            .chain(clicks)
            .collect()
    });
    let finished = work.finished();
    Ok(Stepped {
        screen: Screen::Intro(Box::new(state)),
        finished,
        drawn,
        sounds,
    })
}
