//! Single-threaded replay through the same differential oracle as the UI.
use sm::game_state::Game;
use std::{collections::BTreeSet, ffi::CString, time::Instant};

use sm::verification_checkpoint as checkpoint;
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
                phase: word(core::mem::offset_of!(EnemyData, ai_var_E)),
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
            actor_phase: word(0xf78 + core::mem::offset_of!(sm::types::EnemyData, ai_var_A)),
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
                    == sm::game::constants::game_states::kGameState_8_MainGameplay
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
        if u32::from(state) == sm::game::constants::game_states::kGameState_8_MainGameplay {
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
            .get(&(sm::game::constants::game_states::kGameState_39_EndingAndCredits as u16))
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
        let snes = sm::cpu_infra::SnesInit(filename.as_ptr(), game);
        if snes.is_null() {
            return Err("unable to load ROM".into());
        }
        Ok(snes)
    }
}

pub(super) unsafe fn begin_verifier_drawing(game: &mut Game) {
    unsafe {
        let snes = game.g_snes;
        sm::snes::ppu::PpuBeginDrawing(
            (*snes).snes_ppu.cast(),
            (&raw mut super::g_pixels).cast(),
            1024,
            0,
        );
        sm::snes::ppu::PpuBeginDrawing(
            (*snes).my_ppu.cast(),
            (&raw mut super::g_my_pixels).cast(),
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
        core::ptr::write_bytes(&raw mut super::g_pixels, 0, 1);
        core::ptr::write_bytes(&raw mut super::g_my_pixels, 0, 1);
        sm::rtl::RtlRunFrame(i32::from(pad), game);
        sm::rtl::RtlRenderAudio(&mut expected_audio, game);
        let continuation = checkpoint::capture(game)?;
        checkpoint::reset_retaining_prepared_world(game)?;
        if checkpoint::load(path, game, origin, inputs)? != completed
            || checkpoint::capture(game)? != before
        {
            return Err("prepared checkpoint did not round-trip the complete capture".into());
        }
        let mut restored_audio = [[0; 2]; 534];
        core::ptr::write_bytes(&raw mut super::g_pixels, 0, 1);
        core::ptr::write_bytes(&raw mut super::g_my_pixels, 0, 1);
        sm::rtl::RtlRunFrame(i32::from(pad), game);
        sm::rtl::RtlRenderAudio(&mut restored_audio, game);
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
            sm::rtl::RtlReadSram(game);
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
            core::ptr::write_bytes(&raw mut super::g_pixels, 0, 1);
            core::ptr::write_bytes(&raw mut super::g_my_pixels, 0, 1);
            sm::rtl::RtlRunFrame(i32::from(pad), game);
            // Advance the music driver once per frame without playing audio.
            sm::rtl::RtlRenderAudio(&mut discarded_audio, game);
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
