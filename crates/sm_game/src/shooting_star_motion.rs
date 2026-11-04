//! Shooting-star motion and drawing for the ending's star field.
//!
//! # Storage
//!
//! The original keeps 40 16-byte star records at $7E:0E0C..$108B, over enemy
//! globals, the enemy drawing queues and actor slots 0-4, all idle during the
//! ending. Each of those bytes has a Session owner. [`StarRecords`] names
//! every star word and maps it to the owner field that holds the same two
//! bytes (the table is on [`StarRecords`]). The records are those fields:
//! nothing is copied, and the Session encodes the same RAM the projected
//! stores wrote.
//!
//! The routines run on the Session lease, as the ending's palette effects do
//! each frame. The lease's external observations are refreshed once when a
//! routine ends, as the projected stores did after each byte; nothing reads
//! them in between.
use crate::actors::codec::record_word_mut;
use crate::actors::state::Actor;
use crate::imported::{Imported, word as table_word};
use crate::session::Session;
use crate::session_access::SessionAccess;
use sm_data::shooting_stars::{TILE_ADDRESS as STAR_TILE_ADDRESS, Tiles};

/// The number of star records.
pub const STARS: usize = 40;
/// The object cursor's offset in OAM.
const OBJECT_CURSOR: usize = 0x220;
const CENTRE: u16 = 128;
const RESET_WAIT: u16 = 32;
const CENTRED: [(Field, u16); 4] = [
    (Field::X, CENTRE),
    (Field::Y, CENTRE),
    (Field::XFraction, 0),
    (Field::YFraction, 0),
];
const STILL: [(Field, u16); 2] = [(Field::XSpeed, 0), (Field::YSpeed, 0)];

/// The lease the star field runs on.
pub trait Lease:
    SessionAccess + crate::records::Records + crate::sprite_drawing::ObjectAccess
{
    /// The star tile table, imported from the loaded image.
    fn star_tiles(&self) -> &Option<Imported<Tiles>>;
}

/// A star record's words, in record order.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Field {
    /// The star's index (low byte), its animation frame (high byte, from
    /// bit 9) and, in bit 15, whether it is waiting to start.
    State,
    X,
    XFraction,
    Y,
    YFraction,
    Timer,
    XSpeed,
    YSpeed,
}
impl Field {
    pub const ALL: [Self; 8] = [
        Self::State,
        Self::X,
        Self::XFraction,
        Self::Y,
        Self::YFraction,
        Self::Timer,
        Self::XSpeed,
        Self::YSpeed,
    ];
}

/// The Session storage of one star word: a word field, or the two byte
/// fields that hold its low and high bytes.
enum Lane<'a> {
    Word(&'a mut u16),
    Bytes(&'a mut u8, &'a mut u8),
}
impl Lane<'_> {
    fn get(&self) -> u16 {
        match self {
            Self::Word(word) => **word,
            Self::Bytes(low, high) => u16::from_le_bytes([**low, **high]),
        }
    }
    fn set(self, value: u16) {
        match self {
            Self::Word(word) => *word = value,
            Self::Bytes(low, high) => [*low, *high] = value.to_le_bytes(),
        }
    }
}

/// Bytes `2 * pair` and `2 * pair + 1` of `bytes`.
fn byte_pair(bytes: &mut [u8], pair: usize) -> Lane<'_> {
    let [low, high] = bytes.as_chunks_mut::<2>().0[pair].each_mut();
    Lane::Bytes(low, high)
}

/// The record word at byte `offset` of `actor`.
fn record_lane(actor: &mut Actor, offset: usize) -> Lane<'_> {
    if offset == 0x2e {
        return Lane::Bytes(&mut actor.bank, &mut actor.hurt_time);
    }
    let Some(word) = record_word_mut(actor, offset) else {
        unreachable!("every other even record offset is a word field")
    };
    Lane::Word(word)
}

