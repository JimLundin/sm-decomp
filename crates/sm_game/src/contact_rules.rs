//! Shared geometry and projectile eligibility for contact scans.
#[derive(Clone, Copy)]
pub enum HitboxEdge {
    Left,
    Right,
    Top,
    Bottom,
}
pub struct Bounds {
    left: u16,
    right: u16,
    top: u16,
    bottom: u16,
}
impl Bounds {
    pub fn new(x: u16, y: u16, width: u16, height: u16) -> Self {
        Self {
            left: x.wrapping_sub(width),
            right: x.wrapping_add(width),
            top: y.wrapping_sub(height),
            bottom: y.wrapping_add(height),
        }
    }
    pub fn hits(
        &self,
        x: u16,
        y: u16,
        mut read: impl FnMut(HitboxEdge) -> u16,
        touch: bool,
    ) -> bool {
        let mut edge = |field, origin: u16| read(field).wrapping_add(origin);
        if touch {
            (edge(HitboxEdge::Left, x).wrapping_sub(self.right) as i16) < 0
                && (edge(HitboxEdge::Right, x).wrapping_sub(self.left) as i16) >= 0
                && (edge(HitboxEdge::Top, y).wrapping_sub(self.bottom) as i16) < 0
                && (edge(HitboxEdge::Bottom, y).wrapping_sub(self.top) as i16) >= 0
        } else {
            (self.right.wrapping_sub(edge(HitboxEdge::Left, x)) as i16) >= 0
                && (self.left.wrapping_sub(edge(HitboxEdge::Right, x)) as i16) < 0
                && (self.bottom.wrapping_sub(edge(HitboxEdge::Top, y)) as i16) >= 0
                && (self.top.wrapping_sub(edge(HitboxEdge::Bottom, y)) as i16) < 0
        }
    }
}
pub fn axis(a: u16, b: u16, radius: u16, enemy_radius: u16) -> bool {
    let distance = (a.wrapping_sub(b) as i16).wrapping_abs() as u16;
    i32::from(distance) - i32::from(radius) < i32::from(enemy_radius)
}

pub fn is_shot(kind: u16) -> bool {
    kind != 0 && kind & 0xf00 != 0x300 && kind & 0xf00 != 0x500 && kind & 0xf00 < 0x700
}

pub fn collision_span(position: u16, radius: u16) -> u16 {
    radius
        .wrapping_add(position)
        .wrapping_sub(1)
        .wrapping_sub(position.wrapping_sub(radius) & 0xfff0)
        >> 4
}
