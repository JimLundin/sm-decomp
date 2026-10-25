//! Visual adapters on the one aggregate Session, with explicit legacy effects.
use crate::game::actors::access::{AccessMode::LittleEndian as Le, ActorAccess, ActorKey as Key};
use crate::game::projectile_presentation_ram as sprite_output;
use crate::game::sm_8b::ConfigureTitleSequenceGradientHDMA;
use crate::ram_map::checked_word;
use crate::{
    game::{
        animated_tiles, enemy_projectiles, palette_effects, player_projectiles,
        session_runtime::Runtime,
        visual_host::{self, AnimationMember, PaletteMember, Source, StatueProjectile},
        visual_state::{EyeBeamPhase, HazePhase, HdmaWave, PaletteBuffer, Upload, Weather},
    },
    game_state::Game,
    rom::RomRecord,
    types::SpawnHdmaObject_Args,
};
fn hdma_definition(args: &SpawnHdmaObject_Args) -> [u16; 2] {
    [
        u16::from_ne_bytes([args.dma_control, args.target_register]),
        args.instruction_list,
    ]
}

// Hardware-facing entry points require a live, exclusively accessed SNES backend.
pub(crate) unsafe fn spawn_hdma(bank: u8, args: &SpawnHdmaObject_Args, game: &mut Game) -> u16 {
    unsafe {
        let words = hdma_definition(args);
        spawn_hdma_words(bank, |index| words[index], game)
    }
}

pub(crate) unsafe fn spawn_hdma_last(bank: u8, args: &SpawnHdmaObject_Args, game: &mut Game) {
    unsafe {
        let words = hdma_definition(args);
        initialize_hdma(5, bank, |index| words[index], game);
    }
}

pub(crate) unsafe fn spawn_hdma_words(
    bank: u8,
    read_word: impl Fn(usize) -> u16,
    game: &mut Game,
) -> u16 {
    unsafe {
        match (0..6).find(|&slot| checked_word!(game.hdma_object_channels_bitmask[slot]) == 0) {
            Some(slot) => initialize_hdma(slot, bank, read_word, game),
            None => u16::MAX,
        }
    }
}

unsafe fn initialize_hdma(
    slot: usize,
    bank: u8,
    read_word: impl Fn(usize) -> u16,
    game: &mut Game,
) -> u16 {
    checked_word!(game.hdma_object_pre_instructions[slot] = 0x84b8);
    checked_word!(game.hdma_object_pre_instruction_bank[slot] = 0x88);
    // A native definition reads its program here, and control only after publication.
    checked_word!(game.hdma_object_instruction_list_pointers[slot] = read_word(1));
    checked_word!(game.hdma_object_instruction_timers[slot] = 1);
    for field in [0x1908, 0x1914, 0x1920, 0x192c, 0x1938] {
        checked_word!(game, field + 2 * slot; = 0);
    }
    checked_word!(game.hdma_object_channels_bitmask[slot] = 4 << slot);
    let channel = 0x20 + 0x10 * slot as u16;
    checked_word!(game.hdma_object_bank_slot[slot] = (u16::from(bank) << 8) | channel);
    let control = read_word(0);
    unsafe {
        crate::rtl::WriteRegWord(crate::rtl::DMAP0 as u16 + channel, control, game);
    }
    2 * slot as u16
}

pub(crate) fn set_hdma_enabled(enabled: bool, game: &mut Game) {
    let flags = checked_word!(game.hdma_objects_enable_flag);
    checked_word!(game.hdma_objects_enable_flag = (flags & 0x7fff) | (u16::from(enabled) << 15));
}

pub(crate) fn clear_hdma_channels(game: &mut Game) {
    game.g_ram[0x85] = 0;
    word_transfers!(game, descending, 6; le(hdma_object_channels_bitmask, 5) <- value(0));
}

pub(crate) unsafe fn clear_hdma(game: &mut Game) {
    unsafe {
        crate::rtl::register_writes! { game;
            byte(crate::rtl::MDMAEN as u16, 0); byte(crate::rtl::HDMAEN as u16, 0);
        }
    }
    clear_hdma_channels(game);
}

