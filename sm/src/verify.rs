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

#[cfg(any(test, sm_verify_dll))]
mod coverage_checks {
    use super::*;
    use sm::game::message_composition::{prepared::*, *};
    use sm::game::scene::component as sd;
    use sm::game::scene::hdma_sources as hs;
    use sm::game::scene::records as sc;
    use sm::game::scene::start as ss;
    use sm::game::scene::state as ds;
    use sm::rendering::{self as r, IndexedSource, prepared::*};
    use std::sync::Arc;

    #[cfg(sm_verify_dll)]
    pub(super) fn run() -> i32 {
        // Keep names and function pointers paired without duplicating assertions.
        macro_rules! cases {
            ($($test:ident),+ $(,)?) => { [$( (stringify!($test), $test as fn()), )+] };
        }
        let tests: [(&str, fn()); 48] = cases![
            prepared_message_pixels_delay_queue_and_checkpoint,
            prepared_message_rejects_changed_content_and_resets,
            independently_authored_raster_pixels_and_prefixes,
            room_pack_uses_native_header_selection_and_final_block_mirroring,
            room_pack_rejects_checksum_valid_structural_errors,
            room_pack_selection_rejects_missing_duplicate_and_option_values,
            expectation_parser_rejects_unknown_or_malformed_values,
            inventory_events_require_a_new_award_inside_the_same_live_room,
            boss_flags_require_a_rising_bit_in_the_corresponding_area,
            credits_require_enabled_script_advancement_not_ending_presence,
            death_must_begin_in_the_segment_and_reach_game_over,
            every_expectation_is_required_and_repeated_hits_keep_the_first_frame,
            suit_award_waits_for_later_hdma_and_control_return,
            door_requires_fresh_admission_and_the_same_completed_destination,
            elevator_landing_requires_the_linked_traversal_and_fresh_unlock,
            ceres_requires_observed_combat_and_the_same_encounter_escape_edge,
            bomb_requires_observed_damage_death_and_later_completion,
            bomb_deletion_only_finishes_an_already_observed_death,
            bomb_resets_on_room_actor_or_gameplay_discontinuity,
            bomb_does_not_reuse_an_episode_after_healing_or_completion,
            fresh_save_death_selected_restore_and_initialized_arrival_complete_once,
            loaded_history_and_unordered_restore_do_not_qualify,
            reset_and_slot_change_discard_save_history,
            state_rollbacks_discard_episode,
            wrong_first_arrival_cannot_be_repaired_by_later_room_entry,
            restoration_and_gameplay_initialization_are_required,
            scene_pixels_preserve_line_incarnations_and_redraw,
            scene_rejects_missing_inconsistent_and_unsupported_history,
            scene_media_rejects_missing_revisions_and_stale_aliases,
            scene_rejects_reference_quantization,
            scene_start_rejects_missing_fields_and_invalid_history_prefix,
            scene_start_rejects_changed_profile_and_audio_prefix,
            scene_component_preserves_complete_open_checkpoint_and_reopen,
            scene_component_reset_preserves_complete_owned_buffers_and_alias_views,
            scene_component_rejects_missing_resident_history_and_media_replacement,
            scene_component_failed_source_replacement_preserves_complete_owner,
            scene_component_rejects_missing_final_resident_assets,
            scene_component_rejects_each_native_resident_latch_alias,
            scene_component_rejects_missing_hdma_catalog,
            scene_component_rejects_changed_hdma_payload_under_same_fingerprint,
            scene_component_rejects_wrong_hdma_kinds_and_required_unbound_sources,
            scene_component_rejects_invalid_hdma_bank_windows,
            scene_component_admits_explicitly_disabled_hdma_sources,
            scene_component_rejects_conflicting_physical_hdma_aliases,
            scene_component_preserves_coherent_physical_hdma_aliases_on_reopen,
            scene_component_reset_preserves_coherent_physical_hdma_aliases,
            scene_component_preserves_schema3_delivery_effective_and_written_selection,
            scene_component_rejects_schema2_and_invalid_effective_color_selectors,
        ];
        let mut failed = 0usize;
        for (name, test) in &tests {
            let passed = std::panic::catch_unwind(*test).is_ok();
            println!(
                "test verify::coverage_checks::{name} ... {}",
                if passed { "ok" } else { "FAILED" }
            );
            failed += usize::from(!passed);
        }
        println!(
            "Verifier unit tests: {} passed; {failed} failed; {} total",
            tests.len() - failed,
            tests.len()
        );
        i32::from(failed != 0)
    }

    // Movies cannot author portable media identities or complete per-line incarnations.
    // This synthetic display fixture reuses the accepted message font, map and colors;
    // it supplies no complete G start, Reference capture or native audio evidence.
    fn scene_fixture() -> (sc::SceneDescription, Arc<sc::SceneMedia>) {
        let (_, prepared) = prepared_message_fixture(0x3801);
        let revision = sc::Revision([1; 32]);
        let reference = |id| sc::AssetRef {
            id: sc::AssetId(id),
            revision,
        };
        let image_ref = |incarnation| sc::MediaRef {
            asset: reference(1),
            incarnation,
        };
        let palette_ref = |incarnation| sc::MediaRef {
            asset: reference(6),
            incarnation,
        };
        let saved = &prepared.saved.media;
        let colors = core::array::from_fn(|index| saved.palette(index as u8).unwrap());
        let restored_colors =
            core::array::from_fn(|index| prepared.restored.media.palette(index as u8).unwrap());
        let mut words: Vec<u16> = (0..0x8000)
            .map(|index| saved.word(index).unwrap())
            .collect();
        // Explicit authored latch words and the required 26 palette aliases.
        words.extend([0; 5]);
        words.extend_from_slice(&colors[..26]);
        let mut later_words = words.clone();
        for word in &mut later_words[0x5880..0x5c00] {
            *word ^= 3;
        }
        later_words[0x8005..0x801f].copy_from_slice(&restored_colors[..26]);
        let cells = |image: &[u16], first: usize| -> Arc<[sc::TileCell]> {
            image[first..first + 1024]
                .iter()
                .map(|word| sc::TileCell {
                    tile: word & 0x3ff,
                    palette: (word >> 10 & 7) as u8,
                    priority: word & 0x2000 != 0,
                    flip_horizontal: word & 0x4000 != 0,
                    flip_vertical: word & 0x8000 != 0,
                })
                .collect::<Vec<_>>()
                .into()
        };
        let blank_cells = cells(&words, 0x5000);
        let hud_cells = cells(&words, 0x5880);
        let later_cells = cells(&later_words, 0x5880);
        let assets = vec![
            sc::SceneAsset::Room(reference(1)),
            sc::SceneAsset::Atlas(sc::TileAtlas {
                reference: reference(2),
                encoding: sc::TileEncoding::Planar4,
                tile_count: 1,
                data: vec![0; 16].into(),
            }),
            sc::SceneAsset::Atlas(sc::TileAtlas {
                reference: reference(3),
                encoding: sc::TileEncoding::Planar2,
                tile_count: 256,
                data: words[0x4000..0x4800].to_vec().into(),
            }),
            sc::SceneAsset::Map(sc::Tilemap {
                reference: reference(4),
                atlas: reference(2),
                palette: reference(6),
                width: 32,
                height: 32,
                cells: blank_cells.clone(),
            }),
            sc::SceneAsset::Map(sc::Tilemap {
                reference: reference(5),
                atlas: reference(3),
                palette: reference(6),
                width: 32,
                height: 32,
                cells: hud_cells.clone(),
            }),
            sc::SceneAsset::Palette(sc::Palette {
                reference: reference(6),
                colors,
            }),
            sc::SceneAsset::Visual(sc::Visual {
                reference: reference(7),
                frames: vec![sc::VisualFrame {
                    parts: vec![sc::VisualPart {
                        atlas: reference(2),
                        material: reference(6),
                        tile: 0,
                        offset: sc::ScreenPosition { x: 0, y: 0 },
                        size: sc::ObjectSize::Small,
                    }]
                    .into(),
                }]
                .into(),
            }),
            sc::SceneAsset::Font(sc::Font {
                reference: reference(8),
                atlas: reference(3),
            }),
        ];
        let resident = assets
            .iter()
            .map(sc::SceneAsset::reference)
            .collect::<Vec<_>>()
            .into();
        let atlases: Arc<[sc::AtlasBinding]> = vec![
            sc::AtlasBinding {
                atlas: reference(2),
                first_word: 0,
                resident_data: words[0..16].to_vec().into(),
            },
            sc::AtlasBinding {
                atlas: reference(3),
                first_word: 0x4000,
                resident_data: words[0x4000..0x4800].to_vec().into(),
            },
        ]
        .into();
        let maps: Arc<[sc::MapBinding]> = vec![
            sc::MapBinding {
                map: reference(4),
                first_word: 0x5000,
            },
            sc::MapBinding {
                map: reference(5),
                first_word: 0x5880,
            },
        ]
        .into();
        let images = vec![
            sc::CompiledImage {
                reference: image_ref(1),
                palette: palette_ref(1),
                words: words.into(),
                atlases: atlases.clone(),
                maps: maps.clone(),
            },
            sc::CompiledImage {
                reference: image_ref(2),
                palette: palette_ref(2),
                words: later_words.into(),
                atlases,
                maps,
            },
        ];
        let palettes = vec![
            sc::PaletteIncarnation {
                reference: palette_ref(1),
                colors,
            },
            sc::PaletteIncarnation {
                reference: palette_ref(2),
                colors: restored_colors,
            },
        ];
        let media = Arc::new(sc::SceneMedia::new(revision, assets, palettes, images).unwrap());
        let objects: Arc<[sc::SceneObject]> = (0..128)
            .map(|order| sc::SceneObject {
                order,
                visual: reference(7),
                frame: 0,
                part: 0,
                atlas: reference(2),
                material: reference(6),
                tile: 0,
                position: sc::ScreenPosition { x: -256, y: 0 },
                size: sc::ObjectSize::Small,
                palette: 0,
                priority: 0,
                flip_horizontal: false,
                flip_vertical: false,
            })
            .collect::<Vec<_>>()
            .into();
        let plane = |role, map, atlas, cells, palette| sc::ScenePlane {
            role,
            map: reference(map),
            atlas: reference(atlas),
            palette,
            width: 32,
            height: 32,
            cells,
            scroll: sc::ScreenPosition { x: 0, y: -1 },
        };
        let lines = (0..224)
            .map(|line| {
                let incarnation = if line < 112 { 1 } else { 2 };
                let palette = palette_ref(incarnation);
                sc::SceneLine {
                    media: image_ref(incarnation),
                    palette,
                    mode: sc::SceneMode::TilePlanes,
                    blank: false,
                    planes: [
                        plane(sc::PlaneRole::Main, 4, 2, blank_cells.clone(), palette),
                        plane(sc::PlaneRole::Secondary, 4, 2, blank_cells.clone(), palette),
                        plane(
                            sc::PlaneRole::HudFx,
                            5,
                            3,
                            if line < 112 {
                                hud_cells.clone()
                            } else {
                                later_cells.clone()
                            },
                            palette,
                        ),
                    ],
                    objects: objects.clone(),
                    object_selection: sc::ObjectSelection {
                        atlases: [reference(2); 2],
                        size_set: 0,
                        first_order: None,
                        interlaced: false,
                        even_frame: false,
                    },
                    windows: sc::SceneWindows {
                        extents: [sc::WindowExtent { first: 0, last: 0 }; 2],
                        layers: [sc::LayerWindow {
                            first_enabled: false,
                            first_inverted: false,
                            second_enabled: false,
                            second_inverted: false,
                            combination: sc::WindowCombination::Union,
                        }; 6],
                    },
                    routing: [
                        sc::ScreenRouting {
                            visible: [false, false, true, false, false],
                            masked: [false; 5],
                        },
                        sc::ScreenRouting {
                            visible: [false; 5],
                            masked: [false; 5],
                        },
                    ],
                    affine: sc::SceneAffine {
                        matrix: [0; 4],
                        center: sc::ScreenPosition { x: 0, y: 0 },
                        scroll: sc::ScreenPosition { x: 0, y: 0 },
                        flip_horizontal: false,
                        flip_vertical: false,
                        bounded_field: false,
                        fill_outside: false,
                    },
                    pixelation: sc::ScenePixelation {
                        enabled: [false; 3],
                        size: 1,
                        origin: 1,
                        modulo: [0; 256],
                    },
                    color: sc::SceneColor {
                        clip: sc::WindowCondition::Never,
                        prevent_math: sc::WindowCondition::Never,
                        enabled: [false; 6],
                        use_subscreen: false,
                        subtract: false,
                        half: false,
                        fixed: [0; 3],
                        brightness: 15,
                    },
                    object_aliases: Arc::from([]),
                }
            })
            .collect::<Vec<_>>()
            .into();
        (
            sc::SceneDescription {
                tick: 7,
                gameplay: [2; 32],
                reference: [3; 32],
                revision,
                view: sc::SceneView {
                    room: reference(1),
                    camera: sc::ScreenPosition { x: 17, y: -3 },
                },
                geometry: sc::SceneGeometry {
                    width: 256,
                    height: 224,
                    crop_origin: sc::ScreenPosition { x: 0, y: 0 },
                    interlaced: false,
                    even_frame: false,
                    overscan: false,
                    color: sc::ColorInterpretation::Bgr15Native,
                    quantization: sc::SceneQuantization::Native,
                },
                resident,
                lines,
                status: sc::DisplayStatus {
                    range_overflow: false,
                    time_overflow: false,
                    has_objects: false,
                    even_frame: false,
                    interlaced: false,
                    overscan: false,
                },
                initial_object_overflow: sc::ObjectOverflow {
                    range: false,
                    time: false,
                },
            },
            media,
        )
    }

    // Movies cannot reach authored portable media/incarnation changes or prove redraw independence.
    #[cfg_attr(test, test)]
    fn scene_pixels_preserve_line_incarnations_and_redraw() {
        let (description, media) = scene_fixture();
        let scene = sc::Scene::new(description.clone(), media.clone()).unwrap();
        let before = scene.clone();
        let frame = scene.render().unwrap();
        assert_eq!((frame.tick, frame.revision), (7, description.revision));
        assert_eq!(frame.pixels.len(), 224 * 256);
        for (index, pixel) in frame.pixels.iter().enumerate() {
            let band = index % 256 / 8 % 2;
            let expected = if band == 0 {
                [0, 255, 0, 0]
            } else if index / 256 < 112 {
                [0, 0, 255, 0]
            } else {
                [255, 0, 0, 0]
            };
            assert_eq!(*pixel, expected, "Scene pixel {index}");
        }
        let mut later = description;
        later.tick = 8;
        let last = later.lines[223].clone();
        Arc::make_mut(&mut later.lines).fill(last);
        let replacement = sc::Scene::new(later, media.clone()).unwrap();
        assert_ne!(replacement.render().unwrap().pixels, frame.pixels);
        drop(media);
        assert_eq!(scene.render().unwrap(), frame);
        assert_eq!(scene, before);
        assert_eq!(scene.description(), before.description());
        assert_eq!(scene.lines()[0].media.incarnation, 1);
        assert_eq!(scene.lines()[223].palette.incarnation, 2);
        assert_eq!(
            frame.raster_overflow,
            r::Overflow {
                range: false,
                time: false
            }
        );
    }

