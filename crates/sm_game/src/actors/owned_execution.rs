//! Actor execution on the current owned Session.
//! The scheduler, contacts, scripts, deaths and scene producer share this host.
//! Species-specific callback implementations are bound by the prepared owner;
//! the retained compatibility dispatcher is not a prepared implementation.
#![forbid(unsafe_code)]
use crate::actors::access::{AccessMode, ActorAccess, ActorKey};
use crate::actors::api::{DeathHandling, DeathKind};
use crate::actors::content::{ActorCatalog, ActorRoutine};
use crate::actors::effects::ActorHost;
use crate::actors::{animation, combat, drawing, population, scheduling};
use crate::combat_rules::LatchMode;
use crate::enemy_projectiles::host::Host as ProjectileHost;
use crate::enemy_script::EnemyScript;
use crate::room_mechanisms::host::Effects as MechanismEffects;

/// A finite common callback, independent of an enemy definition address.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum CommonAi {
    Resume,
    LatchPassive,
    LatchActive,
    GrappleDeath,
    SelectFrozen,
    UpdateFrozen,
}
impl CommonAi {
    /// Decode only at the original dispatch observation point. Unknown routines
    /// remain with their species dispatcher; no bank or actor-key truncation.
    pub fn from_routine(routine: ActorRoutine) -> Option<Self> {
        Some(match routine.0 {
            10715136 | 10846208 | 10911744 | 10977280 | 11042816 | 11763712 => Self::Resume,
            10649605 | 10780677 => Self::LatchPassive,
            11042836 => Self::LatchActive,
            10649610 | 10715146 | 11042826 | 11763722 => Self::GrappleDeath,
            10649615 | 10715151 | 10846223 | 10911759 | 10977295 | 11042831 | 11042846
            | 11173903 | 11763727 => Self::SelectFrozen,
            10649665 | 10715201 | 10780737 | 10846273 | 10911809 | 10977345 | 11042881
            | 11173953 | 11763777 => Self::UpdateFrozen,
            _ => return None,
        })
    }
}

/// These are the real shared reactions, including synchronous death allocation,
/// captured actor identity, selector changes and their source-order stores.
pub fn common_ai<H: ActorHost + ProjectileHost + MechanismEffects>(kind: CommonAi, h: &mut H) {
    match kind {
        CommonAi::Resume => combat::resume_ai(h),
        CommonAi::LatchPassive => combat::latch(h, LatchMode::Passive),
        CommonAi::LatchActive => combat::latch(h, LatchMode::Active),
        CommonAi::GrappleDeath => combat::die(h, DeathKind::Grapple),
        CommonAi::SelectFrozen => combat::select_frozen_ai(h),
        CommonAi::UpdateFrozen => scheduling::update_frozen_enemy(h),
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum OpenError {
    World,
    ActorDefinitions,
    SaveState,
    Presentation,
}

/// One exclusive actor interval on the actual aggregate. Opening neither
/// recreates actors nor resets hidden script state. The supplied host must
/// bind every ActorEffects callback to our owned species implementation.
/// This core deliberately supplies no compatibility callback implementation.
pub struct PreparedActorExecution<'a, H> {
    host: &'a mut H,
}
impl<'a, H: ActorHost + ProjectileHost + MechanismEffects> PreparedActorExecution<'a, H> {
    pub fn open(host: &'a mut H) -> Result<Self, OpenError> {
        if host.session().world.is_canonical_legacy() {
            return Err(OpenError::World);
        }
        if !matches!(host.session().world.actors(), ActorCatalog::Prepared(_)) {
            return Err(OpenError::ActorDefinitions);
        }
        Ok(Self { host })
    }
    /// Room-entry population lifecycle is an explicit operation, not startup
    /// work replayed when a suspended gameplay interval resumes.
    pub fn install_room_population(&mut self) {
        population::install_population(self.host);
    }
    pub fn select(&mut self) {
        scheduling::select(self.host);
    }
    /// Same active traversal, immediate contacts, main/time-frozen callbacks,
    /// animation, pending death, drawing queue and sprite-update order.
    pub fn advance_selected(&mut self) {
        scheduling::advance_selected(self.host);
    }
    /// Animation captures the original actor and allocation, then takes a fresh
    /// actor selector for each instruction callback. It does not predecode.
    pub fn advance_animation(&mut self) {
        animation::advance_animation(self.host);
    }
    pub fn delete_connected(&mut self) {
        population::delete_connected(self.host);
    }
    pub fn respawn(&mut self, actor: ActorKey) {
        population::respawn(self.host, actor);
    }
    pub fn current_common_ai(&mut self, kind: CommonAi) {
        common_ai(kind, self.host);
    }
    pub fn contact_reaction(&mut self, handling: DeathHandling) {
        combat::contact_reaction(self.host, handling);
    }
    pub fn die(&mut self, kind: DeathKind) {
        combat::die(self.host, kind);
    }
}
impl<H: ActorHost + ProjectileHost + crate::player_projectiles::host::Host>
    PreparedActorExecution<'_, H>
{
    /// Uses the existing presentation owner and exact sprite/shot/actor layer
    /// order. The graphics hook executes after the actor queues are consumed.
    pub fn draw_scene(&mut self) {
        drawing::draw_scene(self.host);
    }
}

/// The current actor key remains an exact full-width identity. Species-owned
/// callbacks use this when the original captures its selector at entry.
pub fn captured_actor(h: &impl ActorAccess) -> ActorKey {
    ActorKey::from_legacy(h.session().actors.controls.selected)
}
/// A captured key's definition is read when the source asks for it, rather than
/// cached during prepared-start validation.
pub fn live_definition(h: &impl ActorAccess, actor: ActorKey) -> u16 {
    h.resolve_actor(actor, AccessMode::Native).definition
}

/// The two Etecoon entry routines reach the single owned state machine. This
/// dispatch never opens a compatibility lease or reads an external selector.
pub fn etecoon_ai<H: crate::actors::etecoon::owned::Host>(
    routine: ActorRoutine,
    host: &mut H,
) -> bool {
    match routine.0 {
        11004178 => crate::actors::etecoon::owned::initialize(host),
        11004224 => crate::actors::etecoon::owned::main_ai(host),
        _ => return false,
    }
    true
}

/// Immutable allocations supplied by the prepared-content constructor. The
/// compatible script/record representation remains a migration dependency;
/// this value never opens a ROM, borrows a native pointer or copies RAM.
#[derive(Clone)]
pub struct ActorAllocations {
    loaded: Option<sm_data::cartridge::RomImage>,
    native: Option<sm_data::cartridge::RomImage>,
    speeds: std::rc::Rc<Option<crate::imported::Imported<sm_data::enemy_speeds::EnemySpeeds>>>,
    records: std::rc::Rc<crate::records::RomRecords>,
    revision: crate::actors::content::GameplayRevision,
}
impl ActorAllocations {
    pub fn loaded(&self) -> &sm_data::cartridge::RomImage {
        self.loaded.as_ref().expect("ROM must be loaded")
    }
    pub fn native(&self) -> &sm_data::cartridge::RomImage {
        self.native.as_ref().expect("native ROM must be registered")
    }
    /// Record source presence/origins after a diagnostic, without acquiring bytes
    /// or forcing a missing source ahead of the original getter observation.
    pub fn replace_sources(
        &mut self,
        loaded: Option<sm_data::cartridge::RomImage>,
        native: Option<sm_data::cartridge::RomImage>,
        speeds: std::rc::Rc<Option<crate::imported::Imported<sm_data::enemy_speeds::EnemySpeeds>>>,
        records: std::rc::Rc<crate::records::RomRecords>,
    ) {
        self.loaded = loaded;
        self.native = native;
        self.speeds = speeds;
        self.records = records;
    }
    pub fn new(
        world: &crate::room_entry::world::WorldBinding,
        loaded: sm_data::cartridge::RomImage,
        native: sm_data::cartridge::RomImage,
        speeds: std::rc::Rc<Option<crate::imported::Imported<sm_data::enemy_speeds::EnemySpeeds>>>,
        records: std::rc::Rc<crate::records::RomRecords>,
        revision: crate::actors::content::GameplayRevision,
    ) -> Result<Self, OpenError> {
        if world.actors().revision() != Some(revision) {
            return Err(OpenError::ActorDefinitions);
        }
        Ok(Self {
            loaded: Some(loaded),
            native: Some(native),
            speeds,
            records,
            revision,
        })
    }
}

/// Explicit remaining allocation aliases. Ordinary accesses never enter this
/// port: the existing codec routes them to the canonical Session field.
/// Implementations resolve this CURRENT aggregate, not an earlier Session or
/// a writable always-live mirror. This is a retained dependency until every
/// admitted cross-owner alias and low-window diagnostic has an owned binding.
/// These original projectile record acquisition edges have different
/// diagnostics/observer prefixes; their allocation must be captured afterward.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ProjectileRecordSource {
    PlayerLoaded,
    PlayerNative,
    EnemyLoaded,
    EnemyNative,
}
#[path = "../room_entry/owned_bindings.rs"]
pub mod room_bindings;
pub trait ImportedActorAllocation:
    PlayerPorts
    + crate::actors::prepared_player_frame::FramePorts
    + room_bindings::RoomPorts
    + crate::actors::remaining_owned::bindings::Ports
    + crate::owned_hdma_update::bindings::Ports
{
    fn presentation_diagnostics(
        &mut self,
    ) -> &mut dyn crate::scene::actor_presentation::Diagnostics;
    fn dialogue_controller_word(
        &mut self,
        owner: &mut crate::persistent_session::PersistentSession,
        register: u16,
    ) -> u16;
    fn dialogue_request_reset(&mut self, owner: &mut crate::persistent_session::PersistentSession);
    fn low_window_projectile_record(
        &mut self,
        owner: &mut crate::persistent_session::PersistentSession,
        allocations: &mut ActorAllocations,
        source: ProjectileRecordSource,
        bank: u8,
        address: u16,
    ) -> sm_data::cartridge::RomRecord;
    /// Original lazy decompression diagnostic before loaded byte acquisition.
    fn report_invalid_decompression_address(
        &mut self,
        owner: &mut crate::persistent_session::PersistentSession,
        address: u32,
    );
    fn actor_alias_byte(&self, state: &crate::session::Session, at: usize) -> u8;
    fn set_actor_alias_byte(&mut self, state: &mut crate::session::Session, at: usize, value: u8);
    /// Preserve the diagnostic/effect prefix before capturing the actual loaded
    /// allocation for the original low-window actor content fetch.
    /// The same diagnostic-prefix contract, with the native allocation
    /// captured afterwards rather than the loaded record allocation.
    /// The original mechanism diagnostic occurs on this current owner after
    /// every preceding setup/script effect, and preserves its return/fatal edge.
    fn mechanism_diagnostic(
        &mut self,
        owner: &mut crate::persistent_session::PersistentSession,
        diagnostic: crate::room_mechanisms::host::Diagnostic,
    ) -> u8;
    fn low_window_native_actor_record(
        &mut self,
        state: &mut crate::persistent_session::PersistentSession,
        allocations: &mut ActorAllocations,
        bank: u8,
        address: u16,
    ) -> sm_data::cartridge::RomRecord;
    fn low_window_actor_record(
        &mut self,
        state: &mut crate::persistent_session::PersistentSession,
        allocations: &mut ActorAllocations,
        bank: u8,
        address: u16,
    ) -> sm_data::cartridge::RomRecord;
}

