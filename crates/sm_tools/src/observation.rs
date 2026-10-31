//! Typed per-Tick observations, split by domain (docs/SCENARIO_HARNESS.md).
//!
//! Both backends project their own state into an [`Observation`]: the oracle
//! decodes its work RAM, the engine reads its Session. The comparison then
//! needs no shared memory layout. Each domain's field table is generated from
//! the same declaration as its struct, so names, widths and units in reports
//! always match the code. Adding a field is one line in one place.
//!
//! This module holds no addresses and no RAM: projections live with their
//! backends.

use std::fmt;

/// How many bits a field's value occupies.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Width {
    Byte,
    Word,
}

/// How a field's bits are read when shown to a person.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Sign {
    Unsigned,
    /// Two's complement at the field's width.
    Signed,
}

/// What the game does when the value passes the end of its width.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Overflow {
    /// Arithmetic wraps at the width: positions, counters, timers.
    Wraps,
    /// The game keeps the value inside its range: bit sets, identifiers, supplies.
    Bounded,
}

/// When in the Tick a field is sampled.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Phase {
    /// After the Tick's frame has completed: the state the next Tick starts from.
    EndOfTick,
}

/// One row of a domain's field table.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Field {
    pub name: &'static str,
    pub width: Width,
    pub sign: Sign,
    pub overflow: Overflow,
    /// Elements in the field: 1 for a scalar, N for an array.
    pub count: usize,
    pub phase: Phase,
}

impl Field {
    /// `value` as this field's signedness and width read it.
    pub fn display(&self, value: u16) -> i32 {
        match (self.sign, self.width) {
            (Sign::Unsigned, _) => i32::from(value),
            (Sign::Signed, Width::Word) => i32::from(value.cast_signed()),
            (Sign::Signed, Width::Byte) => i32::from(value.to_le_bytes()[0].cast_signed()),
        }
    }
}

/// A value that a field holds: a scalar or an array of scalars.
pub trait Value {
    const COUNT: usize;
    const ZERO: Self;
    /// Visit each element with its index, widened to a word.
    fn elements(&self, visit: &mut dyn FnMut(usize, u16));
}

impl Value for u16 {
    const COUNT: usize = 1;
    const ZERO: Self = 0;
    fn elements(&self, visit: &mut dyn FnMut(usize, u16)) {
        visit(0, *self);
    }
}

impl Value for u8 {
    const COUNT: usize = 1;
    const ZERO: Self = 0;
    fn elements(&self, visit: &mut dyn FnMut(usize, u16)) {
        visit(0, u16::from(*self));
    }
}

impl<const N: usize> Value for [u8; N] {
    const COUNT: usize = N;
    const ZERO: Self = [0; N];
    fn elements(&self, visit: &mut dyn FnMut(usize, u16)) {
        for (index, &element) in self.iter().enumerate() {
            visit(index, u16::from(element));
        }
    }
}

/// A fixed set of fields with a table describing them.
pub trait Record {
    const FIELDS: &'static [Field];
    /// Visit each field element in table order: field index, element index, value.
    fn visit(&self, visit: &mut dyn FnMut(usize, usize, u16));
}

