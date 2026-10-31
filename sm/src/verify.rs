//! Single-threaded replay through the same differential oracle as the UI.
use sm::game_state::Game;
use std::{collections::BTreeSet, ffi::CString, time::Instant};

use sm::verification_checkpoint as checkpoint;
// Explicit so the desktop binary in sm_app can share this module.
#[path = "verify/seed.rs"]
mod seed;

#[derive(Clone, Debug, PartialEq, Eq)]
enum Expectation {
    State(u16),
    Area(u16),
    Room(u16),
    Door(u16),
    Elevator(u16),
    CeresRidley,
    SaveDeathContinue,
    Boss { area: usize, mask: u8 },
    Item { column: usize, mask: u16 },
    Pickup(usize),
    Death,
    Credits,
    CreditsComplete,
}

impl Expectation {
    fn parse(value: &str) -> Result<Self, String> {
        let bad = || format!("unknown or invalid expectation: {value}");
        if value == "death" {
            return Ok(Self::Death);
        }
        if value == "credits" {
            return Ok(Self::Credits);
        }
        if value == "credits-complete" {
            return Ok(Self::CreditsComplete);
        }
        if value == "save-death-continue" {
            return Ok(Self::SaveDeathContinue);
        }
        let (kind, name) = value.split_once(':').ok_or_else(bad)?;
        match kind {
            "state" if !name.is_empty() && name.bytes().all(|byte| byte.is_ascii_digit()) => name
                .parse::<u16>()
                .ok()
                .filter(|&state| state <= 44)
                .map(Self::State)
                .ok_or_else(bad),
            "area" => Ok(Self::Area(match name {
                "crateria" => 0,
                "brinstar" => 1,
                "norfair" => 2,
                "wrecked-ship" => 3,
                "maridia" => 4,
                "tourian" => 5,
                "ceres" => 6,
                _ => return Err(bad()),
            })),
            "encounter" if name == "ceres-ridley" => Ok(Self::CeresRidley),
            "room" | "door" | "elevator"
                if name.len() == 4 && name.bytes().all(|byte| byte.is_ascii_hexdigit()) =>
            {
                let room = u16::from_str_radix(name, 16).map_err(|_| bad())?;
                if room < 0x8000 {
                    return Err(bad());
                }
                Ok(match kind {
                    "door" => Self::Door(room),
                    "elevator" => Self::Elevator(room),
                    _ => Self::Room(room),
                })
            }
            "boss" => {
                // Bomb Torizo has a damage/death episode observer below. Other
                // bosses are defeat flags and need encounter/completion probes.
                let (area, mask) = match name {
                    "bomb-torizo" => (0, 4),
                    "kraid" => (1, 1),
                    "spore-spawn" => (1, 2),
                    "ridley" => (2, 1),
                    "crocomire" => (2, 2),
                    "golden-torizo" => (2, 4),
                    "phantoon" => (3, 1),
                    "draygon" => (4, 1),
                    "botwoon" => (4, 2),
                    "mother-brain" => (5, 2),
                    _ => return Err(bad()),
                };
                Ok(Self::Boss { area, mask })
            }
            "item" => {
                let (column, mask) = match name {
                    "varia" => (0, 1),
                    "spring-ball" => (0, 2),
                    "morph-ball" => (0, 4),
                    "screw-attack" => (0, 8),
                    "gravity" => (0, 0x20),
                    "hi-jump" => (0, 0x100),
                    "space-jump" => (0, 0x200),
                    "bombs" => (0, 0x1000),
                    "speed-booster" => (0, 0x2000),
                    "grapple" => (0, 0x4000),
                    "x-ray" => (0, 0x8000),
                    "wave" => (1, 1),
                    "ice" => (1, 2),
                    "spazer" => (1, 4),
                    "plasma" => (1, 8),
                    "charge" => (1, 0x1000),
                    _ => return Err(bad()),
                };
                Ok(Self::Item { column, mask })
            }
            "pickup" => Ok(Self::Pickup(match name {
                "energy" => 0,
                "missile" => 1,
                "super" => 2,
                "power-bomb" => 3,
                "reserve" => 4,
                _ => return Err(bad()),
            })),
            _ => Err(bad()),
        }
    }

    fn observed(&self, before: &CoverageState, after: &CoverageState, death: bool) -> bool {
        let live = before.same_gameplay_room(after);
        match *self {
            Self::State(state) => after.state == state,
            Self::Area(area) => after.state == 8 && after.area == area,
            Self::Room(room) => after.state == 8 && after.room == room,
            // These require episode history, handled by observe_expectations.
            Self::Door(_)
            | Self::Elevator(_)
            | Self::CeresRidley
            | Self::SaveDeathContinue
            | Self::Boss { area: 0, mask: 4 } => false,
            Self::Boss { area, mask } => {
                live && usize::from(after.area) == area
                    && before.bosses[area] & mask == 0
                    && after.bosses[area] & mask != 0
            }
            Self::Item { column, mask } => {
                live && before.items[column] & mask == 0 && after.items[column] & mask != 0
            }
            Self::Pickup(column) => live && after.capacity[column] > before.capacity[column],
            Self::Death => death,
            // CreditsObject_ProcessOne advances scroll/cursor only for an enabled,
            // nonempty script in state39. A loaded cursor or state39 alone is no hit.
            Self::Credits => {
                before.state == 39
                    && after.state == 39
                    && before.credits_enabled & 0x8000 != 0
                    && before.credits_cursor != 0
                    && (after.credits_cursor != before.credits_cursor
                        || after.credits_scroll != before.credits_scroll)
            }
            // The roll's $8BF6FE instruction disables the credits object,
            // deletes its cursor and starts the post-roll scene at $E0F4.
            Self::CreditsComplete => {
                before.state == 39
                    && after.state == 39
                    && before.credits_enabled & 0x8000 != 0
                    && before.credits_cursor != 0
                    && before.cinematic_function == 0xdb9d
                    && after.credits_enabled & 0x8000 == 0
                    && after.credits_cursor == 0
                    && after.cinematic_function == 0xe0f4
            }
        }
    }
}

#[derive(Clone, Copy, PartialEq, Eq)]
enum SuitProgress {
    AwaitingAward,
    AwaitingAnimation,
    AwaitingControl,
}

struct RequiredExpectation {
    name: String,
    kind: Expectation,
    first_frame: Option<usize>,
    suit_progress: SuitProgress,
}

// Observe the unique Bomb Torizo actor without gameplay writes.
#[derive(Clone, Copy)]
struct BombActor {
    slot: usize,
    definition: u16,
    deleted: bool,
    health: u16,
    phase: u16,
    parameter: u16,
}

impl BombActor {
    fn read(game: &Game) -> Option<Self> {
        use sm::types::EnemyData;
        let mut found = None;
        for slot in 0..32 {
            let base = 0xf78 + slot * core::mem::size_of::<EnemyData>();
            let word = |offset| ReplayCoverage::word(game, base + offset);
            let definition = word(core::mem::offset_of!(EnemyData, enemy_ptr));
            // Bank A0 names EEFF as Bomb Torizo; EF3F is its orb.
            if definition != 0xeeff
                || game
                    .ram()
                    .read_byte(base + core::mem::offset_of!(EnemyData, bank))
                    != 0xaa
            {
                continue;
            }
            // Ambiguity must remain a missing expectation, never pick a slot.
            if found.is_some() {
                return None;
            }
            found = Some(Self {
                slot,
                definition,
                // The death script sets Deleted immediately after its boss flag.
                deleted: word(core::mem::offset_of!(EnemyData, properties)) & 0x200 != 0,
                health: word(core::mem::offset_of!(EnemyData, health)),
                phase: word(core::mem::offset_of!(EnemyData, ai_var_e)),
                parameter: word(core::mem::offset_of!(EnemyData, parameter_2)),
            });
        }
        found
    }
}

#[derive(Clone, Copy, Default)]
struct CoverageState {
    state: u16,
    area: u16,
    room: u16,
    bosses: [u8; 8],
    items: [u16; 2],
    capacity: [u16; 5],
    credits_enabled: u16,
    credits_cursor: u16,
    credits_scroll: u16,
    cinematic_function: u16,
    suit_substate: u16,
    player_handlers: [u16; 3],
    door_exit: u16,
    door_phase: u16,
    door_interlocks: [u16; 2],
    elevator_status: u16,
    elevator_flags: u16,
    ceres_status: u16,
    timer_status: u16,
    actor_definition: u16,
    actor_bank: u8,
    actor_phase: u16,
    bomb_actor: Option<BombActor>,
    selected_save_slot: u16,
    load_station: u16,
    loading_state: u16,
    saved_station: u16,
    saved_area: u16,
    coroutines_idle: bool,
}

impl CoverageState {
    fn same_gameplay_room(&self, other: &Self) -> bool {
        self.state == 8 && other.state == 8 && self.area == other.area && self.room == other.room
    }

    fn read(game: &Game) -> Self {
        let word = |address| ReplayCoverage::word(game, address);
        // ram_map: game_state, area_index, room_ptr, boss_bits_for_area,
        // collected_items/beams, maximum supplies, cinematic_var25/21/10.
        Self {
            state: word(0x998),
            area: word(0x79f),
            room: word(0x79b),
            bosses: core::array::from_fn(|area| game.ram().read_byte(0xd828 + area)),
            items: [word(0x9a4), word(0x9a8)],
            capacity: [0x9c4, 0x9c8, 0x9cc, 0x9d0, 0x9d4].map(word),
            credits_enabled: word(0x19ff),
            credits_cursor: word(0x19f7),
            credits_scroll: word(0x1997),
            cinematic_function: word(0x1f51),
            suit_substate: word(0xdec),
            player_handlers: [word(0xa42), word(0xa44), word(0xa5c)],
            door_exit: word(0x78d),
            door_phase: word(0x99c),
            door_interlocks: [word(0x795), word(0x797)],
            elevator_status: word(0xe18),
            elevator_flags: word(0xe16),
            ceres_status: word(0x93f),
            timer_status: word(0x943),
            // EnemyData slot0: identity, bank and ai_var_A (Ceres var_A).
            actor_definition: word(0xf78),
            actor_bank: game
                .ram()
                .read_byte(0xf78 + core::mem::offset_of!(sm::types::EnemyData, bank)),
            actor_phase: word(0xf78 + core::mem::offset_of!(sm::types::EnemyData, ai_var_a)),
            selected_save_slot: word(0x952),
            load_station: word(0x78b),
            loading_state: word(0xd914),
            saved_station: word(0xd916),
            saved_area: word(0xd918),
            // ram_map lists five coroutine bytes; 0617 is the APU upload flag.
            coroutines_idle: (0x611..0x616).all(|address| game.ram().read_byte(address) == 0),
            bomb_actor: if word(0x998) == 8 && word(0x79f) == 0 && word(0x79b) == 0x9804 {
                BombActor::read(game)
            } else {
                None
            },
        }
    }
}

