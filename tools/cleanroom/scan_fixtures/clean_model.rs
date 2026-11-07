//! A synthetic clean model: a runner's horizontal speed, in player terms.

/// How the runner's speed is changing.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum Pace {
    /// Getting faster.
    #[default]
    Faster,
    /// Holding steady at the top speed.
    Steady,
    /// Slowing to a stop.
    Slower,
}

/// Which way the runner faces.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Heading {
    /// Towards the left edge of the screen.
    Left,
    /// Towards the right edge.
    Right,
}

/// The parameters of one way of moving.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Gait {
    /// Speed gained per frame while the direction is held.
    pub gain: u32,
    /// The highest speed.
    pub top: u32,
    /// Speed lost per frame once the direction is released.
    pub loss: u32,
}

/// A runner on flat ground.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Runner {
    /// Horizontal position, in sixteenths of a pixel.
    pub x: u32,
    /// Current speed.
    pub speed: u32,
    /// Its trend.
    pub pace: Pace,
    /// Which way it faces.
    pub heading: Heading,
}

impl Runner {
    /// One frame: speed up while the pad is held the way she faces, otherwise slow down.
    #[must_use]
    pub fn advance(self, gait: Gait, held: Option<Heading>) -> Self {
        let pushing = held == Some(self.heading);
        let speed = if pushing {
            self.speed.saturating_add(gait.gain).min(gait.top)
        } else {
            self.speed.saturating_sub(gait.loss)
        };
        let pace = match (pushing, speed == gait.top) {
            (true, true) => Pace::Steady,
            (true, false) => Pace::Faster,
            (false, _) => Pace::Slower,
        };
        let x = match self.heading {
            Heading::Left => self.x.saturating_sub(speed),
            Heading::Right => self.x.saturating_add(speed),
        };
        Self { x, speed, pace, heading: self.heading }
    }
}
