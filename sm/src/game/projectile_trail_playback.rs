//! Trail-side playback with named commands and complete owner transfers.
#![forbid(unsafe_code)]
use crate::game::projectile_trail_emission::{Half, Owner, Write};

pub enum Instruction<C> {
    Frame { duration: u16 },
    Move { target: Half, delta_y: i16, next: C },
    Unknown { next: C },
}

pub trait Definitions<O: Owner> {
    type Cursor: Copy;
    fn start(&mut self, cursor: O::Cursor, half: Half) -> Self::Cursor;
    fn instruction(&mut self, owner: O, cursor: Self::Cursor) -> (O, Instruction<Self::Cursor>);
    fn frame(&mut self, owner: O, cursor: Self::Cursor) -> (O, O::Sprite, O::Cursor);
    fn unknown(&mut self, owner: O) -> O;
}

pub trait Presentation<O: Owner> {
    fn draw(&mut self, owner: O, pair: usize, half: Half) -> O;
}

pub fn advance<O: Owner>(
    mut owner: O,
    pair: usize,
    half: Half,
    definitions: &mut impl Definitions<O>,
) -> (O, bool) {
    let remaining = owner.remaining(pair, half);
    if remaining == 0 {
        return (owner, false);
    }
    owner.write(pair, half, Write::Timer(remaining.wrapping_sub(1)));
    if remaining != 1 {
        return (owner, true);
    }
    let mut cursor = definitions.start(owner.cursor(pair, half), half);
    loop {
        let (next, instruction) = definitions.instruction(owner, cursor);
        owner = next;
        match instruction {
            Instruction::Move {
                target,
                delta_y,
                next,
            } => {
                owner.write(
                    pair,
                    target,
                    Write::Y(owner.y(pair, target).wrapping_add(delta_y as u16)),
                );
                cursor = next;
            }
            Instruction::Unknown { next } => {
                owner = definitions.unknown(owner);
                cursor = next;
            }
            Instruction::Frame { duration } => {
                owner.write(pair, half, Write::Timer(duration));
                if duration == 0 {
                    return (owner, false);
                }
                let (next, sprite, following) = definitions.frame(owner, cursor);
                owner = next;
                owner.write(pair, half, Write::Tile(sprite));
                owner.write(pair, half, Write::Cursor(following));
                return (owner, true);
            }
        }
    }
}

pub fn present<O: Owner>(
    mut owner: O,
    frozen: bool,
    context: &mut (impl Definitions<O> + Presentation<O>),
) -> O {
    for pair in (0..18).rev() {
        for half in [Half::Left, Half::Right] {
            let enabled = if frozen {
                owner.remaining(pair, half) != 0
            } else {
                let (next, enabled) = advance(owner, pair, half, context);
                owner = next;
                enabled
            };
            if enabled {
                owner = context.draw(owner, pair, half);
            }
        }
    }
    owner
}
