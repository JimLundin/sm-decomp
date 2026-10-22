//! Animation behavior independent of encoding, transporting the complete owner.
#![forbid(unsafe_code)]

#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct Animation<C, S> {
    pub cursor: C,
    pub remaining: u16,
    pub sprite: S,
    pub x_radius: u16,
    pub y_radius: u16,
}

#[derive(Clone, Copy)]
pub enum Write<C, S> {
    Timer(u16),
    Cursor(C),
    Sprite(S),
    XRadius(u16),
    YRadius(u16),
}
pub trait Owner: Sized {
    type Cursor: Copy;
    type Sprite: Copy;
    type Slot: Copy;
    fn remaining(&self, slot: Self::Slot) -> u16;
    fn cursor(&self, slot: Self::Slot) -> Self::Cursor;
    fn write(&mut self, slot: Self::Slot, value: Write<Self::Cursor, Self::Sprite>);
}
impl<C: Copy, S: Copy> Owner for Animation<C, S> {
    type Cursor = C;
    type Sprite = S;
    type Slot = ();
    fn remaining(&self, _: ()) -> u16 {
        self.remaining
    }
    fn cursor(&self, _: ()) -> C {
        self.cursor
    }
    fn write(&mut self, _: (), value: Write<C, S>) {
        match value {
            Write::Timer(v) => self.remaining = v,
            Write::Cursor(v) => self.cursor = v,
            Write::Sprite(v) => self.sprite = v,
            Write::XRadius(v) => self.x_radius = v,
            Write::YRadius(v) => self.y_radius = v,
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub enum Command {
    Show { duration: u16 },
    Jump,
    Release,
    Unknown,
}
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum Outcome {
    Continue,
    Release,
}

/// Pure field reads from a captured definition, in the original observation order.
pub trait Frame<C, S> {
    fn command(&self) -> Command;
    fn sprite(&self) -> S;
    fn x_radius(&self) -> u16;
    fn y_radius(&self) -> u16;
    fn next(&self) -> C;
}

#[derive(Clone, Copy)]
pub enum Field {
    Command,
    Sprite,
    XRadius,
    YRadius,
}

pub trait Definitions<O: Owner> {
    type Frame: Frame<O::Cursor, O::Sprite>;
    /// Encoded providers publish committed state before a checked read failure.
    /// Prepared definitions need no failure boundary.
    fn validate(&mut self, owner: O, _: &Self::Frame, _: Field) -> O {
        owner
    }
    fn open(&mut self, owner: O, cursor: O::Cursor) -> (O, Self::Frame);
    fn follow_jump(&mut self, owner: O, cursor: O::Cursor) -> (O, Option<O::Cursor>);
    fn unknown(&mut self, owner: O) -> (O, Option<O::Cursor>);
}

pub fn advance<O: Owner>(
    mut owner: O,
    slot: O::Slot,
    defs: &mut impl Definitions<O>,
) -> (O, Outcome) {
    let prior = owner.remaining(slot);
    owner.write(slot, Write::Timer(prior.wrapping_sub(1)));
    if prior != 1 {
        return (owner, Outcome::Continue);
    }
    let mut cursor = owner.cursor(slot);
    loop {
        let (next, frame) = defs.open(owner, cursor);
        owner = next;
        owner = defs.validate(owner, &frame, Field::Command);
        match frame.command() {
            Command::Show { duration } => {
                owner.write(slot, Write::Timer(duration));
                owner = defs.validate(owner, &frame, Field::Sprite);
                owner.write(slot, Write::Sprite(frame.sprite()));
                owner = defs.validate(owner, &frame, Field::XRadius);
                owner.write(slot, Write::XRadius(frame.x_radius()));
                owner = defs.validate(owner, &frame, Field::YRadius);
                owner.write(slot, Write::YRadius(frame.y_radius()));
                owner.write(slot, Write::Cursor(frame.next()));
                return (owner, Outcome::Continue);
            }
            Command::Release => return (owner, Outcome::Release),
            Command::Jump => {
                let (next, target) = defs.follow_jump(owner, cursor);
                owner = next;
                if let Some(target) = target {
                    cursor = target;
                } else {
                    return (owner, Outcome::Continue);
                }
            }
            Command::Unknown => {
                let (next, target) = defs.unknown(owner);
                owner = next;
                if let Some(target) = target {
                    cursor = target;
                } else {
                    return (owner, Outcome::Continue);
                }
            }
        }
    }
}
