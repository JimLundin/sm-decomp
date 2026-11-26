//! The front end's screens from the title on (the title, file select, the options menu and the
//! intro): their borrowed inputs and the fields compared at each cut. The intro's own fields are
//! the intro run's (`super::intro`).
//!
//! Compared at every cut: the screen, blank or lit, and its level; which part of the game runs
//! (the game state); and the frame owner's count of finished frames. At finished cuts: the
//! random number, which a lag cut masks because where the draw falls in a frame is platform
//! timing (T1); the sounds asked for since the last finished cut against the entries added to
//! the music queue and the sound queues of libraries 1 and 3; the title's countdown while the
//! title screen shows or is left; file select's cursor once its menu is set up; the options'
//! cursor wherever the State holds one, the dissolve's block size and the three settings.

use super::platform::{Controller, music_ready};
use super::word;
use crate::replay::compare::Observation;
use crate::replay::{data_failure, result::Outcome};
use crate::source;
use game::production::played::front_end::{Button, Platform, Pressed, Screen, Screens, Sound};
use oracle_record::FrameRecord;
use screens_intro::{MusicRequest, TrackReply, Tune};
use types::RandomSequence;
use types::file_select::{
    FileSelectStage, MenuItem, MenuPhase, MenuState, Slot, WayOut as FileSelectWayOut,
};
use types::options::{
    DissolveCursor, Language, OptionsItem, OptionsStage, Settings, SpecialItem, WayOut,
};
use types::power_on::ScreenLight;
use types::title::TitleStage;

const GAME_STATE: source::WramWord = source::WramWord {
    name: "GAME_STATE",
    offset: 0x0998,
};
const DEMO_COUNTDOWN: source::WramWord = source::WramWord {
    name: "TITLE_DEMO_TIMER",
    offset: 0x1F53,
};
const FILE_SELECT_CURSOR: source::WramWord = source::WramWord {
    name: "FILE_SELECT_CURSOR",
    offset: 0x0952,
};
const OPTIONS_CURSOR: source::WramWord = source::WramWord {
    name: "OPTIONS_CURSOR",
    offset: 0x099E,
};
const JAPANESE_TEXT: source::WramWord = source::WramWord {
    name: "JAPANESE_TEXT",
    offset: 0x09E2,
};
const MOONWALK: source::WramWord = source::WramWord {
    name: "MOONWALK",
    offset: 0x09E4,
};
const WEAPON_RESET: source::WramWord = source::WramWord {
    name: "ICON_CANCEL",
    offset: 0x09EA,
};
/// The mosaic register's copy: the block size in the high nibble, on when the low bits are 3.
const MOSAIC: usize = 0x0057;
const MOSAIC_ON: u8 = 0x03;

/// The game states of the title, file select and the options menu.
const TITLE: u16 = 0x01;
const OPTIONS: u16 = 0x02;
const FILE_SELECT: u16 = 0x04;
const INTRO: u16 = 0x1E;

/// The display register's forced blank bit and its brightness.
const BLANK: u8 = 0x80;
const LEVEL: u8 = 0x0F;

/// The music queue's entries, and the sound queues of libraries 1 and 3: each a ring of 16
/// bytes with its end index.
const MUSIC_QUEUE: usize = 0x0619;
const MUSIC_ENTRIES: usize = 8;
const LIBRARY_ONE: Library = Library {
    tag: 1,
    end: 0x0646,
    entries: 0x0656,
};
const LIBRARY_THREE: Library = Library {
    tag: 3,
    end: 0x0648,
    entries: 0x0676,
};
const RING: usize = 16;
/// The music queue's tag in a sound's code.
const MUSIC: u8 = 0;

/// A sound library's queue.
struct Library {
    tag: u8,
    end: usize,
    entries: usize,
}

/// The controller's buttons as the pad's bits hold them.
const BUTTONS: [(u16, Button); 12] = [
    (0x8000, Button::B),
    (0x4000, Button::Y),
    (0x2000, Button::Select),
    (0x1000, Button::Start),
    (0x0800, Button::Up),
    (0x0400, Button::Down),
    (0x0200, Button::Left),
    (0x0100, Button::Right),
    (0x0080, Button::A),
    (0x0040, Button::X),
    (0x0020, Button::L),
    (0x0010, Button::R),
];

