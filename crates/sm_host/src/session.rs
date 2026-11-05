//! The Session interface of the slice: open from prepared content and a sealed
//! start, step one Tick at a time, take the Tick's Scene and effects as owned
//! records. Nothing here opens a file or a device.
use crate::audio_output::AudioOutput;
use crate::cpu_infra::{RM_BOTH, RM_MINE, rtl_run_frame_compare};
use crate::game_state::Game;
use sm_data::digest::{Digest, MACHINE};
use sm_data::prepared::{PreparedContent, PreparedStart};
use std::cell::RefCell;
use std::hash::{Hash, Hasher};
use std::rc::Rc;

const FRAME_BYTES: usize = 256 * 240 * 4;
const SAMPLES_PER_TICK: usize = 534;

unsafe fn die(message: *const core::ffi::c_char) {
    let message = unsafe { core::ffi::CStr::from_ptr(message) };
    panic!("{}", message.to_string_lossy());
}
unsafe fn warning(message: *const core::ffi::c_char) {
    let message = unsafe { core::ffi::CStr::from_ptr(message) };
    eprintln!("{}", message.to_string_lossy());
}
unsafe fn no_lock() {}
fn install_hooks() {
    static ONCE: std::sync::Once = std::sync::Once::new();
    ONCE.call_once(|| {
        crate::host::install_host_hooks(crate::host::HostHooks {
            die,
            warning,
            apu_lock: no_lock,
            apu_unlock: no_lock,
        })
    });
}

/// The committed Tick's image, BGRA at 256x240.
#[derive(Clone)]
pub struct Scene {
    pub pixels: Vec<u8>,
}

/// One Tick's sound: its cues as the game asked for them and the driver's samples.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct AudioBatch {
    pub cues: Vec<sm_data::EmittedCue>,
    pub samples: Vec<[i16; 2]>,
}

/// A save the game committed this Tick: the image to store, owned.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct SaveRequest {
    pub sram: Vec<u8>,
}

#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct TickEffects {
    pub audio: AudioBatch,
    pub save: Option<SaveRequest>,
}

/// A host's answer to an earlier effect, delivered with a later Tick's input.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum HostResult {
    SaveStored { tick: u64, stored: bool },
}

/// One Tick's input.
#[derive(Clone, Debug, Default)]
pub struct TickInput {
    pub buttons: u16,
    /// The controller word the Tick's closing NMI latches: the next Tick's
    /// buttons on a replay, the device's word at vblank in live play.
    pub latched: Option<u16>,
    pub host_results: Vec<HostResult>,
}

/// One committed Tick.
pub struct TickOutput {
    pub tick: u64,
    pub scene: Scene,
    pub effects: TickEffects,
}

/// The Tick the start committed: its Scene, and no effects.
pub struct InitialOutput {
    pub tick: u64,
    pub scene: Scene,
}

pub struct OpenedSession {
    pub session: Session,
    pub initial: InitialOutput,
}

/// A Session that will not advance again.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum OpenError {
    Prepared(sm_data::prepared::PreparedError),
    Content,
    Start(String),
}

impl core::fmt::Display for OpenError {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        match self {
            Self::Prepared(error) => write!(f, "{error}"),
            Self::Content => f.write_str("the content image is not valid"),
            Self::Start(error) => write!(f, "the start does not open: {error}"),
        }
    }
}

/// One deterministic game: everything it owns advances only in `step`.
pub struct Session {
    game: Box<Game>,
    pixels: Vec<u8>,
    samples: Vec<[i16; 2]>,
    tick: u64,
    cues: Rc<RefCell<Vec<sm_data::EmittedCue>>>,
    stored_sram: Vec<u8>,
    trace: Digest,
    receipts: Vec<HostResult>,
}

