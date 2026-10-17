//! A window and audio sink for already-completed native game output.
//!
//! This module never advances gameplay or audio emulation. The caller samples
//! held input, advances its Game, presents the result, and owns 60 Hz pacing.
//! SDL2 must be supplied by the host; no SDL source or binary is bundled here.

mod sdl;

use std::ffi::c_void;
use std::fmt;
use std::marker::PhantomData;
use std::path::{Path, PathBuf};
use std::ptr;
use std::rc::Rc;
use std::sync::atomic::{AtomicBool, Ordering};

pub const SAMPLE_RATE: u32 = 32_000;
pub const CHANNELS: usize = 2;
/// At most 250 ms of queued stereo signed-16-bit PCM.
pub const MAX_QUEUED_AUDIO_BYTES: u32 = SAMPLE_RATE;
pub const CONTROLS: &str = "Arrows: move; Z: jump (A); X: shoot (X); C: run (B); \
    D: Y; A/S: L/R; Shift: Select; Enter: Start; Escape: quit";

#[derive(Debug)]
pub struct Error(String);

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(&self.0)
    }
}

impl std::error::Error for Error {}

static WINDOW_ACTIVE: AtomicBool = AtomicBool::new(false);

struct Session;

impl Session {
    fn acquire() -> Result<Self, Error> {
        WINDOW_ACTIVE
            .compare_exchange(false, true, Ordering::AcqRel, Ordering::Acquire)
            .map_err(|_| Error("only one SDL presentation window may be open".into()))?;
        Ok(Self)
    }
}

impl Drop for Session {
    fn drop(&mut self) {
        WINDOW_ACTIVE.store(false, Ordering::Release);
    }
}

/// An SDL2 presentation owner, used and dropped on the thread that creates it.
///
/// Use the application's main thread. An error creating audio is reported to
/// the caller; set SDL_AUDIODRIVER=dummy explicitly for silent/headless smoke.
pub struct Window {
    api: sdl::Api,
    window: *mut c_void,
    renderer: *mut c_void,
    texture: *mut c_void,
    audio: u32,
    width: u32,
    pixel_count: usize,
    keys: [bool; 512],
    closed: bool,
    audio_resets: u64,
    _session: Session,
    _main_thread: PhantomData<Rc<()>>,
}

impl Window {
    /// Load `sdl_library`, then `SM_RESTART_SDL2`, then the system SDL2 name.
    /// Width/height describe packed ARGB8888 pixels; alpha is ignored.
    pub fn new(width: u32, height: u32, scale: u32, sdl_library: Option<&Path>) -> Result<Self, Error> {
        let (screen_width, screen_height, pixel_count) = dimensions(width, height, scale)?;
        let session = Session::acquire()?;
        let library = sdl_library
            .map(Path::to_path_buf)
            .or_else(|| std::env::var_os("SM_RESTART_SDL2").map(PathBuf::from))
            .unwrap_or_else(|| PathBuf::from(default_library()));
        let api = sdl::Api::load(&library)?;
        let mut this = Self {
            api,
            window: ptr::null_mut(),
            renderer: ptr::null_mut(),
            texture: ptr::null_mut(),
            audio: 0,
            width,
            pixel_count,
            keys: [false; 512],
            closed: false,
            audio_resets: 0,
            _session: session,
            _main_thread: PhantomData,
        };
        // SAFETY: all calls use SDL2's declared signatures and initialized
        // arguments. Each acquired resource is stored immediately for Drop;
        // failures clean up the resources acquired so far, in reverse order.
        unsafe {
            this.check((this.api.init)(sdl::INIT), "initialize SDL2")?;
            this.window = (this.api.create_window)(
                c"Super Metroid — Rust restart".as_ptr(),
                0x2fff0000, // SDL_WINDOWPOS_CENTERED
                0x2fff0000,
                screen_width,
                screen_height,
                0x20, // SDL_WINDOW_RESIZABLE
            );
            if this.window.is_null() {
                return Err(this.api.error("create window"));
            }
            // Software rendering needs no OpenGL context and works with the
            // dummy video driver as well as the ordinary desktop driver.
            this.renderer = (this.api.create_renderer)(this.window, -1, 1);
            if this.renderer.is_null() {
                return Err(this.api.error("create renderer"));
            }
            this.check((this.api.set_logical_size)(this.renderer, width as i32, height as i32), "set logical size")?;
            this.texture = (this.api.create_texture)(
                this.renderer,
                sdl::ARGB8888,
                1, // SDL_TEXTUREACCESS_STREAMING
                width as i32,
                height as i32,
            );
            if this.texture.is_null() {
                return Err(this.api.error("create texture"));
            }
            this.check((this.api.set_blend_mode)(this.texture, 0), "disable blending")?;
            let wanted = sdl::AudioSpec {
                freq: SAMPLE_RATE as i32,
                format: if cfg!(target_endian = "little") { 0x8010 } else { 0x9010 },
                channels: CHANNELS as u8,
                silence: 0,
                samples: 512,
                padding: 0,
                size: 0,
                callback: None, // Queue-only: SDL never calls the game.
                userdata: ptr::null_mut(),
            };
            // With allowed_changes=0, SDL converts to hardware format itself
            // while keeping the caller's exact rate/channel/sample contract.
            this.audio = (this.api.open_audio)(ptr::null(), 0, &wanted, ptr::null_mut(), 0);
            if this.audio == 0 {
                return Err(this.api.error("open 32000 Hz stereo audio"));
            }
            (this.api.pause_audio)(this.audio, 0);
        }
        Ok(this)
    }

