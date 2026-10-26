//! Stable actor media definitions; no gameplay timing or collision ownership.
#![forbid(unsafe_code)]
use super::content::{PartRoleId, PoseId};
use crate::game::session::Point;
use std::collections::BTreeMap;
macro_rules! ids { ($($name:ident),+ $(,)?) => { $(
    #[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
    pub struct $name(pub u32);
)+ }; }
ids!(AppearanceId, FrameId, VisualResourceId, MaterialId);
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct MediaRevision(pub [u8; 32]);
#[derive(Clone, Debug, PartialEq)]
pub struct ActorMediaDefinitions {
    pub appearances: BTreeMap<AppearanceId, AppearanceDefinition>,
    pub frames: BTreeMap<FrameId, FrameDefinition>,
    pub materials: BTreeMap<MaterialId, ActorMaterialDefinition>,
    pub resources: BTreeMap<VisualResourceId, ActorVisualResource>,
}
#[derive(Clone, Debug, PartialEq)]
pub struct AppearanceDefinition {
    pub frames: BTreeMap<(PoseId, PartRoleId), FrameId>,
}
#[derive(Clone, Debug, PartialEq)]
pub enum FrameDefinition {
    Sprites(Vec<VisualPieceDefinition>),
    Tilemap {
        resource: VisualResourceId,
        extent: Point,
        material: MaterialId,
    },
}
#[derive(Clone, Debug, PartialEq)]
pub struct VisualPieceDefinition {
    pub resource: VisualResourceId,
    pub source_origin: Point,
    pub source_extent: Point,
    pub offset: Point,
    pub flip_x: bool,
    pub flip_y: bool,
    pub material: MaterialId,
}
#[derive(Clone, Debug, PartialEq)]
pub enum ActorVisualResource {
    CompatibleSprites {
        resource: crate::game::room_entry::world::ResourceId,
    },
    CompatibleTilemap {
        resource: crate::game::room_entry::world::ResourceId,
    },
    CompatiblePalette {
        resource: crate::game::room_entry::world::ResourceId,
    },
}
#[derive(Clone, Debug, PartialEq)]
pub(crate) enum ActorMediaCatalog {
    LegacyBound,
    Prepared {
        definitions: ActorMediaDefinitions,
        revision: MediaRevision,
    },
}

/// Normal palette resource. Flash/frozen policy remains runtime presentation state.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ActorMaterialDefinition {
    pub palette: VisualResourceId,
    /// Existing staged CGRAM row: OBJ8..15, background0..7. No upload/allocation at draw time.
    pub cgram_row: u8,
}