// One declaration produces the struct, its field table and its visitor.
macro_rules! record {
    (
        $(#[$meta:meta])*
        pub struct $name:ident {
            $(
                $(#[$field_meta:meta])*
                $field:ident: $ty:ty => $width:ident $sign:ident $overflow:ident,
            )*
        }
    ) => {
        $(#[$meta])*
        #[derive(Clone, Copy, Debug, PartialEq, Eq)]
        pub struct $name {
            $($(#[$field_meta])* pub $field: $ty,)*
        }
        impl Default for $name {
            fn default() -> Self {
                Self { $($field: <$ty as Value>::ZERO,)* }
            }
        }
        impl Record for $name {
            const FIELDS: &'static [Field] = &[$(Field {
                name: stringify!($field),
                width: Width::$width,
                sign: Sign::$sign,
                overflow: Overflow::$overflow,
                count: <$ty as Value>::COUNT,
                phase: Phase::EndOfTick,
            }),*];
            fn visit(&self, visit: &mut dyn FnMut(usize, usize, u16)) {
                let mut fields = 0usize..;
                $(
                    let field = fields.next().unwrap_or_default();
                    self.$field.elements(&mut |element, value| visit(field, element, value));
                )*
            }
        }
    };
}

record! {
    /// The Tick's controller read and frame counters.
    pub struct Input {
        /// The polled controller 1 word.
        held: u16 => Word Unsigned Bounded,
        /// Buttons newly pressed this Tick.
        pressed: u16 => Word Unsigned Bounded,
        /// Frames the game counted, excluding lag.
        frame: u16 => Word Unsigned Wraps,
        /// Frames including lag frames.
        frame_including_lag: u16 => Word Unsigned Wraps,
        /// Lag frames the game counted before this one.
        missed_frames: u8 => Byte Unsigned Wraps,
    }
}

record! {
    /// Samus's position, motion, pose, supplies, timers and equipment.
    pub struct Samus {
        x: u16 => Word Unsigned Wraps,
        x_fraction: u16 => Word Unsigned Wraps,
        y: u16 => Word Unsigned Wraps,
        y_fraction: u16 => Word Unsigned Wraps,
        /// Vertical speed in pixels, with its fraction; the direction is separate.
        y_speed: u16 => Word Unsigned Wraps,
        y_speed_fraction: u16 => Word Unsigned Wraps,
        pose: u16 => Word Unsigned Bounded,
        /// The pose's facing in the low byte, its movement type in the high byte.
        movement_and_facing: u16 => Word Unsigned Bounded,
        health: u16 => Word Unsigned Bounded,
        missiles: u16 => Word Unsigned Bounded,
        super_missiles: u16 => Word Unsigned Bounded,
        power_bombs: u16 => Word Unsigned Bounded,
        /// Equipped items and beams (bit sets).
        equipment: u16 => Word Unsigned Bounded,
        beams: u16 => Word Unsigned Bounded,
        invincibility: u16 => Word Unsigned Wraps,
        shine_timer: u16 => Word Unsigned Wraps,
        /// The beam charge counter.
        charge: u16 => Word Unsigned Wraps,
    }
}

record! {
    /// The layer positions the screen shows.
    pub struct Camera {
        layer1_x: u16 => Word Unsigned Wraps,
        layer1_x_fraction: u16 => Word Unsigned Wraps,
        layer1_y: u16 => Word Unsigned Wraps,
        layer1_y_fraction: u16 => Word Unsigned Wraps,
        layer2_x: u16 => Word Unsigned Wraps,
        layer2_y: u16 => Word Unsigned Wraps,
        /// How far ahead of Samus the camera looks, as a table index.
        distance_index: u16 => Word Unsigned Bounded,
    }
}

record! {
    /// Where the game is: area, room, room state, door and the room size.
    pub struct Room {
        area: u16 => Word Unsigned Bounded,
        /// The room's and room state's identifying keys.
        room: u16 => Word Unsigned Bounded,
        state: u16 => Word Unsigned Bounded,
        /// The door the room was entered by.
        door: u16 => Word Unsigned Bounded,
        width: u16 => Word Unsigned Bounded,
        height: u16 => Word Unsigned Bounded,
    }
}

record! {
    /// What the player has achieved: events, bosses, items, doors, capacities.
    pub struct Progress {
        events: [u8; 8] => Byte Unsigned Bounded,
        bosses: [u8; 8] => Byte Unsigned Bounded,
        items: [u8; 64] => Byte Unsigned Bounded,
        doors: [u8; 64] => Byte Unsigned Bounded,
        collected_equipment: u16 => Word Unsigned Bounded,
        collected_beams: u16 => Word Unsigned Bounded,
        max_health: u16 => Word Unsigned Bounded,
        max_missiles: u16 => Word Unsigned Bounded,
    }
}

record! {
    /// One occupied enemy slot.
    pub struct Actor {
        slot: u16 => Word Unsigned Bounded,
        /// The species: its enemy definition key.
        species: u16 => Word Unsigned Bounded,
        x: u16 => Word Unsigned Wraps,
        x_fraction: u16 => Word Unsigned Wraps,
        y: u16 => Word Unsigned Wraps,
        y_fraction: u16 => Word Unsigned Wraps,
        health: u16 => Word Unsigned Bounded,
        ai_handlers: u16 => Word Unsigned Bounded,
        instruction: u16 => Word Unsigned Bounded,
        frozen: u16 => Word Unsigned Wraps,
        invincibility: u16 => Word Unsigned Wraps,
        /// The species' five AI state words.
        ai_a: u16 => Word Unsigned Wraps,
        ai_b: u16 => Word Unsigned Wraps,
        ai_c: u16 => Word Unsigned Wraps,
        ai_d: u16 => Word Unsigned Wraps,
        ai_e: u16 => Word Unsigned Wraps,
    }
}

record! {
    /// One sound effect request waiting in its library's queue.
    pub struct Sound {
        library: u8 => Byte Unsigned Bounded,
        sound: u8 => Byte Unsigned Bounded,
    }
}

record! {
    /// One music request waiting in the music queue.
    pub struct Music {
        track: u16 => Word Unsigned Bounded,
        delay: u16 => Word Unsigned Wraps,
    }
}

/// The ordered enemy list.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct Actors {
    /// Occupied slots in slot order.
    pub enemies: Vec<Actor>,
}

/// Requests the game has made and not yet handed to the sound driver.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct Effects {
    /// Sound effect requests, by library then queue order.
    pub sounds: Vec<Sound>,
    /// Music requests in queue order.
    pub music: Vec<Music>,
}

