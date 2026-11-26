//! The intro's work frame and the playable room it schedules between its stage and scripts.

use super::{
    ActorFile, Actors, ActorsContent, Answers, Content, Fault, Screen, Sound, Stepped, Work,
};
use crate::production::played::intro::{self, State};
use screens_intro::{FightReply, FlashbackRequest, IntroEvent, Report, Rules, TrackReply};
use types::Parity;
use types::intro::IntroStage;
use types::options::Settings;

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
    Fight { pressed: bool, track: TrackReply },
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
        pressed: bool,
        track: TrackReply,
        content: &Content,
        rules: Rules,
    ) -> Result<(), intro::Fault> {
        let mut scene = self.scene.clone();
        let report =
            intro::play_work_frame(&mut scene, pressed, track, &content.intro, rules, |_| {
                Ok(FightReply::Fighting)
            })?;
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
        pressed: bool,
        track: TrackReply,
        content: &Content,
        rules: Rules,
    ) -> Result<(), intro::Fault> {
        if self.pending.is_some() {
            return Ok(());
        }
        if self.scene.intro.stage == IntroStage::Begins {
            return self.begin_opening(pressed, track, content, rules);
        }
        if self.scene.intro.stage == IntroStage::PageOneWaits && pressed {
            // Apply the room install before the NMI. Retain the frame inputs for the
            // scripts and the room turn when the same frame finishes.
            let mut preview = self.scene.intro.clone();
            let frame =
                screens_intro::begin_frame(&mut preview, pressed, track, &content.intro, rules);
            if frame
                .requests()
                .any(|request| request == FlashbackRequest::StartZebesFight)
            {
                self.actors
                    .start(frame.state().height, actors_content(content)?)?;
                self.pending = Some(Pending::Fight { pressed, track });
            }
        }
        Ok(())
    }

    fn finish(
        &mut self,
        pressed: bool,
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
            None => (pressed, track, false),
        };
        let actors = &mut self.actors;
        let report = intro::play_work_frame(
            &mut self.scene,
            pressed,
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
                actors.turn(frame, parity, waiting, actors_content(content)?)
            },
        )?;
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
    let report = match work {
        Work::Unfinished => {
            state
                .interrupt(answers.pending_press(), answers.track(), content, rules)
                .map_err(Fault::Intro)?;
            None
        }
        Work::Finished(pressed) => Some(
            state
                .finish(
                    pressed.values().any(|&down| down),
                    answers.track(),
                    count,
                    content,
                    rules,
                    answers.sounds_waiting(),
                )
                .map_err(Fault::Intro)?,
        ),
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
        drawn: finished,
        sounds,
    })
}