    /// Pump events and return held native bits B,Y,Select,Start,Up,Down,Left,
    /// Right,A,X,L,R, or None after Escape/window close. Focus loss releases all.
    pub fn poll(&mut self) -> Result<Option<u16>, Error> {
        let mut event = sdl::Event { padding: [0; 56] };
        // SAFETY: SDL writes one initialized SDL_Event into an adequately
        // sized/aligned union. We inspect only the variant selected by kind.
        unsafe {
            while (self.api.poll_event)(&mut event) != 0 {
                match event.kind {
                    sdl::QUIT => self.closed = true,
                    sdl::WINDOW_EVENT => match event.window.event {
                        sdl::FOCUS_LOST => self.keys.fill(false),
                        sdl::WINDOW_CLOSE => self.closed = true,
                        _ => {}
                    },
                    sdl::KEY_DOWN | sdl::KEY_UP => {
                        let code = event.key.keysym.scancode;
                        if code == 41 && event.kind == sdl::KEY_DOWN {
                            // Escape
                            self.closed = true;
                        }
                        if let Ok(index) = usize::try_from(code)
                            && let Some(key) = self.keys.get_mut(index)
                        {
                            *key = event.kind == sdl::KEY_DOWN;
                        }
                    }
                    _ => {}
                }
            }
        }
        Ok((!self.closed).then(|| buttons(&self.keys)))
    }