    // Movies cannot reach authored portable missing media, inconsistent history or unsupported starts.
    #[cfg_attr(test, test)]
    fn scene_rejects_missing_inconsistent_and_unsupported_history() {
        let (description, media) = scene_fixture();
        for case in 0..9 {
            let mut changed = description.clone();
            let expected = match case {
                0 => {
                    changed.lines = changed.lines[..223].to_vec().into();
                    sc::SceneError::Incomplete
                }
                1 => {
                    changed.resident = changed
                        .resident
                        .iter()
                        .copied()
                        .filter(|asset| asset.id != sc::AssetId(7))
                        .collect::<Vec<_>>()
                        .into();
                    sc::SceneError::MissingAsset(sc::AssetId(7))
                }
                2 => {
                    Arc::make_mut(&mut changed.lines)[0].media.incarnation = 99;
                    sc::SceneError::MissingIncarnation(changed.lines[0].media)
                }
                3 => {
                    Arc::make_mut(&mut changed.lines)[0].palette.incarnation = 99;
                    sc::SceneError::MissingIncarnation(changed.lines[0].palette)
                }
                4 => {
                    Arc::make_mut(&mut changed.lines)[0].palette = changed.lines[223].palette;
                    sc::SceneError::InconsistentMedia(changed.lines[0].media)
                }
                5 => {
                    Arc::make_mut(&mut changed.lines)[0].planes[0].role = sc::PlaneRole::HudFx;
                    sc::SceneError::Order
                }
                6 => {
                    let objects = &mut Arc::make_mut(&mut changed.lines)[0].objects;
                    Arc::make_mut(objects)[0].order = 127;
                    sc::SceneError::Order
                }
                7 => {
                    Arc::make_mut(&mut changed.lines)[0].mode =
                        sc::SceneMode::AffinePlane { extended: true };
                    sc::SceneError::ExtendedBackground
                }
                _ => {
                    Arc::make_mut(&mut changed.lines)[0].pixelation.enabled[0] = true;
                    sc::SceneError::EnabledMosaic(0)
                }
            };
            assert_eq!(
                sc::Scene::new(changed, media.clone()).err(),
                Some(expected),
                "Scene rejection {case}"
            );
        }
        let mut changed = description.clone();
        changed.geometry.width = 255;
        assert_eq!(
            sc::Scene::new(changed, media.clone()).err(),
            Some(sc::SceneError::Geometry)
        );
        let mut changed = description;
        changed.gameplay = [0; 32];
        assert_eq!(
            sc::Scene::new(changed, media).err(),
            Some(sc::SceneError::Identity)
        );
    }

    // Movies cannot reach authored portable registry/revision/start rejection or stale palette aliases.
    #[cfg_attr(test, test)]
    fn scene_media_rejects_missing_revisions_and_stale_aliases() {
        let (_, media) = scene_fixture();
        for case in 0..7 {
            let mut assets = media.assets().cloned().collect::<Vec<_>>();
            let mut palettes = media.palette_incarnations().cloned().collect::<Vec<_>>();
            let mut images = media.compiled_images().cloned().collect::<Vec<_>>();
            let expected = match case {
                0 => {
                    assets.retain(|asset| asset.reference().id != sc::AssetId(6));
                    sc::SceneError::MissingAsset(sc::AssetId(6))
                }
                1 => {
                    palettes[0].reference.asset.revision = sc::Revision([99; 32]);
                    sc::SceneError::Revision
                }
                2 => {
                    images.push(images[0].clone());
                    sc::SceneError::DuplicateIncarnation(images[0].reference)
                }
                3 => {
                    palettes.remove(0);
                    sc::SceneError::MissingIncarnation(images[0].palette)
                }
                4 => {
                    images[0].words = images[0].words[..r::MAX_INDEXED_WORDS - 1].to_vec().into();
                    sc::SceneError::Incomplete
                }
                5 => {
                    Arc::make_mut(&mut images[1].words)[0x8005] ^= 1;
                    sc::SceneError::InconsistentMedia(images[1].reference)
                }
                _ => {
                    Arc::make_mut(&mut images[0].words)[0x4008] ^= 1;
                    sc::SceneError::InconsistentMedia(images[0].reference)
                }
            };
            assert_eq!(
                sc::SceneMedia::new(media.revision(), assets, palettes, images).err(),
                Some(expected),
                "media rejection {case}"
            );
        }
    }

    // Movies cannot reach authored portable Reference quantization/start rejection.
    // SOURCE01's Scene consumer admits Native and cannot claim Reference image support.
    #[cfg_attr(test, test)]
    fn scene_rejects_reference_quantization() {
        let (mut description, media) = scene_fixture();
        description.geometry.quantization = sc::SceneQuantization::Reference;
        assert_eq!(
            sc::Scene::new(description, media).err(),
            Some(sc::SceneError::Unsupported)
        );
    }

    // Movies cannot author portable start/profile rejection. This synthetic metadata
    // prefix intentionally omits DisplayDescription and cannot seal a G or display start.
    // It exercises only the concrete constructor's admission before MissingDisplay.
    fn scene_start_prefix() -> (
        ss::DisplayStartRequirements,
        ss::DisplayStartDescription,
        Arc<sc::SceneMedia>,
    ) {
        let (mut scene, media) = scene_fixture();
        scene.tick = 0;
        let reference = ss::ReferenceProfile {
            fingerprint: scene.reference,
            release: [11; 32],
            capture: [12; 32],
            ticks_numerator: 37,
            ticks_denominator: 5,
            arithmetic_policy: [13; 32],
            input_policy: [14; 32],
            observation_policy: [15; 32],
            color_policy: [16; 32],
            slowdown_policy: [17; 32],
            glitch_policy: [18; 32],
            audio_timing_policy: [19; 32],
            image: ss::SceneImagePolicy {
                width: scene.geometry.width,
                height: scene.geometry.height,
                crop_origin: scene.geometry.crop_origin,
                color: scene.geometry.color,
                quantization: scene.geometry.quantization,
            },
        };
        let identity = ss::StartIdentity {
            schema: ss::DISPLAY_START_SCHEMA,
            gameplay: scene.gameplay,
            reference: scene.reference,
            capture: [4; 32],
            media: scene.revision,
            committed_tick: 0,
            next_input_record: 23,
        };
        let input_history = ss::InputHistory {
            capture: identity.capture,
            policy: reference.input_policy,
            ports: [ss::PortHistory {
                sampled: 3,
                previous: 1,
                pressed: 2,
                repeated: 0,
                repeat_counter: 17,
            }; 2],
            debug_second_port: true,
            repeat_first: 31,
            repeat_next: 7,
            released: 4,
            timed_held: 3,
            previous_timed_held: 1,
            newly_held: 2,
            held_timer: 13,
            held_timer_reset: 29,
            samus_held: 3,
            samus_pressed: 2,
            bindings: ss::ActionBindings {
                shoot: 1,
                jump: 2,
                run: 4,
                cancel: 8,
                switch: 16,
                aim_down: 32,
                aim_up: 64,
            },
            bug_fix_counter: 9,
        };
        let continuation = ss::ContinuationHistory {
            capture: identity.capture,
            observation_policy: reference.observation_policy,
            phase: ss::CommittedPhase::InitialObservation,
            next_input_record: identity.next_input_record,
            outer: 18,
            primary: 3,
            secondary: 7,
            music: 2,
            message: 4,
            auxiliary: 1,
            captured_mode: Some(8),
        };
        let audio_revision = sc::Revision([5; 32]);
        let declarations: Arc<[ss::AudioProgramDeclaration]> = vec![ss::AudioProgramDeclaration {
            program: sc::AssetId(1),
            bank: sc::AssetId(2),
            revision: audio_revision,
            definition: [6; 32],
        }]
        .into();
        let audio = ss::InitialAudioIntent {
            capture: identity.capture,
            revision: audio_revision,
            timing_policy: reference.audio_timing_policy,
            declarations: declarations.clone(),
            music: ss::OngoingMusic::Playing(ss::ProgramAnchor {
                program: sc::AssetId(1),
                revision: audio_revision,
                source_interval: 23,
                phase: 59,
                timing_policy: reference.audio_timing_policy,
            }),
            ambience: Arc::from([]),
        };
        let requirements = ss::DisplayStartRequirements {
            schema: identity.schema,
            gameplay: identity.gameplay,
            hdma_content: Arc::new(hs::HdmaCatalog::new(identity.gameplay, vec![]).unwrap()),
            capture: identity.capture,
            reference,
            media: identity.media,
            audio_revision,
            audio_programs: declarations,
        };
        let description = ss::DisplayStartDescription {
            identity: Some(identity),
            reference: Some(reference),
            input_history: Some(input_history),
            continuation: Some(continuation),
            audio: Some(audio),
            display: None,
            resident: None,
            hdma_content: None,
            committed_scene: Some(sc::Scene::new(scene, media.clone()).unwrap()),
        };
        (requirements, description, media)
    }

    // Movies cannot reach authored portable missing start fields or malformed history admission.
    #[cfg_attr(test, test)]
    fn scene_start_rejects_missing_fields_and_invalid_history_prefix() {
        let (requirements, description, media) = scene_start_prefix();
        assert!(matches!(
            ss::ValidatedDisplayStart::construct(description, media, &requirements),
            Err(ss::StartError::MissingDisplay)
        ));
        for case in 0..12 {
            let (requirements, mut description, media) = scene_start_prefix();
            let expected = match case {
                0 => {
                    description.identity = None;
                    ss::StartError::MissingIdentity
                }
                1 => {
                    description.reference = None;
                    ss::StartError::MissingReference
                }
                2 => {
                    description.input_history = None;
                    ss::StartError::MissingInputHistory
                }
                3 => {
                    description.continuation = None;
                    ss::StartError::MissingContinuation
                }
                4 => {
                    description.audio = None;
                    ss::StartError::MissingAudioAnchor
                }
                5 => {
                    description.identity.as_mut().unwrap().schema += 1;
                    ss::StartError::Schema
                }
                6 => {
                    description.input_history.as_mut().unwrap().capture = [99; 32];
                    ss::StartError::InputHistory
                }
                7 => {
                    description.continuation.as_mut().unwrap().next_input_record += 1;
                    ss::StartError::Continuation
                }
                8 => {
                    description.identity.as_mut().unwrap().committed_tick = 1;
                    ss::StartError::Continuation
                }
                9 => {
                    description.continuation.as_mut().unwrap().outer = 4;
                    ss::StartError::Continuation
                }
                10 => {
                    description.continuation.as_mut().unwrap().captured_mode = Some(44);
                    ss::StartError::Continuation
                }
                _ => {
                    let history = description.continuation.as_mut().unwrap();
                    history.outer = 0;
                    history.captured_mode = Some(246);
                    ss::StartError::Continuation
                }
            };
            let error = ss::ValidatedDisplayStart::construct(description, media, &requirements)
                .err()
                .unwrap();
            assert_eq!(
                std::mem::discriminant(&error),
                std::mem::discriminant(&expected),
                "start prefix rejection {case}: {error:?}"
            );
        }
    }

    // Movies cannot reach authored portable changed-profile or missing audio-anchor rejection.
    #[cfg_attr(test, test)]
    fn scene_start_rejects_changed_profile_and_audio_prefix() {
        for case in 0..12 {
            let (requirements, mut description, media) = scene_start_prefix();
            let profile = description.reference.as_mut().unwrap();
            match case {
                0 => profile.ticks_numerator += 1,
                1 => profile.release[0] ^= 1,
                2 => profile.capture[0] ^= 1,
                3 => profile.arithmetic_policy[0] ^= 1,
                4 => profile.input_policy[0] ^= 1,
                5 => profile.observation_policy[0] ^= 1,
                6 => profile.color_policy[0] ^= 1,
                7 => profile.slowdown_policy[0] ^= 1,
                8 => profile.glitch_policy[0] ^= 1,
                9 => profile.audio_timing_policy[0] ^= 1,
                10 => profile.image.width -= 1,
                _ => profile.image.quantization = sc::SceneQuantization::Reference,
            }
            assert_eq!(profile.fingerprint, requirements.reference.fingerprint);
            assert!(matches!(
                ss::ValidatedDisplayStart::construct(description, media, &requirements),
                Err(ss::StartError::Fingerprint)
            ));
        }
        for case in 0..4 {
            let (requirements, mut description, media) = scene_start_prefix();
            let audio = description.audio.as_mut().unwrap();
            match case {
                0 => audio.capture = [99; 32],
                1 => audio.timing_policy = [99; 32],
                2 => {
                    if let ss::OngoingMusic::Playing(anchor) = &mut audio.music {
                        anchor.source_interval += 1;
                    }
                }
                _ => audio.declarations = Arc::from([]),
            }
            let error = ss::ValidatedDisplayStart::construct(description, media, &requirements)
                .err()
                .unwrap();
            assert!(matches!(
                (case, error),
                (0..=2, ss::StartError::MissingAudioAnchor) | (3, ss::StartError::AudioDeclaration)
            ));
        }
    }

    // Movies cannot supply an independently authored complete saved display.
    fn prepared_message_line() -> DisplayLine {
        let background = r::Background {
            horizontal_scroll: 0,
            vertical_scroll: u16::MAX,
            map_words: 0x5880,
            tile_words: 0x4000,
            wider: false,
            higher: false,
        };
        let window = r::Window {
            first_enabled: false,
            first_inverted: false,
            second_enabled: false,
            second_inverted: false,
            logic: 0,
        };
        DisplayLine {
            mode: 1,
            extended_background: false,
            forced_blank: false,
            backgrounds: [background; 3],
            windows: r::Windows {
                first: [0; 2],
                second: [0; 2],
                native_flags: 0,
                layers: [window; 6],
            },
            routing: r::Routing {
                enabled: [4, 0],
                windowed: [0; 2],
            },
            affine: r::Affine {
                matrix: [0; 8],
                horizontal_flip: false,
                vertical_flip: false,
                large_field: false,
                character_fill: false,
            },
            color: r::ColorMath {
                clip: 0,
                prevent: 0,
                enabled: [false; 6],
                add_subscreen: false,
                subtract: false,
                half: false,
                fixed: [0; 3],
                brightness: 15,
            },
        }
    }

    // Movies cannot author complete font/media identities or differing restored palettes.
    fn prepared_message_fixture(text: u16) -> (Arc<Definitions>, PreparedStart) {
        let revision = Revision([1; 32]);
        let mut font = vec![0; 0x800];
        font[8..16].fill(0xff00);
        font[16..24].fill(0x00ff);
        let message = MessageDefinition {
            id: MessageId(1),
            kind: MessageKind::Content,
            size: BoxSize::Small,
            modifier: ButtonModifier::None,
            button_byte_offset: None,
            text: (0..32).map(|i| text + (i & 1)).collect::<Vec<_>>().into(),
        };
        let map = MessageDefinition {
            id: MessageId(2),
            kind: MessageKind::Map,
            size: BoxSize::Large,
            ..message.clone()
        };
        let definition = FontDefinition {
            id: AssetId(1),
            revision,
            first_word: 0x4000,
            words: font.clone().into(),
        };
        let definitions =
            Arc::new(Definitions::new(revision, definition, vec![message, map]).unwrap());
        let image = |restored: bool| {
            let mut words = vec![0; 0x8000];
            words[0x4000..0x4800].copy_from_slice(&font);
            for (i, word) in words[0x5880..0x5c00].iter_mut().enumerate() {
                *word = if restored {
                    14
                } else {
                    0x3801 + (i & 1) as u16
                };
            }
            let mut palette = [0; 256];
            palette[25] = if restored { 31 << 5 } else { 31 };
            palette[26] = if restored { 31 << 10 } else { 31 << 5 };
            Arc::new(IndexedImage::new(words, palette).unwrap())
        };
        let saved = FullDisplay {
            media_id: AssetId(2),
            revision,
            media: image(false),
            sprites: Arc::new(PreparedSprites {
                words: [0; 256],
                extension: [0x55; 32],
                tile_words: [0; 2],
                size: 0,
                rotation: None,
                interlace: false,
                even_frame: false,
            }),
            mosaic: Arc::new(PreparedMosaic {
                native_enabled: 0,
                background0_enabled: false,
                size: 1,
                start_line: 1,
                modulo: [0; 256],
            }),
            lines: Box::new([prepared_message_line(); 224]),
            overflow: r::Overflow {
                range: false,
                time: false,
            },
        };
        let restored = FullDisplay {
            media_id: AssetId(4),
            media: image(true),
            ..saved.clone()
        };
        let mut active = saved.clone();
        active.media_id = AssetId(3);
        for line in active.lines.iter_mut() {
            line.backgrounds[2].map_words = 0x5800;
        }
        (
            definitions,
            PreparedStart {
                saved,
                active,
                restored,
                tile_workspace: [0; 768],
                shoot_button: 0x80,
                run_button: 0x20,
                initial_interval: 0,
                initial_wait_pending: false,
            },
        )
    }

    fn prepared_message_pixels(
        frame: &MessageFrame,
        bands: [[u8; 4]; 2],
        rows: std::ops::Range<usize>,
    ) {
        assert_eq!(frame.pixels.len(), 224 * 256);
        for (i, pixel) in frame.pixels.iter().enumerate() {
            let expected = if rows.contains(&(i / 256)) {
                bands[i % 256 / 8 % 2]
            } else {
                [0; 4]
            };
            assert_eq!(*pixel, expected, "prepared pixel {i}");
        }
    }