/// The buttons a pad's bits hold.
pub(super) fn pressed(controller: Controller) -> Pressed {
    BUTTONS
        .iter()
        .map(|&(bit, button)| (button, controller.pressed & bit != 0))
        .collect()
}

/// The frames of work finished in the interval ending at `current`: one at a requested NMI,
/// none at a lag cut, and two where the game's own count steps by 2 at a requested NMI.
pub(super) fn finished_frames(
    previous: &FrameRecord,
    current: &FrameRecord,
    finished: bool,
) -> Result<u16, Outcome> {
    let step = word(&current.wram, source::NMI_FRAME_COUNTER_WORD)?
        .wrapping_sub(word(&previous.wram, source::NMI_FRAME_COUNTER_WORD)?);
    match (finished, step) {
        (false, 0) => Ok(0),
        (true, 1 | 2) => Ok(step),
        _ => Err(data_failure(None, "played frames finished in the interval")),
    }
}

/// The aural and random replies supplied by the platform seam.
pub(super) struct Borrowed {
    settled_random: RandomSequence,
    track: TrackReply,
    waiting: samus_appearance::SoundsWaiting,
    timing: game::production::played::front_end::IntroTiming,
}

impl Borrowed {
    pub(super) fn read(
        began: &FrameRecord,
        previous: &FrameRecord,
        current: &FrameRecord,
    ) -> Result<Self, Outcome> {
        Ok(Self {
            timing: super::power_on::intro_timing(current)?,
            waiting: crate::replay::appearance::sounds_waiting(&began.wram)?,
            settled_random: super::frame::random(&previous.wram)?,
            track: if music_ready(&began.wram)? {
                TrackReply::NoneWaits
            } else {
                TrackReply::TrackWaits
            },
        })
    }
}

impl Platform for Borrowed {
    fn intro_timing(&self) -> game::production::played::front_end::IntroTiming {
        self.timing
    }
    fn sounds_waiting(&self) -> samus_appearance::SoundsWaiting {
        self.waiting
    }
    fn settled_random(&self) -> RandomSequence {
        self.settled_random
    }

    fn track(&self) -> TrackReply {
        self.track
    }
}

/// The queues as one cut holds them, to find the entries added by the next.
#[derive(Clone)]
pub(super) struct Queues {
    music: [u16; MUSIC_ENTRIES],
    ends: [u8; 2],
}

impl Queues {
    pub(super) fn read(record: &FrameRecord) -> Result<Self, Outcome> {
        let bad = || data_failure(None, "played sound queues");
        let wram = &record.wram;
        let mut music = [0; MUSIC_ENTRIES];
        for (at, entry) in music.iter_mut().enumerate() {
            let offset = MUSIC_QUEUE.wrapping_add(at.wrapping_mul(2));
            *entry = wram
                .get(offset..offset.wrapping_add(2))
                .and_then(|bytes| bytes.try_into().ok())
                .map(u16::from_le_bytes)
                .ok_or_else(bad)?;
        }
        let end = |library: &Library| wram.get(library.end).copied().ok_or_else(bad);
        Ok(Self {
            music,
            ends: [end(&LIBRARY_ONE)?, end(&LIBRARY_THREE)?],
        })
    }

    /// The sounds `record` holds that these queues didn't, each as its code, in order.
    pub(super) fn added(&self, record: &FrameRecord) -> Result<Vec<[u8; 3]>, Outcome> {
        let bad = || data_failure(None, "played sound queues");
        let now = Self::read(record)?;
        let mut added: Vec<[u8; 3]> = now
            .music
            .iter()
            .zip(self.music)
            .filter(|&(&entry, before)| entry != 0 && entry != before)
            .map(|(&entry, _)| code(MUSIC, entry))
            .collect();
        for ((library, before), after) in [LIBRARY_ONE, LIBRARY_THREE]
            .iter()
            .zip(self.ends)
            .zip(now.ends)
        {
            let count = usize::from(after.wrapping_sub(before)) % RING;
            for step in 0..count {
                let slot = usize::from(before).wrapping_add(step) % RING;
                let entry = record
                    .wram
                    .get(library.entries.wrapping_add(slot))
                    .copied()
                    .ok_or_else(bad)?;
                added.push(code(library.tag, u16::from(entry)));
            }
        }
        added.sort_unstable();
        Ok(added)
    }
}

