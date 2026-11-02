//! Boss HDMA allocation and Phantoon's motion setup, on finite owner ports.
//! The HDMA table port is the common consumer packet's retained owner.
//! The prepared physical packet supplies the actual DMA channel publisher.
#![forbid(unsafe_code)]
use crate::actors::access::{AccessMode::Native, ActorAccess, ActorKey};
use crate::display::hdma_spawn::HdmaObjectDefinition;
use crate::hdma_objects::HdmaTable;

#[derive(Clone, Copy)]
pub enum DmaField {
    Control,
    Target,
}

pub trait Host: HdmaTable {
    /// Publish the actual selected channel's byte, preserving failure prefixes.
    /// `channel` is the hardware channel number (2 through 7 for allocated objects).
    fn write_dma_channel(&mut self, channel: usize, field: DmaField, value: u8);
}

fn hdma_definition(args: &HdmaObjectDefinition) -> [u16; 2] {
    [
        u16::from_ne_bytes([args.dma_control, args.target_register]),
        args.instruction_list,
    ]
}

pub fn spawn_hdma_owned<H: Host + ?Sized>(
    bank: u8,
    args: &HdmaObjectDefinition,
    game: &mut H,
) -> u16 {
    let words = hdma_definition(args);
    spawn_hdma_words(bank, |index| words[index], game)
}

pub fn spawn_hdma_words<H: Host + ?Sized>(
    bank: u8,
    read_word: impl Fn(usize) -> u16,
    game: &mut H,
) -> u16 {
    place_hdma(None, bank, read_word, game)
}

pub fn spawn_bg3_scroll<H: Host + ?Sized>(game: &mut H) {
    let words = hdma_definition(&HdmaObjectDefinition::new(0x43, 0x11, 0xd8d0));
    place_hdma(Some(5), 0x88, |index| words[index], game);
}

fn place_hdma<H: Host + ?Sized>(
    requested: Option<usize>,
    bank: u8,
    read_word: impl Fn(usize) -> u16,
    game: &mut H,
) -> u16 {
    let slot = requested.or_else(|| {
        (0..crate::hdma::OBJECTS).find(|&slot| crate::hdma_word!(&*game, slot, Channels) == 0)
    });
    let Some(slot) = slot else {
        return u16::MAX;
    };
    // The first two stores precede even the instruction-definition query.
    crate::hdma_word!(game,slot,PreInstruction => 0x84b8);
    crate::hdma_word!(game,slot,PreInstructionBank => 0x88);
    crate::hdma_word!(game,slot,InstructionList => read_word(1));
    crate::hdma_word!(game,slot,InstructionTimer => 1);
    crate::hdma_word!(game,slot,Timer => 0);
    crate::hdma_word!(game,slot,A => 0);
    crate::hdma_word!(game,slot,B => 0);
    crate::hdma_word!(game,slot,C => 0);
    crate::hdma_word!(game,slot,D => 0);
    crate::hdma_word!(game,slot,Channels => 4 << slot);
    crate::hdma_word!(game,slot,BankSlot => (u16::from(bank) << 8) | channel(slot));
    // Control/target are acquired only after the allocated table is published.
    let [control, target] = read_word(0).to_le_bytes();
    game.write_dma_channel(slot + 2, DmaField::Control, control);
    game.write_dma_channel(slot + 2, DmaField::Target, target);
    2 * u16::try_from(slot).expect("HDMA object slot")
}

fn channel(slot: usize) -> u16 {
    0x20 + (0x10 * u16::try_from(slot).expect("HDMA object slot"))
}

pub fn initialize_phantoon_motion<H: Host + ActorAccess + ?Sized>(
    direction: u16,
    speed: u16,
    game: &mut H,
) {
    game.edit_actor(ActorKey::from_legacy(0x80), Native, |actor| {
        actor.parameters[0] = direction
    });
    game.edit_actor(ActorKey::from_legacy(0xc0), Native, |actor| {
        actor.variables[3] = 0;
        actor.variables[4] = 0;
        actor.pre_instruction = speed;
    });
    spawn_hdma_owned(0x88, &HdmaObjectDefinition::new(0x42, 0x0f, 0xe4a8), game);
}

/// Original out-of-table frees still write the full aliased channel word.
pub fn free_hdma(slot: u16, game: &mut crate::actors::owned_execution::PreparedActorState<'_>) {
    let index = usize::from(slot >> 1);
    if index < crate::hdma::OBJECTS {
        crate::hdma_word!(game,index,Channels=>0);
    } else {
        game.write_current_slice(0x18b4 + (2 * index), &0u16.to_le_bytes());
    }
}