/// The concrete exclusive state/content host. Canonical actor edits borrow the
/// existing slots. Exceptional full/odd/native identities use the existing
/// changed-field guard, including its unwind prefix. No actor state is copied
/// into a second owner and opening preserves hidden/suspended continuation.
/// Domain effects and all-species dispatch still need their owned bindings
/// before this host can serve the complete PreparedActorExecution interval.
pub(crate) enum SamusContentBinding<'a> {
    Opened(crate::scene::prepared_samus::PreparedSamusContent),
    Borrowed(&'a crate::scene::prepared_samus::PreparedSamusContent),
}
impl std::ops::Deref for SamusContentBinding<'_> {
    type Target = crate::scene::prepared_samus::PreparedSamusContent;
    fn deref(&self) -> &Self::Target {
        match self {
            Self::Opened(value) => value,
            Self::Borrowed(value) => value,
        }
    }
}
pub struct PreparedActorState<'a> {
    pub(crate) owner: &'a mut crate::persistent_session::PersistentSession,
    pub(crate) allocations: &'a mut ActorAllocations,
    pub(crate) aliases: &'a mut dyn ImportedActorAllocation,
    pub(crate) presentation: &'a crate::scene::actor_presentation::Binding,
    pub(crate) samus_content: SamusContentBinding<'a>,
}
impl<'a> PreparedActorState<'a> {
    /// Reborrow the exact fields of an already admitted actor interval.
    /// No state/source copy, resource admission or runner is created here.
    pub fn reborrow(
        owner: &'a mut crate::persistent_session::PersistentSession,
        allocations: &'a mut ActorAllocations,
        aliases: &'a mut dyn ImportedActorAllocation,
        presentation: &'a crate::scene::actor_presentation::Binding,
        samus_content: &'a crate::scene::prepared_samus::PreparedSamusContent,
    ) -> Self {
        Self {
            owner,
            allocations,
            aliases,
            presentation,
            samus_content: SamusContentBinding::Borrowed(samus_content),
        }
    }
    pub fn open(
        owner: &'a mut crate::persistent_session::PersistentSession,
        allocations: &'a mut ActorAllocations,
        aliases: &'a mut dyn ImportedActorAllocation,
        presentation: &'a crate::scene::actor_presentation::Binding,
    ) -> Result<Self, OpenError> {
        if owner.gameplay.world.is_canonical_legacy() {
            return Err(OpenError::World);
        }
        if owner.gameplay.world.actors().revision() != Some(allocations.revision) {
            return Err(OpenError::ActorDefinitions);
        }
        if !owner.gameplay.save_state.initialized {
            return Err(OpenError::SaveState);
        }
        let content = owner
            .actor_content
            .as_ref()
            .ok_or(OpenError::Presentation)?;
        if !std::ptr::eq(presentation, content.binding()) {
            return Err(OpenError::Presentation);
        }
        content
            .check_sources(&crate::actors::presentation_content::SourceAllocations {
                loaded: allocations.loaded().clone(),
                native: allocations.native().clone(),
            })
            .map_err(|_| OpenError::Presentation)?;
        // Resource admission belongs to validated start/reopen before an interval.
        // Opening a suspended actor view checks it, without publishing a queue.
        presentation
            .check(&owner.gameplay)
            .map_err(|_| OpenError::Presentation)?;
        let samus_content = crate::scene::prepared_samus::PreparedSamusContent::prepare(owner)
            .map_err(|_| OpenError::Presentation)?;
        Ok(Self {
            owner,
            allocations,
            aliases,
            presentation,
            samus_content: SamusContentBinding::Opened(samus_content),
        })
    }
    pub(crate) fn current_byte(&self, at: usize) -> u8 {
        crate::session_encoding::projected_byte(&self.owner.gameplay, at)
            .unwrap_or_else(|| self.aliases.actor_alias_byte(&self.owner.gameplay, at))
    }
    pub(crate) fn set_current_byte(&mut self, at: usize, value: u8) {
        if !crate::session_encoding::write_projected_byte(&mut self.owner.gameplay, at, value) {
            self.aliases
                .set_actor_alias_byte(&mut self.owner.gameplay, at, value);
        }
        if crate::session_owners::Presentation::requires_semantic_counterpart(at) {
            self.owner
                .gameplay
                .presentation
                .owner_mut()
                .expect("owned actor display")
                .state
                .phase = crate::scene::state::DisplayPhase::Failed;
        }
        crate::session_access::SessionAccess::refresh_inputs(self);
    }
    pub(crate) fn write_current_slice(&mut self, at: usize, bytes: &[u8]) {
        // Runtime::write_slice admitted the full 128-KiB allocation before
        // any byte store. This check is local even for canonical typed spans;
        // no imported provider is queried ahead of the original first write.
        let end = at.checked_add(bytes.len()).expect("actor slice extent");
        assert!(at <= end && end <= 0x20000);
        for (offset, value) in bytes.iter().copied().enumerate() {
            self.set_current_byte(at.checked_add(offset).expect("actor slice offset"), value);
        }
    }
    pub fn select(&mut self) {
        self.guarded_actor_call(scheduling::select);
    }
    /// State-only common callbacks need no content or external effect.
    pub fn resume_current_ai(&mut self) {
        combat::resume_ai(self);
    }
    pub fn select_current_frozen_ai(&mut self) {
        self.guarded_actor_call(combat::select_frozen_ai);
    }
    pub fn update_current_frozen_ai(&mut self) {
        self.guarded_actor_call(scheduling::update_frozen_enemy);
    }
}
impl crate::session_access::SessionAccess for PreparedActorState<'_> {
    fn session(&self) -> &crate::session::Session {
        &self.owner.gameplay
    }
    fn session_mut(&mut self) -> &mut crate::session::Session {
        &mut self.owner.gameplay
    }
    fn inputs(&self) -> &crate::external_inputs::ExternalInputs {
        &self.owner.observations
    }
    fn refresh_inputs(&mut self) {
        self.owner.observations =
            crate::session_encoding::inputs_from_session(&self.owner.gameplay, &|at| {
                self.current_byte(at)
            });
    }
    fn probe(&self, _: &'static str) {}
}
impl ActorAccess for PreparedActorState<'_> {
    fn resolve_actor<'b>(
        &'b self,
        actor: ActorKey,
        mode: AccessMode,
    ) -> crate::actors::access::ActorRead<'b> {
        crate::actors::access::resolve_actor_with(self, actor, mode, Self::current_byte)
    }
    fn edit_actor<R>(
        &mut self,
        actor: ActorKey,
        mode: AccessMode,
        edit: impl for<'edit> FnOnce(&'edit mut crate::actors::state::Actor) -> R,
    ) -> R {
        crate::actors::access::edit_actor_with(
            self,
            actor,
            mode,
            Self::current_byte,
            Self::set_current_byte,
            edit,
        )
    }
}
impl crate::actors::access::SpawnAccess for PreparedActorState<'_> {
    fn resolve_spawn<'b>(
        &'b self,
        actor: ActorKey,
        mode: AccessMode,
    ) -> crate::actors::access::SpawnRead<'b> {
        crate::actors::access::resolve_spawn_with(self, actor, mode, Self::current_byte)
    }
    fn edit_spawn<R>(
        &mut self,
        actor: ActorKey,
        mode: AccessMode,
        edit: impl for<'edit> FnOnce(&'edit mut crate::actors::state::SpawnRecord) -> R,
    ) -> R {
        crate::actors::access::edit_spawn_with(
            self,
            actor,
            mode,
            Self::current_byte,
            Self::set_current_byte,
            edit,
        )
    }
}
impl crate::actors::session_slots::SessionSlots for PreparedActorState<'_> {
    fn projected_slot_byte(&self, at: usize) -> u8 {
        self.current_byte(at)
    }
    fn store_projected_slot_byte(&mut self, at: usize, value: u8) {
        self.set_current_byte(at, value);
    }
}
impl crate::actors::access::ActorCollections for PreparedActorState<'_> {
    fn collection_word(
        &self,
        window: crate::actors::access::CollectionWindow,
        offset: usize,
        mode: AccessMode,
    ) -> u16 {
        let read = |lane| {
            self.collection_typed_byte(window, lane).unwrap_or_else(|| {
                crate::actors::access::collection_byte_with(self, window, lane, Self::current_byte)
            })
        };
        let bytes = [
            read(offset),
            read(offset.checked_add(1).expect("collection word extent")),
        ];
        match mode {
            AccessMode::LittleEndian => u16::from_le_bytes(bytes),
            AccessMode::Native => u16::from_ne_bytes(bytes),
        }
    }
    fn collection_byte(
        &self,
        window: crate::actors::access::CollectionWindow,
        offset: usize,
    ) -> u8 {
        self.collection_typed_byte(window, offset)
            .unwrap_or_else(|| {
                crate::actors::access::collection_byte_with(
                    self,
                    window,
                    offset,
                    Self::current_byte,
                )
            })
    }
    fn write_collection_word(
        &mut self,
        window: crate::actors::access::CollectionWindow,
        offset: usize,
        value: u16,
        store: crate::actors::access::CollectionStore,
    ) {
        if matches!(window, crate::actors::access::CollectionWindow::VramQueue) {
            let at = 0xd0usize
                .checked_add(offset)
                .expect("actor queue word offset");
            let bytes = match store {
                crate::actors::access::CollectionStore::PrefixLe
                | crate::actors::access::CollectionStore::SliceLe => value.to_le_bytes(),
                crate::actors::access::CollectionStore::SliceNative => value.to_ne_bytes(),
            };
            if !matches!(store, crate::actors::access::CollectionStore::PrefixLe) {
                let end = at.checked_add(2).expect("actor queue word extent");
                assert!(at <= end && end <= 0x20000);
            }
            self.guarded_actor_call(|host| {
                for (lane, value) in bytes.into_iter().enumerate() {
                    host.presentation
                        .queue_byte(
                            &mut host.owner.gameplay,
                            offset + lane,
                            value,
                            host.aliases.presentation_diagnostics(),
                        )
                        .expect("actor queue word publication");
                    crate::session_access::SessionAccess::refresh_inputs(host);
                }
            });
            return;
        }
        let bytes = match store {
            crate::actors::access::CollectionStore::SliceNative => value.to_ne_bytes(),
            _ => value.to_le_bytes(),
        };
        if !matches!(store, crate::actors::access::CollectionStore::PrefixLe) {
            let at = window.address(offset);
            let end = at.checked_add(2).expect("collection word extent");
            assert!(at <= end && end <= 0x20000);
        }
        for (lane, value) in bytes.into_iter().enumerate() {
            crate::actors::access::ActorCollections::write_collection_byte(
                self,
                window,
                offset.checked_add(lane).expect("collection word offset"),
                value,
            );
        }
    }
    fn write_collection_byte(
        &mut self,
        window: crate::actors::access::CollectionWindow,
        offset: usize,
        value: u8,
    ) {
        if matches!(window, crate::actors::access::CollectionWindow::VramQueue) {
            self.guarded_actor_call(|host| {
                host.presentation
                    .queue_byte(
                        &mut host.owner.gameplay,
                        offset,
                        value,
                        host.aliases.presentation_diagnostics(),
                    )
                    .expect("actor queue byte publication");
                crate::session_access::SessionAccess::refresh_inputs(host);
            });
            return;
        }
        if self.set_collection_typed_byte(window, offset, value) {
            crate::session_access::SessionAccess::refresh_inputs(self);
        } else {
            crate::actors::access::write_collection_byte_with(
                self,
                window,
                offset,
                value,
                Self::set_current_byte,
            );
        }
    }
    fn write_tile_source(&mut self, offset: usize, source: [u8; 3]) {
        let first = offset.checked_add(2).expect("tile source offset");
        let at = 0xe5ausize.checked_add(first).expect("tile source address");
        let end = at.checked_add(source.len()).expect("tile source extent");
        assert!(at <= end && end <= 0x20000);
        for (lane, value) in source.into_iter().enumerate() {
            crate::actors::access::ActorCollections::write_collection_byte(
                self,
                crate::actors::access::CollectionWindow::TileLoad,
                first + lane,
                value,
            );
        }
    }
    fn copy_palette_row(&mut self, row: u8, colors: [u8; 32]) {
        let at = 0xc200 + (usize::from(row) + 8) * 32;
        let end = at
            .checked_add(colors.len())
            .expect("actor palette row extent");
        assert!(at <= end && end <= 0x20000);
        for (lane, value) in colors.into_iter().enumerate() {
            let address = at + lane;
            if (0xc200..0xc400).contains(&address) {
                let offset = address - 0xc200;
                let field = &mut self.owner.gameplay.presentation.visual_mut().targets[offset / 2];
                let mut bytes = field.to_le_bytes();
                bytes[offset % 2] = value;
                *field = u16::from_le_bytes(bytes);
                crate::session_access::SessionAccess::refresh_inputs(self);
            } else {
                self.set_current_byte(address, value);
            }
        }
    }
}
impl crate::actors::content_records::ContentSource for PreparedActorState<'_> {
    fn content_image(&self) -> sm_data::cartridge::RomImage {
        self.allocations.loaded().clone()
    }
    fn native_content_image(&self) -> sm_data::cartridge::RomImage {
        self.allocations.native().clone()
    }
    fn content_record(&mut self, bank: u8, address: u16) -> sm_data::cartridge::RomRecord {
        if address & 0x8000 == 0 {
            return self.aliases.low_window_actor_record(
                self.owner,
                self.allocations,
                bank,
                address,
            );
        }
        sm_data::cartridge::RomRecord {
            image: self.allocations.loaded().clone(),
            base: usize::try_from(sm_data::cartridge::lorom_offset(
                (u32::from(bank) << 16) | u32::from(address),
            ))
            .expect("actor record offset"),
        }
    }
}

