//! Map-menu motion owned by the committed menu authority.
#![forbid(unsafe_code)]
/// Fractional/whole words of a window edge or its velocity.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct Position {
    pub fraction: u16,
    pub whole: u16,
}

/// A complete supplied word image, including fractional positions and velocities.
/// Initialization certifies supplied bytes; it does not infer an active animation.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct WindowState {
    initialized: bool,
    pub right_margin: u16,
    pub left_margin: u16,
    pub positions: [Position; 4],
    pub velocities: [Position; 4],
    pub timer: u16,
}
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Field {
    RightMargin,
    LeftMargin,
    Fraction(usize),
    Whole(usize),
    VelocityFraction(usize),
    VelocityWhole(usize),
    Timer,
}
impl Field {
    pub const ALL: [Self; 19] = [
        Self::RightMargin,
        Self::LeftMargin,
        Self::Fraction(0),
        Self::Whole(0),
        Self::Fraction(1),
        Self::Whole(1),
        Self::Fraction(2),
        Self::Whole(2),
        Self::Fraction(3),
        Self::Whole(3),
        Self::VelocityFraction(0),
        Self::VelocityWhole(0),
        Self::VelocityFraction(1),
        Self::VelocityWhole(1),
        Self::VelocityFraction(2),
        Self::VelocityWhole(2),
        Self::VelocityFraction(3),
        Self::VelocityWhole(3),
        Self::Timer,
    ];
    pub fn get(self, state: &WindowState) -> u16 {
        match self {
            Self::RightMargin => state.right_margin,
            Self::LeftMargin => state.left_margin,
            Self::Fraction(i) => state.positions[i].fraction,
            Self::Whole(i) => state.positions[i].whole,
            Self::VelocityFraction(i) => state.velocities[i].fraction,
            Self::VelocityWhole(i) => state.velocities[i].whole,
            Self::Timer => state.timer,
        }
    }
    pub fn get_mut(self, state: &mut WindowState) -> &mut u16 {
        match self {
            Self::RightMargin => &mut state.right_margin,
            Self::LeftMargin => &mut state.left_margin,
            Self::Fraction(i) => &mut state.positions[i].fraction,
            Self::Whole(i) => &mut state.positions[i].whole,
            Self::VelocityFraction(i) => &mut state.velocities[i].fraction,
            Self::VelocityWhole(i) => &mut state.velocities[i].whole,
            Self::Timer => &mut state.timer,
        }
    }
}
impl WindowState {
    pub fn initialized(
        right_margin: u16,
        left_margin: u16,
        positions: [Position; 4],
        velocities: [Position; 4],
        timer: u16,
    ) -> Self {
        Self {
            initialized: true,
            right_margin,
            left_margin,
            positions,
            velocities,
            timer,
        }
    }
    pub fn is_initialized(&self) -> bool {
        self.initialized
    }
    pub fn decrement_timer(&mut self) -> bool {
        self.timer = self.timer.wrapping_sub(1);
        self.timer & 0x8000 != 0
    }
    /// Native position reads/stores and LE velocity/limit reads remain distinct.
    pub fn advance_edge(&mut self, edge: usize, contract: bool) {
        let velocity = self.velocities[edge];
        let delta = u32::from(velocity.fraction) | (u32::from(velocity.whole) << 16);
        // Keep the original signed-negation overflow before either position store.
        let delta = if contract {
            u32::from_ne_bytes((-i32::from_ne_bytes(delta.to_ne_bytes())).to_ne_bytes())
        } else {
            delta
        };
        let position = self.positions[edge];
        let low = u16::from_ne_bytes(position.fraction.to_le_bytes());
        let high = u16::from_ne_bytes(position.whole.to_le_bytes());
        let sum = (u32::from(low) | (u32::from(high) << 16)).wrapping_add(delta);
        let bytes = sum.to_ne_bytes();
        let (low, high) = if cfg!(target_endian = "little") {
            ([bytes[0], bytes[1]], [bytes[2], bytes[3]])
        } else {
            ([bytes[2], bytes[3]], [bytes[0], bytes[1]])
        };
        self.positions[edge] = Position {
            fraction: u16::from_le_bytes(low),
            whole: u16::from_le_bytes(high),
        };
    }
    pub fn clamp_edge(&mut self, edge: usize) -> bool {
        let (boundary, limit, minimum) = [
            (1, 1, true),
            (256, 255, false),
            (1, 1, true),
            (224, 224, false),
        ][edge];
        let below = i16::from_ne_bytes(
            self.positions[edge]
                .whole
                .wrapping_sub(boundary)
                .to_ne_bytes(),
        ) < 0;
        if below == minimum {
            self.positions[edge].whole = limit;
            true
        } else {
            false
        }
    }
}
/// Reads select the current committed authority. Each publication preserves its
/// ordered unconditional byte stores and revokes on a partially failed effect.
pub trait Access {
    fn window_state(&self) -> WindowState;
    fn publish_window(&mut self, value: WindowState, fields: &[Field]);
}
impl Access for WindowState {
    fn window_state(&self) -> WindowState {
        *self
    }
    fn publish_window(&mut self, value: WindowState, fields: &[Field]) {
        for &field in fields {
            *field.get_mut(self) = field.get(&value);
        }
    }
}

#[derive(Clone, Copy)]
pub enum ArrowAction {
    Reset(u16),
    Advance,
}
/// Exact ordered timer publications; an expiry writes zero before its reset.
pub struct ArrowChange {
    pub publications: [Option<u16>; 2],
    pub rotate: bool,
}
pub fn update_arrow(timer: &mut u16, action: ArrowAction) -> ArrowChange {
    match action {
        ArrowAction::Reset(value) => {
            *timer = value;
            ArrowChange {
                publications: [Some(value), None],
                rotate: false,
            }
        }
        ArrowAction::Advance if *timer == 0 => ArrowChange {
            publications: [None, None],
            rotate: false,
        },
        ArrowAction::Advance => {
            *timer = timer.wrapping_sub(1);
            let first = *timer;
            let rotate = first == 0;
            if rotate {
                *timer = 4;
            }
            ArrowChange {
                publications: [Some(first), rotate.then_some(4)],
                rotate,
            }
        }
    }
}
pub trait ArrowAccess {
    fn file_arrow(&mut self, action: ArrowAction) -> bool;
}