/// A domain's table: its fields, under a list name when entries repeat.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Table {
    pub list: Option<&'static str>,
    pub fields: &'static [Field],
}

/// The observation domains, in comparison order.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub enum Domain {
    Input,
    Samus,
    Camera,
    Room,
    Progress,
    Actors,
    Effects,
}

impl Domain {
    pub const ALL: [Domain; 7] = [
        Domain::Input,
        Domain::Samus,
        Domain::Camera,
        Domain::Room,
        Domain::Progress,
        Domain::Actors,
        Domain::Effects,
    ];

    pub fn name(self) -> &'static str {
        match self {
            Domain::Input => "input",
            Domain::Samus => "samus",
            Domain::Camera => "camera",
            Domain::Room => "room",
            Domain::Progress => "progress",
            Domain::Actors => "actors",
            Domain::Effects => "effects",
        }
    }

    pub fn tables(self) -> &'static [Table] {
        match self {
            Domain::Input => &[Table {
                list: None,
                fields: Input::FIELDS,
            }],
            Domain::Samus => &[Table {
                list: None,
                fields: Samus::FIELDS,
            }],
            Domain::Camera => &[Table {
                list: None,
                fields: Camera::FIELDS,
            }],
            Domain::Room => &[Table {
                list: None,
                fields: Room::FIELDS,
            }],
            Domain::Progress => &[Table {
                list: None,
                fields: Progress::FIELDS,
            }],
            Domain::Actors => &[Table {
                list: Some("enemies"),
                fields: Actor::FIELDS,
            }],
            Domain::Effects => &[
                Table {
                    list: Some("sounds"),
                    fields: Sound::FIELDS,
                },
                Table {
                    list: Some("music"),
                    fields: Music::FIELDS,
                },
            ],
        }
    }
}

/// Where a value sits in its domain.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Location {
    /// A field element of a single-record domain.
    Field { field: usize, element: usize },
    /// A list's length.
    Length { table: usize },
    /// A field element of a list entry.
    Entry {
        table: usize,
        entry: usize,
        field: usize,
        element: usize,
    },
}

/// A domain value's address: printable as e.g. `samus.x`, `progress.items[3]`,
/// `actors.enemies[2].health` or `effects.sounds.len`.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Path {
    pub domain: Domain,
    pub location: Location,
}

impl Path {
    /// The table row describing this value; a list length has none.
    pub fn field(&self) -> Option<&'static Field> {
        let tables = self.domain.tables();
        match self.location {
            Location::Field { field, .. } => tables[0].fields.get(field),
            Location::Length { .. } => None,
            Location::Entry { table, field, .. } => tables[table].fields.get(field),
        }
    }

    /// The value as its field's signedness reads it.
    pub fn display(&self, value: u16) -> i32 {
        self.field()
            .map_or(i32::from(value), |field| field.display(value))
    }
}

