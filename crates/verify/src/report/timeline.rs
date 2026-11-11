//! The timeline: for each whole movie the lock's rows are cut from, which stretches of its
//! frames each check covers (Jim, 2026-11-11: a visual picture of the progress), drawn as
//! `docs/timeline.svg` for the README and given in figures in the report.
//!
//! It reads committed data only, so CI keeps it current without the ROM or the cache: the
//! movies' frame counts (`movies/manifest.toml`), each capture's seeds' movie frames
//! (`movies/room-seeds-<capture>.csv`, the `room_index` and `movie_frame` columns alone), the
//! lock's rows ([`LOCK`]), the harness floors, the free-run gate's green pairs ([`GREEN`]) and
//! the lockstep ratchet ([`LOCKSTEP`]). A row cut from seed `CAPTURE:INDEX` covers the movie's
//! frames from that seed's frame for the row's frames; its capture's movie is the capture
//! name's first word (`100-input-phase` is the 100% movie's). What it writes is frame counts,
//! row counts and the checks' names: nothing of the game.
//!
//! Each track is a row of the picture: per frame of the movie, how much of it the check covers,
//! from 0 to 1, the most any row gives it. The recorded and cached tracks give 1 to a row's
//! frames; a model's track gives the share of the row's frames its harness floor matches; the
//! gate's the share of the row's streams that are green; the ratchet's the share of its Ticks
//! fully equal. The picture shades each pixel's stretch by the mean over its frames.

use std::collections::{BTreeMap, BTreeSet};
use std::fmt::Write as _;
use std::path::Path;

use serde::Deserialize;

use super::status::{GREEN, LOCK, LOCKSTEP};
use super::{grouped, read, table};
use crate::gate::Ratchet;
use crate::lock::{Lock, Start};
use crate::ratchet;

/// Where the picture is committed, from the repository root.
pub(super) const SVG: &str = "docs/timeline.svg";
/// The movies' manifest.
const MANIFEST: &str = "movies/manifest.toml";
/// A capture's seeds and their movie frames, by the capture's name.
const SEEDS: (&str, &str) = ("movies/room-seeds-", ".csv");
/// What a movie's file name starts with; its key follows, up to the next `-`.
const MOVIE_FILE: &str = "super-metroid-";
/// The frames between two ticks on the picture's axis: about ten minutes at 60 frames a second.
const TICK_FRAMES: usize = 36_000;
/// The picture's width for the longest movie, in pixels: one stretch of frames each.
const BAR_WIDTH: usize = 700;
/// Where the bars start, right of the labels.
const BAR_LEFT: usize = 250;
/// A track's height and the gap below it.
const TRACK: (usize, usize) = (12, 4);
/// The height of the picture's title and legend, above the first movie.
const HEAD: usize = 54;
/// The picture's width.
const WIDTH: usize = BAR_LEFT + BAR_WIDTH + 20;
/// A whole cover, per mille.
const WHOLE: u32 = 1000;
/// The shades a stretch can take, besides none.
const SHADES: u32 = 10;
/// The covered marks' hue, the track's ground, the chart's surface and its two inks.
const COLOURS: Colours = Colours {
    mark: "#2a78d6",
    ground: "#f0efec",
    surface: "#fcfcfb",
    ink: "#0b0b0b",
    muted: "#52514e",
};

/// The picture's colours.
struct Colours {
    mark: &'static str,
    ground: &'static str,
    surface: &'static str,
    ink: &'static str,
    muted: &'static str,
}

#[derive(Deserialize)]
struct Manifest {
    movies: Vec<MovieEntry>,
}

#[derive(Deserialize)]
struct MovieEntry {
    category: String,
    file: String,
    frames: usize,
}

/// One movie's tracks.
struct Movie {
    /// Its key, the first word of its captures' names.
    key: String,
    category: String,
    frames: usize,
    /// The lock's rows cut from it.
    rows: usize,
    tracks: Vec<Track>,
}

/// How many of a movie's tracks every movie draws: the recorded and the cached.
const ALWAYS_DRAWN: usize = 2;
/// The free-run gate's track.
const GATE: &str = "free-run gate: green streams";

