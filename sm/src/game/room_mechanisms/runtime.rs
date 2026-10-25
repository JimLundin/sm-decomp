//! Finite imported aliases and real effects on the shared aggregate runtime.
use super::host::*;
use crate::game::session_runtime::Runtime;
use crate::ram_map::{word_arrays as arrays, words as ram};
use crate::types::EnemyData;
use core::mem::offset_of;

fn slot_base(member: SlotMember) -> usize {
    match member {
        SlotMember::Header => 0x1c37,
        SlotMember::Block => 0x1c87,
        SlotMember::PreAction => 0x1cd7,
        SlotMember::Program => 0x1d27,
        SlotMember::Counter => 0x1d77,
        SlotMember::RoomArgument => 0x1dc7,
        SlotMember::Work0 => 0x1e17,
        SlotMember::InstructionDelay => 0xde1c,
        SlotMember::DrawProgram => 0xde6c,
        SlotMember::ReturnProgram => 0xdebc,
        SlotMember::Work1 => 0xdf0c,
    }
}
fn slot_address(member: SlotMember, index: isize) -> usize {
    slot_base(member)
        .checked_add_signed(index.checked_mul(2).expect("mechanism slot extent"))
        .expect("mechanism slot address")
}
impl ImportedAliases for Runtime<'_> {
    fn mechanism_word(&self, member: SlotMember, index: isize) -> u16 {
        self.word(slot_address(member, index))
    }
    fn mechanism_byte(&self, member: SlotMember, index: isize, lane: usize) -> u8 {
        self.byte(
            slot_address(member, index)
                .checked_add(lane)
                .expect("mechanism byte extent"),
        )
    }
    fn set_mechanism_word(
        &mut self,
        member: SlotMember,
        index: isize,
        value: u16,
        order: StoreOrder,
    ) {
        let at = slot_address(member, index);
        match order {
            StoreOrder::PrefixLittle => self.write_word_le(value, at),
            StoreOrder::SliceLittle => self.write_slice_le(at, value),
            StoreOrder::SliceNative => self.write_slice_native(at, value),
        }
    }
    fn set_mechanism_byte(&mut self, member: SlotMember, index: isize, lane: usize, value: u8) {
        self.set_byte(
            slot_address(member, index)
                .checked_add(lane)
                .expect("mechanism byte extent"),
            value,
        );
    }
    fn projectile_kind(&self, index: isize, order: ByteOrder) -> u16 {
        let at = 0xc18usize
            .checked_add_signed(index.checked_mul(2).expect("projectile kind extent"))
            .expect("projectile kind address");
        match order {
            ByteOrder::Little => self.word(at),
            ByteOrder::Native => self.native_word(at),
        }
    }
    fn terrain_word(&self, block: usize, order: ByteOrder) -> u16 {
        let at = 0x10002usize.checked_add(block).expect("terrain extent");
        match order {
            ByteOrder::Little => self.word(at),
            ByteOrder::Native => self.native_word(at),
        }
    }
    fn set_terrain_word(&mut self, block: usize, value: u16, order: ByteOrder) {
        let at = 0x10002usize.checked_add(block).expect("terrain extent");
        match order {
            ByteOrder::Little => self.write_slice_le(at, value),
            ByteOrder::Native => self.write_slice_native(at, value),
        }
    }
    fn terrain_high(&self, block: u16) -> u8 {
        self.byte(0x10003 + usize::from(block))
    }
    fn set_terrain_high(&mut self, block: u16, value: u8) {
        self.set_byte(0x10003 + usize::from(block), value);
    }
    fn behavior_byte(&self, index: usize) -> u8 {
        self.byte(0x16402usize.checked_add(index).expect("behavior extent"))
    }
    fn set_behavior_byte(&mut self, index: usize, value: u8) {
        self.set_byte(
            0x16402usize.checked_add(index).expect("behavior extent"),
            value,
        );
    }
    fn set_behavior_pair(&mut self, index: usize, value: u16) {
        self.write_slice_native(
            0x16402usize.checked_add(index).expect("behavior extent"),
            value,
        );
    }
    fn contact_displacement(&self, token: &ContactIdentity) -> i32 {
        i32::from_ne_bytes(core::array::from_fn(|lane| self.byte(token.0 + lane)))
    }
    fn set_contact_displacement(&mut self, token: &ContactIdentity, value: i32) {
        self.write_slice(token.0, &value.to_ne_bytes());
    }
    fn scroll_pair(&mut self, index: usize, value: u16) {
        self.write_slice_native(
            0xcd20usize.checked_add(index).expect("scroll extent"),
            value,
        );
    }
    fn set_scroll(&mut self, index: usize, value: u8) {
        self.set_byte(
            0xcd20usize.checked_add(index).expect("scroll extent"),
            value,
        );
    }
    fn signal_turret(&mut self, association: u16) {
        self.write_slice_native(usize::from(association), 1);
    }
    fn saved_graphics_word(&self, index: usize) -> u16 {
        self.word(0x1c2f + index * 2)
    }
    fn set_saved_graphics_word(&mut self, index: usize, value: u16, order: StoreOrder) {
        let at = 0x1c2f + index * 2;
        match order {
            StoreOrder::PrefixLittle => self.write_word_le(value, at),
            StoreOrder::SliceLittle => self.write_slice_le(at, value),
            StoreOrder::SliceNative => self.write_slice_native(at, value),
        }
    }
}
impl Host for Runtime<'_> {}
impl Effects for Runtime<'_> {
    fn sound(&mut self, channel: SoundChannel, maximum: u8, cue: u16) {
        self.foreign(|game| {
            use crate::game::sm_80::*;
            match (channel, maximum) {
                (SoundChannel::One, 1) => QueueSfx1_Max1(cue, game),
                (SoundChannel::One, 3) => QueueSfx1_Max3(cue, game),
                (SoundChannel::One, 6) => QueueSfx1_Max6(cue, game),
                (SoundChannel::One, 9) => QueueSfx1_Max9(cue, game),
                (SoundChannel::One, 15) => QueueSfx1_Max15(cue, game),
                (SoundChannel::Two, 1) => QueueSfx2_Max1(cue, game),
                (SoundChannel::Two, 3) => QueueSfx2_Max3(cue, game),
                (SoundChannel::Two, 6) => QueueSfx2_Max6(cue, game),
                (SoundChannel::Two, 9) => QueueSfx2_Max9(cue, game),
                (SoundChannel::Two, 15) => QueueSfx2_Max15(cue, game),
                (SoundChannel::Three, 1) => QueueSfx3_Max1(cue, game),
                (SoundChannel::Three, 3) => QueueSfx3_Max3(cue, game),
                (SoundChannel::Three, 6) => QueueSfx3_Max6(cue, game),
                (SoundChannel::Three, 9) => QueueSfx3_Max9(cue, game),
                (SoundChannel::Three, 15) => QueueSfx3_Max15(cue, game),
                _ => unreachable!("mechanism sound limit"),
            }
        });
    }
    fn music(&mut self, track: u16) {
        self.foreign(|game| crate::game::music::QueueMusic_Delayed8(track, game));
    }
    fn clear_music_queue(&mut self) {
        self.foreign(crate::game::music::clear_queue);
    }
    fn delayed_room_music(&mut self, frames: u16) {
        self.foreign(|game| crate::game::music::PlayRoomMusicTrackAfterAFrames(frames, game));
    }
    fn display_message(&mut self, request: u16) -> u8 {
        self.foreign(|game| unsafe {
            #[cfg(sm_verify_dll)]
            {
                crate::game::room_mechanism_checks::observe_message_display(request, game)
            }
            #[cfg(not(sm_verify_dll))]
            {
                crate::game::message_boxes::display(request, game)
            }
        })
    }
    fn poll_message(&mut self, request: u16) -> i32 {
        self.foreign(|game| crate::game::message_boxes::poll(request, game))
    }
    fn player_control(&mut self, operation: u16) {
        self.foreign(|game| unsafe {
            crate::game::sm_90::CallSomeSamusCode(operation, game);
        });
    }
    fn face_forward(&mut self) {
        self.foreign(|game| unsafe { crate::game::sm_91::MakeSamusFaceForward(game) });
    }
    fn suit_pickup(&mut self, pickup: SuitPickup) {
        self.owned_call(|runtime| match pickup {
            SuitPickup::Varia => runtime.begin_suit_pickup(false),
            SuitPickup::Gravity => runtime.begin_suit_pickup(true),
            SuitPickup::Unknown(_) => {
                runtime.foreign(|game| unsafe { crate::rtl::Unreachable(game) });
            }
        });
    }
    fn pickup_visual(&mut self, visual: VisualUpdate) {
        self.foreign(|game| unsafe {
            use crate::game::sm_80::*;
            match visual {
                VisualUpdate::None => {}
                VisualUpdate::Beams => crate::game::sm_90::UpdateBeamTilesAndPalette(game),
                VisualUpdate::Grapple => AddGrappleToHudTilemap(game),
                VisualUpdate::Xray => AddXrayToHudTilemap(game),
                VisualUpdate::Missiles => AddMissilesToHudTilemap(game),
                VisualUpdate::SuperMissiles => AddSuperMissilesToHudTilemap(game),
                VisualUpdate::PowerBombs => AddPowerBombsToHudTilemap(game),
            }
        });
    }
    fn save(&mut self, slot: u16) {
        self.foreign(|game| unsafe {
            crate::game::sm_81::SaveToSram(slot, game);
        });
    }
    fn enemy_projectile(&mut self, definition: u16, parameter: u16) {
        crate::game::enemy_projectiles::lifecycle::spawn_room(self, definition, parameter);
    }
    fn wake_projectile_for_block(&mut self, block: u16) {
        if let Some(slot) = self
            .session_mut()
            .enemy_projectiles
            .slots
            .iter_mut()
            .rev()
            .find(|slot| slot.behavior_a == block)
        {
            slot.delay = 1;
            slot.program = slot.program.wrapping_add(2);
            return;
        }
        // The legacy failed search uses byte index -2; retain its exceptional alias.
        let index = -2isize;
        let timer = arrays::eproj_instr_timers
            .checked_add_signed(index)
            .unwrap();
        let program = arrays::eproj_instr_list_ptr
            .checked_add_signed(index)
            .unwrap();
        self.write_slice_le(timer, 1);
        let value = self.word(program).wrapping_add(2);
        self.write_slice_le(program, value);
    }
    fn map_used(&self) -> bool {
        self.byte(0xd908 + usize::from(self.session().room.area)) != 0
    }
    fn mark_map_used(&mut self) {
        let area = usize::from(self.session().room.area);
        self.set_byte(0xd908 + area, 255);
    }
    fn set_has_area_map(&mut self, value: u16) {
        self.foreign(|game| game.set_has_area_map(value));
    }
    fn mark_save_station(&mut self, station: u16) {
        self.write_slice_le(ram::load_station_index, station);
        let station = self.word(ram::load_station_index);
        crate::game::progress_flags::prepare_flag_mask(self.session_mut(), station);
        let address = 0xd8f8 + usize::from(self.session().room.area) * 2;
        let value = self.byte(address) | self.session().flag_compatibility.bitmask as u8;
        self.set_byte(address, value);
    }
    fn selected_save_slot(&self) -> u16 {
        self.word(ram::selected_save_slot)
    }
    fn statue_signal(&mut self) {
        self.write_slice_native(0xf78 + offset_of!(EnemyData, parameter_1), 1);
    }
    fn statue_wait_actor(&mut self, actor: u16, waiting: bool) {
        let actor = 0xf78 + usize::from(actor);
        let properties = actor + offset_of!(EnemyData, properties);
        let old = self.native_word(properties);
        self.write_slice_native(properties, if waiting { old | 0x400 } else { old & !0x400 });
        if !waiting {
            let instruction = actor + offset_of!(EnemyData, current_instruction);
            let value = self.native_word(instruction).wrapping_add(2);
            self.write_slice_native(instruction, value);
            self.write_slice_native(actor + offset_of!(EnemyData, instruction_timer), 1);
        }
    }
    fn diagnostic(&mut self, diagnostic: Diagnostic) -> u8 {
        self.foreign(|game|match diagnostic {
            Diagnostic::MissingScroll=>loop {},
            Diagnostic::MissingDisplacement=>unsafe{
                crate::c_library::printf(c"ERROR: PlmSetup_QuicksandSurfaceB is broken!!\n".as_ptr());
                crate::rtl::Unreachable(game);0
            },
            Diagnostic::Unreachable=>unsafe{crate::rtl::Unreachable(game) as u8},
            Diagnostic::UnsupportedSetup=>{
                use crate::c_library::__assert_fail;use core::ffi::c_char;
                unsafe{crate::native_assertion! { '_c2rust_label; b"0\0", b"src/sm_84.c\0", 1370, b"uint8 UNUSED_sub_84B3E3(uint16)\0" };}0
            },
            Diagnostic::InvalidRamDraw(address)=>{
                use crate::c_library::__assert_fail;use core::ffi::c_char;
                unsafe{crate::native_assertion! { '_c2rust_label: address < 0x2000; b"addr < 0x2000\0", b"src/sm_84.c\0", 265, b"uint8 *RomPtr_84orRAM(uint16_t)\0" };}0
            },
        })
    }
}
