//! Named words a Session owner already projects, for code outside a lease.
//!
//! Each [`Word`] is the RAM address of a word whose two bytes
//! `session_encoding` projects from the Session field named on the variant.
//! Code outside a lease reads and stores them as the fields of a
//! [`SessionWords`] copy, through [`crate::session_cohort`]: loaded from the
//! host's projection, and the words it changed stored back once. The layout
//! tests check that every word is projected; the cohort test that each field
//! is the one projected at its address.
use crate::session::Session;

macro_rules! session_words {
    ($($(#[$doc:meta])* $word:ident = $at:literal, $ram:ident, $field:ident: |$s:ident| $path:expr;)+) => {
        /// A Session-owned word, by its RAM address.
        #[derive(Clone, Copy, Debug, PartialEq, Eq)]
        pub enum Word {
            $($(#[$doc])* $word = $at,)+
        }

        impl Word {
            /// Every word, in declaration order.
            pub const ALL: &[Word] = &[$(Word::$word),+];
        }

        const _: () = {
            use crate::ram::words as ram;
            $(assert!(ram::$ram == Word::$word as usize);)+
        };

        /// The Session-owned words as a typed copy, by their RAM names.
        #[derive(Clone, Copy, Default, Debug, PartialEq, Eq)]
        pub struct SessionWords {
            $($(#[$doc])* pub $field: u16,)+
        }


        impl SessionWords {
            pub fn apply_changed(self, before: &Self, session: &mut Session) {
                $({
                    if self.$field != before.$field {
                        let $s = &mut *session;
                        $path = self.$field;
                    }
                })+
            }
            pub fn get(self, word: Word) -> u16 {
                match word { $(Word::$word => self.$field,)+ }
            }
            pub fn set(&mut self, word: Word, value: u16) {
                match word { $(Word::$word => self.$field = value,)+ }
            }
        }

        impl crate::session_cohort::Cohort for SessionWords {
            fn words(&mut self, mut visit: impl FnMut(usize, &mut u16)) {
                $(visit($at, &mut self.$field);)+
            }
            fn session_words(session: &mut Session, mut visit: impl FnMut(usize, &mut u16)) {
                $({
                    let $s = &mut *session;
                    visit($at, &mut $path);
                })+
            }
        }
    };
}

session_words! {
    /// `feedback.sound_disabled`.
    DebugDisableSounds = 0x05f5, DEBUG_DISABLE_SOUNDS, debug_disable_sounds: |s| s.feedback.sound_disabled;
    /// `room.camera.layer1[0]`.
    Layer1XPos = 0x0911, LAYER1_X_POS, layer1_x_pos: |s| s.room.camera.layer1[0];
    /// `room.camera.layer1[1]`.
    Layer1YPos = 0x0915, LAYER1_Y_POS, layer1_y_pos: |s| s.room.camera.layer1[1];
    /// `room.ceres_status`.
    CeresStatus = 0x093f, CERES_STATUS, ceres_status: |s| s.room.ceres_status;
    /// `timing.timer_status`.
    TimerStatus = 0x0943, TIMER_STATUS, timer_status: |s| s.timing.timer_status;
    /// `projectiles.controls.selected_weapon`.
    HudItemIndex = 0x09d2, HUD_ITEM_INDEX, hud_item_index: |s| s.projectiles.controls.selected_weapon;
    /// `options.japanese_text`.
    JapaneseTextFlag = 0x09e2, JAPANESE_TEXT_FLAG, japanese_text_flag: |s| s.options.japanese_text;
    /// `projectiles.controls.automatic_weapon`.
    SamusAutoCancelHudItemIndex = 0x0a04, SAMUS_AUTO_CANCEL_HUD_ITEM_INDEX, samus_auto_cancel_hud_item_index: |s| s.projectiles.controls.automatic_weapon;
    /// `player.pose`.
    SamusPose = 0x0a1c, SAMUS_POSE, samus_pose: |s| s.player.pose;
    /// The presentation palette's `suit_index`.
    SamusSuitPaletteIndex = 0x0a74, SAMUS_SUIT_PALETTE_INDEX, samus_suit_palette_index: |s| s.presentation.palette_mut().suit_index;
    /// `timing.time_frozen`.
    TimeIsFrozenFlag = 0x0a78, TIME_IS_FROZEN_FLAG, time_is_frozen_flag: |s| s.timing.time_frozen;
    /// `player.knockback_x_direction`.
    KnockbackXDir = 0x0a54, KNOCKBACK_X_DIR, knockback_x_dir: |s| s.player.knockback_x_direction;
    /// `player.transitions.elevator_fade_timer`.
    ReachedCeresElevatorFadeTimer = 0x0aa0, REACHED_CERES_ELEVATOR_FADE_TIMER, reached_ceres_elevator_fade_timer: |s| s.player.transitions.elevator_fade_timer;
    /// `player.contact.y_speed.fraction`.
    SamusYSubspeed = 0x0b2c, SAMUS_Y_SUBSPEED, samus_y_subspeed: |s| s.player.contact.y_speed.fraction;
    /// `player.contact.y_speed.whole`.
    SamusYSpeed = 0x0b2e, SAMUS_Y_SPEED, samus_y_speed: |s| s.player.contact.y_speed.whole;
    /// `projectiles.controls.charge`.
    FlareCounter = 0x0cd0, FLARE_COUNTER, flare_counter: |s| s.projectiles.controls.charge;
    /// `player.swing.start_animation_timer`.
    GrapplePointAnimTimer = 0x0d3e, GRAPPLE_POINT_ANIM_TIMER, grapple_point_anim_timer: |s| s.player.swing.start_animation_timer;
    /// `player.swing.start_animation_tiles`.
    GrapplePointAnimPtr = 0x0d40, GRAPPLE_POINT_ANIM_PTR, grapple_point_anim_ptr: |s| s.player.swing.start_animation_tiles;
    /// `messages.queued`.
    QueuedMessageBoxIndex = 0x0dc8, QUEUED_MESSAGE_BOX_INDEX, queued_message_box_index: |s| s.messages.queued;
    /// `shared_sequence.stage`.
    Substate = 0x0dec, SUBSTATE, substate: |s| s.shared_sequence.stage;
    /// `shared_sequence.value`.
    SuitPickupLightBeamPos = 0x0dee, SUIT_PICKUP_LIGHT_BEAM_POS, suit_pickup_light_beam_pos: |s| s.shared_sequence.value;
    /// `actors.diagnostics.sprite_contacts_disabled`.
    DebugDisableSpriteInteract = 0x0e12, DEBUG_DISABLE_SPRITE_INTERACT, debug_disable_sprite_interact: |s| s.actors.diagnostics.sprite_contacts_disabled;
    /// `room.fx_blending`.
    FxLayerBlendingConfigA = 0x1982, FX_LAYER_BLENDING_CONFIG_A, fx_layer_blending_config_a: |s| s.room.fx_blending;
    /// `player.invincibility`.
    SamusInvincibilityTimer = 0x18a8, SAMUS_INVINCIBILITY_TIMER, samus_invincibility_timer: |s| s.player.invincibility;
    /// `player.knockback_timer`.
    SamusKnockbackTimer = 0x18aa, SAMUS_KNOCKBACK_TIMER, samus_knockback_timer: |s| s.player.knockback_timer;
    /// The presentation's palette fade `step`.
    PaletteChangeNum = 0xc400, PALETTE_CHANGE_NUM, palette_change_num: |s| s.presentation.visual_mut().palette_fade.step;
}

/// The Session-owned words as the host's projection holds them.
pub fn current<H: crate::sequence_state::Access + ?Sized>(game: &H) -> SessionWords {
    game.sequence_words()
}

impl SessionWords {
    pub fn from_session(s: &Session) -> Self {
        Self {
            debug_disable_sounds: s.feedback.sound_disabled,
            layer1_x_pos: s.room.camera.layer1[0],
            layer1_y_pos: s.room.camera.layer1[1],
            ceres_status: s.room.ceres_status,
            timer_status: s.timing.timer_status,
            hud_item_index: s.projectiles.controls.selected_weapon,
            japanese_text_flag: s.options.japanese_text,
            samus_auto_cancel_hud_item_index: s.projectiles.controls.automatic_weapon,
            samus_pose: s.player.pose,
            samus_suit_palette_index: s.presentation.palette().suit_index,
            time_is_frozen_flag: s.timing.time_frozen,
            knockback_x_dir: s.player.knockback_x_direction,
            reached_ceres_elevator_fade_timer: s.player.transitions.elevator_fade_timer,
            samus_y_subspeed: s.player.contact.y_speed.fraction,
            samus_y_speed: s.player.contact.y_speed.whole,
            flare_counter: s.projectiles.controls.charge,
            grapple_point_anim_timer: s.player.swing.start_animation_timer,
            grapple_point_anim_ptr: s.player.swing.start_animation_tiles,
            queued_message_box_index: s.messages.queued,
            substate: s.shared_sequence.stage,
            suit_pickup_light_beam_pos: s.shared_sequence.value,
            debug_disable_sprite_interact: s.actors.diagnostics.sprite_contacts_disabled,
            fx_layer_blending_config_a: s.room.fx_blending,
            samus_invincibility_timer: s.player.invincibility,
            samus_knockback_timer: s.player.knockback_timer,
            palette_change_num: s.presentation.visual().palette_fade.step,
        }
    }
}

impl SessionWords {
    pub fn changes(self, before: &Self) -> Vec<(usize, u16)> {
        let mut changed = Vec::new();
        if self.debug_disable_sounds != before.debug_disable_sounds {
            changed.push((0x5f5, self.debug_disable_sounds));
        }
        if self.layer1_x_pos != before.layer1_x_pos {
            changed.push((0x911, self.layer1_x_pos));
        }
        if self.layer1_y_pos != before.layer1_y_pos {
            changed.push((0x915, self.layer1_y_pos));
        }
        if self.ceres_status != before.ceres_status {
            changed.push((0x93f, self.ceres_status));
        }
        if self.timer_status != before.timer_status {
            changed.push((0x943, self.timer_status));
        }
        if self.hud_item_index != before.hud_item_index {
            changed.push((0x9d2, self.hud_item_index));
        }
        if self.japanese_text_flag != before.japanese_text_flag {
            changed.push((0x9e2, self.japanese_text_flag));
        }
        if self.samus_auto_cancel_hud_item_index != before.samus_auto_cancel_hud_item_index {
            changed.push((0xa04, self.samus_auto_cancel_hud_item_index));
        }
        if self.samus_pose != before.samus_pose {
            changed.push((0xa1c, self.samus_pose));
        }
        if self.samus_suit_palette_index != before.samus_suit_palette_index {
            changed.push((0xa74, self.samus_suit_palette_index));
        }
        if self.time_is_frozen_flag != before.time_is_frozen_flag {
            changed.push((0xa78, self.time_is_frozen_flag));
        }
        if self.knockback_x_dir != before.knockback_x_dir {
            changed.push((0xa54, self.knockback_x_dir));
        }
        if self.reached_ceres_elevator_fade_timer != before.reached_ceres_elevator_fade_timer {
            changed.push((0xaa0, self.reached_ceres_elevator_fade_timer));
        }
        if self.samus_y_subspeed != before.samus_y_subspeed {
            changed.push((0xb2c, self.samus_y_subspeed));
        }
        if self.samus_y_speed != before.samus_y_speed {
            changed.push((0xb2e, self.samus_y_speed));
        }
        if self.flare_counter != before.flare_counter {
            changed.push((0xcd0, self.flare_counter));
        }
        if self.grapple_point_anim_timer != before.grapple_point_anim_timer {
            changed.push((0xd3e, self.grapple_point_anim_timer));
        }
        if self.grapple_point_anim_ptr != before.grapple_point_anim_ptr {
            changed.push((0xd40, self.grapple_point_anim_ptr));
        }
        if self.queued_message_box_index != before.queued_message_box_index {
            changed.push((0xdc8, self.queued_message_box_index));
        }
        if self.substate != before.substate {
            changed.push((0xdec, self.substate));
        }
        if self.suit_pickup_light_beam_pos != before.suit_pickup_light_beam_pos {
            changed.push((0xdee, self.suit_pickup_light_beam_pos));
        }
        if self.debug_disable_sprite_interact != before.debug_disable_sprite_interact {
            changed.push((0xe12, self.debug_disable_sprite_interact));
        }
        if self.fx_layer_blending_config_a != before.fx_layer_blending_config_a {
            changed.push((0x1982, self.fx_layer_blending_config_a));
        }
        if self.samus_invincibility_timer != before.samus_invincibility_timer {
            changed.push((0x18a8, self.samus_invincibility_timer));
        }
        if self.samus_knockback_timer != before.samus_knockback_timer {
            changed.push((0x18aa, self.samus_knockback_timer));
        }
        if self.palette_change_num != before.palette_change_num {
            changed.push((0xc400, self.palette_change_num));
        }
        changed
    }
}
