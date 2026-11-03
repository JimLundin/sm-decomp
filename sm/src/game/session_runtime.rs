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

/// Magdollite's original diagnostic for an undefined lava height.
pub(crate) fn warn_lava_height(value: i32) {
    unsafe { crate::c_library::printf(c"Y undefined %d!\n".as_ptr(), value) };
}

/// Write `operand` to the multiplicand register, for Yapping Maw.
pub(crate) fn write_multiplicand(operand: u8, game: &mut Game) {
    unsafe { crate::rtl::write_reg(crate::rtl::WRMPYA as u16, operand, game) };
}

impl sm_game::actors::species::AiHost for Game {
    fn native_ai(&mut self, ea: u32) {
        unsafe { native_ai(ea, self) }
    }
    fn warn_reflection(&mut self) {
        unsafe { crate::c_library::printf(c"Possible bug. What is X?\n".as_ptr()) };
    }
    fn write_multiplicand(&mut self, operand: u8) {
        write_multiplicand(operand, self);
    }
    fn install_draygon_layers(&mut self) {
        crate::game::visual_runtime::initialize_draygon_layer(self);
    }
}

impl sm_game::actors::species::PreInstructionHost for Game {
    fn native_pre_instruction(&mut self, ea: u32, k: u16) {
        match ea {
            11767039 => Runtime::new(self).owned_call(sm_game::actors::gamet::settle),
            11771328 | 11771517 | 11771616 | 11771700 | 11771770 => {
                Runtime::new(self)
                    .owned_call(|runtime| actors::botwoon::pre_instruction(ea, k, runtime));
            }
            _ => {
                crate::rtl::report_unreachable(self);
            }
        }
    }
}

impl sm_game::actors::gunship::Host for Runtime<'_> {
    fn spawn_palette_effect(&mut self, definition: u16) {
        self.foreign(|game| crate::game::palette_effects::spawn(definition, game));
    }
    fn face_player_forward(&mut self) {
        self.owned_player_call(crate::game::player_transitions::equipment::face_forward);
    }
    fn event_happened(&mut self, event: u16) -> bool {
        self.owned_call(|runtime| sm_game::progress::event_happened(runtime, event))
    }
    fn restore(&mut self, resource: sm_game::item_pickups::Resource, amount: u16) {
        self.foreign(|game| crate::game::item_pickups_runtime::restore(resource, amount, game));
    }
    fn message_box_response(&mut self, kind: u16) -> i32 {
        self.foreign(|game| sm_game::message_boxes::poll(kind, game))
    }
    fn save_game(&mut self, slot: u16) {
        self.foreign(|game| sm_game::menus::save_slots::save(slot, game));
    }
    fn run_other_pre_instruction(&mut self, address: u32) {
        if sm_game::actors::species::is_torizo_pre_instruction(address) {
            self.owned_call(|runtime| {
                let k = Runtime::word(runtime, 0xe54);
                sm_game::actors::species::torizo_pre_instruction(address, k, runtime);
            });
        } else {
            self.foreign(|game| sm_game::actors::species::pre_instruction(address, game));
        }
    }
}