    fn prepared_message_waits(
        owner: &mut PreparedOwner,
        sample: InputSample,
        intervals: std::ops::RangeInclusive<u64>,
    ) -> StepOutput {
        intervals
            .map(|interval| {
                let step = owner.resume(interval, sample).unwrap();
                assert_eq!(step.outcome, Outcome::Suspended);
                assert_eq!(step.effects, [Effect::AdvanceAudio]);
                step
            })
            .last()
            .unwrap()
    }

    // Shared movie pixels cannot prove saved-map recovery, 360 waits or stale-completion ownership.
    #[cfg_attr(test, test)]
    fn prepared_message_pixels_delay_queue_and_checkpoint() {
        let (definitions, start) = prepared_message_fixture(0x3801);
        let (mut owner, frame) = PreparedOwner::open(definitions.clone(), start.clone()).unwrap();
        prepared_message_pixels(&frame, [[0, 255, 0, 0], [0, 0, 255, 0]], 0..224);
        let sample = InputSample {
            new_keys: 1,
            held_keys: 0,
            bug_fix_counter: 1,
        };
        assert_eq!(owner.resume(1, sample).err(), Some(Fault::NoRequest));
        owner.queue(MessageId(1)).unwrap();
        let mut step = owner.resume(1, sample).unwrap();
        assert_eq!(step.outcome, Outcome::Suspended);
        assert_eq!(step.effects, [Effect::CancelSounds]);
        prepared_message_pixels(&step.frame, [[0; 4]; 2], 0..224);
        let checkpoint = owner.checkpoint().unwrap();
        (owner, _) = PreparedOwner::reopen(definitions.clone(), start.clone(), checkpoint).unwrap();
        step = prepared_message_waits(&mut owner, sample, 2..=14);
        prepared_message_pixels(&step.frame, [[0, 0, 255, 0], [16, 239, 140, 0]], 119..127);
        assert_eq!(owner.resume(14, sample).err(), Some(Fault::Interval));
        assert_eq!(owner.poll(MessageId(2)).unwrap(), None);
        owner.queue(MessageId(999)).unwrap();
        step = prepared_message_waits(&mut owner, sample, 15..=214);
        let checkpoint = owner.checkpoint().unwrap();
        let reopened =
            PreparedOwner::reopen(definitions.clone(), start.clone(), checkpoint).unwrap();
        assert_eq!(reopened.1.pixels, step.frame.pixels);
        owner = reopened.0;
        prepared_message_waits(&mut owner, sample, 215..=374);
        let dismiss = InputSample {
            held_keys: 1,
            ..sample
        };
        for (interval, input) in [(375, sample), (376, dismiss)] {
            step = owner.resume(interval, input).unwrap();
            assert_eq!(step.outcome, Outcome::Suspended);
            assert!(step.effects.is_empty());
        }
        prepared_message_waits(&mut owner, sample, 377..=388);
        step = owner.resume(389, sample).unwrap();
        assert_eq!(step.outcome, Outcome::Complete);
        assert_eq!(
            step.effects,
            [Effect::AdvanceAudio, Effect::ResumeMovementSound]
        );
        prepared_message_pixels(&step.frame, [[255, 0, 0, 0], [0, 255, 0, 0]], 0..224);
        assert_eq!(owner.resume(390, sample).err(), Some(Fault::NoRequest));
        owner.queue(MessageId(2)).unwrap();
        step = owner.resume(390, sample).unwrap();
        assert_eq!(step.outcome, Outcome::Suspended);
        assert_eq!(step.effects, [Effect::CancelSounds]);
        assert_eq!(owner.poll(MessageId(1)).unwrap(), Some(0));
        owner.queue(MessageId(999)).unwrap();
        prepared_message_waits(&mut owner, sample, 391..=413);
        step = owner.resume(414, dismiss).unwrap();
        assert_eq!(step.outcome, Outcome::Suspended);
        assert!(step.effects.is_empty());
        prepared_message_waits(&mut owner, sample, 415..=426);
        step = owner.resume(427, sample).unwrap();
        assert_eq!(step.outcome, Outcome::Complete);
        assert_eq!(
            step.effects,
            [
                Effect::AdvanceAudio,
                Effect::ResumeMovementSound,
                Effect::SelectPause
            ]
        );
        prepared_message_pixels(&step.frame, [[255, 0, 0, 0], [0, 255, 0, 0]], 0..224);
        assert_eq!(owner.poll(MessageId(2)).unwrap(), Some(0));
        assert_eq!(owner.resume(428, sample).err(), Some(Fault::NoRequest));
        owner.queue(MessageId(999)).unwrap();
        assert_eq!(
            owner.resume(428, sample).err(),
            Some(Fault::MissingMessage(MessageId(999)))
        );
    }

    // Movies cannot author missing media IDs, same-revision changes or fresh prepared reset admission.
    #[cfg_attr(test, test)]
    fn prepared_message_rejects_changed_content_and_resets() {
        let (definitions, start) = prepared_message_fixture(0x3801);
        let (mut owner, _) = PreparedOwner::open(definitions.clone(), start.clone()).unwrap();
        let sample = InputSample {
            new_keys: 0,
            held_keys: 0,
            bug_fix_counter: 0,
        };
        owner.queue(MessageId(1)).unwrap();
        owner.resume(1, sample).unwrap();
        let checkpoint = owner.checkpoint().unwrap();
        for index in 0..3 {
            let mut missing = start.clone();
            match index {
                0 => &mut missing.saved,
                1 => &mut missing.active,
                _ => &mut missing.restored,
            }
            .media_id = AssetId(0);
            assert_eq!(
                owner.reset(missing.clone()).err(),
                Some(Fault::MissingDisplay)
            );
            assert_eq!(
                PreparedOwner::open(definitions.clone(), missing).err(),
                Some(Fault::MissingDisplay)
            );
            let mut changed = start.clone();
            match index {
                0 => &mut changed.saved,
                1 => &mut changed.active,
                _ => &mut changed.restored,
            }
            .media_id = AssetId(99);
            assert_eq!(
                PreparedOwner::reopen(definitions.clone(), changed, checkpoint.clone()).err(),
                Some(Fault::ChangedContent)
            );
        }
        let (changed, _) = prepared_message_fixture(0x3802);
        assert_eq!(
            PreparedOwner::reopen(changed, start.clone(), checkpoint.clone()).err(),
            Some(Fault::ChangedContent)
        );
        for (word_delta, palette_delta) in [(1, 0), (0, 1)] {
            let mut changed = start.clone();
            let mut words = (0..0x8000)
                .map(|i| start.saved.media.word(i).unwrap())
                .collect::<Vec<_>>();
            let mut palette = core::array::from_fn(|i| start.saved.media.palette(i as u8).unwrap());
            words[0x5880] ^= word_delta;
            palette[26] ^= palette_delta;
            changed.saved.media = Arc::new(IndexedImage::new(words, palette).unwrap());
            assert_eq!(
                PreparedOwner::reopen(definitions.clone(), changed, checkpoint.clone()).err(),
                Some(Fault::ChangedContent)
            );
        }
        let mut changed = start.clone();
        changed.initial_wait_pending = true;
        assert_eq!(
            PreparedOwner::reopen(definitions.clone(), changed, checkpoint).err(),
            Some(Fault::ChangedContent)
        );
        let (mut reset, frame) = owner.reset(start).unwrap();
        prepared_message_pixels(&frame, [[0, 255, 0, 0], [0, 0, 255, 0]], 0..224);
        assert_eq!(reset.resume(1, sample).err(), Some(Fault::NoRequest));
        reset.queue(MessageId(1)).unwrap();
        assert_eq!(
            reset.resume(1, sample).unwrap().effects,
            [Effect::CancelSounds]
        );
    }

    // Movies share the renderer and cannot supply independent pixel/failure expectations.
    #[cfg_attr(test, test)]
    fn independently_authored_raster_pixels_and_prefixes() {
        for id in 38..=43 {
            assert!(
                sm::verify_room_background_case(id).is_some(),
                "raster case {id}"
            );
        }
    }

    // Movies cannot select tied/negative native headers, unsupported mappers or raw mirrors.
    #[cfg_attr(test, test)]
    fn room_pack_uses_native_header_selection_and_final_block_mirroring() {
        fn header(source: &mut [u8], location: usize, opcode: u8) {
            source[location + 0x15] = 0x20;
            source[location + 0x16] = 0;
            source[location + 0x19] = 0;
            source[location + 0x1c..location + 0x20].copy_from_slice(&[0xff, 0xff, 0, 0]);
            source[location + 0x3c..location + 0x3e].copy_from_slice(&[0, 0x80]);
            source[location + 0x40 - 0x8000] = opcode;
        }
        let mut raw = vec![0; 0x10200];
        // HiROM scores-2 while LoROM scores-16: max=0 still selects candidate0.
        raw[0xffd5] = 0x20;
        raw[0xffd6] = 7;
        raw[0x8000] = 0x78;
        assert_eq!(
            sm::room_pack::normalize_source(&raw).unwrap().len(),
            0x20000
        );
        raw.fill(0);
        header(&mut raw, 0x7fc0, 0x78);
        header(&mut raw, 0x81c0, 0x78);
        assert_eq!(
            sm::room_pack::normalize_source(&raw).unwrap().len(),
            0x20000
        );
        raw.fill(0);
        header(&mut raw, 0x81c0, 0x78);
        let normalized = sm::room_pack::normalize_source(&raw).unwrap();
        assert_eq!(normalized.len(), 0x10000);
        assert_eq!(normalized, raw[0x200..]);
        let mut odd = vec![0; 0x1007b];
        header(&mut odd, 0x81c0, 0x78);
        assert_eq!(
            sm::room_pack::normalize_source(&odd).unwrap().len(),
            0x10000
        );
        for location in [0xffc0, 0x101c0] {
            raw.fill(0);
            header(&mut raw, 0x7fc0, 0x4c);
            header(&mut raw, location, 0x78);
            assert_eq!(
                sm::room_pack::normalize_source(&raw)
                    .unwrap_err()
                    .to_string(),
                "room pack vanilla profile requires native-selected LoROM cartridge",
            );
        }
        let mut three_mib = vec![0x11; 0x300000];
        three_mib[0x100000..0x200000].fill(0x22);
        three_mib[0x200000..].fill(0x33);
        header(&mut three_mib, 0x7fc0, 0x78);
        let normalized = sm::room_pack::normalize_source(&three_mib).unwrap();
        assert_eq!(normalized.len(), 0x400000);
        assert_eq!(&normalized[..three_mib.len()], three_mib);
        assert_eq!(&normalized[three_mib.len()..], &three_mib[0x200000..]);
        assert!(sm::room_pack::normalize_source(&[]).is_err());
    }

    // Movies cannot supply deliberately malformed, checksum-valid portable packs.
    #[cfg_attr(test, test)]
    fn room_pack_rejects_checksum_valid_structural_errors() {
        fn pack(metadata: &[u8]) -> Vec<u8> {
            let mut bytes = b"SMRP\x01\x00\x01\x00\x00\x00\x00\x00".to_vec();
            bytes.extend_from_slice(&(metadata.len() as u32).to_le_bytes());
            bytes.extend_from_slice(&0u32.to_le_bytes());
            bytes.extend_from_slice(&[0; 32]);
            bytes.extend_from_slice(metadata);
            let checksum = sm::room_pack::content_fingerprint(&bytes[..20], &bytes[52..]);
            bytes[20..52].copy_from_slice(&checksum);
            bytes
        }
        fn rejected(metadata: &[u8], message: &str) {
            assert_eq!(
                sm::room_pack::load(&pack(metadata))
                    .unwrap_err()
                    .to_string(),
                message
            );
        }
        let mut room = 7u32.to_le_bytes().to_vec();
        room.extend_from_slice(&[0, 0, 0, 0, 1, 1, 0, 0, 2]);
        room.extend_from_slice(&7u32.to_le_bytes());
        room.extend_from_slice(&1u32.to_le_bytes());
        room.push(0);
        room.extend_from_slice(&7u32.to_le_bytes());
        for id in [7u32, 6] {
            let mut metadata = 2u32.to_le_bytes().to_vec();
            metadata.extend_from_slice(&room);
            metadata.extend_from_slice(&id.to_le_bytes());
            rejected(&metadata, "duplicate or noncanonical world/key ID");
        }
        let mut unknown = 1u32.to_le_bytes().to_vec();
        room[21] = 0xff;
        unknown.extend_from_slice(&room);
        rejected(&unknown, "unknown room selector tag");
        rejected(&65_537u32.to_le_bytes(), "world/key count is out of bounds");
        let mut truncated = 1u32.to_le_bytes().to_vec();
        truncated.extend_from_slice(&room[..7]);
        rejected(&truncated, "truncated world/key record");
        rejected(&[0; 56], "room pack lacks CurrentLoadStation site 6/0");
        rejected(&[0; 57], "trailing world/key records");
        for (offset, value, message) in [
            (4, 2, "unsupported room pack schema"),
            (6, 2, "unsupported room pack profile/options"),
            (8, 1, "unsupported room pack profile/options"),
        ] {
            let mut bytes = pack(&[0; 56]);
            bytes[offset] = value;
            let checksum = sm::room_pack::content_fingerprint(&bytes[..20], &bytes[52..]);
            bytes[20..52].copy_from_slice(&checksum);
            assert_eq!(
                sm::room_pack::load(&bytes).unwrap_err().to_string(),
                message
            );
        }
    }

    // Movies cannot select duplicate or missing application pack arguments.
    #[cfg_attr(test, test)]
    fn room_pack_selection_rejects_missing_duplicate_and_option_values() {
        for (args, message) in [
            (vec!["--room-pack"], "--room-pack requires a path"),
            (vec!["--room-pack", ""], "--room-pack requires a path"),
            (
                vec!["--room-pack", "--frames", "2"],
                "--room-pack requires a path",
            ),
            (vec!["--room-pack", "-h"], "--room-pack requires a path"),
            (
                vec!["--room-pack=path"],
                "room pack selection requires --room-pack PATH",
            ),
            (
                vec!["--room-pack", "absent", "--room-pack", "other"],
                "duplicate --room-pack selection",
            ),
        ] {
            let args: Vec<String> = args.into_iter().map(str::to_owned).collect();
            assert_eq!(
                sm::room_pack::select(&args).err().unwrap().to_string(),
                message
            );
        }
        let args = ["verify", "--debug", "rom", "inputs"].map(str::to_owned);
        let selection = sm::room_pack::select(&args).unwrap();
        assert!(selection.world.is_none());
        assert_eq!(selection.arguments, args);
    }

    fn gameplay() -> CoverageState {
        CoverageState {
            state: 8,
            area: 1,
            room: 0x9dc7,
            ..CoverageState::default()
        }
    }

    fn required(name: &str) -> RequiredExpectation {
        RequiredExpectation {
            name: name.into(),
            kind: Expectation::parse(name).unwrap(),
            first_frame: None,
            suit_progress: SuitProgress::AwaitingAward,
        }
    }

    #[cfg_attr(test, test)]
    fn expectation_parser_rejects_unknown_or_malformed_values() {
        for value in [
            "",
            "area:7",
            "area:Crateria",
            "boss:ceres-ridley",
            "item:unknown",
            "pickup:health",
            "room:0000",
            "room:0x91f8",
            "room:FFFFF",
            "room:zzzz",
            "door:0000",
            "door:0x91f8",
            "elevator:FFFFF",
            "elevator:zzzz",
            "encounter:ridley",
            "credits:1",
            "state:",
            "state:-1",
            "state:+2",
            "state:45",
            "state:65536",
        ] {
            assert!(Expectation::parse(value).is_err(), "accepted {value}");
        }
        let menu = CoverageState {
            state: 2,
            ..CoverageState::default()
        };
        let state = Expectation::parse("state:2").unwrap();
        assert!(state.observed(&gameplay(), &menu, false));
        assert!(!state.observed(&menu, &gameplay(), false));
        assert_eq!(
            Expectation::parse("room:91f8"),
            Expectation::parse("room:91F8")
        );
        assert_eq!(
            Expectation::parse("door:a7de").unwrap(),
            Expectation::Door(0xa7de)
        );
        assert_eq!(
            Expectation::parse("elevator:A7DE").unwrap(),
            Expectation::Elevator(0xa7de)
        );
        assert_eq!(
            Expectation::parse("encounter:ceres-ridley").unwrap(),
            Expectation::CeresRidley
        );
        assert_eq!(
            Expectation::parse("boss:mother-brain").unwrap(),
            Expectation::Boss { area: 5, mask: 2 }
        );
    }