/// The 40 star records over the Session fields that hold their bytes.
///
/// Star word `w` is word `8 * star + field` of the 640-byte block:
///
/// | Star words | Original bytes | Session field |
/// | --- | --- | --- |
/// | 0-2 | $0E0C..$0E11 | `actors.compatibility.spare_0e0c[w]` |
/// | 3 | $0E12 | `actors.diagnostics.sprite_contacts_disabled` |
/// | 4 | $0E14 | `actors.compatibility.spare_0e14` |
/// | 5 | $0E16 | `room.entry.elevator_flags` |
/// | 6 | $0E18 | `room.entry.elevator_status` |
/// | 7 | $0E1A | `enemy_projectiles.drop_bias` |
/// | 8 | $0E1C | `actors.controls.shake_request` |
/// | 9 | $0E1E | `actors.drawing.tilemap_pending` |
/// | 10 | $0E20 | `compatibility_enemy_scratch` |
/// | 11-15 | $0E22..$0E2B | `actors.compatibility.scratch[w - 11]` |
/// | 16-25 | $0E2C..$0E3F | `actors.compatibility.spare_0e2c[w - 16]` |
/// | 26 | $0E40 | `actors.combat.last_power` |
/// | 27 | $0E42 | `actors.compatibility.spare_0e42` |
/// | 28 | $0E44 | `actors.controls.phase_counter` |
/// | 29 | $0E46 | `actors.controls.selection_visits` |
/// | 30 | $0E48 | `actors.controls.population_scratch` |
/// | 31 | $0E4A | `actors.controls.compatibility_new_index` |
/// | 32 | $0E4C | `actors.controls.first_free` |
/// | 33 | $0E4E | `actors.controls.population_count` |
/// | 34 | $0E50 | `room.enemy_deaths` |
/// | 35 | $0E52 | `room.enemy_quota` |
/// | 36 | $0E54 | `actors.controls.selected` |
/// | 37 | $0E56 | `actors.controls.compatibility_selected_backup` |
/// | 38 | $0E58 | `actors.controls.compatibility_data_pointer` |
/// | 39-54 | $0E5A..$0E79 | byte pairs of `actors.staging.descriptor_prefix` |
/// | 55 | $0E7A | `actors.staging.descriptor_write_position` |
/// | 56 | $0E7C | `actors.staging.transfer_source` |
/// | 57 | $0E7E | `actors.staging.transfer_destination` |
/// | 58-59 | $0E80..$0E83 | byte pairs of `actors.staging.gap` |
/// | 60-173 | $0E84..$0F67 | `presentation.actor_entry(w - 60)` |
/// | 174-181 | $0F68..$0F77 | `presentation.actor_size(w - 174)` |
/// | 182-319 | $0F78..$108B | `actors.slots[0..=4]`'s record words (`bank` and `hurt_time` at 2Eh) |
pub struct StarRecords<'a>(pub &'a mut Session);
impl StarRecords<'_> {
    fn lane(&mut self, star: usize, field: Field) -> Lane<'_> {
        sm_data::table::check_index(star, STARS);
        let s = &mut *self.0;
        match 8 * star + field as usize {
            word @ 0..=2 => Lane::Word(&mut s.actors.compatibility.spare_0e0c[word]),
            3 => Lane::Word(&mut s.actors.diagnostics.sprite_contacts_disabled),
            4 => Lane::Word(&mut s.actors.compatibility.spare_0e14),
            5 => Lane::Word(&mut s.room.entry.elevator_flags),
            6 => Lane::Word(&mut s.room.entry.elevator_status),
            7 => Lane::Word(&mut s.enemy_projectiles.drop_bias),
            8 => Lane::Word(&mut s.actors.controls.shake_request),
            9 => Lane::Word(&mut s.actors.drawing.tilemap_pending),
            10 => Lane::Word(&mut s.compatibility_enemy_scratch),
            word @ 11..=15 => Lane::Word(&mut s.actors.compatibility.scratch[word - 11]),
            word @ 16..=25 => Lane::Word(&mut s.actors.compatibility.spare_0e2c[word - 16]),
            26 => Lane::Word(&mut s.actors.combat.last_power),
            27 => Lane::Word(&mut s.actors.compatibility.spare_0e42),
            28 => Lane::Word(&mut s.actors.controls.phase_counter),
            29 => Lane::Word(&mut s.actors.controls.selection_visits),
            30 => Lane::Word(&mut s.actors.controls.population_scratch),
            31 => Lane::Word(&mut s.actors.controls.compatibility_new_index),
            32 => Lane::Word(&mut s.actors.controls.first_free),
            33 => Lane::Word(&mut s.actors.controls.population_count),
            34 => Lane::Word(&mut s.room.enemy_deaths),
            35 => Lane::Word(&mut s.room.enemy_quota),
            36 => Lane::Word(&mut s.actors.controls.selected),
            37 => Lane::Word(&mut s.actors.controls.compatibility_selected_backup),
            38 => Lane::Word(&mut s.actors.controls.compatibility_data_pointer),
            word @ 39..=54 => byte_pair(&mut s.actors.staging.descriptor_prefix, word - 39),
            55 => Lane::Word(&mut s.actors.staging.descriptor_write_position),
            56 => Lane::Word(&mut s.actors.staging.transfer_source),
            57 => Lane::Word(&mut s.actors.staging.transfer_destination),
            word @ 58..=59 => byte_pair(&mut s.actors.staging.gap, word - 58),
            word @ 60..=173 => Lane::Word(s.presentation.actor_entry_mut(word - 60)),
            word @ 174..=181 => Lane::Word(s.presentation.actor_size_mut(word - 174)),
            word => {
                let offset = 2 * (word - 182);
                record_lane(&mut s.actors.slots[offset / 64], offset % 64)
            }
        }
    }
    /// Star `star`'s `field`.
    pub fn get(&mut self, star: usize, field: Field) -> u16 {
        self.lane(star, field).get()
    }
    /// Store `value` to star `star`'s `field`.
    pub fn set(&mut self, star: usize, field: Field, value: u16) {
        self.lane(star, field).set(value);
    }
}