#[derive(Clone, Copy)]
enum SavePhase {
    Saved,
    Dying,
    GameOver,
    Restored,
    Loading,
    Fading,
}

#[derive(Clone, Copy)]
struct SaveEpisode {
    phase: SavePhase,
    slot: u16,
    area: u16,
    room: u16,
    station: u16,
}

struct DoorEpisode {
    source_area: u16,
    source_room: u16,
    exit: u16,
    destination: Option<u16>,
    elevator_departed: bool,
    elevator: bool,
    arrived: bool,
}

#[derive(Default)]
struct ReplayCoverage {
    states: std::collections::BTreeMap<u16, usize>,
    gameplay_frames: usize,
    rooms: BTreeSet<u16>,
    areas: BTreeSet<u16>,
    previous: CoverageState,
    death_in_progress: bool,
    door_episode: Option<DoorEpisode>,
    elevator_departure: Option<(u16, u16)>,
    ceres_combat: bool,
    bomb_damaged: bool,
    bomb_dying: bool,
    save_episode: Option<SaveEpisode>,
    expected: Vec<RequiredExpectation>,
    enemies: BTreeSet<u16>,
    last_state_room: Option<(u16, u16)>,
    unchanged_frames: usize,
    longest_unchanged: usize,
}

impl ReplayCoverage {
    fn new(game: &Game, expected: Vec<RequiredExpectation>) -> Self {
        // Setup and checkpoint state establish the baseline, never event coverage.
        Self {
            previous: CoverageState::read(game),
            expected,
            ..Self::default()
        }
    }

    fn observe_traversal(&mut self, current: &CoverageState) -> (Option<u16>, Option<u16>) {
        let before = self.previous;
        // Elevator_Func_1 starts a ride through SamusCode_07. The subsequent
        // destination actor is a new actor, so link the ride by room/traversal.
        if before.state != 8
            || !matches!(current.state, 8 | 9)
            || before.room != current.room
            || before.area != current.area
            || current.elevator_status != 1
            || current.elevator_flags == 0
            || current.player_handlers[1..] != [0xe8ec, 0xec14]
        {
            self.elevator_departure = None;
        } else if before.elevator_status == 0 {
            self.elevator_departure = Some((current.area, current.room));
        }
        // room_entry::selection::contact accepts a real exit and enters state9.
        if before.state == 8
            && current.state == 9
            && before.room == current.room
            && before.area == current.area
            && current.door_exit >= 0x8000
            && current.door_phase == 0xe17d
        {
            self.door_episode = Some(DoorEpisode {
                source_area: before.area,
                source_room: before.room,
                exit: current.door_exit,
                destination: None,
                elevator_departed: self.elevator_departure.take()
                    == Some((before.area, before.room)),
                elevator: false,
                arrived: false,
            });
            return (None, None);
        }
        let Some(mut episode) = self.door_episode.take() else {
            return (None, None);
        };
        if current.door_exit != episode.exit || current.room < 0x8000 {
            return (None, None);
        }
        let mut door = None;
        if episode.arrived {
            if !before.same_gameplay_room(current)
                || episode.destination != Some(current.room)
                || current.door_interlocks != [0, 0]
            {
                return (None, None);
            }
        } else {
            if !matches!(
                (before.state, current.state),
                (9, 9..=11) | (10, 10..=11) | (11, 11 | 8)
            ) {
                return (None, None);
            }
            // load_door publishes room before load_header publishes its area.
            // Require the real publication edge, including same-room self-loops.
            if before.state == 11
                && current.state == 11
                && before.door_phase == 0xe2f7
                && current.door_phase == 0xe310
            {
                if episode.destination.is_some() || before.room != episode.source_room {
                    return (None, None);
                }
                episode.destination = Some(current.room);
            } else if current.room != before.room
                || current.room != episode.destination.unwrap_or(episode.source_room)
            {
                return (None, None);
            }
            if current.area != before.area
                && (before.area != episode.source_area
                    || current.state != 11
                    || episode.destination.is_none())
            {
                return (None, None);
            }
            // initialize_door_objects freshly sets2 for an elevator destination.
            if episode.elevator_departed
                && current.state == 11
                && before.elevator_status == 1
                && current.elevator_status == 2
                && current.elevator_flags != 0
                && episode.destination.is_some()
            {
                episode.elevator = true;
            }
            if current.state != 8
                && episode.elevator_departed
                && (current.elevator_flags == 0
                    || current.elevator_status != if episode.elevator { 2 } else { 1 })
            {
                episode.elevator_departed = false;
                episode.elevator = false;
            }
            if current.state == 8 {
                if episode.destination != Some(current.room)
                    || before.room != current.room
                    || before.area != current.area
                    || current.door_interlocks != [0, 0]
                    || current.door_phase != 0xe737
                {
                    return (None, None);
                }
                door = Some(current.room);
                episode.arrived = true;
            }
        }
        if episode.arrived {
            if !episode.elevator {
                return (door, None);
            }
            // Elevator_Func_3 calls Func3b immediately: landing may be2->0.
            if matches!(before.elevator_status, 2 | 3)
                && current.elevator_status == 0
                && current.elevator_flags == 0
                && current.player_handlers == [0xe695, 0xe725, 0xeb52]
            {
                return (door, Some(current.room));
            }
            if !matches!(current.elevator_status, 2 | 3)
                || current.elevator_flags == 0
                || (before.elevator_status == 3 && current.elevator_status == 2)
            {
                return (door, None);
            }
        }
        self.door_episode = Some(episode);
        (door, None)
    }

    fn observe_bomb_torizo(&mut self, current: &CoverageState) -> bool {
        let before = self.previous;
        let pair = before.bomb_actor.zip(current.bomb_actor);
        let same = before.same_gameplay_room(current)
            && current.area == 0
            && current.room == 0x9804
            && pair.is_some_and(|(a, b)| a.slot == b.slot && a.definition == b.definition);
        if !same || before.bosses[0] & 4 != 0 {
            self.bomb_damaged = false;
            self.bomb_dying = false;
            return false;
        }
        let (a, b) = pair.unwrap(); // same requires two matching actor samples.
        if a.deleted || b.deleted {
            // Instr38 and Delete run without an intervening frame. Only a
            // previously observed death may complete on this final actor sample.
            let completed = self.bomb_dying
                && !a.deleted
                && b.deleted
                && a.health == 0
                && b.health == 0
                && current.bosses[0] & 4 != 0;
            self.bomb_damaged = false;
            self.bomb_dying = false;
            return completed;
        }
        if b.health > a.health {
            self.bomb_damaged = false;
            self.bomb_dying = false;
        }
        // Only a damage transition observed inside this replay arms death.
        if a.health > b.health && before.bosses[0] & 4 == 0 && current.bosses[0] & 4 == 0 {
            self.bomb_damaged = true;
        }
        // Torizo_Shot enters the death script. Do not require current_instruction:
        // the interpreter can advance it before the post-frame observation.
        if self.bomb_damaged
            && a.health > 0
            && b.health == 0
            && b.phase == 0xc6ab
            && b.parameter & 0xc000 == 0xc000
            && current.bosses[0] & 4 == 0
        {
            self.bomb_dying = true;
        }
        // Torizo_Instr_38 publishes bit4 later in the same actor/room episode.
        let completed = self.bomb_dying
            && a.health == 0
            && b.health == 0
            && before.bosses[0] & 4 == 0
            && current.bosses[0] & 4 != 0;
        if current.bosses[0] & 4 != 0 {
            self.bomb_damaged = false;
            self.bomb_dying = false;
        }
        completed
    }

    fn observe_save_continue(
        &mut self,
        current: &CoverageState,
        saved_now: bool,
        restored_now: bool,
    ) -> bool {
        let before = self.previous;
        if saved_now {
            self.save_episode = None;
            // A fresh in-game save, never a loaded checkpoint or Ceres auto-save.
            if !restored_now
                && before.same_gameplay_room(current)
                && current.selected_save_slot < 3
                && current.saved_area == current.area
                && current.saved_station == current.load_station
            {
                self.save_episode = Some(SaveEpisode {
                    phase: SavePhase::Saved,
                    slot: current.selected_save_slot,
                    area: current.area,
                    room: current.room,
                    station: current.load_station,
                });
            }
            return false;
        }
        let Some(mut episode) = self.save_episode else {
            return false;
        };
        if current.selected_save_slot != episode.slot {
            self.save_episode = None;
            return false;
        }
        if restored_now {
            // The probe is from GameOverMenu_4_Main's selected Yes load,
            // after checksum validation and player/map/station restoration.
            if matches!(episode.phase, SavePhase::GameOver)
                && before.state == 26
                && current.state == 26
                && current.loading_state != 31
                && current.area == episode.area
                && current.saved_area == episode.area
                && current.load_station == episode.station
                && current.saved_station == episode.station
            {
                episode.phase = SavePhase::Restored;
                self.save_episode = Some(episode);
            } else {
                self.save_episode = None;
            }
            return false;
        }
        let mut valid = true;
        let mut complete = false;
        match episode.phase {
            SavePhase::Saved => {
                if before.state == 8 && current.state == 19 {
                    episode.phase = SavePhase::Dying;
                } else if !(8..=18).contains(&current.state) {
                    valid = false;
                }
            }
            SavePhase::Dying => {
                if (19..=25).contains(&before.state) && current.state == 26 {
                    episode.phase = SavePhase::GameOver;
                } else if !(19..=25).contains(&current.state) || current.state < before.state {
                    valid = false;
                }
            }
            SavePhase::GameOver => valid = current.state == 26,
            SavePhase::Restored => {
                if current.state == 6 {
                    episode.phase = SavePhase::Loading;
                } else {
                    valid = current.state == 5 || (before.state == 26 && current.state == 26);
                }
            }
            SavePhase::Loading => {
                if current.state == 7 {
                    episode.phase = SavePhase::Fading;
                } else {
                    valid = current.state == 6;
                }
            }
            SavePhase::Fading => {
                valid = matches!(current.state, 7 | 8);
                if current.state == 8
                    && (current.area != episode.area
                        || current.room != episode.room
                        || current.load_station != episode.station)
                {
                    valid = false;
                }
                complete = valid
                    && current.state == 8
                    && current.area == episode.area
                    && current.room == episode.room
                    && current.load_station == episode.station
                    && current.loading_state != 31
                    && current.player_handlers == [0xe695, 0xe725, 0xeb52]
                    && current.coroutines_idle;
            }
        }
        self.save_episode = if valid && !complete {
            Some(episode)
        } else {
            None
        };
        complete
    }

