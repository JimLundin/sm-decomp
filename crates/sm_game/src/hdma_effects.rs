//! HDMA wave, layer, haze and eye-beam effects over the host's session view.
#![forbid(unsafe_code)]
use crate::actors::access::{AccessMode::LittleEndian as Le, ActorAccess, ActorKey as Key};
use crate::player_projectiles::access::Store;
use crate::session_access::SessionAccess;
use crate::visual_host::{Content, Record, Source};
use crate::visual_state::{EyeBeamPhase, HazePhase, HdmaWave};

/// The session view these effects read and write. Its bytes are the live
/// projection of RAM and the owned Session, as the compatibility Runtime keeps it.
pub trait Host: SessionAccess + ActorAccess + Content {
    fn byte(&self, at: usize) -> u8;
    fn set_byte(&mut self, at: usize, value: u8);
    /// Store `bytes` in order through the projection.
    fn write_slice(&mut self, at: usize, bytes: &[u8]);
    fn store(&mut self, at: usize, value: u16, store: Store);
    fn word(&self, at: usize) -> u16 {
        u16::from_le_bytes([self.byte(at), self.byte(at + 1)])
    }
    fn native_word(&self, at: usize) -> u16 {
        u16::from_ne_bytes([self.byte(at), self.byte(at + 1)])
    }
    fn write_slice_le(&mut self, at: usize, value: u16) {
        self.write_slice(at, &value.to_le_bytes());
    }
    fn write_slice_native(&mut self, at: usize, value: u16) {
        self.write_slice(at, &value.to_ne_bytes());
    }
    /// Build the eye beam's window table outside the session view.
    fn calculate_xray_window(
        &mut self,
        x: u16,
        y: u16,
        angle: u16,
        width: u16,
        off_screen: bool,
        table: usize,
    );
}

pub trait HdmaEffects: Host {
    fn initialize_wave_pair(&mut self, base: usize, primary: u16) {
        let source = (0xc00000 | u32::from(primary)).to_ne_bytes();
        self.write_slice_le(base, 192);
        self.write_slice_le(base + 6, 192);
        for at in [base + 1, base + 7] {
            // Keep high-half, low-word, alternate-pointer publication order.
            self.write_slice(at + 2, &source[2..]);
            self.write_slice_native(at, primary);
            self.write_slice_le(at + 3, primary.wrapping_add(128));
        }
        self.write_slice_le(base + 12, 0);
    }

    fn reset_hdma_wave(&mut self, slot: u16) {
        let object = usize::from(slot & !1);
        for (field, value) in [(0x1914, 0xfffe), (0x1920, 1), (0x192c, 0), (0x1938, 0)] {
            self.write_slice_le(field + object, value);
        }
    }

    fn initialize_phantoon_wave(&mut self, slot: u16) {
        use crate::actors::access::AccessMode::Native;
        let motion = Key::from_legacy(0x80);
        let controller = Key::from_legacy(0x40);
        let mode = self.resolve_actor(motion, Native).parameters[0];
        self.edit_actor(controller, Native, |actor| actor.parameters[0] = mode);
        if self.resolve_actor(motion, Native).parameters[0] & 1 != 0 {
            self.initialize_wave_pair(0x9000, 0x9100);
        } else {
            for at in [0x9000, 0x9006, 0x900c, 0x900f, 0x9012, 0x9015] {
                self.write_slice_le(at, 160);
            }
            for at in [0x9001, 0x9007] {
                self.write_slice(at + 2, &0xa09100u32.to_ne_bytes()[2..]);
                self.write_slice_native(at, 0x9100);
            }
            for at in [0x900d, 0x9013] {
                self.write_slice_native(at, 0x9100);
            }
            for at in [0x9004, 0x900a] {
                self.write_slice_le(at, 0x9140);
            }
            for at in [0x9010, 0x9016] {
                self.write_slice_native(at, 0x9140);
            }
            self.write_slice_le(0x9018, 0);
        }
        self.reset_hdma_wave(slot);
    }

