//! The `data` crate: portable records, rules and the cost table.

/// A scanline, 0 at the top of the picture.
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub struct Line(pub u16);

/// A position within a line, in dots.
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub struct Dot(pub u16);

/// Where a display change takes effect. The dot is an oracle rule (D11).
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub struct LinePoint {
    /// The line.
    pub line: Line,
    /// The dot within it.
    pub dot: Dot,
}

/// Which layers the main screen shows.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct Screens {
    /// Background layer 1.
    pub bg1: bool,
    /// Background layer 3, the HUD.
    pub bg3: bool,
}

/// The architectural registers the sketch needs.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct Registers {
    /// The main screen's layers.
    pub screens: Screens,
    /// Layer 1's horizontal scroll.
    pub bg1_scroll_x: u16,
}

/// A typed register change.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Change {
    /// The main screen's layers change.
    Screens(Screens),
}

/// The exact display output of one picture.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct DisplayFrame {
    /// The registers in effect at line 0.
    pub start: Registers,
    /// Changes from a point on.
    pub changes: Vec<(LinePoint, Change)>,
}

/// A sprite entry as the drawing code produces it, before the original's OAM encoding.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct SpriteEntry {
    /// Screen X, at full precision.
    pub x: i32,
    /// Screen Y, at full precision.
    pub y: i32,
    /// The tile.
    pub tile: u16,
}

/// Master-clock cycles.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, PartialOrd, Ord)]
pub struct Cycles(pub u32);

impl Cycles {
    /// The sum, saturating: a busy span can't be longer than the clock can count.
    #[must_use]
    pub const fn plus(self, other: Self) -> Self {
        Self(self.0.saturating_add(other.0))
    }
    /// The difference, or `None` if `other` is larger.
    #[must_use]
    pub const fn minus(self, other: Self) -> Option<Self> {
        match self.0.checked_sub(other.0) {
            Some(left) => Some(Self(left)),
            None => None,
        }
    }
}

/// Named costs of the routines whose frame counts depend on CPU time (D9).
#[derive(Clone, Copy, Debug)]
pub struct CostTable {
    /// Per decompressed byte. The real table is per command kind and length.
    pub per_decompressed_byte: Cycles,
    /// The CPU time a frame leaves the main thread, after the NMI, IRQs and HDMA.
    pub frame_budget: Cycles,
}

/// The original's "on screen" window, read by every gameplay rule that tests it.
#[derive(Clone, Copy, Debug)]
pub struct Viewport {
    /// How far left of the camera an object still counts (the original: 1 px).
    pub left_margin: i32,
    /// How far right of the camera's left edge an object still counts (the original: 100h).
    pub width: i32,
}

impl Viewport {
    /// The vanilla window of `$A0:8EB6`.
    pub const VANILLA: Self = Self { left_margin: 1, width: 0x100 };

    /// Whether an object spanning `left..=right` overlaps the window at `camera_x`.
    #[must_use]
    pub const fn overlaps(self, camera_x: i32, left: i32, right: i32) -> bool {
        right.saturating_add(self.left_margin) >= camera_x
            && left <= camera_x.saturating_add(self.width)
    }
}

/// The oracle's measured rules (D19's `OracleRules`).
#[derive(Clone, Copy, Debug)]
pub struct OracleRules {
    /// Where the HUD IRQ's screen write lands on its line.
    pub hud_write: LinePoint,
}

/// Resolved gameplay rules and content, shared read-only by the Session.
#[derive(Clone, Copy, Debug)]
pub struct Content {
    /// The gameplay window.
    pub viewport: Viewport,
    /// The cost model.
    pub costs: CostTable,
    /// The primary oracle's rules.
    pub oracle: OracleRules,
}

impl Content {
    /// A vanilla content set with made-up costs.
    #[must_use]
    pub const fn vanilla() -> Self {
        Self {
            viewport: Viewport::VANILLA,
            costs: CostTable { per_decompressed_byte: Cycles(40), frame_budget: Cycles(300_000) },
            oracle: OracleRules { hud_write: LinePoint { line: Line(0x1F), dot: Dot(1024) } },
        }
    }
}
