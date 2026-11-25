//! The README: the project's thesis and its progress in frames of the milestone's movie, counted
//! from the reviewed played runs of [`PLAYED`]: over the whole movie, by the stretches being
//! worked and by the plan's rungs.
//!
//! A lock row's cut c is movie frame f + c, f its seed's movie frame, or 0 for a row from
//! power-on, so a row's cut 0 is the frame after the previous row's last cut. A run seeded at
//! frame s that held h cuts holds frames s + 1 through s + h. The milestone's frames are 1
//! through the frame of the seed that ends it; frame 0 is power-on.

mod stretches;

use std::fmt::Write as _;
use std::path::Path;

use serde::Deserialize;

use super::plan::{Plan, seed};
use super::seeds::{first_frame, following, row_movie, seed_frame};
use super::{COMMAND, grouped, read};
use crate::lock::{Lock, Row, RowId, Start};

/// The README, from the repository root.
pub(super) const PATH: &str = "README.md";
/// The reviewed played runs, from the repository root.
pub(super) const PLAYED: &str = "scenarios/replay/played.toml";
/// The lock whose rows the runs play.
const LOCK: &str = "scenarios/locks/lsnes.toml";

/// What the project is for: MODEL §1 and RULES, "The goal and its measure".
const THESIS: &str = "**The goal** is to model Super Metroid's behaviour, all of it, as named behaviours, so \
each one can be reproduced exactly, adjusted (the space jump timing), added (respin) or removed \
(lag). In the end only this engine runs. The emulator, running the original, unmodified ROM, is \
the oracle that proves the vanilla behaviours exact: every compared frame cut and output byte \
matches.";

/// The played-runs file.
#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct Listed {
    /// The movie the runs play, the milestone's.
    movie: String,
    #[serde(default)]
    run: Vec<Listing>,
}

/// A run as the file states it: in cuts of its lock row, and in movie frames for its reviewer.
#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct Listing {
    /// The lock row played.
    row: String,
    /// The seed cut.
    start: u64,
    /// The last cut asked for, counted from the row's cut 0; it may lie in the rows that follow
    /// back to back.
    end: u64,
    /// The cuts after the seed whose compared fields all matched.
    held: u64,
    stop: Stop,
    /// The seed cut's movie frame.
    seed_frame: u64,
    /// The first and last movie frame held; none when it held none.
    frames: Option<[u64; 2]>,
    /// Whether it starts from the console's power-on, frame 0.
    power_on: bool,
}

/// Where a reviewed run stopped.
#[derive(Deserialize, Clone, Debug, PartialEq, Eq)]
#[serde(rename_all = "kebab-case")]
enum Stop {
    /// Every cut it asked for held.
    End,
    /// The first difference, missing owner or bad data, at the cut after the last it held.
    Failed(String),
}

/// A reviewed played run, in movie frames.
#[derive(Clone, Debug, PartialEq, Eq)]
struct Run {
    /// The lock row played.
    row: String,
    /// The seed cut.
    start: u64,
    /// The seed cut's movie frame.
    seed: u64,
    /// The frames held after the seed.
    held: u64,
    stop: Stop,
    /// Whether the seed is power-on.
    power_on: bool,
}

impl Run {
    /// The first and last movie frame it held; none when it held none.
    fn frames(&self) -> Option<(u64, u64)> {
        (self.held > 0).then(|| {
            (
                self.seed.saturating_add(1),
                self.seed.saturating_add(self.held),
            )
        })
    }
}

/// The reviewed runs of `movie`, each checked against the lock.
///
/// # Errors
///
/// Why the file, the lock or a seed can't be read, or the first run the lock contradicts.
fn runs(root: &Path, movie: &str) -> Result<Vec<Run>, String> {
    listed(root, &read(&root.join(PLAYED))?, movie)
}

/// The runs `text` lists for `movie`, each checked against the lock under `root`.
fn listed(root: &Path, text: &str, movie: &str) -> Result<Vec<Run>, String> {
    let listed: Listed = toml::from_str(text).map_err(|e| format!("{PLAYED}: {e}"))?;
    if listed.movie != movie {
        return Err(format!(
            "{PLAYED}: the runs play {}, not the milestone's {movie}",
            listed.movie
        ));
    }
    let lock = Lock::parse(&read(&root.join(LOCK))?).map_err(|e| format!("{LOCK}: {e}"))?;
    listed
        .run
        .iter()
        .map(|listing| checked(root, &lock, movie, listing))
        .collect()
}

