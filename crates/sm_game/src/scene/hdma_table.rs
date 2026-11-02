//! Finite scalar access to the one current presentation HDMA interpreter table.
#![forbid(unsafe_code)]
use crate::hdma::{HdmaObjects, ObjectWord, Word};

pub fn word(table: &HdmaObjects, key: Word) -> u16 {
    match key {
        Word::Enabled => table.enabled,
        Word::Selected => table.selected,
        Word::Object { index, field } => {
            sm_data::table::check_index(index, crate::hdma::OBJECTS);
            let object = &table.objects[index];
            match field {
                ObjectWord::Channels => object.channels,
                ObjectWord::BankSlot => object.bank_slot,
                ObjectWord::InstructionList => object.instruction_list,
                ObjectWord::Table => object.table,
                ObjectWord::InstructionTimer => object.instruction_timer,
                ObjectWord::PreInstruction => object.pre_instruction,
                ObjectWord::PreInstructionBank => object.pre_instruction_bank,
                ObjectWord::Timer => object.timer,
                ObjectWord::A => object.a,
                ObjectWord::B => object.b,
                ObjectWord::C => object.c,
                ObjectWord::D => object.d,
                ObjectWord::E => object.e,
                ObjectWord::F => object.f,
            }
        }
    }
}
pub fn store(table: &mut HdmaObjects, key: Word, value: u16) {
    match key {
        Word::Enabled => table.enabled = value,
        Word::Selected => table.selected = value,
        Word::Object { index, field } => {
            sm_data::table::check_index(index, crate::hdma::OBJECTS);
            let object = &mut table.objects[index];
            *match field {
                ObjectWord::Channels => &mut object.channels,
                ObjectWord::BankSlot => &mut object.bank_slot,
                ObjectWord::InstructionList => &mut object.instruction_list,
                ObjectWord::Table => &mut object.table,
                ObjectWord::InstructionTimer => &mut object.instruction_timer,
                ObjectWord::PreInstruction => &mut object.pre_instruction,
                ObjectWord::PreInstructionBank => &mut object.pre_instruction_bank,
                ObjectWord::Timer => &mut object.timer,
                ObjectWord::A => &mut object.a,
                ObjectWord::B => &mut object.b,
                ObjectWord::C => &mut object.c,
                ObjectWord::D => &mut object.d,
                ObjectWord::E => &mut object.e,
                ObjectWord::F => &mut object.f,
            } = value;
        }
    }
}
