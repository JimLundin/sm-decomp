//! Mechanism commands commit owned state before synchronous effects.
#![forbid(unsafe_code)]
use super::addresses;
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
use crate::item_pickups::PickupKind;
use crate::progress_flags as flags;
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
pub fn run<H: Host>(h: &mut H, ea: u32, p: Script<H::Record>, k: u16) -> Option<Script<H::Record>> {
    let slot = usize::from(k >> 1);
    match ea {
        addresses::instructions::SLEEP => {
            mechanism!(h, slot as isize, program = (p.position()).wrapping_sub(2));
            None
        }
        addresses::instructions::SET_PRE_INSTRUCTION => {
            mechanism!(h, slot as isize, pre_action = p.word(0));
            Some(p.advance(2))
        }
        addresses::instructions::CALL_FUNCTION => {
            let identity = p.load24(0);
            h.suit_pickup(match identity {
                9557220 => SuitPickup::Varia,
                9557434 => SuitPickup::Gravity,
                _ => SuitPickup::Unknown(identity),
            });
            Some(p.advance(3))
        }
        addresses::instructions::SET_TIMER => {
            mechanism_byte!(h, slot as isize, counter, usize::from(k & 1) => p.byte(0));
            Some(p.advance(1))
        }
        addresses::instructions::GO_TO_IF_DOOR_SET => {
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
        addresses::instructions::GO_TO_IF_SAMUS_NEAR => {
            creation::coordinates(h, slot);
            let x = (h.session().player.x >> 4).wrapping_sub(h.session().mechanisms.block_x) as i16;
            let y = (h.session().player.y >> 4).wrapping_sub(h.session().mechanisms.block_y) as i16;
            if x.wrapping_abs() as u8 <= p.byte(0) && y.wrapping_abs() as u8 <= p.byte(1) {
                p.jump_word(2)
            } else {
                Some(p.advance(4))
            }
        }
        addresses::instructions::MOVE_RIGHT_ONE_BLOCK => {
            mechanism!(
                h,
                slot as isize,
                block = mechanism!(h, slot as isize, block).wrapping_add(2)
            );
            Some(p)
        }
        addresses::instructions::GO_TO_IF_MISSILES_FULL
        | addresses::instructions::GO_TO_IF_HEALTH_FULL => {
            let full = if ea == addresses::instructions::GO_TO_IF_MISSILES_FULL {
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
        addresses::instructions::SET_GREY_DOOR_PRE_INSTRUCTION => {
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
        addresses::instructions::SET_BTS_1 => {
            let block = mechanism!(h, slot as isize, block);
            set_behavior_byte(h, usize::from(block >> 1), 1);
            Some(p)
        }
        addresses::instructions::SPAWN_EYE_DOOR_SMOKE => {
            h.enemy_projectile(0xe517, 0x30a);
            h.enemy_projectile(0xe517, 0x30a);
            Some(p)
        }
        addresses::instructions::DAMAGE_DRAYGON_TURRET
        | addresses::instructions::DAMAGE_DRAYGON_TURRET_DC36
        | addresses::instructions::DAMAGE_DRAYGON_TURRET_DOWN_RIGHT
        | addresses::instructions::DAMAGE_DRAYGON_TURRET_UP_RIGHT
        | addresses::instructions::DAMAGE_DRAYGON_TURRET_DOWN_LEFT
        | addresses::instructions::DAMAGE_DRAYGON_TURRET_UP_LEFT => {
            damage_turret(h, ea, slot);
            Some(p)
        }
        addresses::instructions::DELETE => {
            mechanism!(h, slot as isize, header = 0);
            None
        }
        addresses::instructions::RETURN => p.jump(mechanism!(h, slot as isize, return_program)),
        addresses::instructions::CLEAR_PRE_INSTRUCTION => {
            mechanism!(h, slot as isize, pre_action = 0x86d0);
            Some(p)
        }
        addresses::instructions::DECREMENT_TIMER_AND_GO_TO => {
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
        addresses::instructions::COLLECT_POWER_BOMB_TANK => collect(h, p, PickupKind::PowerBombs),
        addresses::instructions::PICK_UP_BEAM => collect(h, p, PickupKind::Beam),
        addresses::instructions::PICK_UP_EQUIPMENT => collect(h, p, PickupKind::Equipment),
        addresses::instructions::PICK_UP_GRAPPLE => collect(h, p, PickupKind::Grapple),
        addresses::instructions::PICK_UP_XRAY => collect(h, p, PickupKind::Xray),
        addresses::instructions::COLLECT_ENERGY_TANK => collect(h, p, PickupKind::Energy),
        addresses::instructions::COLLECT_RESERVE_TANK => collect(h, p, PickupKind::Reserve),
        addresses::instructions::COLLECT_MISSILE_TANK => collect(h, p, PickupKind::Missiles),
        addresses::instructions::COLLECT_SUPER_MISSILE_TANK => {
            collect(h, p, PickupKind::SuperMissiles)
        }
        addresses::instructions::QUEUE_MUSIC_TRACK => {
            h.clear_music_queue();
            h.music(u16::from(p.byte(0)));
            Some(p.advance(1))
        }
        addresses::instructions::GO_TO_IF_NO_BOMBS => {
            if h.session().inventory.equipment.owned & 0x1000 != 0 {
                Some(p.advance(2))
            } else {
                p.jump_word(0)
            }
        }
        addresses::instructions::LOAD_ITEM_GRAPHICS => Some(graphics::load_graphics(h, p, k)),
        addresses::instructions::TRANSFER_TO_VRAM => Some(graphics::copy_vram(h, p)),
        addresses::instructions::ACTIVATE_MAP_STATION => {
            h.mark_map_used();
            h.session_mut().messages.queued = 0x14;
            h.set_has_area_map(1);
            Some(p)
        }
        addresses::instructions::GO_TO_IF_BOSS_BITS => {
            if flags::boss_defeated(h, u16::from(p.byte(0))) {
                p.jump_word(1)
            } else {
                Some(p.advance(3))
            }
        }
        addresses::instructions::GO_TO_IF_EVENT => {
            if flags::event_happened(h, p.word(0)) {
                p.jump_word(2)
            } else {
                Some(p.advance(4))
            }
        }
        addresses::instructions::SET_EVENT => {
            flags::set_event(h, p.word(0));
            Some(p.advance(2))
        }
        addresses::instructions::GO_TO_IF_CHOZO_BLOCK_DESTROYED
        | addresses::instructions::GO_TO_IF_ITEM_COLLECTED => {
            let argument = mechanism!(h, slot as isize, room_argument);
            let found = if ea == addresses::instructions::GO_TO_IF_CHOZO_BLOCK_DESTROYED {
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
        addresses::instructions::SET_CHOZO_BLOCK_DESTROYED
        | addresses::instructions::SET_ITEM_COLLECTED => {
            let argument = mechanism!(h, slot as isize, room_argument);
            if argument & 0x8000 == 0 {
                if ea == addresses::instructions::SET_CHOZO_BLOCK_DESTROYED {
                    flags::use_room_object(h, argument);
                } else {
                    flags::collect_item(h, argument);
                }
            }
            Some(p)
        }
        addresses::instructions::SET_LINK_INSTRUCTION => {
            mechanism!(h, slot as isize, return_program = p.word(0));
            Some(p.advance(2))
        }
        addresses::instructions::CALL => {
            mechanism!(
                h,
                slot as isize,
                return_program = (p.position()).wrapping_add(2)
            );
            p.jump_word(0)
        }
        addresses::instructions::HIT_DOOR => {
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
        addresses::instructions::INCREMENT_ROOM_ARGUMENT => {
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
        addresses::instructions::SET_BTS => {
            let bts = p.byte(0);
            let block = mechanism!(h, slot as isize, block);
            set_behavior_byte(h, usize::from(block >> 1), bts);
            Some(p.advance(1))
        }
        addresses::instructions::DRAW_BLOCK | addresses::instructions::DRAW_BLOCK_8B17 => {
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
        addresses::instructions::AIR_SCROLL_UPDATE
        | addresses::instructions::SOLID_SCROLL_UPDATE => {
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
            let value = level(h, block) & 0xfff
                | if ea == addresses::instructions::AIR_SCROLL_UPDATE {
                    0x3000
                } else {
                    0xb000
                };
            set_level(h, block, value);
            Some(p)
        }
        addresses::instructions::QUEUE_MUSIC_TRACK_KEEPING_QUEUE => {
            h.music(u16::from(p.byte(0)));
            Some(p.advance(1))
        }
        addresses::instructions::QUEUE_SOUND_1_MAX_6
        | addresses::instructions::QUEUE_SOUND_2_MAX_6
        | addresses::instructions::QUEUE_SOUND_3_MAX_6
        | addresses::instructions::QUEUE_SOUND_1_MAX_15
        | addresses::instructions::QUEUE_SOUND_2_MAX_15
        | addresses::instructions::QUEUE_SOUND_3_MAX_15
        | addresses::instructions::QUEUE_SOUND_1_MAX_3
        | addresses::instructions::QUEUE_SOUND_2_MAX_3
        | addresses::instructions::QUEUE_SOUND_3_MAX_3
        | addresses::instructions::QUEUE_SOUND_1_MAX_9
        | addresses::instructions::QUEUE_SOUND_2_MAX_9
        | addresses::instructions::QUEUE_SOUND_3_MAX_9
        | addresses::instructions::QUEUE_SOUND_1_MAX_1
        | addresses::instructions::QUEUE_SOUND_2_MAX_1
        | addresses::instructions::QUEUE_SOUND_3_MAX_1 => {
            let index = (ea - addresses::instructions::QUEUE_SOUND_1_MAX_6) / 9;
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
        addresses::instructions::ACTIVATE_ENERGY_STATION
        | addresses::instructions::ACTIVATE_MISSILE_STATION => {
            let health = ea == addresses::instructions::ACTIVATE_ENERGY_STATION;
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
        addresses::instructions::SAVE_OR_GO_TO => {
            let result = h.poll_message(23);
            if result < 0 {
                return Some(p.advance(-2));
            }
            if result == 2 {
                return p.jump_word(0);
            }
            h.enemy_projectile(
                crate::constants::bank_86::ADDR_K_EPROJ_SAVE_STATION_ELECTRICITY as u16,
                0,
            );
            let selected = usize::from(h.session().mechanisms.selected >> 1);
            let station = mechanism!(h, selected as isize, room_argument) & 7;
            h.mark_save_station(station);
            let save = h.selected_save_slot();
            h.save(save);
            Some(p.advance(2))
        }
        addresses::instructions::PLACE_SAMUS_ON_SAVE_STATION => {
            h.session_mut().player.x = h.session().player.x.wrapping_add(8) & 0xfff0;
            h.face_forward();
            Some(p)
        }
        addresses::instructions::SHOW_GAME_SAVED => {
            h.session_mut().messages.queued = 0x18;
            Some(p)
        }
        addresses::instructions::FINISH_SAVE => {
            h.player_control(1);
            h.session_mut().mechanisms.save_locked = 1;
            Some(p)
        }
        addresses::instructions::CLEAR_TRIGGER => {
            mechanism!(h, slot as isize, counter = 0);
            Some(p)
        }
        addresses::instructions::SPAWN_ENEMY_PROJECTILE => {
            let definition = p.word(0);
            h.enemy_projectile(definition, definition);
            Some(p.advance(2))
        }
        addresses::instructions::WAKE_ENEMY_PROJECTILE => {
            let block = mechanism!(h, slot as isize, block);
            h.wake_projectile_for_block(block);
            Some(p.advance(2))
        }
        addresses::instructions::WRECKED_SHIP_SPIKES_TO_SLOPES
        | addresses::instructions::WRECKED_SHIP_SLOPES_TO_SPIKES => {
            install(
                h,
                0x1608,
                if ea == addresses::instructions::WRECKED_SHIP_SPIKES_TO_SLOPES {
                    0x1012
                } else {
                    0xa000
                },
            );
            install(
                h,
                0x160a,
                if ea == addresses::instructions::WRECKED_SHIP_SPIKES_TO_SLOPES {
                    0x1013
                } else {
                    0xa000
                },
            );
            Some(p)
        }
        addresses::instructions::SHOOT_EYE_DOOR_PROJECTILE => {
            h.enemy_projectile(
                crate::constants::bank_86::ADDR_K_EPROJ_EYE_DOOR_PROJECTILE as u16,
                p.word(0),
            );
            h.sound(SoundChannel::Two, 6, 0x4c);
            Some(p.advance(2))
        }
        addresses::instructions::SPAWN_EYE_DOOR_SWEAT => {
            h.enemy_projectile(
                crate::constants::bank_86::ADDR_K_EPROJ_EYE_DOOR_SWEAT as u16,
                p.word(0),
            );
            Some(p.advance(2))
        }
        addresses::instructions::SPAWN_EYE_DOOR_SWEAT_DROP => {
            h.enemy_projectile(
                crate::constants::bank_86::ADDR_K_EPROJ_EYE_DOOR_SMOKE as u16,
                0xb,
            );
            Some(p)
        }
        addresses::instructions::DRAW_AND_WAIT_4
        | addresses::instructions::DRAW_ITEM_FRAME_0
        | addresses::instructions::DRAW_ITEM_FRAME_1 => {
            if ea != addresses::instructions::DRAW_AND_WAIT_4 {
                let index = mechanism!(h, slot as isize, work1) >> 1;
                let table = h.native_record(ContentAddress {
                    bank: 0x84,
                    offset: if ea == addresses::instructions::DRAW_ITEM_FRAME_0 {
                        0xe05f
                    } else {
                        0xe077
                    },
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
        addresses::instructions::CLEAR_CHARGE_COUNTER => {
            h.session_mut().projectiles.controls.charge = 0;
            Some(p)
        }
        addresses::instructions::SET_FX_Y_VELOCITY => {
            h.session_mut().room.liquid_velocity = (-32i16) as u16;
            Some(p)
        }
        addresses::instructions::ENABLE_WATER_PHYSICS
        | addresses::instructions::SPAWN_N00B_TUBE_CRACK
        | addresses::instructions::N00B_TUBE_EARTHQUAKE
        | addresses::instructions::SPAWN_N00B_TUBE_SHARDS
        | addresses::instructions::LOCK_SAMUS
        | addresses::instructions::UNLOCK_SAMUS => {
            use rules::noob_tube::Command::*;
            let kind = match ea {
                addresses::instructions::ENABLE_WATER_PHYSICS => Water,
                addresses::instructions::SPAWN_N00B_TUBE_CRACK => Crack,
                addresses::instructions::N00B_TUBE_EARTHQUAKE => Quake,
                addresses::instructions::SPAWN_N00B_TUBE_SHARDS => Shatter,
                addresses::instructions::LOCK_SAMUS => Lock,
                _ => Unlock,
            };
            rules::noob_tube::command(h, kind);
            Some(p)
        }
        addresses::instructions::SPAWN_TORIZO_STATUE_BREAKING => {
            h.enemy_projectile(0xa993, p.word(0));
            Some(p.advance(2))
        }
        addresses::instructions::QUEUE_SONG_1 => {
            h.music(6);
            Some(p)
        }
        addresses::instructions::SET_FX_BASE_Y => {
            rules::hands::lower_acid(h);
            Some(p)
        }
        addresses::instructions::DRAW_RIGHTWARDS_TREADMILL
        | addresses::instructions::DRAW_LEFTWARDS_TREADMILL => {
            rules::treadmill::activate(
                h,
                slot,
                if ea == addresses::instructions::DRAW_RIGHTWARDS_TREADMILL {
                    rules::treadmill::Direction::Right
                } else {
                    rules::treadmill::Direction::Left
                },
            );
            Some(p)
        }
        addresses::instructions::SCROLL_0_1_BLUE => {
            rules::crumbling_wall::open_passage(h);
            Some(p)
        }
        addresses::instructions::MOVE_DOWN_ONE_BLOCK
        | addresses::instructions::MOVE_DOWN_ONE_BLOCK_AB59 => {
            rules::crumbling_wall::move_down(h, slot);
            Some(p)
        }
        addresses::instructions::GO_TO_IF_ROOM_ARGUMENT_BELOW
        | addresses::instructions::SPAWN_GLASS_SHARDS => rules::mother_brain_glass::command(
            h,
            p,
            slot,
            if ea == addresses::instructions::GO_TO_IF_ROOM_ARGUMENT_BELOW {
                rules::mother_brain_glass::Command::Threshold
            } else {
                rules::mother_brain_glass::Command::Shatter
            },
        ),
        addresses::instructions::SET_CRITTERS_ESCAPED => {
            rules::animal_rescue::mark_rescued(h);
            Some(p)
        }
        addresses::instructions::MOVE_RIGHT_FOUR_BLOCKS => {
            rules::animal_rescue::move_passage(h, slot);
            Some(p)
        }
        addresses::instructions::GO_TO => p.jump_word(0),
        addresses::instructions::DAMAGE_SAMUS_2 => {
            rules::plants::hurt(h);
            Some(p)
        }
        addresses::instructions::GIVE_SAMUS_INVINCIBILITY => {
            rules::plants::protect(h);
            Some(p)
        }
        addresses::instructions::MAKE_BLUE_DOOR_RIGHT
        | addresses::instructions::MAKE_BLUE_DOOR_LEFT => {
            rules::eye_door::convert(
                h,
                slot,
                if ea == addresses::instructions::MAKE_BLUE_DOOR_RIGHT {
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
    if ea == addresses::instructions::DAMAGE_DRAYGON_TURRET
        || ea == addresses::instructions::DAMAGE_DRAYGON_TURRET_DC36
    {
        let below = below(h, block);
        install(h, below, 0xa003);
        return;
    }
    let right = ea == addresses::instructions::DAMAGE_DRAYGON_TURRET_DOWN_RIGHT
        || ea == addresses::instructions::DAMAGE_DRAYGON_TURRET_UP_RIGHT;
    let down = ea == addresses::instructions::DAMAGE_DRAYGON_TURRET_DOWN_RIGHT
        || ea == addresses::instructions::DAMAGE_DRAYGON_TURRET_DOWN_LEFT;
    let step = if right { 2 } else { 0xfffe };
    install(h, block.wrapping_add(step), if down { 0xa003 } else { 0 });
    let origin = if ea == addresses::instructions::DAMAGE_DRAYGON_TURRET_DOWN_RIGHT {
        mechanism!(h, (h.session().mechanisms.selected >> 1) as isize, block)
    } else {
        block
    };
    let below = below(h, origin);
    install(h, below, 0xa003);
    install(h, below.wrapping_add(step), if down { 0 } else { 0xa003 });
}
