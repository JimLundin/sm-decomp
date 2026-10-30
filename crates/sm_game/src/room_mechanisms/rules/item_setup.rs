//! Item placement and collision-trigger association.
use super::super::access::*;
use super::super::host::*;
use super::terrain_setup as terrain;
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Container {
    Visible,
    ShotBlock,
}
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Graphic {
    EnergyTank,
    MissileTank,
    SuperMissileTank,
    PowerBombTank,
}
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Setup {
    Initialize(Container, Option<Graphic>),
    Trigger,
}

pub fn setup(c: &mut impl Host, slot: usize, kind: Setup) -> u8 {
    match kind {
        Setup::Initialize(container, graphic) => {
            if let Some(graphic) = graphic {
                mechanism!(
                    c,
                    slot,
                    work1 = match graphic {
                        Graphic::EnergyTank => 8,
                        Graphic::MissileTank => 10,
                        Graphic::SuperMissileTank => 12,
                        Graphic::PowerBombTank => 14,
                    }
                );
            }
            terrain::install(
                c,
                mechanism!(c, slot, block),
                if container == Container::Visible {
                    0x45
                } else {
                    0xc045
                },
            );
            if container == Container::ShotBlock {
                mechanism!(
                    c,
                    slot,
                    work0 = terrain_word(
                        c,
                        usize::from((mechanism!(c, slot, block)) & !1),
                        ByteOrder::Native
                    )
                );
            }
            c.session_mut().room.loaded_items = c.session().room.loaded_items.wrapping_add(1);
        }
        Setup::Trigger => {
            if c.inputs().frozen != 0 {
                mechanism!(c, slot, header = 0);
            } else {
                let block = mechanism!(c, slot, block);
                mechanism!(c, slot, block = 0);
                let found = (0..40)
                    .rev()
                    .find(|&found| mechanism!(c, found, block) == block);
                {
                    let index = found.map_or(-1, |slot| slot as isize);
                    mechanism!(c, index, counter = 255)
                };
            }
        }
    }
    0
}