impl Movie {
    /// The tracks the picture draws: all of them, or, when the harnesses run none of the movie's
    /// rows and so nothing past them covers it, the recorded and the cached alone.
    fn drawn(&self) -> &[Track] {
        let cached = self
            .tracks
            .get(1)
            .is_some_and(|track| track.cover.iter().any(|&value| value > 0));
        if cached {
            &self.tracks
        } else {
            self.tracks.get(..ALWAYS_DRAWN).unwrap_or(&self.tracks)
        }
    }
}

/// A check's cover of a movie, per frame, per mille.
struct Track {
    label: String,
    cover: Vec<u16>,
}

impl Track {
    fn new(label: String, frames: usize) -> Self {
        Self {
            label,
            cover: vec![0; frames],
        }
    }

    /// Covers `span` by `share` per mille, where it doesn't already cover more.
    fn cover(&mut self, span: (usize, usize), share: u32) {
        let share = u16::try_from(share.min(WHOLE)).unwrap_or_default();
        let (start, frames) = span;
        for value in self.cover.iter_mut().skip(start).take(frames) {
            *value = (*value).max(share);
        }
    }

    /// The frames covered, each weighed by its cover.
    fn frames(&self) -> usize {
        let sum: u64 = self.cover.iter().map(|&value| u64::from(value)).sum();
        let frames = sum
            .saturating_add(u64::from(WHOLE / 2))
            .checked_div(u64::from(WHOLE));
        usize::try_from(frames.unwrap_or_default()).unwrap_or_default()
    }
}

/// `part` of `whole` per mille, rounded down.
fn per_mille(part: u64, whole: u64) -> u32 {
    let share = part.saturating_mul(u64::from(WHOLE)).checked_div(whole);
    u32::try_from(share.unwrap_or_default()).unwrap_or(WHOLE)
}

/// The movie frame of the seed `index` of `capture`.
///
/// # Errors
///
/// The path and the error when the capture's seeds can't be read or don't hold the seed.
pub(super) fn seed_frame(root: &Path, capture: &str, index: u32) -> Result<usize, String> {
    seed_frames(root, &BTreeSet::from([capture]))?
        .get(&(capture.to_owned(), index))
        .copied()
        .ok_or_else(|| format!("{}{capture}{}: no seed {index}", SEEDS.0, SEEDS.1))
}

/// The movie frame of each capture's seed, by (capture, index).
fn seed_frames(
    root: &Path,
    captures: &BTreeSet<&str>,
) -> Result<BTreeMap<(String, u32), usize>, String> {
    let mut frames = BTreeMap::new();
    for capture in captures {
        let path = format!("{}{capture}{}", SEEDS.0, SEEDS.1);
        let Ok(text) = std::fs::read_to_string(root.join(&path)) else {
            continue;
        };
        let mut lines = text.lines();
        let header: Vec<&str> = lines.next().unwrap_or_default().split(',').collect();
        let column = |name: &str| {
            header
                .iter()
                .position(|column| column.trim() == name)
                .ok_or_else(|| format!("{path}: no `{name}` column"))
        };
        let (index_at, frame_at) = (column("room_index")?, column("movie_frame")?);
        for line in lines.filter(|line| !line.trim().is_empty()) {
            let fields: Vec<&str> = line.split(',').map(str::trim).collect();
            let field = |at: usize| {
                fields
                    .get(at)
                    .ok_or_else(|| format!("{path}: {line}: too few columns"))
            };
            let index = field(index_at)?
                .parse::<u32>()
                .map_err(|e| format!("{path}: {line}: {e}"))?;
            let frame = field(frame_at)?
                .parse::<usize>()
                .map_err(|e| format!("{path}: {line}: {e}"))?;
            frames.insert(((*capture).to_owned(), index), frame);
        }
    }
    Ok(frames)
}

/// Each movie of the manifest with its tracks, empty: the recorded and cached rows, each
/// harness domain's model, the gate and lockstep.
fn movies(manifest: &Manifest, domains: &BTreeSet<&str>) -> Vec<Movie> {
    manifest
        .movies
        .iter()
        .filter_map(|entry| {
            let key = entry.file.strip_prefix(MOVIE_FILE)?.split('-').next()?;
            let labels = ["recorded: the lock's rows", "cached for the harnesses"]
                .into_iter()
                .map(str::to_owned)
                .chain(
                    domains
                        .iter()
                        .map(|domain| format!("{domain} model matches")),
                )
                .chain(
                    [GATE, "lockstep: Ticks fully equal"]
                        .into_iter()
                        .map(str::to_owned),
                );
            Some(Movie {
                key: key.to_owned(),
                category: entry.category.clone(),
                frames: entry.frames,
                rows: 0,
                tracks: labels
                    .map(|label| Track::new(label, entry.frames))
                    .collect(),
            })
        })
        .collect()
}

