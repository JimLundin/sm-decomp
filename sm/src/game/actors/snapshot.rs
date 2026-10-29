//! Paired timeline/presentation snapshots; live handle epochs never rewind.
#![forbid(unsafe_code)]
use super::{
    content::GameplayRevision,
    identity::{ActorIdentityError, ActorIdentityState, ActorTimeline},
    media::MediaRevision,
};
use crate::game::room_entry::world::WorldBinding;
use crate::game::scene::component::DisplayStartMetadata;
use crate::game::session::{Point, Position};
use sm_game::actors::state::*;
use std::sync::Arc;

// The declaration is both the writer and its layout. Layout expansion never
// evaluates a value expression or needs an initialized actor/world allocation.
macro_rules! checkpoint_layout {
    ($layout:ident, $write:ident($bytes:ident $(, $arg:ident: $arg_ty:ty)*) { $($fields:tt)* }) => {
        #[cfg(any(feature = "frontend", sm_verify_dll))]
        fn $layout() -> String {
            let mut layout = String::new();
            checkpoint_layout!(@layout layout; $($fields)*);
            layout
        }
        fn $write($bytes: &mut Vec<u8>, $($arg: $arg_ty),*) {
            checkpoint_layout!(@write $bytes; $($fields)*);
        }
    };
    (@$mode:ident $out:ident;) => {};
    (@write $out:ident; bind $pattern:pat_param = $value:expr; $($rest:tt)*) => {
        let $pattern = $value;
        checkpoint_layout!(@write $out; $($rest)*);
    };
    (@layout $out:ident; bind $pattern:pat_param = $value:expr; $($rest:tt)*) => {
        checkpoint_layout!(@layout $out; $($rest)*);
    };
    (@write $out:ident; field $name:ident: $ty:ty = $value:expr; $($rest:tt)*) => {
        let value: $ty = $value;
        $out.extend_from_slice(&value.to_le_bytes());
        checkpoint_layout!(@write $out; $($rest)*);
    };
    (@layout $out:ident; field $name:ident: $ty:ty = $value:expr; $($rest:tt)*) => {
        $out.push_str(concat!(stringify!($name), ":", stringify!($ty), "le;"));
        checkpoint_layout!(@layout $out; $($rest)*);
    };
    (@write $out:ident; fixed $name:ident: $ty:ty = $value:expr; $($rest:tt)*) => {
        let value: $ty = $value;
        $out.extend_from_slice(&value);
        checkpoint_layout!(@write $out; $($rest)*);
    };
    (@layout $out:ident; fixed $name:ident: $ty:ty = $value:expr; $($rest:tt)*) => {
        $out.push_str(concat!(stringify!($name), ":", stringify!($ty), ";"));
        checkpoint_layout!(@layout $out; $($rest)*);
    };
    (@write $out:ident; magic $name:ident = $value:literal; $($rest:tt)*) => {
        $out.extend_from_slice($value);
        checkpoint_layout!(@write $out; $($rest)*);
    };
    (@layout $out:ident; magic $name:ident = $value:literal; $($rest:tt)*) => {
        $out.push_str(concat!(stringify!($name), ":", stringify!($value), ";"));
        checkpoint_layout!(@layout $out; $($rest)*);
    };
    (@write $out:ident; boolean $name:ident = $value:expr; $($rest:tt)*) => {
        let value: bool = $value;
        $out.push(u8::from(value));
        checkpoint_layout!(@write $out; $($rest)*);
    };
    (@layout $out:ident; boolean $name:ident = $value:expr; $($rest:tt)*) => {
        $out.push_str(concat!(stringify!($name), ":bool-u8(0,1);"));
        checkpoint_layout!(@layout $out; $($rest)*);
    };
    (@write $out:ident; option $name:ident: [u8; $size:literal] = $value:expr; $($rest:tt)*) => {
        let value: Option<[u8; $size]> = $value;
        $out.push(u8::from(value.is_some()));
        if let Some(value) = value { $out.extend_from_slice(&value); }
        checkpoint_layout!(@write $out; $($rest)*);
    };
    (@write $out:ident; option $name:ident: u32 = $value:expr; $($rest:tt)*) => {
        let value: Option<u32> = $value;
        $crate::game::actors::snapshot::optional_id($out, value);
        checkpoint_layout!(@write $out; $($rest)*);
    };
    (@layout $out:ident; option $name:ident: u32 = $value:expr; $($rest:tt)*) => {
        $out.push_str(concat!(stringify!($name), ":option-u8(0:none,1:u32le);"));
        checkpoint_layout!(@layout $out; $($rest)*);
    };
    (@layout $out:ident; option $name:ident: $ty:ty = $value:expr; $($rest:tt)*) => {
        $out.push_str(concat!(stringify!($name), ":option-u8(0:none,1:", stringify!($ty), ");"));
        checkpoint_layout!(@layout $out; $($rest)*);
    };
    (@write $out:ident; class $name:ident = $value:expr; $($rest:tt)*) => {
        let value: Option<usize> = $value;
        $out.push(value.map_or(u8::MAX, |index| index as u8));
        checkpoint_layout!(@write $out; $($rest)*);
    };
    (@layout $out:ident; class $name:ident = $value:expr; $($rest:tt)*) => {
        $out.push_str(concat!(stringify!($name), ":class-u8(255:none);"));
        checkpoint_layout!(@layout $out; $($rest)*);
    };
    (@write $out:ident; array $name:ident: $ty:ty = ($value:expr) => ($pattern:pat_param) { $($body:tt)* } $($rest:tt)*) => {
        let values: &$ty = &$value;
        for $pattern in values.iter().enumerate() { checkpoint_layout!(@write $out; $($body)*); }
        checkpoint_layout!(@write $out; $($rest)*);
    };
    (@layout $out:ident; array $name:ident: $ty:ty = ($value:expr) => ($pattern:pat_param) { $($body:tt)* } $($rest:tt)*) => {
        $out.push_str(concat!(stringify!($name), ":", stringify!($ty), "{"));
        checkpoint_layout!(@layout $out; $($body)*);
        $out.push_str("};");
        checkpoint_layout!(@layout $out; $($rest)*);
    };
    (@write $out:ident; repeat $name:ident by $count:ident = ($value:expr) => ($pattern:pat_param) { $($body:tt)* } $($rest:tt)*) => {
        for $pattern in $value { checkpoint_layout!(@write $out; $($body)*); }
        checkpoint_layout!(@write $out; $($rest)*);
    };
    (@layout $out:ident; repeat $name:ident by $count:ident = ($value:expr) => ($pattern:pat_param) { $($body:tt)* } $($rest:tt)*) => {
        $out.push_str(concat!(stringify!($name), ":repeat(", stringify!($count), "){"));
        checkpoint_layout!(@layout $out; $($body)*);
        $out.push_str("};");
        checkpoint_layout!(@layout $out; $($rest)*);
    };
    (@write $out:ident; variants $name:ident = ($value:expr) { $($tag:literal $variant:ident ($pattern:pat_param) => { $($body:tt)* })* } $($rest:tt)*) => {
        match $value { $($pattern => { $out.push($tag); checkpoint_layout!(@write $out; $($body)*); })* }
        checkpoint_layout!(@write $out; $($rest)*);
    };
    (@layout $out:ident; variants $name:ident = ($value:expr) { $($tag:literal $variant:ident ($pattern:pat_param) => { $($body:tt)* })* } $($rest:tt)*) => {
        $out.push_str(concat!(stringify!($name), ":tag-u8{"));
        $($out.push_str(concat!(stringify!($tag), ":", stringify!($variant), "{"));
          checkpoint_layout!(@layout $out; $($body)*); $out.push_str("};");)*
        $out.push_str("};");
        checkpoint_layout!(@layout $out; $($rest)*);
    };
    (@write $out:ident; blob $name:ident = $value:expr; $($rest:tt)*) => {
        let image: &crate::rom::RomImage = $value;
        $out.extend_from_slice(&(image.len() as u64).to_le_bytes());
        $out.extend((0..image.len()).map(|at| image.read(at)));
        checkpoint_layout!(@write $out; $($rest)*);
    };
    (@layout $out:ident; blob $name:ident = $value:expr; $($rest:tt)*) => {
        $out.push_str(concat!(stringify!($name), ":{length:u64le;bytes:[u8;length]};"));
        checkpoint_layout!(@layout $out; $($rest)*);
    };
    (@write $out:ident; nested $name:ident: $ty:ty = $value:expr; $($rest:tt)*) => {
        $out.extend_from_slice($value);
        checkpoint_layout!(@write $out; $($rest)*);
    };
    (@layout $out:ident; nested $name:ident: $ty:ty = $value:expr; $($rest:tt)*) => {
        $out.push_str(concat!(stringify!($name), ":{"));
        $out.push_str(&<$ty>::serialized_layout());
        $out.push_str("};");
        checkpoint_layout!(@layout $out; $($rest)*);
    };
}
pub(super) use checkpoint_layout;

