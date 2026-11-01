//! One aggregate authority for player, projectiles, mechanisms and their room.
use crate::game::enemy_speeds::SpeedTable;
use crate::{
    game_state::Game,
    rom::{RomImage, RomRecord},
};
use sm_game::display::sequence::Registers;
use sm_game::player_projectiles::{
    access::{ShotLane, Store},
    content,
    host::{Drawing, Host, PaletteEffect, PlayerStep},
};
use sm_game::scene::owned_samus::SamusSources as _;
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

/// Runs the bank $AA function at `address` for the current enemy. Was in bank $AA.
pub(crate) fn run_aa_function(address: u32, game: &mut Game) {
    unsafe { actor_species_pre_instruction(address, game) };
}

/// Runs the bank $A7 function at `address` for the current enemy. Was in bank $A7.
pub(crate) fn run_a7_function(address: u32, game: &mut Game) {
    unsafe { actor_species_pre_instruction(address, game) };
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
    fn warn_reflection(&mut self) {
        unsafe { crate::c_library::printf(c"Possible bug. What is X?\n".as_ptr()) };
    }
    fn write_multiplicand(&mut self, operand: u8) {
        unsafe { crate::rtl::write_reg(crate::rtl::WRMPYA as u16, operand, self) };
    }
    fn install_draygon_layers(&mut self) {
        crate::game::visual_runtime::initialize_draygon_layer(self);
    }
}

impl sm_game::actors::species::PreInstructionHost for Game {
    fn warn_lava_height(&mut self, value: i32) {
        unsafe { crate::c_library::printf(c"Y undefined %d!\n".as_ptr(), value) };
    }
    fn initialize_morph_eye_beam(&mut self) {
        crate::game::visual_runtime::initialize_morph_eye_beam(self);
    }
    fn native_pre_instruction(&mut self, _ea: u32, _k: u16) {
        // Every pre-instruction routine is in sm_game's species table now.
        crate::rtl::report_unreachable(self);
    }
}

