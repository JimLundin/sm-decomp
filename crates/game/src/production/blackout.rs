//! The Ceres elevator's scheduler, the interrupted room picture and its two-stage cartridge save.
mod body;
mod picture;
use super::save_files;
use crate::exact::save_files::Format;
use crate::samus_appearance_bridge::elevator::Art as StandingArt;
use crate::state::video::{Colour, Oam, StandingStrips, Video};
pub use body::StandingFrame;
use canonical::{
    Canonical,
    model::{Model, Pinned},
};
use data::assets::countdown::Figures;
use interface_hud::{Countdown, CountdownInputs, CountdownPicture, CountdownRuns, FinishedFrames};
pub use picture::Platform;
use screens_blackout::{Opportunity, Reach, RoomPass};
pub use screens_blackout::{Request, SoundChannel};
use serde::{Deserialize, Serialize};
use types::ceres_blackout::{
    BlackoutLight, BlackoutPhase, BlackoutState, FlightRoute, ShaftStage, TimerActivity,
    TimerCentiseconds, WorkPass,
};
use types::save_files::{ExploredCells, PendingAction, RunSnapshot};
use types::{Area, StationId};

/// The immutable timer art and the cartridge format.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Content {
    /// The timer's frame and digits.
    pub figures: Figures,
    /// ROM banks referenced by the room's regular VRAM queue.
    pub graphics: data::pose::tiles::Graphics,
    /// The selected cartridge format.
    pub format: Format,
    /// Imported body art when the selected scope includes Samus's standing picture.
    pub standing: Option<StandingArt>,
}

/// Owners carried from the shaft. The HUD and save files retain their own complete state.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct Owners {
    /// Blackout rule state, including the queued picture identities.
    pub scene: BlackoutState,
    /// Running HUD timer, retired when the cartridge completes.
    pub countdown: Countdown,
    /// Cartridge, active slot and pending write.
    pub files: save_files::State,
    /// The live profile, map history and progress to capture.
    pub run: RunSnapshot,
    /// The actual explored map in the room.
    pub live_map: ExploredCells,
    /// Area and station of the running room.
    pub location: (Area, StationId),
    /// Actual current Health.
    pub health: samus_health::State,
    /// Room's shared RNG.
    pub random: types::RandomSequence,
}

/// Wire representation of the variable-length cartridge and run owners.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct OwnersPin;
impl Pinned for OwnersPin {
    type State = Owners;
    const NAME: &'static str = "ceres-elevator-blackout-v1";
    // The wire test combines the traced scene with actual fixed-length cartridge owners.
    const SHAPE: u64 = 0x9BB6_E1C1_0FE4_AE6A;
}

/// The production continuation, including the prepared and published pictures.
#[derive(Canonical, Clone, Debug, PartialEq, Eq)]
pub struct State {
    /// Current collection of semantic owners.
    pub owners: Model<OwnersPin>,
    /// E2's pending batch and its live display.
    pub video: Box<Video>,
    /// A platform write to drain with the prepared room batch.
    pub platform: Option<Platform>,
}

/// Which continuation the interrupted main strand reached.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Work {
    /// A fresh room or scene opportunity. B8 supplies the elevator reach after motion.
    Fresh {
        /// The shaft reached the elevator during this turn.
        reached: bool,
    },
    /// No work supplied by the scheduler.
    Interrupted,
    /// The shaft may prepare its display and begin writing.
    VerticalEnded,
    /// The cartridge can publish the checks on its pending payload.
    SaveCompleted,
}

/// Scheduling stimuli. The shaft supplies its reach once, after motion.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Cut {
    /// Work the scheduler admits.
    pub work: Work,
    /// The display services the requested NMI.
    pub publish: bool,
    /// Drawing inputs supplied by Movement, her clip, the shaft and the camera.
    pub standing: Option<StandingFrame>,
}

/// First new opportunity after publication. The caller transfers these live owners to the station.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Dispatch {
    /// Flight selected by the completed save.
    pub route: FlightRoute,
    /// Finished shaft work count on entry.
    pub finished: u16,
    /// Live Health and its capacity.
    pub health: samus_health::State,
    /// Maximum energy.
    pub maximum: u16,
    /// Whether to display Japanese subtitles.
    pub japanese_subtitles: bool,
    /// Last published palette.
    pub palette: [Colour; 256],
    /// Last published sprites.
    pub oam: Oam,
}

