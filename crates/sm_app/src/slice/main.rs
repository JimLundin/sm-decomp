//! The Gate 1 slice: one window, one sound device, one owned Session.
//!
//! `sm_slice <content> <start> [--inputs <file> --ticks <n> --digests-out <file>]`
//!
//! Live input by default; `--inputs` replays through the same loop, window and
//! sound path. The clock is the reference cadence, never the display.
#![deny(unsafe_op_in_unsafe_fn)]
use sdl2_sys::{
    AUDIO_S16LSB, SDL_AudioDeviceID, SDL_AudioSpec, SDL_CreateRenderer, SDL_CreateTexture,
    SDL_CreateWindow, SDL_Delay, SDL_Event, SDL_EventType, SDL_GetError, SDL_GetKeyboardState,
    SDL_GetPerformanceCounter, SDL_GetPerformanceFrequency, SDL_GetQueuedAudioSize, SDL_INIT_AUDIO,
    SDL_INIT_EVENTS, SDL_INIT_VIDEO, SDL_Init, SDL_OpenAudioDevice, SDL_PauseAudioDevice,
    SDL_PixelFormatEnum, SDL_PollEvent, SDL_QueueAudio, SDL_Quit, SDL_RenderCopy,
    SDL_RenderPresent, SDL_Scancode, SDL_TextureAccess, SDL_UpdateTexture,
    SDL_WINDOWPOS_CENTERED_MASK, SDL_WindowFlags,
};
use sm_data::prepared::{PreparedContent, PreparedStart};
use sm_host::session::{AudioBatch, HostResult, SaveRequest, Session, TickInput};
use sm_runtime::outbox::{EffectId, Outbox, Sink, TickBatch};
use sm_runtime::save_storage::{SaveFile, SaveStorage};
use std::ffi::CStr;
use std::sync::mpsc;

/// NTSC cadence as an exact ratio: master clock over cycles per frame.
const TICKS_P: u64 = 21_477_272;
const TICKS_Q: u64 = 357_366;
/// Catch-up per event-loop turn; the remaining debt waits.
const CATCH_UP: u32 = 4;
const SAMPLE_RATE: i32 = 32040;
/// The audio queue's bound: a few Ticks of samples, in bytes.
const AUDIO_QUEUE_BYTES: u32 = 534 * 4 * 8;

struct Device(SDL_AudioDeviceID);

impl Sink<AudioBatch> for Device {
    fn offer(&mut self, _id: EffectId, batch: AudioBatch) -> Result<(), AudioBatch> {
        // SAFETY: an open device; the samples outlive the call, which copies them.
        unsafe {
            if self.0 != 0 && SDL_GetQueuedAudioSize(self.0) > AUDIO_QUEUE_BYTES {
                return Err(batch);
            }
            if self.0 != 0 {
                SDL_QueueAudio(
                    self.0,
                    batch.samples.as_ptr().cast(),
                    (batch.samples.len() * 4) as u32,
                );
            }
        }
        Ok(())
    }
}

/// The storage worker: acceptance is enqueueing; the receipt comes later.
struct SaveWorker {
    requests: mpsc::SyncSender<(u64, Vec<u8>)>,
    receipts: mpsc::Receiver<HostResult>,
}

impl SaveWorker {
    fn start(path: String) -> Self {
        let (requests, waiting) = mpsc::sync_channel::<(u64, Vec<u8>)>(1);
        let (done, receipts) = mpsc::sync_channel(2);
        std::thread::spawn(move || {
            let mut file = SaveFile::new(path);
            for (tick, sram) in waiting {
                let stored = file.store(&sram);
                if done.send(HostResult::SaveStored { tick, stored }).is_err() {
                    break;
                }
            }
        });
        Self { requests, receipts }
    }
}

impl Sink<SaveRequest> for SaveWorker {
    fn offer(&mut self, id: EffectId, request: SaveRequest) -> Result<(), SaveRequest> {
        match self.requests.try_send((id.tick, request.sram)) {
            Ok(()) => Ok(()),
            Err(mpsc::TrySendError::Full((_, sram)))
            | Err(mpsc::TrySendError::Disconnected((_, sram))) => Err(SaveRequest { sram }),
        }
    }
}

fn sdl_error() -> String {
    // SAFETY: SDL returns a NUL-terminated string it keeps until the next call.
    unsafe { CStr::from_ptr(SDL_GetError()) }
        .to_string_lossy()
        .into_owned()
}

