//! Ordered enemy contact scans. Checked handles retain slots across diagnostics.
#![deny(unsafe_op_in_unsafe_fn)]
use crate::game::constants::bank_a0::{
    addr_kEnemyDef_DAFF, addr_kExtendedSpritemap_Nothing_A0, addr_kSpritemap_Nothing_A0,
};
use crate::game::enemy_ram::{native_read, EnemyField as F, EnemySlot};
use crate::game::sm_a0::{CallEnemyAi, CallHitboxShot, CallHitboxTouch};
use crate::game_state::Game;
use crate::ram_map::checked_word as word;
use crate::rom::RomRecord;
use crate::rtl::{rom_record, Unreachable};
use crate::types::{EnemyDef, EnemySpawnData, ExtendedSpriteMap, Hitbox};
use core::mem::{offset_of, size_of};
use crate::game::contact_rules::{axis, is_shot, Bounds};

fn current(game: &Game) -> EnemySlot {
    EnemySlot::new(word!(game.cur_enemy_index))
}
fn ignored(ai: u16) -> bool {
    ai == 0x804b || ai == 0x804c
}
fn mark_projectile(index: u16, game: &mut Game) {
    word!(game.projectile_dir[index] = word!(game.projectile_dir[index]) | 0x10);
}
pub unsafe fn process_contacts(game: &mut Game) {
    if current(game).word(F::ExtraProperties, game) & 4 != 0 {
        unsafe {
            projectile_contacts(Attack::Shot, true, game);
            projectile_contacts(Attack::Bomb, true, game);
            touch_hitboxes(game);
        }
    } else {
        unsafe {
            projectile_contacts(Attack::Shot, false, game);
            projectile_contacts(Attack::Bomb, false, game);
            touch_body(game);
        }
    }
}