pub fn dachora_ai<H: crate::actors::dachora_run::owned::Host>(
    routine: ActorRoutine,
    host: &mut H,
) -> bool {
    match routine.0 {
        11007197 => crate::actors::dachora_run::owned::initialize(host),
        11007278 => crate::actors::dachora_run::owned::main_ai(host),
        _ => return false,
    }
    true
}
pub fn mini_kraid_ai<H: crate::actors::mini_kraid::owned::Host>(
    routine: ActorRoutine,
    host: &mut H,
) -> bool {
    match routine.0 {
        10918488 => crate::actors::mini_kraid::owned::initialize(host),
        10918594 => crate::actors::mini_kraid::owned::main_ai(host),
        10918946 => crate::actors::mini_kraid::owned::touch(host),
        10918969 => crate::actors::mini_kraid::owned::shot(host),
        _ => return false,
    }
    true
}

/// Complete Gripper/Ripper II/Ripper callbacks use the same body as compat.
pub fn rippers_ai<H: crate::actors::rippers::owned::Host>(
    routine: ActorRoutine,
    host: &mut H,
) -> bool {
    use crate::actors::rippers::{Kind, owned};
    match routine.0 {
        10674643 => owned::initialize(Kind::Gripper, host),
        10674968 => owned::initialize(Kind::RipperII, host),
        10675359 => owned::initialize(Kind::Ripper, host),
        10674721 => owned::main_ai(Kind::Gripper, host),
        10675027 => owned::main_ai(Kind::RipperII, host),
        10675418 => owned::main_ai(Kind::Ripper, host),
        10675113 => owned::shot(host),
        _ => return false,
    }
    true
}