/// The controller word the keyboard holds, in the movie file's bit order.
fn held_buttons() -> u16 {
    use SDL_Scancode::*;
    let mut count = 0;
    // SAFETY: SDL owns the state array, valid for `count` entries until SDL_Quit.
    let keys = unsafe {
        let state = SDL_GetKeyboardState(&mut count);
        std::slice::from_raw_parts(state, count as usize)
    };
    [
        (SDL_SCANCODE_Z, 0x8000),
        (SDL_SCANCODE_A, 0x4000),
        (SDL_SCANCODE_RSHIFT, 0x2000),
        (SDL_SCANCODE_RETURN, 0x1000),
        (SDL_SCANCODE_UP, 0x0800),
        (SDL_SCANCODE_DOWN, 0x0400),
        (SDL_SCANCODE_LEFT, 0x0200),
        (SDL_SCANCODE_RIGHT, 0x0100),
        (SDL_SCANCODE_X, 0x0080),
        (SDL_SCANCODE_S, 0x0040),
        (SDL_SCANCODE_D, 0x0020),
        (SDL_SCANCODE_C, 0x0010),
    ]
    .into_iter()
    .filter(|&(key, _)| keys[key as usize] != 0)
    .fold(0, |word, (_, bit)| word | bit)
}

#[derive(Default)]
struct Options {
    content: String,
    start: String,
    inputs: Option<Vec<u16>>,
    ticks: Option<u64>,
    /// Where each Tick's digest line goes.
    digests_out: Option<String>,
    save: String,
}

fn options() -> Result<Options, String> {
    let mut args = std::env::args().skip(1);
    let mut options = Options {
        content: args
            .next()
            .ok_or("usage: sm_slice <content> <start> [options]")?,
        start: args
            .next()
            .ok_or("usage: sm_slice <content> <start> [options]")?,
        save: "saves/slice.srm".into(),
        ..Options::default()
    };
    while let Some(arg) = args.next() {
        let mut value = || args.next().ok_or(format!("{arg} needs a value"));
        match arg.as_str() {
            "--inputs" => {
                let path = value()?;
                let text = std::fs::read_to_string(&path).map_err(|e| format!("{path}: {e}"))?;
                options.inputs = Some(
                    text.lines()
                        .map(str::trim)
                        .filter(|line| !line.is_empty() && !line.starts_with('#'))
                        .map(|line| {
                            u16::from_str_radix(line, 16).map_err(|_| format!("bad input {line}"))
                        })
                        .collect::<Result<_, _>>()?,
                );
            }
            "--ticks" => options.ticks = Some(value()?.parse().map_err(|_| "bad tick count")?),
            "--digests-out" => options.digests_out = Some(value()?),
            "--save" => options.save = value()?,
            other => return Err(format!("unknown option {other}")),
        }
    }
    // A replay of `--ticks` Ticks keeps the word after its last, as `sm_replay`
    // and the verifier do: the closing NMI of the last Tick latches it.
    if let (Some(pads), Some(ticks)) = (&mut options.inputs, options.ticks) {
        pads.truncate(
            usize::try_from(ticks)
                .unwrap_or(usize::MAX)
                .saturating_add(1),
        );
    }
    Ok(options)
}

