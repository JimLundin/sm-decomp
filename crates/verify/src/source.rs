//! The original's work RAM, as the named sources the decode rules read.
//!
//! Each source carries its `ram_map.rs` name (`crates/sm_host/src/ram_map.rs`, from
//! `sm_game::ram_table!`) and its offset in bank $7E, checked against the bank logs
//! (<https://patrickjohnston.org/bank/>). The engine crates hold no addresses (K1); this
//! table is the one place the decoder learns where the original keeps a value.

use crate::decode::DecodeError;

/// A byte of work RAM.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct WramByte {
    /// The `ram_map.rs` name.
    pub name: &'static str,
    /// The offset from $7E:0000.
    pub offset: u16,
}

/// A little-endian 16-bit word of work RAM.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct WramWord {
    /// The `ram_map.rs` name.
    pub name: &'static str,
    /// The offset of the low byte from $7E:0000.
    pub offset: u16,
}

/// A run of work RAM bytes.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct WramBlock {
    /// The `ram_map.rs` name.
    pub name: &'static str,
    /// The offset of the first byte from $7E:0000; bank $7F's follow from $1:0000.
    pub offset: u32,
    /// The length in bytes.
    pub len: u16,
}

impl WramBlock {
    /// The bytes in `wram`, a record's work RAM from $7E:0000.
    ///
    /// # Errors
    ///
    /// [`DecodeError::WramTooShort`] when `wram` ends before the block does.
    pub fn read(self, wram: &[u8]) -> Result<&[u8], DecodeError> {
        usize::try_from(self.offset)
            .ok()
            .and_then(|start| wram.get(start..start.checked_add(usize::from(self.len))?))
            .ok_or(DecodeError::WramTooShort { source: self.name })
    }
}

impl WramByte {
    /// The byte in `wram`, a record's work RAM from $7E:0000.
    ///
    /// # Errors
    ///
    /// [`DecodeError::WramTooShort`] when `wram` ends before the byte.
    pub fn read(self, wram: &[u8]) -> Result<u8, DecodeError> {
        wram.get(usize::from(self.offset))
            .copied()
            .ok_or(DecodeError::WramTooShort { source: self.name })
    }
}

impl WramWord {
    /// The word in `wram`, a record's work RAM from $7E:0000.
    ///
    /// # Errors
    ///
    /// [`DecodeError::WramTooShort`] when `wram` ends before the word's high byte.
    pub fn read(self, wram: &[u8]) -> Result<u16, DecodeError> {
        wram.get(usize::from(self.offset)..)
            .and_then(<[u8]>::first_chunk::<2>)
            .map(|bytes| u16::from_le_bytes(*bytes))
            .ok_or(DecodeError::WramTooShort { source: self.name })
    }
}

/// Bank $80 log: the NMI handler counts every NMI here, lag ones included.
pub const NMI_FRAME_COUNTER_INCLUDING_LAG: WramWord = WramWord {
    name: "NMI_FRAME_COUNTER_INCLUDING_LAG",
    offset: 0x05B8,
};
/// Bank $80 log: the NMI handler counts the NMIs the main thread requested here.
pub const NMI_FRAME_COUNTER_WORD: WramWord = WramWord {
    name: "NMI_FRAME_COUNTER_WORD",
    offset: 0x05B6,
};
/// Bank $80 log: lag NMIs since the last requested one.
pub const NMI_FRAMES_MISSED: WramByte = WramByte {
    name: "NMI_FRAMES_MISSED",
    offset: 0x05BA,
};
/// Bank $80 log: the longest run of lag NMIs.
pub const NMI_FRAMES_MISSED_MAX: WramByte = WramByte {
    name: "NMI_FRAMES_MISSED_MAX",
    offset: 0x05BB,
};

/// Bank $82 log: the in-game time's frames within the second.
pub const GAME_TIME_FRAMES: WramWord = WramWord {
    name: "GAME_TIME_FRAMES",
    offset: 0x09DA,
};
/// Bank $82 log: the in-game time's seconds.
pub const GAME_TIME_SECONDS: WramWord = WramWord {
    name: "GAME_TIME_SECONDS",
    offset: 0x09DC,
};
/// Bank $82 log: the in-game time's minutes.
pub const GAME_TIME_MINUTES: WramWord = WramWord {
    name: "GAME_TIME_MINUTES",
    offset: 0x09DE,
};
/// Bank $82 log: the in-game time's hours.
pub const GAME_TIME_HOURS: WramWord = WramWord {
    name: "GAME_TIME_HOURS",
    offset: 0x09E0,
};

/// Bank $80 log: the RNG word the random number routine advances.
pub const RANDOM_NUMBER: WramWord = WramWord {
    name: "RANDOM_NUMBER",
    offset: 0x05E5,
};

/// Bank $80 log: controller 1 held this frame, in JOY1 order.
pub const JOYPAD1_LASTKEYS: WramWord = WramWord {
    name: "JOYPAD1_LASTKEYS",
    offset: 0x008B,
};
/// Bank $80 log: controller 1 newly pressed this frame.
pub const JOYPAD1_NEWKEYS: WramWord = WramWord {
    name: "JOYPAD1_NEWKEYS",
    offset: 0x008F,
};
/// Bank $80 log: controller 1's auto-repeat presses.
pub const JOYPAD1_NEWKEYS2_UNUSED: WramWord = WramWord {
    name: "JOYPAD1_NEWKEYS2_UNUSED",
    offset: 0x0093,
};
/// Bank $80 log: controller 1 held last frame.
pub const JOYPAD1_PREV: WramWord = WramWord {
    name: "JOYPAD1_PREV",
    offset: 0x0097,
};
/// Bank $80 log: frames until controller 1's next auto-repeat.
pub const JOYPAD1_KEYREPEAT_CTR: WramWord = WramWord {
    name: "JOYPAD1_KEYREPEAT_CTR",
    offset: 0x00A3,
};

