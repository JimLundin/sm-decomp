//! Visual adapters on the one aggregate Session, with explicit legacy effects.
use crate::game::actors::access::{AccessMode::LittleEndian as Le, ActorAccess, ActorKey as Key};
use crate::{
    game::{
        animated_tiles, enemy_projectiles, palette_effects, player_projectiles,
        session_runtime::Runtime,
        visual_host::{self, AnimationMember, PaletteMember, Source, StatueProjectile},
        visual_state::{EyeBeamPhase, PaletteBuffer, Upload},
    },
    game_state::Game,
    rom::RomRecord,
};
impl visual_host::Record for RomRecord {
    fn byte(&self, displacement: usize) -> u8 {
        RomRecord::byte(self, displacement)
    }
    fn native_word(&self, displacement: usize) -> u16 {
        RomRecord::word(self, displacement)
    }
    fn bytes<const N: usize>(&self, displacement: usize) -> [u8; N] {
        let start = self
            .base
            .checked_add(displacement)
            .expect("ROM copy offset overflow");
        let end = start.checked_add(N).expect("ROM copy extent overflow");
        assert!(end <= self.image.len(), "ROM copy source extent");
        core::array::from_fn(|index| self.image.read(start + index))
    }
}
impl visual_host::Content for Runtime<'_> {
    type Record = RomRecord;
    fn record(&mut self, source: Source, bank: u8, address: u16) -> RomRecord {
        match source {
            Source::Loaded => player_projectiles::host::Host::record(self, bank, address),
            Source::Native => player_projectiles::host::Host::native_record(self, bank, address),
        }
    }
    fn legacy_native_record(&mut self, bank: u8, address: u16) -> RomRecord {
        let address = (u32::from(bank) << 16) | u32::from(address);
        if address & 0x8000 == 0 {
            self.foreign(|game| {
                crate::game::session_runtime::invalid_window(game, address);
                #[cfg(sm_verify_dll)]
                crate::room_contact_start_observer::diagnostic(address, game);
            });
        }
        player_projectiles::content::captured(self.native_image(), address)
    }
    fn table(&self, source: Source, address: u32) -> RomRecord {
        let image = match source {
            Source::Loaded => self.loaded_image().expect("ROM must be loaded"),
            Source::Native => self.native_image(),
        };
        player_projectiles::content::captured(image, address)
    }
}
fn palette_address(member: PaletteMember, handle: u16) -> usize {
    let base = match member {
        PaletteMember::Definition => 0x1e7d,
        PaletteMember::ColorOffset => 0x1e8d,
        PaletteMember::PreAction => 0x1ead,
        PaletteMember::Program => 0x1ebd,
        PaletteMember::Delay => 0x1ecd,
        PaletteMember::Counter => 0x1edd,
    };
    base + usize::from(handle & !1)
}
fn animation_address(member: AnimationMember, handle: u16) -> usize {
    let base = match member {
        AnimationMember::Definition => 0x1ef5,
        AnimationMember::Counter => 0x1f01,
        AnimationMember::Program => 0x1f0d,
        AnimationMember::Delay => 0x1f19,
        AnimationMember::Source => 0x1f25,
    };
    base + usize::from(handle & !1)
}
impl visual_host::Aliases for Runtime<'_> {
    fn palette_word(&self, member: PaletteMember, handle: u16) -> u16 {
        self.word(palette_address(member, handle))
    }
    fn set_palette_word_slice(&mut self, member: PaletteMember, handle: u16, value: u16) {
        self.write_slice_le(palette_address(member, handle), value);
    }
    fn set_palette_counter_byte(&mut self, handle: u16, value: u8) {
        self.set_byte(0x1edd + usize::from(handle), value);
    }
    fn palette_flag_native(&self, displacement: u16) -> u16 {
        self.native_word(0x1e79 + usize::from(displacement))
    }
    fn animation_word(&self, member: AnimationMember, handle: u16) -> u16 {
        self.word(animation_address(member, handle))
    }
    fn set_animation_word_prefix(&mut self, member: AnimationMember, handle: u16, value: u16) {
        self.write_word_le(value, animation_address(member, handle));
    }
    fn set_animation_counter_byte(&mut self, handle: u16, value: u8) {
        self.set_byte(0x1f01 + usize::from(handle), value);
    }
    fn set_palette_color_slice(&mut self, displacement: u16, value: u16) {
        self.write_slice_le(0xc000 + usize::from(displacement & !1), value);
    }
    fn set_color_prefix(&mut self, buffer: PaletteBuffer, index: usize, value: u16) {
        let base: usize = match buffer {
            PaletteBuffer::Current => 0xc000,
            PaletteBuffer::Target => 0xc200,
        };
        let at = base
            .checked_add(index.checked_mul(2).expect("palette index overflow"))
            .expect("palette address overflow");
        self.write_word_le(value, at);
    }
}
impl visual_host::Effects for Runtime<'_> {
    fn draw_samus_sprite(&mut self, index: u16, x: u16, y: u16) {
        self.foreign(|game| crate::game::sm_81::DrawSamusSpritemap(index, x, y, game));
    }
    fn draw_atmospheric_tile(&mut self, slot: usize, kind: u16) {
        use visual_host::{Content, Record};
        let frame_offset =
            2 * u16::from(self.session().atmospheric.slots[slot].frame_and_kind as u8);
        let saved = self.word(0x590);
        let x = self.session().atmospheric.slots[slot]
            .x
            .wrapping_sub(self.word(0x911))
            .wrapping_sub(4);
        if x as i16 >= 0
            && self.session().atmospheric.slots[slot]
                .x
                .wrapping_sub(self.word(0x911))
                .wrapping_sub(260)
                & 0x8000
                != 0
        {
            let entry = 0x370 + usize::from(self.word(0x590));
            self.set_byte(entry, x as u8);
            // X may alias the y coordinate, camera, cursor or another owned field.
            let y = self.session().atmospheric.slots[slot]
                .y
                .wrapping_sub(self.word(0x915))
                .wrapping_sub(4);
            if y as i16 >= 0
                && self.session().atmospheric.slots[slot]
                    .y
                    .wrapping_sub(self.word(0x915))
                    .wrapping_sub(260)
                    & 0x8000
                    != 0
            {
                self.set_byte(entry + 1, y as u8);
                let pointer = self
                    .table(Source::Loaded, 0x908bff)
                    .native_word(2 * usize::from(kind));
                let tile = self
                    .legacy_native_record(0x90, pointer.wrapping_add(frame_offset))
                    .native_word(0);
                self.write_slice_native(entry + 2, tile);
                self.write_word_le(saved.wrapping_add(4), 0x590);
                crate::game::session_access::SessionAccess::probe(self, "atmosphere.tile.written");
            }
        }
    }

    fn sound(&mut self, channel: u8, limit: u8, sound: u16) {
        player_projectiles::host::Host::sound(self, channel, limit, sound);
    }
    fn music_delayed8(&mut self, track: u16) {
        self.foreign(|game| crate::game::music::QueueMusic_Delayed8(track, game));
    }
    fn resume_movement_sound(&mut self) {
        #[cfg(sm_verify_dll)]
        samus_observer::event("command", 0x1c);
        self.foreign(|game| {
            crate::game::session_runtime::player_command(0x1c, game);
        });
    }
    fn upload(&mut self, request: Upload) {
        self.foreign(|game| animated_tiles::native::upload(request, game));
    }
    fn invalid_palette_instruction(&mut self) {
        self.foreign(|game| unsafe {
            crate::rtl::Unreachable(game);
        });
    }
    fn invalid_animation_instruction(&mut self) -> u16 {
        self.foreign(animated_tiles::native::unreachable)
    }
    fn color_written(&mut self, _index: u16, _value: u16) {}
}
impl visual_host::Composition for Runtime<'_> {
    fn spawn_palette(&mut self, definition: u16) {
        palette_effects::execution::spawn(self, definition);
    }
    fn spawn_statue_projectile(&mut self, kind: StatueProjectile, parameter: u16) {
        use crate::game::constants::bank_86::{
            addr_kEproj_TourianStatueEyeGlow, addr_kEproj_TourianStatueSoul,
        };
        let definition = match kind {
            StatueProjectile::EyeGlow => addr_kEproj_TourianStatueEyeGlow,
            StatueProjectile::Soul => addr_kEproj_TourianStatueSoul,
        };
        enemy_projectiles::lifecycle::spawn_room(self, definition as u16, parameter);
    }
}
impl visual_host::Host for Runtime<'_> {}

