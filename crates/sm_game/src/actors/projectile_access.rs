//! Finite projectile actor fields on the current canonical Session owner.
#![forbid(unsafe_code)]
use super::PreparedActorState;
use crate::actors::layouts::{EnemyLayout72, EnemyLayout104};
use crate::enemy_projectiles::host::{BotwoonField, FireArcField, SpeciesActors};
use core::mem::offset_of;
fn waffle_offset(field: FireArcField) -> usize {
    match field {
        FireArcField::Behavior => offset_of!(EnemyLayout72, var_a),
        FireArcField::Reverse => offset_of!(EnemyLayout72, var_e),
        FireArcField::Radius => offset_of!(EnemyLayout72, var_d),
        FireArcField::PhaseFraction => offset_of!(EnemyLayout72, var_20),
        FireArcField::Phase => offset_of!(EnemyLayout72, var_21),
        FireArcField::StartPhase => offset_of!(EnemyLayout72, var_23),
        FireArcField::PhaseSpeedFraction => offset_of!(EnemyLayout72, var_24),
        FireArcField::PhaseSpeed => offset_of!(EnemyLayout72, var_25),
        FireArcField::CenterX => offset_of!(EnemyLayout72, var_26),
        FireArcField::CenterY => offset_of!(EnemyLayout72, var_27),
        FireArcField::PartCursor => offset_of!(EnemyLayout72, var_2a),
        FireArcField::Spacing => offset_of!(EnemyLayout72, var_2b),
        FireArcField::PhaseLead => offset_of!(EnemyLayout72, var_2c),
        FireArcField::EndPhase => offset_of!(EnemyLayout72, var_2e),
        FireArcField::FirstTransition => offset_of!(EnemyLayout72, var_2f),
        FireArcField::SecondTransition => offset_of!(EnemyLayout72, var_30),
        FireArcField::HeadStage => offset_of!(EnemyLayout72, var_33),
        FireArcField::Palette => offset_of!(EnemyLayout72, var_34),
        FireArcField::Projectile => offset_of!(EnemyLayout72, var_00),
        FireArcField::PartStage => offset_of!(EnemyLayout72, var_08),
        FireArcField::Sprite => offset_of!(EnemyLayout72, var_10),
        FireArcField::SpriteStage => offset_of!(EnemyLayout72, var_18),
    }
}
fn botwoon_offset(field: BotwoonField) -> usize {
    match field {
        BotwoonField::HistoryCursor => offset_of!(EnemyLayout104, var_b),
        BotwoonField::HistorySpacing => offset_of!(EnemyLayout104, var_c),
        BotwoonField::Projectile => offset_of!(EnemyLayout104, var_00),
        BotwoonField::Visible => offset_of!(EnemyLayout104, var_10),
        BotwoonField::TransitionPending => offset_of!(EnemyLayout104, var_35),
        BotwoonField::TransitionCursor => offset_of!(EnemyLayout104, var_36),
        BotwoonField::TailState => offset_of!(EnemyLayout104, var_3e),
        BotwoonField::Pattern => offset_of!(EnemyLayout104, var_33),
    }
}
impl SpeciesActors for PreparedActorState<'_> {
    fn mini_kraid_spit_x_velocity(&self, index: u16) -> u16 {
        self.extended_actor_word(0, usize::from(index), 0)
    }
    fn mini_kraid_spit_y_velocity(&self, index: u16) -> u16 {
        self.extended_actor_word(0, usize::from(index), 1)
    }
    fn mini_kraid_spit_x_offset(&self, index: u16) -> u16 {
        self.extended_actor_word(0, usize::from(index), 2)
    }
    fn boss_death_quake_sound_timer(&self) -> u16 {
        *self.owner.gameplay.actors.extended.word(3, 0)
    }
    fn fire_arc_part_offset(&self, index: u16) -> u16 {
        self.extended_actor_word(1, usize::from(index), 10)
    }
    fn enemy_projectile_origin_x(&self, index: u16) -> u16 {
        self.extended_actor_word(1, usize::from(index), 17)
    }
    fn enemy_projectile_origin_y(&self, index: u16) -> u16 {
        self.extended_actor_word(1, usize::from(index), 18)
    }
    fn maw_body_part_index(&self, index: u16) -> u16 {
        self.extended_actor_word(2, usize::from(index), 4)
    }
    fn kraid_health_eighth(&self, index: u16) -> u16 {
        self.extended_actor_word(0, usize::from(index), 6)
    }
    fn mini_kraid_spike_variant(&self, index: u16) -> u16 {
        self.extended_actor_word(0, usize::from(index), 6)
    }
    fn set_enemy_linked_projectile(&mut self, index: u16, value: u16) {
        self.store_extended_actor_word(0, usize::from(index), 0, value);
    }
    fn set_botwoon_body_active(&mut self, index: u16, value: u16) {
        if index < 2048 && index.is_multiple_of(64) {
            for (lane, byte) in value.to_le_bytes().into_iter().enumerate() {
                let target = &mut self.owner.gameplay.actors.extended.botwoon_body
                    [usize::from(index) / 64]
                    .visibility[0];
                let mut bytes = target.to_le_bytes();
                bytes[lane] = byte;
                *target = u16::from_le_bytes(bytes);
                crate::session_access::SessionAccess::refresh_inputs(self);
            }
        } else {
            self.store_extended_actor_word(0, usize::from(index), 16, value);
        }
    }
    fn set_maw_body_link(&mut self, index: u16, value: u16) {
        self.store_extended_actor_word(2, usize::from(index), 0, value);
    }
    fn mother_brain_active_bombs(&self) -> u16 {
        *self.owner.gameplay.actors.extended.word(0, 37)
    }
    fn mother_brain_blue_ring_contacts(&self) -> u16 {
        *self.owner.gameplay.actors.extended.word(0, 20)
    }
    fn mother_brain_target_enemy(&self) -> u16 {
        *self.owner.gameplay.actors.extended.word(0, 42)
    }
    fn mother_brain_turret_destroyed(&self) -> u16 {
        *self.owner.gameplay.actors.extended.word(0, 29)
    }
    fn botwoon_death_phase(&self) -> u16 {
        self.owner.gameplay.actors.extended.botwoon_head[0].active
    }
    fn ridley_fireball_x_velocity(&self) -> u16 {
        *self.owner.gameplay.actors.extended.word(0, 25)
    }
    fn ridley_fireball_y_velocity(&self) -> u16 {
        *self.owner.gameplay.actors.extended.word(0, 26)
    }
    fn set_mother_brain_active_bombs(&mut self, value: u16) {
        self.store_extended_actor_word(0, 0, 37, value);
    }
    fn set_mother_brain_blue_ring_contacts(&mut self, value: u16) {
        self.store_extended_actor_word(0, 0, 20, value);
    }
    fn set_mother_brain_hurt_frame(&mut self, value: u16) {
        self.store_extended_actor_word(0, 0, 53, value);
    }
    fn set_mother_brain_purple_breath_active(&mut self, value: u16) {
        self.store_extended_actor_word(0, 0, 53, value);
    }
    fn mother_brain_beam_x_fraction(&self) -> u16 {
        *self.owner.gameplay.actors.extended.word(1, 3)
    }
    fn set_mother_brain_beam_x_fraction(&mut self, value: u16) {
        self.store_extended_actor_word(1, 0, 3, value);
    }
    fn mother_brain_beam_x(&self) -> u16 {
        *self.owner.gameplay.actors.extended.word(1, 4)
    }
    fn set_mother_brain_beam_x(&mut self, value: u16) {
        self.store_extended_actor_word(1, 0, 4, value);
    }
    fn mother_brain_beam_y_fraction(&self) -> u16 {
        *self.owner.gameplay.actors.extended.word(1, 5)
    }
    fn set_mother_brain_beam_y_fraction(&mut self, value: u16) {
        self.store_extended_actor_word(1, 0, 5, value);
    }
    fn mother_brain_beam_y(&self) -> u16 {
        *self.owner.gameplay.actors.extended.word(1, 6)
    }
    fn set_mother_brain_beam_y(&mut self, value: u16) {
        self.store_extended_actor_word(1, 0, 6, value);
    }
    fn mother_brain_beam_velocity_x(&self) -> u16 {
        *self.owner.gameplay.actors.extended.word(1, 7)
    }
    fn set_mother_brain_beam_velocity_x(&mut self, value: u16) {
        self.store_extended_actor_word(1, 0, 7, value);
    }
    fn mother_brain_beam_velocity_y(&self) -> u16 {
        *self.owner.gameplay.actors.extended.word(1, 8)
    }
    fn set_mother_brain_beam_velocity_y(&mut self, value: u16) {
        self.store_extended_actor_word(1, 0, 8, value);
    }
    fn mother_brain_beam_angle(&self) -> u16 {
        *self.owner.gameplay.actors.extended.word(1, 9)
    }
    fn set_mother_brain_beam_angle(&mut self, value: u16) {
        self.store_extended_actor_word(1, 0, 9, value);
    }
    fn set_enemy_extra_health_fraction(&mut self, index: u16, element: usize, value: u16) {
        self.store_extended_actor_word(0, usize::from(index), 6 + element, value);
    }
    fn set_botwoon_body_landed(&mut self, value: u16) {
        for (lane, byte) in value.to_le_bytes().into_iter().enumerate() {
            let target = &mut self.owner.gameplay.actors.extended.botwoon_path[0].body_landed;
            let mut bytes = target.to_le_bytes();
            bytes[lane] = byte;
            *target = u16::from_le_bytes(bytes);
            crate::session_access::SessionAccess::refresh_inputs(self);
        }
    }
    fn spore_stem_anchor_x(&self) -> u16 {
        *self.owner.gameplay.actors.extended.word(0, 4)
    }
    fn spore_stem_anchor_y(&self) -> u16 {
        *self.owner.gameplay.actors.extended.word(0, 5)
    }
    fn set_spore_stem_control_x(&mut self, part: u8, value: u16) {
        sm_data::table::check_index(usize::from(part), 3);
        self.store_extended_actor_word(1, 0, usize::from(part), value);
    }
    fn set_spore_stem_control_y(&mut self, part: u8, value: u16) {
        sm_data::table::check_index(usize::from(part), 3);
        self.store_extended_actor_word(1, 0, 3 + usize::from(part), value);
    }
    fn maw_segment_handle(&self, enemy: u16, part: u8) -> u16 {
        sm_data::table::check_index(usize::from(part), 4);
        self.extended_actor_word(2, usize::from(enemy), usize::from(part))
    }
    fn maw_anchor_x(&self, enemy: u16) -> u16 {
        self.extended_actor_word(0, usize::from(enemy), 12)
    }
    fn maw_anchor_y(&self, enemy: u16) -> u16 {
        self.extended_actor_word(0, usize::from(enemy), 13)
    }
    fn maw_segment_offset_x(&self, enemy: u16, part: u8) -> u16 {
        sm_data::table::check_index(usize::from(part).checked_sub(1).unwrap(), 3);
        self.extended_actor_word(
            0,
            usize::from(enemy),
            usize::from(part).checked_sub(1).unwrap() * 2,
        )
    }
    fn maw_segment_offset_y(&self, enemy: u16, part: u8) -> u16 {
        sm_data::table::check_index(usize::from(part).checked_sub(1).unwrap(), 3);
        self.extended_actor_word(
            0,
            usize::from(enemy),
            1 + usize::from(part).checked_sub(1).unwrap() * 2,
        )
    }
    fn maw_palette(&self, enemy: u16) -> u16 {
        self.extended_actor_word(2, usize::from(enemy), 5)
    }
    fn expanding_square_origin(&self) -> u16 {
        self.current_word(0x9e00)
    }
    fn fire_arc_field(&self, enemy: usize, field: FireArcField) -> u16 {
        self.actor_layout_word(enemy, waffle_offset(field))
    }
    fn set_fire_arc_field(&mut self, enemy: usize, field: FireArcField, value: u16) {
        self.store_actor_layout_word(enemy, waffle_offset(field), value);
    }
    fn botwoon_field(&self, enemy: usize, field: BotwoonField) -> u16 {
        self.botwoon_owned_field(enemy, field)
            .copied()
            .unwrap_or_else(|| self.actor_layout_word(enemy, botwoon_offset(field)))
    }
    fn set_botwoon_field(&mut self, enemy: usize, field: BotwoonField, value: u16) {
        if self.botwoon_owned_field(enemy, field).is_some() {
            for (lane, byte) in value.to_le_bytes().into_iter().enumerate() {
                let target = self
                    .botwoon_owned_field_mut(enemy, field)
                    .expect("same Botwoon field");
                let mut bytes = target.to_le_bytes();
                bytes[lane] = byte;
                *target = u16::from_le_bytes(bytes);
                crate::session_access::SessionAccess::refresh_inputs(self);
            }
        } else {
            self.store_actor_layout_word(enemy, botwoon_offset(field), value);
        }
    }
    fn botwoon_history_x(&self, cursor: u16) -> u16 {
        if cursor.is_multiple_of(4) && cursor < 0x400 {
            return self.owner.gameplay.actors.extended.botwoon_history[usize::from(cursor) / 4].x;
        }
        if cursor.is_multiple_of(2) && usize::from(cursor) / 2 + 0 < 1006 {
            *self
                .owner
                .gameplay
                .actors
                .extended
                .word(3, usize::from(cursor) / 2 + 0)
        } else {
            self.current_word(0x9000 + usize::from(cursor) + 0)
        }
    }
    fn set_botwoon_history_x(&mut self, cursor: u16, value: u16) {
        if cursor.is_multiple_of(4) && cursor < 0x400 {
            for (lane, byte) in value.to_le_bytes().into_iter().enumerate() {
                let target = &mut self.owner.gameplay.actors.extended.botwoon_history
                    [usize::from(cursor) / 4]
                    .x;
                let mut bytes = target.to_le_bytes();
                bytes[lane] = byte;
                *target = u16::from_le_bytes(bytes);
                crate::session_access::SessionAccess::refresh_inputs(self);
            }
            return;
        }
        if cursor.is_multiple_of(2) && usize::from(cursor) / 2 + 0 < 1006 {
            self.store_extended_actor_word(3, 0, usize::from(cursor) / 2 + 0, value);
        } else {
            self.write_current_word_le(0x9000 + usize::from(cursor) + 0, value);
        }
    }
    fn botwoon_history_y(&self, cursor: u16) -> u16 {
        if cursor.is_multiple_of(4) && cursor < 0x400 {
            return self.owner.gameplay.actors.extended.botwoon_history[usize::from(cursor) / 4].y;
        }
        if cursor.is_multiple_of(2) && usize::from(cursor) / 2 + 1 < 1006 {
            *self
                .owner
                .gameplay
                .actors
                .extended
                .word(3, usize::from(cursor) / 2 + 1)
        } else {
            self.current_word(0x9000 + usize::from(cursor) + 2)
        }
    }
    fn set_botwoon_history_y(&mut self, cursor: u16, value: u16) {
        if cursor.is_multiple_of(4) && cursor < 0x400 {
            for (lane, byte) in value.to_le_bytes().into_iter().enumerate() {
                let target = &mut self.owner.gameplay.actors.extended.botwoon_history
                    [usize::from(cursor) / 4]
                    .y;
                let mut bytes = target.to_le_bytes();
                bytes[lane] = byte;
                *target = u16::from_le_bytes(bytes);
                crate::session_access::SessionAccess::refresh_inputs(self);
            }
            return;
        }
        if cursor.is_multiple_of(2) && usize::from(cursor) / 2 + 1 < 1006 {
            self.store_extended_actor_word(3, 0, usize::from(cursor) / 2 + 1, value);
        } else {
            self.write_current_word_le(0x9000 + usize::from(cursor) + 2, value);
        }
    }
}

