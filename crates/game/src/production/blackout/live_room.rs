//! Run the blackout scene with the work already performed by the live shaft.

use super::{
    BlackoutLight, BlackoutPhase, BlackoutState, Countdown, Fault, LockedHud, Opportunity, Owners,
    Platform, Reach, Request, RoomPass, ShaftStage, State, TimerActivity, TimerCentiseconds,
    WorkPass, picture, save_files,
};
use crate::production::played::{self, front_end};
use crate::state::video::Video;
use boss_ceres_ridley::ElevatorLock;
use canonical::model::Model;
use types::ceres_blackout::{
    BlackoutLevel, BlackoutTimer, FinishedPasses, PreparedPicture, PublishedPicture, TimerSprite,
};
use types::{Area, StationId};

/// Blackout phase and publication while the shaft still supplies ordinary room work.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct LiveRoom {
    scene: BlackoutState,
    standing: Option<super::StandingFrame>,
}

impl LiveRoom {
    /// Consume the elevator reach after the ordinary room has already worked.
    ///
    /// # Errors
    /// The live timer, display light and finished-pass count must agree with the handoff.
    pub fn reached(
        room: &played::State,
        reach: ElevatorLock,
        before: FinishedPasses,
        publishes: bool,
    ) -> Result<Self, Fault> {
        let Countdown::Begun(timer) = room.hud.0.countdown else {
            return Err(Fault::Timer);
        };
        let time = TimerCentiseconds(timer.time_left.hundredths());
        let light = room_light(room)?;
        let mut scene = BlackoutState {
            phase: BlackoutPhase::Shaft,
            light,
            published: PublishedPicture {
                light,
                timer_sprites: sprites(&room.clock_picture.published),
            },
            pass: WorkPass::Finished,
            timer: BlackoutTimer {
                activity: TimerActivity::Active,
                time,
            },
            shaft: ShaftStage::Rotating,
            finished: before,
        };
        let requests = screens_blackout::advance(
            &mut scene,
            Opportunity::RoomWork(RoomPass {
                reach: Reach::Reached(time),
                picture: prepared(room),
            }),
        );
        if requests != [Request::StandAndLock]
            || !matches!(scene.phase, BlackoutPhase::Hold(wait) if wait.get() == reach.hold)
        {
            return Err(Fault::State);
        }
        publish(&mut scene, room, publishes)?;
        Ok(Self {
            scene,
            standing: None,
        })
    }

    /// Whether the final prepared room pass is waiting for the end of vblank.
    #[must_use]
    pub const fn awaits_blank(&self) -> bool {
        matches!(self.scene.phase, BlackoutPhase::AwaitBlank)
    }

    /// Whether the next fresh locked room work will prepare forced blank.
    #[must_use]
    pub fn needs_final_room_work(&self) -> bool {
        self.scene.phase == BlackoutPhase::Fade
            && matches!(self.scene.light, BlackoutLight::Lit(level) if level.lower().is_none())
    }

    /// Advance the blackout from one already-worked fresh locked pass.
    ///
    /// # Errors
    /// The phase must accept new room work and publication must agree with the carried count.
    pub fn worked(
        &mut self,
        room: &mut played::State,
        standing: super::StandingFrame,
        publishes: bool,
    ) -> Result<(), Fault> {
        if !matches!(
            self.scene.phase,
            BlackoutPhase::Hold(_) | BlackoutPhase::Fade
        ) || !matches!(self.scene.pass, WorkPass::Finished)
        {
            return Err(Fault::State);
        }
        let mut scene = self.scene.clone();
        let requests = screens_blackout::advance(
            &mut scene,
            Opportunity::RoomWork(RoomPass {
                reach: Reach::NotReached,
                picture: prepared(room),
            }),
        );
        if requests != [Request::ContinueRoom] {
            return Err(Fault::State);
        }
        publish(&mut scene, room, publishes)?;
        let video = &mut room.room_video.video;
        picture::light(&mut video.batch.registers, scene.light);
        if publishes {
            picture::light(&mut video.committed, scene.published.light);
        }
        self.scene = scene;
        self.standing = Some(standing);
        Ok(())
    }

    /// Move the running room's actual cartridge, state and pending picture into the scheduler.
    ///
    /// # Errors
    /// The final worked pass must still be waiting and the selected cartridge must be ready.
    pub fn into_blackout(
        self,
        room: &played::State,
        video: Box<Video>,
        cartridge: front_end::Cartridge,
        location: (Area, StationId),
        platform: Option<Platform>,
        hud: LockedHud,
    ) -> Result<State, TransferFault> {
        if !self.awaits_blank() || self.standing.is_none() {
            return Err(TransferFault::Phase);
        }
        let (files, capture) = cartridge
            .into_shaft_save(room, location)
            .map_err(TransferFault::Cartridge)?;
        Ok(State {
            owners: Model(Owners {
                scene: self.scene,
                countdown: Countdown::Dormant,
                files: save_files::State::new(files),
                run: capture.run,
                live_map: capture.live_map,
                location: capture.location,
                health: room.health.0,
                random: room.frame.random,
            }),
            video,
            platform,
            hud: Some(hud),
        })
    }
}

/// A live room cannot move into the departure scheduler before its prepared final pass.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum TransferFault {
    /// The room has not suspended the final blackout pass.
    Phase,
    /// The active cartridge or its live file is unavailable.
    Cartridge(front_end::Fault),
}

fn room_light(room: &played::State) -> Result<BlackoutLight, Fault> {
    let brightness = room.arrival.arrival.brightness;
    if brightness.blanking != types::brightness::Blanking::Shown {
        return Err(Fault::Picture);
    }
    BlackoutLevel::new(brightness.level.get())
        .map(BlackoutLight::Lit)
        .ok_or(Fault::Picture)
}

fn sprites(entries: &[[u8; 5]]) -> Vec<TimerSprite> {
    entries
        .iter()
        .map(|&[_, _, tile, attributes, _]| TimerSprite(u16::from_le_bytes([tile, attributes])))
        .collect()
}

fn prepared(room: &played::State) -> PreparedPicture {
    PreparedPicture {
        timer_sprites: sprites(&room.clock_picture.prepared),
    }
}

fn publish(scene: &mut BlackoutState, room: &played::State, publishes: bool) -> Result<(), Fault> {
    if publishes {
        screens_blackout::advance(scene, Opportunity::Publication);
        if !matches!(scene.pass, WorkPass::Finished)
            || scene.published.timer_sprites != sprites(&room.clock_picture.published)
        {
            return Err(Fault::Picture);
        }
    }
    if scene.finished.0 != room.frame.finished_frames {
        return Err(Fault::State);
    }
    Ok(())
}