/// Bank $82 log: the game state the main loop dispatches on.
pub const GAME_STATE: WramWord = WramWord {
    name: "GAME_STATE",
    offset: 0x0998,
};

/// Bank $90 log: Samus's X position, whole pixels.
pub const SAMUS_X_POS: WramWord = WramWord {
    name: "SAMUS_X_POS",
    offset: 0x0AF6,
};
/// Bank $90 log: Samus's X position, subpixels.
pub const SAMUS_X_SUBPOS: WramWord = WramWord {
    name: "SAMUS_X_SUBPOS",
    offset: 0x0AF8,
};
/// Bank $90 log: Samus's Y position, whole pixels.
pub const SAMUS_Y_POS: WramWord = WramWord {
    name: "SAMUS_Y_POS",
    offset: 0x0AFA,
};
/// Bank $90 log: Samus's Y position, subpixels.
pub const SAMUS_Y_SUBPOS: WramWord = WramWord {
    name: "SAMUS_Y_SUBPOS",
    offset: 0x0AFC,
};

/// Bank $80 log, update OAM and CGRAM `$80:933A`: the OAM buffer the NMI copies to OAM,
/// $0370..$058F, the low table and then the high table.
pub const OAM: WramBlock = WramBlock {
    name: "OAM",
    offset: 0x0370,
    len: 0x0220,
};

/// Bank $82 log, load room header: the room's width in blocks, 16 a screen.
pub const ROOM_WIDTH_IN_BLOCKS: WramWord = WramWord {
    name: "ROOM_WIDTH_IN_BLOCKS",
    offset: 0x07A5,
};

/// Bank $82 log, load room header: the room's height in blocks, 16 a screen.
pub const ROOM_HEIGHT_IN_BLOCKS: WramWord = WramWord {
    name: "ROOM_HEIGHT_IN_BLOCKS",
    offset: 0x07A7,
};

/// Bank $94 log, the block reactions (`LDA $7F0002,x`): layer 1, a word a block, row-major;
/// the level data decompresses to $7F:0000 behind its size word (bank $82 log, `$82:E7D3`),
/// room for 50 screens.
pub const LEVEL_DATA: WramBlock = WramBlock {
    name: "LEVEL_DATA",
    offset: 0x1_0002,
    len: 0x6400,
};

/// Bank $94 log, the block reactions (`LDA $7F6402,x`): the BTS, a byte a block, which
/// level loading moves here from behind layer 1 (bank $82 log, `$82:E7D3`).
pub const BTS: WramBlock = WramBlock {
    name: "BTS",
    offset: 0x1_6402,
    len: 0x3200,
};

/// Bank $82 log, `$82:EA73`: the scroll areas, a byte a screen, row-major: 0 red, 1 blue,
/// 2 green; 50 bytes.
pub const SCROLLS: WramBlock = WramBlock {
    name: "SCROLLS",
    offset: 0xCD20,
    len: 0x0032,
};

/// Bank $82 log: the current room's header, a pointer into bank $8F.
pub const ROOM_PTR: WramWord = WramWord {
    name: "ROOM_PTR",
    offset: 0x079B,
};

/// Bank $A3 log, `$A3:9581`: the way the last elevator departure went, 0 down or $8000 up,
/// which the door transition reads.
pub const ELEVATOR_DIRECTION: WramWord = WramWord {
    name: "ELEVATOR_DIRECTION",
    offset: 0x0799,
};

/// Bank $A3 log, `$A3:952A`: the elevator properties, nonzero while the elevator AI runs
/// with no ride on.
///
/// Door loading sets it to the door's elevator property bit $80, standing in the elevator's
/// doorway to 1 (bank $82 log), and an idle frame without the call button clears it
/// (`$A3:9575`).
pub const ELEVATOR_FLAGS: WramWord = WramWord {
    name: "ELEVATOR_FLAGS",
    offset: 0x0E16,
};

/// Bank $A3 log, `$A3:9540`: the elevator status, 0 inactive, 1 leaving the room, 2 the door
/// transition, 3 entering the room.
pub const ELEVATOR_STATUS: WramWord = WramWord {
    name: "ELEVATOR_STATUS",
    offset: 0x0E18,
};

/// Bank $80 log: the NMI request flag. The main thread sets it to 1 when it waits for NMI
/// and the NMI handler clears it; a set flag is what makes an NMI a requested one.
pub const WAITING_FOR_NMI: WramByte = WramByte {
    name: "WAITING_FOR_NMI",
    offset: 0x05B4,
};

