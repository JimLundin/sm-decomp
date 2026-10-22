//! Projectile pass rules, carrying the complete owner across output callbacks.
#![forbid(unsafe_code)]

#[derive(Clone, Copy, Debug, PartialEq)]
pub enum Pass {
    Ordinary,
    Explosions,
}
pub trait Cursor: Copy {
    fn active(self) -> bool;
}
pub trait Sprite: Copy {
    fn drawable(self) -> bool;
}
pub trait Kind: Copy {
    fn visible(
        self,
        pass: Pass,
        slot: u16,
        frame: impl FnOnce() -> u16,
        phase: impl FnOnce() -> u16,
    ) -> bool;
    fn project(self, pass: Pass) -> bool;
}
impl Cursor for u16 {
    fn active(self) -> bool {
        self != 0
    }
}
impl<C: Copy> Cursor for Option<C> {
    fn active(self) -> bool {
        self.is_some()
    }
}
impl Sprite for u16 {
    fn drawable(self) -> bool {
        self & 0x8000 != 0
    }
}
impl Kind for u16 {
    fn visible(
        self,
        pass: Pass,
        slot: u16,
        frame: impl FnOnce() -> u16,
        phase: impl FnOnce() -> u16,
    ) -> bool {
        if pass == Pass::Explosions {
            self & 0xf00 >= 0x300 && (self & 0xf00 != 0x300 || phase() != 0)
        } else if self & 0xf10 != 0 {
            self & 0xf00 < 0x300
        } else if self & 0xc != 0 {
            (slot & 2 != 0) == (frame() & 2 != 0)
        } else {
            (slot & 2 == 0) == (frame() & 1 != 0)
        }
    }
    fn project(self, pass: Pass) -> bool {
        pass == Pass::Ordinary || self & 0xf00 != 0x300 && self & 0xf00 != 0x500
    }
}
pub trait Owner: Sized {
    type Cursor: Cursor;
    type Sprite: Sprite;
    type Kind: Kind;
    fn contains(&self, slot: u16) -> bool;
    fn cursor(&self, slot: u16) -> Self::Cursor;
    fn kind(&self, slot: u16) -> Self::Kind;
    fn phase(&self, slot: u16) -> u16;
    fn sprite(&self, slot: u16) -> Self::Sprite;
    fn x(&self, slot: u16) -> u16;
    fn y(&self, slot: u16) -> u16;
    fn camera_x(&self) -> u16;
    fn camera_y(&self) -> u16;
    fn frame(&self) -> u16;
    fn projected(&self) -> bool;
    fn selected(&self) -> u16;
    fn select(&mut self, slot: u16);
}
pub trait Context<O> {
    fn project(&mut self, owner: O, slot: u16) -> (O, (u16, u16));
    fn draw(&mut self, owner: O, slot: u16, position: (u16, u16)) -> O;
    fn finish(&mut self, owner: O) -> O;
    fn resume(&mut self, owner: O, pass: Pass, slot: u16, position: (u16, u16)) -> O;
}

pub fn present<O: Owner>(mut owner: O, pass: Pass, context: &mut impl Context<O>) -> O {
    let index = if pass == Pass::Explosions { 18 } else { 8 };
    owner.select(index);
    owner = continue_pass(owner, pass, index, (0, 0), context);
    if pass == Pass::Ordinary {
        owner = context.finish(owner);
    }
    owner
}

/// Complete only the slot loop. The caller retains responsibility for its tail.
pub fn continue_pass<O: Owner>(
    mut owner: O,
    pass: Pass,
    mut index: u16,
    (mut x, mut y): (u16, u16),
    context: &mut impl Context<O>,
) -> O {
    let explosions = pass == Pass::Explosions;
    loop {
        if !owner.contains(index) {
            return context.resume(owner, pass, index, (x, y));
        }
        if owner.cursor(index).active() {
            let kind = owner.kind(index);
            if kind.visible(pass, index, || owner.frame(), || owner.phase(index)) {
                'draw: {
                    let clip_y = if owner.projected() && kind.project(pass) {
                        let (next, point) = context.project(owner, index);
                        owner = next;
                        // Ceres projectiles clip transformed Y but draw carried Y;
                        // explosions discard both projected coordinates.
                        if explosions {
                            y
                        } else {
                            x = point.0;
                            point.1
                        }
                    } else {
                        x = owner.x(index).wrapping_sub(owner.camera_x());
                        if explosions
                            && (x.wrapping_sub(304) & 0x8000 == 0
                                || x.wrapping_add(48) & 0x8000 != 0)
                        {
                            break 'draw;
                        }
                        y = owner.y(index).wrapping_sub(owner.camera_y());
                        y
                    };
                    if clip_y & 0xff00 == 0 && (explosions || owner.sprite(index).drawable()) {
                        owner = context.draw(owner, index, (x, y));
                    }
                    index = owner.selected();
                }
            }
        }
        index = index.wrapping_sub(2);
        owner.select(index);
        if index & 0x8000 != 0 {
            return owner;
        }
    }
}

/// The original projection deliberately uses A for both diagonal products.
pub fn project((dx, dy): (u16, u16), (a, b, c): (u16, u16, u16)) -> (u16, u16) {
    let scale = |value: u16, coefficient: u16| {
        ((i32::from(value as i16) * i32::from(coefficient as i16)) as u32 >> 8) as u16
    };
    (
        scale(dx, a).wrapping_add(scale(dy, b)),
        scale(dx, c).wrapping_add(scale(dy, a)),
    )
}
