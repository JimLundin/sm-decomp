//! A method's argument object, passed by reference.
pub struct Mover;

struct Look {
    at: u16,
    r: u16,
}

impl Mover {
    fn look(&self, look: &Look) -> u16 {
        look.at.wrapping_sub(look.r)
    }

    /// Looks ahead.
    pub fn ahead(&self, at: u16) -> u16 {
        self.look(&Look { at, r: 1 })
    }
}
