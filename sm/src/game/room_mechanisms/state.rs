//! Private owned mechanism state and its current WRAM byte encoding.
#![forbid(unsafe_code)]

#[derive(Clone, Copy, Debug)]
pub(super) enum Column {
    Header,
    Block,
    PreAction,
    Program,
    Counter,
    RoomArgument,
    Work0,
    InstructionDelay,
    DrawProgram,
    ReturnProgram,
    Work1,
}
impl Column {
    pub(super) const fn address(self) -> usize {
        match self {
            Self::Header => 0x1c37,
            Self::Block => 0x1c87,
            Self::PreAction => 0x1cd7,
            Self::Program => 0x1d27,
            Self::Counter => 0x1d77,
            Self::RoomArgument => 0x1dc7,
            Self::Work0 => 0x1e17,
            Self::InstructionDelay => 0xde1c,
            Self::DrawProgram => 0xde6c,
            Self::ReturnProgram => 0xdebc,
            Self::Work1 => 0xdf0c,
        }
    }
}
#[derive(Clone, Copy, Default)]
struct Slot {
    header: u16,
    block: u16,
    pre_action: u16,
    program: u16,
    counter: u16,
    room_argument: u16,
    work0: u16,
    instruction_delay: u16,
    draw_program: u16,
    return_program: u16,
    work1: u16,
}
impl Slot {
    fn field(&mut self, column: Column) -> &mut u16 {
        match column {
            Column::Header => &mut self.header,
            Column::Block => &mut self.block,
            Column::PreAction => &mut self.pre_action,
            Column::Program => &mut self.program,
            Column::Counter => &mut self.counter,
            Column::RoomArgument => &mut self.room_argument,
            Column::Work0 => &mut self.work0,
            Column::InstructionDelay => &mut self.instruction_delay,
            Column::DrawProgram => &mut self.draw_program,
            Column::ReturnProgram => &mut self.return_program,
            Column::Work1 => &mut self.work1,
        }
    }
}

#[derive(Clone, Copy)]
enum Location {
    Slot(usize, Column),
    Control(usize),
    Tile(usize),
    Custom(usize),
}
const REGIONS: [(usize, usize); 5] = [
    (0x1c23, 10),
    (0x1c37, 280),
    (0xde1c, 160),
    (0xc6c8, 256),
    (0x1e67, 3),
];
fn location(address: usize) -> Option<(Location, usize)> {
    let (base, index) = if (0x1c37..0x1e67).contains(&address) {
        let index = (address - 0x1c37) / 80;
        let column = [
            Column::Header,
            Column::Block,
            Column::PreAction,
            Column::Program,
            Column::Counter,
            Column::RoomArgument,
            Column::Work0,
        ][index];
        let base = column.address();
        return Some((
            Location::Slot((address - base) / 2, column),
            (address - base) & 1,
        ));
    } else if (0xde1c..0xdf5c).contains(&address) {
        let index = (address - 0xde1c) / 80;
        let column = [
            Column::InstructionDelay,
            Column::DrawProgram,
            Column::ReturnProgram,
            Column::Work1,
        ][index];
        let base = column.address();
        return Some((
            Location::Slot((address - base) / 2, column),
            (address - base) & 1,
        ));
    } else if (0x1c23..0x1c37).contains(&address) {
        (0x1c23, Location::Control((address - 0x1c23) / 2))
    } else if (0xc6c8..0xc8c8).contains(&address) {
        (0xc6c8, Location::Tile((address - 0xc6c8) / 2))
    } else if (0x1e67..0x1e6d).contains(&address) {
        (0x1e67, Location::Custom((address - 0x1e67) / 2))
    } else {
        return None;
    };
    Some((index, (address - base) & 1))
}

pub(super) struct RoomMechanisms {
    slots: [Slot; 40],
    flags: u16,
    draw_cursor: u16,
    selected: u16,
    block_x: u16,
    block_y: u16,
    next_graphics: u16,
    saved_graphics: [u16; 4],
    tilemap: [u16; 256],
    custom_draw: [u16; 3],
}
impl RoomMechanisms {
    fn word_mut(&mut self, location: Location) -> &mut u16 {
        match location {
            Location::Slot(index, column) => self.slots[index].field(column),
            Location::Tile(index) => &mut self.tilemap[index],
            Location::Custom(index) => &mut self.custom_draw[index],
            Location::Control(index) => match index {
                0 => &mut self.flags,
                1 => &mut self.draw_cursor,
                2 => &mut self.selected,
                3 => &mut self.block_x,
                4 => &mut self.block_y,
                5 => &mut self.next_graphics,
                _ => &mut self.saved_graphics[index - 6],
            },
        }
    }
    fn word(&self, location: Location) -> u16 {
        match location {
            Location::Slot(index, column) => {
                let mut value = self.slots[index];
                *value.field(column)
            }
            Location::Tile(index) => self.tilemap[index],
            Location::Custom(index) => self.custom_draw[index],
            Location::Control(index) => match index {
                0 => self.flags,
                1 => self.draw_cursor,
                2 => self.selected,
                3 => self.block_x,
                4 => self.block_y,
                5 => self.next_graphics,
                _ => self.saved_graphics[index - 6],
            },
        }
    }
}
impl super::transport::Codec for RoomMechanisms {
    fn decode(ram: &[u8]) -> Self {
        let mut state = Self {
            slots: [Slot::default(); 40],
            flags: 0,
            draw_cursor: 0,
            selected: 0,
            block_x: 0,
            block_y: 0,
            next_graphics: 0,
            saved_graphics: [0; 4],
            tilemap: [0; 256],
            custom_draw: [0; 3],
        };
        for (base, count) in REGIONS {
            for i in 0..count {
                let at = base + 2 * i;
                *state.word_mut(location(at).unwrap().0) =
                    u16::from_le_bytes([ram[at], ram[at + 1]]);
            }
        }
        state
    }
    fn encode(&self, ram: &mut [u8]) {
        for (base, count) in REGIONS {
            for i in 0..count {
                let at = base + 2 * i;
                ram[at..at + 2].copy_from_slice(&self.word(location(at).unwrap().0).to_le_bytes());
            }
        }
    }
    fn byte(&self, address: usize) -> Option<u8> {
        let (location, lane) = location(address)?;
        Some(self.word(location).to_le_bytes()[lane])
    }
    fn put_byte(&mut self, address: usize, value: u8) -> bool {
        let Some((location, lane)) = location(address) else {
            return false;
        };
        let word = self.word_mut(location);
        let mut bytes = word.to_le_bytes();
        bytes[lane] = value;
        *word = u16::from_le_bytes(bytes);
        true
    }
}