/// The movie frame of the seed `index` of `capture`.
fn frame_at(root: &Path, (capture, index): (&str, u32)) -> Result<u64, String> {
    seed_frame(root, capture, index)
}

/// The cuts that follow `row`'s cut 0 back to back, reached as far as past cut `wanted`: its
/// own, and those of the rows that follow it with no frame between, from whichever capture of
/// the movie.
fn reach(root: &Path, lock: &Lock, row: &Row, wanted: u64) -> Result<u64, String> {
    let mut cuts = row.frames.0;
    let mut last = row;
    while cuts <= wanted
        && let Some(next) = following(root, lock, last)?
    {
        cuts = cuts.saturating_add(next.frames.0);
        last = next;
    }
    Ok(cuts)
}

/// `listing` as a run, when the lock agrees with every figure it states.
fn checked(root: &Path, lock: &Lock, movie: &str, listing: &Listing) -> Result<Run, String> {
    let wrong = |what: String| {
        format!(
            "{PLAYED}: {} from cut {}: {what}",
            listing.row, listing.start
        )
    };
    let row = lock
        .row(&RowId(listing.row.clone()))
        .ok_or_else(|| wrong(format!("no such row in {LOCK}")))?;
    if row_movie(row) != Some(movie) {
        return Err(wrong(format!("not a row of the {movie} movie")));
    }
    let first = first_frame(root, row)?;
    let cuts = reach(root, lock, row, listing.end)?;
    if listing.start >= listing.end || listing.end >= cuts {
        return Err(wrong(format!(
            "asks for cuts {} to {}, outside the {cuts} it can play back to back",
            listing.start, listing.end
        )));
    }
    let asked = listing.end.saturating_sub(listing.start);
    let stopped = match &listing.stop {
        Stop::End => listing.held == asked,
        Stop::Failed(_) => listing.held < asked,
    };
    if listing.held > asked || !stopped {
        return Err(wrong(format!(
            "holds {} of the {asked} cuts it asks for, which its stop {:?} contradicts",
            listing.held, listing.stop
        )));
    }
    let run = Run {
        row: listing.row.clone(),
        start: listing.start,
        seed: first.saturating_add(listing.start),
        held: listing.held,
        stop: listing.stop.clone(),
        power_on: row.start == Start::PowerOn && listing.start == 0,
    };
    if listing.seed_frame != run.seed {
        return Err(wrong(format!("seeds at movie frame {}", run.seed)));
    }
    if listing.frames != run.frames().map(<[u64; 2]>::from) {
        return Err(wrong(format!("holds movie frames {:?}", run.frames())));
    }
    if listing.power_on != run.power_on {
        return Err(wrong(format!("power_on is {}", run.power_on)));
    }
    Ok(run)
}

/// The frames after `after` through `through` that some run holds, each counted once.
fn passing(runs: &[Run], after: u64, through: u64) -> u64 {
    let mut spans: Vec<(u64, u64)> = runs
        .iter()
        .filter_map(Run::frames)
        .map(|(from, to)| (from.max(after.saturating_add(1)), to.min(through)))
        .filter(|(from, to)| from <= to)
        .collect();
    spans.sort_unstable();
    let mut counted = 0_u64;
    let mut uncounted = after.saturating_add(1);
    for (from, to) in spans {
        let from = from.max(uncounted);
        if from <= to {
            counted = counted.saturating_add(to.saturating_sub(from).saturating_add(1));
            uncounted = to.saturating_add(1);
        }
    }
    counted
}

/// The frames held back to back from power-on, with nothing fed back.
#[derive(Debug, PartialEq, Eq)]
struct FromStart {
    /// How many.
    held: u64,
    /// Why the frame after them failed; none when they reach the milestone's end.
    stop: Option<String>,
}

/// The longest reviewed run from power-on, within the milestone's `total` frames.
fn from_start(runs: &[Run], total: u64) -> FromStart {
    let Some(run) = runs
        .iter()
        .filter(|run| run.power_on)
        .max_by_key(|run| run.held)
    else {
        return FromStart {
            held: 0,
            stop: Some(String::from(
                "power-on is not played, as no reviewed run starts there",
            )),
        };
    };
    let held = run.held.min(total);
    let stop = (held < total).then(|| match &run.stop {
        Stop::Failed(why) => why.clone(),
        Stop::End => format!("the reviewed run of `{}` ends there", run.row),
    });
    FromStart { held, stop }
}