    fn select_phantoon_layer(&mut self) {
        use crate::actors::access::AccessMode::Native;
        use crate::ram::words as ram;
        if self.word(ram::phantom_related_layer_flag) & 0x4000 != 0 {
            self.write_slice_le(ram::fx_layer_blending_config_c, 26);
        } else {
            // The native first byte belongs to EnemyData record 3, parameter 1.
            let alpha = self
                .resolve_actor(Key::from_legacy(0xc0), Native)
                .parameters[0]
                .to_ne_bytes()[0];
            match alpha {
                0 => self.write_slice_le(ram::fx_layer_blending_config_c, 4),
                0xff => {
                    self.write_slice_le(ram::fx_layer_blending_config_c, 4);
                    let object = usize::from(self.word(ram::hdma_object_index) & !1);
                    self.write_slice_le(
                        0x18cc + object,
                        self.word(0x18cc + object).wrapping_add(2),
                    );
                    self.write_slice_le(0x18e4 + object, 1);
                }
                _ => {}
            }
        }
    }

    fn select_draygon_layer(&mut self, slot: u16) {
        use crate::actors::access::AccessMode::Native;
        let actor = self.resolve_actor(Key::from_legacy(0), Native);
        let x = i32::from(actor.x.whole) - i32::from(self.inputs().camera_x);
        let y = i32::from(actor.y.whole) - i32::from(self.inputs().camera_y);
        let visible = actor.properties & 0x200 == 0
            && (x + 64) as i16 >= 0
            && (x - 320) & 0x8000 != 0
            && (y + 16) as i16 >= 0
            && (y - 304) & 0x8000 != 0;
        let program = if !visible {
            0xdf71
        } else if (i32::from(y as i16) - 40) & 0x8000 != 0 {
            0xdf6b
        } else if (i32::from(y as i16) - 192) & 0x8000 != 0 {
            0xdf5f
        } else {
            0xdf65
        };
        let object = usize::from(slot & !1);
        self.write_slice_le(0x18e4 + object, 1);
        self.write_slice_le(0x18cc + object, program);
    }

    fn advance_hdma_wave(&mut self, slot: u16, wave: HdmaWave) {
        use crate::actors::access::AccessMode::Native;
        use crate::player_projectiles::access::Store;
        use crate::projectile_geometry::SINE_COSINE;
        use crate::ram::{word_arrays as arrays, words as ram};
        let controller = Key::from_legacy(0x40);
        let motion = Key::from_legacy(0xc0);
        let phantoon = matches!(wave, HdmaWave::Phantoon);
        let mode = if phantoon {
            self.resolve_actor(controller, Native).parameters[0]
        } else {
            self.word(0xd9c)
        };
        if mode == 0 {
            self.advance_hdma_program();
            return;
        }
        let step = if phantoon && mode & 1 == 0 { 8 } else { 4 };
        let object = usize::from(slot & !1);
        let advance = if phantoon {
            let delay = self.word(0x1920 + object);
            self.write_slice_le(0x1920 + object, delay.wrapping_sub(1));
            if delay == 1 {
                self.write_slice_le(0x1920 + object, 1);
            }
            delay == 1
        } else {
            true
        };
        if advance {
            let speed = if phantoon {
                self.resolve_actor(motion, Native).pre_instruction
            } else {
                self.word(ram::loop_counter_transfer_enemies_to_vram)
            };
            let phase = speed
                .wrapping_mul(2)
                .wrapping_add(self.word(0x1914 + object))
                & 0x1ff;
            self.write_slice_le(0x1914 + object, phase);
        }
        let mut phase = self.word(0x1914 + object);
        // Cinematic BG3 data is separate from Phantoon's BG2 table.
        let (scroll, store, table) = if phantoon {
            (ram::reg_BG2HOFS, Store::SliceNative, ram::g_word_7E9100)
        } else {
            (ram::reg_BG3HOFS, Store::SliceLe, arrays::hdma_table_1)
        };
        for row in 0..256 / step {
            let sample = SINE_COSINE[usize::from(phase >> 1) + 64];
            let (low, high) = if phantoon {
                let amplitude = self.resolve_actor(motion, Native).variables[3];
                (amplitude as u8, amplitude.to_ne_bytes()[1])
            } else {
                (self.byte(0xd9e), self.byte(0xd9f))
            };
            let offset = hdma_wave_offset(sample, low, high);
            phase = phase.wrapping_add(step) & 0x1ff;
            let value = self.word(scroll).wrapping_add(offset);
            self.store(table + 2 * usize::from(row), value, store);
        }
        // Phantoon samples its mode again after the first half is published.
        let narrow = phantoon && self.resolve_actor(controller, Native).parameters[0] & 1 == 0;
        let rows = if narrow { 32 } else { 64 };
        for row in (0..rows).rev() {
            let origin = self.word(scroll).wrapping_add(self.word(scroll));
            let sample = if phantoon {
                self.native_word(table + 2 * row)
            } else {
                self.word(table + 2 * row)
            };
            let value = origin.wrapping_sub(sample);
            self.store(table + 2 * (rows + row), value, store);
        }
    }