impl ActorIdentityState {
    /// Owned Session payload: explicit numeric occupancy, immutable bindings and
    /// palette allocation roles. No RAM image or external handle epoch is encoded.
    pub(in crate::game) fn encode_owned_checkpoint(
        &self,
        actors: &Actors,
        world: &Arc<WorldBinding>,
        metadata: &DisplayStartMetadata,
        loaded: Option<&crate::rom::RomImage>,
        native: Option<&crate::rom::RomImage>,
        sources: &[(
            crate::game::room_entry::world::ResourceId,
            crate::rom::RomImage,
        )],
    ) -> Result<Vec<u8>, String> {
        let payload = ActorCheckpointPayload::capture(self, world)?;
        if metadata.identity.reference != metadata.reference.fingerprint {
            return Err("actor owned checkpoint Reference identity mismatch".into());
        }
        let binding = crate::game::checkpoint_binding::observation_binding(metadata);
        let body = payload.encode_occupancy(loaded, native, sources, &|index| {
            actors.slots[index].definition != 0
        })?;
        let mut bytes = Vec::from(b"ACTOROWN1".as_slice());
        bytes.extend_from_slice(&(binding.len() as u64).to_le_bytes());
        bytes.extend_from_slice(&binding);
        bytes.extend_from_slice(&body);
        Ok(bytes)
    }

