//! The headless engine: open a prepared start over prepared content, step Ticks.
use crate::cpu_infra::{RM_BOTH, RM_MINE, rtl_run_frame_compare};
use crate::game_state::Game;
use sm_data::digest::MACHINE;

const FRAME_BYTES: usize = 256 * 240 * 4;

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

/// One running slice. It owns the frame the renderer draws into.
pub struct Engine {
    game: Box<Game>,
    pixels: Vec<u8>,
    discarded_audio: Vec<[i16; 2]>,
}

impl Engine {
    /// Open `start` (a prepared start) over `content` (the content image).
    pub fn open(content: &[u8], start: &[u8]) -> Result<Self, String> {
        install_hooks();
        let mut game = Game::new();
        let mut pixels = vec![0; FRAME_BYTES];
        unsafe {
            let g = &mut *game;
            g.VERIFY_ACTIVE = true;
            g.VERIFY_FAILURES = 0;
            g.g_runmode = RM_BOTH as u8;
            g.note_samus_external_mutation();
            g.initialize_backend();
            if !crate::snes::snes_other::snes_load_rom(content.as_ptr(), content.len() as i32, g) {
                return Err("content image rejected".into());
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
            crate::verification_checkpoint::restore_capture_with_sources(start, g, &[])?;
            crate::snes::ppu::observe_pixels(true);
        }
        Ok(Self {
            game,
            pixels,
            discarded_audio: vec![[0; 2]; 534],
        })
    }

    /// One Tick on the controller word as a movie stores it.
    pub fn step(&mut self, pad: u16) {
        self.pixels.fill(0);
        let game = &mut *self.game;
        game.g_runmode = RM_MINE as u8;
        unsafe {
            crate::rtl::rtl_run_frame(crate::rtl::TickInput { buttons: pad }, game);
            crate::rtl::rtl_render_audio(&mut self.discarded_audio, game);
        }
    }

    /// The Tick's machine digests, as the oracle lock's streams name them.
    pub fn digests(&mut self) -> [u64; MACHINE.len()] {
        crate::machine::capture_result(&mut self.game);
        crate::machine::result_digests(&mut self.game)
    }

    /// The frame the renderer drew, BGRA at 256x240.
    pub fn frame(&self) -> &[u8] {
        &self.pixels
    }

    /// Failures the engine recorded.
    pub fn failed(&self) -> bool {
        self.game.g_fail
    }
}