/// Bank $84 log: the items switched on, one bit each.
///
/// The item PLMs' equipment instruction sets them: Varia $0001, spring ball $0002, morph
/// ball $0004, screw attack $0008, gravity $0020, hi-jump $0100, space jump $0200, bombs
/// $1000, speed booster $2000, grapple $4000, X-ray $8000. The other five bits are unused.
pub const EQUIPPED_ITEMS: WramWord = WramWord {
    name: "EQUIPPED_ITEMS",
    offset: 0x09A2,
};
/// Bank $84 log: the items collected, in `EQUIPPED_ITEMS`'s bit layout.
pub const COLLECTED_ITEMS: WramWord = WramWord {
    name: "COLLECTED_ITEMS",
    offset: 0x09A4,
};
/// Bank $90 log: Samus's energy.
pub const SAMUS_HEALTH: WramWord = WramWord {
    name: "SAMUS_HEALTH",
    offset: 0x09C2,
};
/// Bank $90 log: Samus's energy ceiling, raised by energy tanks.
pub const SAMUS_MAX_HEALTH: WramWord = WramWord {
    name: "SAMUS_MAX_HEALTH",
    offset: 0x09C4,
};
/// Bank $90 log: Samus's missiles.
pub const SAMUS_MISSILES: WramWord = WramWord {
    name: "SAMUS_MISSILES",
    offset: 0x09C6,
};
/// Bank $90 log: Samus's missile ceiling, raised by missile tanks.
pub const SAMUS_MAX_MISSILES: WramWord = WramWord {
    name: "SAMUS_MAX_MISSILES",
    offset: 0x09C8,
};
/// Bank $90 log: Samus's super missiles.
pub const SAMUS_SUPER_MISSILES: WramWord = WramWord {
    name: "SAMUS_SUPER_MISSILES",
    offset: 0x09CA,
};
/// Bank $90 log: Samus's super missile ceiling.
pub const SAMUS_MAX_SUPER_MISSILES: WramWord = WramWord {
    name: "SAMUS_MAX_SUPER_MISSILES",
    offset: 0x09CC,
};
/// Bank $90 log: Samus's power bombs.
pub const SAMUS_POWER_BOMBS: WramWord = WramWord {
    name: "SAMUS_POWER_BOMBS",
    offset: 0x09CE,
};
/// Bank $90 log: Samus's power bomb ceiling.
pub const SAMUS_MAX_POWER_BOMBS: WramWord = WramWord {
    name: "SAMUS_MAX_POWER_BOMBS",
    offset: 0x09D0,
};
/// Bank $80 log, `$80:9B4E`: the reserve tanks' mode, 0 none, 1 automatic, 2 manual.
pub const RESERVE_HEALTH_MODE: WramWord = WramWord {
    name: "RESERVE_HEALTH_MODE",
    offset: 0x09C0,
};
/// Bank $84 log: the reserve tanks' ceiling, raised by each reserve tank.
pub const SAMUS_MAX_RESERVE_HEALTH: WramWord = WramWord {
    name: "SAMUS_MAX_RESERVE_HEALTH",
    offset: 0x09D4,
};
/// Bank $90 log: the energy held in the reserve tanks.
pub const SAMUS_RESERVE_HEALTH: WramWord = WramWord {
    name: "SAMUS_RESERVE_HEALTH",
    offset: 0x09D6,
};

/// Bank $80 log: layer 1's X scroll in the room, subpixels.
pub const LAYER1_X_SUBPOS: WramWord = WramWord {
    name: "LAYER1_X_SUBPOS",
    offset: 0x090F,
};
/// Bank $80 log: layer 1's X scroll in the room, whole pixels: the camera's left edge.
pub const LAYER1_X_POS: WramWord = WramWord {
    name: "LAYER1_X_POS",
    offset: 0x0911,
};
/// Bank $80 log: layer 1's Y scroll in the room, subpixels.
pub const LAYER1_Y_SUBPOS: WramWord = WramWord {
    name: "LAYER1_Y_SUBPOS",
    offset: 0x0913,
};
/// Bank $80 log: layer 1's Y scroll in the room, whole pixels: the camera's top edge.
pub const LAYER1_Y_POS: WramWord = WramWord {
    name: "LAYER1_Y_POS",
    offset: 0x0915,
};