impl ActorVisualResource {
    pub(super) fn binding(
        &self,
    ) -> (
        crate::game::room_entry::world::ResourceId,
        crate::game::room_entry::world::ResourceKind,
    ) {
        use crate::game::room_entry::world::ResourceKind;
        match *self {
            Self::CompatibleSprites { resource } => (resource, ResourceKind::ActorSprites),
            Self::CompatibleTilemap { resource } => (resource, ResourceKind::ActorTilemap),
            Self::CompatiblePalette { resource } => (resource, ResourceKind::Palette),
        }
    }
}
impl ActorMediaCatalog {
    pub(crate) fn prepared(
        definitions: ActorMediaDefinitions,
        actors: &super::content::ActorCatalog,
        world: &crate::game::room_entry::world::WorldBinding,
    ) -> Result<Self, super::content::ActorContentError> {
        use super::content::ActorContentError as Error;
        use crate::game::room_entry::world::ResourceKind;
        let gameplay = actors.definitions().ok_or(Error::UnsupportedForm)?;
        let world_definition = world.definition().ok_or(Error::UnsupportedForm)?;
        for (&id, resource) in &definitions.resources {
            let (key, kind) = resource.binding();
            match world_definition.resources.get(&key) {
                None => return Err(Error::MissingResource(id)),
                Some(actual) if *actual != kind => return Err(Error::WrongResourceKind(id)),
                Some(_) => {}
            }
        }
        for (&id, material) in &definitions.materials {
            require_resource(&definitions, material.palette, ResourceKind::Palette)?;
            if material.cgram_row >= 16 {
                return Err(Error::UnsupportedMaterialRow(id));
            }
        }
        for frame in definitions.frames.values() {
            match frame {
                FrameDefinition::Sprites(pieces) => {
                    for piece in pieces {
                        // Zero origin/extent denotes the complete compatible stream.
                        // The retained renderer has no crop/scale/flip operation here.
                        if piece.source_origin != Point::default()
                            || piece.source_extent != Point::default()
                            || piece.flip_x
                            || piece.flip_y
                        {
                            return Err(Error::UnsupportedForm);
                        }
                        require_resource(&definitions, piece.resource, ResourceKind::ActorSprites)?;
                        let material = require_material(&definitions, piece.material)?;
                        if material.cgram_row < 8 {
                            return Err(Error::UnsupportedMaterialRow(piece.material));
                        }
                    }
                }
                FrameDefinition::Tilemap {
                    resource,
                    extent,
                    material,
                } => {
                    if *extent != Point::default() {
                        return Err(Error::UnsupportedForm);
                    }
                    require_resource(&definitions, *resource, ResourceKind::ActorTilemap)?;
                    let value = require_material(&definitions, *material)?;
                    if value.cgram_row >= 8 {
                        return Err(Error::UnsupportedMaterialRow(*material));
                    }
                }
            }
        }
        for (&appearance, definition) in &definitions.appearances {
            for (&(pose, role), &frame) in &definition.frames {
                let pose_definition = gameplay.poses.get(&pose).ok_or(Error::MissingPose(pose))?;
                if !pose_definition.parts.iter().any(|part| part.role == role) {
                    return Err(Error::UnsupportedForm);
                }
                if !definitions.frames.contains_key(&frame) {
                    return Err(Error::MissingFrame(frame));
                }
            }
            // No per-species reachability schema exists in this compatible milestone.
            // Every selected appearance therefore covers every declared gameplay pose.
            for (&pose, pose_definition) in &gameplay.poses {
                let mut tilemap = false;
                for part in &pose_definition.parts {
                    let frame = definition.frames.get(&(pose, part.role)).ok_or(
                        Error::MissingPoseFrame {
                            appearance,
                            pose,
                            role: part.role,
                        },
                    )?;
                    if matches!(definitions.frames[frame], FrameDefinition::Tilemap { .. }) {
                        if part.offset != Point::default() {
                            return Err(Error::UnsupportedForm);
                        }
                        tilemap = true;
                    }
                }
                if tilemap && pose_definition.parts.len() != 1 {
                    return Err(Error::UnsupportedForm);
                }
            }
        }
        let revision = media_revision(&definitions, gameplay, world);
        Ok(Self::Prepared {
            definitions,
            revision,
        })
    }

    pub(crate) fn definitions(&self) -> Option<&ActorMediaDefinitions> {
        match self {
            Self::LegacyBound => None,
            Self::Prepared { definitions, .. } => Some(definitions),
        }
    }
    pub(crate) fn revision(&self) -> Option<MediaRevision> {
        match self {
            Self::LegacyBound => None,
            Self::Prepared { revision, .. } => Some(*revision),
        }
    }

    pub(super) fn resolve(
        &self,
        appearance: AppearanceId,
        pose: PoseId,
        definition: &super::content::PoseDefinition,
    ) -> Result<super::scene::ResolvedActorVisual, super::content::ActorContentError> {
        use super::scene::{ActorVisualForm, ResolvedActorVisual};
        let mut pieces = Vec::new();
        for part in &definition.parts {
            let visual = self.resolve_part(appearance, pose, part.role, part.offset)?;
            match visual.form {
                ActorVisualForm::Sprites(mut sprites) => pieces.append(&mut sprites),
                ActorVisualForm::Tilemap(tilemap) if definition.parts.len() == 1 => {
                    return Ok(ResolvedActorVisual {
                        appearance,
                        pose,
                        form: ActorVisualForm::Tilemap(tilemap),
                    });
                }
                ActorVisualForm::Tilemap(_) => {
                    return Err(super::content::ActorContentError::UnsupportedForm);
                }
            }
        }
        Ok(ResolvedActorVisual {
            appearance,
            pose,
            form: ActorVisualForm::Sprites(pieces),
        })
    }

