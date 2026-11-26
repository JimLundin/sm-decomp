//! Spec: `ShellScatters`. The shell breaks and its fragments arc away.

use types::Pair;
use types::discovery::{FlyingFragment, Fragments, PieceId};

use crate::scene::FragmentContent;

/// Spec: `ShellScatters`. The fragment cohort placed at its anchors, before any flight.
#[must_use]
pub fn scatter(cohort: &[FragmentContent]) -> Fragments {
    Fragments::gathered(
        (0..=u8::MAX)
            .zip(cohort)
            .map(|(identity, content)| FlyingFragment {
                identity: PieceId(identity),
                place: content.anchor,
                steps: 0,
            }),
    )
}

/// Spec: `ShellScatters`. One finished frame of every fragment's flight.
pub fn fly(fragments: &mut Fragments, cohort: &[FragmentContent], floor_y: i32) {
    fragments.update(|fragment| {
        let content = cohort.get(usize::from(fragment.identity.0))?;
        flown(fragment, content, floor_y)
    });
}

/// Spec: `ShellScatters`. A fragment one frame further: moved by its drift and its next vertical
/// displacement, and kept while its whole-pixel y is above the floor. A fragment whose track is
/// exhausted, or which reaches the floor, is removed before the draw.
fn flown(
    fragment: FlyingFragment,
    content: &FragmentContent,
    floor_y: i32,
) -> Option<FlyingFragment> {
    let fall = content.track.at(fragment.steps)?;
    let place = Pair {
        x: fragment.place.x.moved(content.drift),
        y: fragment.place.y.moved(fall),
    };
    (place.y.whole() < floor_y).then_some(FlyingFragment {
        place,
        steps: fragment.steps.saturating_add(1),
        ..fragment
    })
}