pub fn mini_kraid_instruction<H: crate::actors::mini_kraid::owned::Host>(
    routine: ActorRoutine,
    script: crate::enemy_script::EnemyScript,
    host: &mut H,
) -> Result<crate::enemy_script::EnemyScript, crate::enemy_script::EnemyScript> {
    use crate::actors::mini_kraid::owned;
    match routine.0 {
        10918694 => owned::step(host),
        10918772 => return Ok(owned::choose_animation(host)),
        10918834 => owned::roar(host),
        10918852 => owned::spit_left(host),
        10918914 => owned::spit_right(host),
        _ => return Err(script),
    }
    Ok(script)
}
pub fn zebetites_ai<H: crate::actors::zebetites::owned::Host>(
    routine: ActorRoutine,
    host: &mut H,
) -> bool {
    use crate::actors::zebetites::owned;
    match routine.0 {
        10943346 => owned::initialize(host),
        10943539 => owned::main_ai(host),
        10943911 => owned::touch(host),
        10943916 => owned::shot(host),
        _ => return false,
    }
    true
}
/// Failure remains after the Maw's main state writes and before any body
/// segment update. It is not a prepared-start validation or silent no-op.
pub trait MawDispatchHost: crate::actors::yapping_maw::owned::Host {
    fn unknown_maw_function(&mut self) -> !;
}
pub fn yapping_maw_ai<H: MawDispatchHost>(routine: ActorRoutine, host: &mut H) -> bool {
    use crate::actors::yapping_maw::owned;
    match routine.0 {
        11051336 => owned::initialize(host),
        11052953 => owned::touch(host),
        11052989 => owned::shot(host),
        11053109 => owned::frozen(host),
        11051537 => {
            if !owned::main_ai(host) {
                host.unknown_maw_function();
            }
            owned::update_segments(host);
        }
        _ => return false,
    }
    true
}
pub fn yapping_maw_instruction<H: crate::actors::yapping_maw::owned::Host>(
    routine: ActorRoutine,
    script: crate::enemy_script::EnemyScript,
    host: &mut H,
) -> Result<crate::enemy_script::EnemyScript, crate::enemy_script::EnemyScript> {
    match routine.0 {
        11051207 | 11051225 | 11051243 | 11051261 | 11051279 | 11051297 | 11051315 => Ok(
            crate::actors::yapping_maw::owned::instruction(routine.0, script, host),
        ),
        _ => Err(script),
    }
}

