//! The Session-space comparison (docs/handoff/session-compare.md).
//!
//! Each paired frame, the verifier decodes the oracle's RAM into a Session
//! and hands it over with the engine's own Session, compared directly, with
//! the fields RAM cannot carry (`Uncarried`) taken from the oracle. This
//! compares them field by field and names the first differing field path.
//! It is report-only: the whole-RAM comparison stays authoritative, and
//! nothing here sets a frame's pass or fail.
//!
//! Both Sessions are the frame's end. Game-side code can still write RAM
//! after the engine's last lease (settlement), so the verifier compares the
//! Session the engine's next lease takes up, and counts the fields written
//! after the last lease separately: they are timing, not divergence.
//!
//! Console text names ticks, field paths and RAM addresses, never values,
//! because values can be derived from the ROM.

use sm_game::session::Session;
use std::fmt::Debug;
use std::ops::Range;
use std::time::Duration;

/// Lists every `Session` field the comparison covers, and generates the
/// per-field comparison and the field's debug text from that one list.
macro_rules! session_fields {
    ($($field:ident => $group:ident),* $(,)?) => {
        /// The Session's compared fields, in declaration order.
        pub const FIELDS: &[&str] = &[$(stringify!($field)),*];

        /// Each field's lock stream group, in `FIELDS` order.
        const FIELD_GROUPS: &[Group] = &[$(Group::$group),*];

        /// The digest of the field's debug text.
        fn field_digest(session: &Session, index: usize) -> u64 {
            let mut at = 0;
            $(
                if at == index {
                    return text_digest(&session.$field);
                }
                at += 1;
            )*
            unreachable!("field index {index} of {at}")
        }

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
    game_loop => Loop,
    actor_identity => Actors,
    actors => Actors,
    botwoon_death => Actors,
    timing => Loop,
    fx => Display,
    fx_control_ownership => Display,
    fx_sources => Display,
    options => Loop,
    flow => Loop,
    menu => Menus,
    menu_window => Menus,
    menu_screens => Menus,
    prepared_dialogue => Menus,
    control => Loop,
    gameplay_flags => Room,
    flag_compatibility => Room,
    sprite_effects => Display,
    sprite_compatibility => Display,
    presentation => Display,
    projectiles => Player,
    enemy_projectiles => Actors,
    player => Player,
    minimap => Menus,
    saved_maps => Save,
    saved_player => Save,
    compressed_maps => Save,
    pause_background => Menus,
    ending_tilemap => Menus,
    inventory => Player,
    mechanisms => Room,
    room => Room,
    queued_tilemaps => Display,
    feedback => Display,
    hdma => Display,
    messages => Menus,
    spawn_scratch => Scratch,
    compatibility_enemy_scratch => Scratch,
    shared_sequence => Player,
    multiply_scratch => Scratch,
    pointer_scratch => Scratch,
    wavy_samus => Player,
);

/// The lock's Session streams (docs/handoff/session-compare.md, slice 5):
/// the top-level Session fields in groups, one running digest each.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Group {
    /// The game loop, timing, options, flow and control.
    Loop,
    /// The actors, their identities and the enemy projectiles.
    Actors,
    /// Samus, her projectiles and the inventory.
    Player,
    /// The room, its mechanisms and the gameplay flags.
    Room,
    /// The presentation, FX, HDMA, sprite effects and queued transfers.
    Display,
    /// The menus, messages, the minimap and the tilemap buffers.
    Menus,
    /// The save file's copies.
    Save,
    /// The scratch words the Session owns.
    Scratch,
}

impl Group {
    pub const ALL: [Self; 8] = [
        Self::Loop,
        Self::Actors,
        Self::Player,
        Self::Room,
        Self::Display,
        Self::Menus,
        Self::Save,
        Self::Scratch,
    ];

    /// The group's lock stream name.
    pub fn stream(self) -> &'static str {
        match self {
            Self::Loop => "session_loop",
            Self::Actors => "session_actors",
            Self::Player => "session_player",
            Self::Room => "session_room",
            Self::Display => "session_display",
            Self::Menus => "session_menus",
            Self::Save => "session_save",
            Self::Scratch => "session_scratch",
        }
    }

    fn index(self) -> usize {
        Self::ALL
            .iter()
            .position(|&group| group == self)
            .expect("listed group")
    }
}

/// A digest of text written in pieces, equal to the digest of the whole
/// text however it was split.
struct TextDigest {
    digest: crate::trace::Digest,
    pending: [u8; 8],
    filled: usize,
}