    fn observe_ceres(&mut self, current: &CoverageState) -> bool {
        let before = self.previous;
        let same = before.same_gameplay_room(current)
            && current.area == 6
            && current.room == 0xe0b5
            && before.actor_definition != 0
            && before.actor_definition == current.actor_definition
            && before.actor_bank == 0xa6
            && current.actor_bank == 0xa6;
        if !same || current.ceres_status < before.ceres_status || current.actor_phase == 0xa35b {
            self.ceres_combat = false;
        }
        // CeresRidley_Func_11 is active combat, unlike a preloaded getaway.
        if same
            && before.actor_phase == 0xa6e8
            && current.actor_phase == 0xa6e8
            && before.ceres_status == 0
            && current.ceres_status == 0
            && before.bosses[6] & 1 == 0
            && current.bosses[6] & 1 == 0
        {
            self.ceres_combat = true;
        }
        // Ridley_Func_58 finishes the getaway, starts the escape timer and
        // publishes Ceres bit1. This is not the Norfair defeated-boss flag.
        let completed = self.ceres_combat
            && same
            && before.ceres_status == 1
            && current.ceres_status == 2
            && before.bosses[6] & 1 == 0
            && current.bosses[6] & 1 != 0
            && current.timer_status != 0
            && current.actor_phase == 0xaa50;
        if current.ceres_status >= 2 || current.bosses[6] & 1 != 0 {
            self.ceres_combat = false;
        }
        completed
    }

    fn observe_expectations(&mut self, current: CoverageState, frame: usize) {
        let before = self.previous;
        let dying = (19..=25).contains(&current.state);
        if dying && !(19..=25).contains(&before.state) {
            self.death_in_progress = true;
        }
        let death = self.death_in_progress && current.state == 26;
        let (door, elevator) = self.observe_traversal(&current);
        let ceres = self.observe_ceres(&current);
        let bomb = self.observe_bomb_torizo(&current);
        let save_continue = self.observe_save_continue(
            &current,
            sm::verify_probe::last_hit_frame("save.sram.committed") == Some(frame),
            sm::verify_probe::last_hit_frame("save.continue.restored") == Some(frame),
        );
        for expected in &mut self.expected {
            if expected.first_frame.is_some() {
                continue;
            }
            let observed = match expected.kind {
                Expectation::Door(room) => door == Some(room),
                Expectation::Elevator(room) => elevator == Some(room),
                Expectation::CeresRidley => ceres,
                Expectation::SaveDeathContinue => save_continue,
                Expectation::Boss { area: 0, mask: 4 } => bomb,
                _ => expected.kind.observed(&before, &current, death),
            };
            if let Expectation::Item {
                column: 0,
                mask: mask @ (1 | 0x20),
            } = expected.kind
            {
                // The PLM grants equipment before its message and suit HDMA.
                // Stage3 re-ORs that bit; it is not a second acquisition edge.
                if !before.same_gameplay_room(&current) || current.items[0] & mask == 0 {
                    expected.suit_progress = SuitProgress::AwaitingAward;
                }
                if observed {
                    expected.suit_progress = SuitProgress::AwaitingAnimation;
                }
                if expected.suit_progress == SuitProgress::AwaitingAnimation
                    && before.suit_substate == 3
                    && current.suit_substate == 4
                    && current.player_handlers == [0xe713, 0xe8cd, 0xec1d]
                {
                    expected.suit_progress = SuitProgress::AwaitingControl;
                }
                if expected.suit_progress == SuitProgress::AwaitingControl
                    && current.suit_substate == 0
                    && current.player_handlers == [0xe695, 0xe725, 0xeb52]
                {
                    expected.first_frame = Some(frame);
                    expected.suit_progress = SuitProgress::AwaitingAward;
                }
            } else if observed {
                expected.first_frame = Some(frame);
            }
        }
        if !dying {
            self.death_in_progress = false;
        }
        self.previous = current;
    }

    fn all_expected_frame(&self) -> Option<usize> {
        if self
            .expected
            .iter()
            .any(|expected| expected.first_frame.is_none())
        {
            None
        } else {
            self.expected
                .iter()
                .filter_map(|expected| expected.first_frame)
                .max()
        }
    }

    fn report_expectations(&self) -> Result<(), String> {
        let mut missing = Vec::new();
        for expected in &self.expected {
            match expected.first_frame {
                Some(frame) => eprintln!(
                    "Expectation {}: observed at input frame {frame}",
                    expected.name
                ),
                None => {
                    eprintln!("Expectation {}: not observed", expected.name);
                    missing.push(expected.name.as_str());
                }
            }
        }
        if missing.is_empty() {
            Ok(())
        } else {
            Err(format!(
                "segment expectations not observed: {}",
                missing.join(", ")
            ))
        }
    }

    fn word(game: &Game, address: usize) -> u16 {
        u16::from_le_bytes([
            game.ram().read_byte(address),
            game.ram().read_byte(address + 1),
        ])
    }

    fn observe(&mut self, game: &Game, frame: usize, trace: bool) {
        // Addresses and slot stride match ram_map::RAM_VARS / EnemyData.
        let current = CoverageState::read(game);
        let state = current.state;
        let room = current.room;
        self.observe_expectations(current, frame);
        *self.states.entry(state).or_default() += 1;
        let state_room = (state, room);
        if trace
            && (self.last_state_room != Some(state_room)
                || (u32::from(state)
                    == sm::game::constants::game_states::K_GAME_STATE_8_MAIN_GAMEPLAY
                    && frame % 100 == 0))
        {
            eprintln!(
                "Replay trace: frame {frame}, state {state}, room ${room:04X}, samus_x_pos {}, samus_y_pos {}, samus_pose ${:04X}, timer_status {}, equipped_items ${:04X}, collected_items ${:04X}, missiles {}/{}",
                Self::word(game, 0xaf6),
                Self::word(game, 0xafa),
                Self::word(game, 0xa1c),
                Self::word(game, 0x943),
                Self::word(game, 0x9a2),
                Self::word(game, 0x9a4),
                Self::word(game, 0x9c6),
                Self::word(game, 0x9c8)
            );
        }
        if self.last_state_room == Some(state_room) {
            self.unchanged_frames += 1;
        } else {
            self.last_state_room = Some(state_room);
            self.unchanged_frames = 1;
        }
        self.longest_unchanged = self.longest_unchanged.max(self.unchanged_frames);
        if u32::from(state) == sm::game::constants::game_states::K_GAME_STATE_8_MAIN_GAMEPLAY {
            self.gameplay_frames += 1;
            self.rooms.insert(room);
            self.areas.insert(current.area);
            for slot in (0xf78..0x1778).step_by(core::mem::size_of::<sm::types::EnemyData>()) {
                let definition = Self::word(game, slot);
                if definition != 0 {
                    self.enemies.insert(definition);
                }
            }
        }
    }

    fn report(&self) {
        let (state, room) = self.last_state_room.unwrap();
        println!(
            "Replay coverage: {} gameplay frames, {} rooms, {} enemy definitions; final state {state}, room ${room:04X}; longest unchanged state/room {} frames",
            self.gameplay_frames,
            self.rooms.len(),
            self.enemies.len(),
            self.longest_unchanged
        );
        println!("Observed game states (state:frames): {:?}", self.states);
        println!("Ending/credits state frames: {}", self.credits_frames());
        println!("Gameplay areas: {:?}", self.areas);
        println!("Gameplay rooms: {:04X?}", self.rooms);
        println!("Gameplay enemy definitions: {:04X?}", self.enemies);
    }

    fn credits_frames(&self) -> usize {
        self.states
            .get(&(sm::game::constants::game_states::K_GAME_STATE_39_ENDING_AND_CREDITS as u16))
            .copied()
            .unwrap_or(0)
    }

    fn report_progress(&self) {
        let (state, room) = self.last_state_room.unwrap();
        eprintln!(
            "Coverage so far: {} gameplay frames, {} rooms, {} enemy definitions; state {state}, room ${room:04X}",
            self.gameplay_frames,
            self.rooms.len(),
            self.enemies.len()
        );
    }
}

pub fn run_args(args: &[String], game: &mut Game) -> i32 {
    match run_checked(args, game) {
        Ok(failures) => i32::from(failures != 0),
        Err(error) => {
            eprintln!("verify: {error}");
            2
        }
    }
}

pub(super) unsafe fn initialize_checkpoint_machine(
    game: &mut Game,
    filename: &CString,
) -> Result<*mut sm::types::Snes, String> {
    unsafe {
        game.VERIFY_ACTIVE = true;
        game.VERIFY_FAILURES = 0;
        game.g_runmode = if game.supports_legacy_persistence() {
            sm::cpu_infra::RM_BOTH
        } else {
            sm::cpu_infra::RM_MINE
        } as u8;
        if std::env::var_os("SM_VERIFY_PHYSICAL_AUDIO").is_some() {
            game.g_use_my_apu_code = false;
        }
        let snes = sm::cpu_infra::snes_init(filename.as_ptr(), game);
        if snes.is_null() {
            return Err("unable to load ROM".into());
        }
        Ok(snes)
    }
}

pub(super) unsafe fn begin_verifier_drawing(game: &mut Game) {
    unsafe {
        let snes = game.g_snes;
        sm::snes::ppu::ppu_begin_drawing(
            (*snes).snes_ppu.cast(),
            (&raw mut super::PIXELS).cast(),
            1024,
            0,
        );
        sm::snes::ppu::ppu_begin_drawing(
            (*snes).my_ppu.cast(),
            (&raw mut super::MY_PIXELS).cast(),
            1024,
            0,
        );
    }
}

