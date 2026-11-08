//! The room as the camera reads it (Spec: §1.3): its screens, run on as one strip, and its aim
//! heights.

use core::iter::repeat;
use core::num::Wrapping;

use clean_types::Px;

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

/// Why a room can't be built from the rows given.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum RoomError {
    /// A row below the room holds more screens than the room is across.
    BelowRowTooWide,
    /// The room has no screens.
    NoScreens,
    /// The room is more than 256 screens across or down: past that its positions would not fit
    /// the 16-bit pixel part.
    TooLarge,
    /// The room's rows don't all hold the same number of screens.
    UnevenRows,
}

/// The room as the camera reads it, as its loading supplies it (Spec: §1.3, §5).
///
/// Its screens are held as the one strip they run on as (Spec: ScreensRunOn): the room's rows,
/// top first, each left to right, then the rows the room's loading left below it, each read as a
/// full row (Spec: BelowTheRoom).
#[derive(Clone, Debug)]
pub struct Room {
    /// How many screens the room is across (the spec's w).
    width: usize,
    /// The near edge of its last column, the furthest right the view goes (Spec: RoomEdges).
    right_edge: Px,
    /// The top edge of its last row (Spec: RoomEdges).
    last_row_edge: Px,
    /// The screens in the order they run on.
    strip: Vec<ScreenKind>,
    /// Its aim heights.
    heights: Heights,
}

impl Room {
    /// A room from its rows of screens, top row first, each left to right, the rows its loading
    /// left below it, and its aim heights.
    ///
    /// # Errors
    ///
    /// When it has no screens, its rows differ in length, a row below it is longer than it is
    /// across, or it is more than 256 screens across or down.
    pub fn new(
        screens: &[Vec<ScreenKind>],
        below: &[Vec<ScreenKind>],
        heights: Heights,
    ) -> Result<Self, RoomError> {
        let width = screens.first().map_or(0, Vec::len);
        let right_edge = last_edge(width)?;
        let last_row_edge = last_edge(screens.len())?;
        if screens.iter().any(|row| row.len() != width) {
            return Err(RoomError::UnevenRows);
        }
        if below.iter().any(|row| row.len() > width) {
            return Err(RoomError::BelowRowTooWide);
        }
        let strip = screens
            .iter()
            .flatten()
            .copied()
            .chain(below_the_room(below, width))
            .collect();
        Ok(Self {
            width,
            right_edge,
            last_row_edge,
            strip,
            heights,
        })
    }

    /// The room's aim heights.
    pub(crate) const fn heights(&self) -> Heights {
        self.heights
    }

    /// The near edge of the room's last column, (w − 1) × 256 (Spec: RoomEdges, xr).
    pub(crate) const fn right_edge(&self) -> Px {
        self.right_edge
    }

    /// The top edge of the room's last row, (h − 1) × 256 (Spec: RoomEdges, yb before
    /// ShortScreens' rows).
    pub(crate) const fn last_row_edge(&self) -> Px {
        self.last_row_edge
    }

    /// The kind of the screen in the column holding position `x` and the row holding position
    /// `y`, each read as 0 to FFFF, 256 px a screen (Spec: §1.3, Which screens it looks at).
    ///
    /// A screen past the strip, below the rows the room's loading left, is closed (Spec:
    /// BelowTheRoom; Open questions, 1).
    pub(crate) fn screen(&self, x: Px, y: Px) -> ScreenKind {
        screens_run_on(screen_number(x), screen_number(y), self.width)
            .and_then(|place| self.strip.get(place).copied())
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

/// Spec: ScreensRunOn. The place in the strip of the screen `column` across and `row` down,
/// row × w + column; none past what the platform can count.
///
/// Reproduces the original's behaviour: the screens run on as one strip, so a screen k columns
/// after a row's last is the kth screen of the row below, and after the room's last row the strip
/// runs on into the rows below the room.
fn screens_run_on(column: usize, row: usize, width: usize) -> Option<usize> {
    row.checked_mul(width)?.checked_add(column)
}

/// Spec: BelowTheRoom. The rows the room's loading left below its last row, top first, each read
/// as a full row of `width` screens, a screen the row doesn't give closed.
///
/// Reproduces the original's behaviour: below the room's last row, the view reads the screens the
/// room's loading left there, from earlier rooms.
fn below_the_room(below: &[Vec<ScreenKind>], width: usize) -> impl Iterator<Item = ScreenKind> {
    below.iter().flat_map(move |row| {
        row.iter()
            .copied()
            .chain(repeat(ScreenKind::Closed))
            .take(width)
    })
}