    #[cfg_attr(test, test)]
    fn inventory_events_require_a_new_award_inside_the_same_live_room() {
        let before = gameplay();
        let mut after = before;
        after.items[0] = 4;
        after.capacity[1] = 5;
        let item = Expectation::parse("item:morph-ball").unwrap();
        let pickup = Expectation::parse("pickup:missile").unwrap();
        assert!(item.observed(&before, &after, false));
        assert!(pickup.observed(&before, &after, false));
        assert!(
            !item.observed(&after, &after, false),
            "loaded inventory is not a pickup"
        );
        assert!(!pickup.observed(&after, &after, false));
        for moved in [
            CoverageState {
                room: 0x9999,
                ..after
            },
            CoverageState { area: 2, ..after },
            CoverageState { state: 7, ..after },
        ] {
            assert!(!item.observed(&before, &moved, false));
            assert!(!pickup.observed(&before, &moved, false));
        }
        assert!(!item.observed(&CoverageState { state: 7, ..before }, &after, false));
    }

    #[cfg_attr(test, test)]
    fn boss_flags_require_a_rising_bit_in_the_corresponding_area() {
        let boss = Expectation::parse("boss:mother-brain").unwrap();
        let before = CoverageState {
            area: 5,
            ..gameplay()
        };
        let mut after = before;
        after.bosses[5] = 1;
        assert!(!boss.observed(&before, &after, false));
        after.bosses[5] = 2;
        assert!(boss.observed(&before, &after, false));
        assert!(!boss.observed(&after, &after, false));
        assert!(!boss.observed(
            &CoverageState { area: 1, ..before },
            &CoverageState { area: 1, ..after },
            false
        ));
    }

    #[cfg_attr(test, test)]
    fn credits_require_enabled_script_advancement_not_ending_presence() {
        let ending = CoverageState {
            state: 39,
            ..CoverageState::default()
        };
        let running = CoverageState {
            credits_enabled: 0x8000,
            credits_cursor: 0x8000,
            ..ending
        };
        assert!(!Expectation::Credits.observed(&ending, &ending, false));
        assert!(!Expectation::Credits.observed(&running, &running, false));
        assert!(Expectation::Credits.observed(
            &running,
            &CoverageState {
                credits_scroll: 1,
                ..running
            },
            false
        ));
        assert!(!Expectation::Credits.observed(
            &CoverageState {
                state: 1,
                ..running
            },
            &CoverageState {
                state: 1,
                credits_scroll: 1,
                ..running
            },
            false
        ));
        let finishing = CoverageState {
            cinematic_function: 0xdb9d,
            ..running
        };
        let finished = CoverageState {
            cinematic_function: 0xe0f4,
            ..ending
        };
        let completion = Expectation::parse("credits-complete").unwrap();
        assert!(completion.observed(&finishing, &finished, false));
        assert!(!completion.observed(&finished, &finished, false));
        assert!(!completion.observed(&finishing, &finishing, false));
        assert!(!completion.observed(&finishing, &ending, false));
        assert!(!completion.observed(&running, &finished, false));
    }

    #[cfg_attr(test, test)]
    fn death_must_begin_in_the_segment_and_reach_game_over() {
        let mut coverage = ReplayCoverage {
            previous: gameplay(),
            expected: vec![required("death")],
            ..ReplayCoverage::default()
        };
        coverage.observe_expectations(
            CoverageState {
                state: 19,
                ..gameplay()
            },
            100,
        );
        assert_eq!(coverage.all_expected_frame(), None);
        coverage.observe_expectations(
            CoverageState {
                state: 25,
                ..gameplay()
            },
            110,
        );
        coverage.observe_expectations(
            CoverageState {
                state: 26,
                ..gameplay()
            },
            111,
        );
        assert_eq!(coverage.all_expected_frame(), Some(111));
        let mut loaded = ReplayCoverage {
            previous: CoverageState {
                state: 25,
                ..gameplay()
            },
            expected: vec![required("death")],
            ..ReplayCoverage::default()
        };
        loaded.observe_expectations(
            CoverageState {
                state: 26,
                ..gameplay()
            },
            200,
        );
        assert_eq!(
            loaded.all_expected_frame(),
            None,
            "loaded death phase is not an observed sequence"
        );
    }

    #[cfg_attr(test, test)]
    fn every_expectation_is_required_and_repeated_hits_keep_the_first_frame() {
        let mut coverage = ReplayCoverage {
            previous: gameplay(),
            expected: vec![required("area:brinstar"), required("pickup:energy")],
            ..ReplayCoverage::default()
        };
        coverage.observe_expectations(gameplay(), 100);
        assert_eq!(coverage.all_expected_frame(), None);
        let after = CoverageState {
            capacity: [199, 0, 0, 0, 0],
            ..gameplay()
        };
        coverage.observe_expectations(after, 200);
        assert_eq!(coverage.all_expected_frame(), Some(200));
        coverage.observe_expectations(after, 300);
        assert_eq!(coverage.all_expected_frame(), Some(200));
    }
    #[cfg_attr(test, test)]
    fn suit_award_waits_for_later_hdma_and_control_return() {
        for (name, mask) in [("item:varia", 1), ("item:gravity", 0x20)] {
            let before = CoverageState {
                player_handlers: [0xe695, 0xe725, 0xeb52],
                ..gameplay()
            };
            // Equipment collection precedes the message and HDMA animation.
            let awarded = CoverageState {
                items: [mask, 0],
                ..before
            };
            let stage3 = CoverageState {
                suit_substate: 3,
                player_handlers: [0xe713, 0xe8cd, 0xec1d],
                ..awarded
            };
            let stage4 = CoverageState {
                suit_substate: 4,
                ..stage3
            };
            let completed = CoverageState {
                suit_substate: 0,
                player_handlers: [0xe695, 0xe725, 0xeb52],
                ..stage4
            };
            let mut coverage = ReplayCoverage {
                previous: before,
                expected: vec![required(name)],
                ..ReplayCoverage::default()
            };
            coverage.observe_expectations(awarded, 100);
            coverage.observe_expectations(awarded, 120);
            assert_eq!(
                coverage.all_expected_frame(),
                None,
                "fresh award with normal handlers is not animation completion"
            );
            coverage.observe_expectations(stage3, 140);
            coverage.observe_expectations(stage4, 160);
            assert_eq!(coverage.all_expected_frame(), None, "HDMA has not finished");
            coverage.observe_expectations(completed, 200);
            assert_eq!(coverage.all_expected_frame(), Some(200));
            let mut loaded = ReplayCoverage {
                previous: awarded,
                expected: vec![required(name)],
                ..ReplayCoverage::default()
            };
            loaded.observe_expectations(stage3, 140);
            loaded.observe_expectations(stage4, 160);
            loaded.observe_expectations(completed, 200);
            assert_eq!(
                loaded.all_expected_frame(),
                None,
                "preloaded suit must not count even if its animation executes"
            );
            for lost_from in [stage4, completed] {
                let mut lost = ReplayCoverage {
                    previous: before,
                    expected: vec![required(name)],
                    ..ReplayCoverage::default()
                };
                lost.observe_expectations(awarded, 100);
                lost.observe_expectations(stage3, 140);
                lost.observe_expectations(stage4, 160);
                lost.observe_expectations(
                    CoverageState {
                        items: [0, 0],
                        ..lost_from
                    },
                    180,
                );
                assert_eq!(lost.all_expected_frame(), None, "lost suit must not count");
                lost.observe_expectations(completed, 200);
                assert_eq!(
                    lost.all_expected_frame(),
                    None,
                    "a restored bit must not finish an interrupted animation"
                );
            }
            for interruption in [
                CoverageState {
                    room: 0xa000,
                    ..awarded
                },
                CoverageState {
                    state: 12,
                    ..awarded
                },
            ] {
                let mut interrupted = ReplayCoverage {
                    previous: before,
                    expected: vec![required(name)],
                    ..ReplayCoverage::default()
                };
                interrupted.observe_expectations(awarded, 100);
                interrupted.observe_expectations(interruption, 120);
                interrupted.observe_expectations(stage3, 140);
                interrupted.observe_expectations(stage4, 160);
                interrupted.observe_expectations(completed, 200);
                assert_eq!(
                    interrupted.all_expected_frame(),
                    None,
                    "room/state re-entry must not finish an old award"
                );
            }
        }
    }

    fn traversal() -> [CoverageState; 6] {
        let start = CoverageState {
            player_handlers: [0xe695, 0xe725, 0xeb52],
            ..gameplay()
        };
        let admission = CoverageState {
            state: 9,
            door_exit: 0x8100,
            door_phase: 0xe17d,
            ..start
        };
        let loading = CoverageState {
            state: 11,
            door_phase: 0xe2f7,
            door_interlocks: [1, 1],
            ..admission
        };
        let destination = CoverageState {
            room: 0xa7de,
            door_phase: 0xe310,
            ..loading
        };
        let header = CoverageState {
            area: 2,
            door_phase: 0xe38e,
            ..destination
        };
        let completed = CoverageState {
            state: 8,
            door_phase: 0xe737,
            door_interlocks: [0, 0],
            ..header
        };
        [start, admission, loading, destination, header, completed]
    }

    fn episode(name: &str, samples: &[CoverageState]) -> ReplayCoverage {
        let mut coverage = ReplayCoverage {
            previous: samples[0],
            expected: vec![required(name)],
            ..ReplayCoverage::default()
        };
        for (frame, &sample) in samples.iter().enumerate().skip(1) {
            coverage.observe_expectations(sample, frame);
        }
        coverage
    }

    #[cfg_attr(test, test)]
    fn door_requires_fresh_admission_and_the_same_completed_destination() {
        let route = traversal();
        assert_eq!(episode("door:A7DE", &route).all_expected_frame(), Some(5));
        assert_eq!(episode("door:A7DF", &route).all_expected_frame(), None);
        assert_eq!(episode("door:A7DE", &route[1..]).all_expected_frame(), None);
        assert_eq!(episode("door:A7DE", &route[..5]).all_expected_frame(), None);
        for replacement in [
            CoverageState {
                state: 7,
                ..route[4]
            },
            CoverageState {
                room: 0xa7df,
                ..route[4]
            },
            CoverageState {
                door_exit: 0x8200,
                ..route[4]
            },
        ] {
            let mut broken = route;
            broken[4] = replacement;
            assert_eq!(episode("door:A7DE", &broken).all_expected_frame(), None);
        }
        let mut unpublished = route;
        unpublished[3].door_phase = 0xe353;
        assert_eq!(
            episode("door:A7DE", &unpublished).all_expected_frame(),
            None
        );
        let mut interlocked = route;
        interlocked[5].door_interlocks = [1, 0];
        assert_eq!(
            episode("door:A7DE", &interlocked).all_expected_frame(),
            None
        );
        let mut loopback = route;
        for sample in &mut loopback[3..] {
            sample.room = route[0].room;
            sample.area = route[0].area;
        }
        assert_eq!(
            episode("door:9DC7", &loopback).all_expected_frame(),
            Some(5)
        );
    }

    #[cfg_attr(test, test)]
    fn elevator_landing_requires_the_linked_traversal_and_fresh_unlock() {
        let mut route = traversal();
        for sample in &mut route[1..4] {
            sample.elevator_status = 1;
            sample.elevator_flags = 1;
            sample.player_handlers = [0xe695, 0xe8ec, 0xec14];
        }
        for sample in &mut route[4..] {
            sample.elevator_status = 2;
            sample.elevator_flags = 0x80;
            sample.player_handlers = [0xe713, 0xe8ec, 0xec14];
        }
        let landed = CoverageState {
            elevator_status: 0,
            elevator_flags: 0,
            player_handlers: [0xe695, 0xe725, 0xeb52],
            ..route[5]
        };
        for previous_status in [2, 3] {
            let mut coverage = episode("elevator:A7DE", &route);
            assert_eq!(
                coverage.all_expected_frame(),
                None,
                "door arrival is not landing"
            );
            if previous_status == 3 {
                coverage.observe_expectations(
                    CoverageState {
                        elevator_status: 3,
                        ..route[5]
                    },
                    6,
                );
            }
            coverage.observe_expectations(landed, 7);
            assert_eq!(coverage.all_expected_frame(), Some(7));
        }
        assert_eq!(
            episode("elevator:A7DE", &[route[5], landed]).all_expected_frame(),
            None
        );
        let departure = CoverageState {
            state: 8,
            door_phase: 0,
            ..route[1]
        };
        let mut source_ride = vec![route[0], departure, departure, departure];
        source_ride.extend_from_slice(&route[1..]);
        let mut fresh = episode("elevator:A7DE", &source_ride);
        fresh.observe_expectations(landed, 10);
        assert_eq!(fresh.all_expected_frame(), Some(10));
        source_ride[2].state = 12;
        let mut interrupted_source = episode("elevator:A7DE", &source_ride);
        interrupted_source.observe_expectations(landed, 10);
        assert_eq!(
            interrupted_source.all_expected_frame(),
            None,
            "paused departure must restart"
        );
        let mut preloaded_departure = route;
        preloaded_departure[0].elevator_status = 1;
        preloaded_departure[0].elevator_flags = 1;
        let mut preloaded = episode("elevator:A7DE", &preloaded_departure);
        preloaded.observe_expectations(landed, 7);
        assert_eq!(
            preloaded.all_expected_frame(),
            None,
            "preloaded departure is not fresh"
        );
        let mut normal_door = route;
        for sample in &mut normal_door[..4] {
            sample.elevator_status = 0;
            sample.elevator_flags = 0;
        }
        let mut entered = episode("elevator:A7DE", &normal_door);
        entered.observe_expectations(landed, 7);
        assert_eq!(
            entered.all_expected_frame(),
            None,
            "an elevator destination alone is insufficient"
        );
        for interrupted in [
            CoverageState {
                elevator_flags: 0,
                ..route[5]
            },
            CoverageState {
                room: 0xa7df,
                ..route[5]
            },
            CoverageState {
                state: 12,
                ..route[5]
            },
            CoverageState {
                elevator_status: 0,
                ..route[5]
            },
        ] {
            let mut coverage = episode("elevator:A7DE", &route);
            coverage.observe_expectations(interrupted, 6);
            coverage.observe_expectations(landed, 7);
            assert_eq!(coverage.all_expected_frame(), None);
        }
        for bad_landing in [
            CoverageState {
                elevator_flags: 1,
                ..landed
            },
            CoverageState {
                player_handlers: [0xe713, 0xe8ec, 0xec14],
                ..landed
            },
        ] {
            let mut coverage = episode("elevator:A7DE", &route);
            coverage.observe_expectations(bad_landing, 6);
            coverage.observe_expectations(landed, 7);
            assert_eq!(
                coverage.all_expected_frame(),
                None,
                "late unlock is not a landing edge"
            );
        }
    }

    #[cfg_attr(test, test)]
    fn ceres_requires_observed_combat_and_the_same_encounter_escape_edge() {
        let combat = CoverageState {
            area: 6,
            room: 0xe0b5,
            actor_definition: 0x8000,
            actor_bank: 0xa6,
            actor_phase: 0xa6e8,
            ..gameplay()
        };
        let getaway = CoverageState {
            ceres_status: 1,
            actor_phase: 0xc04e,
            ..combat
        };
        let mut escaped = CoverageState {
            ceres_status: 2,
            timer_status: 1,
            actor_phase: 0xaa50,
            ..getaway
        };
        escaped.bosses[6] = 1;
        let route = [combat, combat, getaway, escaped];
        assert_eq!(
            episode("encounter:ceres-ridley", &route).all_expected_frame(),
            Some(3)
        );
        assert_eq!(
            episode("encounter:ceres-ridley", &route[2..]).all_expected_frame(),
            None
        );
        assert_eq!(
            episode("encounter:ceres-ridley", &route[..3]).all_expected_frame(),
            None
        );
        for interruption in [
            CoverageState {
                room: 0xe06b,
                ..getaway
            },
            CoverageState {
                state: 9,
                ..getaway
            },
            CoverageState {
                actor_definition: 0x8002,
                ..getaway
            },
            CoverageState {
                actor_bank: 0xaa,
                ..getaway
            },
            CoverageState {
                actor_phase: 0xa35b,
                ..getaway
            },
        ] {
            let mut broken = route;
            broken[2] = interruption;
            assert_eq!(
                episode("encounter:ceres-ridley", &broken).all_expected_frame(),
                None
            );
        }
        let mut preloaded = route;
        for sample in &mut preloaded {
            sample.bosses[6] = 1;
        }
        assert_eq!(
            episode("encounter:ceres-ridley", &preloaded).all_expected_frame(),
            None
        );
        let mut norfair = route;
        for sample in &mut norfair {
            sample.area = 2;
        }
        assert_eq!(
            episode("encounter:ceres-ridley", &norfair).all_expected_frame(),
            None
        );
    }