/// A sound's code: its queue and its number.
const fn code(queue: u8, number: u16) -> [u8; 3] {
    let [low, high] = number.to_le_bytes();
    [queue, low, high]
}

/// The code the game queues a sound as; none for the music's stop, which the queue holds as an
/// empty entry, so the queues can't show it.
pub(super) const fn sound_code(sound: Sound) -> Option<[u8; 3]> {
    use screens_file_select::MenuSound as FileSelectSound;
    use screens_options::MenuSound as OptionsSound;
    use screens_title::TuneRequest;
    Some(match sound {
        Sound::Title(TuneRequest::TitleMusic) => code(MUSIC, 0xFF03),
        Sound::Title(TuneRequest::OpeningTheme) | Sound::IntroMusic(MusicRequest::Play { .. }) => {
            code(MUSIC, 0x0005)
        }
        Sound::Title(TuneRequest::TitleTheme) => code(MUSIC, 0x0006),
        Sound::FileSelect(FileSelectSound::MenuOpens) => code(LIBRARY_THREE.tag, 0x01),
        Sound::FileSelect(FileSelectSound::CursorMoved)
        | Sound::Options(OptionsSound::CursorMoved) => code(LIBRARY_ONE.tag, 0x37),
        Sound::FileSelect(FileSelectSound::ChoiceMade) => code(LIBRARY_ONE.tag, 0x2A),
        Sound::Options(OptionsSound::Selected) => code(LIBRARY_ONE.tag, 0x38),
        Sound::Appearance(offer) => {
            use samus_appearance::{Sound as Cue, SoundQueue};
            let queue = match offer.queue {
                SoundQueue::Footsteps => LIBRARY_THREE.tag,
                SoundQueue::Samus => LIBRARY_ONE.tag,
            };
            let number = match offer.sound {
                Cue::Footstep => 0x06,
                Cue::Boosting => 0x03,
                Cue::PlainSpin => 0x31,
                Cue::SpaceJump => 0x3e,
                Cue::ScrewAttack => 0x33,
                Cue::SpinEnded => 0x32,
                Cue::SamusSilence => 0x02,
                Cue::Hurt => 0x35,
            };
            code(queue, number)
        }
        Sound::IntroMusic(MusicRequest::Stop) => return None,
        Sound::IntroMusic(MusicRequest::Load(Tune::OpeningFirst)) => code(MUSIC, 0xFF3F),
        Sound::IntroMusic(MusicRequest::Load(Tune::OpeningSecond)) => code(MUSIC, 0xFF42),
        Sound::IntroMusic(MusicRequest::Load(Tune::Pages)) => code(MUSIC, 0xFF36),
        Sound::IntroClick => code(LIBRARY_THREE.tag, 0x0D),
    })
}

/// The expected and actual observations, in the order a difference is reported.
#[derive(Default)]
pub(super) struct Pairs {
    pub(super) expected: Vec<Observation>,
    pub(super) actual: Vec<Observation>,
}

impl Pairs {
    fn push(&mut self, field: &str, expected: i64, actual: i64) {
        self.expected.push(Observation::quantity(field, expected));
        self.actual.push(Observation::quantity(field, actual));
    }

    fn push_bytes(&mut self, field: &str, expected: Vec<u8>, actual: Vec<u8>) {
        self.expected.push(Observation {
            field: field.into(),
            bytes: expected,
        });
        self.actual.push(Observation {
            field: field.into(),
            bytes: actual,
        });
    }
}

/// The sounds asked for since the last finished cut, expected and played, each sorted: the
/// queues don't keep the order of two libraries' sounds.
pub(super) struct Asked {
    pub(super) expected: Vec<[u8; 3]>,
    pub(super) played: Vec<[u8; 3]>,
}

