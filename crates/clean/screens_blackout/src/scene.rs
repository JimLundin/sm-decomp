//! The scene, one work opportunity at a time: what each opportunity changes in the State and
//! what the scene asks of the other parts of the game.

use types::ceres_blackout::{
    BlackoutLight, BlackoutPhase, BlackoutState, FlightRoute, PreparedPicture, PublishedPicture,
    ShaftStage, TimerActivity, TimerCentiseconds, WaitPasses, WorkPass,
};

/// Whether a pass of ordinary shaft work reports that Samus reached the elevator.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Reach {
    /// No report: she has not reached it this pass.
    NotReached,
    /// She reached it, after her movement and the room's drawing; the escape timer shows the
    /// time that pass left it at.
    Reached(TimerCentiseconds),
}

/// A room work pass the scheduler completed: the reach it reports, read only during ordinary
/// shaft work, and the picture it drew.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct RoomPass {
    /// Whether Samus reached the elevator.
    pub reach: Reach,
    /// The picture the pass drew, the timer's sprites among it.
    pub picture: PreparedPicture,
}

/// A work opportunity the scheduler supplies.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Opportunity {
    /// A new room work pass ran.
    RoomWork(RoomPass),
    /// The scheduler cut in after work, before the pass finished.
    Interrupted,
    /// A vertical interval ended: the safe moment for the handoff's display work.
    VerticalBlankEnded,
    /// The cartridge finished writing the save.
    SaveCompleted,
    /// The display published the prepared picture, finishing the pass.
    Publication,
    /// A fresh work opportunity, which may dispatch a pending scene.
    NewWork,
}

/// A number of work intervals, which a request waits before it acts.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct WorkIntervals(pub u8);

/// Spec: `DepartureHandoff`. The music stops after 8 work intervals.
pub const MUSIC_STOP_DELAY: WorkIntervals = WorkIntervals(8);

/// One of the sound channels a silence goes to.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum SoundChannel {
    /// The first sound channel.
    One,
    /// The second sound channel.
    Two,
    /// The third sound channel.
    Three,
}

/// Spec: `DepartureHandoff`. One silence on each of the three sound channels, in this order.
pub const SILENCED_CHANNELS: &[SoundChannel] =
    &[SoundChannel::One, SoundChannel::Two, SoundChannel::Three];

/// What the scene asks of the other parts of the game, in the order it asks.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Request {
    /// Spec: `ElevatorStopsSamus`. Samus's movement and appearance give her a standing pose with
    /// her current facing, cancel a pending pose change and lock her.
    StandAndLock,
    /// Spec: `PlatformWait`. Run the locked room this pass: its shaft, actors, weapon updates,
    /// game clock and shake, keeping the picture it draws and the state it leaves.
    ContinueRoom,
    /// Spec: `DepartureHandoff`. Shut off the shaft's interrupt and raster work.
    DisableShaftInterrupts,
    /// Spec: `DepartureHandoff`. Prepare the gameplay display mode with only sprites on its
    /// primary image, colour blending and window masking off.
    PrepareSpriteOnly,
    /// Spec: `DepartureHandoff`. Save files capture the active file for this route from the
    /// live Ceres area and station, with her actual energy and elapsed play time, and write it.
    CaptureEscapeSave(FlightRoute),
    /// Spec: `DepartureHandoff`. Audio stops the music after this delay.
    StopMusic(WorkIntervals),
    /// Spec: `DepartureHandoff`. Audio silences this sound channel.
    Silence(SoundChannel),
    /// Spec: `DepartureHandoff`. The station explosion is pending, with its destination.
    PrepareStationExplosion(FlightRoute),
    /// Spec: `DepartureHandoff`. The pending explosion takes the scene.
    DispatchExplosion(FlightRoute),
}

/// The scene's answer to one work opportunity: the State it leaves and the requests it makes,
/// in order.
pub fn advance(state: &mut BlackoutState, opportunity: Opportunity) -> Vec<Request> {
    match opportunity {
        Opportunity::RoomWork(pass) => room_work(state, pass),
        Opportunity::Interrupted => Vec::new(),
        Opportunity::VerticalBlankEnded => vertical_blank_ended(state),
        Opportunity::SaveCompleted => save_completed(state),
        Opportunity::Publication => {
            publish(state);
            Vec::new()
        }
        Opportunity::NewWork => new_work(state),
    }
}

/// A room work pass, by phase. Spec: `PlatformWait`: once the pass is suspended for the
/// blank, no new room work runs; the interrupted cut keeps its prepared picture, its room
/// clocks and the earlier publication until the work resumes.
fn room_work(state: &mut BlackoutState, pass: RoomPass) -> Vec<Request> {
    match state.phase {
        BlackoutPhase::Shaft => shaft_work(state, pass),
        BlackoutPhase::Hold(wait) => {
            state.pass = WorkPass::Prepared(pass.picture);
            state.phase = waited(wait);
            vec![Request::ContinueRoom]
        }
        BlackoutPhase::Fade => {
            state.pass = WorkPass::Prepared(pass.picture);
            dim(state);
            vec![Request::ContinueRoom]
        }
        BlackoutPhase::AwaitBlank
        | BlackoutPhase::Saving(_)
        | BlackoutPhase::Explosion(_)
        | BlackoutPhase::Dispatched(_) => Vec::new(),
    }
}

