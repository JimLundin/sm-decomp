//! One aggregate authority for player, projectiles, mechanisms and their room.
use crate::game::player_projectiles::{
    access::{ShotLane, Store},
    content,
    host::{Drawing, Host, PaletteEffect, PlayerStep},
};
use crate::game::{
    external_inputs::ExternalInputs, session::Session, session_encoding as encoding,
};
use crate::{
    game_state::Game,
    rom::{RomImage, RomRecord},
};

pub(crate) fn invalid_window(game: &mut Game, address: u32) {
    unsafe {
        crate::c_library::printf(c"RomPtr - Invalid access 0x%x!\n".as_ptr(), address);
    }
    game.g_fail = true;
}

#[forbid(unsafe_code)]
fn draw_trail(game: &mut Game, pair: usize, right: bool, frozen: bool) {
    let read = encoding::read;
    let saved = read(&game.g_ram, 0x590);
    if (saved as i16).wrapping_sub(512) >= 0 {
        return;
    }
    let (timer, x_field, y_field, tile_field) = if right {
        (0xd67c, 0xd754, 0xd79c, 0xd70c)
    } else {
        (0xd658, 0xd730, 0xd778, 0xd6e8)
    };
    if frozen && read(&game.g_ram, timer + 2 * pair) == 0 {
        return;
    }
    crate::verify_probe::hit(match (frozen, right) {
        (true, false) => "trails.draw.frozen_left",
        (true, true) => "trails.draw.frozen_right",
        (false, false) => "trails.draw.active_left",
        (false, true) => "trails.draw.active_right",
    });
    let x = read(&game.g_ram, x_field + 2 * pair).wrapping_sub(read(&game.g_ram, 0x911));
    if x & 0xff00 == 0 {
        let entry = 0x370 + usize::from(read(&game.g_ram, 0x590));
        game.g_ram[entry..entry + 2].copy_from_slice(&x.to_ne_bytes());
        // Preserve rereads after each overlapping native OAM store.
        let y = read(&game.g_ram, y_field + 2 * pair).wrapping_sub(read(&game.g_ram, 0x915));
        if y & 0xff00 == 0 {
            game.g_ram[entry + 1..entry + 3].copy_from_slice(&y.to_ne_bytes());
            let tile = read(&game.g_ram, tile_field + 2 * pair);
            game.g_ram[entry + 2..entry + 4].copy_from_slice(&tile.to_ne_bytes());
            game.g_ram[0x590..0x592].copy_from_slice(&saved.wrapping_add(4).to_le_bytes());
        }
    }
}

