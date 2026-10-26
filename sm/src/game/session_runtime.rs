//! One aggregate authority for player, projectiles, mechanisms and their room.
use crate::game::player_projectiles::{
    access::{ShotLane, Store},
    content,
    host::{Drawing, Host, PaletteEffect, PlayerStep},
};
use crate::game::session_access::SessionAccess;
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

pub(crate) fn native_rom_diagnostic(address: u32, game: &mut Game) {
    invalid_window(game, address);
    #[cfg(sm_verify_dll)]
    crate::room_contact_start_observer::diagnostic(address, game);
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

#[cfg(sm_verify_dll)]
#[path = "../verification/actor_movement_runtime_observer.rs"]
mod actor_movement_runtime_observer;

pub(crate) struct Runtime<'g> {
    game: &'g mut Game,
    state: Option<Session>,
    inputs: ExternalInputs,
}
impl<'g> Runtime<'g> {
    pub fn new(game: &'g mut Game) -> Self {
        #[cfg(sm_verify_dll)]
        player_normal_transfers::record(game, 0);
        Self {
            state: Some(import_session(game)),
            inputs: encoding::inputs(&game.g_ram),
            game,
        }
    }
    pub fn foreign<R>(&mut self, action: impl FnOnce(&mut Game) -> R) -> R {
        preflight_export(self.session(), self.game);
        let state = self
            .state
            .take()
            .expect("projectile state unavailable during external effect");
        export_session(state, self.game);
        let result = action(self.game);
        self.state = Some(import_session(self.game));
        self.inputs = encoding::inputs(&self.game.g_ram);
        result
    }
    /// Compose an already-owned operation while retaining the former foreign
    /// entry's preflight, unwind publication, and refreshed input observations.
    pub(crate) fn owned_call<R>(&mut self, action: impl FnOnce(&mut Self) -> R) -> R {
        preflight_export(self.session(), self.game);
        struct OwnedCall<'a, 'g> {
            runtime: &'a mut Runtime<'g>,
            completed: bool,
        }
        impl Drop for OwnedCall<'_, '_> {
            fn drop(&mut self) {
                if !self.completed {
                    self.runtime.relinquish();
                }
            }
        }
        let mut call = OwnedCall {
            runtime: self,
            completed: false,
        };
        let result = action(&mut *call.runtime);
        call.runtime.inputs.enemy_zero_health = call.runtime.native_word(0xF8C);
        call.runtime.inputs.current_enemy = call.runtime.word(0xE54);
        call.runtime.inputs.random_enemy_counter = call.runtime.word(0xE44);
        call.runtime.inputs.debug_enemy_freeze = call.runtime.word(0x185E);
        call.completed = true;
        result
    }
    fn relinquish(&mut self) {
        if let Some(state) = self.state.take() {
            let matching = std::sync::Arc::ptr_eq(&state.world, self.game.world())
                && state.world.revision() == self.game.world().revision();
            if matching && self.game.actor_identity_is_with_session() {
                export_session(state, self.game);
            } else if self.game.actor_identity_is_with_session() {
                // A rejected foreign call retains this Runtime's identity owner;
                // discard invalid byte/native publication rather than panic again.
                self.game.return_actor_identity(state.actor_identity);
            }
        }
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
    // Verification observes relinquishment before Drop can mask a retained owner.
    #[cfg(sm_verify_dll)]
    pub(crate) fn has_session_for_verification(&self) -> bool {
        self.state.is_some()
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
        // Preserve observation refresh timing while A5A is owned by Session.
        self.inputs.transitions.frame_gamma = self.session().player.transitions.frame_gamma;
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
        self.relinquish();
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
            if !matches!(
                (bank, limit),
                (1, 6 | 9 | 15) | (2, 6) | (3, 3 | 6 | 9 | 15)
            ) {
                unreachable!("unknown projectile sound queue");
            }
            crate::game::music::queue_sfx(usize::from(bank - 1), sound as u8, limit, g);
        });
    }
    fn player_step(&mut self, step: PlayerStep) {
        if matches!(step, PlayerStep::SuitIndex) {
            crate::game::samus_palette::update_suit_index(self.session_mut());
            return;
        }
        if matches!(step, PlayerStep::Acceleration) {
            self.owned_call(|runtime| {
                crate::game::player_movement::speed::update_vertical_acceleration(runtime);
            });
            return;
        }
        if matches!(step, PlayerStep::InsideBlock) {
            self.owned_call(|runtime| {
                crate::game::player_collision::operations::sample_inside(runtime);
            });
            return;
        }
        if matches!(step, PlayerStep::Xray) {
            self.owned_call(Self::request_xray);
            return;
        }
        if matches!(step, PlayerStep::Input) {
            self.owned_call(player_transitions_runtime::dispatch_input);
            return;
        }
        self.foreign(|g| unsafe {
            use crate::game::sm_9b;
            match step {
                PlayerStep::Radius => set_player_radius(g),
                PlayerStep::Input => unreachable!("input handled by shared owner"),
                PlayerStep::SuitIndex => unreachable!("suit index handled by shared owner"),
                PlayerStep::Acceleration => unreachable!("acceleration handled by shared owner"),
                PlayerStep::InsideBlock => unreachable!("inside block handled by shared owner"),
                PlayerStep::Grapple => sm_9b::GrappleBeamHandler(g),
                PlayerStep::Xray => unreachable!("X-ray handled by shared owner"),
                PlayerStep::RestoreControl => {
                    crate::game::session_runtime::player_command(0x14, g);
                }
            }
        });
    }
    fn palette(&mut self, effect: PaletteEffect) -> u8 {
        use crate::game::{samus_palette, visual_state::PaletteBuffer};
        match effect {
            PaletteEffect::Suit => {
                samus_palette::load_suit(self, PaletteBuffer::Current);
                0
            }
            PaletteEffect::SuitTarget => {
                samus_palette::load_suit(self, PaletteBuffer::Target);
                0
            }
            PaletteEffect::Spawn(definition) => {
                crate::game::palette_effects::execution::spawn(self, definition);
                0
            }
        }
    }
    fn enable_hdma(&mut self) {
        self.foreign(|g| crate::game::visual_runtime::set_hdma_enabled(true, g));
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
                Drawing::Beam { sprite, x, y } => output::beam(sprite, (x, y), g),
                Drawing::CrashEchoes => output::crash_echoes(g, &mut output::Native),
                Drawing::GrappleSegment { x, y, tile } => output::grapple(g, Some((x, y, tile))),
                Drawing::GrappleEnd => output::grapple(g, None),
            }
        });
    }
    fn draw_trail(&mut self, pair: usize, right: bool, frozen: bool) {
        self.foreign(|g| draw_trail(g, pair, right, frozen));
    }
    fn invalid_damage(&mut self) {
        self.foreign(|g| crate::rtl::invalid_interrupt(g));
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

/// Capture the caller's pass before importing Session. Runtime publishes its
/// shared continuation bytes on return; this caller then publishes its local
/// result in the original order. Do not refresh the local pass after callbacks.
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

pub(crate) fn import_session(game: &mut Game) -> Session {
    #[cfg(sm_verify_dll)]
    player_normal_transfers::record(game, 1);
    let mut state = encoding::decode(&game.g_ram, game.world().clone());
    import_native_spawn(&mut state, game);
    state.actor_identity = game.take_actor_identity();
    state
}
fn preflight_export(state: &Session, game: &Game) {
    assert!(
        std::sync::Arc::ptr_eq(&state.world, game.world())
            && state.world.revision() == game.world().revision(),
        "Session belongs to a different world binding"
    );
    assert!(
        game.actor_identity_is_with_session(),
        "Game already owns actor identity"
    );
}
/// Holds identity until it is returned, including an unwind after partial encoding.
struct ExportIdentity<'g> {
    state: Option<Session>,
    game: &'g mut Game,
}
impl Drop for ExportIdentity<'_> {
    fn drop(&mut self) {
        if let Some(state) = self.state.take() {
            self.game.return_actor_identity(state.actor_identity);
        }
    }
}
pub(crate) fn export_session(state: Session, game: &mut Game) {
    preflight_export(&state, game);
    let guard = ExportIdentity {
        state: Some(state),
        game,
    };
    #[cfg(sm_verify_dll)]
    player_normal_transfers::record(guard.game, 2);
    let state = guard.state.as_ref().expect("export owns Session");
    encoding::encode(state, &mut guard.game.g_ram);
    export_native_spawn(state, guard.game);
    // Drop moves identity exactly once, after the byte/native publication.
}
#[forbid(unsafe_code)]
pub(crate) fn import_native_spawn(state: &mut Session, game: &Game) {
    let spawn = &mut state.enemy_projectiles.spawn;
    spawn.rectangle.origin.x = game.eproj_spawn_rect.x;
    spawn.rectangle.origin.y = game.eproj_spawn_rect.y;
    spawn.rectangle.extent.x = game.eproj_spawn_rect.w;
    spawn.rectangle.extent.y = game.eproj_spawn_rect.h;
    spawn.variant = game.eproj_spawn_r22;
    spawn.point.x = game.eproj_spawn_pt.x;
    spawn.point.y = game.eproj_spawn_pt.y;
    spawn.drop_definition = game.eproj_spawn_varE24;
}
#[forbid(unsafe_code)]
pub(crate) fn export_native_spawn(state: &Session, game: &mut Game) {
    let spawn = &state.enemy_projectiles.spawn;
    game.eproj_spawn_rect.x = spawn.rectangle.origin.x;
    game.eproj_spawn_rect.y = spawn.rectangle.origin.y;
    game.eproj_spawn_rect.w = spawn.rectangle.extent.x;
    game.eproj_spawn_rect.h = spawn.rectangle.extent.y;
    game.eproj_spawn_r22 = spawn.variant;
    game.eproj_spawn_pt.x = spawn.point.x;
    game.eproj_spawn_pt.y = spawn.point.y;
    game.eproj_spawn_varE24 = spawn.drop_definition;
}
impl SessionAccess for Runtime<'_> {
    fn session(&self) -> &Session {
        Runtime::session(self)
    }
    fn session_mut(&mut self) -> &mut Session {
        Runtime::session_mut(self)
    }
    fn inputs(&self) -> &ExternalInputs {
        Runtime::inputs(self)
    }
    fn probe(&self, name: &'static str) {
        #[cfg(sm_verify_dll)]
        crate::game::visual_runtime::samus_observer::observe(name, self.session(), self.game);
        crate::verify_probe::hit(name);
    }
}

