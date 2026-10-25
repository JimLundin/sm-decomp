//! Single-threaded replay through the same differential oracle as the UI.
use sm::game_state::Game;
use std::{collections::BTreeSet, ffi::CString, time::Instant};

mod checkpoint;
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
                || game.g_ram[base + core::mem::offset_of!(EnemyData, bank)] != 0xaa
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
            bosses: core::array::from_fn(|area| game.g_ram[0xd828 + area]),
            items: [word(0x9a4), word(0x9a8)],
            capacity: [0x9c4, 0x9c8, 0x9cc, 0x9d0, 0x9d4].map(word),
            credits_enabled: word(0x19ff),
            credits_cursor: word(0x19f7),
            credits_scroll: word(0x1997),
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
            actor_bank: game.g_ram[0xf78 + core::mem::offset_of!(sm::types::EnemyData, bank)],
            actor_phase: word(0xf78 + core::mem::offset_of!(sm::types::EnemyData, ai_var_A)),
            selected_save_slot: word(0x952),
            load_station: word(0x78b),
            loading_state: word(0xd914),
            saved_station: word(0xd916),
            saved_area: word(0xd918),
            // ram_map lists five coroutine bytes; 0617 is the APU upload flag.
            coroutines_idle: game.g_ram[0x611..0x616].iter().all(|&byte| byte == 0),
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
        u16::from_le_bytes([game.g_ram[address], game.g_ram[address + 1]])
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
                || (u32::from(state) == sm::game::sm_82::kGameState_8_MainGameplay
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
        if u32::from(state) == sm::game::sm_82::kGameState_8_MainGameplay {
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
            .get(&(sm::game::sm_82::kGameState_39_EndingAndCredits as u16))
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

pub fn run(game: &mut Game) -> i32 {
    let args: Vec<String> = std::env::args().skip(2).collect();
    run_args(&args, game)
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

fn run_checked(args: &[String], game: &mut Game) -> Result<u64, String> {
    if args.len() < 2 {
        return Err("usage: sm verify <rom> <inputs> [--frames N] [--load-sram] [--seed PATH] [--checkpoint-in PATH] [--checkpoint-out PATH] [--require-hit NAME]... [--after-hits N] [--expect TYPE:VALUE|death|credits|save-death-continue]... [--after-expectations N]".into());
    }
    let mut frame_limit = None;
    let mut after_hits = None;
    let mut after_expectations = None;
    let mut expected: Vec<RequiredExpectation> = Vec::new();
    let mut load_sram = false;
    let mut checkpoint_in = None;
    let mut seed_in = None;
    let mut checkpoint_out = None;
    let mut required_probes = Vec::new();
    let mut options = args[2..].iter();
    while let Some(option) = options.next() {
        match option.as_str() {
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
    if seed_in.is_some() && (checkpoint_in.is_some() || checkpoint_out.is_some() || load_sram) {
        return Err("--seed cannot be combined with checkpoints or SRAM loading".into());
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
        if pad & 0xf != 0 {
            return Err(format!("non-gamepad bits at line {}", line + 1));
        }
        // Movie files use the SNES hardware layout; RtlRunFrame uses B at
        // bit 0, Y at bit 1, ... R at bit 11 (the input latch reverses it).
        inputs.push(pad.reverse_bits());
    }
    if frame_limit == Some(0) || frame_limit.is_some_and(|n| n > inputs.len()) {
        return Err(format!(
            "invalid frame limit, inputs contain {}",
            inputs.len()
        ));
    }
    let rom = if checkpoint_in.is_some() || checkpoint_out.is_some() {
        std::fs::read(&args[0]).map_err(|e| format!("{}: {e}", args[0]))?
    } else {
        Vec::new()
    };
    let filename = CString::new(args[0].as_bytes()).map_err(|_| "invalid ROM path")?;
    let start = Instant::now();
    unsafe {
        game.VERIFY_ACTIVE = true;
        game.VERIFY_FAILURES = 0;
        game.g_runmode = sm::cpu_infra::RM_BOTH as u8;
        // Backend-specific ROM patches are installed during initialization.
        if std::env::var_os("SM_VERIFY_PHYSICAL_AUDIO").is_some() {
            game.g_use_my_apu_code = false;
        }
        let snes = sm::cpu_infra::SnesInit(filename.as_ptr(), game);
        if snes.is_null() {
            return Err("unable to load ROM".into());
        }
        if let Some(saved_sram) = saved_sram {
            sm::rtl::RtlReadSram(game);
            if game.g_sram.bytes(0..8192) != saved_sram {
                return Err("loaded SRAM differs from saves/sm.srm".into());
            }
        }
        super::initialize_verify_audio(game)?;
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
        let first = match checkpoint_in {
            Some(path) => checkpoint::load(std::path::Path::new(path), game, &rom, &inputs)?,
            None => 0,
        };
        if let Some(path) = seed_in {
            seed::load(std::path::Path::new(path), game)?;
        }
        let frames = frame_limit.unwrap_or(inputs.len() - first);
        let end = first
            .checked_add(frames)
            .filter(|&end| end <= inputs.len())
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
            checkpoint::save(std::path::Path::new(path), game, &rom, &inputs, end)?;
            eprintln!("Saved verifier checkpoint before input frame {end}: {path}");
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

    #[cfg(sm_verify_dll)]
    pub(super) fn run() -> i32 {
        // Keep names and function pointers paired without duplicating assertions.
        macro_rules! cases {
            ($($test:ident),+ $(,)?) => { [$( (stringify!($test), $test as fn()), )+] };
        }
        let tests: [(&str, fn()); 20] = cases![
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
}
