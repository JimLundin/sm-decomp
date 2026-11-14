//! Accepted ordinary travel retained between the movement and projectile passes.

use canonical::model::Pinned;

/// The caller-held movement interval, pinned for checkpoints.
pub struct TravelPin;

impl Pinned for TravelPin {
    type State = samus_movement::TravelledDistances;
    const NAME: &'static str = "samus-movement-travel";
    const SHAPE: u64 = 0x614F_299D_A431_20A0;
}

#[cfg(test)]
mod tests {
    #[test]
    fn a_checkpoint_preserves_signed_fractional_travel() {
        let mut state = crate::state::State::default();
        for (direction, value) in [
            (types::Direction::Down, 0x0001_2000),
            (types::Direction::Left, 0xFFFF_C000),
            (types::Direction::Right, 0x0003_4000),
            (types::Direction::Up, 0xFFFD_8000),
        ] {
            state.movement_travel.0.travels[direction] =
                types::TravelDistance(core::num::Wrapping(value));
        }
        // enum-map's compact serde tuple uses declaration order: down, left, right, up.
        // Keep the named ordering pinned as well as the tuple's structural shape.
        let interval = canonical::to_bytes(&state.movement_travel);
        assert_eq!(
            interval,
            [
                16, 0, 0, 0, 0, 32, 1, 0, 0, 192, 255, 255, 0, 64, 3, 0, 0, 128, 253, 255
            ]
        );
        let saved = canonical::to_bytes(&state);
        assert_eq!(
            canonical::from_bytes::<crate::state::State>(&saved).as_ref(),
            Ok(&state)
        );
    }

    #[test]
    fn the_travel_state_shape_is_pinned() {
        assert_eq!(canonical::model::pin_holds::<super::TravelPin>(), Ok(()));
    }
}