impl Runtime<'_> {
    /// Preserve native-pointer diagnostics before capturing the registered allocation.
    pub(crate) fn legacy_native_record(&mut self, bank: u8, address: u16) -> RomRecord {
        let address = (u32::from(bank) << 16) | u32::from(address);
        if address & 0x8000 == 0 {
            self.foreign(|game| {
                native_rom_diagnostic(address, game);
            });
        }
        content::captured(self.native_image(), address)
    }

    /// Preserve the actor content diagnostic observer and allocation capture order.
    pub(crate) fn actor_legacy_record(&mut self, bank: u8, address: u16) -> RomRecord {
        if address & 0x8000 == 0 {
            self.foreign(|game| unsafe { crate::rtl::rom_record(bank, address, game) })
        } else {
            content::captured(
                content::image(self),
                (u32::from(bank) << 16) | u32::from(address),
            )
        }
    }
}

use crate::game::actors::api as actor_api;
use crate::game::combat_rules::LatchMode;
use crate::game::enemy_script::EnemyScript;
use crate::game::moving_platforms::{self as mechanisms, Event, Kind};
use crate::game::{
    sm_a2, sm_a3, sm_a4, sm_a5, sm_a6, sm_a7, sm_a8, sm_a9, sm_aa, sm_b3, space_pirates,
};
crate::game::address_dispatch! {
    unsafe fn actor_species_draw_hook(ea: u32, game: &mut Game) => ea;
        call (game); default { crate::rtl::Unreachable(game); };
    {
        10518604 => [()]; 10738444 => sm_a3::Reflec_Func_1; 10851138 => sm_a5::Draygon_Func_36;
        10920690 => sm_a6::Ridley_A2F2; 11042892 => [()]; 11055282 => sm_a8::NorfairLavaMan_Func_6;
        11062375 => crate::game::actors::wrecked_ship_robots::palette; 11069550 => sm_a8::BlueBrinstarFaceBlock_Func_1;
        11110278 => [()]; 11110345 => sm_a9::MotherBrain_DrawBrainNeck_EnemyGfxDrawHook;
        11110365 => sm_a9::MotherBrainsBrain_GfxDrawHook; 11129754 => sm_a9::DeadTorizo_MainGfxHook;
    }
}

