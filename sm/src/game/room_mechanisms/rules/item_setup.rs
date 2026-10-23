//! Item placement and collision-trigger association.
#![forbid(unsafe_code)]
use super::super::{
    fields::{Fields, Global},
    state::Column,
};
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

pub fn setup(c: &mut impl Fields, slot: usize, kind: Setup) -> u8 {
    match kind {
        Setup::Initialize(container, graphic) => {
            if let Some(graphic) = graphic {
                c.set_slot(
                    Column::Work1,
                    slot,
                    match graphic {
                        Graphic::EnergyTank => 8,
                        Graphic::MissileTank => 10,
                        Graphic::SuperMissileTank => 12,
                        Graphic::PowerBombTank => 14,
                    },
                );
            }
            c.install(
                c.slot(Column::Block, slot),
                if container == Container::Visible {
                    0x45
                } else {
                    0xc045
                },
            );
            if container == Container::ShotBlock {
                c.set_slot(
                    Column::Work0,
                    slot,
                    c.native_level(c.slot(Column::Block, slot)),
                );
            }
            c.set_global(
                Global::LoadedItems,
                c.global(Global::LoadedItems).wrapping_add(1),
            );
        }
        Setup::Trigger => {
            if c.global(Global::FrozenTime) != 0 {
                c.release(slot);
            } else {
                let block = c.slot(Column::Block, slot);
                c.set_slot(Column::Block, slot, 0);
                let found = (0..40)
                    .rev()
                    .find(|&found| c.slot(Column::Block, found) == block);
                c.set_signed_slot(Column::Counter, found.map_or(-1, |slot| slot as isize), 255);
            }
        }
    }
    0
}