/// Bank $80 log, `$80:A2F9`: layer 2's X position, which follows layer 1 at the room's rate.
pub const LAYER2_X_POS: WramWord = WramWord {
    name: "LAYER2_X_POS",
    offset: 0x0917,
};
/// Bank $80 log, `$80:A33A`: layer 2's Y position.
pub const LAYER2_Y_POS: WramWord = WramWord {
    name: "LAYER2_Y_POS",
    offset: 0x0919,
};
/// Bank $80 log, `$80:A2F9`: layer 2's X scroll rate, from the room state.
pub const LAYER2_SCROLL_X: WramByte = WramByte {
    name: "LAYER2_SCROLL_X",
    offset: 0x091B,
};
/// Bank $80 log, `$80:A33A`: layer 2's Y scroll rate, from the room state.
pub const LAYER2_SCROLL_Y: WramByte = WramByte {
    name: "LAYER2_SCROLL_Y",
    offset: 0x091C,
};
/// Bank $80 log, `$80:A3AB`: what BG1's X scroll adds to layer 1's X position.
pub const BG1_X_OFFSET: WramWord = WramWord {
    name: "BG1_X_OFFSET",
    offset: 0x091D,
};
/// Bank $80 log, `$80:A3AB`: what BG1's Y scroll adds to layer 1's Y position.
pub const BG1_Y_OFFSET: WramWord = WramWord {
    name: "BG1_Y_OFFSET",
    offset: 0x091F,
};
/// Bank $80 log, `$80:A3AB`: what BG2's X scroll adds to layer 2's X position.
pub const BG2_X_OFFSET: WramWord = WramWord {
    name: "BG2_X_OFFSET",
    offset: 0x0921,
};
/// Bank $80 log, `$80:A3AB`: what BG2's Y scroll adds to layer 2's Y position.
pub const BG2_Y_OFFSET: WramWord = WramWord {
    name: "BG2_Y_OFFSET",
    offset: 0x0923,
};
/// Bank $90 log, `$90:95E7`: the camera distance index, 0 to 6.
pub const CAMERA_DISTANCE_INDEX: WramWord = WramWord {
    name: "CAMERA_DISTANCE_INDEX",
    offset: 0x0941,
};
/// Bank $90 log, `$90:966E`: the room header's up scroller.
pub const UP_SCROLLER: WramWord = WramWord {
    name: "UP_SCROLLER",
    offset: 0x07AD,
};
/// Bank $90 log, `$90:967A`: the room header's down scroller.
pub const DOWN_SCROLLER: WramWord = WramWord {
    name: "DOWN_SCROLLER",
    offset: 0x07AF,
};
/// Bank $90 log, `$90:956E`: Samus's X position when the camera last followed her.
pub const SAMUS_PREV_X_POS: WramWord = WramWord {
    name: "SAMUS_PREV_X_POS",
    offset: 0x0B10,
};
/// Bank $90 log, `$90:9574`: its subpixels.
pub const SAMUS_PREV_X_SUBPOS: WramWord = WramWord {
    name: "SAMUS_PREV_X_SUBPOS",
    offset: 0x0B12,
};
/// Bank $90 log, `$90:957A`: Samus's Y position when the camera last followed her.
pub const SAMUS_PREV_Y_POS: WramWord = WramWord {
    name: "SAMUS_PREV_Y_POS",
    offset: 0x0B14,
};
/// Bank $90 log, `$90:9580`: its subpixels.
pub const SAMUS_PREV_Y_SUBPOS: WramWord = WramWord {
    name: "SAMUS_PREV_Y_SUBPOS",
    offset: 0x0B16,
};

/// Bank $84 log: the PLM handler keeps the slot it is processing here, times two
/// (`$84:85C5 STX $1C27`).
pub const PLM_ID: WramWord = WramWord {
    name: "PLM_ID",
    offset: 0x1C27,
};

/// Bank $85 log: the message box animation's Y radius, which open and close step by $200.
pub const MESSAGE_BOX_ANIMATION_Y_RADIUS: WramWord = WramWord {
    name: "MESSAGE_BOX_ANIMATION_Y_RADIUS",
    offset: 0x05A2,
};

/// Bank $85 log: the message box's message index, stored on entry to the display routine
/// (`$85:8080`).
pub const MESSAGE_BOX_INDEX: WramWord = WramWord {
    name: "MESSAGE_BOX_INDEX",
    offset: 0x1C1F,
};

/// Bank $80 log: the interrupt command the IRQ handler runs next.
pub const CUR_IRQ_HANDLER: WramWord = WramWord {
    name: "CUR_IRQ_HANDLER",
    offset: 0x00AB,
};

/// Bank $80 log: the interrupt command a door transition queues after the HUD's.
pub const IRQHANDLER_NEXT_HANDLER: WramWord = WramWord {
    name: "IRQHANDLER_NEXT_HANDLER",
    offset: 0x00A7,
};

/// Bank $80 log: the frames a pad must stay held before its first auto-repeat, which boot
/// sets and the controller read (`$80:9459`) reloads the repeat counter from.
pub const JOYPAD_CTR_REPEAT_FIRST: WramWord = WramWord {
    name: "JOYPAD_CTR_REPEAT_FIRST",
    offset: 0x0087,
};

/// Bank $80 log: the frames between later auto-repeats, which boot sets.
pub const JOYPAD_CTR_REPEAT_NEXT: WramWord = WramWord {
    name: "JOYPAD_CTR_REPEAT_NEXT",
    offset: 0x0089,
};

/// Bank $80 log, update OAM and CGRAM `$80:933A`: the palette buffer every requested NMI
/// uploads to CGRAM whole, 256 little-endian colours.
pub const PALETTE_BUFFER: WramBlock = WramBlock {
    name: "PALETTE_BUFFER",
    offset: 0xC000,
    len: 0x0200,
};

/// Bank $80 log, process VRAM write table `$80:8C83`: the uploads game code queues.
///
/// Seven bytes each: the size in bytes, the 24-bit source and the VRAM
/// word address, whose top bit selects the 32-word step.
pub const VRAM_WRITE_QUEUE: WramBlock = WramBlock {
    name: "vram_write_queue",
    offset: 0x00D0,
    len: 0x0200,
};

/// Bank $80 log: the VRAM write queue's end, in bytes; the NMI clears it.
pub const VRAM_WRITE_QUEUE_TAIL: WramWord = WramWord {
    name: "VRAM_WRITE_QUEUE_TAIL",
    offset: 0x0330,
};

/// Bank $80 log, update IO registers `$80:91EE`: INIDISP's mirror.
pub const REG_INIDISP: WramByte = WramByte {
    name: "REG_INIDISP",
    offset: 0x0051,
};

/// Bank $80 log, `$80:91EE`: TM's mirror, the main screen's layers.
pub const REG_TM: WramByte = WramByte {
    name: "REG_TM",
    offset: 0x0069,
};

/// Bank $80 log, `$80:91EE`: TS's mirror, the sub screen's layers.
pub const REG_TS: WramByte = WramByte {
    name: "REG_TS",
    offset: 0x006B,
};