#[derive(Debug)]
pub enum ActorExecutionFault {
    UnknownFunction,
    Assertion(crate::actors::effects::ActorAssertion),
}
impl PreparedActorState<'_> {
    pub(crate) fn speed_word(&self, table: sm_data::enemy_speeds::SpeedTable, index: isize) -> u16 {
        crate::imported::Imported::from(&self.allocations.speeds, self.allocations.loaded())
            .and_then(|speeds| speeds.word(table, index))
            .unwrap_or_else(|| {
                self.allocations.loaded().table_word(
                    usize::try_from(sm_data::cartridge::lorom_offset(table.address()))
                        .expect("speed table"),
                    index,
                )
            })
    }
    pub(crate) fn loaded_table_word(&self, address: u32, index: isize) -> u16 {
        self.allocations.loaded().table_word(
            usize::try_from(sm_data::cartridge::lorom_offset(address)).expect("actor table"),
            index,
        )
    }
    pub(crate) fn native_actor_record(
        &mut self,
        bank: u8,
        address: u16,
    ) -> sm_data::cartridge::RomRecord {
        if address & 0x8000 == 0 {
            return self.aliases.low_window_native_actor_record(
                self.owner,
                self.allocations,
                bank,
                address,
            );
        }
        sm_data::cartridge::RomRecord {
            image: self.allocations.native().clone(),
            base: usize::try_from(sm_data::cartridge::lorom_offset(
                (u32::from(bank) << 16) | u32::from(address),
            ))
            .expect("native actor record"),
        }
    }
    pub(crate) fn unknown_function(&mut self) -> ! {
        self.owner.poison();
        std::panic::panic_any(ActorExecutionFault::UnknownFunction)
    }
    /// Real prepared initialization and state-machine callbacks over this
    /// concrete owner. No compatibility callback is involved.
    pub fn initialize_choot(&mut self) {
        crate::actors::choot::owned::initialize(self);
    }
    pub fn advance_choot(&mut self) {
        crate::actors::choot::owned::main_ai(self);
    }
    pub fn initialize_zoa(&mut self) {
        crate::actors::zoa::owned::initialize(self);
    }
    pub fn advance_zoa(&mut self) {
        crate::actors::zoa::owned::main_ai(self);
    }
}
impl crate::actors::choot::owned::Host for PreparedActorState<'_> {
    fn choot_speed(&self, table: sm_data::enemy_speeds::SpeedTable, index: isize) -> u16 {
        self.speed_word(table, index)
    }
    fn choot_table(&self, table: crate::actors::choot::owned::Table, index: isize) -> u16 {
        let address = match table {
            crate::actors::choot::owned::Table::Patterns => 0xa2_df5e,
            crate::actors::choot::owned::Table::PatternYDistances => 0xa2_df6a,
        };
        self.loaded_table_word(address, index)
    }
    fn choot_record(&mut self, address: u16) -> sm_data::cartridge::RomRecord {
        self.native_actor_record(0xa2, address)
    }
    fn unknown_choot_function(&mut self) {
        self.unknown_function();
    }
}
impl crate::actors::zoa::owned::Host for PreparedActorState<'_> {
    fn zoa_table(&self, table: crate::actors::zoa::owned::Table, index: isize) -> u16 {
        let address = match table {
            crate::actors::zoa::owned::Table::InstructionLists => 0xa3_b40d,
            crate::actors::zoa::owned::Table::XSpeeds => 0xa3_b415,
        };
        self.loaded_table_word(address, index)
    }
    fn unknown_zoa_function(&mut self) {
        self.unknown_function();
    }
}
pub fn choot_ai<H: crate::actors::choot::owned::Host>(routine: ActorRoutine, host: &mut H) -> bool {
    match routine.0 {
        10674038 => crate::actors::choot::owned::initialize(host),
        10674222 => crate::actors::choot::owned::main_ai(host),
        _ => return false,
    }
    true
}
pub fn zoa_ai<H: crate::actors::zoa::owned::Host>(routine: ActorRoutine, host: &mut H) -> bool {
    match routine.0 {
        10728522 => crate::actors::zoa::owned::initialize(host),
        10728572 => crate::actors::zoa::owned::main_ai(host),
        _ => return false,
    }
    true
}
pub fn zoa_instruction<H: crate::actors::zoa::owned::Host>(
    routine: ActorRoutine,
    script: crate::enemy_script::EnemyScript,
    host: &mut H,
) -> Result<crate::enemy_script::EnemyScript, crate::enemy_script::EnemyScript> {
    let value = match routine.0 {
        10728489 => 4,
        10728500 => 8,
        10728511 => 12,
        _ => return Err(script),
    };
    Ok(crate::actors::zoa::owned::set_x_speed_index(
        value, script, host,
    ))
}

pub fn roach_ai<H: crate::actors::roach::owned::Host>(routine: ActorRoutine, host: &mut H) -> bool {
    match routine.0 {
        10723661 => crate::actors::roach::owned::initialize(host),
        10724048 => crate::actors::roach::owned::main_ai(host),
        _ => return false,
    }
    true
}
pub fn tripper_ai<H: crate::actors::tripper::owned::Host>(
    routine: ActorRoutine,
    host: &mut H,
) -> bool {
    use crate::actors::tripper::owned;
    match routine.0 {
        10722463 => owned::initialize(true, host),
        10722490 => owned::initialize(false, host),
        10722582 => owned::main_ai(host),
        10723080 => owned::shot(host),
        _ => return false,
    }
    true
}
pub fn sidehopper_ai<H: crate::actors::sidehopper::owned::Host>(
    routine: ActorRoutine,
    host: &mut H,
) -> bool {
    match routine.0 {
        10726153 => crate::actors::sidehopper::owned::initialize(host),
        10726351 => crate::actors::sidehopper::owned::main_ai(host),
        _ => return false,
    }
    true
}
pub fn tripper_instruction<H: crate::actors::tripper::owned::Host>(
    routine: ActorRoutine,
    script: EnemyScript,
    host: &mut H,
) -> Result<EnemyScript, EnemyScript> {
    match routine.0 {
        10722411 | 10722433 => Ok(crate::actors::tripper::owned::set_x_movement(
            false, script, host,
        )),
        10722422 | 10722444 => Ok(crate::actors::tripper::owned::set_x_movement(
            true, script, host,
        )),
        _ => Err(script),
    }
}
pub fn sidehopper_instruction<H: crate::actors::sidehopper::owned::Host>(
    routine: ActorRoutine,
    script: EnemyScript,
    host: &mut H,
) -> Result<EnemyScript, EnemyScript> {
    match routine.0 {
        10725992 => Ok(crate::actors::sidehopper::owned::queue_sound(script, host)),
        10726142 => Ok(crate::actors::sidehopper::owned::ready_to_hop(script, host)),
        _ => Err(script),
    }
}

