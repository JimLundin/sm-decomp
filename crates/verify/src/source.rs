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

/// Bank $82 log: the current room's header, a pointer into bank $8F.
pub const ROOM_PTR: WramWord = WramWord {
    name: "ROOM_PTR",
    offset: 0x079B,
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