/// The movies' tracks and the lock's rows cut from no movie, with their frames.
pub(super) struct Timeline {
    movies: Vec<Movie>,
    outside: (usize, u64),
}

impl Timeline {
    /// The timeline for the tree at `root`.
    ///
    /// # Errors
    ///
    /// The path and the error when a file can't be read or doesn't parse.
    pub(super) fn measure(
        root: &Path,
        floors: &BTreeMap<(String, String), usize>,
    ) -> Result<Self, String> {
        let manifest: Manifest =
            toml::from_str(&read(&root.join(MANIFEST))?).map_err(|e| format!("{MANIFEST}: {e}"))?;
        let lock = Lock::parse(&read(&root.join(LOCK))?).map_err(|e| format!("{LOCK}: {e}"))?;
        let green =
            Ratchet::parse(&read(&root.join(GREEN))?).map_err(|e| format!("{GREEN}: {e}"))?;
        let lockstep =
            ratchet::parse(&read(&root.join(LOCKSTEP))?).map_err(|e| format!("{LOCKSTEP}: {e}"))?;
        let captures: BTreeSet<&str> = lock
            .rows
            .iter()
            .filter_map(|row| match &row.start {
                Start::Seed { capture, .. } => Some(capture.as_str()),
                Start::PowerOn => None,
            })
            .collect();
        let seeds = seed_frames(root, &captures)?;
        let domains: BTreeSet<&str> = floors.keys().map(|(domain, _)| domain.as_str()).collect();
        let mut movies = movies(&manifest, &domains);
        let mut outside = (0usize, 0u64);
        for row in &lock.rows {
            let id = row.id.0.as_str();
            let placed = match &row.start {
                Start::Seed { capture, index } => {
                    let key = capture.split('-').next().unwrap_or_default();
                    seeds.get(&(capture.clone(), *index)).and_then(|&frame| {
                        Some((movies.iter_mut().find(|movie| movie.key == key)?, frame))
                    })
                }
                Start::PowerOn => None,
            };
            let Some((movie, start)) = placed else {
                outside = (
                    outside.0.saturating_add(1),
                    outside.1.saturating_add(row.frames.0),
                );
                continue;
            };
            movie.rows = movie.rows.saturating_add(1);
            let span = (start, usize::try_from(row.frames.0).unwrap_or_default());
            let streams = u64::try_from(row.streams.len()).unwrap_or_default();
            let green_streams = green
                .pairs
                .iter()
                .filter(|(green, _)| green.0 == id)
                .count();
            let floors: Vec<Option<u32>> = domains
                .iter()
                .map(|domain| {
                    floors
                        .get(&((*domain).to_owned(), id.to_owned()))
                        .map(|&matched| {
                            per_mille(u64::try_from(matched).unwrap_or_default(), row.frames.0)
                        })
                })
                .collect();
            let cached = floors.iter().any(Option::is_some);
            let equal = lockstep
                .iter()
                .find(|summary| summary.row == id)
                .map(|summary| per_mille(summary.equal, summary.ticks));
            let shares = [Some(WHOLE), cached.then_some(WHOLE)]
                .into_iter()
                .chain(floors)
                .chain([
                    (green_streams > 0).then(|| {
                        per_mille(u64::try_from(green_streams).unwrap_or_default(), streams)
                    }),
                    equal,
                ]);
            for (track, share) in movie.tracks.iter_mut().zip(shares) {
                if let Some(share) = share {
                    track.cover(span, share);
                }
            }
        }
        movies.retain(|movie| movie.rows > 0);
        Ok(Self { movies, outside })
    }

