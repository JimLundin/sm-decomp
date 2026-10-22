//! Motion retained between Samus movement and projectile processing.
//! Recording replaces one direction; firing observes without consuming; the
//! end of projectile processing clears all four directions for later movement.
#![forbid(unsafe_code)]

#[derive(Clone, Copy)]
pub(crate) enum MovementDirection {
    Left,
    Right,
    Up,
    Down,
}

#[derive(Clone, Copy)]
pub(crate) enum Aim {
    Up,
    UpRight,
    Right,
    DownRight,
    Down,
    DownLeft,
    Left,
    UpLeft,
}

#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub(crate) struct LaunchMotion {
    /// Signed 16.16 displacements, in left/right/up/down order.
    pub(super) displacements: [i32; 4],
}

#[derive(Debug, PartialEq)]
pub(crate) struct LaunchVelocity {
    pub x: i16,
    pub y: i16,
}

impl LaunchMotion {
    pub(crate) fn record_movement(&mut self, direction: MovementDirection, displacement: i32) {
        self.displacements[direction as usize] = displacement;
    }

    pub(crate) fn finish_projectile_phase(&mut self) {
        *self = Self::default();
    }

    /// Produce signed 8.8 velocity. Absolute Y travel has a separate lifetime
    /// (camera accounting); sample its fraction when launching, even after clear.
    pub(crate) fn launch(&self, aim: Aim, speed: u16, absolute_y_fraction: u16) -> LaunchVelocity {
        let [left, right, up, down] = self.displacements;
        // Original inheritance joins the preceding direction's fraction with
        // this direction's integer byte. Preserve that coupling explicitly.
        let inherited = |previous_fraction: u16, current: i32| {
            (previous_fraction >> 8) | (((current as u32 >> 16) as u16 & 0xff) << 8)
        };
        let y = match aim {
            Aim::Up | Aim::UpRight | Aim::UpLeft => {
                let motion = if (up as u32 >> 16) as u8 != 0 {
                    (inherited(right as u16, up) >> 2) | 0xc000
                } else {
                    0
                };
                motion.wrapping_sub(speed)
            }
            Aim::Down | Aim::DownRight | Aim::DownLeft => {
                inherited(up as u16, down).wrapping_add(speed)
            }
            Aim::Left | Aim::Right => 0,
        };
        let x = match aim {
            Aim::UpRight | Aim::Right | Aim::DownRight => {
                inherited(left as u16, right).wrapping_add(speed)
            }
            Aim::DownLeft | Aim::Left | Aim::UpLeft => {
                inherited(absolute_y_fraction, left).wrapping_sub(speed)
            }
            Aim::Up | Aim::Down => 0,
        };
        LaunchVelocity {
            x: x as i16,
            y: y as i16,
        }
    }
}