fn get<L: Lease + ?Sized>(game: &mut L, star: usize, field: Field) -> u16 {
    StarRecords(game.session_mut()).get(star, field)
}

fn set<L: Lease + ?Sized>(game: &mut L, star: usize, field: Field, value: u16) {
    StarRecords(game.session_mut()).set(star, field, value);
}

/// Store each (field, value) word of star `star`, in order.
fn store_fields<L: Lease + ?Sized>(game: &mut L, star: usize, fields: &[(Field, u16)]) {
    for &(field, value) in fields {
        set(game, star, field, value);
    }
}

/// Clear a star's frame and wait bit, keeping its index.
fn rewind<L: Lease + ?Sized>(game: &mut L, star: usize) {
    let index = get(game, star, Field::State) & 0xff;
    set(game, star, Field::State, index);
}

/// Reset every star record to the centre with its start delay.
pub fn initialize_shooting_stars<L: Lease + ?Sized>(game: &mut L) {
    initialize(game);
}

fn initialize<L: Lease + ?Sized>(game: &mut L) {
    for (star, index) in (0..STARS).zip(0u16..) {
        set(game, star, Field::State, index);
        store_fields(game, star, &STILL);
        let delay = crate::shooting_stars::star(game, star).start_delay;
        set(
            game,
            star,
            Field::Timer,
            if delay == 0 { RESET_WAIT } else { delay },
        );
        if delay != 0 {
            let waiting = get(game, star, Field::State) | 0x8000;
            set(game, star, Field::State, waiting);
        }
        store_fields(game, star, &CENTRED);
    }
    game.words_stored();
}