    /// The movie whose key is `movie`, if the lock cuts a row from it: its category, and of its
    /// first `frames` frames, those inside a row whose every stream the free-run gate holds
    /// green.
    pub(super) fn green_from_start(&self, movie: &str, frames: usize) -> Option<(&str, usize)> {
        let movie = self.movies.iter().find(|listed| listed.key == movie)?;
        let gate = movie.tracks.iter().find(|track| track.label == GATE)?;
        let green = gate
            .cover
            .iter()
            .take(frames)
            .filter(|&&cover| u32::from(cover) >= WHOLE)
            .count();
        Some((&movie.category, green))
    }

    /// The picture, as SVG text.
    pub(super) fn svg(&self) -> String {
        let longest = self
            .movies
            .iter()
            .map(|movie| movie.frames)
            .max()
            .unwrap_or(1);
        let per_pixel = longest.div_ceil(BAR_WIDTH).max(1);
        let block = |movie: &Movie| {
            28_usize
                .saturating_add(
                    movie
                        .drawn()
                        .len()
                        .saturating_mul(TRACK.0.saturating_add(TRACK.1)),
                )
                .saturating_add(26)
        };
        let height = self
            .movies
            .iter()
            .map(block)
            .sum::<usize>()
            .saturating_add(HEAD);
        let mut out = format!(
            "<svg xmlns=\"http://www.w3.org/2000/svg\" width=\"{WIDTH}\" height=\"{height}\" \
             viewBox=\"0 0 {WIDTH} {height}\" font-family=\"system-ui, -apple-system, Segoe UI, \
             sans-serif\" font-size=\"11\">\n<title>How much of each movie the checks cover</title>\n\
             <rect width=\"{WIDTH}\" height=\"{height}\" fill=\"{}\"/>\n",
            COLOURS.surface
        );
        let _ = writeln!(
            out,
            "<text x=\"10\" y=\"20\" font-size=\"13\" font-weight=\"600\" fill=\"{}\">How much of \
             each movie the checks cover</text>\n<text x=\"10\" y=\"34\" fill=\"{}\">Darker is \
             more of a stretch's frames covered; one pixel is {} frames; a tick every {} frames, \
             about 10 minutes.</text>\n<text x=\"10\" y=\"48\" fill=\"{}\">A movie the harnesses \
             run no row of shows only its first two tracks.</text>",
            COLOURS.ink,
            COLOURS.muted,
            grouped(per_pixel),
            grouped(TICK_FRAMES),
            COLOURS.muted,
        );
        let mut top = HEAD;
        for movie in &self.movies {
            let width = movie.frames.div_ceil(per_pixel);
            let _ = writeln!(
                out,
                "<text x=\"10\" y=\"{}\" font-weight=\"600\" fill=\"{}\">{} movie: {} frames, {} \
                 rows of the lock</text>",
                top.saturating_add(18),
                COLOURS.ink,
                escape(&movie.category),
                grouped(movie.frames),
                grouped(movie.rows)
            );
            let mut y = top.saturating_add(28);
            for track in movie.drawn() {
                let _ = writeln!(
                    out,
                    "<text x=\"{}\" y=\"{}\" text-anchor=\"end\" fill=\"{}\">{}</text>\n\
                     <rect x=\"{BAR_LEFT}\" y=\"{y}\" width=\"{width}\" height=\"{}\" fill=\"{}\"/>",
                    BAR_LEFT.saturating_sub(8),
                    y.saturating_add(10),
                    COLOURS.muted,
                    escape(&track.label),
                    TRACK.0,
                    COLOURS.ground
                );
                for (start, run, shade) in runs(&track.cover, per_pixel) {
                    let _ = writeln!(
                        out,
                        "<rect x=\"{}\" y=\"{y}\" width=\"{run}\" height=\"{}\" fill=\"{}\" \
                         fill-opacity=\"{}\"/>",
                        BAR_LEFT.saturating_add(start),
                        TRACK.0,
                        COLOURS.mark,
                        opacity(shade)
                    );
                }
                y = y.saturating_add(TRACK.0).saturating_add(TRACK.1);
            }
            for tick in (0..=movie.frames).step_by(TICK_FRAMES) {
                let x = BAR_LEFT.saturating_add(tick.checked_div(per_pixel).unwrap_or_default());
                let _ = writeln!(
                    out,
                    "<line x1=\"{x}\" y1=\"{y}\" x2=\"{x}\" y2=\"{}\" stroke=\"{}\"/>\n<text \
                     x=\"{x}\" y=\"{}\" text-anchor=\"middle\" font-size=\"9\" fill=\"{}\">{}</text>",
                    y.saturating_add(4),
                    COLOURS.muted,
                    y.saturating_add(14),
                    COLOURS.muted,
                    grouped(tick)
                );
            }
            top = top.saturating_add(block(movie));
        }
        out.push_str("</svg>\n");
        out
    }

