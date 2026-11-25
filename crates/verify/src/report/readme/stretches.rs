//! The milestone's movie in the stretches being worked, from [`STRETCHES`]: each drawn where it
//! lies in the whole movie, with a bar of its frames that pass in the headline's marks or a tick
//! once all of them do, and the first that doesn't in bold.

use std::fmt::{self, Display, Write as _};

use serde::Deserialize;

use super::{MOVIE, Run, bar, cell_frames, passing};

/// The stretches, from the repository root.
pub(super) const STRETCHES: &str = "scenarios/replay/stretches.toml";

/// Where the README links a work issue.
const ISSUES: &str = "https://github.com/JimLundin/sm-decomp/issues";

/// The cells of a stretch's bar of frames that pass, half the movie's, so the table stays narrow.
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

    /// The frames of it among the first `started` held continuously from power-on.
    fn started(&self, started: u64) -> u64 {
        let [first, last] = self.frames;
        started.min(last).saturating_add(1).saturating_sub(first)
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

impl Stretch {
    /// Where it lies in the movie of `total` frames: the cells of [`MOVIE`] that draw any of its
    /// frames, `█`, among the others, `·`.
    fn place(&self, total: u64) -> String {
        let [first, last] = self.frames;
        (0..MOVIE)
            .map(|cell| {
                let (from, to) = cell_frames(cell, total);
                if from <= to && from <= last && first <= to {
                    '█'
                } else {
                    '·'
                }
            })
            .collect()
    }
}

/// The table of `stretches` with the frames of each that `runs` hold, the first `started` of the
/// movie continuously from power-on, each drawn in the movie they divide.
pub(super) fn table(stretches: &[Stretch], runs: &[Run], started: u64) -> String {
    let total = stretches.last().map_or(0, |stretch| stretch.frames[1]);
    let mut out = String::from(
        "| Stretch | In the movie | Frames that pass | Work |\n| --- | --- | --- | --- |\n",
    );
    let mut bolded = false;
    for stretch in stretches {
        let (held, of) = (stretch.passing(runs), stretch.len());
        let (name, passes) = if held == of {
            (stretch.name.clone(), String::from("✓"))
        } else {
            let passes = format!("`{}`", bar(stretch.started(started), held, of, CELLS));
            if bolded {
                (stretch.name.clone(), passes)
            } else {
                bolded = true;
                (format!("**{}**", stretch.name), passes)
            }
        };
        let _ = writeln!(
            out,
            "| {name} | `{}` | {passes} | {} |",
            stretch.place(total),
            stretch.issue
        );
    }
    out
}

#[cfg(test)]
mod tests {
    use super::{STRETCHES, Stretch, stretches, table};
    use crate::report::plan::{PLAN, Plan, seed};
    use crate::report::readme::{PLAYED, Run, listed, passing};
    use crate::report::seeds::seed_frame;

    /// The repository root, from this crate's manifest.
    fn root() -> &'static std::path::Path {
        std::path::Path::new(concat!(env!("CARGO_MANIFEST_DIR"), "/../.."))
    }

    /// The milestone's last frame: the movie frame of the seed that ends it, in the plan.
    fn total() -> u64 {
        let rebuilt = crate::report::DOMAINS
            .iter()
            .map(|domain| domain.name)
            .collect();
        let plan = std::fs::read_to_string(root().join(PLAN)).unwrap_or_default();
        let ends = Plan::parse(&plan, &rebuilt)
            .and_then(|plan| {
                plan.milestone()
                    .end()
                    .map(|(capture, index)| (capture.to_owned(), index))
            })
            .and_then(|(capture, index)| seed_frame(root(), &capture, index));
        assert!(ends.is_ok(), "{ends:?}");
        ends.unwrap_or_default()
    }

    /// The committed stretches, as text, as a table and parsed.
    fn committed() -> (String, toml::Table, Result<Vec<Stretch>, String>) {
        let text = std::fs::read_to_string(root().join(STRETCHES)).unwrap_or_default();
        let table = text.parse().unwrap_or_default();
        let parsed = stretches(&text, "any", total());
        (text, table, parsed)
    }

    /// A run seeded at movie frame `seed` that held `held` frames.
    fn run(seed: u64, held: u64) -> Run {
        Run {
            seed,
            held,
            power_on: false,
        }
    }

    /// `file` with its stretches changed by `fault`, as text.
    fn faulted(
        file: &toml::Table,
        fault: impl FnOnce(&mut Vec<toml::Value>) -> Option<()>,
    ) -> Option<String> {
        let mut file = file.clone();
        fault(file.get_mut("stretch")?.as_array_mut()?)?;
        Some(file.to_string())
    }

    /// Frame `which` (0 the first, 1 the last) of stretch `index`, set by `to` from its first and
    /// last.
    fn set_frame(
        all: &mut [toml::Value],
        index: usize,
        which: usize,
        to: impl FnOnce(i64, i64) -> Option<i64>,
    ) -> Option<()> {
        let frames = all.get_mut(index)?.get_mut("frames")?.as_array_mut()?;
        let first = frames.first()?.as_integer()?;
        let last = frames.get(1)?.as_integer()?;
        *frames.get_mut(which)? = toml::Value::Integer(to(first, last)?);
        Some(())
    }

    #[test]
    fn the_stretches_cover_the_milestone_back_to_back() {
        let (text, file, parsed) = committed();
        let total = total();
        let count = parsed.as_ref().map(Vec::len);
        assert!(count.is_ok_and(|count| count > 2), "{parsed:?}");
        let last = count.unwrap_or_default().saturating_sub(1);
        // The file written back unchanged parses the same, so a fault below is its figure.
        assert_eq!(stretches(&file.to_string(), "any", total), parsed);
        assert_eq!(stretches(&text, "any", total.saturating_add(1)).ok(), None);
        // A gap, an overlap, a short or long end, a stretch that ends before it starts, a missing
        // issue and another movie all fail, each moved off the committed figures.
        let planted = [
            faulted(&file, |all| {
                set_frame(all, 1, 0, |first, _| first.checked_add(1))
            }),
            faulted(&file, |all| {
                set_frame(all, 1, 0, |first, _| first.checked_sub(1))
            }),
            faulted(&file, |all| {
                set_frame(all, last, 1, |_, end| end.checked_sub(1))
            }),
            faulted(&file, |all| {
                set_frame(all, last, 1, |_, end| end.checked_add(1))
            }),
            // Ends the frame before it starts, the next stretch starting where it did: still
            // back to back.
            faulted(&file, |all| {
                let mut start = None;
                set_frame(all, 1, 1, |first, _| {
                    start = Some(first);
                    first.checked_sub(1)
                })?;
                set_frame(all, 2, 0, |_, _| start)
            }),
            faulted(&file, |all| {
                all.get_mut(1)?.as_table_mut()?.remove("issue").map(drop)
            }),
        ];
        for (fault, planted) in planted.into_iter().enumerate() {
            assert!(
                planted.is_some_and(|text| stretches(&text, "any", total).is_err()),
                "fault {fault}"
            );
        }
        let mut other = file;
        other.insert(
            String::from("movie"),
            toml::Value::String(String::from("100")),
        );
        assert!(stretches(&other.to_string(), "any", total).is_err());
    }

    #[test]
    fn the_stretches_start_after_the_seeds_that_open_them() {
        let Ok(all) = committed().2 else {
            panic!("the committed stretches don't parse");
        };
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
        let Ok(all) = committed().2 else {
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
            assert_eq!(by_stretch, passing(&runs, 0, total()));
        }
    }

    /// Three stretches of a 400-frame movie, 10 frames a cell of it: Boot frames 1 to 100,
    /// Title 101 to 105 and Intro 106 to 400.
    fn three() -> Vec<Stretch> {
        stretches(
            "movie = \"any\"\n\
             [[stretch]]\nname = \"Boot\"\nframes = [1, 100]\nissue = 1\n\
             [[stretch]]\nname = \"Title\"\nframes = [101, 105]\nissue = 2\n\
             [[stretch]]\nname = \"Intro\"\nframes = [106, 400]\nissue = 3\n",
            "any",
            400,
        )
        .unwrap_or_default()
    }

    #[test]
    fn a_stretch_is_drawn_where_it_lies_in_the_movie() {
        let three = three();
        let places: Vec<String> = three.iter().map(|stretch| stretch.place(400)).collect();
        assert_eq!(
            places,
            [
                format!("{}{}", "█".repeat(10), "·".repeat(30)),
                // Five frames still draw the cell that holds them, which the next shares.
                format!("{}█{}", "·".repeat(10), "·".repeat(29)),
                format!("{}{}", "·".repeat(10), "█".repeat(30)),
            ]
        );
    }

    #[test]
    fn a_finished_stretch_shows_a_tick_and_the_first_unfinished_is_bold() {
        let three = three();
        // Boot plays whole from power-on and the title in part; a later seed holds a little of the
        // intro.
        let page = table(&three, &[run(0, 103), run(200, 30)], 103);
        let rows: Vec<&str> = page.lines().skip(2).collect();
        let link = |issue: u32| {
            format!("[#{issue}](https://github.com/JimLundin/sm-decomp/issues/{issue})")
        };
        let place = |stretch: usize| three.get(stretch).map(|stretch| stretch.place(400));
        assert_eq!(
            rows,
            [
                format!(
                    "| Boot | `{}` | ✓ | {} |",
                    place(0).unwrap_or_default(),
                    link(1)
                ),
                format!(
                    "| **Title** | `{}` | `████████████░░░░░░░░` | {} |",
                    place(1).unwrap_or_default(),
                    link(2)
                ),
                format!(
                    "| Intro | `{}` | `▓▓░░░░░░░░░░░░░░░░░░` | {} |",
                    place(2).unwrap_or_default(),
                    link(3)
                ),
            ]
        );
        // With none whole, the first is bold; with all whole, none is.
        assert!(table(&three, &[], 0).contains("| **Boot** |"));
        assert!(!table(&three, &[run(0, 400)], 400).contains("**"));
    }
}