/// `count` of `total` as a percentage to a tenth.
fn percent(count: u64, total: u64) -> String {
    let tenths = count
        .saturating_mul(1000)
        .saturating_add(total / 2)
        .checked_div(total)
        .unwrap_or(0);
    format!("{}.{}%", tenths / 10, tenths % 10)
}

/// A movie frame or a count as the README writes it.
fn frame(value: u64) -> String {
    usize::try_from(value).map_or_else(|_| value.to_string(), grouped)
}

/// The cells of a headline bar: [`BAR`] wide in a monospace font, whatever the count.
const BAR: u64 = 40;

/// `count` of `total` as a bar of `cells`, filled ones first, rounded down: only the whole total
/// fills them all, and a count above none fills at least one, so neither end is overstated.
fn bar(count: u64, total: u64, cells: u64) -> String {
    let filled = if count == 0 {
        0
    } else {
        count
            .min(total)
            .saturating_mul(cells)
            .checked_div(total)
            .map_or(0, |filled| filled.max(1))
    };
    (0..cells)
        .map(|cell| if cell < filled { '█' } else { '░' })
        .collect()
}

/// The README's figures of `total` frames as aligned bars, each with its count and percentage.
fn bars(figures: &[(&str, u64)], total: u64) -> String {
    let label = figures
        .iter()
        .map(|(name, _)| name.len())
        .max()
        .unwrap_or_default();
    let of = frame(total);
    let mut out = String::from("```text\n");
    for (name, count) in figures {
        let _ = writeln!(
            out,
            "{name:<label$}  {}  {:>width$} of {of}  {:>6}",
            bar(*count, total, BAR),
            frame(*count),
            percent(*count, total),
            width = of.len()
        );
    }
    out.push_str("```\n");
    out
}

/// The README for the tree at `root`.
///
/// # Errors
///
/// Why the plan's seeds, the lock or the played runs can't be read, or the first run the lock
/// contradicts.
pub(super) fn page(root: &Path, plan: &Plan) -> Result<String, String> {
    let milestone = plan.milestone();
    let total = frame_at(root, milestone.end()?)?;
    let runs = runs(root, &milestone.movie)?;
    let mut out = format!(
        "# sm-decomp\n\n{THESIS}\n\n<!-- Generated by `{COMMAND}` from {PLAYED}, the lock and \
         the plan (`verify::report::readme`); never edit it by hand. -->\n\n## Progress\n\n"
    );
    let _ = writeln!(
        out,
        "The milestone is **{}**: the {}% movie from power-on through frame {}. A frame passes \
         when the engine, playing alone from a recorded seed on the movie's pads, matches every \
         compared field at its cut.\n",
        milestone.name,
        milestone.movie,
        frame(total)
    );
    let passes = passing(&runs, 0, total);
    let start = from_start(&runs, total);
    out.push_str(&bars(
        &[
            ("Frames that pass", passes),
            ("Continuous from power-on", start.held),
        ],
        total,
    ));
    out.push_str(
        "\n- **Frames that pass:** every frame some reviewed run holds, counted once.\n\
         - **Continuous from power-on:** the frames held back to back from power-on, with nothing \
         fed back.",
    );
    match &start.stop {
        Some(why) => {
            let _ = writeln!(
                out,
                " It stops at frame {}: {why}.",
                frame(start.held.saturating_add(1))
            );
        }
        None => out.push('\n'),
    }
    if let Some((run, (from, to))) = runs
        .iter()
        .filter_map(|run| Some((run, run.frames()?)))
        .max_by_key(|(run, _)| run.held)
    {
        let _ = writeln!(
            out,
            "- **Longest continuous run from any seed:** frames {} to {} ({} frames), `{}` from \
             its cut {}.",
            frame(from),
            frame(to),
            frame(run.held),
            run.row,
            run.start
        );
    }
    out.push_str(
        "\n### By stretch\n\nThe movie in the stretches being worked, in the order it plays; the \
         first not yet whole is next.\n\n",
    );
    let stretches = stretches::stretches(
        &read(&root.join(stretches::STRETCHES))?,
        &milestone.movie,
        total,
    )?;
    out.push_str(&stretches::table(&stretches, &runs));
    out.push_str(
        "\n### By rung\n\nThe plan's ladder of shorter stretches toward the milestone \
         ([domains.md §4](docs/architecture/domains.md#4-order)).\n\n| Rung | Movie frames | \
         Frames that pass |\n| --- | --- | ---: |\n",
    );
    for rung in plan.rungs() {
        let after = frame_at(root, seed(&rung.starts)?)?;
        let through = frame_at(root, seed(&rung.ends)?)?;
        let _ = writeln!(
            out,
            "| {} {} | {} to {} | {} of {} |",
            rung.id,
            rung.name,
            frame(after.saturating_add(1)),
            frame(through),
            frame(passing(&runs, after, through)),
            frame(through.saturating_sub(after))
        );
    }
    out.push_str(
        "\nBuilding and checking it, and the command that prints the models' sizes and the \
         indexed replay evidence, are in [BUILDING](docs/BUILDING.md).\n",
    );
    Ok(out)
}