impl std::fmt::Write for TextDigest {
    fn write_str(&mut self, text: &str) -> std::fmt::Result {
        use std::hash::Hasher as _;
        for &byte in text.as_bytes() {
            self.pending[self.filled] = byte;
            self.filled += 1;
            if self.filled == self.pending.len() {
                self.digest.write(&self.pending);
                self.filled = 0;
            }
        }
        Ok(())
    }
}

/// The digest of a value's debug text: the canonical form of a decoded
/// Session field, which names no value outside the digest.
fn text_digest(value: &impl Debug) -> u64 {
    use std::fmt::Write as _;
    use std::hash::Hasher as _;
    let mut text = TextDigest {
        digest: crate::trace::Digest::default(),
        pending: [0; 8],
        filled: 0,
    };
    write!(text, "{value:?}").expect("digest write");
    text.digest.write(&text.pending[..text.filled]);
    text.digest.finish()
}

/// The Session streams of a run's Ticks: per Tick, one digest per `Group`
/// of its fields' debug texts. A field's text is digested again only on the
/// Ticks it changes.
#[derive(Default)]
pub struct SessionStreams {
    previous: Option<(Session, Vec<u64>)>,
}

impl SessionStreams {
    /// The Tick's digest of each group, in `Group::ALL` order.
    pub fn digests(&mut self, session: Session) -> [u64; Group::ALL.len()] {
        let fields = match self.previous.take() {
            Some((previous, mut fields)) => {
                for index in differing(&session, &previous) {
                    fields[index] = field_digest(&session, index);
                }
                fields
            }
            None => (0..FIELDS.len())
                .map(|index| field_digest(&session, index))
                .collect(),
        };
        use std::hash::Hasher as _;
        let mut groups = [crate::trace::Digest::default().finish(); Group::ALL.len()];
        for (&group, &digest) in FIELD_GROUPS.iter().zip(&fields) {
            let running = &mut groups[group.index()];
            *running = crate::trace::fold(*running, digest);
        }
        self.previous = Some((session, fields));
        groups
    }
}

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

/// The Session fields RAM cannot carry (docs/handoff/session-compare.md, slice
/// 4). The engine's Session is compared directly and the oracle's is decoded
/// from its RAM, so the oracle's value of these fields is the decoder's, and
/// a difference in them means nothing: the comparison gives the engine's
/// Session the oracle's value of each before comparing.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Uncarried {
    /// `actor_identity`: the actors' lineage and palette residency, which
    /// the engine keeps beside the actor slots and RAM never holds.
    ActorIdentity,
    /// `enemy_projectiles.spawn`'s rectangle, variant, point and drop
    /// definition: the spawn context Game holds between leases
    /// (`export_native_spawn`), which no RAM byte carries.
    NativeSpawn,
    /// `control.primary` and `control.secondary` ($0612..0613): snesrev's
    /// coroutine state, which a faithful oracle never writes.
    Coroutines,
    /// `messages.delay` and `room.unpause_queue_tail` ($077E..0781): the
    /// engine keeps them in snesrev's counter RAM, which the original leaves
    /// unused.
    SnesrevCounters,
}

impl Uncarried {
    pub const ALL: [Self; 4] = [
        Self::ActorIdentity,
        Self::NativeSpawn,
        Self::Coroutines,
        Self::SnesrevCounters,
    ];

    /// Give `engine` the oracle's value of the field.
    pub fn take(self, engine: &mut Session, oracle: &Session) {
        match self {
            Self::ActorIdentity => engine.actor_identity.clone_from(&oracle.actor_identity),
            Self::NativeSpawn => {
                let (to, from) = (
                    &mut engine.enemy_projectiles.spawn,
                    &oracle.enemy_projectiles.spawn,
                );
                to.rectangle = from.rectangle;
                to.variant = from.variant;
                to.point = from.point;
                to.drop_definition = from.drop_definition;
            }
            Self::Coroutines => {
                *engine.control.primary_mut() = *oracle.control.primary();
                *engine.control.secondary_mut() = *oracle.control.secondary();
            }
            Self::SnesrevCounters => {
                engine.messages.delay = oracle.messages.delay;
                engine.room.unpause_queue_tail = oracle.room.unpause_queue_tail;
            }
        }
    }

    /// Give `engine` the oracle's value of every field RAM cannot carry.
    pub fn take_all(engine: &mut Session, oracle: &Session) {
        for field in Self::ALL {
            field.take(engine, oracle);
        }
    }
}

/// Why a paired frame had no Session-space comparison.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Skip {
    /// A side's RAM image did not decode into a Session.
    Undecodable,
}