    fn frame(health: u16, dead: bool, flag: bool) -> CoverageState {
        CoverageState {
            state: 8,
            area: 0,
            room: 0x9804,
            bosses: [if flag { 4 } else { 0 }, 0, 0, 0, 0, 0, 0, 0],
            bomb_actor: Some(BombActor {
                slot: 7,
                definition: 0xeeff,
                deleted: false,
                health,
                phase: if dead { 0xc6ab } else { 0xc6bf },
                parameter: if dead { 0xc000 } else { 0 },
            }),
            ..CoverageState::default()
        }
    }

    fn hits(frames: &[CoverageState]) -> Vec<usize> {
        let mut coverage = ReplayCoverage {
            previous: frames[0],
            ..ReplayCoverage::default()
        };
        let mut hits = Vec::new();
        for (index, frame) in frames.iter().enumerate().skip(1) {
            if coverage.observe_bomb_torizo(frame) {
                hits.push(index);
            }
            coverage.previous = *frame;
        }
        hits
    }

    #[cfg_attr(test, test)]
    fn bomb_requires_observed_damage_death_and_later_completion() {
        let live = frame(100, false, false);
        let damage = frame(50, false, false);
        let death = frame(0, true, false);
        let done = frame(0, true, true);
        assert_eq!(hits(&[live, damage, death, death, done]), vec![4]);
        // A lethal first observed hit still proves actual in-window damage.
        assert_eq!(hits(&[live, death, done]), vec![2]);
        assert_eq!(
            episode("boss:bomb-torizo", &[live, death, done]).all_expected_frame(),
            Some(2)
        );
        assert_eq!(
            episode("boss:bomb-torizo", &[live, done]).all_expected_frame(),
            None
        );
        assert!(hits(&[live, damage, death]).is_empty());
        assert!(hits(&[death, done]).is_empty());
        assert!(hits(&[live, done]).is_empty());
        assert!(hits(&[done, done]).is_empty());
        assert!(hits(&[live, frame(0, false, false), done]).is_empty());
    }

    #[cfg_attr(test, test)]
    fn bomb_deletion_only_finishes_an_already_observed_death() {
        let live = frame(100, false, false);
        let death = frame(0, true, false);
        let mut deleted = death;
        deleted.bomb_actor.as_mut().unwrap().deleted = true;
        let mut done = deleted;
        done.bosses[0] = 4;
        assert_eq!(hits(&[live, death, done, done]), vec![2]);
        assert_eq!(
            episode("boss:bomb-torizo", &[live, death, done]).all_expected_frame(),
            Some(2)
        );
        assert!(
            hits(&[death, done]).is_empty(),
            "preloaded death cannot complete"
        );
        assert!(
            hits(&[deleted, done]).is_empty(),
            "preloaded deletion cannot complete"
        );
        assert!(
            hits(&[live, deleted, done]).is_empty(),
            "deleted actor cannot arm death"
        );
        assert!(
            hits(&[live, death, deleted, done]).is_empty(),
            "late flag cannot revive deletion"
        );
        assert!(
            hits(&[live, death, deleted, death, done]).is_empty(),
            "reappearance cannot revive death"
        );
    }

    #[cfg_attr(test, test)]
    fn bomb_resets_on_room_actor_or_gameplay_discontinuity() {
        let live = frame(100, false, false);
        let death = frame(0, true, false);
        let done = frame(0, true, true);
        for variant in 0..6 {
            let mut interrupted = death;
            match variant {
                0 => interrupted.room = 0x9bc8,
                1 => interrupted.area = 2,
                2 => interrupted.state = 9,
                3 => interrupted.bomb_actor = None,
                4 => interrupted.bomb_actor.as_mut().unwrap().slot = 8,
                5 => interrupted.bomb_actor.as_mut().unwrap().definition = 0xef3f,
                _ => unreachable!(),
            }
            assert!(
                hits(&[live, death, interrupted, death, done]).is_empty(),
                "variant {variant}"
            );
        }
    }

    #[cfg_attr(test, test)]
    fn bomb_does_not_reuse_an_episode_after_healing_or_completion() {
        let live = frame(100, false, false);
        let death = frame(0, true, false);
        let done = frame(0, true, true);
        assert!(hits(&[live, death, live, done]).is_empty());
        assert_eq!(hits(&[live, death, done, death, done]), vec![2]);
    }

    fn saved_state() -> CoverageState {
        CoverageState {
            state: 8,
            area: 1,
            room: 0xa201,
            selected_save_slot: 0,
            load_station: 3,
            saved_station: 3,
            saved_area: 1,
            player_handlers: [0xe695, 0xe725, 0xeb52],
            coroutines_idle: true,
            ..CoverageState::default()
        }
    }

    fn step(c: &mut ReplayCoverage, s: CoverageState, save: bool, restore: bool) -> bool {
        let complete = c.observe_save_continue(&s, save, restore);
        c.previous = s;
        complete
    }

    fn through_restore() -> (ReplayCoverage, CoverageState) {
        let mut s = saved_state();
        let mut c = ReplayCoverage {
            previous: s,
            ..ReplayCoverage::default()
        };
        assert!(!step(&mut c, s, true, false));
        s.room = 0x9cb3;
        assert!(!step(&mut c, s, false, false));
        for state in 19..=26 {
            s.state = state;
            assert!(!step(&mut c, s, false, false));
        }
        assert!(!step(&mut c, s, false, true));
        assert!(c.save_episode.is_some());
        (c, s)
    }

    fn finish(c: &mut ReplayCoverage, mut s: CoverageState) -> bool {
        s.room = 0xa201;
        for state in [5, 6, 7] {
            s.state = state;
            assert!(!step(c, s, false, false));
        }
        s.state = 8;
        step(c, s, false, false)
    }

    #[cfg_attr(test, test)]
    fn fresh_save_death_selected_restore_and_initialized_arrival_complete_once() {
        let (mut c, s) = through_restore();
        assert!(finish(&mut c, s));
        assert!(!step(&mut c, saved_state(), false, false));
    }

    #[cfg_attr(test, test)]
    fn loaded_history_and_unordered_restore_do_not_qualify() {
        let mut s = saved_state();
        let mut c = ReplayCoverage {
            previous: s,
            ..ReplayCoverage::default()
        };
        s.state = 26;
        assert!(!step(&mut c, s, false, true));
        assert!(!finish(&mut c, s));
        s = saved_state();
        c.previous = s;
        assert!(!step(&mut c, s, true, false));
        s.state = 26;
        assert!(!step(&mut c, s, false, false));
        assert!(!step(&mut c, s, false, true));
        assert!(!finish(&mut c, s));
    }

    #[cfg_attr(test, test)]
    fn reset_and_slot_change_discard_save_history() {
        for state in [0, 1, 4, 5, 6, 7, 27, 31, 39] {
            let mut s = saved_state();
            let mut c = ReplayCoverage {
                previous: s,
                ..ReplayCoverage::default()
            };
            step(&mut c, s, true, false);
            s.state = state;
            assert!(!step(&mut c, s, false, false));
            assert!(c.save_episode.is_none());
        }
        let (mut c, mut s) = through_restore();
        s.selected_save_slot = 1;
        assert!(!step(&mut c, s, false, false));
        assert!(c.save_episode.is_none());
    }

    #[cfg_attr(test, test)]
    fn state_rollbacks_discard_episode() {
        let mut s = saved_state();
        let mut c = ReplayCoverage {
            previous: s,
            ..ReplayCoverage::default()
        };
        step(&mut c, s, true, false);
        for state in [19, 20, 19] {
            s.state = state;
            step(&mut c, s, false, false);
        }
        assert!(c.save_episode.is_none());
        let (mut c, mut s) = through_restore();
        s.state = 5;
        step(&mut c, s, false, false);
        s.state = 26;
        step(&mut c, s, false, false);
        assert!(c.save_episode.is_none());
    }

    #[cfg_attr(test, test)]
    fn wrong_first_arrival_cannot_be_repaired_by_later_room_entry() {
        let (mut c, mut s) = through_restore();
        for state in [5, 6, 7, 8] {
            s.state = state;
            assert!(!step(&mut c, s, false, false));
        }
        assert!(c.save_episode.is_none());
        s.room = 0xa201;
        assert!(!step(&mut c, s, false, false));
    }

    #[cfg_attr(test, test)]
    fn restoration_and_gameplay_initialization_are_required() {
        let (mut c, mut s) = through_restore();
        for state in [5, 6, 7] {
            s.state = state;
            step(&mut c, s, false, false);
        }
        s.state = 8;
        s.room = 0xa201;
        s.coroutines_idle = false;
        assert!(!step(&mut c, s, false, false));
        s.coroutines_idle = true;
        s.player_handlers[0] = 0;
        assert!(!step(&mut c, s, false, false));
        s.player_handlers[0] = 0xe695;
        assert!(step(&mut c, s, false, false));
        let (mut c, mut s) = through_restore();
        s.state = 8;
        s.room = 0xa201;
        assert!(!step(&mut c, s, false, false));
        assert!(c.save_episode.is_none());
    }

    // Movies cannot author portable HDMA definitions or the independently expected registry.
    fn scene_hdma_catalog(gameplay: [u8; 32], first_count: u8) -> Arc<hs::HdmaCatalog> {
        let reference = |id| hs::DefinitionRef {
            id: sc::AssetId(id),
            gameplay,
            definition: [21; 32],
        };
        Arc::new(
            hs::HdmaCatalog::new(
                gameplay,
                vec![
                    hs::HdmaDefinition::Program(hs::HdmaProgramDefinition {
                        reference: reference(100),
                        schema: hs::HDMA_CONTENT_SCHEMA,
                        bytes: vec![0, 0].into(),
                    }),
                    hs::HdmaDefinition::Table(hs::HdmaTableDefinition {
                        reference: reference(101),
                        schema: hs::HDMA_CONTENT_SCHEMA,
                        bytes: vec![first_count, 0, 0x98, 0].into(),
                    }),
                    hs::HdmaDefinition::Indirect(hs::HdmaIndirectDefinition {
                        reference: reference(102),
                        schema: hs::HDMA_CONTENT_SCHEMA,
                        bytes: vec![0x12, 0x34, 0x56, 0x78].into(),
                    }),
                ],
            )
            .unwrap(),
        )
    }

