//! Checkpoint and placement contracts for Spec: `RememberedHeightFraction` and Remembering.

#[cfg(test)]
mod tests {
    use core::num::Wrapping;
    use types::{Pair, VerticalFraction};
    use world_camera::{Camera, RememberedPosition, ViewPosition};

    #[test]
    fn every_receipt_can_be_restored_and_survives_position_placement() {
        let view = ViewPosition(Pair {
            x: Wrapping(123),
            y: Wrapping(456),
        });
        let seen = RememberedPosition(Pair {
            x: Wrapping(789),
            y: Wrapping(321),
        });
        for count in 0..=u16::MAX {
            let camera = Camera::restore(view, seen, VerticalFraction(count));
            let placed = camera
                .with_view(ViewPosition(seen.0))
                .with_remembered(RememberedPosition(view.0));
            assert_eq!(placed.vertical_fraction(), VerticalFraction(count));
            assert_eq!(placed.position(), seen.0);
            assert_eq!(placed.seen(), view.0);
        }
    }

    #[test]
    fn checkpoint_preserves_all_three_distinct_quantities() {
        let camera = Camera::restore(
            ViewPosition(Pair {
                x: Wrapping(123),
                y: Wrapping(456),
            }),
            RememberedPosition(Pair {
                x: Wrapping(789),
                y: Wrapping(321),
            }),
            VerticalFraction(u16::MAX),
        );
        let checkpoint = toml::to_string(&camera).unwrap();
        let resumed: Camera = toml::from_str(&checkpoint).unwrap();
        assert_eq!(camera, resumed);
        assert_eq!(resumed.vertical_fraction(), VerticalFraction(u16::MAX));
        let fresh = Camera::initialise(
            ViewPosition(camera.position()),
            RememberedPosition(camera.seen()),
        );
        assert_eq!(fresh.vertical_fraction(), VerticalFraction::ZERO);
    }
}
