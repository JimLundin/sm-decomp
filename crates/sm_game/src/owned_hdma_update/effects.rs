//! Common effect callbacks on the prepared current owner.
//! Descriptor/Phantoon/cinematic aliases remain explicit dependencies below.
#![forbid(unsafe_code)]
use super::context::Context;
use crate::actors::access::{AccessMode::LittleEndian as Le, ActorKey as Key};

use crate::scene::hdma_sources::LiveOwner;
use crate::scene::state::WordOrder;
use crate::visual_host::Source;
use crate::visual_state::{EyeBeamPhase, HazePhase, HdmaWave, Weather};
fn high(word: u16) -> u8 {
    (word >> 8) as u8
}

/// Read each color byte at its original field expression.
fn colors<H: Context>(index: usize, game: &H) -> [u8; 3] {
    [
        crate::hdma_word!(game, index, A) as u8,
        high(crate::hdma_word!(game, index, A)),
        crate::hdma_word!(game, index, B) as u8,
    ]
}
fn set_colors<H: Context>(index: usize, [red, green, blue]: [u8; 3], game: &mut H) {
    crate::hdma_byte!(game, index, A, false => red);
    crate::hdma_byte!(game, index, A, true => green);
    crate::hdma_byte!(game, index, B, false => blue);
}
fn advance_program<H: Context>(index: usize, game: &mut H) {
    crate::hdma_word!(game, index, InstructionList =>
        crate::hdma_word!(&*game, index, InstructionList).wrapping_add(2));
    crate::hdma_word!(game, index, InstructionTimer => 1);
}

pub(super) trait Effects: Context {
    fn current_hdma_index(&self) -> usize {
        usize::from(self.hdma_word(crate::hdma::Word::Selected) >> 1)
    }

    fn initialize_wave_pair(&mut self, base: usize, primary: u16) {
        let source = (0xc00000 | u32::from(primary)).to_ne_bytes();
        self.hdma_slice_le(base, 192);
        self.hdma_slice_le(base + 6, 192);
        for at in [base + 1, base + 7] {
            // Keep high-half, low-word, alternate-pointer publication order.
            self.write_current_slice(at + 2, &source[2..]);
            self.hdma_slice_native(at, primary);
            self.hdma_slice_le(at + 3, primary.wrapping_add(128));
        }
        self.hdma_slice_le(base + 12, 0);
    }