    fn begin_haze(&mut self, slot: u16, color: u16) {
        let object = usize::from(slot as u8 & !1);
        self.write_slice_le(0x1920 + object, color);
        self.write_slice_le(0x1914 + object, 0);
        if self.session().room.entry.transition_selector == 0xe737 {
            self.write_slice_le(0x18f0 + object, 0xde2d);
            self.advance_haze(slot, HazePhase::FadingIn);
        }
    }

    fn advance_haze(&mut self, slot: u16, phase: HazePhase) {
        self.write_slice(0x74, &[0x20, 0x40, 0x80]);
        self.write_slice_le(0x1986, 44);
        let byte_slot = usize::from(slot as u8);
        let object = byte_slot & !1;
        let level = 0x1914 + object;
        match phase {
            HazePhase::FadedIn => {
                if self.session().room.entry.transition_selector == 0xe2db {
                    self.write_slice_le(0x18f0 + object, 0xde96);
                }
                return;
            }
            HazePhase::FadingIn if self.word(level) == 16 => {
                self.write_slice_le(0x18f0 + object, 0xde74);
                return;
            }
            HazePhase::FadingOut if self.word(level) == 0 => return,
            _ => {}
        }
        // Color samples use the byte slot even when its word slot is rounded down.
        let color = self.byte(0x1920 + byte_slot);
        let mut shade = self.byte(0x1914 + byte_slot) as i8;
        for row in (0..16).rev() {
            self.set_byte(0x9d00 + row, color | shade as u8);
            shade = shade.wrapping_sub(1);
            if shade < 0 {
                shade = 0;
            }
        }
        let delta = if matches!(phase, HazePhase::FadingIn) {
            1
        } else {
            u16::MAX
        };
        self.write_slice_le(level, self.word(level).wrapping_add(delta));
    }

    fn advance_hdma_program(&mut self) {
        let current = usize::from(self.word(0x18b2) & !1);
        let program = 0x18cc + current;
        self.write_slice_le(program, self.word(program).wrapping_add(2));
        self.write_slice_le(0x18e4 + current, 1);
    }

    fn add_eye_beam_fixed(&mut self, whole: usize, fraction: usize, increment: u32) -> u32 {
        let value = (u32::from(self.word(fraction)) | u32::from(self.word(whole)) << 16)
            .wrapping_add(increment);
        self.write_slice_le(fraction, value as u16);
        self.write_slice_le(whole, (value >> 16) as u16);
        value
    }

    fn initialize_eye_beam(&mut self, slot: u16) {
        for (color, fixed, value) in [
            (0x1914, 0x74, 0x30),
            (0x1915, 0x75, 0x50),
            (0x1920, 0x76, 0x80),
        ] {
            self.set_byte(color + usize::from(slot), value);
            self.set_byte(fixed, value);
        }
        self.write_slice_le(0x1986, 16);
        // The original stores also clear the byte after the HDMA terminator.
        self.write_slice(
            0x9000,
            &[0xe4, 0x00, 0x91, 0xe4, 0xc8, 0x91, 0x98, 0x90, 0x92, 0, 0],
        );
        self.edit_actor(Key::from_legacy(0x40), Le, |actor| actor.variables[2] = 1);
        let even_slot = usize::from(slot & !1);
        self.write_slice_le(0x192c + even_slot, 0);
        self.write_slice_le(0x1938 + even_slot, 0);
        self.write_slice(0x9080, &[0; 4]);
        self.write_slice_le(0x9090, 0);
    }