crate::game::callback_dispatch! {
    unsafe fn actor_species_ai(ea: u32, game: &mut Game) => ea;
    call (game); default { crate::rtl::Unreachable(game); };
    @external {
        10715136 | 10846208 | 10911744 | 10977280 | 11042816 | 11763712 => [actor_api::resume_ai(&mut Runtime::new(game))];
        10649605 | 10780677 => [actor_api::latch(&mut Runtime::new(game), LatchMode::Passive)];
        11042836 => [actor_api::latch(&mut Runtime::new(game), LatchMode::Active)];
        10518583 | 10649655 | 10715191 | 10780727 | 10846263 | 10911799 | 10977335 | 11042871 | 11108407 | 11173943 | 11698231 | 11763767 => [actor_api::power_bomb_reaction(&mut Runtime::new(game), actor_api::DeathHandling::Finalize)];
        10649676 => [{
            #[cfg(sm_verify_dll)]
            crate::game::actors::reference_population_animation::dispatch_noop(game);
        }];
        10656319 => crate::game::hopping_blobs::initialize;
        10656381 => sm_a2::ThinHoppingBlobs_Main;
        10664233 => [{
                    crate::game::rinka::frozen(crate::ram_map::word!(game.cur_enemy_index), game);
                }];
        10664275 => [{
                    crate::game::rinka::power_bomb(crate::ram_map::word!(game.cur_enemy_index), game);
                }];
        10669268 => sm_a2::MaridiaLargeSnail_Init;
        10676698 => [mechanisms::initialize(Kind::TimedShutter, game)];
        10676918 => [mechanisms::update(Kind::TimedShutter, game)];
        10677765 => [mechanisms::initialize(Kind::Platform, game)];
        10677778 => [mechanisms::initialize(Kind::VerticalShutter, game)];
        10677969 => [mechanisms::update(Kind::Platform, game)];
        10678429 => [mechanisms::react(Kind::Platform, Event::Touch, game)];
        10678434 => [mechanisms::react(Kind::Platform, Event::Shot, game)];
        10678442 => [mechanisms::react(Kind::VerticalShutter, Event::Shot, game)];
        10678454 => [mechanisms::react(Kind::Platform, Event::PowerBomb, game)];
        10678545 => [mechanisms::initialize(Kind::HorizontalShutter, game)];
        10678750 => [mechanisms::update(Kind::HorizontalShutter, game)];
        10679256 => [mechanisms::react(Kind::HorizontalShutter, Event::Touch, game)];
        10679310 => [mechanisms::react(Kind::HorizontalShutter, Event::Shot, game)];
        10679322 => [mechanisms::react(Kind::HorizontalShutter, Event::PowerBomb, game)];
        10715212 => [()]; 10716909 => sm_a3::Waver_Init;
        10717004 => sm_a3::Waver_Main; 10717536 => sm_a3::Metalee_Init;
        10717561 => sm_a3::Metalee_Main; 10717967 => sm_a3::Metalee_Shot;
        10718827 => [{
                    sm_a3::Fireflea_Touch(crate::ram_map::word!(game.cur_enemy_index), game);
                }];
        10719413 => sm_a3::MaridiaFish_Init; 10719531 => sm_a3::MaridiaFish_Main;
        10723079 => [()];
        10723661 => sm_a3::Roach_Init;
        10724048 => sm_a3::Roach_Main;
        10726351 => sm_a3::Sidehopper_Main;
        10733230 => sm_a3::Skree_Init; 10733255 => sm_a3::Skree_Main;
        10738715 => [()];
        10743874 => [{
                    sm_a3::Metroid_Powerbomb(crate::ram_map::word!(game.cur_enemy_index), game);
                }];
        10780748 => [()];
        10795344 => [()];
        10811067 => [sm_a4::CrocomireTongue_Main()];
        10846284 => [()];
        10851824 => sm_a5::Draygon_Shot;
        10864042 => [()];
        10864068 => [()];
        10874354 => [()];
        10911820 => [()];
        10913525 => sm_a6::Boulder_Init; 10913683 => sm_a6::Boulder_Main;
        10914607 => crate::game::spikey_platforms::initialize; 10914733 => crate::game::spikey_platforms::run;
        10920181 => sm_a6::CeresRidley_Init; 10920584 => sm_a6::CeresRidley_Main;
        10924583 => sm_a6::Ridley_Main;
        10924682 => sm_a6::Ridley_Func_2; 10924695 => sm_a6::Ridley_Hurt;
        10942752 => [()];
        10977356 => [()]; 10982543 => [()];
        10982581 => [()];
        11001242 => [()];
        11001873 => [()];
        11042892 => [()];
        11044832 => crate::game::actors::evir::initialize;
        11045040 => crate::game::actors::evir::initialize_projectile;
        11045147 => crate::game::actors::evir::advance;
        11045278 => crate::game::actors::evir::advance_projectile;
        11045638 | 11045644 | 11045650 => [crate::game::actors::evir::damage(ea, game)];
        11048675 => crate::game::actors::fune::initialize;
        11048752 => crate::game::actors::fune::advance;
        11049710 => crate::game::actors::wrecked_ship_ghost::initialize;
        11049788 => crate::game::actors::wrecked_ship_ghost::advance;
        11051336 => sm_a8::YappingMaw_Init;
        11051537 => sm_a8::YappingMaw_Main; 11052953 => sm_a8::YappingMaw_Touch;
        11053894 => crate::game::actors::kago::initialize;
        11053941 => crate::game::actors::kago::advance; 11053955 => crate::game::actors::kago::shot;
        11057014 => crate::game::actors::beetom::initialize;
        11057165 => crate::game::actors::beetom::advance; 11058734 => crate::game::actors::beetom::touch;
        11058860 => crate::game::actors::beetom::shot;
        11059657 => crate::game::actors::maridia_floater::initialize;
        11059740 => [{ let address = crate::game::actors::maridia_floater::state_address(game); actor_species_pre_instruction(address, game); }];
        11060670 => crate::game::actors::maridia_floater::touch;
        11060719 => crate::game::actors::maridia_floater::shot;
        11060799 => crate::game::actors::maridia_floater::power_bomb;
        11062135 => crate::game::actors::wrecked_ship_robots::initialize;
        11062220 => crate::game::actors::wrecked_ship_robots::initialize_dormant;
        11062326 => crate::game::actors::wrecked_ship_robots::advance;
        11063668 => crate::game::actors::wrecked_ship_robots::touch_dormant;
        11063693 => crate::game::actors::wrecked_ship_robots::shot_dormant;
        11063698 => crate::game::actors::wrecked_ship_robots::shot;
        11062374 => [()];
        11065545 => crate::game::actors::maridia_puffer::initialize;
        11065611 => crate::game::actors::maridia_puffer::advance; 11066132 => crate::game::actors::maridia_puffer::shot;
        11066573 => crate::game::actors::lava_seahorse::initialize;
        11066731 => crate::game::actors::lava_seahorse::advance;
        11068296 => sm_a8::WreckedShipOrbs_Init;
        11071880 => crate::game::actors::ki_hunter::initialize;
        11072020 => crate::game::actors::ki_hunter::initialize_wings; 11072092 => crate::game::actors::ki_hunter::advance;
        11072098 => crate::game::actors::ki_hunter::advance;
        11073281 => crate::game::actors::ki_hunter::shot;
        11108428 => [()];
        11111301 => [{
                    sm_a9::MotherBrainsTubesFalling_Main(crate::ram_map::word!(game.cur_enemy_index), game);
                }];
        11121927 => sm_a9::MotherBrainsBrain_Shot;
        11122117 => [()];
        11173964 => [()];
        11196359 => [()];
        11200475 => [()];
        11200476 => [()];
        11698252 => [()];
        11763788 => [()];
        11765518 => [sm_b3::UnusedSpinningTurtleEye_Main()];
        11767628 => sm_b3::BrinstarYellowPipeBug_Init;
        11767726 => sm_b3::BrinstarYellowPipeBug_Main; 11769219 => sm_b3::Botwoon_Init;
        11771926 => sm_b3::Botwoon_Shot;
        11791130 => [()];
        0xa20000..=0xa2ffff => [sm_a2::DispatchEnemyAi(ea, game)];
        0xa30000..=0xa3ffff => [sm_a3::DispatchEnemyAi(ea, game)];
        0xa40000..=0xa4ffff => [sm_a4::DispatchEnemyAi(ea, game)];
        0xa50000..=0xa5ffff => [sm_a5::DispatchEnemyAi(ea, game)];
        0xa60000..=0xa6ffff => [sm_a6::DispatchEnemyAi(ea, game)];
        0xa70000..=0xa7ffff => [sm_a7::DispatchEnemyAi(ea, game)];
        0xa80000..=0xa8ffff => [sm_a8::DispatchEnemyAi(ea, game)];
        0xa90000..=0xa9ffff => [sm_a9::DispatchEnemyAi(ea, game)];
        0xaa0000..=0xaaffff => [sm_aa::DispatchEnemyAi(ea, game)];
        0xb20000..=0xb2ffff => [space_pirates::dispatch_ai(ea, game)];
        0xb30000..=0xb3ffff => [sm_b3::DispatchEnemyAi(ea, game)];
    }
}
crate::game::address_dispatch! {
    pub(crate) unsafe fn actor_species_pre_instruction(ea: u32, game: &mut Game) => ea;
        setup { let k: u16 = crate::ram_map::word!(game.cur_enemy_index); }
        call (k, game); default { crate::rtl::Unreachable(game); };
    {
        10518651 => [()]; 10649723 => [()];
        10652277 => [()];
        10657819 => [()]; 10659799 => [()];
        10676097 => [()]; 10730689 => [sm_a3::Bang_Func_7(k)];
        10735455 => [()];
        10739850 => [()]; 10741441 => [()];
        10998952 => [()];
        11047388 => [()];
        11055761 => [sm_a8::sub_A8B291(k)]; 11055765 => [sm_a8::NorfairLavaMan_Func_13(game)];
        11059843 | 11059878 | 11059919 | 11060075 | 11060193 | 11060329 | 11060444 | 11060480 | 11060509 | 11060584 | 11060585 | 11060639 => [crate::game::actors::maridia_floater::advance(ea as u16, k, game)];
        11192670 => [()]; 11197610 => [()]; 11200422 => [()]; 11174011 => [()]; 11763835 => [()];
        11192146 => sm_aa::Torizo_Func_5;
        11192364 => sm_aa::Torizo_Func_7;
        11195889 => sm_aa::Torizo_D5F1;
        11200474 => [()];
        11197611 => [()];
        0xa20000..=0xa2ffff => [sm_a2::DispatchEnemyPreInstr(ea, k, game)];
        0xa30000..=0xa3ffff => [sm_a3::DispatchEnemyPreInstr(ea, k, game)];
        0xa70000..=0xa7ffff => [sm_a7::DispatchEnemyPreInstr(ea, k, game)];
        0xa80000..=0xa8ffff => [sm_a8::DispatchEnemyPreInstr(ea, k, game)];
        0xaa0000..=0xaaffff => [sm_aa::DispatchEnemyPreInstr(ea, k, game)];
        0xb30000..=0xb3ffff => [sm_b3::DispatchEnemyPreInstr(ea, k, game)];
    }
}
crate::game::address_dispatch! {
    unsafe fn actor_species_touch(ea: u32, game: &mut Game) => ea;
        call (game); default { crate::rtl::Unreachable(game); };
    {
        10518563 => [actor_api::contact_reaction(&mut Runtime::new(game), actor_api::DeathHandling::Finalize)]; 10670984 => sm_a2::MaridiaLargeSnail_Func_12;
        10670988 => sm_a2::MaridiaLargeSnail_Touch; 10780707 => sm_a4::Enemy_NormalTouchAI_A4;
        10795325 => sm_a4::Crocomire_Func_92; 10795344 => [()]; 10846284 => [()];
        10851818 => sm_a5::Draygon_Touch; 10874348 => sm_a5::SporeSpawn_Touch;
        10936153 => sm_a6::sub_A6DF59; 10940479 => crate::game::ceres_doors::steam_touch; 10977356 => [()];
        10982539 => sm_a7::Kraid_Touch_ArmFoot; 10982544 => sm_a7::KraidsArm_Touch;
        11001237 => sm_a7::Phantoon_Touch; 11122117 => [()];
        11122118 => sm_a9::MotherBrainsBrain_Touch; 11192695 => sm_aa::GoldTorizo_Touch;
        11700076 => space_pirates::touch;
    }
}
crate::game::callback_dispatch! {
    unsafe fn actor_species_shot(ea: u32, j: u16, game: &mut Game) => ea;
    call (game); default { crate::rtl::Unreachable(game); };
    @external {
        10518573 => [actor_api::shot_reaction(&mut Runtime::new(game), actor_api::ShotHandling::Finalize)];
        10846278 => [actor_api::create_dud(&mut Runtime::new(game))];
        10649676 => [()];
        10671028 => sm_a2::MaridiaLargeSnail_Shot; 10780717 => sm_a4::Enemy_NormalShotAI_A4;
        10795345 => sm_a4::Crocomire_Func_93; 10795368 => sm_a4::Crocomire_Func_94;
        10795525 => sm_a4::Crocomire_Func_95; 10795700 => sm_a4::Crocomire_Func_1;
        10846253 => sm_a5::Enemy_NormalShotAI_A5; 10846284 => [()];
        10851824 => sm_a5::Draygon_Shot; 10874202 => sm_a5::SporeSpawn_Shot; 10911820 => [()];
        10936202 => sm_a6::Ridley_Shot; 10977356 => [()]; 10982581 => [()];
        10982582 => [sm_a7::Kraid_Arm_Shot(j, game)]; 11001243 => sm_a7::Phantoon_Shot;
        11121923 => sm_a9::MotherBrainsBody_Shot; 11121927 => sm_a9::MotherBrainsBrain_Shot;
        11192700 => sm_aa::Torizo_Shot; 11192769 => [()]; 11192770 => sm_aa::Torizo_Func_8;
        11700089 => space_pirates::shot; 11700168 => space_pirates::check_shot;
        11700286 => space_pirates::reflect;
    }
}