    /// Validate completely against decoded numeric Actors and supplied sources,
    /// then return a detached replacement with this owner's checked fresh epoch.
    /// Failure never mutates the current owner or supplied allocation contents.
    pub(in crate::game) fn decode_owned_checkpoint(
        bytes: &[u8],
        current: &Self,
        actors: &Actors,
        world: &Arc<WorldBinding>,
        metadata: &DisplayStartMetadata,
        loaded: Option<&crate::rom::RomImage>,
        native: Option<&crate::rom::RomImage>,
        sources: &[(
            crate::game::room_entry::world::ResourceId,
            crate::rom::RomImage,
        )],
    ) -> Result<Self, String> {
        current
            .checked_next_epoch()
            .map_err(|error| format!("{error:?}"))?;
        let mut read = CheckpointReader(bytes);
        if read.bytes(9)? != b"ACTOROWN1" {
            return Err("unsupported actor owned checkpoint schema".into());
        }
        let size = usize::try_from(read.counter()?)
            .map_err(|_| "actor owned checkpoint binding length overflow")?;
        if metadata.identity.reference != metadata.reference.fingerprint
            || read.bytes(size)? != crate::game::checkpoint_binding::observation_binding(metadata)
        {
            return Err("actor owned checkpoint source identity or Reference changed".into());
        }
        let prepared = ActorCheckpointPayload::prepare_occupancy(
            read.0,
            current,
            world,
            loaded,
            native,
            sources,
            &|index| actors.slots[index].definition != 0,
            None,
        )?;
        Ok(Self::from_checkpoint(
            prepared.next_live_epoch,
            prepared.timeline,
            prepared.residency,
        ))
    }
}