/// The screens' fields at `cut`, compared where the played State holds them.
pub(super) fn compared(
    record: &FrameRecord,
    screens: &Screens,
    finished: bool,
    asked: Option<Asked>,
) -> Result<Pairs, Outcome> {
    let wram = &record.wram;
    let mut pairs = Pairs::default();
    pairs.push(
        "front-end.screen",
        i64::from(record.ppu.inidisp & (BLANK | LEVEL)),
        i64::from(screen(&screens.screen)),
    );
    if let Some(state) = game_state(&screens.screen) {
        pairs.push(
            "front-end.game-state",
            i64::from(word(wram, GAME_STATE)?),
            i64::from(state),
        );
    }
    pairs.push(
        "front-end.finished-frames",
        i64::from(word(wram, source::NMI_FRAME_COUNTER_WORD)?),
        i64::from(screens.counts.finished_frames),
    );
    if let Screen::Title(title) = &screens.screen
        && let TitleStage::TitleScreen { countdown } | TitleStage::Leaving { countdown, .. } =
            title.stage
    {
        pairs.push(
            "title.countdown",
            i64::from(word(wram, DEMO_COUNTDOWN)?),
            i64::from(countdown.get()),
        );
    }
    if !finished {
        return Ok(pairs);
    }
    pairs.push(
        "front-end.random",
        i64::from(word(wram, source::RANDOM_NUMBER)?),
        i64::from(screens.counts.random.number.0),
    );
    if let Some(asked) = asked {
        pairs.push_bytes(
            "front-end.sounds",
            asked.expected.concat(),
            asked.played.concat(),
        );
    }
    match &screens.screen {
        Screen::Title(_) | Screen::Intro(_) => {}
        Screen::FileSelect(file_select) => {
            if let FileSelectStage::Menu(MenuState { cursor, .. }) = file_select.menu.stage {
                pairs.push(
                    "file-select.cursor",
                    i64::from(word(wram, FILE_SELECT_CURSOR)?),
                    file_select_cursor(cursor),
                );
            }
        }
        Screen::Options(options) => {
            let menu = options.menu;
            if let Some(cursor) = options_cursor(menu.stage) {
                pairs.push(
                    "options.cursor",
                    i64::from(word(wram, OPTIONS_CURSOR)?),
                    cursor,
                );
            }
            let mosaic = wram
                .get(MOSAIC)
                .copied()
                .ok_or_else(|| data_failure(None, "played mosaic"))?;
            let recorded = if mosaic & MOSAIC_ON == MOSAIC_ON {
                i64::from(mosaic >> 4)
            } else {
                -1
            };
            pairs.push("options.blocks", recorded, blocks(menu.stage));
            settings(&mut pairs, wram, menu.settings)?;
        }
    }
    Ok(pairs)
}

fn settings(pairs: &mut Pairs, wram: &[u8], settings: Settings) -> Result<(), Outcome> {
    let japanese = settings.language == Language::Japanese;
    for (field, at, on) in [
        ("options.japanese", JAPANESE_TEXT, japanese),
        ("options.weapon-reset", WEAPON_RESET, settings.weapon_reset),
        ("options.moonwalk", MOONWALK, settings.moonwalk),
    ] {
        pairs.push(field, i64::from(word(wram, at)?), i64::from(on));
    }
    Ok(())
}

/// The screen as the display register holds it.
fn screen(screen: &Screen) -> u8 {
    let light = match screen {
        Screen::Title(title) => title.screen,
        Screen::FileSelect(file_select) => file_select.menu.screen,
        Screen::Options(options) => options.menu.screen,
        Screen::Intro(intro) => {
            let brightness = intro.scene.intro.brightness;
            let level = brightness.level.number();
            return match brightness.blanking {
                types::intro::Blanking::Blanked => BLANK | level,
                types::intro::Blanking::Shown => level,
            };
        }
    };
    match light {
        ScreenLight::Blank(level) => BLANK | level.get(),
        ScreenLight::Lit(level) => level.get(),
    }
}