impl PreparedActorState<'_> {
    /// Actual loaded little-endian vector samples, with each original complete
    /// two-byte bounds check before that sample is read. Full amplitude survives.
    pub fn ordinary_direction_vector(
        &mut self,
        phase: u16,
        amplitude: u16,
    ) -> crate::geometry::Vector32 {
        let sample = |offset: u16| {
            let index = usize::from(phase.wrapping_add(offset) & 0x7f);
            let base = usize::try_from(sm_data::cartridge::lorom_offset(0xa0_b7ee))
                .expect("direction table");
            let start = base
                .checked_add(index.checked_mul(2).expect("direction index"))
                .expect("direction offset");
            let end = start.checked_add(2).expect("direction extent");
            assert!(
                end <= self.allocations.loaded().len(),
                "direction allocation extent"
            );
            let word = u16::from_le_bytes([
                self.allocations.loaded().read(start),
                self.allocations.loaded().read(start + 1),
            ]);
            u32::from(word) * u32::from(amplitude)
        };
        let x = sample(0x40);
        let y = sample(0x80);
        crate::geometry::Vector32 { x, y }
    }
    /// Roll canonical timing at the same foreign-effect observation boundary.
    pub fn ordinary_random(&mut self) -> u16 {
        let value = crate::random::next(self.owner.gameplay.timing.random_number);
        self.owner.gameplay.timing.random_number = value;
        crate::session_access::SessionAccess::refresh_inputs(self);
        value
    }
    pub fn roach_table_word(
        &self,
        table: crate::actors::roach::owned::RoachTable,
        index: isize,
    ) -> u16 {
        use crate::actors::roach::owned::RoachTable::*;
        self.loaded_table_word(
            match table {
                InstructionLists => 0xa3_a111,
                ActivateFunctions => 0xa3_a121,
            },
            index,
        )
    }
    pub fn ordinary_speed_word(
        &self,
        table: sm_data::enemy_speeds::SpeedTable,
        index: isize,
    ) -> u16 {
        self.speed_word(table, index)
    }
    pub fn sidehopper_table_word(
        &self,
        table: crate::actors::sidehopper::owned::SidehopperTable,
        index: isize,
    ) -> u16 {
        use crate::actors::sidehopper::owned::SidehopperTable::*;
        self.loaded_table_word(
            match table {
                LandedUp => 0xa3_aac2,
                LandedDown => 0xa3_aaca,
                HoppingUp => 0xa3_aad2,
                HoppingDown => 0xa3_aada,
                HopFunctions => 0xa3_aae2,
                SmallDelta => 0xa3_aae6,
                SmallSpeed => 0xa3_aaea,
                SmallHeight => 0xa3_aaee,
                BigDelta => 0xa3_aaf2,
                BigSpeed => 0xa3_aaf6,
                BigHeight => 0xa3_aafa,
            },
            index,
        )
    }
    /// Definition lookup stays lazy and preserves the loaded low-window port.
    pub fn sidehopper_variant_word(&mut self, definition: u16) -> u16 {
        crate::actors::content_records::ContentSource::content_record(self, 0xa0, definition)
            .word(0x2a)
    }
    /// A phase invocation captures this immutable native allocation exactly once.
    pub fn capture_sidehopper_hop_speeds(&self) -> sm_data::cartridge::RomImage {
        self.allocations.native().clone()
    }
}

/// Actual owned Draygon grab/swoop/goop/death state steps. The prepared owner
/// still must bind the separate real initialization, intro and eye callbacks.
pub fn draygon_state_ai<H>(routine: ActorRoutine, k: u16, game: &mut H) -> bool
where
    H: crate::actors::draygon_death::owned::Host + crate::actors::draygon_sweep::owned::Host,
{
    use crate::actors::{draygon_death, draygon_flight, draygon_grab, draygon_sweep};
    match routine.0 {
        10848691 => {
            draygon_flight::owned::fire_wall_turret(game);
            draygon_sweep::owned::swoop_setup(false, game);
        }
        10850693 => draygon_death::owned::drift_to_death_spot(game),
        10850964 => draygon_death::owned::wait_for_evirs(game),
        10850987 => draygon_death::owned::buried_by_evirs(game),
        10849817 => draygon_grab::owned::chase_samus(game),
        10850064 => draygon_grab::owned::repelled_by_grapple(game),
        10850600 => draygon_grab::owned::flail_tail_and_fly_straight_up(game),
        10850644 => draygon_grab::owned::fly_straight_up(game),
        10848244 => draygon_sweep::owned::swoop_setup(true, game),
        10848433 => draygon_sweep::owned::swoop(true, false, game),
        10848546 => draygon_sweep::owned::swoop_apex(true, game),
        10848593 => draygon_sweep::owned::swoop(true, true, game),
        10848768 => draygon_sweep::owned::swoop(false, false, game),
        10848848 => draygon_sweep::owned::swoop_apex(false, game),
        10848912 => draygon_sweep::owned::swoop(false, true, game),
        10849034 => draygon_flight::owned::goop_setup(true, game),
        10849422 => draygon_flight::owned::goop_setup(false, game),
        10849106 => draygon_flight::owned::goop_move_until_in_range(true, game),
        10849492 => draygon_flight::owned::goop_move_until_in_range(false, game),
        10849198 => draygon_flight::owned::goop_firing(true, game),
        10849584 => draygon_flight::owned::goop_firing(false, game),
        10849331 => draygon_flight::owned::goop_move_until_off_screen(true, game),
        10849714 => draygon_flight::owned::goop_move_until_off_screen(false, game),
        10850078 => draygon_grab::owned::move_to_target(k, game),
        10850262 => draygon_grab::owned::rising_spiral(k, game),
        10850516 => draygon_grab::owned::tail_whip(k, game),
        10850565 => draygon_grab::owned::final_tail_whips(k, game),
        10850596 => draygon_grab::owned::move_samus_with_draygon(k, game),
        _ => return false,
    }
    true
}
pub fn draygon_reaction_ai<H: crate::actors::draygon_death::owned::Host>(
    routine: ActorRoutine,
    game: &mut H,
) -> bool {
    use crate::actors::{draygon_death, draygon_flight};
    match routine.0 {
        10851661 => draygon_death::owned::hurt_ai(game),
        10851824 => draygon_death::owned::shot(game),
        10851818 => draygon_death::owned::touch(game),
        10851847 => draygon_death::owned::power_bomb(game),
        10864025 => draygon_flight::owned::initialize_tail_or_arms(false, game),
        10864045 => draygon_flight::owned::initialize_tail_or_arms(true, game),
        _ => return false,
    }
    true
}
pub fn draygon_instruction<H: crate::actors::draygon_death::owned::Host>(
    routine: ActorRoutine,
    script: EnemyScript,
    game: &mut H,
) -> Result<EnemyScript, EnemyScript> {
    use crate::actors::{draygon_death, draygon_flight, draygon_grab};
    Ok(match routine.0 {
        10852159 => draygon_death::owned::spawn_dying_sprite(21, script, game),
        10852178 => draygon_death::owned::spawn_dying_sprite(3, script, game),
        10852197 => draygon_death::owned::spawn_dying_sprite(29, script, game),
        10852216 => draygon_death::owned::spawn_dying_sprite(24, script, game),
        10854268 | 10854318 => {
            draygon_death::owned::spawn_goop(routine.0 == 10854318, script, game)
        }
        10851549 => draygon_flight::owned::set_instruction_lists(script, game),
        10852563 => draygon_flight::owned::paralyse_tail_and_arms(script, game),
        10852591 => draygon_flight::owned::set_intangible(script, game),
        10853274 => draygon_grab::owned::tail_whip_hit(script, game),
        _ => return Err(script),
    })
}

