//! Brinstar plants capture and hold the player through damage and recovery.
#![forbid(unsafe_code)]
use super::super::access::*;
use super::super::host::*;
use super::terrain_setup as terrain;
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Surface {
    Floor,
    Ceiling,
}

pub fn begin(c: &mut impl Host, slot: usize, surface: Surface) -> u8 {
    let aligned = match surface {
        Surface::Floor => {
            c.session()
                .player
                .contact
                .radius
                .wrapping_add(c.session().player.y)
                & 15
                == 0
        }
        Surface::Ceiling => {
            c.session()
                .player
                .y
                .wrapping_sub(c.session().player.contact.radius)
                & 15
                == 0
        }
    };
    if aligned {
        let block = mechanism!(c, slot, block);
        let level = terrain::level(c, block) & 0x8fff;
        terrain::set_level(c, block, level);
        mechanism!(c, slot, work0 = c.session().player.x);
        let y = match surface {
            Surface::Floor => c.session().player.y.wrapping_sub(1),
            Surface::Ceiling => c.session().player.y.wrapping_add(1),
        };
        mechanism!(c, slot, work1 = y);
    } else {
        mechanism!(c, slot, header = 0);
    }
    0
}
pub fn hold(c: &mut impl Host, slot: usize) {
    c.session_mut().player.x = mechanism!(c, slot, work0);
    c.session_mut().player.y = mechanism!(c, slot, work1);
    c.session_mut().player.invincibility = c.session().player.invincibility | 0x10;
}
pub fn hurt(c: &mut impl Host) {
    c.session_mut().player.periodic_damage = c.session().player.periodic_damage.wrapping_add(2);
}
pub fn protect(c: &mut impl Host) {
    c.session_mut().player.invincibility = 48;
}
