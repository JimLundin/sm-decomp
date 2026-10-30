//! HDMA wave, layer, haze and eye-beam effects over the host's session view.
#![forbid(unsafe_code)]
use crate::actors::access::{AccessMode::LittleEndian as Le, ActorAccess, ActorKey as Key};
use crate::hdma::HdmaObject;
use crate::player_projectiles::access::Store;
use crate::runtime_access::SessionRam;
use crate::session_access::SessionAccess;
use crate::visual_host::{Content, Record, Source};
use crate::visual_state::{EyeBeamPhase, HazePhase, HdmaWave, Weather};

/// The session view these effects read and write: the lease's live RAM and
/// Session projection, plus the eye beam's window table outside it.
pub trait Host: SessionRam + SessionAccess + ActorAccess + Content {
    /// Build the eye beam's window table outside the session view.
    fn calculate_xray_window(&mut self, window: crate::beam_windows::XrayWindow);
}

/// Low and high bytes of an object word, as the original's byte stores.
fn set_low(word: &mut u16, value: u8) {
    *word = (*word & 0xff00) | u16::from(value);
}
fn set_high(word: &mut u16, value: u8) {
    *word = (*word & 0x00ff) | u16::from(value) << 8;
}
fn high(word: u16) -> u8 {
    (word >> 8) as u8
}

/// The color bytes the haze and eye beam keep in A's two bytes and B's low byte.
fn colors(object: &HdmaObject) -> [u8; 3] {
    [object.a as u8, high(object.a), object.b as u8]
}
fn set_colors(object: &mut HdmaObject, [red, green, blue]: [u8; 3]) {
    set_low(&mut object.a, red);
    set_high(&mut object.a, green);
    set_low(&mut object.b, blue);
}

/// Step an object's program past its wait.
fn advance_program(object: &mut HdmaObject) {
    object.instruction_list = object.instruction_list.wrapping_add(2);
    object.instruction_timer = 1;
}

pub trait HdmaEffects: Host {
    /// The HDMA object at byte slot `slot`. Every caller passes the
    /// interpreter's even slot below 12, or 0.
    fn hdma_object(&mut self, slot: u16) -> &mut HdmaObject {
        &mut self.session_mut().hdma.objects[usize::from(slot >> 1)]
    }