/// The morph ball eye runs on the scheduler's lease; its beam starts on the game.
impl sm_game::actors::morph_ball_eye::Host for Runtime<'_> {
    fn start_beam(&mut self) {
        self.foreign(crate::game::visual_runtime::initialize_morph_eye_beam);
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
    pub(crate) fn persistent_is_current(&self) -> bool {
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
        let spans = state
            .presentation
            .owner()
            .map(|owner| sm_game::fx_encoding::physical_source_spans(&owner.state))
            .unwrap_or_default();
        // The exported Session stands for a decode of the RAM it leaves unless a Display
        // owner travels with the call.
        let mut exported = (!state.presentation.is_owned()).then(|| state.clone());
        export_session_for_foreign(state, exported.as_ref(), mutation.game());
        if exported.is_some() && !mutation.game().ram().start_write_log() {
            exported = None;
        }
        let _source_interval = mutation.game().ram().begin_source_interval(&spans);
        mutation.game().revoke_inventory_domain();
        mutation.game().revoke_menu_domain();
        mutation.game().revoke_fx_domain();
        mutation.game().revoke_scroll_domain();
        let result = action(mutation.game());
        let written = exported
            .is_some()
            .then(|| mutation.game().ram().close_write_log())
            .flatten();
        let payload = mutation.game().capture_samus_snapshot();
        let prepared = mutation
            .game()
            .prepare_samus_adoption(&payload)
            .expect("Samus resumption rejected");
        let mut state = resume_session(exported, written, mutation.game());
        let presentation = mutation
            .game()
            .pop_foreign_presentation()
            .expect("foreign presentation frame")
            .expect("foreign presentation returned");
        adopt_carried_presentation(&mut state, presentation, mutation.game());
        // Retain the actual owner before a fallible source import. Runtime Drop
        // quarantines this state without encoding over a completed native prefix.
        self.state = Some(state);
        let state = self.state.as_mut().expect("resumed Session owner");
        if state.fx_sources.is_none() {
            let ram = mutation.game().ram();
            sm_game::fx_encoding::adopt_session_sources(state, &|at| ram.read_byte(at), &|at| {
                ram.source_publication(at)
            })
            .expect("native source adoption rejected");
        }
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
        self.inputs = self.observed_inputs();
    }
    /// The external observations as `byte` reads them, with the Session's projection of
    /// every input byte taken at once.
    fn observed_inputs(&self) -> ExternalInputs {
        let projection = self
            .persistent_is_current()
            .then(|| encoding::InputProjection::new(self.session()));
        encoding::inputs(&|at| {
            if (0xb1..0xc1).contains(&at) && !self.display_owned() {
                return self.game.scroll_byte(at);
            }
            projection
                .as_ref()
                .and_then(|projection| projection.byte(at))
                .unwrap_or_else(|| self.game.ram().read_byte(at))
        })
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
                    && sm_game::fx_encoding::validate_control_ownership(&state).is_ok()
                    && epoch == self.game.compatibility_write_epoch()
                    && self.game.preflight_samus_runtime().is_ok();
                if valid {
                    // Publish compatibility bytes while keeping the same gameplay
                    // and private Display owner; no decode or identity transfer.
                    self.game
                        .ram()
                        .copy_spans(|ram| encoding::encode_spans(&state, ram));
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
                && sm_game::fx_encoding::validate_control_ownership(&state).is_ok()
                && self.game.actor_identity_is_with_session()
                && self.game.preflight_samus_runtime().is_ok()
            {
                if self.game.frame_session.active && !self.game.foreign_presentation_active() {
                    let mut kept = state.clone();
                    kept.actor_identity =
                        crate::game::actors::identity::ActorIdentityState::empty();
                    let kept = export_frame_session(state, kept, self.game);
                    self.game.frame_session.kept = Some(KeptSession {
                        state: kept,
                        inputs: self.inputs.clone(),
                        epoch: self.game.compatibility_write_epoch(),
                        crossed: false,
                    });
                    self.game.ram().open_write_log();
                } else {
                    export_session_preflighted(state, self.game);
                }
            } else if self.game.actor_identity_is_with_session() {
                // A rejected foreign call retains this Runtime's identity owner;
                // discard invalid byte/native publication rather than panic again.
                let window = state.menu_window;
                self.game.return_actor_identity(state.actor_identity);
                self.game.return_inventory_domain(state.inventory, false);
                self.game.return_fx_domain(
                    state.fx,
                    state.presentation.visual().fireflea,
                    state.fx_sources,
                    false,
                );
                self.game.return_menu_domain(
                    sm_game::menus::state::State {
                        menu: state.menu,
                        window,
                        screens: state.menu_screens,
                    },
                    false,
                );
                if state.presentation.is_owned() {
                    if let Some(frame) = self.game.last_foreign_presentation_mut() {
                        assert!(frame.is_none(), "failed nested display owner occupied");
                        *frame = Some(state.presentation);
                    }
                }
                self.game.revoke_samus_authority();
            } else {
                self.game.revoke_inventory_domain();
                self.game.revoke_menu_domain();
                self.game.revoke_fx_domain();
            }
        }
    }
    pub(crate) fn byte(&self, at: usize) -> u8 {
        if (0xb1..0xc1).contains(&at) && !self.display_owned() {
            return self.game.scroll_byte(at);
        }
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
        if sm_game::fx_encoding::control_lane(at).is_some() {
            return self.word(at);
        }
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
        if sm_game::fx_encoding::control_lane(at).is_some() {
            return self.write_word_le(value, at);
        }
        if self.persistent_is_current() {
            *field(self.session_mut()) = value;
        } else {
            self.write_word_le(value, at);
        }
    }
    /// Point the direct-page long pointer scratch at bank $00 `address`, $03
    /// unchanged: the Session's field while this lease is current, the
    /// projected bytes $00..$02 otherwise. Refreshes the observations as the
    /// byte stores did.
    pub(crate) fn point_pointer_scratch(&mut self, address: u16) {
        if self.persistent_is_current() {
            self.session_mut().pointer_scratch.point_at(address);
            self.refresh_written_inputs();
        } else {
            let [low, high] = address.to_le_bytes();
            for (at, byte) in [(0, low), (1, high), (2, 0)] {
                self.set_byte(at, byte);
            }
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
        if (0xb1..0xc1).contains(&at) && self.persistent_is_current() && !self.display_owned() {
            self.game.set_scroll_byte(at, value);
            self.refresh_scroll_inputs();
            return;
        }
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
        self.refresh_written_inputs();
    }
    /// The scroll stores' refresh while the Session is current: the external
    /// observations as RAM holds them, except the drop origin, the moonwalk
    /// setting, the turn request and the grapple beam's previous start, which
    /// the Session owns and RAM holds only once it is published.
    pub(crate) fn refresh_scroll_inputs(&mut self) {
        self.inputs = encoding::inputs(&|lane| self.game.ram().read_byte(lane));
        let session = self.session();
        let origin = session.enemy_projectiles.drop_origin;
        let (moonwalk, turn_request) = (session.player.moonwalk, session.player.turn_request);
        let previous_start = session.player.swing.previous_start;
        self.inputs.drop_origin_x = origin.x;
        self.inputs.drop_origin_y = origin.y;
        self.inputs.transitions.moonwalk_enabled = moonwalk;
        self.inputs.transitions.turn_request = turn_request;
        self.inputs.grapple_x = previous_start.x;
        self.inputs.grapple_y = previous_start.y;
    }
    pub(crate) fn refresh_written_inputs(&mut self) {
        self.inputs =
            encoding::inputs_from_session(self.session(), &|at| self.game.ram().read_byte(at));
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
        // Both former foreign steps keep its observation refresh.
        self.owned_player_call(|runtime| match step {
            PlayerStep::Radius => {
                // SamusPoseParams has eight bytes, with y_radius at byte 6.
                let pose_offset = usize::from(runtime.session().player.pose) * 8;
                let radius = runtime.game.read_rom_byte(0x91b629, pose_offset + 6);
                runtime.write_slice_le(0xb00, u16::from(radius));
                runtime.write_slice_le(0xafe, 5);
            }
            PlayerStep::RestoreControl => {
                player_transitions_runtime::command_in(runtime, 0x14);
            }
            PlayerStep::Input
            | PlayerStep::SuitIndex
            | PlayerStep::Acceleration
            | PlayerStep::InsideBlock
            | PlayerStep::Grapple
            | PlayerStep::Xray => unreachable!("handled by shared owners"),
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
        use sm_game::sprite_output as output;
        self.draw_objects(|objects| match drawing {
            Drawing::Projectile { slot, x, y } => {
                output::projectile(slot, (x, y), objects, &mut output::Native);
                crate::verify_probe::hit("player_projectiles.draw.completed");
            }
            Drawing::Beam { sprite, x, y } => output::beam(sprite, (x, y), objects),
            Drawing::CrashEchoes => output::crash_echoes(objects, &mut output::Native),
            Drawing::GrappleSegment { x, y, tile } => output::grapple(objects, Some((x, y, tile))),
            Drawing::GrappleEnd => output::grapple(objects, None),
        });
    }
    fn draw_trail(&mut self, pair: usize, right: bool, frozen: bool) {
        self.draw_objects(|drawing| drawing.draw_trail(pair, right, frozen));
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

/// The Session a frame tick's last lease exported, kept with a log of the RAM written since, so
/// the next lease, in this tick or a later one, borrows it and takes those bytes rather than
/// decoding the RAM again. Only a lease in a frame tick (`with_frame_session`) keeps one.
#[derive(Default)]
pub(crate) struct FrameSession {
    active: bool,
    kept: Option<KeptSession>,
    /// The span layouts of the frame Session exports, by shape.
    layouts: Vec<(ExportShape, std::rc::Rc<crate::game_state::ExportLayout>)>,
}
impl FrameSession {
    fn layout(&self, shape: ExportShape) -> Option<std::rc::Rc<crate::game_state::ExportLayout>> {
        self.layouts
            .iter()
            .find(|(known, _)| *known == shape)
            .map(|(_, layout)| std::rc::Rc::clone(layout))
    }
}
struct KeptSession {
    /// `None` while it is the Session of RAM's deferred export.
    state: Option<Session>,
    inputs: ExternalInputs,
    /// The RAM write epoch after the export.
    epoch: u64,
    /// Whether the tick that kept it has ended.
    crossed: bool,
}

/// Run a frame tick with its leases sharing one Session, which the next tick's first lease
/// takes up.
pub(crate) fn with_frame_session<R>(game: &mut Game, tick: impl FnOnce(&mut Game) -> R) -> R {
    game.frame_session.active = true;
    let result = tick(game);
    game.frame_session.active = false;
    if let Some(kept) = game.frame_session.kept.as_mut() {
        kept.crossed = true;
    }
    result
}

/// Bring `state` up to the RAM bytes in `written`: each byte the Session projects and the RAM
/// now holds differently goes in as a store of that byte would put it.
fn take_written_bytes(state: &mut Session, game: &Game, written: &[std::ops::Range<usize>]) {
    let ram = game.ram();
    for at in written.iter().flat_map(Clone::clone) {
        let value = ram.read_byte(at);
        if encoding::projected_byte(state, at).is_some_and(|byte| byte != value) {
            encoding::write_projected_byte(state, at, value);
        }
    }
}
/// The kept Session for a new lease, when the RAM log, world and Samus state still admit it;
/// it takes the RAM written since its export, the native spawn fields and actor identity as an
/// import would.
fn reuse_frame_session(game: &mut Game) -> Option<(Session, ExternalInputs)> {
    if game.foreign_presentation_active() {
        return None;
    }
    let KeptSession {
        state,
        mut inputs,
        epoch,
        crossed,
    } = game.frame_session.kept.take()?;
    let written = game.ram().close_write_log();
    let mut state = state
        .or_else(|| game.ram().take_deferred_session())
        .expect("kept Session");
    if !same_world(&state, game) {
        return None;
    }
    let written = if epoch == game.compatibility_write_epoch() {
        None
    } else {
        Some(written?)
    };
    game.preflight_samus_runtime()
        .expect("Samus import rejected");
    if let Some(written) = &written {
        // The RAM changed since the export: take the written bytes, as a decode would read them.
        take_written_bytes(&mut state, game, written);
    }
    if written.is_some() || crossed {
        // Observe the inputs from the RAM as an import does.
        inputs = encoding::inputs(&|at| game.ram().read_byte(at));
    }
    #[cfg(sm_verify_dll)]
    player_normal_transfers::record(game, 1);
    import_native_spawn(&mut state, game);
    // A Session whose export never reached RAM is the very Session the last lease released.
    #[cfg(sm_verify_dll)]
    if !game.ram().export_deferred() {
        assert_projects_ram(&state, game);
    }
    take_game_owners(&mut state, game);
    Some((state, inputs))
}
/// The Session a foreign call resumes: the Session it exported with the RAM bytes it wrote,
/// when the log holds them; a decode of the RAM otherwise.
fn resume_session(
    exported: Option<Session>,
    written: Option<Vec<std::ops::Range<usize>>>,
    game: &mut Game,
) -> Session {
    let (Some(mut state), Some(written)) = (exported, written) else {
        return import_session_image(game);
    };
    if !same_world(&state, game) {
        return import_session_image(game);
    }
    take_written_bytes(&mut state, game, &written);
    state.actor_identity = crate::game::actors::identity::ActorIdentityState::empty();
    #[cfg(sm_verify_dll)]
    player_normal_transfers::record(game, 1);
    import_native_spawn(&mut state, game);
    // A Session whose export never reached RAM is the very Session the call exported.
    #[cfg(sm_verify_dll)]
    if !game.ram().export_deferred() {
        assert_projects_ram(&state, game);
    }
    take_game_owners(&mut state, game);
    state
}
fn same_world(state: &Session, game: &Game) -> bool {
    std::sync::Arc::ptr_eq(&state.world, game.world())
        && state.world.revision() == game.world().revision()
}
/// The verifier's check that a Session taken up without a decode equals a decode of the RAM.
#[cfg(sm_verify_dll)]
fn assert_projects_ram(state: &Session, game: &Game) {
    let mut fresh = encoding::decode(
        &|at| game.ram().read_byte(at),
        0x20000,
        game.world().clone(),
    );
    import_native_spawn(&mut fresh, game);
    assert!(
        fresh == *state,
        "a Session taken up without a decode differs from its RAM projection"
    );
}
/// The owners Game holds between leases, as an import takes them into the Session.
fn take_game_owners(state: &mut Session, game: &mut Game) {
    state.actor_identity = game.take_actor_identity();
    if let Some(inventory) = game.take_inventory_domain() {
        state.inventory = inventory;
    }
    if let Some(fx) = game.take_fx_domain() {
        state.fx = fx.numeric;
        state.presentation.visual_mut().fireflea = fx.lighting;
        state.fx_sources = Some(fx.sources);
    }
    if let Some(menu) = game.take_menu_domain() {
        state.menu = menu.menu;
        sm_game::fx_encoding::import_window(state, menu.window);
        state.menu_screens = menu.screens;
    }
}
pub(crate) fn import_session(game: &mut Game) -> Session {
    game.preflight_samus_runtime()
        .expect("Samus import rejected");
    import_session_preflighted(game)
}
fn import_session_preflighted(game: &mut Game) -> Session {
    let mut state = import_session_image(game);
    if let Some(presentation) = game.last_foreign_presentation_mut().and_then(|frame| {
        if frame
            .as_ref()
            .is_some_and(sm_game::session_owners::Presentation::is_owned)
        {
            frame.take()
        } else {
            None
        }
    }) {
        adopt_carried_presentation(&mut state, presentation, game);
        let mut guard = ExportIdentity {
            state: Some(state),
            game,
            carry_presentation: false,
        };
        let state = guard.state.as_mut().expect("nested import owner");
        if state.fx_sources.is_none() {
            sm_game::fx_encoding::adopt_session_sources(
                state,
                &|at| guard.game.ram().read_byte(at),
                &|at| guard.game.ram().source_publication(at),
            )
            .expect("nested native source adoption rejected");
        }
        return guard.state.take().expect("nested imported Session");
    }
    state
}
fn adopt_carried_presentation(
    state: &mut Session,
    mut presentation: sm_game::session_owners::Presentation,
    game: &Game,
) {
    if presentation.is_owned() {
        presentation.adopt_projection(&|at| game.ram().read_byte(at));
        *presentation.visual_mut() = state.presentation.visual().clone();
        *presentation.palette_mut() = state.presentation.palette().clone();
        *presentation.atmospheric_mut() = state.presentation.atmospheric().clone();
        state.presentation = presentation;
        state.fx_sources = None;
        state.hdma = Default::default();
        sm_game::fx_encoding::adopt_session_sources(state, &|at| game.ram().read_byte(at), &|at| {
            game.ram().source_publication(at)
        })
        .expect("carried control ownership");
    }
}
fn import_session_image(game: &mut Game) -> Session {
    #[cfg(sm_verify_dll)]
    player_normal_transfers::record(game, 1);
    let mut state = encoding::decode(
        &|at| game.ram().read_byte(at),
        0x20000,
        game.world().clone(),
    );
    import_native_spawn(&mut state, game);
    take_game_owners(&mut state, game);
    state
}
fn preflight_export(state: &Session, game: &Game) {
    sm_game::fx_encoding::validate_control_ownership(state).expect("Session control ownership");
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
    carry_presentation: bool,
}
impl Drop for ExportIdentity<'_> {
    fn drop(&mut self) {
        if let Some(state) = self.state.take() {
            let completed = !std::thread::panicking();
            let lighting = if completed {
                sm_game::fx_encoding::session_lighting(&state)
            } else {
                state.presentation.visual().fireflea
            };
            let window = if completed {
                sm_game::fx_encoding::session_window(&state)
            } else {
                state.menu_window
            };
            let numeric = if completed {
                sm_game::fx_encoding::session_controls(&state)
            } else {
                state.fx
            };
            let presentation = state.presentation;
            if self.game.actor_identity_is_with_session() {
                self.game.return_actor_identity(state.actor_identity);
            }
            self.game
                .return_inventory_domain(state.inventory, !std::thread::panicking());
            self.game.return_fx_domain(
                numeric,
                lighting,
                state.fx_sources,
                !std::thread::panicking(),
            );
            self.game.return_menu_domain(
                sm_game::menus::state::State {
                    menu: state.menu,
                    window,
                    screens: state.menu_screens,
                },
                !std::thread::panicking(),
            );
            if self.carry_presentation {
                self.game.push_foreign_presentation(presentation);
            } else if presentation.is_owned() {
                if let Some(frame) = self.game.last_foreign_presentation_mut() {
                    assert!(frame.is_none(), "nested display owner occupied");
                    *frame = Some(presentation);
                }
            }
        }
    }
}
#[cfg(sm_verify_dll)]
pub(crate) fn export_session(state: Session, game: &mut Game) {
    let guard = ExportIdentity {
        state: Some(state),
        game,
        carry_presentation: false,
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
        carry_presentation: false,
    });
}
/// What decides a Session export's span layout; everything else it writes has a fixed place.
/// Only Sessions with a Legacy presentation and control have a shape: the owned ones lay out
/// their own spans.
#[derive(Clone, Copy, PartialEq, Eq)]
struct ExportShape {
    sources: bool,
    workspace: usize,
}
impl ExportShape {
    fn of(state: &Session) -> Option<Self> {
        (!state.presentation.is_owned()
            && matches!(state.control, sm_game::session_owners::Control::Legacy(_)))
        .then(|| Self {
            sources: state.fx_sources.is_some(),
            workspace: state.presentation.workspace().shared_bytes().len(),
        })
    }
}
/// Export a frame lease's Session `state`, of which the frame keeps the copy `exported`.
/// Once an export of its shape has recorded the layout, and while the write log is closed,
/// RAM takes the export's epochs and source publications now and its bytes when RAM they
/// cover is next read or written; the kept Session is then the deferred one (`None`).
fn export_frame_session(state: Session, exported: Session, game: &mut Game) -> Option<Session> {
    let shape = ExportShape::of(&exported);
    let known = shape.and_then(|shape| game.frame_session.layout(shape));
    let guard = ExportIdentity {
        state: Some(state),
        game,
        carry_presentation: false,
    };
    #[cfg(sm_verify_dll)]
    player_normal_transfers::record(guard.game, 2);
    let state = guard.state.as_ref().expect("export owns Session");
    if let Some(layout) = known
        .as_ref()
        .filter(|_| !guard.game.ram().write_log_open())
    {
        guard.game.ram().defer_export(exported, layout);
        export_native_spawn(state, guard.game);
        // Drop moves identity exactly once, after the byte/native publication.
        return None;
    }
    let layout = guard
        .game
        .ram()
        .copy_spans_recorded(|ram| encoding::encode_spans(state, ram));
    export_native_spawn(state, guard.game);
    guard.game.ram().forget_written_export();
    if let (Some(shape), None) = (shape, known) {
        guard
            .game
            .frame_session
            .layouts
            .push((shape, std::rc::Rc::new(layout)));
    }
    Some(exported)
}
/// Export `state` for a foreign call. When the call resumes the copy `exported`, an export of
/// a shape whose layout a frame export recorded is deferred as `export_frame_session` defers
/// one.
fn export_session_for_foreign(state: Session, exported: Option<&Session>, game: &mut Game) {
    let layout = exported
        .and_then(ExportShape::of)
        .and_then(|shape| game.frame_session.layout(shape))
        .filter(|_| !game.ram().write_log_open());
    let guard = ExportIdentity {
        state: Some(state),
        game,
        carry_presentation: true,
    };
    let (Some(layout), Some(exported)) = (layout, exported) else {
        encode_session(guard);
        return;
    };
    // A kept frame Session that is the deferred one takes its copy before this export
    // replaces it.
    if guard
        .game
        .frame_session
        .kept
        .as_ref()
        .is_some_and(|kept| kept.state.is_none())
    {
        let state = guard.game.ram().take_deferred_session();
        if let Some(kept) = guard.game.frame_session.kept.as_mut() {
            kept.state = state;
        }
    }
    #[cfg(sm_verify_dll)]
    player_normal_transfers::record(guard.game, 2);
    let mut deferred = exported.clone();
    deferred.actor_identity = crate::game::actors::identity::ActorIdentityState::empty();
    guard.game.ram().defer_export(deferred, &layout);
    export_native_spawn(
        guard.state.as_ref().expect("export owns Session"),
        guard.game,
    );
    // Drop moves identity exactly once, after the byte/native publication.
}
fn encode_session(guard: ExportIdentity<'_>) {
    #[cfg(sm_verify_dll)]
    player_normal_transfers::record(guard.game, 2);
    let state = guard.state.as_ref().expect("export owns Session");
    guard
        .game
        .ram()
        .copy_spans(|ram| encoding::encode_spans(state, ram));
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
    /// The projected store's refresh after each byte.
    fn words_stored(&mut self) {
        self.refresh_written_inputs();
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
            11769219 => self.owned_call(actors::botwoon::initialize),
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
            11051336 => self.owned_call(actors::yapping_maw::initialize),
            11052953 => self.owned_call(actors::yapping_maw::touch),
            11052989 => self.owned_call(actors::yapping_maw::shot),
            11053109 => self.owned_call(actors::yapping_maw::frozen),
            11051537 => self.owned_call(actors::yapping_maw::run_main_ai),
            11765819 => self.owned_call(sm_game::actors::zeb::initialize),
            11765882 => self.owned_call(sm_game::actors::zeb::main_ai),
            11767628 => self.owned_call(sm_game::actors::geega::initialize),
            11767726 => self.owned_call(sm_game::actors::geega::main_ai),
            11059657 => self.owned_call(sm_game::actors::powamp::initialize),
            11059740 => self.owned_call(sm_game::actors::powamp::main_ai),
            11060670 => self.owned_call(sm_game::actors::powamp::touch),
            11060719 => self.owned_call(sm_game::actors::powamp::shot),
            11060799 => self.owned_call(sm_game::actors::powamp::power_bomb),
            11068983 => self.owned_call(sm_game::actors::wrecked_ship_spark::initialize),
            11069070 => self.owned_call(sm_game::actors::wrecked_ship_spark::main_ai),
            11069486 => self.owned_call(sm_game::actors::blue_brinstar_face_block::initialize),
            11069614 => self.owned_call(sm_game::actors::blue_brinstar_face_block::main_ai),
            11047000 => self.owned_call(sm_game::actors::morph_ball_eye::initialize),
            11047138 => self.owned_call(sm_game::actors::morph_ball_eye::main_ai),
            11790027 | 11791077 => self.owned_call(|runtime| {
                actors::escape_critters::initialize(routine == 11791077, runtime)
            }),
            11789909 => self.owned_call(actors::escape_critters::main_ai),
            11054987 => self.owned_call(actors::magdollite::initialize),
            11055370 => self.owned_call(actors::magdollite::main_ai),
            10873386 => self.owned_call(actors::spore_spawn::initialize),
            10873619 => self.owned_call(actors::spore_spawn::main_ai),
            10874202 => self.owned_call(actors::spore_spawn::shot),
            10874348 => self.owned_call(actors::spore_spawn::touch),
            10783748 => self.owned_call(actors::crocomire_reactions::main_ai),
            10782343 => self.owned_call(actors::crocomire_reactions::hurt),
            10811002 => self.owned_call(actors::crocomire_reactions::initialize_tongue),
            10783322 => self.owned_call(actors::crocomire_reactions::initialize),
            10795410 => self.owned_call(actors::crocomire_reactions::power_bomb),
            11056128 | 11056134 | 11056140 => {
                self.owned_call(|runtime| actors::magdollite::damage(routine, runtime))
            }
            10997235 => self.owned_call(sm_game::actors::phantoon::initialize),
            10997333 => self.owned_call(sm_game::actors::phantoon::initialize_part),
            10997414 => self.owned_call(sm_game::actors::phantoon::main_ai),
            11001151 => self.owned_call(sm_game::actors::phantoon::hurt),
            11001237 => self.owned_call(sm_game::actors::phantoon::touch),
            11001243 => self.owned_call(sm_game::actors::phantoon::shot),
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
        if routine == 10977418 {
            self.owned_call(|runtime| {
                let address = u32::from(script.word(0)) | 0xa70000;
                sm_game::actors::phantoon::callback(
                    runtime,
                    sm_game::actors::phantoon::Callback::Instruction { address, actor },
                );
            });
            return script.offset(1);
        }
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
            11769031 | 11769095 | 11769111 | 11769175 => {
                return self
                    .owned_call(|runtime| actors::botwoon::set_hitbox(8, 16, script, runtime));
            }
            11769047 | 11769079 | 11769127 | 11769159 => {
                return self
                    .owned_call(|runtime| actors::botwoon::set_hitbox(12, 12, script, runtime));
            }
            11769063 | 11769143 => {
                return self
                    .owned_call(|runtime| actors::botwoon::set_hitbox(16, 8, script, runtime));
            }
            11769191 => {
                return self.owned_call(|runtime| actors::botwoon::set_spitting(script, runtime));
            }
            0xa40000..=0xa4ffff => {
                return self.owned_call(|runtime| {
                    actors::crocomire_reactions::instruction(routine, actor, script, runtime)
                });
            }
            10872671 | 10872689 | 10872877 | 10872956 | 10873018 | 10873009 | 10873034
            | 10873116 | 10873198 | 10873265 => {
                return self.owned_call(|runtime| {
                    actors::spore_spawn::run_instruction(routine, actor, script, runtime)
                });
            }
            11054610 | 11054630 | 11054640 | 11054650 | 11054661 | 11054672 | 11054682
            | 11054692 | 11054728 | 11054742 | 11054778 | 11054794 | 11054820 | 11054846
            | 11054872 | 11054916 => {
                return self.owned_call(|runtime| {
                    actors::magdollite::instruction(routine, script, runtime)
                });
            }
            11789637 | 11791016 | 11791032 | 11789840 | 11791049 | 11791063 => {
                return self.owned_call(|runtime| {
                    actors::escape_critters::instruction(routine, actor, script, runtime)
                });
            }
            11068957 | 11068970 => {
                return self.owned_call(|runtime| {
                    use sm_game::actors::wrecked_ship_spark::set_intangible;
                    set_intangible(runtime, routine == 11068957, script)
                });
            }
            11051207 | 11051225 | 11051243 | 11051261 | 11051279 | 11051297 | 11051315 => {
                return self.owned_call(|runtime| {
                    actors::yapping_maw::instruction(routine, script, runtime)
                });
            }
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
        if self.kraid_and_hosts_touch(routine) {
            return;
        }
        if routine == 11001237 {
            self.owned_call(sm_game::actors::phantoon::touch);
        } else if routine == 10940479 {
            self.owned_call(sm_game::actors::ceres_doors::steam_touch);
        } else if routine == 10874348 {
            self.owned_call(actors::spore_spawn::touch);
        } else if routine == 10795325 {
            self.owned_call(actors::crocomire_reactions::touch);
        } else if !self.bank_a9_routine(routine) && !species::lease_touch(routine, self) {
            self.foreign(|game| species::touch(routine, game));
        }
    }
    pub(crate) fn actor_shot_legacy(&mut self, routine: u32, argument: u16) {
        use sm_game::actors::species;
        if self.kraid_and_hosts_shot(routine, argument) {
            return;
        }
        if routine == 11001243 {
            self.owned_call(sm_game::actors::phantoon::shot);
        } else if routine == 10936202 {
            self.owned_call(sm_game::actors::ridley::shot);
        } else if routine == 10874202 {
            self.owned_call(actors::spore_spawn::shot);
        } else if routine == 10795345 {
            self.owned_call(actors::crocomire_reactions::count_hit);
        } else if matches!(routine, 10795368 | 10795700) {
            self.owned_call(actors::crocomire_reactions::impact);
        } else if routine == 10795525 {
            self.owned_call(actors::crocomire_reactions::hit);
        } else if !self.bank_a9_routine(routine) && !species::lease_shot(routine, self) {
            self.foreign(|game| species::shot(routine, argument, game));
        }
    }
    pub(crate) fn actor_hook_legacy(&mut self, routine: u32) {
        use sm_game::actors::species;
        if routine == 10920690 {
            self.owned_call(sm_game::actors::ridley::draw_baby_metroid_and_door);
        } else if routine == 11055282 {
            self.owned_call(actors::magdollite::cycle_palette);
        } else if routine == 11069550 {
            self.owned_call(sm_game::actors::blue_brinstar_face_block::cycle_palette);
        } else if species::EMPTY_DRAW_HOOKS.contains(&routine) {
            // Nothing to draw; keep the former foreign call's observation refresh.
            self.owned_player_call(|_| ());
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
        self.draw_objects(|drawing| {
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
                drawing,
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
        self.draw_objects(|drawing| {
            use sm_game::sprite_drawing::Clip;
            use sm_game::sprite_output::{Native, sprite_map};
            if base && address == 0 {
                return;
            }
            let map = sprite_map(bank, address, drawing, &mut Native);
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
                drawing,
            );
        });
    }
}

#[path = "lease_drawing.rs"]
mod lease_drawing;
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
    command as player_command, command_in as player_command_in, damage_player, finish_player_frame,
    finish_xray, initialize as initialize_player, mother_brain as player_mother_brain,
};
#[path = "player_update_runtime.rs"]
pub(crate) mod player_update_runtime;

#[path = "screen_fade_runtime.rs"]
mod screen_fade_runtime;

#[path = "scroll_runtime.rs"]
mod scroll_runtime;

#[path = "sequence_runtime.rs"]
mod sequence_runtime;

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
    fn observed_inputs(&self) -> sm_game::external_inputs::ExternalInputs {
        Runtime::observed_inputs(self)
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
impl sm_game::shooting_star_motion::Lease for Runtime<'_> {
    fn star_tiles(&self) -> &Option<sm_game::imported::Imported<sm_data::shooting_stars::Tiles>> {
        &self.game.visual_tables.star_tiles
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
        use sm_game::actors::{
            boulder, dachora_run, etecoon, fire_arc, kraid, mini_kraid, statues,
        };
        match routine {
            10982544 => self.owned_call(kraid::arm_touch),
            10982559 => self.owned_call(kraid::touch),
            10987865 => self.owned_call(kraid::initialize),
            10988355 => self.owned_call(kraid::initialize_arm),
            10988392 => self.owned_call(|runtime| kraid::initialize_lint(runtime, 0x80)),
            10988444 => self.owned_call(|runtime| kraid::initialize_lint(runtime, 0xc0)),
            10988490 => self.owned_call(|runtime| kraid::initialize_lint(runtime, 0x100)),
            10988536 => self.owned_call(kraid::initialize_foot),
            10988577 => self.owned_call(kraid::main_ai),
            10991549 => self.owned_call(kraid::arm_main),
            10991617 => self.owned_call(|runtime| kraid::lint_main_ai(runtime, 0x80)),
            10991629 => self.owned_call(|runtime| kraid::lint_main_ai(runtime, 0xc0)),
            10991641 => self.owned_call(|runtime| kraid::lint_main_ai(runtime, 0x100)),
            10992118 => self.owned_call(kraid::foot_main),
            10992847 | 10992862 => self.owned_call(kraid::fingernail_touch),
            10992879 | 10992941 => self.owned_call(kraid::initialize_fingernail),
            10992946 => self.owned_call(|runtime| kraid::fingernail_main(runtime, 0x180)),
            10992969 => self.owned_call(|runtime| kraid::fingernail_main(runtime, 0x1c0)),
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
        use sm_game::actors::{kraid, mini_kraid, statues};
        // The instructions that only act return the script unchanged.
        let action: Option<fn(&mut Self)> = match routine {
            10991155 => Some(|_| ()),
            10991158 => Some(kraid::step_up),
            10991164 => Some(kraid::step_down_and_shake),
            10991182 => Some(kraid::stomp_sound),
            10991194 | 10991207 => Some(kraid::step_back),
            10991220 => Some(kraid::step_forward),
            10991235 => Some(kraid::lunge_right),
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
            10979983 => match self.owned_call(kraid::arm_swipe_when_hurt) {
                Some(list) => EnemyScript::jump(list),
                None => script,
            },
            11199887 => {
                self.owned_call(|runtime| statues::spike_clearing_footstep(runtime, actor, script))
            }
            11199960 => self.owned_call(|runtime| statues::movement(runtime, actor, script)),
            _ => return Err(script),
        })
    }
    fn kraid_and_hosts_touch(&mut self, routine: u32) -> bool {
        use sm_game::actors::kraid;
        match routine {
            10982539 => self.owned_call(kraid::touch_arm_or_foot),
            10982544 => self.owned_call(kraid::arm_touch),
            _ => return false,
        }
        true
    }
    fn kraid_and_hosts_shot(&mut self, routine: u32, argument: u16) -> bool {
        use sm_game::actors::kraid;
        match routine {
            10982582 => self.owned_call(|runtime| kraid::arm_shot(runtime, argument)),
            _ => return false,
        }
        true
    }
}

#[path = "boss_interaction_runtime.rs"]
mod boss_interaction_runtime;

// Frame display controls use the existing admitted Display, never a new lease.
#[forbid(unsafe_code)]
impl sm_game::display::preparation::Access for Runtime<'_> {
    fn display_control_byte(&self, field: sm_game::display::preparation::Field) -> u8 {
        if self.display_owned() {
            let display = self
                .session()
                .presentation
                .owner()
                .expect("current Display");
            sm_game::display::preparation::display_byte(&display.state, field)
        } else {
            self.byte(crate::game_state::frame_field_address(field))
        }
    }
    fn store_display_control_byte(
        &mut self,
        field: sm_game::display::preparation::Field,
        value: u8,
        _store: sm_game::display::preparation::Store,
    ) {
        if self.display_owned() {
            sm_game::display::preparation::set_display_byte(
                &mut self
                    .session_mut()
                    .presentation
                    .owner_mut()
                    .expect("current Display")
                    .state,
                field,
                value,
            );
            self.refresh_written_inputs();
        } else {
            self.set_byte(crate::game_state::frame_field_address(field), value);
        }
    }
    fn display_history_word(&self, word: sm_game::display::preparation::Word) -> u16 {
        if self.display_owned() {
            sm_game::display::preparation::display_word(
                &self
                    .session()
                    .presentation
                    .owner()
                    .expect("current Display")
                    .state,
                word,
            )
        } else {
            self.word(crate::game_state::frame_word_address(word))
        }
    }
    fn store_display_history_word(
        &mut self,
        word: sm_game::display::preparation::Word,
        value: u16,
        _store: sm_game::display::preparation::Store,
    ) {
        let at = crate::game_state::frame_word_address(word);
        for (lane, byte) in value.to_le_bytes().into_iter().enumerate() {
            if self.display_owned() {
                let display = &mut self
                    .session_mut()
                    .presentation
                    .owner_mut()
                    .expect("current Display")
                    .state;
                let mut bytes =
                    sm_game::display::preparation::display_word(display, word).to_le_bytes();
                bytes[lane] = byte;
                sm_game::display::preparation::set_display_word(
                    display,
                    word,
                    u16::from_le_bytes(bytes),
                );
                self.refresh_written_inputs();
            } else {
                self.set_byte(at + lane, byte);
            }
        }
    }
    fn hud_tile(&self, index: isize) -> u16 {
        let at = crate::game_state::frame_hud_address(index);
        if self.display_owned()
            && let Some(&value) = usize::try_from(index).ok().and_then(|index| {
                self.session()
                    .presentation
                    .owner()
                    .expect("current Display")
                    .state
                    .hud
                    .tiles
                    .get(index)
            })
        {
            value
        } else {
            self.word(at)
        }
    }
    fn store_hud_tile(
        &mut self,
        index: isize,
        value: u16,
        _store: sm_game::display::preparation::Store,
    ) {
        let at = crate::game_state::frame_hud_address(index);
        let normal = usize::try_from(index).ok().filter(|&index| index < 96);
        for (lane, byte) in value.to_le_bytes().into_iter().enumerate() {
            if self.display_owned()
                && let Some(index) = normal
            {
                let tile = &mut self
                    .session_mut()
                    .presentation
                    .owner_mut()
                    .expect("current Display")
                    .state
                    .hud
                    .tiles[index];
                let mut bytes = tile.to_le_bytes();
                bytes[lane] = byte;
                *tile = u16::from_le_bytes(bytes);
                self.refresh_written_inputs();
            } else {
                self.set_byte(at + lane, byte);
            }
        }
    }
    fn physical_display_control_byte(&self, field: sm_game::display::preparation::Field) -> u8 {
        self.byte(crate::game_state::frame_field_address(field))
    }
    fn store_physical_display_control_byte(
        &mut self,
        field: sm_game::display::preparation::Field,
        value: u8,
    ) {
        self.set_byte(crate::game_state::frame_field_address(field), value);
    }
}

#[forbid(unsafe_code)]
impl sm_game::gameplay_context::Access for Runtime<'_> {
    fn grapple_animation(&self) -> sm_game::gameplay_context::GrappleAnimation {
        if self.persistent_is_current() {
            let state = &self.session().player.swing;
            sm_game::gameplay_context::GrappleAnimation {
                timer: state.start_animation_timer,
                tiles: state.start_animation_tiles,
            }
        } else {
            sm_game::gameplay_context::Access::grapple_animation(self.game)
        }
    }
    fn commit_grapple_animation(
        &mut self,
        before: sm_game::gameplay_context::GrappleAnimation,
        after: sm_game::gameplay_context::GrappleAnimation,
    ) {
        if before.timer != after.timer {
            self.write_word_le(after.timer, 0xd3e);
        }
        if before.tiles != after.tiles {
            self.write_word_le(after.tiles, 0xd40);
        }
    }

    fn inventory_state(&self) -> sm_game::inventory::Inventory {
        if self.persistent_is_current() {
            self.session().inventory.clone()
        } else {
            sm_game::gameplay_context::Access::inventory_state(self.game)
        }
    }
    fn commit_inventory(
        &mut self,
        before: &sm_game::inventory::Inventory,
        mut after: sm_game::inventory::Inventory,
    ) {
        use sm_game::session_cohort::Cohort as _;
        if before == &after {
            return;
        }
        let mut old = Vec::new();
        before.clone().words(|_, word| old.push(*word));
        let mut changes = Vec::new();
        let mut index = 0;
        after.words(|at, word| {
            if old[index] != *word {
                changes.push((index, at, *word));
            }
            index += 1;
        });
        if self.persistent_is_current() {
            let mut index = 0;
            sm_game::inventory::Inventory::session_words(self.session_mut(), |_, word| {
                if let Some(&(_, _, value)) = changes.iter().find(|change| change.0 == index) {
                    *word = value;
                }
                index += 1;
            });
            self.refresh_inputs();
        } else {
            for (_, at, word) in changes {
                self.write_word_le(word, at);
            }
        }
    }

    fn gameplay_word(&self, word: sm_game::gameplay_context::Word) -> u16 {
        if self.persistent_is_current() {
            word.get_session(self.session())
        } else {
            sm_game::gameplay_context::Access::gameplay_word(self.game, word)
        }
    }
    fn set_gameplay_byte(&mut self, word: sm_game::gameplay_context::Word, lane: usize, byte: u8) {
        assert!(lane < 2);
        if self.persistent_is_current() {
            let field = word.get_session_mut(self.session_mut());
            let mut bytes = field.to_le_bytes();
            bytes[lane] = byte;
            *field = u16::from_le_bytes(bytes);
            self.refresh_inputs();
        } else {
            sm_game::gameplay_context::Access::set_gameplay_byte(self.game, word, lane, byte);
        }
    }
}

// Actual source authority on the same Runtime lease as the HDMA interpreter.
impl sm_game::fx_domain::Sources for Runtime<'_> {
    fn store_source_signed_byte(
        &mut self,
        owner: sm_game::scene::hdma_sources::LiveOwner,
        offset: isize,
        byte: u8,
    ) {
        if let Ok(offset) = usize::try_from(offset) {
            sm_game::fx_domain::Sources::store_source_byte(self, owner, offset, byte);
            return;
        }
        if self.persistent_is_current() {
            assert!(
                self.session().fx_sources.is_some(),
                "prepared source outside allocation"
            );
        }
        let base = sm_game::fx_encoding::source_address(owner, 0).expect("FX source role");
        let at = base
            .checked_add_signed(offset)
            .expect("source address outside RAM");
        self.set_byte(at, byte);
    }
    fn source_byte(&self, owner: sm_game::scene::hdma_sources::LiveOwner, offset: usize) -> u8 {
        if self.persistent_is_current() {
            let selected = self.session();
            match sm_game::fx_encoding::session_source_byte(selected, owner, offset) {
                Ok(byte) => return byte,
                Err(error) if selected.fx_sources.is_none() => {
                    panic!("prepared FX source: {error:?}")
                }
                Err(_) => {}
            }
        }
        let at = sm_game::fx_encoding::source_address(owner, offset).expect("FX source role");
        self.byte(at)
    }
    fn store_source_byte(
        &mut self,
        owner: sm_game::scene::hdma_sources::LiveOwner,
        offset: usize,
        byte: u8,
    ) {
        if self.persistent_is_current() {
            let selected = self.session_mut();
            match sm_game::fx_encoding::store_session_source_byte(selected, owner, offset, byte) {
                Ok(()) => return,
                Err(error) if selected.fx_sources.is_none() => {
                    panic!("prepared FX source: {error:?}")
                }
                Err(_) => {}
            }
        }
        let at = sm_game::fx_encoding::source_address(owner, offset).expect("FX source role");
        self.set_byte(at, byte);
    }
    // The projected byte store at the buffer's home: the FX source owners
    // claim it while one exists (`write_source_byte`), RAM otherwise.
    fn store_buffer_byte(
        &mut self,
        owner: sm_game::scene::hdma_sources::LiveOwner,
        offset: usize,
        byte: u8,
    ) {
        let at = sm_game::fx_encoding::source_address(owner, offset).expect("FX buffer role");
        self.set_byte(at, byte);
    }
    fn add_parallax_source(&mut self, selector: u16, delta: u32) {
        let at = usize::from(selector);
        let low = u16::from_le_bytes([self.parallax_byte(at), self.parallax_byte(at + 1)]);
        let high = u16::from_le_bytes([self.parallax_byte(at + 2), self.parallax_byte(at + 3)]);
        let sum = (u32::from(low) | (u32::from(high) << 16)).wrapping_add(delta);
        for (offset, byte) in sum.to_le_bytes().into_iter().enumerate() {
            self.store_parallax_byte(at + offset, byte);
        }
    }
}

impl sm_game::fx_domain::Controls for Runtime<'_> {
    fn room_blending(&self) -> sm_game::fx_domain::RoomBlending {
        sm_game::fx_domain::RoomBlending {
            primary: self.word(0x1982),
            liquid: self.word(0x1984),
        }
    }
    fn publish_room_blending(&mut self, value: sm_game::fx_domain::RoomBlending, primary: bool) {
        let (at, value) = if primary {
            (0x1982, value.primary)
        } else {
            (0x1984, value.liquid)
        };
        let bytes = value.to_le_bytes();
        self.set_byte(at, bytes[0]);
        self.set_byte(at + 1, bytes[1]);
    }
    fn fx_controls(&self) -> sm_game::fx_state::FxState {
        if self.persistent_is_current() {
            return sm_game::fx_encoding::session_controls(self.session());
        }
        sm_game::fx_encoding::decode(&|at| self.byte(at))
    }
    fn publish_fx_controls(
        &mut self,
        value: sm_game::fx_state::FxState,
        fields: &[sm_game::fx_state::Field],
    ) {
        for &field in fields {
            let at = sm_game::fx_encoding::address(field);
            let bytes = field.get(&value).to_le_bytes();
            self.set_byte(at, bytes[0]);
            self.set_byte(at + 1, bytes[1]);
        }
    }
}
impl sm_game::fx_lighting::Access for Runtime<'_> {
    fn lighting(&self) -> sm_game::visual_state::FirefleaLighting {
        if self.persistent_is_current() {
            return sm_game::fx_encoding::session_lighting(self.session());
        }
        sm_game::fx_encoding::decode_lighting(&|at| self.byte(at))
    }
    fn publish_lighting(
        &mut self,
        value: sm_game::visual_state::FirefleaLighting,
        fields: &[sm_game::fx_lighting::Field],
    ) {
        for &field in fields {
            let at = sm_game::fx_encoding::lighting_address(field);
            let bytes = field.get(&value).to_le_bytes();
            self.set_byte(at, bytes[0]);
            self.set_byte(at + 1, bytes[1]);
        }
    }
}