// Numeric actors are a separate component from the identity timeline and media
// allocation payload below. Neither component alone is a complete Session.
trait NumericCheckpoint: Sized {
    fn write(&self, bytes: &mut Vec<u8>);
    fn read(read: &mut CheckpointReader<'_>) -> Result<Self, String>;
}
impl NumericCheckpoint for u16 {
    fn write(&self, bytes: &mut Vec<u8>) {
        bytes.extend_from_slice(&self.to_le_bytes());
    }
    fn read(read: &mut CheckpointReader<'_>) -> Result<Self, String> {
        read.word()
    }
}
impl NumericCheckpoint for u8 {
    fn write(&self, bytes: &mut Vec<u8>) {
        bytes.push(*self);
    }
    fn read(read: &mut CheckpointReader<'_>) -> Result<Self, String> {
        read.byte()
    }
}
impl<T: NumericCheckpoint, const N: usize> NumericCheckpoint for [T; N] {
    fn write(&self, bytes: &mut Vec<u8>) {
        for value in self {
            value.write(bytes);
        }
    }
    fn read(read: &mut CheckpointReader<'_>) -> Result<Self, String> {
        let mut values = Vec::with_capacity(N);
        for _ in 0..N {
            values.push(T::read(read)?);
        }
        values
            .try_into()
            .map_err(|_| "actor numeric checkpoint array count".into())
    }
}
// Each table constructs every field explicitly; adding state fails compilation
// until its codec is extended. No Rust layout or RAM addresses are serialized.
macro_rules! numeric_checkpoint {
    ($kind:ty, $($field:ident),+ $(,)?) => {
        impl NumericCheckpoint for $kind {
            fn write(&self, bytes: &mut Vec<u8>) { $(self.$field.write(bytes);)+ }
            fn read(read: &mut CheckpointReader<'_>) -> Result<Self, String> {
                Ok(Self { $($field: NumericCheckpoint::read(read)?,)+ })
            }
        }
    };
}
numeric_checkpoint!(Position, whole, fraction);
numeric_checkpoint!(Point, x, y);
numeric_checkpoint!(
    Actor,
    definition,
    x,
    y,
    radius,
    properties,
    extra_properties,
    ai_handlers,
    health,
    spritemap,
    timer,
    instruction,
    instruction_delay,
    palette,
    tile,
    layer,
    flash,
    frozen,
    invincibility,
    shake,
    frame,
    bank,
    hurt_time,
    variables,
    pre_instruction,
    parameters
);
numeric_checkpoint!(
    SpawnRecord,
    contact_mode,
    death_cause,
    scratch_04,
    tile,
    palette,
    scratch_0a,
    draw_offset,
    scratch_14,
    definition,
    position,
    instruction,
    properties,
    extra_properties,
    parameters,
    name
);
numeric_checkpoint!(
    ActorControls,
    phase_counter,
    selection_visits,
    population_scratch,
    compatibility_new_index,
    first_free,
    population_count,
    selected,
    compatibility_selected_backup,
    compatibility_data_pointer,
    shake_request,
    projectile_contacts_disabled,
    process_all
);
numeric_checkpoint!(ActorCombat, last_power);
numeric_checkpoint!(
    ActorStaging,
    descriptor_prefix,
    descriptor_write_position,
    transfer_source,
    transfer_destination,
    gap
);
numeric_checkpoint!(
    ActorGraphics,
    definitions,
    tile_offsets,
    palette_routes,
    compatibility_write_position,
    compatibility_next_tile
);
numeric_checkpoint!(
    ActorDrawing,
    hook_address,
    hook_work,
    tilemap_size,
    tilemap_pending
);
numeric_checkpoint!(
    ActorSchedule,
    boss_id,
    compatibility_179e,
    active_write_position,
    interactive_write_position,
    compatibility_active_cursor,
    compatibility_interactive_cursor,
    active,
    interactive,
    colliding,
    compatibility_distances
);
numeric_checkpoint!(
    ActorCompatibility,
    pre_hook,
    post_hook,
    debug_enemy_index,
    log_index
);
numeric_checkpoint!(
    ActorDiagnostics,
    index,
    freeze,
    text,
    population_entry,
    population_key,
    spawn,
    gap,
    sprite_contacts_disabled
);
numeric_checkpoint!(
    Actors,
    slots,
    saved,
    controls,
    combat,
    staging,
    graphics,
    drawing,
    schedule,
    compatibility,
    diagnostics
);
/// The actor table's numeric checkpoint, bound to its display start metadata.
pub(in crate::game) trait ActorsCheckpoint: Sized {
    fn encode_numeric_checkpoint(&self, metadata: &DisplayStartMetadata)
    -> Result<Vec<u8>, String>;
    fn decode_numeric_checkpoint(
        bytes: &[u8],
        metadata: &DisplayStartMetadata,
    ) -> Result<Self, String>;
}
impl ActorsCheckpoint for Actors {
    fn encode_numeric_checkpoint(
        &self,
        metadata: &DisplayStartMetadata,
    ) -> Result<Vec<u8>, String> {
        if metadata.identity.reference != metadata.reference.fingerprint {
            return Err("actor numeric checkpoint Reference identity mismatch".into());
        }
        let mut bytes = Vec::from(b"SMACTORS\x01".as_slice());
        let binding = crate::game::checkpoint_binding::observation_binding(metadata);
        bytes.extend_from_slice(&(binding.len() as u64).to_le_bytes());
        bytes.extend_from_slice(&binding);
        self.write(&mut bytes);
        Ok(bytes)
    }
    fn decode_numeric_checkpoint(
        bytes: &[u8],
        metadata: &DisplayStartMetadata,
    ) -> Result<Self, String> {
        let mut read = CheckpointReader(bytes);
        if read.bytes(9)? != b"SMACTORS\x01" {
            return Err("unsupported actor numeric checkpoint schema".into());
        }
        let size = usize::try_from(read.counter()?)
            .map_err(|_| "actor numeric checkpoint binding length overflow")?;
        if metadata.identity.reference != metadata.reference.fingerprint
            || read.bytes(size)? != crate::game::checkpoint_binding::observation_binding(metadata)
        {
            return Err("actor numeric checkpoint source identity or Reference changed".into());
        }
        let actors = Self::read(&mut read)?;
        if !read.0.is_empty() {
            return Err("trailing actor numeric checkpoint fields".into());
        }
        Ok(actors)
    }
}

#[cfg(sm_verify_dll)]
#[path = "../../verification/actor_numeric_checkpoint_cases.rs"]
mod numeric_checkpoint_cases;
#[cfg(sm_verify_dll)]
pub(crate) use numeric_checkpoint_cases::assert_numeric_checkpoint;

#[derive(Clone, Debug, PartialEq)]
pub(crate) struct ActorSnapshotPayload {
    timeline: ActorTimeline,
    binding: Arc<WorldBinding>,
    gameplay: GameplayRevision,
    media: MediaRevision,
    residency: super::media::ActorPaletteResidency,
}
/// Constructed only after binding/payload/checked-next-epoch preflight, before RAM writes.
pub(crate) struct PreparedActorRestore {
    next_live_epoch: u64,
    timeline: ActorTimeline,
    residency: super::media::ActorPaletteResidency,
}
impl ActorSnapshotPayload {
    pub(crate) fn capture(state: &ActorIdentityState, world: &Arc<WorldBinding>) -> Self {
        assert!(
            !state.is_poisoned(),
            "poisoned actors cannot produce a snapshot"
        );
        Self {
            timeline: state.timeline().clone(),
            residency: state.palette_residency().clone(),
            binding: world.clone(),
            gameplay: world
                .actor_gameplay_revision()
                .unwrap_or(GameplayRevision([0; 32])),
            media: world
                .actor_media_revision()
                .unwrap_or(MediaRevision([0; 32])),
        }
    }