pub(crate) unsafe fn initialize_room_effects(game: &mut Game) {
    use crate::ram_map::word;
    use crate::rtl::{WMADDH, WMADDL, WMADDM};
    word! { game, 0; earthquake_sfx_index, earthquake_sfx_timer }
    if matches!(
        word!(game.room_ptr),
        0x9804 | 0x96ba | 0xb32e | 0xb457 | 0xdd58 | 0xdede
    ) {
        word!(game.earthquake_sfx_timer = 0xffff);
    }
    word!(game.debug_disable_minimap = 0);
    for channel in (0x20..0x80u16).step_by(0x10) {
        for register in [0, 1, 2, 3, 4, 5, 6, 8, 9] {
            let value = if register == 1 { 0x13 } else { 0 };
            unsafe {
                crate::rtl::WriteReg(0x4300 + channel + register, value, game);
            }
        }
    }
    word! {game; fx_y_subpos = 0; fx_y_pos = -1i32 as u16; lava_acid_y_subpos = 0;
    lava_acid_y_pos = -1i32 as u16; }
    word! { game, 0;
        hud_bg3_xpos, hud_bg3_ypos, bg3_xpos, bg3_ypos, irq_enable_mode7, camera_distance_index,
        tourian_entrance_statue_animstate, tourian_entrance_statue_finished, earthquake_timer,
        phantom_related_layer_flag, power_bomb_explosion_status, power_bomb_flag,
        power_bomb_explosion_radius, fx_tilemap_ptr, fx_type,
    }
    word!(game.fx_base_y_subpos = 0x8000);
    word! { game, 0;
        fx_base_y_pos, fx_target_y_pos, fx_y_vel, fx_liquid_options, fx_timer, tide_phase,
        fx_y_suboffset, fx_y_offset,
    }
    word!(game.fx_layer_blending_config_a = 2);
    word! { game, 0;
        reg_BG3HOFS, reg_BG3VOFS, layer2_x_pos, layer2_y_pos, room_loading_irq_handler,
    }
    word_transfers! { game, ascending, 1;
        native(0x602) <- value(0x8800), native(0x605) <- value(0x8800),
        native(0x601) <- value(0x83e1), native(0x604) <- value(0x83e1),
    }
    unsafe {
        crate::rtl::register_writes! { game;
            byte(WMADDL as u16, 0xf0); byte(WMADDM as u16, 0xff);
            byte(WMADDH as u16, 1);
        }
    }
    game.g_ram[0x85] = 0;
    game.g_ram[0x74] = 32;
    game.g_ram[0x75] = 64;
    game.g_ram[0x76] = 0x80;
    game.g_ram[0x57] = 0;
    game.g_ram[0x69] = 19;
    game.g_ram[0x6f] = 0;
    game.g_ram[0x72] = 0;
    game.g_ram[0x5d] = 0;
    game.g_ram[0x5e] = 4;
    game.g_ram[0x59] = 73;
    game.g_ram[0x5a] = 90;
    game.g_ram[0x5b] = 90;
}

pub(crate) unsafe fn update_hdma(game: &mut Game) {
    unsafe {
        crate::game::music::HandleMusicQueue(game);
    }
    if checked_word!(game.time_is_frozen_flag) == 0
        && checked_word!(game.power_bomb_explosion_status) & 0x4000 != 0
    {
        checked_word!(game.power_bomb_explosion_status = 0x8000);
        unsafe {
            spawn_hdma(0x88, &SpawnHdmaObject_Args::new(0x40, 0x28, 0x8ace), game);
            spawn_hdma(0x88, &SpawnHdmaObject_Args::new(0x40, 0x29, 0x8b80), game);
        }
    }
    checked_word!(game.fx_layer_blending_config_c = checked_word!(game.fx_layer_blending_config_a));
    if checked_word!(game.hdma_objects_enable_flag) & 0x8000 != 0 {
        game.g_ram[0x85] = 0;
        let mut slot = 0usize;
        loop {
            checked_word!(game.hdma_object_index = slot as u16);
            let channels = game.g_ram[0x18b4 + slot];
            if channels != 0 {
                game.g_ram[0x85] |= channels;
                unsafe {
                    run_hdma_program(slot as u8, game);
                }
                slot = usize::from(checked_word!(game.hdma_object_index));
            }
            slot += 2;
            if slot == 12 {
                break;
            }
        }
        unsafe {
            crate::game::layer_blending::update(game);
        }
    }
}

