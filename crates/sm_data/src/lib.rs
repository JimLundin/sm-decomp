//! Validated game content that the compiler keeps free of the emulator.
//!
//! Target package `sm_data` (docs/target-architecture). Records here are built
//! from bytes by an importer and checked when they are built; no ROM layout,
//! RAM or backend type crosses this boundary.
#![forbid(unsafe_code)]

pub mod area_select_palettes;
pub mod ceres_elevator;
pub mod demo_rooms;
pub mod enemy_speeds;
pub mod grapple_origins;
pub mod intro_palette;
pub mod kraid;
pub mod message;
pub mod phantoon;
pub mod raster;
pub mod sand_heap;
pub mod save_slots;
pub mod scene;
pub mod shooting_stars;
pub mod table;
pub mod world;

// Immutable media identities and ordered semantic audio records.
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub struct AssetId(pub u32);
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub struct Revision(pub [u8; 32]);
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub struct AssetRef {
    pub id: AssetId,
    pub revision: Revision,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum MusicMeaning {
    Silence,
    Bank {
        bank: AssetRef,
        after: MusicContinuation,
    },
    Program {
        program: AssetRef,
        phase: u64,
    },
}
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum MusicContinuation {
    Retain,
    Silence,
    Program { program: AssetRef, phase: u64 },
}
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum SoundMeaning {
    Stop,
    Program { program: AssetRef, phase: u64 },
}

/// Selectors in the original programs, distinct from independently authored assets.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct OriginalMusicBank(pub u8);
/// A song selection in the currently selected original music bank.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct OriginalMusicTrack(pub u8);
/// An effect selection in the original sound program; zero retains its protocol meaning.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct OriginalSoundEffect(pub u8);

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum SoundChannel {
    First,
    Second,
    Third,
}
impl SoundChannel {
    pub fn from_index(channel: u8) -> Option<Self> {
        match channel {
            0 => Some(Self::First),
            1 => Some(Self::Second),
            2 => Some(Self::Third),
            _ => None,
        }
    }
    pub fn index(self) -> u8 {
        match self {
            Self::First => 0,
            Self::Second => 1,
            Self::Third => 2,
        }
    }
}

/// The existing producer's publication provenance, not a device clock or transport epoch.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct CueSource {
    pub source_tick: u64,
    pub source_interval: u64,
}
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum MusicBankCue {
    Original(OriginalMusicBank),
    Authored {
        bank: AssetRef,
        after: MusicContinuation,
    },
}
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum MusicCue {
    Original(OriginalMusicTrack),
    Authored(MusicMeaning),
}
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum SoundCue {
    Original(OriginalSoundEffect),
    Authored(SoundMeaning),
}
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum CueAction {
    InitializeAudio,
    MusicBank(MusicBankCue),
    Music(MusicCue),
    Sound {
        channel: SoundChannel,
        sound: SoundCue,
    },
}
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Cue {
    pub source: Option<CueSource>,
    pub action: CueAction,
}

impl Cue {
    pub fn music_bank(
        selector: OriginalMusicBank,
        source: Option<CueSource>,
        meaning: Option<&MusicMeaning>,
    ) -> Self {
        let bank = match meaning {
            Some(MusicMeaning::Bank { bank, after }) => MusicBankCue::Authored {
                bank: *bank,
                after: after.clone(),
            },
            _ => MusicBankCue::Original(selector),
        };
        Self {
            source,
            action: CueAction::MusicBank(bank),
        }
    }
    pub fn music(
        selector: OriginalMusicTrack,
        source: Option<CueSource>,
        meaning: Option<&MusicMeaning>,
    ) -> Self {
        let music = match meaning {
            Some(value @ (MusicMeaning::Silence | MusicMeaning::Program { .. })) => {
                MusicCue::Authored(value.clone())
            }
            _ => MusicCue::Original(selector),
        };
        Self {
            source,
            action: CueAction::Music(music),
        }
    }
    pub fn sound(
        channel: SoundChannel,
        selector: OriginalSoundEffect,
        source: Option<CueSource>,
        meaning: Option<&SoundMeaning>,
    ) -> Self {
        let sound = match meaning {
            Some(value) => SoundCue::Authored(value.clone()),
            None => SoundCue::Original(selector),
        };
        Self {
            source,
            action: CueAction::Sound { channel, sound },
        }
    }
}