impl Session {
    /// Open a complete start. It runs no game code and performs no I/O.
    pub fn open(
        content: &PreparedContent,
        start: &PreparedStart,
    ) -> Result<OpenedSession, OpenError> {
        let state = start.state_for(content).map_err(OpenError::Prepared)?;
        install_hooks();
        let mut game = Game::new();
        let mut pixels = vec![0; FRAME_BYTES];
        let image = content.image();
        // SAFETY: the backend these calls install lives in `game` for the
        // Session's life, and only this thread and owner reach it.
        unsafe {
            let g = &mut *game;
            g.VERIFY_ACTIVE = true;
            g.VERIFY_FAILURES = 0;
            g.g_runmode = RM_BOTH as u8;
            g.note_samus_external_mutation();
            g.initialize_backend();
            if !crate::snes::snes_other::snes_load_rom(image.as_ptr(), image.len() as i32, g) {
                return Err(OpenError::Content);
            }
            g.g_sram = (*(*g.g_snes).cart).ram.clone();
            let cartridge = (*(*g.g_snes).cart).rom.clone();
            let engine = cartridge.to_rom_image();
            g.cartridge_rom = Some(cartridge);
            g.register_native_rom(engine.clone());
            g.register_rom(engine);
            crate::rtl::rtl_setup_emu_callbacks(
                core::ptr::null_mut(),
                Some(rtl_run_frame_compare as unsafe fn(u16, i32, &mut Game) -> ()),
                None,
                g,
            );
            g.create_native_player();
            g.initialize_native_player();
            let snes = g.g_snes;
            for ppu in [(*snes).snes_ppu, (*snes).my_ppu] {
                crate::snes::ppu::ppu_begin_drawing(ppu.cast(), pixels.as_mut_ptr(), 1024, 0);
            }
            crate::verification_checkpoint::restore_capture_with_sources(state, g, &[])
                .map_err(OpenError::Start)?;
            crate::snes::ppu::observe_pixels(true);
        }
        let cues = Rc::new(RefCell::new(Vec::new()));
        let recorded = cues.clone();
        game.audio_output = crate::audio_output::AudioOutputSink::new(move |output| {
            if let AudioOutput::CueBegin {
                emission: Some(cue),
                ..
            } = output
            {
                recorded.borrow_mut().push(cue);
            }
            Ok(())
        });
        game.audio_output.new_epoch();
        let stored_sram = game.g_sram.bytes(0..0x2000);
        let scene = Scene {
            pixels: pixels.clone(),
        };
        Ok(OpenedSession {
            session: Self {
                game,
                pixels,
                samples: vec![[0; 2]; SAMPLES_PER_TICK],
                tick: 0,
                cues,
                stored_sram,
                trace: Digest::default(),
                receipts: Vec::new(),
            },
            initial: InitialOutput { tick: 0, scene },
        })
    }

    /// Advance exactly one Tick: input index i commits Tick i + 1.
    pub fn step(&mut self, input: TickInput) -> TickOutput {
        self.receipts.extend(input.host_results);
        self.pixels.fill(0);
        let game = &mut *self.game;
        game.g_runmode = RM_MINE as u8;
        // SAFETY: as in `open`; a frame runs to completion on this owner.
        unsafe {
            crate::oracle_mode::set_next_frame_input(input.latched.map(u16::reverse_bits));
            crate::rtl::rtl_run_frame(
                crate::rtl::TickInput {
                    buttons: input.buttons,
                },
                game,
            );
            crate::oracle_mode::set_next_frame_input(None);
            crate::rtl::rtl_render_audio(&mut self.samples, game);
        }
        self.tick += 1;
        let cues = std::mem::take(&mut *self.cues.borrow_mut());
        let sram = self.game.g_sram.bytes(0..0x2000);
        let save = (sram != self.stored_sram).then(|| {
            self.stored_sram = sram.clone();
            SaveRequest { sram }
        });
        let audio = AudioBatch {
            cues,
            samples: self.samples.clone(),
        };
        self.tick.hash(&mut self.trace);
        format!("{:?}", audio.cues).hash(&mut self.trace);
        audio.samples.hash(&mut self.trace);
        save.is_some().hash(&mut self.trace);
        TickOutput {
            tick: self.tick,
            scene: Scene {
                pixels: self.pixels.clone(),
            },
            effects: TickEffects { audio, save },
        }
    }

    /// The committed Tick's digests as one text line: the Tick, then each
    /// machine stream in `MACHINE` order. Two hosts' lines compare Tick by Tick.
    pub fn digest_line(&mut self) -> String {
        let digests = self.digests();
        self.line_of(&digests)
    }

    /// `digests`, taken once for the committed Tick, as `digest_line` writes them.
    /// Taking the digests is an observation: `SM_DIGEST_LOG` logs each one.
    pub fn line_of(&self, digests: &[u64; MACHINE.len()]) -> String {
        let mut line = self.tick.to_string();
        for digest in digests {
            line.push_str(&format!(" {digest:016x}"));
        }
        line
    }

    /// The committed Tick's digests of the machine streams the oracle lock names.
    pub fn digests(&mut self) -> [u64; MACHINE.len()] {
        crate::machine::capture_result(&mut self.game);
        crate::machine::result_digests(&mut self.game)
    }

    /// The running digest of every effect so far: the semantic effect trace.
    pub fn effect_trace(&self) -> u64 {
        self.trace.finish()
    }

    /// The host results delivered so far, in order.
    pub fn receipts(&self) -> &[HostResult] {
        &self.receipts
    }

    /// The Ticks committed so far.
    pub fn step_count(&self) -> u64 {
        self.tick
    }

    /// Whether the engine recorded a fault.
    pub fn faulted(&self) -> bool {
        self.game.g_fail
    }
}