/// Bank $80 log, `$80:91EE`: OBSEL's mirror.
pub const REG_OBSEL: WramByte = WramByte {
    name: "REG_OBSEL",
    offset: 0x0052,
};

/// Bank $80 log, `$80:91EE`: BGMODE's mirror.
pub const REG_BGMODE: WramByte = WramByte {
    name: "REG_BGMODE",
    offset: 0x0055,
};

/// Bank $80 log, `$80:91EE`: MOSAIC's mirror.
pub const REG_MOSAIC: WramByte = WramByte {
    name: "REG_MOSAIC",
    offset: 0x0057,
};

/// Bank $80 log, `$80:91EE`: the tilemap base mirrors, BG1SC to BG4SC (`$5B` between BG3's and BG4's is gameplay's
/// saved BG3SC).
pub const REG_BG_SC: [WramByte; 4] = [
    WramByte {
        name: "REG_BG1SC",
        offset: 0x0058,
    },
    WramByte {
        name: "REG_BG2SC",
        offset: 0x0059,
    },
    WramByte {
        name: "REG_BG3SC",
        offset: 0x005A,
    },
    WramByte {
        name: "REG_BG4SC",
        offset: 0x005C,
    },
];

/// Bank $80 log, `$80:91EE`: the tile base mirrors, BG12NBA and BG34NBA.
pub const REG_BG_NBA: [WramByte; 2] = [
    WramByte {
        name: "REG_BG12NBA",
        offset: 0x005D,
    },
    WramByte {
        name: "REG_BG34NBA",
        offset: 0x005E,
    },
];

/// Bank $80 log, `$80:91EE`: M7SEL's mirror.
pub const REG_M7SEL: WramByte = WramByte {
    name: "REG_M7SEL",
    offset: 0x005F,
};

/// Bank $80 log, `$80:91EE`: the window settings' mirrors, W12SEL, W34SEL, WOBJSEL.
pub const REG_WINDOW_SELECT: [WramByte; 3] = [
    WramByte {
        name: "REG_W12SEL",
        offset: 0x0060,
    },
    WramByte {
        name: "REG_W34SEL",
        offset: 0x0061,
    },
    WramByte {
        name: "REG_WOBJSEL",
        offset: 0x0062,
    },
];

/// Bank $80 log, `$80:91EE`: the window edges' mirrors, WH0 to WH3.
pub const REG_WINDOW_EDGES: [WramByte; 4] = [
    WramByte {
        name: "REG_WH0",
        offset: 0x0063,
    },
    WramByte {
        name: "REG_WH1",
        offset: 0x0064,
    },
    WramByte {
        name: "REG_WH2",
        offset: 0x0065,
    },
    WramByte {
        name: "REG_WH3",
        offset: 0x0066,
    },
];

/// Bank $80 log, `$80:91EE`: WBGLOG's mirror.
pub const REG_WBGLOG: WramByte = WramByte {
    name: "REG_WBGLOG",
    offset: 0x0067,
};

/// Bank $80 log, `$80:91EE`: WOBJLOG's mirror.
pub const REG_WOBJLOG: WramByte = WramByte {
    name: "REG_WOBJLOG",
    offset: 0x0068,
};

/// Bank $80 log, `$80:91EE`: TMW's mirror.
pub const REG_TMW: WramByte = WramByte {
    name: "REG_TMW",
    offset: 0x006C,
};

/// Bank $80 log, `$80:91EE`: TSW's mirror.
pub const REG_TSW: WramByte = WramByte {
    name: "REG_TSW",
    offset: 0x006D,
};

/// Bank $80 log, `$80:91EE`: CGWSEL's mirror.
pub const REG_CGWSEL: WramByte = WramByte {
    name: "REG_CGWSEL",
    offset: 0x006F,
};

/// Bank $80 log, `$80:91EE`: CGADSUB's mirror.
pub const REG_CGADSUB: WramByte = WramByte {
    name: "REG_CGADSUB",
    offset: 0x0072,
};

/// Bank $80 log, `$80:91EE`: the three bytes written to COLDATA in turn (`REG_COLDATA`).
pub const REG_COLDATA: [WramByte; 3] = [
    WramByte {
        name: "REG_COLDATA",
        offset: 0x0074,
    },
    WramByte {
        name: "REG_COLDATA[1]",
        offset: 0x0075,
    },
    WramByte {
        name: "REG_COLDATA[2]",
        offset: 0x0076,
    },
];

/// Bank $80 log, `$80:91EE`: SETINI's mirror.
pub const REG_SETINI: WramByte = WramByte {
    name: "REG_SETINI",
    offset: 0x0077,
};

/// Bank $80 log, `$80:91EE`: the mode 7 mirrors, M7A to M7Y, written only in mode 7.
pub const REG_M7: [WramWord; 6] = [
    WramWord {
        name: "REG_M7A",
        offset: 0x0078,
    },
    WramWord {
        name: "REG_M7B",
        offset: 0x007A,
    },
    WramWord {
        name: "REG_M7C",
        offset: 0x007C,
    },
    WramWord {
        name: "REG_M7D",
        offset: 0x007E,
    },
    WramWord {
        name: "REG_M7X",
        offset: 0x0080,
    },
    WramWord {
        name: "REG_M7Y",
        offset: 0x0082,
    },
];