#[forbid(unsafe_code)]
fn queue_cannon_tiles(runtime: &mut Runtime<'_>, tiles: player_projectiles::cannon::Tiles) {
    use player_projectiles::cannon::Tiles;
    use visual_host::Content;
    let (size, destination) = match tiles {
        Tiles::Beam(_) => (256, 0x6300),
        Tiles::Cannon(_) => (32, 0x61f0),
    };
    let saved = runtime.word(0x330);
    runtime.write_slice_native(0xd0 + usize::from(saved), size);
    // Size publication precedes source acquisition and every source read.
    let source = match tiles {
        Tiles::Beam(index) => runtime
            .table(Source::Loaded, 0x90c3b1)
            .word(usize::from(index) * 2),
        Tiles::Cannon(pointer) => runtime.legacy_native_record(0x90, pointer).word(0),
    };
    runtime.write_slice_native(0xd0 + usize::from(saved.wrapping_add(2)), source);
    runtime.set_byte(0xd0 + usize::from(saved.wrapping_add(4)), 0x9a);
    runtime.write_slice_native(0xd0 + usize::from(saved.wrapping_add(5)), destination);
    runtime.write_word_le(saved.wrapping_add(7), 0x330);
}

#[forbid(unsafe_code)]
fn cannon_tile(runtime: &mut Runtime<'_>, x: u16, y: u16, body_y: u16, character: u16) {
    let saved = runtime.word(0x590);
    let x = x
        .wrapping_add(runtime.session().player.x)
        .wrapping_sub(runtime.word(0x911));
    if x < 256 {
        let entry = 0x370 + usize::from(runtime.word(0x590));
        runtime.set_byte(entry, x as u8);
        let y = y
            .wrapping_add(runtime.session().player.y)
            .wrapping_sub(body_y)
            .wrapping_sub(runtime.word(0x915));
        if y < 256 {
            runtime.set_byte(entry + 1, y as u8);
            runtime.write_slice_native(entry + 2, character);
            runtime.write_word_le(saved.wrapping_add(4), 0x590);
        }
    }
}