impl Runtime<'_> {
    fn parallax_byte(&self, at: usize) -> u8 {
        if let Some((owner, offset)) =
            sm_game::fx_encoding::physical_source_coordinate(self.session(), at)
                .expect("prepared FX source resolution")
        {
            return sm_game::fx_domain::Sources::source_byte(self, owner, offset);
        }
        // LEGACY(original-overflow): a parallax selector outside the FX sources.
        sm_game::legacy_overflow::slot_byte(self, at)
    }
    fn store_parallax_byte(&mut self, at: usize, byte: u8) {
        if let Some((owner, offset)) =
            sm_game::fx_encoding::physical_source_coordinate(self.session(), at)
                .expect("prepared FX source resolution")
        {
            sm_game::fx_domain::Sources::store_source_byte(self, owner, offset, byte);
        } else {
            // LEGACY(original-overflow): a parallax selector outside the FX sources.
            sm_game::legacy_overflow::set_slot_byte(self, at, byte);
        }
    }
}

impl Runtime<'_> {
    pub(crate) fn hdma_domain_word(&self, word: sm_game::hdma::Word) -> u16 {
        if self.persistent_is_current() {
            return self.session().hdma_word(word);
        }
        self.game.hdma_domain_word(word)
    }
    pub(crate) fn store_hdma_domain_byte(
        &mut self,
        word: sm_game::hdma::Word,
        high: bool,
        value: u8,
    ) {
        self.set_byte(word.address() + usize::from(high), value);
    }
    pub(crate) fn store_hdma_domain_word(
        &mut self,
        word: sm_game::hdma::Word,
        value: u16,
        order: sm_game::scene::state::WordOrder,
    ) {
        let at = word.address();
        let [first, second] = order.encode(value);
        self.set_byte(at, first);
        self.set_byte(at + 1, second);
    }
}

