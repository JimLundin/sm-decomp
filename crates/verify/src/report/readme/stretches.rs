//! The milestone's movie in the stretches being worked, from [`STRETCHES`]: each with its frames
//! that pass, collapsed once all of them do, and the first that doesn't marked as next.

use std::fmt::{self, Display, Write as _};

use serde::Deserialize;

use super::{Run, bar, frame, passing};

/// The stretches, from the repository root.
pub(super) const STRETCHES: &str = "scenarios/replay/stretches.toml";

/// Where the README links a work issue.
const ISSUES: &str = "https://github.com/JimLundin/sm-decomp/issues";

/// The cells of a stretch's bar, half the headline's, so the table stays narrow.
const CELLS: u64 = 20;

/// The stretches file.
#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct Listed {
    /// The movie they divide, the milestone's.
    movie: String,
    stretch: Vec<Stretch>,
}

/// A piece of the movie being worked.
#[derive(Deserialize, Clone, Debug, PartialEq, Eq)]
#[serde(deny_unknown_fields)]
pub(super) struct Stretch {
    /// What the player sees.
    name: String,
    /// The first and last movie frame.
    frames: [u64; 2],
    /// The work issue that plays it.
    issue: Issue,
}

/// A work issue's number.
#[derive(Deserialize, Clone, Copy, Debug, PartialEq, Eq)]
#[serde(transparent)]
struct Issue(u32);

impl Display for Issue {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "[#{}]({ISSUES}/{})", self.0, self.0)
    }
}

impl Stretch {
    /// How many frames it has.
    const fn len(&self) -> u64 {
        let [first, last] = self.frames;
        last.saturating_sub(first).saturating_add(1)
    }

    /// The frames of it some run holds.
    fn passing(&self, runs: &[Run]) -> u64 {
        let [first, last] = self.frames;
        passing(runs, first.saturating_sub(1), last)
    }
}

/// The stretches `text` divides `movie` into, when they cover its frames 1 to `total` back to
/// back.
///
/// # Errors
///
/// Why the file can't be read as stretches of `movie`, or the first frame a stretch leaves
/// uncovered or covers twice.
pub(super) fn stretches(text: &str, movie: &str, total: u64) -> Result<Vec<Stretch>, String> {
    let listed: Listed = toml::from_str(text).map_err(|e| format!("{STRETCHES}: {e}"))?;
    if listed.movie != movie {
        return Err(format!(
            "{STRETCHES}: the stretches divide {}, not the milestone's {movie}",
            listed.movie
        ));
    }
    let mut next = 1_u64;
    for stretch in &listed.stretch {
        let [first, last] = stretch.frames;
        if first != next || last < first {
            return Err(format!(
                "{STRETCHES}: {} has frames {first} to {last}, not from {next}, the frame after \
                 the stretch before",
                stretch.name
            ));
        }
        next = last.saturating_add(1);
    }
    if next != total.saturating_add(1) {
        return Err(format!(
            "{STRETCHES}: the stretches end at frame {}, not the milestone's {total}",
            next.saturating_sub(1)
        ));
    }
    Ok(listed.stretch)
}

/// The table of `stretches` with the frames of each that `runs` hold.
pub(super) fn table(stretches: &[Stretch], runs: &[Run]) -> String {
    let mut out = String::from(
        "| Stretch | Movie frames | Frames that pass | Work |\n| --- | --- | --- | --- |\n",
    );
    let mut next_marked = false;
    for stretch in stretches {
        let [first, last] = stretch.frames;
        let (held, of) = (stretch.passing(runs), stretch.len());
        let (name, passes) = if held == of {
            (stretch.name.clone(), format!("✓ all {}", frame(of)))
        } else {
            let passes = format!(
                "`{}` {} of {}",
                bar(held, of, CELLS),
                frame(held),
                frame(of)
            );
            if next_marked {
                (stretch.name.clone(), passes)
            } else {
                next_marked = true;
                (format!("**Next: {}**", stretch.name), passes)
            }
        };
        let _ = writeln!(
            out,
            "| {name} | {} to {} | {passes} | {} |",
            frame(first),
            frame(last),
            stretch.issue
        );
    }
    out
}

#[cfg(test)]
mod tests {
    use super::{CELLS, STRETCHES, Stretch, stretches, table};
    use crate::report::plan::seed;
    use crate::report::readme::{PLAYED, Run, Stop, bar, listed, passing};
    use crate::report::seeds::seed_frame;