impl PreparedActorState<'_> {
    pub(crate) fn current_word(&self, at: usize) -> u16 {
        u16::from_le_bytes([self.current_byte(at), self.current_byte(at + 1)])
    }
    pub(crate) fn write_current_word_le(&mut self, at: usize, value: u16) {
        self.write_current_slice(at, &value.to_le_bytes());
    }
}
#[path = "projectile_access.rs"]
mod projectile_access;

impl PreparedActorState<'_> {
    pub(crate) fn projectile_record(
        &mut self,
        source: ProjectileRecordSource,
        bank: u8,
        address: u16,
    ) -> sm_data::cartridge::RomRecord {
        if address & 0x8000 == 0 {
            return self.aliases.low_window_projectile_record(
                self.owner,
                self.allocations,
                source,
                bank,
                address,
            );
        }
        let image = match source {
            ProjectileRecordSource::PlayerLoaded | ProjectileRecordSource::EnemyLoaded => {
                self.allocations.loaded().clone()
            }
            ProjectileRecordSource::PlayerNative | ProjectileRecordSource::EnemyNative => {
                self.allocations.native().clone()
            }
        };
        sm_data::cartridge::RomRecord {
            image,
            base: usize::try_from(sm_data::cartridge::lorom_offset(
                (u32::from(bank) << 16) | u32::from(address),
            ))
            .expect("projectile record offset"),
        }
    }
    pub(crate) fn current_native_word(&self, at: usize) -> u16 {
        let next = at.checked_add(1).expect("native word extent");
        u16::from_ne_bytes([self.current_byte(at), self.current_byte(next)])
    }
}
impl crate::cartridge::LoadedImage for PreparedActorState<'_> {
    fn loaded_image(&self) -> &sm_data::cartridge::RomImage {
        self.allocations.loaded()
    }
}
impl crate::cartridge::NativeImage for PreparedActorState<'_> {
    fn native_image(&self) -> sm_data::cartridge::RomImage {
        self.allocations.native().clone()
    }
}
impl crate::records::Records for PreparedActorState<'_> {
    fn records(&self) -> &crate::records::RomRecords {
        &self.allocations.records
    }
}
impl crate::player_position::PlayerPosition for PreparedActorState<'_> {
    fn player_word(&self, word: crate::player_position::PlayerWord) -> u16 {
        word.get(&self.owner.gameplay.player)
    }
    fn set_player_word(&mut self, word: crate::player_position::PlayerWord, value: u16) {
        use crate::player_position::PlayerWord::*;
        for (lane, value) in value.to_le_bytes().into_iter().enumerate() {
            let player = &mut self.owner.gameplay.player;
            let field = match word {
                PeriodicDamage => &mut player.periodic_damage,
                X => &mut player.x,
                XFraction => &mut player.x_fraction,
                Y => &mut player.y,
                YFraction => &mut player.y_fraction,
                YRadius => &mut player.contact.radius,
                PreviousX => &mut player.previous_position.x.whole,
                PreviousXFraction => &mut player.previous_position.x.fraction,
                PreviousY => &mut player.previous_position.y.whole,
                PreviousYFraction => &mut player.previous_position.y.fraction,
                Invincibility => &mut player.invincibility,
                Knockback => &mut player.knockback_timer,
            };
            let mut bytes = field.to_le_bytes();
            bytes[lane] = value;
            *field = u16::from_le_bytes(bytes);
            crate::session_access::SessionAccess::refresh_inputs(self);
        }
    }
}
impl crate::player_position::PlayerFacts for PreparedActorState<'_> {
    fn facing(&self) -> u8 {
        self.owner.gameplay.player.movement_and_facing.to_le_bytes()[0]
    }
    fn movement_type(&self) -> u8 {
        self.owner.gameplay.player.movement_and_facing.to_le_bytes()[1]
    }
    fn grapple_angle(&self) -> u8 {
        self.owner.gameplay.player.grapple_angle.to_le_bytes()[1]
    }
    fn grapple_mirrored_angle(&self) -> u8 {
        self.owner
            .gameplay
            .player
            .swing
            .mirrored_angle
            .to_le_bytes()[1]
    }
}
impl crate::sprite_effect_host::Aliases for PreparedActorState<'_> {
    fn sprite_word(
        &self,
        member: crate::sprite_effect_host::SpriteMember,
        index: crate::sprite_effect_state::LegacySpriteWordIndex,
    ) -> u16 {
        self.typed_sprite_word(member, index.0)
            .unwrap_or_else(|| self.current_word(member.address(index)))
    }
    fn set_sprite_word_le_slice(
        &mut self,
        member: crate::sprite_effect_host::SpriteMember,
        index: crate::sprite_effect_state::LegacySpriteWordIndex,
        value: u16,
    ) {
        if index.0 < 64 {
            for (lane, value) in value.to_le_bytes().into_iter().enumerate() {
                assert!(self.store_typed_sprite_byte(member, index.0, lane, value));
                crate::session_access::SessionAccess::refresh_inputs(self);
            }
        } else {
            self.write_current_slice(member.address(index), &value.to_le_bytes());
        }
    }
    fn sprite_word_native(
        &self,
        member: crate::sprite_effect_host::SpriteMember,
        index: crate::sprite_effect_state::LegacySpriteWordIndex,
    ) -> u16 {
        self.typed_sprite_word(member, index.0)
            .map(|value| u16::from_ne_bytes(value.to_le_bytes()))
            .unwrap_or_else(|| self.current_native_word(member.address(index)))
    }
    fn set_sprite_word_native_slice(
        &mut self,
        member: crate::sprite_effect_host::SpriteMember,
        index: crate::sprite_effect_state::LegacySpriteWordIndex,
        value: u16,
    ) {
        if index.0 < 64 {
            for (lane, value) in value.to_ne_bytes().into_iter().enumerate() {
                assert!(self.store_typed_sprite_byte(member, index.0, lane, value));
                crate::session_access::SessionAccess::refresh_inputs(self);
            }
        } else {
            self.write_current_slice(member.address(index), &value.to_ne_bytes());
        }
    }
}
impl crate::sprite_effect_host::ControllerInputs for PreparedActorState<'_> {
    fn attachment_word(&self, at: usize) -> u16 {
        self.current_native_word(at)
    }
    fn attachment_x(&self, enemy: u16) -> u16 {
        if enemy < 2048 && enemy.is_multiple_of(64) {
            self.owner.gameplay.actors.slots[usize::from(enemy) / 64]
                .x
                .whole
        } else {
            self.current_native_word(0xf78 + usize::from(enemy) + 2)
        }
    }
    fn attachment_y(&self, enemy: u16) -> u16 {
        if enemy < 2048 && enemy.is_multiple_of(64) {
            self.owner.gameplay.actors.slots[usize::from(enemy) / 64]
                .y
                .whole
        } else {
            self.current_native_word(0xf78 + usize::from(enemy) + 6)
        }
    }
    fn attachment_frozen_timer(&self, enemy: u16) -> u16 {
        if enemy < 2048 && enemy.is_multiple_of(64) {
            self.owner.gameplay.actors.slots[usize::from(enemy) / 64].frozen
        } else {
            self.current_native_word(0xf78 + usize::from(enemy) + 0x26)
        }
    }
}