#[forbid(unsafe_code)]
impl sm_game::presentation_workspace::Access for Runtime<'_> {
    fn screen_apply<T>(
        &mut self,
        action: impl FnOnce(
            &mut dyn sm_game::presentation_workspace::Canvas,
        ) -> Result<T, sm_game::scene::state::DisplayError>,
    ) -> T {
        action(self.session_mut()).expect("screen operation")
    }
    fn screen_byte(&self, point: sm_game::presentation_workspace::Point) -> u8 {
        let workspace = self.session().presentation.workspace();
        match workspace.byte(point) {
            Ok(byte) => byte,
            Err(error) => match workspace.alias_address(point).expect("screen binding") {
                Some(at) => self.byte(at),
                None => panic!("screen allocation {error:?}"),
            },
        }
    }
    fn screen_write_byte(&mut self, point: sm_game::presentation_workspace::Point, value: u8) {
        let alias = self
            .session()
            .presentation
            .workspace()
            .alias_address(point)
            .expect("screen binding");
        if let Err(error) = self.session_mut().publish_workspace_byte(point, value) {
            match alias {
                Some(at) => self.set_byte(at, value),
                None => panic!("screen allocation {error:?}"),
            }
        }
    }
    fn screen_write_slice(&mut self, point: sm_game::presentation_workspace::Point, bytes: &[u8]) {
        let alias = self
            .session()
            .presentation
            .workspace()
            .alias_address(point)
            .expect("screen binding");
        if let Err(error) = self.session_mut().publish_workspace_slice(point, bytes) {
            match alias {
                Some(at) => self.write_slice(at, bytes),
                None => panic!("screen allocation {error:?}"),
            }
        }
    }
}
#[forbid(unsafe_code)]
impl sm_game::presentation_workspace::PaletteAccess for Runtime<'_> {
    fn palette_byte(&self, target: bool, offset: usize) -> u8 {
        self.byte(
            (if target { 0xc200usize } else { 0xc000usize })
                .checked_add(offset)
                .expect("palette offset"),
        )
    }
    fn palette_native_word(&self, target: bool, offset: usize) -> u16 {
        self.native_word(
            (if target { 0xc200usize } else { 0xc000usize })
                .checked_add(offset)
                .expect("palette offset"),
        )
    }
    fn palette_word(&self, target: bool, offset: usize) -> u16 {
        self.word(
            (if target { 0xc200usize } else { 0xc000usize })
                .checked_add(offset)
                .expect("palette offset"),
        )
    }
    fn palette_write_word(&mut self, target: bool, offset: usize, value: u16) {
        self.write_word_le(
            value,
            (if target { 0xc200usize } else { 0xc000usize })
                .checked_add(offset)
                .expect("palette offset"),
        );
    }
    fn palette_write_native_word(&mut self, target: bool, offset: usize, value: u16) {
        let at = (if target { 0xc200usize } else { 0xc000usize })
            .checked_add(offset)
            .expect("palette offset");
        let bytes = value.to_ne_bytes();
        self.set_byte(at, bytes[0]);
        self.set_byte(at + 1, bytes[1]);
    }
    fn palette_write_native_slice(&mut self, target: bool, offset: usize, value: u16) {
        self.write_slice_native(
            (if target { 0xc200usize } else { 0xc000usize })
                .checked_add(offset)
                .expect("palette offset"),
            value,
        );
    }
}
#[forbid(unsafe_code)]
impl sm_game::presentation_workspace::QueueAccess for Runtime<'_> {
    fn queue_byte_at(&mut self, offset: usize, value: u8) {
        self.set_byte(0xd0usize.checked_add(offset).expect("queue offset"), value);
    }
    fn queue_native_slice_at(&mut self, offset: usize, value: u16) {
        self.write_slice_native(0xd0usize.checked_add(offset).expect("queue offset"), value);
    }
}
#[forbid(unsafe_code)]
impl sm_game::presentation_workspace::ContinuationAccess for Runtime<'_> {
    fn screen_primary(&self) -> u8 {
        *self.session().control.primary()
    }
    fn screen_set_primary(&mut self, value: u8) {
        *self.session_mut().control.primary_mut() = value;
    }
    fn demo_phase(&self) -> u8 {
        self.byte(0x614)
    }
    fn demo_set_phase(&mut self, value: u8) {
        self.set_byte(0x614, value);
    }
}