    /// Upload a completed image and queue completed interleaved stereo PCM.
    ///
    /// A stalled/overproducing caller never builds an unbounded audio backlog:
    /// exceeding 250 ms clears the old queue before adding the current block.
    /// This affects audible output only. The caller owns game/presentation pace.
    pub fn present(&mut self, pixels: &[u32], audio: &[i16]) -> Result<(), Error> {
        if pixels.len() != self.pixel_count {
            return Err(Error(format!("expected {} pixels, got {}", self.pixel_count, pixels.len())));
        }
        if !audio.len().is_multiple_of(CHANNELS) {
            return Err(Error("audio must contain complete stereo frames".into()));
        }
        let audio_bytes = std::mem::size_of_val(audio);
        if audio_bytes > MAX_QUEUED_AUDIO_BYTES as usize {
            return Err(Error("one audio block exceeds the 250 ms queue limit".into()));
        }
        // SAFETY: texture dimensions/pitch match the validated pixel slice;
        // SDL copies pixels and PCM during these calls. No borrow escapes.
        unsafe {
            self.check(
                (self.api.update_texture)(self.texture, ptr::null(), pixels.as_ptr().cast(), (self.width * 4) as i32),
                "upload pixels",
            )?;
            self.check((self.api.render_clear)(self.renderer), "clear renderer")?;
            self.check((self.api.render_copy)(self.renderer, self.texture, ptr::null(), ptr::null()), "copy texture")?;
            (self.api.render_present)(self.renderer);
            if audio_bytes != 0 {
                if (self.api.queued_audio_size)(self.audio) + audio_bytes as u32 > MAX_QUEUED_AUDIO_BYTES {
                    (self.api.clear_audio)(self.audio);
                    self.audio_resets += 1;
                }
                self.check(
                    (self.api.queue_audio)(self.audio, audio.as_ptr().cast(), audio_bytes as u32),
                    "queue audio",
                )?;
            }
        }
        Ok(())
    }

    pub fn queued_audio_bytes(&self) -> u32 {
        // SAFETY: the audio device remains owned and open until Drop.
        unsafe { (self.api.queued_audio_size)(self.audio) }
    }

    /// Number of old audio queues discarded to bound latency.
    pub fn audio_resets(&self) -> u64 {
        self.audio_resets
    }

    fn check(&self, result: i32, operation: &str) -> Result<(), Error> {
        if result < 0 { Err(self.api.error(operation)) } else { Ok(()) }
    }
}

impl Drop for Window {
    fn drop(&mut self) {
        // SAFETY: resources are owned exclusively; pointer handles may be null
        // only during failed construction. SDL2 code remains loaded throughout.
        unsafe {
            if self.audio != 0 {
                (self.api.close_audio)(self.audio);
            }
            if !self.texture.is_null() {
                (self.api.destroy_texture)(self.texture);
            }
            if !self.renderer.is_null() {
                (self.api.destroy_renderer)(self.renderer);
            }
            if !self.window.is_null() {
                (self.api.destroy_window)(self.window);
            }
            (self.api.quit)();
        }
    }
}

fn dimensions(width: u32, height: u32, scale: u32) -> Result<(i32, i32, usize), Error> {
    let invalid = || Error("nonzero dimensions, pitch and scaled size must fit an SDL integer".into());
    if width == 0 || height == 0 || scale == 0 {
        return Err(invalid());
    }
    width.checked_mul(4).filter(|pitch| *pitch <= i32::MAX as u32).ok_or_else(invalid)?;
    let screen_width = width.checked_mul(scale).and_then(|n| i32::try_from(n).ok()).ok_or_else(invalid)?;
    let screen_height = height.checked_mul(scale).and_then(|n| i32::try_from(n).ok()).ok_or_else(invalid)?;
    let pixels = (width as usize).checked_mul(height as usize).ok_or_else(invalid)?;
    Ok((screen_width, screen_height, pixels))
}

fn default_library() -> &'static str {
    if cfg!(target_os = "windows") {
        "SDL2.dll"
    } else if cfg!(target_os = "macos") {
        "libSDL2-2.0.0.dylib"
    } else {
        "libSDL2-2.0.so.0"
    }
}

fn buttons(keys: &[bool; 512]) -> u16 {
    // SDL physical scancodes. Native order follows RtlRunFrame/input1.currentState.
    // Fresh recovered-C defaults use native A to jump, X to shoot, B to run.
    let mapping = [
        (6, 0),
        (7, 1),
        (225, 2),
        (229, 2),
        (40, 3),
        (82, 4),
        (81, 5),
        (80, 6),
        (79, 7),
        (29, 8),
        (27, 9),
        (4, 10),
        (22, 11),
    ];
    mapping.into_iter().fold(0, |held, (key, bit)| held | (u16::from(keys[key]) << bit))
}

#[cfg(test)]
mod tests;
