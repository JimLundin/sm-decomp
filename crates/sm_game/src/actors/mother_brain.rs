//! Mother Brain's two slots: the body in slot 0 and the brain in slot $40.
//!
//! Was `Enemy_MotherBrain` (`EnemyLayout72`) in the original decompilation.
//! Each view names its slot words over the same RAM bytes, after the
//! Mother Brain notes in the bank logs. The body's extra words are the
//! fight-wide state at $7E:7800 and $7E:8000; the brain's are at $7E:7840 and
//! $7E:8040. Unnamed words keep their layout names.
#![forbid(unsafe_code)]
use crate::actors::layouts::part_view;

part_view! {
    /// The body's view of slot 0: the fight's phase and pose, its attacks,
    /// the room and brain palettes, the brain's instruction list and the
    /// rainbow beam.
    MotherBrainBody: EnemyLayout72 {
        words: [
            // $0FA8: the body's bank $A9 function.
            function,
            var_B,
            // $0FAC: the zebetites' palette animation index.
            zebetite_palette_index,
            // $0FAE: brain palette counter (low byte) and timer (high byte).
            brain_palette_counter,
            // $0FB0: brain palette timer reload (low byte) and palette set
            // (high byte: 0 normal, 2 charging the laser).
            brain_palette_reload,
            // $0FB2: the body function's timer; the bomb's target X.
            function_timer,
            // $0FB4: attack cooldown; rainbow beam explosion index; Samus's
            // custom X velocity.
            cooldown,
            // $0FB6: rainbow beam explosion timer; Samus's custom Y velocity.
            explosion_timer,
        ],
        pad1: [u8; 26696],
        words: [
            // $7800: first phase, fake death, second phase, drained, third phase.
            form,
            // $7802: the brain's initial instruction list.
            initial_brain_instructions,
            // $7804: standing, walking, crouching, hand beam or leaning.
            pose,
            var_03,
            // $7808: which of the body, brain and neck hitboxes are enabled.
            hitboxes,
            var_05,
            // $780C: disables the third phase's attacks.
            attacks_disabled,
            // $780E: counts walks to choose the walking direction.
            walk_counter,
            var_08,
            // $7812: the rising and rainbow beam HDMA object.
            hdma_object,
            // $7814, $7816: where the neck starts, from the body's position.
            neck_base_x,
            neck_base_y,
            // $7818, $781A: neck and brain palette indices.
            neck_palette,
            brain_palette,
            // $781C, $781E: the room palette flash's instruction list and timer.
            room_palette_list,
            room_palette_timer,
            var_10,
            var_11,
            var_12,
            // $7826: blue ring volleys fired at the Shitroid while it refills.
            ring_volleys,
            // $7828: play the Shitroid's cry.
            shitroid_cry,
            // $782A: rainbow beam sounds still to queue.
            beam_sounds_left,
            // $782C: the rainbow beam sound is playing (checked on unpause).
            beam_sound_playing,
            // $782E: the hand beam attack's phase.
            hand_beam_phase,
            // $7830: try, cooldown or end.
            attack_phase,
            var_19,
            // $7834: the blue rings' target angle.
            ring_target_angle,
            var_1B,
            var_1C,
            // $783A: delete the turrets and rinkas once the first phase ends.
            delete_turrets,
            // $783C: active, moving rinkas.
            rinkas,
            // $783E: initial, corpse, or recovered (phase 2).
            corpse_state,
        ],
        pad2: [u8; 1984],
        words: [
            // $8000, $8002: the brain's instruction timer and list.
            brain_instruction_timer,
            brain_instructions,
            // $8004: the sprite tile transfer entry.
            sprite_tile_transfer,
            // $8006-$8012: the hand beam's next position, velocity and angle.
            hand_beam_x_subpos,
            hand_beam_x,
            hand_beam_y_subpos,
            hand_beam_y,
            hand_beam_x_speed,
            hand_beam_y_speed,
            hand_beam_angle,
            var_2A,
            var_2B,
            var_2C,
            var_2D,
            var_2E,
            var_2F,
            var_30,
            // $8022: the rainbow beam's angle.
            beam_angle,
            var_32,
            // $8026: the rainbow beam's width, widened up to a cap.
            beam_width,
            var_34,
            var_35,
            var_36,
            // $802E: the grey and room-lights transition counter.
            grey_transition,
            // $8030, $8032: the fake death's explosion timer and index.
            fake_death_explosion_timer,
            fake_death_explosion_index,
            // $8034-$803E: the rainbow beam's edge angles and origins.
            beam_right_edge,
            beam_left_edge,
            beam_left_origin_x,
            beam_left_origin_y,
            beam_right_origin_x,
            beam_right_origin_y,
        ],
    }
}