/// Bank $80 log, `$80:91EE`: the scroll mirrors of backgrounds 1 to 4, horizontal then
/// vertical: `REG_BG1HOFS` to `REG_BG4VOFS`, written in pairs to `$210D`-`$2114`.
pub const REG_BG_SCROLL: [[WramWord; 2]; 4] = [
    [
        WramWord {
            name: "REG_BG1HOFS",
            offset: 0x00B1,
        },
        WramWord {
            name: "REG_BG1VOFS",
            offset: 0x00B3,
        },
    ],
    [
        WramWord {
            name: "REG_BG2HOFS",
            offset: 0x00B5,
        },
        WramWord {
            name: "REG_BG2VOFS",
            offset: 0x00B7,
        },
    ],
    [
        WramWord {
            name: "REG_BG3HOFS",
            offset: 0x00B9,
        },
        WramWord {
            name: "REG_BG3VOFS",
            offset: 0x00BB,
        },
    ],
    [
        WramWord {
            name: "REG_BG4HOFS",
            offset: 0x00BD,
        },
        WramWord {
            name: "REG_BG4VOFS",
            offset: 0x00BF,
        },
    ],
];

/// Bank $90 log, `$90:90E2`: Samus's vertical speed's subpixels.
pub const SAMUS_Y_SUBSPEED: WramWord = WramWord {
    name: "SAMUS_Y_SUBSPEED",
    offset: 0x0B2C,
};
/// Bank $90 log, `$90:90E2`: Samus's vertical speed's pixels a frame.
pub const SAMUS_Y_SPEED: WramWord = WramWord {
    name: "SAMUS_Y_SPEED",
    offset: 0x0B2E,
};
/// Bank $90 log, `$90:90E2`: 0 none, 1 up, 2 down.
pub const SAMUS_Y_DIR: WramWord = WramWord {
    name: "SAMUS_Y_DIR",
    offset: 0x0B36,
};
/// Bank $90 log, `$90:973E`: whether Samus kept her running momentum (the extra run speed
/// survives leaving the running movement), 0 or 1.
pub const SAMUS_HAS_MOMENTUM_FLAG: WramWord = WramWord {
    name: "SAMUS_HAS_MOMENTUM_FLAG",
    offset: 0x0B3C,
};
/// Bank $90 log, `$90:973E`: the extra run speed's pixels a frame.
pub const SAMUS_X_EXTRA_RUN_SPEED: WramWord = WramWord {
    name: "SAMUS_X_EXTRA_RUN_SPEED",
    offset: 0x0B42,
};
/// Bank $90 log, `$90:973E`: the extra run speed's subpixels.
pub const SAMUS_X_EXTRA_RUN_SUBSPEED: WramWord = WramWord {
    name: "SAMUS_X_EXTRA_RUN_SUBSPEED",
    offset: 0x0B44,
};
/// Bank $90 log, `$90:9A7E`: the base speed's pixels a frame.
pub const SAMUS_X_BASE_SPEED: WramWord = WramWord {
    name: "SAMUS_X_BASE_SPEED",
    offset: 0x0B46,
};
/// Bank $90 log, `$90:9A7E`: the base speed's subpixels.
pub const SAMUS_X_BASE_SUBSPEED: WramWord = WramWord {
    name: "SAMUS_X_BASE_SUBSPEED",
    offset: 0x0B48,
};
/// Bank $90 log, `$90:9A7E`: 0 accelerating, 1 turning around, 2 decelerating.
pub const SAMUS_X_ACCEL_MODE: WramWord = WramWord {
    name: "SAMUS_X_ACCEL_MODE",
    offset: 0x0B4A,
};
/// Bank $90/$91 logs: Samus's pose, an index into the pose definitions (`$91:B629`).
pub const SAMUS_POSE: WramWord = WramWord {
    name: "SAMUS_POSE",
    offset: 0x0A1C,
};
/// Bank $91 log: the pose's X direction, copied from its definition.
pub const SAMUS_POSE_X_DIR: WramByte = WramByte {
    name: "SAMUS_POSE_X_DIR",
    offset: 0x0A1E,
};
/// Bank $91 log: the pose's movement type, copied from its definition; the movement
/// handler dispatches on it.
pub const SAMUS_MOVEMENT_TYPE: WramByte = WramByte {
    name: "SAMUS_MOVEMENT_TYPE",
    offset: 0x0A1F,
};
/// Bank $91 log: the pose before the last pose change.
pub const SAMUS_PREV_POSE: WramWord = WramWord {
    name: "SAMUS_PREV_POSE",
    offset: 0x0A20,
};
/// Bank $91 log: the previous pose's X direction.
pub const SAMUS_PREV_POSE_X_DIR: WramByte = WramByte {
    name: "SAMUS_PREV_POSE_X_DIR",
    offset: 0x0A22,
};
/// Bank $91 log: the previous pose's movement type.
pub const SAMUS_PREV_MOVEMENT_TYPE2: WramByte = WramByte {
    name: "SAMUS_PREV_MOVEMENT_TYPE2",
    offset: 0x0A23,
};
/// Bank $91 log: the last pose that differed from the current one.
pub const SAMUS_LAST_DIFFERENT_POSE: WramWord = WramWord {
    name: "SAMUS_LAST_DIFFERENT_POSE",
    offset: 0x0A24,
};
/// Bank $91 log: the last different pose's X direction.
pub const SAMUS_LAST_DIFFERENT_POSE_X_DIR: WramByte = WramByte {
    name: "SAMUS_LAST_DIFFERENT_POSE_X_DIR",
    offset: 0x0A26,
};
/// Bank $91 log: the last different pose's movement type.
pub const SAMUS_LAST_DIFFERENT_POSE_MOVEMENT_TYPE: WramByte = WramByte {
    name: "SAMUS_LAST_DIFFERENT_POSE_MOVEMENT_TYPE",
    offset: 0x0A27,
};

