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
use std::ffi::c_int;
use std::ops::Range;
use std::rc::Rc;
use std::sync::Arc;

pub(crate) use sm_game::scene::owned_samus::SamusExecution;
pub(crate) use sm_game::scene::samus_binding::{
    PreparedSamusBinding, SamusAuthority, SamusBinding, SamusBindingDescription, SamusImages,
    SamusOwner,
};
/// A branch observation, not a second live selector or a copy of mutable ROM bytes.
#[derive(Clone)]
pub(crate) struct SamusSnapshotPayload {
    world: Arc<WorldBinding>,
    binding: Option<Rc<SamusBinding>>,
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
        self.completed = self.game.samus.mutation_depth() > 1;
    }
}
impl Drop for SamusMutation<'_> {
    fn drop(&mut self) {
        self.game.samus.end_mutation();
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
pub struct WorkRam(
    Rc<RamBytes>,
    Rc<Cell<u64>>,
    Rc<Cell<u64>>,
    Rc<Cell<u64>>,
    Rc<Cell<u64>>,
    Rc<Cell<u64>>,
    Rc<Cell<u64>>,
);
#[forbid(unsafe_code)]
impl WorkRam {
    pub(crate) const LEN: usize = 0x20000;
    fn new() -> Self {
        Self(
            Rc::new(RamBytes {
                bytes: [const { Cell::new(0) }; Self::LEN],
            }),
            Rc::new(Cell::new(0)),
            Rc::new(Cell::new(0)),
            Rc::new(Cell::new(0)),
            Rc::new(Cell::new(0)),
            Rc::new(Cell::new(0)),
            Rc::new(Cell::new(0)),
        )
    }

    pub fn read_byte(&self, at: usize) -> u8 {
        self.0.bytes[at].get()
    }

    pub fn write_byte(&self, at: usize, value: u8) {
        let cell = &self.0.bytes[at];
        self.note_write();
        self.note_inventory_write(at..at + 1);
        self.note_menu_write(at..at + 1);
        self.note_scroll_write(at..at + 1);
        self.note_frame_display_write(at..at + 1);
        self.note_sequence_write(at..at + 1);
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
            self.note_inventory_write(destination..destination + count);
            self.note_menu_write(destination..destination + count);
            self.note_scroll_write(destination..destination + count);
            self.note_frame_display_write(destination..destination + count);
            self.note_sequence_write(destination..destination + count);
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
            self.note_inventory_write(at..end);
            self.note_menu_write(at..end);
            self.note_scroll_write(at..end);
            self.note_frame_display_write(at..end);
            self.note_sequence_write(at..end);
        }
        for (cell, &byte) in cells.iter().zip(input) {
            cell.set(byte);
        }
    }

    pub fn fill(&self, range: Range<usize>, value: u8) {
        let cells = &self.0.bytes[range.clone()];
        if !cells.is_empty() {
            self.note_write();
            self.note_inventory_write(range.clone());
            self.note_menu_write(range.clone());
            self.note_scroll_write(range.clone());
            self.note_frame_display_write(range.clone());
            self.note_sequence_write(range);
        }
        for cell in cells {
            cell.set(value);
        }
    }

    pub(crate) fn share_for_backend(&self) -> Self {
        Self(
            Rc::clone(&self.0),
            Rc::clone(&self.1),
            Rc::clone(&self.2),
            Rc::clone(&self.3),
            Rc::clone(&self.4),
            Rc::clone(&self.5),
            Rc::clone(&self.6),
        )
    }
    fn note_write(&self) {
        self.1.set(self.1.get().wrapping_add(1));
    }
    fn note_inventory_write(&self, range: Range<usize>) {
        if [
            0x9a2..0x9aa,
            0x9c0..0x9d2,
            0x9d4..0x9da,
            0xa4c..0xa4e,
            0xa76..0xa78,
        ]
        .iter()
        .any(|owned| range.start < owned.end && owned.start < range.end)
        {
            self.2.set(self.2.get().wrapping_add(1));
        }
    }
    fn note_menu_write(&self, range: Range<usize>) {
        if menu_ranges()
            .iter()
            .any(|owned| range.start < owned.end && owned.start < range.end)
        {
            self.3.set(self.3.get().wrapping_add(1));
        }
    }
    fn menu_epoch(&self) -> u64 {
        self.3.get()
    }
    fn note_scroll_write(&self, range: Range<usize>) {
        if range.start < 0xc1 && 0xb1 < range.end {
            self.4.set(self.4.get().wrapping_add(1));
        }
    }
    pub(crate) fn scroll_epoch(&self) -> u64 {
        self.4.get()
    }
    fn note_sequence_write(&self, range: Range<usize>) {
        if sequence_ranges()
            .iter()
            .any(|owned| range.start < owned.end && owned.start < range.end)
        {
            self.5.set(self.5.get().wrapping_add(1));
        }
    }
    fn note_frame_display_write(&self, range: Range<usize>) {
        if frame_display_ranges()
            .iter()
            .any(|owned| range.start < owned.end && owned.start < range.end)
        {
            self.6.set(self.6.get().wrapping_add(1));
        }
    }
    fn frame_display_epoch(&self) -> u64 {
        self.6.get()
    }
    pub(crate) fn sequence_epoch(&self) -> u64 {
        self.5.get()
    }
    fn inventory_epoch(&self) -> u64 {
        self.2.get()
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
    /// The native sound driver, once created.
    pub native_player: Option<crate::spc_player::NativePlayer>,
    actor_identity: Option<crate::game::actors::identity::ActorIdentityState>,
    pub(crate) actor_snapshots: sm_game::actors::snapshot::ActorSnapshotSidecars,
    /// The Session the frame's last lease exported; see `session_runtime::FrameSession`.
    pub(crate) frame_session: crate::game::session_runtime::FrameSession,
    world: Arc<WorldBinding>,
    ram: WorkRam,
    pub(crate) loaded_rom: Option<crate::rom::RomImage>,
    native_rom: Option<crate::rom::RomImage>,
    /// The oracle's cartridge ROM, which its compatibility patches rewrite;
    /// the engine's images are an unpatched copy, as hardware reads.
    pub(crate) cartridge_rom: Option<crate::snes::cart::CartRom>,
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
    samus: SamusOwner,
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

    pub g_sram: sm_game::save_file::SaveRam,
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
    /// The verifier's typed observation (lockstep or single-side), when enabled.
    #[cfg(sm_verify_dll)]
    pub observation: Option<Box<crate::observation::Observer>>,
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
            cartridge_rom,
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
            self.native_player.as_mut(),
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

    /// Create the native sound driver, replacing any earlier one.
    pub fn create_native_player(&mut self) {
        self.native_player = Some(crate::spc_player::NativePlayer {
            probe: crate::verify_probe::hit,
            ..crate::spc_player::NativePlayer::new()
        });
    }

    /// The native player; panics before `create_native_player`.
    pub fn native_player_mut(&mut self) -> &mut crate::spc_player::NativePlayer {
        self.native_player
            .as_mut()
            .expect("native audio requires the native player")
    }

    /// Initialize the player at the original caller's lifecycle phase.
    pub fn initialize_native_player(&mut self) {
        self.native_player_mut().initialize();
    }

    /// Export the driver's variables and optional queued ports into its RAM.
    pub fn save_native_audio_variables(&mut self, include_queued_commands: bool) {
        let queue = self.apu_queue;
        let player = self.native_player_mut();
        if include_queued_commands {
            for writes in queue.queued() {
                player.apply_ports(writes);
            }
        }
        player.save_state();
    }

    /// Import the actual current RAM image before resuming sequencing.
    pub fn load_native_audio_variables(&mut self) {
        self.native_player_mut().load_state();
    }

    /// Capture the player into separate snapshot storage; none leaves it invalid.
    pub fn capture_native_player_snapshot(
        &mut self,
        snapshot: &mut crate::audio_snapshot::PlayerSnapshot,
    ) {
        snapshot.capture_from(self.native_player.as_ref());
    }
    /// Restore a valid captured image into the player.
    pub fn restore_native_player_snapshot(&mut self, source: PlayerSnapshotSource<'_>) {
        let snapshot = match source {
            PlayerSnapshotSource::Before => &self.g_snapshot_before.player,
            PlayerSnapshotSource::Mine => &self.g_snapshot_mine.player,
            PlayerSnapshotSource::Theirs => &self.g_snapshot_theirs.player,
            PlayerSnapshotSource::Local(snapshot) => snapshot,
        };
        snapshot.restore_into(self.native_player.as_mut());
    }

    pub(crate) fn apply_native_audio_ports(&mut self, writes: [u8; 4]) {
        self.native_player_mut().apply_ports(writes);
    }

    pub(crate) fn render_native_audio(&mut self, samples: &mut [[i16; 2]]) {
        self.native_player_mut().render_samples(samples);
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
            crate::native_rom::NativeRomPointer::as_ptr(image)
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

    /// Whether the engine reads `original`, the unpatched cartridge payload, as
    /// hardware does: its loaded and native images are one allocation holding it.
    #[forbid(unsafe_code)]
    pub(crate) fn engine_reads_unpatched(&self, original: &[u8]) -> bool {
        match (&self.loaded_rom, &self.native_rom) {
            (Some(loaded), Some(native)) => {
                loaded.same_allocation(native) && loaded.bytes().starts_with(original)
            }
            _ => false,
        }
    }

    /// Whether the oracle cartridge, which its CPU executes, holds `original`'s
    /// `length` bytes from LoROM `address`.
    #[forbid(unsafe_code)]
    pub(crate) fn cartridge_matches(&self, original: &[u8], address: u32, length: usize) -> bool {
        let Some(cartridge) = self.cartridge_rom.as_ref() else {
            return false;
        };
        let start = crate::rtl::lorom_offset(address) as usize;
        start.checked_add(length).is_some_and(|end| {
            end <= cartridge.len()
                && original.get(start..end).is_some_and(|bytes| {
                    (start..end)
                        .zip(bytes)
                        .all(|(at, &b)| cartridge.read(at) == b)
                })
        })
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
        self.records.actor_tables = sm_game::actor_tables::import(image);
        self.records.palette_programs =
            sm_game::program_banks::import(image, sm_game::program_banks::Bank::Palette);
        self.records.animated_programs =
            sm_game::program_banks::import(image, sm_game::program_banks::Bank::Animated);
        self.records.sprite_programs =
            sm_game::program_banks::import(image, sm_game::program_banks::Bank::Sprite);
        self.records.gameplay_tables = sm_game::gameplay_tables::import(image);
        self.records.enemy_lists = sm_game::enemy_lists::import(image);
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
            native_player: None,
            world,
            actor_identity: Some(crate::game::actors::identity::ActorIdentityState::empty()),
            actor_snapshots: Default::default(),
            frame_session: Default::default(),
            ram: WorkRam::new(),
            loaded_rom: None,
            cartridge_rom: None,
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
            samus: SamusOwner::default(),
            persistent_session: persistent_session::SessionSlot::new(),
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
            #[cfg(sm_verify_dll)]
            observation: None,
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
            let current_identity = match &game.persistent_session.aggregate {
                persistent_session::AggregateSlot::Ready(state)
                | persistent_session::AggregateSlot::Failed(Some(state)) => {
                    &state.gameplay.actor_identity
                }
                persistent_session::AggregateSlot::Legacy => game
                    .actor_identity
                    .as_ref()
                    .ok_or("Session restore actor identity unavailable")?,
                persistent_session::AggregateSlot::Leased
                | persistent_session::AggregateSlot::Failed(None) => {
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
        use persistent_session::{AggregateSlot as SessionSlot, Control};
        if matches!(self.persistent_session.aggregate, SessionSlot::Legacy) {
            return Ok(None);
        }
        if self.persistent_ram_epoch != self.ram.write_epoch() {
            self.poison_persistent_session();
            return Err(DisplayError::Poisoned);
        }
        self.preflight_samus_runtime()?;
        let SessionSlot::Ready(state) = &mut self.persistent_session.aggregate else {
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
    /// The retained Session while it is current with RAM, for observation.
    #[cfg(sm_verify_dll)]
    pub(crate) fn observed_session(&self) -> Option<&sm_game::session::Session> {
        match &self.persistent_session.aggregate {
            persistent_session::AggregateSlot::Ready(state)
                if self.persistent_ram_epoch == self.ram.write_epoch()
                    && self.preflight_samus_runtime().is_ok() =>
            {
                Some(&state.gameplay)
            }
            _ => None,
        }
    }
    /// The HDMA object table as `compatibility_byte` projects it: the
    /// retained Session's while it is current, the RAM words otherwise.
    pub(crate) fn projected_hdma_objects(&self) -> sm_game::hdma::HdmaObjects {
        if let persistent_session::AggregateSlot::Ready(state) = &self.persistent_session.aggregate
        {
            if self.persistent_ram_epoch == self.ram.write_epoch()
                && self.preflight_samus_runtime().is_ok()
            {
                return state.gameplay.hdma;
            }
        }
        sm_game::hdma_objects::read_table(self)
    }
    pub fn compatibility_byte(&self, at: usize) -> u8 {
        if menu_ranges().iter().any(|range| range.contains(&at)) {
            return self.menu_byte(at);
        }
        if (0xb1..0xc1).contains(&at) {
            return self.scroll_byte(at);
        }
        if inventory_address(at) {
            return self.inventory_byte(at);
        }
        if let persistent_session::AggregateSlot::Ready(state) = &self.persistent_session.aggregate
        {
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
            self.persistent_session.aggregate,
            persistent_session::AggregateSlot::Ready(_)
        ) && (self.persistent_ram_epoch != self.ram.write_epoch()
            || self.preflight_samus_runtime().is_err()
            || persistent_session::Presentation::requires_semantic_counterpart(at))
        {
            self.poison_persistent_session();
        }
        if let persistent_session::AggregateSlot::Ready(state) =
            &mut self.persistent_session.aggregate
        {
            crate::game::session_encoding::write_projected_byte(&mut state.gameplay, at, byte);
            self.ram.write_byte(at, byte);
            state.observations = crate::game::session_encoding::inputs(&|lane| {
                crate::game::session_encoding::projected_byte(&state.gameplay, lane)
                    .unwrap_or_else(|| self.ram.read_byte(lane))
            });
            self.persistent_ram_epoch = self.ram.write_epoch();
            *self.persistent_session.menu.get_mut() = sm_game::menus::state::Domain::InSession {
                epoch: self.ram.menu_epoch(),
            };
            if matches!(
                self.persistent_session.inventory.get_mut(),
                sm_game::inventory_domain::InventoryDomain::InSession { .. }
            ) {
                *self.persistent_session.inventory.get_mut() =
                    sm_game::inventory_domain::InventoryDomain::InSession {
                        epoch: self.ram.inventory_epoch(),
                    };
            }
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
        use persistent_session::{AggregateSlot as SessionSlot, SessionStartError};
        if matches!(self.persistent_session.aggregate, SessionSlot::Leased)
            || (!scoped_adoption && self.samus.mutation_depth() != 0)
        {
            return Err(SessionStartError::Phase);
        }
        prepared
            .binding
            .binding()
            .check(self.samus_images())
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
        self.samus.install(binding);
        self.actor_identity = None;
        self.persistent_ram_epoch = self.ram.write_epoch();
        self.persistent_session.aggregate = SessionSlot::Ready(Box::new(session));
        *self.persistent_session.inventory.get_mut() =
            sm_game::inventory_domain::InventoryDomain::InSession {
                epoch: self.ram.inventory_epoch(),
            };
        *self.persistent_session.menu.get_mut() = sm_game::menus::state::Domain::InSession {
            epoch: self.ram.menu_epoch(),
        };
        self.adopt_scroll_image();
        self.adopt_frame_display_image();
        self.adopt_sequence_image();
        Ok(())
    }
    /// Whether `Runtime::new` would succeed now, without changing any state:
    /// the Samus runtime admits import, and any retained persistent Session
    /// is current with RAM.
    pub(crate) fn session_lease_admissible(&self) -> bool {
        use persistent_session::AggregateSlot as SessionSlot;
        self.preflight_samus_runtime().is_ok()
            && match &self.persistent_session.aggregate {
                SessionSlot::Legacy => true,
                SessionSlot::Ready(_) => self.persistent_ram_epoch == self.ram.write_epoch(),
                SessionSlot::Leased | SessionSlot::Failed(_) => false,
            }
    }
    /// Whether a retained persistent Session is installed and current.
    pub(crate) fn persistent_session_current(&self) -> bool {
        matches!(
            self.persistent_session.aggregate,
            persistent_session::AggregateSlot::Ready(_)
        ) && self.session_lease_admissible()
    }
    pub(crate) fn lease_persistent_session(
        &mut self,
    ) -> Result<Option<persistent_session::PersistentSession>, DisplayError> {
        use persistent_session::AggregateSlot as SessionSlot;
        if matches!(self.persistent_session.aggregate, SessionSlot::Legacy) {
            return Ok(None);
        }
        self.preflight_samus_runtime()?;
        if self.persistent_ram_epoch != self.ram.write_epoch() {
            self.poison_persistent_session();
            return Err(DisplayError::Poisoned);
        }
        match std::mem::replace(&mut self.persistent_session.aggregate, SessionSlot::Leased) {
            SessionSlot::Ready(session) => {
                *self.persistent_session.inventory.get_mut() =
                    sm_game::inventory_domain::InventoryDomain::Leased;
                *self.persistent_session.menu.get_mut() = sm_game::menus::state::Domain::Leased;
                Ok(Some(*session))
            }
            other => {
                self.persistent_session.aggregate = other;
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
        use persistent_session::AggregateSlot as SessionSlot;
        assert!(
            matches!(self.persistent_session.aggregate, SessionSlot::Leased),
            "Session lease owner"
        );
        self.persistent_session.aggregate = if completed && admitted_epoch == self.ram.write_epoch()
        {
            self.persistent_ram_epoch = admitted_epoch;
            SessionSlot::Ready(Box::new(session))
        } else {
            self.samus.force_revoke();
            session.poison();
            SessionSlot::Failed(Some(Box::new(session)))
        };
        *self.persistent_session.inventory.get_mut() =
            if completed && admitted_epoch == self.ram.write_epoch() {
                sm_game::inventory_domain::InventoryDomain::InSession {
                    epoch: self.ram.inventory_epoch(),
                }
            } else {
                sm_game::inventory_domain::InventoryDomain::Revoked
            };
        *self.persistent_session.menu.get_mut() =
            if completed && admitted_epoch == self.ram.write_epoch() {
                sm_game::menus::state::Domain::InSession {
                    epoch: self.ram.menu_epoch(),
                }
            } else {
                sm_game::menus::state::Domain::Revoked
            };
        if completed && admitted_epoch == self.ram.write_epoch() {
            self.adopt_scroll_image();
            self.adopt_frame_display_image();
            self.adopt_sequence_image();
        } else {
            self.revoke_scroll_domain();
            self.revoke_frame_display_domain();
            self.revoke_sequence_domain();
        }
    }
    fn poison_persistent_session(&mut self) {
        // Explicit aggregate failure also ends the detached FrameDisplay interval.
        self.revoke_frame_display_domain();
        use persistent_session::AggregateSlot as SessionSlot;
        match std::mem::replace(
            &mut self.persistent_session.aggregate,
            SessionSlot::Failed(None),
        ) {
            SessionSlot::Ready(mut state) | SessionSlot::Failed(Some(mut state)) => {
                state.poison();
                self.persistent_session.aggregate = SessionSlot::Failed(Some(state));
            }
            other => self.persistent_session.aggregate = other,
        }
    }
    pub(crate) fn checkpoint_session(
        &self,
    ) -> Result<persistent_session::SessionCheckpoint, DisplayError> {
        if self.persistent_ram_epoch != self.ram.write_epoch() {
            return Err(DisplayError::Poisoned);
        }
        self.preflight_samus_runtime()?;
        match &self.persistent_session.aggregate {
            persistent_session::AggregateSlot::Ready(session) => session.checkpoint(),
            _ => Err(DisplayError::Phase),
        }
    }
    pub(crate) fn capture_session_snapshot(
        &self,
    ) -> Result<persistent_session::SessionSnapshotPayload, DisplayError> {
        match &self.persistent_session.aggregate {
            persistent_session::AggregateSlot::Legacy => {
                Ok(persistent_session::SessionSnapshotPayload(None))
            }
            _ => self
                .checkpoint_session()
                .map(|state| persistent_session::SessionSnapshotPayload(Some(state))),
        }
    }
    pub(crate) fn begin_session_interval(&mut self, fresh_entry: bool) -> Result<(), DisplayError> {
        if matches!(
            self.persistent_session.aggregate,
            persistent_session::AggregateSlot::Legacy
        ) {
            self.adopt_inventory_image();
            self.adopt_menu_image();
            self.adopt_scroll_image();
            self.adopt_frame_display_image();
            self.adopt_sequence_image();
            return Ok(());
        }
        if self.persistent_ram_epoch != self.ram.write_epoch() {
            return Err(DisplayError::Poisoned);
        }
        self.preflight_samus_runtime()?;
        let result = match &mut self.persistent_session.aggregate {
            persistent_session::AggregateSlot::Legacy => Ok(()),
            persistent_session::AggregateSlot::Ready(state) => state.begin_interval(fresh_entry),
            _ => Err(DisplayError::Phase),
        };
        if result.is_ok() {
            self.adopt_inventory_image();
            self.adopt_menu_image();
            self.adopt_scroll_image();
            self.adopt_frame_display_image();
            self.adopt_sequence_image();
        }
        result
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
            self.persistent_session.aggregate,
            persistent_session::AggregateSlot::Legacy
        ) {
            return Err(persistent_session::SessionStartError::Binding(
                DisplayError::Incomplete,
            ));
        }
        self.adopt_inventory_image();
        self.adopt_menu_image();
        self.adopt_scroll_image();
        self.adopt_frame_display_image();
        self.adopt_sequence_image();
        Ok(())
    }
    pub(crate) fn revoke_session_authority(&mut self) {
        self.revoke_inventory_domain();
        self.revoke_menu_domain();
        self.revoke_scroll_domain();
        self.revoke_frame_display_domain();
        self.revoke_sequence_domain();
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
        if let persistent_session::AggregateSlot::Ready(session) =
            &self.persistent_session.aggregate
        {
            return &session.gameplay.actor_identity;
        }
        self.actor_identity
            .as_ref()
            .expect("actor identity currently owned by Session")
    }
    pub(crate) fn actor_identity_mut(
        &mut self,
    ) -> &mut crate::game::actors::identity::ActorIdentityState {
        if let persistent_session::AggregateSlot::Ready(session) =
            &mut self.persistent_session.aggregate
        {
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
    /// The world and ROM images Samus's registrations read.
    fn samus_images(&self) -> SamusImages<'_> {
        SamusImages {
            world: &self.world,
            loaded: &self.loaded_rom,
            native: &self.native_rom,
        }
    }
    pub(crate) fn prepare_samus_binding(
        &self,
        supplied: SamusBindingDescription,
        expected: &SamusBindingDescription,
    ) -> Result<PreparedSamusBinding, DisplayError> {
        SamusBinding::prepare(supplied, expected, self.samus_images())
    }
    #[cfg(sm_verify_dll)]
    pub(crate) fn install_samus_binding(
        &mut self,
        prepared: PreparedSamusBinding,
    ) -> Result<(), DisplayError> {
        if self.actor_identity_is_with_session() {
            return Err(DisplayError::Phase);
        }
        prepared.binding().check(self.samus_images())?;
        self.samus.install(prepared);
        Ok(())
    }
    pub(crate) fn samus_authority(&self) -> SamusAuthority {
        self.samus.authority(self.samus_images())
    }
    pub(crate) fn revoke_samus_authority(&mut self) {
        self.revoke_inventory_domain();
        self.revoke_menu_domain();
        self.revoke_scroll_domain();
        self.revoke_frame_display_domain();
        self.revoke_sequence_domain();
        self.poison_persistent_session();
        self.samus.revoke();
    }
    /// Unscoped backend/ROM writers invalidate resumption. Scoped writers remain
    /// revoked until their guard successfully adopts the actual selected state.
    pub(crate) fn note_samus_external_mutation(&mut self) {
        if self.samus.mutation_depth() == 0 {
            self.revoke_samus_authority();
        }
    }
    /// [`Self::begin_samus_mutation`] where a rejection is a broken invariant.
    pub(crate) fn samus_mutation(&mut self) -> SamusMutation<'_> {
        self.begin_samus_mutation()
            .expect("Samus mutation rejected")
    }
    pub(crate) fn begin_samus_mutation(&mut self) -> Result<SamusMutation<'_>, DisplayError> {
        // Native compatibility remains available for the legacy/reference path.
        let legacy = matches!(
            self.persistent_session.aggregate,
            persistent_session::AggregateSlot::Legacy
        );
        self.samus.begin_mutation(legacy)?;
        self.revoke_inventory_domain();
        self.revoke_menu_domain();
        self.revoke_scroll_domain();
        self.revoke_frame_display_domain();
        self.revoke_sequence_domain();
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
        self.samus.preflight(self.samus_images())
    }
    pub(crate) fn capture_samus_snapshot(&self) -> SamusSnapshotPayload {
        SamusSnapshotPayload {
            world: self.world.clone(),
            binding: self.samus.binding().cloned(),
            loaded: self.loaded_rom.clone(),
            native: self.native_rom.clone(),
            authority: self.samus_authority(),
            admitted: self.samus.admitted(),
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
            binding.check(SamusImages {
                world: &self.world,
                loaded: &payload.loaded,
                native: &payload.native,
            })?;
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
        self.samus.adopt(payload.binding);
        self.adopt_inventory_image();
        self.adopt_menu_image();
        self.adopt_scroll_image();
        self.adopt_frame_display_image();
        self.adopt_sequence_image();
        Ok(())
    }
    pub(crate) fn samus_execution(&self) -> Result<Option<SamusExecution>, DisplayError> {
        self.samus.execution(self.samus_images())
    }
    #[cfg(sm_verify_dll)]
    pub(crate) fn samus_request_media(&self) -> Result<Option<Arc<SceneMedia>>, DisplayError> {
        self.samus.request_media(self.samus_images())
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
    fn samus_execution(&self) -> Result<Option<SamusExecution>, DisplayError> {
        Game::samus_execution(self)
    }
    fn capture_samus_role(
        &self,
        role: content::Role,
        registration: content::Registration,
        displacement: usize,
        selected: &crate::rom::RomImage,
    ) -> Result<Option<content::RoleLease<native_content::CapturedRom>>, DisplayError> {
        self.samus.capture_role(
            self.samus_images(),
            role,
            registration,
            displacement,
            selected,
        )
    }
    fn prepared_samus_role(
        &self,
        role: content::Role,
        registration: content::Registration,
        displacement: usize,
    ) -> Result<Option<content::RoleLease<content::PreparedBytes>>, DisplayError> {
        self.samus
            .prepared_role(self.samus_images(), role, registration, displacement)
    }
    fn samus_role_base(
        &self,
        role: content::Role,
        registration: content::Registration,
    ) -> Result<Option<usize>, DisplayError> {
        self.samus
            .role_base(self.samus_images(), role, registration)
    }
    fn samus_request(
        &self,
        half: sm_game::scene::samus_body::Half,
        pointer: u16,
    ) -> Result<Option<TileRequest>, DisplayError> {
        self.samus.request(self.samus_images(), half, pointer)
    }
    fn samus_spritemaps(&self) -> Result<Option<Arc<SamusCatalog>>, DisplayError> {
        self.samus.spritemaps(self.samus_images())
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
    fn native_image_binding(&self) -> Option<crate::rom::RomImage> {
        self.capture_native_rom_binding()
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

fn inventory_address(at: usize) -> bool {
    [
        0x9a2..0x9aa,
        0x9c0..0x9d2,
        0x9d4..0x9da,
        0xa4c..0xa4e,
        0xa76..0xa78,
    ]
    .iter()
    .any(|range| range.contains(&at))
}

#[forbid(unsafe_code)]
impl Game {
    fn inventory_byte(&self, at: usize) -> u8 {
        use sm_game::inventory_domain::InventoryDomain;
        let domain = self.persistent_session.inventory.borrow();
        let inventory = match &*domain {
            InventoryDomain::Owned { inventory, epoch } if *epoch == self.ram.inventory_epoch() => {
                Some(inventory)
            }
            InventoryDomain::InSession { epoch } if *epoch == self.ram.inventory_epoch() => {
                match &self.persistent_session.aggregate {
                    persistent_session::AggregateSlot::Ready(state)
                    | persistent_session::AggregateSlot::Failed(Some(state)) => {
                        Some(&state.gameplay.inventory)
                    }
                    _ => None,
                }
            }
            _ => None,
        };
        inventory.map_or_else(
            || self.ram.read_byte(at),
            |inventory| {
                // The adapter owns addresses; the domain core owns ordinary fields.
                let word = sm_game::inventory::Word::from_address(at & !1).expect("inventory word");
                inventory.word(word).to_le_bytes()[at & 1]
            },
        )
    }
    pub(crate) fn revoke_inventory_domain(&mut self) {
        *self.persistent_session.inventory.get_mut() =
            sm_game::inventory_domain::InventoryDomain::Revoked;
    }
    pub(crate) fn adopt_inventory_image(&mut self) {
        use sm_game::inventory_domain::InventoryDomain;
        if matches!(
            self.persistent_session.inventory.get_mut(),
            InventoryDomain::Leased
        ) {
            // The child Runtime already imported the actual selected image.
            return;
        }
        let epoch = self.ram.inventory_epoch();
        if matches!(
            self.persistent_session.inventory.get_mut(),
            InventoryDomain::Owned { epoch: current, .. }
                | InventoryDomain::InSession { epoch: current }
                if *current == epoch
        ) {
            return;
        }
        let inventory = sm_game::inventory::decode(&|at| self.ram.read_byte(at));
        match &mut self.persistent_session.aggregate {
            persistent_session::AggregateSlot::Ready(state)
            | persistent_session::AggregateSlot::Failed(Some(state)) => {
                state.gameplay.inventory = inventory;
                *self.persistent_session.inventory.get_mut() = InventoryDomain::InSession { epoch };
            }
            _ => {
                *self.persistent_session.inventory.get_mut() =
                    InventoryDomain::Owned { inventory, epoch }
            }
        }
    }
    pub(crate) fn take_inventory_domain(&mut self) -> Option<sm_game::inventory::Inventory> {
        use sm_game::inventory_domain::InventoryDomain;
        let domain = std::mem::replace(
            self.persistent_session.inventory.get_mut(),
            InventoryDomain::Leased,
        );
        match domain {
            InventoryDomain::Owned { inventory, epoch } if epoch == self.ram.inventory_epoch() => {
                Some(inventory)
            }
            _ => None,
        }
    }
    pub(crate) fn return_inventory_domain(
        &mut self,
        inventory: sm_game::inventory::Inventory,
        completed: bool,
    ) {
        use sm_game::inventory_domain::InventoryDomain;
        *self.persistent_session.inventory.get_mut() = if completed {
            InventoryDomain::Owned {
                inventory,
                epoch: self.ram.inventory_epoch(),
            }
        } else {
            InventoryDomain::Revoked
        };
    }
}

#[forbid(unsafe_code)]
impl sm_game::inventory_domain::InventoryAccess for Game {
    fn reserve_state(&self) -> sm_game::inventory_domain::ReserveState {
        use sm_game::inventory_domain::{InventoryDomain, ReserveState};
        let mut domain = self.persistent_session.inventory.borrow_mut();
        // First domain entry establishes only inventory, never an incomplete Session.
        // Dirty/revoked owners remain byte-authoritative until scoped adoption.
        if matches!(*domain, InventoryDomain::Unowned) {
            *domain = InventoryDomain::Owned {
                inventory: sm_game::inventory::decode(&|at| self.ram.read_byte(at)),
                epoch: self.ram.inventory_epoch(),
            };
        }
        match &*domain {
            InventoryDomain::Owned { inventory, epoch } if *epoch == self.ram.inventory_epoch() => {
                ReserveState::from(inventory)
            }
            InventoryDomain::InSession { epoch } if *epoch == self.ram.inventory_epoch() => {
                match &self.persistent_session.aggregate {
                    persistent_session::AggregateSlot::Ready(state)
                    | persistent_session::AggregateSlot::Failed(Some(state)) => {
                        ReserveState::from(&state.gameplay.inventory)
                    }
                    _ => panic!("inventory resident outside Session"),
                }
            }
            _ => ReserveState::from(&sm_game::inventory::decode(&|at| self.ram.read_byte(at))),
        }
    }
    fn commit_reserve(&mut self, change: sm_game::inventory_domain::ReserveChange) {
        if change.is_empty() {
            return;
        }
        if self.persistent_session_current() {
            // Exactly the former changed cohort store's aggregate admission seam.
            use sm_game::session_access::SessionAccess as _;
            let mut runtime = crate::game::session_runtime::Runtime::new(self);
            change.apply(&mut runtime.session_mut().inventory);
            runtime.refresh_inputs();
            return;
        }
        let inventory = self.take_inventory_domain();
        // Keep the owner unavailable across physical publication, including unwind.
        struct Publish<'a> {
            game: &'a mut Game,
            inventory: Option<sm_game::inventory::Inventory>,
            completed: bool,
        }
        impl Drop for Publish<'_> {
            fn drop(&mut self) {
                if let Some(inventory) = self.inventory.take() {
                    self.game.return_inventory_domain(inventory, self.completed);
                } else {
                    self.game.revoke_inventory_domain();
                }
            }
        }
        let mut publish = Publish {
            game: self,
            inventory,
            completed: false,
        };
        if let Some(inventory) = &mut publish.inventory {
            change.apply(inventory);
        }
        for (at, value) in [
            (0x9c0, change.mode),
            (0x9c2, change.health),
            (0x9d6, change.reserve),
        ] {
            if let Some(value) = value {
                let [low, high] = value.to_le_bytes();
                publish.game.compatibility_write_byte(at, low);
                publish.game.compatibility_write_byte(at + 1, high);
            }
        }
        publish.completed = true;
    }
}

// Exact UI encoding lanes; all aliases, bulk stores and CPU writes advance this epoch.
fn menu_ranges() -> [Range<usize>; 10] {
    [
        0x727..0x765,
        0x5ac..0x5b4,
        0x5fb..0x601,
        0x776..0x77c,
        0x787..0x78d,
        0x950..0x956,
        0xd914..0xd91a,
        0x99e..0x9a0,
        0xde0..0xde4,
        0xde4..0xdea,
    ]
}

#[forbid(unsafe_code)]
impl Game {
    fn read_menu_image(&self) -> sm_game::menus::state::State {
        // Scoped selected-image adoption is infallible and admits no Samus/actor lease.
        // This decoder is a retained adapter, never an engine read implementation.
        use sm_game::session_cohort::Cohort;
        fn decode<C: Cohort>(game: &Game) -> C {
            let mut value = C::default();
            value.words(|at, word| {
                *word = u16::from_le_bytes([game.ram.read_byte(at), game.ram.read_byte(at + 1)])
            });
            value
        }
        sm_game::menus::state::State {
            menu: decode(self),
            screens: decode(self),
        }
    }
    fn menu_byte(&self, at: usize) -> u8 {
        use sm_game::menus::state::Access;
        use sm_game::session_cohort::Cohort;
        let state = self.menu_state();
        let mut value = (state.menu, state.screens);
        let mut byte = None;
        value.words(|start, word| {
            if at == start || at == start + 1 {
                byte = Some(word.to_le_bytes()[at - start]);
            }
        });
        byte.expect("menu encoding lane")
    }
    pub(crate) fn revoke_menu_domain(&mut self) {
        *self.persistent_session.menu.get_mut() = sm_game::menus::state::Domain::Revoked;
    }
    pub(crate) fn adopt_menu_image(&mut self) {
        use sm_game::menus::state::Domain;
        if matches!(self.persistent_session.menu.get_mut(), Domain::Leased) {
            return;
        }
        let epoch = self.ram.menu_epoch();
        if matches!(self.persistent_session.menu.get_mut(),
            Domain::Owned { epoch: current, .. } | Domain::InSession { epoch: current }
            if *current == epoch)
        {
            return;
        }
        let state = self.read_menu_image();
        match &mut self.persistent_session.aggregate {
            persistent_session::AggregateSlot::Ready(session)
            | persistent_session::AggregateSlot::Failed(Some(session)) => {
                session.gameplay.menu = state.menu;
                session.gameplay.menu_screens = state.screens;
                *self.persistent_session.menu.get_mut() = Domain::InSession { epoch };
            }
            _ => *self.persistent_session.menu.get_mut() = Domain::Owned { state, epoch },
        }
    }
    pub(crate) fn take_menu_domain(&mut self) -> Option<sm_game::menus::state::State> {
        use sm_game::menus::state::Domain;
        let domain = std::mem::replace(self.persistent_session.menu.get_mut(), Domain::Leased);
        match domain {
            Domain::Owned { state, epoch } if epoch == self.ram.menu_epoch() => Some(state),
            _ => None,
        }
    }
    pub(crate) fn return_menu_domain(
        &mut self,
        state: sm_game::menus::state::State,
        completed: bool,
    ) {
        use sm_game::menus::state::Domain;
        *self.persistent_session.menu.get_mut() = if completed {
            Domain::Owned {
                state,
                epoch: self.ram.menu_epoch(),
            }
        } else {
            Domain::Revoked
        };
    }
}

#[forbid(unsafe_code)]
impl sm_game::menus::state::Access for Game {
    fn reset_pause_menu(&mut self) {
        let mut state = self.menu_state();
        state.clear_pause_menu();
        struct Clear<'a> {
            game: &'a mut Game,
            state: sm_game::menus::state::State,
            completed: bool,
        }
        impl Drop for Clear<'_> {
            fn drop(&mut self) {
                self.game.return_menu_domain(self.state, self.completed);
            }
        }
        *self.persistent_session.menu.get_mut() = sm_game::menus::state::Domain::Leased;
        let mut clear = Clear {
            game: self,
            state,
            completed: false,
        };
        // The old three ascending LE transfers, including zero-to-zero stores.
        // Keep physical publication here so a subsequent player-command failure
        // retains exactly the original clear prefix and admission ordering.
        for (start, words) in [(0x727, 10), (0x73d, 9), (0x751, 9)] {
            for index in 0..words {
                let at = start + 2 * index;
                clear.game.compatibility_write_byte(at, 0);
                clear.game.compatibility_write_byte(at + 1, 0);
            }
        }
        clear.completed = true;
    }
    fn menu_state(&self) -> sm_game::menus::state::State {
        use sm_game::menus::state::{Domain, State};
        let epoch = self.ram.menu_epoch();
        let mut domain = self.persistent_session.menu.borrow_mut();
        match &*domain {
            Domain::Owned {
                state,
                epoch: current,
            } if *current == epoch => return *state,
            Domain::InSession { epoch: current } if *current == epoch => {
                if let persistent_session::AggregateSlot::Ready(session)
                | persistent_session::AggregateSlot::Failed(Some(session)) =
                    &self.persistent_session.aggregate
                {
                    return State {
                        menu: session.gameplay.menu,
                        screens: session.gameplay.menu_screens,
                    };
                }
            }
            Domain::Leased => return self.read_menu_image(),
            _ => {}
        }
        // An external writer revoked the old authority. Rebuild once from the actual
        // selected bytes, without changed-store admission or unrelated preflight.
        let state = self.read_menu_image();
        *domain = Domain::Owned { state, epoch };
        state
    }
    fn commit_menu<C: sm_game::menus::state::Value>(&mut self, before: &C, after: C) {
        use sm_game::menus::state::State;
        use sm_game::session_cohort::Cohort;
        if after == *before {
            return;
        }
        if self.persistent_session_current() {
            // Preserve precisely the former changed-copy store's lease admission.
            use sm_game::session_access::SessionAccess;
            let mut runtime = crate::game::session_runtime::Runtime::new(self);
            let session = runtime.session_mut();
            let mut state = State {
                menu: session.menu,
                screens: session.menu_screens,
            };
            C::merge(before, &after, &mut state);
            session.menu = state.menu;
            session.menu_screens = state.screens;
            runtime.refresh_inputs();
            return;
        }
        let previous = self.menu_state();
        let mut state = previous;
        C::merge(before, &after, &mut state);
        // Select physical stores from the draft's original before/after values,
        // even if a native alias already wrote the same result during the call.
        // Unchanged draft fields preserve the completed foreign byte prefix.
        let mut original = previous;
        C::replace(before, &mut original);
        let mut changed = original;
        C::merge(before, &after, &mut changed);
        let mut old = (original.menu, original.screens);
        let mut words = Vec::new();
        old.words(|at, word| words.push((at, *word)));
        let mut new = (changed.menu, changed.screens);
        let mut changes = Vec::new();
        let mut index = 0;
        new.words(|at, word| {
            if words[index].1 != *word {
                changes.push((at, *word));
            }
            index += 1;
        });
        struct Publish<'a> {
            game: &'a mut Game,
            state: State,
            completed: bool,
        }
        impl Drop for Publish<'_> {
            fn drop(&mut self) {
                self.game.return_menu_domain(self.state, self.completed);
            }
        }
        *self.persistent_session.menu.get_mut() = sm_game::menus::state::Domain::Leased;
        let mut publish = Publish {
            game: self,
            state,
            completed: false,
        };
        for (at, word) in changes {
            let [low, high] = word.to_le_bytes();
            publish.game.compatibility_write_byte(at, low);
            publish.game.compatibility_write_byte(at + 1, high);
        }
        publish.completed = true;
    }
}

use sm_game::display_scroll::Access as _;

fn scroll_address(word: sm_game::display_scroll::Word) -> usize {
    0xb1 + 2 * usize::from(word)
}
fn scroll_lane(at: usize) -> (sm_game::display_scroll::Word, usize) {
    let offset = at - 0xb1;
    (sm_game::display_scroll::WORDS[offset / 2], offset % 2)
}

#[forbid(unsafe_code)]
impl Game {
    pub(crate) fn revoke_scroll_domain(&mut self) {
        *self.persistent_session.scroll.get_mut() = sm_game::display_scroll::ScrollDomain::Revoked;
    }
    pub(crate) fn adopt_scroll_image(&mut self) {
        if self.persistent_session_current() {
            if let persistent_session::AggregateSlot::Ready(state) =
                &self.persistent_session.aggregate
            {
                if state.gameplay.presentation.is_owned() {
                    *self.persistent_session.scroll.get_mut() =
                        sm_game::display_scroll::ScrollDomain::InSession {
                            epoch: self.ram().scroll_epoch(),
                        };
                    return;
                }
            }
        }
        let epoch = self.ram().scroll_epoch();
        if matches!(self.persistent_session.scroll.get_mut(),
            sm_game::display_scroll::ScrollDomain::Owned { epoch: current, .. } if *current == epoch)
        {
            return;
        }
        let scroll = sm_game::display_scroll::WORDS.map(|word| {
            let at = scroll_address(word);
            u16::from_le_bytes([self.ram().read_byte(at), self.ram().read_byte(at + 1)])
        });
        *self.persistent_session.scroll.get_mut() =
            sm_game::display_scroll::ScrollDomain::Owned { scroll, epoch };
    }
    pub(crate) fn scroll_byte(&self, at: usize) -> u8 {
        let (word, offset) = scroll_lane(at);
        self.scroll_word(word).to_le_bytes()[offset]
    }
    pub(crate) fn set_scroll_byte(&mut self, at: usize, byte: u8) {
        let (word, offset) = scroll_lane(at);
        let mut bytes = self.scroll_word(word).to_le_bytes();
        bytes[offset] = byte;
        self.publish_scroll(word, u16::from_le_bytes(bytes), Some(offset));
    }
    fn publish_scroll(
        &mut self,
        word: sm_game::display_scroll::Word,
        value: u16,
        byte: Option<usize>,
    ) {
        if self.persistent_session_current() {
            if let persistent_session::AggregateSlot::Ready(state) =
                &mut self.persistent_session.aggregate
            {
                if let Ok(display) = state.gameplay.presentation.owner_mut() {
                    display.state.controls.scroll_words[usize::from(word)] = value;
                    for (offset, byte_value) in value.to_le_bytes().into_iter().enumerate() {
                        if byte.is_none_or(|selected| selected == offset) {
                            self.compatibility_write_owned_byte(
                                scroll_address(word) + offset,
                                byte_value,
                            );
                        }
                    }
                    *self.persistent_session.scroll.get_mut() =
                        sm_game::display_scroll::ScrollDomain::InSession {
                            epoch: self.ram().scroll_epoch(),
                        };
                    return;
                }
            }
        }
        // First entry establishes the complete actual image, never a Display.
        if matches!(
            *self.persistent_session.scroll.borrow(),
            sm_game::display_scroll::ScrollDomain::Unowned
        ) {
            self.adopt_scroll_image();
        }
        let slot = std::mem::replace(
            self.persistent_session.scroll.get_mut(),
            sm_game::display_scroll::ScrollDomain::Leased,
        );
        let mut scroll = match slot {
            sm_game::display_scroll::ScrollDomain::Owned { scroll, epoch }
                if epoch == self.ram().scroll_epoch() =>
            {
                Some(scroll)
            }
            _ => None,
        };
        if let Some(scroll) = &mut scroll {
            scroll[usize::from(word)] = value;
        }
        struct Publish<'a> {
            game: &'a mut Game,
            scroll: Option<[u16; 8]>,
            completed: bool,
        }
        impl Drop for Publish<'_> {
            fn drop(&mut self) {
                *self.game.persistent_session.scroll.get_mut() = match self.scroll.take() {
                    Some(scroll) if self.completed => {
                        sm_game::display_scroll::ScrollDomain::Owned {
                            scroll,
                            epoch: self.game.ram().scroll_epoch(),
                        }
                    }
                    _ => sm_game::display_scroll::ScrollDomain::Revoked,
                };
            }
        }
        let mut publish = Publish {
            game: self,
            scroll,
            completed: false,
        };
        for (offset, byte_value) in value.to_le_bytes().into_iter().enumerate() {
            if byte.is_none_or(|selected| selected == offset) {
                publish
                    .game
                    .compatibility_write_byte(scroll_address(word) + offset, byte_value);
            }
        }
        publish.completed = true;
    }
}

#[forbid(unsafe_code)]
impl sm_game::display_scroll::Access for Game {
    fn scroll_word(&self, word: sm_game::display_scroll::Word) -> u16 {
        if self.persistent_session_current() {
            if let persistent_session::AggregateSlot::Ready(state) =
                &self.persistent_session.aggregate
            {
                if let Ok(display) = state.gameplay.presentation.owner() {
                    return display.state.controls.scroll_words[usize::from(word)];
                }
            }
        }
        let mut domain = self.persistent_session.scroll.borrow_mut();
        if matches!(*domain, sm_game::display_scroll::ScrollDomain::Unowned) {
            let scroll = sm_game::display_scroll::WORDS.map(|word| {
                let at = scroll_address(word);
                u16::from_le_bytes([self.ram().read_byte(at), self.ram().read_byte(at + 1)])
            });
            *domain = sm_game::display_scroll::ScrollDomain::Owned {
                scroll,
                epoch: self.ram().scroll_epoch(),
            };
        }
        match &*domain {
            sm_game::display_scroll::ScrollDomain::Owned { scroll, epoch }
                if *epoch == self.ram().scroll_epoch() =>
            {
                scroll[usize::from(word)]
            }
            _ => {
                // Revoked, dirty or suspended ownership exposes selected bytes.
                let at = scroll_address(word);
                u16::from_le_bytes([self.ram().read_byte(at), self.ram().read_byte(at + 1)])
            }
        }
    }
    fn actor_scroll_word(&self, word: sm_game::display_scroll::Word) -> u16 {
        // The old boss view called Game::slot_byte, which samples raw RAM.
        u16::from_le_bytes([
            self.physical_scroll_byte(word, false),
            self.physical_scroll_byte(word, true),
        ])
    }
    fn set_scroll_word(&mut self, word: sm_game::display_scroll::Word, value: u16) {
        self.publish_scroll(word, value, None);
    }
    fn physical_scroll_byte(&self, word: sm_game::display_scroll::Word, high: bool) -> u8 {
        self.ram()
            .read_byte(scroll_address(word) + usize::from(high))
    }
    fn write_physical_scroll_byte(
        &mut self,
        word: sm_game::display_scroll::Word,
        high: bool,
        value: u8,
    ) {
        self.ram()
            .write_byte(scroll_address(word) + usize::from(high), value);
    }
}

fn sequence_ranges() -> [Range<usize>; 27] {
    [
        0x5f5..0x5f7,
        0x911..0x913,
        0x915..0x917,
        0x93f..0x941,
        0x943..0x945,
        0x9d2..0x9d4,
        0x9e2..0x9e4,
        0xa04..0xa06,
        0xa1c..0xa1e,
        0xa74..0xa76,
        0xa78..0xa7a,
        0xa54..0xa56,
        0xaa0..0xaa2,
        0xb2c..0xb2e,
        0xb2e..0xb30,
        0xcd0..0xcd2,
        0xd3e..0xd40,
        0xd40..0xd42,
        0xdc8..0xdca,
        0xdec..0xdee,
        0xdee..0xdf0,
        0xe12..0xe14,
        0x1982..0x1984,
        0x18a8..0x18aa,
        0x18aa..0x18ac,
        0xc400..0xc402,
        0x1f51..0x1f5b,
    ]
}

#[forbid(unsafe_code)]
impl Game {
    fn selected_sequence_image(&self) -> sm_game::sequence_state::State {
        use sm_game::session_cohort::Cohort as _;
        let mut words = sm_game::session_words::SessionWords::default();
        words.words(|at, value| {
            *value =
                u16::from_le_bytes([self.compatibility_byte(at), self.compatibility_byte(at + 1)])
        });
        sm_game::sequence_state::State {
            flow: sm_game::session_encoding::flow(&|at| self.compatibility_byte(at)),
            words,
        }
    }
    pub(crate) fn revoke_sequence_domain(&mut self) {
        *self.persistent_session.sequence.get_mut() = sm_game::sequence_state::Domain::Revoked;
    }
    pub(crate) fn adopt_sequence_image(&mut self) {
        use sm_game::sequence_state::Domain;
        let epoch = self.ram.sequence_epoch();
        if self.persistent_session_current() {
            *self.persistent_session.sequence.get_mut() = Domain::InSession { epoch };
            return;
        }
        let state = self.selected_sequence_image();
        *self.persistent_session.sequence.get_mut() = Domain::Owned { state, epoch };
    }
    fn sequence_state(&self) -> sm_game::sequence_state::State {
        use sm_game::sequence_state::{Domain, State};
        if self.persistent_session_current() {
            if let persistent_session::AggregateSlot::Ready(state) =
                &self.persistent_session.aggregate
            {
                return State::from_session(&state.gameplay);
            }
        }
        let mut domain = self.persistent_session.sequence.borrow_mut();
        if matches!(*domain, Domain::Unowned) {
            *domain = Domain::Owned {
                state: self.selected_sequence_image(),
                epoch: self.ram.sequence_epoch(),
            };
        }
        if matches!(&*domain, Domain::Owned { epoch, .. } if *epoch != self.ram.sequence_epoch())
            && matches!(
                self.persistent_session.aggregate,
                persistent_session::AggregateSlot::Legacy
            )
        {
            *domain = Domain::Owned {
                state: self.selected_sequence_image(),
                epoch: self.ram.sequence_epoch(),
            };
        }
        match &*domain {
            Domain::Owned { state, epoch } if *epoch == self.ram.sequence_epoch() => *state,
            // Revoked/dirty/suspended state cannot hide the selected physical image.
            _ => self.selected_sequence_image(),
        }
    }
    fn publish_sequence_words(&mut self, changed: Vec<(usize, u16)>) {
        use sm_game::sequence_state::Domain;
        if changed.is_empty() {
            return;
        }
        // The first actual writer establishes the supplied physical image once.
        // Explicit revocation and failed admission never revive an owner here.
        let needs_selected_image = {
            let domain = self.persistent_session.sequence.borrow();
            matches!(*domain, Domain::Unowned)
                || (matches!(&*domain, Domain::Owned { epoch, .. } if *epoch != self.ram.sequence_epoch())
                    && matches!(
                        self.persistent_session.aggregate,
                        persistent_session::AggregateSlot::Legacy
                    ))
        };
        if needs_selected_image {
            self.adopt_sequence_image();
        }
        let slot = std::mem::replace(self.persistent_session.sequence.get_mut(), Domain::Leased);
        let mut state = match slot {
            Domain::Owned { state, epoch } if epoch == self.ram.sequence_epoch() => Some(state),
            _ => None,
        };
        if let Some(state) = &mut state {
            for &(at, value) in &changed {
                if let Some(word) = sm_game::session_words::Word::ALL
                    .iter()
                    .find(|word| **word as usize == at)
                {
                    state.words.set(*word, value);
                } else {
                    match at {
                        0x1f51 => state.flow.cinematic_function = value,
                        0x1f53 => state.flow.demo_timer = value,
                        0x1f55 => state.flow.demo_set = value,
                        0x1f57 => state.flow.demo_scene = value,
                        0x1f59 => state.flow.num_demo_sets = value,
                        _ => unreachable!("sequence publication outside its field lanes"),
                    }
                }
            }
        }
        struct Publish<'a> {
            game: &'a mut Game,
            state: Option<sm_game::sequence_state::State>,
            completed: bool,
        }
        impl Drop for Publish<'_> {
            fn drop(&mut self) {
                *self.game.persistent_session.sequence.get_mut() = if self.completed {
                    if self.game.persistent_session_current() {
                        Domain::InSession {
                            epoch: self.game.ram.sequence_epoch(),
                        }
                    } else if let Some(state) = self.state.take() {
                        // The mutated typed owner stays authoritative. These
                        // bytes are its published compatibility representation.
                        Domain::Owned {
                            state,
                            epoch: self.game.ram.sequence_epoch(),
                        }
                    } else {
                        Domain::Revoked
                    }
                } else {
                    Domain::Revoked
                };
            }
        }
        let mut publish = Publish {
            game: self,
            state,
            completed: false,
        };
        for (at, value) in changed {
            for (part, byte) in value.to_le_bytes().into_iter().enumerate() {
                publish.game.compatibility_write_byte(at + part, byte);
            }
        }
        publish.completed = true;
    }
}
#[forbid(unsafe_code)]
impl sm_game::sequence_state::Access for Game {
    fn set_cinematic_function(&mut self, value: u16) {
        self.publish_sequence_words(vec![(0x1f51, value)]);
    }
    fn sequence_flow(&self) -> sm_game::flow::Flow {
        self.sequence_state().flow
    }
    fn sequence_words(&self) -> sm_game::session_words::SessionWords {
        self.sequence_state().words
    }
    fn set_sequence_word(&mut self, word: sm_game::session_words::Word, value: u16) {
        self.publish_sequence_words(vec![(word as usize, value)]);
    }
    fn commit_flow(&mut self, before: &sm_game::flow::Flow, after: sm_game::flow::Flow) {
        if *before == after {
            return;
        }
        if self.persistent_session_current() {
            use sm_game::session_access::SessionAccess as _;
            let mut runtime = crate::game::session_runtime::Runtime::new(self);
            after.apply_changed(before, &mut runtime.session_mut().flow);
            runtime.refresh_inputs();
            return;
        }
        self.publish_sequence_words(after.changes(before));
    }
    fn commit_sequence_words(
        &mut self,
        before: &sm_game::session_words::SessionWords,
        after: sm_game::session_words::SessionWords,
    ) {
        if *before == after {
            return;
        }
        if self.persistent_session_current() {
            use sm_game::session_access::SessionAccess as _;
            let mut runtime = crate::game::session_runtime::Runtime::new(self);
            after.apply_changed(before, runtime.session_mut());
            runtime.refresh_inputs();
            return;
        }
        self.publish_sequence_words(after.changes(before));
    }
}

fn frame_display_ranges() -> [Range<usize>; 20] {
    [
        0x51..0x53,
        0x55..0x86,
        0x5b4..0x5bc,
        0x782..0x783,
        0x7ec..0x7ed,
        0xa7..0xa9,
        0xab..0xad,
        0x77c..0x77e,
        0x9ec..0x9ee,
        0xa06..0xa08,
        0xa08..0xa0a,
        0xa0a..0xa0c,
        0xa0c..0xa0e,
        0xa0e..0xa10,
        0xc608..0xc63c,
        0xc646..0xc648,
        0xc648..0xc67c,
        0xc686..0xc688,
        0xc688..0xc6bc,
        0xc6c6..0xc6c8,
    ]
}
pub(crate) fn frame_field_address(field: sm_game::display::preparation::Field) -> usize {
    use sm_game::display::preparation::Field;
    match field {
        Field::Brightness => 0x51,
        Field::ObjectLayout => 0x52,
        Field::Mode => 0x55,
        Field::FakeMode => 0x56,
        Field::Mosaic => 0x57,
        Field::Screens0 => 0x58,
        Field::Screens1 => 0x59,
        Field::Screens2 => 0x5a,
        Field::Background3Map => 0x5b,
        Field::Screens3 => 0x5c,
        Field::Characters0 => 0x5d,
        Field::Characters1 => 0x5e,
        Field::AffineSelection => 0x5f,
        Field::WindowSelection0 => 0x60,
        Field::WindowSelection1 => 0x61,
        Field::WindowSelection2 => 0x62,
        Field::WindowPositions0 => 0x63,
        Field::WindowPositions1 => 0x64,
        Field::WindowPositions2 => 0x65,
        Field::WindowPositions3 => 0x66,
        Field::WindowLogic0 => 0x67,
        Field::WindowLogic1 => 0x68,
        Field::MainLayers => 0x69,
        Field::PreviousMainLayers => 0x6a,
        Field::SubLayers => 0x6b,
        Field::MainWindow => 0x6c,
        Field::SubWindow => 0x6d,
        Field::NextColorSelection => 0x6e,
        Field::ColorSelection => 0x6f,
        Field::PreviousColorSelection => 0x70,
        Field::NextColorMath => 0x71,
        Field::ColorMath => 0x72,
        Field::PreviousColorMath => 0x73,
        Field::FixedColor0 => 0x74,
        Field::FixedColor1 => 0x75,
        Field::FixedColor2 => 0x76,
        Field::Geometry => 0x77,
        Field::AffineWords0Low => 0x78,
        Field::AffineWords0High => 0x79,
        Field::AffineWords1Low => 0x7a,
        Field::AffineWords1High => 0x7b,
        Field::AffineWords2Low => 0x7c,
        Field::AffineWords2High => 0x7d,
        Field::AffineWords3Low => 0x7e,
        Field::AffineWords3High => 0x7f,
        Field::AffineWords4Low => 0x80,
        Field::AffineWords4High => 0x81,
        Field::AffineWords5Low => 0x82,
        Field::AffineWords5High => 0x83,
        Field::NmiEnable => 0x84,
        Field::HdmaEnable => 0x85,
        Field::NmiPending => 0x5b4,
        Field::NmiFrameByte => 0x5b5,
        Field::NmiFrameWordLow => 0x5b6,
        Field::NmiFrameWordHigh => 0x5b7,
        Field::NmiIncludingLagLow => 0x5b8,
        Field::NmiIncludingLagHigh => 0x5b9,
        Field::NmiMissed => 0x5ba,
        Field::NmiMaxMissed => 0x5bb,
        Field::NmiAcknowledgment => 0x782,
        Field::CommittedFakeMode => 0x7ec,
    }
}
pub(crate) fn frame_hud_address(index: isize) -> usize {
    let at = 0xc608usize
        .checked_add_signed(index.checked_mul(2).expect("HUD index overflow"))
        .expect("HUD address outside RAM");
    // The named word checked both byte bounds before reads or either store.
    assert!(at < 131071, "HUD word outside RAM");
    at
}
pub(crate) fn frame_word_address(word: sm_game::display::preparation::Word) -> usize {
    use sm_game::display::preparation::Word;
    match word {
        Word::PendingInterrupt => 0xa7,
        Word::CurrentInterrupt => 0xab,
        Word::HudPalette => 0x77c,
        Word::HudAutoCancel => 0x9ec,
        Word::PreviousHealth => 0xa06,
        Word::PreviousMissiles => 0xa08,
        Word::PreviousSupers => 0xa0a,
        Word::PreviousBombs => 0xa0c,
        Word::PreviousSelection => 0xa0e,
    }
}

#[forbid(unsafe_code)]
impl Game {
    fn selected_frame_display(&self) -> sm_game::display::preparation::State {
        sm_game::display::preparation::State::read(
            |field| self.ram.read_byte(frame_field_address(field)),
            |word| {
                let at = frame_word_address(word);
                u16::from_le_bytes([self.ram.read_byte(at), self.ram.read_byte(at + 1)])
            },
            |index| {
                let at = 0xc608 + 2 * index;
                u16::from_le_bytes([self.ram.read_byte(at), self.ram.read_byte(at + 1)])
            },
        )
    }
    pub(crate) fn revoke_frame_display_domain(&mut self) {
        *self.persistent_session.frame_display.get_mut() =
            sm_game::display::preparation::Domain::Revoked;
    }
    pub(crate) fn adopt_frame_display_image(&mut self) {
        use sm_game::display::preparation::Domain;
        let epoch = self.ram.frame_display_epoch();
        // Residency metadata does not admit a Session or run Samus preflight.
        if self.persistent_ram_epoch == self.ram.write_epoch()
            && matches!(&self.persistent_session.aggregate,
                persistent_session::AggregateSlot::Ready(session) if session.gameplay.presentation.is_owned())
        {
            *self.persistent_session.frame_display.get_mut() = Domain::InSession { epoch };
            return;
        }
        if matches!(self.persistent_session.frame_display.get_mut(),
            Domain::Owned { epoch: current, .. } if *current == epoch)
        {
            return;
        }
        let state = Box::new(self.selected_frame_display());
        *self.persistent_session.frame_display.get_mut() = Domain::Owned { state, epoch };
    }
    fn with_frame_display<R>(
        &self,
        owned: impl FnOnce(&sm_game::display::preparation::State) -> R,
        physical: impl FnOnce(&Self) -> R,
    ) -> R {
        use sm_game::display::preparation::Domain;
        let epoch = self.ram.frame_display_epoch();
        let mut domain = self.persistent_session.frame_display.borrow_mut();
        if matches!(&*domain, Domain::Unowned)
            || matches!(&*domain, Domain::Owned { epoch: current, .. } if *current != epoch)
        {
            // An expired Legacy image selects the actual completed bytes once.
            *domain = Domain::Owned {
                state: Box::new(self.selected_frame_display()),
                epoch,
            };
        }
        if let Domain::Owned {
            state,
            epoch: current,
        } = &*domain
            && *current == epoch
        {
            return owned(state);
        }
        // Explicit failure/suspension never invents resumption or semantic admission.
        physical(self)
    }
    fn publish_frame_display(
        &mut self,
        changes: &[(usize, u8)],
        update: impl FnOnce(&mut sm_game::display::preparation::State),
        store: sm_game::display::preparation::Store,
    ) {
        use sm_game::display::preparation::{Domain, Store};
        let epoch = self.ram.frame_display_epoch();
        if matches!(
            &*self.persistent_session.frame_display.borrow(),
            Domain::Unowned
        ) || matches!(&*self.persistent_session.frame_display.borrow(), Domain::Owned { epoch: current, .. } if *current != epoch)
        {
            self.adopt_frame_display_image();
        }
        let slot = std::mem::replace(
            self.persistent_session.frame_display.get_mut(),
            Domain::Leased,
        );
        let mut state = match slot {
            Domain::Owned {
                state,
                epoch: current,
            } if current == epoch => Some(state),
            _ => None,
        };
        if let Some(state) = &mut state {
            update(state);
        }
        struct Publish<'a> {
            game: &'a mut Game,
            state: Option<Box<sm_game::display::preparation::State>>,
            completed: bool,
            expected_epoch: u64,
        }
        impl Drop for Publish<'_> {
            fn drop(&mut self) {
                use sm_game::display::preparation::Domain;
                // A real lifecycle callback may revoke/adopt during a store.
                // Never resurrect the draft over its chosen resulting authority.
                if !matches!(
                    self.game.persistent_session.frame_display.get_mut(),
                    Domain::Leased
                ) {
                    return;
                }
                if !self.completed {
                    self.game.revoke_frame_display_domain();
                    return;
                }
                if self.game.ram.frame_display_epoch() != self.expected_epoch {
                    if self.state.is_some() {
                        self.game.adopt_frame_display_image();
                    } else {
                        self.game.revoke_frame_display_domain();
                    }
                    return;
                }
                *self.game.persistent_session.frame_display.get_mut() = if let Some(state) =
                    self.state.take()
                {
                    Domain::Owned {
                        state,
                        epoch: self.game.ram.frame_display_epoch(),
                    }
                } else if self.game.persistent_ram_epoch == self.game.ram.write_epoch()
                    && matches!(&self.game.persistent_session.aggregate,
                        persistent_session::AggregateSlot::Ready(session) if session.gameplay.presentation.is_owned())
                {
                    Domain::InSession {
                        epoch: self.game.ram.frame_display_epoch(),
                    }
                } else {
                    Domain::Revoked
                };
            }
        }
        let expected_epoch =
            epoch.wrapping_add(u64::try_from(changes.len()).expect("frame stores"));
        let mut publish = Publish {
            game: self,
            state,
            completed: false,
            expected_epoch,
        };
        for &(at, byte) in changes {
            match store {
                Store::Owned => publish.game.compatibility_write_owned_byte(at, byte),
                Store::Named => publish.game.compatibility_write_byte(at, byte),
            }
        }
        publish.completed = true;
    }
}
#[forbid(unsafe_code)]
impl sm_game::display::preparation::Access for Game {
    fn display_control_byte(&self, field: sm_game::display::preparation::Field) -> u8 {
        // This is precisely the original projected read's current-owner predicate.
        if let persistent_session::AggregateSlot::Ready(session) =
            &self.persistent_session.aggregate
            && self.persistent_ram_epoch == self.ram.write_epoch()
            && self.preflight_samus_runtime().is_ok()
            && let Ok(display) = session.gameplay.presentation.owner()
        {
            return sm_game::display::preparation::display_byte(&display.state, field);
        }
        self.with_frame_display(
            |state| state.byte(field),
            |game| game.ram.read_byte(frame_field_address(field)),
        )
    }
    fn store_display_control_byte(
        &mut self,
        field: sm_game::display::preparation::Field,
        value: u8,
        store: sm_game::display::preparation::Store,
    ) {
        self.publish_frame_display(
            &[(frame_field_address(field), value)],
            |state| state.set_byte(field, value),
            store,
        );
    }
    fn display_history_word(&self, word: sm_game::display::preparation::Word) -> u16 {
        if let persistent_session::AggregateSlot::Ready(session) =
            &self.persistent_session.aggregate
            && self.persistent_ram_epoch == self.ram.write_epoch()
            && self.preflight_samus_runtime().is_ok()
            && let Ok(display) = session.gameplay.presentation.owner()
        {
            return sm_game::display::preparation::display_word(&display.state, word);
        }
        self.with_frame_display(
            |state| state.word(word),
            |game| {
                let at = frame_word_address(word);
                u16::from_le_bytes([game.ram.read_byte(at), game.ram.read_byte(at + 1)])
            },
        )
    }
    fn store_display_history_word(
        &mut self,
        word: sm_game::display::preparation::Word,
        value: u16,
        store: sm_game::display::preparation::Store,
    ) {
        let at = frame_word_address(word);
        let [low, high] = value.to_le_bytes();
        self.publish_frame_display(
            &[(at, low), (at + 1, high)],
            |state| state.set_word(word, value),
            store,
        );
    }
    fn hud_tile(&self, index: isize) -> u16 {
        let at = frame_hud_address(index);
        let normal = usize::try_from(index).ok();
        if let persistent_session::AggregateSlot::Ready(session) =
            &self.persistent_session.aggregate
            && self.persistent_ram_epoch == self.ram.write_epoch()
            && self.preflight_samus_runtime().is_ok()
            && let Ok(display) = session.gameplay.presentation.owner()
            && let Some(&value) = normal.and_then(|index| display.state.hud.tiles.get(index))
        {
            return value;
        }
        let Some(index) =
            normal.filter(|&index| sm_game::display::preparation::hud_index(index).is_some())
        else {
            // Signed/off-table aliases and minimap retain the original projection.
            return u16::from_le_bytes([
                self.compatibility_byte(at),
                self.compatibility_byte(at + 1),
            ]);
        };
        self.with_frame_display(
            |state| state.tile(index).expect("non-minimap HUD cell"),
            |game| u16::from_le_bytes([game.ram.read_byte(at), game.ram.read_byte(at + 1)]),
        )
    }
    fn store_hud_tile(
        &mut self,
        index: isize,
        value: u16,
        store: sm_game::display::preparation::Store,
    ) {
        let at = frame_hud_address(index);
        let [low, high] = value.to_le_bytes();
        let normal = usize::try_from(index)
            .ok()
            .filter(|&index| sm_game::display::preparation::hud_index(index).is_some());
        let Some(index) = normal else {
            match store {
                sm_game::display::preparation::Store::Owned => {
                    self.compatibility_write_owned_byte(at, low);
                    self.compatibility_write_owned_byte(at + 1, high);
                }
                sm_game::display::preparation::Store::Named => {
                    self.compatibility_write_byte(at, low);
                    self.compatibility_write_byte(at + 1, high);
                }
            }
            return;
        };
        self.publish_frame_display(
            &[(at, low), (at + 1, high)],
            |state| {
                state.set_tile(index, value);
            },
            store,
        );
    }
    fn physical_display_control_byte(&self, field: sm_game::display::preparation::Field) -> u8 {
        self.ram.read_byte(frame_field_address(field))
    }
    fn store_physical_display_control_byte(
        &mut self,
        field: sm_game::display::preparation::Field,
        value: u8,
    ) {
        self.ram.write_byte(frame_field_address(field), value);
    }
}