unsafe fn run_hdma_program(slot: u8, game: &mut Game) {
    use crate::game::enemy_script::{EnemyScript as Script, bank_base};
    let index = usize::from(slot >> 1);
    let pre_instruction = (u32::from(checked_word!(game.hdma_object_pre_instruction_bank[index]))
        << 16)
        | u32::from(checked_word!(game.hdma_object_pre_instructions[index]));
    unsafe {
        crate::game::sm_88::CallHdmaobjPreInstr(pre_instruction, u16::from(slot), game);
    }
    let timer = checked_word!(game.hdma_object_instruction_timers[index]);
    checked_word!(game.hdma_object_instruction_timers[index] = timer.wrapping_sub(1));
    if timer != 1 {
        return;
    }
    let base = bank_base(game.g_ram[0x18c1 + usize::from(slot)]);
    let image = game.capture_native_rom();
    let program = checked_word!(game.hdma_object_instruction_list_pointers[index]);
    let position = base + program as isize;
    let mut script = Script::at(
        image.clone(),
        usize::try_from(position).expect("HDMA program outside ROM"),
    );
    while script.word(0) & 0x8000 != 0 {
        let opcode = u32::from(script.word(0)) | 0x880000;
        let next =
            unsafe { execute_hdma_instruction(opcode, u16::from(slot), script.advance(2), game) };
        let Some(next) = next.resolve(&image, base) else {
            return;
        };
        script = next;
    }
    checked_word!(game.hdma_object_instruction_timers[index] = script.word(0));
    checked_word!(game.hdma_object_table_pointers[index] = script.word(2));
    checked_word!(
        game.hdma_object_instruction_list_pointers[index] =
            script.advance(4).offset_from(base) as u16
    );
}

