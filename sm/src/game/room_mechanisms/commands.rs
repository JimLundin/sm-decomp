//! Mechanism instruction execution. Native content validity remains explicit.
use super::{
    fields::{Fields, Global, SoundChannel},
    port::Execution,
    rules,
    state::Column,
};
use crate::game::enemy_script::{EnemyScript as Script, bank_base};
use crate::game::item_pickups::PickupKind;
use crate::ram_map::words as ram;
impl Execution<'_> {
    fn bit_index(&mut self, value: u16) -> usize {
        self.foreign(|g| usize::from(crate::game::progress_flags::PrepareBitAccess(value, g)))
    }
    fn queue_message(&mut self, value: u16) {
        self.foreign(|g| crate::game::message_boxes::queue(value, g));
    }
    fn collect(&mut self, p: Script, kind: PickupKind) -> Script {
        let amount = match kind {
            PickupKind::Beam | PickupKind::SuperMissiles => {
                u16::from_le_bytes([p.byte(0), p.byte(1)])
            }
            _ => p.word(0),
        };
        let effects = self.foreign(|g| crate::game::item_pickups_ram::collect(g, kind, amount));
        self.foreign(|g| super::effects::pickup_visual(effects.visual, g));
        self.foreign(|g| crate::game::music::PlayRoomMusicTrackAfterAFrames(0x168, g));
        let message = effects.message.unwrap_or_else(|| u16::from(p.byte(2)));
        self.queue_message(message);
        p.advance(if effects.message.is_none() { 3 } else { 2 })
    }
    /// Native ROM and external player routines require initialized runtime state.
    pub(super) unsafe fn command(&mut self, ea: u32, p: Script, k: u16) -> Script {
        let slot = usize::from(k >> 1);
        match ea {
            8685236 => {
                self.set_slot(
                    Column::Program,
                    slot,
                    (p.offset_from(bank_base(0x84)) as u16).wrapping_sub(2),
                );
                Script::Stop
            }
            8685249 => {
                self.set_slot(Column::PreAction, slot, p.word(0));
                p.advance(2)
            }
            8685323 => self.call_function(p, super::effects::instruction_function),
            8685390 => {
                self.port_mut().put_byte(0x1d77 + usize::from(k), p.byte(0));
                p.advance(1)
            }
            8686194 => {
                let argument = self.slot(Column::RoomArgument, slot);
                if argument & 0x8000 != 0 {
                    return p.advance(2);
                }
                let index = self.bit_index(argument);
                if self.port().word(0x5e7) & u16::from(self.port().byte(0xd8b0 + index)) != 0 {
                    Script::jump(p.word(0))
                } else {
                    p.advance(2)
                }
            }
            8686913 => {
                self.coordinates(slot);
                let x = (self.global(Global::PlayerX) >> 4)
                    .wrapping_sub(self.global(Global::BlockX)) as i16;
                let y = (self.global(Global::PlayerY) >> 4)
                    .wrapping_sub(self.global(Global::BlockY)) as i16;
                if x.wrapping_abs() as u8 <= p.byte(0) && y.wrapping_abs() as u8 <= p.byte(1) {
                    Script::jump(p.word(2))
                } else {
                    p.advance(4)
                }
            }
            8694742 => {
                self.set_slot(
                    Column::Block,
                    slot,
                    self.slot(Column::Block, slot).wrapping_add(2),
                );
                p
            }
            8695487 | 8695349 => {
                let full = if ea == 8695487 {
                    self.global(Global::MaxMissiles) == self.global(Global::Missiles)
                } else {
                    self.global(Global::MaxHealth) == self.global(Global::Health)
                };
                if !full {
                    return p.advance(2);
                }
                self.player_control(1);
                Script::jump(p.word(0))
            }
            8699455 => {
                let index = self.slot(Column::Work0, slot) >> 1;
                // The native ROM allocation must remain valid for this captured table read.
                let sequence = unsafe { super::native_content::sequence(self, 0x84, 0xbe4b) };
                let value = unsafe { sequence.word(isize::try_from(index).unwrap() * 2) };
                self.set_slot(Column::PreAction, slot, value);
                p
            }
            8703379 => {
                self.set_behavior(self.slot(Column::Block, slot), 1);
                p
            }
            8705951 => {
                self.eproj(0xe517, 0x30a);
                self.eproj(0xe517, 0x30a);
                p
            }
            8706958 | 8707126 | 8707000 | 8707063 | 8707168 | 8707231 => {
                self.damage_turret(ea, slot);
                p
            }
            8685244 => {
                self.release(slot);
                Script::Stop
            }
            8686138 => Script::jump(self.slot(Column::ReturnProgram, slot)),
            8685258 => {
                self.set_slot(Column::PreAction, slot, 0x86d0);
                p
            }
            8685375 => {
                let old = self.slot(Column::Counter, slot);
                // The old unaligned update decodes and writes little endian.
                self.port_mut()
                    .put_word(0x1d77 + slot * 2, old.wrapping_sub(1));
                if old == 1 {
                    p.advance(2)
                } else {
                    Script::jump(p.word(0))
                }
            }
            8686075 => self.collect(p, PickupKind::PowerBombs),
            8685744 => self.collect(p, PickupKind::Beam),
            8685811 => self.collect(p, PickupKind::Equipment),
            8685850 => self.collect(p, PickupKind::Grapple),
            8685889 => self.collect(p, PickupKind::Xray),
            8685928 => self.collect(p, PickupKind::Energy),
            8685958 => self.collect(p, PickupKind::Reserve),
            8685993 => self.collect(p, PickupKind::Missiles),
            8686034 => self.collect(p, PickupKind::SuperMissiles),
            8686557 => {
                self.foreign(crate::game::music::clear_queue);
                self.music(u16::from(p.byte(0)));
                p.advance(1)
            }
            8698479 => {
                if self.global(Global::CollectedItems) & 0x1000 != 0 {
                    p.advance(2)
                } else {
                    Script::jump(p.word(0))
                }
            }
            8685412 => self.load_graphics(p, k),
            8685541 => self.copy_vram(p),
            8686735 => {
                let area = usize::from(self.global(Global::Area));
                self.port_mut().put_byte(0xd908 + area, 255);
                self.queue_message(0x14);
                self.foreign(|g| g.set_has_area_map(1));
                p
            }
            8685582 => {
                if self.boss(u16::from(p.byte(0))) != 0 {
                    Script::jump(p.word(1))
                } else {
                    p.advance(3)
                }
            }
            8685613 => {
                if self.event(p.word(0)) {
                    Script::jump(p.word(2))
                } else {
                    p.advance(4)
                }
            }
            8685630 => {
                self.mark_event(p.word(0));
                p.advance(2)
            }
            8685640 | 8685692 => {
                let argument = self.slot(Column::RoomArgument, slot);
                let index = self.bit_index(argument);
                let base = if ea == 8685640 { 0xd830 } else { 0xd870 };
                if argument & 0x8000 == 0
                    && self.port().word(0x5e7) & u16::from(self.port().byte(base + index)) != 0
                {
                    Script::jump(p.word(0))
                } else {
                    p.advance(2)
                }
            }
            8685669 | 8685721 => {
                let argument = self.slot(Column::RoomArgument, slot);
                if argument & 0x8000 == 0 {
                    let index = self.bit_index(argument);
                    let base = if ea == 8685669 { 0xd830 } else { 0xd870 };
                    let value = self.port().byte(base + index) | self.port().word(0x5e7) as u8;
                    self.port_mut().put_byte(base + index, value);
                }
                p
            }
            8686116 => {
                self.set_slot(Column::ReturnProgram, slot, p.word(0));
                p.advance(2)
            }
            8686126 => {
                self.set_slot(
                    Column::ReturnProgram,
                    slot,
                    (p.offset_from(bank_base(0x84)) as u16).wrapping_add(2),
                );
                Script::jump(p.word(0))
            }
            8686225 => {
                let hits = self.slot(Column::Work1, slot).wrapping_add(1);
                self.set_slot(Column::Work1, slot, hits);
                if (hits as u8) < p.byte(0) {
                    return p.advance(3);
                }
                let argument = self.slot(Column::RoomArgument, slot);
                if argument & 0x8000 == 0 {
                    let index = self.bit_index(argument);
                    let address = 0xd8b0 + index;
                    let value = self.port().byte(address) | self.port().word(0x5e7) as u8;
                    self.port_mut().put_byte(address, value);
                    self.set_slot(
                        Column::RoomArgument,
                        slot,
                        self.port().word(address) | 0x8000,
                    );
                }
                self.set_slot(Column::PreAction, slot, 0x8aa6);
                Script::jump(p.word(1))
            }
            8686285 => {
                let value = self.port().byte(0x1dc7 + usize::from(k)).wrapping_add(1);
                if value >= p.byte(0) {
                    self.set_slot(Column::RoomArgument, slot, 0xffff);
                    self.set_slot(Column::PreAction, slot, 0x8ae0);
                    Script::jump(p.word(1))
                } else {
                    self.set_slot(Column::RoomArgument, slot, u16::from(value));
                    p.advance(3)
                }
            }
            8686321 => {
                let bts = p.byte(0);
                self.set_behavior(self.slot(Column::Block, slot), bts);
                p.advance(1)
            }
            8686341 | 8686359 => {
                let block = self.slot(Column::Work0, slot);
                self.set_level(self.slot(Column::Block, slot), block);
                self.port_mut().put_word(0x1e69, block);
                self.port_mut().put_word(0x1e67, 1);
                self.port_mut().put_word(0x1e6b, 0);
                self.set_slot(Column::InstructionDelay, slot, 1);
                self.set_slot(Column::DrawProgram, slot, 0x1e67);
                self.set_slot(Column::Program, slot, p.offset_from(bank_base(0x84)) as u16);
                self.process_draw(k);
                let selected = self.global(Global::Selected);
                self.coordinates(usize::from(self.global(Global::Selected) >> 1));
                unsafe { self.draw(selected) };
                Script::Stop
            }
            8686421 | 8686483 => {
                self.set_slot(Column::Work0, slot, 0);
                let address = self.slot(Column::RoomArgument, slot);
                // This native sequence remains captured for the original pointer loop.
                let entries = unsafe { super::native_content::sequence(self, 0x8f, address) };
                let mut offset = 0;
                while unsafe { entries.byte(offset) } & 0x80 == 0 {
                    let value = unsafe { entries.byte(offset + 1) };
                    let index = unsafe { entries.byte(offset) };
                    self.port_mut().put_byte(0xcd20 + usize::from(index), value);
                    offset += 2;
                }
                let block = self.slot(Column::Block, slot);
                self.set_level(
                    block,
                    self.level(block) & 0xfff | if ea == 8686421 { 0x3000 } else { 0xb000 },
                );
                p
            }
            8686545 => {
                self.music(u16::from(p.byte(0)));
                p.advance(1)
            }
            8686599 | 8686608 | 8686617 | 8686626 | 8686635 | 8686644 | 8686653 | 8686662
            | 8686671 | 8686680 | 8686689 | 8686698 | 8686707 | 8686716 | 8686725 => {
                let index = (ea - 8686599) / 9;
                let channel = match index % 3 {
                    0 => SoundChannel::One,
                    1 => SoundChannel::Two,
                    _ => SoundChannel::Three,
                };
                self.sound(
                    channel,
                    [6, 15, 3, 9, 1][index as usize / 3],
                    u16::from(p.byte(0)),
                );
                p.advance(1)
            }
            8686767 | 8686800 => {
                let (maximum, current, message) = if ea == 8686767 {
                    (Global::MaxHealth, Global::Health, 0x15)
                } else {
                    (Global::MaxMissiles, Global::Missiles, 0x16)
                };
                if self.global(maximum) != self.global(current) {
                    self.queue_message(message);
                    self.set_global(current, self.global(maximum));
                }
                self.player_control(1);
                p
            }
            8686833 => {
                let result = self.foreign(|g| crate::game::message_boxes::poll(23, g));
                if result < 0 {
                    return p.advance(-2);
                }
                if result == 2 {
                    return Script::jump(p.word(0));
                }
                self.eproj(
                    crate::game::constants::bank_86::addr_kEproj_SaveStationElectricity as u16,
                    0,
                );
                let selected = usize::from(self.global(Global::Selected) >> 1);
                let station = self.port().native_word(0x1dc7 + selected * 2) & 7;
                self.port_mut().put_word(ram::load_station_index, station);
                self.bit_index(self.port().word(ram::load_station_index));
                let address = 0xd8f8 + usize::from(self.global(Global::Area)) * 2;
                let value = self.port().byte(address) | self.port().word(0x5e7) as u8;
                self.port_mut().put_byte(address, value);
                let save = self.port().word(ram::selected_save_slot);
                self.foreign(|g| super::effects::save(save, g));
                p.advance(2)
            }
            8695822 => self.place_on_station(p, super::effects::face_forward),
            8695844 => {
                self.queue_message(0x18);
                p
            }
            8695856 => {
                self.player_control(1);
                self.set_global(Global::SaveLocked, 1);
                p
            }
            8698845 => {
                self.set_slot(Column::Counter, slot, 0);
                p
            }
            8698849 => {
                self.eproj(p.word(0), 0);
                p.advance(2)
            }
            8698864 => {
                let block = self.slot(Column::Block, slot);
                let mut index = 34i32;
                while index >= 0 {
                    if block
                        == self
                            .port()
                            .word(crate::ram_map::word_arrays::eproj_E + index as usize)
                    {
                        break;
                    }
                    index -= 2;
                }
                let timer = (crate::ram_map::word_arrays::eproj_instr_timers as isize
                    + index as isize) as usize;
                let program = (crate::ram_map::word_arrays::eproj_instr_list_ptr as isize
                    + index as isize) as usize;
                self.port_mut().put_word(timer, 1);
                let value = self.port().word(program).wrapping_add(2);
                self.port_mut().put_word(program, value);
                p.advance(2)
            }
            8704983 | 8705012 => {
                self.install(0x1608, if ea == 8704983 { 0x1012 } else { 0xa000 });
                self.install(0x160a, if ea == 8704983 { 0x1013 } else { 0xa000 });
                p
            }
            8705914 => {
                self.eproj(
                    crate::game::constants::bank_86::addr_kEproj_EyeDoorProjectile as u16,
                    p.word(0),
                );
                self.sound(SoundChannel::Two, 6, 0x4c);
                p.advance(2)
            }
            8705936 => {
                self.eproj(
                    crate::game::constants::bank_86::addr_kEproj_EyeDoorSweat as u16,
                    p.word(0),
                );
                p.advance(2)
            }
            8705974 => {
                self.eproj(
                    crate::game::constants::bank_86::addr_kEproj_EyeDoorSmoke as u16,
                    0xb,
                );
                p
            }
            8708223 | 8708175 | 8708199 => {
                if ea != 8708223 {
                    let index = self.slot(Column::Work1, slot) >> 1;
                    let table = unsafe {
                        super::native_content::sequence(
                            self,
                            0x84,
                            if ea == 8708175 { 0xe05f } else { 0xe077 },
                        )
                    };
                    let draw = unsafe { table.word(isize::try_from(index).unwrap() * 2) };
                    self.set_slot(Column::DrawProgram, slot, draw);
                }
                self.set_slot(Column::InstructionDelay, slot, 4);
                self.set_slot(Column::Program, slot, p.offset_from(bank_base(0x84)) as u16);
                self.process_draw(k);
                let selected = self.global(Global::Selected);
                self.coordinates(usize::from(self.global(Global::Selected) >> 1));
                unsafe { self.draw(selected) };
                Script::Stop
            }
            8708765 => {
                self.port_mut().put_word(ram::flare_counter, 0);
                p
            }
            8709691 => {
                self.set_global(Global::LiquidVelocity, (-32i16) as u16);
                p
            }
            8705317 | 8705324 | 8705334 | 8705347 | 8705510 | 8705518 => {
                use rules::noob_tube::Command::*;
                let kind = match ea {
                    8705317 => Water,
                    8705324 => Crack,
                    8705334 => Quake,
                    8705347 => Shatter,
                    8705510 => Lock,
                    _ => Unlock,
                };
                rules::noob_tube::command(self, kind);
                p
            }
            8704855 => {
                self.eproj(0xa993, p.word(0));
                p.advance(2)
            }
            8704967 => {
                self.music(6);
                p
            }
            8704341 => {
                rules::hands::lower_acid(self);
                p
            }
            8695107 | 8695128 => {
                rules::treadmill::activate(
                    self,
                    slot,
                    if ea == 8695107 {
                        rules::treadmill::Direction::Right
                    } else {
                        rules::treadmill::Direction::Left
                    },
                );
                p
            }
            8694609 => {
                rules::crumbling_wall::open_passage(self);
                p
            }
            8694528 | 8694617 => {
                rules::crumbling_wall::move_down(self, slot);
                p
            }
            8704761 | 8704779 => rules::mother_brain_glass::command(
                self,
                p,
                slot,
                if ea == 8704761 {
                    rules::mother_brain_glass::Command::Threshold
                } else {
                    rules::mother_brain_glass::Command::Shatter
                },
            ),
            8698297 => {
                rules::animal_rescue::mark_rescued(self);
                p
            }
            8698661 => {
                rules::animal_rescue::move_passage(self, slot);
                p
            }
            8685348 => Script::jump(p.word(0)),
            8694941 => {
                rules::plants::hurt(self);
                p
            }
            8694961 => {
                rules::plants::protect(self);
                p
            }
            8705987 | 8706010 => {
                rules::eye_door::convert(
                    self,
                    slot,
                    if ea == 8705987 {
                        rules::eye_door::Side::Right
                    } else {
                        rules::eye_door::Side::Left
                    },
                );
                p
            }
            _ => {
                self.foreign(super::effects::unreachable);
                Script::Stop
            }
        }
    }
    pub(super) fn call_function(
        &mut self,
        p: Script,
        call: fn(u32, &mut crate::game_state::Game),
    ) -> Script {
        self.foreign(|game| call(p.load24(0), game));
        p.advance(3)
    }
    pub(super) fn place_on_station(
        &mut self,
        p: Script,
        call: fn(&mut crate::game_state::Game),
    ) -> Script {
        self.set_global(
            Global::PlayerX,
            self.global(Global::PlayerX).wrapping_add(8) & 0xfff0,
        );
        self.foreign(call);
        p
    }
    fn damage_turret(&mut self, ea: u32, slot: usize) {
        let target = usize::from(self.slot(Column::Work0, slot));
        self.port_mut().put_native_word(target, 1);
        let block = self.slot(Column::Block, slot);
        self.install(block, 0xa003);
        if ea == 8706958 || ea == 8707126 {
            let below = self.below(block);
            self.install(below, 0xa003);
            return;
        }
        let right = ea == 8707000 || ea == 8707063;
        let down = ea == 8707000 || ea == 8707168;
        let step = if right { 2 } else { 0xfffe };
        self.install(block.wrapping_add(step), if down { 0xa003 } else { 0 });
        let origin = if ea == 8707000 {
            self.port()
                .native_word(0x1c87 + usize::from(self.global(Global::Selected) >> 1) * 2)
        } else {
            block
        };
        let below = self.below(origin);
        self.install(below, 0xa003);
        self.install(below.wrapping_add(step), if down { 0 } else { 0xa003 });
    }
}
