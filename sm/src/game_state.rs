//! Explicit per-game ownership; RAM bytes retain their exact SNES layout.
use crate::cpu_infra::Snapshot;
use sm_game::room_entry::world::WorldBinding;
use {
    crate::game::scene::records::SceneMedia,
    sm_game::scene::{
        samus::SamusCatalog, samus_content as content, samus_native_content as native_content,
        samus_position_tiles::TileRequest, state::DisplayError,
    },
};
#[path = "game/persistent_session.rs"]
pub mod persistent_session;
use crate::initial_state::InitialState;
use crate::rtl::StateRecorder;
use crate::types::*;
use std::cell::Cell;
use std::collections::BTreeMap;
use std::ffi::c_int;
use std::ops::Range;
use std::rc::Rc;
use std::sync::Arc;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum SamusExecution {
    Prepared,
    RetainedNative,
}
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum SamusAuthority {
    LegacyUnrouted,
    Available,
    Revoked,
}
/// Supplied initialization, independently expected before the selection is published.
pub(crate) struct SamusBindingDescription {
    pub world: Arc<WorldBinding>,
    pub roles: content::ContentDescription,
    pub spritemaps: Arc<SamusCatalog>,
    pub request_media: Arc<SceneMedia>,
    pub execution: SamusExecution,
}
struct SamusBinding {
    world: Arc<WorldBinding>,
    roles: Arc<content::ValidatedContent>,
    expected_roles: Arc<content::ContentDescription>,
    spritemaps: Arc<SamusCatalog>,
    #[cfg(sm_verify_dll)]
    request_media: Arc<SceneMedia>,
    registrations:
        BTreeMap<(content::Registration, content::AllocationId), native_content::RegisteredRom>,
    execution: SamusExecution,
}
/// Unpublished; only Game can construct or install this complete replacement.
pub(crate) struct PreparedSamusBinding(Arc<SamusBinding>);
/// A branch observation, not a second live selector or a copy of mutable ROM bytes.
#[derive(Clone)]
pub(crate) struct SamusSnapshotPayload {
    world: Arc<WorldBinding>,
    binding: Option<Arc<SamusBinding>>,
    loaded: Option<crate::rom::RomImage>,
    native: Option<crate::rom::RomImage>,
    authority: SamusAuthority,
    admitted: bool,
}
pub(crate) struct PreparedSamusAdoption(SamusSnapshotPayload);
/// Exclusive external interval. Drop only invalidates authority and closes the scope.
pub(crate) struct SamusMutation<'g> {
    game: &'g mut Game,
    completed: bool,
}
impl SamusMutation<'_> {
    pub(crate) fn game(&mut self) -> &mut Game {
        self.game
    }
    pub(crate) fn finish_current(self) -> Result<(), DisplayError> {
        let payload = self.game.capture_samus_snapshot();
        let prepared = self.game.prepare_samus_adoption(&payload)?;
        self.finish(prepared)
    }
    pub(crate) fn finish(mut self, prepared: PreparedSamusAdoption) -> Result<(), DisplayError> {
        self.game.commit_samus_adoption(prepared)?;
        self.completed = true;
        Ok(())
    }
    /// A known incomplete legacy load must finish its original callbacks. Its
    /// enclosing load retains revocation and will also skip adoption on this status.
    pub(crate) fn defer_incomplete(mut self) {
        self.completed = self.game.samus_mutation_depth > 1;
    }
}
impl Drop for SamusMutation<'_> {
    fn drop(&mut self) {
        self.game.samus_mutation_depth -= 1;
        if !self.completed {
            self.game.revoke_samus_authority();
        }
    }
}
// Cell preserves the native byte representation while permitting shared byte
// writes. Both owners copy byte values; no live u8 reference or raw base escapes.
#[repr(C, align(16))]
struct RamBytes {
    bytes: [Cell<u8>; WorkRam::LEN],
}
const _: () = {
    assert!(std::mem::size_of::<Cell<u8>>() == 1);
    assert!(std::mem::align_of::<Cell<u8>>() == 1);
    assert!(std::mem::offset_of!(RamBytes, bytes) == 0);
    assert!(std::mem::size_of::<RamBytes>() == 0x20000);
    assert!(std::mem::align_of::<RamBytes>() == 16);
};

/// The compatibility allocation shared only by Game and its retained backend.
/// Safe operations copy values; they never borrow live bytes as u8 references.
pub struct WorkRam(Rc<RamBytes>, Rc<Cell<u64>>);
#[forbid(unsafe_code)]
impl WorkRam {
    pub(crate) const LEN: usize = 0x20000;
    fn new() -> Self {
        Self(
            Rc::new(RamBytes {
                bytes: [const { Cell::new(0) }; Self::LEN],
            }),
            Rc::new(Cell::new(0)),
        )
    }

    pub fn read_byte(&self, at: usize) -> u8 {
        self.0.bytes[at].get()
    }

    pub fn write_byte(&self, at: usize, value: u8) {
        let cell = &self.0.bytes[at];
        self.note_write();
        cell.set(value);
    }

    pub fn len(&self) -> usize {
        self.0.bytes.len()
    }

    pub fn copy_within(&self, source: Range<usize>, destination: usize) {
        // Check the entire source range, then the destination, before any store.
        let count = self.0.bytes[source.clone()].len();
        assert!(destination <= self.len() - count, "dest is out of bounds");
        if count != 0 {
            self.note_write();
        }
        if destination > source.start {
            for offset in (0..count).rev() {
                let byte = self.0.bytes[source.start + offset].get();
                self.0.bytes[destination + offset].set(byte);
            }
        } else {
            for offset in 0..count {
                let byte = self.0.bytes[source.start + offset].get();
                self.0.bytes[destination + offset].set(byte);
            }
        }
    }

    pub fn copy_out(&self, at: usize, out: &mut [u8]) {
        let end = at.checked_add(out.len()).expect("RAM range overflow");
        let cells = &self.0.bytes[at..end];
        for (byte, cell) in out.iter_mut().zip(cells) {
            *byte = cell.get();
        }
    }

    pub fn copy_in(&self, at: usize, input: &[u8]) {
        let end = at.checked_add(input.len()).expect("RAM range overflow");
        let cells = &self.0.bytes[at..end];
        if !cells.is_empty() {
            self.note_write();
        }
        for (cell, &byte) in cells.iter().zip(input) {
            cell.set(byte);
        }
    }

    pub fn fill(&self, range: Range<usize>, value: u8) {
        let cells = &self.0.bytes[range];
        if !cells.is_empty() {
            self.note_write();
        }
        for cell in cells {
            cell.set(value);
        }
    }

    pub(crate) fn share_for_backend(&self) -> Self {
        Self(Rc::clone(&self.0), Rc::clone(&self.1))
    }
    fn note_write(&self) {
        self.1.set(self.1.get().wrapping_add(1));
    }
    fn write_epoch(&self) -> u64 {
        self.1.get()
    }
}

/// Select one existing oracle snapshot or a separately borrowed local snapshot.
pub enum PlayerSnapshotSource<'a> {
    Before,
    Mine,
    Theirs,
    Local(&'a crate::audio_snapshot::PlayerSnapshot),
}

/// State for one game and its differential oracle. The backend retains the same
/// stable RAM allocation independently of this Game's address.
pub struct Game {
    backend: Option<crate::snes::snes::Backend>,
    _native_player: Option<crate::spc_player::NativePlayer>,
    actor_identity: Option<crate::game::actors::identity::ActorIdentityState>,
    pub(crate) actor_snapshots: sm_game::actors::snapshot::ActorSnapshotSidecars,
    world: Arc<WorldBinding>,
    ram: WorkRam,
    pub(crate) loaded_rom: Option<crate::rom::RomImage>,
    native_rom: Option<crate::rom::RomImage>,
    /// Imported from `loaded_rom` by `register_rom`; see game::enemy_speeds.
    pub(crate) enemy_speeds: Option<crate::game::enemy_speeds::ImportedSpeeds>,
    /// Imported from `native_rom` by `register_native_rom`; see game::enemy_speeds.
    pub(crate) enemy_speeds32: Option<crate::game::enemy_speeds::ImportedSpeeds32>,
    /// Imported from `loaded_rom` by `register_rom`; see sm_game::menus::save_slots.
    pub(crate) save_slots: Option<sm_game::menus::save_slots::ImportedSlots>,
    /// Imported from `loaded_rom` by `register_rom`; see sm_game::demo_rooms.
    pub(crate) demo_sets: Option<sm_game::demo_rooms::ImportedSets>,
    /// Imported from `native_rom` by `register_native_rom`; see sm_game::demo_rooms.
    pub(crate) demo_scenes: Option<sm_game::demo_rooms::ImportedScenes>,
    /// Imported from `loaded_rom` by `register_rom`; see game::visual_runtime.
    pub(crate) visual_tables: sm_game::visual_tables::ImportedTables,
    /// Imported from `loaded_rom` by `register_rom`; see sm_game::menus::map_menu.
    pub(crate) pause_menu: sm_game::menus::map_menu::ImportedTables,
    /// Imported from `loaded_rom` by `register_rom`; see game::actors::crocomire_reactions.
    pub(crate) crocomire: crate::game::actors::crocomire_reactions::ImportedTables,
    /// Imported from `loaded_rom` by `register_rom`; see game::actors::magdollite.
    pub(crate) magdollite: crate::game::actors::magdollite::ImportedTables,
    /// Imported from `loaded_rom` by `register_rom`; see sm_game::menus::equipment_menu.
    pub(crate) equipment_screen: sm_game::menus::equipment_menu::ImportedTables,
    /// Imported from `loaded_rom` by `register_rom`; see game::mother_brain_visuals.
    pub(crate) mother_brain_visuals: sm_game::mother_brain_visuals::ImportedTables,
    /// Imported from `loaded_rom` by `register_rom`; see game::beam_windows.
    /// Imported from `loaded_rom` by `register_rom`; see game::actors::escape_critters.
    pub(crate) escape_etecoons: crate::game::actors::escape_critters::ImportedTables,
    /// Imported from `loaded_rom` by `register_rom`; see game::actors::botwoon.
    pub(crate) botwoon_speeds: Option<crate::game::actors::botwoon::ImportedSpeeds>,
    /// Imported from `loaded_rom` by `register_rom`; see game::actors::botwoon.
    pub(crate) botwoon_holes: Option<crate::game::actors::botwoon::ImportedHoles>,
    /// Imported from `loaded_rom` by `register_rom`; see sm_game::intro_pages.
    pub(crate) intro_palette: Option<sm_game::intro_pages::ImportedIntroPalette>,
    /// Imported from `loaded_rom` by `register_rom`; see sm_game::menus::menu_tilemap.
    pub(crate) area_select_palettes: Option<sm_game::menus::menu_tilemap::ImportedAreaPalettes>,
    /// Imported from `loaded_rom` by `register_rom`; see sm_game::records.
    pub(crate) records: sm_game::records::RomRecords,
    samus_binding: Option<Arc<SamusBinding>>,
    samus_authority: SamusAuthority,
    samus_mutation_depth: usize,
    persistent_session: persistent_session::SessionSlot,
    persistent_ram_epoch: u64,
    pending_supplied_start: Option<persistent_session::SuppliedSessionStart>,
    pub decompress_src: u32,
    pub eproj_spawn_rect: Rect16U,
    pub eproj_spawn_r22: u16,
    pub eproj_spawn_pt: Point16U,
    pub eproj_spawn_varE24: u16,
    pub message_box_das0l_value: u16,
    /// Whether a message box is running the HDMA object handler.
    pub message_box_hdma: bool,
    pub k_out: u16,