crate::game::callback_dispatch! {
    unsafe fn actor_species_instruction(ea: u32, k: u16, j: EnemyScript, game: &mut Game) -> EnemyScript => ea;
    call (k, j, game); default { crate::rtl::Unreachable(game); EnemyScript::Stop };
    @external {

        10651845 => [j];
        11044751 | 11044763 | 11044790 | 11044811 => [crate::game::actors::evir::instruction(ea, j, game)];
        10716899 => sm_a3::Waver_Instr_1;
        10991155 => [sm_a7::Kraid_Instr_1(k, j)];
        11048485 | 11048497 | 11048522 | 11048547 | 11048572 | 11048597 | 11048628 => [crate::game::actors::fune::instruction(ea, j, game)];
        11056990 => [j];
        11062537 | 11062692 | 11062762 | 11062917 | 11062987 | 11062991 | 11063146 | 11063216 | 11063371 | 11063441 | 11063490 | 11063494 | 11063506 | 11063552 | 11063559 | 11063601 | 11063613 | 11063659 => [crate::game::actors::wrecked_ship_robots::instruction(ea, k, j, game)];
        11067164 | 11067187 | 11067193 | 11067199 | 11067235 | 11067249 => [crate::game::actors::lava_seahorse::instruction(ea, j, game)];
        11072806 | 11072996 | 11073151 | 11073234 | 11073240 => [crate::game::actors::ki_hunter::instruction(ea, k, j, game)];
        11128756 => [sm_a9::Shitroid_Instr_1(k, j)];
        11128778 => [sm_a9::Shitroid_Instr_2(k, j)];
        11139360 => [sm_a9::Shitroid_Instr_3(k, j)]; 11139382 => [sm_a9::Shitroid_Instr_4(k, j)];
        11139472 => [sm_a9::Shitroid_Instr_6(k, j)];
        11190984 => [sm_aa::Torizo_Instr_7(k, j)];
        11196831 => [sm_aa::Shaktool_Instr_6(k, j)];
        0xa20000..=0xa2ffff => [sm_a2::DispatchEnemyInstr(ea, k, j, game)];
        0xa30000..=0xa3ffff => [sm_a3::DispatchEnemyInstr(ea, k, j, game)];
        0xa40000..=0xa4ffff => [sm_a4::DispatchEnemyInstr(ea, k, j, game)];
        0xa50000..=0xa5ffff => [sm_a5::DispatchEnemyInstr(ea, k, j, game)];
        0xa60000..=0xa6ffff => [sm_a6::DispatchEnemyInstr(ea, k, j, game)];
        0xa70000..=0xa7ffff => [sm_a7::DispatchEnemyInstr(ea, k, j, game)];
        0xa80000..=0xa8ffff => [sm_a8::DispatchEnemyInstr(ea, k, j, game)];
        0xa90000..=0xa9ffff => [sm_a9::DispatchEnemyInstr(ea, k, j, game)];
        0xaa0000..=0xaaffff => [sm_aa::DispatchEnemyInstr(ea, k, j, game)];
        0xb20000..=0xb2ffff => [space_pirates::dispatch_instruction(ea, k, j, game)];
        0xb30000..=0xb3ffff => [sm_b3::DispatchEnemyInstr(ea, k, j, game)];
    }
}
impl Runtime<'_> {
    pub(crate) fn actor_ai_legacy(&mut self, routine: u32) {
        self.foreign(|game| unsafe { actor_species_ai(routine, game) });
    }
    pub(crate) fn actor_instruction_legacy(
        &mut self,
        routine: u32,
        actor: u16,
        script: EnemyScript,
    ) -> EnemyScript {
        self.foreign(|game| unsafe { actor_species_instruction(routine, actor, script, game) })
    }
    pub(crate) fn actor_touch_legacy(&mut self, routine: u32) {
        self.foreign(|game| unsafe { actor_species_touch(routine, game) });
    }
    pub(crate) fn actor_shot_legacy(&mut self, routine: u32, argument: u16) {
        self.foreign(|game| unsafe { actor_species_shot(routine, argument, game) });
    }
    pub(crate) fn actor_hook_legacy(&mut self, routine: u32) {
        self.foreign(|game| unsafe { actor_species_draw_hook(routine, game) });
    }
    pub(crate) fn actor_player_scene_legacy(&mut self) {
        self.foreign(crate::game::visual_runtime::draw_player_frame);
    }
    pub(crate) fn actor_draw_compatible(
        &mut self,
        bank: u8,
        address: u16,
        x: u16,
        y: u16,
        palette: u16,
        tile: u16,
        base: bool,
        offscreen: bool,
    ) {
        self.foreign(|game| {
            use crate::game::sprite_drawing::Clip;
            let clip = if base {
                Clip::None
            } else if offscreen {
                Clip::Outside
            } else {
                Clip::Inside
            };
            crate::game::projectile_presentation_ram::with_base_tile(
                bank,
                address,
                (x, y),
                palette,
                tile,
                clip,
                game,
            );
        });
    }
    pub(crate) fn actor_copy_legacy_tilemap(&mut self, bank: u8, address: u16) {
        self.actor_copy_tilemap(bank, address, None);
    }
    pub(crate) fn actor_copy_prepared_tilemap(
        &mut self,
        bank: u8,
        address: u16,
        palette_bits: u16,
    ) {
        self.actor_copy_tilemap(bank, address, Some(palette_bits));
    }
    fn actor_copy_tilemap(&mut self, bank: u8, address: u16, palette_bits: Option<u16>) {
        let mut record = self.actor_legacy_record(bank, address.wrapping_add(2));
        loop {
            let destination = usize::from(record.word(0));
            if destination == 0xffff {
                break;
            }
            let bytes = usize::from(record.word(2)) * 2;
            record.base += 4;
            let _ = &self.game.g_ram[destination..destination + bytes];
            for i in 0..bytes {
                let mut value = record.byte(i);
                if i & 1 != 0 {
                    if let Some(bits) = palette_bits {
                        value = (value & !0x1c) | ((bits >> 8) as u8);
                    }
                }
                self.set_byte(destination + i, value);
            }
            record.base += bytes;
        }
    }
    pub(crate) fn actor_assertion_legacy(&mut self, drawing: bool) {
        self.foreign(|_game| unsafe {
            if drawing {
                crate::c_library::__assert_fail(
                    c"addr < 0x2000".as_ptr(),
                    c"src/sm_rtl.h".as_ptr(),
                    51,
                    c"uint8 *RomPtr_RAM(uint16_t)".as_ptr(),
                );
            } else {
                crate::c_library::__assert_fail(
                    c"E->current_instruction & 0x8000".as_ptr(),
                    c"src/sm_a0.c".as_ptr(),
                    3399,
                    c"void ProcessEnemyInstructions(void)".as_ptr(),
                );
            }
        });
    }
}

