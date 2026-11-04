//! HDMA wave, layer, haze and eye-beam effects over the host's session view.
use crate::actors::access::{AccessMode::LittleEndian as Le, ActorAccess, ActorKey as Key};
use crate::runtime_access::SessionRam;
use crate::scene::hdma_sources::LiveOwner;
use crate::scene::state::WordOrder;
use crate::session_access::SessionAccess;
use crate::visual_host::{Content, Record, Source};
use crate::visual_state::{EyeBeamPhase, HazePhase, HdmaWave, Weather};

/// The session view these effects read and write: the lease's live RAM and
/// Session projection, plus the eye beam's window table outside it.
pub trait Host:
    crate::fx_domain::Controls
    + crate::fx_domain::Sources
    + crate::display::preparation::Access
    + SessionRam
    + SessionAccess
    + ActorAccess
    + Content
    + crate::display_scroll::Access
    + crate::hdma_objects::HdmaTable
{
    /// Build the eye beam's window table outside the session view; returns
    /// the $0D1E word the view stores
    /// (see [`crate::beam_windows::calculate_xray_window`]).
    fn calculate_xray_window(&mut self, window: crate::beam_windows::XrayWindow) -> Option<u16>;
}

fn high(word: u16) -> u8 {
    (word >> 8) as u8
}

/// Phantoon's wave tables and the eye beam's tables and counters live in the
/// bosses' shared RAM ($7E:9000..$7E:97DB): the Session's fourth extended
/// actor block (`actors.extended.blocks[3]`), whose words hold the RAM's bytes
/// little-endian, as the actor codec projects them. Offsets below are bytes
/// from $7E:9000.
const SHARED_BLOCK: usize = 3;
/// $7E:9100: Phantoon's BG2 wave rows and the eye beam's window rows.
const WAVE_ROWS: usize = 0x100;
/// $7E:9080/$7E:9082: the eye beam's widening speed, whole and fraction.
const BEAM_SPEED: usize = 0x80;
const BEAM_SPEED_FRACTION: usize = 0x82;
/// $7E:9090: the eye beam's colour cycle index.
const BEAM_COLOR_INDEX: usize = 0x90;

/// Store `bytes` from shared byte `offset` on, in order, then refresh the
/// host's observations as the projected byte stores did.
fn store_shared_bytes<H: SessionAccess + ?Sized>(h: &mut H, offset: usize, bytes: &[u8]) {
    let block = &mut h.session_mut().actors.extended.blocks[SHARED_BLOCK];
    for (at, &byte) in (offset..).zip(bytes) {
        crate::actors::slot_view::store_byte(&mut block[at / 2], at % 2, byte);
    }
    h.words_stored();
}
/// Shared word at even byte `offset`.
fn shared_word<H: SessionAccess + ?Sized>(h: &H, offset: usize) -> u16 {
    h.session().actors.extended.blocks[SHARED_BLOCK][offset / 2]
}
/// Store shared word at even byte `offset`, then refresh the observations.
fn store_shared_word<H: SessionAccess + ?Sized>(h: &mut H, offset: usize, value: u16) {
    h.session_mut().actors.extended.blocks[SHARED_BLOCK][offset / 2] = value;
    h.words_stored();
}

/// Read each color byte at its original field expression.
fn colors<H: crate::hdma_objects::HdmaTable + ?Sized>(index: usize, game: &H) -> [u8; 3] {
    [
        crate::hdma_word!(game, index, A) as u8,
        high(crate::hdma_word!(game, index, A)),
        crate::hdma_word!(game, index, B) as u8,
    ]
}
fn set_colors<H: crate::hdma_objects::HdmaTable + ?Sized>(
    index: usize,
    [red, green, blue]: [u8; 3],
    game: &mut H,
) {
    crate::hdma_byte!(game, index, A, false => red);
    crate::hdma_byte!(game, index, A, true => green);
    crate::hdma_byte!(game, index, B, false => blue);
}
fn advance_program<H: crate::hdma_objects::HdmaTable + ?Sized>(index: usize, game: &mut H) {
    crate::hdma_word!(game, index, InstructionList =>
        crate::hdma_word!(&*game, index, InstructionList).wrapping_add(2));
    crate::hdma_word!(game, index, InstructionTimer => 1);
}