#[forbid(unsafe_code)]
impl sm_game::dialogue_ram::Access for Runtime<'_> {
    fn dialogue_state(&self) -> sm_game::dialogue::Dialogue<u16> {
        sm_game::dialogue_ram::decode(sm_game::dialogue_ram::Encoded {
            index: self.word(0x1c1f),
            parent: self.byte(0x614),
            child: self.byte(0x615),
            delay: self.word(0x77e),
            radius: self.word(0x5a2),
            selection: self.word(0x5f9),
        })
    }
    fn dialogue_publish(&mut self, state: sm_game::dialogue::Dialogue<u16>) {
        let value = sm_game::dialogue_ram::encode(state);
        self.write_word_le(value.index, 0x1c1f);
        self.write_word_le(value.delay, 0x77e);
        self.write_word_le(value.radius, 0x5a2);
        self.write_word_le(value.selection, 0x5f9);
        self.set_byte(0x615, value.child);
        self.set_byte(0x614, value.parent);
    }
    fn dialogue_request(&self) -> u16 {
        self.word(0xdc8)
    }
    fn dialogue_set_request(&mut self, value: u16) {
        self.write_word_le(value, 0xdc8);
    }
    fn dialogue_active_index(&self) -> u16 {
        self.word(0x1c1f)
    }
    fn dialogue_clear_active(&mut self) {
        self.write_word_le(0u16, 0x1c1f);
    }
    fn dialogue_selection(&self) -> u16 {
        self.word(0x5f9)
    }
    fn dialogue_set_selection(&mut self, value: u16) {
        self.write_word_le(value, 0x5f9);
    }
    fn dialogue_radius(&self) -> u16 {
        self.word(0x5a2)
    }
    fn dialogue_set_radius(&mut self, value: u16) {
        self.write_word_le(value, 0x5a2);
    }
    fn dialogue_scratch(&self) -> [u16; 4] {
        [0x5a4, 0x5a6, 0x5a8, 0x5aa].map(|at| self.word(at))
    }
    fn dialogue_set_scratch(&mut self, index: usize, value: u16) {
        sm_data::table::check_index(index, 4);
        let at = 0x5a4 + 2 * index;
        self.write_word_le(value, at);
    }
    fn dialogue_input(&self) -> sm_game::dialogue::Input {
        let keys = self.word(0x8f);
        sm_game::dialogue::Input {
            confirm: keys & 0x80 != 0,
            cancel: keys & 0x8000 != 0,
            toggle: keys & 0x2300 != 0,
            dismiss: self.word(0x8b) != 0,
        }
    }
    fn dialogue_binding(&self, shoot: bool) -> u16 {
        let at = if shoot { 0x9b2 } else { 0x9b6 };
        self.word(at)
    }
    fn dialogue_select_pause(&mut self) {
        self.write_word_le(12u16, 0x998);
    }
}
#[forbid(unsafe_code)]
impl sm_game::menus::host::MapFacts for Runtime<'_> {
    fn file_palette(&self) -> u16 {
        u16::from_ne_bytes([self.byte(0xf96), self.byte(0xf97)])
    }
    fn set_file_palette(&mut self, value: u16) {
        self.write_word_le(value, 0xf96);
    }
    fn map_explored_byte(&self, offset: isize) -> u8 {
        let at = 0x7f7usize.checked_add_signed(offset).expect("map offset");
        self.byte(at)
    }
    fn store_map_explored_byte(&mut self, offset: isize, value: u8) {
        let at = 0x7f7usize.checked_add_signed(offset).expect("map offset");
        self.set_byte(at, value);
    }
    fn map_station_byte(&self, area: usize) -> u8 {
        self.byte(0xd908usize.checked_add(area).expect("station offset"))
    }
    fn used_station_native_word(&self, offset: usize) -> u16 {
        let at = 0xd8f8usize.checked_add(offset).expect("station offset");
        u16::from_ne_bytes([self.byte(at), self.byte(at + 1)])
    }
}