impl Runtime<'_> {
    pub(crate) fn actor_draw_prepared(
        &mut self,
        bank: u8,
        address: u16,
        x: u16,
        y: u16,
        palette: u16,
        tile: u16,
        base: bool,
        offscreen: bool,
    ) {
        self.foreign(|game| {
            use crate::game::{
                projectile_presentation_ram::{Native, sprite_map},
                sprite_drawing::Clip,
            };
            if base && address == 0 {
                return;
            }
            let map = sprite_map(bank, address, game, &mut Native);
            let clip = if base {
                Clip::None
            } else if offscreen {
                Clip::Outside
            } else {
                Clip::Inside
            };
            map.draw(
                (x, y),
                |entry| (entry.wrapping_add(tile) & 0xf1ff) | palette,
                clip,
                game,
            );
        });
    }
}

#[path = "player_collision_runtime.rs"]
mod player_collision_runtime;
pub(crate) use player_collision_runtime::{player_bottom, set_player_radius, settle_grapple};
#[path = "player_movement_runtime.rs"]
mod player_movement_runtime;
pub(crate) use player_movement_runtime::{
    advance_special_movement, ceres_movement, correct_previous_position,
};
#[path = "player_transitions_runtime.rs"]
mod player_transitions_runtime;
pub(crate) use player_transitions_runtime::{
    command as player_command, damage_player, face_forward as player_face_forward,
    finish_player_frame, finish_xray, initialize as initialize_player,
    mother_brain as player_mother_brain,
};
#[path = "player_update_runtime.rs"]
pub(crate) mod player_update_runtime;

