//! The README: the project's thesis and its progress over the milestone's movie, counted in
//! frames from the reviewed played runs of [`PLAYED`]. The headline shows progress across the
//! movie. Each stretch shows its place in the movie and its progress from power-on.
//!
//! A lock row's cut c is movie frame f + c, f its seed's movie frame, or 0 for a row from
//! power-on, so a row's cut 0 is the frame after the previous row's last cut. A run seeded at
//! frame s that held h cuts holds frames s + 1 through s + h. The milestone's frames are 1
//! through the frame of the seed that ends it; frame 0 is power-on.

mod milestone;
mod stretches;

use std::fmt::Write as _;
use std::path::Path;

use self::milestone::Milestone;
use super::seeds::{first_frame, following, row_movie, seed_frame};
use super::{COMMAND, read};
use crate::lock::{Lock, Row, RowId, Start};
use crate::replay::played::declarations::{Listed, Listing, PATH as PLAYED};

/// The README, from the repository root.
pub(super) const PATH: &str = "README.md";
/// The lock whose rows the runs play.
const LOCK: &str = "scenarios/locks/lsnes.toml";

/// What the project is for: MODEL §1 and RULES, "The goal and its measure".
const THESIS: &str = "**The goal** is to model Super Metroid's behaviour, all of it, as named behaviours, so \
each one can be reproduced exactly, adjusted (the space jump timing), added (respin) or removed \
(lag). In the end only this engine runs. The emulator, running the original, unmodified ROM, is \
the oracle that proves the vanilla behaviours exact: every compared frame cut and output byte \
matches.";

/// A reviewed played run, in movie frames.
#[derive(Clone, Debug, PartialEq, Eq)]
struct Run {
    /// The seed cut's movie frame.
    seed: u64,
    /// The frames held after the seed.
    held: u64,
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
    listed(root, &Listed::read(root)?, movie)
}