part_view! {
    /// The brain's view of slot $40: its shake, the painful walk, the
    /// Shitroid, palettes, drool and breath, the third phase's neck and walk,
    /// and the neck's joints and segments.
    MotherBrainBrain: EnemyLayout72 {
        words: [
            // $0FE8: the brain's bank $A9 function.
            function,
            var_B,
            var_C,
            var_D,
            // $0FF0: a sub-step (the fake death's tube collapse, the death and
            // escape door explosion interval).
            subfunction,
            // $0FF2: the sub-step's timer; the death explosion and escape
            // door indices.
            subfunction_timer,
            parameter_1,
            parameter_2,
        ],
        pad1: [u8; 26696],
        words: [
            // $7840: the brain's main shake timer.
            shake_timer,
            // $7842: the body's rainbow beam palette animation index.
            beam_palette_index,
            // $7844: enables the unpause hook.
            unpause_hook,
            var_03,
            var_04,
            // $784A: bombs out.
            bombs,
            // $784C-$7852: the painful walk's stage (0-7, slower as it rises),
            // animation delay, function and function timer.
            painful_walk_stage,
            painful_walk_delay,
            painful_walk_function,
            painful_walk_timer,
            // $7854: the Shitroid's enemy index.
            shitroid,
            // $7E:7856: set when the Shitroid lets Samus go (never read).
            shitroid_released,
            var_0C,
            var_0D,
            var_0E,
            var_0F,
            // $7860, $7862: enable the brain and health-based palettes.
            brain_palette_enabled,
            health_palette_enabled,
            // $7864, $7866: enable drool, and its projectile parameter (0-5).
            drool_enabled,
            drool_parameter,
            // $7868, $786A: enable the small purple breath; it is active.
            breath_enabled,
            breath_active,
            // $786C: Samus's rainbow palette function.
            samus_rainbow_function,
            var_17,
            // $7870, $7872: the third phase's neck function and its timer.
            neck_function,
            neck_timer,
            // $7874, $7876: the third phase's walk function and target X.
            walk_function,
            target_x,
            // $7878: the third phase's crouch timer.
            crouch_timer,
            // $787A: Samus's rainbow palette animation counter.
            samus_rainbow_counter,
            var_1E,
            var_1F,
        ],
        pad2: [u8; 1984],
        words: [
            // $8040, $8042: the lower (segments 1-3) and upper (4-5) neck
            // angles; the joint's direction is in the high byte.
            lower_angle,
            upper_angle,
            // $8044-$8060: each neck segment's position and distance.
            segment_1_x,
            segment_1_y,
            segment_1_length,
            segment_2_x,
            segment_2_y,
            segment_2_length,
            segment_3_x,
            segment_3_y,
            segment_3_length,
            segment_4_x,
            segment_4_y,
            segment_4_length,
            segment_5_x,
            segment_5_y,
            segment_5_length,
            // $8062-$8068: neck movement on, the lower and upper motions, and
            // the angle step.
            neck_moving,
            lower_motion,
            upper_motion,
            neck_speed,
            var_35,
            var_36,
            var_37,
            var_38,
            var_39,
            var_3A,
            var_3B,
            var_3C,
            var_3D,
            var_3E,
            var_3F,
        ],
    }
}

/// Mother Brain's body slot and brain slot.
pub const BODY: i32 = 0;
pub const BRAIN: i32 = 0x40;
