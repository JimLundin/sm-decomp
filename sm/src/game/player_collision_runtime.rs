//! Finite content and synchronous runtime effects for owned player collision.
//! Registered beneath session_runtime so checked loaded reads reuse Game's codec.
use crate::{
    game::{
        player_collision::effects::{Content, Effects, ImportedCollisionReads, MechanismHeader},
        room_entry::{self, ArrivalRequest, ContactSite, EntryPoll},
        room_mechanisms::{
            creation,
            host::{Contact, SpawnSource},
        },
        session_runtime::Runtime,
        visual_host::{self, Source},
    },
    rom::RomRecord,
};

impl Content for Runtime<'_> {
    type Record = RomRecord;

    fn pose_table(&mut self) -> RomRecord {
        visual_host::Content::table(self, Source::Native, 0x91b629)
    }

    fn walked_pose_table(&mut self) -> RomRecord {
        // The caller captures this before evaluating its native pose-table index.
        visual_host::Content::table(self, Source::Loaded, 0x91eb74)
    }

    fn special_header(&mut self, behavior: u8, solid: bool) -> MechanismHeader {
        let header = if behavior & 0x80 != 0 {
            let area = self.session().room.area;
            let table = self.game.read_rom_word(
                if solid { 0x9492e9 } else { 0x9492d9 },
                usize::from(area) * 2,
            );
            if table & 0x8000 == 0 {
                // Preserve diagnostic, verifier callback, and native allocation
                // acquisition before the later fresh loaded-ROM lookup.
                self.foreign(|game| unsafe {
                    crate::rtl::rom_bank_ptr::<0x94>(table, game);
                });
            }
            self.game.read_rom_word(
                0x948000 | u32::from(table),
                usize::from(behavior & 0x7f) * 2,
            )
        } else {
            self.game.read_rom_word(0x949139, usize::from(behavior) * 2)
        };
        MechanismHeader(header)
    }

    fn bomb_header(&mut self, behavior: u8) -> MechanismHeader {
        MechanismHeader(self.game.read_rom_word(0x94936b, usize::from(behavior) * 2))
    }
}

impl ImportedCollisionReads for Runtime<'_> {
    fn imported_level_word(&self, byte_offset: usize) -> u16 {
        self.word(0x10002 + byte_offset)
    }

    fn imported_behavior_byte(&self, index: usize) -> u8 {
        self.byte(0x16402 + index)
    }

    fn imported_interactive_word(&self, byte_offset: usize) -> u16 {
        self.word(0x17ec + byte_offset)
    }
}

impl Effects for Runtime<'_> {
    fn apply_mechanism(&mut self, header: MechanismHeader, displacement: &mut i32) -> u8 {
        self.owned_call(|runtime| {
            creation::spawn(
                runtime,
                SpawnSource::Collision { header: header.0 },
                &mut Contact::Local(displacement),
            )
        })
    }

    fn enter_door(&mut self) -> EntryPoll {
        room_entry::begin(self, ArrivalRequest::Contact(ContactSite::CurrentCollision))
    }

    fn cancel_speed_boost(&mut self) {
        crate::game::samus_palette::cancel_speed_boost(self);
    }

    fn boss_defeated(&mut self) -> bool {
        crate::game::progress_flags::boss_defeated(self, 1)
    }

    fn vertical_slope_bts(&mut self, block: u16) -> u16 {
        let address = 0x7f6402 + u32::from(block);
        self.foreign(|game| unsafe {
            let low = crate::snes::snes::snes_read(game.g_snes, address, game);
            let high = crate::snes::snes::snes_read(game.g_snes, address + 1, game);
            u16::from_le_bytes([low, high])
        })
    }

    fn invalid_clearance_flags(&mut self, _flags: u16) {
        self.foreign(|game| {
            crate::rtl::Unreachable(game);
        });
    }
}

use crate::game_state::Game;

pub(crate) fn set_player_radius(game: &mut Game) {
    let pose_offset = usize::from(game.samus_pose()) * 8;
    let radius = u16::from(game.read_rom_byte(0x91b629, pose_offset + 6));
    game.g_ram[0xb00..0xb02].copy_from_slice(&radius.to_le_bytes());
    game.g_ram[0xafe..0xb00].copy_from_slice(&5u16.to_le_bytes());
}
pub(crate) fn player_bottom(game: &Game) -> u16 {
    let y = game.samus_y_pos();
    // SamusPoseParams has eight bytes, with y_radius at byte 6.
    let pose_offset = usize::from(game.samus_pose()) * 8;
    let radius = game.read_rom_byte(0x91b629, pose_offset + 6);
    y.wrapping_add(u16::from(radius)).wrapping_sub(1)
}
pub(crate) fn settle_grapple(game: &mut Game) {
    crate::game::player_collision::operations::settle_after_grapple(
        &mut crate::game::session_runtime::Runtime::new(game),
    );
}
