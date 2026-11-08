//! The tuning the camera reads: `Rules`, whose reference profile holds the spec's vanilla values.

use core::num::Wrapping;

use clean_types::{Fixed, Px};

/// `LeadingHer`: a lead setting's leads, how far her x is from the view's left edge.
#[derive(Clone, Copy, Debug)]
pub struct Leads {
    /// `lr`, when she faces right or the screen.
    pub right: Px,
    /// `ll`, when she faces left.
    pub left: Px,
}

/// The camera's tuning parameters.
#[derive(Clone, Copy, Debug)]
pub struct Rules {
    /// `FollowingHer`: the extra pixel on top of how far she moved.
    pub extra: Fixed,
    /// `LeadingHer`: the leads of `Ahead`.
    pub ahead: Leads,
    /// `LeadingHer`: the leads of `KeepLeft`.
    pub keep_left: Leads,
    /// `LeadingHer`: the leads of `KeepRight`.
    pub keep_right: Leads,
    /// `LeadingHer`: the leads of `Near`.
    pub near: Leads,
    /// `ClosedScreens`: the push, beyond the speed.
    pub push: Px,
    /// `ShortScreens`: how much lower than on a short screen the view stops on any other.
    pub short_rows: Px,
}

impl Rules {
    /// The reference profile: the spec's vanilla values.
    pub const REFERENCE: Self = Self {
        extra: Wrapping(0x1_0000),
        ahead: Leads {
            right: Wrapping(96),
            left: Wrapping(160),
        },
        keep_left: Leads {
            right: Wrapping(32),
            left: Wrapping(32),
        },
        keep_right: Leads {
            right: Wrapping(224),
            left: Wrapping(224),
        },
        near: Leads {
            right: Wrapping(64),
            left: Wrapping(80),
        },
        push: Wrapping(2),
        short_rows: Wrapping(31),
    };
}
