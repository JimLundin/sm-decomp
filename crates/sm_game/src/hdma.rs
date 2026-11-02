//! HDMA objects as Session-owned state ($18B0–$195B): step 2 of the liquid
//! and HDMA domain described in `crate::liquid`.
//!
//! The serialized block contains the enable flag, selected object and fourteen
//! six-word arrays. Active callers use the current source owner's word/byte API.
//! Whole records remain capture/codec values; full native aliases retain their
//! original byte identity at the compatibility boundary.
//!
//! Lane:
//! 1. This type, its `session_encoding` entries and the persistent
//!    `SessionCheckpoint` fields. No writer changes; compat writers stay on
//!    the RAM bridge.
//! 2. The interpreter (`visual_runtime`) under one lease per frame.
//! 3. The callback families (liquid, sky, x-ray, explosion windows, Mother
//!    Brain, Crocomire, Ceres, haze, menus) one at a time.
//!
//! Every word keeps its full stored `u16`.

/// Objects the original runs at once.
pub const OBJECTS: usize = 6;

/// One HDMA object's words, at `$18B4 + 12 * field + 2 * object`.
#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct HdmaObject {
    /// $18B4: the HDMA channel mask; zero for a free object.
    pub channels: u16,
    /// $18C0: the channel's register slot.
    pub bank_slot: u16,
    /// $18CC: the instruction list cursor.
    pub instruction_list: u16,
    /// $18D8: the HDMA table pointer.
    pub table: u16,
    /// $18E4.
    pub instruction_timer: u16,
    /// $18F0: the pre-instruction run each frame.
    pub pre_instruction: u16,
    /// $18FC: the pre-instruction's bank.
    pub pre_instruction_bank: u16,
    /// $1908.
    pub timer: u16,
    /// $1914–$1950: words whose meaning depends on the object.
    pub a: u16,
    pub b: u16,
    pub c: u16,
    pub d: u16,
    pub e: u16,
    pub f: u16,
}

#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct HdmaObjects {
    /// $18B0: objects run only while this is set.
    pub enabled: u16,
    /// $18B2: the object being run, as a byte offset.
    pub selected: u16,
    pub objects: [HdmaObject; OBJECTS],
}

impl HdmaObject {
    pub fn variable(&self, index: usize) -> &u16 {
        [&self.a, &self.b, &self.c, &self.d, &self.e, &self.f][index]
    }
    pub fn variable_mut(&mut self, index: usize) -> &mut u16 {
        [
            &mut self.a,
            &mut self.b,
            &mut self.c,
            &mut self.d,
            &mut self.e,
            &mut self.f,
        ][index]
    }
}

/// A stored word of one of the six HDMA objects.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ObjectWord {
    Channels,
    BankSlot,
    InstructionList,
    Table,
    InstructionTimer,
    PreInstruction,
    PreInstructionBank,
    Timer,
    A,
    B,
    C,
    D,
    E,
    F,
}

impl ObjectWord {
    pub const ALL: [Self; 14] = [
        Self::Channels,
        Self::BankSlot,
        Self::InstructionList,
        Self::Table,
        Self::InstructionTimer,
        Self::PreInstruction,
        Self::PreInstructionBank,
        Self::Timer,
        Self::A,
        Self::B,
        Self::C,
        Self::D,
        Self::E,
        Self::F,
    ];
    pub const fn ordinal(self) -> usize {
        self as usize
    }
}

/// Common HDMA words, resolved by the host's actual current source owner.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Word {
    Enabled,
    Selected,
    Object { index: usize, field: ObjectWord },
}

impl Word {
    /// Resolve a full even byte identity within the common table, including
    /// an indexed field that aliases a later array. Outside points stay outside.
    pub fn at_address(address: usize) -> Option<Self> {
        if !(0x18b0..0x195c).contains(&address) || address & 1 != 0 {
            return None;
        }
        Some(match address {
            0x18b0 => Self::Enabled,
            0x18b2 => Self::Selected,
            _ => {
                let offset = address - 0x18b4;
                Self::Object {
                    index: (offset % 12) / 2,
                    field: ObjectWord::ALL[offset / 12],
                }
            }
        })
    }

    pub fn address(self) -> usize {
        match self {
            Self::Enabled => 0x18b0,
            Self::Selected => 0x18b2,
            Self::Object { index, field } => {
                sm_data::table::check_index(index, OBJECTS);
                0x18b4 + (12 * field.ordinal()) + (2 * index)
            }
        }
    }
}

/// Access one field at its original expression, retaining a captured object index.
#[macro_export]
macro_rules! hdma_word {
    ($host:expr, $index:expr, $field:ident => $value:expr) => {{
        let index = $index;
        sm_data::table::check_index(index, $crate::hdma::OBJECTS);
        let word = $crate::hdma::Word::Object {
            index,
            field: $crate::hdma::ObjectWord::$field,
        };
        let value = $value;
        $crate::hdma_objects::HdmaTable::store_hdma_word(
            $host,
            word,
            value,
            $crate::scene::state::WordOrder::Little,
        );
    }};
    ($host:expr, $index:expr, $field:ident) => {
        $crate::hdma_objects::HdmaTable::hdma_word(
            $host,
            $crate::hdma::Word::Object {
                index: $index,
                field: $crate::hdma::ObjectWord::$field,
            },
        )
    };
}

/// Publish only the byte the original operation writes.
#[macro_export]
macro_rules! hdma_byte {
    ($host:expr, $index:expr, $field:ident, $high:expr => $value:expr) => {{
        let index = $index;
        sm_data::table::check_index(index, $crate::hdma::OBJECTS);
        let word = $crate::hdma::Word::Object {
            index,
            field: $crate::hdma::ObjectWord::$field,
        };
        let value = $value;
        $crate::hdma_objects::HdmaTable::store_hdma_byte($host, word, $high, value);
    }};
}