unsafe fn touch_body(game: &mut Game) {
    let enemy = current(game);
    word!(game.enemy_processing_stage = 9);
    if enemy.word(F::Map, game) == 0 {
        return;
    }
    if word!(game.samus_contact_damage_index) != 0 {
        word!(game.samus_invincibility_timer = 0);
    } else if word!(game.samus_invincibility_timer) != 0 {
        if enemy.word(F::Definition, game) != addr_kEnemyDef_DAFF as u16 {
            return;
        }
        let flag = native_read(
            game,
            0x7000
                + usize::from(word!(game.cur_enemy_index))
                + offset_of!(EnemySpawnData, some_flag),
        );
        if flag == 0 || flag == 8 {
            return;
        }
    }
    let definition = unsafe { rom_record(0xa0, enemy.word(F::Definition, game), game) };
    if ignored(definition.word(offset_of!(EnemyDef, touch_ai))) {
        return;
    }
    if axis(
        word!(game.samus_x_pos),
        enemy.word(F::X, game),
        word!(game.samus_x_radius),
        enemy.word(F::Width, game),
    ) && axis(
        word!(game.samus_y_pos),
        enemy.word(F::Y, game),
        word!(game.samus_y_radius),
        enemy.word(F::Height, game),
    ) && (enemy.word(F::Definition, game) == addr_kEnemyDef_DAFF as u16
        || enemy.word(F::Frozen, game) == 0)
    {
        unsafe {
            CallEnemyAi(
                (u32::from(enemy.bank(game)) << 16)
                    | u32::from(definition.word(offset_of!(EnemyDef, touch_ai))),
                game,
            );
        }
    }
}
unsafe fn touch_hitboxes(game: &mut Game) {
    let enemy = current(game);
    word!(game.enemy_processing_stage = 6);
    if enemy.word(F::Map, game) == 0 {
        return;
    }
    let definition = unsafe { rom_record(0xa0, enemy.word(F::Definition, game), game) };
    if ignored(definition.word(offset_of!(EnemyDef, touch_ai))) {
        return;
    }
    if word!(game.samus_contact_damage_index) != 0 {
        word!(game.samus_invincibility_timer = 0);
    } else if word!(game.samus_invincibility_timer) != 0 {
        return;
    }
    if enemy.word(F::Map, game) & 0x8000 == 0 {
        return;
    }
    let bounds = Bounds::new(
        word!(game.samus_x_pos),
        word!(game.samus_y_pos),
        word!(game.samus_x_radius),
        word!(game.samus_y_radius),
    );
    let mut remaining =
        i32::from(unsafe { rom_record(enemy.bank(game), enemy.word(F::Map, game), game) }.byte(0));
    let mut address = enemy.word(F::Map, game).wrapping_add(2);
    // Touch visits the first entry even for count zero, and wraps each bank address.
    loop {
        let entry = unsafe { rom_record(enemy.bank(game), address, game) };
        let x = entry
            .word(offset_of!(ExtendedSpriteMap, xpos))
            .wrapping_add(enemy.word(F::X, game));
        let y = entry
            .word(offset_of!(ExtendedSpriteMap, ypos))
            .wrapping_add(enemy.word(F::Y, game));
        let hitboxes = unsafe {
            rom_record(
                enemy.bank(game),
                entry.word(offset_of!(ExtendedSpriteMap, hitbox_ptr_)),
                game,
            )
        };
        for index in 0..usize::from(hitboxes.word(0)) {
            let at = 2 + index * size_of::<Hitbox>();
            if bounds.hits(x, y, &hitboxes, at, true) {
                unsafe {
                    CallHitboxTouch(
                        (u32::from(enemy.bank(game)) << 16)
                            | u32::from(hitboxes.word(at + offset_of!(Hitbox, func_ptr))),
                        game,
                    );
                }
                return;
            }
        }
        address = address.wrapping_add(size_of::<ExtendedSpriteMap>() as u16);
        remaining = remaining.wrapping_sub(1);
        if remaining == 0 {
            return;
        }
    }
}
#[derive(Clone, Copy, PartialEq)]
enum Attack {
    Shot,
    Bomb,
}
impl Attack {
    fn eligible(self, index: u16, extended: bool, game: &Game) -> bool {
        let kind = word!(game.projectile_type[index]);
        if self == Self::Shot {
            is_shot(kind)
        } else if extended {
            word!(game.projectile_x_pos[index]) != 0
                && kind != 0
                && kind & 0xf00 == 0x500
                && word!(game.projectile_variables[index]) == 0
        } else {
            kind != 0
                && word!(game.projectile_variables[index]) == 0
                && (kind & 0xf00 == 0x500 || kind & 0x8000 != 0)
        }
    }
}

