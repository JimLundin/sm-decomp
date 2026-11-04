//! The Session-space comparison (docs/handoff/session-compare.md).
//!
//! Each paired frame, the verifier decodes the oracle's RAM into a Session
//! and hands it over with the engine's Session, both read through the same
//! codec. This compares them field by field and names the first differing
//! field path. It is report-only: the whole-RAM comparison stays
//! authoritative, and nothing here sets a frame's pass or fail.
//!
//! Console text names ticks and field paths, never values, because values
//! can be derived from the ROM.

use sm_game::session::Session;
use std::fmt::Debug;
use std::time::Duration;

/// Lists every `Session` field the comparison covers, and generates the
/// per-field comparison and the field's debug text from that one list.
macro_rules! session_fields {
    ($($field:ident),* $(,)?) => {
        /// The Session's compared fields, in declaration order.
        pub const FIELDS: &[&str] = &[$(stringify!($field)),*];

        /// The indexes into `FIELDS` of the fields that differ.
        fn differing(engine: &Session, oracle: &Session) -> Vec<usize> {
            // Exhaustive, so a new Session field must be listed or excluded here.
            // `world` is the content binding, not state the RAM holds.
            let Session { $($field: _,)* world: _ } = engine;
            let mut differing = Vec::new();
            let mut index = 0;
            $(
                if engine.$field != oracle.$field {
                    differing.push(index);
                }
                index += 1;
            )*
            debug_assert_eq!(index, FIELDS.len());
            differing
        }

        /// The field's pretty debug text on both sides.
        fn debug_texts(engine: &Session, oracle: &Session, index: usize) -> (String, String) {
            let mut at = 0;
            $(
                if at == index {
                    return (pretty(&engine.$field), pretty(&oracle.$field));
                }
                at += 1;
            )*
            unreachable!("field index {index} of {at}")
        }
    };
}

session_fields!(
    game_loop,
    actor_identity,
    actors,
    botwoon_death,
    timing,
    fx,
    fx_control_ownership,
    fx_sources,
    options,
    flow,
    menu,
    menu_window,
    menu_screens,
    prepared_dialogue,
    control,
    gameplay_flags,
    flag_compatibility,
    sprite_effects,
    sprite_compatibility,
    presentation,
    projectiles,
    enemy_projectiles,
    player,
    minimap,
    saved_maps,
    saved_player,
    compressed_maps,
    inventory,
    mechanisms,
    room,
    queued_tilemaps,
    feedback,
    hdma,
    messages,
    spawn_scratch,
    compatibility_enemy_scratch,
    shared_sequence,
    multiply_scratch,
    pointer_scratch,
    wavy_samus,
);

fn pretty(value: &impl Debug) -> String {
    format!("{value:#?}")
}

fn is_identifier(text: &str) -> bool {
    let mut characters = text.chars();
    characters
        .next()
        .is_some_and(|first| first.is_ascii_alphabetic() || first == '_')
        && characters.all(|rest| rest.is_ascii_alphanumeric() || rest == '_')
}

/// The path, from `root`, of the first line where two pretty debug texts
/// differ: struct fields by name, list and tuple entries by index. Only
/// names and indexes are taken from the text, never values.
fn first_differing_path(root: &str, engine: &str, oracle: &str) -> String {
    // Each open level: its label and how many entries it has had so far.
    let mut open: Vec<(String, usize)> = vec![(root.to_owned(), 0)];
    let (mut engine_lines, mut oracle_lines) = (engine.lines(), oracle.lines());
    loop {
        let (engine_line, oracle_line) = (engine_lines.next(), oracle_lines.next());
        let Some(line) = engine_line.or(oracle_line) else {
            break;
        };
        let text = line.trim_start();
        let depth = (line.len() - text.len()) / 4;
        if text.starts_with(['}', ']', ')']) {
            // A closing line belongs to the level its opening line began.
            open.truncate(depth + 1);
        } else if depth > 0 {
            open.truncate(depth);
            let parent = open.last_mut().expect("the root stays open");
            let index = parent.1;
            parent.1 += 1;
            let label = match text.split_once(": ") {
                Some((name, _)) if is_identifier(name) => format!(".{name}"),
                _ => format!("[{index}]"),
            };
            open.push((label, 0));
        }
        if engine_line != oracle_line {
            break;
        }
    }
    open.into_iter().map(|(label, _)| label).collect()
}

/// Why a paired frame had no Session-space comparison.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Skip {
    /// The engine's Tick released no lease and kept no current Session.
    NoEngineSession,
    /// A side's RAM image did not decode into a Session.
    Undecodable,
}

/// The first Tick at which the Sessions differed.
struct Divergence {
    tick: i32,
    path: String,
    /// Other top-level fields that also differed at that Tick.
    also: Vec<usize>,
}

/// Session-space comparison state for one replay.
#[derive(Default)]
pub struct SessionCompare {
    ticks: u64,
    compared: u64,
    divergent: u64,
    no_engine_session: u64,
    undecodable: u64,
    /// Per field of `FIELDS`, the Ticks on which it differed.
    field_ticks: Vec<u64>,
    first: Option<Divergence>,
    /// This Tick's result, until the RAM verdict arrives.
    pending: Option<bool>,
    both_passed: u64,
    both_failed: u64,
    ram_only: (u64, Option<i32>),
    session_only: (u64, Option<i32>),
    /// Time spent decoding and comparing.
    pub elapsed: Duration,
}

