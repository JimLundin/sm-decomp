//! One aggregate authority for player, projectiles, mechanisms and their room.
use crate::game::actors::{bang, maridia_snail, metroid, reflec};
use crate::game::enemy_speeds::SpeedTable;
use crate::{
    game_state::Game,
    rom::{RomImage, RomRecord},
};
use sm_game::player_projectiles::{
    access::{ShotLane, Store},
    content,
    host::{Drawing, Host, PaletteEffect, PlayerStep},
};
use sm_game::session_access::SessionAccess;
use {
    crate::game::session_encoding as encoding,
    sm_game::{external_inputs::ExternalInputs, session::Session},
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

/// The native image at `bank:cursor`, as one original RomPtr fetch: a cursor
/// below $8000 reports the invalid-window diagnostic and still reads its offset.
pub(crate) fn native_record(bank: u8, cursor: u16, game: &mut Game) -> RomRecord {
    let address = (u32::from(bank) << 16) | u32::from(cursor);
    if address & 0x8000 == 0 {
        native_rom_diagnostic(address, game);
    }
    RomRecord {
        image: game.capture_native_rom(),
        base: crate::rom::lorom_offset(address) as usize,
    }
}

#[forbid(unsafe_code)]
fn draw_trail(game: &mut Game, pair: usize, right: bool, frozen: bool) {
    let ram = game.ram();
    let read = |at| encoding::read(&|at| ram.read_byte(at), at);
    let saved = read(0x590);
    if (saved as i16).wrapping_sub(512) >= 0 {
        return;
    }
    let (timer, x_field, y_field, tile_field) = if right {
        (0xd67c, 0xd754, 0xd79c, 0xd70c)
    } else {
        (0xd658, 0xd730, 0xd778, 0xd6e8)
    };
    if frozen && read(timer + 2 * pair) == 0 {
        return;
    }
    crate::verify_probe::hit(match (frozen, right) {
        (true, false) => "trails.draw.frozen_left",
        (true, true) => "trails.draw.frozen_right",
        (false, false) => "trails.draw.active_left",
        (false, true) => "trails.draw.active_right",
    });
    let x = read(x_field + 2 * pair).wrapping_sub(read(0x911));
    if x & 0xff00 == 0 {
        let entry = 0x370 + usize::from(read(0x590));
        game.ram().copy_in(entry, &x.to_ne_bytes());
        // Preserve rereads after each overlapping native OAM store.
        let y = read(y_field + 2 * pair).wrapping_sub(read(0x915));
        if y & 0xff00 == 0 {
            game.ram().copy_in(entry + 1, &y.to_ne_bytes());
            let tile = read(tile_field + 2 * pair);
            game.ram().copy_in(entry + 2, &tile.to_ne_bytes());
            game.ram()
                .copy_in(0x590, &saved.wrapping_add(4).to_le_bytes());
        }
    }
}

#[cfg(sm_verify_dll)]
#[path = "../verification/actor_movement_runtime_observer.rs"]
mod actor_movement_runtime_observer;

/// `pub` inside the crate-private module, so `LeaseActors` can name it.
pub struct Runtime<'g> {
    game: &'g mut Game,
    state: Option<Session>,
    inputs: ExternalInputs,
    persistent_epoch: Option<u64>,
    persistent: Option<(
        sm_game::scene::samus_content::ContentDescription,
        std::sync::Arc<crate::game_state::persistent_session::SessionStartRequirements>,
    )>,
}
/// Spawns the collision PLM with `header` at the selected block. Was in bank $AA.
pub(crate) fn spawn_collision_plm(header: u16, game: &mut Game) {
    unsafe { crate::game::room_mechanisms::spawn_collision(header, game) };
}

/// Runs the bank $AA function at `address` for the current enemy. Was in bank $AA.
pub(crate) fn run_aa_function(address: u32, game: &mut Game) {
    unsafe { actor_species_pre_instruction(address, game) };
}

/// Reflec's shot reaction, with the original diagnostic when it cannot tell
/// which projectile hit it. Was `Reflec_Shot` in bank $A3.
fn reflec_shot(game: &mut Game) {
    if let Some(target) = reflec::reflect_shot(game) {
        unsafe { crate::c_library::printf(c"Possible bug. What is X?\n".as_ptr()) };
        reflec::absorb_shot(target, game);
    }
}

/// The original "X undefined" debug message Kraid's foot and fingernails
/// print. Was in bank $A7.
pub(crate) fn print_x_undefined() {
    unsafe { crate::c_library::printf(c"X undefined\n".as_ptr()) };
}

/// Sets the shot AI of Kraid's enemy definition at `enemy_ptr`. Was in bank $A7.
pub(crate) fn set_kraid_shot_ai(enemy_ptr: u16, shot_ai: u16, game: &mut Game) {
    (unsafe { crate::rtl::rom_record(0xa0, enemy_ptr, game) }).set_word(
        core::mem::offset_of!(crate::game::prelude::EnemyDef, shot_ai),
        shot_ai,
    );
    #[cfg(sm_verify_dll)]
    crate::verify_probe::hit("actors.kraid.shot_ai.disabled");
}

/// Runs the bank $A7 function at `address` for the current enemy. Was in bank $A7.
pub(crate) fn run_a7_function(address: u32, game: &mut Game) {
    unsafe { actor_species_pre_instruction(address, game) };
}

/// The steam's next script at `bank:address`, as the original ROM pointer.
/// Was in bank $A6.
pub(crate) fn steam_script(bank: u8, address: u16, game: &mut Game) -> EnemyScript {
    let record = unsafe { crate::rtl::rom_record(bank, address, game) };
    EnemyScript::at(record.image, record.base)
}

/// Magdollite's diagnostic for an undefined lava height. Was in bank $A8.
fn warn_lava_height(value: i32) {
    unsafe { crate::c_library::printf(c"Y undefined %d!\n".as_ptr(), value) };
}

/// Every pre-instruction enters through the sm_game table, which reads the
/// current enemy once and falls back to the compatibility arms.
pub(crate) unsafe fn actor_species_pre_instruction(ea: u32, game: &mut Game) {
    sm_game::actors::species::pre_instruction(ea, game);
}

impl sm_game::actors::species::AiHost for Game {
    fn native_ai(&mut self, ea: u32) {
        unsafe { native_ai(ea, self) }
    }
    fn grapple_kill(&mut self) {
        actors::combat::die(&mut Runtime::new(self), actors::api::DeathKind::Grapple);
    }
    fn install_draygon_layers(&mut self) {
        crate::game::visual_runtime::initialize_draygon_layer(self);
    }
    fn warn_draygon_swoop(&mut self) {
        unsafe { crate::c_library::printf(c"Wtf is varE26?\n".as_ptr()) };
    }
    fn native_draygon_function(&mut self, ea: u32, k: u16) {
        spore_spawn_function(ea, k, self);
    }
}

/// Spore Spawn's functions, which Draygon's function dispatch shared. Was
/// `CallDraygonFunc` in bank $A5.
fn spore_spawn_function(ea: u32, k: u16, game: &mut Game) {
    match ea {
        10873627 => actors::spore_spawn::descend(game),
        10873755 => actors::spore_spawn::begin_return(game),
        10873838 => actors::spore_spawn::finish_return(game),
        10873626 => (),
        10873682 => actors::spore_spawn::orbit(k, game),
        _ => {
            crate::rtl::Unreachable(game);
        }
    }
}

impl sm_game::actors::species::PreInstructionHost for Game {
    fn native_pre_instruction(&mut self, ea: u32, k: u16) {
        unsafe { native_pre_instruction(ea, k, self) }
    }
}

impl sm_game::actors::species::InstructionHost for Game {
    fn native_instruction(&mut self, ea: u32, k: u16, j: EnemyScript) -> EnemyScript {
        unsafe { native_instruction(ea, k, j, self) }
    }
}