unsafe fn projectile_contacts(attack: Attack, extended: bool, game: &mut Game) {
    let enemy = current(game);
    let bomb = attack == Attack::Bomb;
    word!(
        game.enemy_processing_stage = if extended {
            if bomb {
                4
            } else {
                3
            }
        } else if bomb {
            8
        } else {
            7
        }
    );
    // The four entry gates intentionally sample diagnostics and counters in different orders.
    if extended {
        if bomb {
            if enemy.word(F::Map, game) == 0
                || enemy.word(F::Properties, game) & 0x400 != 0
                || enemy.word(F::Invincibility, game) != 0
            {
                return;
            }
        } else if word!(game.projectile_counter) == 0
            || enemy.word(F::Map, game) == 0
            || enemy.word(F::Map, game) == addr_kExtendedSpritemap_Nothing_A0 as u16
        {
            return;
        }
        let definition = unsafe { rom_record(0xa0, enemy.word(F::Definition, game), game) };
        if ignored(definition.word(offset_of!(EnemyDef, shot_ai))) {
            return;
        }
        if bomb {
            if word!(game.bomb_counter) == 0 {
                return;
            }
        } else if enemy.word(F::Properties, game) & 0x400 != 0
            || enemy.word(F::Invincibility, game) != 0
            || enemy.word(F::Definition, game) == addr_kEnemyDef_DAFF as u16
        {
            return;
        }
    } else if bomb {
        if word!(game.bomb_counter) == 0
            || enemy.word(F::Map, game) == 0
            || enemy.word(F::Invincibility, game) != 0
            || enemy.word(F::Definition, game) == addr_kEnemyDef_DAFF as u16
        {
            return;
        }
    } else if word!(game.projectile_counter) == 0
        || enemy.word(F::Map, game) == 0
        || enemy.word(F::Map, game) == addr_kSpritemap_Nothing_A0 as u16
        || enemy.word(F::Properties, game) & 0x400 != 0
        || enemy.word(F::Definition, game) == addr_kEnemyDef_DAFF as u16
        || enemy.word(F::Invincibility, game) != 0
    {
        return;
    }
    for index in if bomb { 5..10 } else { 0..5 } {
        if !attack.eligible(index, extended, game) {
            continue;
        }
        let hitbox = if extended {
            let found = unsafe { projectile_hitbox(enemy, index, bomb, game) };
            if found.is_none() {
                continue;
            }
            found
        } else {
            if !axis(
                word!(game.projectile_x_pos[index]),
                enemy.word(F::X, game),
                word!(game.projectile_x_radius[index]),
                enemy.word(F::Width, game),
            ) || !axis(
                word!(game.projectile_y_pos[index]),
                enemy.word(F::Y, game),
                word!(game.projectile_y_radius[index]),
                enemy.word(F::Height, game),
            ) {
                continue;
            }
            None
        };
        if !bomb && word!(game.projectile_type[index]) & 0xf00 == 0x200 {
            word!(game.earthquake_timer = 30);
            word!(game.earthquake_type = 18);
        }
        if bomb && !extended {
            word!(game.collision_detection_index = index);
        }
        if bomb
            || enemy.word(F::Properties, game) & 0x1000 != 0
            || word!(game.projectile_type[index]) & 8 == 0
        {
            mark_projectile(index, game);
        }
        if !bomb || extended {
            word!(game.collision_detection_index = index);
        }
        if let Some(function) = hitbox {
            unsafe {
                CallHitboxShot(
                    (u32::from(enemy.bank(game)) << 16) | u32::from(function),
                    index * 2,
                    game,
                );
            }
        } else {
            // The bank is captured before the definition's diagnostic can mutate it.
            let bank = enemy.bank(game);
            let definition = unsafe { rom_record(0xa0, enemy.word(F::Definition, game), game) };
            unsafe {
                CallEnemyAi(
                    (u32::from(bank) << 16)
                        | u32::from(definition.word(offset_of!(EnemyDef, shot_ai))),
                    game,
                );
            }
        }
        return;
    }
}
unsafe fn projectile_hitbox(
    enemy: EnemySlot,
    index: u16,
    bomb: bool,
    game: &mut Game,
) -> Option<u16> {
    if enemy.word(F::Map, game) & 0x8000 == 0 {
        unsafe {
            Unreachable(game);
        }
    }
    let entries = unsafe { rom_record(enemy.bank(game), enemy.word(F::Map, game), game) };
    // Shots use a byte count; bombs use a native word. Both retain linear allocation offsets.
    let count = if bomb {
        entries.word(0)
    } else {
        u16::from(entries.byte(0))
    };
    for entry in 0..usize::from(count) {
        let at = 2 + entry * size_of::<ExtendedSpriteMap>();
        let x = entries
            .word(at + offset_of!(ExtendedSpriteMap, xpos))
            .wrapping_add(enemy.word(F::X, game));
        let y = entries
            .word(at + offset_of!(ExtendedSpriteMap, ypos))
            .wrapping_add(enemy.word(F::Y, game));
        let hitboxes = unsafe {
            rom_record(
                enemy.bank(game),
                entries.word(at + offset_of!(ExtendedSpriteMap, hitbox_ptr_)),
                game,
            )
        };
        let bounds = Bounds::new(
            word!(game.projectile_x_pos[index]),
            word!(game.projectile_y_pos[index]),
            word!(game.projectile_x_radius[index]),
            word!(game.projectile_y_radius[index]),
        );
        for hitbox in 0..usize::from(hitboxes.word(0)) {
            let at = 2 + hitbox * size_of::<Hitbox>();
            if bounds.hits(x, y, &hitboxes, at, false) {
                return Some(hitboxes.word(at + offset_of!(Hitbox, func_ptrA)));
            }
        }
    }
    None
}