/// Bank $90 log, `$90:8000`: the frames Samus's animation frame still shows, a word that
/// goes negative before the frame advances.
pub const SAMUS_ANIM_FRAME_TIMER: WramWord = WramWord {
    name: "SAMUS_ANIM_FRAME_TIMER",
    offset: 0x0A94,
};
/// Bank $90 log, `$90:82DC`: Samus's animation frame, a byte offset into her pose's delay
/// list.
pub const SAMUS_ANIM_FRAME: WramWord = WramWord {
    name: "SAMUS_ANIM_FRAME",
    offset: 0x0A96,
};
/// Bank $91 log, `$91:FB08`: the frame a new pose's animation starts on, $8000 to keep it.
pub const SAMUS_ANIM_FRAME_SKIP: WramWord = WramWord {
    name: "SAMUS_ANIM_FRAME_SKIP",
    offset: 0x0A9A,
};
/// Bank $91 log, `$91:EB88`: the super special prospective pose, $FFFF for none.
pub const SAMUS_NEW_POSE_TRANSITIONAL: WramWord = WramWord {
    name: "SAMUS_NEW_POSE_TRANSITIONAL",
    offset: 0x0A2C,
};
/// Bank $90 log, `$90:EB4B`: Samus's drawing handler, a bank $90 routine pointer.
pub const SAMUS_DRAW_HANDLER: WramWord = WramWord {
    name: "SAMUS_DRAW_HANDLER",
    offset: 0x0A5C,
};
/// Bank $91 log, `$91:EB88`: the super special prospective pose change command, 0 to 8.
pub const SAMUS_HURT_SWITCH_INDEX: WramWord = WramWord {
    name: "SAMUS_HURT_SWITCH_INDEX",
    offset: 0x0A32,
};
/// Bank $91 log, `$91:D83F`: the visor palette timer in the low byte, the index into the
/// visor colours `$9B:A3C0` in the high byte (6, 8 or $A in the cycle).
pub const SAMUS_VISOR_PALETTE_TIMER_INDEX: WramWord = WramWord {
    name: "SAMUS_VISOR_PALETTE_TIMER_INDEX",
    offset: 0x0A72,
};
/// Bank $91 log, `$91:D849`: the default layer blending configuration, from the room's FX.
pub const FX_LAYER_BLENDING_CONFIG_A: WramWord = WramWord {
    name: "FX_LAYER_BLENDING_CONFIG_A",
    offset: 0x1982,
};

/// Bank $80 log, NMI `$80:95ED`: the requested NMIs in eight bits, which the HUD's
/// auto-cancel flash reads.
pub const NMI_FRAME_COUNTER_BYTE: WramByte = WramByte {
    name: "NMI_FRAME_COUNTER_BYTE",
    offset: 0x05B5,
};
/// Bank $80 log, `$80:9CAA`: the HUD tilemap, three rows of 32 words, queued to VRAM `$5820`
/// every gameplay frame.
pub const HUD_TILEMAP: WramBlock = WramBlock {
    name: "HUD_TILEMAP",
    offset: 0xC608,
    len: 0x00C0,
};
/// Bank $80 log, `$80:9C55`: the selected HUD item, 0 for none, then missiles to X-ray.
pub const HUD_ITEM_INDEX: WramWord = WramWord {
    name: "HUD_ITEM_INDEX",
    offset: 0x09D2,
};
/// Bank $80 log, `$80:9CA4`: the HUD item that cancels itself automatically.
pub const SAMUS_AUTO_CANCEL_HUD_ITEM_INDEX: WramWord = WramWord {
    name: "SAMUS_AUTO_CANCEL_HUD_ITEM_INDEX",
    offset: 0x0A04,
};
/// Bank $80 log, `$80:9B8E`: the energy the HUD last drew.
pub const SAMUS_PREV_HEALTH: WramWord = WramWord {
    name: "SAMUS_PREV_HEALTH",
    offset: 0x0A06,
};
/// Bank $80 log, `$80:9C08`: the missiles the HUD last drew.
pub const SAMUS_PREV_MISSILES: WramWord = WramWord {
    name: "SAMUS_PREV_MISSILES",
    offset: 0x0A08,
};
/// Bank $80 log, `$80:9C1E`: the super missiles the HUD last drew.
pub const SAMUS_PREV_SUPER_MISSILES: WramWord = WramWord {
    name: "SAMUS_PREV_SUPER_MISSILES",
    offset: 0x0A0A,
};
/// Bank $80 log, `$80:9C47`: the power bombs the HUD last drew.
pub const SAMUS_PREV_POWER_BOMBS: WramWord = WramWord {
    name: "SAMUS_PREV_POWER_BOMBS",
    offset: 0x0A0C,
};
/// Bank $80 log, `$80:9C58`: the HUD item the HUD last highlighted.
pub const SAMUS_PREV_HUD_ITEM_INDEX: WramWord = WramWord {
    name: "SAMUS_PREV_HUD_ITEM_INDEX",
    offset: 0x0A0E,
};
/// Bank $80 log, `$80:9CEA`: the palette bits the last item highlight wrote.
pub const HUD_ITEM_TILEMAP_PALETTE_BITS: WramWord = WramWord {
    name: "HUD_ITEM_TILEMAP_PALETTE_BITS",
    offset: 0x077C,
};