impl PreparedActorState<'_> {
    fn extended_actor_field(&self, block: usize, key: usize, field: usize) -> Option<&u16> {
        if key >= 2048 || !key.is_multiple_of(64) {
            return None;
        }
        let word = key / 2 + field;
        match (block, word) {
            (1, 15) => Some(&self.owner.gameplay.actors.extended.botwoon_head[0].pending_hit),
            (1, 16) => Some(&self.owner.gameplay.actors.extended.botwoon_head[0].active),
            (1, 30) => Some(&self.owner.gameplay.actors.extended.botwoon_head[0].tail_ready),
            (2, 12) => Some(&self.owner.gameplay.actors.extended.botwoon_path[0].previous_health),
            (2, 31) => Some(&self.owner.gameplay.actors.extended.botwoon_path[0].body_landed),
            (3, word) if word >= 1006 => None,
            _ => (block < 6 && word < 1024)
                .then(|| self.owner.gameplay.actors.extended.word(block, word)),
        }
    }
    fn extended_actor_field_mut(
        &mut self,
        block: usize,
        key: usize,
        field: usize,
    ) -> Option<&mut u16> {
        if key >= 2048 || !key.is_multiple_of(64) {
            return None;
        }
        let word = key / 2 + field;
        match (block, word) {
            (1, 15) => Some(&mut self.owner.gameplay.actors.extended.botwoon_head[0].pending_hit),
            (1, 16) => Some(&mut self.owner.gameplay.actors.extended.botwoon_head[0].active),
            (1, 30) => Some(&mut self.owner.gameplay.actors.extended.botwoon_head[0].tail_ready),
            (2, 12) => {
                Some(&mut self.owner.gameplay.actors.extended.botwoon_path[0].previous_health)
            }
            (2, 31) => Some(&mut self.owner.gameplay.actors.extended.botwoon_path[0].body_landed),
            (3, word) if word >= 1006 => None,
            _ => (block < 6 && word < 1024)
                .then(|| self.owner.gameplay.actors.extended.word_mut(block, word)),
        }
    }
    fn extended_actor_word(&self, block: usize, key: usize, field: usize) -> u16 {
        self.extended_actor_field(block, key, field)
            .copied()
            .unwrap_or_else(|| self.current_word(0x7800 + block * 0x800 + key + field * 2))
    }
    fn store_extended_actor_word(&mut self, block: usize, key: usize, field: usize, value: u16) {
        if self.extended_actor_field(block, key, field).is_some() {
            for (lane, byte) in value.to_le_bytes().into_iter().enumerate() {
                let word = self
                    .extended_actor_field_mut(block, key, field)
                    .expect("current typed field");
                let mut bytes = word.to_le_bytes();
                bytes[lane] = byte;
                *word = u16::from_le_bytes(bytes);
                crate::session_access::SessionAccess::refresh_inputs(self);
            }
        } else {
            self.write_current_word_le(0x7800 + block * 0x800 + key + field * 2, value);
        }
    }
    fn actor_layout_word(&self, key: usize, offset: usize) -> u16 {
        if key < 2048
            && key.is_multiple_of(64)
            && (0x30..=0x38).contains(&offset)
            && offset.is_multiple_of(2)
        {
            return self.owner.gameplay.actors.slots[key / 64].variables[(offset - 0x30) / 2];
        }
        if (0x6888..0x8088).contains(&offset) && offset.is_multiple_of(2) {
            let extra = offset - 0x6888;
            return self.extended_actor_word(extra / 0x800, key, (extra % 0x800) / 2);
        }
        self.current_word(0xf78 + key + offset)
    }
    fn store_actor_layout_word(&mut self, key: usize, offset: usize, value: u16) {
        if key < 2048
            && key.is_multiple_of(64)
            && (0x30..=0x38).contains(&offset)
            && offset.is_multiple_of(2)
        {
            for (lane, byte) in value.to_le_bytes().into_iter().enumerate() {
                let word =
                    &mut self.owner.gameplay.actors.slots[key / 64].variables[(offset - 0x30) / 2];
                let mut bytes = word.to_le_bytes();
                bytes[lane] = byte;
                *word = u16::from_le_bytes(bytes);
                crate::session_access::SessionAccess::refresh_inputs(self);
            }
        } else if (0x6888..0x8088).contains(&offset) && offset.is_multiple_of(2) {
            let extra = offset - 0x6888;
            self.store_extended_actor_word(extra / 0x800, key, (extra % 0x800) / 2, value);
        } else {
            self.write_current_word_le(0xf78 + key + offset, value);
        }
    }
}

