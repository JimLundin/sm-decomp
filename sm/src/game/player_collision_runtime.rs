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
                self.report_native_diagnostic(0x940000 | u32::from(table));
                drop(self.samus_game().capture_native_rom_binding());
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
        let [low, high] = sm_game::player_collision::geometry::level_word_bytes(byte_offset);
        u16::from_le_bytes([self.byte(low), self.byte(high)])
    }

    fn imported_behavior_byte(&self, index: usize) -> u8 {
        self.byte(0x16402 + index)
    }

    fn imported_interactive_word(&self, byte_offset: usize) -> u16 {
        self.word(0x17ec + byte_offset)
    }
}

impl Effects for Runtime<'_> {
    #[cfg(sm_verify_dll)]
    fn record_launch(
        &mut self,
        direction: sm_game::launch_motion::MovementDirection,
        displacement: i32,
    ) {
        crate::game::player_collision_owned_checks::record_launch(direction, displacement);
    }
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
        sm_game::samus_palette::cancel_speed_boost(self);
    }

    fn boss_defeated(&mut self) -> bool {
        crate::game::progress_flags::boss_defeated(self, 1)
    }

    fn vertical_slope_bts(&mut self, block: u16) -> u16 {
        use sm_game::hardware::Bus;
        // A block past the level's end reads on into bank $80: its WRAM mirror,
        // the PPU and CPU registers, the WRAM port and the cartridge.
        let address = 0x7f6402 + u32::from(block);
        let low = self.read_bus_byte(address);
        let high = self.read_bus_byte(address + 1);
        u16::from_le_bytes([low, high])
    }

    fn invalid_clearance_flags(&mut self, _flags: u16) {
        crate::rtl::unreachable()
    }
}

/// Samus's collision radii for her pose, on the lease: the pose table's Y radius and an
/// X radius of 5.
pub(crate) fn set_player_radius(runtime: &mut Runtime<'_>) {
    let pose_offset = usize::from(runtime.session().player.pose) * 8;
    let radius = u16::from(runtime.rom_byte(0x91b629, pose_offset + 6));
    runtime.write_slice(0xb00, &radius.to_le_bytes());
    runtime.write_slice(0xafe, &5u16.to_le_bytes());
}
