//! The music the intro asks audio for (`IntroBegins`, `OpeningUnfolds`, `PageOneIsMadeReady`,
//! and the stop as page 6 begins, `PressTurnsThePage`), and audio's reply that a tune asked for still waits to load (`OpeningUnfolds`,
//! `PageOneBegins`).

/// Each music request is asked 8 frames ahead unless it names a longer delay.
pub const REQUEST_DELAY: u16 = 8;
/// `OpeningUnfolds`, `PageOneIsMadeReady`: the second tune and the pages' tune start 14 frames
/// after they load.
const TUNE_STARTS_AFTER_LOADING: u16 = 14;

/// A tune the intro plays.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Tune {
    /// The opening's first tune, under its words.
    OpeningFirst,
    /// The opening's second tune, which follows it.
    OpeningSecond,
    /// The pages' tune, from page 1 on.
    Pages,
}

/// A request to audio about the music.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum MusicRequest {
    /// Stop the music.
    Stop,
    /// Load this tune.
    Load(Tune),
    /// Play the loaded tune, these frames after.
    Play {
        /// The frames before it starts.
        after: u16,
    },
}

impl MusicRequest {
    /// The frames ahead the request is asked with: a play's own delay, 8 for the others.
    #[must_use]
    pub const fn delay(self) -> u16 {
        match self {
            Self::Stop | Self::Load(_) => REQUEST_DELAY,
            Self::Play { after } => after,
        }
    }
}

/// The music stopped and `tune` asked to load: the requests as a new tune is asked for.
#[must_use]
pub const fn tune_asked_for(tune: Tune) -> [MusicRequest; 2] {
    [MusicRequest::Stop, MusicRequest::Load(tune)]
}

/// The music stopped, `tune` asked to load and to play 14 frames after: the second tune and the
/// pages' tune.
#[must_use]
pub const fn tune_asked_to_follow(tune: Tune) -> [MusicRequest; 3] {
    let [stop, load] = tune_asked_for(tune);
    [
        stop,
        load,
        MusicRequest::Play {
            after: TUNE_STARTS_AFTER_LOADING,
        },
    ]
}

/// Audio's reply: whether a tune asked for still waits to load and start.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum TrackReply {
    /// A track still waits: the loading stages hold.
    TrackWaits,
    /// None waits: the tune asked for has loaded.
    NoneWaits,
}