impl<'g> Runtime<'g> {
    pub fn new(game: &'g mut Game) -> Self {
        #[cfg(sm_verify_dll)]
        player_normal_transfers::record(game, 0);
        if let Some(session) = game
            .lease_persistent_session()
            .expect("Session lease rejected")
        {
            return Self {
                state: Some(session.gameplay),
                inputs: session.observations,
                persistent: Some((session.roles, session.required)),
                persistent_epoch: Some(game.compatibility_write_epoch()),
                game,
            };
        }
        Self {
            state: Some(import_session(game)),
            inputs: encoding::inputs(&|at| game.ram().read_byte(at)),
            persistent: None,
            persistent_epoch: None,
            game,
        }
    }
    fn persistent_is_current(&self) -> bool {
        self.persistent_epoch.is_none_or(|epoch| {
            epoch == self.game.compatibility_write_epoch()
                && self.game.preflight_samus_runtime().is_ok()
        })
    }
    pub(crate) fn samus_game(&self) -> &Game {
        self.game
    }
    pub(crate) fn samus_source_diagnostic(&mut self, address: u32) {
        native_rom_diagnostic(address, self.game);
    }
    pub(crate) fn has_owned_display(&self) -> bool {
        self.session().presentation.is_owned()
    }
    pub(crate) fn draw_owned_body(&mut self, mode: sm_game::scene::samus_body::Body) {
        use crate::game_state::persistent_session::{self, Presentation};
        assert!(self.persistent_is_current(), "Session lease is dirty");
        let state = self.state.as_mut().expect("Session lease");
        let Presentation::Owned {
            display,
            marker,
            samus_publications,
            ..
        } = &mut state.presentation
        else {
            panic!("owned Display required");
        };
        persistent_session::draw_body(
            display.owner_mut(),
            self.game,
            &mut state.player,
            &self.inputs,
            samus_publications.clone(),
            marker,
            mode,
        )
        .expect("owned Samus body");
    }
    pub(crate) fn owned_bottom_visible(&mut self) -> bool {
        use crate::game_state::persistent_session::{self, Presentation};
        assert!(self.persistent_is_current(), "Session lease is dirty");
        let state = self.state.as_mut().expect("Session lease");
        let Presentation::Owned {
            display,
            marker,
            samus_publications,
            ..
        } = &mut state.presentation
        else {
            panic!("owned Display required");
        };
        persistent_session::bottom_visible(
            display.owner_mut(),
            self.game,
            &mut state.player,
            &self.inputs,
            samus_publications.clone(),
            marker,
        )
        .expect("owned Samus bottom")
    }
    pub(crate) fn draw_owned_echo(&mut self, slot: u16) {
        use sm_game::scene::samus_content::{Registration, Role};
        let index = usize::from(slot >> 1);
        let y = self.word(0xab8 + index * 2);
        let source = self
            .game
            .prepared_samus_role(Role::PositionFallback, Registration::Loaded, 0)
            .expect("echo role")
            .expect("supplied loaded echo role");
        let height = source
            .byte(usize::from(self.session().player.pose) * 8 + 4)
            .expect("echo height source");
        let y = y
            .wrapping_sub(u16::from(height))
            .wrapping_sub(self.inputs.camera_y);
        if (y as i16) >= 0 && y.wrapping_sub(248) & 0x8000 != 0 {
            let producer = self
                .session()
                .presentation
                .publications()
                .expect("Samus publications")
                .echoes
                .get(index)
                .copied()
                .flatten()
                .expect("supplied live echo producer");
            let sprite = self.session().player.top_sprite;
            let x = self
                .word(0xab0 + index * 2)
                .wrapping_sub(self.inputs.camera_x);
            self.draw_owned_samus(producer, sprite, (x, y));
            if self.session().player.bottom_sprite != 0 {
                let sprite = self.session().player.bottom_sprite;
                let x = self
                    .word(0xab0 + index * 2)
                    .wrapping_sub(self.inputs.camera_x);
                self.draw_owned_samus(producer, sprite, (x, y));
            }
        }
    }
    pub(crate) fn draw_echo_samus_sprite(&mut self, slot: u16, index: u16, x: u16, y: u16) {
        if self.has_owned_display() {
            let producer = self
                .session()
                .presentation
                .publications()
                .expect("Samus publications")
                .echoes
                .get(usize::from(slot >> 1))
                .copied()
                .flatten()
                .expect("supplied live echo producer");
            self.draw_owned_samus(producer, index, (x, y));
        } else {
            sm_game::visual_host::Effects::draw_samus_sprite(self, index, x, y);
        }
    }
    pub(crate) fn samus_request_written(
        &mut self,
        half: sm_game::scene::samus_body::Half,
        pointer: u16,
    ) {
        let request = self
            .game
            .samus_request(half, pointer)
            .expect("Samus request admission");
        if !self.has_owned_display() {
            return;
        }
        let request = request.expect("supplied Samus request");
        let media = self
            .session()
            .presentation
            .owner()
            .expect("Display owner")
            .media()
            .clone();
        request
            .validate(pointer, &media)
            .expect("Samus request content");
        let slot = match half {
            sm_game::scene::samus_body::Half::Top => 0,
            sm_game::scene::samus_body::Half::Bottom => 1,
        };
        let owner = self
            .session_mut()
            .presentation
            .owner_mut()
            .expect("Display owner");
        assert!(
            owner.state.phase == sm_game::scene::state::DisplayPhase::Producing,
            "Samus publication phase"
        );
        assert!(
            owner.state.samus[slot].pointer == pointer && owner.state.samus[slot].enabled == 1,
            "Samus request write prefix"
        );
        owner.state.samus[slot] = request.half;
    }
    pub(crate) fn draw_owned_samus(
        &mut self,
        producer: crate::game::scene::records::ProducerObjectId,
        index: u16,
        position: (u16, u16),
    ) {
        use crate::game_state::persistent_session::{self, Presentation};
        assert!(self.persistent_is_current(), "Session lease is dirty");
        let state = self.state.as_mut().expect("Session lease");
        let Presentation::Owned {
            display,
            samus_publications,
            ..
        } = &mut state.presentation
        else {
            panic!("owned Display required");
        };
        persistent_session::draw_sprite(
            display.owner_mut(),
            self.game,
            &mut state.player,
            &self.inputs,
            samus_publications.clone(),
            producer,
            index,
            position,
        )
        .expect("owned Samus publication");
    }
    pub fn foreign<R>(&mut self, action: impl FnOnce(&mut Game) -> R) -> R {
        assert!(self.persistent_is_current(), "Session lease is dirty");
        preflight_export(self.session(), self.game);
        let mut mutation = self
            .game
            .begin_samus_mutation()
            .expect("Samus mutation rejected");
        let state = self
            .state
            .take()
            .expect("projectile state unavailable during external effect");
        // Export was authorized before revocation; do not repeat ready preflight
        // while the external interval is deliberately unavailable.
        export_session_preflighted(state, mutation.game());
        let result = action(mutation.game());
        let payload = mutation.game().capture_samus_snapshot();
        let prepared = mutation
            .game()
            .prepare_samus_adoption(&payload)
            .expect("Samus resumption rejected");
        self.state = Some(import_session_preflighted(mutation.game()));
        mutation.finish(prepared).expect("Samus adoption rejected");
        self.inputs = encoding::inputs(&|at| self.game.ram().read_byte(at));
        result
    }
    /// Compose an already-owned operation while retaining the former foreign
    /// entry's preflight, unwind publication, and refreshed input observations.
    pub(crate) fn owned_call<R>(&mut self, action: impl FnOnce(&mut Self) -> R) -> R {
        assert!(self.persistent_is_current(), "Session lease is dirty");
        preflight_export(self.session(), self.game);
        struct OwnedCall<'a, 'g> {
            runtime: &'a mut Runtime<'g>,
            completed: bool,
        }
        impl Drop for OwnedCall<'_, '_> {
            fn drop(&mut self) {
                if !self.completed {
                    self.runtime.relinquish();
                    self.runtime.game.revoke_samus_authority();
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
    /// Match the player driver's former Runtime entry and successful return seams.
    pub(crate) fn owned_player_call<R>(&mut self, action: impl FnOnce(&mut Self) -> R) -> R {
        self.owned_call(|runtime| {
            runtime.refresh_player_observations();
            let result = action(runtime);
            runtime.refresh_player_observations();
            result
        })
    }
    pub(crate) fn refresh_player_observations(&mut self) {
        assert!(self.persistent_is_current(), "Session lease is dirty");
        self.inputs = encoding::inputs(&|at| self.byte(at));
    }
    fn relinquish(&mut self) {
        if let Some(state) = self.state.take() {
            if let Some((roles, required)) = self.persistent.take() {
                let matching = std::sync::Arc::ptr_eq(&state.world, self.game.world())
                    && state.world.revision() == self.game.world().revision();
                let mut epoch = self
                    .persistent_epoch
                    .take()
                    .expect("persistent lease epoch");
                let valid = matching
                    && epoch == self.game.compatibility_write_epoch()
                    && self.game.preflight_samus_runtime().is_ok();
                if valid {
                    // Publish compatibility bytes while keeping the same gameplay
                    // and private Display owner; no decode or identity transfer.
                    encoding::encode(&state, |at, bytes| self.game.ram().copy_in(at, bytes));
                    export_native_spawn(&state, self.game);
                    epoch = self.game.compatibility_write_epoch();
                }
                let completed = valid
                    && !std::thread::panicking()
                    && state.presentation.owner().is_ok_and(|owner| {
                        owner.state.phase != sm_game::scene::state::DisplayPhase::Failed
                    });
                self.game.return_persistent_session(
                    crate::game_state::persistent_session::PersistentSession {
                        gameplay: state,
                        observations: self.inputs.clone(),
                        roles,
                        required,
                    },
                    completed,
                    epoch,
                );
                return;
            }
            let matching = std::sync::Arc::ptr_eq(&state.world, self.game.world())
                && state.world.revision() == self.game.world().revision();
            if matching
                && self.game.actor_identity_is_with_session()
                && self.game.preflight_samus_runtime().is_ok()
            {
                export_session_preflighted(state, self.game);
            } else if self.game.actor_identity_is_with_session() {
                // A rejected foreign call retains this Runtime's identity owner;
                // discard invalid byte/native publication rather than panic again.
                self.game.return_actor_identity(state.actor_identity);
                self.game.revoke_samus_authority();
            }
        }
    }
    pub(crate) fn byte(&self, at: usize) -> u8 {
        if self.persistent_is_current() {
            if let Some(byte) = encoding::projected_byte(self.session(), at) {
                return byte;
            }
        }
        self.game.ram().read_byte(at)
    }
    pub(crate) fn word(&self, at: usize) -> u16 {
        u16::from_le_bytes([self.byte(at), self.byte(at + 1)])
    }
    /// A Samus field from the owned Session while this lease is current; the
    /// projected word at `at` otherwise. Both read the same value whenever the
    /// projection would serve the Session, so callers observe no difference.
    /// A Session field while this lease is current; the projected word at `at`
    /// otherwise, which is the same value whenever the projection serves it.
    pub(crate) fn owned_session(&self, field: impl Fn(&Session) -> u16, at: usize) -> u16 {
        if self.persistent_is_current() {
            field(self.session())
        } else {
            self.word(at)
        }
    }
    pub(crate) fn owned_player(
        &self,
        field: impl Fn(&sm_game::session::Player) -> u16,
        at: usize,
    ) -> u16 {
        if self.persistent_is_current() {
            field(&self.session().player)
        } else {
            self.word(at)
        }
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
        assert!(self.persistent_is_current(), "Session lease is dirty");
        self.state
            .as_ref()
            .expect("aggregate unavailable during external effect")
    }
    pub(crate) fn session_mut(&mut self) -> &mut Session {
        assert!(self.persistent_is_current(), "Session lease is dirty");
        self.state
            .as_mut()
            .expect("aggregate unavailable during external effect")
    }
    pub(crate) fn inputs(&self) -> &ExternalInputs {
        assert!(self.persistent_is_current(), "Session lease is dirty");
        &self.inputs
    }
    // Legacy aliases observe the raw result before Drop republishes Session.
    pub(crate) fn update_maw_visibility(&mut self, actor: i32) {
        assert!(self.persistent_is_current(), "Session lease is dirty");
        use crate::game::enemy_layouts::EnemyLayout104 as Maw;
        let outside = u16::from(crate::game::actors::api::current_outside_screen(&*self));
        let game = &mut *self.game;
        crate::ram_map::slot!(game, Maw[actor].var_36 = outside);
        // Retain raw LOW/HIGH stores and all old owned-region overwrites.
        // Only newly owned death/body lanes and the actors' extended RAM,
        // which holds this word, adopt this raw store.
        let at = crate::ram_map::slot!(@at Maw, actor, var_36);
        let bytes = outside.to_ne_bytes();
        let state = self.state.as_mut().expect("aggregate unavailable");
        encoding::write_botwoon_death_byte(state, at, bytes[0]);
        encoding::write_botwoon_death_byte(state, at + 1, bytes[1]);
        for lane in [at, at + 1] {
            if (0xb04..0xb08).contains(&lane) || (0x7800..0x9000).contains(&lane) {
                encoding::write_projected_byte(state, lane, game.ram().read_byte(lane));
            }
        }
        if self.persistent_epoch.is_some() {
            self.persistent_epoch = Some(game.compatibility_write_epoch());
        }
    }
    pub(crate) fn native_word(&self, at: usize) -> u16 {
        u16::from_ne_bytes([self.byte(at), self.byte(at + 1)])
    }
    pub(crate) fn set_byte(&mut self, at: usize, value: u8) {
        if !self.persistent_is_current() {
            self.game.ram().write_byte(at, value);
            self.inputs = encoding::inputs(&|lane| self.game.ram().read_byte(lane));
            return;
        }
        // Ordinary RAM and owned fields are one live projection during an interval.
        // A foreign destination may alias an external observation used next.
        if !encoding::write_projected_byte(
            self.state.as_mut().expect("aggregate unavailable"),
            at,
            value,
        ) {
            self.game.ram().write_byte(at, value);
            if self.persistent_epoch.is_some() {
                self.persistent_epoch = Some(self.game.compatibility_write_epoch());
            }
        }
        if self.persistent.is_some()
            && crate::game_state::persistent_session::Presentation::requires_semantic_counterpart(
                at,
            )
        {
            // Keep completed byte/typed prefixes for Drop publication, then
            // retain this owner as failed until a complete semantic adoption.
            self.session_mut()
                .presentation
                .owner_mut()
                .expect("persistent Display")
                .state
                .phase = sm_game::scene::state::DisplayPhase::Failed;
        }
        self.inputs = encoding::inputs(&|at| {
            if self.persistent.is_some() {
                self.byte(at)
            } else {
                self.game.ram().read_byte(at)
            }
        });
        // Preserve observation refresh timing while A5A is owned by Session.
        self.inputs.transitions.frame_gamma = self.session().player.transitions.frame_gamma;
        // The camera and cinematic observations are Session words; refresh them from
        // the store just made, as the legacy RAM read did before they were owned.
        let camera = self.session().room.camera;
        self.inputs.camera_x = camera.layer1[0];
        self.inputs.camera_y = camera.layer1[1];
        self.inputs.background_page_x = camera.bg1_offset[0];
        self.inputs.cinematic = self.session().flow.cinematic_function;
    }
    pub(crate) fn write_slice(&mut self, at: usize, bytes: &[u8]) {
        let end = at.checked_add(bytes.len()).expect("aggregate write extent");
        assert!(at <= end && end <= 0x20000);
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
    /// A word of the 32-bit speed table read the way native_image readers did.
    pub(crate) fn native_speed32(&self, offset: usize) -> u32 {
        let native = self.game.capture_native_rom();
        crate::game::enemy_speeds::word32(&self.game.enemy_speeds32, &native, offset)
    }
    pub(crate) fn enemy_speed(&self, table: SpeedTable, index: isize) -> u16 {
        crate::game::enemy_speeds::word(
            &self.game.enemy_speeds,
            &self.game.loaded_rom,
            table,
            index,
        )
    }
}
impl actors::botwoon_death::Host for Runtime<'_> {
    fn native_death_signal(&self, enemy: u16, signal: actors::botwoon_death::Signal) -> u16 {
        self.native_word(signal.address(enemy))
    }
    fn set_native_death_signal(
        &mut self,
        enemy: u16,
        signal: actors::botwoon_death::Signal,
        value: u16,
    ) {
        let at = signal.address(enemy);
        let [low, high] = value.to_ne_bytes();
        self.set_byte(at, low);
        self.set_byte(at + 1, high);
    }
    fn set_collapse_delay(&mut self, enemy: u16, value: u16) {
        let at = actors::botwoon_death::collapse_delay_address(enemy);
        let [low, high] = value.to_ne_bytes();
        self.set_byte(at, low);
        self.set_byte(at + 1, high);
    }
}

impl crate::game_state::persistent_session::AudioHost for Runtime<'_> {
    fn audio_byte(&self, at: usize) -> u8 {
        self.byte(at)
    }
    fn audio_store_byte(&mut self, at: usize, byte: u8) {
        self.set_byte(at, byte);
    }
    fn audio_mutation<R>(
        &mut self,
        operation: impl FnOnce(
            &mut crate::game_state::persistent_session::AudioRequests,
            &mut sm_game::scene::start::InitialAudioIntent,
            crate::game_state::persistent_session::AudioStamp,
        ) -> Result<R, sm_game::scene::state::DisplayError>,
    ) -> Result<Option<R>, sm_game::scene::state::DisplayError> {
        use crate::game_state::persistent_session::Control;
        use sm_game::scene::state::{DisplayError, DisplayPhase};
        if self.persistent.is_none() {
            return Ok(None);
        }
        if !self.persistent_is_current() {
            return Err(DisplayError::Poisoned);
        }
        let state = self.state.as_mut().ok_or(DisplayError::Phase)?;
        let producing = state.presentation.owner()?.state.phase == DisplayPhase::Producing;
        let Control::Owned(control) = &mut state.control else {
            return Err(DisplayError::Phase);
        };
        let result = control.audio_mutation(producing, operation);
        if result.is_err() {
            state.presentation.owner_mut()?.state.phase = DisplayPhase::Failed;
        }
        result.map(Some)
    }
}
impl Drop for Runtime<'_> {
    fn drop(&mut self) {
        self.relinquish();
    }
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
        if self.has_owned_display() {
            self.owned_call(|runtime| {
                if !matches!(
                    (bank, limit),
                    (1, 6 | 9 | 15) | (2, 6) | (3, 3 | 6 | 9 | 15)
                ) {
                    unreachable!("unknown projectile sound queue");
                }
                let channel = usize::from(bank - 1);
                #[cfg(sm_verify_dll)]
                crate::game::visual_runtime::samus_observer::sound(channel, sound as u8, limit);
                crate::game::music::requests::queue_sfx(channel, sound as u8, limit, runtime)
                    .expect("audio sound enqueue");
            });
            return;
        }
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
            sm_game::samus_palette::update_suit_index(self.session_mut());
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
        if matches!(step, PlayerStep::Grapple) {
            self.owned_call(|runtime| sm_game::grapple_swing::update(runtime));
            return;
        }
        self.foreign(|g| {
            match step {
                PlayerStep::Radius => set_player_radius(g),
                PlayerStep::Input => unreachable!("input handled by shared owner"),
                PlayerStep::SuitIndex => unreachable!("suit index handled by shared owner"),
                PlayerStep::Acceleration => unreachable!("acceleration handled by shared owner"),
                PlayerStep::InsideBlock => unreachable!("inside block handled by shared owner"),
                PlayerStep::Grapple => unreachable!("grapple handled by shared owner"),
                PlayerStep::Xray => unreachable!("X-ray handled by shared owner"),
                PlayerStep::RestoreControl => {
                    crate::game::session_runtime::player_command(0x14, g);
                }
            };
        });
    }
    fn palette(&mut self, effect: PaletteEffect) -> u8 {
        use sm_game::{samus_palette, visual_state::PaletteBuffer};
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
                sm_game::palette_effects::execution::spawn(self, definition);
                0
            }
        }
    }
    fn enable_hdma(&mut self) {
        self.foreign(|g| crate::game::visual_runtime::set_hdma_enabled(true, g));
    }
    fn spawn_power_bomb(&mut self) {
        self.foreign(|g| crate::game::explosion_window::spawn_power_bomb(g));
    }
    fn spawn_mechanism(&mut self, header: u16) {
        let mut contact = sm_game::room_mechanisms::host::Contact::Absent;
        sm_game::room_mechanisms::creation::spawn(
            self,
            sm_game::room_mechanisms::host::SpawnSource::Collision { header },
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
        self.foreign(|g| crate::rtl::Unreachable(g))
    }
    fn room_level(&self, cell: u16) -> u16 {
        self.word(0x10002 + usize::from(cell) * 2)
    }
    fn room_bts(&self, cell: u16) -> u8 {
        self.byte(0x16402 + usize::from(cell))
    }
    fn alias_shot_read(&self, lane: ShotLane, raw: u16) -> u16 {
        self.word(lane.address(raw))
    }
    fn alias_shot_write(&mut self, lane: ShotLane, raw: u16, value: u16, store: Store) {
        self.store(lane.address(raw), value, store);
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
    context: sm_game::room_mechanisms::host::CallerContext,
) -> u8 {
    use sm_game::room_mechanisms::host::{CallerContext, Pass, RunResult};
    let mut pass = match game.ram().read_byte(0x613) {
        0 => Pass::Instructions,
        1 => Pass::DisplayMessage,
        other => Pass::Invalid(other),
    };
    let result = {
        let mut runtime = Runtime::new(game);
        sm_game::room_mechanisms::scheduler::advance(&mut runtime, &mut pass)
    };
    match context {
        CallerContext::Gameplay | CallerContext::DoorTransition => match result {
            RunResult::Complete => {
                game.compatibility_write_owned_byte(0x613, 0);
                0
            }
            RunResult::Yield(value) => {
                game.compatibility_write_owned_byte(0x613, 1);
                value.get()
            }
        },
    }
}

pub(crate) fn import_session(game: &mut Game) -> Session {
    game.preflight_samus_runtime()
        .expect("Samus import rejected");
    import_session_preflighted(game)
}
fn import_session_preflighted(game: &mut Game) -> Session {
    #[cfg(sm_verify_dll)]
    player_normal_transfers::record(game, 1);
    let mut state = encoding::decode(
        &|at| game.ram().read_byte(at),
        0x20000,
        game.world().clone(),
    );
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
    game.preflight_samus_runtime()
        .expect("Samus export rejected");
}
/// Holds identity until it is returned, including an unwind after partial encoding.
struct ExportIdentity<'g> {
    state: Option<Session>,
    game: &'g mut Game,
}
impl Drop for ExportIdentity<'_> {
    fn drop(&mut self) {
        if let Some(state) = self.state.take() {
            if self.game.actor_identity_is_with_session() {
                self.game.return_actor_identity(state.actor_identity);
            }
        }
    }
}
pub(crate) fn export_session(state: Session, game: &mut Game) {
    let guard = ExportIdentity {
        state: Some(state),
        game,
    };
    preflight_export(
        guard.state.as_ref().expect("export owns Session"),
        guard.game,
    );
    encode_session(guard);
}
fn export_session_preflighted(state: Session, game: &mut Game) {
    encode_session(ExportIdentity {
        state: Some(state),
        game,
    });
}
fn encode_session(guard: ExportIdentity<'_>) {
    #[cfg(sm_verify_dll)]
    player_normal_transfers::record(guard.game, 2);
    let state = guard.state.as_ref().expect("export owns Session");
    encoding::encode(state, |at, bytes| guard.game.ram().copy_in(at, bytes));
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

use crate::game::actors::{self, api as actor_api};
use crate::game::moving_platforms::{self as mechanisms, Event, Kind};
use crate::game::rippers::{self, Kind as RipperKind};
use crate::game::sm_a9;
use crate::game::{ground_patrol, large_snail};
use crate::types::EnemyData;
use sm_game::actors::access::{AccessMode, ActorAccess, ActorKey as Key};
use sm_game::combat_rules::LatchMode;
use sm_game::enemy_script::EnemyScript;

crate::game::callback_dispatch! {
    unsafe fn native_ai(ea: u32, game: &mut Game) => ea;
    call (game); default { crate::rtl::Unreachable(game); };
    @external {
        10715136 | 10846208 | 10911744 | 10977280 | 11042816 | 11763712 => [actor_api::resume_ai(&mut Runtime::new(game))];
        10649605 | 10780677 => [actor_api::latch(&mut Runtime::new(game), LatchMode::Passive)];
        11042836 => [actor_api::latch(&mut Runtime::new(game), LatchMode::Active)];
        11763722 => [crate::game::actors::combat::die(&mut Runtime::new(game), actor_api::DeathKind::Grapple)];
        11763727 => [crate::game::actors::combat::select_frozen_ai(&mut Runtime::new(game))];

        11763777 => [crate::game::actors::scheduling::update_frozen_enemy(&mut Runtime::new(game))];
        10649676 => [{
            #[cfg(sm_verify_dll)]
            crate::game::actors::reference_population_animation::dispatch_noop(game);
        }];
        10656319 => crate::game::hopping_blobs::initialize;
        10656381 => [{ if !crate::game::hopping_blobs::advance(game) { crate::rtl::Unreachable(game); } }];
        10664233 => [{
                    crate::game::rinka::frozen(crate::ram_map::word!(game.cur_enemy_index), game);
                }];
        10664275 => [{
                    crate::game::rinka::power_bomb(crate::ram_map::word!(game.cur_enemy_index), game);
                }];
        10669268 => crate::game::large_snail::initialize;
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
        10716909 => crate::game::waver::Waver_Init;
        10717004 => crate::game::waver::Waver_Main;

        10730277 => [{ let address = bang::main_ai(game); crate::cpu_infra::Call(address, game); }];
        10738716 => reflec_shot;
        10715146 => [crate::game::actors::combat::die(&mut Runtime::new(game), actor_api::DeathKind::Grapple)];
        10715151 => [crate::game::actors::combat::select_frozen_ai(&mut Runtime::new(game))];

        10715201 => [crate::game::actors::scheduling::update_frozen_enemy(&mut Runtime::new(game))];
        10735074 => maridia_snail::initialize;
        10735204 => [{ let phase = maridia_snail::prepare(game); actor_species_pre_instruction(phase, game); }];
        10736560 => maridia_snail::touch; 10736745 => maridia_snail::shot;
        10742351 => metroid::initialize; 10742502 => metroid::frozen;
        10742579 => metroid::hurt; 10742680 => metroid::advance;
        10743275 => metroid::touch; 10743559 => metroid::shot;
        10743874 => metroid::power_bomb;




        10914607 => crate::game::spikey_platforms::initialize; 10914733 => crate::game::spikey_platforms::run;



        11051537 => [{
            if !crate::game::actors::yapping_maw::main_ai(game, |operand, game| {
                crate::rtl::WriteReg(crate::rtl::WRMPYA as u16, operand, game);
            }) {
                crate::rtl::Unreachable(game);
            }
            crate::game::actors::yapping_maw::update_segments(game);
        }];

        11055370 => [{ let address = crate::game::actors::magdollite::main_ai(game); actor_species_pre_instruction(address, game); }];
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
        11065545 => crate::game::actors::maridia_puffer::initialize;
        11065611 => crate::game::actors::maridia_puffer::advance; 11066132 => crate::game::actors::maridia_puffer::shot;
        11068983 => crate::game::actors::wrecked_ship_spark::initialize;
        11069070 => crate::game::actors::wrecked_ship_spark::advance;
        11069198 | 11069725 => [crate::ram_map::word!(game.projectile_dir[crate::ram_map::word!(game.collision_detection_index)] &= !(0x10))];
        11071880 => crate::game::actors::ki_hunter::initialize;
        11072020 => crate::game::actors::ki_hunter::initialize_wings; 11072092 => crate::game::actors::ki_hunter::advance;
        11072098 => crate::game::actors::ki_hunter::advance;
        11073281 => crate::game::actors::ki_hunter::shot;
        11111301 => [{
                    sm_a9::MotherBrainsTubesFalling_Main(crate::ram_map::word!(game.cur_enemy_index), game);
                }];
        11121927 => sm_a9::MotherBrainsBrain_Shot;



        11765499 => [{
            let e = i32::from(crate::ram_map::word!(game.cur_enemy_index));
            crate::ram_map::slot!(game, EnemyData[e].properties, |old| old | crate::game::constants::enemy_properties::kEnemyProps_DisableSamusColl as u16);
            crate::ram_map::slot!(game, EnemyData[e].current_instruction = 0x86a7);
        }];

        11765882 => [{ let address = crate::game::actors::brinstar_pipe_bug::state_address(game); actor_species_pre_instruction(address, game); }];
        11766686 => [{
            let address = crate::game::actors::norfair_pipe_bug::state_address(game);
            sm_game::actors::species::pipe_bug_state(address, crate::ram_map::word!(game.cur_enemy_index), game);
            crate::game::actors::norfair_pipe_bug::reset_offscreen(game);
        }];
        11767726 => [{
            let address = crate::game::actors::yellow_pipe_bug::state_address(game);
            sm_game::actors::species::pipe_bug_state(address, crate::ram_map::word!(game.cur_enemy_index), game);
        }];
        11769448 => [actors::botwoon::main_ai(&mut Runtime::new(game))];
        11771903 => [actors::botwoon::contact(false, &mut Runtime::new(game))];
        11771969 => [actors::botwoon::contact(true, &mut Runtime::new(game))];
        11771926 => [actors::botwoon::shot(&mut Runtime::new(game))];
        11789909 => [actor_species_pre_instruction(actors::escape_critters::main_ai(game), game)];

        10652144 => [{
            let index = game.cur_enemy_index();
            let preinstruction = Runtime::new(game)
                .resolve_actor(Key::from_legacy(index), AccessMode::LittleEndian)
                .pre_instruction;
            actor_species_pre_instruction(u32::from(preinstruction) | 0xa20000, game);
        }];
        10653138 => [{
            if !crate::game::maridia_turtles::advance(false, game) { crate::rtl::Unreachable(game); }
        }];
        10653998 => [{
            if !crate::game::maridia_turtles::advance(true, game) { crate::rtl::Unreachable(game); }
        }];
        10659396 => [crate::game::gunship::initialize_top(game)];
        10659538 => [crate::game::gunship::initialize_bottom(game)];
        10659673 => [{
            let phase = crate::game::gunship::advance(game);
            actor_species_pre_instruction(phase, game);
        }];
        10663426 => [crate::game::rinka::initialize(game)];
        10663876 => [{
            if !crate::game::rinka::advance(game) {
                crate::rtl::Unreachable(game);
            }
        }];
        10664263 => [crate::game::rinka::contact(game)];
        10664269 => [crate::game::rinka::shot(game)];
        10664909 => [crate::game::rio::initialize(game)];
        10664931 => [{
            game.next_random();
            if !crate::game::rio::advance(game) {
                crate::rtl::Unreachable(game);
            }
        }];
        10669331 => [{
            let action = large_snail::prepare(game);
            match action {
                0xcde6 | 0xce2b => {
                    let walking = action == 0xce2b;
                    let (raw, phase) = large_snail::begin_action(walking, game);
                    if let Some(phase) = phase {
                        actor_species_pre_instruction(0xa20000 | u32::from(phase), game);
                    }
                    if walking {
                        large_snail::walk(raw, game);
                    }
                }
                0xcf40 => large_snail::end_action(game),
                _ => {
                    crate::rtl::Unreachable(game);
                }
            }
            large_snail::finish(game);
        }];
        10674643 => [rippers::initialize(RipperKind::Gripper, game)];
        10674721 => [rippers::advance(RipperKind::Gripper, game)];
        10674968 => [rippers::initialize(RipperKind::JetPowered, game)];
        10675027 => [rippers::advance(RipperKind::JetPowered, game)];
        10675113 => [rippers::jet_shot(game)];
        10675359 => [rippers::initialize(RipperKind::Ripper, game)];
        10675418 => [rippers::advance(RipperKind::Ripper, game)];
        10657715 => [{
            let phase = {
                let runtime = Runtime::new(game);
                runtime.resolve_actor(Key::from_legacy(runtime.word(0xe54)), actors::access::AccessMode::Native).pre_instruction
            };
            actor_species_pre_instruction(0xa20000 | u32::from(phase), game);
        }];
        10662880 => [crate::game::erratic_fireball::update(true, game)];
        10662927 => [crate::game::erratic_fireball::update(false, game)];
        10665625 => [crate::game::lava_jumper::initialize(game)];
        10661995 => [crate::game::flies::initialize(game)];
        10666562 => [crate::game::norfair_rio::initialize(false, game)];
        10667763 => [crate::game::norfair_rio::initialize(true, game)];
        10649615 => [actors::combat::select_frozen_ai(&mut Runtime::new(game))];
        10649610 => [actors::combat::die(&mut Runtime::new(game), actors::api::DeathKind::Grapple)];
        10649665 => [actors::scheduling::update_frozen_enemy(&mut Runtime::new(game))];

        10670988 => [large_snail::touch(false, game)];
        10671028 => large_snail::shot;
        10658809 => ground_patrol::initialize_shell;
        10658942 => ground_patrol::advance_shell;
        10659193 => ground_patrol::shot_shell;
        10674038 => [crate::game::rising_falling::update(true, game)];
        10674222 => [crate::game::rising_falling::update(false, game)];
        10675718 => crate::game::lava_seahorse_pair::initialize;
        10675790 => [actor_species_pre_instruction(crate::game::lava_seahorse_pair::pre_instruction(game), game)];
        10676168 => [{ actors::combat::contact_reaction(&mut Runtime::new(game), actors::api::DeathHandling::Finalize); crate::game::lava_seahorse_pair::synchronize(game); }];
        10676174 => [{ actors::combat::shot_reaction(&mut Runtime::new(game), actors::api::ShotHandling::Finalize); crate::game::lava_seahorse_pair::synchronize(game); }];
        10676180 => [{ actors::combat::power_bomb_reaction(&mut Runtime::new(game), actors::api::DeathHandling::Finalize); crate::game::lava_seahorse_pair::synchronize(game); }];
        10663280 => [crate::game::lavaquake::update(true, game)];
        10663311 => [crate::game::lavaquake::update(false, game)];
        10653036 => [crate::game::maridia_turtles::initialize(false, game)];
        10653085 => [crate::game::maridia_turtles::initialize(true, game)];
        10654337 => [crate::game::maridia_turtles::touch(false, game)];
        10654367 => [crate::game::maridia_turtles::touch(true, game)];
        10654479 => crate::game::maridia_turtles::shot;
        10657608 => ground_patrol::initialize_plant;
        10652077 => crate::game::mini_crocomire::initialize;
        10651420 => crate::game::goofball::initialize;
        10651548 => crate::game::goofball::advance;
        10662175 | 10665682 | 10666615 | 10667812 => [{
            game.next_random();
            let phase = {
                let runtime = Runtime::new(game);
                let key = Key::from_legacy(runtime.word(0xe54));
                runtime.resolve_actor(key, AccessMode::Native).pre_instruction
            };
            actor_species_pre_instruction(0xa20000 | u32::from(phase), game);
        }];
        0xa30000..=0xa3ffff => [{ crate::rtl::Unreachable(game); }];
        10783748 => crate::game::visual_runtime::advance_crocomire;

        10780737 => [actors::scheduling::update_frozen_enemy(&mut Runtime::new(game))];

        10873386 => [actors::spore_spawn::initialize(game)];
        10873619 => [actors::spore_spawn::update(game)];
        10874202 => [actors::spore_spawn::shot(game)];
        10874348 => [actors::spore_spawn::touch(game)];
        10915836 => [crate::game::fire_geysers::initialize(game)];
        10915875 => [crate::game::fire_geysers::run(game)];
        0xa90000..=0xa9ffff => [sm_a9::DispatchEnemyAi(ea, game)];
        0xb30000..=0xb3ffff => [{ crate::rtl::Unreachable(game); }];
    }
}
crate::game::address_dispatch! {
    unsafe fn native_pre_instruction(ea: u32, k: u16, game: &mut Game) => ea;
        call (k, game); default { crate::rtl::Unreachable(game); };
    {


        10735526 => [maridia_snail::crawl(k, actor_api::Axis::Y, 0xa3cce2, game)];
        10735543 => [maridia_snail::crawl(k, actor_api::Axis::X, 0xa3ccea, game)];
        10735549 => [maridia_snail::crawl(k, actor_api::Axis::Y, 0xa3ccf2, game)];
        10735566 => [maridia_snail::crawl(k, actor_api::Axis::X, 0xa3ccfa, game)];
        10735572 => [maridia_snail::crawl(k, actor_api::Axis::Y, 0xa3cd02, game)];
        10735589 => [maridia_snail::crawl(k, actor_api::Axis::X, 0xa3cd0a, game)];
        10735595 => [maridia_snail::crawl(k, actor_api::Axis::Y, 0xa3cd12, game)];
        10735612 => [maridia_snail::crawl(k, actor_api::Axis::X, 0xa3cd1a, game)];
        10735456 => [maridia_snail::probe_edge(k, game)]; 10736051 => [maridia_snail::falling(k, game)];

        11047153 | 11047214 | 11047264 | 11047374 => [{
            if crate::game::actors::morph_ball_eye::run_function(ea as u16, k, game) {
                crate::game::visual_runtime::initialize_morph_eye_beam(game);
                crate::game::actors::morph_ball_eye::start_tracking(k, game);
            }
        }];

        11055620 => [{
            let (actor, diagnostic) = crate::game::actors::magdollite::grow_arm(game);
            if let Some(value) = diagnostic { warn_lava_height(value); }
            crate::game::actors::magdollite::finish_arm_growth(actor, game);
        }];
        11059843 | 11059878 | 11059919 | 11060075 | 11060193 | 11060329 | 11060444 | 11060480 | 11060509 | 11060584 | 11060585 | 11060639 => [crate::game::actors::maridia_floater::advance(ea as u16, k, game)];

        10665692 => [crate::game::lava_jumper::follow_parent(k, game)];
        10665754 => [crate::game::lava_jumper::launch(k, game)];
        10665790 => [crate::game::lava_jumper::advance(k, crate::game::lava_jumper::Flight::Rising, game)];
        10665852 => [crate::game::lava_jumper::advance(k, crate::game::lava_jumper::Flight::Turning, game)];
        10665916 => [crate::game::lava_jumper::advance(k, crate::game::lava_jumper::Flight::Falling, game)];
        10662222 => [crate::game::flies::circle(k, false, game)];
        10662268 => [crate::game::flies::circle(k, true, game)];
        10662314 => [crate::game::flies::dive(k, false, game)];
        10662354 => [crate::game::flies::dive(k, true, game)];
        10666625 => [crate::game::norfair_rio::follow_parent(k, false, game)];
        10666727 => [crate::game::norfair_rio::wait_to_dive(k, false, game)];
        10666815 => [crate::game::norfair_rio::finish_animation(k, false, false, game)];
        10667014 => [crate::game::norfair_rio::finish_animation(k, false, true, game)];
        10667822 => [crate::game::norfair_rio::follow_parent(k, true, game)];
        10667889 => [crate::game::norfair_rio::wait_to_dive(k, true, game)];
        10667963 => [crate::game::norfair_rio::finish_animation(k, true, false, game)];
        10668168 => [crate::game::norfair_rio::finish_animation(k, true, true, game)];
        10666849 => [crate::game::norfair_rio::dive(k, false, true, game)];
        10666929 => [crate::game::norfair_rio::dive(k, false, false, game)];
        10667990 => [crate::game::norfair_rio::dive(k, true, true, game)];
        10668077 => [crate::game::norfair_rio::dive(k, true, false, game)];
        10669926 => [large_snail::bounce(false, k, game)];
        10669993 => [large_snail::bounce(true, k, game)];
        10675796 | 10675885 | 10675953 | 10676020 | 10676041
            => [crate::game::lava_seahorse_pair::phase(k, ea as u16, game)];
        10657722 => [ground_patrol::advance_plant(false, game)];
        10657772 => [ground_patrol::advance_plant(true, game)];
        10652227 => [crate::game::mini_crocomire::patrol(false, game)];
        10652252 => [crate::game::mini_crocomire::patrol(true, game)];
        10659852 => [crate::game::gunship::phase(k, 0xa80c, game)];
        10660048 => [crate::game::gunship::phase(k, 0xa8d0, game)];
        10660162 => [crate::game::gunship::phase(k, 0xa942, game)];
        10660176 => [crate::game::gunship::phase(k, 0xa950, game)];
        10660231 => [crate::game::gunship::phase(k, 0xa987, game)];
        10660285 => [crate::game::gunship::phase(k, 0xa9bd, game)];
        10660431 => [crate::game::gunship::phase(k, 0xaa4f, game)];
        10660445 => [crate::game::gunship::phase(k, 0xaa5d, game)];
        10660500 => [crate::game::gunship::phase(k, 0xaa94, game)];
        10660514 => [crate::game::gunship::phase(k, 0xaaa2, game)];
        10660639 => [crate::game::gunship::phase(k, 0xab1f, game)];
        10660704 => [crate::game::gunship::phase(k, 0xab60, game)];
        10660718 => [crate::game::gunship::phase(k, 0xab6e, game)];
        10660773 => [crate::game::gunship::phase(k, 0xaba5, game)];
        10660807 => [crate::game::gunship::phase(k, 0xabc7, game)];
        10660891 => [crate::game::gunship::phase(k, 0xac1b, game)];
        10661079 => [crate::game::gunship::phase(k, 0xacd7, game)];
        10661134 => [crate::game::gunship::phase(k, 0xad0e, game)];
        10661165 => [crate::game::gunship::phase(k, 0xad2d, game)];
    }
}
crate::game::callback_dispatch! {
    unsafe fn native_instruction(ea: u32, k: u16, j: EnemyScript, game: &mut Game) -> EnemyScript => ea;
    call (k, j, game); default { crate::rtl::Unreachable(game); EnemyScript::Stop };
    @external {
        11763819 | 11763828 => [{
            use crate::types::EnemyData;
            let value = if ea == 11763819 { j.word(0) } else { 0x807b };
            crate::ram_map::slot!(game, EnemyData[i32::from(k)].ai_preinstr = value);
            if ea == 11763819 { j.offset(1) } else { j }
        }];

        10651845 => [j];
        10716899 => crate::game::waver::Waver_Instr_1;
        10734646 => [maridia_snail::instruction(k, 0, j, game)];
        10734655 => [maridia_snail::instruction(k, 1, j, game)];
        10734664 => [maridia_snail::instruction(k, 2, j, game)];
        10734687 => [maridia_snail::instruction(k, 3, j, game)];
        10734712 => [maridia_snail::instruction(k, 4, j, game)];
        10742437 => [metroid::sound(false, j, game)]; 10742449 => [metroid::sound(true, j, game)];
        11056990 => [j];
        11068957 | 11068970 => [crate::game::actors::wrecked_ship_spark::instruction(ea == 11068957, j, game)];
        11062537 | 11062692 | 11062762 | 11062917 | 11062987 | 11062991 | 11063146 | 11063216 | 11063371 | 11063441 | 11063490 | 11063494 | 11063506 | 11063552 | 11063559 | 11063601 | 11063613 | 11063659 => [crate::game::actors::wrecked_ship_robots::instruction(ea, k, j, game)];
        11072806 | 11072996 | 11073151 | 11073234 | 11073240 => [crate::game::actors::ki_hunter::instruction(ea, k, j, game)];
        11128756 => [sm_game::actors::baby_metroid_cutscene::BabyMetroid_Instr_1(k, j)];
        11128778 => [sm_game::actors::baby_metroid_cutscene::BabyMetroid_Instr_2(k, j)];
        11139360 => [sm_game::actors::baby_metroid_encounter::BabyMetroid_Instr_3(k, j)]; 11139382 => [sm_game::actors::baby_metroid_encounter::BabyMetroid_Instr_4(k, j)];
        11139472 => [sm_game::actors::baby_metroid_encounter::BabyMetroid_Instr_6(k, j)];
        10664371 => [{
            crate::game::rinka::hide(k, false, game);
            j
        }];
        10664381 => [{
            crate::game::rinka::hide(k, true, game);
            j
        }];
        10664391 => [{
            crate::game::rinka::launch(k, game);
            j
        }];
        10664899 => [{
            crate::game::rio::instruction_finished(game);
            j
        }];
        10665614 => [{
            crate::game::lava_jumper::instruction_finished(game);
            j
        }];
        10668907 => [{ crate::game::music::QueueSfx2_Max6(0xe, game); j }];
        10669235 => [{ large_snail::script_flag(4, 1, game); j }];
        10669246 => [{ large_snail::script_flag(6, 1, game); j }];
        10669257 => [{ large_snail::script_flag(6, 0, game); j }];
        10675707 => [crate::game::lava_seahorse_pair::fire_ready(j, game)];
        10666441 | 10666452 | 10666463 | 10666474 | 10666485 | 10666496 | 10666507
        | 10666518 | 10666529 | 10666540 | 10666551 | 10667730 | 10667741 | 10667752
            => [crate::game::norfair_rio::instruction(ea, j, game)];
        10654593 => [crate::game::maridia_turtles::instruction(0x9381, j, game)];
        10654738 => [crate::game::maridia_turtles::instruction(0x9412, j, game)];
        10654791 => [crate::game::maridia_turtles::instruction(0x9447, j, game)];
        10654801 => [crate::game::maridia_turtles::instruction(0x9451, j, game)];
        10654827 => [crate::game::maridia_turtles::instruction(0x946b, j, game)];
        10654853 => [crate::game::maridia_turtles::instruction(0x9485, j, game)];
        10654881 => [crate::game::maridia_turtles::instruction(0x94a1, j, game)];
        10654919 => [crate::game::maridia_turtles::instruction(0x94c7, j, game)];
        10654929 => [{ crate::game::music::QueueSfx2_Max6(0x3a, game); j }];
        10657578 => [{ crate::game::music::QueueSfx2_Max6(0x34, game); j }];
        10659181 => [{ ground_patrol::direct_shell(k, 0, game); j }];
        10659185 => [{ ground_patrol::direct_shell(k, 1, game); j }];
        10657941 => [{ ground_patrol::resume_plant(k, game); j }];
        10657959 => [crate::game::enemy_projectiles::spawn_scripted(0xdafe, j, game)];
        10652030 => [crate::game::enemy_projectiles::spawn_scripted(0xdbf2, j, game)];
        10652048 => [crate::game::mini_crocomire::resume(false, j, game)];
        10652061 => [crate::game::mini_crocomire::resume(true, j, game)];
        10651846 => [crate::game::goofball::landed(j, game)];
        0xa30000..=0xa3ffff => [{ crate::rtl::Unreachable(game); EnemyScript::Stop }];
        10872671 => [{ actors::spore_spawn::increase_max_x_radius(game); j }];
        10872689 => [{ actors::spore_spawn::clear_damaged_flag(game); j }];
        10872877 => [actors::spore_spawn::set_max_x_radius_and_angle_delta(j, game)];
        10872946 => [{ crate::ram_map::word!(game.kraid_unk9000 = j.word(0)); j.offset(1) }];
        10872956 => [{ actors::spore_spawn::harden(game); j }];
        10872981 => [{ crate::game::music::QueueSfx2_Max6(j.word(0), game); j.offset(1) }];
        10873018 => [{ actors::spore_spawn::set_function(j.word(0), game); j.offset(1) }];
        10873034 => [actors::spore_spawn::fade(false, j, game)];
        10873116 => [actors::spore_spawn::fade(true, j, game)];
        10873198 => [actors::spore_spawn::emit_spore(j, game)];
        10873265 => [actors::spore_spawn::emit_hurt(j, game)];
        10873009 => [{ crate::game::enemy_projectiles::scatter_drop(sm_game::enemy_projectiles::drops::DropSource::SporeSpawn, k, game); j }];
        10852501 => [{ crate::ram_map::word!(game.room_loading_irq_handler = 12); j }];
        10853514 => [{ crate::ram_map::word!(game.room_loading_irq_handler = 12); j }];
        10854240 => [{ crate::game::music::QueueSfx2_Max6(j.word(0), game); j.offset(1) }];
        10854254 => [{ crate::game::music::QueueSfx3_Max6(j.word(0), game); j.offset(1) }];
        10852159 => [{ crate::game::sprite_effect_runtime::draygon_random_sprite(21, game); j }];
        10852178 => [{ crate::game::sprite_effect_runtime::draygon_random_sprite(3, game); j }];
        10852197 => [{ crate::game::sprite_effect_runtime::draygon_random_sprite(29, game); j }];
        10852216 => [{ crate::game::sprite_effect_runtime::draygon_random_sprite(24, game); j }];
        10854268 => [{ crate::game::sprite_effect_runtime::draygon_spit(false, game); j }];
        10854318 => [{ crate::game::sprite_effect_runtime::draygon_spit(true, game); j }];
        10915247 | 10915347..=10915793 => [crate::game::fire_geysers::instruction(ea, j, game)];
        10940701 => [crate::game::ceres_doors::change_properties(k, sm_game::actors::ceres_doors::VANISH, 0, j, game)];
        10940725 => [crate::game::ceres_doors::change_properties(k, 0, sm_game::actors::ceres_doors::VANISH, j, game)];
        10942014 => crate::game::ceres_doors::branch_unless_samus_near;
        10942058 => [crate::game::ceres_doors::branch_unless_boss_defeated(j, game)];
        10942072 => [crate::game::ceres_doors::branch_unless_escape(j, game)];
        10942091 => [crate::game::ceres_doors::change_properties(k, crate::game::ceres_doors::TANGIBLE, 0, j, game)];
        10942101 => [crate::game::ceres_doors::change_properties(k, 0, crate::game::ceres_doors::TANGIBLE, j, game)];
        10942111 => [crate::game::ceres_doors::mark_open(k, true, j, game)];
        10942118 => [crate::game::ceres_doors::change_properties(k, crate::game::ceres_doors::INVISIBLE, 0, j, game)];
        10942128 => [crate::game::ceres_doors::mark_open(k, false, j, game)];
        10942131 => [crate::game::ceres_doors::change_properties(k, 0, crate::game::ceres_doors::INVISIBLE, j, game)];
        10942141 => [crate::game::ceres_doors::play_open_sound(j, game)];

        0xa90000..=0xa9ffff => [sm_a9::DispatchEnemyInstr(ea, k, j, game)];
        11174004 => [{ crate::ram_map::slot!(game, EnemyData[k as i32].ai_preinstr = (0xaa807bi32) as u16); j }];
        11173995 => [{ crate::ram_map::slot!(game, EnemyData[k as i32].ai_preinstr = j.word(0)); j.offset(1) }];


        11769031 | 11769095 | 11769111 | 11769175 => [{ actors::botwoon::set_hitbox(8, 16, game); j }];
        11769047 | 11769079 | 11769127 | 11769159 => [{ actors::botwoon::set_hitbox(12, 12, game); j }];
        11769063 | 11769143 => [{ actors::botwoon::set_hitbox(16, 8, game); j }];
        11769191 => [{ actors::botwoon::set_spitting(game); j }];
        11769202 => [{ crate::game::music::QueueSfx2_Max6(0x7c, game); j }];
        0xb30000..=0xb3ffff => [{ crate::rtl::Unreachable(game); EnemyScript::Stop }];
    }
}
impl Runtime<'_> {
    pub(crate) fn actor_ai_legacy(&mut self, routine: u32) {
        match routine {
            11769448 => self.owned_call(actors::botwoon::main_ai),
            10920584 => self.owned_call(sm_game::actors::ceres_ridley::main),
            10924583 => self.owned_call(sm_game::actors::ridley::main),
            10988577 => self.owned_call(sm_game::actors::kraid::main_ai),
            10997414 => self.owned_call(sm_game::actors::phantoon::main_ai),
            10847996 | 10863750 => self.owned_call(|runtime| {
                use sm_game::actors::{draygon_flight, species};
                draygon_flight::main_ai(routine == 10847996, species::draygon_function, runtime)
            }),
            11771926 => self.owned_call(actors::botwoon::shot),
            11771903 | 11771969 => {
                self.owned_call(|runtime| actors::botwoon::contact(routine == 11771969, runtime))
            }
            _ => self.foreign(|game| sm_game::actors::species::ai(routine, game)),
        }
    }
    pub(crate) fn actor_instruction_legacy(
        &mut self,
        routine: u32,
        actor: u16,
        script: EnemyScript,
    ) -> EnemyScript {
        self.foreign(|game| sm_game::actors::species::instruction(routine, actor, script, game))
    }
    pub(crate) fn actor_touch_legacy(&mut self, routine: u32) {
        self.foreign(|game| sm_game::actors::species::touch(routine, game));
    }
    pub(crate) fn actor_shot_legacy(&mut self, routine: u32, argument: u16) {
        self.foreign(|game| sm_game::actors::species::shot(routine, argument, game));
    }
    pub(crate) fn actor_hook_legacy(&mut self, routine: u32) {
        self.foreign(|game| sm_game::actors::species::draw_hook(routine, game));
    }
    pub(crate) fn produce_player_scene(&mut self) {
        self.owned_player_call(|runtime| {
            use crate::{game::scene::state::DisplayError, game_state::SamusExecution};
            // The former actor foreign call rejected Prepared at its first import.
            let ready = match runtime.game.samus_execution() {
                Ok(Some(SamusExecution::Prepared)) => Err(DisplayError::Phase),
                result => result.map(|_| ()),
            };
            ready.expect("Samus import rejected");
            sm_game::player_drawing::draw_player_frame(runtime);
        });
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
            let end = destination + bytes;
            assert!(destination <= end && end <= 0x20000);
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
        self.foreign(|_game| legacy_assertion(drawing));
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
pub(crate) use player_collision_runtime::set_player_radius;
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
) -> sm_game::session::Vector32 {
    let [x, y] = [0x40u16, 0x80].map(|offset| {
        let index = usize::from(phase.wrapping_add(offset) & 0x7f);
        u32::from(game.read_rom_word(0xa0b7ee, index * 2)) * u32::from(amplitude)
    });
    sm_game::session::Vector32 { x, y }
}

// Shared native diagnostic; callers retain their own publication boundary.
pub(crate) fn legacy_assertion(drawing: bool) -> ! {
    unsafe {
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
    }
}

// The grapple runs as one owned-Session lease: its RAM words, Samus facts and
// player rules read and write the lease, and only its game-side effects (sound,
// unreachable failure, pose command) go out through foreign.
impl sm_game::ram::CompatibilityRam for Runtime<'_> {
    fn compatibility_byte(&self, at: usize) -> u8 {
        Runtime::byte(self, at)
    }
    fn compatibility_write_byte(&mut self, at: usize, value: u8) {
        Runtime::set_byte(self, at, value)
    }
}
impl sm_game::effects::SoundEffects for Runtime<'_> {
    fn queue_sound(&mut self, channel: usize, sound: u8, max_queued: u8) {
        // As Runtime's projectile sounds: an owned Display queues on the
        // lease, otherwise the compat queue runs on the exported game.
        if self.has_owned_display() {
            self.owned_call(|runtime| {
                #[cfg(sm_verify_dll)]
                crate::game::visual_runtime::samus_observer::sound(channel, sound, max_queued);
                crate::game::music::requests::queue_sfx(channel, sound, max_queued, runtime)
                    .expect("audio sound enqueue");
            });
            return;
        }
        self.foreign(|game| crate::game::music::queue_sfx(channel, sound, max_queued, game));
    }
}
impl sm_game::cartridge::LoadedImage for Runtime<'_> {
    fn loaded_image(&self) -> &crate::rom::RomImage {
        self.game.loaded_rom.as_ref().expect("ROM must be loaded")
    }
}
impl sm_game::cartridge::NativeImage for Runtime<'_> {
    fn native_image(&self) -> crate::rom::RomImage {
        self.game.capture_native_rom()
    }
}
impl sm_game::records::Records for Runtime<'_> {
    fn records(&self) -> &sm_game::records::RomRecords {
        &self.game.records
    }
}
impl sm_game::player_position::PlayerFacts for Runtime<'_> {
    fn facing(&self) -> u8 {
        Runtime::byte(self, 0xa1e)
    }
    fn movement_type(&self) -> u8 {
        Runtime::byte(self, 0xa1f)
    }
    fn grapple_angle(&self) -> u8 {
        Runtime::byte(self, 0xcfb)
    }
    fn grapple_mirrored_angle(&self) -> u8 {
        Runtime::byte(self, 0xcfd)
    }
}
impl<'g> sm_game::grapple_swing::Host for Runtime<'g> {
    type Lease<'a>
        = Runtime<'g>
    where
        Self: 'a;
    fn with_lease<'s, R>(&'s mut self, action: impl FnOnce(&mut Self::Lease<'s>) -> R) -> R {
        action(self)
    }
    fn grapple_unreachable(&mut self) -> bool {
        self.foreign(|game| sm_game::menus::host::MenuRuntime::rtl_unreachable(game))
    }
    fn grapple_pose_command(&mut self, command: u16) -> u16 {
        // The command runs on this lease; no export, nested lease or re-import.
        player_transitions_runtime::command_in(self, command)
    }
    fn multiply_with_scratch(&mut self, a: u16, b: u16) -> u32 {
        self.write_slice(0x5e9, &a.to_le_bytes());
        self.write_slice(0x5eb, &b.to_le_bytes());
        (a as u32).wrapping_mul(b as u32)
    }
    fn player_bottom(&self) -> u16 {
        use sm_game::player_position::PlayerPosition as _;
        let y = self.player_y();
        // SamusPoseParams has eight bytes, with y_radius at byte 6.
        let pose_offset = usize::from(Runtime::word(self, 0xa1c)) * 8;
        let radius = self.game.read_rom_byte(0x91b629, pose_offset + 6);
        y.wrapping_add(u16::from(radius)).wrapping_sub(1)
    }
    fn settle_grapple(&mut self) {
        crate::game::player_collision::operations::settle_after_grapple(self);
    }
    fn block_behavior_word(&self, index: usize) -> i16 {
        let at = 0x16402 + index;
        i16::from_ne_bytes([Runtime::byte(self, at), Runtime::byte(self, at + 1)])
    }
    fn periodic_damage(&self) -> (u16, u16) {
        let low = u16::from_ne_bytes([Runtime::byte(self, 0xa4e), Runtime::byte(self, 0xa4f)]);
        let high = u16::from_ne_bytes([Runtime::byte(self, 0xa50), Runtime::byte(self, 0xa51)]);
        (low, high)
    }
}