/// Where the engine's frame-end Session came from: the Session its next
/// lease takes up.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum SessionSource {
    /// The retained Session, current with RAM.
    Retained,
    /// The Tick's last lease Session, kept for the next frame, with the RAM
    /// written since its export.
    Kept,
    /// A decode of the engine's RAM, which no Session is current with.
    Imported,
}

/// Per field of `FIELDS`, the Ticks on which it differed, and the first.
#[derive(Default)]
struct FieldTicks {
    ticks: u64,
    by_field: Vec<u64>,
    first: Option<Divergence>,
}

impl FieldTicks {
    /// Count one Tick's differing fields; the first Tick's path is named.
    /// The first Divergence, when this call recorded it.
    fn count(&mut self, tick: i32, engine: &Session, oracle: &Session) -> Option<&Divergence> {
        let differing = differing(engine, oracle);
        let (&field, also) = differing.split_first()?;
        self.ticks += 1;
        self.by_field.resize(FIELDS.len(), 0);
        for &index in &differing {
            self.by_field[index] += 1;
        }
        if self.first.is_some() {
            return None;
        }
        let (engine_text, oracle_text) = debug_texts(engine, oracle, field);
        self.first = Some(Divergence {
            tick,
            path: first_differing_path(FIELDS[field], &engine_text, &oracle_text),
            also: also.to_vec(),
        });
        self.first.as_ref()
    }

    /// `name 3, other 1`, or `none`.
    fn fields(&self) -> String {
        let fields: Vec<String> = FIELDS
            .iter()
            .zip(&self.by_field)
            .filter(|(_, ticks)| **ticks != 0)
            .map(|(name, ticks)| format!("{name} {ticks}"))
            .collect();
        if fields.is_empty() {
            "none".to_owned()
        } else {
            fields.join(", ")
        }
    }

    /// `first at tick 3 in path (and 1 more fields)`, or `none`.
    fn first(&self, what: &str) -> String {
        match &self.first {
            Some(first) => format!(
                "first {what} at tick {} in {}{}",
                first.tick,
                first.path,
                if first.also.is_empty() {
                    String::new()
                } else {
                    format!(" (and {} more fields)", first.also.len())
                }
            ),
            None => format!("no {what}"),
        }
    }
}

/// Which Work RAM bytes the Session's encoder writes (one Session per replay marks
/// them; its spans do not depend on state): slice 2's
/// byte-to-field coverage map. A byte is decoded (the encoder writes it for
/// some Session field), unowned (a listed range the comparison does not own,
/// with its reason), both, held (an engine owner outside the Session codec
/// keeps it, so the Session compare does not see it yet), or neither.
#[derive(Default)]
pub struct Coverage {
    /// Per byte, whether a span covered it; empty until the first span.
    encoded: Vec<bool>,
}

impl Coverage {
    /// Work RAM's size.
    pub const BYTES: usize = 0x20000;
    /// How many ranges of each kind the summary names.
    const NAMED: usize = 12;

    /// Whether any span has been marked.
    pub fn is_marked(&self) -> bool {
        !self.encoded.is_empty()
    }

    /// Mark a span the encoder wrote.
    pub fn mark(&mut self, span: Range<usize>) {
        if self.encoded.is_empty() {
            self.encoded.resize(Self::BYTES, false);
        }
        self.encoded[span].fill(true);
    }

    /// Class every byte against the ranges the comparison does not own and
    /// the ranges engine owners outside the Session hold.
    pub fn classes(&self, unowned: &[Range<usize>], held: &[Held]) -> Classes {
        let mut listed = vec![false; Self::BYTES];
        for range in unowned {
            listed[range.clone()].fill(true);
        }
        // Per byte, the index of its holder in `held_by_owner`, if any.
        let mut holder = vec![None; Self::BYTES];
        let mut classes = Classes::default();
        for entry in held {
            let index = classes
                .held_by_owner
                .iter()
                .position(|(owner, _)| *owner == entry.owner)
                .unwrap_or_else(|| {
                    classes.held_by_owner.push((entry.owner, 0));
                    classes.held_by_owner.len() - 1
                });
            holder[entry.range.clone()].fill(Some(index));
        }
        let unmarked = vec![false; Self::BYTES];
        let encoded = if self.encoded.is_empty() {
            &unmarked
        } else {
            &self.encoded
        };
        for (at, ((&encoded, &listed), &holder)) in
            encoded.iter().zip(&listed).zip(&holder).enumerate()
        {
            let (count, ranges) = match (encoded, listed, holder) {
                (true, false, _) => (&mut classes.decoded, None),
                (false, true, _) => (&mut classes.unowned, None),
                (true, true, _) => (&mut classes.both, Some(&mut classes.both_ranges)),
                (false, false, Some(index)) => {
                    classes.held_by_owner[index].1 += 1;
                    (&mut classes.held, None)
                }
                (false, false, None) => (&mut classes.neither, Some(&mut classes.neither_ranges)),
            };
            *count += 1;
            if let Some(ranges) = ranges {
                match ranges.last_mut() {
                    Some(Range { end, .. }) if *end == at => *end = at + 1,
                    _ => ranges.push(at..at + 1),
                }
            }
        }
        classes
    }

