//! Paired timeline/presentation snapshots; live handle epochs never rewind.
#![forbid(unsafe_code)]
use super::{
    content::GameplayRevision,
    identity::{ActorIdentityError, ActorIdentityState, ActorTimeline},
    media::MediaRevision,
};
use crate::game::room_entry::world::WorldBinding;
use std::sync::Arc;
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
pub(super) struct CheckpointReader<'a>(pub(super) &'a [u8]);
impl<'a> CheckpointReader<'a> {
    pub(super) fn bytes(&mut self, length: usize) -> Result<&'a [u8], String> {
        let (head, tail) = self
            .0
            .split_at_checked(length)
            .ok_or("truncated actor checkpoint")?;
        self.0 = tail;
        Ok(head)
    }
    pub(super) fn byte(&mut self) -> Result<u8, String> {
        Ok(self.bytes(1)?[0])
    }
    pub(super) fn word(&mut self) -> Result<u16, String> {
        Ok(u16::from_le_bytes(self.bytes(2)?.try_into().unwrap()))
    }
    pub(super) fn id(&mut self) -> Result<u32, String> {
        Ok(u32::from_le_bytes(self.bytes(4)?.try_into().unwrap()))
    }
    pub(super) fn counter(&mut self) -> Result<u64, String> {
        Ok(u64::from_le_bytes(self.bytes(8)?.try_into().unwrap()))
    }
    pub(super) fn optional_id(&mut self) -> Result<Option<u32>, String> {
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
impl ActorCheckpointPayload {
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
        ram: &[u8; 131072],
    ) -> Result<Vec<u8>, String> {
        let world = &self.actors.binding;
        let named = named_images(sources, world)?;
        let mut timeline = Vec::new();
        self.actors.timeline.encode(&mut timeline, ram);
        ActorTimeline::decode(&mut CheckpointReader(&timeline), world, ram)?;
        let mut images = Vec::new();
        let loaded = loaded.map(|image| image_class(&mut images, image));
        let native = native.map(|image| image_class(&mut images, image));
        let rows: Vec<_> = self
            .actors
            .residency
            .checkpoint_rows()
            .map(|(row, resource, image)| (row, resource, image_class(&mut images, image)))
            .collect();
        let mut roles = Vec::new();
        for (resource, image) in named {
            if rows.iter().any(|(_, receipt, _)| *receipt == resource) {
                roles.push((resource, image_class(&mut images, &image)));
            }
        }
        super::media::ActorPaletteResidency::checkpoint_restore(&rows, &images, world)?;
        let mut bytes = Vec::from(b"ACTORCP2".as_slice());
        bytes.extend_from_slice(&world.revision().0);
        for revision in [
            world.actor_gameplay_revision().map(|id| id.0),
            world.actor_media_revision().map(|id| id.0),
        ] {
            bytes.push(u8::from(revision.is_some()));
            if let Some(revision) = revision {
                bytes.extend_from_slice(&revision);
            }
        }
        bytes.push(images.len() as u8);
        for role in [loaded, native] {
            bytes.push(role.map_or(u8::MAX, |index| index as u8));
        }
        bytes.push(roles.len() as u8);
        for (resource, image) in roles {
            bytes.extend_from_slice(&resource.0.to_le_bytes());
            bytes.push(image as u8);
        }
        for image in &images {
            bytes.extend_from_slice(&(image.len() as u64).to_le_bytes());
            bytes.extend((0..image.len()).map(|at| image.read(at)));
        }
        bytes.extend_from_slice(&timeline);
        bytes.push(rows.len() as u8);
        for (row, resource, image) in rows {
            bytes.push(row as u8);
            bytes.extend_from_slice(&resource.0.to_le_bytes());
            bytes.push(image as u8);
        }
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
        ram: &[u8; 131072],
        loaded_after_patch: Option<&crate::rom::RomImage>,
    ) -> Result<PreparedActorRestore, String> {
        let next_epoch = current
            .checked_next_epoch()
            .map_err(|error| format!("{error:?}"))?;
        let named = named_images(sources, world)?;
        let mut read = CheckpointReader(bytes);
        if read.bytes(8)? != b"ACTORCP2" || read.bytes(32)? != world.revision().0 {
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
        let timeline = ActorTimeline::decode(&mut read, world, ram)?;
        let row_count = usize::from(read.byte()?);
        if row_count > 16 {
            return Err("too many checkpoint palette rows".into());
        }
        let mut rows = Vec::new();
        let mut row_ids = std::collections::BTreeSet::new();
        for _ in 0..row_count {
            let row = usize::from(read.byte()?);
            let resource = crate::game::room_entry::world::ResourceId(read.id()?);
            let image = usize::from(read.byte()?);
            if row >= 16
                || image >= count
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
                || !rows
                    .iter()
                    .any(|(_, row_resource, _)| row_resource == resource)
        }) {
            return Err("invalid checkpoint named allocation role".into());
        }
        if (0..count).any(|class| {
            loaded_class as usize != class
                && native_class as usize != class
                && !roles.iter().any(|(_, value)| *value as usize == class)
                && !rows.iter().any(|(_, _, value)| *value == class)
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
