use super::{
    Answers, Content, Counts, Fault, FileStart, FrameEnd, Options, Pressed, Rules, Screen, Screens,
    Sound, State, Work,
};
use screens_boot::{DriverUpload, FinishedFrames, Level, PowerOnStage, ScreenLight};
use screens_intro::{IntroContent, TrackReply};
use screens_options::ChosenFile;
use types::file_select::Slot;
use types::options::{FileBegins, Settings};
use types::title::TitleStage;
use types::{RandomNumber, RandomSequence, Set};

/// No slot holds a file, so a chosen one is a new file beginning with the intro.
struct NewGame;

impl screens_file_select::SaveMemory for NewGame {
    fn files(&self) -> Set<Slot> {
        Set::default()
    }

    fn last_chosen(&self) -> Option<Slot> {
        None
    }
}

fn actor_file() -> super::ActorFile {
    use types::save_files::{Gear, Supply};
    let no_gear = Gear {
        items: Set::default(),
        beams: Set::default(),
    };
    super::ActorFile {
        energy: Supply {
            current: 99,
            maximum: 99,
        },
        reserves: Supply {
            current: 0,
            maximum: 0,
        },
        owned: no_gear,
        gear: no_gear,
        buttons: super::intro_buttons(),
        ammunition: enum_map::EnumMap::from_fn(|_| Supply {
            current: 0,
            maximum: 0,
        }),
    }
}

impl Answers for NewGame {
    fn intro_timing(&self) -> super::IntroTiming {
        super::IntroTiming::Unspecified
    }
    fn pending_buttons(&self) -> Pressed {
        Pressed::default()
    }
    fn sounds_waiting(&self) -> samus_appearance::SoundsWaiting {
        samus_appearance::SoundsWaiting::new(0).unwrap()
    }
    fn settled_random(&self) -> RandomSequence {
        RandomSequence {
            number: RandomNumber(0x61),
        }
    }

    fn track(&self) -> TrackReply {
        TrackReply::NoneWaits
    }

    fn file(&self, _: Slot) -> Option<FileStart> {
        Some(FileStart {
            settings: Settings::default(),
            begins: ChosenFile::Begins(FileBegins::Intro),
            actor: actor_file(),
        })
    }
}

const fn end(work: Work) -> FrameEnd {
    FrameEnd {
        work,
        upload: DriverUpload::Continuing,
    }
}

fn finished() -> FrameEnd {
    end(Work::Finished(Pressed::default()))
}

/// The intro's content with no script, colour or set: the hand-on reads none.
fn content() -> Content {
    Content {
        actors: None,
        intro: IntroContent {
            pages: enum_map::EnumMap::default(),
            eyes: enum_map::EnumMap::default(),
            baby: enum_map::EnumMap::default(),
            caret: enum_map::EnumMap::default(),
            full: types::Palette([types::Colour::BLACK; types::PALETTE_SLOTS]),
            sets: enum_map::enum_map! {
                _ => screens_intro::DissolveSets {
                    dims: Vec::new(),
                    brightens: Vec::new(),
                },
            },
            text: Vec::new(),
            caret_colours: Vec::new(),
            caret_home: types::Pair {
                x: core::num::Wrapping(0),
                y: core::num::Wrapping(0),
            },
        },
    }
}

fn step(state: &mut State, frame: FrameEnd) -> Result<Vec<Sound>, Fault> {
    state.step(frame, &Rules::REFERENCE, &content(), &NewGame)
}

fn power_on(state: &State) -> screens_boot::PowerOnState {
    match *state {
        State::PowerOn(boot) => boot.model,
        State::Screens(_) => panic!("the title has begun"),
    }
}

#[test]
fn the_boot_runs_from_power_on_until_the_title_begins() {
    let mut state = State::AT_POWER_ON;
    for _ in 0..17 {
        step(&mut state, end(Work::Unfinished)).unwrap();
    }
    assert_eq!(power_on(&state).stage, PowerOnStage::BeforeLogo);
    step(&mut state, finished()).unwrap();
    let lit = power_on(&state);
    assert_eq!(lit.screen, ScreenLight::Lit(Level::new(1).unwrap()));
    assert_eq!(lit.frames, FinishedFrames(1));
    // The logo brightens, holds and dims on finished frames, then the sound loads.
    while power_on(&state).stage != PowerOnStage::SoundLoading {
        step(&mut state, finished()).unwrap();
    }
    step(&mut state, end(Work::Unfinished)).unwrap();
    let ended = FrameEnd {
        work: Work::Unfinished,
        upload: DriverUpload::Ended,
    };
    step(&mut state, ended).unwrap();
    assert_eq!(power_on(&state).stage, PowerOnStage::Settling);
    assert_eq!(power_on(&state).frames, FinishedFrames::ZERO);
    // The settled game's first finished frame is the title's, counted and drawn.
    step(&mut state, finished()).unwrap();
    let State::Screens(screens) = state.clone() else {
        panic!("the title has not begun");
    };
    assert!(matches!(
        &screens.screen,
        Screen::Title(title) if title.stage == TitleStage::Loading
    ));
    assert_eq!(screens.counts.finished_frames, 1);
    assert_ne!(screens.counts.random.number, RandomNumber(0x61));
    let two = step(
        &mut state,
        end(Work::FinishedTwice([Pressed::default(); 2])),
    );
    assert_eq!(
        two,
        Err(Fault::TitleLeft),
        "the opening plays on, a later piece"
    );
    assert_eq!(
        state,
        State::Screens(screens),
        "a refused frame changes nothing"
    );
}

