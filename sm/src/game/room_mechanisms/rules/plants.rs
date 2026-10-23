//! Brinstar plants capture and hold the player through damage and recovery.
#![forbid(unsafe_code)]
use super::super::{
    fields::{Fields, Global},
    state::Column,
};
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Surface {
    Floor,
    Ceiling,
}

pub fn begin(c: &mut impl Fields, slot: usize, surface: Surface) -> u8 {
    let aligned = match surface {
        Surface::Floor => {
            c.global(Global::PlayerRadius)
                .wrapping_add(c.global(Global::PlayerY))
                & 15
                == 0
        }
        Surface::Ceiling => {
            c.global(Global::PlayerY)
                .wrapping_sub(c.global(Global::PlayerRadius))
                & 15
                == 0
        }
    };
    if aligned {
        let block = c.slot(Column::Block, slot);
        c.set_level(block, c.level(block) & 0x8fff);
        c.set_slot(Column::Work0, slot, c.global(Global::PlayerX));
        let y = match surface {
            Surface::Floor => c.global(Global::PlayerY).wrapping_sub(1),
            Surface::Ceiling => c.global(Global::PlayerY).wrapping_add(1),
        };
        c.set_slot(Column::Work1, slot, y);
    } else {
        c.release(slot);
    }
    0
}
pub fn hold(c: &mut impl Fields, slot: usize) {
    c.set_global(Global::PlayerX, c.slot(Column::Work0, slot));
    c.set_global(Global::PlayerY, c.slot(Column::Work1, slot));
    c.set_global(
        Global::ContactProtection,
        c.global(Global::ContactProtection) | 0x10,
    );
}
pub fn hurt(c: &mut impl Fields) {
    c.set_global(
        Global::PeriodicDamage,
        c.global(Global::PeriodicDamage).wrapping_add(2),
    );
}
pub fn protect(c: &mut impl Fields) {
    c.set_global(Global::ContactProtection, 48);
}