pub(crate) fn update_arm_cannon(game: &mut Game) {
    player_projectiles::cannon::update(&mut Runtime::new(game));
}
pub(crate) fn draw_player_with_cannon(game: &mut Game) {
    let mut runtime = Runtime::new(game);
    // Layer order is captured before atmospheric output can alias the mode word.
    let mode = runtime.session().projectiles.controls.cannon_drawing_mode & 0xf;
    crate::game::player_transitions::draw_atmospheric(&mut runtime);
    if mode != 0 && mode != 2 {
        player_projectiles::cannon::draw(&mut runtime, cannon_tile, queue_cannon_tiles);
    }
    runtime.foreign(crate::game::samus_animation::Samus_Draw);
    if mode == 2 {
        player_projectiles::cannon::draw(&mut runtime, cannon_tile, queue_cannon_tiles);
    }
}
pub(crate) fn update_beam_tiles_and_palette(game: &mut Game) {
    player_projectiles::cannon::update_beam(&mut Runtime::new(game), queue_cannon_tiles);
}
#[path = "../verification/animated_runtime_checks.rs"]
mod animation_guard_checks;
#[path = "../verification/palette_runtime_checks.rs"]
mod palette_guard_checks;
pub(crate) fn check_palette_guard(game: &mut Game) -> bool {
    palette_guard_checks::run(game)
}
pub(crate) fn check_animation_guard(game: &mut Game) -> bool {
    animation_guard_checks::run(game)
}
#[path = "../verification/samus_palette_observer.rs"]
pub(crate) mod samus_observer;

