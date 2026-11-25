use super::{Answers, Fault, FileStart, FrameEnd, Pressed, Rules, Screen, Sound, State, Work};
use screens_boot::{DriverUpload, FinishedFrames, Level, PowerOnStage, ScreenLight};
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

impl Answers for NewGame {
    fn settled_random(&self) -> RandomSequence {
        RandomSequence {
            number: RandomNumber(0x61),
        }
    }

    fn file(&self, _: Slot) -> Option<FileStart> {
        Some(FileStart {
            settings: Settings::default(),
            begins: ChosenFile::Begins(FileBegins::Intro),
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

fn step(state: &mut State, frame: FrameEnd) -> Result<Vec<Sound>, Fault> {
    state.step(frame, &Rules::REFERENCE, &NewGame)
}

fn power_on(state: State) -> screens_boot::PowerOnState {
    match state {
        State::PowerOn(power_on) => power_on,
        State::Screens(_) => panic!("the title has begun"),
    }
}

#[test]
fn the_boot_runs_from_power_on_until_the_title_begins() {
    let mut state = State::AT_POWER_ON;
    for _ in 0..17 {
        step(&mut state, end(Work::Unfinished)).unwrap();
    }
    assert_eq!(power_on(state).stage, PowerOnStage::BeforeLogo);
    step(&mut state, finished()).unwrap();
    let lit = power_on(state);
    assert_eq!(lit.screen, ScreenLight::Lit(Level::new(1).unwrap()));
    assert_eq!(lit.frames, FinishedFrames(1));
    // The logo brightens, holds and dims on finished frames, then the sound loads.
    while power_on(state).stage != PowerOnStage::SoundLoading {
        step(&mut state, finished()).unwrap();
    }
    step(&mut state, end(Work::Unfinished)).unwrap();
    let ended = FrameEnd {
        work: Work::Unfinished,
        upload: DriverUpload::Ended,
    };
    step(&mut state, ended).unwrap();
    assert_eq!(power_on(state).stage, PowerOnStage::Settling);
    assert_eq!(power_on(state).frames, FinishedFrames::ZERO);
    // The settled game's first finished frame is the title's, counted and drawn.
    step(&mut state, finished()).unwrap();
    let State::Screens(screens) = state else {
        panic!("the title has not begun");
    };
    assert!(matches!(
        screens.screen,
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