unsafe fn execute_hdma_instruction(
    opcode: u32,
    slot: u16,
    script: crate::game::enemy_script::EnemyScript,
    game: &mut Game,
) -> crate::game::enemy_script::EnemyScript {
    use crate::game::enemy_script::{EnemyScript as Script, bank_base};
    let index = usize::from(slot >> 1);
    match opcode {
        8969006 => {}
        8947049 => {
            checked_word!(game.hdma_object_channels_bitmask[index] = 0);
            return Script::Stop;
        }
        8947056 => {
            checked_word!(game.hdma_object_pre_instructions[index] = script.word(0));
            game.g_ram[0x18fc + usize::from(slot)] = script.byte(2);
            return script.advance(3);
        }
        8947076 => checked_word!(game.hdma_object_pre_instructions[index] = 0x858a),
        8947124 => {
            unsafe {
                crate::game::sm_88::CallHdmaobjInstrFunc(script.load24(0), slot, game);
            }
            return script.advance(3);
        }
        8947180 | 8947185 | 8947207 | 8947215 => {
            let relative = matches!(opcode, 8947185 | 8947215);
            if matches!(opcode, 8947207 | 8947215) {
                let timer = checked_word!(game.hdma_object_timers[index]);
                checked_word!(game.hdma_object_timers[index] = timer.wrapping_sub(1));
                if timer == 1 {
                    return script.advance(if relative { 1 } else { 2 });
                }
            }
            return if relative {
                script.advance(script.byte(0) as i8 as isize)
            } else {
                Script::jump(script.word(0))
            };
        }
        8947222 => {
            game.g_ram[0x1908 + usize::from(slot)] = script.byte(0);
            return script.advance(1);
        }
        8947234 | 8947255 | 8947285 | 8947306 => {
            let register = match opcode {
                8947234 => crate::rtl::DMAP0,
                8947255 => crate::rtl::BBAD0,
                8947285 => crate::rtl::A1B0,
                _ => crate::rtl::DAS00,
            } as u16;
            let channel = u16::from(game.g_ram[0x18c0 + 2 * index]);
            unsafe {
                crate::rtl::WriteReg(register + channel, script.byte(0), game);
            }
            return script.advance(1);
        }
        8947276 => {
            checked_word!(game.hdma_object_table_pointers[index] = script.word(0));
            return script.advance(2);
        }
        8947330 => {
            let base = bank_base(game.g_ram[0x18c1 + usize::from(slot)]);
            // Sleep refreshes the bank/source after callbacks, retaining linear position.
            let _binding = game.capture_native_rom_binding();
            checked_word!(
                game.hdma_object_instruction_list_pointers[index] =
                    (script.offset_from(base) - 2) as u16
            );
            return Script::Stop;
        }
        8955500 | 8959182 | 8963175 | 8963458 => checked_word!(game.hdma_object_B[index] = 1),
        8958889 => checked_word!(game.hdma_object_C[index] = 112),
        8968470 | 8968521 => checked_word!(game.hdma_data_table_in_ceres = 9),
        8968577 => {
            checked_word!(
                game.hdma_object_D[index] = crate::rtl::rom_table_word!(
                    game,
                    0x88d992,
                    ((checked_word!(game.random_number) >> 1 & 6) >> 1) as isize
                )
            );
        }
        8969419 => {
            checked_word!(game.hdma_object_C[index] = 0);
            checked_word!(game.hdma_object_A[index] = 0);
            let opened = crate::game::gameplay_flag_runtime::CheckEventHappened(0xa, game) != 0;
            let offset: u16 = if opened { 0xff10 } else { 0 };
            checked_word!(game.hdma_object_B[index] = offset);
            let [low, high] = offset.to_ne_bytes();
            game.g_ram[0x9e00] = low;
            game.g_ram[0x9e01] = high;
            if opened {
                return Script::jump(script.word(0));
            }
            let [low, high] = 1u16.to_ne_bytes();
            game.g_ram[0xcd20] = low;
            game.g_ram[0xcd21] = high;
            return script.advance(2);
        }
        8971453 => Runtime::new(game).initialize_phantoon_wave(slot),
        0x88e917 => Runtime::new(game).initialize_eye_beam(slot),
        8973471 => Runtime::new(game).reset_hdma_wave(slot),
        8973215 => {
            game.g_ram[0x6f] = 0;
            unsafe {
                ConfigureTitleSequenceGradientHDMA(game);
            }
        }
        _ => {
            unsafe {
                crate::rtl::Unreachable(game);
            }
            return Script::Stop;
        }
    }
    script
}

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
        self.foreign(|game| sprite_output::samus(index, (x, y), game, &mut sprite_output::Native));
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

pub(crate) fn advance_title_gradient(backdrop: bool, game: &mut Game) {
    if backdrop {
        unsafe {
            ConfigureTitleSequenceGradientHDMA(game);
        }
    }
    if checked_word!(game.cinematic_function) == 0x9b68 {
        Runtime::new(game).advance_hdma_program();
    }
}

pub(crate) fn advance_intro_crossfade(game: &mut Game) {
    if checked_word!(game.eproj_x_pos[0]) == 1 {
        Runtime::new(game).advance_hdma_program();
    }
}

pub(crate) unsafe fn initialize_weather(weather: Weather, game: &mut Game) {
    use crate::game::constants::bank_87::{addr_kAnimtiles_Rain, addr_kAnimtiles_Spores};
    let (control, program, tiles) = match weather {
        Weather::Rain => (0x43, 0xd96c, Some(addr_kAnimtiles_Rain)),
        Weather::Spores => (0x42, 0xda2d, Some(addr_kAnimtiles_Spores)),
        Weather::Fog => (0x43, 0xdb19, None),
    };
    game.g_ram[0x5b] = 92;
    unsafe {
        spawn_hdma(
            0x88,
            &SpawnHdmaObject_Args::new(control, 0x11, program),
            game,
        );
        if let Some(tiles) = tiles {
            crate::game::sm_88::SpawnBG3ScrollHdmaObject(game);
            animated_tiles::spawn(tiles as u16, game);
        }
    }
}