impl fmt::Display for Path {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let tables = self.domain.tables();
        let element = |f: &mut fmt::Formatter<'_>, field: &Field, element: usize| {
            if field.count > 1 {
                write!(f, "[{element}]")
            } else {
                Ok(())
            }
        };
        write!(f, "{}", self.domain.name())?;
        match self.location {
            Location::Field {
                field,
                element: index,
            } => {
                let field = &tables[0].fields[field];
                write!(f, ".{}", field.name)?;
                element(f, field, index)
            }
            Location::Length { table } => {
                write!(f, ".{}.len", tables[table].list.unwrap_or("entries"))
            }
            Location::Entry {
                table,
                entry,
                field,
                element: index,
            } => {
                let list = tables[table].list.unwrap_or("entries");
                let field = &tables[table].fields[field];
                write!(f, ".{list}[{entry}].{}", field.name)?;
                element(f, field, index)
            }
        }
    }
}

/// One Tick's observation of every domain.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct Observation {
    pub input: Input,
    pub samus: Samus,
    pub camera: Camera,
    pub room: Room,
    pub progress: Progress,
    pub actors: Actors,
    pub effects: Effects,
}

impl Location {
    // Visit order within a domain: by table, then a list's length before its
    // entries, then entry, field and element.
    fn order(self) -> (usize, usize, usize, usize, usize) {
        match self {
            Location::Field { field, element } => (0, 0, 0, field, element),
            Location::Length { table } => (table, 0, 0, 0, 0),
            Location::Entry {
                table,
                entry,
                field,
                element,
            } => (table, 1, entry, field, element),
        }
    }
}

fn visit_record<T: Record>(record: &T, visit: &mut dyn FnMut(Location, u16)) {
    record.visit(&mut |field, element, value| visit(Location::Field { field, element }, value));
}

fn visit_list<T: Record>(table: usize, entries: &[T], visit: &mut dyn FnMut(Location, u16)) {
    // A list longer than a word can count still differs in its entries.
    visit(
        Location::Length { table },
        u16::try_from(entries.len()).unwrap_or(u16::MAX),
    );
    for (entry, record) in entries.iter().enumerate() {
        record.visit(&mut |field, element, value| {
            visit(
                Location::Entry {
                    table,
                    entry,
                    field,
                    element,
                },
                value,
            )
        });
    }
}

impl Observation {
    /// Visit every value of `domain` in table order.
    pub fn visit(&self, domain: Domain, visit: &mut dyn FnMut(Location, u16)) {
        match domain {
            Domain::Input => visit_record(&self.input, visit),
            Domain::Samus => visit_record(&self.samus, visit),
            Domain::Camera => visit_record(&self.camera, visit),
            Domain::Room => visit_record(&self.room, visit),
            Domain::Progress => visit_record(&self.progress, visit),
            Domain::Actors => visit_list(0, &self.actors.enemies, visit),
            Domain::Effects => {
                visit_list(0, &self.effects.sounds, visit);
                visit_list(1, &self.effects.music, visit);
            }
        }
    }

    fn values(&self, domain: Domain) -> Vec<(Location, u16)> {
        let mut values = Vec::new();
        self.visit(domain, &mut |location, value| {
            values.push((location, value))
        });
        values
    }

    /// A 64-bit FNV-1a hash of `domain`'s values and their locations.
    pub fn hash(&self, domain: Domain) -> u64 {
        const OFFSET: u64 = 0xcbf2_9ce4_8422_2325;
        const PRIME: u64 = 0x0000_0100_0000_01b3;
        let mut hash = OFFSET;
        self.visit(domain, &mut |location, value| {
            let (a, b, c, d, e) = location.order();
            for part in [a, b, c, d, e] {
                for byte in u64::try_from(part).unwrap_or(u64::MAX).to_le_bytes() {
                    hash = (hash ^ u64::from(byte)).wrapping_mul(PRIME);
                }
            }
            for byte in value.to_le_bytes() {
                hash = (hash ^ u64::from(byte)).wrapping_mul(PRIME);
            }
        });
        hash
    }

