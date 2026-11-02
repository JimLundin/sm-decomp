//! Corpse-only semantic helpers, extracted from Mother Brain's shared bodies.
#![forbid(unsafe_code)]
use crate::actors::access::{AccessMode, ActorKey};
use crate::actors::owned_execution::PreparedActorState;
use crate::actors::session_slots::session_slot;
use crate::menus::prelude::EnemyData;
pub(super) const BANK: u8 = 0xa9;
pub(super) const BANK_BASE: u32 = 0xa90000;
pub(super) mod cgram {
    pub const SPRITE_PALETTE_1: u16 = 0x122;
}
pub(super) mod sound {
    pub const LAVA_ACID_DAMAGE: u16 = 0x10;
}
pub(super) fn bank_unreachable(h: &mut impl Host) -> ! {
    h.remaining_unknown()
}
pub(super) fn random_number(h: &impl Host) -> u16 {
    h.session().timing.random_number
}
pub(super) fn native_record(address: u32, h: &mut impl Host) -> sm_data::cartridge::RomRecord {
    let bytes = address.to_le_bytes();
    h.native_actor_record(bytes[2], u16::from_le_bytes([bytes[0], bytes[1]]))
}
pub(super) fn native_word(at: usize, h: &impl Host) -> u16 {
    h.corpse_native_word(at)
}
pub(super) fn set_native_word(at: usize, value: u16, h: &mut impl Host) {
    // The queue path publishes each completed byte; other aliases stay with
    // the exclusive current aggregate. The full slice is admitted first.
    h.write_corpse_native_word(at, value);
}
pub(super) fn queue_sfx2_max3(sound: u16, h: &mut impl Host) {
    crate::enemy_projectiles::host::Effects::queue_sound(h, 2, 3, sound);
}
pub(super) fn queue_sfx2_max6(sound: u16, h: &mut impl Host) {
    crate::enemy_projectiles::host::Effects::queue_sound(h, 2, 6, sound);
}
pub(super) fn spawn_room_projectile(definition: u16, parameter: u16, h: &mut impl Host) {
    crate::enemy_projectiles::lifecycle::spawn_room(h, definition, parameter);
}
pub(super) fn set_enemy_instruction_list(k: u16, a: u16, h: &mut impl Host) {
    let k = i32::from(k);
    session_slot! { h, EnemyData[k].base; current_instruction = a, instruction_timer = 1 }
    session_slot!(h, EnemyData[k].base.timer = 0);
}
pub(super) fn set_graphics_drawn_hook(hook: u32, h: &mut impl Host) {
    let bytes = hook.to_le_bytes();
    for (index, byte) in bytes[..3].iter().copied().enumerate() {
        h.session_mut().actors.drawing.hook_address[index] = byte;
    }
}
pub(super) fn move_enemy_with_velocity(h: &mut impl Host) {
    let k = i32::from(h.current_enemy());
    let vx = session_slot!(h, EnemyData[k].ai_var_b);
    let mut carry = i32::from(session_slot!(byte h, EnemyData[k].base.x_subpos[1]))
        + i32::from(vx.to_ne_bytes()[0]);
    session_slot!(byte h, EnemyData[k].base.x_subpos[1] = carry.to_le_bytes()[0]);
    session_slot!(h, EnemyData[k].base.x_pos, |old| old.wrapping_add_signed(
        i16::from(vx.to_ne_bytes()[1] as i8) + i16::try_from(carry >> 8).expect("horizontal carry")
    ));
    let vy = session_slot!(h, EnemyData[k].ai_var_c);
    carry = i32::from(session_slot!(byte h, EnemyData[k].base.y_subpos[1]))
        + i32::from(vy.to_ne_bytes()[0]);
    session_slot!(byte h, EnemyData[k].base.y_subpos[1] = carry.to_le_bytes()[0]);
    session_slot!(h, EnemyData[k].base.y_pos, |old| old.wrapping_add_signed(
        i16::from(vy.to_ne_bytes()[1] as i8) + i16::try_from(carry >> 8).expect("vertical carry")
    ));
}
pub(super) fn copy_rom_colors_to_palette(
    mut target: u16,
    bank: u8,
    mut source: u16,
    count: u16,
    h: &mut impl Host,
) {
    let mut left = i32::from(count);
    loop {
        let value = h.native_actor_record(bank, source).word(0);
        h.set_remaining_color(false, usize::from(target & !1), value, false);
        target = target.wrapping_add(2);
        source = source.wrapping_add(2);
        left -= 1;
        if left == 0 {
            break;
        }
    }
}
pub(super) fn corpse_rotting_finished(h: &mut impl Host) {
    let brain = ActorKey::from_legacy(0x40);
    let x = h
        .resolve_actor(brain, AccessMode::Native)
        .x
        .whole
        .wrapping_add(random_number(h) & 0x1f)
        .wrapping_sub(16);
    let y = h
        .resolve_actor(brain, AccessMode::Native)
        .y
        .whole
        .wrapping_add(16);
    h.session_mut().enemy_projectiles.spawn.point = crate::geometry::Point { x, y };
    spawn_room_projectile(
        crate::constants::bank_86::ADDR_K_EPROJ_DUST_CLOUD_EXPLOSION as u16,
        0xa,
        h,
    );
    if h.session().actors.controls.phase_counter & 7 == 0 {
        queue_sfx2_max3(sound::LAVA_ACID_DAMAGE, h);
    }
}
pub(super) fn add_spritemap_to_oam(map: u16, x: u16, y: u16, palette: u16, h: &mut impl Host) {
    h.draw_corpse_spritemap(map, x, y, palette);
}
pub(super) fn dead_torizo_contact(h: &mut impl Host) -> bool {
    let actor = h.resolve_actor(ActorKey::from_legacy(0), AccessMode::Native);
    let (x, y) = (actor.x.whole, actor.y.whole);
    let boxes = h.native_actor_record(
        BANK,
        crate::constants::bank_a9::ADDR_K_DEAD_TORIZO_HITBOX_D77C as u16,
    );
    let abs = |value: u16| i32::from((value as i16).unsigned_abs());
    for part in 0..usize::from(boxes.word(0)) {
        let at = 2 + 8 * part;
        let sy = h.session().player.y;
        let (distance, extent) = if (sy as i16).wrapping_sub(y as i16) >= 0 {
            (sy.wrapping_sub(y), boxes.word(at + 6))
        } else {
            (y.wrapping_sub(sy), boxes.word(at + 2))
        };
        if ((i32::from(h.session().player.contact.radius) + abs(extent) - i32::from(distance))
            as i16)
            < 0
        {
            continue;
        }
        let sx = h.session().player.x;
        let (distance, extent) = if (sx as i16).wrapping_sub(x as i16) >= 0 {
            (sx.wrapping_sub(x), boxes.word(at + 4))
        } else {
            (x.wrapping_sub(sx), boxes.word(at))
        };
        let mut overlap = (i32::from(h.session().player.contact.x_radius) + abs(extent)
            - i32::from(distance)) as i16;
        if overlap < 0 {
            continue;
        }
        if (i32::from(overlap) - 4) & 0x8000 != 0 {
            overlap = 4;
        }
        h.session_mut().player.contact.external_x.whole = overlap as u16;
        h.session_mut().player.contact.external_y.whole = 4;
        h.session_mut().player.contact.external_x.fraction = 0;
        h.session_mut().player.contact.external_y.fraction = 0;
        return true;
    }
    false
}

