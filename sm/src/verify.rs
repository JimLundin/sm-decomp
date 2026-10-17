//! Single-threaded replay through the same differential oracle as the UI.
use std::{ffi::CString, time::Instant};

pub fn run() -> i32 {
    let args: Vec<String> = std::env::args().skip(2).collect();
    run_args(&args)
}

pub fn run_args(args: &[String]) -> i32 {
    match run_checked(args) {
        Ok(failures) => i32::from(failures != 0),
        Err(error) => {
            eprintln!("verify: {error}");
            2
        }
    }
}

fn run_checked(args: &[String]) -> Result<u64, String> {
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
        sm::cpu_infra::VERIFY_ACTIVE = true;
        sm::cpu_infra::VERIFY_FAILURES = 0;
        sm::cpu_infra::g_runmode = sm::cpu_infra::RM_BOTH as u8;
        let snes = sm::cpu_infra::SnesInit(filename.as_ptr());
        if snes.is_null() {
            return Err("unable to load ROM".into());
        }
        super::initialize_verify_audio()?;
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
        for (index, &pad) in inputs[..frames].iter().enumerate() {
            sm::rtl::RtlRunFrame(i32::from(pad));
            // Advance the music driver once per frame without playing audio.
            sm::rtl::RtlRenderAudio(discarded_audio.as_mut_ptr(), 534, 2);
            let completed = index + 1;
            if completed % 10_000 == 0 {
                let failures = sm::cpu_infra::VERIFY_FAILURES;
                eprintln!(
                    "Progress {completed}/{frames} frames: {failures} failing frames ({:.2}s)",
                    start.elapsed().as_secs_f64()
                );
            }
        }
        super::finish_verify_audio();
        let failures = sm::cpu_infra::VERIFY_FAILURES;
        println!(
            "Verified {frames} frames: {failures} failing frames ({:.2}s)",
            start.elapsed().as_secs_f64()
        );
        Ok(failures)
    }
}