#[cfg(sm_verify_dll)]
#[path = "../verification/collision_mechanism_checks.rs"]
pub(crate) mod collision_mechanism_checks;

#[cfg(sm_verify_dll)]
#[path = "../verification/player_spin_bounds.rs"]
pub(crate) mod player_spin_bounds;

#[cfg(sm_verify_dll)]
#[path = "../verification/player_movement_edges.rs"]
pub(crate) mod player_movement_edges;

#[cfg(sm_verify_dll)]
#[path = "../verification/player_minimap_bytes.rs"]
pub(crate) mod player_minimap_bytes;

#[cfg(sm_verify_dll)]
#[path = "../verification/player_legacy_edges.rs"]
pub(crate) mod player_legacy_edges;

#[cfg(sm_verify_dll)]
#[path = "../verification/player_update_aliases.rs"]
pub(crate) mod player_update_aliases;

#[cfg(sm_verify_dll)]
#[path = "../verification/player_minimap_boss.rs"]
pub(crate) mod player_minimap_boss;

#[cfg(sm_verify_dll)]
#[path = "../verification/player_minimap_edges.rs"]
pub(crate) mod player_minimap_edges;

#[cfg(sm_verify_dll)]
#[path = "../verification/player_normal_transfers.rs"]
pub(crate) mod player_normal_transfers;