fn run() -> Result<(), String> {
    let options = options()?;
    let read = |path: &str| std::fs::read(path).map_err(|e| format!("{path}: {e}"));
    let content = PreparedContent::decode(&read(&options.content)?).map_err(|e| e.to_string())?;
    let start = PreparedStart::decode(&read(&options.start)?).map_err(|e| e.to_string())?;
    let opened = Session::open(&content, &start).map_err(|e| e.to_string())?;
    let mut session = opened.session;

    // SAFETY: SDL's single-threaded setup; every handle stays on this thread.
    let (renderer, texture, device, frequency) = unsafe {
        if SDL_Init(SDL_INIT_VIDEO | SDL_INIT_AUDIO | SDL_INIT_EVENTS) != 0 {
            return Err(sdl_error());
        }
        let window = SDL_CreateWindow(
            c"Super Metroid: Gate 1 slice".as_ptr(),
            SDL_WINDOWPOS_CENTERED_MASK as i32,
            SDL_WINDOWPOS_CENTERED_MASK as i32,
            512,
            480,
            SDL_WindowFlags::SDL_WINDOW_RESIZABLE as u32,
        );
        if window.is_null() {
            return Err(sdl_error());
        }
        let renderer = SDL_CreateRenderer(window, -1, 0);
        if renderer.is_null() {
            return Err(sdl_error());
        }
        let texture = SDL_CreateTexture(
            renderer,
            SDL_PixelFormatEnum::SDL_PIXELFORMAT_ARGB8888 as u32,
            SDL_TextureAccess::SDL_TEXTUREACCESS_STREAMING as i32,
            256,
            240,
        );
        if texture.is_null() {
            return Err(sdl_error());
        }
        let wanted = SDL_AudioSpec {
            freq: SAMPLE_RATE,
            format: AUDIO_S16LSB as u16,
            channels: 2,
            samples: 1024,
            ..std::mem::zeroed()
        };
        let mut obtained = std::mem::zeroed();
        let device = SDL_OpenAudioDevice(core::ptr::null(), 0, &wanted, &mut obtained, 0);
        if device != 0 {
            SDL_PauseAudioDevice(device, 0);
        } else {
            // A diagnosed no-device sink: the effects keep their whole stream.
            eprintln!("No sound device: {}", sdl_error());
        }
        (renderer, texture, device, SDL_GetPerformanceFrequency())
    };
    let present = |pixels: &[u8]| {
        // SAFETY: a 256x240 streaming texture and a BGRA buffer of that size.
        unsafe {
            SDL_UpdateTexture(texture, core::ptr::null(), pixels.as_ptr().cast(), 1024);
            SDL_RenderCopy(renderer, texture, core::ptr::null(), core::ptr::null());
            SDL_RenderPresent(renderer);
        }
    };
    present(&opened.initial.scene.pixels);

    let mut audio = Device(device);
    let mut saves = SaveWorker::start(options.save.clone());
    let mut outbox: Outbox<AudioBatch, SaveRequest> = Outbox::new(1);
    let replaying = options.inputs.is_some();
    // SAFETY: SDL is initialized.
    let began = unsafe { SDL_GetPerformanceCounter() };
    let (mut index, mut admitted) = (0u64, 0u64);
    let mut receipts = Vec::new();
    let mut digest_lines = Vec::new();
    'game: loop {
        // SAFETY: a zeroed event is a valid buffer for SDL to fill.
        unsafe {
            let mut event: SDL_Event = std::mem::zeroed();
            while SDL_PollEvent(&mut event) != 0 {
                if event.type_ == SDL_EventType::SDL_QUIT as u32 {
                    break 'game;
                }
            }
        }
        receipts.extend(saves.receipts.try_iter());
        outbox.offer(&mut audio, &mut saves);
        // A replay advances every requested Tick without the wall clock.
        let due = if replaying {
            admitted + u64::from(CATCH_UP)
        } else {
            // SAFETY: SDL is initialized.
            let elapsed = unsafe { SDL_GetPerformanceCounter() } - began;
            (u128::from(elapsed) * u128::from(TICKS_P)
                / u128::from(TICKS_Q)
                / u128::from(frequency)) as u64
        };
        let mut scene = None;
        for _ in 0..CATCH_UP {
            if admitted >= due || !outbox.admits() {
                break;
            }
            if options.ticks.is_some_and(|ticks| index >= ticks) {
                break 'game;
            }
            let buttons = match &options.inputs {
                Some(pads) => match pads.get(index as usize) {
                    Some(&pad) => pad,
                    None => break 'game,
                },
                None => held_buttons(),
            };
            let latched = match &options.inputs {
                Some(pads) => pads.get(index as usize + 1).copied(),
                None => Some(held_buttons()),
            };
            let output = session.step(TickInput {
                buttons,
                latched,
                host_results: std::mem::take(&mut receipts),
            });
            if options.digests_out.is_some() {
                digest_lines.push(session.digest_line());
            }
            scene = Some(output.scene);
            if outbox
                .put(TickBatch {
                    tick: output.tick,
                    audio: output.effects.audio,
                    save: output.effects.save,
                })
                .is_err()
            {
                return Err("a Tick ran while effects waited".into());
            }
            outbox.offer(&mut audio, &mut saves);
            index += 1;
            admitted += 1;
        }
        // The latest complete Scene; several Ticks may precede one presentation.
        if let Some(scene) = scene {
            present(&scene.pixels);
        } else if !replaying {
            // SAFETY: SDL is initialized.
            unsafe { SDL_Delay(1) };
        }
    }
    while !outbox.admits() {
        outbox.offer(&mut audio, &mut saves);
    }
    drop(saves);
    println!(
        "Slice ran {} ticks; effect trace {:016x}; fault: {}",
        session.step_count(),
        session.effect_trace(),
        session.faulted()
    );
    if let Some(path) = &options.digests_out {
        let mut text = digest_lines.join("\n");
        text.push('\n');
        std::fs::write(path, text).map_err(|e| format!("{path}: {e}"))?;
    }
    // SAFETY: nothing uses SDL after this.
    unsafe { SDL_Quit() };
    Ok(())
}

fn main() {
    if let Err(error) = run() {
        eprintln!("Error: {error}");
        std::process::exit(1);
    }
}
