//! Single-threaded replay through the same differential oracle as the UI.
use sm::game_state::Game;
use std::{collections::BTreeSet, ffi::CString, time::Instant};

mod checkpoint;

#[derive(Default)]
struct ReplayCoverage {
    states: std::collections::BTreeMap<u16, usize>,
    gameplay_frames: usize,
    rooms: BTreeSet<u16>,
    enemies: BTreeSet<u16>,
    last_state_room: Option<(u16, u16)>,
    unchanged_frames: usize,
    longest_unchanged: usize,
}

impl ReplayCoverage {
    fn word(game: &Game, address: usize) -> u16 {
        u16::from_le_bytes([game.g_ram[address], game.g_ram[address + 1]])
    }

    fn observe(&mut self, game: &Game, frame: usize, trace: bool) {
        // Addresses and slot stride match ram_map::RAM_VARS / EnemyData.
        let state = Self::word(game, 0x998);
        let room = Self::word(game, 0x79b);
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
            self.gameplay_frames, self.rooms.len(), self.enemies.len(), self.longest_unchanged
        );
        println!("Observed game states (state:frames): {:?}", self.states);
        println!("Ending/credits state frames: {}", self.credits_frames());
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
            self.gameplay_frames, self.rooms.len(), self.enemies.len()
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
        return Err("usage: sm verify <rom> <inputs> [--frames N] [--load-sram] [--checkpoint-in PATH] [--checkpoint-out PATH] [--require-hit NAME]... [--after-hits N]".into());
    }
    let mut frame_limit = None;
    let mut after_hits = None;
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
            eprintln!("Checkpoint segment: input frames {first}..{end}; prefix was not replayed by this build");
        }
        // Initialization and checkpoint loading are setup, not segment hits.
        sm::verify_probe::configure(&required_probes)?;
        let mut discarded_audio = [0; 534 * 2];
        let mut coverage = ReplayCoverage::default();
        // Optional console diagnostics share the normal CLI and DLL replay path.
        let trace = std::env::var_os("SM_VERIFY_TRACE").is_some();
        let mut completed = 0;
        let mut hit_tail_complete = after_hits.is_none();
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
            sm::rtl::RtlRenderAudio(discarded_audio.as_mut_ptr(), 534, 2, game);
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
            if let Some(tail) = after_hits {
                if sm::verify_probe::all_hit_frame().is_some_and(|frame| index - frame >= tail) {
                    hit_tail_complete = true;
                    eprintln!(
                        "Required hits and {tail} follow-up frames complete: input frames {first}..{}",
                        index + 1
                    );
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
        if failures != 0 {
            if checkpoint_out.is_some() {
                eprintln!("Checkpoint not saved: replay failed");
            }
            return Ok(failures);
        }
        probes?;
        if !hit_tail_complete {
            return Err(
                "required probe follow-up window did not complete within the replay frame limit"
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
