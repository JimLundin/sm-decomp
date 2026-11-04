//! One aggregate authority for player, projectiles, mechanisms and their room.
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
use sm_game::scene::owned_samus::{SamusDrawing, SamusSources as _};
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

/// The native diagnostics a Samus draw reported, made on Game in order once
/// the draw returns: the draw reads only the content view, so Game is as it
/// was at each report.
fn report_draw_diagnostics(diagnostics: Vec<u32>, game: &mut Game) {
    for address in diagnostics {
        native_rom_diagnostic(address, game);
    }
}

#[forbid(unsafe_code)]
#[cfg(sm_verify_dll)]
#[path = "../verification/actor_movement_runtime_observer.rs"]
mod actor_movement_runtime_observer;

#[forbid(unsafe_code)]
#[cfg(sm_verify_dll)]
#[path = "../verification/transfer_checks.rs"]
mod transfer_checks;

/// `pub` inside the crate-private module, so `LeaseActors` can name it.
/// The enemy projectile spawn words as Game holds them: the rectangle, the
/// variant, the point and the drop definition.
#[cfg(sm_verify_dll)]
pub(crate) type NativeSpawn = (crate::types::Rect16U, u16, crate::types::Point16U, u16);
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

/// A compressed asset a lease decompresses into work RAM: a prepared room stream,
/// or the loaded image's bytes at a LoROM address.
pub(crate) enum Compressed<'a> {
    Prepared(&'a [u8]),
    Rom(u32),
}
impl Compressed<'_> {
    /// Decode into work RAM at `at` on the lease's byte path; the bytes produced.
    fn decompress(self, at: usize, game: &mut Runtime<'_>) -> usize {
        match self {
            Self::Prepared(stream) => {
                sm_game::decompression::decompress_prepared_table(stream, at, game)
            }
            Self::Rom(address) => sm_game::decompression::decompress_to_ram(address, at, game),
        }
    }
}

/// Magdollite's original diagnostic for an undefined lava height.
pub(crate) fn warn_lava_height(value: i32) {
    unsafe { crate::c_library::printf(c"Y undefined %d!\n".as_ptr(), value) };
}

/// The generic species tables run on the scheduler's lease; these are the arms they
/// still reach on this side, each on the lease.
impl sm_game::actors::species::TableLease for Runtime<'_> {
    fn lease_native_ai(&mut self, ea: u32) {
        match ea {
            // $A2:804C returns at once; the verifier's initializer and population cases hook it.
            10649676 => {
                #[cfg(sm_verify_dll)]
                crate::game::actors::reference_population_animation::dispatch_noop_on_lease(self);
            }
            // Main AI - Mother Brain's falling tubes ($A9:8B85).
            11111301 => {
                let k = self.session().actors.controls.selected;
                sm_game::actors::mother_brain_tubes::main_ai(self, k);
            }
            // Bank $A9's grapple reaction (cancel beam) and frozen AI.
            11108367 => actors::combat::select_frozen_ai(self),
            11108417 => actors::scheduling::update_frozen_enemy(self),
            _ => crate::rtl::unreachable(),
        }
    }
    fn lease_native_pre_instruction(&mut self, ea: u32, k: u16) {
        match ea {
            11767039 => self.owned_call(sm_game::actors::gamet::settle),
            11771328 | 11771517 | 11771616 | 11771700 | 11771770 => {
                self.owned_call(|runtime| actors::botwoon::pre_instruction(ea, k, runtime));
            }
            _ => crate::rtl::unreachable(),
        }
    }
    fn lease_native_instruction(&mut self, ea: u32, _actor: u16, j: EnemyScript) -> EnemyScript {
        // Mother Brain's body's instructions, in bank $A9.
        if ea >> 16 == 0xa9 {
            self.owned_call(|runtime| sm_a9::dispatch_enemy_instr(ea, j, runtime))
        } else {
            crate::rtl::unreachable()
        }
    }
    fn lease_warn_reflection(&mut self) {
        unsafe { crate::c_library::printf(c"Possible bug. What is X?\n".as_ptr()) };
    }
    fn lease_phantoon(&mut self, callback: sm_game::actors::phantoon::Callback) {
        self.owned_call(|runtime| sm_game::actors::phantoon::callback(runtime, callback));
    }
}

impl sm_game::actors::gunship::Host for Runtime<'_> {
    fn spawn_palette_effect(&mut self, definition: u16) {
        self.owned_player_call(|runtime| {
            crate::game::palette_effects::execution::spawn(runtime, definition)
        });
    }
    fn face_player_forward(&mut self) {
        self.owned_player_call(crate::game::player_transitions::equipment::face_forward);
    }
    fn event_happened(&mut self, event: u16) -> bool {
        self.owned_call(|runtime| sm_game::progress::event_happened(runtime, event))
    }
    fn restore(&mut self, resource: sm_game::item_pickups::Resource, amount: u16) {
        // The former foreign entry restored on a nested lease; this one keeps
        // its observation refresh.
        self.owned_player_call(|runtime| {
            crate::game::item_pickups_runtime::restore(resource, amount, runtime)
        });
    }
    fn message_box_response(&mut self, kind: u16) -> i32 {
        // The dialogue words are the lease's; the former foreign entry's
        // observation refresh is kept.
        self.owned_player_call(|runtime| sm_game::message_boxes::poll(kind, runtime))
    }
    fn save_game(&mut self, slot: u16) {
        // The save's words are the lease's; the observations are refreshed on
        // entry and on return, as the former foreign call left them.
        self.owned_player_call(|runtime| sm_game::menus::save_slots::save(slot, runtime));
    }
    fn run_other_pre_instruction(&mut self, address: u32) {
        // The pre-instruction table runs on this lease, as one owned call where the
        // game's had run in one foreign call.
        self.owned_call(|runtime| {
            sm_game::actors::species::table_pre_instruction(address, runtime);
        });
    }
}

/// The morph ball eye runs on the scheduler's lease, and its beam starts on
/// it; the observations are refreshed on entry and on return, as the former
/// foreign call left them.
impl sm_game::actors::morph_ball_eye::Host for Runtime<'_> {
    fn start_beam(&mut self) {
        self.owned_player_call(crate::game::visual_runtime::initialize_morph_eye_beam);
    }
}

