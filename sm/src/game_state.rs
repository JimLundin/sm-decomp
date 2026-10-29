//! Explicit per-game ownership; RAM bytes retain their exact SNES layout.
use crate::cpu_infra::Snapshot;
use crate::game::room_entry::world::WorldBinding;
use crate::game::scene::{
    records::SceneMedia, samus::SamusCatalog, samus_content as content,
    samus_native_content as native_content, samus_position_tiles::TileRequest, state::DisplayError,
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
    pub(crate) actor_snapshots: crate::game::actors::snapshot::ActorSnapshotSidecars,
    world: Arc<WorldBinding>,
    ram: WorkRam,
    pub(crate) loaded_rom: Option<crate::rom::RomImage>,
    native_rom: Option<crate::rom::RomImage>,
    /// Imported from `loaded_rom` by `register_rom`; see game::enemy_speeds.
    pub(crate) enemy_speeds: Option<crate::game::enemy_speeds::ImportedSpeeds>,
    /// Imported from `native_rom` by `register_native_rom`; see game::enemy_speeds.
    pub(crate) enemy_speeds32: Option<crate::game::enemy_speeds::ImportedSpeeds32>,
    /// Imported from `loaded_rom` by `register_rom`; see game::sand_heap.
    pub(crate) sand_heap: Option<crate::game::sand_heap::ImportedSandHeap>,
    /// Imported from `loaded_rom` by `register_rom`; see game::ceres_elevator.
    pub(crate) ceres_rotation: Option<crate::game::ceres_elevator::ImportedRotation>,
    /// Imported from `loaded_rom` by `register_rom`; see game::shooting_stars.
    pub(crate) shooting_stars: Option<crate::game::shooting_stars::ImportedShootingStars>,
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
    pub k_out: u16,

    pub g_sram: crate::save_ram::SaveRam,
    pub g_rtl_memory_ptr: *mut u8,
    pub g_rtl_runframe: Option<RunFrameFunc>,
    pub g_rtl_syncall: Option<SyncAllFunc>,
    pub state_recorder: StateRecorder,
    pub apu_queue: crate::apu_queue::ApuQueue,
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
        let Self {
            backend,
            state_recorder,
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
            g_debug_apu_cycles,
            g_new_ppu,
            decompress_src,
            eproj_spawn_rect,
            eproj_spawn_r22,
            eproj_spawn_pt,
            eproj_spawn_varE24,
            ..
        } = self;
        (
            backend.as_mut(),
            crate::execution::HostContext {
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
                g_debug_apu_cycles,
                g_new_ppu,
                decompress_src,
                eproj_spawn_rect,
                eproj_spawn_r22,
                eproj_spawn_pt,
                eproj_spawn_varE24,
            },
            state_recorder,
        )
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
            if let Some(player) = self
                ._native_player
                .as_mut()
                .filter(|player| player.view() == self.g_spc_player)
            {
                if player.try_initialize() {
                    return;
                }
            }
            crate::spc_player::SpcPlayer_Initialize(self.g_spc_player);
        }
    }

    /// Export current fields and optional queued ports into this player's RAM.
    /// # Safety
    /// Hold the existing APU lock with exclusive non-reentrant native views.
    pub unsafe fn save_native_audio_variables(&mut self, include_queued_commands: bool) {
        unsafe {
            if let Some(player) = self
                ._native_player
                .as_mut()
                .filter(|player| player.view() == self.g_spc_player)
            {
                player.save_state(include_queued_commands.then_some(&self.apu_queue));
            } else {
                if include_queued_commands {
                    self.apu_queue
                        .apply_queued(&mut (*self.g_spc_player).state.input_ports);
                }
                crate::spc_player::SpcPlayer_CopyVariablesToRam(self.g_spc_player);
            }
        }
    }

    /// Import the actual current RAM image before resuming sequencing.
    /// # Safety
    /// Hold the existing APU lock with exclusive non-reentrant native views.
    pub unsafe fn load_native_audio_variables(&mut self) {
        unsafe {
            if let Some(player) = self
                ._native_player
                .as_mut()
                .filter(|player| player.view() == self.g_spc_player)
            {
                player.load_state();
            } else {
                crate::spc_player::SpcPlayer_CopyVariablesFromRam(self.g_spc_player);
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
            if let Some(player) = self
                ._native_player
                .as_mut()
                .filter(|player| player.view() == self.g_spc_player)
            {
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
            if let Some(player) = self
                ._native_player
                .as_mut()
                .filter(|player| player.view() == self.g_spc_player)
            {
                if player.try_render_samples(samples) {
                    return;
                }
            }
            // The owned borrow ended before selected/history/imported playback.
            crate::spc_player::SpcPlayer_GenerateSamples(self.g_spc_player);
            (*(*self.g_spc_player).state.dsp)
                .state
                .write_samples(samples);
        }
    }

    pub(crate) fn save_backend_observation(&self, output: &mut crate::recording::RecordingBytes) {
        self.backend
            .as_ref()
            .expect("observation requires an owned backend")
            .save_observation(self.g_snes, self.g_cpu, output);
    }

    pub(crate) fn next_random(&mut self) -> u16 {
        let mut product = (u16::from(self.compatibility_byte(0x5e5)) * 5).to_ne_bytes();
        let carry =
            u16::from(product[1]) + u16::from(self.compatibility_byte(0x5e6).wrapping_mul(5)) + 1;
        product[1] = carry as u8;
        let result = u16::from_ne_bytes(product).wrapping_add(17 + (carry >> 8));
        crate::ram_map::checked_word!(self.random_number = result);
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
        crate::ram_map::word!(self.game_state = u16::MAX);
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
        /// super_missiles count/capacity in the original RAM layout.
        samus_super_missiles: 0x9ca;
        samus_max_super_missiles: 0x9cc;
        /// power_bombs count/capacity in the original RAM layout.
        samus_power_bombs: 0x9ce;
        samus_max_power_bombs: 0x9d0;
        /// Health and reserve fields retain their original byte-backed RAM layout.
        samus_health: 0x9c2;
        /// Low-health sound latch ($7E:0A6A in ram_map).
        samus_health_warning: 0xa6a;
        /// Missile fields in ram_map at $7E:09C6/09C8/09D8.
        samus_missiles: 0x9c6;
        samus_max_missiles: 0x9c8;
        samus_reserve_missiles: 0x9d8;
        samus_max_health: 0x9c4;
        samus_reserve_health: 0x9d6;
        samus_max_reserve_health: 0x9d4;
        reserve_health_mode: 0x9c0;
        /// random_number ($7E:05E5 in ram_map).
        random_number: 0x5e5;
        /// special_death_item_drop_x_origin_pos ($7E:F434 in ram_map).
        special_death_item_drop_x_origin_pos: 0xf434;
        /// special_death_item_drop_y_origin_pos ($7E:F436 in ram_map).
        special_death_item_drop_y_origin_pos: 0xf436;
        /// timer_status ($7E:0943 in ram_map).
        timer_status: 0x943;
        /// door_transition_function ($7E:099C in ram_map).
        door_transition_function: 0x99c;
        /// hdma_objects_enable_flag ($7E:18B0 in ram_map).
        hdma_objects_enable_flag: 0x18b0;
        /// room_loading_irq_handler ($7E:00A9 in ram_map).
        room_loading_irq_handler: 0xa9;
        layer1_x_pos: 0x911;
        layer1_y_pos: 0x915;
        /// door_direction ($7E:0791 in ram_map).
        door_direction: 0x791;
        /// elevator_door_properties_orientation ($7E:0793 in ram_map).
        elevator_door_properties_orientation: 0x793;
        /// elevator_direction ($7E:0799 in ram_map).
        elevator_direction: 0x799;
        /// elevator_flags ($7E:0E16 in ram_map).
        elevator_flags: 0xe16;
        /// cre_bitset ($7E:07B3 in ram_map).
        cre_bitset: 0x7b3;
        /// door_def_ptr ($7E:078D in ram_map).
        door_def_ptr: 0x78d;
        /// room_ptr ($7E:079B in ram_map).
        room_ptr: 0x79b;
        /// room_height_in_blocks ($7E:07A7 in ram_map).
        room_height_in_blocks: 0x7a7;
        /// room_width_in_scrolls ($7E:07A9 in ram_map).
        room_width_in_scrolls: 0x7a9;
        /// room_height_in_scrolls ($7E:07AB in ram_map).
        room_height_in_scrolls: 0x7ab;
        /// roomdefroomstate_ptr ($7E:07BB in ram_map).
        roomdefroomstate_ptr: 0x7bb;
        /// reg_BG1VOFS ($7E:00B3 in ram_map).
        reg_bg1vofs: 0xb3;
        /// samus_x_pos ($7E:0AF6 in ram_map).
        samus_x_pos: 0xaf6;
        /// samus_x_subpos ($7E:0AF8 in ram_map).
        samus_x_subpos: 0xaf8;
        /// samus_y_pos ($7E:0AFA in ram_map).
        samus_y_pos: 0xafa;
        /// distance_to_eject_samus_up ($7E:0E08 in ram_map).
        distance_to_eject_samus_up: 0xe08;
        /// distance_to_eject_samus_down ($7E:0E0A in ram_map).
        distance_to_eject_samus_down: 0xe0a;
        /// samus_y_subpos ($7E:0AFC in ram_map).
        samus_y_subpos: 0xafc;
        /// samus_x_radius ($7E:0AFE in ram_map).
        samus_x_radius: 0xafe;
        /// extra_samus_y_displacement ($7E:0B5C in ram_map).
        extra_samus_y_displacement: 0xb5c;
        /// samus_y_radius ($7E:0B00 in ram_map).
        samus_y_radius: 0xb00;
        /// samus_pose ($7E:0A1C in ram_map).
        samus_pose: 0xa1c;
        /// samus_anim_frame ($7E:0A96 in ram_map).
        samus_anim_frame: 0xa96;
        /// joypad1_lastkeys ($7E:008B in ram_map).
        joypad1_lastkeys: 0x8b;
        /// samus_prev_x_pos ($7E:0B10 in ram_map).
        samus_prev_x_pos: 0xb10;
        /// samus_prev_y_pos ($7E:0B14 in ram_map).
        samus_prev_y_pos: 0xb14;
        debug_invincibility: 0xde0;
        /// game_state ($7E:0998 in ram_map).
        game_state: 0x998;
        /// debug_disable_sounds ($7E:05F5 in ram_map).
        debug_disable_sounds: 0x5f5;
        /// power_bomb_explosion_status ($7E:0592 in ram_map).
        power_bomb_explosion_status: 0x592;
        pausemenu_button_label_mode: 0x753;
        /// loading_game_state ($7E:D914 in ram_map).
        loading_game_state: 0xd914;
        /// area_index ($7E:079F in ram_map).
        area_index: 0x79f;
        collected_items: 0x9a4;
        /// demo_set ($7E:1F55 in ram_map).
        demo_set: 0x1f55;
        /// demo_scene ($7E:1F57 in ram_map).
        demo_scene: 0x1f57;
        /// demo_input_pre_instr ($7E:0A7A in ram_map).
        demo_input_pre_instr: 0xa7a;
        /// demo_input_instr_timer ($7E:0A7C in ram_map).
        demo_input_instr_timer: 0xa7c;
        /// demo_input_instr_ptr ($7E:0A7E in ram_map).
        demo_input_instr_ptr: 0xa7e;
        /// demo_input ($7E:0A84 in ram_map).
        demo_input: 0xa84;
        /// demo_input_new ($7E:0A86 in ram_map).
        demo_input_new: 0xa86;
        /// demo_enable ($7E:0A88 in ram_map), shared with hdma_ptr_1.
        demo_enable: 0xa88;
        /// demo_input_prev ($7E:0A8C in ram_map).
        demo_input_prev: 0xa8c;
        /// demo_input_prev_new ($7E:0A8E in ram_map), shared with hdma_ptr_3.
        demo_input_prev_new: 0xa8e;
        /// button_config_jump_a ($7E:09B4 in ram_map).
        button_config_jump_a: 0x9b4;
        /// joypad1_newkeys ($7E:008F in ram_map).
        joypad1_newkeys: 0x8f;
        /// collision_detection_index ($7E:18A6 in ram_map).
        collision_detection_index: 0x18a6;
        /// interactive_enemy_indexes_write_ptr ($7E:17A6 in ram_map).
        interactive_enemy_indexes_write_ptr: 0x17a6;
        /// samus_collision_direction ($7E:0B02 in ram_map).
        samus_collision_direction: 0xb02;
        /// samus_collision_flag ($7E:0DD0 in ram_map).
        samus_collision_flag: 0xdd0;
        /// flag_samus_in_quicksand ($7E:1E71 in ram_map).
        flag_samus_in_quicksand: 0x1e71;
        /// inside_block_reaction_samus_point ($7E:1E73 in ram_map).
        inside_block_reaction_samus_point: 0x1e73;
        /// Current collision block index ($7E:0DC4 in ram_map).
        cur_block_index: 0xdc4;
        current_slope_bts: 0x1e77;
        /// enemy_tile_vram_src ($7E:0E7C in ram_map).
        enemy_tile_vram_src: 0xe7c;
        /// enemy_tile_vram_dst ($7E:0E7E in ram_map).
        enemy_tile_vram_dst: 0xe7e;
        /// nmi_flag_bg2_enemy_vram_transfer ($7E:0E1E in ram_map).
        nmi_flag_bg2_enemy_vram_transfer: 0xe1e;
        /// enemy_bg2_tilemap_size ($7E:179A in ram_map).
        enemy_bg2_tilemap_size: 0x179a;
        /// debug_time_frozen_for_enemies ($7E:185E in ram_map).
        debug_time_frozen_for_enemies: 0x185e;
        /// time_is_frozen_flag ($7E:0A78 in ram_map).
        time_is_frozen_flag: 0xa78;
        /// VRAM write queue byte cursor ($7E:0330 in ram_map).
        vram_write_queue_tail: 0x330;
        /// Room width in blocks ($7E:07A5 in ram_map).
        room_width_in_blocks: 0x7a5;
        /// Current enemy slot offset ($7E:0E54 in ram_map).
        cur_enemy_index: 0xe54;
    }

    ram_word_accessors! {
        write;
        set_samus_super_missiles: 0x9ca;
        set_samus_power_bombs: 0x9ce;
        set_samus_health_warning: 0xa6a;
        set_samus_missiles: 0x9c6;
        set_samus_reserve_missiles: 0x9d8;
        set_samus_health: 0x9c2;
        set_samus_reserve_health: 0x9d6;
        set_reserve_health_mode: 0x9c0;
        set_door_transition_function: 0x99c;
        set_hdma_objects_enable_flag: 0x18b0;
        /// irqhandler_next_handler ($7E:00A7 in ram_map).
        set_irqhandler_next_handler: 0xa7;
        /// debug_disable_sounds ($7E:05F5 in ram_map).
        set_debug_disable_sounds: 0x5f5;
        /// door_transition_flag_elevator_zebetites ($7E:0795 in ram_map).
        set_door_transition_flag_elevator_zebetites: 0x795;
        /// door_transition_flag_enemies ($7E:0797 in ram_map).
        set_door_transition_flag_enemies: 0x797;
        /// game_state ($7E:0998 in ram_map).
        set_game_state: 0x998;
        /// layer1_x_pos ($7E:0911 in ram_map).
        set_layer1_x_pos: 0x911;
        /// layer1_y_pos ($7E:0915 in ram_map).
        set_layer1_y_pos: 0x915;
        set_door_direction: 0x791;
        set_elevator_door_properties_orientation: 0x793;
        set_elevator_flags: 0xe16;
        /// previous_cre_bitset ($7E:07B1 in ram_map).
        set_previous_cre_bitset: 0x7b1;
        set_cre_bitset: 0x7b3;
        set_room_ptr: 0x79b;
        /// room_index ($7E:079D in ram_map).
        set_room_index: 0x79d;
        /// area_index ($7E:079F in ram_map).
        set_area_index: 0x79f;
        /// room_x_coordinate_on_map ($7E:07A1 in ram_map).
        set_room_x_coordinate_on_map: 0x7a1;
        /// room_y_coordinate_on_map ($7E:07A3 in ram_map).
        set_room_y_coordinate_on_map: 0x7a3;
        set_room_height_in_blocks: 0x7a7;
        set_room_width_in_scrolls: 0x7a9;
        set_room_height_in_scrolls: 0x7ab;
        /// up_scroller ($7E:07AD in ram_map).
        set_up_scroller: 0x7ad;
        /// down_scroller ($7E:07AF in ram_map).
        set_down_scroller: 0x7af;
        /// door_list_pointer ($7E:07B5 in ram_map).
        set_door_list_pointer: 0x7b5;
        /// room_size_in_blocks ($7E:07B9 in ram_map).
        set_room_size_in_blocks: 0x7b9;
        /// room_music_track_index ($7E:07C9 in ram_map).
        set_room_music_track_index: 0x7c9;
        /// room_music_data_index ($7E:07CB in ram_map).
        set_room_music_data_index: 0x7cb;
        /// room_layer3_asm_ptr ($7E:07CD in ram_map).
        set_room_layer3_asm_ptr: 0x7cd;
        /// room_enemy_population_ptr ($7E:07CF in ram_map).
        set_room_enemy_population_ptr: 0x7cf;
        /// room_enemy_tilesets_ptr ($7E:07D1 in ram_map).
        set_room_enemy_tilesets_ptr: 0x7d1;
        /// room_main_code_ptr ($7E:07DF in ram_map).
        set_room_main_code_ptr: 0x7df;
        set_reg_bg1vofs: 0xb3;
        /// reg_BG2HOFS ($7E:00B5 in ram_map).
        set_reg_bg2hofs: 0xb5;
        /// reg_BG2VOFS ($7E:00B7 in ram_map).
        set_reg_bg2vofs: 0xb7;
        /// scrolling_finished_hook ($7E:07E9 in ram_map).
        set_scrolling_finished_hook: 0x7e9;
        /// door_transition_frame_counter ($7E:0925 in ram_map).
        set_door_transition_frame_counter: 0x925;
        /// door_destination_x_pos ($7E:0927 in ram_map).
        set_door_destination_x_pos: 0x927;
        /// door_destination_y_pos ($7E:0929 in ram_map).
        set_door_destination_y_pos: 0x929;
        /// samus_door_transition_subspeed ($7E:092B in ram_map).
        set_samus_door_transition_subspeed: 0x92b;
        /// samus_door_transition_speed ($7E:092D in ram_map).
        set_samus_door_transition_speed: 0x92d;
        set_samus_x_pos: 0xaf6;
        /// door_transition_flag ($7E:0931 in ram_map).
        set_door_transition_flag: 0x931;
        set_samus_x_subpos: 0xaf8;
        set_samus_y_pos: 0xafa;
        set_samus_y_subpos: 0xafc;
        set_extra_samus_y_displacement: 0xb5c;
        /// samus_pose ($7E:0A1C in ram_map).
        set_samus_pose: 0xa1c;
        /// samus_prev_pose ($7E:0A20 in ram_map).
        set_samus_prev_pose: 0xa20;
        /// samus_last_different_pose ($7E:0A24 in ram_map).
        set_samus_last_different_pose: 0xa24;
        /// samus_anim_frame_timer ($7E:0A94 in ram_map).
        set_samus_anim_frame_timer: 0xa94;
        set_samus_anim_frame: 0xa96;
        set_joypad1_lastkeys: 0x8b;
        /// input_to_pose_calc ($7E:0DC6 in ram_map).
        set_input_to_pose_calc: 0xdc6;
        set_samus_prev_x_pos: 0xb10;
        set_samus_prev_y_pos: 0xb14;
        /// samus_momentum_routine_index ($7E:0A2E in ram_map).
        set_samus_momentum_routine_index: 0xa2e;
        /// samus_special_transgfx_index ($7E:0A30 in ram_map).
        set_samus_special_transgfx_index: 0xa30;
        /// samus_hurt_switch_index ($7E:0A32 in ram_map).
        set_samus_hurt_switch_index: 0xa32;
        /// absolute_moved_last_frame_x ($7E:0DA2 in ram_map).
        set_absolute_moved_last_frame_x: 0xda2;
        /// absolute_moved_last_frame_x_fract ($7E:0DA4 in ram_map).
        set_absolute_moved_last_frame_x_fract: 0xda4;
        /// absolute_moved_last_frame_y ($7E:0DA6 in ram_map).
        set_absolute_moved_last_frame_y: 0xda6;
        /// absolute_moved_last_frame_y_fract ($7E:0DA8 in ram_map).
        set_absolute_moved_last_frame_y_fract: 0xda8;
        /// samus_y_subaccel ($7E:0B32 in ram_map).
        set_samus_y_subaccel: 0xb32;
        /// samus_y_accel ($7E:0B34 in ram_map).
        set_samus_y_accel: 0xb34;
        /// debug_invincibility ($7E:0DE0 in ram_map).
        set_debug_invincibility: 0xde0;
        /// unused_word_7e0a5e ($7E:0A5E in ram_map).
        set_unused_word_7e0a5e: 0xa5e;
        /// samus_prev_health_for_flash ($7E:0A12 in ram_map).
        set_samus_prev_health_for_flash: 0xa12;
        /// samus_visor_palette_timer_index ($7E:0A72 in ram_map).
        set_samus_visor_palette_timer_index: 0xa72;
        /// enable_horiz_slope_coll ($7E:0A46 in ram_map).
        set_enable_horiz_slope_coll: 0xa46;
        /// samus_hurt_flash_counter ($7E:0A48 in ram_map).
        set_samus_hurt_flash_counter: 0xa48;
        /// samus_special_super_palette_flags ($7E:0A4A in ram_map).
        set_samus_special_super_palette_flags: 0xa4a;
        /// fx_y_pos ($7E:195E in ram_map).
        set_fx_y_pos: 0x195e;
        /// lava_acid_y_pos ($7E:1962 in ram_map).
        set_lava_acid_y_pos: 0x1962;
        /// cinematic_function ($7E:1F51 in ram_map).
        set_cinematic_function: 0x1f51;
        /// hud_item_index ($7E:09D2 in ram_map).
        set_hud_item_index: 0x9d2;
        /// samus_auto_cancel_hud_item_index ($7E:0A04 in ram_map).
        set_samus_auto_cancel_hud_item_index: 0xa04;
        /// samus_invincibility_timer ($7E:18A8 in ram_map).
        set_samus_invincibility_timer: 0x18a8;
        /// samus_knockback_timer ($7E:18AA in ram_map).
        set_samus_knockback_timer: 0x18aa;
        /// has_area_map ($7E:0789 in ram_map).
        set_has_area_map: 0x789;
        /// collected_items ($7E:09A4 in ram_map).
        set_collected_items: 0x9a4;
        /// equipped_items ($7E:09A2 in ram_map).
        set_equipped_items: 0x9a2;
        /// samus_max_missiles ($7E:09C8 in ram_map).
        set_samus_max_missiles: 0x9c8;
        /// samus_max_super_missiles ($7E:09CC in ram_map).
        set_samus_max_super_missiles: 0x9cc;
        /// samus_max_power_bombs ($7E:09D0 in ram_map).
        set_samus_max_power_bombs: 0x9d0;
        /// samus_max_health ($7E:09C4 in ram_map).
        set_samus_max_health: 0x9c4;
        /// collected_beams ($7E:09A8 in ram_map).
        set_collected_beams: 0x9a8;
        /// equipped_beams ($7E:09A6 in ram_map).
        set_equipped_beams: 0x9a6;
        /// button_config_up ($7E:09AA in ram_map).
        set_button_config_up: 0x9aa;
        /// button_config_down ($7E:09AC in ram_map).
        set_button_config_down: 0x9ac;
        /// button_config_left ($7E:09AE in ram_map).
        set_button_config_left: 0x9ae;
        /// button_config_right ($7E:09B0 in ram_map).
        set_button_config_right: 0x9b0;
        /// button_config_shoot_x ($7E:09B2 in ram_map).
        set_button_config_shoot_x: 0x9b2;
        /// button_config_jump_a ($7E:09B4 in ram_map).
        set_button_config_jump_a: 0x9b4;
        /// button_config_run_b ($7E:09B6 in ram_map).
        set_button_config_run_b: 0x9b6;
        /// button_config_itemcancel_y ($7E:09B8 in ram_map).
        set_button_config_itemcancel_y: 0x9b8;
        /// button_config_itemswitch ($7E:09BA in ram_map).
        set_button_config_itemswitch: 0x9ba;
        /// button_config_aim_up_r ($7E:09BE in ram_map).
        set_button_config_aim_up_r: 0x9be;
        /// button_config_aim_down_l ($7E:09BC in ram_map).
        set_button_config_aim_down_l: 0x9bc;
        /// unused_word_7e09e8 ($7E:09E8 in ram_map).
        set_unused_word_7e09e8: 0x9e8;
        /// debug_flag ($7E:09E6 in ram_map).
        set_debug_flag: 0x9e6;
        /// moonwalk_flag ($7E:09E4 in ram_map).
        set_moonwalk_flag: 0x9e4;
        /// unused_word_7e0df8 ($7E:0DF8 in ram_map).
        set_unused_word_7e0df8: 0xdf8;
        /// unused_word_7e0dfa ($7E:0DFA in ram_map).
        set_unused_word_7e0dfa: 0xdfa;
        /// unused_word_7e0dfc ($7E:0DFC in ram_map).
        set_unused_word_7e0dfc: 0xdfc;
        set_demo_input_pre_instr: 0xa7a;
        set_demo_input_instr_timer: 0xa7c;
        set_demo_input_instr_ptr: 0xa7e;
        /// demo_timer_counter ($7E:0A80 in ram_map).
        set_demo_timer_counter: 0xa80;
        /// xray_angle ($7E:0A82 in ram_map), also used by demo input objects.
        set_xray_angle: 0xa82;
        set_demo_input: 0xa84;
        set_demo_input_new: 0xa86;
        set_demo_enable: 0xa88;
        set_demo_input_prev: 0xa8c;
        set_demo_input_prev_new: 0xa8e;
        /// joypad1_input_samusfilter ($7E:0DFE in ram_map).
        set_joypad1_input_samusfilter: 0xdfe;
        /// joypad1_newinput_samusfilter ($7E:0E00 in ram_map).
        set_joypad1_newinput_samusfilter: 0xe00;
        /// frame_handler_alfa ($7E:0A42 in ram_map).
        set_frame_handler_alfa: 0xa42;
        /// frame_handler_beta ($7E:0A44 in ram_map).
        set_frame_handler_beta: 0xa44;
        /// frame_handler_gamma ($7E:0A5A in ram_map).
        set_frame_handler_gamma: 0xa5a;
        /// samus_movement_handler ($7E:0A58 in ram_map).
        set_samus_movement_handler: 0xa58;
        /// samus_draw_handler ($7E:0A5C in ram_map).
        set_samus_draw_handler: 0xa5c;
        /// samus_input_handler ($7E:0A60 in ram_map).
        set_samus_input_handler: 0xa60;
        /// grapple_beam_function ($7E:0D32 in ram_map).
        set_grapple_beam_function: 0xd32;
        /// samus_new_pose ($7E:0A28 in ram_map).
        set_samus_new_pose: 0xa28;
        /// samus_new_pose_interrupted ($7E:0A2A in ram_map).
        set_samus_new_pose_interrupted: 0xa2a;
        /// samus_new_pose_transitional ($7E:0A2C in ram_map).
        set_samus_new_pose_transitional: 0xa2c;
        set_joypad1_newkeys: 0x8f;
        set_collision_detection_index: 0x18a6;
        /// enemy_index_to_shake ($7E:0E1C in ram_map).
        set_enemy_index_to_shake: 0xe1c;
        set_samus_collision_direction: 0xb02;
        set_samus_collision_flag: 0xdd0;
        set_samus_pos_adjusted_by_slope_flag: 0xdba;
        set_flag_samus_in_quicksand: 0x1e71;
        set_inside_block_reaction_samus_point: 0x1e73;
        set_cur_block_index: 0xdc4;
        /// BTS saved by horizontal nonsquare slope dispatch ($7E:1E77 in ram_map).
        set_current_slope_bts: 0x1e77;
        set_enemy_tile_vram_src: 0xe7c;
        set_enemy_tile_vram_dst: 0xe7e;
        set_nmi_flag_bg2_enemy_vram_transfer: 0xe1e;
        set_vram_write_queue_tail: 0x330;
        set_room_width_in_blocks: 0x7a5;
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
        self.enemy_speeds32 = crate::game::enemy_speeds::import32(&image);
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
    pub(crate) fn register_rom(&mut self, image: crate::rom::RomImage) {
        if !self
            .loaded_rom
            .as_ref()
            .is_some_and(|old| old.same_allocation(&image))
        {
            self.revoke_samus_authority();
        }
        self.enemy_speeds = crate::game::enemy_speeds::import(&image);
        self.sand_heap = crate::game::sand_heap::import(&image);
        self.ceres_rotation = crate::game::ceres_elevator::import(&image);
        self.shooting_stars = crate::game::shooting_stars::import(&image);
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

    fn long_pointer_bytes(pointer: LongPtr) -> [u8; 3] {
        let [low, high] = pointer.addr.to_le_bytes();
        [low, high, pointer.bank]
    }

    /// room_compr_level_data_ptr ($7E:07BD in ram_map).
    pub fn set_room_compr_level_data_ptr(&mut self, value: LongPtr) {
        self.compatibility_copy_in(0x7bd, &Self::long_pointer_bytes(value));
    }

    /// tileset_tile_table_pointer ($7E:07C0 in ram_map).
    pub fn set_tileset_tile_table_pointer(&mut self, value: LongPtr) {
        self.compatibility_copy_in(0x7c0, &Self::long_pointer_bytes(value));
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

    pub fn set_tileset_tiles_pointer(&mut self, value: LongPtr) {
        self.compatibility_copy_in(0x7c3, &Self::long_pointer_bytes(value));
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

    pub fn set_tileset_compr_palette_ptr(&mut self, value: LongPtr) {
        self.compatibility_copy_in(0x7c6, &Self::long_pointer_bytes(value));
    }

    /// layer2_scroll_x/y ($7E:091B/091C in ram_map).
    pub fn set_layer2_scroll_x_and_y(&mut self, x: u8, y: u8) {
        self.compatibility_copy_in(0x91b, &[x, y]);
    }

    /// samus_pose_x_dir / samus_movement_type ($7E:0A1E/0A1F).
    /// Preserve the original paired-byte word store.
    pub fn set_samus_pose_direction_and_movement_type(&mut self, direction: u8, movement_type: u8) {
        self.compatibility_copy_in(0xa1e, &[direction, movement_type]);
    }

    /// samus_movement_type ($7E:0A1F in ram_map).
    pub fn samus_movement_type(&self) -> u8 {
        self.compatibility_byte(0xa1f)
    }

    /// samus_prev_pose_x_dir / samus_prev_movement_type2 ($7E:0A22/0A23).
    /// Preserve the original paired-byte word store.
    pub fn set_samus_prev_pose_direction_and_movement_type(
        &mut self,
        direction: u8,
        movement_type: u8,
    ) {
        self.compatibility_copy_in(0xa22, &[direction, movement_type]);
    }

    /// samus_last_different_pose_x_dir / samus_last_different_pose_movement_type ($7E:0A26/0A27).
    /// Preserve the original paired-byte word store.
    pub fn set_samus_last_different_pose_direction_and_movement_type(
        &mut self,
        direction: u8,
        movement_type: u8,
    ) {
        self.compatibility_copy_in(0xa26, &[direction, movement_type]);
    }

    /// samus_last_different_pose_movement_type ($7E:0A27 in ram_map).
    pub fn samus_last_different_pose_movement_type(&self) -> u8 {
        self.compatibility_byte(0xa27)
    }

    /// samus_pose_x_dir ($7E:0A1E in ram_map); this is a byte, not a word.
    pub fn samus_pose_x_dir(&self) -> u8 {
        self.compatibility_byte(0xa1e)
    }

    /// Read a word of interactive_enemy_indexes ($7E:17EC in ram_map).
    /// Sentinel traversal uses backing RAM, including words beyond the list.
    pub fn interactive_enemy_index(&self, index: usize) -> u16 {
        let offset = 0x17ec + index * 2;
        u16::from_le_bytes([
            self.compatibility_byte(offset),
            self.compatibility_byte(offset + 1),
        ])
    }

    /// enemy_index_colliding_dirs ($7E:182C in ram_map).
    pub fn set_enemy_index_colliding_dir(&mut self, direction: u16, index: u16) {
        let offset = 0x182c + usize::from(direction & 3) * 2;
        self.compatibility_copy_in(offset, &index.to_le_bytes());
    }

    /// BTS behavior byte for the live collision block ($7F:6402 in ram_map).
    pub fn current_block_bts(&self) -> u8 {
        self.compatibility_byte(0x16402 + usize::from(self.cur_block_index()))
    }

    /// Live level-data word selected by the collision block index.
    pub fn current_level_block(&self) -> u16 {
        let offset = 0x10002 + usize::from(self.cur_block_index()) * 2;
        u16::from_le_bytes([
            self.compatibility_byte(offset),
            self.compatibility_byte(offset + 1),
        ])
    }

    /// Follow extension blocks until the selected reaction finishes.
    pub(crate) fn react_to_block(&mut self, mut react: impl FnMut(u16, &mut Game) -> u8) -> u8 {
        loop {
            // Callbacks can change the live block index before requesting another reaction.
            let block_type = self.current_level_block() >> 12;
            let result = react(block_type, self);
            if result & 0x80 == 0 {
                return result;
            }
        }
    }

    /// projectile_bomb_pre_instructions ($7E:0C68); slots 5..10 are bomb_functions.
    pub fn set_projectile_bomb_pre_instruction(&mut self, slot: usize, value: u16) {
        assert!(slot < 10);
        let offset = 0xc68 + slot * 2;
        self.compatibility_copy_in(offset, &value.to_le_bytes());
    }

    pub fn write_vram_queue_byte(&mut self, cursor: u16, field: usize, value: u8) {
        self.compatibility_write_owned_byte(0xd0 + usize::from(cursor) + field, value);
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
            sand_heap: None,
            ceres_rotation: None,
            shooting_stars: None,
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
            k_out: 0,
            g_sram: Default::default(),
            g_rtl_memory_ptr: core::ptr::null_mut(),
            g_rtl_runframe: None,
            g_rtl_syncall: None,
            state_recorder: StateRecorder::default(),
            apu_queue: Default::default(),
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
    pub(crate) fn prepare_session_start(
        &self,
        supplied: persistent_session::SessionStartDescription,
        media: Arc<SceneMedia>,
        required: &persistent_session::SessionStartRequirements,
    ) -> Result<persistent_session::PreparedSessionStart, persistent_session::SessionStartError>
    {
        self.with(|game| persistent_session::prepare(game, supplied, media, required))
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
    /// Transport a complete unpublished start to the actual app startup callback.
    pub fn supply_session_start(&self, supplied: persistent_session::SuppliedSessionStart) {
        self.with(|game| game.pending_supplied_start = Some(supplied));
    }
    pub fn start_prepared_session(
        &self,
        prepared: persistent_session::PreparedSessionStart,
    ) -> Result<(), persistent_session::SessionStartError> {
        self.with(|game| game.install_session_start(prepared))
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
        sources: &[(
            crate::game::room_entry::world::ResourceId,
            crate::rom::RomImage,
        )],
    ) -> Result<Vec<u8>, String> {
        self.with(|game| {
            game.checkpoint_session()
                .map_err(|error| format!("Session capture {error:?}"))?
                .encode_binary(
                    game.loaded_rom.as_ref(),
                    game.capture_native_rom_binding().as_ref(),
                    sources,
                )
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
        sources: &[(
            crate::game::room_entry::world::ResourceId,
            crate::rom::RomImage,
        )],
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
                game.loaded_rom.as_ref(),
                game.capture_native_rom_binding().as_ref(),
                sources,
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
            &mut crate::game::scene::start::InitialAudioIntent,
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
            == crate::game::scene::state::DisplayPhase::Producing;
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
    pub fn compatibility_copy_out(&self, at: usize, bytes: &mut [u8]) {
        let end = at.checked_add(bytes.len()).expect("RAM range overflow");
        let _ = &self.ram.0.bytes[at..end];
        for (offset, byte) in bytes.iter_mut().enumerate() {
            *byte = self.compatibility_byte(at + offset);
        }
    }
    pub fn compatibility_copy_within(&mut self, source: Range<usize>, destination: usize) {
        let count = self.ram.0.bytes[source.clone()].len();
        assert!(
            destination <= self.ram.len() - count,
            "dest is out of bounds"
        );
        if destination > source.start {
            for offset in (0..count).rev() {
                let byte = self.compatibility_byte(source.start + offset);
                self.compatibility_write_owned_byte(destination + offset, byte);
            }
        } else {
            for offset in 0..count {
                let byte = self.compatibility_byte(source.start + offset);
                self.compatibility_write_owned_byte(destination + offset, byte);
            }
        }
    }
    pub fn compatibility_fill(&mut self, range: Range<usize>, byte: u8) {
        let _ = &self.ram.0.bytes[range.clone()];
        for at in range {
            self.compatibility_write_owned_byte(at, byte);
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
    pub(crate) fn finish_session_interval(
        &mut self,
        metadata: persistent_session::IntervalMetadata,
        description: crate::game::scene::records::SceneDescription,
        media: Arc<SceneMedia>,
        effects: &mut impl crate::game::scene::commit::SettlementEffects,
    ) -> Result<(), DisplayError> {
        if self.persistent_ram_epoch != self.ram.write_epoch() {
            return Err(DisplayError::Poisoned);
        }
        self.preflight_samus_runtime()?;
        let persistent_session::SessionSlot::Ready(state) = &mut self.persistent_session else {
            return Err(DisplayError::Phase);
        };
        if let Err(error) = state.finish_interval(metadata, description, media, effects) {
            state.poison();
            self.samus_authority = SamusAuthority::Revoked;
            return Err(error);
        }
        crate::game::session_encoding::encode(&state.gameplay, |at, bytes| {
            self.ram.copy_in(at, bytes)
        });
        self.persistent_ram_epoch = self.ram.write_epoch();
        Ok(())
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

#[forbid(unsafe_code)]
impl Game {
    fn samus_rom(&self, registration: content::Registration) -> Option<&crate::rom::RomImage> {
        match registration {
            content::Registration::Loaded => self.loaded_rom.as_ref(),
            content::Registration::Native => self.native_rom.as_ref(),
        }
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
        if !Arc::ptr_eq(&binding.world, self.world())
            || binding.world.revision() != self.world().revision()
        {
            return Err(DisplayError::Identity);
        }
        for (&(registration, identity), admitted) in &binding.registrations {
            let image = match registration {
                content::Registration::Loaded => loaded,
                content::Registration::Native => native,
            }
            .as_ref()
            .ok_or(DisplayError::Identity)?;
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
                let image = match registration {
                    content::Registration::Loaded => loaded,
                    content::Registration::Native => native,
                }
                .as_ref()
                .ok_or(DisplayError::Identity)?;
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
            || supplied.world.revision() != expected.world.revision()
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
            request_media: supplied.request_media,
            registrations,
            execution: supplied.execution,
        })))
    }
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
        let mut mutation = self
            .begin_samus_mutation()
            .expect("Samus mutation rejected");
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
        if !Arc::ptr_eq(&payload.world, self.world())
            || payload.world.revision() != self.world().revision()
        {
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
        if !Arc::ptr_eq(&binding.world, self.world())
            || binding.world.revision() != self.world().revision()
        {
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
    pub(crate) fn prepared_samus_role(
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
    pub(crate) fn samus_role_base(
        &self,
        role: content::Role,
        registration: content::Registration,
    ) -> Result<Option<usize>, DisplayError> {
        if self.samus_execution()?.is_none() {
            return Ok(None);
        }
        self.samus_binding
            .as_ref()
            .unwrap()
            .roles
            .description()
            .roles
            .get(&(role, registration))
            .map(|declaration| Some(declaration.mapped_base))
            .ok_or(DisplayError::Incomplete)
    }
    pub(crate) fn samus_request(
        &self,
        half: crate::game::scene::samus_body::Half,
        pointer: u16,
    ) -> Result<Option<TileRequest>, DisplayError> {
        if self.samus_execution()?.is_none() {
            return Ok(None);
        }
        self.samus_binding
            .as_ref()
            .unwrap()
            .roles
            .request(half, pointer)
            .map(Some)
    }
    pub(crate) fn samus_spritemaps(&self) -> Result<Option<Arc<SamusCatalog>>, DisplayError> {
        if self.samus_execution()?.is_none() {
            return Ok(None);
        }
        Ok(Some(
            self.samus_binding.as_ref().unwrap().spritemaps.clone(),
        ))
    }
    pub(crate) fn samus_request_media(&self) -> Result<Option<Arc<SceneMedia>>, DisplayError> {
        if self.samus_execution()?.is_none() {
            return Ok(None);
        }
        Ok(Some(
            self.samus_binding.as_ref().unwrap().request_media.clone(),
        ))
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
    pub(crate) fn capture_actor_snapshot(
        &self,
    ) -> crate::game::actors::snapshot::ActorSnapshotPayload {
        crate::game::actors::snapshot::ActorSnapshotPayload::capture(
            self.actor_identity(),
            self.world(),
        )
    }
}