pub(crate) fn scroll_weather(slot: u16, weather: Weather, game: &mut Game) {
    use crate::ram_map::{checked_word, words as ram};
    game.g_ram[0x5b] = 92;
    checked_word!(game.fx_layer_blending_config_c = checked_word!(game.fx_layer_blending_config_b));
    if checked_word!(game.time_is_frozen_flag) != 0 {
        return;
    }
    let runtime = &mut Runtime::new(game);
    let object = usize::from(slot as u8 & !1);
    let phase_base = if matches!(weather, Weather::Spores) {
        0x192c
    } else {
        0x1914
    };
    let rain = matches!(weather, Weather::Rain);
    // Finish each axis, including rain's camera history, before reading the next.
    for (axis, camera, output) in [
        (0, ram::layer1_y_pos, ram::bg3_ypos),
        (12, ram::layer1_x_pos, ram::bg3_xpos),
    ] {
        let phase = phase_base + object + axis;
        let history = 0x1944 + object + axis;
        let origin = if rain {
            runtime.word(history).wrapping_sub(runtime.word(camera))
        } else {
            runtime.word(camera)
        };
        let scroll = origin.wrapping_add(runtime.byte(phase + 1) as i8 as u16);
        runtime.write_slice_le(output, scroll);
        let previous_phase = runtime.word(phase);
        let delta = match (weather, axis) {
            (Weather::Rain, 0) => 0u16.wrapping_sub(1536),
            (_, 0) => 0u16.wrapping_sub(64),
            (Weather::Rain, _) => runtime.word(0x1938 + object),
            (Weather::Fog, _) => 80,
            // Spores retain the original idempotent horizontal-phase store.
            (Weather::Spores, _) => 0,
        };
        runtime.write_slice_le(phase, previous_phase.wrapping_add(delta));
        if rain {
            runtime.write_slice_le(history, runtime.word(camera));
        }
    }
}

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

impl Runtime<'_> {
    pub(crate) fn initialize_wave_pair(&mut self, base: usize, primary: u16) {
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

    pub(crate) fn reset_hdma_wave(&mut self, slot: u16) {
        let object = usize::from(slot & !1);
        for (field, value) in [(0x1914, 0xfffe), (0x1920, 1), (0x192c, 0), (0x1938, 0)] {
            self.write_slice_le(field + object, value);
        }
    }

    pub(crate) fn initialize_phantoon_wave(&mut self, slot: u16) {
        use crate::game::actors::access::AccessMode::Native;
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

    pub(crate) fn select_draygon_layer(&mut self, slot: u16) {
        use crate::game::actors::access::AccessMode::Native;
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

    pub(crate) fn advance_hdma_wave(&mut self, slot: u16, wave: HdmaWave) {
        use crate::game::actors::access::AccessMode::Native;
        use crate::game::player_projectiles::access::Store;
        use crate::game::projectile_geometry::SINE_COSINE;
        use crate::ram_map::words as ram;
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
        let (scroll, store) = if phantoon {
            (ram::reg_BG2HOFS, Store::SliceNative)
        } else {
            (ram::reg_BG3HOFS, Store::SliceLe)
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
            self.store(0x9100 + 2 * usize::from(row), value, store);
        }
        // Phantoon samples its mode again after the first half is published.
        let narrow = phantoon && self.resolve_actor(controller, Native).parameters[0] & 1 == 0;
        let rows = if narrow { 32 } else { 64 };
        for row in (0..rows).rev() {
            let origin = self.word(scroll).wrapping_add(self.word(scroll));
            let sample = if phantoon {
                self.native_word(0x9100 + 2 * row)
            } else {
                self.word(0x9100 + 2 * row)
            };
            let value = origin.wrapping_sub(sample);
            self.store(0x9100 + 2 * (rows + row), value, store);
        }
    }

    pub(crate) fn begin_haze(&mut self, slot: u16, color: u16) {
        let object = usize::from(slot as u8 & !1);
        self.write_slice_le(0x1920 + object, color);
        self.write_slice_le(0x1914 + object, 0);
        if self.session().room.entry.transition_selector == 0xe737 {
            self.write_slice_le(0x18f0 + object, 0xde2d);
            self.advance_haze(slot, HazePhase::FadingIn);
        }
    }

    pub(crate) fn advance_haze(&mut self, slot: u16, phase: HazePhase) {
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
