//! Nominal collection bytes resolve directly to their typed owner.
//! Odd/full/outspan aliases fall back only after the bounded family lookup.
#![forbid(unsafe_code)]
use super::PreparedActorState;
use crate::actors::access::CollectionWindow;
impl PreparedActorState<'_> {
    pub(super) fn collection_typed_word(
        &self,
        window: CollectionWindow,
        offset: usize,
    ) -> Option<u16> {
        let index = offset / 2;
        match window {
            CollectionWindow::Active if index < 32 => {
                Some(self.owner.gameplay.actors.schedule.active[index])
            }
            CollectionWindow::Interactive if index < 32 => {
                Some(self.owner.gameplay.actors.schedule.interactive[index])
            }
            CollectionWindow::CollisionDirections if index < 4 => {
                Some(self.owner.gameplay.actors.schedule.colliding[index])
            }
            CollectionWindow::GraphicsDefinitions if index < 4 => {
                Some(self.owner.gameplay.actors.graphics.definitions[index])
            }
            CollectionWindow::GraphicsTiles if index < 4 => {
                Some(self.owner.gameplay.actors.graphics.tile_offsets[index])
            }
            CollectionWindow::GraphicsRoutes if index < 4 => {
                Some(self.owner.gameplay.actors.graphics.palette_routes[index])
            }
            CollectionWindow::DrawEntries if index < 114 => {
                Some(*self.owner.gameplay.presentation.actor_entry(index))
            }
            CollectionWindow::DrawSizes if index < 8 => {
                Some(*self.owner.gameplay.presentation.actor_size(index))
            }
            CollectionWindow::AbsoluteDrawAddress(address) => {
                let at = usize::from(address).checked_add(offset)?;
                if (0xe84..0xf68).contains(&at) {
                    Some(
                        *self
                            .owner
                            .gameplay
                            .presentation
                            .actor_entry((at - 0xe84) / 2),
                    )
                } else {
                    None
                }
            }
            CollectionWindow::SpeciesExtended if offset < 0x1fdc => {
                let at = 0x7800 + (offset & !1);
                if matches!(at, 0x801e | 0x8020 | 0x803c | 0x8818 | 0x883e) {
                    None
                } else {
                    Some(
                        *self
                            .owner
                            .gameplay
                            .actors
                            .extended
                            .word(offset / 0x800, (offset % 0x800) / 2),
                    )
                }
            }
            _ => None,
        }
    }
    pub(super) fn collection_typed_byte(
        &self,
        window: CollectionWindow,
        offset: usize,
    ) -> Option<u8> {
        if matches!(window, CollectionWindow::TileLoad) && offset < 32 {
            return Some(self.owner.gameplay.actors.staging.descriptor_prefix[offset]);
        }
        if matches!(window, CollectionWindow::VramQueue) && offset < 0x200 {
            return Some(self.owner.gameplay.queued_tilemaps.queue[offset]);
        }
        let lane = if let CollectionWindow::AbsoluteDrawAddress(address) = window {
            (usize::from(address) + offset) % 2
        } else {
            offset % 2
        };
        self.collection_typed_word(window, offset)
            .map(|word| word.to_le_bytes()[lane])
    }
    pub(super) fn set_collection_typed_byte(
        &mut self,
        window: CollectionWindow,
        offset: usize,
        value: u8,
    ) -> bool {
        if matches!(window, CollectionWindow::TileLoad) && offset < 32 {
            self.owner.gameplay.actors.staging.descriptor_prefix[offset] = value;
            return true;
        }
        let index = offset / 2;
        let lane = if let CollectionWindow::AbsoluteDrawAddress(address) = window {
            (usize::from(address) + offset) % 2
        } else {
            offset % 2
        };
        let field = match window {
            CollectionWindow::Active if index < 32 => {
                &mut self.owner.gameplay.actors.schedule.active[index]
            }
            CollectionWindow::Interactive if index < 32 => {
                &mut self.owner.gameplay.actors.schedule.interactive[index]
            }
            CollectionWindow::CollisionDirections if index < 4 => {
                &mut self.owner.gameplay.actors.schedule.colliding[index]
            }
            CollectionWindow::GraphicsDefinitions if index < 4 => {
                &mut self.owner.gameplay.actors.graphics.definitions[index]
            }
            CollectionWindow::GraphicsTiles if index < 4 => {
                &mut self.owner.gameplay.actors.graphics.tile_offsets[index]
            }
            CollectionWindow::GraphicsRoutes if index < 4 => {
                &mut self.owner.gameplay.actors.graphics.palette_routes[index]
            }
            CollectionWindow::DrawEntries if index < 114 => {
                self.owner.gameplay.presentation.actor_entry_mut(index)
            }
            CollectionWindow::DrawSizes if index < 8 => {
                self.owner.gameplay.presentation.actor_size_mut(index)
            }
            CollectionWindow::AbsoluteDrawAddress(address) => {
                let at = usize::from(address)
                    .checked_add(offset)
                    .expect("draw address extent");
                if (0xe84..0xf68).contains(&at) {
                    self.owner
                        .gameplay
                        .presentation
                        .actor_entry_mut((at - 0xe84) / 2)
                } else {
                    return false;
                }
            }
            CollectionWindow::SpeciesExtended if offset < 0x1fdc => {
                let at = 0x7800 + (offset & !1);
                if matches!(at, 0x801e | 0x8020 | 0x803c | 0x8818 | 0x883e) {
                    return false;
                }
                &mut *self
                    .owner
                    .gameplay
                    .actors
                    .extended
                    .word_mut(offset / 0x800, (offset % 0x800) / 2)
            }
            _ => return false,
        };
        let mut bytes = field.to_le_bytes();
        bytes[lane] = value;
        *field = u16::from_le_bytes(bytes);
        true
    }
}