#[cfg(test)]
mod tests {
    use super::{
        BAR, FromStart, PLAYED, Run, Stop, bar, bars, from_start, listed, passing, percent,
    };

    /// A run seeded at movie frame `seed` that held `held` frames.
    fn run(seed: u64, held: u64, power_on: bool) -> Run {
        Run {
            row: String::from("row"),
            start: 0,
            seed,
            held,
            stop: Stop::Failed(String::from("a difference")),
            power_on,
        }
    }

    /// The committed runs file, as text and as a table.
    fn committed() -> (std::path::PathBuf, String, toml::Table) {
        let root = std::path::Path::new(concat!(env!("CARGO_MANIFEST_DIR"), "/../.."));
        let text = std::fs::read_to_string(root.join(PLAYED)).unwrap_or_default();
        let table = text.parse().unwrap_or_default();
        (root.to_path_buf(), text, table)
    }

    /// A fault planted in one run's figures; none when they don't have the figure it moves.
    type Fault = fn(&mut toml::Table) -> Option<()>;

    /// `file` with run `index` changed by `fault`, as text; none when the fault doesn't apply to
    /// that run's figures.
    fn faulted(file: &toml::Table, index: usize, fault: Fault) -> Option<String> {
        let mut file = file.clone();
        let run = file
            .get_mut("run")?
            .as_array_mut()?
            .get_mut(index)?
            .as_table_mut()?;
        fault(run)?;
        Some(file.to_string())
    }

    /// The integer `key` of `run`, changed by `by`.
    fn shift(run: &mut toml::Table, key: &str, by: i64) -> Option<()> {
        let value = run.get_mut(key)?;
        *value = toml::Value::Integer(value.as_integer()?.checked_add(by)?);
        Some(())
    }