    /// offset is zero when the caller already applied the gameplay part position.
    pub(super) fn resolve_part(
        &self,
        appearance: AppearanceId,
        pose: PoseId,
        role: PartRoleId,
        offset: Point,
    ) -> Result<super::scene::ResolvedActorVisual, super::content::ActorContentError> {
        use super::{
            content::ActorContentError as Error,
            scene::{ActorVisualForm, ResolvedActorVisual, SpritePiece, TilemapVisual},
        };
        let definitions = self
            .definitions()
            .ok_or(Error::MissingAppearance(appearance))?;
        let appearance_definition = definitions
            .appearances
            .get(&appearance)
            .ok_or(Error::MissingAppearance(appearance))?;
        let frame =
            *appearance_definition
                .frames
                .get(&(pose, role))
                .ok_or(Error::MissingPoseFrame {
                    appearance,
                    pose,
                    role,
                })?;
        let form = match definitions
            .frames
            .get(&frame)
            .ok_or(Error::MissingFrame(frame))?
        {
            FrameDefinition::Sprites(pieces) => ActorVisualForm::Sprites(
                pieces
                    .iter()
                    .map(|piece| SpritePiece {
                        frame,
                        resource: piece.resource,
                        source_origin: piece.source_origin,
                        source_extent: piece.source_extent,
                        offset: Point {
                            x: offset.x.wrapping_add(piece.offset.x),
                            y: offset.y.wrapping_add(piece.offset.y),
                        },
                        flip_x: piece.flip_x,
                        flip_y: piece.flip_y,
                        material: piece.material,
                    })
                    .collect(),
            ),
            FrameDefinition::Tilemap {
                resource,
                extent,
                material,
            } => ActorVisualForm::Tilemap(TilemapVisual {
                resource: *resource,
                extent: *extent,
                offset,
                material: *material,
            }),
        };
        Ok(ResolvedActorVisual {
            appearance,
            pose,
            form,
        })
    }
}
fn require_material(
    definitions: &ActorMediaDefinitions,
    material: MaterialId,
) -> Result<&ActorMaterialDefinition, super::content::ActorContentError> {
    definitions
        .materials
        .get(&material)
        .ok_or(super::content::ActorContentError::MissingMaterial(material))
}
fn require_resource(
    definitions: &ActorMediaDefinitions,
    id: VisualResourceId,
    expected: crate::game::room_entry::world::ResourceKind,
) -> Result<(), super::content::ActorContentError> {
    let resource = definitions
        .resources
        .get(&id)
        .ok_or(super::content::ActorContentError::MissingResource(id))?;
    if resource.binding().1 == expected {
        Ok(())
    } else {
        Err(super::content::ActorContentError::WrongResourceKind(id))
    }
}

