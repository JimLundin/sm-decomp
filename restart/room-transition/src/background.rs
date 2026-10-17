//! Eight bank-8F background commands, including their private nested calls.
use crate::{Host, Result, Routine, SyncService as S, scheduler::Coordinator};

const ROUTINES: [Routine; 8] = [
    Routine::BackgroundFinish,
    Routine::BackgroundTransfer,
    Routine::BackgroundDecompress,
    Routine::BackgroundClearFx,
    Routine::BackgroundTransferAndSetBg3,
    Routine::BackgroundClearBg2,
    Routine::BackgroundClearKraidBg2,
    Routine::BackgroundDoorTransfer,
];

impl<H: Host> Coordinator<'_, H> {
    pub fn background(&mut self, mut cursor: u16) -> Result<()> {
        // Every legal command advances by at least its two-byte opcode. There
        // are no looping branch commands in this closed eight-command stream.
        for _ in 0..0x4000 {
            let opcode = self.rom.word(cursor)?;
            // C indexes by opcode >> 1; preserve this for the admitted 0..15
            // range instead of silently indexing outside its eight functions.
            if opcode >= 16 {
                return Err("invalid room background command".into());
            }
            cursor = self.background_command(opcode >> 1, cursor.wrapping_add(2))?;
            if cursor == 0 {
                return Ok(());
            }
        }
        Err("room background command stream did not terminate within bank 8F".into())
    }

    fn background_command(&mut self, command: u16, cursor: u16) -> Result<u16> {
        let routine = *ROUTINES.get(usize::from(command)).ok_or("invalid private background command")?;
        self.enter(routine, cursor)?;
        let result = match command {
            0 => 0,
            1 => {
                let bytes = self.rom.bytes(cursor, 7)?;
                let source = u32::from(bytes[0]) | u32::from(bytes[1]) << 8 | u32::from(bytes[2]) << 16;
                let vram_dst = u16::from_le_bytes([bytes[3], bytes[4]]);
                let size = u16::from_le_bytes([bytes[5], bytes[6]]);
                self.sync(S::CopyToVramNow { vram_dst, source, size })?;
                cursor.wrapping_add(7)
            }
            2 => {
                let bytes = self.rom.bytes(cursor, 5)?;
                let source = u32::from(bytes[0]) | u32::from(bytes[1]) << 8 | u32::from(bytes[2]) << 16;
                let destination = u16::from_le_bytes([bytes[3], bytes[4]]);
                self.sync(S::DecompressToMem { source, destination })?;
                cursor.wrapping_add(5)
            }
            3 => {
                self.sync(S::ClearFxTilemap)?;
                cursor
            }
            4 => {
                let next = self.background_command(1, cursor)?;
                self.ram.0[0x5e] = 2;
                next
            }
            5 => {
                self.sync(S::ClearBg2Tilemap)?;
                cursor
            }
            6 => {
                self.sync(S::ClearKraidBg2Tilemap)?;
                cursor
            }
            7 => {
                if self.ram.word(0x78d) == self.rom.word(cursor)? {
                    self.background_command(1, cursor.wrapping_add(2))?
                } else {
                    cursor.wrapping_add(9)
                }
            }
            _ => unreachable!(),
        };
        self.leave(routine, result)?;
        Ok(result)
    }
}
