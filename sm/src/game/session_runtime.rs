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
        #[cfg(sm_verify_dll)]
        player_legacy_edges::transfer(game, 0);
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
            use crate::game::sm_80::*;
            match (bank, limit) {
                (1, 6) => QueueSfx1_Max6(sound, g),
                (1, 9) => QueueSfx1_Max9(sound, g),
                (1, 15) => QueueSfx1_Max15(sound, g),
                (2, 6) => QueueSfx2_Max6(sound, g),
                (3, 3) => QueueSfx3_Max3(sound, g),
                (3, 6) => QueueSfx3_Max6(sound, g),
                (3, 9) => QueueSfx3_Max9(sound, g),
                (3, 15) => QueueSfx3_Max15(sound, g),
                _ => unreachable!("unknown projectile sound queue"),
            }
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
        self.foreign(|g| unsafe {
            use crate::game::{sm_9b, sm_90, sm_91};
            match step {
                PlayerStep::Radius => sm_90::Samus_SetRadius(g),
                PlayerStep::Input => sm_90::Samus_CallInputHandler(g),
                PlayerStep::SuitIndex => unreachable!("suit index handled by shared owner"),
                PlayerStep::Acceleration => unreachable!("acceleration handled by shared owner"),
                PlayerStep::InsideBlock => unreachable!("inside block handled by shared owner"),
                PlayerStep::Grapple => sm_9b::GrappleBeamHandler(g),
                PlayerStep::Xray => sm_91::XrayRunHandler(g),
                PlayerStep::RestoreControl => {
                    sm_90::CallSomeSamusCode(0x14, g);
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
    #[cfg(sm_verify_dll)]
    player_legacy_edges::transfer(game, 1);
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
    player_legacy_edges::transfer(guard.game, 2);
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

use crate::game::enemy_script::EnemyScript;
use crate::game::{sm_a2, sm_a3, sm_a4, sm_a5, sm_a6, sm_a7, sm_a8, sm_a9, sm_aa, sm_b2, sm_b3};
crate::game::callback_dispatch! {
    unsafe fn actor_species_instruction(ea: u32, k: u16, j: EnemyScript, game: &mut Game) -> EnemyScript => ea;
    call (k, j, game); default { crate::rtl::Unreachable(game); EnemyScript::Stop };
    @external {

        10651845 => [sm_a2::BouncingGoofball_Instr_88C5(k, j)];
        10716899 => sm_a3::Waver_Instr_1;
        10991155 => [sm_a7::Kraid_Instr_1(k, j)];
        11056990 => [sm_a8::Beetom_Instr_1(k, j)];
        11062987 => [sm_a8::WreckedShipRobot_Instr_7(k, j)];
        11063490 => [sm_a8::WreckedShipRobot_Instr_10(k, j)];
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
        0xb20000..=0xb2ffff => [sm_b2::DispatchEnemyInstr(ea, k, j, game)];
        0xb30000..=0xb3ffff => [sm_b3::DispatchEnemyInstr(ea, k, j, game)];
    }
}
impl Runtime<'_> {
    pub(crate) fn actor_ai_legacy(&mut self, routine: u32) {
        self.foreign(|game| unsafe { crate::game::sm_a0::CallEnemyAi(routine, game) });
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
        self.foreign(|game| unsafe { crate::game::sm_a0::CallHitboxTouch(routine, game) });
    }
    pub(crate) fn actor_shot_legacy(&mut self, routine: u32, argument: u16) {
        self.foreign(|game| unsafe { crate::game::sm_a0::CallHitboxShot(routine, argument, game) });
    }
    pub(crate) fn actor_hook_legacy(&mut self, routine: u32) {
        self.foreign(|game| unsafe { crate::game::sm_a0::CallEnemyGfxDrawHook(routine, game) });
    }
    pub(crate) fn actor_player_scene_legacy(&mut self) {
        self.foreign(|game| unsafe { crate::game::sm_90::DrawSamusAndProjectiles(game) });
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
        self.foreign(|game| unsafe {
            use crate::game::sm_81::{
                DrawSpritemapWithBaseTile, DrawSpritemapWithBaseTile2,
                DrawSpritemapWithBaseTileOffscreen,
            };
            let draw = if base {
                DrawSpritemapWithBaseTile
            } else if offscreen {
                DrawSpritemapWithBaseTileOffscreen
            } else {
                DrawSpritemapWithBaseTile2
            };
            draw(bank, address, x, y, palette, tile, game);
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
#[path = "player_movement_runtime.rs"]
mod player_movement_runtime;
#[path = "player_transitions_runtime.rs"]
mod player_transitions_runtime;
#[path = "player_update_runtime.rs"]
mod player_update_runtime;

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