#[forbid(unsafe_code)]
impl sm_game::sprite_drawing::ObjectAccess for Runtime<'_> {
    fn object_native_word(&self, offset: usize) -> u16 {
        let order = self
            .session()
            .presentation
            .owner()
            .ok()
            .map(|owner| owner.state.objects.native_order);
        let at = 0x370usize.checked_add(offset).expect("object offset");
        let bytes = [self.byte(at), self.byte(at + 1)];
        order.map_or_else(|| u16::from_ne_bytes(bytes), |order| order.decode(bytes))
    }
    fn object_little_word(&self, offset: usize) -> u16 {
        let at = 0x370usize.checked_add(offset).expect("object offset");
        u16::from_le_bytes([self.byte(at), self.byte(at + 1)])
    }
    fn object_write_byte(&mut self, offset: usize, value: u8) {
        self.set_byte(
            0x370usize.checked_add(offset).expect("object offset"),
            value,
        );
    }
    fn object_write_native_slice(&mut self, offset: usize, value: u16) {
        let order = self
            .session()
            .presentation
            .owner()
            .ok()
            .map(|owner| owner.state.objects.native_order);
        let encoded = order.map_or_else(|| value.to_ne_bytes(), |order| order.encode(value));
        self.write_slice(
            0x370usize.checked_add(offset).expect("object offset"),
            &encoded,
        );
    }
    fn object_write_native_bytes(&mut self, offset: usize, value: u16) {
        let order = self
            .session()
            .presentation
            .owner()
            .ok()
            .map(|owner| owner.state.objects.native_order);
        let encoded = order.map_or_else(|| value.to_ne_bytes(), |order| order.encode(value));
        let at = 0x370usize.checked_add(offset).expect("object offset");
        let bytes = encoded;
        self.set_byte(at, bytes[0]);
        self.set_byte(at + 1, bytes[1]);
    }
}