    /// The summary lines, given the ranges the comparison does not own and
    /// the ranges engine owners outside the Session hold.
    pub fn finish(&self, unowned: &[Range<usize>], held: &[Held]) -> Vec<String> {
        let classes = self.classes(unowned, held);
        let named = |ranges: &[Range<usize>]| -> String {
            let mut text: Vec<String> = ranges
                .iter()
                .take(Self::NAMED)
                .map(|range| format!("${:05X}-${:05X}", range.start, range.end - 1))
                .collect();
            if ranges.len() > Self::NAMED {
                text.push(format!("and {} more", ranges.len() - Self::NAMED));
            }
            if text.is_empty() {
                "none".to_owned()
            } else {
                text.join(", ")
            }
        };
        let holders: Vec<String> = classes
            .held_by_owner
            .iter()
            .map(|(owner, bytes)| format!("{owner} {bytes}"))
            .collect();
        vec![
            format!(
                "Session RAM coverage: {} bytes decoded, {} unowned, {} both, {} held outside the Session, {} neither (in {} ranges)",
                classes.decoded,
                classes.unowned,
                classes.both,
                classes.held,
                classes.neither,
                classes.neither_ranges.len()
            ),
            format!(
                "Session RAM held outside the Session: {}",
                if holders.is_empty() {
                    "none".to_owned()
                } else {
                    holders.join(", ")
                }
            ),
            format!(
                "Session RAM neither decoded nor unowned: {}",
                named(&classes.neither_ranges)
            ),
            format!(
                "Session RAM decoded and unowned: {}",
                named(&classes.both_ranges)
            ),
        ]
    }
}

/// A Work RAM range that an engine owner outside the Session codec holds:
/// state the engine keeps (so it is not unowned), which the Session compare
/// does not see until the Session owns it.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Held {
    /// The owner's name, for the summary.
    pub owner: &'static str,
    pub range: Range<usize>,
}

/// Work RAM's bytes by coverage class: counts, and the ranges of the two
/// classes slice 2 must resolve.
#[derive(Debug, Default, PartialEq, Eq)]
pub struct Classes {
    /// Bytes the encoder writes, outside the unowned ranges.
    pub decoded: usize,
    /// Bytes in the unowned ranges the encoder never writes.
    pub unowned: usize,
    /// Bytes the encoder writes inside the unowned ranges.
    pub both: usize,
    /// Bytes neither written nor listed that an owner outside the Session holds.
    pub held: usize,
    /// Bytes neither written, listed nor held.
    pub neither: usize,
    /// The held bytes by owner, in the order the owners are first listed.
    pub held_by_owner: Vec<(&'static str, usize)>,
    pub both_ranges: Vec<Range<usize>>,
    pub neither_ranges: Vec<Range<usize>>,
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
    undecodable: u64,
    /// Ticks by the engine Session's source: retained, kept, imported.
    sources: [u64; 3],
    /// The fields that differed between the Sessions.
    divergence: FieldTicks,
    /// The fields written after the engine's last lease, before the frame's end.
    after_lease: FieldTicks,
    /// The encoder's spans, marked from the replay's first engine Session.
    pub coverage: Coverage,
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
        let before = self.divergence.ticks;
        if let Some(first) = self.divergence.count(tick, engine, oracle) {
            let also_names: Vec<&str> = first.also.iter().map(|&index| FIELDS[index]).collect();
            eprintln!(
                "@{tick}: Session divergence in {}{}",
                first.path,
                if also_names.is_empty() {
                    String::new()
                } else {
                    format!("; also differs: {}", also_names.join(", "))
                }
            );
        }
        let diverged = self.divergence.ticks != before;
        self.pending = Some(diverged);
        diverged
    }

    /// Count where this Tick's engine Session came from.
    pub fn source(&mut self, source: SessionSource) {
        let index = match source {
            SessionSource::Retained => 0,
            SessionSource::Kept => 1,
            SessionSource::Imported => 2,
        };
        self.sources[index] += 1;
    }

