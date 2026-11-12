use std::collections::BTreeMap;
use std::path::Path;

use super::{GATE, Movie, Timeline, Track, percent, runs, seed_frame, start_in_movie};
use crate::lock::Start;

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

#[test]
fn the_milestone_counts_the_frames_whose_every_stream_is_green() {
    let mut gate = Track::new(GATE.to_owned(), 6);
    gate.cover((0, 2), 1000);
    gate.cover((2, 2), 500);
    gate.cover((4, 2), 1000);
    let timeline = Timeline {
        movies: vec![Movie {
            key: String::from("any"),
            category: String::from("any%"),
            frames: 6,
            rows: 1,
            tracks: vec![gate],
        }],
        outside: (0, 0),
    };
    // Of the first five frames, the two whole ones and the first of the last two.
    assert_eq!(timeline.green_from_start("any", 5), Some(("any%", 3)));
    assert_eq!(timeline.green_from_start("100", 5), None);
}

#[test]
fn a_seed_gives_its_movie_frame() {
    assert_eq!(seed_frame(root(), "any", 12), Ok(15_428));
    assert!(seed_frame(root(), "any", 100_000).is_err());
    assert!(seed_frame(root(), "no-such-capture", 1).is_err());
}

#[test]
fn a_row_from_power_on_opens_the_movie_its_id_names() {
    let seeds = BTreeMap::from([((String::from("any-opening"), 1), 568)]);
    // A movie starts at power-on, so its power-on row covers it from frame 0.
    assert_eq!(
        start_in_movie("any-power-on", &Start::PowerOn, &seeds),
        Some(("any", 0))
    );
    let seed = Start::Seed {
        capture: String::from("any-opening"),
        index: 1,
    };
    assert_eq!(
        start_in_movie("any-intro-0568", &seed, &seeds),
        Some(("any", 568))
    );
    // A seed the capture doesn't list places the row nowhere.
    let unlisted = Start::Seed {
        capture: String::from("any-opening"),
        index: 2,
    };
    assert_eq!(start_in_movie("any-intro-2219", &unlisted, &seeds), None);
}

#[test]
fn the_any_movie_counts_its_power_on_frames() {
    let floors = super::super::floors(
        &std::fs::read_to_string(root().join(super::super::FLOORS)).unwrap_or_default(),
    );
    let Ok(timeline) = floors.and_then(|floors| Timeline::measure(root(), &floors)) else {
        panic!("the timeline doesn't measure");
    };
    let Some(any) = timeline.movies.iter().find(|movie| movie.key == "any") else {
        panic!("no any% movie");
    };
    // `any-power-on`, from power-on, records the movie's first 568 frames.
    let recorded = any
        .tracks
        .first()
        .map(|track| track.cover.iter().take(568).all(|&cover| cover > 0));
    assert_eq!(recorded, Some(true));
}