impl PreparedActorState<'_> {
    fn botwoon_owned_field(&self, enemy: usize, field: BotwoonField) -> Option<&u16> {
        let state = &self.owner.gameplay.actors;
        if matches!(field, BotwoonField::Projectile | BotwoonField::Visible) {
            if enemy > 24 || !enemy.is_multiple_of(2) {
                return None;
            }
            return Some(match field {
                BotwoonField::Projectile => &state.extended.botwoon_body[0].projectiles[enemy / 2],
                BotwoonField::Visible => &state.extended.botwoon_body[0].visibility[enemy / 2],
                _ => unreachable!(),
            });
        }
        if enemy >= 2048 || !enemy.is_multiple_of(64) {
            return None;
        }
        let slot = enemy / 64;
        Some(match field {
            BotwoonField::HistoryCursor => &state.slots[slot].variables[1],
            BotwoonField::HistorySpacing => &state.slots[slot].variables[2],
            BotwoonField::TransitionPending => {
                &state.extended.botwoon_head[slot].hole_collision_disabled
            }
            BotwoonField::TransitionCursor => {
                &state.extended.botwoon_head[slot].target_history_index
            }
            BotwoonField::TailState => &state.extended.botwoon_head[slot].tail_ready,
            BotwoonField::Pattern => &state.extended.botwoon_head[slot].head_hidden,
            BotwoonField::Projectile | BotwoonField::Visible => unreachable!(),
        })
    }
    fn botwoon_owned_field_mut(&mut self, enemy: usize, field: BotwoonField) -> Option<&mut u16> {
        let state = &mut self.owner.gameplay.actors;
        if matches!(field, BotwoonField::Projectile | BotwoonField::Visible) {
            if enemy > 24 || !enemy.is_multiple_of(2) {
                return None;
            }
            return Some(match field {
                BotwoonField::Projectile => {
                    &mut state.extended.botwoon_body[0].projectiles[enemy / 2]
                }
                BotwoonField::Visible => &mut state.extended.botwoon_body[0].visibility[enemy / 2],
                _ => unreachable!(),
            });
        }
        if enemy >= 2048 || !enemy.is_multiple_of(64) {
            return None;
        }
        let slot = enemy / 64;
        Some(match field {
            BotwoonField::HistoryCursor => &mut state.slots[slot].variables[1],
            BotwoonField::HistorySpacing => &mut state.slots[slot].variables[2],
            BotwoonField::TransitionPending => {
                &mut state.extended.botwoon_head[slot].hole_collision_disabled
            }
            BotwoonField::TransitionCursor => {
                &mut state.extended.botwoon_head[slot].target_history_index
            }
            BotwoonField::TailState => &mut state.extended.botwoon_head[slot].tail_ready,
            BotwoonField::Pattern => &mut state.extended.botwoon_head[slot].head_hidden,
            BotwoonField::Projectile | BotwoonField::Visible => unreachable!(),
        })
    }
}