    /// Count the fields written after the Tick's last lease: its Session
    /// against the frame-end Session. Timing, not divergence.
    pub fn written_after_lease(&mut self, tick: i32, last_lease: &Session, frame_end: &Session) {
        self.after_lease.count(tick, last_lease, frame_end);
    }

    /// Count a paired frame that had no comparison.
    pub fn skip(&mut self, skip: Skip) {
        self.ticks += 1;
        self.pending = None;
        match skip {
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
        let first_tick = |first: Option<i32>| {
            first.map_or_else(String::new, |tick| format!(", first tick {tick}"))
        };
        let [retained, kept, imported] = self.sources;
        vec![
            format!(
                "Session compare: {} ticks, {} compared, {} divergent; {}; {} undecodable; engine Session {retained} retained, {kept} kept, {imported} imported; {:.2}s comparing",
                self.ticks,
                self.compared,
                self.divergence.ticks,
                self.divergence.first("divergence"),
                self.undecodable,
                self.elapsed.as_secs_f64()
            ),
            format!(
                "Session divergent ticks by field: {}",
                self.divergence.fields()
            ),
            format!(
                "Session written after the last lease: {} ticks; {}; by field: {}",
                self.after_lease.ticks,
                self.after_lease.first("write"),
                self.after_lease.fields()
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
    fn a_text_digest_ignores_how_the_text_was_split() {
        use std::fmt::Write as _;
        use std::hash::Hasher as _;
        let text = "Session { room: Room { width: 4 }, cells: [1, 2, 3] }";
        let mut pieces = TextDigest {
            digest: crate::trace::Digest::default(),
            pending: [0; 8],
            filled: 0,
        };
        for piece in text.split_inclusive(' ') {
            pieces.write_str(piece).expect("write");
        }
        pieces.digest.write(&pieces.pending[..pieces.filled]);
        assert_eq!(
            pieces.digest.finish(),
            crate::trace::Digest::of(text.as_bytes())
        );
        assert_eq!(text_digest(&"ab"), crate::trace::Digest::of(b"\"ab\""));
    }

    #[test]
    fn a_changed_field_changes_only_its_group() {
        let blank = || {
            sm_game::session_encoding::decode(
                &|_| 0,
                0x20000,
                sm_game::room_entry::world::WorldBinding::canonical_legacy(),
            )
        };
        assert_eq!(FIELD_GROUPS.len(), FIELDS.len());
        let mut streams = SessionStreams::default();
        let first = streams.digests(blank());
        assert_eq!(streams.digests(blank()), first);
        assert_eq!(SessionStreams::default().digests(blank()), first);
        let mut moved = blank();
        moved.player.x = 1;
        let changed = streams.digests(moved);
        for (group, (before, after)) in Group::ALL.iter().zip(first.iter().zip(&changed)) {
            assert_eq!(before == after, *group != Group::Player, "{group:?}");
        }
    }

    #[test]
    fn uncarried_fields_take_the_oracles_value() {
        let blank = || {
            sm_game::session_encoding::decode(
                &|_| 0,
                0x20000,
                sm_game::room_entry::world::WorldBinding::canonical_legacy(),
            )
        };
        let oracle = blank();
        let mut engine = blank();
        engine.enemy_projectiles.spawn.variant = 3;
        engine.enemy_projectiles.spawn.point.x = 5;
        *engine.control.primary_mut() = 2;
        engine.messages.delay = 1;
        engine.room.unpause_queue_tail = 4;
        engine.player.x = 6;
        Uncarried::take_all(&mut engine, &oracle);
        let still = differing(&engine, &oracle);
        assert_eq!(
            still.iter().map(|&index| FIELDS[index]).collect::<Vec<_>>(),
            ["player"]
        );
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

    #[test]
    fn coverage_classes_each_byte() {
        let mut coverage = Coverage::default();
        coverage.mark(0x10..0x20);
        coverage.mark(0x44..0x50);
        let held = [
            Held {
                owner: "a",
                range: 0x0c..0x14,
            },
            Held {
                owner: "b",
                range: 0x50..0x54,
            },
            Held {
                owner: "a",
                range: 0x2c..0x34,
            },
        ];
        assert_eq!(
            coverage.classes(&[0x18..0x30, 0x40..0x48], &held),
            Classes {
                decoded: 16,
                unowned: 20,
                both: 12,
                held: 12,
                neither: Coverage::BYTES - 60,
                held_by_owner: vec![("a", 8), ("b", 4)],
                both_ranges: vec![0x18..0x20, 0x44..0x48],
                neither_ranges: vec![0..0x0c, 0x34..0x40, 0x54..Coverage::BYTES],
            }
        );
    }
}
