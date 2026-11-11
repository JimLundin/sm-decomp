use std::path::Path;

use super::{Movie, Timeline, Track, percent, runs};

fn root() -> &'static Path {
    Path::new(concat!(env!("CARGO_MANIFEST_DIR"), "/../.."))
}

#[test]
fn a_track_keeps_the_most_any_row_gives_a_frame() {
    let mut track = Track::new(String::from("t"), 10);
    track.cover((2, 4), 500);
    track.cover((4, 4), 1000);
    track.cover((0, 3), 2000);
    assert_eq!(
        track.cover,
        [1000, 1000, 1000, 500, 1000, 1000, 1000, 1000, 0, 0]
    );
    // A span past the movie's end stops at it.
    track.cover((9, 5), 250);
    assert_eq!(track.cover.get(9), Some(&250));
    assert_eq!(track.frames(), 8);
}

#[test]
fn stretches_shade_by_their_mean_and_merge_into_runs() {
    let cover = [0, 0, 1000, 1000, 1000, 1000, 500, 0, 10, 0];
    // Two frames a pixel: none, whole, whole, a quarter, a hundredth shown as the faintest.
    assert_eq!(runs(&cover, 2), vec![(1, 2, 10), (3, 1, 3), (4, 1, 1)]);
    assert_eq!(percent(1, 3), "33.3 %");
    assert_eq!(percent(0, 0), "0.0 %");
}

#[test]
fn a_movie_without_cached_rows_draws_its_first_two_tracks() {
    let tracks = ["recorded", "cached", "model"]
        .map(|label| Track::new(label.to_owned(), 4))
        .into_iter()
        .collect();
    let mut movie = Movie {
        key: String::from("any"),
        category: String::from("any%"),
        frames: 4,
        rows: 1,
        tracks,
    };
    assert_eq!(movie.drawn().len(), 2);
    if let Some(cached) = movie.tracks.get_mut(1) {
        cached.cover((0, 1), 1000);
    }
    assert_eq!(movie.drawn().len(), 3);
}

#[test]
fn the_picture_regenerates_byte_identically() {
    let floors = super::super::floors(
        &std::fs::read_to_string(root().join(super::super::FLOORS)).unwrap_or_default(),
    );
    let first = floors
        .as_ref()
        .map_err(Clone::clone)
        .and_then(|floors| Timeline::measure(root(), floors));
    let second = floors.and_then(|floors| Timeline::measure(root(), &floors));
    let (Ok(first), Ok(second)) = (first, second) else {
        panic!("the timeline doesn't measure");
    };
    assert_eq!(first.svg(), second.svg());
    let committed = std::fs::read_to_string(root().join(super::SVG)).unwrap_or_default();
    assert_eq!(committed.replace("\r\n", "\n"), first.svg());
    // Nothing of the game: the picture holds counts, labels and the checks' names.
    assert!(!first.svg().contains("seed"));
}
