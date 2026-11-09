//! The tuning the camera reads: `Rules`, whose reference profile holds the spec's vanilla values.

use core::num::Wrapping;

use clean_types::{Facing, Fixed, PIXEL, Px};

use crate::input::LeadSetting;

/// A lead setting's two leads: how far her x is from the view's left edge, by the way she
/// travels (Spec: LeadingHer).
#[derive(Clone, Copy, Debug)]
pub struct Leads {
    /// When she travels right, or faces the screen going forwards (the spec's lr).
    pub right: Px,
    /// When she travels left (the spec's ll).
    pub left: Px,
}

impl Leads {
    /// The lead for the way she travels: facing the screen leads as facing right.
    const fn toward(self, travel: Facing) -> Px {
        match travel {
            Facing::Left => self.left,
            Facing::Right | Facing::Screen => self.right,
        }
    }
}

/// The camera's tuning parameters.
#[derive(Clone, Copy, Debug)]
pub struct Rules {
    /// The extra pixel the view moves on top of how far she moved (Spec: FollowingHer).
    pub extra: Fixed,
    /// The leads of `LeadSetting::Ahead` (Spec: LeadingHer).
    pub ahead: Leads,
    /// The leads of `LeadSetting::KeepLeft` (Spec: LeadingHer).
    pub keep_left: Leads,
    /// The leads of `LeadSetting::KeepRight` (Spec: LeadingHer).
    pub keep_right: Leads,
    /// The leads of `LeadSetting::Near` (Spec: LeadingHer).
    pub near: Leads,
    /// How far a push moves the view beyond its speed (Spec: ClosedScreens).
    pub push: Px,
    /// The rows of any screen but a short one: how far below its top edge the view stops where
    /// it can go no lower, a short screen's being 0 (Spec: ShortScreens).
    pub rows_not_short: Px,
}

impl Rules {
    /// The lead a lead setting gives for the way she travels (Spec: LeadingHer).
    pub(crate) const fn lead(&self, setting: LeadSetting, travel: Facing) -> Px {
        let leads = match setting {
            LeadSetting::Ahead => self.ahead,
            LeadSetting::KeepLeft => self.keep_left,
            LeadSetting::KeepRight => self.keep_right,
            LeadSetting::Near => self.near,
        };
        leads.toward(travel)
    }

    /// The reference profile: the spec's vanilla values.
    pub const REFERENCE: Self = Self {
        extra: PIXEL,
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
        rows_not_short: Wrapping(31),
    };
}