    // Movies cannot author portable display starts. Every synthetic field is supplied
    // explicitly; this fixture establishes component persistence at schema3 only.
    fn scene_display_description(
        media: &sc::SceneMedia,
        gameplay: [u8; 32],
        colors: [u16; 256],
        objects: [u8; 544],
    ) -> ds::DisplayDescription {
        let revision = media.revision();
        let reference = |id| sc::AssetRef {
            id: sc::AssetId(id),
            revision,
        };
        let program = hs::SourceBinding::Prepared {
            reference: hs::DefinitionRef {
                id: sc::AssetId(100),
                gameplay,
                definition: [21; 32],
            },
            windows: vec![hs::BankWindow {
                bank: 0x88,
                address: 0x8000,
                offset: 0,
                length: 2,
            }]
            .into(),
        };
        let table = hs::SourceBinding::Prepared {
            reference: hs::DefinitionRef {
                id: sc::AssetId(101),
                gameplay,
                definition: [21; 32],
            },
            windows: vec![hs::BankWindow {
                bank: 0x88,
                address: 0x9000,
                offset: 0,
                length: 4,
            }]
            .into(),
        };
        let indirect = hs::SourceBinding::LiveOwner {
            owner: hs::LiveOwner::HdmaTable1,
            windows: vec![hs::BankWindow {
                bank: 0x7e,
                address: 0x9800,
                offset: 0,
                length: 4,
            }]
            .into(),
        };
        let variable = hs::LiveOwner::ObjectVariable {
            object: 0,
            variable: hs::HdmaVariable::A,
        };
        let hdma_live = hs::HdmaLiveDescription::new(
            vec![hs::HdmaOwnedBuffer {
                owner: hs::LiveOwner::HdmaTable1,
                bytes: vec![0x81, 0x12, 0x34, 0x56],
            }],
            vec![
                hs::HdmaLiveRegion {
                    owner: hs::LiveOwner::HdmaTable1,
                    source: hs::HdmaRegionSource::Buffer {
                        storage: hs::LiveOwner::HdmaTable1,
                        offset: 0,
                    },
                    length: 4,
                },
                hs::HdmaLiveRegion {
                    owner: hs::LiveOwner::WindowLeft,
                    source: hs::HdmaRegionSource::Buffer {
                        storage: hs::LiveOwner::HdmaTable1,
                        offset: 1,
                    },
                    length: 2,
                },
                hs::HdmaLiveRegion {
                    owner: variable,
                    source: hs::HdmaRegionSource::ObjectVariable {
                        object: 0,
                        variable: hs::HdmaVariable::A,
                    },
                    length: 2,
                },
            ],
        )
        .unwrap();
        let sc::SceneAsset::Atlas(atlas) = media.asset(reference(2)).unwrap() else {
            panic!("fixture atlas")
        };
        let source = ds::SourceSpan {
            asset: atlas.reference,
            bytes: atlas
                .data
                .iter()
                .flat_map(|word| word.to_le_bytes())
                .collect::<Vec<_>>()
                .into(),
            offset: 0,
        };
        let destination = ds::TileDestination {
            word: 0x2000,
            step: 1,
            lanes: ds::ByteLanes::Both,
            remap: 0,
            increment_on_high: true,
        };
        let transfer = ds::TileTransfer {
            source: ds::TransferSource::Asset(source.clone()),
            count: 2,
            destination,
        };
        let scrolling = core::array::from_fn(|slot| {
            let transfers = |wrapped| {
                core::array::from_fn(|half| ds::TileTransfer {
                    source: ds::TransferSource::Scroll {
                        slot,
                        half,
                        wrapped,
                        offset: 0,
                    },
                    count: 2,
                    destination: ds::TileDestination {
                        word: 0x3000 + slot as u16 * 32 + half as u16 * 2,
                        ..destination
                    },
                })
            };
            ds::ScrollTransfer {
                enabled: 1,
                layout: if slot < 2 {
                    ds::ScrollLayout::Column
                } else {
                    ds::ScrollLayout::Row
                },
                first: transfers(false),
                wrapped: transfers(true),
                wrapped_count: 2,
                first_bytes: core::array::from_fn(|_| vec![0x12, 0x34]),
                wrapped_bytes: core::array::from_fn(|_| vec![0x56, 0x78]),
            }
        });
        let visual = ds::supplied_visuals(
            ds::FirefleaDescription {
                timer: 23,
                flash_index: 0,
                compatibility_word: 0,
                darkness: 0,
                compatibility_limit: 0,
                compatibility_shade: 0,
            },
            colors,
            [0x1234; 256],
            ds::PaletteEffects {
                slots: [ds::PaletteSlot {
                    definition: 0,
                    color_offset: 0,
                    pre_action: 0,
                    program: 0,
                    delay: 0,
                    counter: 0,
                }; 8],
                flags: 0,
                selected: 0,
                heat: 5,
                previous_heat: 7,
            },
            ds::AnimatedTiles {
                slots: [ds::AnimationSlot {
                    definition: 0,
                    counter: 0,
                    program: 0,
                    delay: 0,
                    source: 0,
                    size: 0,
                    vram: 0,
                }; 6],
                flags: 0,
                selected: 0,
                last_instruction: 29,
            },
        );
        ds::DisplayDescription {
            media_revision: revision,
            phase: ds::DisplayPhase::Committed,
            objects: ds::ObjectStaging {
                bytes: objects,
                cursor: 512,
                native_order: ds::WordOrder::Little,
            },
            hud: ds::HudState {
                tiles: [0x3801; 96],
                health: 99,
                maximum_health: 199,
                previous_health: 98,
                ammunition: [1, 2, 3],
                maximum_ammunition: [11, 12, 13],
                previous_ammunition: [0, 1, 2],
                selection: 2,
                previous_selection: 1,
                auto_cancel: 0,
                reserve_mode: 1,
                reserve_health: 31,
                palette: 0x1800,
                media: reference(5),
            },
            visual,
            player_palette: ds::PlayerPalette {
                hurt_flash: 27,
                visor_timer: 29,
                suit_index: 31,
                xray_progress: 33,
                timer: 35,
                suit_beam_speed: 37,
                suit_red_green: 39,
                crystal_flash_delay: 41,
            },
            atmospheric: ds::AtmosphericEffects {
                slots: core::array::from_fn(|slot| ds::AtmosphericSlot {
                    remaining: 51 + slot as u16,
                    x: 61 + slot as u16,
                    y: 71 + slot as u16,
                    frame_and_kind: 0,
                }),
            },
            nmi: ds::NmiState {
                pending: 1,
                frame_byte: 11,
                frame_word: 0x1234,
                including_lag: 0xabcd,
                missed: 3,
                max_missed: 5,
                acknowledgment: 1,
            },
            samus: core::array::from_fn(|_| ds::SamusHalf {
                enabled: 1,
                descriptor: reference(8),
                first: transfer.clone(),
                second: ds::TileTransfer {
                    count: 0,
                    ..transfer.clone()
                },
            }),
            backgrounds: core::array::from_fn(|plane| ds::BackgroundState {
                tilemap: reference(if plane == 2 { 5 } else { 4 }),
                atlas: reference(if plane == 2 { 3 } else { 2 }),
                palette: reference(6),
                camera: [17, 0xfffd],
                scroll: [0, 0xffff],
                screen_layout: 0,
                character_layout: 0,
            }),
            scrolling,
            uploads: vec![ds::QueuedUpload {
                cursor: 0,
                transfer: transfer.clone(),
            }],
            upload_tail: 7,
            mode7_uploads: vec![ds::Mode7Upload {
                cursor: 0,
                source: ds::TransferSource::Asset(source.clone()),
                count: 2,
                target: ds::Mode7Target::Tiles(destination),
                transfer_mode: 0,
            }],
            mode7_tail: 7,
            readbacks: vec![ds::Readback {
                cursor: 0,
                destination: reference(2),
                first_word: 0,
                count: 2,
            }],
            readback_tail: 9,
            actor_queues: core::array::from_fn(|slot| ds::ActorDrawQueue {
                size: 4,
                cursor: 2,
                actors: vec![slot as u16 * 2 + 1, slot as u16 * 2 + 2],
            }),
            actor_tiles: ds::ActorTileStaging {
                pending: 1,
                size: 2,
                source: source.clone(),
                destination: 0x2222,
                transfer_source: 0x13,
                transfer_destination: 0x15,
                tilemap_bytes: vec![0xca, 0xfe],
            },
            controls: ds::DisplayControls {
                brightness: 15,
                object_layout: 0,
                mode: 1,
                fake_mode: 1,
                mosaic: 0,
                screens: [0; 4],
                previous_background3_map: 0x35,
                characters: [0; 2],
                affine_selection: 0,
                window_selection: [0; 3],
                window_positions: [0; 4],
                window_logic: [0; 2],
                main_layers: 4,
                previous_main_layers: 4,
                sub_layers: 0,
                main_window: 0,
                sub_window: 0,
                next_color_selection: 0xb2,
                color_selection: 0,
                previous_color_selection: 0,
                next_color_math: 0xa5,
                color_math: 0,
                previous_color_math: 0,
                fixed_color: [0; 3],
                geometry: 0,
                scroll_words: [0; 8],
                affine_words: [0; 6],
                nmi_enable: 0,
                hdma_enable: 0,
                committed_fake_mode: 1,
            },
            hdma: ds::HdmaState {
                bindings: [ds::HdmaBinding {
                    enable: 1,
                    bank_slot: 0,
                    table: 0x9000,
                }; 6],
                objects: core::array::from_fn(|_| ds::HdmaObject {
                    instruction: program.clone(),
                    instruction_cursor: 0x8000,
                    instruction_timer: 0,
                    pre_instruction: 0,
                    pre_instruction_bank: 0x88,
                    timer: 0,
                    variables: [0x3344, 0x5566, 0, 0, 0, 0],
                }),
                channels: core::array::from_fn(|_| ds::HdmaChannel {
                    source: indirect.clone(),
                    table: table.clone(),
                    active: true,
                    terminated: false,
                    do_transfer: true,
                    repeat_count: 0x81,
                    source_address: 0x9000,
                    table_cursor: 0x9003,
                    indirect_cursor: 0x9800,
                    mode: 0,
                    from_destination: false,
                    indirect: true,
                    fixed: true,
                    decrement: false,
                    source_bank: 0x88,
                    indirect_bank: 0x7e,
                    destination: 0,
                    transfer_cursor: 0,
                    unused_bit: false,
                    unused_byte: 0,
                    dma_active: false,
                }),
                selected: 0,
                flags: 0x8000,
                hdma_timer: 17,
                dma_timer: 19,
                dma_busy: false,
            },
            hdma_live,
            interrupt: ds::InterruptState {
                current: 0,
                pending: 0,
                vertical: 0,
                horizontal: 0,
                previous_cre: 0,
                current_cre: 0,
                transition_flag: 0,
                transition_continuation: 0,
            },
            // These authored delivered values stay independent of staged and saved controls.
            delivery: ds::DisplayDeliveryDescription {
                horizontal_position: 37,
                vertical_position: 241,
                beam_frames: 0x1234_5678,
                horizontal_irq_enabled: true,
                vertical_irq_enabled: false,
                nmi_enabled: true,
                nmi_wanted: false,
                irq_wanted: true,
                in_nmi: false,
                in_irq: true,
                in_vblank: true,
                automatic_input_read: true,
                automatic_input_timer: 0x4567,
                counter_latch_gate: true,
                raster_suppressed: false,
            },
            effective: ds::EffectiveDisplayDescription {
                background3_map_words: 0x2c00,
                background3_map_wider: true,
                background3_map_higher: false,
                color_clip: 2,
                color_prevent_math: 1,
                color_add_subscreen: true,
                direct_color: true,
                color_math_enabled: [true, false, true, false, true, false],
                color_subtract: true,
                color_half: false,
                main_layers: 0xa4,
                forced_blank: true,
            },
            latches: ds::DisplayLatches {
                tile_pointer: 0x2345,
                tile_increment: 2,
                tile_remap: 0,
                tile_read_buffer: 0x6789,
                tile_increment_on_high: true,
                tile_padding: [0; 2],
                palette_pointer: 119,
                palette_second: true,
                palette_buffer: 0x99,
                object_written_address: 0x101,
                object_written_high_table: false,
                object_address: 0x102,
                object_high_table: true,
                object_second: true,
                object_buffer: 0x66,
                scroll_previous: 0x77,
                scroll_previous_second: 0x88,
                affine_previous: 0xaa,
                latched_horizontal: 137,
                latched_vertical: 139,
                horizontal_second: true,
                vertical_second: false,
                counter_latched: true,
                open_bus: 0xbb,
                ppu_open_bus: [0xcc, 0xdd],
            },
            raster: ds::RasterState {
                range_overflow: false,
                time_overflow: false,
                has_objects: false,
                even_frame: false,
                interlaced: false,
                frame_interlaced: false,
                overscan: false,
                frame_overscan: false,
                mosaic_origin: 1,
                mosaic_phase: 0,
                visible: true,
            },
            animated_sources: core::array::from_fn(|_| source.clone()),
        }
    }

    // Movies cannot author portable final resident media that differs from visible line history.
    fn scene_component_fixture() -> (
        ss::DisplayStartRequirements,
        ss::DisplayStartDescription,
        Arc<sc::SceneMedia>,
    ) {
        let (mut requirements, mut start, media) = scene_start_prefix();
        let mut images = media.compiled_images().cloned().collect::<Vec<_>>();
        let mut palettes = media.palette_incarnations().cloned().collect::<Vec<_>>();
        let mut image = images[1].clone();
        image.reference.asset.id = sc::AssetId(9);
        image.reference.incarnation = 3;
        image.palette.asset.id = sc::AssetId(10);
        image.palette.incarnation = 3;
        let mut palette = palettes[1].clone();
        let staged_colors = palette.colors;
        palette.reference.asset.id = sc::AssetId(10);
        palette.reference.incarnation = 3;
        palette.colors[0] = 31 << 10;
        palette.colors[25] = 31;
        palette.colors[26] = 0x7fff;
        let words = Arc::make_mut(&mut image.words);
        words[0x8000..0x8005].copy_from_slice(&[0x2345, 1, 2, 0, 0x6789]);
        words[0x8005..0x801f].copy_from_slice(&palette.colors[..26]);
        let mut objects = [0; 544];
        objects[512..].fill(0x55);
        let mut resident_objects = objects;
        resident_objects[0] = 0x42;
        let resident = sm::game::scene::ResidentDisplayDescription {
            image: image.reference,
            palette: palette.reference,
            words: image.words.clone(),
            colors: palette.colors,
            objects: resident_objects,
        };
        images.push(image);
        palettes.push(palette);
        let mut assets = media.assets().cloned().collect::<Vec<_>>();
        let revision = media.revision();
        let room = sc::AssetRef {
            id: sc::AssetId(9),
            revision,
        };
        let final_palette = sc::AssetRef {
            id: sc::AssetId(10),
            revision,
        };
        assets.push(sc::SceneAsset::Room(room));
        assets.push(sc::SceneAsset::Palette(sc::Palette {
            reference: final_palette,
            colors: resident.colors,
        }));
        let media =
            Arc::new(sc::SceneMedia::new(media.revision(), assets, palettes, images).unwrap());
        let committed = start.committed_scene.take().unwrap();
        let mut description = committed.description().clone();
        let mut resident_assets = description.resident.to_vec();
        resident_assets.extend([room, final_palette]);
        description.resident = resident_assets.into();
        start.committed_scene = Some(sc::Scene::new(description, media.clone()).unwrap());
        let hdma_content = scene_hdma_catalog(requirements.gameplay, 0x81);
        requirements.hdma_content = hdma_content.clone();
        start.hdma_content = Some(hdma_content);
        start.display = Some(scene_display_description(
            &media,
            requirements.gameplay,
            staged_colors,
            objects,
        ));
        start.resident = Some(resident);
        (requirements, start, media)
    }

    fn scene_copy_metadata(metadata: &sd::DisplayStartMetadata) -> sd::DisplayStartMetadata {
        sd::DisplayStartMetadata {
            identity: metadata.identity,
            reference: metadata.reference,
            input_history: metadata.input_history,
            continuation: metadata.continuation,
            audio: metadata.audio.clone(),
        }
    }

    fn scene_metadata(start: &ss::DisplayStartDescription) -> sd::DisplayStartMetadata {
        sd::DisplayStartMetadata {
            identity: start.identity.unwrap(),
            reference: start.reference.unwrap(),
            input_history: start.input_history.unwrap(),
            continuation: start.continuation.unwrap(),
            audio: start.audio.as_ref().unwrap().clone(),
        }
    }

    fn scene_same_metadata(actual: &sd::DisplayStartMetadata, expected: &sd::DisplayStartMetadata) {
        assert_eq!(actual.identity, expected.identity);
        assert_eq!(actual.reference, expected.reference);
        assert_eq!(actual.input_history, expected.input_history);
        assert_eq!(actual.continuation, expected.continuation);
        assert_eq!(actual.audio, expected.audio);
    }

    fn scene_same_resident(
        actual: &sm::game::scene::ResidentDisplayDescription,
        expected: &sm::game::scene::ResidentDisplayDescription,
    ) {
        assert_eq!(
            (actual.image, actual.palette),
            (expected.image, expected.palette)
        );
        assert_eq!(actual.words, expected.words);
        assert_eq!(actual.colors, expected.colors);
        assert_eq!(actual.objects, expected.objects);
    }

    // Movies cannot author divergent delivered controls or written OAM selection at portable reopening.
    #[cfg_attr(test, test)]
    fn scene_component_preserves_schema3_delivery_effective_and_written_selection() {
        let (requirements, start, media) = scene_component_fixture();
        let opened = sd::DisplayComponent::open(
            ss::ValidatedDisplayStart::construct(start, media, &requirements).unwrap(),
        )
        .unwrap();
        let checkpoint = opened.display.checkpoint().unwrap();
        let (description, _) = checkpoint.into_start_description(opened.metadata);
        let state = description.display.as_ref().unwrap();
        assert_eq!(
            state.delivery,
            ds::DisplayDeliveryDescription {
                horizontal_position: 37,
                vertical_position: 241,
                beam_frames: 0x1234_5678,
                horizontal_irq_enabled: true,
                vertical_irq_enabled: false,
                nmi_enabled: true,
                nmi_wanted: false,
                irq_wanted: true,
                in_nmi: false,
                in_irq: true,
                in_vblank: true,
                automatic_input_read: true,
                automatic_input_timer: 0x4567,
                counter_latch_gate: true,
                raster_suppressed: false,
            }
        );
        assert_eq!(
            state.effective,
            ds::EffectiveDisplayDescription {
                background3_map_words: 0x2c00,
                background3_map_wider: true,
                background3_map_higher: false,
                color_clip: 2,
                color_prevent_math: 1,
                color_add_subscreen: true,
                direct_color: true,
                color_math_enabled: [true, false, true, false, true, false],
                color_subtract: true,
                color_half: false,
                main_layers: 0xa4,
                forced_blank: true,
            }
        );
        assert_eq!(
            (
                state.controls.screens[2],
                state.controls.previous_background3_map,
                state.controls.color_selection,
                state.controls.previous_color_selection,
                state.controls.next_color_selection,
                state.controls.color_math,
                state.controls.previous_color_math,
                state.controls.next_color_math
            ),
            (0, 0x35, 0, 0, 0xb2, 0, 0, 0xa5)
        );
        assert_eq!(
            (
                state.latches.object_written_address,
                state.latches.object_written_high_table,
                state.latches.object_address,
                state.latches.object_high_table
            ),
            (0x101, false, 0x102, true)
        );
        assert_eq!(
            (state.interrupt.vertical, state.interrupt.horizontal),
            (0, 0)
        );
        assert_eq!((state.hdma.hdma_timer, state.hdma.dma_timer), (17, 19));
    }

    // Movies cannot reach schema3 rejection of old starts or malformed effective color selectors.
    #[cfg_attr(test, test)]
    fn scene_component_rejects_schema2_and_invalid_effective_color_selectors() {
        for case in 0..4 {
            let (mut requirements, mut start, media) = scene_component_fixture();
            match case {
                0 => start.identity.as_mut().unwrap().schema = 2,
                1 => {
                    start.identity.as_mut().unwrap().schema = 2;
                    requirements.schema = 2;
                }
                2 => start.display.as_mut().unwrap().effective.color_clip = 4,
                _ => start.display.as_mut().unwrap().effective.color_prevent_math = 4,
            }
            let error = ss::ValidatedDisplayStart::construct(start, media, &requirements)
                .err()
                .unwrap();
            match case {
                0 | 1 => assert!(matches!(error, ss::StartError::Schema)),
                _ => assert!(matches!(
                    error,
                    ss::StartError::Display(ds::DisplayError::Geometry)
                )),
            }
        }
    }

