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
pub fn current<H: crate::ram::CompatibilityRam + ?Sized>(game: &H) -> SessionWords {
    crate::session_cohort::load(game)
}