/// A wave pair's indirect table ($7E:9000 or $7E:0D82, 14 bytes): two entries
/// of a 192-line count and a long pointer, 6 bytes apart, the second half's
/// pointers and the terminator. Built store by store in the original's
/// publication order (high half, low word, alternate pointer), each store
/// overwriting the previous one's bytes.
fn wave_pair_table(primary: u16) -> [u8; 14] {
    let mut table = [0; 14];
    let mut store = |at: usize, bytes: &[u8]| table[at..at + bytes.len()].copy_from_slice(bytes);
    let source = (0xc00000 | u32::from(primary)).to_ne_bytes();
    store(0, &192u16.to_le_bytes());
    store(6, &192u16.to_le_bytes());
    for at in [1, 7] {
        store(at + 2, &source[2..]);
        store(at, &primary.to_ne_bytes());
        store(at + 3, &primary.wrapping_add(128).to_le_bytes());
    }
    store(12, &0u16.to_le_bytes());
    table
}

pub trait HdmaEffects: Host {
    fn current_hdma_index(&self) -> usize {
        usize::from(self.hdma_word(crate::hdma::Word::Selected) >> 1)
    }

    fn initialize_wave_pair(&mut self, table: crate::hdma_objects::WaveTable, primary: u16) {
        let bytes = wave_pair_table(primary);
        match table {
            crate::hdma_objects::WaveTable::Phantoon => store_shared_bytes(self, 0, &bytes),
            crate::hdma_objects::WaveTable::Intro => {
                let words = &mut self.session_mut().game_loop.saved_bindings;
                for (word, pair) in words.iter_mut().zip(bytes.as_chunks::<2>().0) {
                    *word = u16::from_le_bytes(*pair);
                }
                self.words_stored();
            }
        }
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
            self.initialize_wave_pair(crate::hdma_objects::WaveTable::Phantoon, 0x9100);
        } else {
            // The narrow wave's indirect table at $7E:9000, store by store.
            for at in [0x00, 0x06, 0x0c, 0x0f, 0x12, 0x15] {
                store_shared_bytes(self, at, &160u16.to_le_bytes());
            }
            for at in [0x01, 0x07] {
                store_shared_bytes(self, at + 2, &0xa09100u32.to_ne_bytes()[2..]);
                store_shared_bytes(self, at, &0x9100u16.to_ne_bytes());
            }
            for at in [0x0d, 0x13] {
                store_shared_bytes(self, at, &0x9100u16.to_ne_bytes());
            }
            for at in [0x04, 0x0a] {
                store_shared_bytes(self, at, &0x9140u16.to_le_bytes());
            }
            for at in [0x10, 0x16] {
                store_shared_bytes(self, at, &0x9140u16.to_ne_bytes());
            }
            store_shared_bytes(self, 0x18, &0u16.to_le_bytes());
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
        use crate::projectile_geometry::SINE_COSINE;
        let controller = Key::from_legacy(0x40);
        let motion = Key::from_legacy(0xc0);
        let phantoon = matches!(wave, HdmaWave::Phantoon);
        let mode = if phantoon {
            self.resolve_actor(controller, Native).parameters[0]
        } else {
            self.session().wavy_samus.enabled
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
                self.session().wavy_samus.speed
            };
            let hdma_index = usize::from(slot >> 1);
            crate::hdma_word!(self, hdma_index, A => speed.wrapping_mul(2).wrapping_add(crate::hdma_word!(self, hdma_index, A)) & 0x1ff);
        }
        let mut phase = crate::hdma_word!(self, usize::from(slot >> 1), A);
        // Cinematic BG3 data is separate from Phantoon's BG2 table.
        let scroll = if phantoon {
            crate::display_scroll::Word::Bg2H
        } else {
            crate::display_scroll::Word::Bg3H
        };
        for row in 0..256 / step {
            let sample = SINE_COSINE[usize::from(phase >> 1) + 64];
            let (low, high) = if phantoon {
                let amplitude = self.resolve_actor(motion, Native).variables[3];
                (amplitude as u8, amplitude.to_ne_bytes()[1])
            } else {
                let [low, high] = self.session().wavy_samus.amplitude.to_le_bytes();
                (low, high)
            };
            let offset = hdma_wave_offset(sample, low, high);
            phase = phase.wrapping_add(step) & 0x1ff;
            let value = self.scroll_word(scroll).wrapping_add(offset);
            if phantoon {
                // Native byte order, as Phantoon's slice store wrote it.
                store_shared_bytes(self, WAVE_ROWS + 2 * usize::from(row), &value.to_ne_bytes());
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
                u16::from_ne_bytes(shared_word(self, WAVE_ROWS + 2 * row).to_le_bytes())
            } else {
                self.source_word(LiveOwner::HdmaTable1, 2 * row, WordOrder::Little)
            };
            let value = origin.wrapping_sub(sample);
            if phantoon {
                store_shared_bytes(self, WAVE_ROWS + 2 * (rows + row), &value.to_ne_bytes());
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
        self.write_slice(0x74, &[0x20, 0x40, 0x80]);
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

    /// Add `increment` to the eye beam's 16.16 widening speed; the new speed.
    fn add_eye_beam_speed(&mut self, increment: u32) -> u32 {
        let value = (u32::from(shared_word(self, BEAM_SPEED_FRACTION))
            | (u32::from(shared_word(self, BEAM_SPEED)) << 16))
            .wrapping_add(increment);
        store_shared_word(self, BEAM_SPEED_FRACTION, value as u16);
        store_shared_word(self, BEAM_SPEED, (value >> 16) as u16);
        value
    }

    fn initialize_eye_beam(&mut self, slot: u16) {
        set_colors(usize::from(slot >> 1), [0x30, 0x50, 0x80], self);
        self.write_slice(0x74, &[0x30, 0x50, 0x80]);
        {
            let mut fx = self.fx_controls();
            fx.blending = 16;
            self.publish_fx_controls(fx, &[crate::fx_state::Field::Blending]);
        };
        // The original stores also clear the byte after the HDMA terminator.
        store_shared_bytes(
            self,
            0,
            &[0xe4, 0x00, 0x91, 0xe4, 0xc8, 0x91, 0x98, 0x90, 0x92, 0, 0],
        );
        self.edit_actor(Key::from_legacy(0x40), Le, |actor| actor.variables[2] = 1);
        // C and D are the beam's width, whole and fraction.
        let hdma_index = usize::from(slot >> 1);
        crate::hdma_word!(self, hdma_index, C => 0);
        crate::hdma_word!(self, hdma_index, D => 0);
        store_shared_bytes(self, BEAM_SPEED, &[0; 4]);
        store_shared_word(self, BEAM_COLOR_INDEX, 0);
    }

    fn advance_eye_beam(&mut self, slot: u16, phase: EyeBeamPhase) {
        {
            let mut fx = self.fx_controls();
            fx.blending = 16;
            self.publish_fx_controls(fx, &[crate::fx_state::Field::Blending]);
        };
        match phase {
            EyeBeamPhase::Widening => {
                let speed = self.add_eye_beam_speed(0x4000);
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
                    let index = usize::from(shared_word(self, BEAM_COLOR_INDEX).wrapping_mul(4));
                    let table = self.table(Source::Loaded, 0x88ea8b);
                    let cycle = [0, 1, 2].map(|lane| table.byte(index + lane));
                    set_colors(usize::from(slot >> 1), cycle, self);
                    let next = shared_word(self, BEAM_COLOR_INDEX).wrapping_add(1) & 0xf;
                    store_shared_word(self, BEAM_COLOR_INDEX, next);
                }
            }
            EyeBeamPhase::Fading => {
                if high(crate::hdma_word!(self, usize::from(slot >> 1), A)) == 0x40 {
                    self.write_slice(0x74, &[0x20, 0x40, 0x80]);
                    let hdma_index = usize::from(slot >> 1);
                    crate::hdma_word!(self, hdma_index, D => 0);
                    crate::hdma_word!(self, hdma_index, C => 0);
                    // Close the beam's window rows, last first.
                    let block = &mut self.session_mut().actors.extended.blocks[SHARED_BLOCK];
                    for row in block[WAVE_ROWS / 2..WAVE_ROWS / 2 + 256].iter_mut().rev() {
                        *row = 0xff;
                    }
                    self.words_stored();
                    store_shared_bytes(self, 0, &[0; 10]);
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
        let crossing = self.calculate_xray_window(crate::beam_windows::XrayWindow {
            x,
            y,
            angle,
            width,
            off_screen: x >= 256,
            table: 0x9100,
        });
        if let Some(row) = crossing {
            self.session_mut().player.grapple_extension_compatibility.x = row;
        }
        let colors = colors(usize::from(slot >> 1), &*self);
        for (target, color) in (0x74..).zip(colors) {
            self.set_byte(target, color);
        }
    }

    /// Scroll BG3 by the weather's phases and the camera, through the session view.
    fn scroll_weather_layers(&mut self, slot: u16, weather: Weather) {
        use crate::fx_state::Field::{EffectHorizontal, EffectVertical};
        let spores = matches!(weather, Weather::Spores);
        let rain = matches!(weather, Weather::Rain);
        // Each axis has a phase word (A and B, or C and D for spores) and
        // rain's camera history (E and F). Finish each axis, including the
        // history, before reading the next.
        for (vertical, camera, output) in [(true, 1, EffectVertical), (false, 0, EffectHorizontal)]
        {
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
            let mut fx = self.fx_controls();
            *output.get_mut(&mut fx) = scroll;
            self.publish_fx_controls(fx, &[output]);
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

/// Run HDMA object effect `effect` for the object in `slot`, through the
/// owned-Session view.
pub fn apply<H>(effect: crate::hdma_objects::HdmaEffect, slot: u16, h: &mut H)
where
    H: HdmaEffects
        + crate::actors::mother_brain_rainbow::Host
        + crate::player_transitions::effects::SamusLease,
{
    use crate::hdma_objects::HdmaEffect::*;
    use crate::player_transitions::{suit, xray};
    match effect {
        InitializeRainbowBeam => crate::actors::mother_brain_rainbow::initialize_hdma(h),
        AdvanceRainbowBeam => crate::actors::mother_brain_rainbow::advance_hdma(h),
        AdvanceProgram => h.advance_hdma_program(),
        InitializePhantoonWave => h.initialize_phantoon_wave(slot),
        InitializeEyeBeam => h.initialize_eye_beam(slot),
        ResetWave => h.reset_hdma_wave(slot),
        InitializeWavePair { table, primary } => h.initialize_wave_pair(table, primary),
        SetPhantoonMotion { direction, speed } => h.set_phantoon_motion(direction, speed),
        ScrollWeather(weather) => h.scroll_weather_layers(slot, weather),
        BeginHaze { color } => h.begin_haze(slot, color),
        AdvanceHaze(phase) => h.advance_haze(slot, phase),
        SelectDraygonLayer => h.select_draygon_layer(slot),
        SelectPhantoonLayer => h.select_phantoon_layer(),
        AdvanceWave(wave) => h.advance_hdma_wave(slot, wave),
        AdvanceEyeBeam(phase) => h.advance_eye_beam(slot, phase),
        AdvanceXrayBackground => xray::advance_background(h),
        AdvanceXrayBeam(phase) => xray::advance_beam(h, phase),
        InitializeSuitPickup => suit::initialize_hdma(h),
        AdvanceSuitPickup { gravity } => suit::advance_pickup(h, gravity),
    }
}
