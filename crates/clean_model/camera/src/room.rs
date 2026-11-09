//! The room as the camera reads it (Spec: §1.3): its screens, run on as one strip, and its aim
//! heights.

use core::num::Wrapping;

use clean_types::{Pair, Px};

/// Notation: a screen is 256 px square.
pub(crate) const SCREEN: Px = Wrapping(256);

/// The kind of a screen of the room (Spec: §1.3, Screen kinds). Listed alphabetically; the order
/// means nothing.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum ScreenKind {
    /// The view may not show it; a view that would is pushed back to its edge (Spec:
    /// ClosedScreens).
    Closed,
    /// The view may show all of it.
    Open,
    /// Like `Open`, but where the view can go no lower it stops with its top on the screen's top
    /// edge (Spec: ShortScreens).
    Short,
}

impl ScreenKind {
    /// Whether the view may not show this screen.
    pub(crate) const fn is_closed(self) -> bool {
        matches!(self, Self::Closed)
    }

    /// Whether the view stops with its top on this screen's top edge where it can go no lower.
    pub(crate) const fn is_short(self) -> bool {
        matches!(self, Self::Short)
    }
}

/// The room's two aim heights: how far below the view's top edge it keeps her, in px (Spec:
/// AimingHeight).
#[derive(Clone, Copy, Debug)]
pub struct Heights {
    /// While she isn't rising (the spec's hn).
    pub level: Px,
    /// While she rises (the spec's hr).
    pub rising: Px,
}

impl Heights {
    /// The height the view keeps her at: the rising one while she rises, the level one otherwise.
    pub(crate) const fn height(self, rising: bool) -> Px {
        if rising { self.rising } else { self.level }
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
    /// The near edges of its last column and its last row: across, the furthest right the view
    /// goes, xr; down, yb less the rows (Spec: RoomEdges).
    pub(crate) last: Pair<Px>,
    /// The run of screens: the room's rows, top first, each left to right, then the screens below
    /// the room continuing it (Spec: ScreensRunOn, BelowTheRoom).
    run: Vec<ScreenKind>,
    /// Its aim heights.
    pub(crate) heights: Heights,
}

impl Room {
    /// A room `width` screens across and `height` rows down, from its `run` of screens: its rows,
    /// top first, each left to right, then the screens the room's loading left after its last,
    /// which continue the run below it, a row of `width` screens at a time (Spec: BelowTheRoom).
    ///
    /// # Errors
    ///
    /// When it is no screens across or down, more than 256 across or down, or the run holds
    /// fewer screens than the room.
    pub fn new(
        run: Vec<ScreenKind>,
        width: usize,
        height: usize,
        heights: Heights,
    ) -> Result<Self, RoomError> {
        let last = Pair {
            x: last_edge(width)?,
            y: last_edge(height)?,
        };
        if width
            .checked_mul(height)
            .is_none_or(|room| run.len() < room)
        {
            return Err(RoomError::MissingScreens);
        }
        Ok(Self {
            width,
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