// Movies cannot select changed immutable catalogs or test rejection before adoption.
#[cfg(sm_verify_dll)]
unsafe fn prepared_checkpoint(
    path: &std::path::Path,
    reject: bool,
    game: &mut Game,
    origin: u64,
    inputs: &[u16],
    completed: usize,
) -> Result<(), String> {
    unsafe {
        let before = checkpoint::capture(game)?;
        if reject {
            let error = checkpoint::load(path, game, origin, inputs)
                .err()
                .ok_or("changed room pack accepted the original checkpoint")?;
            if error != "actor checkpoint world revision differs" {
                return Err(format!(
                    "changed room pack failed the wrong preflight: {error}"
                ));
            }
            if checkpoint::capture(game)? != before {
                return Err("changed room pack rejection changed the complete capture".into());
            }
            eprintln!("Prepared checkpoint changed-world rejection: complete capture unchanged");
            return Ok(());
        }
        let pad = *inputs
            .get(completed)
            .ok_or("prepared checkpoint check requires a continuation input frame")?;
        checkpoint::save(path, game, origin, inputs, completed)?;
        let mut expected_audio = [[0; 2]; 534];
        core::ptr::write_bytes(&raw mut super::PIXELS, 0, 1);
        core::ptr::write_bytes(&raw mut super::MY_PIXELS, 0, 1);
        sm::rtl::rtl_run_frame(i32::from(pad), game);
        sm::rtl::rtl_render_audio(&mut expected_audio, game);
        let continuation = checkpoint::capture(game)?;
        checkpoint::reset_retaining_prepared_world(game)?;
        if checkpoint::load(path, game, origin, inputs)? != completed
            || checkpoint::capture(game)? != before
        {
            return Err("prepared checkpoint did not round-trip the complete capture".into());
        }
        let mut restored_audio = [[0; 2]; 534];
        core::ptr::write_bytes(&raw mut super::PIXELS, 0, 1);
        core::ptr::write_bytes(&raw mut super::MY_PIXELS, 0, 1);
        sm::rtl::rtl_run_frame(i32::from(pad), game);
        sm::rtl::rtl_render_audio(&mut restored_audio, game);
        if restored_audio != expected_audio || checkpoint::capture(game)? != continuation {
            return Err("prepared checkpoint continuation differs".into());
        }
        eprintln!(
            "Prepared checkpoint saved at input frame {completed}: reset binding, complete roundtrip and one-frame audio/state continuation pass; {}",
            path.display()
        );
        Ok(())
    }
}

fn run_checked(args: &[String], game: &mut Game) -> Result<u64, String> {
    if args.len() < 2 {
        return Err("usage: sm verify <rom> <inputs> [--frames N] [--load-sram] [--seed PATH] [--checkpoint-in PATH] [--checkpoint-out PATH] [--require-hit NAME]... [--after-hits N] [--expect TYPE:VALUE|death|credits|credits-complete|save-death-continue]... [--after-expectations N]".into());
    }
    let mut frame_limit = None;
    let mut after_hits = None;
    let mut after_expectations = None;
    let mut expected: Vec<RequiredExpectation> = Vec::new();
    let mut load_sram = false;
    let mut checkpoint_in = None;
    let mut seed_in = None;
    let mut checkpoint_out = None;
    let mut audio_trace_path = None;
    #[cfg(sm_verify_dll)]
    let mut prepared_checkpoint_check = None;
    #[cfg(sm_verify_dll)]
    let mut prepared_checkpoint_reject = None;
    let mut required_probes = Vec::new();
    let mut options = args[2..].iter();
    while let Some(option) = options.next() {
        match option.as_str() {
            #[cfg(sm_verify_dll)]
            "--prepared-checkpoint-check" if prepared_checkpoint_check.is_none() => {
                prepared_checkpoint_check = Some(
                    options
                        .next()
                        .ok_or("missing prepared checkpoint check path")?,
                );
            }
            #[cfg(sm_verify_dll)]
            "--prepared-checkpoint-reject" if prepared_checkpoint_reject.is_none() => {
                prepared_checkpoint_reject = Some(
                    options
                        .next()
                        .ok_or("missing prepared checkpoint rejection path")?,
                );
            }
            "--audio-trace" if audio_trace_path.is_none() => {
                audio_trace_path = Some(options.next().ok_or("missing audio trace path")?);
            }
            "--frames" if frame_limit.is_none() => {
                frame_limit = Some(
                    options
                        .next()
                        .ok_or("missing frame limit")?
                        .parse::<usize>()
                        .map_err(|_| "invalid frame limit")?,
                );
            }
            "--load-sram" if !load_sram => load_sram = true,
            "--after-hits" if after_hits.is_none() => {
                after_hits = Some(
                    options
                        .next()
                        .ok_or("missing follow-up frame count")?
                        .parse::<usize>()
                        .map_err(|_| "invalid follow-up frame count")?,
                );
            }
            "--expect" => {
                let name = options.next().ok_or("missing expectation")?;
                let kind = Expectation::parse(name)?;
                if expected.iter().any(|previous| previous.kind == kind) {
                    return Err(format!("repeated expectation: {name}"));
                }
                expected.push(RequiredExpectation {
                    name: name.clone(),
                    kind,
                    first_frame: None,
                    suit_progress: SuitProgress::AwaitingAward,
                });
            }
            "--after-expectations" if after_expectations.is_none() => {
                after_expectations = Some(
                    options
                        .next()
                        .ok_or("missing expectation follow-up count")?
                        .parse::<usize>()
                        .map_err(|_| "invalid expectation follow-up count")?,
                );
            }
            "--checkpoint-in" if checkpoint_in.is_none() => {
                checkpoint_in = Some(options.next().ok_or("missing checkpoint input path")?);
            }
            "--seed" if seed_in.is_none() => {
                seed_in = Some(options.next().ok_or("missing seed input path")?);
            }
            "--checkpoint-out" if checkpoint_out.is_none() => {
                checkpoint_out = Some(options.next().ok_or("missing checkpoint output path")?);
            }
            "--require-hit" => {
                required_probes.push(options.next().ok_or("missing probe name")?.clone());
            }
            _ => return Err(format!("unknown or repeated option: {option}")),
        }
    }
    #[cfg(sm_verify_dll)]
    let prepared_checkpoint_request = {
        if prepared_checkpoint_check.is_some() && prepared_checkpoint_reject.is_some() {
            return Err(
                "prepared checkpoint check and rejection are separate process openings".into(),
            );
        }
        let request = prepared_checkpoint_check
            .map(|path| (path, false))
            .or_else(|| prepared_checkpoint_reject.map(|path| (path, true)));
        if request.is_some() {
            if game.supports_legacy_persistence() {
                return Err("prepared checkpoint check/rejection requires --room-pack".into());
            }
            if load_sram
                || seed_in.is_some()
                || checkpoint_in.is_some()
                || checkpoint_out.is_some()
                || !required_probes.is_empty()
                || !expected.is_empty()
                || after_hits.is_some()
                || after_expectations.is_some()
            {
                return Err(
                    "prepared checkpoint check/rejection only combines with --frames".into(),
                );
            }
            if prepared_checkpoint_check.is_some_and(|path| std::path::Path::new(path).exists()) {
                return Err("prepared checkpoint output already exists; choose a new path".into());
            }
            frame_limit = Some(frame_limit.unwrap_or(1));
        }
        request
    };
    if expected
        .iter()
        .any(|e| e.kind == Expectation::SaveDeathContinue)
        && ["save.sram.committed", "save.continue.restored"]
            .iter()
            .any(|name| {
                !required_probes
                    .iter()
                    .any(|required| required.as_str() == *name)
            })
    {
        return Err("--expect save-death-continue requires --require-hit save.sram.committed and --require-hit save.continue.restored".into());
    }
    sm::verify_probe::configure(&required_probes)?;
    if after_hits.is_some() && required_probes.is_empty() {
        return Err("--after-hits requires at least one --require-hit".into());
    }
    if after_expectations.is_some() && expected.is_empty() {
        return Err("--after-expectations requires at least one --expect".into());
    }
    if load_sram && (checkpoint_in.is_some() || checkpoint_out.is_some()) {
        return Err("checkpoints require the deterministic clean-SRAM boot".into());
    }
    if seed_in.is_some() && load_sram {
        return Err("--seed cannot be combined with SRAM loading".into());
    }
    if checkpoint_out.is_some_and(|path| std::path::Path::new(path).exists()) {
        return Err("checkpoint output already exists; choose a new path".into());
    }
    if audio_trace_path.is_some()
        && (!game.supports_legacy_persistence()
            || std::env::var_os("SM_VERIFY_PHYSICAL_AUDIO").is_none())
    {
        return Err(
            "ordered Cue/original port trace requires the paired physical-audio oracle".into(),
        );
    }
    let saved_sram = if load_sram {
        let sram = std::fs::read("saves/sm.srm").map_err(|e| format!("saves/sm.srm: {e}"))?;
        if sram.len() != 8192 {
            return Err(format!(
                "saves/sm.srm: expected 8192 bytes, got {}",
                sram.len()
            ));
        }
        Some(sram)
    } else {
        None
    };
    let text = std::fs::read_to_string(&args[1]).map_err(|e| format!("{}: {e}", args[1]))?;
    let mut inputs = Vec::new();
    for (line, word) in text.lines().enumerate() {
        let word = word.trim();
        if word.is_empty() || word.starts_with('#') {
            continue;
        }
        let pad =
            u16::from_str_radix(word, 16).map_err(|_| format!("bad input at line {}", line + 1))?;
        // Preserve all 16 observed hardware bits, including SMV reserved bits.
        // Movie files use the SNES hardware layout; RtlRunFrame uses B at
        // bit 0, Y at bit 1, ... R at bit 11 (the input latch reverses it).
        inputs.push(pad.reverse_bits());
    }
    // Only explicit early stops may use a ceiling beyond the available inputs.
    // The existing hit, expectation and tail checks still reject premature EOF.
    let early_stop = after_hits.is_some() || after_expectations.is_some();
    if frame_limit == Some(0) || (!early_stop && frame_limit.is_some_and(|n| n > inputs.len())) {
        return Err(format!(
            "invalid frame limit, inputs contain {}",
            inputs.len()
        ));
    }
    #[cfg(sm_verify_dll)]
    let prepared_checkpoint_rom = prepared_checkpoint_request.is_some();
    #[cfg(not(sm_verify_dll))]
    let prepared_checkpoint_rom = false;
    let rom = if checkpoint_in.is_some()
        || checkpoint_out.is_some()
        || seed_in.is_some()
        || prepared_checkpoint_rom
        || audio_trace_path.is_some()
    {
        std::fs::read(&args[0]).map_err(|e| format!("{}: {e}", args[0]))?
    } else {
        Vec::new()
    };
    let filename = CString::new(args[0].as_bytes()).map_err(|_| "invalid ROM path")?;
    #[cfg(sm_verify_dll)]
    if prepared_checkpoint_check.is_some()
        && frame_limit.is_some_and(|frames| frames >= inputs.len())
    {
        return Err("prepared checkpoint check requires a continuation input frame".into());
    }
    let start = Instant::now();
    unsafe {
        let _snes = initialize_checkpoint_machine(game, &filename)?;
        if let Some(saved_sram) = saved_sram {
            sm::rtl::rtl_read_sram(game);
            if game.g_sram.bytes(0..8192) != saved_sram {
                return Err("loaded SRAM differs from saves/sm.srm".into());
            }
        }
        super::initialize_verify_audio(game)?;
        begin_verifier_drawing(game);
        let origin = checkpoint::origin(&rom, seed_in.map(std::path::Path::new))?;
        let first = match checkpoint_in {
            Some(path) => checkpoint::load(std::path::Path::new(path), game, origin, &inputs)?,
            None => 0,
        };
        if let Some(path) = seed_in.filter(|_| checkpoint_in.is_none()) {
            let first_input = inputs
                .first()
                .copied()
                .ok_or("room seed requires an input frame")?
                .reverse_bits();
            seed::load(std::path::Path::new(path), game, first_input, &rom)?;
        }
        let frames = frame_limit.unwrap_or(inputs.len() - first);
        let end = first
            .checked_add(frames)
            .filter(|&end| early_stop || end <= inputs.len())
            .map(|end| end.min(inputs.len()))
            .ok_or("requested segment extends past the movie")?;
        if frames == 0 {
            return Err("requested segment is empty".into());
        }
        if checkpoint_in.is_some() {
            eprintln!(
                "Checkpoint segment: input frames {first}..{end}; prefix was not replayed by this build"
            );
        }
        // Initialization and checkpoint loading are setup, not segment hits.
        sm::verify_probe::configure(&required_probes)?;
        let audio_trace = if let Some(path) = audio_trace_path {
            use std::io::Write;
            let source =
                option_env!("SM_VERIFY_SOURCE_HASH").ok_or("audio trace source is unbound")?;
            let native = std::env::var("SM_AUDIO_TRACE_NATIVE_SHA256")
                .map_err(|_| "audio trace native is unbound")?;
            unsafe extern "C" {
                fn sm_backend_source_hash() -> *const std::ffi::c_char;
            }
            let actual_native = std::ffi::CStr::from_ptr(sm_backend_source_hash())
                .to_str()
                .map_err(|_| "invalid native audio trace source")?;
            if actual_native != native {
                return Err("audio trace native source differs".into());
            }
            let rom_sha = std::env::var("SM_AUDIO_TRACE_ROM_SHA256")
                .map_err(|_| "audio trace ROM is unbound")?;
            let input_sha = std::env::var("SM_AUDIO_TRACE_INPUT_SHA256")
                .map_err(|_| "audio trace inputs are unbound")?;
            let dll_sha = std::env::var("SM_AUDIO_TRACE_DLL_SHA256")
                .map_err(|_| "audio trace DLL is unbound")?;
            for hash in [
                source,
                native.as_str(),
                rom_sha.as_str(),
                input_sha.as_str(),
                dll_sha.as_str(),
            ] {
                if hash.len() != 64 || !hash.bytes().all(|b| b.is_ascii_hexdigit()) {
                    return Err("audio trace provenance hash is invalid".into());
                }
            }
            let mut writer = std::io::BufWriter::new(
                std::fs::OpenOptions::new()
                    .write(true)
                    .create_new(true)
                    .open(path)
                    .map_err(|e| format!("{path}: {e}"))?,
            );
            writeln!(writer,
                "{{\"audio_trace\":1,\"source\":\"{source}\",\"native\":\"{native}\",\"rom\":\"{rom_sha}\",\"inputs\":\"{input_sha}\",\"dll\":\"{dll_sha}\",\"first\":{first},\"ceiling_end\":{end}}}"
            ).map_err(|e| e.to_string())?;
            let state = std::rc::Rc::new(std::cell::RefCell::new(AudioTrace {
                writer,
                input: None,
                last_input: None,
                ordinal: 0,
                branches: 0,
                branch: 0,
                oracle: Vec::new(),
                compared: 0,
                cue: None,
                frames: 0,
                cues: 0,
                ports: 0,
            }));
            let consume = state.clone();
            game.audio_output = sm::audio_output::AudioOutputSink::new(move |output| {
                consume.borrow_mut().output(output)
            });
            game.audio_output.new_epoch();
            Some(AudioTraceFile(state))
        } else {
            None
        };
        let mut discarded_audio = [[0; 2]; 534];
        let mut coverage = ReplayCoverage::new(game, expected);
        // Optional console diagnostics share the normal CLI and DLL replay path.
        let trace = std::env::var_os("SM_VERIFY_TRACE").is_some();
        let mut completed = 0;
        let mut hit_tail_complete = after_hits.is_none();
        let mut expectation_tail_complete = after_expectations.is_none();
        for (offset, &pad) in inputs[first..end].iter().enumerate() {
            let index = first + offset;
            if !required_probes.is_empty() {
                sm::verify_probe::begin_frame(index);
            }
            // Undrawn rows must not retain a prior mismatch or stale pixels.
            // No frame is running; keep access raw while each PPU retains
            // its pointer to the array. Each write clears one whole array.
            core::ptr::write_bytes(&raw mut super::PIXELS, 0, 1);
            core::ptr::write_bytes(&raw mut super::MY_PIXELS, 0, 1);
            sm::oracle_mode::set_next_frame_input(inputs.get(index + 1).copied());
            sm::rtl::rtl_run_frame(i32::from(pad), game);
            sm::oracle_mode::set_next_frame_input(None);
            // Advance the music driver once per frame without playing audio.
            sm::rtl::rtl_render_audio(&mut discarded_audio, game);
            if audio_trace.is_some() {
                if let Some(error) = game.audio_output.failure() {
                    return Err(format!("input frame {index}: {error}"));
                }
            }
            coverage.observe(game, index, trace);
            completed = offset + 1;
            if completed % 10_000 == 0 {
                let failures = game.VERIFY_FAILURES;
                eprintln!(
                    "Progress {completed}/{frames} frames: {failures} failing frames ({:.2}s)",
                    start.elapsed().as_secs_f64()
                );
                coverage.report_progress();
            }
            if after_hits.is_some() || after_expectations.is_some() {
                let probe_frame = sm::verify_probe::all_hit_frame();
                let expected_frame = coverage.all_expected_frame();
                hit_tail_complete = after_hits
                    .is_none_or(|tail| probe_frame.is_some_and(|frame| index - frame >= tail));
                expectation_tail_complete = after_expectations
                    .is_none_or(|tail| expected_frame.is_some_and(|frame| index - frame >= tail));
                // A short probe tail must not stop before a later expectation,
                // or vice versa, even when the other requirement has no tail.
                if (required_probes.is_empty() || probe_frame.is_some())
                    && (coverage.expected.is_empty() || expected_frame.is_some())
                    && hit_tail_complete
                    && expectation_tail_complete
                {
                    if coverage.expected.is_empty() {
                        let tail = after_hits.expect("probe-only stop requires its tail");
                        eprintln!(
                            "Required hits and {tail} follow-up frames complete: input frames {first}..{}",
                            index + 1
                        );
                    } else {
                        eprintln!(
                            "Required observations and follow-up frames complete: input frames {first}..{}",
                            index + 1
                        );
                    }
                    break;
                }
            }
        }
        super::finish_verify_audio();
        let failures = game.VERIFY_FAILURES;
        coverage.report();
        println!(
            "Verified {completed} frames: {failures} failing frames ({:.2}s)",
            start.elapsed().as_secs_f64()
        );
        let probes = sm::verify_probe::report();
        let expectations = coverage.report_expectations();
        if failures != 0 {
            if checkpoint_out.is_some() {
                eprintln!("Checkpoint not saved: replay failed");
            }
            return Ok(failures);
        }
        probes?;
        expectations?;
        if !hit_tail_complete {
            return Err(
                "required probe follow-up window did not complete within the replay frame limit"
                    .into(),
            );
        }
        if !expectation_tail_complete {
            return Err(
                "expectation follow-up window did not complete within the replay frame limit"
                    .into(),
            );
        }
        if std::env::var_os("SM_VERIFY_REQUIRE_CREDITS").is_some() && coverage.credits_frames() == 0
        {
            return Err("replay never entered the ending/credits state (39)".into());
        }
        if let Some(trace) = audio_trace {
            use std::io::Write;
            let mut trace = trace.0.borrow_mut();
            if trace.frames != completed || trace.input.is_some() || trace.cue.is_some() {
                return Err("audio trace has missing/unclosed input outputs".into());
            }
            let frames = trace.frames;
            let cues = trace.cues;
            let ports = trace.ports;
            writeln!(trace.writer,
                "{{\"audio_trace_complete\":true,\"first\":{first},\"end\":{},\"frames\":{frames},\"cues\":{cues},\"ports\":{ports}}}", first + completed
            ).map_err(|e| e.to_string())?;
            trace.writer.flush().map_err(|e| e.to_string())?;
            game.audio_output = Default::default();
        }
        if let Some(path) = checkpoint_out {
            let end = first + completed;
            if checkpoint::origin(&rom, seed_in.map(std::path::Path::new))? != origin {
                return Err("seed changed during replay".into());
            }
            checkpoint::save(std::path::Path::new(path), game, origin, &inputs, end)?;
            eprintln!("Saved verifier checkpoint before input frame {end}: {path}");
        }
        #[cfg(sm_verify_dll)]
        if let Some((path, reject)) = prepared_checkpoint_request {
            prepared_checkpoint(
                std::path::Path::new(path),
                reject,
                game,
                origin,
                &inputs,
                first + completed,
            )?;
        }
        Ok(failures)
    }
}