// Process-local, independently versioned media identity. Gameplay choices are excluded.
fn media_revision(
    definitions: &ActorMediaDefinitions,
    actors: &super::content::ActorDefinitions,
    world: &crate::game::room_entry::world::WorldBinding,
) -> MediaRevision {
    use std::hash::{DefaultHasher, Hash, Hasher};
    let mut revision = [0; 32];
    for (lane, bytes) in revision.chunks_exact_mut(8).enumerate() {
        let mut hash = DefaultHasher::new();
        ("actor-media-v1", lane as u64).hash(&mut hash);
        for (id, appearance) in &definitions.appearances {
            (id, &appearance.frames).hash(&mut hash);
        }
        for (id, frame) in &definitions.frames {
            id.hash(&mut hash);
            match frame {
                FrameDefinition::Sprites(pieces) => {
                    (0u8, pieces.len()).hash(&mut hash);
                    for piece in pieces {
                        (
                            piece.resource,
                            piece.source_origin.x,
                            piece.source_origin.y,
                            piece.source_extent.x,
                            piece.source_extent.y,
                            piece.offset.x,
                            piece.offset.y,
                            piece.flip_x,
                            piece.flip_y,
                            piece.material,
                        )
                            .hash(&mut hash);
                    }
                }
                FrameDefinition::Tilemap {
                    resource,
                    extent,
                    material,
                } => {
                    (1u8, resource, extent.x, extent.y, material).hash(&mut hash);
                }
            }
        }
        for (id, material) in &definitions.materials {
            (id, material.palette, material.cgram_row).hash(&mut hash);
        }
        for (id, resource) in &definitions.resources {
            let (resource, kind) = resource.binding();
            (id, resource, kind, world.resource_key(resource)).hash(&mut hash);
        }
        for (id, species) in &actors.species {
            (id, species.default_appearance).hash(&mut hash);
        }
        for (id, population) in &actors.populations {
            id.hash(&mut hash);
            for entry in &population.entries {
                (entry.id, entry.appearance).hash(&mut hash);
            }
        }
        for (id, entry) in &actors.templates {
            (id, entry.appearance).hash(&mut hash);
        }
        bytes.copy_from_slice(&hash.finish().to_le_bytes());
    }
    MediaRevision(revision)
}

/// Room-local semantic provenance, not another palette byte owner. Fade/animation
/// writes do not change this receipt; actual staging/replacement does.
#[derive(Clone, Default)]
pub(crate) struct ActorPaletteResidency {
    rows: [Option<ResidentPalette>; 16],
}
#[derive(Clone)]
struct ResidentPalette {
    resource: crate::game::room_entry::world::ResourceId,
    image: crate::rom::RomImage,
}
impl PartialEq for ActorPaletteResidency {
    fn eq(&self, other: &Self) -> bool {
        self.rows
            .iter()
            .zip(&other.rows)
            .all(|(left, right)| match (left, right) {
                (None, None) => true,
                (Some(left), Some(right)) => {
                    left.resource == right.resource && left.image.same_allocation(&right.image)
                }
                _ => false,
            })
    }
}
impl core::fmt::Debug for ActorPaletteResidency {
    fn fmt(&self, formatter: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        formatter
            .debug_list()
            .entries(
                self.rows
                    .iter()
                    .map(|row| row.as_ref().map(|receipt| receipt.resource)),
            )
            .finish()
    }
}
impl ActorPaletteResidency {
    pub(super) fn clear(&mut self) {
        self.rows.fill(None);
    }
    pub(super) fn clear_actor_rows(&mut self) {
        self.rows[8..].fill(None);
    }

    /// Called after the complete source copy. An unknown overwrite revokes the row.
    pub(super) fn record(
        &mut self,
        row: u16,
        resource: Option<crate::game::room_entry::world::ResourceId>,
        image: crate::rom::RomImage,
    ) {
        if let Some(receipt) = self.rows.get_mut(usize::from(row)) {
            *receipt = resource.map(|resource| ResidentPalette { resource, image });
        }
    }
}
impl ActorMediaCatalog {
    /// Only a selected material is required in this room; unrelated world materials
    /// need not be resident. Rebound source allocations invalidate old receipts.
    pub(super) fn validate_material_residency(
        &self,
        material: MaterialId,
        residency: &ActorPaletteResidency,
        image: &crate::rom::RomImage,
    ) -> Result<(), super::content::ActorContentError> {
        use super::content::ActorContentError as Error;
        let definitions = self.definitions().ok_or(Error::MissingMaterial(material))?;
        let definition = definitions
            .materials
            .get(&material)
            .ok_or(Error::MissingMaterial(material))?;
        let resource = definitions
            .resources
            .get(&definition.palette)
            .ok_or(Error::MissingResource(definition.palette))?
            .binding()
            .0;
        let receipt = residency
            .rows
            .get(usize::from(definition.cgram_row))
            .and_then(Option::as_ref);
        if receipt.is_some_and(|receipt| {
            receipt.resource == resource && receipt.image.same_allocation(image)
        }) {
            Ok(())
        } else {
            Err(Error::PaletteNotResident(material))
        }
    }
}