/// Ordered owner intents and the one-shot receiving scene.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct Output {
    /// Lock, room, audio and handoff requests in production order.
    pub requests: Vec<Request>,
    /// Consumed only on a fresh work opportunity.
    pub dispatch: Option<Box<Dispatch>>,
    /// Samus's tile sources that video serviced on this cut.
    pub standing_publication: StandingStrips,
}

/// The selected scope cannot consume this continuation or asset.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Fault {
    /// Current phase and its consumers disagree.
    State,
    /// Timer cannot advance or draw.
    Timer,
    /// Unavailable room graphics.
    Picture,
    /// The draw opportunity and its pending resources disagree.
    StandingPicture,
    /// The current pose is outside the standing drawing.
    StandingPose,
    /// The admitted facing and clip step have no imported body art.
    StandingArt,
    /// Cartridge refused the request.
    Save(::save_files::Refusal),
}

impl State {
    pub(super) fn output(
        &mut self,
        content: &Content,
        cut: Cut,
    ) -> Result<super::ProductionOutputs, super::ProductionFault> {
        self.step(cut, content)
            .map(|out| super::ProductionOutputs::Blackout(Box::new(out)))
            .map_err(|why| super::ProductionFault::Unsupported(super::Unsupported::Blackout(why)))
    }

    pub(super) fn validate(&self) -> Result<(), Fault> {
        let o = &self.owners.0;
        let s = &o.scene;
        let prepared = matches!(s.pass, WorkPass::Prepared(_));
        let retired = matches!(
            s.phase,
            BlackoutPhase::Explosion(_) | BlackoutPhase::Dispatched(_)
        );
        let waiting = matches!(
            s.phase,
            BlackoutPhase::Saving(_) | BlackoutPhase::AwaitBlank
        );
        let saving = matches!(s.phase, BlackoutPhase::Saving(_));
        if !prepared && (waiting || self.platform.is_some())
            || matches!(s.phase, BlackoutPhase::Dispatched(_)) && prepared
            || (waiting || retired) != (s.light == BlackoutLight::ForcedBlank)
            || retired != (s.timer.activity == TimerActivity::Retired)
            || retired != (s.shaft == ShaftStage::Cleared)
            || retired != matches!(o.countdown, Countdown::Dormant)
            || saving != matches!(o.files.cartridge.pending, PendingAction::Write { .. })
            || retired && !prepared && s.published.light != BlackoutLight::ForcedBlank
            || self.video.memory.vram.len() != crate::state::video::VRAM_WORDS
        {
            return Err(Fault::State);
        }
        Ok(())
    }

    /// Run the selected work once, then publish only if the NMI owns a finished pass.
    ///
    /// # Errors
    /// An unavailable consumer leaves the prior continuation intact.
    pub fn advance(&mut self, cut: Cut, content: &Content) -> Result<Output, Fault> {
        let mut next = self.clone();
        let output = next.step(cut, content)?;
        *self = next;
        Ok(output)
    }