/// Move every star, then draw the visible ones into OAM, while `enabled`
/// (`shooting_stars_enabled`) is set.
pub fn advance_shooting_stars<L: Lease + ?Sized>(enabled: u16, game: &mut L) {
    if enabled == 0 {
        return;
    }
    advance(game);
}

fn advance<L: Lease + ?Sized>(game: &mut L) {
    for star in 0..STARS {
        let identity = get(game, star, Field::State);
        if (identity as i16) < 0 {
            let timer = get(game, star, Field::Timer).wrapping_sub(1);
            set(game, star, Field::Timer, timer);
            if (timer as i16) < 0 {
                set(game, star, Field::Timer, RESET_WAIT);
                rewind(game, star);
            }
            continue;
        }
        let record = crate::shooting_stars::star(game, usize::from(identity as u8));
        let fast = ((i32::from(identity & 0xff00) - 1024) & 0x8000) == 0;
        let x_step = record.x_acceleration.wrapping_mul(1 + u16::from(fast));
        let x_speed = get(game, star, Field::XSpeed).wrapping_add(x_step);
        set(game, star, Field::XSpeed, x_speed);
        let boost = if fast { record.y_acceleration } else { 0 };
        let y_speed = get(game, star, Field::YSpeed).wrapping_add(boost);
        let y_speed = record.y_acceleration.wrapping_add(y_speed);
        set(game, star, Field::YSpeed, y_speed);
        for (position, fraction, velocity) in [
            (Field::X, Field::XFraction, Field::XSpeed),
            (Field::Y, Field::YFraction, Field::YSpeed),
        ] {
            let speed = get(game, star, velocity);
            let whole = speed.to_ne_bytes()[1] as i8 as u16;
            let (fractional, carry) = (speed << 8).overflowing_add(get(game, star, fraction));
            set(game, star, fraction, fractional);
            let step = whole.wrapping_add(u16::from(carry));
            let moved = get(game, star, position).wrapping_add(step);
            set(game, star, position, moved);
        }
    }
    let mut output = game.object_little_word(OBJECT_CURSOR);
    for star in 0..STARS {
        if (get(game, star, Field::State) as i16) < 0 {
            continue;
        }
        let x_word = get(game, star, Field::X);
        let x = x_word as i16;
        let entry = usize::from(output);
        let y: i16;
        if (x_word >> 8) != 0
            || ((i32::from(x) - 4) & 0xff00) != 0
            || {
                // This overlapping word write precedes even the Y rejection.
                game.object_write_native_slice(entry, (x as u16).wrapping_sub(4));
                let y_word = get(game, star, Field::Y);
                y = y_word as i16;
                (y_word >> 8) != 0
            }
            || ((i32::from(y) - 4) & 0xff00) != 0
        {
            set(game, star, Field::Timer, RESET_WAIT);
            store_fields(game, star, &CENTRED);
            store_fields(game, star, &STILL);
            rewind(game, star);
            continue;
        }
        game.object_write_native_slice(entry + 1, (y as u16).wrapping_sub(4));
        let timer = get(game, star, Field::Timer).wrapping_sub(1);
        set(game, star, Field::Timer, timer);
        if timer as i16 <= 0 {
            let identity = get(game, star, Field::State);
            let frame_delay =
                crate::shooting_stars::star(game, usize::from(identity as u8)).frame_delay;
            set(game, star, Field::Timer, frame_delay);
            set(game, star, Field::State, identity.wrapping_add(512));
        }
        let state = get(game, star, Field::State);
        if (state >> 8) != 0 {
            let frame = ((state & 0xff00) >> 9) as isize;
            let tile = table_word(game.star_tiles(), game, STAR_TILE_ADDRESS, frame);
            game.object_write_native_slice(entry + 2, tile);
            output = output.wrapping_add(4);
        }
    }
    for (lane, byte) in output.to_le_bytes().into_iter().enumerate() {
        game.object_write_byte(OBJECT_CURSOR + lane, byte);
    }
    game.words_stored();
}