/// Checks the reviewed runs against the lock under `root`.
fn listed(root: &Path, listed: &Listed, movie: &str) -> Result<Vec<Run>, String> {
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
    if listing.end >= cuts {
        return Err(wrong(format!(
            "asks for cuts {} to {}, outside the {cuts} it can play back to back",
            listing.start, listing.end
        )));
    }
    let run = Run {
        seed: first.saturating_add(listing.start),
        held: listing.held,
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

/// The frames held back to back from power-on, with nothing fed back: the longest reviewed run
/// from power-on, within the milestone's `total` frames.
fn from_start(runs: &[Run], total: u64) -> u64 {
    runs.iter()
        .filter(|run| run.power_on)
        .map(|run| run.held.min(total))
        .max()
        .unwrap_or_default()
}

/// `count` of `total` as a percentage to a tenth, rounded down as the bars are: only the whole
/// total shows 100.0%, and a count above none shows at least 0.1%.
fn percent(count: u64, total: u64) -> String {
    let tenths = count
        .min(total)
        .saturating_mul(1000)
        .checked_div(total)
        .unwrap_or(0);
    let tenths = if count == 0 || total == 0 {
        0
    } else if count >= total {
        1000
    } else {
        tenths.clamp(1, 999)
    };
    format!("{}.{}%", tenths / 10, tenths % 10)
}

/// The cells of `cells` that `count` of `total` fills, rounded down: only the whole total fills
/// them all, and a count above none fills at least one, so neither end is overstated.
fn filled(count: u64, total: u64, cells: u64) -> u64 {
    if count == 0 {
        0
    } else {
        count
            .min(total)
            .saturating_mul(cells)
            .checked_div(total)
            .map_or(0, |filled| filled.max(1))
    }
}

/// The share of a stretch held continuously from power-on, on its own scale.
fn power_on_bar(started: u64, total: u64, cells: u64) -> String {
    let started = filled(started, total, cells);
    (0..cells)
        .map(|cell| if cell < started { '█' } else { '░' })
        .collect()
}

/// The cells the whole movie is drawn in, in a monospace font: the headline and each stretch's
/// place in the movie alike, so they share one scale.
const MOVIE: u64 = 40;

/// The first and last movie frame of `total` that cell `cell` of [`MOVIE`] draws; the last comes
/// before the first when the movie has fewer frames than cells and the cell draws none.
fn cell_frames(cell: u64, total: u64) -> (u64, u64) {
    let edge = |cell: u64| cell.saturating_mul(total) / MOVIE;
    (edge(cell).saturating_add(1), edge(cell.saturating_add(1)))
}

/// The movie of `total` frames as one bar: a cell passes when most of its frames pass, and is
/// drawn `█` when most of those are among the first `started` held from power-on, `▓` when they
/// pass from other seeds, and `░` when it doesn't pass.
///
/// Like the stretches' bars, it fills only when the movie is whole: when it isn't and no cell is
/// drawn `░`, the cell with the most frames not held, the first of those, is.
fn headline(runs: &[Run], total: u64, started: u64) -> String {
    let counted: Vec<(u64, u64, u64)> = (0..MOVIE)
        .map(|cell| {
            let (first, last) = cell_frames(cell, total);
            let frames = last.saturating_add(1).saturating_sub(first);
            let held = passing(runs, first.saturating_sub(1), last);
            let from_start = started.min(last).saturating_add(1).saturating_sub(first);
            (frames, held, from_start)
        })
        .collect();
    let mut cells: Vec<char> = counted
        .iter()
        .map(|&(frames, held, from_start)| {
            if held <= frames.saturating_sub(held) {
                '░'
            } else if from_start > held.saturating_sub(from_start) {
                '█'
            } else {
                '▓'
            }
        })
        .collect();
    let whole = passing(runs, 0, total) >= total;
    if !whole && !cells.contains(&'░') {
        let unheld = |&(frames, held, _): &(u64, u64, u64)| frames.saturating_sub(held);
        let most = counted.iter().map(unheld).max().unwrap_or_default();
        if let Some(cell) = counted
            .iter()
            .position(|counts| unheld(counts) == most)
            .and_then(|cell| cells.get_mut(cell))
        {
            *cell = '░';
        }
    }
    cells.into_iter().collect()
}

/// The README for the tree at `root`.
///
/// # Errors
///
/// Why the plan's seeds, the lock or the played runs can't be read, or the first run the lock
/// contradicts.
pub(super) fn page(root: &Path) -> Result<String, String> {
    let milestone = Milestone::read(root)?;
    let total = frame_at(root, milestone.end()?)?;
    let runs = runs(root, &milestone.movie)?;
    let stretches = stretches::stretches(
        &read(&root.join(stretches::STRETCHES))?,
        &milestone.movie,
        total,
    )?;
    let mut out = format!(
        "# sm-decomp\n\n{THESIS}\n\n<!-- Generated by `{COMMAND}` from {PLAYED}, {}, the lock \
         and the plan (`verify::report::readme`); never edit it by hand. -->\n\n## Progress\n\n",
        stretches::STRETCHES
    );
    let _ = writeln!(
        out,
        "The milestone is **{}** in the {}% movie. A frame passes when the engine, playing alone \
         from a recorded seed on the movie's pads, matches every compared field at its cut.\n",
        milestone.name, milestone.movie,
    );
    let started = from_start(&runs, total);
    let _ = writeln!(
        out,
        "```text\n{}  {} · {}\n```\n\n`█` held continuously from power-on \
         · `▓` passes from a later seed · `░` doesn't pass yet\n",
        headline(&runs, total, started),
        percent(started, total),
        percent(passing(&runs, 0, total), total),
    );
    out.push_str(
        "### By stretch\n\nA ✓ marks a stretch completed in a single run from power-on. The first \
         stretch still in progress from power-on is in bold.\n\n",
    );
    out.push_str(&stretches::table(&stretches, started));
    out.push_str("\nBuilding and checking it are in [BUILDING](docs/BUILDING.md).\n");
    Ok(out)
}

#[cfg(test)]
mod tests {
    use super::{
        Listed, MOVIE, PLAYED, Run, cell_frames, from_start, headline, passing, percent,
        power_on_bar,
    };

    /// A run seeded at movie frame `seed` that held `held` frames.
    fn run(seed: u64, held: u64, power_on: bool) -> Run {
        Run {
            seed,
            held,
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

    fn listed(root: &std::path::Path, text: &str, movie: &str) -> Result<Vec<Run>, String> {
        super::listed(root, &Listed::parse(text)?, movie)
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
        // Each fault changes one declaration. The shared reader checks outcome consistency,
        // and the report checks the row and movie coordinates against the lock.
        let faults: [(&str, Fault); 10] = [
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
            ("an unsupported outcome with no dependency", |run| {
                let outcome = run.get_mut("outcome")?.as_table_mut()?;
                *outcome.get_mut("dependency")? = toml::Value::String(String::new());
                Some(())
            }),
            ("a boundary one cut late", |run| {
                shift(run.get_mut("outcome")?.as_table_mut()?, "cut", 1)
            }),
            ("a completed outcome at an unsupported boundary", |run| {
                let outcome = run.get_mut("outcome")?.as_table_mut()?;
                outcome.get("dependency")?;
                outcome.clear();
                outcome.insert(
                    String::from("status"),
                    toml::Value::String(String::from("pass")),
                );
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
        assert_eq!(from_start(std::slice::from_ref(&seeded), 100), 0);
        assert_eq!(
            from_start(&[seeded, run(0, 7, true), run(0, 3, true)], 100),
            7
        );
        assert_eq!(from_start(&[run(0, 120, true)], 100), 100);
    }

    #[test]
    fn a_bar_fills_its_share_of_the_total() {
        let filled = |bar: &str| bar.chars().filter(|&cell| cell == '█').count();
        let empty = power_on_bar(0, 15_196, 40);
        assert_eq!((filled(&empty), empty.chars().count()), (0, 40));
        // 2,136 of 15,196 is 5.6 cells, rounded down.
        assert_eq!(filled(&power_on_bar(2_136, 15_196, 40)), 5);
        assert!(power_on_bar(2_136, 15_196, 40).starts_with("█████░"));
        // One frame shows a cell, and one frame short leaves a cell empty.
        assert_eq!(filled(&power_on_bar(1, 15_196, 40)), 1);
        assert_eq!(filled(&power_on_bar(15_195, 15_196, 40)), 39);
        assert_eq!(power_on_bar(15_196, 15_196, 40), "█".repeat(40));
        assert_eq!(filled(&power_on_bar(0, 0, 40)), 0);
    }

    #[test]
    fn the_movies_cells_draw_each_frame_once_in_order() {
        for total in [15_196, MOVIE, 1_000, 7] {
            let mut next = 1;
            for cell in 0..MOVIE {
                let (first, last) = cell_frames(cell, total);
                assert_eq!(first, next, "{total}: cell {cell}");
                next = last.saturating_add(1);
            }
            assert_eq!(next, total.saturating_add(1), "{total}");
        }
        // Equal shares: 15,196 frames in 40 cells are 379 or 380 frames each.
        assert_eq!(cell_frames(0, 15_196), (1, 379));
        assert_eq!(cell_frames(39, 15_196), (14_817, 15_196));
    }

    #[test]
    fn the_headline_draws_each_cell_as_most_of_its_frames() {
        // 400 frames, 10 a cell: power-on holds 1 to 25, a seed at 50 holds 51 to 64 and one at
        // 100 holds 101 to 104.
        let runs = [run(0, 25, true), run(50, 14, false), run(100, 4, false)];
        let drawn = headline(&runs, 400, 25);
        let cells: Vec<char> = drawn.chars().collect();
        assert_eq!(cells.len(), 40);
        // Cells 0 and 1 are held from power-on; cell 2 holds 5 of its 10, not most of them.
        assert_eq!(cells.get(..3), Some(&['█', '█', '░'][..]));
        // Cell 5 passes from a later seed; cell 6 holds 4 of its 10; cell 10 holds 4 of its 10.
        assert_eq!(cells.get(5..7), Some(&['▓', '░'][..]));
        assert_eq!(cells.get(10), Some(&'░'));
        assert_eq!(drawn.chars().filter(|&cell| cell != '░').count(), 3);
        // Of the frames a cell holds, most from power-on draw it as power-on's.
        assert_eq!(
            headline(&[run(0, 6, true), run(5, 10, false)], 400, 6)
                .chars()
                .next(),
            Some('█')
        );
        assert_eq!(
            headline(&[run(0, 4, true), run(4, 10, false)], 400, 4)
                .chars()
                .next(),
            Some('▓')
        );
        assert_eq!(headline(&[], 400, 0), "░".repeat(40));
        // A tie between power-on's frames and a later seed's draws the lesser claim.
        assert_eq!(
            headline(&[run(0, 5, true), run(5, 5, false)], 400, 5)
                .chars()
                .next(),
            Some('▓')
        );
    }

    #[test]
    fn the_headline_fills_only_when_the_movie_is_whole() {
        // Each cell holds 6 of its 10 frames, most of them, but the movie isn't whole: the cell
        // with the most frames not held, the first, is drawn not passing.
        let most: Vec<Run> = (0..40).map(|cell| run(cell * 10, 6, false)).collect();
        assert_eq!(headline(&most, 400, 0), format!("░{}", "▓".repeat(39)));
        // Cell 3 holds least.
        let mut fewer = most;
        if let Some(third) = fewer.get_mut(3) {
            third.held = 5;
        }
        assert_eq!(headline(&fewer, 400, 0), format!("▓▓▓░{}", "▓".repeat(36)));
        assert_eq!(headline(&[run(0, 400, true)], 400, 400), "█".repeat(40));
    }

    #[test]
    fn percentages_round_down_to_a_tenth() {
        // 3.99%, 34.56%.
        assert_eq!(percent(607, 15_196), "3.9%");
        assert_eq!(percent(5_252, 15_196), "34.5%");
        assert_eq!(percent(0, 15_196), "0.0%");
        assert_eq!(percent(15_196, 15_196), "100.0%");
        // Neither end is overstated: one frame short of whole isn't 100.0%, one frame isn't 0.0%.
        assert_eq!(percent(15_195, 15_196), "99.9%");
        assert_eq!(percent(1, 15_196), "0.1%");
        assert_eq!(percent(0, 0), "0.0%");
    }
}
