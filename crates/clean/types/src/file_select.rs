//! File select's State: its stage, the screen, and the menu with its cursor and its fade.
//!
//! The game carries it from one frame end to the next, from the frame end after the title's
//! `FileSelectBegins` until a way out, and may save and restore it.

use enum_map::Enum;
use serde::{Deserialize, Serialize};

use crate::power_on::ScreenLight;

/// Spec: `MenuBrightens`. The fewest finished frames per level, 1.
const MENU_PACE_FEWEST: u16 = 1;
/// Spec: `MenuBrightens`. The most finished frames per level, 256.
const MENU_PACE_MOST: u16 = 256;
/// Spec: `MenuBrightens`. The reference menu pace, 2 finished frames per level.
const REFERENCE_MENU_PACE: u16 = 2;
/// Spec: `HelmetTurns`. The reference helmet turn, 57 frames of work.
const REFERENCE_HELMET_TURN: u8 = 57;

/// Spec: `MenuLoads`. The load's finished frame ends so far, before its fourth: 0 to 3. The
/// fourth makes the menu ready, and the load's last step (step 4) is `MenuPhase::LoadEnding`.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum LoadStep {
    /// No finished frame end yet: where file select begins.
    Step0,
    /// One finished frame end.
    Step1,
    /// Two finished frame ends.
    Step2,
    /// Three finished frame ends.
    Step3,
}

/// One of the three slots, the menu's top three items.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Enum, Serialize, Deserialize)]
pub enum Slot {
    /// Slot A, the top item.
    First,
    /// Slot B.
    Second,
    /// Slot C.
    Third,
}

/// An item of file select's menu: what the cursor is on.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum MenuItem {
    /// The file in one of the three slots.
    File(Slot),
    /// Copy a file.
    Copy,
    /// Clear a file.
    Clear,
    /// Back to the title.
    Exit,
}

impl MenuItem {
    /// Spec: `CursorMoves`. The menu's items, top to bottom.
    pub const TOP_TO_BOTTOM: [Self; 6] = [
        Self::File(Slot::First),
        Self::File(Slot::Second),
        Self::File(Slot::Third),
        Self::Copy,
        Self::Clear,
        Self::Exit,
    ];
}

/// Spec: `CursorStarts`. Whether copy and clear are on offer: they are when some slot holds a
/// file.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum CopyAndClear {
    /// Some slot holds a file: the cursor may rest on copy and clear.
    Offered,
    /// No slot holds a file: the cursor passes copy and clear by.
    NotOffered,
}

/// Spec: `MenuBrightens`, `MenuDims`. Finished frames before the fade's next level, from 0 to
/// the menu pace less one.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct FadeWait(pub u16);

/// Spec: `MenuBrightens`. The menu pace: finished frames per level of the fade, 1 to 256.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct MenuPace(u16);

impl MenuPace {
    /// The reference pace: a level every 2 finished frames.
    pub const REFERENCE: Self = Self(REFERENCE_MENU_PACE);

    /// The pace of this many finished frames per level, if it is one (1 to 256).
    #[must_use]
    pub const fn new(frames_per_level: u16) -> Option<Self> {
        if frames_per_level >= MENU_PACE_FEWEST && frames_per_level <= MENU_PACE_MOST {
            Some(Self(frames_per_level))
        } else {
            None
        }
    }

    /// Its finished frames per level.
    #[must_use]
    pub const fn get(self) -> u16 {
        self.0
    }

    /// The wait that starts each level: the pace less one, as the level itself takes a frame.
    #[must_use]
    pub const fn wait(self) -> FadeWait {
        FadeWait(self.0.saturating_sub(1))
    }
}

/// Spec: `HelmetTurns`. The helmet turn: frames of work the chosen file's helmet turns, 1 to 255.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct HelmetTurn(u8);

impl HelmetTurn {
    /// The reference turn: 57 frames of work.
    pub const REFERENCE: Self = Self(REFERENCE_HELMET_TURN);