    /// Every value of `domain` that differs between `self` (the oracle's) and
    /// `engine`, in table order. Entries past the shorter list are missing on
    /// one side.
    pub fn differences(&self, engine: &Observation, domain: Domain) -> Vec<Difference> {
        let mut oracle = self.values(domain).into_iter().peekable();
        let mut engine = engine.values(domain).into_iter().peekable();
        let mut differences = Vec::new();
        let mut push = |location, oracle, engine| {
            differences.push(Difference {
                path: Path { domain, location },
                oracle,
                engine,
            })
        };
        loop {
            match (oracle.peek().copied(), engine.peek().copied()) {
                (None, None) => break,
                (Some((left, mine)), Some((right, theirs))) if left == right => {
                    if mine != theirs {
                        push(left, Some(mine), Some(theirs));
                    }
                    oracle.next();
                    engine.next();
                }
                (Some((left, value)), Some((right, _))) if left.order() < right.order() => {
                    push(left, Some(value), None);
                    oracle.next();
                }
                (Some((left, value)), None) => {
                    push(left, Some(value), None);
                    oracle.next();
                }
                (_, Some((right, value))) => {
                    push(right, None, Some(value));
                    engine.next();
                }
            }
        }
        differences
    }
}

/// One value that differs between the two projections. A side's value is
/// `None` when its list has no such entry.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Difference {
    pub path: Path,
    pub oracle: Option<u16>,
    pub engine: Option<u16>,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn tables_follow_the_declarations() {
        let names: Vec<_> = Samus::FIELDS.iter().map(|field| field.name).collect();
        assert_eq!(names[..4], ["x", "x_fraction", "y", "y_fraction"]);
        let items = Progress::FIELDS
            .iter()
            .find(|field| field.name == "items")
            .expect("items field");
        assert_eq!((items.count, items.width), (64, Width::Byte));
        assert_eq!(Input::FIELDS[4].width, Width::Byte);
    }

    #[test]
    fn equal_observations_hash_equal_and_have_no_differences() {
        let observation = Observation::default();
        for domain in Domain::ALL {
            assert_eq!(observation.hash(domain), observation.clone().hash(domain));
            assert!(observation.differences(&observation, domain).is_empty());
        }
    }

    #[test]
    fn a_changed_field_is_named_with_both_values() {
        let oracle = Observation::default();
        let mut engine = oracle.clone();
        engine.samus.x_fraction = 0x8000;
        assert_ne!(oracle.hash(Domain::Samus), engine.hash(Domain::Samus));
        assert_eq!(oracle.hash(Domain::Camera), engine.hash(Domain::Camera));
        let differences = oracle.differences(&engine, Domain::Samus);
        assert_eq!(differences.len(), 1);
        assert_eq!(differences[0].path.to_string(), "samus.x_fraction");
        assert_eq!(
            (differences[0].oracle, differences[0].engine),
            (Some(0), Some(0x8000))
        );
    }

    #[test]
    fn array_elements_and_list_entries_have_their_own_paths() {
        let mut oracle = Observation::default();
        let mut engine = oracle.clone();
        engine.progress.items[3] = 1;
        let differences = oracle.differences(&engine, Domain::Progress);
        assert_eq!(differences[0].path.to_string(), "progress.items[3]");

        oracle.actors.enemies.push(Actor::default());
        engine.actors.enemies.push(Actor {
            health: 20,
            ..Actor::default()
        });
        engine.actors.enemies.push(Actor::default());
        let names: Vec<_> = oracle
            .differences(&engine, Domain::Actors)
            .iter()
            .map(|difference| difference.path.to_string())
            .collect();
        assert_eq!(names[0], "actors.enemies.len");
        assert_eq!(names[1], "actors.enemies[0].health");
        assert!(
            names[2..]
                .iter()
                .all(|name| name.starts_with("actors.enemies[1]."))
        );
    }

    #[test]
    fn swapped_effect_lists_do_not_hash_alike() {
        let mut oracle = Observation::default();
        let mut engine = oracle.clone();
        oracle.effects.sounds.push(Sound::default());
        engine.effects.music.push(Music::default());
        assert_ne!(oracle.hash(Domain::Effects), engine.hash(Domain::Effects));
    }

    #[test]
    fn signed_fields_display_as_signed() {
        let field = Field {
            name: "speed",
            width: Width::Word,
            sign: Sign::Signed,
            overflow: Overflow::Wraps,
            count: 1,
            phase: Phase::EndOfTick,
        };
        assert_eq!(field.display(0xffff), -1);
        let byte = Field {
            width: Width::Byte,
            ..field
        };
        assert_eq!(byte.display(0x00ff), -1);
    }
}
