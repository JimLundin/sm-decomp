//! Caller-held transition firing aim between POST and PRE.

use canonical::model::Pinned;

/// The optional named direction, pinned for checkpoints.
pub struct PendingAimPin;

impl Pinned for PendingAimPin {
    type State = Option<types::FiringDirection>;
    const NAME: &'static str = "samus-pending-aim";
    const SHAPE: u64 = 0x042D_D698_A125_2F27;
}

#[cfg(test)]
mod tests {
    #[test]
    fn the_pending_direction_shape_is_pinned() {
        assert_eq!(
            canonical::model::pin_holds::<super::PendingAimPin>(),
            Ok(())
        );
    }

    #[test]
    fn a_checkpoint_preserves_vertical_facing_and_physical_press_history() {
        let mut state = crate::state::State::default();
        state.pending_aim.0 = Some(types::FiringDirection::UpFacingLeft);
        state
            .controller
            .prior_pressed
            .insert(crate::state::Button::X);
        let saved = canonical::to_bytes(&state);
        assert_eq!(
            canonical::from_bytes::<crate::state::State>(&saved).as_ref(),
            Ok(&state)
        );
    }
}