impl sm_game::actors::species::InstructionHost for Game {
    fn native_instruction(&mut self, ea: u32, _actor: u16, j: EnemyScript) -> EnemyScript {
        native_instruction(ea, j, self)
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
        let (state, inputs) = reuse_frame_session(game).unwrap_or_else(|| {
            let state = import_session(game);
            (state, encoding::inputs(&|at| game.ram().read_byte(at)))
        });
        Self {
            state: Some(state),
            inputs,
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
    /// Whether this lease is current and its Session owns the display.
    pub(crate) fn display_owned(&self) -> bool {
        self.persistent_is_current() && self.session().presentation.is_owned()
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
            sm_game::scene::owned_samus::Sprite {
                producer,
                index,
                position,
            },
        )
        .expect("owned Samus publication");
    }
    pub fn foreign<R>(&mut self, action: impl FnOnce(&mut Game) -> R) -> R {
        assert!(self.persistent_is_current(), "Session lease is dirty");
        preflight_export(self.session(), self.game);
        let mut mutation = self.game.samus_mutation();
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
        // The former foreign entry imported these fresh, so refresh them on entry too.
        call.runtime.refresh_actor_observations();
        let result = action(&mut *call.runtime);
        call.runtime.refresh_actor_observations();
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
    fn refresh_actor_observations(&mut self) {
        self.inputs.enemy_zero_health = self.native_word(0xF8C);
        self.inputs.current_enemy = self.word(0xE54);
        self.inputs.random_enemy_counter = self.word(0xE44);
        self.inputs.debug_enemy_freeze = self.word(0x185E);
    }
    pub(crate) fn refresh_player_observations(&mut self) {
        assert!(self.persistent_is_current(), "Session lease is dirty");
        self.inputs = encoding::inputs(&|at| self.byte(at));
    }
    fn relinquish(&mut self) {
        if let Some(state) = self.state.take() {
            // The verifier observes the engine's typed Session as the lease releases it.
            #[cfg(sm_verify_dll)]
            if let Some(observer) = self.game.observation.as_mut() {
                observer.leases.capture(&state);
            }
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
                let kept = self.game.frame_session.active.then(|| state.clone());
                export_session_preflighted(state, self.game);
                if let Some(mut kept) = kept {
                    kept.actor_identity =
                        crate::game::actors::identity::ActorIdentityState::empty();
                    let epoch = self.game.compatibility_write_epoch();
                    self.game.frame_session.kept = Some((kept, self.inputs.clone(), epoch));
                }
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
    /// Store a Session field while this lease is current; the projected word
    /// at `at`, low byte then high, otherwise.
    pub(crate) fn store_owned_session(
        &mut self,
        field: impl FnOnce(&mut Session) -> &mut u16,
        at: usize,
        value: u16,
    ) {
        if self.persistent_is_current() {
            *field(self.session_mut()) = value;
        } else {
            self.write_word_le(value, at);
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
        use sm_game::actors::layouts::EnemyLayout104 as Maw;
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
    fn sound(&mut self, bank: u8, limit: u8, sound: u16) -> u16 {
        if self.has_owned_display() {
            return self.owned_call(|runtime| {
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
                    .expect("audio sound enqueue")
            });
        }
        self.foreign(|g| {
            if !matches!(
                (bank, limit),
                (1, 6 | 9 | 15) | (2, 6) | (3, 3 | 6 | 9 | 15)
            ) {
                unreachable!("unknown projectile sound queue");
            }
            crate::game::music::queue_sfx(usize::from(bank - 1), sound as u8, limit, g)
        })
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
        self.foreign(|g| sm_game::explosion_window::spawn_power_bomb(g));
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
            use sm_game::sprite_output as output;
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
        self.foreign(|g| crate::rtl::report_unreachable(g))
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

/// The Session a frame's last lease exported, kept while nothing has written the compatibility
/// RAM since, so the frame's next lease borrows it rather than decoding the RAM again. Only a
/// frame tick (`with_frame_session`) keeps one.
#[derive(Default)]
pub(crate) struct FrameSession {
    active: bool,
    kept: Option<(Session, ExternalInputs, u64)>,
}

/// Run a frame tick with its leases sharing one Session while the RAM is unchanged.
pub(crate) fn with_frame_session<R>(game: &mut Game, tick: impl FnOnce(&mut Game) -> R) -> R {
    game.frame_session.active = true;
    let result = tick(game);
    game.frame_session = FrameSession::default();
    result
}

/// The kept Session for a new lease, when the RAM, world and Samus state still admit it; it
/// takes the native spawn fields and actor identity as an import would.
fn reuse_frame_session(game: &mut Game) -> Option<(Session, ExternalInputs)> {
    let (mut state, inputs, epoch) = game.frame_session.kept.take()?;
    let same_world = std::sync::Arc::ptr_eq(&state.world, game.world())
        && state.world.revision() == game.world().revision();
    if epoch != game.compatibility_write_epoch() || !same_world {
        return None;
    }
    game.preflight_samus_runtime()
        .expect("Samus import rejected");
    #[cfg(sm_verify_dll)]
    player_normal_transfers::record(game, 1);
    import_native_spawn(&mut state, game);
    #[cfg(sm_verify_dll)]
    {
        let mut fresh = encoding::decode(
            &|at| game.ram().read_byte(at),
            0x20000,
            game.world().clone(),
        );
        import_native_spawn(&mut fresh, game);
        assert!(
            fresh == state,
            "the kept frame Session differs from its RAM projection"
        );
    }
    state.actor_identity = game.take_actor_identity();
    Some((state, inputs))
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
#[cfg(sm_verify_dll)]
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
    fn refresh_inputs(&mut self) {
        self.refresh_player_observations();
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

use crate::game::actors;
use crate::game::sm_a9;
use sm_game::enemy_script::EnemyScript;

crate::game::callback_dispatch! {
    unsafe fn native_ai(ea: u32, game: &mut Game) => ea;
    call (game); default { crate::rtl::report_unreachable(game); };
    @external {
        10649676 => [{
            #[cfg(sm_verify_dll)]
            crate::game::actors::reference_population_animation::dispatch_noop(game);
        }];
        11111301 => [{
                    sm_a9::tubes_falling_main_ai(crate::ram_map::word!(game.CUR_ENEMY_INDEX), game);
                }];
        10783748 => crate::game::visual_runtime::advance_crocomire;
        0xa90000..=0xa9ffff => [sm_a9::dispatch_enemy_ai(ea, game)];
    }
}
/// The instructions still in the compatibility crate: Mother Brain's body's, in bank $A9.
fn native_instruction(ea: u32, j: EnemyScript, game: &mut Game) -> EnemyScript {
    if ea >> 16 == 0xa9 {
        sm_a9::dispatch_enemy_instr(ea, j, game)
    } else {
        crate::rtl::report_unreachable(game);
        EnemyScript::Stop
    }
}
impl Runtime<'_> {
    pub(crate) fn actor_ai_legacy(&mut self, routine: u32) {
        if self.kraid_and_hosts_ai(routine) {
            return;
        }
        if sm_game::actors::species::lease_ai(routine, self) || self.bank_a9_routine(routine) {
            return;
        }
        match routine {
            11769448 => self.owned_call(actors::botwoon::main_ai),
            10920181 => self.owned_call(sm_game::actors::ceres_ridley::init),
            10920584 => self.owned_call(sm_game::actors::ceres_ridley::main),
            10920659 => self.owned_call(sm_game::actors::ceres_ridley::hurt),
            10924583 => self.owned_call(sm_game::actors::ridley::main),
            10924682 => self.owned_call(sm_game::actors::ridley::time_frozen),
            10924695 => self.owned_call(sm_game::actors::ridley::hurt),
            10929814 => self.owned_call(sm_game::actors::ridley::explosion_init),
            10930388 => self.owned_call(sm_game::actors::ridley::explosion_main),
            10936202 => self.owned_call(sm_game::actors::ridley::shot),
            10936242 => self.owned_call(sm_game::actors::ridley::power_bomb),
            10940337 => self.owned_call(sm_game::actors::ceres_doors::initialize_steam),
            10940429 => self.owned_call(sm_game::actors::ceres_doors::steam_main),
            10942149 => self.owned_call(sm_game::actors::ceres_doors::initialize_door),
            10942309 => self.owned_call(sm_game::actors::ceres_doors::door_main),
            10943346 => self.owned_call(sm_game::actors::zebetites::initialize),
            10943539 => self.owned_call(sm_game::actors::zebetites::main_ai),
            10943911 => self.owned_call(sm_game::actors::zebetites::touch),
            10943916 => self.owned_call(sm_game::actors::zebetites::shot),
            11766625 => self.owned_call(sm_game::actors::gamet::initialize),
            11766686 => self.owned_call(sm_game::actors::gamet::main_ai),
            11068296 => self.owned_call(actors::atomic::initialize),
            11068355 => self.owned_call(actors::atomic::main_ai),
            10718509 => self.owned_call(actors::fireflea::initialize),
            10718702 => self.owned_call(actors::fireflea::main_ai),
            10718827 => self.owned_call(actors::fireflea::touch),
            10718851 => self.owned_call(actors::fireflea::power_bomb),
            10718857 => self.owned_call(actors::fireflea::shot),
            10719413 => self.owned_call(actors::skultera::initialize),
            10719531 => self.owned_call(actors::skultera::main_ai),
            11197603 => self.owned_call(actors::shaktool::main_ai),
            11198019 => self.owned_call(actors::shaktool::initialize),
            11198255 => self.owned_call(actors::shaktool::touch),
            11198260 => self.owned_call(actors::shaktool::shot),
            11065545 => self.owned_call(sm_game::actors::bull::initialize),
            11065611 => self.owned_call(sm_game::actors::bull::main_ai),
            11066132 => self.owned_call(sm_game::actors::bull::shot),
            11071880 => self.owned_call(sm_game::actors::kihunter::initialize),
            11072020 => self.owned_call(sm_game::actors::kihunter::initialize_wings),
            11072092 | 11072098 => self.owned_call(sm_game::actors::kihunter::main_ai),
            11073281 => self.owned_call(sm_game::actors::kihunter::shot),
            10988577 => self.owned_call(sm_game::actors::kraid::main_ai),
            10997414 => self.owned_call(sm_game::actors::phantoon::main_ai),
            11191972 => self.owned_call(sm_game::actors::torizo::main_ai),
            11195241 => self.owned_call(sm_game::actors::torizo::golden_main_ai),
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
        use sm_game::actors::{ridley, species};
        let script = match self.kraid_and_hosts_instruction(routine, actor, script) {
            Ok(script) => return script,
            Err(script) => script,
        };
        let script = match species::lease_instruction(routine, actor, script, self) {
            Ok(script) => return script,
            Err(script) => script,
        };
        let mut script = match self.bank_a9_instruction(routine, actor, script) {
            Ok(script) => return script,
            Err(script) => script,
        };
        match routine {
            11072806 | 11072996 | 11073151 | 11073234 | 11073240 => {
                return self.owned_call(|runtime| {
                    sm_game::actors::kihunter::instruction(runtime, routine, actor, script)
                });
            }
            11196721 | 11196735 | 11196746 | 11196755 | 11196858 => {
                return self.owned_call(|runtime| {
                    actors::shaktool::instruction(routine, actor, script, runtime)
                });
            }
            10719382 | 10719392 | 10719402 => {
                return self
                    .owned_call(|runtime| actors::skultera::instruction(routine, script, runtime));
            }
            10940711 => {
                return self.owned_call(|runtime| {
                    sm_game::actors::ceres_doors::steam_activate(actor, script, runtime)
                });
            }
            10940701 | 10940725 | 10942014 | 10942058 | 10942072 | 10942091 | 10942101
            | 10942111 | 10942118 | 10942128 | 10942131 | 10942141 => {
                return self.owned_call(|runtime| {
                    sm_game::actors::ceres_doors::instruction(routine, actor, script, runtime)
                });
            }
            _ => {}
        }
        if routine >> 16 == 0xa6 {
            match self.owned_call(|runtime| ridley::instruction(routine, script, runtime)) {
                Ok(next) => return next,
                Err(unrun) => script = unrun,
            }
        }
        self.foreign(|game| species::instruction(routine, actor, script, game))
    }
    pub(crate) fn actor_touch_legacy(&mut self, routine: u32) {
        use sm_game::actors::species;
        if routine == 10940479 {
            self.owned_call(sm_game::actors::ceres_doors::steam_touch);
        } else if !self.bank_a9_routine(routine) && !species::lease_touch(routine, self) {
            self.foreign(|game| species::touch(routine, game));
        }
    }
    pub(crate) fn actor_shot_legacy(&mut self, routine: u32, argument: u16) {
        use sm_game::actors::species;
        if routine == 10936202 {
            self.owned_call(sm_game::actors::ridley::shot);
        } else if !self.bank_a9_routine(routine) && !species::lease_shot(routine, self) {
            self.foreign(|game| species::shot(routine, argument, game));
        }
    }
    pub(crate) fn actor_hook_legacy(&mut self, routine: u32) {
        use sm_game::actors::species;
        if routine == 10920690 {
            self.owned_call(sm_game::actors::ridley::draw_baby_metroid_and_door);
        } else if !self.bank_a9_routine(routine) && !species::lease_draw_hook(routine, self) {
            self.foreign(|game| species::draw_hook(routine, game));
        }
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
            use sm_game::sprite_drawing::Clip;
            let clip = if base {
                Clip::None
            } else if offscreen {
                Clip::EnemyOutside
            } else {
                Clip::EnemyInside
            };
            sm_game::sprite_output::with_base_tile(
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
            use sm_game::sprite_drawing::Clip;
            use sm_game::sprite_output::{Native, sprite_map};
            if base && address == 0 {
                return;
            }
            let map = sprite_map(bank, address, game, &mut Native);
            let clip = if base {
                Clip::None
            } else if offscreen {
                Clip::EnemyOutside
            } else {
                Clip::EnemyInside
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
// A register write may read or change RAM (DMA, the WRAM port), so each one
// runs on the exported game and the lease re-imports what it left.
impl Registers for Runtime<'_> {
    fn write_register(&mut self, register: u16, value: u8) {
        self.foreign(|game| Registers::write_register(game, register, value));
    }
    fn write_register_word(&mut self, register: u16, value: u16) {
        self.foreign(|game| Registers::write_register_word(game, register, value));
    }
    fn read_register_word(&mut self, register: u16) -> u16 {
        self.foreign(|game| Registers::read_register_word(game, register))
    }
    fn read_register(&mut self, register: u16) -> u8 {
        self.foreign(|game| Registers::read_register(game, register))
    }
}
// The lease's projection stores display mirrors in the owned display state.
impl sm_game::display_refresh::DisplayOwner for Runtime<'_> {
    fn write_owned_byte(&mut self, at: usize, value: u8) {
        Runtime::set_byte(self, at, value);
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
impl sm_game::mother_brain_visuals::PaletteHost for Runtime<'_> {
    fn mother_brain_visuals(&self) -> &sm_game::mother_brain_visuals::ImportedTables {
        &self.game.mother_brain_visuals
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
/// Actor work the scheduler runs inside this lease stays on it, as an owned call.
impl<'g> sm_game::actors::host::LeaseScope for Runtime<'g> {
    type Scoped<'a>
        = Runtime<'g>
    where
        Self: 'a;
    fn with_lease<'s, R>(&'s mut self, action: impl FnOnce(&mut Self::Scoped<'s>) -> R) -> R {
        self.owned_call(action)
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

// Kraid's, Phantoon's and the other bank $A6, $A7 and $AA hosts' entry points,
// as owned calls on the scheduler's lease.
impl Runtime<'_> {
    fn kraid_and_hosts_ai(&mut self, routine: u32) -> bool {
        use sm_game::actors::{boulder, dachora_run, etecoon, fire_arc, mini_kraid, statues};
        match routine {
            10918488 => self.owned_call(mini_kraid::initialize),
            10918594 => self.owned_call(mini_kraid::main_ai),
            10918946 => self.owned_call(mini_kraid::touch),
            10918969 => self.owned_call(mini_kraid::shot),
            11004178 => self.owned_call(etecoon::initialize),
            11004224 => self.owned_call(etecoon::main_ai),
            11007197 => self.owned_call(dachora_run::initialize),
            11007278 => self.owned_call(dachora_run::main_ai),
            11196360 => self.owned_call(statues::initialize_tourian_statue),
            11200278 => self.owned_call(statues::initialize_tube_cracks),
            11200293 => self.owned_call(statues::initialize_chozo_statue),
            11200423 => self.owned_call(statues::main_ai),
            10913525 => self.owned_call(boulder::initialize),
            10913683 => self.owned_call(boulder::main_ai),
            10914693 | 10914713 => self.owned_call(fire_arc::initialize_spike_platform_bottom),
            10917060 => self.owned_call(fire_arc::initialize),
            10917390 => self.owned_call(fire_arc::main_ai),
            _ => return false,
        }
        true
    }
    fn kraid_and_hosts_instruction(
        &mut self,
        routine: u32,
        actor: u16,
        script: EnemyScript,
    ) -> Result<EnemyScript, EnemyScript> {
        use sm_game::actors::{mini_kraid, statues};
        // The instructions that only act return the script unchanged.
        let action: Option<fn(&mut Self)> = match routine {
            10918694 => Some(mini_kraid::step),
            10918834 => Some(mini_kraid::roar),
            10918852 => Some(mini_kraid::spit_left),
            10918914 => Some(mini_kraid::spit_right),
            11199529 => Some(statues::start_lowering_acid),
            11199542 => Some(statues::set_lowered_acid),
            11199549 => Some(statues::unlock_samus),
            11199871 => Some(statues::grab_sound),
            11199879 => Some(statues::footstep_sound),
            11200240 => Some(statues::release_samus),
            _ => None,
        };
        if let Some(action) = action {
            self.owned_call(action);
            return Ok(script);
        }
        Ok(match routine {
            10918772 => self.owned_call(mini_kraid::choose_animation),
            11199887 => {
                self.owned_call(|runtime| statues::spike_clearing_footstep(runtime, actor, script))
            }
            11199960 => self.owned_call(|runtime| statues::movement(runtime, actor, script)),
            _ => return Err(script),
        })
    }
}