    pub g_sram: crate::save_ram::SaveRam,
    pub g_rtl_memory_ptr: *mut u8,
    pub g_rtl_runframe: Option<RunFrameFunc>,
    pub g_rtl_syncall: Option<SyncAllFunc>,
    pub state_recorder: StateRecorder,
    pub apu_queue: crate::apu_queue::ApuQueue,
    pub audio_output: crate::audio_output::AudioOutputSink,
    pub g_runmode: u8,
    pub g_snes: *mut Snes,
    pub g_cpu: *mut Cpu,
    pub g_calling_asm_from_c: bool,
    pub g_calling_asm_from_c_ret: i32,
    pub g_fail: bool,
    pub g_use_my_apu_code: bool,
    pub g_snapshot_mine: Box<Snapshot>,
    pub g_snapshot_theirs: Box<Snapshot>,
    pub g_snapshot_before: Box<Snapshot>,
    pub hookmode: u32,
    pub hookcnt: u32,
    pub hookadr: u32,
    pub hook_orgbyte: [u8; 1024],
    pub hook_fixbug_orgbyte: [u8; 1024],
    pub kPatchedCarrysOrg: [u8; 155],
    pub currently_installed_bug_fix_counter: u16,
    pub VERIFY_ACTIVE: bool,
    pub VERIFY_FAILURES: u64,
    pub g_spc_player: *mut SpcPlayer,
    pub g_got_mismatch_count: c_int,
    pub g_other_image: bool,
    pub g_debug_flag: bool,
    pub g_new_ppu: bool,
    pub snes_frame_counter: i32,
    pub pc_hist: [u32; 8],
    pub pc_hist_ctr: u32,
    pub pc_bp: u32,
    pub g_debug_apu_cycles: bool,
}
// Fixed word fields share one little-endian read/write definition. Indexed
// fields and operations on other widths remain explicit methods below.
macro_rules! ram_word_accessors {
    (read; $( $(#[$meta:meta])* $name:ident: $address:literal; )*) => {
        $(
            $(#[$meta])*
            pub fn $name(&self) -> u16 {
                u16::from_le_bytes([self.compatibility_byte($address), self.compatibility_byte($address + 1)])
            }
        )*
    };
    (write; $( $(#[$meta:meta])* $name:ident: $address:literal; )*) => {
        $(
            $(#[$meta])*
            pub fn $name(&mut self, value: u16) {
                self.compatibility_copy_in($address, &value.to_le_bytes());
            }
        )*
    };
}

/// The owned native player, when it is the one the game currently selects.
fn selected_player(
    player: &mut Option<crate::spc_player::NativePlayer>,
    selected: *mut SpcPlayer,
) -> Option<&mut crate::spc_player::NativePlayer> {
    player.as_mut().filter(|player| player.view() == selected)
}

// Oracle snapshots own separate allocations, keeping construction stack usage bounded.
impl Game {
    fn execution_parts_and_recorder(
        &mut self,
    ) -> (
        Option<&mut crate::snes::snes::Backend>,
        crate::execution::HostContext<'_>,
        &mut StateRecorder,
    ) {
        self.note_samus_external_mutation();
        // The host context borrows these fields, each once, beside the backend
        // and recorder.
        macro_rules! split {
            ($($field:ident),+) => {{
                let Self { backend, state_recorder, $($field,)+ .. } = self;
                let host = crate::execution::HostContext { $($field,)+ };
                (backend.as_mut(), host, state_recorder)
            }};
        }
        split!(
            ram,
            loaded_rom,
            native_rom,
            g_fail,
            hookmode,
            hookcnt,
            hookadr,
            hook_fixbug_orgbyte,
            kPatchedCarrysOrg,
            g_calling_asm_from_c,
            g_calling_asm_from_c_ret,
            pc_hist,
            pc_hist_ctr,
            snes_frame_counter,
            apu_queue,
            audio_output,
            g_debug_apu_cycles,
            g_new_ppu,
            decompress_src,
            eproj_spawn_rect,
            eproj_spawn_r22,
            eproj_spawn_pt,
            eproj_spawn_varE24
        )
    }

    pub(crate) fn audio_execution_position(&self) -> crate::audio_output::ExecutionPosition {
        // The optional Backend owns both views for this synchronous observation.
        unsafe {
            crate::rtl::audio_execution_position(self.backend.as_ref(), self.snes_frame_counter)
        }
    }

    pub(crate) fn execution_parts(
        &mut self,
    ) -> (
        Option<&mut crate::snes::snes::Backend>,
        crate::execution::HostContext<'_>,
    ) {
        let (backend, host, _) = self.execution_parts_and_recorder();
        (backend, host)
    }

    pub(crate) unsafe fn execution_and_recorder(
        &mut self,
    ) -> (crate::execution::Execution<'_>, &mut StateRecorder) {
        let (backend, host, recorder) = self.execution_parts_and_recorder();
        (
            crate::execution::Execution {
                backend: backend.expect("execution requires an owned backend"),
                host,
            },
            recorder,
        )
    }

    /// Requires the live, exclusive retained backend; callbacks must not recover
    /// a Game borrow while this execution interval is active.
    pub(crate) unsafe fn execution(&mut self) -> crate::execution::Execution<'_> {
        let (backend, host) = self.execution_parts();
        crate::execution::Execution {
            backend: backend.expect("execution requires an owned backend"),
            host,
        }
    }

    /// Step the retained reference CPU for the external verifier binary.
    /// # Safety
    /// The owned backend must be live and exclusive. Native callbacks must not
    /// recover Game, and no device borrow may survive this operation.
    pub unsafe fn step_reference_opcode(&mut self) -> i32 {
        let selected_cpu = self.g_cpu;
        unsafe {
            let mut execution = self.execution();
            assert_eq!(
                selected_cpu,
                execution.cpu(),
                "reference opcode requires the owned CPU"
            );
            execution.step_cpu()
        }
    }

    pub fn ram(&self) -> &WorkRam {
        &self.ram
    }

    // These borrows expose only detached snapshot/recording storage alongside
    // the opaque live RAM owner. Callers capture raw device views beforehand.
    pub(crate) fn ram_and_recorder(&mut self) -> (&WorkRam, &mut StateRecorder) {
        (&self.ram, &mut self.state_recorder)
    }

    pub(crate) fn ram_and_owned_snapshots(
        &mut self,
    ) -> (
        &crate::snes::snes::Backend,
        &WorkRam,
        &mut Snapshot,
        &mut Snapshot,
        &mut Snapshot,
        Option<&mut crate::spc_player::NativePlayer>,
    ) {
        (
            self.backend
                .as_ref()
                .expect("snapshot requires an owned backend"),
            &self.ram,
            &mut self.g_snapshot_mine,
            &mut self.g_snapshot_theirs,
            &mut self.g_snapshot_before,
            self._native_player.as_mut(),
        )
    }

    /// Create absent retained storage; an existing backend and its views stay owned.
    pub(crate) fn ensure_backend(&mut self) {
        if self.backend.is_none() {
            let backend = crate::snes::snes::Backend::new(self.ram.share_for_backend());
            (self.g_snes, self.g_cpu) = backend.views();
            self.backend = Some(backend);
        }
    }

    /// Borrow the first retained input port only until the next callback.
    pub fn input1_mut(&mut self) -> &mut Input {
        self.backend
            .as_mut()
            .expect("input access requires an owned backend")
            .input1_mut()
    }

    /// Borrow the second retained input port only until the next callback.
    pub fn input2_mut(&mut self) -> &mut Input {
        self.backend
            .as_mut()
            .expect("input access requires an owned backend")
            .input2_mut()
    }

    /// Copy channels from the owned DMA storage, without following routing links.
    pub(crate) fn dma_channels(&self) -> [DmaChannel; 8] {
        self.backend
            .as_ref()
            .expect("DMA observation requires an owned backend")
            .dma_channels()
    }

    /// Copy inert DMA values from the retained owner; no device link escapes.
    pub fn dma_state(&self) -> Dma {
        self.backend
            .as_ref()
            .expect("DMA access requires an owned backend")
            .dma_state()
    }

    /// Adopt a detached snapshot at a quiescent save/load phase.
    pub fn set_dma_state(&mut self, state: Dma) {
        self.backend
            .as_ref()
            .expect("DMA access requires an owned backend")
            .set_dma_state(state);
    }

    /// Apply one captured register byte before the next seed field is validated.
    pub fn write_dma_register(&mut self, address: u16, value: u8) {
        self.backend
            .as_ref()
            .expect("DMA access requires an owned backend")
            .write_dma_register(address, value);
    }

    pub(crate) fn dma_busy(&self) -> bool {
        self.backend
            .as_ref()
            .expect("DMA access requires an owned backend")
            .dma_busy()
    }

    /// Transfer retained storage after its users have stopped.
    /// # Safety
    /// All device/native callbacks, references and raw borrowers must be quiescent.
    /// The caller must retain the returned owner until its shutdown drop point.
    pub(crate) unsafe fn take_backend_for_shutdown(
        &mut self,
    ) -> Option<crate::snes::snes::Backend> {
        self.backend.take()
    }

    /// Replace the complete backend, including failed-initialization storage.
    /// # Safety
    /// No device reference or raw consumer may survive replacement. Keep this
    /// retained RAM handle alive until all backend execution has stopped.
    pub(crate) unsafe fn initialize_backend(&mut self) {
        let backend = crate::snes::snes::Backend::new(self.ram.share_for_backend());
        (self.g_snes, self.g_cpu) = backend.views();
        self.backend = Some(backend);
    }

    /// Allocate the complete replacement before publishing its borrowed view.
    /// Existing raw views become invalid. Code dereferencing native/audio views
    /// must keep their owner live and stop before replacement or Game destruction.
    /// Foreign DSP/history links stay borrowed; creation never dereferences them.
    #[forbid(unsafe_code)]
    pub fn create_native_player(&mut self) {
        let player = crate::spc_player::NativePlayer::new();
        self.g_spc_player = player.view();
        self._native_player = Some(player);
    }

    /// Initialize the current player at the original caller's lifecycle phase.
    /// # Safety
    /// Native/audio users must be exclusive, stopped for reset and non-reentrant;
    /// imported DSP/history allocations retain their original disjoint lifetimes.
    pub unsafe fn initialize_native_player(&mut self) {
        unsafe {
            if let Some(player) = selected_player(&mut self._native_player, self.g_spc_player) {
                if player.try_initialize() {
                    return;
                }
            }
            crate::spc_player::spc_player_initialize(self.g_spc_player);
        }
    }

    /// Export current fields and optional queued ports into this player's RAM.
    /// # Safety
    /// Hold the existing APU lock with exclusive non-reentrant native views.
    pub unsafe fn save_native_audio_variables(&mut self, include_queued_commands: bool) {
        unsafe {
            if let Some(player) = selected_player(&mut self._native_player, self.g_spc_player) {
                player.save_state(include_queued_commands.then_some(&self.apu_queue));
            } else {
                if include_queued_commands {
                    self.apu_queue
                        .apply_queued(&mut (*self.g_spc_player).state.input_ports);
                }
                crate::spc_player::spc_player_copy_variables_to_ram(self.g_spc_player);
            }
        }
    }

    /// Import the actual current RAM image before resuming sequencing.
    /// # Safety
    /// Hold the existing APU lock with exclusive non-reentrant native views.
    pub unsafe fn load_native_audio_variables(&mut self) {
        unsafe {
            if let Some(player) = selected_player(&mut self._native_player, self.g_spc_player) {
                player.load_state();
            } else {
                crate::spc_player::spc_player_copy_variables_from_ram(self.g_spc_player);
            }
        }
    }

    /// Capture the actual selected player into separate snapshot storage.
    /// # Safety
    /// Snapshot and native allocations must be disjoint, stopped and exclusive;
    /// retained foreign DSP/history views stay live and non-reentrant.
    pub unsafe fn capture_native_player_snapshot(
        &mut self,
        snapshot: &mut crate::audio_snapshot::PlayerSnapshot,
    ) {
        unsafe {
            snapshot.capture_with_owner(self._native_player.as_mut(), self.g_spc_player);
        }
    }
    /// Restore the selected completed image; allocation links stay destination-owned.
    /// # Safety
    /// Keep the same native/snapshot disjointness, history topology and quiescence.
    pub unsafe fn restore_native_player_snapshot(&mut self, source: PlayerSnapshotSource<'_>) {
        unsafe {
            let snapshot = match source {
                PlayerSnapshotSource::Before => &self.g_snapshot_before.player,
                PlayerSnapshotSource::Mine => &self.g_snapshot_mine.player,
                PlayerSnapshotSource::Theirs => &self.g_snapshot_theirs.player,
                PlayerSnapshotSource::Local(snapshot) => snapshot,
            };
            snapshot.restore_with_owner(self._native_player.as_mut(), self.g_spc_player);
        }
    }

    /// Call under the existing APU lock with exclusive native/audio users.
    pub(crate) unsafe fn apply_native_audio_ports(&mut self, writes: [u8; 4]) {
        unsafe {
            if let Some(player) = selected_player(&mut self._native_player, self.g_spc_player) {
                player.apply_ports(writes);
            } else {
                crate::apu_queue::ApuQueue::apply_ports(
                    writes,
                    &mut (*self.g_spc_player).state.input_ports,
                );
            }
        }
    }

    /// The APU lock, stopped replacement/Drop and disjoint non-reentrant native
    /// views/history remain required; selected external players retain the bridge.
    pub(crate) unsafe fn render_native_audio(&mut self, samples: &mut [[i16; 2]]) {
        unsafe {
            if let Some(player) = selected_player(&mut self._native_player, self.g_spc_player) {
                if player.try_render_samples(samples) {
                    return;
                }
            }
            // The owned borrow ended before selected/history/imported playback.
            crate::spc_player::spc_player_generate_samples(self.g_spc_player);
            (*(*self.g_spc_player).state.dsp)
                .state
                .write_samples(samples);
        }
    }

    #[cfg(sm_verify_dll)]
    pub(crate) fn save_backend_observation(&self, output: &mut crate::recording::RecordingBytes) {
        self.backend
            .as_ref()
            .expect("observation requires an owned backend")
            .save_observation(self.g_snes, self.g_cpu, output);
    }

    pub(crate) fn next_random(&mut self) -> u16 {
        let seed = u16::from_le_bytes([
            self.compatibility_byte(0x5e5),
            self.compatibility_byte(0x5e6),
        ]);
        let result = sm_game::random::next(seed);
        crate::ram_map::checked_word!(self.RANDOM_NUMBER = result);
        #[cfg(sm_verify_dll)]
        crate::cpu_infra::player_transitions_composed::random_after(self);
        result
    }

    pub(crate) fn multiply_with_scratch(&mut self, a: u16, b: u16) -> u32 {
        self.compatibility_copy_in(0x5e9, &a.to_le_bytes());
        self.compatibility_copy_in(0x5eb, &b.to_le_bytes());
        let result: u32 = (a as u32).wrapping_mul(b as u32);
        result
    }

    pub(crate) fn request_reset(&mut self) {
        crate::ram_map::word!(self.GAME_STATE = u16::MAX);
    }

    pub(crate) fn world(&self) -> &Arc<WorldBinding> {
        &self.world
    }

    /// Legacy files have no world identity and support only the canonical world.
    pub fn supports_legacy_persistence(&self) -> bool {
        self.world.is_canonical_legacy()
    }

    /// APU upload flag at $7E:0617.
    pub fn uploading_apu(&self) -> bool {
        self.compatibility_byte(0x617) != 0 || self.compatibility_byte(0x618) != 0
    }

    ram_word_accessors! {
        read;
        /// Health and reserve fields retain their original byte-backed RAM layout.
        samus_health: 0x9c2;
        /// Low-health sound latch ($7E:0A6A in ram_map).
        samus_health_warning: 0xa6a;
        /// random_number ($7E:05E5 in ram_map).
        random_number: 0x5e5;
        /// timer_status ($7E:0943 in ram_map).
        timer_status: 0x943;
        layer1_y_pos: 0x915;
        /// elevator_door_properties_orientation ($7E:0793 in ram_map).
        elevator_door_properties_orientation: 0x793;
        /// elevator_direction ($7E:0799 in ram_map).
        elevator_direction: 0x799;
        /// elevator_flags ($7E:0E16 in ram_map).
        elevator_flags: 0xe16;
        /// room_ptr ($7E:079B in ram_map).
        room_ptr: 0x79b;
        /// samus_x_pos ($7E:0AF6 in ram_map).
        samus_x_pos: 0xaf6;
        /// samus_y_pos ($7E:0AFA in ram_map).
        samus_y_pos: 0xafa;
        /// samus_y_subpos ($7E:0AFC in ram_map).
        samus_y_subpos: 0xafc;
        /// samus_pose ($7E:0A1C in ram_map).
        samus_pose: 0xa1c;
        /// samus_anim_frame ($7E:0A96 in ram_map).
        samus_anim_frame: 0xa96;
        debug_invincibility: 0xde0;
        /// game_state ($7E:0998 in ram_map).
        game_state: 0x998;
        pausemenu_button_label_mode: 0x753;
        /// area_index ($7E:079F in ram_map).
        area_index: 0x79f;
        collected_items: 0x9a4;
        /// demo_set ($7E:1F55 in ram_map).
        demo_set: 0x1f55;
        /// demo_scene ($7E:1F57 in ram_map).
        demo_scene: 0x1f57;
        /// collision_detection_index ($7E:18A6 in ram_map).
        collision_detection_index: 0x18a6;
        /// Current collision block index ($7E:0DC4 in ram_map).
        cur_block_index: 0xdc4;
        /// VRAM write queue byte cursor ($7E:0330 in ram_map).
        vram_write_queue_tail: 0x330;
        /// Room width in blocks ($7E:07A5 in ram_map).
        room_width_in_blocks: 0x7a5;
        /// Current enemy slot offset ($7E:0E54 in ram_map).
        cur_enemy_index: 0xe54;
    }

    ram_word_accessors! {
        write;
        set_samus_health_warning: 0xa6a;
        /// has_area_map ($7E:0789 in ram_map).
        set_has_area_map: 0x789;
        /// frame_handler_beta ($7E:0A44 in ram_map).
        set_frame_handler_beta: 0xa44;
        set_vram_write_queue_tail: 0x330;
    }

    /// Bind the native source to its sole owner. Loaded-ROM reads retain their
    /// separate registration and may deliberately select another allocation.
    #[forbid(unsafe_code)]
    pub(crate) fn register_native_rom(&mut self, image: crate::rom::RomImage) {
        if !self
            .native_rom
            .as_ref()
            .is_some_and(|old| old.same_allocation(&image))
        {
            self.revoke_samus_authority();
        }
        self.import_native_records(&image);
        self.native_rom = Some(image);
    }

    /// Capture the exact native allocation; never substitute the loaded image.
    #[forbid(unsafe_code)]
    pub(crate) fn capture_native_rom(&self) -> crate::rom::RomImage {
        self.native_rom
            .as_ref()
            .expect("native ROM must be registered")
            .clone()
    }

    /// Temporary pointer for unmigrated consumers; they retain their explicit
    /// unsafe read operations. Missing-source behavior remains a null pointer.
    #[forbid(unsafe_code)]
    pub(crate) fn legacy_native_rom_ptr(&self) -> *const u8 {
        self.native_rom.as_ref().map_or(core::ptr::null(), |image| {
            crate::native_rom::NativeRomPointer::as_mut_ptr(image).cast_const()
        })
    }

    /// Preserve an optional binding for comparisons, including startup.
    #[forbid(unsafe_code)]
    pub(crate) fn capture_native_rom_binding(&self) -> Option<crate::rom::RomImage> {
        self.native_rom.clone()
    }

    #[forbid(unsafe_code)]
    pub(crate) fn restore_native_rom_binding(&mut self, image: Option<crate::rom::RomImage>) {
        if !matches!((&self.native_rom, &image), (Some(old), Some(new)) if old.same_allocation(new))
        {
            self.revoke_samus_authority();
        }
        self.native_rom = image;
    }

    /// Share the cartridge image without copying or borrowing its allocation.
    /// Import the `sm_data` records read from the native image.
    fn import_native_records(&mut self, image: &crate::rom::RomImage) {
        self.enemy_speeds32 = crate::game::enemy_speeds::import32(image);
        self.demo_scenes = sm_game::demo_rooms::import_scenes(image);
    }

    /// Import the `sm_data` records read from the loaded image.
    fn import_loaded_records(&mut self, image: &crate::rom::RomImage) {
        self.enemy_speeds = crate::game::enemy_speeds::import(image);
        self.records.sand_heap = sm_game::sand_heap::import(image);
        self.save_slots = sm_game::menus::save_slots::import(image);
        self.records.ceres_rotation = sm_game::ceres_elevator::import(image);
        self.demo_sets = sm_game::demo_rooms::import_sets(image);
        self.records.shooting_stars = sm_game::shooting_stars::import(image);
        self.records.grapple_origins = sm_game::grapple_origins::import(image);
        self.records.grapple_aim = sm_game::grapple_aim::import(image);
        self.records.kraid_tables = sm_game::kraid_tables::import(image);
        self.records.shaktool_tables = sm_game::shaktool_tables::import(image);
        self.records.statue_tables = sm_game::statue_tables::import(image);
        self.records.torizo_tables = sm_game::torizo_tables::import(image);
        self.records.corpse_tables = sm_game::corpse_tables::import(image);
        self.records.torizo_corpse_tables = sm_game::torizo_corpse_tables::import(image);
        self.records.baby_metroid_tables = sm_game::baby_metroid_tables::import(image);
        self.records.mother_brain_tables = sm_game::mother_brain_tables::import(image);
        self.visual_tables = sm_game::visual_tables::import(image);
        self.pause_menu = sm_game::menus::map_menu::import(image);
        self.crocomire = crate::game::actors::crocomire_reactions::import(image);
        self.magdollite = crate::game::actors::magdollite::import(image);
        self.equipment_screen = sm_game::menus::equipment_menu::import(image);
        self.mother_brain_visuals = sm_game::mother_brain_visuals::import(image);
        self.escape_etecoons = crate::game::actors::escape_critters::import(image);
        self.records.flame_rain = sm_game::phantoon_tables::import(image);
        self.records.phantoon_timers = sm_game::phantoon_tables::import_timers(image);
        self.botwoon_speeds = crate::game::actors::botwoon::import_speeds(image);
        self.botwoon_holes = crate::game::actors::botwoon::import_holes(image);
        self.intro_palette = sm_game::intro_pages::import_intro_palette(image);
        self.area_select_palettes = sm_game::menus::menu_tilemap::import_area_palettes(image);
        self.records.ceres_ridley_tables = sm_game::ceres_ridley_tables::import(image);
        self.records.ridley_tables = sm_game::ridley_data::import(image);
        self.records.ridley_explosion_tables = sm_game::ridley_explosion_data::import(image);
        self.records.ceres_door_tables = sm_game::ceres_door_data::import(image);
        self.records.zebetite_tables = sm_game::zebetite_data::import(image);
        self.records.menu_lists = sm_game::menu_lists::import(image);
        self.records.boss_lists = sm_game::boss_lists::import(image);
        self.records.samus_palettes = sm_game::samus_palette::import(image);
        self.records.gameplay_tables = sm_game::gameplay_tables::import(image);
        self.records.enemy_lists = sm_game::enemy_lists::import(image);
    }

    /// Import every record again after the images were patched in place;
    /// until then the patched images serve their original reads.
    pub(crate) fn refresh_rom_records(&mut self) {
        if let Some(image) = self.native_rom.clone() {
            self.import_native_records(&image);
        }
        if let Some(image) = self.loaded_rom.clone() {
            self.import_loaded_records(&image);
        }
    }

    pub(crate) fn register_rom(&mut self, image: crate::rom::RomImage) {
        if !self
            .loaded_rom
            .as_ref()
            .is_some_and(|old| old.same_allocation(&image))
        {
            self.revoke_samus_authority();
        }
        self.import_loaded_records(&image);
        self.loaded_rom = Some(image);
    }

    /// Read a byte at a LoROM address plus a linear offset in the allocation.
    pub fn read_rom_byte(&self, address: u32, byte_offset: usize) -> u8 {
        let start = self.rom_offset(address, byte_offset, 1);
        let rom = self.loaded_rom.as_ref().expect("ROM must be loaded");
        rom.read(start)
    }

    /// Read a little-endian word at a LoROM address plus a linear byte offset.
    /// Adding the offset after mapping preserves table reads across bank edges.
    pub fn read_rom_word(&self, address: u32, byte_offset: usize) -> u16 {
        let start = self.rom_offset(address, byte_offset, 2);
        let rom = self.loaded_rom.as_ref().expect("ROM must be loaded");
        u16::from_le_bytes([rom.read(start), rom.read(start + 1)])
    }

    fn rom_offset(&self, address: u32, byte_offset: usize, width: usize) -> usize {
        assert!(
            address & 0x8000 != 0,
            "ROM address must select a LoROM window"
        );
        let rom = self.loaded_rom.as_ref().expect("ROM must be loaded");
        let base = crate::rtl::lorom_offset(address) as usize;
        let start = base.checked_add(byte_offset).expect("ROM offset overflow");
        let end = start.checked_add(width).expect("ROM offset overflow");
        assert!(end <= rom.len(), "ROM read outside loaded allocation");
        start
    }

    /// Metadata only: retain the original window/missing/width panic order before capture.
    pub(crate) fn preflight_loaded_rom(
        &self,
        address: u32,
        byte_offset: usize,
        width: usize,
    ) -> (crate::rom::RomImage, usize) {
        let start = self.rom_offset(address, byte_offset, width);
        (
            self.loaded_rom
                .as_ref()
                .expect("ROM must be loaded")
                .clone(),
            start,
        )
    }

    /// sfx_readpos / sfx_writepos ($7E:0643/0646 in ram_map).
    pub fn sfx_queue_is_empty(&self, channel: usize) -> bool {
        self.compatibility_byte(0x646 + channel)
            .wrapping_sub(self.compatibility_byte(0x643 + channel))
            & 0xf
            == 0
    }

    /// coroutine_state_1 ($7E:0612 in ram_map).
    pub fn coroutine_state_1(&self) -> u8 {
        self.compatibility_byte(0x612)
    }
    pub fn set_coroutine_state_1(&mut self, value: u8) {
        self.compatibility_write_owned_byte(0x612, value);
    }

    /// Compressed tileset graphics address ($7E:07C3 in ram_map).
    pub fn tileset_tiles_pointer(&self) -> u32 {
        u32::from_le_bytes([
            self.compatibility_byte(0x7c3),
            self.compatibility_byte(0x7c4),
            self.compatibility_byte(0x7c5),
            0,
        ])
    }

    /// Compressed tileset palette address ($7E:07C6 in ram_map).
    pub fn tileset_compr_palette_ptr(&self) -> u32 {
        u32::from_le_bytes([
            self.compatibility_byte(0x7c6),
            self.compatibility_byte(0x7c7),
            self.compatibility_byte(0x7c8),
            0,
        ])
    }

    /// samus_movement_type ($7E:0A1F in ram_map).
    pub fn samus_movement_type(&self) -> u8 {
        self.compatibility_byte(0xa1f)
    }

    /// Write a word in the backing VRAM queue, including overlapping fields.
    pub fn write_vram_queue_word(&mut self, cursor: u16, field: usize, value: u16) {
        let offset = 0xd0 + usize::from(cursor) + field;
        self.compatibility_copy_in(offset, &value.to_le_bytes());
    }

    /// Preserve the original zero initialization, including RM_BOTH (0).
    #[forbid(unsafe_code)]
    pub fn new() -> Box<Self> {
        Self::with_world(WorldBinding::canonical_legacy())
    }

    /// Construct the actual game with a validated immutable world binding.
    #[forbid(unsafe_code)]
    pub fn with_world(world: Arc<WorldBinding>) -> Box<Self> {
        let run_mode = if world.is_canonical_legacy() {
            crate::cpu_infra::RM_BOTH
        } else {
            crate::cpu_infra::RM_MINE
        };
        Box::new(Self {
            backend: None,
            _native_player: None,
            world,
            actor_identity: Some(crate::game::actors::identity::ActorIdentityState::empty()),
            actor_snapshots: Default::default(),
            ram: WorkRam::new(),
            loaded_rom: None,
            native_rom: None,
            enemy_speeds: None,
            enemy_speeds32: None,
            save_slots: None,
            demo_sets: None,
            demo_scenes: None,
            botwoon_speeds: None,
            botwoon_holes: None,
            intro_palette: None,
            area_select_palettes: None,
            visual_tables: Default::default(),
            pause_menu: Default::default(),
            crocomire: Default::default(),
            magdollite: Default::default(),
            equipment_screen: Default::default(),
            mother_brain_visuals: Default::default(),
            escape_etecoons: Default::default(),
            records: Default::default(),
            samus_binding: None,
            samus_authority: SamusAuthority::LegacyUnrouted,
            samus_mutation_depth: 0,
            persistent_session: persistent_session::SessionSlot::Legacy,
            persistent_ram_epoch: 0,
            pending_supplied_start: None,
            decompress_src: 0,
            eproj_spawn_rect: Rect16U {
                x: 0,
                y: 0,
                w: 0,
                h: 0,
            },
            eproj_spawn_r22: 0,
            eproj_spawn_pt: Point16U { x: 0, y: 0 },
            eproj_spawn_varE24: 0,
            message_box_das0l_value: 0,
            message_box_hdma: false,
            k_out: 0,
            g_sram: Default::default(),
            g_rtl_memory_ptr: core::ptr::null_mut(),
            g_rtl_runframe: None,
            g_rtl_syncall: None,
            state_recorder: StateRecorder::default(),
            apu_queue: Default::default(),
            audio_output: Default::default(),
            g_runmode: run_mode as u8,
            g_snes: core::ptr::null_mut(),
            g_cpu: core::ptr::null_mut(),
            g_calling_asm_from_c: false,
            g_calling_asm_from_c_ret: 0,
            g_fail: false,
            g_use_my_apu_code: true,
            g_snapshot_mine: Box::new(Snapshot::INITIAL),
            g_snapshot_theirs: Box::new(Snapshot::INITIAL),
            g_snapshot_before: Box::new(Snapshot::INITIAL),
            hookmode: 0,
            hookcnt: 0,
            hookadr: 0,
            hook_orgbyte: [0; 1024],
            hook_fixbug_orgbyte: [0; 1024],
            kPatchedCarrysOrg: [0; 155],
            currently_installed_bug_fix_counter: 0,
            VERIFY_ACTIVE: false,
            VERIFY_FAILURES: 0,
            g_spc_player: core::ptr::null_mut(),
            g_got_mismatch_count: 0,
            g_other_image: false,
            g_debug_flag: false,
            g_new_ppu: true,
            snes_frame_counter: 0,
            pc_hist: [0; 8],
            pc_hist_ctr: 0,
            pc_bp: 0,
            g_debug_apu_cycles: false,
        })
    }
}
// Mutex ownership is explicit and supplied through SDL userdata. No global
// Game exists; callback and frame acquire exclusivity before creating &mut.
pub struct GameOwner {
    gate: std::sync::Mutex<()>,
    game: std::cell::UnsafeCell<Box<Game>>,
}
// Shared Rc-backed state can escape with(), so this owner cannot implement Sync.
impl GameOwner {
    pub fn new() -> Box<Self> {
        Self::with_world(WorldBinding::canonical_legacy())
    }
    pub fn with_world(world: Arc<WorldBinding>) -> Box<Self> {
        Box::new(Self {
            gate: std::sync::Mutex::new(()),
            game: std::cell::UnsafeCell::new(Game::with_world(world)),
        })
    }
    /// Complete supplied startup/reset. Every replacement is validated before
    /// publishing any new Session, Display, input, audio or content owner.
    pub(crate) fn start_session(
        &self,
        supplied: persistent_session::SessionStartDescription,
        media: Arc<SceneMedia>,
        required: &persistent_session::SessionStartRequirements,
    ) -> Result<(), persistent_session::SessionStartError> {
        self.with(|game| {
            let prepared = persistent_session::prepare(game, supplied, media, required)?;
            game.install_session_start(prepared)
        })
    }
    pub fn start_supplied_session(
        &self,
        supplied: persistent_session::SuppliedSessionStart,
    ) -> Result<(), persistent_session::SessionStartError> {
        let (start, media, required) = supplied.into_parts();
        self.start_session(start, media, &required)
    }
    pub fn checkpoint_session(
        &self,
    ) -> Result<persistent_session::SessionCheckpoint, DisplayError> {
        self.with(|game| game.checkpoint_session())
    }
    pub fn encode_session_checkpoint(&self) -> Result<Vec<u8>, String> {
        self.encode_session_checkpoint_with_sources(&[])
    }
    pub(crate) fn encode_session_checkpoint_with_sources(
        &self,
        sources: &[(sm_game::room_entry::world::ResourceId, crate::rom::RomImage)],
    ) -> Result<Vec<u8>, String> {
        self.with(|game| {
            game.checkpoint_session()
                .map_err(|error| format!("Session capture {error:?}"))?
                .encode_binary(sm_game::actors::snapshot::CheckpointImages {
                    loaded: game.loaded_rom.as_ref(),
                    native: game.capture_native_rom_binding().as_ref(),
                    sources: sources,
                    policy_dependent: &crate::rom_patches::policy_dependent,
                })
        })
    }
    /// Decode and install under the same gate, keeping epoch preflight and
    /// complete replacement validation ahead of the first live-state mutation.
    pub fn restore_session_checkpoint(
        &self,
        bytes: &[u8],
        media: Arc<SceneMedia>,
        contract: &persistent_session::SessionStartContract,
    ) -> Result<(), String> {
        self.restore_session_checkpoint_with_sources(bytes, media, contract, &[])
    }
    pub(crate) fn restore_session_checkpoint_with_sources(
        &self,
        bytes: &[u8],
        media: Arc<SceneMedia>,
        contract: &persistent_session::SessionStartContract,
        sources: &[(sm_game::room_entry::world::ResourceId, crate::rom::RomImage)],
    ) -> Result<(), String> {
        self.with(|game| {
            let current_identity = match &game.persistent_session {
                persistent_session::SessionSlot::Ready(state)
                | persistent_session::SessionSlot::Failed(Some(state)) => {
                    &state.gameplay.actor_identity
                }
                persistent_session::SessionSlot::Legacy => game
                    .actor_identity
                    .as_ref()
                    .ok_or("Session restore actor identity unavailable")?,
                persistent_session::SessionSlot::Leased
                | persistent_session::SessionSlot::Failed(None) => {
                    return Err("Session restore ownership unavailable".into());
                }
            };
            let supplied = persistent_session::SessionCheckpoint::decode_binary(
                bytes,
                current_identity,
                game.world().clone(),
                media,
                contract.requirements().clone(),
                sm_game::actors::snapshot::CheckpointImages {
                    loaded: game.loaded_rom.as_ref(),
                    native: game.capture_native_rom_binding().as_ref(),
                    sources: sources,
                    policy_dependent: &crate::rom_patches::policy_dependent,
                },
            )?;
            let (description, media, required) = supplied.into_parts();
            let prepared = persistent_session::prepare(game, description, media, &required)
                .map_err(|error| format!("Session restore validation {error:?}"))?;
            game.install_session_start(prepared)
                .map_err(|error| format!("Session restore installation {error:?}"))
        })
    }
    pub fn reopen_supplied_session(
        &self,
        checkpoint: persistent_session::SessionCheckpoint,
        contract: &persistent_session::SessionStartContract,
    ) -> Result<(), persistent_session::SessionStartError> {
        self.reopen_session(checkpoint, contract.requirements())
    }
    pub(crate) fn reopen_session(
        &self,
        checkpoint: persistent_session::SessionCheckpoint,
        required: &persistent_session::SessionStartRequirements,
    ) -> Result<(), persistent_session::SessionStartError> {
        let (start, media) = checkpoint
            .into_start()
            .map_err(persistent_session::SessionStartError::Binding)?;
        self.start_session(start, media, required)
    }
    pub fn with<R>(&self, operation: impl FnOnce(&mut Game) -> R) -> R {
        let _guard = self.gate.lock().unwrap();
        operation(unsafe { &mut **self.game.get() })
    }
}

impl persistent_session::AudioHost for Game {
    fn audio_byte(&self, at: usize) -> u8 {
        self.compatibility_byte(at)
    }
    fn audio_store_byte(&mut self, at: usize, byte: u8) {
        self.compatibility_write_owned_byte(at, byte);
    }
    fn audio_mutation<R>(
        &mut self,
        operation: impl FnOnce(
            &mut persistent_session::AudioRequests,
            &mut sm_game::scene::start::InitialAudioIntent,
            persistent_session::AudioStamp,
        ) -> Result<R, DisplayError>,
    ) -> Result<Option<R>, DisplayError> {
        use persistent_session::{Control, SessionSlot};
        if matches!(self.persistent_session, SessionSlot::Legacy) {
            return Ok(None);
        }
        if self.persistent_ram_epoch != self.ram.write_epoch() {
            self.poison_persistent_session();
            return Err(DisplayError::Poisoned);
        }
        self.preflight_samus_runtime()?;
        let SessionSlot::Ready(state) = &mut self.persistent_session else {
            return Err(DisplayError::Phase);
        };
        let producing = state.gameplay.presentation.owner()?.state.phase
            == sm_game::scene::state::DisplayPhase::Producing;
        let Control::Owned(control) = &mut state.gameplay.control else {
            return Err(DisplayError::Phase);
        };
        let result = control.audio_mutation(producing, operation);
        if result.is_err() {
            state.poison();
        }
        result.map(Some)
    }
}
impl Game {
    /// Named callers can observe the current projection. Native byte writes
    /// suspend its authority; mutable owned writers update that same owner.
    pub(crate) fn compatibility_write_epoch(&self) -> u64 {
        self.ram.write_epoch()
    }
    /// The HDMA object table as `compatibility_byte` projects it: the
    /// retained Session's while it is current, the RAM words otherwise.
    pub(crate) fn projected_hdma_objects(&self) -> sm_game::hdma::HdmaObjects {
        if let persistent_session::SessionSlot::Ready(state) = &self.persistent_session {
            if self.persistent_ram_epoch == self.ram.write_epoch()
                && self.preflight_samus_runtime().is_ok()
            {
                return state.gameplay.hdma;
            }
        }
        sm_game::hdma_objects::read_table(self)
    }
    pub fn compatibility_byte(&self, at: usize) -> u8 {
        if let persistent_session::SessionSlot::Ready(state) = &self.persistent_session {
            if self.persistent_ram_epoch == self.ram.write_epoch()
                && self.preflight_samus_runtime().is_ok()
            {
                if let Some(byte) =
                    crate::game::session_encoding::projected_byte(&state.gameplay, at)
                {
                    return byte;
                }
            }
        }
        // A suspended or dirty owner cannot hide completed native writes.
        // Reading their bytes does not decode or readmit semantic authority.
        self.ram.read_byte(at)
    }
    /// Native macro writes preserve held RAM aliases. Their changed epoch keeps
    /// the retained semantic owner suspended until complete scoped adoption.
    pub fn compatibility_write_byte(&self, at: usize, byte: u8) {
        self.ram.write_byte(at, byte);
    }
    pub fn compatibility_write_owned_byte(&mut self, at: usize, byte: u8) {
        self.ram.read_byte(at); // Preserve the original byte bounds failure before mutation.
        if matches!(
            self.persistent_session,
            persistent_session::SessionSlot::Ready(_)
        ) && (self.persistent_ram_epoch != self.ram.write_epoch()
            || self.preflight_samus_runtime().is_err()
            || persistent_session::Presentation::requires_semantic_counterpart(at))
        {
            self.poison_persistent_session();
        }
        if let persistent_session::SessionSlot::Ready(state) = &mut self.persistent_session {
            crate::game::session_encoding::write_projected_byte(&mut state.gameplay, at, byte);
            self.ram.write_byte(at, byte);
            state.observations = crate::game::session_encoding::inputs(&|lane| {
                crate::game::session_encoding::projected_byte(&state.gameplay, lane)
                    .unwrap_or_else(|| self.ram.read_byte(lane))
            });
            self.persistent_ram_epoch = self.ram.write_epoch();
        } else {
            self.ram.write_byte(at, byte);
        }
    }
    /// Slice operations retain complete source/destination preflight before stores.
    pub fn compatibility_copy_in(&mut self, at: usize, bytes: &[u8]) {
        let end = at.checked_add(bytes.len()).expect("RAM range overflow");
        let _ = &self.ram.0.bytes[at..end];
        for (offset, &byte) in bytes.iter().enumerate() {
            self.compatibility_write_owned_byte(at + offset, byte);
        }
    }
    pub fn start_pending_supplied_session(
        &mut self,
    ) -> Option<Result<(), persistent_session::SessionStartError>> {
        let supplied = self.pending_supplied_start.take()?;
        let (start, media, required) = supplied.into_parts();
        Some(
            persistent_session::prepare(self, start, media, &required)
                .and_then(|prepared| self.install_session_start(prepared)),
        )
    }
    pub(crate) fn install_session_start(
        &mut self,
        prepared: persistent_session::PreparedSessionStart,
    ) -> Result<(), persistent_session::SessionStartError> {
        self.install_session_start_inner(prepared, false)
    }
    fn install_session_start_inner(
        &mut self,
        prepared: persistent_session::PreparedSessionStart,
        scoped_adoption: bool,
    ) -> Result<(), persistent_session::SessionStartError> {
        use persistent_session::{SessionSlot, SessionStartError};
        if matches!(self.persistent_session, SessionSlot::Leased)
            || (!scoped_adoption && self.samus_mutation_depth != 0)
        {
            return Err(SessionStartError::Phase);
        }
        self.check_samus_binding(&prepared.binding.0)
            .map_err(SessionStartError::Binding)?;
        if prepared.gameplay.actor_identity.is_poisoned() {
            return Err(SessionStartError::Binding(DisplayError::Poisoned));
        }
        let (session, binding) = prepared.into_session();
        // An encoding failure leaves the completed compatibility prefix and
        // prevents the prior owner from resuming against those changed bytes.
        self.poison_persistent_session();
        crate::game::session_encoding::encode(&session.gameplay, |at, bytes| {
            self.ram.copy_in(at, bytes)
        });
        crate::game::session_runtime::export_native_spawn(&session.gameplay, self);
        self.samus_binding = Some(binding.0);
        self.samus_authority = SamusAuthority::Available;
        self.actor_identity = None;
        self.persistent_ram_epoch = self.ram.write_epoch();
        self.persistent_session = SessionSlot::Ready(Box::new(session));
        Ok(())
    }
    /// Whether `Runtime::new` would succeed now, without changing any state:
    /// the Samus runtime admits import, and any retained persistent Session
    /// is current with RAM.
    pub(crate) fn session_lease_admissible(&self) -> bool {
        use persistent_session::SessionSlot;
        self.preflight_samus_runtime().is_ok()
            && match &self.persistent_session {
                SessionSlot::Legacy => true,
                SessionSlot::Ready(_) => self.persistent_ram_epoch == self.ram.write_epoch(),
                SessionSlot::Leased | SessionSlot::Failed(_) => false,
            }
    }
    pub(crate) fn lease_persistent_session(
        &mut self,
    ) -> Result<Option<persistent_session::PersistentSession>, DisplayError> {
        use persistent_session::SessionSlot;
        if matches!(self.persistent_session, SessionSlot::Legacy) {
            return Ok(None);
        }
        self.preflight_samus_runtime()?;
        if self.persistent_ram_epoch != self.ram.write_epoch() {
            self.poison_persistent_session();
            return Err(DisplayError::Poisoned);
        }
        match std::mem::replace(&mut self.persistent_session, SessionSlot::Leased) {
            SessionSlot::Ready(session) => Ok(Some(*session)),
            other => {
                self.persistent_session = other;
                Err(DisplayError::Phase)
            }
        }
    }
    pub(crate) fn return_persistent_session(
        &mut self,
        mut session: persistent_session::PersistentSession,
        completed: bool,
        admitted_epoch: u64,
    ) {
        use persistent_session::SessionSlot;
        assert!(
            matches!(self.persistent_session, SessionSlot::Leased),
            "Session lease owner"
        );
        self.persistent_session = if completed && admitted_epoch == self.ram.write_epoch() {
            self.persistent_ram_epoch = admitted_epoch;
            SessionSlot::Ready(Box::new(session))
        } else {
            self.samus_authority = SamusAuthority::Revoked;
            session.poison();
            SessionSlot::Failed(Some(Box::new(session)))
        };
    }
    fn poison_persistent_session(&mut self) {
        use persistent_session::SessionSlot;
        match std::mem::replace(&mut self.persistent_session, SessionSlot::Failed(None)) {
            SessionSlot::Ready(mut state) | SessionSlot::Failed(Some(mut state)) => {
                state.poison();
                self.persistent_session = SessionSlot::Failed(Some(state));
            }
            other => self.persistent_session = other,
        }
    }
    pub(crate) fn checkpoint_session(
        &self,
    ) -> Result<persistent_session::SessionCheckpoint, DisplayError> {
        if self.persistent_ram_epoch != self.ram.write_epoch() {
            return Err(DisplayError::Poisoned);
        }
        self.preflight_samus_runtime()?;
        match &self.persistent_session {
            persistent_session::SessionSlot::Ready(session) => session.checkpoint(),
            _ => Err(DisplayError::Phase),
        }
    }
    pub(crate) fn capture_session_snapshot(
        &self,
    ) -> Result<persistent_session::SessionSnapshotPayload, DisplayError> {
        match &self.persistent_session {
            persistent_session::SessionSlot::Legacy => {
                Ok(persistent_session::SessionSnapshotPayload(None))
            }
            _ => self
                .checkpoint_session()
                .map(|state| persistent_session::SessionSnapshotPayload(Some(state))),
        }
    }
    pub(crate) fn begin_session_interval(&mut self, fresh_entry: bool) -> Result<(), DisplayError> {
        if matches!(
            self.persistent_session,
            persistent_session::SessionSlot::Legacy
        ) {
            return Ok(());
        }
        if self.persistent_ram_epoch != self.ram.write_epoch() {
            return Err(DisplayError::Poisoned);
        }
        self.preflight_samus_runtime()?;
        match &mut self.persistent_session {
            persistent_session::SessionSlot::Legacy => Ok(()),
            persistent_session::SessionSlot::Ready(state) => state.begin_interval(fresh_entry),
            _ => Err(DisplayError::Phase),
        }
    }
    pub(crate) fn prepare_session_adoption(
        &self,
        payload: &persistent_session::SessionSnapshotPayload,
    ) -> Result<persistent_session::PreparedSessionAdoption, persistent_session::SessionStartError>
    {
        persistent_session::prepare_snapshot(self, payload)
    }
    pub(crate) fn commit_session_adoption(
        &mut self,
        prepared: persistent_session::PreparedSessionAdoption,
    ) -> Result<(), persistent_session::SessionStartError> {
        if let Some(start) = prepared.0 {
            persistent_session::validate_normalized_projection(self, &start)
                .map_err(persistent_session::SessionStartError::Binding)?;
            self.install_session_start_inner(start, true)?;
        } else if !matches!(
            self.persistent_session,
            persistent_session::SessionSlot::Legacy
        ) {
            return Err(persistent_session::SessionStartError::Binding(
                DisplayError::Incomplete,
            ));
        }
        Ok(())
    }
    pub(crate) fn revoke_session_authority(&mut self) {
        self.revoke_samus_authority();
    }
    pub(crate) fn take_actor_identity(
        &mut self,
    ) -> crate::game::actors::identity::ActorIdentityState {
        if let Some(identity) = &self.actor_identity {
            assert!(
                !identity.is_poisoned(),
                "prepared checkpoint failed; restore a complete checkpoint before resuming"
            );
        }
        self.actor_identity
            .take()
            .expect("actor identity already owned by Session")
    }
    pub(crate) fn return_actor_identity(
        &mut self,
        identity: crate::game::actors::identity::ActorIdentityState,
    ) {
        assert!(
            self.actor_identity.is_none(),
            "Game already owns actor identity"
        );
        self.actor_identity = Some(identity);
    }
    pub(crate) fn actor_identity(&self) -> &crate::game::actors::identity::ActorIdentityState {
        if let persistent_session::SessionSlot::Ready(session) = &self.persistent_session {
            return &session.gameplay.actor_identity;
        }
        self.actor_identity
            .as_ref()
            .expect("actor identity currently owned by Session")
    }
    pub(crate) fn actor_identity_mut(
        &mut self,
    ) -> &mut crate::game::actors::identity::ActorIdentityState {
        if let persistent_session::SessionSlot::Ready(session) = &mut self.persistent_session {
            return &mut session.gameplay.actor_identity;
        }
        self.actor_identity
            .as_mut()
            .expect("actor identity currently owned by Session")
    }
    pub(crate) fn actor_identity_is_with_session(&self) -> bool {
        self.actor_identity.is_none()
    }
}

/// The ROM image a Samus registration reads.
fn registered_image<'a>(
    registration: content::Registration,
    loaded: &'a Option<crate::rom::RomImage>,
    native: &'a Option<crate::rom::RomImage>,
) -> Result<&'a crate::rom::RomImage, DisplayError> {
    match registration {
        content::Registration::Loaded => loaded,
        content::Registration::Native => native,
    }
    .as_ref()
    .ok_or(DisplayError::Identity)
}

#[forbid(unsafe_code)]
impl Game {
    fn samus_rom(&self, registration: content::Registration) -> Option<&crate::rom::RomImage> {
        registered_image(registration, &self.loaded_rom, &self.native_rom).ok()
    }
    fn check_samus_registration_metadata(
        &self,
        binding: &SamusBinding,
    ) -> Result<(), DisplayError> {
        self.check_samus_registration_images(binding, &self.loaded_rom, &self.native_rom)
    }
    fn check_samus_registration_images(
        &self,
        binding: &SamusBinding,
        loaded: &Option<crate::rom::RomImage>,
        native: &Option<crate::rom::RomImage>,
    ) -> Result<(), DisplayError> {
        if !Arc::ptr_eq(&binding.world, self.world()) {
            return Err(DisplayError::Identity);
        }
        for (&(registration, identity), admitted) in &binding.registrations {
            let image = registered_image(registration, loaded, native)?;
            let declaration = &binding.roles.description().allocations[&identity];
            if image.len() != declaration.extent {
                return Err(DisplayError::Source);
            }
            admitted.capture(registration, image)?;
        }
        Ok(())
    }
    fn check_samus_binding(&self, binding: &SamusBinding) -> Result<(), DisplayError> {
        self.check_samus_binding_images(binding, &self.loaded_rom, &self.native_rom)
    }
    fn check_samus_binding_images(
        &self,
        binding: &SamusBinding,
        loaded: &Option<crate::rom::RomImage>,
        native: &Option<crate::rom::RomImage>,
    ) -> Result<(), DisplayError> {
        self.check_samus_registration_images(binding, loaded, native)?;
        if binding.execution == SamusExecution::Prepared {
            for &(registration, identity) in binding.registrations.keys() {
                let image = registered_image(registration, loaded, native)?;
                let declaration = &binding.roles.description().allocations[&identity];
                native_content::RegisteredRom::admit(
                    registration,
                    image.clone(),
                    declaration.clone(),
                    &binding.expected_roles.allocations[&identity],
                )?;
            }
        }
        Ok(())
    }
    pub(crate) fn prepare_samus_binding(
        &self,
        supplied: SamusBindingDescription,
        expected: &SamusBindingDescription,
    ) -> Result<PreparedSamusBinding, DisplayError> {
        if !Arc::ptr_eq(&supplied.world, &expected.world)
            || !Arc::ptr_eq(&supplied.world, self.world())
        {
            return Err(DisplayError::Identity);
        }
        if supplied.execution != expected.execution {
            return Err(DisplayError::Incomplete);
        }
        supplied
            .request_media
            .validate()
            .map_err(|_| DisplayError::Scene)?;
        if supplied.request_media != expected.request_media {
            return Err(DisplayError::Incomplete);
        }
        supplied.spritemaps.validate(&supplied.request_media)?;
        supplied
            .spritemaps
            .validate_expected(&expected.spritemaps)?;
        let mut expected_requests = expected.roles.requests.iter();
        let roles = Arc::new(content::ValidatedContent::construct(
            supplied.roles,
            &expected.roles,
            |request| {
                request.validate(
                    expected_requests
                        .next()
                        .ok_or(DisplayError::Incomplete)?
                        .request
                        .pointer,
                    &supplied.request_media,
                )
            },
        )?);
        let mut registrations = BTreeMap::new();
        for (&(role, registration), declaration) in &roles.description().roles {
            if registration == content::Registration::Native
                && matches!(
                    role,
                    content::Role::Landing | content::Role::Crouching | content::Role::Drained
                )
            {
                continue;
            }
            let key = (registration, declaration.allocation);
            if registrations.contains_key(&key) {
                continue;
            }
            let allocation = roles
                .description()
                .allocations
                .get(&declaration.allocation)
                .ok_or(DisplayError::Incomplete)?;
            let expected_allocation = expected
                .roles
                .allocations
                .get(&declaration.allocation)
                .ok_or(DisplayError::Incomplete)?;
            let image = self
                .samus_rom(registration)
                .ok_or(DisplayError::Identity)?
                .clone();
            registrations.insert(
                key,
                native_content::RegisteredRom::admit(
                    registration,
                    image,
                    allocation.clone(),
                    expected_allocation,
                )?,
            );
        }
        Ok(PreparedSamusBinding(Arc::new(SamusBinding {
            world: supplied.world,
            roles,
            expected_roles: Arc::new(expected.roles.clone()),
            spritemaps: supplied.spritemaps,
            #[cfg(sm_verify_dll)]
            request_media: supplied.request_media,
            registrations,
            execution: supplied.execution,
        })))
    }
    #[cfg(sm_verify_dll)]
    pub(crate) fn install_samus_binding(
        &mut self,
        prepared: PreparedSamusBinding,
    ) -> Result<(), DisplayError> {
        if self.actor_identity_is_with_session() {
            return Err(DisplayError::Phase);
        }
        self.check_samus_binding(&prepared.0)?;
        self.samus_binding = Some(prepared.0);
        self.samus_authority = SamusAuthority::Available;
        Ok(())
    }
    pub(crate) fn samus_authority(&self) -> SamusAuthority {
        if self.samus_binding.is_some() && self.samus_mutation_depth != 0 {
            return SamusAuthority::Revoked;
        }
        if self.samus_authority == SamusAuthority::Available {
            if let Some(binding) = &self.samus_binding {
                if binding.execution == SamusExecution::Prepared
                    && self.check_samus_registration_metadata(binding).is_err()
                {
                    return SamusAuthority::Revoked;
                }
            }
        }
        self.samus_authority
    }
    pub(crate) fn revoke_samus_authority(&mut self) {
        self.poison_persistent_session();
        if self.samus_binding.is_some() {
            self.samus_authority = SamusAuthority::Revoked;
        }
    }
    /// Unscoped backend/ROM writers invalidate resumption. Scoped writers remain
    /// revoked until their guard successfully adopts the actual selected state.
    pub(crate) fn note_samus_external_mutation(&mut self) {
        if self.samus_mutation_depth == 0 {
            self.revoke_samus_authority();
        }
    }
    /// [`Self::begin_samus_mutation`] where a rejection is a broken invariant.
    pub(crate) fn samus_mutation(&mut self) -> SamusMutation<'_> {
        self.begin_samus_mutation()
            .expect("Samus mutation rejected")
    }
    pub(crate) fn begin_samus_mutation(&mut self) -> Result<SamusMutation<'_>, DisplayError> {
        // A supplied Prepared owner cannot enter unrestricted native mutation.
        // Native compatibility remains available for the legacy/reference path.
        if !matches!(
            self.persistent_session,
            persistent_session::SessionSlot::Legacy
        ) && self
            .samus_binding
            .as_ref()
            .is_some_and(|binding| binding.execution == SamusExecution::Prepared)
        {
            return Err(DisplayError::Phase);
        }
        self.samus_mutation_depth = self
            .samus_mutation_depth
            .checked_add(1)
            .ok_or(DisplayError::Phase)?;
        Ok(SamusMutation {
            game: self,
            completed: false,
        })
    }
    /// Legacy reference imports have no supplied semantic content payload. Keep
    /// their whole load/patch/restore interval revoked, including callback drops.
    pub fn with_reference_import<R>(&mut self, action: impl FnOnce(&mut Game) -> R) -> R {
        let mut mutation = self.samus_mutation();
        action(mutation.game())
    }
    pub(crate) fn preflight_samus_runtime(&self) -> Result<(), DisplayError> {
        let Some(binding) = &self.samus_binding else {
            return Ok(());
        };
        if self.samus_authority != SamusAuthority::Available {
            return Err(DisplayError::Poisoned);
        }
        self.check_samus_registration_metadata(binding)?;
        // A fresh numeric compatibility interval inside a retained native call is
        // permitted; it never promotes prepared authority during an outer load.
        if self.samus_mutation_depth != 0 && binding.execution == SamusExecution::Prepared {
            return Err(DisplayError::Phase);
        }
        Ok(())
    }
    pub(crate) fn capture_samus_snapshot(&self) -> SamusSnapshotPayload {
        SamusSnapshotPayload {
            world: self.world.clone(),
            binding: self.samus_binding.clone(),
            loaded: self.loaded_rom.clone(),
            native: self.native_rom.clone(),
            authority: self.samus_authority(),
            admitted: self.samus_binding.is_none()
                || self.samus_authority == SamusAuthority::Available,
        }
    }
    /// Validate the payload's own registrations before a restore. A target branch
    /// can select allocations different from the registrations currently in Game.
    pub(crate) fn prepare_samus_adoption(
        &self,
        payload: &SamusSnapshotPayload,
    ) -> Result<PreparedSamusAdoption, DisplayError> {
        if !Arc::ptr_eq(&payload.world, self.world()) {
            return Err(DisplayError::Identity);
        }
        if !payload.admitted {
            return Err(DisplayError::Poisoned);
        }
        if let Some(binding) = &payload.binding {
            self.check_samus_binding_images(binding, &payload.loaded, &payload.native)?;
        } else if payload.authority != SamusAuthority::LegacyUnrouted {
            return Err(DisplayError::Incomplete);
        }
        Ok(PreparedSamusAdoption(payload.clone()))
    }
    /// The caller has adopted the actual machine and actor state. Recheck retained
    /// Cell-backed images before publishing registrations, selection and authority.
    pub(crate) fn commit_samus_adoption(
        &mut self,
        prepared: PreparedSamusAdoption,
    ) -> Result<(), DisplayError> {
        self.prepare_samus_adoption(&prepared.0)?;
        let payload = prepared.0;
        self.loaded_rom = payload.loaded;
        self.native_rom = payload.native;
        self.samus_binding = payload.binding;
        self.samus_authority = if self.samus_binding.is_some() {
            SamusAuthority::Available
        } else {
            SamusAuthority::LegacyUnrouted
        };
        Ok(())
    }
    pub(crate) fn samus_execution(&self) -> Result<Option<SamusExecution>, DisplayError> {
        let Some(binding) = &self.samus_binding else {
            return Ok(None);
        };
        if !Arc::ptr_eq(&binding.world, self.world()) {
            return Err(DisplayError::Identity);
        }
        if binding.execution == SamusExecution::Prepared
            && (self.samus_authority == SamusAuthority::Revoked || self.samus_mutation_depth != 0)
        {
            return Err(DisplayError::Poisoned);
        }
        if binding.execution == SamusExecution::Prepared {
            self.check_samus_registration_metadata(binding)?;
        }
        Ok(Some(binding.execution))
    }
    /// The Samus binding, once `samus_execution` admits it.
    fn admitted_samus_binding(&self) -> Result<Option<&SamusBinding>, DisplayError> {
        Ok(self.samus_execution()?.and(self.samus_binding.as_deref()))
    }
    /// The caller performs this observation's original width/diagnostic preflight first.
    /// Displacement is linear from the sealed role base; no consumer supplies an identity.
    pub(crate) fn capture_samus_role(
        &self,
        role: content::Role,
        registration: content::Registration,
        displacement: usize,
        selected: &crate::rom::RomImage,
    ) -> Result<Option<content::RoleLease<native_content::CapturedRom>>, DisplayError> {
        let Some(execution) = self.samus_execution()? else {
            return Ok(None);
        };
        if execution != SamusExecution::RetainedNative {
            return Err(DisplayError::Phase);
        }
        let binding = self.samus_binding.as_ref().unwrap();
        let declaration = binding
            .roles
            .description()
            .roles
            .get(&(role, registration))
            .ok_or(DisplayError::Incomplete)?;
        let image = self.samus_rom(registration).ok_or(DisplayError::Identity)?;
        if !image.same_allocation(selected)
            || image.len()
                != binding.roles.description().allocations[&declaration.allocation].extent
        {
            return Err(DisplayError::Identity);
        }
        let admitted = binding
            .registrations
            .get(&(registration, declaration.allocation))
            .ok_or(DisplayError::Incomplete)?;
        let capsule = admitted.capture(registration, selected)?;
        if capsule.registration() != registration {
            return Err(DisplayError::Identity);
        }
        match registration {
            content::Registration::Loaded => {
                binding.roles.acquire_loaded(role, displacement, capsule)
            }
            content::Registration::Native => {
                binding.roles.acquire_native(role, displacement, capsule)
            }
        }
        .map(Some)
    }
    #[cfg(sm_verify_dll)]
    pub(crate) fn samus_request_media(&self) -> Result<Option<Arc<SceneMedia>>, DisplayError> {
        Ok(self
            .admitted_samus_binding()?
            .map(|binding| binding.request_media.clone()))
    }
}

impl Game {
    pub(crate) fn prepare_actor_canonical(
        &self,
    ) -> crate::game::actors::identity::PreparedCanonicalTransition {
        self.actor_identity()
            .prepare_canonical_transition()
            .expect("actor handle epoch exhausted")
    }
    pub(crate) fn commit_actor_canonical(
        &mut self,
        transition: crate::game::actors::identity::PreparedCanonicalTransition,
    ) {
        let retain_poison =
            !self.supports_legacy_persistence() && self.actor_identity().is_poisoned();
        let ram = &self.ram;
        self.actor_identity
            .as_mut()
            .expect("Game owns actors during canonical adoption")
            .commit_canonical_transition(transition, &|at| ram.read_byte(at));
        if retain_poison {
            self.actor_identity_mut().retain_poison();
        }
    }
    pub(crate) fn capture_actor_snapshot(&self) -> sm_game::actors::snapshot::ActorSnapshotPayload {
        sm_game::actors::snapshot::ActorSnapshotPayload::capture(
            self.actor_identity(),
            self.world(),
        )
    }
}

/// The named-RAM interface `sm_game` modules use; the same projection as the
/// compat `word!` macros.
impl sm_game::ram::CompatibilityRam for Game {
    fn compatibility_byte(&self, at: usize) -> u8 {
        Game::compatibility_byte(self, at)
    }
    fn compatibility_write_byte(&mut self, at: usize, value: u8) {
        Game::compatibility_write_byte(self, at, value)
    }
}

impl sm_game::effects::SoundEffects for Game {
    fn queue_sound(&mut self, channel: usize, sound: u8, max_queued: u8) {
        crate::game::music::queue_sfx(channel, sound, max_queued, self);
    }
}

impl sm_game::effects::DisplayRefresh for Game {
    fn fade(&mut self, fade_in: bool) {
        crate::game::display_refresh::fade(fade_in, self);
    }
    fn set_blank(&mut self, blank: bool) {
        crate::game::display_refresh::set_blank(blank, self);
    }
}

impl sm_game::scene::owned_samus::SamusSources for Game {
    fn prepared_samus_role(
        &self,
        role: content::Role,
        registration: content::Registration,
        displacement: usize,
    ) -> Result<Option<content::RoleLease<content::PreparedBytes>>, DisplayError> {
        let Some(execution) = self.samus_execution()? else {
            return Ok(None);
        };
        if execution == SamusExecution::RetainedNative
            && !(registration == content::Registration::Native
                && matches!(
                    role,
                    content::Role::Landing | content::Role::Crouching | content::Role::Drained
                ))
        {
            return Err(DisplayError::Phase);
        }
        self.samus_binding
            .as_ref()
            .unwrap()
            .roles
            .acquire_prepared(role, registration, displacement)
            .map(Some)
    }
    fn samus_role_base(
        &self,
        role: content::Role,
        registration: content::Registration,
    ) -> Result<Option<usize>, DisplayError> {
        let Some(binding) = self.admitted_samus_binding()? else {
            return Ok(None);
        };
        binding
            .roles
            .description()
            .roles
            .get(&(role, registration))
            .map(|declaration| Some(declaration.mapped_base))
            .ok_or(DisplayError::Incomplete)
    }
    fn samus_request(
        &self,
        half: sm_game::scene::samus_body::Half,
        pointer: u16,
    ) -> Result<Option<TileRequest>, DisplayError> {
        self.admitted_samus_binding()?
            .map(|binding| binding.roles.request(half, pointer))
            .transpose()
    }
    fn samus_spritemaps(&self) -> Result<Option<Arc<SamusCatalog>>, DisplayError> {
        Ok(self
            .admitted_samus_binding()?
            .map(|binding| binding.spritemaps.clone()))
    }
}

impl sm_game::records::Records for Game {
    fn records(&self) -> &sm_game::records::RomRecords {
        &self.records
    }
}

impl sm_game::cartridge::LoadedImage for Game {
    fn loaded_image(&self) -> &crate::rom::RomImage {
        self.loaded_rom.as_ref().expect("ROM must be loaded")
    }
}

impl sm_game::actors::host::Random for Game {
    fn next_random(&mut self) -> u16 {
        Game::next_random(self)
    }
}

impl sm_game::cartridge::NativeImage for Game {
    fn native_image(&self) -> crate::rom::RomImage {
        self.capture_native_rom()
    }
}

impl sm_game::display::sequence::Registers for Game {
    fn write_register(&mut self, register: u16, value: u8) {
        assert!(!self.g_snes.is_null(), "SNES backend must be installed");
        // SAFETY: the backend is installed, and `&mut self` excludes other access.
        unsafe { crate::rtl::write_reg(register, value, self) }
    }
    fn write_register_word(&mut self, register: u16, value: u16) {
        assert!(!self.g_snes.is_null(), "SNES backend must be installed");
        // SAFETY: as for `write_register`.
        unsafe { crate::rtl::write_reg_word(register, value, self) }
    }
    fn read_register_word(&mut self, register: u16) -> u16 {
        assert!(!self.g_snes.is_null(), "SNES backend must be installed");
        // SAFETY: as for `write_register`.
        unsafe { crate::rtl::read_reg_word(register, self) }
    }
    fn read_register(&mut self, register: u16) -> u8 {
        assert!(!self.g_snes.is_null(), "SNES backend must be installed");
        // SAFETY: as for `write_register`.
        unsafe { crate::rtl::read_reg(register, self) }
    }
}

impl sm_game::ram::RawRam for WorkRam {
    fn read_byte(&self, at: usize) -> u8 {
        WorkRam::read_byte(self, at)
    }
    fn write_byte(&self, at: usize, value: u8) {
        WorkRam::write_byte(self, at, value)
    }
    fn copy_in(&self, at: usize, input: &[u8]) {
        WorkRam::copy_in(self, at, input)
    }
    fn copy_out(&self, at: usize, out: &mut [u8]) {
        WorkRam::copy_out(self, at, out)
    }
    fn fill(&self, range: core::ops::Range<usize>, value: u8) {
        WorkRam::fill(self, range, value)
    }
    fn copy_within(&self, source: core::ops::Range<usize>, destination: usize) {
        WorkRam::copy_within(self, source, destination)
    }
}

/// Game's projection: compatibility reads and RAM-image stores, as named RAM words use.
impl sm_game::runtime_access::SessionRam for Game {
    fn byte(&self, at: usize) -> u8 {
        Game::compatibility_byte(self, at)
    }
    fn set_byte(&mut self, at: usize, value: u8) {
        self.ram().write_byte(at, value);
    }
    fn write_slice(&mut self, at: usize, bytes: &[u8]) {
        self.ram().copy_in(at, bytes);
    }
}

/// Slot words on the RAM image, native order, as `slot!` reads and stores them.
impl sm_game::actors::session_slots::SlotAccess for Game {
    fn slot_word(&self, at: usize) -> u16 {
        u16::from_ne_bytes([self.ram().read_byte(at), self.ram().read_byte(at + 1)])
    }
    fn set_slot_word(&mut self, at: usize, value: u16) {
        let bytes = value.to_ne_bytes();
        self.ram().write_byte(at, bytes[0]);
        self.ram().write_byte(at + 1, bytes[1]);
    }
    fn slot_byte(&self, at: usize) -> u8 {
        self.ram().read_byte(at)
    }
    fn set_slot_byte(&mut self, at: usize, value: u8) {
        self.ram().write_byte(at, value);
    }
}

impl sm_game::ram::WorkRamAccess for Game {
    type Ram = WorkRam;
    fn ram(&self) -> &WorkRam {
        Game::ram(self)
    }
}