#[cfg(sm_verify_dll)]
pub(super) fn run_unit_tests() -> i32 {
    coverage_checks::run()
}

// Fixtures remain in the DLL source guard; test edits do not change state compatibility.
#[cfg(any(test, sm_verify_dll))]
#[path = "verification/coverage_checks.rs"]
mod coverage_checks;

// Movies cannot independently separate the captured native SFX slot from its DSP selector and instrument ID.
#[cfg(any(test, sm_verify_dll))]
#[cfg_attr(test, test)]
fn sfx_native_note_keeps_captured_slot_and_literal_programming() {
    use sm::{spc_player::sfx1_process, types::DspRegWriteHistory};
    let mut game = Game::new();
    let mut history = Box::new(DspRegWriteHistory {
        count: 0,
        addr: [0; 256],
        val: [0; 256],
    });
    // The existing owner keeps player, DSP and RAM live and disjoint; no device or callback reentry.
    unsafe {
        game.create_native_player();
        let p = game.g_spc_player;
        let ram = (*p).ram.clone();
        (*p).state.reg_write_history = &raw mut *history;
        let state = &mut (*p).state;
        state.sfx1.init_flag = 255;
        state.is_chan_on = 4;
        state.cur_chan_bit = 255;
        state.reg_NON = 255;
        state.key_ON = 0x80;
        state.key_OFF = 0x20;
        for s in &mut state.sfx_chans_1 {
            s.disable = 255;
        }
        let s = &mut state.sfx_chans_1[0];
        s.disable = 0;
        s.ptr = 0x0200;
        s.instr_timer = 1;
        s.release_flag = 255;
        s.release_timer = 1;
        s.voice_index = 4;
        s.voice_bitset = 4;
        s.channel_mask = 0xfb;
        state.channel[2].index = 1;
        state.channel[2].instrument_id = 9;
        state.channel[2].instrument_pitch_base = 0x2222;
        state.channel[1].instrument_pitch_base = 0x3333;
        state.channel[1].final_volume = 0x44;
        for (i, v) in [3, 0x81, 0x23, 0x45, 1, 0].into_iter().enumerate() {
            ram.write(0x6c0c + i as u16, v);
        }
        for (i, v) in [2, 128, 10, 24, 7].into_iter().enumerate() {
            ram.write(0x0200 + i as u16, v);
        }
        sfx1_process(p);
        assert_eq!(history.count, 8);
        assert_eq!(
            &history.addr[..8],
            &[0x14, 0x15, 0x16, 0x17, 0x10, 0x11, 0x12, 0x13]
        );
        assert_eq!(&history.val[..8], &[3, 0x81, 0x23, 0x45, 40, 40, 11, 1]);
        let state = &(*p).state;
        let c = &state.channel[2];
        assert_eq!(
            (c.index, c.instrument_id, c.instrument_pitch_base),
            (1, 9, 256)
        );
        assert_eq!((c.final_volume, c.pan_flag_with_phase_invert), (128, 0));
        assert_eq!(
            (
                state.channel[1].instrument_pitch_base,
                state.channel[1].final_volume
            ),
            (0x3333, 0x44)
        );
        let s = &state.sfx_chans_1[0];
        assert_eq!(
            (s.ptr_idx, s.note, s.subnote, s.instr_timer, s.release_flag),
            (5, 24, 0, 7, 0)
        );
        assert_eq!(
            (
                state.key_ON,
                state.key_OFF,
                state.cur_chan_bit,
                state.reg_NON,
                state.is_chan_on
            ),
            (0x84, 0x20, 0xfb, 0xfb, 4)
        );
        let dsp = &(*state.dsp).state;
        let voice = &dsp.channel[1];
        assert_eq!(
            (voice.srcn, voice.volumeL, voice.volumeR, voice.pitch),
            (3, 40, 40, 267)
        );
        assert_eq!(&dsp.ram[0x14..0x18], &[3, 0x81, 0x23, 0x45]);
    }
}

