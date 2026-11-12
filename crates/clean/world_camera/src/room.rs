//! The room as the camera reads it (Spec: §1.3): its screens, run on as one strip, and its aim
//! heights.
//!
//! A screen's kind is the shared `ScreenKind`, which the blocks' scroll cells write. The camera
//! tells only two kinds from the rest: `Closed`, which the view may not show (Spec:
//! ClosedScreens), and `Short`, where the view that can go no lower stops with its top on the
//! screen's top edge (Spec: ShortScreens). Every other screen is open to it: the view may show all
//! of it (Spec: §1.3, Screen kinds).

use core::num::Wrapping;

use types::{Pair, Px, ScreenKind};

/// Notation: a screen is 256 px square.
pub(crate) const SCREEN: Px = Wrapping(256);

/// The room's two aim heights: how far below the view's top edge it keeps her, in px (Spec:
/// AimingHeight).
#[derive(Clone, Copy, Debug)]
pub struct Heights {
    /// The height while she isn't rising (the spec's hn).
    pub while_not_rising: Px,
    /// The height while she rises (the spec's hr).
    pub while_rising: Px,
}

impl Heights {
    /// The height the view keeps her at, by whether she is `rising`.
    pub(crate) const fn height(self, rising: bool) -> Px {
        if rising {
            self.while_rising
        } else {
            self.while_not_rising
        }
    }
}

/// Why a room can't be built from the run given.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum RoomError {
    /// The run holds fewer screens than the room, its width times its height.
    MissingScreens,
    /// The room is no screens across or down.
    NoScreens,
    /// The room is more than 256 screens across or down: past that its positions would not fit
    /// the 16-bit pixel part.
    TooLarge,
}

/// The room as the camera reads it, as its loading supplies it (Spec: §1.3, §5).
#[derive(Clone, Debug)]
pub struct Room {
    /// How many screens the room is across (the spec's w).
    width: usize,
    /// The near edges of its last column and its last row: across, xr, the furthest right the
    /// view goes; down, the last row's top edge, to which the camera adds the margin of the
    /// screen it reads, giving yb, the furthest down the view goes (Spec: RoomEdges).
    pub(crate) last: Pair<Px>,
    /// The run of screens: the room's rows, top first, each left to right, then the screens below
    /// the room continuing it (Spec: ScreensRunOn, BelowTheRoom).
    run: Vec<ScreenKind>,
    /// Its aim heights.
    pub(crate) heights: Heights,
}

impl Room {
    /// A room of `size` screens, `size.x` across (the spec's w) and `size.y` down (h), from its
    /// `run` of screens: its rows, top first, each left to right, then the screens the room's
    /// loading left after its last, which continue the run below it, a row of w screens at a
    /// time (Spec: BelowTheRoom).
    ///
    /// # Errors
    ///
    /// When it is no screens across or down, more than 256 across or down, or the run holds
    /// fewer screens than the room.
    pub fn new(
        run: Vec<ScreenKind>,
        size: Pair<usize>,
        heights: Heights,
    ) -> Result<Self, RoomError> {
        let last = Pair {
            x: last_edge(size.x)?,
            y: last_edge(size.y)?,
        };
        if size
            .x
            .checked_mul(size.y)
            .is_none_or(|room| run.len() < room)
        {
            return Err(RoomError::MissingScreens);
        }
        Ok(Self {
            width: size.x,
            last,
            run,
            heights,
        })
    }

    /// The kind of the screen in the column holding `at.x` and the row holding `at.y`, each read
    /// as 0 to FFFF, 256 px a screen (Spec: §1.3, Which screens it looks at).
    ///
    /// Spec: ScreensRunOn. Reproduces the original's behaviour: the screens run on as one strip,
    /// the screen in a column and a row being the run's row × w + column, so a screen k columns
    /// after a row's last is the kth of the row below, and from the room's last row the first
    /// below the room.
    ///
    /// Spec: BelowTheRoom. Reproduces the original's behaviour: below the room's last row the
    /// run goes on into the screens the room's loading left after its last; a screen past the
    /// run's end is closed (Spec: Open questions, 1).
    pub(crate) fn screen(&self, at: Pair<Px>) -> ScreenKind {
        screen_number(at.y)
            .checked_mul(self.width)
            .and_then(|row| row.checked_add(screen_number(at.x)))
            .and_then(|place| self.run.get(place).copied())
            .unwrap_or(ScreenKind::Closed)
    }
}

/// The near edge of the last of `count` screens on an axis, (count − 1) × 256, for a count from 1
/// to 256.
fn last_edge(count: usize) -> Result<Px, RoomError> {
    let last = count.checked_sub(1).ok_or(RoomError::NoScreens)?;
    let last = u8::try_from(last).map_err(|_| RoomError::TooLarge)?;
    Ok(Wrapping(u16::from(last)) * SCREEN)
}

/// The number of the screen holding a position on an axis: its pixel part's high byte, 256 px a
/// screen.
fn screen_number(p: Px) -> usize {
    let [number, _] = p.0.to_be_bytes();
    usize::from(number)
}