pub(crate) struct Runtime<'g> {
    game: &'g mut Game,
    state: Option<Session>,
    inputs: ExternalInputs,
}
impl<'g> Runtime<'g> {
    pub fn new(game: &'g mut Game) -> Self {
        Self {
            state: Some(encoding::decode(&game.g_ram)),
            inputs: encoding::inputs(&game.g_ram),
            game,
        }
    }
    pub fn foreign<R>(&mut self, action: impl FnOnce(&mut Game) -> R) -> R {
        let state = self
            .state
            .take()
            .expect("projectile state unavailable during external effect");
        encoding::encode(&state, &mut self.game.g_ram);
        drop(state);
        let result = action(self.game);
        self.state = Some(encoding::decode(&self.game.g_ram));
        self.inputs = encoding::inputs(&self.game.g_ram);
        result
    }
    pub(crate) fn byte(&self, at: usize) -> u8 {
        encoding::projected_byte(self.session(), at).unwrap_or_else(|| self.game.g_ram[at])
    }
    pub(crate) fn word(&self, at: usize) -> u16 {
        u16::from_le_bytes([self.byte(at), self.byte(at + 1)])
    }
    pub(crate) fn store(&mut self, at: usize, value: u16, store: Store) {
        match store {
            Store::PrefixLe => self.write_word_le(value, at),
            Store::SliceLe => self.write_slice_le(at, value),
            Store::SliceNative => self.write_slice_native(at, value),
        }
    }
    pub(crate) fn session(&self) -> &Session {
        self.state
            .as_ref()
            .expect("aggregate unavailable during external effect")
    }
    pub(crate) fn session_mut(&mut self) -> &mut Session {
        self.state
            .as_mut()
            .expect("aggregate unavailable during external effect")
    }
    pub(crate) fn inputs(&self) -> &ExternalInputs {
        &self.inputs
    }
    pub(crate) fn native_word(&self, at: usize) -> u16 {
        u16::from_ne_bytes([self.byte(at), self.byte(at + 1)])
    }
    pub(crate) fn set_byte(&mut self, at: usize, value: u8) {
        // Ordinary RAM and owned fields are one live projection during an interval.
        // A foreign destination may alias an external observation used next.
        if !encoding::write_projected_byte(
            self.state.as_mut().expect("aggregate unavailable"),
            at,
            value,
        ) {
            self.game.g_ram[at] = value;
        }
        self.inputs = encoding::inputs(&self.game.g_ram);
    }
    pub(crate) fn write_slice(&mut self, at: usize, bytes: &[u8]) {
        let end = at.checked_add(bytes.len()).expect("aggregate write extent");
        let _ = &self.game.g_ram[at..end];
        for (lane, &value) in bytes.iter().enumerate() {
            self.set_byte(at + lane, value);
        }
    }
    pub(crate) fn write_word_le(&mut self, value: u16, at: usize) {
        let [low, high] = value.to_le_bytes();
        self.set_byte(at, low);
        self.set_byte(at + 1, high);
    }
    pub(crate) fn write_slice_le(&mut self, at: usize, value: u16) {
        self.write_slice(at, &value.to_le_bytes());
    }
    pub(crate) fn write_slice_native(&mut self, at: usize, value: u16) {
        self.write_slice(at, &value.to_ne_bytes());
    }
    pub(crate) fn loaded_image(&self) -> Option<RomImage> {
        self.game.loaded_rom.clone()
    }
    pub(crate) fn native_image(&self) -> RomImage {
        self.game.capture_native_rom()
    }
}
impl Drop for Runtime<'_> {
    fn drop(&mut self) {
        if let Some(state) = self.state.take() {
            encoding::encode(&state, &mut self.game.g_ram);
        }
    }
}
fn shot_address(lane: ShotLane, raw: u16) -> usize {
    let base = match lane {
        ShotLane::X => 0xb64,
        ShotLane::Y => 0xb78,
        ShotLane::XFraction => 0xb8c,
        ShotLane::YFraction => 0xba0,
        ShotLane::XRadius => 0xbb4,
        ShotLane::YRadius => 0xbc8,
        ShotLane::XSpeed => 0xbdc,
        ShotLane::YSpeed => 0xbf0,
        ShotLane::Direction => 0xc04,
        ShotLane::Kind => 0xc18,
        ShotLane::Damage => 0xc2c,
        ShotLane::Cursor => 0xc40,
        ShotLane::Remaining => 0xc54,
        ShotLane::Motion => 0xc68,
        ShotLane::Variable => 0xc7c,
        ShotLane::TrailTime => 0xc90,
        ShotLane::Auxiliary => 0xca4,
        ShotLane::Sprite => 0xcb8,
    };
    base + usize::from(raw & !1)
}
impl Host for Runtime<'_> {
    fn probe(&self, name: &'static str) {
        crate::verify_probe::hit(name);
    }
    fn session(&self) -> &Session {
        self.state
            .as_ref()
            .expect("projectile state unavailable during external effect")
    }
    fn session_mut(&mut self) -> &mut Session {
        self.state
            .as_mut()
            .expect("projectile state unavailable during external effect")
    }
    fn inputs(&self) -> &ExternalInputs {
        &self.inputs
    }
    fn loaded_image(&self) -> Option<RomImage> {
        self.game.loaded_rom.clone()
    }
    fn native_image(&self) -> RomImage {
        self.game.capture_native_rom()
    }
    fn record(&mut self, bank: u8, cursor: u16) -> RomRecord {
        if cursor & 0x8000 == 0 {
            self.foreign(|g| unsafe { crate::rtl::rom_record(bank, cursor, g) })
        } else {
            content::captured(
                content::image(self),
                (u32::from(bank) << 16) | u32::from(cursor),
            )
        }
    }
    fn native_record(&mut self, bank: u8, cursor: u16) -> RomRecord {
        let address = (u32::from(bank) << 16) | u32::from(cursor);
        if cursor & 0x8000 == 0 {
            self.foreign(|g| invalid_window(g, address));
        }
        content::captured(self.game.capture_native_rom(), address)
    }
    fn sound(&mut self, bank: u8, limit: u8, sound: u16) {
        self.foreign(|g| {
            use crate::game::sm_80::*;
            match (bank, limit) {
                (1, 6) => QueueSfx1_Max6(sound, g),
                (1, 9) => QueueSfx1_Max9(sound, g),
                (1, 15) => QueueSfx1_Max15(sound, g),
                (2, 6) => QueueSfx2_Max6(sound, g),
                (3, 6) => QueueSfx3_Max6(sound, g),
                (3, 15) => QueueSfx3_Max15(sound, g),
                _ => unreachable!("unknown projectile sound queue"),
            }
        });
    }
    fn player_step(&mut self, step: PlayerStep) {
        self.foreign(|g| unsafe {
            use crate::game::{sm_9b, sm_90, sm_91, sm_94};
            match step {
                PlayerStep::Radius => sm_90::Samus_SetRadius(g),
                PlayerStep::Input => sm_90::Samus_CallInputHandler(g),
                PlayerStep::SuitIndex => sm_90::Samus_UpdateSuitPaletteIndex(g),
                PlayerStep::Acceleration => sm_90::Samus_DetermineAccel_Y(g),
                PlayerStep::InsideBlock => sm_94::BlockInsideDetection(g),
                PlayerStep::Grapple => sm_9b::GrappleBeamHandler(g),
                PlayerStep::Xray => sm_91::XrayRunHandler(g),
                PlayerStep::RestoreControl => {
                    sm_90::CallSomeSamusCode(0x14, g);
                }
            }
        });
    }
    fn palette(&mut self, effect: PaletteEffect) -> u8 {
        self.foreign(|g| match effect {
            PaletteEffect::Suit => {
                unsafe { crate::game::sm_91::Samus_LoadSuitPalette(g) };
                0
            }
            PaletteEffect::SuitTarget => {
                unsafe { crate::game::sm_91::Samus_LoadSuitTargetPalette(g) };
                0
            }
            PaletteEffect::Spawn(definition) => {
                crate::game::palette_effects::spawn(definition, g);
                0
            }
            PaletteEffect::Charge(output) => charge_palette(g, output),
        })
    }
    fn enable_hdma(&mut self) {
        self.foreign(|g| unsafe { crate::game::sm_88::EnableHdmaObjects(g) });
    }
    fn spawn_power_bomb(&mut self) {
        self.foreign(|g| unsafe { crate::game::explosion_window::spawn_power_bomb(g) });
    }
    fn spawn_mechanism(&mut self, header: u16) {
        let mut contact = crate::game::room_mechanisms::host::Contact::Absent;
        crate::game::room_mechanisms::creation::spawn(
            self,
            crate::game::room_mechanisms::host::SpawnSource::Collision { header },
            &mut contact,
        );
    }
    fn draw(&mut self, drawing: Drawing) {
        self.foreign(|g| {
            use crate::game::projectile_presentation_ram as output;
            match drawing {
                Drawing::Projectile { slot, x, y } => {
                    output::projectile(slot, (x, y), g, &mut output::Native);
                    crate::verify_probe::hit("player_projectiles.draw.completed");
                }
                Drawing::Beam { sprite, x, y } => unsafe {
                    crate::game::sm_81::DrawBeamGrappleSpritemap(sprite, x, y, g)
                },
                Drawing::CrashEchoes => output::crash_echoes(g, &mut output::Native),
            }
        });
    }
    fn draw_trail(&mut self, pair: usize, right: bool, frozen: bool) {
        self.foreign(|g| draw_trail(g, pair, right, frozen));
    }
    fn invalid_damage(&mut self) {
        self.foreign(|g| unsafe { crate::game::sm_80::InvalidInterrupt_Crash(g) });
    }
    fn unknown(&mut self) -> bool {
        self.foreign(|g| unsafe { crate::rtl::Unreachable(g) })
    }
    fn room_level(&self, cell: u16) -> u16 {
        self.word(0x10002 + usize::from(cell) * 2)
    }
    fn room_bts(&self, cell: u16) -> u8 {
        self.byte(0x16402 + usize::from(cell))
    }
    fn alias_shot_read(&self, lane: ShotLane, raw: u16) -> u16 {
        self.word(shot_address(lane, raw))
    }
    fn alias_shot_write(&mut self, lane: ShotLane, raw: u16, value: u16, store: Store) {
        self.store(shot_address(lane, raw), value, store);
    }
    fn variable_byte(&self, raw: u16, high: bool) -> u8 {
        self.byte(0xc7c + usize::from(raw) + usize::from(high))
    }
    fn echo_x(&self, index: usize) -> u16 {
        self.word(0xab0 + index * 2)
    }
    fn set_echo_x(&mut self, index: usize, value: u16) {
        self.store(0xab0 + index * 2, value, Store::PrefixLe);
    }
    fn set_echo_y(&mut self, index: usize, value: u16) {
        self.store(0xab8 + index * 2, value, Store::PrefixLe);
    }
    fn set_echo_index_native(&mut self, raw: u16, value: u16) {
        self.store(0xaae + usize::from(raw), value, Store::SliceNative);
    }
}