/// A table of equal-sized slots in work RAM, one per actor.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct WramTable {
    /// The `ram_map.rs` name.
    pub name: &'static str,
    /// The offset of slot 0 from $7E:0000.
    pub offset: u16,
    /// The bytes from one slot to the next.
    pub stride: u16,
}

/// A word every slot of a table holds, at the same place in each.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct SlotField {
    /// The field's name in `sm_host`'s slot layout.
    pub name: &'static str,
    /// Its offset from the slot's start.
    pub offset: u16,
}

impl WramTable {
    /// Where `field` of slot `slot` is.
    ///
    /// # Errors
    ///
    /// [`DecodeError::WramTooShort`] when the word lies past bank $7E.
    pub fn word(self, field: SlotField, slot: u16) -> Result<WramWord, DecodeError> {
        slot.checked_mul(self.stride)
            .and_then(|start| start.checked_add(self.offset))
            .and_then(|start| start.checked_add(field.offset))
            .map(|offset| WramWord {
                name: field.name,
                offset,
            })
            .ok_or(DecodeError::WramTooShort { source: field.name })
    }
}

/// Bank $A0 log: the enemy slots, `$40` bytes each from `$0F78`.
pub const ENEMY_DATA: WramTable = WramTable {
    name: "enemy_data",
    offset: 0x0F78,
    stride: 0x40,
};

/// A field of an enemy slot (bank $A0 log, `$0F78` onward).
const fn enemy(name: &'static str, offset: u16) -> SlotField {
    SlotField { name, offset }
}

/// The species: the pointer to its header in bank $A0, zero in a free slot.
pub const ENEMY_SPECIES: SlotField = enemy("enemy_ptr", 0x00);
/// The centre's X, whole pixels.
pub const ENEMY_X_POS: SlotField = enemy("x_pos", 0x02);
/// The centre's X, subpixels.
pub const ENEMY_X_SUBPOS: SlotField = enemy("x_subpos", 0x04);
/// The centre's Y, whole pixels.
pub const ENEMY_Y_POS: SlotField = enemy("y_pos", 0x06);
/// The centre's Y, subpixels.
pub const ENEMY_Y_SUBPOS: SlotField = enemy("y_subpos", 0x08);
/// Half the width.
pub const ENEMY_X_RADIUS: SlotField = enemy("x_width", 0x0A);
/// Half the height.
pub const ENEMY_Y_RADIUS: SlotField = enemy("y_height", 0x0C);
/// The properties word.
pub const ENEMY_PROPERTIES: SlotField = enemy("properties", 0x0E);
/// The extra properties word.
pub const ENEMY_EXTRA_PROPERTIES: SlotField = enemy("extra_properties", 0x10);
/// The AI handler bits: 1 grapple, 2 hurt, 4 frozen.
pub const ENEMY_AI_HANDLER_BITS: SlotField = enemy("ai_handler_bits", 0x12);
/// Health left.
pub const ENEMY_HEALTH: SlotField = enemy("health", 0x14);
/// The spritemap its instruction list shows, in its AI bank.
pub const ENEMY_SPRITEMAP: SlotField = enemy("spritemap_pointer", 0x16);
/// The general timer.
pub const ENEMY_TIMER: SlotField = enemy("timer", 0x18);
/// The instruction pointer, in its AI bank.
pub const ENEMY_INSTRUCTION: SlotField = enemy("current_instruction", 0x1A);
/// Frames until the next instruction.
pub const ENEMY_INSTRUCTION_TIMER: SlotField = enemy("instruction_timer", 0x1C);
/// The palette index.
pub const ENEMY_PALETTE_INDEX: SlotField = enemy("palette_index", 0x1E);
/// The VRAM tiles index.
pub const ENEMY_VRAM_TILES_INDEX: SlotField = enemy("vram_tiles_index", 0x20);
/// The layer it draws on.
pub const ENEMY_LAYER: SlotField = enemy("layer", 0x22);
/// The hurt flash timer.
pub const ENEMY_FLASH_TIMER: SlotField = enemy("flash_timer", 0x24);
/// The frozen timer.
pub const ENEMY_FROZEN_TIMER: SlotField = enemy("frozen_timer", 0x26);
/// The invincibility timer.
pub const ENEMY_INVINCIBILITY_TIMER: SlotField = enemy("invincibility_timer", 0x28);
/// The shake timer.
pub const ENEMY_SHAKE_TIMER: SlotField = enemy("shake_timer", 0x2A);
/// The frame counter.
pub const ENEMY_FRAME_COUNTER: SlotField = enemy("frame_counter", 0x2C);
/// The AI bank, the word's low byte: the bank its pointers are in.
pub const ENEMY_BANK: SlotField = enemy("bank", 0x2E);
/// The six AI words, `var_a` to `var_f`.
pub const ENEMY_AI_VARIABLES: [SlotField; 6] = [
    enemy("var_a", 0x30),
    enemy("var_b", 0x32),
    enemy("var_c", 0x34),
    enemy("var_d", 0x36),
    enemy("var_e", 0x38),
    enemy("var_f", 0x3A),
];
/// The population's first parameter.
pub const ENEMY_PARAMETER_1: SlotField = enemy("parameter_1", 0x3C);
/// The population's second parameter.
pub const ENEMY_PARAMETER_2: SlotField = enemy("parameter_2", 0x3E);
