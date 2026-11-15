//! The camera model held across ordinary following, placements and checkpoints.

use canonical::model::{Model, Pinned};
use world_camera::{Camera, RememberedPosition, ViewPosition};

/// The full clean camera state, pinned for checkpoints.
pub struct CameraPin;

impl Pinned for CameraPin {
    type State = Camera;
    const NAME: &'static str = "world-camera";
    const SHAPE: u64 = 0x1ED5_DF03_BD0A_7BD4;
}

/// Fresh Samus initialization starts all camera quantities at zero.
#[must_use]
pub const fn initialise() -> Model<CameraPin> {
    Model(Camera::initialise(
        ViewPosition(types::Pair {
            x: core::num::Wrapping(0),
            y: core::num::Wrapping(0),
        }),
        RememberedPosition(types::Pair {
            x: core::num::Wrapping(0),
            y: core::num::Wrapping(0),
        }),
    ))
}

#[cfg(test)]
mod tests {
    #[test]
    fn a_checkpoint_keeps_the_receipt_independent_of_position_writes() {
        let mut state = crate::state::State::default();
        state.camera.0 = world_camera::Camera::restore(
            world_camera::ViewPosition(types::Pair {
                x: core::num::Wrapping(0),
                y: core::num::Wrapping(0),
            }),
            world_camera::RememberedPosition(types::Pair {
                x: core::num::Wrapping(0),
                y: core::num::Wrapping(0),
            }),
            types::VerticalFraction(0xBEEF),
        )
        .with_view(world_camera::ViewPosition(types::Pair {
            x: core::num::Wrapping(0x1234_5678),
            y: core::num::Wrapping(0x9012_3456),
        }))
        .with_remembered(world_camera::RememberedPosition(types::Pair {
            x: core::num::Wrapping(0xFEDC_BA98),
            y: core::num::Wrapping(0x7654_3210),
        }));
        let bytes = canonical::to_bytes(&state);
        let restored = canonical::from_bytes::<crate::state::State>(&bytes).unwrap();
        assert_eq!(restored, state);
        assert_eq!(
            restored.camera.0.vertical_fraction(),
            types::VerticalFraction(0xBEEF)
        );
        assert_eq!(
            super::initialise().0.vertical_fraction(),
            types::VerticalFraction::ZERO
        );
    }

    #[test]
    fn the_camera_state_shape_is_pinned() {
        assert_eq!(canonical::model::pin_holds::<super::CameraPin>(), Ok(()));
    }
}