/// Map the actual pinned source to a declared resource without reading its pixels
/// or reacquiring ROM. The receipt separately retains this exact allocation.
pub(super) fn palette_resource_for_capture(
    record: &super::content::ActorLiveRecord,
    world: &crate::game::room_entry::world::WorldBinding,
) -> Option<crate::game::room_entry::world::ResourceId> {
    use crate::game::room_entry::world::{LegacyAddress24, ResourceKey, ResourceKind};
    let mut resources = world
        .definition()?
        .resources
        .iter()
        .filter_map(|(&id, &kind)| {
            if kind != ResourceKind::Palette {
                return None;
            }
            let ResourceKey::Long(LegacyAddress24([low, high, bank])) = world.resource_key(id)
            else {
                return None;
            };
            let address = (u32::from(bank) << 16) | u32::from(u16::from_le_bytes([low, high]));
            (crate::rom::lorom_offset(address) as usize == record.base).then_some(id)
        });
    let resource = resources.next()?;
    resources.next().is_none().then_some(resource)
}

impl ActorMediaCatalog {
    /// A selected appearance may need several materials, but unrelated rooms may
    /// reuse their rows. Preflight conflicts and residency before changing actors.
    pub(super) fn validate_appearance_residency(
        &self,
        appearance: AppearanceId,
        residency: &ActorPaletteResidency,
        image: &crate::rom::RomImage,
    ) -> Result<(), super::content::ActorContentError> {
        use super::content::ActorContentError as Error;
        let definitions = self
            .definitions()
            .ok_or(Error::MissingAppearance(appearance))?;
        let appearance = definitions
            .appearances
            .get(&appearance)
            .ok_or(Error::MissingAppearance(appearance))?;
        let mut materials = std::collections::BTreeSet::new();
        for frame in appearance.frames.values() {
            match definitions
                .frames
                .get(frame)
                .ok_or(Error::MissingFrame(*frame))?
            {
                FrameDefinition::Sprites(pieces) => {
                    materials.extend(pieces.iter().map(|piece| piece.material))
                }
                FrameDefinition::Tilemap { material, .. } => {
                    materials.insert(*material);
                }
            }
        }
        let mut rows = BTreeMap::new();
        for &material in &materials {
            let definition = definitions
                .materials
                .get(&material)
                .ok_or(Error::MissingMaterial(material))?;
            let resource = definitions
                .resources
                .get(&definition.palette)
                .ok_or(Error::MissingResource(definition.palette))?
                .binding()
                .0;
            if let Some(previous) = rows.insert(definition.cgram_row, resource) {
                if previous != resource {
                    return Err(Error::ConflictingPaletteRow(definition.cgram_row));
                }
            }
        }
        for material in materials {
            self.validate_material_residency(material, residency, image)?;
        }
        Ok(())
    }
}

impl ActorPaletteResidency {
    pub(super) fn checkpoint_rows(
        &self,
    ) -> impl Iterator<
        Item = (
            usize,
            crate::game::room_entry::world::ResourceId,
            &crate::rom::RomImage,
        ),
    > {
        self.rows.iter().enumerate().filter_map(|(row, value)| {
            value
                .as_ref()
                .map(|receipt| (row, receipt.resource, &receipt.image))
        })
    }

    pub(super) fn checkpoint_restore(
        rows: &[(usize, crate::game::room_entry::world::ResourceId, usize)],
        images: &[crate::rom::RomImage],
        world: &crate::game::room_entry::world::WorldBinding,
    ) -> Result<Self, String> {
        let mut result = Self::default();
        for &(row, resource, image) in rows {
            if row >= 16 || result.rows[row].is_some() || image >= images.len() {
                return Err("invalid actor checkpoint palette receipt".into());
            }
            if !world.definition().is_some_and(|definition| {
                definition.resources.get(&resource)
                    == Some(&crate::game::room_entry::world::ResourceKind::Palette)
            }) {
                return Err("actor checkpoint palette resource is unavailable".into());
            }
            result.rows[row] = Some(ResidentPalette {
                resource,
                image: images[image].clone(),
            });
        }
        Ok(result)
    }
}