// Movies cannot restore DSP selector255 and inspect the history-first checked reset failure before restoration finishes.
#[cfg(any(test, sm_verify_dll))]
#[cfg_attr(test, test)]
fn sfx_native_reset_rejection_keeps_history_and_reset_prefix() {
    use sm::{spc_player::sfx1_process, types::DspRegWriteHistory};
    use std::panic::{AssertUnwindSafe, catch_unwind};
    let mut game = Game::new();
    let mut history = Box::new(DspRegWriteHistory {
        count: 0,
        addr: [0; 256],
        val: [0; 256],
    });
    unsafe {
        game.create_native_player();
        let p = game.g_spc_player;
        let ram = (*p).ram.clone();
        (*p).state.reg_write_history = &raw mut *history;
        let state = &mut (*p).state;
        state.sfx1.init_flag = 255;
        state.sfx1.cur_sound = 9;
        state.sfx1.priority = 7;
        state.sfx1.enabled_voices = 4;
        state.is_chan_on = 4;
        state.cur_chan_bit = 0;
        state.key_OFF = 0x20;
        state.key_ON = 0x80;
        for s in &mut state.sfx_chans_1 {
            s.disable = 255;
        }
        let s = &mut state.sfx_chans_1[0];
        s.disable = 0;
        s.ptr = 0x0200;
        s.instr_timer = 1;
        s.legato = 1;
        s.update_adsr = 255;
        s.voice_index = 4;
        s.voice_bitset = 4;
        s.channel_mask = 0xfb;
        s.volume = 0x66;
        s.phase_invert = 0x80;
        let c = &mut state.channel[2];
        c.index = 255;
        c.instrument_id = 2;
        c.instrument_pitch_base = 0x2222;
        c.final_volume = 0x33;
        c.pan_flag_with_phase_invert = 0x40;
        ram.write(0x0200, 255);
        ram.write(0x6c0c, 3);
        let dsp_before = (*state.dsp).state;
        let fault = catch_unwind(AssertUnwindSafe(|| sfx1_process(p)))
            .expect_err("native reset rejects DSP selector255");
        let text = fault.downcast_ref::<String>().expect("bounds failure text");
        // LEGACY(message-text): asserts on panic or Die text; should assert the failure kind instead.
        assert!(text.contains("len is 128") && text.contains("index is 244"));
        assert_eq!(
            (history.count, history.addr[0], history.val[0]),
            (1, 0xf4, 3)
        );
        let state = &(*p).state;
        let s = &state.sfx_chans_1[0];
        assert_eq!(
            (s.disable, s.update_adsr, s.ptr_idx, s.instr_timer),
            (255, 0, 1, 0)
        );
        assert_eq!(
            (
                state.is_chan_on,
                state.cur_chan_bit,
                state.key_OFF,
                state.key_ON
            ),
            (0, 4, 0x24, 0x80)
        );
        assert_eq!(
            (
                state.sfx1.enabled_voices,
                state.sfx1.cur_sound,
                state.sfx1.priority,
                state.sfx1.init_flag
            ),
            (0, 9, 7, 255)
        );
        let c = &state.channel[2];
        assert_eq!(
            (
                c.instrument_id,
                c.instrument_pitch_base,
                c.final_volume,
                c.pan_flag_with_phase_invert
            ),
            (2, 0x2222, 0x33, 0x40)
        );
        let dsp = &(*state.dsp).state;
        assert_eq!(dsp.ram, dsp_before.ram);
        assert_eq!(
            (dsp.channel[7].srcn, dsp.channel[7].pitch),
            (dsp_before.channel[7].srcn, dsp_before.channel[7].pitch)
        );
    }
}

// Movies cannot choose a live tail-byte RAM origin and contrast lazy pan bounds failure with the table-only native path.
#[cfg(any(test, sm_verify_dll))]
#[cfg_attr(test, test)]
fn sfx_native_pan_origin_failure_keeps_programmed_instrument_prefix() {
    use sm::{spc_player::sfx1_process, types::DspRegWriteHistory};
    use std::panic::{AssertUnwindSafe, catch_unwind};
    for pan in [21, 10] {
        let mut game = Game::new();
        let mut history = Box::new(DspRegWriteHistory {
            count: 0,
            addr: [0; 256],
            val: [0; 256],
        });
        unsafe {
            game.create_native_player();
            let p = game.g_spc_player;
            let ram = (*p).ram.clone();
            (*p).state.reg_write_history = &raw mut *history;
            let state = &mut (*p).state;
            // This is a live in-allocation pointer used only as address metadata, never dereferenced here.
            state.ram = ram.as_mut_ptr().wrapping_add(0xffff);
            state.sfx1.init_flag = 255;
            state.key_ON = 0x80;
            for s in &mut state.sfx_chans_1 {
                s.disable = 255;
            }
            let s = &mut state.sfx_chans_1[0];
            s.disable = 0;
            s.ptr = 0x0200;
            s.instr_timer = 1;
            s.legato = 1;
            s.voice_index = 4;
            s.voice_bitset = 4;
            s.note = 0x31;
            s.subnote = 0x22;
            let c = &mut state.channel[2];
            c.index = 1;
            c.instrument_id = 9;
            c.instrument_pitch_base = 0x2222;
            c.pan_flag_with_phase_invert = 0x80;
            for (i, v) in [3, 0x81, 0x23, 0x45, 1, 0].into_iter().enumerate() {
                ram.write(0x6c0c + i as u16, v);
            }
            for (i, v) in [2, 128, pan, 24, 7].into_iter().enumerate() {
                ram.write(0x0200 + i as u16, v);
            }
            let result = catch_unwind(AssertUnwindSafe(|| sfx1_process(p)));
            assert_eq!(result.is_err(), pan == 21);
            if pan == 21 {
                let fault = result.expect_err("native pan reaches the checked tail-origin read");
                let text = fault.downcast_ref::<String>().expect("bounds failure text");
                // LEGACY(message-text): asserts on panic or Die text; should assert the failure kind instead.
                assert!(text.contains("len is 65536") && text.contains("index is 73265"));
            }
            let count = history.count as usize;
            let writes: Vec<_> = history.addr[..count]
                .iter()
                .copied()
                .zip(history.val.iter().copied())
                .collect();
            let expected = [
                (0x14, 3),
                (0x15, 0x81),
                (0x16, 0x23),
                (0x17, 0x45),
                (0x10, 40),
                (0x11, 40),
                (0x12, 11),
                (0x13, 1),
            ];
            assert_eq!(writes, expected[..if pan == 21 { 4 } else { 8 }]);
            let state = &(*p).state;
            let c = &state.channel[2];
            assert_eq!(
                (
                    c.instrument_id,
                    c.instrument_pitch_base,
                    c.final_volume,
                    c.pan_flag_with_phase_invert
                ),
                (9, 256, 128, 0)
            );
            let s = &state.sfx_chans_1[0];
            assert_eq!(
                (s.ptr_idx, s.note, s.subnote, s.instr_timer, state.key_ON),
                if pan == 21 {
                    (3, 0x31, 0x22, 0, 0x80)
                } else {
                    (5, 24, 0, 7, 0x80)
                }
            );
            assert_eq!(
                &(&(*state.dsp).state.ram)[0x14..0x18],
                &[3, 0x81, 0x23, 0x45]
            );
        }
    }
}