    fn reset_hdma_wave(&mut self, slot: u16) {
        let hdma_index = usize::from(slot >> 1);
        crate::hdma_word!(self, hdma_index, A => 0xfffe);
        crate::hdma_word!(self, hdma_index, B => 1);
        crate::hdma_word!(self, hdma_index, C => 0);
        crate::hdma_word!(self, hdma_index, D => 0);
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
                self.hdma_slice_le(at, 160);
            }
            for at in [0x9001, 0x9007] {
                self.write_current_slice(at + 2, &0xa09100u32.to_ne_bytes()[2..]);
                self.hdma_slice_native(at, 0x9100);
            }
            for at in [0x900d, 0x9013] {
                self.hdma_slice_native(at, 0x9100);
            }
            for at in [0x9004, 0x900a] {
                self.hdma_slice_le(at, 0x9140);
            }
            for at in [0x9010, 0x9016] {
                self.hdma_slice_native(at, 0x9140);
            }
            self.hdma_slice_le(0x9018, 0);
        }
        self.reset_hdma_wave(slot);
    }

    fn select_phantoon_layer(&mut self) {
        use crate::actors::access::AccessMode::Native;
        if self.session().room.fx_layer_flags & 0x4000 != 0 {
            {
                let mut fx = self.fx_controls();
                fx.blending = 26;
                self.publish_fx_controls(fx, &[crate::fx_state::Field::Blending]);
            };
        } else {
            // The native first byte belongs to EnemyData record 3, parameter 1.
            let alpha = self
                .resolve_actor(Key::from_legacy(0xc0), Native)
                .parameters[0]
                .to_ne_bytes()[0];
            match alpha {
                0 => {
                    let mut fx = self.fx_controls();
                    fx.blending = 4;
                    self.publish_fx_controls(fx, &[crate::fx_state::Field::Blending]);
                }
                0xff => {
                    {
                        let mut fx = self.fx_controls();
                        fx.blending = 4;
                        self.publish_fx_controls(fx, &[crate::fx_state::Field::Blending]);
                    };
                    advance_program(self.current_hdma_index(), self);
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
        let hdma_index = usize::from(slot >> 1);
        crate::hdma_word!(self, hdma_index, InstructionTimer => 1);
        crate::hdma_word!(self, hdma_index, InstructionList => program);
    }

    fn advance_hdma_wave(&mut self, slot: u16, wave: HdmaWave) {
        use crate::actors::access::AccessMode::Native;
        use crate::player_projectiles::access::Store;
        use crate::projectile_geometry::SINE_COSINE;
        use crate::ram::words as ram;
        let controller = Key::from_legacy(0x40);
        let motion = Key::from_legacy(0xc0);
        let phantoon = matches!(wave, HdmaWave::Phantoon);
        let mode = if phantoon {
            self.resolve_actor(controller, Native).parameters[0]
        } else {
            self.current_word(0xd9c)
        };
        if mode == 0 {
            self.advance_hdma_program();
            return;
        }
        let step = if phantoon && mode & 1 == 0 { 8 } else { 4 };
        let advance = if phantoon {
            let hdma_index = usize::from(slot >> 1);
            let delay = crate::hdma_word!(self, hdma_index, B);
            crate::hdma_word!(self, hdma_index, B => delay.wrapping_sub(1));
            if delay == 1 {
                crate::hdma_word!(self, hdma_index, B => 1);
            }
            delay == 1
        } else {
            true
        };
        if advance {
            let speed = if phantoon {
                self.resolve_actor(motion, Native).pre_instruction
            } else {
                self.current_word(ram::LOOP_COUNTER_TRANSFER_ENEMIES_TO_VRAM)
            };
            let hdma_index = usize::from(slot >> 1);
            crate::hdma_word!(self, hdma_index, A => speed.wrapping_mul(2).wrapping_add(crate::hdma_word!(self, hdma_index, A)) & 0x1ff);
        }
        let mut phase = crate::hdma_word!(self, usize::from(slot >> 1), A);
        // Cinematic BG3 data is separate from Phantoon's BG2 table.
        let (scroll, store) = if phantoon {
            (crate::display_scroll::Word::Bg2H, Store::SliceNative)
        } else {
            (crate::display_scroll::Word::Bg3H, Store::SliceLe)
        };
        for row in 0..256 / step {
            let sample = SINE_COSINE[usize::from(phase >> 1) + 64];
            let (low, high) = if phantoon {
                let amplitude = self.resolve_actor(motion, Native).variables[3];
                (amplitude as u8, amplitude.to_ne_bytes()[1])
            } else {
                (self.current_byte(0xd9e), self.current_byte(0xd9f))
            };
            let offset = hdma_wave_offset(sample, low, high);
            phase = phase.wrapping_add(step) & 0x1ff;
            let value = self.scroll_word(scroll).wrapping_add(offset);
            if phantoon {
                self.hdma_workspace_word(ram::PHANTOON_WAVES + 2 * usize::from(row), value, store);
            } else {
                self.store_source_word(
                    LiveOwner::HdmaTable1,
                    2 * usize::from(row),
                    WordOrder::Little,
                    value,
                );
            }
        }
        // Phantoon samples its mode again after the first half is published.
        let narrow = phantoon && self.resolve_actor(controller, Native).parameters[0] & 1 == 0;
        let rows = if narrow { 32 } else { 64 };
        for row in (0..rows).rev() {
            let origin = self
                .scroll_word(scroll)
                .wrapping_add(self.scroll_word(scroll));
            let sample = if phantoon {
                self.current_native_word(ram::PHANTOON_WAVES + 2 * row)
            } else {
                self.source_word(LiveOwner::HdmaTable1, 2 * row, WordOrder::Little)
            };
            let value = origin.wrapping_sub(sample);
            if phantoon {
                self.hdma_workspace_word(ram::PHANTOON_WAVES + 2 * (rows + row), value, store);
            } else {
                self.store_source_word(
                    LiveOwner::HdmaTable1,
                    2 * (rows + row),
                    WordOrder::Little,
                    value,
                );
            }
        }
    }

    fn begin_haze(&mut self, slot: u16, color: u16) {
        let hdma_index = usize::from(slot >> 1);
        crate::hdma_word!(self, hdma_index, B => color);
        crate::hdma_word!(self, hdma_index, A => 0);
        if self.session().room.entry.transition_selector == 0xe737 {
            crate::hdma_word!(self, usize::from(slot >> 1), PreInstruction => 0xde2d);
            self.advance_haze(slot, HazePhase::FadingIn);
        }
    }

    fn advance_haze(&mut self, slot: u16, phase: HazePhase) {
        self.hdma_fixed_colors([0x20, 0x40, 0x80]);
        {
            let mut fx = self.fx_controls();
            fx.blending = 44;
            self.publish_fx_controls(fx, &[crate::fx_state::Field::Blending]);
        };
        // A is the haze level and B's low byte its color.
        let level = crate::hdma_word!(self, usize::from(slot >> 1), A);
        match phase {
            HazePhase::FadedIn => {
                if self.session().room.entry.transition_selector == 0xe2db {
                    crate::hdma_word!(self, usize::from(slot >> 1), PreInstruction => 0xde96);
                }
                return;
            }
            HazePhase::FadingIn if level == 16 => {
                crate::hdma_word!(self, usize::from(slot >> 1), PreInstruction => 0xde74);
                return;
            }
            HazePhase::FadingOut if level == 0 => return,
            _ => {}
        }
        let color = crate::hdma_word!(self, usize::from(slot >> 1), B) as u8;
        let mut shade = level as u8 as i8;
        for row in (0..16).rev() {
            self.store_source_byte(LiveOwner::HdmaTable2, row, color | shade as u8);
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
        let hdma_index = usize::from(slot >> 1);
        crate::hdma_word!(self, hdma_index, A => crate::hdma_word!(self, hdma_index, A).wrapping_add(delta));
    }

    fn advance_hdma_program(&mut self) {
        advance_program(self.current_hdma_index(), self);
    }

    fn add_eye_beam_fixed(&mut self, whole: usize, fraction: usize, increment: u32) -> u32 {
        let value = (u32::from(self.current_word(fraction))
            | u32::from(self.current_word(whole)) << 16)
            .wrapping_add(increment);
        self.hdma_slice_le(fraction, value as u16);
        self.hdma_slice_le(whole, (value >> 16) as u16);
        value
    }

    fn initialize_eye_beam(&mut self, slot: u16) {
        set_colors(usize::from(slot >> 1), [0x30, 0x50, 0x80], self);
        self.hdma_fixed_colors([0x30, 0x50, 0x80]);
        {
            let mut fx = self.fx_controls();
            fx.blending = 16;
            self.publish_fx_controls(fx, &[crate::fx_state::Field::Blending]);
        };
        // The original stores also clear the byte after the HDMA terminator.
        self.write_current_slice(
            0x9000,
            &[0xe4, 0x00, 0x91, 0xe4, 0xc8, 0x91, 0x98, 0x90, 0x92, 0, 0],
        );
        self.edit_actor(Key::from_legacy(0x40), Le, |actor| actor.variables[2] = 1);
        // C and D are the beam's width, whole and fraction.
        let hdma_index = usize::from(slot >> 1);
        crate::hdma_word!(self, hdma_index, C => 0);
        crate::hdma_word!(self, hdma_index, D => 0);
        self.write_current_slice(0x9080, &[0; 4]);
        self.hdma_slice_le(0x9090, 0);
    }

    fn advance_eye_beam(&mut self, slot: u16, phase: EyeBeamPhase) {
        {
            let mut fx = self.fx_controls();
            fx.blending = 16;
            self.publish_fx_controls(fx, &[crate::fx_state::Field::Blending]);
        };
        match phase {
            EyeBeamPhase::Widening => {
                let speed = self.add_eye_beam_fixed(0x9080, 0x9082, 0x4000);
                let hdma_index = usize::from(slot >> 1);
                let width = (u32::from(crate::hdma_word!(self, hdma_index, D))
                    | u32::from(crate::hdma_word!(self, hdma_index, C)) << 16)
                    .wrapping_add(speed);
                crate::hdma_word!(self, hdma_index, D => width as u16);
                crate::hdma_word!(self, hdma_index, C => (width >> 16) as u16);
                if ((width >> 16) as u16).wrapping_sub(4) as i16 >= 0 {
                    // Keep the fractional width when the integer part reaches the limit.
                    crate::hdma_word!(self, hdma_index, C => 4);
                    self.advance_hdma_program();
                }
                self.draw_eye_beam(slot);
            }
            EyeBeamPhase::Full => {
                if self.resolve_actor(Key::from_legacy(0x40), Le).variables[2] == 0 {
                    self.advance_hdma_program();
                } else {
                    self.draw_eye_beam(slot);
                    let index = usize::from(self.current_word(0x9090).wrapping_mul(4));
                    let table = self.table(Source::Loaded, 0x88ea8b);
                    let cycle = [0, 1, 2].map(|lane| table.byte(index + lane));
                    set_colors(usize::from(slot >> 1), cycle, self);
                    self.hdma_slice_le(0x9090, self.current_word(0x9090).wrapping_add(1) & 0xf);
                }
            }
            EyeBeamPhase::Fading => {
                if high(crate::hdma_word!(self, usize::from(slot >> 1), A)) == 0x40 {
                    self.hdma_fixed_colors([0x20, 0x40, 0x80]);
                    let hdma_index = usize::from(slot >> 1);
                    crate::hdma_word!(self, hdma_index, D => 0);
                    crate::hdma_word!(self, hdma_index, C => 0);
                    for entry in (0x9100..0x9300).step_by(2).rev() {
                        self.hdma_slice_le(entry, 0xff);
                    }
                    self.write_current_slice(0x9000, &[0; 10]);
                    self.advance_hdma_program();
                } else {
                    self.draw_eye_beam(slot);
                    let hdma_index = self.current_hdma_index();
                    let mut faded = colors(hdma_index, &*self);
                    for (value, black) in faded.iter_mut().zip([0x20, 0x40, 0x80]) {
                        if *value != black {
                            *value = value.wrapping_sub(1);
                        }
                    }
                    set_colors(hdma_index, faded, self);
                }
            }
        }
    }

    fn draw_eye_beam(&mut self, slot: u16) {
        let actor = self.resolve_actor(Key::from_legacy(0x40), Le);
        let angle = actor.variables[3];
        let width = crate::hdma_word!(&*self, usize::from(slot >> 1), C);
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
        let colors = colors(usize::from(slot >> 1), &*self);
        self.hdma_fixed_colors(colors);
    }

    /// Scroll BG3 by the weather's phases and the camera, through the session view.
    fn scroll_weather_layers(&mut self, slot: u16, weather: Weather) {
        let spores = matches!(weather, Weather::Spores);
        let rain = matches!(weather, Weather::Rain);
        // Each axis has a phase word (A and B, or C and D for spores) and
        // rain's camera history (E and F). Finish each axis, including the
        // history, before reading the next.
        for (vertical, camera, output) in [
            (true, 1, crate::display_scroll::Word::Bg3V),
            (false, 0, crate::display_scroll::Word::Bg3H),
        ] {
            let hdma_index = usize::from(slot >> 1);
            let (phase, history) = match (spores, vertical) {
                (false, true) => (
                    crate::hdma_word!(self, hdma_index, A),
                    crate::hdma_word!(self, hdma_index, E),
                ),
                (false, false) => (
                    crate::hdma_word!(self, hdma_index, B),
                    crate::hdma_word!(self, hdma_index, F),
                ),
                (true, true) => (
                    crate::hdma_word!(self, hdma_index, C),
                    crate::hdma_word!(self, hdma_index, E),
                ),
                (true, false) => (
                    crate::hdma_word!(self, hdma_index, D),
                    crate::hdma_word!(self, hdma_index, F),
                ),
            };
            let origin = if rain {
                let layer1 = self.session().room.camera.layer1[camera];
                history.wrapping_sub(layer1)
            } else {
                self.session().room.camera.layer1[camera]
            };
            let scroll = origin.wrapping_add(high(phase) as i8 as u16);
            self.set_scroll_word(output, scroll);
            let delta = match (weather, vertical) {
                (Weather::Rain, true) => 0u16.wrapping_sub(1536),
                (_, true) => 0u16.wrapping_sub(64),
                (Weather::Rain, false) => crate::hdma_word!(self, usize::from(slot >> 1), D),
                (Weather::Fog, false) => 80,
                // Spores retain the original idempotent horizontal-phase store.
                (Weather::Spores, false) => 0,
            };
            let camera = self.session().room.camera.layer1[camera];
            let hdma_index = usize::from(slot >> 1);
            let phase_word = match (spores, vertical) {
                (false, true) => crate::hdma::ObjectWord::A,
                (false, false) => crate::hdma::ObjectWord::B,
                (true, true) => crate::hdma::ObjectWord::C,
                (true, false) => crate::hdma::ObjectWord::D,
            };
            self.store_hdma_word(
                crate::hdma::Word::Object {
                    index: hdma_index,
                    field: phase_word,
                },
                phase.wrapping_add(delta),
                WordOrder::Little,
            );
            if rain {
                if vertical {
                    crate::hdma_word!(self, hdma_index, E => camera);
                } else {
                    crate::hdma_word!(self, hdma_index, F => camera);
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

impl<T: Context> Effects for T {}