    pub(crate) fn preflight(
        &self,
        current: &ActorIdentityState,
        world: &Arc<WorldBinding>,
    ) -> Result<PreparedActorRestore, ActorIdentityError> {
        if !Arc::ptr_eq(&self.binding, world)
            || self.gameplay
                != world
                    .actor_gameplay_revision()
                    .unwrap_or(GameplayRevision([0; 32]))
            || self.media
                != world
                    .actor_media_revision()
                    .unwrap_or(MediaRevision([0; 32]))
        {
            return Err(ActorIdentityError::BindingMismatch);
        }
        Ok(PreparedActorRestore::new(
            current.checked_next_epoch()?,
            self.timeline.clone(),
            self.residency.clone(),
        ))
    }
}
impl PreparedActorRestore {
    pub(super) fn new(
        next_live_epoch: u64,
        timeline: ActorTimeline,
        residency: super::media::ActorPaletteResidency,
    ) -> Self {
        Self {
            next_live_epoch,
            timeline,
            residency,
        }
    }

    /// Noninterleaved with preflight and the paired RAM adoption; no fallible work remains.
    pub(crate) fn commit(self, current: &mut ActorIdentityState) {
        current.replace_timeline(self.next_live_epoch, self.timeline, self.residency);
    }
}

/// Game-owned companions to CPU snapshots: never embedded in raw-copied Snapshot.
#[derive(Clone, Default)]
pub(crate) struct ActorSnapshotSidecars {
    pub(crate) before: Option<ActorSnapshotPayload>,
    pub(crate) theirs: Option<ActorSnapshotPayload>,
    pub(crate) mine: Option<ActorSnapshotPayload>,
}
/// Paired semantic payload for a replay/checkpoint RAM image, with existing CPU authority.
#[derive(Clone)]
pub(crate) struct ActorCheckpointPayload {
    pub(crate) actors: ActorSnapshotPayload,
}

/// Bounded cursor shared by the actor metadata owners; never reads live RAM.
pub(crate) struct CheckpointReader<'a>(pub(crate) &'a [u8]);
impl<'a> CheckpointReader<'a> {
    pub(crate) fn bytes(&mut self, length: usize) -> Result<&'a [u8], String> {
        let (head, tail) = self
            .0
            .split_at_checked(length)
            .ok_or("truncated actor checkpoint")?;
        self.0 = tail;
        Ok(head)
    }
    pub(crate) fn byte(&mut self) -> Result<u8, String> {
        Ok(self.bytes(1)?[0])
    }
    pub(crate) fn word(&mut self) -> Result<u16, String> {
        Ok(u16::from_le_bytes(self.bytes(2)?.try_into().unwrap()))
    }
    pub(crate) fn id(&mut self) -> Result<u32, String> {
        Ok(u32::from_le_bytes(self.bytes(4)?.try_into().unwrap()))
    }
    pub(crate) fn counter(&mut self) -> Result<u64, String> {
        Ok(u64::from_le_bytes(self.bytes(8)?.try_into().unwrap()))
    }
    pub(crate) fn optional_id(&mut self) -> Result<Option<u32>, String> {
        match self.byte()? {
            0 => Ok(None),
            1 => Ok(Some(self.id()?)),
            _ => Err("invalid actor checkpoint optional ID".into()),
        }
    }
}
pub(super) fn optional_id(bytes: &mut Vec<u8>, value: Option<u32>) {
    bytes.push(u8::from(value.is_some()));
    if let Some(value) = value {
        bytes.extend_from_slice(&value.to_le_bytes());
    }
}