// Movies do not call device rendering or inject its full-buffer, history/rebound and cycle-failure entries.
#[cfg(any(test, sm_verify_dll))]
#[cfg_attr(test, test)]
fn owned_player_render_matches_native_schedule_and_failure_prefixes() {
    use sm::spc_player::*;
    use std::panic::{AssertUnwindSafe, catch_unwind};
    for mode in 0..6 {
        let (offset, timer) = match mode {
            0 | 3 => (0, 0),
            1 => (531, 63),
            _ => (534, 64),
        };
        let mut history = [DspRegWriteHistory {
            count: 0,
            addr: [0; 256],
            val: [0; 256],
        }; 2];
        let mut games: [_; 4] = std::array::from_fn(|_| Game::new());
        unsafe {
            for game in &mut games {
                game.create_native_player();
                spc_player_initialize(game.g_spc_player);
            }
            let players = games.each_ref().map(|game| game.g_spc_player);
            let owned_dsp = players.map(|player| (*player).state.dsp);
            for (index, game) in games[..2].iter_mut().enumerate() {
                let s = &mut (*players[index]).state;
                s.timer_cycles = timer;
                s.sfx_timer_accum = 224;
                s.main_tempo_accum = 240;
                (s.sfx1.cur_sound, s.sfx1.voices_to_setup) = (1, 1);
                let dsp = &mut *s.dsp;
                dsp.state.sampleOffset = offset;
                for pair in dsp.state.sampleBuffer.chunks_exact_mut(2) {
                    pair.copy_from_slice(&[17, -23]);
                }
                if mode == 3 {
                    dsp.state.channel[0].adsrState = 255;
                    dsp.state.channel[0].pitch = 1;
                    dsp.state.reset = false;
                }
                if mode == 4 {
                    s.reg_write_history = &raw mut history[index];
                }
                if mode == 5 {
                    let rebound = &mut *owned_dsp[index + 2];
                    let ram = rebound.state.apu_ram;
                    rebound.state = dsp.state;
                    rebound.state.apu_ram = ram;
                    s.dsp = rebound;
                }
                game.apu_queue.pending = [0xf0, 255, 255, 255];
                game.apu_queue.push(false);
            }
            let mut pcm = [[[i16::MIN; 2]; 534]; 2];
            let got = catch_unwind(AssertUnwindSafe(|| {
                sm::rtl::rtl_render_audio(&mut pcm[0], &mut games[0])
            }));
            let writes = games[1].apu_queue.pop().unwrap();
            sm::apu_queue::ApuQueue::apply_ports(writes, &mut (*players[1]).state.input_ports);
            let want = catch_unwind(AssertUnwindSafe(|| {
                spc_player_generate_samples(players[1]);
                (*(*players[1]).state.dsp).state.write_samples(&mut pcm[1]);
            }));
            assert_eq!((got.is_err(), want.is_err()), (mode == 3, mode == 3));
            assert_eq!(pcm[0], pcm[1]);
            assert_eq!(games[0].apu_queue.to_bytes(), games[1].apu_queue.to_bytes());
            let mut ram = [[0u8; 65536]; 2];
            for (index, &p) in players[..2].iter().enumerate() {
                spc_player_copy_variables_to_ram(p);
                (*p).ram.copy_to(&mut ram[index]);
            }
            assert_eq!(ram[0], ram[1]);
            let [a, b] = [0, 1].map(|i| &(*(*players[i]).state.dsp).state);
            assert_eq!(a.ram, b.ram);
            assert_eq!(a.sampleOffset, b.sampleOffset);
            assert_eq!(a.evenCycle, b.evenCycle);
            if mode == 3 {
                assert_eq!((*players[0]).state.timer_cycles, 64);
                assert_eq!((a.sampleOffset, a.channel[0].pitchCounter), (0, 1));
            }
            if offset == 534 {
                assert_eq!(pcm[0], [[17, -23]; 534]);
                assert_eq!((*players[0]).state.timer_cycles, 0);
            }
            if mode == 4 {
                assert!(history[0].count > 0);
                assert_eq!(history[0].count, history[1].count);
                assert_eq!(history[0].addr, history[1].addr);
                assert_eq!(history[0].val, history[1].val);
            }
            for (&p, dsp) in players.iter().zip(owned_dsp) {
                (*p).state.reg_write_history = core::ptr::null_mut();
                (*p).state.dsp = dsp;
            }
        }
    }
}

// Movies cannot inject live history/rebound DSP links and arbitrary RAM during player initialization and queued state transfers.
#[cfg(any(test, sm_verify_dll))]
#[cfg_attr(test, test)]
fn owned_player_initialization_and_transfers_preserve_native_boundaries() {
    use sm::spc_player::*;
    for mode in 0..4 {
        let mut history = [DspRegWriteHistory {
            count: 0,
            addr: [0; 256],
            val: [0; 256],
        }; 2];
        let mut games: [_; 4] = std::array::from_fn(|_| Game::new());
        unsafe {
            for game in &mut games {
                game.create_native_player();
            }
            let players = games.each_ref().map(|game| game.g_spc_player);
            let owned_dsp = players.map(|player| (*player).state.dsp);
            for (i, &p) in players[..2].iter().enumerate() {
                (*p).ram.fill(0x5a);
                if mode == 1 {
                    (*p).state.reg_write_history = &raw mut history[i];
                }
                if mode == 2 {
                    (*p).state.dsp = owned_dsp[i + 2];
                }
                if mode == 3 {
                    (*p).state.ram = (*p).ram.as_mut_ptr().wrapping_add(1);
                }
            }
            games[0].initialize_native_player();
            spc_player_initialize(players[1]);
            let links =
                players.map(|p| ((*p).state.dsp, (*p).state.reg_write_history, (*p).state.ram));
            games[0].save_native_audio_variables(false);
            spc_player_copy_variables_to_ram(players[1]);
            let mut ram = [[0; 65536]; 2];
            for (i, &p) in players[..2].iter().enumerate() {
                (*p).ram.copy_to(&mut ram[i]);
            }
            assert_eq!(ram[0], ram[1]);
            let r = &ram[0];
            assert_eq!((r[0x500], r[0x14ff], r[0x1500]), (0, 0, 0x5a));
            let [a, b] = [0, 1].map(|i| &(*(*players[i]).state.dsp).state);
            assert_eq!(a.ram, b.ram);
            assert_eq!((a.ram[0x0c], a.ram[0x1c], a.ram[0x5d]), (0x60, 0x60, 0x6d));
            let s = &(*players[0]).state;
            assert_eq!((s.tempo, s.timer_cycles), (0x105a, 0));
            for &p in &players[..2] {
                (*p).ram.write(0, 0xa7);
                (*p).ram.write(0x52, 0x34);
                (*p).ram.write(0x53, 0x12);
            }
            games[0].load_native_audio_variables();
            spc_player_copy_variables_from_ram(players[1]);
            for (i, &p) in players[..2].iter().enumerate() {
                let s = &(*p).state;
                assert_eq!((s.dsp, s.reg_write_history, s.ram), links[i]);
                assert_eq!((s.input_ports[0], s.tempo), (0xa7, 0x1234));
                for (j, channel) in s.channel.iter().enumerate() {
                    assert_eq!(channel.index, j as u8);
                }
            }
            for game in &mut games[..2] {
                game.apu_queue.pending = [0xf0, 255, 255, 255];
                game.apu_queue.push(false);
            }
            games[0].save_native_audio_variables(true);
            games[1]
                .apu_queue
                .apply_queued(&mut (*players[1]).state.input_ports);
            spc_player_copy_variables_to_ram(players[1]);
            for (i, &p) in players[..2].iter().enumerate() {
                (*p).ram.copy_to(&mut ram[i]);
            }
            assert_eq!(ram[0], ram[1]);
            assert_eq!(ram[0][0], 0xf0);
            assert_eq!(games[0].apu_queue.to_bytes(), games[1].apu_queue.to_bytes());
            if mode == 1 {
                assert!(history[0].count > 0);
                assert_eq!(history[0].count, history[1].count);
                assert_eq!(history[0].addr, history[1].addr);
                assert_eq!(history[0].val, history[1].val);
            }
            for (&p, dsp) in players.iter().zip(owned_dsp) {
                (*p).state.reg_write_history = core::ptr::null_mut();
                (*p).state.dsp = dsp;
            }
        }
    }
}

// Movies cannot retain raw player selections and borrowed foreign links across owner replacement and destruction.
#[cfg(any(test, sm_verify_dll))]
#[cfg_attr(test, test)]
fn owned_player_creation_replacement_and_drop_keep_borrowed_allocations() {
    use sm::spc_player::DspRegWriteHistory;
    let mut game = Game::new();
    let mut foreign = Game::new();
    assert!(game.g_spc_player.is_null());
    game.create_native_player();
    foreign.create_native_player();
    let mut history = DspRegWriteHistory {
        count: 0,
        addr: [0; 256],
        val: [0; 256],
    };
    unsafe {
        let old = game.g_spc_player;
        let old_dsp = (*old).state.dsp;
        let old_ram = (*old).ram.clone();
        assert_eq!((*old).state.ram, old_ram.as_mut_ptr());
        assert_eq!((*old_dsp).state.apu_ram, old_ram.as_mut_ptr());
        assert!((*old).state.reg_write_history.is_null());
        assert_eq!(((*old).state.tempo, (*old).state.timer_cycles), (0, 0));
        old_ram.write(0x1234, 0x5a);
        game.g_spc_player = foreign.g_spc_player;
        game.create_native_player();
        let player = game.g_spc_player;
        assert_ne!(player, old);
        assert_ne!((*player).state.dsp, old_dsp);
        assert_ne!((*player).state.ram, old_ram.as_mut_ptr());
        assert_eq!(old_ram.read(0x1234), 0x5a);
        assert_eq!((*player).ram.read(0x1234), 0);
        assert_eq!(
            ((*player).state.tempo, (*player).state.timer_cycles),
            (0, 0)
        );
        let retained_ram = (*player).ram.clone();
        retained_ram.write(0x4321, 0xa7);
        let foreign_player = foreign.g_spc_player;
        let foreign_dsp = (*foreign_player).state.dsp;
        let foreign_ram = (*foreign_player).ram.clone();
        foreign_ram.write(0x3456, 0xc3);
        (*foreign_dsp).state.ram[0x0c] = 0x47;
        (*player).state.dsp = foreign_dsp;
        (*player).state.ram = foreign_ram.as_mut_ptr();
        (*player).state.reg_write_history = &raw mut history;
        let mut moved = vec![game];
        let game = moved.pop().unwrap();
        assert_eq!(game.g_spc_player, player);
        assert_eq!((*player).state.ram, foreign_ram.as_mut_ptr());
        drop(game);
        assert_eq!(retained_ram.read(0x4321), 0xa7);
        assert_eq!(foreign_ram.read(0x3456), 0xc3);
        assert_eq!((*foreign_player).state.dsp, foreign_dsp);
        assert_eq!((*foreign_dsp).state.ram[0x0c], 0x47);
        assert_eq!(history.count, 0);
        assert_eq!(history.addr, [0; 256]);
        assert_eq!(history.val, [0; 256]);
        drop(foreign);
        assert_eq!(foreign_ram.read(0x3456), 0xc3);
        assert_eq!(old_ram.read(0x1234), 0x5a);
        assert_eq!(retained_ram.read(0x4321), 0xa7);
    }
}