    // Movies cannot reach authored portable component open/reopen or final resident history.
    #[cfg_attr(test, test)]
    fn scene_component_preserves_complete_open_checkpoint_and_reopen() {
        let (requirements, start, media) = scene_component_fixture();
        let expected_state = format!("{:?}", start.display.as_ref().unwrap());
        let expected_resident = start.resident.as_ref().unwrap().clone();
        let expected_scene = start.committed_scene.as_ref().unwrap().clone();
        let metadata = scene_metadata(&start);
        let sealed = ss::ValidatedDisplayStart::construct(start, media, &requirements).unwrap();
        let opened = sd::DisplayComponent::open(sealed).unwrap();
        scene_same_metadata(&opened.metadata, &metadata);
        let checkpoint = opened.display.checkpoint().unwrap();
        assert_eq!(opened.display.committed_scene(), &expected_scene);
        assert_eq!(checkpoint.committed_scene(), &expected_scene);
        scene_same_resident(checkpoint.resident(), &expected_resident);
        assert_eq!(checkpoint.resident().image.incarnation, 3);
        assert_eq!(checkpoint.resident().palette.incarnation, 3);
        assert_eq!(checkpoint.resident().words[0x8000], 0x2345);
        assert_eq!(checkpoint.resident().colors[26], 0x7fff);
        assert_eq!(
            checkpoint.committed_scene().lines()[223].media.incarnation,
            2
        );
        let frame = opened.display.committed_scene().render().unwrap();
        assert_eq!(
            [frame.pixels[8], frame.pixels[200 * 256 + 8]],
            [[0, 0, 255, 0], [255, 0, 0, 0]]
        );
        let (description, media) = checkpoint.into_start_description(opened.metadata);
        drop(opened.display);
        assert_eq!(
            format!("{:?}", description.display.as_ref().unwrap()),
            expected_state
        );
        assert_eq!(
            description.hdma_content.as_deref(),
            Some(requirements.hdma_content.as_ref())
        );
        let display = description.display.as_ref().unwrap();
        assert_eq!(
            (
                display.nmi.pending,
                display.nmi.acknowledgment,
                display.nmi.including_lag
            ),
            (1, 1, 0xabcd)
        );
        assert_eq!(
            (
                display.objects.cursor,
                display.hud.health,
                display.upload_tail,
                display.mode7_tail,
                display.readback_tail
            ),
            (512, 99, 7, 7, 9)
        );
        let reopened = sd::DisplayComponent::open(
            ss::ValidatedDisplayStart::construct(description, media, &requirements).unwrap(),
        )
        .unwrap();
        scene_same_metadata(&reopened.metadata, &metadata);
        assert_eq!(reopened.display.committed_scene().render().unwrap(), frame);
        let checkpoint = reopened.display.checkpoint().unwrap();
        scene_same_resident(checkpoint.resident(), &expected_resident);
        let (description, _) = checkpoint.into_start_description(reopened.metadata);
        assert_eq!(
            format!("{:?}", description.display.as_ref().unwrap()),
            expected_state
        );
    }

    // Movies cannot reach authored portable buffer aliases, retained checkpoint bytes or successful reset.
    #[cfg_attr(test, test)]
    fn scene_component_reset_preserves_complete_owned_buffers_and_alias_views() {
        let (requirements, start, media) = scene_component_fixture();
        let metadata = scene_metadata(&start);
        let expected_scene = start.committed_scene.as_ref().unwrap().clone();
        let expected_resident = start.resident.as_ref().unwrap().clone();
        let opened = sd::DisplayComponent::open(
            ss::ValidatedDisplayStart::construct(start, media, &requirements).unwrap(),
        )
        .unwrap();
        let checkpoint = opened.display.checkpoint().unwrap();
        let retained = checkpoint.clone();
        let (mut reset, media) = checkpoint.into_start_description(opened.metadata);
        let reset_state = reset.display.as_mut().unwrap();
        assert_eq!(
            reset_state
                .hdma_live
                .read(&reset_state.hdma, hs::LiveOwner::WindowLeft, 0)
                .unwrap(),
            0x12
        );
        let variable = hs::LiveOwner::ObjectVariable {
            object: 0,
            variable: hs::HdmaVariable::A,
        };
        assert_eq!(
            reset_state
                .hdma_live
                .read(&reset_state.hdma, variable, 0)
                .unwrap(),
            0x44
        );
        reset_state
            .hdma_live
            .write(&mut reset_state.hdma, hs::LiveOwner::WindowLeft, 0, 0xee)
            .unwrap();
        reset_state
            .hdma_live
            .write(&mut reset_state.hdma, variable, 1, 0xaa)
            .unwrap();
        assert_eq!(
            reset_state
                .hdma_live
                .read(&reset_state.hdma, hs::LiveOwner::HdmaTable1, 1)
                .unwrap(),
            0xee
        );
        assert_eq!(reset_state.hdma.objects[0].variables[0], 0xaa44);
        let (retained_state, _) = retained.into_start_description(scene_copy_metadata(&metadata));
        let retained_state = retained_state.display.unwrap();
        assert_eq!(
            retained_state
                .hdma_live
                .read(&retained_state.hdma, hs::LiveOwner::HdmaTable1, 1)
                .unwrap(),
            0x12
        );
        assert_eq!(retained_state.hdma.objects[0].variables[0], 0x3344);
        reset.display.as_mut().unwrap().hud.health = 143;
        reset.display.as_mut().unwrap().nmi.acknowledgment = 5;
        let expected_reset = format!("{:?}", reset.display.as_ref().unwrap());
        let mut display = opened.display;
        let reset_metadata = display
            .reset(ss::ValidatedDisplayStart::construct(reset, media, &requirements).unwrap())
            .unwrap();
        scene_same_metadata(&reset_metadata, &metadata);
        let checkpoint = display.checkpoint().unwrap();
        assert_eq!(checkpoint.committed_scene(), &expected_scene);
        scene_same_resident(checkpoint.resident(), &expected_resident);
        let (reset, _) = checkpoint.into_start_description(reset_metadata);
        assert_eq!(
            format!("{:?}", reset.display.as_ref().unwrap()),
            expected_reset
        );
        assert_eq!(
            (
                reset.display.as_ref().unwrap().hud.health,
                reset.display.as_ref().unwrap().nmi.acknowledgment
            ),
            (143, 5)
        );
        let state = reset.display.unwrap();
        assert_eq!(
            state
                .hdma_live
                .read(&state.hdma, hs::LiveOwner::HdmaTable1, 1)
                .unwrap(),
            0xee
        );
        assert_eq!(
            state
                .hdma_live
                .read(&state.hdma, hs::LiveOwner::WindowLeft, 0)
                .unwrap(),
            0xee
        );
        assert_eq!(state.hdma.objects[0].variables[0], 0xaa44);
    }

    // Movies cannot reach authored portable missing resident/history or failed replacement admission.
    #[cfg_attr(test, test)]
    fn scene_component_rejects_missing_resident_history_and_media_replacement() {
        let (requirements, start, media) = scene_component_fixture();
        let opened = sd::DisplayComponent::open(
            ss::ValidatedDisplayStart::construct(start, media, &requirements).unwrap(),
        )
        .unwrap();
        let mut display = opened.display;
        let before = display.checkpoint().unwrap();
        for case in 0..6 {
            let (mut description, media) = before
                .clone()
                .into_start_description(scene_copy_metadata(&opened.metadata));
            match case {
                0 => description.resident = None,
                1 => description.committed_scene = None,
                2 => Arc::make_mut(&mut description.resident.as_mut().unwrap().words)[0] ^= 1,
                3 => description.resident.as_mut().unwrap().palette.incarnation = 99,
                4 => description.identity.as_mut().unwrap().media = sc::Revision([99; 32]),
                _ => {
                    description.identity.as_mut().unwrap().committed_tick = 1;
                    description.continuation.as_mut().unwrap().phase =
                        ss::CommittedPhase::SettledInterval;
                }
            }
            let result = ss::ValidatedDisplayStart::construct(description, media, &requirements)
                .and_then(|start| display.reset(start));
            let error = result.err().unwrap();
            assert!(matches!(
                (case, error),
                (0, ss::StartError::MissingResidentDisplay)
                    | (1, ss::StartError::MissingCommittedHistory)
                    | (2..=3, ss::StartError::Display(ds::DisplayError::Scene))
                    | (4, ss::StartError::Revision)
                    | (5, ss::StartError::CommitIdentity)
            ));
            let after = display.checkpoint().unwrap();
            assert_eq!(after.committed_scene(), before.committed_scene());
            scene_same_resident(after.resident(), before.resident());
            let (before_state, _) = before
                .clone()
                .into_start_description(scene_copy_metadata(&opened.metadata));
            let (after_state, _) =
                after.into_start_description(scene_copy_metadata(&opened.metadata));
            assert_eq!(
                format!("{:?}", before_state.display),
                format!("{:?}", after_state.display)
            );
        }
    }

    // Movies cannot reach authored portable private-source/phase rejection or failed replacement preservation.
    #[cfg_attr(test, test)]
    fn scene_component_failed_source_replacement_preserves_complete_owner() {
        let (requirements, start, media) = scene_component_fixture();
        let opened = sd::DisplayComponent::open(
            ss::ValidatedDisplayStart::construct(start, media, &requirements).unwrap(),
        )
        .unwrap();
        let mut display = opened.display;
        let before = display.checkpoint().unwrap();
        for case in 0..7 {
            let (mut description, media) = before
                .clone()
                .into_start_description(scene_copy_metadata(&opened.metadata));
            let state = description.display.as_mut().unwrap();
            let expected = match case {
                0 => {
                    state.hud.media.id = sc::AssetId(99);
                    ds::DisplayError::Identity
                }
                1 => {
                    if let ds::TransferSource::Asset(source) = &mut state.samus[0].first.source {
                        Arc::make_mut(&mut source.bytes)[0] ^= 1;
                    }
                    ds::DisplayError::Source
                }
                2 => {
                    state.scrolling[0].first[0].count = 3;
                    ds::DisplayError::Source
                }
                3 => {
                    state.actor_tiles.source.offset = 33;
                    ds::DisplayError::Source
                }
                4 => {
                    state.objects.cursor = 513;
                    ds::DisplayError::Alias { offset: 513 }
                }
                5 => {
                    state.phase = ds::DisplayPhase::Producing;
                    ds::DisplayError::Phase
                }
                _ => {
                    state.animated_sources[0].asset.id = sc::AssetId(99);
                    ds::DisplayError::Identity
                }
            };
            let result = ss::ValidatedDisplayStart::construct(description, media, &requirements)
                .and_then(|start| display.reset(start));
            let ss::StartError::Display(error) = result.err().unwrap() else {
                panic!("expected private display-source rejection")
            };
            assert_eq!(error, expected, "private source replacement {case}");
            let after = display.checkpoint().unwrap();
            assert_eq!(after.committed_scene(), before.committed_scene());
            scene_same_resident(after.resident(), before.resident());
            let (before_state, _) = before
                .clone()
                .into_start_description(scene_copy_metadata(&opened.metadata));
            let (after_state, _) =
                after.into_start_description(scene_copy_metadata(&opened.metadata));
            assert_eq!(
                format!("{:?}", before_state.display),
                format!("{:?}", after_state.display)
            );
        }
    }

    fn scene_owner_unchanged(
        display: &sd::DisplayComponent,
        before: &sd::DisplayCheckpoint,
        metadata: &sd::DisplayStartMetadata,
    ) {
        let after = display.checkpoint().unwrap();
        assert_eq!(after.committed_scene(), before.committed_scene());
        scene_same_resident(after.resident(), before.resident());
        let (before_state, before_media) = before
            .clone()
            .into_start_description(scene_copy_metadata(metadata));
        let (after_state, after_media) =
            after.into_start_description(scene_copy_metadata(metadata));
        assert!(Arc::ptr_eq(&before_media, &after_media));
        assert_eq!(before_state.hdma_content, after_state.hdma_content);
        assert_eq!(
            format!("{:?}", before_state.display),
            format!("{:?}", after_state.display)
        );
    }

    // Movies cannot author final resident assets that are absent from otherwise valid visible history.
    #[cfg_attr(test, test)]
    fn scene_component_rejects_missing_final_resident_assets() {
        let (requirements, start, media) = scene_component_fixture();
        let opened = sd::DisplayComponent::open(
            ss::ValidatedDisplayStart::construct(start, media, &requirements).unwrap(),
        )
        .unwrap();
        let mut display = opened.display;
        let before = display.checkpoint().unwrap();
        for missing in [sc::AssetId(9), sc::AssetId(10)] {
            let (mut description, media) = before
                .clone()
                .into_start_description(scene_copy_metadata(&opened.metadata));
            let mut scene = description
                .committed_scene
                .take()
                .unwrap()
                .description()
                .clone();
            scene.resident = scene
                .resident
                .iter()
                .copied()
                .filter(|asset| asset.id != missing)
                .collect::<Vec<_>>()
                .into();
            let scene = sc::Scene::new(scene, media.clone()).unwrap();
            assert_eq!(
                scene.render().unwrap(),
                before.committed_scene().render().unwrap()
            );
            assert_eq!(scene.lines()[223].media.incarnation, 2);
            description.committed_scene = Some(scene);
            let result = ss::ValidatedDisplayStart::construct(description, media, &requirements)
                .and_then(|start| display.reset(start));
            assert!(matches!(
                result.err().unwrap(),
                ss::StartError::MissingResidentMedia
            ));
            scene_owner_unchanged(&display, &before, &opened.metadata);
        }
    }

    // Movies cannot author individually inconsistent latch fields against immutable resident media.
    #[cfg_attr(test, test)]
    fn scene_component_rejects_each_native_resident_latch_alias() {
        let (requirements, start, media) = scene_component_fixture();
        let opened = sd::DisplayComponent::open(
            ss::ValidatedDisplayStart::construct(start, media, &requirements).unwrap(),
        )
        .unwrap();
        let mut display = opened.display;
        let before = display.checkpoint().unwrap();
        assert_eq!(
            &before.resident().words[0x8000..0x8005],
            &[0x2345, 1, 2, 0, 0x6789]
        );
        for alias in 0..7 {
            let (mut description, media) = before
                .clone()
                .into_start_description(scene_copy_metadata(&opened.metadata));
            let latches = &mut description.display.as_mut().unwrap().latches;
            match alias {
                0 => latches.tile_pointer ^= 1,
                1 => latches.tile_increment_on_high = !latches.tile_increment_on_high,
                2 => latches.tile_increment ^= 1,
                3 => latches.tile_remap ^= 1,
                4 => latches.tile_read_buffer ^= 1,
                5 => latches.tile_padding[0] ^= 1,
                _ => latches.tile_padding[1] ^= 1,
            }
            let resident = description.resident.as_ref().unwrap();
            assert_eq!(
                resident.words,
                media
                    .compiled_images()
                    .find(|image| image.reference == resident.image)
                    .unwrap()
                    .words
            );
            assert_eq!(
                resident.colors,
                media
                    .palette_incarnations()
                    .find(|palette| palette.reference == resident.palette)
                    .unwrap()
                    .colors
            );
            let result = ss::ValidatedDisplayStart::construct(description, media, &requirements)
                .and_then(|start| display.reset(start));
            assert!(matches!(
                result.err().unwrap(),
                ss::StartError::Display(ds::DisplayError::Scene)
            ));
            scene_owner_unchanged(&display, &before, &opened.metadata);
        }
    }

    // Movies cannot omit a portable gameplay catalog while supplying complete active HDMA state.
    #[cfg_attr(test, test)]
    fn scene_component_rejects_missing_hdma_catalog() {
        let (requirements, start, media) = scene_component_fixture();
        let opened = sd::DisplayComponent::open(
            ss::ValidatedDisplayStart::construct(start, media, &requirements).unwrap(),
        )
        .unwrap();
        let mut display = opened.display;
        let before = display.checkpoint().unwrap();
        let (mut description, media) = before
            .clone()
            .into_start_description(scene_copy_metadata(&opened.metadata));
        description.hdma_content = None;
        let result = ss::ValidatedDisplayStart::construct(description, media, &requirements)
            .and_then(|start| display.reset(start));
        assert!(matches!(
            result.err().unwrap(),
            ss::StartError::MissingHdmaContent
        ));
        scene_owner_unchanged(&display, &before, &opened.metadata);
    }

    // Movies cannot author a changed catalog payload under unchanged gameplay and definition fingerprints.
    #[cfg_attr(test, test)]
    fn scene_component_rejects_changed_hdma_payload_under_same_fingerprint() {
        let (requirements, start, media) = scene_component_fixture();
        let opened = sd::DisplayComponent::open(
            ss::ValidatedDisplayStart::construct(start, media, &requirements).unwrap(),
        )
        .unwrap();
        let mut display = opened.display;
        let before = display.checkpoint().unwrap();
        let (mut description, media) = before
            .clone()
            .into_start_description(scene_copy_metadata(&opened.metadata));
        let changed = scene_hdma_catalog(requirements.gameplay, 0x80);
        assert_eq!(changed.gameplay(), requirements.hdma_content.gameplay());
        let table = hs::DefinitionRef {
            id: sc::AssetId(101),
            gameplay: requirements.gameplay,
            definition: [21; 32],
        };
        assert_eq!(
            changed
                .definition(table, hs::ContentKind::Table)
                .unwrap()
                .reference(),
            requirements
                .hdma_content
                .definition(table, hs::ContentKind::Table)
                .unwrap()
                .reference()
        );
        description.hdma_content = Some(changed);
        let result = ss::ValidatedDisplayStart::construct(description, media, &requirements)
            .and_then(|start| display.reset(start));
        assert!(matches!(
            result.err().unwrap(),
            ss::StartError::Display(ds::DisplayError::Source)
        ));
        scene_owner_unchanged(&display, &before, &opened.metadata);
    }