/// Spec: `ElevatorStopsSamus`. Ordinary shaft work keeps its picture; a reach stops her,
/// freezes the countdown at the time the pass left it and starts the whole wait, the light held
/// where it is. Without a reach the wait isn't started.
fn shaft_work(state: &mut BlackoutState, pass: RoomPass) -> Vec<Request> {
    state.pass = WorkPass::Prepared(pass.picture);
    match pass.reach {
        Reach::NotReached => Vec::new(),
        Reach::Reached(time) => {
            state.timer.time = time;
            state.phase = BlackoutPhase::Hold(WaitPasses::WHOLE);
            vec![Request::StandAndLock]
        }
    }
}

/// Spec: `PlatformWait`. A waiting pass takes one pass off the wait; at zero the light begins
/// to dim, one level each following pass with no delay between steps, and this pass leaves it
/// alone.
fn waited(wait: WaitPasses) -> BlackoutPhase {
    wait.counted_down()
        .map_or(BlackoutPhase::Fade, BlackoutPhase::Hold)
}

/// Spec: `PlatformWait`. A dimming pass lowers the light one level; from level 1 it forces
/// blank and suspends the pass after its room work, to wait for the end of a vertical interval.
const fn dim(state: &mut BlackoutState) {
    state.light = dimmer(state.light);
    if matches!(state.light, BlackoutLight::ForcedBlank) {
        state.phase = BlackoutPhase::AwaitBlank;
    }
}

/// Spec: `PlatformWait`. One level down; from level 1 the light is forced blank.
const fn dimmer(light: BlackoutLight) -> BlackoutLight {
    let lowered = match light {
        BlackoutLight::Lit(level) => level.lower(),
        BlackoutLight::ForcedBlank => None,
    };
    match lowered {
        Some(level) => BlackoutLight::Lit(level),
        None => BlackoutLight::ForcedBlank,
    }
}

/// Spec: `DepartureHandoff`. At the end of the vertical interval the blank waits for, the shaft's
/// interrupt work stops, the sprite-only display is prepared and the save for the Zebes flight
/// begins. The suspended pass keeps its picture and the earlier publication.
fn vertical_blank_ended(state: &mut BlackoutState) -> Vec<Request> {
    match state.phase {
        BlackoutPhase::AwaitBlank => {
            let route = FlightRoute::ZebesFlight;
            state.phase = BlackoutPhase::Saving(route);
            vec![
                Request::DisableShaftInterrupts,
                Request::PrepareSpriteOnly,
                Request::CaptureEscapeSave(route),
            ]
        }
        BlackoutPhase::Shaft
        | BlackoutPhase::Hold(_)
        | BlackoutPhase::Fade
        | BlackoutPhase::Saving(_)
        | BlackoutPhase::Explosion(_)
        | BlackoutPhase::Dispatched(_) => Vec::new(),
    }
}

/// Spec: `DepartureHandoff`. The completed save clears the shaft's stage and retires the timer,
/// its digits kept; the prepared picture, timer sprites included, stays for the publication. The
/// music stops, each sound channel is silenced, and the explosion is pending with its route.
fn save_completed(state: &mut BlackoutState) -> Vec<Request> {
    match state.phase {
        BlackoutPhase::Saving(route) => {
            state.shaft = ShaftStage::Cleared;
            state.timer.activity = TimerActivity::Retired;
            state.phase = BlackoutPhase::Explosion(route);
            let mut requests = vec![Request::StopMusic(MUSIC_STOP_DELAY)];
            requests.extend(SILENCED_CHANNELS.iter().copied().map(Request::Silence));
            requests.push(Request::PrepareStationExplosion(route));
            requests
        }
        BlackoutPhase::Shaft
        | BlackoutPhase::Hold(_)
        | BlackoutPhase::Fade
        | BlackoutPhase::AwaitBlank
        | BlackoutPhase::Explosion(_)
        | BlackoutPhase::Dispatched(_) => Vec::new(),
    }
}

/// Spec: `PlatformWait`, `DepartureHandoff`. The display publishes a prepared pass: the light the
/// scene intends and the picture the pass drew, counting one finished pass. While the pass is
/// suspended for the blank or the save, the publication waits.
fn publish(state: &mut BlackoutState) {
    match state.phase {
        BlackoutPhase::AwaitBlank | BlackoutPhase::Saving(_) => {}
        BlackoutPhase::Shaft
        | BlackoutPhase::Hold(_)
        | BlackoutPhase::Fade
        | BlackoutPhase::Explosion(_)
        | BlackoutPhase::Dispatched(_) => {
            if let WorkPass::Prepared(picture) =
                core::mem::replace(&mut state.pass, WorkPass::Finished)
            {
                state.published = PublishedPicture {
                    light: state.light,
                    timer_sprites: picture.timer_sprites,
                };
                state.finished = state.finished.counted_up();
            }
        }
    }
}

/// Spec: `DepartureHandoff`. A fresh opportunity after the final room publication dispatches the
/// pending explosion, once; before that publication it waits.
fn new_work(state: &mut BlackoutState) -> Vec<Request> {
    match (state.phase, &state.pass) {
        (BlackoutPhase::Explosion(route), WorkPass::Finished) => {
            state.phase = BlackoutPhase::Dispatched(route);
            vec![Request::DispatchExplosion(route)]
        }
        _ => Vec::new(),
    }
}