    fn advance_eye_beam(&mut self, slot: u16, phase: EyeBeamPhase) {
        self.write_slice_le(0x1986, 16);
        let even_slot = usize::from(slot & !1);
        match phase {
            EyeBeamPhase::Widening => {
                let speed = self.add_eye_beam_fixed(0x9080, 0x9082, 0x4000);
                let width = self.add_eye_beam_fixed(0x192c + even_slot, 0x1938 + even_slot, speed);
                if ((width >> 16) as u16).wrapping_sub(4) as i16 >= 0 {
                    // Keep the fractional width when the integer part reaches the limit.
                    self.write_slice_le(0x192c + even_slot, 4);
                    self.advance_hdma_program();
                }
                self.draw_eye_beam(slot);
            }
            EyeBeamPhase::Full => {
                if self.resolve_actor(Key::from_legacy(0x40), Le).variables[2] == 0 {
                    self.advance_hdma_program();
                } else {
                    self.draw_eye_beam(slot);
                    let index = usize::from(self.word(0x9090).wrapping_mul(4));
                    let colors = self.table(Source::Loaded, 0x88ea8b);
                    for (lane, target) in [0x1914, 0x1915, 0x1920].into_iter().enumerate() {
                        self.set_byte(target + usize::from(slot), colors.byte(index + lane));
                    }
                    // Color destinations can overlap the index, so advance its live value.
                    self.write_slice_le(0x9090, self.word(0x9090).wrapping_add(1) & 0xf);
                }
            }
            EyeBeamPhase::Fading => {
                if self.byte(0x1915 + usize::from(slot)) == 0x40 {
                    self.write_slice(0x74, &[0x20, 0x40, 0x80]);
                    self.write_slice_le(0x1938 + even_slot, 0);
                    self.write_slice_le(0x192c + even_slot, 0);
                    for entry in (0x9100..0x9300).step_by(2).rev() {
                        self.write_slice_le(entry, 0xff);
                    }
                    self.write_slice(0x9000, &[0; 10]);
                    self.advance_hdma_program();
                } else {
                    self.draw_eye_beam(slot);
                    let current = usize::from(self.word(0x18b2));
                    // Colors use byte slots, including odd current object indices.
                    for (color, black) in [(0x1914, 0x20), (0x1915, 0x40), (0x1920, 0x80)] {
                        let value = self.byte(color + current);
                        if value != black {
                            self.set_byte(color + current, value.wrapping_sub(1));
                        }
                    }
                }
            }
        }
    }

    fn draw_eye_beam(&mut self, slot: u16) {
        let actor = self.resolve_actor(Key::from_legacy(0x40), Le);
        let angle = actor.variables[3];
        let width = self.word(0x192c + usize::from(slot & !1));
        let y = actor.y.whole.wrapping_sub(self.inputs().camera_y);
        let x = actor.x.whole.wrapping_sub(self.inputs().camera_x);
        self.calculate_xray_window(x, y, angle, width, x >= 256, 0x9100);
        // Byte-indexed colors can alias the generated table or an imported owner.
        for (target, source) in [(0x74, 0x1914), (0x75, 0x1915), (0x76, 0x1920)] {
            self.set_byte(target, self.byte(source + usize::from(slot)));
        }
    }
}

impl<T: Host + ?Sized> HdmaEffects for T {}

fn hdma_wave_offset(sample: i16, amplitude_low: u8, amplitude_high: u8) -> u16 {
    let amplitude = u16::from_le_bytes([amplitude_low, amplitude_high]);
    let product = u32::from(sample.unsigned_abs()) * u32::from(amplitude);
    let offset = (product >> 16) as u8 as u16;
    if sample < 0 {
        offset.wrapping_neg()
    } else {
        offset
    }
}