    /// The report's section: the legend and each movie's figures.
    pub(super) fn section(&self, out: &mut String) {
        let _ = write!(
            out,
            "\n## Timeline\n\n![How much of each movie the checks cover](timeline.svg)\n\n\
             For each whole movie the lock's rows are cut from, which stretches of its frames each \
             check covers (`verify::report::timeline`, `{SVG}`), from committed data alone: a \
             row cut from a seed covers the movie from that seed's frame (`{}<capture>{}`) for \
             its frames. Per frame a track's cover is from 0 to 1, the most any row gives it: \
             **recorded** and **cached** give a row's frames 1 when the lock holds the row and \
             when the harnesses run it (the rows the floors name); a **model** gives the share \
             of the row's frames its harness floor matches; the **free-run gate** the share of \
             the row's streams that are green; **lockstep** the share of its Ticks fully equal. \
             The picture shades a stretch by the mean over its frames; the figures are the \
             frames covered, each weighed by its cover, and their share of the movie. \
             {} of the lock's rows ({} frames) start from power-on or a derived seed, in no \
             movie, and are not drawn.\n",
            SEEDS.0,
            SEEDS.1,
            grouped(self.outside.0),
            usize::try_from(self.outside.1).map_or_else(|_| self.outside.1.to_string(), grouped)
        );
        for movie in &self.movies {
            let _ = write!(
                out,
                "\n### The {} movie\n\n{} frames, {} rows of the lock.\n\n",
                movie.category,
                grouped(movie.frames),
                grouped(movie.rows)
            );
            table(out, 1, &["Track", "Frames covered", "Share of the movie"]);
            for track in &movie.tracks {
                let frames = track.frames();
                let _ = writeln!(
                    out,
                    "| {} | {} | {} |",
                    track.label,
                    grouped(frames),
                    percent(frames, movie.frames)
                );
            }
        }
    }
}

/// `part` of `whole` as a percentage to one place.
pub(super) fn percent(part: usize, whole: usize) -> String {
    let tenths = part
        .saturating_mul(1000)
        .saturating_add(whole / 2)
        .checked_div(whole)
        .unwrap_or_default();
    format!("{}.{} %", tenths / 10, tenths % 10)
}

/// A shade's fill opacity: one of [`SHADES`] steps.
fn opacity(shade: u32) -> String {
    if shade >= SHADES {
        String::from("1")
    } else {
        format!("0.{shade}")
    }
}

/// The stretches of `per_pixel` frames as runs of equal shade: each run's first pixel, its width
/// and its shade, the unshaded left out.
fn runs(cover: &[u16], per_pixel: usize) -> Vec<(usize, usize, u32)> {
    let mut out: Vec<(usize, usize, u32)> = Vec::new();
    for (pixel, stretch) in cover.chunks(per_pixel).enumerate() {
        let sum: u64 = stretch.iter().map(|&value| u64::from(value)).sum();
        let whole = u64::try_from(stretch.len())
            .unwrap_or(1)
            .saturating_mul(u64::from(WHOLE));
        let shade = u32::try_from(
            sum.saturating_mul(u64::from(SHADES))
                .saturating_add(whole / 2)
                .checked_div(whole)
                .unwrap_or_default(),
        )
        .unwrap_or(SHADES);
        // A stretch with any cover shows, however little.
        let shade = if sum > 0 { shade.max(1) } else { 0 };
        match out.last_mut() {
            Some((start, width, last))
                if *last == shade && start.saturating_add(*width) == pixel =>
            {
                *width = width.saturating_add(1);
            }
            _ if shade > 0 => out.push((pixel, 1, shade)),
            _ => {}
        }
    }
    out
}

/// Text for SVG: its markup characters escaped.
fn escape(text: &str) -> String {
    text.replace('&', "&amp;")
        .replace('<', "&lt;")
        .replace('>', "&gt;")
}

#[cfg(test)]
mod tests;