    /// The object the interpreter is running.
    fn current_hdma_object(&mut self) -> &mut HdmaObject {
        let selected = self.session().hdma.selected;
        self.hdma_object(selected)
    }

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
        let object = self.hdma_object(slot);
        object.a = 0xfffe;
        object.b = 1;
        object.c = 0;
        object.d = 0;
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
        if self.word(ram::PHANTOM_RELATED_LAYER_FLAG) & 0x4000 != 0 {
            self.write_slice_le(ram::FX_LAYER_BLENDING_CONFIG_C, 26);
        } else {
            // The native first byte belongs to EnemyData record 3, parameter 1.
            let alpha = self
                .resolve_actor(Key::from_legacy(0xc0), Native)
                .parameters[0]
                .to_ne_bytes()[0];
            match alpha {
                0 => self.write_slice_le(ram::FX_LAYER_BLENDING_CONFIG_C, 4),
                0xff => {
                    self.write_slice_le(ram::FX_LAYER_BLENDING_CONFIG_C, 4);
                    advance_program(self.current_hdma_object());
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
        let object = self.hdma_object(slot);
        object.instruction_timer = 1;
        object.instruction_list = program;
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
        let advance = if phantoon {
            let object = self.hdma_object(slot);
            let delay = object.b;
            object.b = delay.wrapping_sub(1);
            if delay == 1 {
                object.b = 1;
            }
            delay == 1
        } else {
            true
        };
        if advance {
            let speed = if phantoon {
                self.resolve_actor(motion, Native).pre_instruction
            } else {
                self.word(ram::LOOP_COUNTER_TRANSFER_ENEMIES_TO_VRAM)
            };
            let object = self.hdma_object(slot);
            object.a = speed.wrapping_mul(2).wrapping_add(object.a) & 0x1ff;
        }
        let mut phase = self.hdma_object(slot).a;
        // Cinematic BG3 data is separate from Phantoon's BG2 table.
        let (scroll, store, table) = if phantoon {
            (ram::REG_BG2HOFS, Store::SliceNative, ram::PHANTOON_WAVES)
        } else {
            (ram::REG_BG3HOFS, Store::SliceLe, arrays::HDMA_TABLE_1)
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
            store_word(self, table + 2 * usize::from(row), value, store);
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
            store_word(self, table + 2 * (rows + row), value, store);
        }
    }

    fn begin_haze(&mut self, slot: u16, color: u16) {
        let object = self.hdma_object(slot);
        object.b = color;
        object.a = 0;
        if self.session().room.entry.transition_selector == 0xe737 {
            self.hdma_object(slot).pre_instruction = 0xde2d;
            self.advance_haze(slot, HazePhase::FadingIn);
        }
    }

    fn advance_haze(&mut self, slot: u16, phase: HazePhase) {
        self.write_slice(0x74, &[0x20, 0x40, 0x80]);
        self.write_slice_le(0x1986, 44);
        // A is the haze level and B's low byte its color.
        let level = self.hdma_object(slot).a;
        match phase {
            HazePhase::FadedIn => {
                if self.session().room.entry.transition_selector == 0xe2db {
                    self.hdma_object(slot).pre_instruction = 0xde96;
                }
                return;
            }
            HazePhase::FadingIn if level == 16 => {
                self.hdma_object(slot).pre_instruction = 0xde74;
                return;
            }
            HazePhase::FadingOut if level == 0 => return,
            _ => {}
        }
        let color = self.hdma_object(slot).b as u8;
        let mut shade = level as u8 as i8;
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
        let object = self.hdma_object(slot);
        object.a = object.a.wrapping_add(delta);
    }

    fn advance_hdma_program(&mut self) {
        advance_program(self.current_hdma_object());
    }

    fn add_eye_beam_fixed(&mut self, whole: usize, fraction: usize, increment: u32) -> u32 {
        let value = (u32::from(self.word(fraction)) | u32::from(self.word(whole)) << 16)
            .wrapping_add(increment);
        self.write_slice_le(fraction, value as u16);
        self.write_slice_le(whole, (value >> 16) as u16);
        value
    }

    fn initialize_eye_beam(&mut self, slot: u16) {
        set_colors(self.hdma_object(slot), [0x30, 0x50, 0x80]);
        self.write_slice(0x74, &[0x30, 0x50, 0x80]);
        self.write_slice_le(0x1986, 16);
        // The original stores also clear the byte after the HDMA terminator.
        self.write_slice(
            0x9000,
            &[0xe4, 0x00, 0x91, 0xe4, 0xc8, 0x91, 0x98, 0x90, 0x92, 0, 0],
        );
        self.edit_actor(Key::from_legacy(0x40), Le, |actor| actor.variables[2] = 1);
        // C and D are the beam's width, whole and fraction.
        let object = self.hdma_object(slot);
        object.c = 0;
        object.d = 0;
        self.write_slice(0x9080, &[0; 4]);
        self.write_slice_le(0x9090, 0);
    }

    fn advance_eye_beam(&mut self, slot: u16, phase: EyeBeamPhase) {
        self.write_slice_le(0x1986, 16);
        match phase {
            EyeBeamPhase::Widening => {
                let speed = self.add_eye_beam_fixed(0x9080, 0x9082, 0x4000);
                let object = self.hdma_object(slot);
                let width = (u32::from(object.d) | u32::from(object.c) << 16).wrapping_add(speed);
                object.d = width as u16;
                object.c = (width >> 16) as u16;
                if ((width >> 16) as u16).wrapping_sub(4) as i16 >= 0 {
                    // Keep the fractional width when the integer part reaches the limit.
                    object.c = 4;
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
                    let table = self.table(Source::Loaded, 0x88ea8b);
                    let cycle = [0, 1, 2].map(|lane| table.byte(index + lane));
                    set_colors(self.hdma_object(slot), cycle);
                    self.write_slice_le(0x9090, self.word(0x9090).wrapping_add(1) & 0xf);
                }
            }
            EyeBeamPhase::Fading => {
                if high(self.hdma_object(slot).a) == 0x40 {
                    self.write_slice(0x74, &[0x20, 0x40, 0x80]);
                    let object = self.hdma_object(slot);
                    object.d = 0;
                    object.c = 0;
                    for entry in (0x9100..0x9300).step_by(2).rev() {
                        self.write_slice_le(entry, 0xff);
                    }
                    self.write_slice(0x9000, &[0; 10]);
                    self.advance_hdma_program();
                } else {
                    self.draw_eye_beam(slot);
                    let object = self.current_hdma_object();
                    let mut faded = colors(object);
                    for (value, black) in faded.iter_mut().zip([0x20, 0x40, 0x80]) {
                        if *value != black {
                            *value = value.wrapping_sub(1);
                        }
                    }
                    set_colors(object, faded);
                }
            }
        }
    }

    fn draw_eye_beam(&mut self, slot: u16) {
        let actor = self.resolve_actor(Key::from_legacy(0x40), Le);
        let angle = actor.variables[3];
        let width = self.session().hdma.objects[usize::from(slot >> 1)].c;
        let y = actor.y.whole.wrapping_sub(self.inputs().camera_y);
        let x = actor.x.whole.wrapping_sub(self.inputs().camera_x);
        self.calculate_xray_window(crate::beam_windows::XrayWindow {
            x,
            y,
            angle,
            width,
            off_screen: x >= 256,
            table: 0x9100,
        });
        let colors = colors(self.hdma_object(slot));
        for (target, color) in (0x74..).zip(colors) {
            self.set_byte(target, color);
        }
    }

    /// Scroll BG3 by the weather's phases and the camera, through the session view.
    fn scroll_weather_layers(&mut self, slot: u16, weather: Weather) {
        use crate::ram::words as ram;
        let spores = matches!(weather, Weather::Spores);
        let rain = matches!(weather, Weather::Rain);
        // Each axis has a phase word (A and B, or C and D for spores) and
        // rain's camera history (E and F). Finish each axis, including the
        // history, before reading the next.
        for (vertical, camera, output) in [(true, 1, ram::BG3_YPOS), (false, 0, ram::BG3_XPOS)] {
            let object = *self.hdma_object(slot);
            let (phase, history) = match (spores, vertical) {
                (false, true) => (object.a, object.e),
                (false, false) => (object.b, object.f),
                (true, true) => (object.c, object.e),
                (true, false) => (object.d, object.f),
            };
            let origin = if rain {
                let layer1 = self.session().room.camera.layer1[camera];
                history.wrapping_sub(layer1)
            } else {
                self.session().room.camera.layer1[camera]
            };
            let scroll = origin.wrapping_add(high(phase) as i8 as u16);
            self.write_slice_le(output, scroll);
            let delta = match (weather, vertical) {
                (Weather::Rain, true) => 0u16.wrapping_sub(1536),
                (_, true) => 0u16.wrapping_sub(64),
                (Weather::Rain, false) => self.hdma_object(slot).d,
                (Weather::Fog, false) => 80,
                // Spores retain the original idempotent horizontal-phase store.
                (Weather::Spores, false) => 0,
            };
            let camera = self.session().room.camera.layer1[camera];
            let object = self.hdma_object(slot);
            let phase_word = match (spores, vertical) {
                (false, true) => &mut object.a,
                (false, false) => &mut object.b,
                (true, true) => &mut object.c,
                (true, false) => &mut object.d,
            };
            *phase_word = phase.wrapping_add(delta);
            if rain {
                if vertical {
                    object.e = camera;
                } else {
                    object.f = camera;
                }
            }
        }
    }

    /// Phantoon's motion parameters, set before its wave HDMA spawns.
    fn set_phantoon_motion(&mut self, direction: u16, speed: u16) {
        use crate::actors::access::AccessMode::Native;
        self.edit_actor(Key::from_legacy(0x80), Native, |actor| {
            actor.parameters[0] = direction
        });
        self.edit_actor(Key::from_legacy(0xc0), Native, |actor| {
            actor.variables[3] = 0;
            actor.variables[4] = 0;
            actor.pre_instruction = speed;
        });
    }
}

impl<T: Host + ?Sized> HdmaEffects for T {}

// Keep bits 16..23 of the magnitude product before restoring the sine sign.
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

/// One table word through the projection, in the store's byte order.
fn store_word<H: Host + ?Sized>(h: &mut H, at: usize, value: u16, store: Store) {
    match store {
        Store::PrefixLe => h.write_word_le(value, at),
        Store::SliceLe => h.write_slice_le(at, value),
        Store::SliceNative => h.write_slice_native(at, value),
    }
}