// This is the sole charge-palette output edge formerly in the singleton
// beam_charge_palette_ram adapter. Its source allocation is recaptured at the
// original reads, including after the low-window diagnostic.
fn charge_palette(game: &mut Game, output: crate::game::beam_charge_palette::Output) -> u8 {
    use crate::game::beam_charge_palette::Output;
    let image = |g: &Game| g.loaded_rom.as_ref().expect("ROM must be loaded").clone();
    let palette = match output {
        Output::Hyper(phase) => image(game).table_word(content::offset(0x91d829), phase as isize),
        Output::Charge {
            contact_attack,
            suit,
            phase,
        } => {
            let base = image(game).table_word(
                content::offset(if contact_attack { 0x91d7ff } else { 0x91d7d5 }),
                (suit >> 1) as isize,
            );
            let address = 0x910000 | u32::from(base.wrapping_add(phase));
            if address & 0x8000 == 0 {
                invalid_window(game, address);
            }
            image(game).table_word(content::offset(address), 0)
        }
        Output::Highlight => {
            for i in 0..15 {
                let at = 0xc19e - 2 * i;
                game.g_ram[at..at + 2].copy_from_slice(&1023u16.to_le_bytes());
            }
            return 0;
        }
        Output::Visor => return unsafe { crate::game::sm_91::HandleVisorPalette(game) },
    };
    unsafe {
        crate::game::sm_91::CopyToSamusSuitPalette(palette, game);
    }
    0
}

/// Shared continuation belongs to the calling context, never to Session encoding.
pub(crate) fn run_mechanisms(
    game: &mut Game,
    context: crate::game::room_mechanisms::host::CallerContext,
) -> u8 {
    use crate::game::room_mechanisms::host::{CallerContext, Pass, RunResult};
    let mut pass = match game.g_ram[0x613] {
        0 => Pass::Instructions,
        1 => Pass::DisplayMessage,
        other => Pass::Invalid(other),
    };
    let result = {
        let mut runtime = Runtime::new(game);
        crate::game::room_mechanisms::scheduler::advance(&mut runtime, &mut pass)
    };
    match context {
        CallerContext::Gameplay | CallerContext::DoorTransition => match result {
            RunResult::Complete => {
                game.g_ram[0x613] = 0;
                0
            }
            RunResult::Yield(value) => {
                game.g_ram[0x613] = 1;
                value.get()
            }
        },
    }
}