#[path = "prepared_species.rs"]
mod prepared_species;

#[path = "../room_mechanisms/owned_bindings.rs"]
mod mechanism_bindings;
#[path = "../player_effects/owned_bindings.rs"]
pub mod player_bindings;
#[path = "../enemy_projectiles/owned_bindings.rs"]
mod projectile_bindings;
pub use player_bindings::PlayerPorts;

impl PreparedActorState<'_> {
    /// Every operation retains its completed typed prefix and poisons the one
    /// current owner on unwind. There is no export, second Session or new clock.
    pub(crate) fn guarded_actor_call<R>(&mut self, operation: impl FnOnce(&mut Self) -> R) -> R {
        let outcome = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| operation(self)));
        match outcome {
            Ok(value) => value,
            Err(payload) => {
                self.owner.poison();
                std::panic::resume_unwind(payload)
            }
        }
    }
    pub(crate) fn mechanism_queue_slice(&mut self, at: usize, bytes: &[u8]) {
        let end = at
            .checked_add(bytes.len())
            .expect("mechanism queue slice extent");
        assert!(at <= end && end <= 0x20000);
        self.guarded_actor_call(|host| {
            for (offset, value) in bytes.iter().copied().enumerate() {
                let address = at + offset;
                if (0xd0..0x2d0).contains(&address) {
                    host.presentation
                        .queue_byte(
                            &mut host.owner.gameplay,
                            address - 0xd0,
                            value,
                            host.aliases.presentation_diagnostics(),
                        )
                        .expect("mechanism queue byte publication");
                    crate::session_access::SessionAccess::refresh_inputs(host);
                } else {
                    host.set_current_byte(address, value);
                }
            }
        });
    }
    pub(crate) fn projectile_draw(&mut self, drawing: crate::player_projectiles::host::Drawing) {
        self.guarded_actor_call(|host| {
            host.presentation
                .projectile_draw(
                    &mut host.owner.gameplay,
                    &host.owner.observations,
                    drawing,
                    &host.samus_content,
                    host.aliases.presentation_diagnostics(),
                )
                .expect("projectile draw publication");
        });
    }
    pub(crate) fn projectile_draw_trail(&mut self, pair: usize, right: bool, frozen: bool) {
        self.guarded_actor_call(|host| {
            host.presentation
                .draw_trail(
                    &mut host.owner.gameplay,
                    pair,
                    right,
                    frozen,
                    host.aliases.presentation_diagnostics(),
                )
                .expect("projectile trail publication");
        });
    }
    pub(crate) fn projectile_draw_enemy(
        &mut self,
        offscreen: bool,
        bank: u8,
        map: u16,
        position: crate::geometry::Point,
        tile: u16,
        graphics: u16,
    ) {
        self.guarded_actor_call(|host| {
            host.presentation
                .draw_enemy(
                    &mut host.owner.gameplay,
                    crate::scene::actor_presentation::EnemySpriteRequest {
                        offscreen,
                        bank,
                        map,
                        position,
                        tile,
                        graphics,
                    },
                    host.aliases.presentation_diagnostics(),
                )
                .expect("enemy projectile publication");
        });
    }
    pub(crate) fn draw_current_actor_visual(
        &mut self,
        request: crate::actors::scene::ActorDrawRequest,
    ) {
        self.guarded_actor_call(|host| {
            host.presentation
                .draw(
                    &mut host.owner.gameplay,
                    request,
                    host.aliases.presentation_diagnostics(),
                )
                .expect("actor visual publication");
        });
    }
    pub(crate) fn copy_current_actor_tilemap(
        &mut self,
        source: crate::actors::scene::ActorTilemapSource,
    ) {
        self.guarded_actor_call(|host| {
            host.presentation
                .copy_tilemap(
                    &mut host.owner.gameplay,
                    source,
                    host.aliases.presentation_diagnostics(),
                )
                .expect("actor tilemap publication");
        });
    }
    pub(crate) fn copy_current_actor_chunk(
        &mut self,
        source: crate::actors::scene::ActorTileChunkSource,
        destination: crate::actors::scene::ActorTileDestination,
    ) {
        self.guarded_actor_call(|host| {
            host.presentation
                .copy_chunk(
                    &mut host.owner.gameplay,
                    source,
                    destination,
                    host.aliases.presentation_diagnostics(),
                )
                .expect("actor chunk publication");
        });
    }
}
impl crate::enemy_projectiles::host::Graphics for PreparedActorState<'_> {
    fn vram_write_queue_tail(&self) -> u16 {
        self.owner.gameplay.queued_tilemaps.tail
    }
    fn set_vram_write_queue_tail(&mut self, value: u16) {
        self.guarded_actor_call(|host| {
            host.presentation
                .publish_queue_tail(&mut host.owner.gameplay, value)
                .expect("actor queue tail publication");
            crate::session_access::SessionAccess::refresh_inputs(host);
        });
    }
    fn set_vram_queue_size(&mut self, cursor: u16, value: u16) {
        self.actor_queue_word(cursor, 0, value);
    }
    fn set_vram_queue_source(&mut self, cursor: u16, value: u16) {
        self.actor_queue_word(cursor, 2, value);
    }
    fn set_vram_queue_bank_word(&mut self, cursor: u16, value: u16) {
        self.actor_queue_word(cursor, 4, value);
    }
    fn set_vram_queue_destination(&mut self, cursor: u16, value: u16) {
        self.actor_queue_word(cursor, 5, value);
    }
    fn set_bg2_y_scroll(&mut self, value: u16) {
        self.owner.gameplay.room.camera.bg2_scroll[1] = value;
    }
    fn set_palette_buffer(&mut self, index: isize, value: u16) {
        self.guarded_actor_call(|host| {
            host.presentation
                .palette_word(
                    &mut host.owner.gameplay,
                    index,
                    value,
                    host.aliases.presentation_diagnostics(),
                )
                .expect("actor palette publication");
        });
    }
}
impl PreparedActorState<'_> {
    fn actor_queue_word(&mut self, cursor: u16, field: usize, value: u16) {
        self.guarded_actor_call(|host| {
            host.presentation
                .queue_word(
                    &mut host.owner.gameplay,
                    cursor,
                    field,
                    value,
                    host.aliases.presentation_diagnostics(),
                )
                .expect("actor queue word publication");
            crate::session_access::SessionAccess::refresh_inputs(host);
        });
    }
}
impl crate::sprite_effect_host::Effects for PreparedActorState<'_> {
    fn submit_sprite(&mut self, request: crate::sprite_effect_state::SpriteDraw) {
        self.guarded_actor_call(|host| {
            host.presentation
                .submit_sprite(
                    &mut host.owner.gameplay,
                    request,
                    host.aliases.presentation_diagnostics(),
                )
                .expect("sprite publication");
        });
    }
    fn invalid_sprite_instruction(&mut self) {
        self.unknown_function();
    }
}
impl crate::sprite_effect_host::Host for PreparedActorState<'_> {}

#[path = "prepared_collections.rs"]
mod prepared_collections;

#[path = "prepared_sprite_words.rs"]
mod prepared_sprite_words;