pub(crate) fn actor_cosine(phase: u16, amplitude: u16, game: &mut Game) -> u16 {
    (actor_cosine_full(phase, amplitude, game) >> 16) as u16
}
pub(crate) fn actor_sine(phase: u16, amplitude: u16, game: &mut Game) -> u16 {
    (actor_sine_full(phase, amplitude, game) >> 16) as u16
}
crate::forward_calls! { pub(crate) fn (phase: u16, amplitude: u16, game: &mut Game) -> u32;
    actor_cosine_full => crate::game::actors::queries::sine_product(&mut Runtime::new(game), amplitude, phase.wrapping_add(64) as u8 as u16);
    actor_sine_full => crate::game::actors::queries::sine_product(&mut Runtime::new(game), amplitude, phase.wrapping_add(0x80) as u8 as u16);
}
pub(crate) fn actor_direction_vector(
    phase: u16,
    amplitude: u16,
    game: &mut Game,
) -> crate::game::session::Vector32 {
    let [x, y] = [0x40u16, 0x80].map(|offset| {
        let index = usize::from(phase.wrapping_add(offset) & 0x7f);
        u32::from(game.read_rom_word(0xa0b7ee, index * 2)) * u32::from(amplitude)
    });
    crate::game::session::Vector32 { x, y }
}