/// The corpses' actual tile, native prefix and synchronous drawing capabilities.
/// In-range tile bytes live only in the Session's extended actor owner.
pub trait Host:
    crate::actors::minor_interaction::NativeCapture + crate::display::transfer::TilemapQueue
{
    fn corpse_tile_alias_byte(&self, at: usize) -> u8;
    fn write_corpse_tile_alias_slice(&mut self, at: usize, bytes: &[u8]);
    fn corpse_native_word(&self, at: usize) -> u16;
    fn write_corpse_native_word(&mut self, at: usize, value: u16);
    fn draw_corpse_spritemap(&mut self, map: u16, x: u16, y: u16, palette: u16);
    fn corpse_tile_byte(&self, at: usize) -> u8 {
        if let Some(relative) = at.checked_sub(0x2000).filter(|&r| r < 4096) {
            self.session()
                .actors
                .extended
                .word(4 + relative / 2048, (relative % 2048) / 2)
                .to_le_bytes()[relative & 1]
        } else {
            self.corpse_tile_alias_byte(at)
        }
    }
    fn write_corpse_tile_slice(&mut self, at: usize, bytes: &[u8]) {
        // Admit the same complete slice before publishing its first byte.
        if at >= 0x2000 && at.checked_add(bytes.len()).is_some_and(|end| end <= 0x3000) {
            for (offset, byte) in bytes.iter().copied().enumerate() {
                let relative = at - 0x2000 + offset;
                let word = self
                    .session_mut()
                    .actors
                    .extended
                    .word_mut(4 + relative / 2048, (relative % 2048) / 2);
                let mut lanes = word.to_le_bytes();
                lanes[relative & 1] = byte;
                *word = u16::from_le_bytes(lanes);
            }
        } else {
            self.write_corpse_tile_alias_slice(at, bytes);
        }
    }
    fn write_corpse_tile_word(&mut self, at: usize, value: u16) {
        self.write_corpse_tile_slice(at, &value.to_le_bytes());
    }
}
impl Host for PreparedActorState<'_> {
    fn corpse_tile_alias_byte(&self, at: usize) -> u8 {
        self.current_byte(at)
    }
    fn write_corpse_tile_alias_slice(&mut self, at: usize, bytes: &[u8]) {
        self.write_current_slice(at, bytes);
    }
    fn corpse_native_word(&self, at: usize) -> u16 {
        self.current_native_word(at)
    }
    fn write_corpse_native_word(&mut self, at: usize, value: u16) {
        self.mechanism_queue_slice(at, &value.to_ne_bytes());
    }
    fn draw_corpse_spritemap(&mut self, map: u16, x: u16, y: u16, palette: u16) {
        self.guarded_actor_call(|h| {
            h.presentation
                .draw_room_sprite(
                    &mut h.owner.gameplay,
                    crate::scene::actor_presentation::RoomSpriteRequest {
                        bank: BANK,
                        map,
                        position: crate::geometry::Point { x, y },
                        palette,
                        clip: crate::scene::objects::RoomClip::Above,
                    },
                    h.aliases.presentation_diagnostics(),
                )
                .expect("corpse room spritemap publication");
        });
    }
}
