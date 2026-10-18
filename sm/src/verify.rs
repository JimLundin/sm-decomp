//! Single-threaded replay through the same differential oracle as the UI.
use sm::game_state::Game;
use std::{collections::BTreeSet, ffi::CString, time::Instant};

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
        println!("Gameplay rooms: {:04X?}", self.rooms);
        println!("Gameplay enemy definitions: {:04X?}", self.enemies);
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
    if args.len() != 2 && (args.len() != 4 || args[2] != "--frames") {
        return Err("usage: sm verify <rom> <inputs> [--frames N]".into());
    }
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
    let frames = if args.len() == 4 {
        args[3]
            .parse::<usize>()
            .map_err(|_| "invalid frame limit")?
    } else {
        inputs.len()
    };
    if frames == 0 || frames > inputs.len() {
        return Err(format!(
            "requested {frames} frames, inputs contain {}",
            inputs.len()
        ));
    }
    let filename = CString::new(args[0].as_bytes()).map_err(|_| "invalid ROM path")?;
    let start = Instant::now();
    unsafe {
        game.VERIFY_ACTIVE = true;
        game.VERIFY_FAILURES = 0;
        game.g_runmode = sm::cpu_infra::RM_BOTH as u8;
        let snes = sm::cpu_infra::SnesInit(filename.as_ptr(), game);
        if snes.is_null() {
            return Err("unable to load ROM".into());
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
        let mut discarded_audio = [0i16; 534 * 2];
        let mut coverage = ReplayCoverage::default();
        // Optional console diagnostics share the normal CLI and DLL replay path.
        let trace = std::env::var_os("SM_VERIFY_TRACE").is_some();
        for (index, &pad) in inputs[..frames].iter().enumerate() {
            // Undrawn rows must not retain a prior mismatch or stale pixels.
            super::g_pixels.fill(0);
            super::g_my_pixels.fill(0);
            sm::rtl::RtlRunFrame(i32::from(pad), game);
            // Advance the music driver once per frame without playing audio.
            sm::rtl::RtlRenderAudio(discarded_audio.as_mut_ptr(), 534, 2, game);
            coverage.observe(game, index, trace);
            let completed = index + 1;
            if completed % 10_000 == 0 {
                let failures = game.VERIFY_FAILURES;
                eprintln!(
                    "Progress {completed}/{frames} frames: {failures} failing frames ({:.2}s)",
                    start.elapsed().as_secs_f64()
                );
                coverage.report_progress();
            }
        }
        super::finish_verify_audio();
        let failures = game.VERIFY_FAILURES;
        coverage.report();
        println!(
            "Verified {frames} frames: {failures} failing frames ({:.2}s)",
            start.elapsed().as_secs_f64()
        );
        Ok(failures)
    }
}
