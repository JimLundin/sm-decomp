//! Named words a Session owner already projects, for code outside a lease.
//!
//! Each [`Word`] is the RAM address of a word whose two bytes
//! `session_encoding` projects from the Session field named on the variant.
//! Outside a lease the host's projected bytes are that state, so [`get`] and
//! [`set`] read and store through `crate::projection`, low byte then high, as
//! the named word accesses did. A lease host serves the same bytes from its
//! Session. The layout tests check that every word is projected.
use crate::ram::CompatibilityRam;

macro_rules! session_words {
    ($($(#[$doc:meta])* $word:ident = $at:literal, $ram:ident;)+) => {
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
    };
}

session_words! {
    /// `feedback.sound_disabled`.
    DebugDisableSounds = 0x05f5, DEBUG_DISABLE_SOUNDS;
    /// `room.ceres_status`.
    CeresStatus = 0x093f, CERES_STATUS;
    /// `timing.timer_status`.
    TimerStatus = 0x0943, TIMER_STATUS;
    /// `projectiles.controls.selected_weapon`.
    HudItemIndex = 0x09d2, HUD_ITEM_INDEX;
    /// `options.japanese_text`.
    JapaneseTextFlag = 0x09e2, JAPANESE_TEXT_FLAG;
    /// `projectiles.controls.automatic_weapon`.
    SamusAutoCancelHudItemIndex = 0x0a04, SAMUS_AUTO_CANCEL_HUD_ITEM_INDEX;
    /// `player.pose`.
    SamusPose = 0x0a1c, SAMUS_POSE;
    /// The presentation palette's `suit_index`.
    SamusSuitPaletteIndex = 0x0a74, SAMUS_SUIT_PALETTE_INDEX;
    /// `timing.time_frozen`.
    TimeIsFrozenFlag = 0x0a78, TIME_IS_FROZEN_FLAG;
    /// `player.transitions.elevator_fade_timer`.
    ReachedCeresElevatorFadeTimer = 0x0aa0, REACHED_CERES_ELEVATOR_FADE_TIMER;
    /// `player.contact.y_speed.fraction`.
    SamusYSubspeed = 0x0b2c, SAMUS_Y_SUBSPEED;
    /// `player.contact.y_speed.whole`.
    SamusYSpeed = 0x0b2e, SAMUS_Y_SPEED;
    /// `projectiles.controls.charge`.
    FlareCounter = 0x0cd0, FLARE_COUNTER;
    /// `player.swing.start_animation_timer`.
    GrapplePointAnimTimer = 0x0d3e, GRAPPLE_POINT_ANIM_TIMER;
    /// `player.swing.start_animation_tiles`.
    GrapplePointAnimPtr = 0x0d40, GRAPPLE_POINT_ANIM_PTR;
    /// `messages.queued`.
    QueuedMessageBoxIndex = 0x0dc8, QUEUED_MESSAGE_BOX_INDEX;
    /// `shared_sequence.stage`.
    Substate = 0x0dec, SUBSTATE;
    /// `actors.diagnostics.sprite_contacts_disabled`.
    DebugDisableSpriteInteract = 0x0e12, DEBUG_DISABLE_SPRITE_INTERACT;
    /// `room.fx_blending`.
    FxLayerBlendingConfigA = 0x1982, FX_LAYER_BLENDING_CONFIG_A;
    /// `player.invincibility`.
    SamusInvincibilityTimer = 0x18a8, SAMUS_INVINCIBILITY_TIMER;
    /// `player.knockback_timer`.
    SamusKnockbackTimer = 0x18aa, SAMUS_KNOCKBACK_TIMER;
    /// The presentation's palette fade `step`.
    PaletteChangeNum = 0xc400, PALETTE_CHANGE_NUM;
}

/// `word` as the host's projection holds it, low byte then high.
pub fn get<H: CompatibilityRam + ?Sized>(h: &H, word: Word) -> u16 {
    crate::projection::word(h, word as usize)
}

/// Store `value` to `word`, low byte then high, as the named word store did.
pub fn set<H: CompatibilityRam + ?Sized>(h: &mut H, word: Word, value: u16) {
    crate::projection::set_word(h, word as usize, value);
}
