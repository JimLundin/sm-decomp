//! Single-threaded replay through the same differential oracle as the UI.
use sm::game_state::Game;
use std::{collections::BTreeSet, ffi::CString, time::Instant};

mod checkpoint;

#[derive(Clone, Debug, PartialEq, Eq)]
enum Expectation {
    Area(u16),
    Room(u16),
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
        let (kind, name) = value.split_once(':').ok_or_else(bad)?;
        match kind {
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
            "room" if name.len() == 4 && name.bytes().all(|byte| byte.is_ascii_hexdigit()) => {
                let room = u16::from_str_radix(name, 16).map_err(|_| bad())?;
                if room < 0x8000 {
                    return Err(bad());
                }
                Ok(Self::Room(room))
            }
            "boss" => {
                // Defeat flags, not proof of encountering or fighting the boss.
                // Corpus recipes must also require precise encounter/completion probes.
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
            Self::Area(area) => after.state == 8 && after.area == area,
            Self::Room(room) => after.state == 8 && after.room == room,
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
        }
    }
}

#[derive(Default)]
struct ReplayCoverage {
    states: std::collections::BTreeMap<u16, usize>,
    gameplay_frames: usize,
    rooms: BTreeSet<u16>,
    areas: BTreeSet<u16>,
    previous: CoverageState,
    death_in_progress: bool,
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

    fn observe_expectations(&mut self, current: CoverageState, frame: usize) {
        let before = self.previous;
        let dying = (19..=25).contains(&current.state);
        if dying && !(19..=25).contains(&before.state) {
            self.death_in_progress = true;
        }
        let death = self.death_in_progress && current.state == 26;
        for expected in &mut self.expected {
            if expected.first_frame.is_some() {
                continue;
            }
            let observed = expected.kind.observed(&before, &current, death);
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
        return Err("usage: sm verify <rom> <inputs> [--frames N] [--load-sram] [--checkpoint-in PATH] [--checkpoint-out PATH] [--require-hit NAME]... [--after-hits N] [--expect TYPE:VALUE|death|credits]... [--after-expectations N]".into());
    }
    let mut frame_limit = None;
    let mut after_hits = None;
    let mut after_expectations = None;
    let mut expected: Vec<RequiredExpectation> = Vec::new();
    let mut load_sram = false;
    let mut checkpoint_in = None;
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
            "--checkpoint-out" if checkpoint_out.is_none() => {
                checkpoint_out = Some(options.next().ok_or("missing checkpoint output path")?);
            }
            "--require-hit" => {
                required_probes.push(options.next().ok_or("missing probe name")?.clone());
            }
            _ => return Err(format!("unknown or repeated option: {option}")),
        }
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

#[cfg(test)]
mod coverage_checks {
    use super::*;

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

    #[test]
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
            "credits:1",
        ] {
            assert!(Expectation::parse(value).is_err(), "accepted {value}");
        }
        assert_eq!(
            Expectation::parse("room:91f8"),
            Expectation::parse("room:91F8")
        );
        assert_eq!(
            Expectation::parse("boss:mother-brain").unwrap(),
            Expectation::Boss { area: 5, mask: 2 }
        );
    }

    #[test]
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

    #[test]
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

    #[test]
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

    #[test]
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

    #[test]
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
    #[test]
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
}