    #[test]
    fn the_committed_runs_agree_with_the_lock() {
        let (root, text, file) = committed();
        let runs = listed(&root, &text, "any");
        assert!(runs.as_ref().is_ok_and(|runs| !runs.is_empty()), "{runs:?}");
        let count = file
            .get("run")
            .and_then(toml::Value::as_array)
            .map_or(0, Vec::len);
        assert_eq!(runs.as_ref().map(Vec::len), Ok(count));
        // The file written back unchanged lists the same runs, so a fault below is its figure.
        assert_eq!(listed(&root, &file.to_string(), "any"), runs);
        // Each figure every run states is held to the lock: its row, its cuts within the rows it
        // plays back to back, its stop, its seed's frame, the frames it held and power-on. Each
        // fault is one figure of one run moved off the committed one.
        let faults: [(&str, Fault); 8] = [
            ("a row not in the lock", |run| {
                let row = format!("{}-NONE", run.get("row")?.as_str()?);
                run.insert(String::from("row"), toml::Value::String(row));
                Some(())
            }),
            ("one more cut held", |run| shift(run, "held", 1)),
            ("one fewer cut held", |run| {
                (run.get("held")?.as_integer()? > 0).then_some(())?;
                shift(run, "held", -1)
            }),
            ("an end past the rows played back to back", |run| {
                shift(run, "end", 1_000_000)
            }),
            ("the seed's frame one later", |run| {
                shift(run, "seed_frame", 1)
            }),
            ("the last frame held one later", |run| {
                let frames = run.get_mut("frames")?.as_array_mut()?;
                let last = frames.get_mut(1)?;
                *last = toml::Value::Integer(last.as_integer()?.checked_add(1)?);
                Some(())
            }),
            ("power-on the other way", |run| {
                let power_on = !run.get("power_on")?.as_bool()?;
                run.insert(String::from("power_on"), toml::Value::Boolean(power_on));
                Some(())
            }),
            ("the other stop", |run| {
                let stop = if run.get("stop")?.as_str() == Some("end") {
                    toml::Value::Table(toml::Table::from_iter([(
                        String::from("failed"),
                        toml::Value::String(String::from("a difference")),
                    )]))
                } else {
                    toml::Value::String(String::from("end"))
                };
                run.insert(String::from("stop"), stop);
                Some(())
            }),
        ];
        for (fault, plant) in faults {
            let planted: Vec<(usize, String)> = (0..count)
                .filter_map(|index| Some((index, faulted(&file, index, plant)?)))
                .collect();
            assert_ne!(planted, [], "{fault} fits no run");
            for (index, planted) in planted {
                assert!(
                    listed(&root, &planted, "any").is_err(),
                    "run {index}: {fault}"
                );
            }
        }
        let mut other = file;
        other.insert(
            String::from("movie"),
            toml::Value::String(String::from("100")),
        );
        assert!(listed(&root, &other.to_string(), "any").is_err());
    }

    #[test]
    fn frames_that_pass_count_each_frame_once() {
        // Two overlapping runs and one beside them: frames 11 to 30 and 41 to 45.
        let runs = [run(10, 15, false), run(20, 10, false), run(40, 5, false)];
        assert_eq!(passing(&runs, 0, 100), 25);
        // A stretch counts only its own frames, after its seed through its end.
        assert_eq!(passing(&runs, 20, 42), 12);
        assert_eq!(passing(&[run(10, 0, false)], 0, 100), 0);
        assert_eq!(passing(&[], 0, 100), 0);
    }

    #[test]
    fn only_a_run_from_power_on_counts_from_the_start() {
        let seeded = run(10, 50, false);
        let none = from_start(std::slice::from_ref(&seeded), 100);
        assert_eq!(none.held, 0);
        assert!(none.stop.is_some());
        assert_eq!(
            from_start(&[seeded, run(0, 7, true), run(0, 3, true)], 100),
            FromStart {
                held: 7,
                stop: Some(String::from("a difference"))
            }
        );
        assert_eq!(
            from_start(&[run(0, 120, true)], 100),
            FromStart {
                held: 100,
                stop: None
            }
        );
    }

    #[test]
    fn a_bar_fills_its_share_of_the_total() {
        let filled = |bar: &str| bar.chars().filter(|&cell| cell == '█').count();
        let empty = bar(0, 15_196, BAR);
        assert_eq!((filled(&empty), empty.chars().count()), (0, 40));
        // 2,136 of 15,196 is 5.6 cells, rounded down.
        assert_eq!(filled(&bar(2_136, 15_196, BAR)), 5);
        assert!(bar(2_136, 15_196, BAR).starts_with("█████░"));
        // One frame shows; one frame short of the whole doesn't fill the bar.
        assert_eq!(filled(&bar(1, 15_196, BAR)), 1);
        assert_eq!(filled(&bar(15_195, 15_196, BAR)), 39);
        assert_eq!(bar(15_196, 15_196, BAR), "█".repeat(40));
        assert_eq!(filled(&bar(0, 0, BAR)), 0);
    }

    #[test]
    fn the_bars_line_up_with_their_counts() {
        assert_eq!(
            bars(&[("Passing", 2_136), ("From start", 0)], 15_196),
            format!(
                "```text\nPassing     {}   2,136 of 15,196   14.1%\nFrom start  {}       0 of \
                 15,196    0.0%\n```\n",
                bar(2_136, 15_196, BAR),
                bar(0, 15_196, BAR)
            )
        );
    }

    #[test]
    fn percentages_round_to_a_tenth() {
        assert_eq!(percent(607, 15_196), "4.0%");
        assert_eq!(percent(0, 15_196), "0.0%");
        assert_eq!(percent(15_196, 15_196), "100.0%");
    }
}