fn image_class(images: &mut Vec<crate::rom::RomImage>, image: &crate::rom::RomImage) -> usize {
    if let Some(index) = images.iter().position(|value| value.same_allocation(image)) {
        index
    } else {
        images.push(image.clone());
        images.len() - 1
    }
}
fn named_images(
    sources: &[(
        crate::game::room_entry::world::ResourceId,
        crate::rom::RomImage,
    )],
    world: &WorldBinding,
) -> Result<
    std::collections::BTreeMap<crate::game::room_entry::world::ResourceId, crate::rom::RomImage>,
    String,
> {
    let mut result = std::collections::BTreeMap::new();
    for (resource, image) in sources {
        if !world.definition().is_some_and(|value| {
            value.resources.get(resource)
                == Some(&crate::game::room_entry::world::ResourceKind::Palette)
        }) || result.insert(*resource, image.clone()).is_some()
        {
            return Err("invalid named actor checkpoint palette allocation".into());
        }
    }
    Ok(result)
}
checkpoint_layout!(actor_checkpoint_layout, write_actor_checkpoint(
    bytes,
    world: &WorldBinding,
    loaded: Option<usize>,
    native: Option<usize>,
    roles: Vec<(crate::game::room_entry::world::ResourceId, usize)>,
    images: &[crate::rom::RomImage],
    timeline: &[u8],
    rows: Vec<(usize, crate::game::room_entry::world::ResourceId, super::media::PaletteCheckpoint)>
) {
    magic schema = b"ACTORCP3";
    fixed world_revision: [u8; 32] = world.revision().0;
    bind revisions = [world.actor_gameplay_revision().map(|id| id.0), world.actor_media_revision().map(|id| id.0)];
    option gameplay_revision: [u8; 32] = revisions[0];
    option media_revision: [u8; 32] = revisions[1];
    field image_count: u8 = images.len() as u8;
    class loaded_allocation = loaded;
    class native_allocation = native;
    field role_count: u8 = roles.len() as u8;
    repeat roles by role_count = (roles) => ((resource, image)) {
        field resource: u32 = resource.0;
        field image: u8 = image as u8;
    }
    repeat images by image_count = (images) => (image) {
        blob allocation = image;
    }
    nested timeline: ActorTimeline = timeline;
    field row_count: u8 = rows.len() as u8;
    repeat palette_rows by row_count = (rows) => ((row, resource, image)) {
        field row: u8 = row as u8;
        field resource: u32 = resource.0;
        variants source = (image) {
            0 Legacy (super::media::PaletteCheckpoint::Legacy(image)) => {
                field image: u8 = image as u8;
            }
            1 Prepared (super::media::PaletteCheckpoint::Prepared(revision)) => {
                fixed revision: [u8; 32] = revision;
            }
        }
    }
});

impl ActorCheckpointPayload {
    #[cfg(any(feature = "frontend", sm_verify_dll))]
    pub(crate) fn serialized_layout() -> String {
        actor_checkpoint_layout()
    }
    pub(crate) fn capture(
        state: &ActorIdentityState,
        world: &Arc<WorldBinding>,
    ) -> Result<Self, String> {
        if state.is_poisoned() {
            return Err("poisoned actors cannot produce a checkpoint".into());
        }
        Ok(Self {
            actors: ActorSnapshotPayload::capture(state, world),
        })
    }

    pub(crate) fn encode(
        &self,
        loaded: Option<&crate::rom::RomImage>,
        native: Option<&crate::rom::RomImage>,
        sources: &[(
            crate::game::room_entry::world::ResourceId,
            crate::rom::RomImage,
        )],
        ram: &impl Fn(usize) -> u8,
    ) -> Result<Vec<u8>, String> {
        self.encode_occupancy(loaded, native, sources, &|index| {
            let at = 0xf78 + index * 64;
            [ram(at), ram(at + 1)] != [0, 0]
        })
    }

