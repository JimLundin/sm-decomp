//! Three-second desktop presentation smoke, with no gameplay dependency.
use std::time::{Duration, Instant};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let mut window = sm_presentation::Window::new(256, 224, 3, None)?;
    let mut pixels = vec![0; 256 * 224];
    let audio = vec![0i16; 534 * 2];
    let start = Instant::now();
    let mut next = start;
    let mut frames = 0;
    while start.elapsed() < Duration::from_secs(3) {
        let Some(buttons) = window.poll()? else { break };
        for (index, pixel) in pixels.iter_mut().enumerate() {
            let x = index % 256;
            let y = index / 256;
            *pixel = ((x as u32) << 16) | ((y as u32) << 8) | (u32::from(buttons != 0) * 255);
        }
        window.present(&pixels, &audio)?;
        frames += 1;
        next += Duration::from_nanos(1_000_000_000 / 60);
        std::thread::sleep(next.saturating_duration_since(Instant::now()));
    }
    println!(
        "presentation smoke: {frames} frames; audio_resets={}; queued_bytes={}",
        window.audio_resets(),
        window.queued_audio_bytes()
    );
    Ok(())
}
