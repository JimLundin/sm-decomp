//! Mechanism commands commit owned state before synchronous effects.
#![forbid(unsafe_code)]
use super::rules::terrain_setup::{below, install, level, set_level};
use super::{
    access::{mechanism, mechanism_byte, set_behavior_byte},
    content::Script,
    creation, graphics,
    host::{
        ContentAddress, Diagnostic, Host, InstructionRecord, SoundChannel, StoreOrder, SuitPickup,
    },
    presentation, rules,
};
use crate::game::item_pickups::PickupKind;
use crate::game::progress_flags as flags;
fn collect<H: Host>(
    h: &mut H,
    p: Script<H::Record>,
    kind: PickupKind,
) -> Option<Script<H::Record>> {
    let amount = match kind {
        PickupKind::Beam | PickupKind::SuperMissiles => u16::from_le_bytes([p.byte(0), p.byte(1)]),
        _ => p.word(0),
    };
    let effects = h.session_mut().inventory.collect(kind, amount);
    h.pickup_visual(effects.visual);
    h.delayed_room_music(0x168);
    let message = effects.message.unwrap_or_else(|| u16::from(p.byte(2)));
    h.session_mut().messages.queued = message;
    Some(p.advance(if effects.message.is_none() { 3 } else { 2 }))
}
/// Execute one command against the shared session and its captured content.
pub(super) fn run<H: Host>(
    h: &mut H,
    ea: u32,
    p: Script<H::Record>,
    k: u16,
) -> Option<Script<H::Record>> {
    let slot = usize::from(k >> 1);
    match ea {
        8685236 => {
            mechanism!(h, slot as isize, program = (p.position()).wrapping_sub(2));
            None
        }
        8685249 => {
            mechanism!(h, slot as isize, pre_action = p.word(0));
            Some(p.advance(2))
        }
        8685323 => {
            let identity = p.load24(0);
            h.suit_pickup(match identity {
                9557220 => SuitPickup::Varia,
                9557434 => SuitPickup::Gravity,
                _ => SuitPickup::Unknown(identity),
            });
            Some(p.advance(3))
        }
        8685390 => {
            mechanism_byte!(h, slot as isize, counter, usize::from(k & 1) => p.byte(0));
            Some(p.advance(1))
        }
        8686194 => {
            let argument = mechanism!(h, slot as isize, room_argument);
            if argument & 0x8000 != 0 {
                return Some(p.advance(2));
            }
            if flags::door_opened(h, argument) {
                p.jump_word(0)
            } else {
                Some(p.advance(2))
            }
        }
        8686913 => {
            creation::coordinates(h, slot);
            let x = (h.session().player.x >> 4).wrapping_sub(h.session().mechanisms.block_x) as i16;
            let y = (h.session().player.y >> 4).wrapping_sub(h.session().mechanisms.block_y) as i16;
            if x.wrapping_abs() as u8 <= p.byte(0) && y.wrapping_abs() as u8 <= p.byte(1) {
                p.jump_word(2)
            } else {
                Some(p.advance(4))
            }
        }
        8694742 => {
            mechanism!(
                h,
                slot as isize,
                block = mechanism!(h, slot as isize, block).wrapping_add(2)
            );
            Some(p)
        }
        8695487 | 8695349 => {
            let full = if ea == 8695487 {
                h.session().inventory.ammunition[0].maximum
                    == h.session().inventory.ammunition[0].current
            } else {
                h.session().inventory.health.maximum == h.session().inventory.health.current
            };
            if !full {
                return Some(p.advance(2));
            }
            h.player_control(1);
            p.jump_word(0)
        }
        8699455 => {
            let index = mechanism!(h, slot as isize, work0) >> 1;
            // The native ROM allocation must remain valid for this captured table read.
            let sequence = h.native_record(ContentAddress {
                bank: 0x84,
                offset: 0xbe4b,
            });
            let value = sequence.word(isize::try_from(index).unwrap() * 2);
            mechanism!(h, slot as isize, pre_action = value);
            Some(p)
        }
        8703379 => {
            let block = mechanism!(h, slot as isize, block);
            set_behavior_byte(h, usize::from(block >> 1), 1);
            Some(p)
        }
        8705951 => {
            h.enemy_projectile(0xe517, 0x30a);
            h.enemy_projectile(0xe517, 0x30a);
            Some(p)
        }
        8706958 | 8707126 | 8707000 | 8707063 | 8707168 | 8707231 => {
            damage_turret(h, ea, slot);
            Some(p)
        }
        8685244 => {
            mechanism!(h, slot as isize, header = 0);
            None
        }
        8686138 => p.jump(mechanism!(h, slot as isize, return_program)),
        8685258 => {
            mechanism!(h, slot as isize, pre_action = 0x86d0);
            Some(p)
        }
        8685375 => {
            let old = mechanism!(h, slot as isize, counter);
            // The old unaligned update decodes and writes little endian.
            mechanism!(
                h,
                slot as isize,
                counter = old.wrapping_sub(1),
                StoreOrder::SliceLittle
            );
            if old == 1 {
                Some(p.advance(2))
            } else {
                p.jump_word(0)
            }
        }
        8686075 => collect(h, p, PickupKind::PowerBombs),
        8685744 => collect(h, p, PickupKind::Beam),
        8685811 => collect(h, p, PickupKind::Equipment),
        8685850 => collect(h, p, PickupKind::Grapple),
        8685889 => collect(h, p, PickupKind::Xray),
        8685928 => collect(h, p, PickupKind::Energy),
        8685958 => collect(h, p, PickupKind::Reserve),
        8685993 => collect(h, p, PickupKind::Missiles),
        8686034 => collect(h, p, PickupKind::SuperMissiles),
        8686557 => {
            h.clear_music_queue();
            h.music(u16::from(p.byte(0)));
            Some(p.advance(1))
        }
        8698479 => {
            if h.session().inventory.equipment.owned & 0x1000 != 0 {
                Some(p.advance(2))
            } else {
                p.jump_word(0)
            }
        }
        8685412 => Some(graphics::load_graphics(h, p, k)),
        8685541 => Some(graphics::copy_vram(h, p)),
        8686735 => {
            h.mark_map_used();
            h.session_mut().messages.queued = 0x14;
            h.set_has_area_map(1);
            Some(p)
        }
        8685582 => {
            if flags::boss_defeated(h, u16::from(p.byte(0))) {
                p.jump_word(1)
            } else {
                Some(p.advance(3))
            }
        }
        8685613 => {
            if flags::event_happened(h, p.word(0)) {
                p.jump_word(2)
            } else {
                Some(p.advance(4))
            }
        }
        8685630 => {
            flags::set_event(h, p.word(0));
            Some(p.advance(2))
        }
        8685640 | 8685692 => {
            let argument = mechanism!(h, slot as isize, room_argument);
            let found = if ea == 8685640 {
                flags::room_object_used(h, argument)
            } else {
                flags::item_collected(h, argument)
            };
            if argument & 0x8000 == 0 && found {
                p.jump_word(0)
            } else {
                Some(p.advance(2))
            }
        }
        8685669 | 8685721 => {
            let argument = mechanism!(h, slot as isize, room_argument);
            if argument & 0x8000 == 0 {
                if ea == 8685669 {
                    flags::use_room_object(h, argument);
                } else {
                    flags::collect_item(h, argument);
                }
            }
            Some(p)
        }
        8686116 => {
            mechanism!(h, slot as isize, return_program = p.word(0));
            Some(p.advance(2))
        }
        8686126 => {
            mechanism!(
                h,
                slot as isize,
                return_program = (p.position()).wrapping_add(2)
            );
            p.jump_word(0)
        }
        8686225 => {
            let hits = mechanism!(h, slot as isize, work1).wrapping_add(1);
            mechanism!(h, slot as isize, work1 = hits);
            if (hits as u8) < p.byte(0) {
                return Some(p.advance(3));
            }
            let argument = mechanism!(h, slot as isize, room_argument);
            if argument & 0x8000 == 0 {
                let flags = flags::open_door_and_read_adjacent_flags(h, argument);
                mechanism!(h, slot as isize, room_argument = flags | 0x8000);
            }
            mechanism!(h, slot as isize, pre_action = 0x8aa6);
            p.jump_word(1)
        }
        8686285 => {
            let argument_byte =
                mechanism_byte!(h, slot as isize, room_argument, usize::from(k & 1));
            let value = argument_byte.wrapping_add(1);
            if value >= p.byte(0) {
                mechanism!(h, slot as isize, room_argument = 0xffff);
                mechanism!(h, slot as isize, pre_action = 0x8ae0);
                p.jump_word(1)
            } else {
                mechanism!(h, slot as isize, room_argument = u16::from(value));
                Some(p.advance(3))
            }
        }
        8686321 => {
            let bts = p.byte(0);
            let block = mechanism!(h, slot as isize, block);
            set_behavior_byte(h, usize::from(block >> 1), bts);
            Some(p.advance(1))
        }
        8686341 | 8686359 => {
            let block = mechanism!(h, slot as isize, work0);
            let target = mechanism!(h, slot as isize, block);
            set_level(h, target, block);
            h.session_mut().mechanisms.custom_draw[1] = block;
            h.session_mut().mechanisms.custom_draw[0] = 1;
            h.session_mut().mechanisms.custom_draw[2] = 0;
            mechanism!(h, slot as isize, instruction_delay = 1);
            mechanism!(h, slot as isize, draw_program = 0x1e67);
            mechanism!(h, slot as isize, program = p.position());
            presentation::process_draw(h, k);
            let selected = h.session().mechanisms.selected;
            let coordinate_slot = usize::from(h.session().mechanisms.selected >> 1);
            creation::coordinates(h, coordinate_slot);
            presentation::draw(h, selected);
            None
        }
        8686421 | 8686483 => {
            mechanism!(h, slot as isize, work0 = 0);
            let address = mechanism!(h, slot as isize, room_argument);
            // This native sequence remains captured for the original pointer loop.
            let entries = h.native_record(ContentAddress {
                bank: 0x8f,
                offset: address,
            });
            let mut offset = 0;
            while entries.byte(offset) & 0x80 == 0 {
                let value = entries.byte(offset + 1);
                let index = entries.byte(offset);
                h.set_scroll(usize::from(index), value);
                offset += 2;
            }
            let block = mechanism!(h, slot as isize, block);
            let value = level(h, block) & 0xfff | if ea == 8686421 { 0x3000 } else { 0xb000 };
            set_level(h, block, value);
            Some(p)
        }
        8686545 => {
            h.music(u16::from(p.byte(0)));
            Some(p.advance(1))
        }
        8686599 | 8686608 | 8686617 | 8686626 | 8686635 | 8686644 | 8686653 | 8686662 | 8686671
        | 8686680 | 8686689 | 8686698 | 8686707 | 8686716 | 8686725 => {
            let index = (ea - 8686599) / 9;
            let channel = match index % 3 {
                0 => SoundChannel::One,
                1 => SoundChannel::Two,
                _ => SoundChannel::Three,
            };
            h.sound(
                channel,
                [6, 15, 3, 9, 1][index as usize / 3],
                u16::from(p.byte(0)),
            );
            Some(p.advance(1))
        }
        8686767 | 8686800 => {
            let health = ea == 8686767;
            let supply = if health {
                &h.session().inventory.health
            } else {
                &h.session().inventory.ammunition[0]
            };
            if supply.maximum != supply.current {
                h.session_mut().messages.queued = if health { 0x15 } else { 0x16 };
                let supply = if health {
                    &mut h.session_mut().inventory.health
                } else {
                    &mut h.session_mut().inventory.ammunition[0]
                };
                supply.current = supply.maximum;
            }
            h.player_control(1);
            Some(p)
        }
        8686833 => {
            let result = h.poll_message(23);
            if result < 0 {
                return Some(p.advance(-2));
            }
            if result == 2 {
                return p.jump_word(0);
            }
            h.enemy_projectile(
                crate::game::constants::bank_86::addr_kEproj_SaveStationElectricity as u16,
                0,
            );
            let selected = usize::from(h.session().mechanisms.selected >> 1);
            let station = mechanism!(h, selected as isize, room_argument) & 7;
            h.mark_save_station(station);
            let save = h.selected_save_slot();
            h.save(save);
            Some(p.advance(2))
        }
        8695822 => {
            h.session_mut().player.x = h.session().player.x.wrapping_add(8) & 0xfff0;
            h.face_forward();
            Some(p)
        }
        8695844 => {
            h.session_mut().messages.queued = 0x18;
            Some(p)
        }
        8695856 => {
            h.player_control(1);
            h.session_mut().mechanisms.save_locked = 1;
            Some(p)
        }
        8698845 => {
            mechanism!(h, slot as isize, counter = 0);
            Some(p)
        }
        8698849 => {
            h.enemy_projectile(p.word(0), 0);
            Some(p.advance(2))
        }
        8698864 => {
            let block = mechanism!(h, slot as isize, block);
            h.wake_projectile_for_block(block);
            Some(p.advance(2))
        }
        8704983 | 8705012 => {
            install(h, 0x1608, if ea == 8704983 { 0x1012 } else { 0xa000 });
            install(h, 0x160a, if ea == 8704983 { 0x1013 } else { 0xa000 });
            Some(p)
        }
        8705914 => {
            h.enemy_projectile(
                crate::game::constants::bank_86::addr_kEproj_EyeDoorProjectile as u16,
                p.word(0),
            );
            h.sound(SoundChannel::Two, 6, 0x4c);
            Some(p.advance(2))
        }
        8705936 => {
            h.enemy_projectile(
                crate::game::constants::bank_86::addr_kEproj_EyeDoorSweat as u16,
                p.word(0),
            );
            Some(p.advance(2))
        }
        8705974 => {
            h.enemy_projectile(
                crate::game::constants::bank_86::addr_kEproj_EyeDoorSmoke as u16,
                0xb,
            );
            Some(p)
        }
        8708223 | 8708175 | 8708199 => {
            if ea != 8708223 {
                let index = mechanism!(h, slot as isize, work1) >> 1;
                let table = h.native_record(ContentAddress {
                    bank: 0x84,
                    offset: if ea == 8708175 { 0xe05f } else { 0xe077 },
                });
                let draw = table.word(isize::try_from(index).unwrap() * 2);
                mechanism!(h, slot as isize, draw_program = draw);
            }
            mechanism!(h, slot as isize, instruction_delay = 4);
            mechanism!(h, slot as isize, program = p.position());
            presentation::process_draw(h, k);
            let selected = h.session().mechanisms.selected;
            let coordinate_slot = usize::from(h.session().mechanisms.selected >> 1);
            creation::coordinates(h, coordinate_slot);
            presentation::draw(h, selected);
            None
        }
        8708765 => {
            h.session_mut().projectiles.controls.charge = 0;
            Some(p)
        }
        8709691 => {
            h.session_mut().room.liquid_velocity = (-32i16) as u16;
            Some(p)
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
            rules::noob_tube::command(h, kind);
            Some(p)
        }
        8704855 => {
            h.enemy_projectile(0xa993, p.word(0));
            Some(p.advance(2))
        }
        8704967 => {
            h.music(6);
            Some(p)
        }
        8704341 => {
            rules::hands::lower_acid(h);
            Some(p)
        }
        8695107 | 8695128 => {
            rules::treadmill::activate(
                h,
                slot,
                if ea == 8695107 {
                    rules::treadmill::Direction::Right
                } else {
                    rules::treadmill::Direction::Left
                },
            );
            Some(p)
        }
        8694609 => {
            rules::crumbling_wall::open_passage(h);
            Some(p)
        }
        8694528 | 8694617 => {
            rules::crumbling_wall::move_down(h, slot);
            Some(p)
        }
        8704761 | 8704779 => rules::mother_brain_glass::command(
            h,
            p,
            slot,
            if ea == 8704761 {
                rules::mother_brain_glass::Command::Threshold
            } else {
                rules::mother_brain_glass::Command::Shatter
            },
        ),
        8698297 => {
            rules::animal_rescue::mark_rescued(h);
            Some(p)
        }
        8698661 => {
            rules::animal_rescue::move_passage(h, slot);
            Some(p)
        }
        8685348 => p.jump_word(0),
        8694941 => {
            rules::plants::hurt(h);
            Some(p)
        }
        8694961 => {
            rules::plants::protect(h);
            Some(p)
        }
        8705987 | 8706010 => {
            rules::eye_door::convert(
                h,
                slot,
                if ea == 8705987 {
                    rules::eye_door::Side::Right
                } else {
                    rules::eye_door::Side::Left
                },
            );
            Some(p)
        }
        _ => {
            h.diagnostic(Diagnostic::Unreachable);
            None
        }
    }
}
fn damage_turret<H: Host>(h: &mut H, ea: u32, slot: usize) {
    let target = mechanism!(h, slot as isize, work0);
    h.signal_turret(target);
    let block = mechanism!(h, slot as isize, block);
    install(h, block, 0xa003);
    if ea == 8706958 || ea == 8707126 {
        let below = below(h, block);
        install(h, below, 0xa003);
        return;
    }
    let right = ea == 8707000 || ea == 8707063;
    let down = ea == 8707000 || ea == 8707168;
    let step = if right { 2 } else { 0xfffe };
    install(h, block.wrapping_add(step), if down { 0xa003 } else { 0 });
    let origin = if ea == 8707000 {
        mechanism!(h, (h.session().mechanisms.selected >> 1) as isize, block)
    } else {
        block
    };
    let below = below(h, origin);
    install(h, below, 0xa003);
    install(h, below.wrapping_add(step), if down { 0 } else { 0xa003 });
}