    fn encode_occupancy(
        &self,
        loaded: Option<&crate::rom::RomImage>,
        native: Option<&crate::rom::RomImage>,
        sources: &[(
            crate::game::room_entry::world::ResourceId,
            crate::rom::RomImage,
        )],
        live: &impl Fn(usize) -> bool,
    ) -> Result<Vec<u8>, String> {
        let world = &self.actors.binding;
        let named = named_images(sources, world)?;
        let mut timeline = Vec::new();
        self.actors.timeline.encode_occupancy(&mut timeline, live);
        ActorTimeline::decode_occupancy(&mut CheckpointReader(&timeline), world, live)?;
        let mut images = Vec::new();
        let loaded = loaded.map(|image| image_class(&mut images, image));
        let native = native.map(|image| image_class(&mut images, image));
        let rows: Vec<_> = self
            .actors
            .residency
            .checkpoint_rows()
            .map(|(row, resource, source)| {
                let source = match source {
                    super::media::PaletteProvenance::Legacy(image) => {
                        super::media::PaletteCheckpoint::Legacy(image_class(&mut images, image))
                    }
                    super::media::PaletteProvenance::Prepared(revision) => {
                        super::media::PaletteCheckpoint::Prepared(*revision)
                    }
                };
                (row, resource, source)
            })
            .collect();
        let mut roles = Vec::new();
        for (resource, image) in named {
            if rows.iter().any(|(_, receipt, source)| {
                *receipt == resource && matches!(source, super::media::PaletteCheckpoint::Legacy(_))
            }) {
                roles.push((resource, image_class(&mut images, &image)));
            }
        }
        super::media::ActorPaletteResidency::checkpoint_restore(&rows, &images, world)?;
        let mut bytes = Vec::new();
        write_actor_checkpoint(
            &mut bytes, world, loaded, native, roles, &images, &timeline, rows,
        );
        Ok(bytes)
    }

    pub(crate) fn prepare(
        bytes: &[u8],
        current: &ActorIdentityState,
        world: &Arc<WorldBinding>,
        loaded: Option<&crate::rom::RomImage>,
        native: Option<&crate::rom::RomImage>,
        sources: &[(
            crate::game::room_entry::world::ResourceId,
            crate::rom::RomImage,
        )],
        ram: &impl Fn(usize) -> u8,
        loaded_after_patch: Option<&crate::rom::RomImage>,
    ) -> Result<PreparedActorRestore, String> {
        Self::prepare_occupancy(
            bytes,
            current,
            world,
            loaded,
            native,
            sources,
            &|index| {
                let at = 0xf78 + index * 64;
                [ram(at), ram(at + 1)] != [0, 0]
            },
            loaded_after_patch,
        )
    }