impl SessionCompare {
    /// Compare one Tick's Sessions; true when a field differs.
    pub fn compare(&mut self, tick: i32, engine: &Session, oracle: &Session) -> bool {
        self.ticks += 1;
        self.compared += 1;
        let differing = differing(engine, oracle);
        self.pending = Some(!differing.is_empty());
        let Some((&field, also)) = differing.split_first() else {
            return false;
        };
        self.divergent += 1;
        self.field_ticks.resize(FIELDS.len(), 0);
        for &index in &differing {
            self.field_ticks[index] += 1;
        }
        if self.first.is_none() {
            let (engine_text, oracle_text) = debug_texts(engine, oracle, field);
            let path = first_differing_path(FIELDS[field], &engine_text, &oracle_text);
            let also_names: Vec<&str> = also.iter().map(|&index| FIELDS[index]).collect();
            eprintln!(
                "@{tick}: Session divergence in {path}{}",
                if also.is_empty() {
                    String::new()
                } else {
                    format!("; also differs: {}", also_names.join(", "))
                }
            );
            self.first = Some(Divergence {
                tick,
                path,
                also: also.to_vec(),
            });
        }
        true
    }

    /// Count a paired frame that had no comparison.
    pub fn skip(&mut self, skip: Skip) {
        self.ticks += 1;
        self.pending = None;
        match skip {
            Skip::NoEngineSession => self.no_engine_session += 1,
            Skip::Undecodable => self.undecodable += 1,
        }
    }

    /// Record whether the whole-RAM comparison agreed with this Tick's.
    pub fn agree(&mut self, tick: i32, ram_failed: bool) {
        let Some(session_failed) = self.pending.take() else {
            return;
        };
        let (count, first) = match (session_failed, ram_failed) {
            (false, false) => {
                self.both_passed += 1;
                return;
            }
            (true, true) => {
                self.both_failed += 1;
                return;
            }
            (false, true) => (&mut self.ram_only.0, &mut self.ram_only.1),
            (true, false) => (&mut self.session_only.0, &mut self.session_only.1),
        };
        *count += 1;
        first.get_or_insert(tick);
    }

    /// The summary lines, which name no values.
    pub fn finish(&self) -> Vec<String> {
        let first = match &self.first {
            Some(first) => format!(
                "first divergence at tick {} in {}{}",
                first.tick,
                first.path,
                if first.also.is_empty() {
                    String::new()
                } else {
                    format!(" (and {} more fields)", first.also.len())
                }
            ),
            None => "no divergence".to_owned(),
        };
        let fields: Vec<String> = FIELDS
            .iter()
            .zip(&self.field_ticks)
            .filter(|(_, ticks)| **ticks != 0)
            .map(|(name, ticks)| format!("{name} {ticks}"))
            .collect();
        let first_tick = |first: Option<i32>| {
            first.map_or_else(String::new, |tick| format!(", first tick {tick}"))
        };
        vec![
            format!(
                "Session compare: {} ticks, {} compared, {} divergent; {first}; {} without an engine Session, {} undecodable; {:.2}s comparing",
                self.ticks,
                self.compared,
                self.divergent,
                self.no_engine_session,
                self.undecodable,
                self.elapsed.as_secs_f64()
            ),
            format!(
                "Session divergent ticks by field: {}",
                if fields.is_empty() {
                    "none".to_owned()
                } else {
                    fields.join(", ")
                }
            ),
            format!(
                "Session/RAM agreement: {} both passed, {} both failed, {} RAM-only failures{}, {} Session-only failures{}",
                self.both_passed,
                self.both_failed,
                self.ram_only.0,
                first_tick(self.ram_only.1),
                self.session_only.0,
                first_tick(self.session_only.1)
            ),
        ]
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[derive(Debug)]
    struct Cell {
        level: u16,
        behavior: u8,
    }

    #[derive(Debug)]
    struct Terrain {
        width: u16,
        cells: Vec<Cell>,
    }

    fn terrain(level: u16) -> String {
        let terrain = Terrain {
            width: 4,
            cells: vec![
                Cell {
                    level: 1,
                    behavior: 2,
                },
                Cell { level, behavior: 2 },
            ],
        };
        // The path is read from the debug text alone; the fields only feed it.
        let read = terrain.width + terrain.cells.iter().map(|cell| cell.level).sum::<u16>();
        assert_eq!(read, 5 + level);
        assert!(terrain.cells.iter().all(|cell| cell.behavior == 2));
        pretty(&terrain)
    }

    #[test]
    fn the_path_names_fields_and_indexes_but_no_values() {
        let path = first_differing_path("room", &terrain(7), &terrain(9));
        assert_eq!(path, "room.cells[1].level");
    }

    #[test]
    fn a_scalar_field_is_its_own_path() {
        assert_eq!(first_differing_path("hdma", "1", "2"), "hdma");
    }

    #[test]
    fn a_shorter_list_names_the_list() {
        let short = pretty(&vec![1u8]);
        let long = pretty(&vec![1u8, 2]);
        assert_eq!(first_differing_path("queue", &short, &long), "queue");
    }

    #[test]
    fn every_session_field_is_listed_once() {
        let mut names = FIELDS.to_vec();
        names.sort_unstable();
        names.dedup();
        assert_eq!(names.len(), FIELDS.len());
    }

    #[test]
    fn agreement_counts_each_side() {
        let mut compare = SessionCompare {
            pending: Some(true),
            ..SessionCompare::default()
        };
        compare.agree(1, false);
        compare.pending = Some(false);
        compare.agree(2, true);
        compare.skip(Skip::Undecodable);
        compare.agree(3, true);
        assert_eq!(compare.session_only, (1, Some(1)));
        assert_eq!(compare.ram_only, (1, Some(2)));
        assert_eq!(compare.undecodable, 1);
    }
}