impl<'g> Runtime<'g> {
    pub fn new(game: &'g mut Game) -> Self {
        #[cfg(sm_verify_dll)]
        player_normal_transfers::record(game, 0);
        #[cfg(sm_verify_dll)]
        lease_counters::lease();
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
            #[cfg(sm_verify_dll)]
            lease_counters::import();
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
        let mut drawing = SamusDrawing::new(self.game.samus_content());
        let drawn = persistent_session::draw_body(
            display.owner_mut(),
            &mut drawing,
            &mut state.player,
            &self.inputs,
            samus_publications.clone(),
            marker,
            mode,
        );
        report_draw_diagnostics(drawing.diagnostics, self.game);
        drawn.expect("owned Samus body");
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
        let mut drawing = SamusDrawing::new(self.game.samus_content());
        let drawn = persistent_session::bottom_visible(
            display.owner_mut(),
            &mut drawing,
            &mut state.player,
            &self.inputs,
            samus_publications.clone(),
            marker,
        );
        report_draw_diagnostics(drawing.diagnostics, self.game);
        drawn.expect("owned Samus bottom")
    }
    pub(crate) fn draw_owned_echo(&mut self, slot: u16) {
        use sm_game::scene::samus_content::{Registration, Role};
        let index = usize::from(slot >> 1);
        let y = self.word(0xab8 + index * 2);
        let source = self
            .game
            .samus_content()
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
            .samus_content()
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
        let mut drawing = SamusDrawing::new(self.game.samus_content());
        let drawn = persistent_session::draw_sprite(
            display.owner_mut(),
            &mut drawing,
            &mut state.player,
            &self.inputs,
            samus_publications.clone(),
            sm_game::scene::owned_samus::Sprite {
                producer,
                index,
                position,
            },
        );
        report_draw_diagnostics(drawing.diagnostics, self.game);
        drawn.expect("owned Samus publication");
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
    /// Write this lease's Session to RAM now, as its release would export it, and keep the
    /// lease: the Session stays the authority, nothing is decoded, and RAM projects it. A
    /// port runs this before a DMA that reads work RAM, since the hardware reads the bytes
    /// there (`start_dma`). An export deferred by an earlier release is written first, as
    /// any RAM write writes it; the kept frame Session's write log, when open, logs the
    /// bytes, which the next lease takes up as it would this lease's release. A persistent
    /// lease takes the new write epoch, as its release does.
    pub(crate) fn publish(&mut self) {
        assert!(self.persistent_is_current(), "Session lease is dirty");
        preflight_export(self.session(), self.game);
        let state = self.state.as_ref().expect("Session lease");
        self.game
            .ram()
            .copy_spans(|ram| encoding::encode_spans(state, ram));
        if self.persistent_epoch.is_some() {
            self.persistent_epoch = Some(self.game.compatibility_write_epoch());
        }
    }
    /// Samus's NMI tile upload from the lease. It reads its descriptors
    /// ($7E:071D..0722) from the lease and the cartridge, and writes registers
    /// and VRAM alone. The publish keeps RAM where the NMI's later uploads
    /// found it.
    pub(crate) fn upload_samus_tiles(&mut self) {
        self.owned_player_call(|runtime| {
            runtime.publish();
            let descriptors = std::array::from_fn(|lane| runtime.byte(0x71d + lane));
            // SAFETY: a lease runs with the backend installed, and `&mut Game`
            // excludes other access.
            unsafe { crate::game::display_transfers::upload_samus(descriptors, runtime.game) }
        });
    }
    /// One device register write from the lease (`rtl::write_device`).
    pub(crate) fn write_device(&mut self, register: u16, value: u8) {
        // SAFETY: a lease runs with the backend installed, and `&mut Game`
        // excludes other access.
        unsafe { crate::rtl::write_device(register, value, self.game) }
    }
    /// The typed uploads' bracket around a port's register writes, which reaches
    /// the selected PPU alone (`Game::producing`).
    pub(crate) fn producing(&mut self) {
        self.game.producing();
    }
    /// The bracket's end: `uploads` are what the port's writes did.
    pub(crate) fn produced(
        &mut self,
        producer: &'static str,
        uploads: &[sm_game::display::video::Upload],
    ) {
        self.game.produced(producer, uploads);
    }
    /// One device bus read from the lease (`rtl::read_device`).
    pub(crate) fn read_device(&mut self, address: u32) -> u8 {
        // SAFETY: as for `write_device`.
        unsafe { crate::rtl::read_device(address, self.game) }
    }
    /// Start the DMA channels in `mask` from the lease, as an MDMAEN write starts them,
    /// reading the prepared room image `source` when given. The transfer reads work RAM,
    /// which a port has published (`publish`): verify builds assert that RAM holds the
    /// Session's projection of every work RAM byte the channels read. The work RAM bytes
    /// the channels write (a B-to-A transfer, or one to the WRAM data port) are taken
    /// back into the Session, as a decode of just those bytes would read them.
    pub(crate) fn start_dma(
        &mut self,
        mask: u8,
        source: Option<&crate::execution::DmaRomSource<'_>>,
    ) {
        assert!(self.persistent_is_current(), "Session lease is dirty");
        let (_reads, writes) = self.dma_wram(mask);
        #[cfg(sm_verify_dll)]
        transfer_checks::assert_published(self.session(), self.game, &_reads);
        // SAFETY: as for `write_device`.
        unsafe { crate::rtl::start_device_dma(mask, source, self.game) };
        if !writes.is_empty() {
            let written = writes.into_iter().map(|at| at..at + 1).collect::<Vec<_>>();
            take_written_bytes(
                self.state.as_mut().expect("Session lease"),
                self.game,
                &written,
            );
            if self.persistent_epoch.is_some() {
                self.persistent_epoch = Some(self.game.compatibility_write_epoch());
            }
        }
    }
    /// The work RAM bytes the DMA channels in `mask` read and write, from their channel
    /// registers and the WRAM data port's address, in channel order.
    fn dma_wram(&mut self, mask: u8) -> (Vec<usize>, Vec<usize>) {
        /// The work RAM byte at A bus `bank:address`, if it is one.
        fn wram(bank: u8, address: u16) -> Option<usize> {
            match bank {
                0x7e | 0x7f => Some((usize::from(bank & 1) << 16) | usize::from(address)),
                0x00..=0x3f | 0x80..=0xbf if address < 0x2000 => Some(usize::from(address)),
                _ => None,
            }
        }
        let mut reads = Vec::new();
        let mut writes = Vec::new();
        // SAFETY: as for `write_device`.
        let mut port = unsafe { crate::rtl::wram_port_address(self.game) };
        for channel in (0..8u32).filter(|channel| mask & (1 << channel) != 0) {
            let base = 0x4300 | (channel << 4);
            let mode = self.read_device(base);
            let target = self.read_device(base + 1);
            let start =
                u16::from_le_bytes([self.read_device(base + 2), self.read_device(base + 3)]);
            let bank = self.read_device(base + 4);
            let count =
                u16::from_le_bytes([self.read_device(base + 5), self.read_device(base + 6)]);
            let count = if count == 0 {
                0x10000
            } else {
                u32::from(count)
            };
            // DMAP bit 3 fixes the A bus address; otherwise bit 4 steps it down.
            let step: u16 = match (mode & 0x08 != 0, mode & 0x10 != 0) {
                (true, _) => 0,
                (false, true) => 0xffff,
                (false, false) => 1,
            };
            let to_bus_b = mode & 0x80 == 0;
            let to_wram_port = to_bus_b && target == 0x80;
            let mut address = start;
            for _ in 0..count {
                if let Some(at) = wram(bank, address) {
                    if to_bus_b {
                        reads.push(at);
                    } else {
                        writes.push(at);
                    }
                }
                if to_wram_port {
                    writes.push(port);
                    port = (port + 1) & 0x1ffff;
                }
                address = address.wrapping_add(step);
            }
        }
        (reads, writes)
    }
    /// Decompress `source` into work RAM at `at`, as the original's decoder writes it; the
    /// bytes it produced. The decoder runs on the lease's byte path, where its back
    /// references read the bytes it wrote; then the observations are refreshed, as the
    /// former take-back of the published image's bytes left them.
    pub(crate) fn decompress_to_ram(&mut self, source: Compressed<'_>, at: usize) -> usize {
        let produced = source.decompress(at, self);
        self.refresh_player_observations();
        self.refresh_actor_observations();
        produced
    }
    /// Decompress the room's level data from `source` to $7F:0000, then split its planes
    /// (`copy_level_planes`), on the lease's byte path as `decompress_to_ram` runs.
    pub(crate) fn decompress_level(&mut self, source: Compressed<'_>) {
        source.decompress(0x10000, self);
        self.copy_level_planes();
        self.refresh_player_observations();
        self.refresh_actor_observations();
    }
    /// Split the level data the decoder left at $7F:0000 into its planes: the
    /// layer 2 tiles to $7F:9602 and the BTS to $7F:6402, each sized by the
    /// level data's little-endian size word.
    fn copy_level_planes(&mut self) {
        let size = usize::from(self.word(sm_game::ram::words::RAM7_F_START));
        for (source, destination, count) in [
            (0x10002 + size + size / 2, 0x19602, size),
            (0x10002 + size, 0x16402, size / 2),
        ] {
            // Valid room arrays can overlap. Preserve their pre-copy contents;
            // malformed extents retain the checked forward loop's write prefix.
            if source < destination + count
                && destination < source + count
                && source + count <= 0x20000
                && destination + count <= 0x20000
            {
                crate::verify_probe::hit(if destination == 0x19602 {
                    "room.level.layer2_overlap"
                } else {
                    "room.level.bts_overlap"
                });
                let bytes: Vec<u8> = (source..source + count).map(|at| self.byte(at)).collect();
                self.write_slice(destination, &bytes);
            } else {
                for index in 0..count {
                    let value = self.byte(source + index);
                    self.set_byte(destination + index, value);
                }
            }
        }
    }
    /// The game's imported visual tables.
    pub(crate) fn visual_tables(&self) -> &sm_game::visual_tables::ImportedTables {
        &self.game.visual_tables
    }
    /// The RomPtr invalid-window diagnostic on the lease. It prints and marks the
    /// game failed, reading and storing no RAM, so the Session stays current; the
    /// observations are refreshed on entry and on return, as the former foreign
    /// call left them.
    pub(crate) fn report_invalid_window(&mut self, address: u32) {
        self.owned_player_call(|runtime| invalid_window(runtime.game, address));
    }
    /// The native-pointer RomPtr diagnostic on the lease: it prints and marks the
    /// game failed, as `report_invalid_window`, and the verifier's low-window
    /// observer snapshots the lease's projected RAM where it copied the RAM the
    /// former foreign call had exported. The observations are refreshed on entry
    /// and on return, as the foreign call left them.
    pub(crate) fn report_native_diagnostic(&mut self, address: u32) {
        self.owned_player_call(|runtime| {
            invalid_window(runtime.game, address);
            #[cfg(sm_verify_dll)]
            {
                // The case's injection replaces the loaded image, which the lease
                // does not project: the image after it is the one before.
                let ram = (0..runtime.game.ram().len())
                    .map(|at| runtime.byte(at))
                    .collect::<Vec<u8>>();
                let game = &mut *runtime.game;
                crate::room_contact_start_observer::diagnostic_image(
                    address,
                    || ram.clone(),
                    &mut game.loaded_rom,
                    &game.g_fail,
                );
            }
        });
    }
    /// A RomRecord in a bank's low window ($xx:0000..$7FFF) on the lease, as
    /// `rtl::rom_record` takes it: the RomPtr diagnostic prints and marks the game
    /// failed, the verifier's low-window observers run, and the record is the
    /// loaded image's at the address, captured after them. The observers snapshot
    /// a detached image of the lease's projected RAM where they had the RAM the
    /// former foreign call exported, and the bytes they inject are stored back
    /// through the projection, where the foreign call's import took them. The
    /// observations are refreshed on entry and on return, as the foreign call
    /// left them.
    pub(crate) fn low_window_record(&mut self, bank: u8, address: u16) -> RomRecord {
        let address = (u32::from(bank) << 16) | u32::from(address);
        self.owned_player_call(|runtime| {
            invalid_window(runtime.game, address);
            #[cfg(sm_verify_dll)]
            runtime.observe_low_window(address);
            RomRecord {
                image: runtime
                    .game
                    .loaded_rom
                    .as_ref()
                    .expect("ROM must be loaded")
                    .clone(),
                base: crate::rtl::lorom_offset(address) as usize,
            }
        })
    }
    /// Run the verifier's low-window observers over the lease's projected RAM and
    /// store back each byte they changed, in address order.
    #[cfg(sm_verify_dll)]
    fn observe_low_window(&mut self, address: u32) {
        let spawn = self.native_spawn();
        self.observe_detached(|game, ram| game.observe_low_window(address, ram, spawn));
    }
    /// The Session's enemy projectile spawn words in Game's native shape, as the
    /// release exports them (`export_native_spawn`), for a verifier observer that
    /// snapshots them.
    #[cfg(sm_verify_dll)]
    pub(crate) fn native_spawn(&self) -> NativeSpawn {
        let spawn = &self.session().enemy_projectiles.spawn;
        (
            crate::types::Rect16U {
                x: spawn.rectangle.origin.x,
                y: spawn.rectangle.origin.y,
                w: spawn.rectangle.extent.x,
                h: spawn.rectangle.extent.y,
            },
            spawn.variant,
            crate::types::Point16U {
                x: spawn.point.x,
                y: spawn.point.y,
            },
            spawn.drop_definition,
        )
    }
    /// Run a verifier hook on the lease where it ran on Game: over a detached image
    /// of the lease's projected RAM and the Session's native spawn words, storing
    /// back each byte it changes. The observations are refreshed on entry and on
    /// return, as the former foreign call left them.
    #[cfg(sm_verify_dll)]
    pub(crate) fn observe_hook<R>(
        &mut self,
        observe: impl FnOnce(&mut Game, &crate::game_state::WorkRam, &NativeSpawn) -> R,
    ) -> R {
        self.owned_player_call(|runtime| {
            let spawn = runtime.native_spawn();
            let mut result = None;
            runtime.observe_detached(|game, ram| result = Some(observe(game, ram, &spawn)));
            result.expect("observed hook returns")
        })
    }
    /// Register `image` as the game's native image from the lease: a verifier
    /// hook's environmental input, which reads and stores no RAM.
    #[cfg(sm_verify_dll)]
    pub(crate) fn register_native_rom(&mut self, image: RomImage) {
        self.game.register_native_rom(image);
    }
    /// Run a verifier observer over a detached image of the lease's projected RAM,
    /// where it had the RAM the former foreign call exported, and store back each
    /// byte it changed, in address order, where the foreign call's import took it.
    #[cfg(sm_verify_dll)]
    pub(crate) fn observe_detached(
        &mut self,
        observe: impl FnOnce(&mut Game, &crate::game_state::WorkRam),
    ) {
        let before = (0..self.game.ram().len())
            .map(|at| self.byte(at))
            .collect::<Vec<u8>>();
        let ram = crate::game_state::WorkRam::detached(&before);
        observe(self.game, &ram);
        for (at, old) in before.into_iter().enumerate() {
            let new = ram.read_byte(at);
            if new != old {
                self.set_byte(at, new);
            }
        }
    }
    /// The room background's initial-load RomPtr diagnostic on the lease: it prints
    /// and marks the game failed, then the verifier's room background observer runs
    /// over a detached image of the projected RAM, whose injected bytes are stored
    /// back. The observations are refreshed on entry and on return, as the former
    /// foreign call left them.
    pub(crate) fn room_background_diagnostic(&mut self, address: u32) {
        self.owned_player_call(|runtime| {
            invalid_window(runtime.game, address);
            #[cfg(sm_verify_dll)]
            runtime.observe_detached(|game, ram| game.observe_room_background(address, ram));
        });
    }
    /// NextRandom ($80:8111) on the lease: the Session's random number steps once. The
    /// observations are refreshed on entry and on return, as the former foreign call left them.
    pub(crate) fn next_random_number(&mut self) -> u16 {
        self.owned_player_call(|runtime| {
            let value = sm_game::random::next(runtime.session().timing.random_number);
            runtime.session_mut().timing.random_number = value;
            #[cfg(sm_verify_dll)]
            crate::cpu_infra::player_transitions_composed::random_after_lease(runtime);
            value
        })
    }
    /// The audio request bytes ($7E:0617..0640, $0643..0687, $07F3..07F6) are the Session's on every
    /// lease: its control projects each of them, so a request the lease queues reaches its
    /// owner, and RAM takes it when the lease exports.
    #[cfg(sm_verify_dll)]
    pub(crate) fn assert_audio_requests_owned(&self) {
        let state = self.session();
        assert!(
            AUDIO_REQUESTS
                .iter()
                .flat_map(Clone::clone)
                .all(|at| state.control.projected_byte(at).is_some()),
            "the Session's control projects no audio request byte"
        );
    }
    /// The cue port: deliver `cue` to the APU at its original call point, with the lease
    /// open. Of the audio requests, the APU side reads only the upload flags ($0617, $0618),
    /// from RAM. Only an upload writes them, to RAM, and it leaves them clear, so RAM and the
    /// Session's audio owner hold the same flags; verify builds check that the APU side
    /// decides on the flags the lease holds.
    pub(crate) fn emit_cue(
        &mut self,
        cue: crate::native_audio::CompatibilityCue<'_>,
        phase: sm_data::CuePhase,
    ) {
        #[cfg(sm_verify_dll)]
        {
            let control = &self.session().control;
            let uploading = [0x617, 0x618]
                .into_iter()
                .any(|at| control.projected_byte(at).expect("audio upload flag") != 0);
            assert!(
                self.game.uploading_apu() == uploading,
                "the lease's audio upload flags differ from RAM at a cue"
            );
        }
        unsafe { crate::native_audio::emit(cue, phase, self.game) }
    }
    /// Boot's clear of the APU's four input ports from the lease, in port order.
    /// The ports read only the upload flags ($0617, $0618) from RAM, as a cue
    /// does (`emit_cue`); verify builds check that RAM holds the lease's flags.
    pub(crate) fn clear_apu_ports(&mut self) {
        use crate::rtl::{APUI00, APUI01, APUI02, APUI03};
        #[cfg(sm_verify_dll)]
        {
            let control = &self.session().control;
            let uploading = [0x617, 0x618]
                .into_iter()
                .any(|at| control.projected_byte(at).expect("audio upload flag") != 0);
            assert!(
                self.game.uploading_apu() == uploading,
                "the lease's audio upload flags differ from RAM at the APU ports' clear"
            );
        }
        for port in [APUI00, APUI01, APUI02, APUI03] {
            // SAFETY: a lease runs with the backend installed, and `&mut Game`
            // excludes other access.
            unsafe { crate::rtl::rtl_apu_write(port, 0, self.game) }
        }
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
                if self.game.frame_session.active {
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
                // A rejected lease retains this Runtime's identity owner; discard
                // invalid byte/native publication rather than panic again.
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
                self.game.revoke_samus_authority();
            } else {
                self.game.revoke_inventory_domain();
                self.game.revoke_menu_domain();
                self.game.revoke_fx_domain();
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
    // The Maw's main AI stores its off-screen word on its own lease; two
    // verification cases still check this fresh-lease store (D7).
    #[cfg(sm_verify_dll)]
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
        if sm_game::display_scroll::BYTES.contains(&at)
            && self.persistent_is_current()
            && !self.display_owned()
        {
            // The Legacy presentation's scroll words, refreshed as a scroll store.
            encoding::write_projected_byte(self.session_mut(), at, value);
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
    /// setting, the turn request, the HUD auto-cancel flag, the grapple beam's
    /// previous start and the time-frozen flag, which the Session owns and RAM
    /// holds only once it is published. A device interval publishes nothing:
    /// the flag the X-ray scope sets just before its HDMA spawn, or the reserve
    /// tanks set at the end of a frame, reaches RAM only at the lease's next
    /// export, and the enemy BG2 tilemap transfer reads it after these stores.
    pub(crate) fn refresh_scroll_inputs(&mut self) {
        // The frame display's bytes (the NMI counters, the Mode 7 words) are the
        // Legacy presentation's, which RAM holds once it is published.
        let presentation = &self.session().presentation;
        let inputs = encoding::inputs(&|lane| {
            presentation
                .legacy_display_byte(lane)
                .unwrap_or_else(|| self.game.ram().read_byte(lane))
        });
        self.inputs = inputs;
        let session = self.session();
        let frozen = session.timing.time_frozen;
        let origin = session.enemy_projectiles.drop_origin;
        let (moonwalk, turn_request) = (session.player.moonwalk, session.player.turn_request);
        let previous_start = session.player.swing.previous_start;
        let hud_auto_cancel = session.player.hud_auto_cancel;
        self.inputs.drop_origin_x = origin.x;
        self.inputs.drop_origin_y = origin.y;
        self.inputs.transitions.moonwalk_enabled = moonwalk;
        self.inputs.transitions.turn_request = turn_request;
        self.inputs.hud_auto_cancel = hud_auto_cancel;
        self.inputs.grapple_x = previous_start.x;
        self.inputs.grapple_y = previous_start.y;
        self.inputs.frozen = frozen;
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
    /// The loaded ROM's byte at LoROM `address` plus `offset`, with the game's checks.
    pub(crate) fn rom_byte(&self, address: u32, offset: usize) -> u8 {
        self.game.read_rom_byte(address, offset)
    }
    /// The loaded ROM's little-endian word at LoROM `address` plus `offset`, with
    /// the game's checks.
    pub(crate) fn rom_word(&self, address: u32, offset: usize) -> u16 {
        self.game.read_rom_word(address, offset)
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
            self.low_window_record(bank, cursor)
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
            self.report_invalid_window(address);
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
        if !matches!(
            (bank, limit),
            (1, 6 | 9 | 15) | (2, 6) | (3, 3 | 6 | 9 | 15)
        ) {
            unreachable!("unknown projectile sound queue");
        }
        self.queue_sound_request(usize::from(bank - 1), sound as u8, limit)
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
        self.owned_player_call(|runtime| {
            crate::game::visual_runtime::set_hdma_enabled(true, runtime)
        });
    }
    fn spawn_power_bomb(&mut self) {
        // The explosion words and the HDMA objects are the lease's; the owned
        // call keeps the former foreign entry's observation refresh.
        self.owned_player_call(sm_game::explosion_window::spawn_power_bomb);
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
        crate::rtl::invalid_interrupt_failure()
    }
    fn unknown(&mut self) -> bool {
        crate::rtl::unreachable()
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
    let mut pass = mechanism_pass(game.ram().read_byte(0x613));
    let result = {
        let mut runtime = Runtime::new(game);
        sm_game::room_mechanisms::scheduler::advance(&mut runtime, &mut pass)
    };
    let (resume, yielded) = mechanism_result(context, result);
    game.compatibility_write_owned_byte(0x613, resume);
    yielded
}

/// The gameplay pass's room mechanisms on the lease it holds: the pass byte
/// ($7E:0613) is read and stored on the lease.
pub(crate) fn run_mechanisms_in(
    runtime: &mut Runtime<'_>,
    context: sm_game::room_mechanisms::host::CallerContext,
) -> u8 {
    let mut pass = mechanism_pass(runtime.byte(0x613));
    let result = sm_game::room_mechanisms::scheduler::advance(runtime, &mut pass);
    let (resume, yielded) = mechanism_result(context, result);
    runtime.set_byte(0x613, resume);
    yielded
}

fn mechanism_pass(resume: u8) -> sm_game::room_mechanisms::host::Pass {
    use sm_game::room_mechanisms::host::Pass;
    match resume {
        0 => Pass::Instructions,
        1 => Pass::DisplayMessage,
        other => Pass::Invalid(other),
    }
}

/// The pass byte a run leaves and the value it yields.
fn mechanism_result(
    context: sm_game::room_mechanisms::host::CallerContext,
    result: sm_game::room_mechanisms::host::RunResult,
) -> (u8, u8) {
    use sm_game::room_mechanisms::host::{CallerContext, RunResult};
    match context {
        CallerContext::Gameplay | CallerContext::DoorTransition => match result {
            RunResult::Complete => (0, 0),
            RunResult::Yield(value) => (1, value.get()),
        },
    }
}

/// The audio request bytes: the APU upload flag and the music and sound queues, the sound
/// handler's downtime, and the music data and track indices.
#[cfg(sm_verify_dll)]
const AUDIO_REQUESTS: [std::ops::Range<usize>; 3] = [0x617..0x641, 0x643..0x688, 0x7f3..0x7f7];
/// The verifier's check at an APU-side read on Game, which took `byte` from RAM at audio
/// request byte `at`: while the frame's kept Session is current with RAM, its audio owner
/// holds that very byte.
#[cfg(sm_verify_dll)]
pub(crate) fn assert_kept_audio_matches(game: &Game, at: usize, byte: u8) {
    let Some(KeptSession {
        state: Some(state),
        epoch,
        ..
    }) = game.frame_session.kept.as_ref()
    else {
        return;
    };
    if *epoch != game.compatibility_write_epoch()
        || !AUDIO_REQUESTS.iter().any(|range| range.contains(&at))
    {
        return;
    }
    assert!(
        state.control.projected_byte(at) == Some(byte),
        "the kept Session's audio requests differ from RAM at an APU read"
    );
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
    #[cfg(sm_verify_dll)]
    lease_counters::tick();
    result
}

/// The RAM the next lease takes up beside the kept frame Session, as `reuse_frame_session`
/// reads it: the ranges written since its export (empty when nothing was written), or `None`
/// when the next lease decodes RAM instead (no kept Session, or an overflowed log). For the
/// verifier's Session-space comparison.
#[cfg(sm_verify_dll)]
pub(crate) fn kept_session_writes(game: &Game) -> Option<Vec<std::ops::Range<usize>>> {
    let kept = game.frame_session.kept.as_ref()?;
    if kept.epoch == game.compatibility_write_epoch() {
        return Some(Vec::new());
    }
    game.ram().logged_writes()
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
/// Where a lease opened now would take its Session from, as `Runtime::new` chooses.
#[cfg(sm_verify_dll)]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum LeaseSource {
    /// The persistent Session.
    Persistent,
    /// The kept frame Session, with the RAM written since its export.
    Kept,
    /// A decode of RAM.
    Import,
}
/// A copy of the Session a lease opened now would hold, built from `image` (RAM as
/// `WorkRam::peek_image` sees it) without opening the lease: nothing is taken from Game,
/// no epoch moves and no deferred export is written. Game's held owners (actor identity,
/// inventory, FX and menu domains) are not taken in, so their lanes may differ from RAM.
#[cfg(sm_verify_dll)]
pub(crate) fn preview_lease_session(game: &Game, image: &[u8]) -> (Session, LeaseSource) {
    if let Some(state) = game.peek_persistent_gameplay() {
        return (state, LeaseSource::Persistent);
    }
    let kept = game.frame_session.kept.as_ref().and_then(|kept| {
        let state = kept
            .state
            .clone()
            .or_else(|| game.ram().peek_deferred_session())?;
        let written = if kept.epoch == game.compatibility_write_epoch() {
            Vec::new()
        } else {
            game.ram().logged_writes()?
        };
        same_world(&state, game).then_some((state, written))
    });
    if let Some((mut state, written)) = kept {
        for at in written.iter().flat_map(Clone::clone) {
            let value = image[at];
            if encoding::projected_byte(&state, at).is_some_and(|byte| byte != value) {
                encoding::write_projected_byte(&mut state, at, value);
            }
        }
        import_native_spawn(&mut state, game);
        return (state, LeaseSource::Kept);
    }
    let mut state = encoding::decode(&|at| image[at], 0x20000, game.world().clone());
    import_native_spawn(&mut state, game);
    (state, LeaseSource::Import)
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
    import_session_image(game)
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
            self.report_native_diagnostic(address);
        }
        content::captured(self.native_image(), address)
    }

    /// Preserve the actor content diagnostic observer and allocation capture order.
    pub(crate) fn actor_legacy_record(&mut self, bank: u8, address: u16) -> RomRecord {
        if address & 0x8000 == 0 {
            self.low_window_record(bank, address)
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
            // The generic table on this lease, as one owned call where the game's
            // had run in one foreign call.
            _ => self.owned_call(|runtime| sm_game::actors::species::table_ai(routine, runtime)),
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
        // The generic table on this lease, as one owned call where the game's had run
        // in one foreign call.
        self.owned_call(|runtime| species::table_instruction(routine, actor, script, runtime))
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
            self.empty_or_unreachable(species::EMPTY_TOUCHES.contains(&routine));
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
            self.empty_or_unreachable(species::EMPTY_SHOTS.contains(&routine));
        }
    }
    /// What the game's table did with a routine no lease arm runs: an empty one
    /// returned at once, and that keeps the former foreign call's observation
    /// refresh; any other was the original's unreachable failure, which reads
    /// no game state.
    fn empty_or_unreachable(&mut self, empty: bool) {
        if empty {
            self.owned_player_call(|_| ());
        } else {
            sm_game::diagnostics::Fatal::unreachable(self);
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
            // No other draw hook exists: the table's fallback is the original's
            // unreachable failure, which reads no game state.
            sm_game::diagnostics::Fatal::unreachable(self);
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
        legacy_assertion(drawing)
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
pub(crate) use player_movement_runtime::{advance_special_movement, ceres_movement};
#[path = "player_transitions_runtime.rs"]
mod player_transitions_runtime;
pub(crate) use player_transitions_runtime::{
    command_in as player_command_in, finish_player_frame, initialize as initialize_player,
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
#[path = "../verification/lease_counters.rs"]
mod lease_counters;
#[cfg(sm_verify_dll)]
#[path = "../verification/player_normal_transfers.rs"]
pub(crate) mod player_normal_transfers;

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
// player rules read and write the lease, and its game-side effects (sound,
// unreachable failure, pose command) go out through the lease's ports.
impl sm_game::ram::CompatibilityRam for Runtime<'_> {
    fn observed_inputs(&self) -> sm_game::external_inputs::ExternalInputs {
        Runtime::observed_inputs(self)
    }
}
impl crate::projection::ProjectedBytes for Runtime<'_> {
    fn compatibility_byte(&self, at: usize) -> u8 {
        Runtime::byte(self, at)
    }
    fn compatibility_write_byte(&mut self, at: usize, value: u8) {
        Runtime::set_byte(self, at, value)
    }
}
// The multipliers hold no WRAM, ROM or Samus state: each port call is one device
// interval on the backend, with the observations refreshed on entry and on return
// as the former foreign call left them.
impl sm_game::hardware::Multiplier for Runtime<'_> {
    fn mode7_product(&mut self, multiplicand: u16, multiplier: u8) -> u16 {
        use crate::rtl::{M7A, M7B, MPYM, read_device, write_device};
        let [low, high] = multiplicand.to_le_bytes();
        self.owned_player_call(|runtime| {
            let game = &mut *runtime.game;
            // SAFETY: a lease runs with the backend installed, and `&mut Game`
            // excludes other access.
            unsafe {
                write_device(M7A as u16, low, game);
                write_device(M7A as u16, high, game);
                write_device(M7B as u16, multiplier, game);
                let low = read_device(MPYM, game);
                let high = read_device(MPYM + 1, game);
                u16::from_le_bytes([low, high])
            }
        })
    }
    fn cpu_multiplicand(&mut self, operand: u8) {
        self.owned_player_call(|runtime| {
            // SAFETY: as for `mode7_product`.
            unsafe { crate::rtl::write_device(crate::rtl::WRMPYA as u16, operand, runtime.game) }
        });
    }
}
// A bus read: WRAM and its low mirror come from the lease's projection, as the
// former foreign call's export left them in RAM; the WRAM data port reads the
// lease's byte at its address and advances it; any other address is one device
// read on the backend. The observations are refreshed on entry and on return.
impl sm_game::hardware::Bus for Runtime<'_> {
    fn read_bus_byte(&mut self, address: u32) -> u8 {
        self.owned_player_call(|runtime| {
            let [offset_low, offset_high, bank, _] = address.to_le_bytes();
            let offset = usize::from(u16::from_le_bytes([offset_low, offset_high]));
            let system = matches!(bank, 0x00..=0x3f | 0x80..=0xbf);
            if matches!(bank, 0x7e | 0x7f) {
                return runtime.byte((usize::from(bank & 1) << 16) | offset);
            }
            if system && offset < 0x2000 {
                return runtime.byte(offset);
            }
            // SAFETY: a lease runs with the backend installed, and `&mut Game`
            // excludes other access.
            unsafe {
                if system && offset == 0x2180 {
                    let at = crate::rtl::advance_wram_port(runtime.game);
                    return runtime.byte(at);
                }
                crate::rtl::read_device(address, runtime.game)
            }
        })
    }
}
// The decompressor on the lease: its output and back-references are the lease's
// bytes, its source cursor the game's, and its source the loaded image.
impl sm_game::decompression::DecompressionHost for Runtime<'_> {
    fn decompress_source(&self) -> u32 {
        self.game.decompress_src
    }
    fn set_decompress_source(&mut self, source: u32) {
        self.game.decompress_src = source;
    }
    fn report_invalid_rom_address(&mut self, address: u32) {
        crate::rtl::report_invalid_rom_address(address, self.game);
    }
    fn read_rom_byte(&self, address: u32, offset: usize) -> u8 {
        self.game.read_rom_byte(address, offset)
    }
}
// A device-only display command writes registers alone: the backend encodes it
// in one device interval. A command that starts a DMA or moves WRAM runs on the
// lease's transfer port: the lease is published, so the DMA reads the Session's
// bytes, and the bytes a readback writes return to the Session. Either way the
// observations are refreshed on entry and on return, as the former foreign call
// left them.
impl sm_game::display::video::Video for Runtime<'_> {
    fn apply(&mut self, producer: &'static str, command: sm_game::display::video::Command) {
        use crate::game::display_commands::{apply_device, apply_on_lease, device_only};
        if device_only(&command) {
            self.owned_player_call(|runtime| apply_device(command, runtime.game));
        } else {
            self.owned_player_call(|runtime| apply_on_lease(producer, command, runtime));
        }
    }
}
// The beam on the frame's lease: its events are the device's (`cpu_infra::beam`),
// and HDMA reads work RAM, which the lease has published; each scanline
// interrupt runs on this lease (`scanline_irq::advance_on_lease`).
impl sm_game::scene::settle::SettlementHost for Runtime<'_> {
    type Error = std::convert::Infallible;

    // SAFETY, for each call below: a lease runs with the backend installed, and
    // `&mut Game` excludes other access.
    fn reset_beam(&mut self) {
        unsafe { crate::cpu_infra::beam::reset(self.game) }
    }
    fn nmi_wanted(&self) -> bool {
        unsafe { crate::cpu_infra::beam::nmi_wanted(self.game) }
    }
    fn advance_at(&mut self, position: u16) -> Result<(), Self::Error> {
        unsafe { crate::cpu_infra::beam::advance_at(self.game, position) };
        Ok(())
    }
    fn vertical_irq_enabled(&self) -> bool {
        unsafe { crate::cpu_infra::beam::vertical_irq_enabled(self.game) }
    }
    fn vertical_position(&self) -> u16 {
        unsafe { crate::cpu_infra::beam::vertical_position(self.game) }
    }
    fn vertical_timer(&self) -> u16 {
        unsafe { crate::cpu_infra::beam::vertical_timer(self.game) }
    }
    fn interrupt(&mut self) -> Result<(), Self::Error> {
        let line = self.vertical_timer();
        crate::game::scanline_irq::advance_on_lease(self, line);
        Ok(())
    }
    fn clear_nmi(&mut self) {
        unsafe { crate::cpu_infra::beam::clear_nmi(self.game) }
    }
}
// The channel registers are device registers; the object table is the lease's.
impl sm_game::hdma_objects::Allocator for Runtime<'_> {
    fn write_channel_control(&mut self, register: u16, control: u16) {
        self.owned_player_call(|runtime| {
            // SAFETY: a lease runs with the backend installed, and `&mut Game`
            // excludes other access.
            unsafe { crate::rtl::write_device_word(register, control, runtime.game) }
        });
    }
}
// The HDMA object handler on the lease: the music queue steps on the Session's
// audio owner, and the objects' programs run on this lease.
impl sm_game::hdma_objects::UpdateHost for Runtime<'_> {
    fn advance_music_queue(&mut self) {
        self.advance_music_requests();
    }
    fn blending_room_key(&mut self) -> u16 {
        self.session().room_key()
    }
    fn run_hdma_programs(&mut self, handler_carry: bool) {
        sm_game::hdma_program::run_objects_on(handler_carry, self);
    }
}
// The controllers read through their device registers, as the game reads them.
impl sm_game::menus::controller_input::SampleHost for Runtime<'_> {
    fn sampled_controller_word(&mut self, controller: sm_game::menus::host::Controller) -> u16 {
        use sm_game::menus::host::Controller;
        self.game.read_register_word(match controller {
            Controller::One => crate::rtl::JOY1L,
            Controller::Two => crate::rtl::JOY2L,
        } as u16)
    }
    fn latched_controller_word(&mut self) -> u16 {
        self.game.latched_controller_word()
    }
}
// A lease lends itself, with the observations refreshed as the fresh lease
// the call opened left them.
impl<'g> sm_game::runtime_access::Lend for Runtime<'g> {
    type View<'a>
        = Runtime<'g>
    where
        Self: 'a;
    fn lend<'a, T>(&'a mut self, call: impl FnOnce(&mut Self::View<'a>) -> T) -> T {
        self.owned_player_call(call)
    }
}
// The menus on the lease: the game's tables, its controller registers and its
// message box DMA length, which the lease does not hold; the graphics and the
// HDMA on this lease.
impl sm_game::menus::host::Host for Runtime<'_> {
    fn pause_menu_tables(&self) -> &sm_game::menus::map_menu::ImportedTables {
        &self.game.pause_menu
    }
    fn equipment_screen_tables(&self) -> &sm_game::menus::equipment_menu::ImportedTables {
        &self.game.equipment_screen
    }
    fn demo_sets(&self) -> &Option<sm_game::demo_rooms::ImportedSets> {
        &self.game.demo_sets
    }
    fn demo_scenes(&self) -> &Option<sm_game::demo_rooms::ImportedScenes> {
        &self.game.demo_scenes
    }
    fn intro_palette(&self) -> &Option<sm_game::intro_pages::ImportedIntroPalette> {
        &self.game.intro_palette
    }
    fn read_rom_word(&self, address: u32, offset: usize) -> u16 {
        self.game.read_rom_word(address, offset)
    }
    fn visual_tables(&self) -> &sm_game::visual_tables::ImportedTables {
        &self.game.visual_tables
    }
    fn area_select_palettes(&self) -> &Option<sm_game::menus::menu_tilemap::ImportedAreaPalettes> {
        &self.game.area_select_palettes
    }
}
impl sm_game::menus::host::MenuRuntime for Runtime<'_> {
    fn load_vram_words(&mut self, source: u32, destination: u16, bytes: u16) {
        sm_game::display_refresh::GraphicsAsset::VramWords(source, destination, bytes).load(self)
    }
    fn write_dma_register(&mut self, register: u16, value: u8) {
        crate::rtl::write_dma_register(register, value, self.game)
    }
    fn handle_music_queue(&mut self) {
        self.advance_music_requests();
    }
    fn assert_ram_pointer(&mut self, address: u16) {
        crate::game::menu_host::assert_ram_pointer(address);
    }
    fn message_box_das0l_value(&self) -> u16 {
        self.game.message_box_das0l_value
    }
    fn set_message_box_das0l_value(&mut self, value: u16) {
        self.game.message_box_das0l_value = value;
    }
    fn advance_dialogue_sound_effects(&mut self) {
        self.advance_dialogue_sound_requests();
    }
    fn run_hdma_objects(&mut self) {
        // Closing the box ($85:8589) ends on a borrow, and nothing up to the handler sets C.
        sm_game::hdma_objects::update_hdma(false, self);
    }
    fn spawn_hdma(&mut self, bank: u8, args: &crate::types::HdmaObjectDefinition) -> u16 {
        sm_game::hdma_objects::spawn_hdma_owned(bank, args, self)
    }
    fn rtl_unreachable(&mut self) -> bool {
        crate::rtl::unreachable()
    }
    fn update_beam_tiles_and_palette(&mut self) {
        self.owned_player_call(sm_game::player_drawing::update_beam_tiles_and_palette);
    }
    fn invalid_window(&mut self, address: u32) {
        invalid_window(self.game, address)
    }
    fn legacy_assertion(&self, drawing: bool) -> ! {
        legacy_assertion(drawing)
    }
    fn unreachable_native(&mut self) -> u16 {
        crate::rtl::unreachable()
    }
    #[cfg(sm_verify_dll)]
    fn observe_map_write(&self, byte: usize, index: usize, stage: u8) {
        player_legacy_edges::map_write(self, byte, index, stage)
    }
    #[cfg(sm_verify_dll)]
    fn observe_selected_continue(&mut self, load_result: u8) {
        crate::flags_save_observer::selected_continue(load_result)
    }
}
// The reserve tanks' state on the lease: the Session's inventory, stored as
// the retained Session's inventory store does.
impl sm_game::inventory_domain::InventoryAccess for Runtime<'_> {
    fn reserve_state(&self) -> sm_game::inventory_domain::ReserveState {
        sm_game::inventory_domain::ReserveState::from(&self.session().inventory)
    }
    fn commit_reserve(&mut self, change: sm_game::inventory_domain::ReserveChange) {
        if change.is_empty() {
            return;
        }
        change.apply(&mut self.session_mut().inventory);
        self.refresh_inputs();
    }
}
// The menu window on the lease: the Session's window words, stored as the
// retained Session's window store does.
impl sm_game::menu_window_state::Access for Runtime<'_> {
    fn window_state(&self) -> sm_game::menu_window_state::WindowState {
        sm_game::fx_encoding::session_window(self.session())
    }
    fn publish_window(
        &mut self,
        value: sm_game::menu_window_state::WindowState,
        fields: &[sm_game::menu_window_state::Field],
    ) {
        for &field in fields {
            let at = sm_game::menu_window_encoding::address(field);
            self.write_word_le(field.get(&value), at);
        }
    }
}
// The file copy arrow's palette timer on the lease: the Session's projectile
// table word ($198F), as the retained Session holds it.
impl sm_game::menu_window_state::ArrowAccess for Runtime<'_> {
    fn file_arrow(&mut self, action: sm_game::menu_window_state::ArrowAction) -> bool {
        let mut menu = sm_game::enemy_projectiles::table::FileSelect(
            &mut self.session_mut().enemy_projectiles,
        );
        let change = sm_game::menu_window_state::update_arrow(menu.arrow_palette_timer(), action);
        if change.publications.iter().any(Option::is_some) {
            self.words_stored();
        }
        change.rotate
    }
}
// Sprite output's words on the lease, through its projection.
impl sm_game::sprite_output::SpriteWords for Runtime<'_> {
    fn sprite_word(&self, word: sm_game::sprite_output::SpriteWord) -> u16 {
        self.word(crate::game::menu_host::sprite_word_address(word))
    }
    fn sprite_camera(&self) -> [u16; 2] {
        crate::game::menu_host::SPRITE_CAMERA.map(|at| self.word(at))
    }
    fn projectile_sprite_cursor(&self, slot: u16) -> u16 {
        self.native_word(crate::game::menu_host::projectile_sprite_address(slot))
    }
    fn echo_frame(&self) -> u16 {
        let game = self;
        sm_game::frame_display_word!(game.NmiFrameWordPair)
    }
    fn samus_pose(&self) -> u16 {
        let game = self;
        sm_game::sequence_word!(game.SAMUS_POSE)
    }
}
impl sm_game::sprite_output::SpriteHost for Runtime<'_> {}
// The soft reset's request is the Session's game state word.
impl sm_game::menus::host::ResetRequest for Runtime<'_> {
    fn request_reset(&mut self) {
        sm_game::gameplay_word!(self.GAME_STATE = u16::MAX);
    }
}
// A message box on the lease: its DMA length is the game's, the audio steps on
// the Session's owner, and the HDMA object handler and the movement sound run
// on this lease.
impl sm_game::message_boxes::DialogueHost for Runtime<'_> {
    fn dialogue_dma_length(&self) -> u16 {
        self.game.message_box_das0l_value
    }
    fn set_dialogue_dma_length(&mut self, value: u16) {
        self.game.message_box_das0l_value = value;
    }
    fn advance_dialogue_music(&mut self) {
        self.advance_music_requests();
    }
    fn emit_dialogue_sounds(&mut self) {
        self.advance_dialogue_sound_requests();
    }
    fn update_dialogue_hdma(&mut self) {
        sm_game::hdma_objects::update_hdma(false, self);
    }
    fn resume_movement_sound(&mut self) {
        sm_game::player_projectiles::feedback::restore_sound(self);
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
        // lease as before; otherwise the request goes to the RAM queue.
        if self.has_owned_display() {
            self.owned_call(|runtime| {
                #[cfg(sm_verify_dll)]
                crate::game::visual_runtime::samus_observer::sound(channel, sound, max_queued);
                crate::game::music::requests::queue_sfx(channel, sound, max_queued, runtime)
                    .expect("audio sound enqueue");
            });
            return;
        }
        self.queue_sound_request(channel, sound, max_queued);
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
// Magdollite's imported tables are the game's (`Game::magdollite_glow_colors`).
impl sm_game::actors::magdollite::Host for Runtime<'_> {
    fn magdollite_glow_colors(&self, source: isize) -> [u16; 4] {
        self.game.magdollite_glow_colors(source)
    }
    fn magdollite_arm(&self, address: u32, index: isize) -> u16 {
        self.game.magdollite_arm(address, index)
    }
    fn warn_lava_height(&mut self, value: i32) {
        warn_lava_height(value);
    }
}
// Crocomire's imported tables are the game's; reading them touches no RAM.
impl sm_game::actors::crocomire_reactions::Tables for Runtime<'_> {
    fn crocomire_table_word(&self, address: u32, index: isize) -> u16 {
        crate::game::actors::crocomire_reactions::crocomire_table_word(self.game, address, index)
    }
    fn crocomire_melt_tilemap(&self, table: u32) -> Option<[u16; sm_data::crocomire::MELT_TILES]> {
        crate::game::actors::crocomire_reactions::crocomire_melt_tilemap(self.game, table)
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
/// The explosion windows run on the lease: the Crystal Flash and the pose
/// command are its own player rules, and the window row is the game's.
impl sm_game::explosion_window::Host for Runtime<'_> {
    fn initialize_crystal_flash(&mut self) -> bool {
        sm_game::player_movement::forced::initialize_crystal_flash(self)
    }
    fn pose_command(&mut self, command: u16) {
        player_command_in(self, command);
    }
    fn window_row(&self) -> u16 {
        self.game.k_out
    }
    fn set_window_row(&mut self, row: u16) {
        self.game.k_out = row;
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
        crate::rtl::unreachable()
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
            self.game.grapple_animation()
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
            self.game.inventory_state()
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
            self.game.gameplay_word(word)
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
            self.game.set_gameplay_byte(word, lane, byte);
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
    fn store_buffer_bytes(
        &mut self,
        owner: sm_game::scene::hdma_sources::LiveOwner,
        offset: usize,
        bytes: &[u8],
    ) {
        let at = sm_game::fx_encoding::source_address(owner, offset).expect("FX buffer role");
        for (lane, &byte) in bytes.iter().enumerate() {
            self.set_byte(at + lane, byte);
        }
    }
    fn buffer_byte(&self, owner: sm_game::scene::hdma_sources::LiveOwner, offset: usize) -> u8 {
        let at = sm_game::fx_encoding::source_address(owner, offset).expect("FX buffer role");
        self.byte(at)
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
    fn secondary_continuation(&self) -> u8 {
        *self.session().control.secondary()
    }
    fn set_secondary_continuation(&mut self, value: u8) {
        *self.session_mut().control.secondary_mut() = value;
    }
    fn outer_continuation(&self) -> u8 {
        self.byte(0x611)
    }
    fn set_outer_continuation(&mut self, value: u8) {
        self.set_byte(0x611, value);
    }
    fn message_continuation(&self) -> u8 {
        self.byte(0x615)
    }
    fn clear_pass_continuations(&mut self) {
        self.set_byte(0x615, 0);
        self.set_byte(0x614, 0);
        *self.session_mut().control.secondary_mut() = 0;
        *self.session_mut().control.primary_mut() = 0;
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
    fn used_station_byte(&self, offset: usize) -> u8 {
        self.byte(0xd8f8usize.checked_add(offset).expect("station offset"))
    }
    fn store_used_station_byte(&mut self, offset: usize, value: u8) {
        let at = 0xd8f8usize.checked_add(offset).expect("station offset");
        self.set_byte(at, value);
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