    /// The turn of this many frames of work, if it is one (1 to 255).
    #[must_use]
    pub const fn new(frames: u8) -> Option<Self> {
        if frames == 0 {
            None
        } else {
            Some(Self(frames))
        }
    }

    /// Its frames of work.
    #[must_use]
    pub const fn get(self) -> u8 {
        self.0
    }
}

/// Spec: `HelmetTurns`. Frames of work left of the chosen helmet's turn, 1 to the helmet turn.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(try_from = "u8", into = "u8")]
pub struct TurnLeft(u8);

impl TurnLeft {
    /// The frames left of this number, if it is one (1 to 255).
    #[must_use]
    pub const fn new(frames: u8) -> Option<Self> {
        if frames == 0 {
            None
        } else {
            Some(Self(frames))
        }
    }

    /// The whole turn, as a file is chosen.
    #[must_use]
    pub const fn whole(turn: HelmetTurn) -> Self {
        Self(turn.0)
    }

    /// Its frames left.
    #[must_use]
    pub const fn get(self) -> u8 {
        self.0
    }

    /// One frame of work of the turn: the frames then left, none once the turn has ended.
    #[must_use]
    pub const fn counted_down(self) -> Option<Self> {
        Self::new(self.0.saturating_sub(1))
    }
}

impl TryFrom<u8> for TurnLeft {
    type Error = &'static str;

    fn try_from(frames: u8) -> Result<Self, Self::Error> {
        Self::new(frames).ok_or("a helmet's turn left must be 1 to 255 frames")
    }
}

impl From<TurnLeft> for u8 {
    fn from(left: TurnLeft) -> Self {
        left.get()
    }
}

/// Spec: `ItemPressed`, `CursorMoves`, `MenuDims`. A way out of file select, to the piece that
/// follows it.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum WayOut {
    /// The options menu begins, the file chosen.
    OptionsBegin,
    /// Back to the title.
    TitleReturns,
    /// Copying a file begins.
    CopyBegins,
    /// Clearing a file begins.
    ClearBegins,
}

/// Where the menu stands, from the load's fourth finished frame end on.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum MenuPhase {
    /// Spec: `MenuLoads`. Load step 4: the menu is made ready, lit at level 0, and the load's last
    /// finished frame end is to come.
    LoadEnding,
    /// Spec: `MenuBrightens`. The menu fading in.
    Brightening,
    /// Spec: `ItemPressed`, `CursorMoves`. At full, the cursor in the player's hands.
    Choosing,
    /// Spec: `HelmetTurns`. A file chosen, its helmet turning.
    Confirming {
        /// The file chosen.
        chosen: Slot,
        /// Frames of work left of its helmet's turn.
        turn_left: TurnLeft,
    },
    /// Spec: `MenuDims`. The menu fading out.
    Leaving,
    /// File select is over: the way it went out.
    Out(WayOut),
}

/// The menu, as the load's fourth finished frame end lays it out.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct MenuState {
    /// Where the menu stands.
    pub phase: MenuPhase,
    /// The item the cursor is on.
    pub cursor: MenuItem,
    /// Whether copy and clear are on offer.
    pub copy_and_clear: CopyAndClear,
    /// Finished frames before the fade's next level.
    pub fade_wait: FadeWait,
}

/// Where file select stands.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum FileSelectStage {
    /// Spec: `MenuLoads`. Black while the load runs, before the menu is made ready.
    Loading(LoadStep),
    /// The menu, made ready.
    Menu(MenuState),
}

/// File select's complete State. A caller may set any stage, menu and screen, read them back,
/// carry them forward and restore them.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct FileSelectState {
    /// Where file select stands.
    pub stage: FileSelectStage,
    /// The screen at the last frame end.
    pub screen: ScreenLight,
}

impl FileSelectState {
    /// The State as file select begins: the title's screen, and the load at step 0.
    #[must_use]
    pub const fn begin(title_screen: ScreenLight) -> Self {
        Self {
            stage: FileSelectStage::Loading(LoadStep::Step0),
            screen: title_screen,
        }
    }
}