    pub(super) fn step(&mut self, cut: Cut, content: &Content) -> Result<Output, Fault> {
        let o = &mut self.owners.0;
        let fresh_locked = matches!(cut.work, Work::Fresh { reached: false })
            && matches!(o.scene.phase, BlackoutPhase::Hold(_) | BlackoutPhase::Fade)
            && matches!(o.scene.pass, WorkPass::Finished);
        let draw_body = fresh_locked && content.standing.is_some();
        if draw_body != cut.standing.is_some() {
            return Err(Fault::StandingPicture);
        }
        let opportunity = match cut.work {
            Work::Fresh { reached } => match o.scene.phase {
                BlackoutPhase::Shaft | BlackoutPhase::Hold(_) | BlackoutPhase::Fade
                    if matches!(o.scene.pass, WorkPass::Finished) =>
                {
                    let opportunity = room(o, &mut self.video, reached, content)?;
                    if let Some((standing, art)) = cut.standing.zip(content.standing.as_ref()) {
                        body::prepare(&mut self.video.batch, standing, art)?;
                    }
                    opportunity
                }
                BlackoutPhase::Explosion(_) | BlackoutPhase::Dispatched(_) => Opportunity::NewWork,
                BlackoutPhase::Shaft
                | BlackoutPhase::Hold(_)
                | BlackoutPhase::Fade
                | BlackoutPhase::AwaitBlank
                | BlackoutPhase::Saving(_) => Opportunity::Interrupted,
            },
            Work::VerticalEnded => Opportunity::VerticalBlankEnded,
            Work::SaveCompleted if matches!(o.scene.phase, BlackoutPhase::Saving(_)) => {
                save_files::advance(
                    &mut o.files.cartridge,
                    &content.format,
                    save_files::Input::FinishWrite,
                )
                .map_err(Fault::Save)?;
                Opportunity::SaveCompleted
            }
            Work::SaveCompleted | Work::Interrupted => Opportunity::Interrupted,
        };
        let requests = screens_blackout::advance(&mut o.scene, opportunity);
        let mut dispatch = None;
        for &request in &requests {
            match request {
                Request::PrepareSpriteOnly => {
                    picture::sprites_only(&mut self.video.batch.registers);
                }
                Request::CaptureEscapeSave(FlightRoute::ZebesFlight) => {
                    let (area, station) = o.location;
                    o.run.profile.energy.current = o.health.energy.whole.get();
                    save_files::advance(
                        &mut o.files.cartridge,
                        &content.format,
                        save_files::Input::Write {
                            cut: ::save_files::SaveCut::CeresEscape { area, station },
                            run: &o.run,
                            live_map: &o.live_map,
                            stage: ::save_files::WriteStage::PayloadWritten,
                        },
                    )
                    .map_err(Fault::Save)?;
                }
                Request::PrepareStationExplosion(_) => o.countdown = Countdown::Dormant,
                Request::DispatchExplosion(route) => {
                    dispatch = Some(Box::new(Dispatch {
                        route,
                        finished: o.scene.finished.0,
                        health: o.health,
                        maximum: o.run.profile.energy.maximum,
                        japanese_subtitles: o.run.profile.settings.language
                            == types::options::Language::Japanese,
                        palette: self.video.memory.cgram,
                        oam: self.video.memory.oam,
                    }));
                }
                Request::ContinueRoom
                | Request::StandAndLock
                | Request::DisableShaftInterrupts
                | Request::StopMusic(_)
                | Request::Silence(_) => {}
            }
        }
        picture::light(&mut self.video.batch.registers, o.scene.light);
        let standing_publication = self.publish(cut, content)?;
        Ok(Output {
            requests,
            dispatch,
            standing_publication,
        })
    }

    fn publish(&mut self, cut: Cut, content: &Content) -> Result<StandingStrips, Fault> {
        let mut serviced = StandingStrips::default();
        let queued = self.video.batch.standing_strips;
        let scene = &mut self.owners.0.scene;
        if cut.publish && matches!(scene.pass, WorkPass::Prepared(_)) {
            screens_blackout::advance(scene, Opportunity::Publication);
            if matches!(scene.pass, WorkPass::Finished) {
                self.video.commit_registers();
                self.video.upload_oam_and_cgram();
                if queued != StandingStrips::default() {
                    let art = content
                        .standing
                        .as_ref()
                        .filter(|art| art.accepts(queued))
                        .ok_or(Fault::StandingPicture)?;
                    serviced = self.video.publish_standing_strips(&art.graphics);
                }
                self.video.process_vram_writes(&content.graphics);
                if let Some(platform) = self.platform.take() {
                    platform.publish(&mut self.video)?;
                }
            }
        }
        Ok(serviced)
    }
}

fn room(
    o: &mut Owners,
    video: &mut Video,
    reached: bool,
    content: &Content,
) -> Result<Opportunity, Fault> {
    let runs = if o.scene.phase == BlackoutPhase::Shaft {
        CountdownRuns::MaintainCountdown
    } else {
        CountdownRuns::PresentationOnly
    };
    let timer = o
        .countdown
        .play(CountdownInputs {
            runs,
            start: None,
            finished: FinishedFrames(o.scene.finished.0),
        })
        .map_err(|_| Fault::Timer)?
        .picture;
    let CountdownPicture::Shown { time, .. } = timer else {
        return Err(Fault::Timer);
    };
    let reach = if reached {
        Reach::Reached(TimerCentiseconds(time.hundredths()))
    } else {
        Reach::NotReached
    };
    let picture = picture::draw(&mut video.batch, &content.figures, timer)?;
    o.run.profile.play_time =
        game_time::Clock::restore(o.run.profile.play_time).advance(game_time::QualifiedOpportunity);
    game_random::advance_random(&mut o.random);
    Ok(Opportunity::RoomWork(RoomPass { reach, picture }))
}

#[cfg(test)]
mod tests;