// Movies cannot attach snapshot history/rebound DSP links or request invalid restores and mismatched history topology.
#[cfg(any(test, sm_verify_dll))]
#[cfg_attr(test, test)]
fn owned_player_snapshots_restore_selected_allocations_and_failure_prefixes() {
    use sm::{game_state::PlayerSnapshotSource as Source, spc_player::*};
    for mode in 0..3 {
        let mut history = [DspRegWriteHistory {
            count: 2,
            addr: [17; 256],
            val: [23; 256],
        }; 2];
        let mut games: [_; 4] = std::array::from_fn(|_| Game::new());
        for game in &mut games {
            game.create_native_player();
        }
        let players = games.each_ref().map(|game| game.g_spc_player);
        let mut snapshots = [games[0].g_snapshot_before.player; 2];
        unsafe {
            let dsps = players.map(|p| (*p).state.dsp);
            games[1].g_spc_player = players[2];
            for (i, p) in [players[0], players[2]].into_iter().enumerate() {
                (*p).ram.fill(0x5a);
                (*p).state.tempo = 0x4567;
                (*p).state.ram = (*p).ram.as_mut_ptr().wrapping_add(1);
                if mode == 1 {
                    (*p).state.reg_write_history = &raw mut history[i];
                }
                if mode == 2 {
                    (*p).state.dsp = [dsps[1], dsps[3]][i];
                }
                (*(*p).state.dsp).state.ram[0x0c] = 0x47;
            }
            games[0].capture_native_player_snapshot(&mut snapshots[0]);
            games[1].capture_native_player_snapshot(&mut snapshots[1]);
            for p in [players[0], players[2]] {
                (*p).ram.fill(0xc3);
                (*p).state.tempo = 0xeeee;
                (*(*p).state.dsp).state.ram[0x0c] = 0x99;
            }
            if mode == 1 {
                (*players[0]).state.reg_write_history = core::ptr::null_mut();
                assert!(
                    std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
                        games[0].restore_native_player_snapshot(Source::Local(&snapshots[0]));
                    }))
                    .is_err()
                );
                assert_eq!(
                    ((*players[0]).state.tempo, (*players[0]).ram.read(0)),
                    (0xeeee, 0xc3)
                );
                assert_eq!((*dsps[0]).state.ram[0x0c], 0x99);
                (*players[0]).state.reg_write_history = &raw mut history[0];
                for h in &mut history {
                    h.count = 9;
                    h.val.fill(0xa7);
                }
            }
            games[0].restore_native_player_snapshot(Source::Local(&snapshots[0]));
            games[1].restore_native_player_snapshot(Source::Local(&snapshots[1]));
            for (i, p) in [players[0], players[2]].into_iter().enumerate() {
                assert_eq!(((*p).state.tempo, (*p).ram.read(0)), (0x4567, 0x5a));
                assert_eq!((*p).state.ram, (*p).ram.as_mut_ptr());
                assert_eq!(
                    (*p).state.dsp,
                    if mode == 2 {
                        [dsps[1], dsps[3]][i]
                    } else {
                        [dsps[0], dsps[2]][i]
                    }
                );
                assert_eq!((*(*p).state.dsp).state.ram[0x0c], 0x47);
                if mode == 1 {
                    assert_eq!(history[i].count, 2);
                    assert_eq!(history[i].val, [23; 256]);
                }
            }
            games[0].g_spc_player = core::ptr::null_mut();
            games[0].capture_native_player_snapshot(&mut snapshots[0]);
            games[0].g_spc_player = core::ptr::NonNull::<SpcPlayer>::dangling().as_ptr();
            games[0].restore_native_player_snapshot(Source::Local(&snapshots[0]));
            games[0].g_spc_player = players[0];
            assert_eq!((*players[0]).state.tempo, 0x4567);
            for (p, dsp) in players.into_iter().zip(dsps) {
                (*p).state.reg_write_history = core::ptr::null_mut();
                (*p).state.dsp = dsp;
            }
        }
    }
}

// Consume actual owned sink outputs. Only one input's oracle operations are
// retained: mine operations compare and retire them as the immediate adapter runs.
struct AudioTrace {
    writer: std::io::BufWriter<std::fs::File>,
    input: Option<(u64, u64)>,
    last_input: Option<(u64, u64)>,
    ordinal: u64,
    branches: u8,
    branch: u8,
    oracle: Vec<(
        sm::audio_output::PortTarget,
        u8,
        u8,
        sm::audio_output::DeliveryId,
        sm::audio_output::ExecutionPosition,
    )>,
    compared: usize,
    cue: Option<(
        sm::audio_output::DeliveryId,
        sm::audio_output::EmittedCue,
        usize,
    )>,
    frames: usize,
    cues: usize,
    ports: usize,
}

impl AudioTrace {
    fn output(&mut self, output: sm::audio_output::AudioOutput) -> Result<(), String> {
        use sm::audio_output::AudioOutput;
        use std::io::Write;
        // Record the real first mismatch/incomplete prefix as well as successes.
        writeln!(self.writer, "AudioOutput {output:?}").map_err(|e| e.to_string())?;
        match output {
            AudioOutput::InputBegin { epoch, tick } => {
                if self
                    .last_input
                    .is_some_and(|(previous_epoch, previous_tick)| {
                        epoch < previous_epoch || (epoch == previous_epoch && tick <= previous_tick)
                    })
                {
                    return Err("audio trace duplicated/regressed input identity".into());
                }
                if self.input.replace((epoch, tick)).is_some() || self.cue.is_some() {
                    return Err("audio trace duplicate/incomplete input begin".into());
                }
                self.ordinal = 0;
                self.branches = 0;
                self.branch = 0;
                self.oracle.clear();
                self.compared = 0;
            }
            AudioOutput::ExecutionBegin { position } => {
                let expected = if self.branches == 0 { 1 } else { 2 };
                if self.input.is_none()
                    || self.cue.is_some()
                    || position.branch != expected
                    || self.branches == 3
                    || position.pc.is_none()
                {
                    return Err(format!(
                        "audio trace missing/duplicate execution: {position:?}"
                    ));
                }
                self.branch = position.branch;
                self.branches |= position.branch;
            }
            AudioOutput::CueBegin { id, emission } => {
                self.identity(id)?;
                if self.branch != 2 || self.cue.is_some() {
                    return Err(format!("audio trace duplicate/unpaired Cue {id:?}"));
                }
                let emission = emission.ok_or("audio trace untyped compatibility Cue")?;
                use sm::audio_output::{CueAction, CuePhase};
                let phase_matches = matches!(
                    (&emission.cue.action, emission.phase),
                    (CueAction::InitializeAudio, CuePhase::Initialization)
                        | (CueAction::MusicBank(_), CuePhase::MusicBank)
                        | (CueAction::Music(_), CuePhase::MusicTrack)
                        | (
                            CueAction::Sound { .. },
                            CuePhase::GameplaySound | CuePhase::DialogueSound
                        )
                );
                if !phase_matches {
                    return Err(format!("audio trace Cue phase differs: {emission:?}"));
                }
                self.cue = Some((id, emission, 0));
                self.cues += 1;
            }
            AudioOutput::Port {
                id,
                cue,
                position,
                target,
                port,
                value,
            } => {
                self.identity(id)?;
                if position.branch != self.branch || position.pc.is_none() || port > 3 {
                    return Err(format!("audio trace unbound port provenance {position:?}"));
                }
                self.ports += 1;
                if self.branch == 1 {
                    if cue.is_some() || self.oracle.len() == 262_144 {
                        return Err("audio trace oracle Cue/capacity violation".into());
                    }
                    self.oracle.push((target, port, value, id, position));
                } else if self.branch == 2 {
                    if self
                        .oracle
                        .get(self.compared)
                        .map(|original| (original.0, original.1, original.2))
                        != Some((target, port, value))
                    {
                        return Err(format!(
                            "audio trace first port mismatch at {id:?}, {position:?}, Cue {cue:?}: oracle {:?}, mine {:?}",
                            self.oracle.get(self.compared),
                            (target, port, value)
                        ));
                    }
                    self.compared += 1;
                    match (&mut self.cue, cue) {
                        (Some((active, emission, count)), Some(owner)) if *active == owner => {
                            use sm::audio_output::{CueAction, MusicCue, SoundCue};
                            let expected = match &emission.cue.action {
                                CueAction::Music(MusicCue::Original(selector)) => {
                                    Some((0, selector.0))
                                }
                                CueAction::Sound {
                                    channel,
                                    sound: SoundCue::Original(selector),
                                } => Some((1 + channel.index(), selector.0)),
                                _ => None,
                            };
                            if expected.is_some_and(|expected| expected != (port, value)) {
                                return Err(format!(
                                    "audio trace Cue mapping differs: {emission:?}"
                                ));
                            }
                            *count += 1;
                        }
                        (None, None) => {
                            // Existing compat bootstrap zero writes are real transport
                            // requests outside the five semantic Cue invocation sites.
                        }
                        _ => return Err("audio trace unpaired Cue/port request".into()),
                    }
                } else {
                    return Err("audio trace port outside a visited execution".into());
                }
            }
            AudioOutput::CueComplete { id } => {
                let (owner, emission, count) =
                    self.cue.take().ok_or("audio trace missing Cue begin")?;
                if owner != id || count == 0 {
                    return Err(format!("audio trace incomplete Cue {id:?}: {emission:?}"));
                }
                if matches!(
                    emission.cue.action,
                    sm::audio_output::CueAction::Music(_)
                        | sm::audio_output::CueAction::Sound { .. }
                ) && count != 1
                {
                    return Err(format!(
                        "audio trace Cue write multiplicity differs: {id:?}"
                    ));
                }
            }
            AudioOutput::InputComplete { epoch, tick } => {
                if self.input.take() != Some((epoch, tick))
                    || self.branches != 3
                    || self.cue.is_some()
                    || self.compared != self.oracle.len()
                {
                    return Err(format!(
                        "audio trace missing/incomplete input: {epoch}/{tick}"
                    ));
                }
                self.last_input = Some((epoch, tick));
                self.frames += 1;
                self.oracle.clear();
                self.writer.flush().map_err(|e| e.to_string())?;
            }
        }
        Ok(())
    }

    fn identity(&mut self, id: sm::audio_output::DeliveryId) -> Result<(), String> {
        if self.input != Some((id.epoch, id.input_tick)) || id.ordinal != self.ordinal {
            return Err(format!(
                "audio trace duplicate/gapped delivery identity {id:?}"
            ));
        }
        self.ordinal = self
            .ordinal
            .checked_add(1)
            .ok_or("audio trace ordinal overflow")?;
        Ok(())
    }
}

// Flush the observed failure/unwind prefix even while Game still owns the sink's
// receiver. This is file lifetime management, not a gameplay panic wrapper.
struct AudioTraceFile(std::rc::Rc<std::cell::RefCell<AudioTrace>>);
impl Drop for AudioTraceFile {
    fn drop(&mut self) {
        use std::io::Write;
        if let Ok(mut trace) = self.0.try_borrow_mut() {
            let _ = trace.writer.flush();
        }
    }
}