/// The game state the part that runs is, where it is one the run plays through.
const fn game_state(screen: &Screen) -> Option<u16> {
    match screen {
        Screen::Title(title) => match title.stage {
            TitleStage::FileSelectBegins => Some(FILE_SELECT),
            TitleStage::OpeningPlays | TitleStage::DemoBegins => None,
            TitleStage::Begun
            | TitleStage::Loading
            | TitleStage::SkipDimming
            | TitleStage::Preparing
            | TitleStage::Brightening
            | TitleStage::TitleScreen { .. }
            | TitleStage::Leaving { .. } => Some(TITLE),
        },
        Screen::FileSelect(file_select) => match file_select.menu.stage {
            FileSelectStage::Menu(MenuState {
                phase: MenuPhase::Out(FileSelectWayOut::OptionsBegin),
                ..
            }) => Some(OPTIONS),
            FileSelectStage::Menu(MenuState {
                phase: MenuPhase::Out(_),
                ..
            }) => None,
            FileSelectStage::Menu(_) | FileSelectStage::Loading(_) => Some(FILE_SELECT),
        },
        Screen::Intro(_) => Some(INTRO),
        // The intro's hand-on is the options' until the frame end it hands on at.
        Screen::Options(options) => match options.menu.stage {
            OptionsStage::HandedOn(
                WayOut::EscapeBegins
                | WayOut::PlayContinues
                | WayOut::FileSelectAgain
                | WayOut::ControlsBegin,
            ) => None,
            OptionsStage::HandedOn(WayOut::IntroBegins)
            | OptionsStage::Loading(_)
            | OptionsStage::Brightening(_)
            | OptionsStage::Choosing(_)
            | OptionsStage::Special(_)
            | OptionsStage::DissolvingOut(_)
            | OptionsStage::DissolvingIn(_)
            | OptionsStage::Leaving { .. }
            | OptionsStage::Returning(_) => Some(OPTIONS),
        },
    }
}

/// File select's cursor as the game numbers its items, top to bottom.
const fn file_select_cursor(cursor: MenuItem) -> i64 {
    match cursor {
        MenuItem::File(Slot::First) => 0,
        MenuItem::File(Slot::Second) => 1,
        MenuItem::File(Slot::Third) => 2,
        MenuItem::Copy => 3,
        MenuItem::Clear => 4,
        MenuItem::Exit => 5,
    }
}

/// The options' cursor as the game numbers its items, where the State holds one.
const fn options_cursor(stage: OptionsStage) -> Option<i64> {
    match stage {
        OptionsStage::Brightening(item)
        | OptionsStage::Choosing(item)
        | OptionsStage::Returning(item)
        | OptionsStage::Leaving { cursor: item, .. } => Some(match item {
            OptionsItem::StartGame => 0,
            OptionsItem::EnglishText => 1,
            OptionsItem::JapaneseText => 2,
            OptionsItem::ControllerSettings => 3,
            OptionsItem::SpecialSettings => 4,
        }),
        OptionsStage::Special(item) => Some(match item {
            SpecialItem::WeaponReset => 0,
            SpecialItem::Moonwalk => 1,
            SpecialItem::End => 2,
        }),
        OptionsStage::DissolvingOut(dissolving) | OptionsStage::DissolvingIn(dissolving) => {
            Some(match dissolving.cursor {
                DissolveCursor::StartGame => 0,
                DissolveCursor::ControllerSettings => 3,
                DissolveCursor::SpecialSettings => 4,
            })
        }
        OptionsStage::Loading(_) | OptionsStage::HandedOn(_) => None,
    }
}

/// The dissolve's block size, -1 with no dissolve.
fn blocks(stage: OptionsStage) -> i64 {
    match stage {
        OptionsStage::DissolvingOut(dissolving) | OptionsStage::DissolvingIn(dissolving) => {
            i64::from(dissolving.blocks.get())
        }
        OptionsStage::Loading(_)
        | OptionsStage::Brightening(_)
        | OptionsStage::Choosing(_)
        | OptionsStage::Special(_)
        | OptionsStage::Leaving { .. }
        | OptionsStage::Returning(_)
        | OptionsStage::HandedOn(_) => -1,
    }
}