    /// The repository root, from this crate's manifest.
    fn root() -> &'static std::path::Path {
        std::path::Path::new(concat!(env!("CARGO_MANIFEST_DIR"), "/../.."))
    }

    /// The milestone's last frame: the movie frame of the seed that ends it.
    const TOTAL: u64 = 15_196;

    /// The committed stretches.
    fn committed() -> (String, Result<Vec<Stretch>, String>) {
        let text = std::fs::read_to_string(root().join(STRETCHES)).unwrap_or_default();
        let parsed = stretches(&text, "any", TOTAL);
        (text, parsed)
    }

    /// A run seeded at movie frame `seed` that held `held` frames.
    fn run(seed: u64, held: u64) -> Run {
        Run {
            row: String::from("row"),
            start: 0,
            seed,
            held,
            stop: Stop::Failed(String::from("a difference")),
            power_on: false,
        }
    }

    #[test]
    fn the_stretches_cover_the_milestone_back_to_back() {
        let (text, parsed) = committed();
        assert!(
            parsed.as_ref().is_ok_and(|all| all.len() == 9),
            "{parsed:?}"
        );
        // A gap, an overlap, a short or long end, a missing issue and another movie all fail.
        for (from, to) in [
            ("frames = [271, 568]", "frames = [272, 568]"),
            ("frames = [271, 568]", "frames = [270, 568]"),
            ("frames = [13237, 15196]", "frames = [13237, 15195]"),
            ("frames = [13237, 15196]", "frames = [13237, 15197]"),
            ("frames = [8316, 8538]", "frames = [8316, 8315]"),
            ("issue = 753\n", "\n"),
            ("movie = \"any\"", "movie = \"100\""),
        ] {
            assert!(text.contains(from), "{from}");
            assert!(
                stretches(&text.replacen(from, to, 1), "any", TOTAL).is_err(),
                "{to}"
            );
        }
    }

    #[test]
    fn the_stretches_start_after_the_seeds_that_open_them() {
        let Ok(all) = committed().1 else {
            panic!("the committed stretches don't parse");
        };
        let ends = seed("any-opening:7").and_then(|end| seed_frame(root(), end.0, end.1));
        assert_eq!(ends, Ok(15_196));
        for (name, opened_by) in [
            ("The intro", "any-opening:1"),
            ("The flight to Ceres", "any-opening:5"),
            ("Her arrival", "any-opening:6"),
            ("Rooms 1 to 5", "any-boundary:1"),
            ("Ridley's room", "any-boundary:6"),
        ] {
            let seeded =
                seed(opened_by).and_then(|(capture, index)| seed_frame(root(), capture, index));
            let first = all
                .iter()
                .find(|stretch| stretch.name == name)
                .map(|stretch| stretch.frames[0]);
            assert_eq!(
                seeded.map(|frame| frame.saturating_add(1)).ok(),
                first,
                "{name}"
            );
        }
    }

    #[test]
    fn the_stretches_count_every_frame_the_runs_hold_once() {
        let Ok(all) = committed().1 else {
            panic!("the committed stretches don't parse");
        };
        let text = std::fs::read_to_string(root().join(PLAYED)).unwrap_or_default();
        let committed = listed(root(), &text, "any").unwrap_or_default();
        assert_ne!(committed, []);
        for runs in [
            committed,
            vec![run(0, 271), run(260, 400), run(10_000, 5_196)],
        ] {
            let by_stretch: u64 = all.iter().map(|stretch| stretch.passing(&runs)).sum();
            assert_eq!(by_stretch, passing(&runs, 0, TOTAL));
        }
    }

    #[test]
    fn a_finished_stretch_collapses_and_the_first_unfinished_is_next() {
        let three = stretches(
            "movie = \"any\"\n\
             [[stretch]]\nname = \"Boot\"\nframes = [1, 10]\nissue = 1\n\
             [[stretch]]\nname = \"Title\"\nframes = [11, 20]\nissue = 2\n\
             [[stretch]]\nname = \"Intro\"\nframes = [21, 30]\nissue = 3\n",
            "any",
            30,
        )
        .unwrap_or_default();
        // Boot plays whole, the title in part, the intro not at all.
        let page = table(&three, &[run(0, 14)]);
        let rows: Vec<&str> = page.lines().skip(2).collect();
        let link = |issue: u32| {
            format!("[#{issue}](https://github.com/JimLundin/sm-decomp/issues/{issue})")
        };
        assert_eq!(
            rows,
            [
                format!("| Boot | 1 to 10 | ✓ all 10 | {} |", link(1)),
                format!(
                    "| **Next: Title** | 11 to 20 | `{}` 4 of 10 | {} |",
                    bar(4, 10, CELLS),
                    link(2)
                ),
                format!(
                    "| Intro | 21 to 30 | `{}` 0 of 10 | {} |",
                    bar(0, 10, CELLS),
                    link(3)
                ),
            ]
        );
        // With none whole, the first is next; with all whole, none is.
        assert!(table(&three, &[]).contains("| **Next: Boot** |"));
        assert!(!table(&three, &[run(0, 30)]).contains("Next"));
    }
}