impl Runtime<'_> {
    fn advance_eye_beam_program(&mut self) {
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

    pub(crate) fn initialize_eye_beam(&mut self, slot: u16) {
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

    pub(crate) fn advance_eye_beam(&mut self, slot: u16, phase: EyeBeamPhase) {
        use visual_host::Content;
        self.write_slice_le(0x1986, 16);
        let even_slot = usize::from(slot & !1);
        match phase {
            EyeBeamPhase::Widening => {
                let speed = self.add_eye_beam_fixed(0x9080, 0x9082, 0x4000);
                let width = self.add_eye_beam_fixed(0x192c + even_slot, 0x1938 + even_slot, speed);
                if ((width >> 16) as u16).wrapping_sub(4) as i16 >= 0 {
                    // Keep the fractional width when the integer part reaches the limit.
                    self.write_slice_le(0x192c + even_slot, 4);
                    self.advance_eye_beam_program();
                }
                self.draw_eye_beam(slot);
            }
            EyeBeamPhase::Full => {
                if self.resolve_actor(Key::from_legacy(0x40), Le).variables[2] == 0 {
                    self.advance_eye_beam_program();
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
                    self.advance_eye_beam_program();
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

    pub(crate) fn draw_eye_beam(&mut self, slot: u16) {
        let actor = self.resolve_actor(Key::from_legacy(0x40), Le);
        let angle = actor.variables[3];
        let width = self.word(0x192c + usize::from(slot & !1));
        let y = actor.y.whole.wrapping_sub(self.inputs().camera_y);
        let x = actor.x.whole.wrapping_sub(self.inputs().camera_x);
        self.foreign(|game| {
            crate::game::beam_windows::calculate_xray_window(
                x,
                y,
                angle,
                width,
                x >= 256,
                0x9100,
                game,
            )
        });
        // Byte-indexed colors can alias the generated table or an imported owner.
        for (target, source) in [(0x74, 0x1914), (0x75, 0x1915), (0x76, 0x1920)] {
            self.set_byte(target, self.byte(source + usize::from(slot)));
        }
    }
}

pub(crate) fn draw_player_frame(game: &mut Game) {
    use crate::game::{player_transitions::state::DrawMode, samus_animation as body};
    update_arm_cannon(game);
    let h = &mut Runtime::new(game);
    match h.session().player.transitions.draw_handler {
        DrawMode::Normal | DrawMode::Grapple => {
            let mode = h.session().player.transitions.draw_handler;
            if mode == DrawMode::Normal {
                h.foreign(player_projectiles::flare);
            }
            let grapple = h.session().player.grapple;
            if mode == DrawMode::Normal || (grapple.wrapping_add(0x37aa) as i16) >= 0 {
                h.foreign(draw_player_with_cannon);
                h.foreign(body::Samus_DrawEchoes);
            } else {
                if matches!(grapple, 0xc703 | 0xc759) {
                    h.foreign(|game| unsafe { crate::game::sm_9b::GrappleBeamFunc_BF1B(game) });
                }
                h.foreign(player_projectiles::grapple_flare);
                h.foreign(draw_player_with_cannon);
                h.foreign(|game| unsafe {
                    crate::game::sm_9b::UpdateGrappleBeamTilesAndIncrFlameCtr(game)
                });
                if h.session().player.grapple_length != 0 {
                    h.foreign(crate::game::grapple_swing::draw_rope);
                }
            }
        }
        DrawMode::ShinesparkEnd => {
            h.foreign(body::Samus_Draw);
            for index in [2, 0] {
                h.foreign(|game| body::Samus_DrawShinesparkCrashEchoes(index, game));
            }
        }
        DrawMode::Frozen => {
            h.foreign(body::Samus_DrawWhenNotAnimatingOrDying);
        }
        DrawMode::Elevator if h.inputs().frame & 1 == 0 => {
            h.foreign(body::Samus_DrawWhenNotAnimatingOrDying);
        }
        DrawMode::Disabled | DrawMode::Empty | DrawMode::Elevator => {}
        DrawMode::Unknown(_) => {
            h.foreign(|game| unsafe { crate::rtl::Unreachable(game) });
        }
    }
    h.foreign(player_projectiles::draw_projectiles);
    h.owned_call(crate::game::session_runtime::finish_player_frame);
}