#[test]
fn the_options_hand_on_to_the_intro_which_begins_on_the_next_frame() {
    use types::intro::IntroStage;
    use types::options::{OptionsStage, WayOut};
    let menu = types::options::OptionsState {
        stage: OptionsStage::HandedOn(WayOut::IntroBegins),
        ..types::options::OptionsState::begin(Settings::default())
    };
    let mut state = State::Screens(Screens {
        screen: Screen::Options(Options {
            menu,
            begins: ChosenFile::Begins(FileBegins::Intro),
            actor: actor_file(),
        }),
        counts: Counts {
            finished_frames: 5,
            random: NewGame.settled_random(),
        },
        oam_dma: crate::state::video::DmaChannel::default(),
        enemy_staging: crate::production::played::room_video::EnemyStaging::Cleared,
    });
    let intro_stage = |state: &State| match state {
        State::Screens(Screens {
            screen: Screen::Intro(intro),
            ..
        }) => Some(intro.scene.intro.stage),
        State::PowerOn(_) | State::Screens(_) => None,
    };
    assert_eq!(step(&mut state, finished()), Ok(Vec::new()));
    assert_eq!(intro_stage(&state), Some(IntroStage::Begins));
    step(&mut state, end(Work::Unfinished)).unwrap();
    assert_eq!(
        intro_stage(&state),
        Some(IntroStage::Begins),
        "a lag frame is the platform's"
    );
    let sounds = step(&mut state, finished()).unwrap();
    assert_eq!(intro_stage(&state), Some(IntroStage::OpeningLoads));
    assert!(matches!(
        sounds.as_slice(),
        [Sound::IntroMusic(_), Sound::IntroMusic(_)]
    ));
    let State::Screens(screens) = &state else {
        panic!("the intro has not begun");
    };
    assert_eq!(screens.counts.finished_frames, 7);
}

#[test]
fn intro_loan_preserves_energy_other_supplies_and_the_players_controls() {
    use types::save_files::{Ammunition, Control, PadButton, Supply};
    use types::{Beam, Item};
    let mut file = actor_file();
    file.energy = Supply {
        current: 86,
        maximum: 199,
    };
    file.ammunition = enum_map::enum_map! {
        Ammunition::Missiles => Supply { current: 7, maximum: 15 },
        Ammunition::SuperMissiles => Supply { current: 3, maximum: 5 },
        Ammunition::PowerBombs => Supply { current: 1, maximum: 5 },
    };
    file.owned.items[Item::Bombs] = true;
    file.owned.beams[Beam::Charge] = true;
    file.gear.beams[Beam::Charge] = true;
    file.buttons = enum_map::enum_map! {
        Control::Shoot => PadButton::Y,
        Control::Jump => PadButton::B,
        Control::Run => PadButton::A,
        Control::ItemCancel => PadButton::X,
        Control::ItemSelect => PadButton::Select,
        Control::AimDown => PadButton::R,
        Control::AimUp => PadButton::L,
    };
    let mut actors = super::Actors::waiting(file, Settings::default());
    assert_eq!(actors.energy(), file.energy);
    assert_eq!(
        actors.ammunition(Ammunition::Missiles),
        file.ammunition[Ammunition::Missiles]
    );
    actors.equip(screens_intro::Equipment::LENT).unwrap();
    assert_eq!(actors.buttons, super::intro_buttons());
    assert_eq!(actors.energy(), file.energy);
    assert_eq!(
        actors.ammunition(Ammunition::Missiles),
        Supply {
            current: 900,
            maximum: 900
        }
    );
    assert_eq!(
        actors.ammunition(Ammunition::SuperMissiles),
        file.ammunition[Ammunition::SuperMissiles]
    );
    assert_eq!(
        actors.ammunition(Ammunition::PowerBombs),
        file.ammunition[Ammunition::PowerBombs]
    );
    assert_eq!(actors.file, file);
    actors.equip(screens_intro::Equipment::RETURNED).unwrap();
    assert_eq!(actors.buttons, file.buttons);
    assert_eq!(actors.missile_capacity, 0);
    assert_eq!(actors.rooms.unwrap().actor.carry.0.missiles, 0);
}