#[forbid(unsafe_code)]
impl sm_game::escape_countdown::Access for Runtime<'_> {
    fn countdown_apply<T>(
        &mut self,
        action: impl FnOnce(&mut sm_game::escape_countdown::State, &mut u16) -> T,
    ) -> T {
        let timing = &mut self.session_mut().timing;
        action(&mut timing.countdown, &mut timing.timer_status)
    }
    fn countdown_byte(&self, offset: usize) -> u8 {
        self.byte(0x945usize.checked_add(offset).expect("countdown offset"))
    }
    fn countdown_native_word(&self, offset: usize) -> u16 {
        if offset < 6 {
            return self.session().timing.countdown.native_word(offset);
        }
        let at = 0x945usize.checked_add(offset).expect("countdown offset");
        u16::from_ne_bytes([self.byte(at), self.byte(at + 1)])
    }
}

#[forbid(unsafe_code)]
impl sm_game::grapple_swing::TileAccess for Runtime<'_> {
    fn tile_word(&self, field: sm_game::grapple_swing::TileField) -> u16 {
        if self.persistent_is_current() {
            field.current(self.session())
        } else {
            self.word(field.sequence_word() as usize)
        }
    }
    fn store_tile_word(&mut self, field: sm_game::grapple_swing::TileField, value: u16) {
        self.write_word_le(value, field.sequence_word() as usize);
    }
}