    // Movies cannot author invalid portable HDMA kinds, dormant provenance or missing live owners.
    #[cfg_attr(test, test)]
    fn scene_component_rejects_wrong_hdma_kinds_and_required_unbound_sources() {
        let (requirements, start, media) = scene_component_fixture();
        let opened = sd::DisplayComponent::open(
            ss::ValidatedDisplayStart::construct(start, media, &requirements).unwrap(),
        )
        .unwrap();
        let mut display = opened.display;
        let before = display.checkpoint().unwrap();
        for case in 0..7 {
            let (mut description, media) = before
                .clone()
                .into_start_description(scene_copy_metadata(&opened.metadata));
            let state = description.display.as_mut().unwrap();
            let expected = match case {
                0 => {
                    state.hdma.channels[0].table = state.hdma.objects[0].instruction.clone();
                    ds::DisplayError::Source
                }
                1 => {
                    state.hdma.objects[0].instruction = state.hdma.channels[0].table.clone();
                    ds::DisplayError::Source
                }
                2 => {
                    state.hdma.objects[0].instruction = state.hdma.channels[0].source.clone();
                    ds::DisplayError::Source
                }
                3 => {
                    state.hdma.channels[0].terminated = true;
                    state.hdma.channels[0].table = hs::SourceBinding::Unbound;
                    ds::DisplayError::Incomplete
                }
                4 => {
                    state.hdma.objects[0].instruction_timer = 0x8000;
                    state.hdma.objects[0].instruction = hs::SourceBinding::Unbound;
                    ds::DisplayError::Incomplete
                }
                5 => {
                    state.hdma_live = hs::HdmaLiveDescription::new(vec![], vec![]).unwrap();
                    ds::DisplayError::Source
                }
                _ => {
                    state.hdma.channels[0].source = hs::SourceBinding::Unbound;
                    ds::DisplayError::Incomplete
                }
            };
            let result = ss::ValidatedDisplayStart::construct(description, media, &requirements)
                .and_then(|start| display.reset(start));
            let ss::StartError::Display(error) = result.err().unwrap() else {
                panic!("expected HDMA source admission rejection")
            };
            assert_eq!(error, expected, "HDMA source admission {case}");
            scene_owner_unchanged(&display, &before, &opened.metadata);
        }
    }

    // Movies cannot author malformed portable bank windows independently of transfer execution.
    #[cfg_attr(test, test)]
    fn scene_component_rejects_invalid_hdma_bank_windows() {
        let (requirements, start, media) = scene_component_fixture();
        let opened = sd::DisplayComponent::open(
            ss::ValidatedDisplayStart::construct(start, media, &requirements).unwrap(),
        )
        .unwrap();
        let mut display = opened.display;
        let before = display.checkpoint().unwrap();
        for case in 0..7 {
            let (mut description, media) = before
                .clone()
                .into_start_description(scene_copy_metadata(&opened.metadata));
            let state = description.display.as_mut().unwrap();
            let hs::SourceBinding::Prepared { windows, .. } = &mut state.hdma.channels[0].table
            else {
                panic!("fixture table binding")
            };
            match case {
                0 => *windows = vec![].into(),
                1 => Arc::make_mut(windows)[0].length = 5,
                2 => Arc::make_mut(windows)[0].address = 0xfffe,
                3 => Arc::make_mut(windows)[0].offset = 2,
                4 => {
                    *windows = vec![
                        hs::BankWindow {
                            bank: 0x88,
                            address: 0x9000,
                            offset: 0,
                            length: 3,
                        },
                        hs::BankWindow {
                            bank: 0x88,
                            address: 0x9002,
                            offset: 2,
                            length: 2,
                        },
                    ]
                    .into();
                }
                5 => Arc::make_mut(windows)[0].length = 0,
                _ => Arc::make_mut(windows)[0].offset = usize::MAX,
            }
            let result = ss::ValidatedDisplayStart::construct(description, media, &requirements)
                .and_then(|start| display.reset(start));
            assert!(matches!(
                result.err().unwrap(),
                ss::StartError::Display(ds::DisplayError::Source)
            ));
            scene_owner_unchanged(&display, &before, &opened.metadata);
        }
    }

    // Movies cannot author explicit dormant Unbound sources or inspect their persisted portable descriptions.
    #[cfg_attr(test, test)]
    fn scene_component_admits_explicitly_disabled_hdma_sources() {
        let (requirements, mut start, media) = scene_component_fixture();
        let state = start.display.as_mut().unwrap();
        for channel in &mut state.hdma.channels {
            channel.active = false;
            channel.dma_active = false;
            channel.terminated = true;
            channel.table = hs::SourceBinding::Unbound;
            channel.source = hs::SourceBinding::Unbound;
        }
        for binding in &mut state.hdma.bindings {
            binding.enable = 0;
        }
        for object in &mut state.hdma.objects {
            object.instruction = hs::SourceBinding::Unbound;
            object.instruction_timer = 0x8000;
        }
        let expected = format!("{:?}", start.display.as_ref().unwrap());
        let opened = sd::DisplayComponent::open(
            ss::ValidatedDisplayStart::construct(start, media, &requirements).unwrap(),
        )
        .unwrap();
        let (description, _) = opened
            .display
            .checkpoint()
            .unwrap()
            .into_start_description(opened.metadata);
        assert_eq!(
            format!("{:?}", description.display.as_ref().unwrap()),
            expected
        );
        let state = description.display.unwrap();
        assert_eq!(state.hdma.flags, 0x8000);
        assert!(state.hdma.channels.iter().all(|channel| matches!(
            (&channel.table, &channel.source),
            (hs::SourceBinding::Unbound, hs::SourceBinding::Unbound)
        )));
        assert!(
            state
                .hdma
                .objects
                .iter()
                .all(|object| matches!(object.instruction, hs::SourceBinding::Unbound))
        );
    }

    fn scene_hdma_live_window(
        owner: hs::LiveOwner,
        bank: u8,
        address: u16,
        offset: usize,
        length: usize,
    ) -> hs::SourceBinding {
        hs::SourceBinding::LiveOwner {
            owner,
            windows: vec![hs::BankWindow {
                bank,
                address,
                offset,
                length,
            }]
            .into(),
        }
    }

    fn scene_hdma_bound_byte(
        state: &mut ds::DisplayDescription,
        catalog: &hs::HdmaCatalog,
        channel: usize,
        bank: u8,
        address: u16,
    ) -> u8 {
        state.hdma.channels[channel]
            .source
            .clone()
            .read(
                catalog,
                &mut state.hdma_live,
                &mut state.hdma,
                hs::ContentKind::Indirect,
                bank,
                address,
            )
            .unwrap()
    }

    // Movies cannot author portable overlapping physical windows with independent expected backing bytes.
    fn scene_coherent_hdma_alias_fixture() -> (
        ss::DisplayStartRequirements,
        ss::DisplayStartDescription,
        Arc<sc::SceneMedia>,
    ) {
        let (requirements, mut start, media) = scene_component_fixture();
        let state = start.display.as_mut().unwrap();
        state.hdma.channels[1].source =
            scene_hdma_live_window(hs::LiveOwner::WindowLeft, 0x7e, 0x9801, 0, 2);
        state.hdma.channels[1].indirect_cursor = 0x9801;
        state.hdma.channels[2].source =
            scene_hdma_live_window(hs::LiveOwner::HdmaTable1, 0x7e, 0x9802, 2, 2);
        state.hdma.channels[2].indirect_cursor = 0x9802;
        state.hdma.channels[3].source = scene_hdma_live_window(
            hs::LiveOwner::ObjectVariable {
                object: 0,
                variable: hs::HdmaVariable::A,
            },
            0x7e,
            0x9000,
            0,
            2,
        );
        state.hdma.channels[3].indirect_cursor = 0x9000;
        state.hdma.channels[4].source =
            scene_hdma_live_window(hs::LiveOwner::WindowLeft, 0x7f, 0x9800, 0, 2);
        state.hdma.channels[4].indirect_bank = 0x7f;
        (requirements, start, media)
    }

    // Movies cannot author individually valid portable bindings that conflict at the same physical byte.
    #[cfg_attr(test, test)]
    fn scene_component_rejects_conflicting_physical_hdma_aliases() {
        let (requirements, start, media) = scene_component_fixture();
        let opened = sd::DisplayComponent::open(
            ss::ValidatedDisplayStart::construct(start, media, &requirements).unwrap(),
        )
        .unwrap();
        let mut display = opened.display;
        let before = display.checkpoint().unwrap();
        for case in 0..5 {
            let (mut description, media) = before
                .clone()
                .into_start_description(scene_copy_metadata(&opened.metadata));
            let state = description.display.as_mut().unwrap();
            let binding = match case {
                0 => scene_hdma_live_window(hs::LiveOwner::WindowLeft, 0x7e, 0x9800, 0, 2),
                1 => scene_hdma_live_window(
                    hs::LiveOwner::ObjectVariable {
                        object: 0,
                        variable: hs::HdmaVariable::A,
                    },
                    0x7e,
                    0x9800,
                    0,
                    2,
                ),
                2..=3 => {
                    let mut binding = if case == 2 {
                        state.hdma.channels[1].table.clone()
                    } else {
                        state.hdma.objects[0].instruction.clone()
                    };
                    let hs::SourceBinding::Prepared { windows, .. } = &mut binding else {
                        panic!("fixture prepared source")
                    };
                    let window = &mut Arc::make_mut(windows)[0];
                    window.bank = 0x7e;
                    window.address = 0x9800;
                    binding
                }
                _ => scene_hdma_live_window(hs::LiveOwner::HdmaTable1, 0x7e, 0x9800, 1, 3),
            };
            let kind = match case {
                2 => hs::ContentKind::Table,
                3 => hs::ContentKind::Program,
                _ => hs::ContentKind::Indirect,
            };
            binding
                .validate(&requirements.hdma_content, &state.hdma_live, kind, true)
                .unwrap();
            match case {
                2 => state.hdma.channels[1].table = binding,
                3 => state.hdma.objects[0].instruction = binding,
                _ => state.hdma.channels[1].source = binding,
            }
            let result = ss::ValidatedDisplayStart::construct(description, media, &requirements)
                .and_then(|start| display.reset(start));
            assert!(matches!(
                result.err().unwrap(),
                ss::StartError::Display(ds::DisplayError::Source)
            ));
            scene_owner_unchanged(&display, &before, &opened.metadata);
        }
    }

    // Movies cannot author legal portable physical alias windows or inspect complete checkpoint/reopen state.
    #[cfg_attr(test, test)]
    fn scene_component_preserves_coherent_physical_hdma_aliases_on_reopen() {
        let (requirements, start, media) = scene_coherent_hdma_alias_fixture();
        let expected = format!("{:?}", start.display.as_ref().unwrap());
        let metadata = scene_metadata(&start);
        let opened = sd::DisplayComponent::open(
            ss::ValidatedDisplayStart::construct(start, media, &requirements).unwrap(),
        )
        .unwrap();
        scene_same_metadata(&opened.metadata, &metadata);
        let before = opened.display.checkpoint().unwrap();
        let (mut description, media) = before.clone().into_start_description(opened.metadata);
        assert_eq!(
            format!("{:?}", description.display.as_ref().unwrap()),
            expected
        );
        let state = description.display.as_mut().unwrap();
        assert_eq!(
            scene_hdma_bound_byte(state, &requirements.hdma_content, 0, 0x7e, 0x9801),
            0x12
        );
        assert_eq!(
            scene_hdma_bound_byte(state, &requirements.hdma_content, 1, 0x7e, 0x9801),
            0x12
        );
        assert_eq!(
            scene_hdma_bound_byte(state, &requirements.hdma_content, 2, 0x7e, 0x9802),
            0x34
        );
        assert_eq!(
            scene_hdma_bound_byte(state, &requirements.hdma_content, 3, 0x7e, 0x9000),
            0x44
        );
        assert_eq!(
            scene_hdma_bound_byte(state, &requirements.hdma_content, 4, 0x7f, 0x9800),
            0x12
        );
        drop(opened.display);
        let reopened = sd::DisplayComponent::open(
            ss::ValidatedDisplayStart::construct(description, media, &requirements).unwrap(),
        )
        .unwrap();
        scene_same_metadata(&reopened.metadata, &metadata);
        let after = reopened.display.checkpoint().unwrap();
        assert_eq!(after.committed_scene(), before.committed_scene());
        scene_same_resident(after.resident(), before.resident());
        let (description, _) = after.into_start_description(reopened.metadata);
        assert_eq!(
            format!("{:?}", description.display.as_ref().unwrap()),
            expected
        );
    }

    // Movies cannot edit portable physical alias storage before reset or inspect retained checkpoint bytes.
    #[cfg_attr(test, test)]
    fn scene_component_reset_preserves_coherent_physical_hdma_aliases() {
        let (requirements, start, media) = scene_coherent_hdma_alias_fixture();
        let opened = sd::DisplayComponent::open(
            ss::ValidatedDisplayStart::construct(start, media, &requirements).unwrap(),
        )
        .unwrap();
        let before = opened.display.checkpoint().unwrap();
        let (mut replacement, media) = before
            .clone()
            .into_start_description(scene_copy_metadata(&opened.metadata));
        let state = replacement.display.as_mut().unwrap();
        state
            .hdma_live
            .write(&mut state.hdma, hs::LiveOwner::WindowLeft, 0, 0xab)
            .unwrap();
        state
            .hdma_live
            .write(
                &mut state.hdma,
                hs::LiveOwner::ObjectVariable {
                    object: 0,
                    variable: hs::HdmaVariable::A,
                },
                1,
                0xcd,
            )
            .unwrap();
        let expected = format!("{:?}", replacement.display.as_ref().unwrap());
        let mut display = opened.display;
        let metadata = display
            .reset(ss::ValidatedDisplayStart::construct(replacement, media, &requirements).unwrap())
            .unwrap();
        scene_same_metadata(&metadata, &opened.metadata);
        let after = display.checkpoint().unwrap();
        assert_eq!(after.committed_scene(), before.committed_scene());
        scene_same_resident(after.resident(), before.resident());
        let (mut replacement, _) = after.into_start_description(metadata);
        assert_eq!(
            format!("{:?}", replacement.display.as_ref().unwrap()),
            expected
        );
        let state = replacement.display.as_mut().unwrap();
        assert_eq!(
            scene_hdma_bound_byte(state, &requirements.hdma_content, 0, 0x7e, 0x9801),
            0xab
        );
        assert_eq!(
            scene_hdma_bound_byte(state, &requirements.hdma_content, 1, 0x7e, 0x9801),
            0xab
        );
        assert_eq!(
            scene_hdma_bound_byte(state, &requirements.hdma_content, 2, 0x7e, 0x9802),
            0x34
        );
        assert_eq!(
            scene_hdma_bound_byte(state, &requirements.hdma_content, 3, 0x7e, 0x9001),
            0xcd
        );
        assert_eq!(
            scene_hdma_bound_byte(state, &requirements.hdma_content, 4, 0x7f, 0x9800),
            0xab
        );
        assert_eq!(state.hdma.objects[0].variables[0], 0xcd44);
        let (mut retained, _) = before.into_start_description(opened.metadata);
        let state = retained.display.as_mut().unwrap();
        assert_eq!(
            scene_hdma_bound_byte(state, &requirements.hdma_content, 1, 0x7e, 0x9801),
            0x12
        );
        assert_eq!(
            scene_hdma_bound_byte(state, &requirements.hdma_content, 3, 0x7e, 0x9001),
            0x33
        );
        assert_eq!(state.hdma.objects[0].variables[0], 0x3344);
    }
}