    fn prepare_occupancy(
        bytes: &[u8],
        current: &ActorIdentityState,
        world: &Arc<WorldBinding>,
        loaded: Option<&crate::rom::RomImage>,
        native: Option<&crate::rom::RomImage>,
        sources: &[(
            crate::game::room_entry::world::ResourceId,
            crate::rom::RomImage,
        )],
        live: &impl Fn(usize) -> bool,
        loaded_after_patch: Option<&crate::rom::RomImage>,
    ) -> Result<PreparedActorRestore, String> {
        let next_epoch = current
            .checked_next_epoch()
            .map_err(|error| format!("{error:?}"))?;
        let named = named_images(sources, world)?;
        let mut read = CheckpointReader(bytes);
        if read.bytes(8)? != b"ACTORCP3" || read.bytes(32)? != world.revision().0 {
            return Err("actor checkpoint world revision differs".into());
        }
        for expected in [
            world.actor_gameplay_revision().map(|id| id.0),
            world.actor_media_revision().map(|id| id.0),
        ] {
            match (read.byte()?, expected) {
                (0, None) => {}
                (1, Some(expected)) if read.bytes(32)? == expected => {}
                _ => return Err("actor checkpoint gameplay or media revision differs".into()),
            }
        }
        let count = usize::from(read.byte()?);
        if count > 34 {
            return Err("too many actor checkpoint allocations".into());
        }
        let loaded_class = read.byte()?;
        let native_class = read.byte()?;
        let role_count = usize::from(read.byte()?);
        if role_count > 16 {
            return Err("too many named checkpoint allocations".into());
        }
        let mut roles = Vec::new();
        let mut resources = std::collections::BTreeSet::new();
        for _ in 0..role_count {
            let resource = crate::game::room_entry::world::ResourceId(read.id()?);
            let class = read.byte()?;
            if !resources.insert(resource) {
                return Err("duplicate checkpoint resource role".into());
            }
            roles.push((resource, class));
        }
        let mut data = Vec::new();
        for _ in 0..count {
            let length = usize::try_from(read.counter()?)
                .map_err(|_| "checkpoint allocation length overflow")?;
            data.push(read.bytes(length)?);
        }
        let timeline = ActorTimeline::decode_occupancy(&mut read, world, live)?;
        let row_count = usize::from(read.byte()?);
        if row_count > 16 {
            return Err("too many checkpoint palette rows".into());
        }
        let mut rows = Vec::new();
        let mut row_ids = std::collections::BTreeSet::new();
        for _ in 0..row_count {
            let row = usize::from(read.byte()?);
            let resource = crate::game::room_entry::world::ResourceId(read.id()?);
            let image = match read.byte()? {
                0 => {
                    let image = usize::from(read.byte()?);
                    if image >= count {
                        return Err("invalid checkpoint palette image".into());
                    }
                    super::media::PaletteCheckpoint::Legacy(image)
                }
                1 => super::media::PaletteCheckpoint::Prepared(read.bytes(32)?.try_into().unwrap()),
                _ => return Err("invalid checkpoint palette source tag".into()),
            };
            if row >= 16
                || !row_ids.insert(row)
                || !world.definition().is_some_and(|value| {
                    value.resources.get(&resource)
                        == Some(&crate::game::room_entry::world::ResourceKind::Palette)
                })
            {
                return Err("invalid checkpoint palette row".into());
            }
            rows.push((row, resource, image));
        }
        if !read.0.is_empty() {
            return Err("extra actor checkpoint metadata".into());
        }
        if roles.iter().any(|(resource, class)| {
            usize::from(*class) >= count
                || !rows.iter().any(|(_, row_resource, source)| {
                    row_resource == resource
                        && matches!(source, super::media::PaletteCheckpoint::Legacy(_))
                })
        }) {
            return Err("invalid checkpoint named allocation role".into());
        }
        if (0..count).any(|class| {
            loaded_class as usize != class
                && native_class as usize != class
                && !roles.iter().any(|(_, value)| *value as usize == class)
                && !rows.iter().any(|(_, _, value)| matches!(value, super::media::PaletteCheckpoint::Legacy(index) if *index == class))
        }) {
            return Err("unused checkpoint allocation class".into());
        }
        let mut bound: Vec<Option<crate::rom::RomImage>> = vec![None; count];
        let mut bind = |class: u8, image: Option<&crate::rom::RomImage>| -> Result<(), String> {
            if class == u8::MAX {
                return if image.is_none() {
                    Ok(())
                } else {
                    Err("checkpoint allocation presence differs".into())
                };
            }
            let index = usize::from(class);
            let image = image.ok_or("checkpoint allocation is unavailable")?;
            let content = if loaded.is_some_and(|loaded| loaded.same_allocation(image)) {
                loaded_after_patch.unwrap_or(image)
            } else {
                image
            };
            if index >= count
                || content.len() != data[index].len()
                || data[index]
                    .iter()
                    .enumerate()
                    .any(|(at, byte)| content.read(at) != *byte)
            {
                return Err("checkpoint allocation content differs".into());
            }
            if let Some(previous) = &bound[index] {
                if !previous.same_allocation(image) {
                    return Err("checkpoint allocation alias differs".into());
                }
            }
            if bound.iter().enumerate().any(|(other, value)| {
                other != index
                    && value
                        .as_ref()
                        .is_some_and(|value| value.same_allocation(image))
            }) {
                return Err("checkpoint distinct allocations became aliases".into());
            }
            bound[index] = Some(image.clone());
            Ok(())
        };
        bind(loaded_class, loaded)?;
        bind(native_class, native)?;
        for (resource, class) in roles {
            bind(class, named.get(&resource))?;
        }
        // Detached classes are separate owned allocations even if their bytes equal live sources.
        let images: Vec<_> = bound
            .into_iter()
            .enumerate()
            .map(|(index, image)| {
                image.unwrap_or_else(|| {
                    let image = crate::rom::RomImage::zeroed(data[index].len());
                    for (at, byte) in data[index].iter().enumerate() {
                        image.write(at, *byte);
                    }
                    image
                })
            })
            .collect();
        let residency =
            super::media::ActorPaletteResidency::checkpoint_restore(&rows, &images, world)?;
        Ok(PreparedActorRestore::new(next_epoch, timeline, residency))
    }
}
