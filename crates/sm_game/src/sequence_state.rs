//! Flow and shared sequence controls, with one owner per execution interval.
#![forbid(unsafe_code)]
use crate::{
    flow::Flow,
    session_words::{SessionWords, Word},
};

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct State {
    pub flow: Flow,
    pub words: SessionWords,
}
impl State {
    pub fn from_session(session: &crate::session::Session) -> Self {
        Self {
            flow: session.flow,
            words: SessionWords::from_session(session),
        }
    }
}
/// The adapter may publish the selected image, suspend, and revoke this owner.
/// InSession carries no writable duplicate of the Session fields.
pub enum Domain {
    Unowned,
    Owned { state: State, epoch: u64 },
    InSession { epoch: u64 },
    Leased,
    Revoked,
}
pub trait Access {
    fn sequence_flow(&self) -> Flow;
    fn set_cinematic_function(&mut self, value: u16);
    fn sequence_words(&self) -> SessionWords;
    fn commit_flow(&mut self, before: &Flow, after: Flow);
    fn commit_sequence_words(&mut self, before: &SessionWords, after: SessionWords);
    /// Direct stores retain their original byte publication, even if unchanged.
    fn set_sequence_word(&mut self, word: Word, value: u16);
}
/// Preserve the old leaf's copy rollback: panic never publishes its edits.
pub fn with_flow<H: Access + ?Sized, T>(
    host: &mut H,
    action: impl FnOnce(&mut H, &mut Flow) -> T,
) -> T {
    let before = host.sequence_flow();
    let mut after = before;
    let result = action(host, &mut after);
    if before != after {
        host.commit_flow(&before, after);
    }
    result
}
pub fn with_words<H: Access + ?Sized, T>(
    host: &mut H,
    action: impl FnOnce(&mut H, &mut SessionWords) -> T,
) -> T {
    let before = host.sequence_words();
    let mut after = before;
    let result = action(host, &mut after);
    if before != after {
        host.commit_sequence_words(&before, after);
    }
    result
}
pub fn read<H: Access + ?Sized>(host: &H, word: Word) -> u16 {
    host.sequence_words().get(word)
}
pub fn write<H: Access + ?Sized>(host: &mut H, word: Word, value: u16) {
    host.set_sequence_word(word, value);
}

#[macro_export]
macro_rules! sequence_word {
    ($game:ident.$field:ident = $value:expr $(,)?) => {{
        let value: u16 = $value;
        $crate::sequence_state::write($game, $crate::sequence_word!(@field $field), value)
    }};
    ($game:ident.$field:ident, |$old:ident| $value:expr, reread $(,)?) => {{
        $crate::sequence_word!($game.$field, |$old| $value);
        $crate::sequence_word!($game.$field)
    }};
    ($game:ident.$field:ident, |$old:ident| $value:expr $(,)?) => {{
        let $old = $crate::sequence_word!($game.$field);
        let value: u16 = $value;
        $crate::sequence_state::write($game, $crate::sequence_word!(@field $field), value)
    }};
    ($game:ident.$field:ident $op:tt $value:expr $(, $reread:ident)? $(,)?) => {
        $crate::sequence_word!($game.$field, |old| $crate::sequence_word!(@op old $op $value) $(, $reread)?)
    };
    ($game:ident.$field:ident) => {
        $crate::sequence_state::read(&*$game, $crate::sequence_word!(@field $field))
    };
    (@op $old:ident += $value:expr) => { $old.wrapping_add($value) };
    (@op $old:ident -= $value:expr) => { $old.wrapping_sub($value) };
    (@op $old:ident &= $value:expr) => { $old & $value };
    (@op $old:ident |= $value:expr) => { $old | $value };
    (@op $old:ident ^= $value:expr) => { $old ^ $value };
    (@field DEBUG_DISABLE_SOUNDS) => { $crate::session_words::Word::DebugDisableSounds };
    (@field LAYER1_X_POS) => { $crate::session_words::Word::Layer1XPos };
    (@field LAYER1_Y_POS) => { $crate::session_words::Word::Layer1YPos };
    (@field CERES_STATUS) => { $crate::session_words::Word::CeresStatus };
    (@field TIMER_STATUS) => { $crate::session_words::Word::TimerStatus };
    (@field HUD_ITEM_INDEX) => { $crate::session_words::Word::HudItemIndex };
    (@field JAPANESE_TEXT_FLAG) => { $crate::session_words::Word::JapaneseTextFlag };
    (@field SAMUS_AUTO_CANCEL_HUD_ITEM_INDEX) => { $crate::session_words::Word::SamusAutoCancelHudItemIndex };
    (@field SAMUS_POSE) => { $crate::session_words::Word::SamusPose };
    (@field SAMUS_SUIT_PALETTE_INDEX) => { $crate::session_words::Word::SamusSuitPaletteIndex };
    (@field TIME_IS_FROZEN_FLAG) => { $crate::session_words::Word::TimeIsFrozenFlag };
    (@field KNOCKBACK_X_DIR) => { $crate::session_words::Word::KnockbackXDir };
    (@field REACHED_CERES_ELEVATOR_FADE_TIMER) => { $crate::session_words::Word::ReachedCeresElevatorFadeTimer };
    (@field SAMUS_Y_SUBSPEED) => { $crate::session_words::Word::SamusYSubspeed };
    (@field SAMUS_Y_SPEED) => { $crate::session_words::Word::SamusYSpeed };
    (@field FLARE_COUNTER) => { $crate::session_words::Word::FlareCounter };
    (@field GRAPPLE_POINT_ANIM_TIMER) => { $crate::session_words::Word::GrapplePointAnimTimer };
    (@field GRAPPLE_POINT_ANIM_PTR) => { $crate::session_words::Word::GrapplePointAnimPtr };
    (@field QUEUED_MESSAGE_BOX_INDEX) => { $crate::session_words::Word::QueuedMessageBoxIndex };
    (@field SUBSTATE) => { $crate::session_words::Word::Substate };
    (@field SUIT_PICKUP_LIGHT_BEAM_POS) => { $crate::session_words::Word::SuitPickupLightBeamPos };
    (@field DEBUG_DISABLE_SPRITE_INTERACT) => { $crate::session_words::Word::DebugDisableSpriteInteract };
    (@field FX_LAYER_BLENDING_CONFIG_A) => { $crate::session_words::Word::FxLayerBlendingConfigA };
    (@field SAMUS_INVINCIBILITY_TIMER) => { $crate::session_words::Word::SamusInvincibilityTimer };
    (@field SAMUS_KNOCKBACK_TIMER) => { $crate::session_words::Word::SamusKnockbackTimer };
    (@field PALETTE_CHANGE_NUM) => { $crate::session_words::Word::PaletteChangeNum };
}

/// The former with_projected path commits changed fields directly, without
/// opening a lease. Each low/high publication and failure prefix stays ordered.
pub fn with_words_projected<H: Access + ?Sized, T>(
    host: &mut H,
    action: impl FnOnce(&mut H, &mut SessionWords) -> T,
) -> T {
    let before = host.sequence_words();
    let mut after = before;
    let result = action(host, &mut after);
    for &word in Word::ALL {
        let value = after.get(word);
        if value != before.get(word) {
            host.set_sequence_word(word, value);
        }
    }
    result
}
