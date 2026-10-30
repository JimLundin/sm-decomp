//! Validated actor catalogs and source-ordered live capture contracts.
#![forbid(unsafe_code)]
use super::{
    access::ActorKey,
    media::{AppearanceId, FrameId, VisualResourceId},
};
use crate::geometry::Point;
use crate::{
    rom::RomImage,
    room_entry::{content::Capture, world::ResourceId},
};
use std::collections::BTreeMap;

macro_rules! ids { ($($name:ident),+ $(,)?) => { $(
    #[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
    pub struct $name(pub u32);
)+ }; }
ids!(
    SpeciesId,
    PopulationId,
    PopulationEntryId,
    PoseId,
    CollisionLayoutId,
    ActorTemplateId,
    ActorProgramId,
    BehaviorId,
    ActorCallbackId,
    PartRoleId
);
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct GameplayRevision(pub [u8; 32]);
#[derive(Clone, Debug, PartialEq)]
pub struct ActorDefinitions {
    pub species: BTreeMap<SpeciesId, SpeciesDefinition>,
    pub programs: BTreeMap<ActorProgramId, ActorProgramDefinition>,
    pub populations: BTreeMap<PopulationId, PopulationDefinition>,
    pub templates: BTreeMap<ActorTemplateId, PopulationEntry>,
    pub poses: BTreeMap<PoseId, PoseDefinition>,
    pub collisions: BTreeMap<CollisionLayoutId, CollisionLayout>,
}
#[derive(Clone, Debug, PartialEq)]
pub struct SpeciesDefinition {
    pub initial_health: u16,
    pub contact_damage: u16,
    pub radius: Point,
    pub hurt_time: u8,
    pub layer: u8,
    pub behavior: BehaviorId,
    pub default_appearance: AppearanceId,
    pub multipart_count: u16,
}
#[derive(Clone, Debug, PartialEq)]
pub struct PopulationDefinition {
    pub entries: Vec<PopulationEntry>,
    pub death_quota: u16,
}
#[derive(Clone, Debug, PartialEq)]
pub struct PopulationEntry {
    pub id: PopulationEntryId,
    pub species: SpeciesId,
    pub appearance: Option<AppearanceId>,
    pub position: Point,
    pub initial_program: ActorProgramId,
    pub properties: u16,
    pub extra_properties: u16,
    pub parameters: [u16; 2],
}
#[derive(Clone, Debug, PartialEq)]
pub struct PoseDefinition {
    pub collision: CollisionLayoutId,
    pub parts: Vec<PosePart>,
}
#[derive(Clone, Debug, PartialEq)]
pub struct PosePart {
    pub role: PartRoleId,
    pub offset: Point,
}
#[derive(Clone, Debug, PartialEq)]
pub enum CollisionLayout {
    Body { radius: Point, active: bool },
    Parts(Vec<CollisionPart>),
}
#[derive(Clone, Debug, PartialEq)]
pub struct CollisionPart {
    pub offset: Point,
    pub hitboxes: Vec<CollisionBox>,
}
#[derive(Clone, Debug, PartialEq)]
pub struct CollisionBox {
    pub left: u16,
    pub top: u16,
    pub right: u16,
    pub bottom: u16,
    pub touch: ActorCallbackId,
    pub shot: ActorCallbackId,
}
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum EffectiveSpecies {
    Known(SpeciesId),
    UnresolvedCompatibility,
}
/// Compatibility keys are constructor input, separate from authored identity.
#[derive(Clone, Debug, Default, PartialEq)]
pub struct ActorCompatibilityKeys {
    pub species: BTreeMap<SpeciesId, u16>,
    pub populations: BTreeMap<PopulationId, u16>,
    pub templates: BTreeMap<ActorTemplateId, (u8, u16)>,
    /// BehaviorId names a complete compatible species behavior profile.
    pub behaviors: BTreeMap<BehaviorId, u16>,
    pub programs: BTreeMap<ActorProgramId, (u8, u16)>,
    pub callbacks: BTreeMap<ActorCallbackId, u32>,
    pub poses: BTreeMap<PoseId, (u8, u16)>,
    pub room_populations: BTreeMap<ResourceId, PopulationId>,
}
#[derive(Clone, Debug, PartialEq)]
pub struct PreparedActorCatalog {
    pub definitions: ActorDefinitions,
    pub keys: ActorCompatibilityKeys,
    pub revision: GameplayRevision,
}
#[derive(Clone, Debug, PartialEq)]
pub enum ActorCatalog {
    LegacyBound,
    Prepared(Box<PreparedActorCatalog>),
}
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum ActorContentError {
    MissingSpecies(SpeciesId),
    MissingPopulation(PopulationId),
    MissingPose(PoseId),
    MissingCollision(CollisionLayoutId),
    MissingAppearance(AppearanceId),
    MissingFrame(FrameId),
    MissingResource(VisualResourceId),
    MissingMaterial(super::media::MaterialId),
    UnsupportedMaterialRow(super::media::MaterialId),
    ConflictingPaletteRow(u8),
    PaletteNotResident(super::media::MaterialId),
    WrongResourceKind(VisualResourceId),
    World(crate::room_entry::world::WorldDefinitionError),
    MissingProgram(ActorProgramId),
    MissingTemplate(ActorTemplateId),
    MissingBehavior(BehaviorId),
    MissingCallback(ActorCallbackId),
    MissingEntry,
    DuplicateEntry,
    DuplicatePartRole,
    MissingPoseFrame {
        appearance: AppearanceId,
        pose: PoseId,
        role: PartRoleId,
    },
    MissingCompatibilityKey,
    AmbiguousCompatibilityKey,
    UnsupportedForm,
}
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct ActorRoutine(pub u32);
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct ActorDefinitionKey(pub u16);
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum PopulationCursor {
    Legacy {
        bank: u8,
        address: u16,
    },
    Prepared {
        population: PopulationId,
        entry: usize,
    },
    Template {
        template: ActorTemplateId,
        part: usize,
    },
}
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum PopulationPosition {
    Room {
        population: PopulationId,
        entry: usize,
    },
    Template {
        template: ActorTemplateId,
        part: usize,
    },
}
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct PosePartPosition {
    pub pose: PoseId,
    pub part: usize,
}
#[derive(Clone)]
pub struct ActorPopulationCapture {
    pub world: std::sync::Arc<crate::room_entry::world::WorldBinding>,
    pub position: Option<PopulationPosition>,
    pub record: ActorLiveRecord,
    pub cursor: PopulationCursor,
}
/// Authored identity and exact live allocation are captured together. Scalar
/// overrides never detach the record needed for later source-ordered live reads.
#[derive(Clone)]
pub struct ActorRecordCapture<P> {
    pub world: std::sync::Arc<crate::room_entry::world::WorldBinding>,
    pub position: Option<P>,
    pub record: ActorLiveRecord,
}
pub type ActorDefinitionCapture = ActorRecordCapture<SpeciesId>;
pub type ActorPoseCapture = ActorRecordCapture<PoseId>;
pub type ActorPartCapture = ActorRecordCapture<PosePartPosition>;
pub type ActorCollisionCapture = ActorRecordCapture<CollisionPosition>;
pub type ActorResourceCapture = Capture<VisualResourceId>;
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct ActorShape {
    pub radius: Point,
    pub health: u16,
    pub layer: u8,
    pub bank: u8,
    pub hurt_time: u8,
}
/// Acquisitions only. Token field reads retain captured allocation and source order.
pub trait ActorContent {
    fn actor_script_image(&self) -> RomImage;
    fn actor_pose_count(&mut self, source: &ActorPoseCapture, width: ActorCountWidth) -> u16;
    fn capture_actor_part_at(
        &mut self,
        source: &ActorPoseCapture,
        part: usize,
        traversal: ActorPartTraversal,
    ) -> ActorPartCapture;
    fn actor_part_word(&mut self, source: &ActorPartCapture, field: ActorPartWord) -> u16;
    fn capture_actor_collision_at(
        &mut self,
        source: &ActorPartCapture,
        bank: u8,
    ) -> ActorCollisionCapture;
    fn actor_collision_count(&mut self, source: &ActorCollisionCapture) -> u16;
    fn actor_collision_target(
        &mut self,
        source: &ActorCollisionCapture,
        hitbox: usize,
        field: ActorCollisionWord,
    ) -> ActorCallbackTarget;
    fn actor_collision_word(
        &mut self,
        source: &ActorCollisionCapture,
        hitbox: usize,
        field: ActorCollisionWord,
    ) -> u16;
    fn actor_visual_header(&mut self, bank: u8, address: u16) -> u16;
    fn actor_draw_queue_base(&self, layer: u16) -> u16;
    fn capture_actor_population(&mut self, cursor: &PopulationCursor) -> ActorPopulationCapture;
    fn capture_actor_definition(&mut self, key: ActorDefinitionKey) -> ActorDefinitionCapture;
    fn capture_actor_pose(&mut self, actor: ActorKey) -> ActorPoseCapture;
    fn capture_actor_resource(&mut self, resource: VisualResourceId) -> ActorResourceCapture;
    fn actor_spike_mechanism(&mut self, variant: u16) -> u16;
    fn actor_slope_multiplier(&mut self, index: u16) -> u16;
    fn actor_slope_surface(&mut self, index: u16) -> u16;
    fn actor_slope_surface_native(&mut self, index: u16) -> u8;
    fn actor_sine_sample(&mut self, phase: u8) -> u8;
}
/// A captured live allocation, not frozen definition values or snapshot-owned overlay.
#[derive(Clone)]
pub struct ActorLiveRecord {
    pub image: RomImage,
    pub base: usize,
}

/// Ordered commands, duration and continuation execute through retained EnemyScript.
/// This milestone does not provide an independent authored instruction interpreter.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum ActorProgramDefinition {
    CompatibleEnemyScript,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct CollisionPosition {
    pub layout: CollisionLayoutId,
    pub part: usize,
    pub hitbox: usize,
}
/// Finite live reads; each call is evaluated at its original source observation point.
/// Contact vulnerability is a native word, projectile vulnerability is a byte.
pub trait ActorCaptureReads {
    fn population_definition(&mut self, source: &ActorPopulationCapture) -> u16;
    fn population_word(&mut self, source: &ActorPopulationCapture, field: PopulationWord) -> u16;
    fn definition_word(&mut self, source: &ActorDefinitionCapture, field: DefinitionWord) -> u16;
    fn definition_byte(&mut self, source: &ActorDefinitionCapture, field: DefinitionByte) -> u8;
    fn definition_routine_offset(
        &mut self,
        source: &ActorDefinitionCapture,
        routine: DefinitionRoutine,
    ) -> u16;
    fn population_next(&mut self, source: &ActorPopulationCapture) -> PopulationCursor;
    fn population_terminated(&mut self, source: &ActorPopulationCapture) -> bool;
    fn population_death_quota(&mut self, source: &ActorPopulationCapture) -> u16;
    fn definition_name(&mut self, source: &ActorDefinitionCapture) -> Option<ActorLiveRecord>;
    fn name_word(&mut self, source: &ActorLiveRecord, offset: ActorNameWord) -> u16;
    fn definition_palette(&mut self, source: &ActorDefinitionCapture) -> ActorLiveRecord;
    fn palette_bytes(&mut self, source: &ActorLiveRecord) -> [u8; 32];
    fn definition_tile_size(&mut self, source: &ActorDefinitionCapture) -> u16;
    fn definition_tile_source(&mut self, source: &ActorDefinitionCapture) -> [u8; 3];
    fn capture_tile_chunk(&mut self, source: [u8; 3], offset: usize) -> ActorLiveRecord;
    fn capture_tilesets(&mut self, key: u16) -> ActorLiveRecord;
    fn tileset_definition(&mut self, source: &ActorLiveRecord, linear_index: usize) -> u16;
    fn tileset_destination_word(&mut self, source: &ActorLiveRecord, linear_index: usize) -> u16;
    fn tileset_destination_byte(&mut self, source: &ActorLiveRecord, linear_index: usize) -> u8;
    fn capture_graphics_tileset(&mut self, wrapping_address: u16) -> ActorLiveRecord;
    fn definition_vulnerability_address(&mut self, source: &ActorDefinitionCapture) -> u16;
    fn capture_vulnerability_at(&mut self, address: u16) -> ActorLiveRecord;
    fn contact_vulnerability(&mut self, source: &ActorLiveRecord, wrapping_offset: u16) -> u16;
    fn projectile_vulnerability(
        &mut self,
        source: &ActorLiveRecord,
        wrapping_offset: u16,
        linear_field: usize,
    ) -> u8;
}
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ActorNameWord {
    At0,
    At2,
    At4,
    At6,
    At8,
    At12,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum PopulationWord {
    X,
    Y,
    Program,
    Properties,
    ExtraProperties,
    Parameter1,
    Parameter2,
}
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum DefinitionWord {
    Width,
    Height,
    Health,
    NativeBank,
    Damage,
    HurtSound,
    DeathAnimation,
    MultipartCount,
    BossFight,
}
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum DefinitionByte {
    Bank,
    HurtTime,
    Layer,
}
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum DefinitionRoutine {
    Initialize,
    Main,
    Grapple,
    Hurt,
    Frozen,
    TimeFrozen,
    Touch,
    Shot,
    PowerBomb,
}
// Vulnerability methods wrap the captured u16 base plus wrapping_offset BEFORE
// mapping into the captured allocation. Native contact word and projectile byte
// remain different reads. Tileset list traversal uses linear_index; graphics
// reacquisition wraps its u16 address at the original acquisition point.

// Routine offsets and bank bytes are deliberately separate reads: initialization
// tests its routine word before conditionally reading bank; respawn reads bank
// then initializer. Callers form the full routine only after those observations.
// Projectile vulnerability wraps base+wrapping_offset before mapping, then adds
// linear_field in the captured allocation without a second bank-local wrap.

impl ActorDefinitionKey {
    pub fn from_legacy(key: u16) -> Self {
        Self(key)
    }
}
impl ActorRoutine {
    pub fn from_legacy(address: u32) -> Self {
        Self(address)
    }
    pub fn from_bank_offset(bank: u8, offset: u16) -> Self {
        Self((u32::from(bank) << 16) | u32::from(offset))
    }
}
impl PopulationCursor {
    pub fn position(&self) -> Option<PopulationPosition> {
        match *self {
            Self::Prepared { population, entry } => {
                Some(PopulationPosition::Room { population, entry })
            }
            Self::Template { template, part } => {
                Some(PopulationPosition::Template { template, part })
            }
            Self::Legacy { .. } => None,
        }
    }
}

impl ActorCatalog {
    pub fn prepared(
        definitions: ActorDefinitions,
        keys: ActorCompatibilityKeys,
        world: &crate::room_entry::world::WorldBinding,
        media: &super::media::ActorMediaDefinitions,
    ) -> Result<Self, ActorContentError> {
        use crate::room_entry::world::{ResourceKey, ResourceKind};
        use std::collections::BTreeSet;
        if definitions.species.keys().ne(keys.species.keys())
            || definitions.populations.keys().ne(keys.populations.keys())
            || definitions.templates.keys().ne(keys.templates.keys())
            || definitions.programs.keys().ne(keys.programs.keys())
            || definitions.poses.keys().ne(keys.poses.keys())
        {
            return Err(ActorContentError::MissingCompatibilityKey);
        }
        unique_values(keys.species.values())?;
        unique_values(keys.populations.values())?;
        unique_values(keys.templates.values())?;
        unique_values(keys.poses.values())?;
        for &key in keys
            .species
            .values()
            .chain(keys.populations.values())
            .chain(keys.behaviors.values())
        {
            if key & 0x8000 == 0 {
                return Err(ActorContentError::UnsupportedForm);
            }
        }
        for &(bank, offset) in keys
            .templates
            .values()
            .chain(keys.programs.values())
            .chain(keys.poses.values())
        {
            validate_address(bank, offset)?;
        }
        for &address in keys.callbacks.values() {
            if address > 0xffffff {
                return Err(ActorContentError::UnsupportedForm);
            }
            validate_address((address >> 16) as u8, address as u16)?;
        }
        for (id, species) in &definitions.species {
            if !keys.behaviors.contains_key(&species.behavior) {
                return Err(ActorContentError::MissingBehavior(species.behavior));
            }
            // Legacy AI mutates EnemyDef through the actor's actual definition key.
            // A distinct profile allocation would bypass those live mutations.
            if keys.behaviors[&species.behavior] != keys.species[id] {
                return Err(ActorContentError::UnsupportedForm);
            }
            if !media.appearances.contains_key(&species.default_appearance) {
                return Err(ActorContentError::MissingAppearance(
                    species.default_appearance,
                ));
            }
        }
        for population in definitions.populations.values() {
            let mut entries = BTreeSet::new();
            for entry in &population.entries {
                validate_entry(entry, &definitions, media)?;
                if !entries.insert(entry.id) {
                    return Err(ActorContentError::DuplicateEntry);
                }
            }
        }
        for entry in definitions.templates.values() {
            validate_entry(entry, &definitions, media)?;
        }
        for pose in definitions.poses.values() {
            let collision = definitions
                .collisions
                .get(&pose.collision)
                .ok_or(ActorContentError::MissingCollision(pose.collision))?;
            let CollisionLayout::Parts(collision_parts) = collision else {
                return Err(ActorContentError::UnsupportedForm);
            };
            if pose.parts.len() != collision_parts.len() || pose.parts.len() > usize::from(u16::MAX)
            {
                return Err(ActorContentError::UnsupportedForm);
            }
            let mut roles = BTreeSet::new();
            for part in &pose.parts {
                if !roles.insert(part.role) {
                    return Err(ActorContentError::DuplicatePartRole);
                }
            }
        }
        for collision in definitions.collisions.values() {
            // Ordinary body radius is SpeciesDefinition.radius. Pose-level Body
            // overrides are not consumed by the current compatible contact path.
            let CollisionLayout::Parts(parts) = collision else {
                return Err(ActorContentError::UnsupportedForm);
            };
            if parts.len() > usize::from(u16::MAX)
                || parts
                    .iter()
                    .any(|part| part.hitboxes.len() > usize::from(u16::MAX))
            {
                return Err(ActorContentError::UnsupportedForm);
            }
            for hitbox in parts.iter().flat_map(|part| &part.hitboxes) {
                for callback in [hitbox.touch, hitbox.shot] {
                    if !keys.callbacks.contains_key(&callback) {
                        return Err(ActorContentError::MissingCallback(callback));
                    }
                }
            }
        }
        let world_definition = world
            .definition()
            .ok_or(ActorContentError::UnsupportedForm)?;
        for (&resource, &kind) in &world_definition.resources {
            if kind == ResourceKind::EnemyPopulation {
                let population = keys
                    .room_populations
                    .get(&resource)
                    .ok_or(ActorContentError::MissingCompatibilityKey)?;
                let address = keys
                    .populations
                    .get(population)
                    .ok_or(ActorContentError::MissingPopulation(*population))?;
                if world.resource_key(resource)
                    != (ResourceKey::Word {
                        bank: 0xa1,
                        offset: *address,
                    })
                {
                    return Err(ActorContentError::MissingCompatibilityKey);
                }
            }
        }
        for (&resource, &population) in &keys.room_populations {
            if world_definition.resources.get(&resource) != Some(&ResourceKind::EnemyPopulation) {
                return Err(ActorContentError::World(
                    crate::room_entry::world::WorldDefinitionError::WrongResourceKind(resource),
                ));
            }
            if !definitions.populations.contains_key(&population) {
                return Err(ActorContentError::MissingPopulation(population));
            }
        }
        let revision = gameplay_revision(&definitions, &keys);
        Ok(Self::Prepared(Box::new(PreparedActorCatalog {
            definitions,
            keys,
            revision,
        })))
    }

    pub fn definitions(&self) -> Option<&ActorDefinitions> {
        match self {
            Self::LegacyBound => None,
            Self::Prepared(catalog) => Some(&catalog.definitions),
        }
    }
    pub fn keys(&self) -> Option<&ActorCompatibilityKeys> {
        match self {
            Self::LegacyBound => None,
            Self::Prepared(catalog) => Some(&catalog.keys),
        }
    }
    pub fn revision(&self) -> Option<GameplayRevision> {
        match self {
            Self::LegacyBound => None,
            Self::Prepared(catalog) => Some(catalog.revision),
        }
    }
    pub fn species_for_key(&self, key: u16) -> Option<SpeciesId> {
        self.keys()?
            .species
            .iter()
            .find_map(|(&id, &value)| (value == key).then_some(id))
    }
    pub fn pose_for_key(&self, bank: u8, offset: u16) -> Option<PoseId> {
        self.keys()?
            .poses
            .iter()
            .find_map(|(&id, &value)| (value == (bank, offset)).then_some(id))
    }
    pub fn population_for_key(&self, key: u16) -> Option<PopulationId> {
        self.keys()?
            .populations
            .iter()
            .find_map(|(&id, &value)| (value == key).then_some(id))
    }
    pub fn entry(&self, position: PopulationPosition) -> Option<&PopulationEntry> {
        let definitions = self.definitions()?;
        match position {
            PopulationPosition::Room { population, entry } => {
                definitions.populations.get(&population)?.entries.get(entry)
            }
            PopulationPosition::Template { template, part: 0 } => {
                definitions.templates.get(&template)
            }
            // Only part zero is authored. Later parts retain the live compatible template stream.
            PopulationPosition::Template { .. } => None,
        }
    }

    pub fn birth_metadata(
        &self,
        position: PopulationPosition,
        live_definition: u16,
        requested: Option<AppearanceId>,
        respawn: bool,
    ) -> Result<(super::identity::ActorProvenance, Option<AppearanceId>), ActorContentError> {
        use super::identity::{ActorBirthOrigin, ActorProvenance};
        let definitions = self
            .definitions()
            .ok_or(ActorContentError::UnsupportedForm)?;
        let entry = self.entry(position);
        let authored_species = entry
            .map(|value| value.species)
            .or_else(|| self.species_for_key(live_definition));
        let origin = match position {
            PopulationPosition::Room { population, .. } => ActorBirthOrigin::Population {
                population,
                entry: entry.ok_or(ActorContentError::MissingEntry)?.id,
                respawn,
            },
            PopulationPosition::Template { template, part } => {
                if !definitions.templates.contains_key(&template) {
                    return Err(ActorContentError::MissingTemplate(template));
                }
                ActorBirthOrigin::Dynamic {
                    template,
                    part: part as u16,
                }
            }
        };
        let appearance = requested
            .or_else(|| entry.and_then(|value| value.appearance))
            .or_else(|| {
                authored_species
                    .and_then(|id| definitions.species.get(&id))
                    .map(|value| value.default_appearance)
            });
        Ok((
            ActorProvenance {
                authored_species,
                origin,
            },
            appearance,
        ))
    }
}

/// Preserve the captured world and explicit cursor provenance across callbacks.
/// Legacy capture addresses alone cannot identify overlapping template occurrences.
pub fn birth_metadata_from_capture(
    source: &ActorPopulationCapture,
    origin: Option<PopulationPosition>,
    live_definition: u16,
    requested: Option<AppearanceId>,
    respawn: bool,
) -> Result<(super::identity::ActorProvenance, Option<AppearanceId>), ActorContentError> {
    use super::identity::{ActorBirthOrigin, ActorProvenance};
    let world = &source.world;
    let position = source.position.or(origin);
    if let Some(appearance) = requested {
        let media = world
            .actor_media()
            .definitions()
            .ok_or(ActorContentError::MissingAppearance(appearance))?;
        if !media.appearances.contains_key(&appearance) {
            return Err(ActorContentError::MissingAppearance(appearance));
        }
    }
    if let Some(position) = position
        && world.actors().definitions().is_some()
    {
        return world
            .actors()
            .birth_metadata(position, live_definition, requested, respawn);
    }
    Ok((
        ActorProvenance {
            authored_species: None,
            origin: ActorBirthOrigin::CanonicalImport,
        },
        requested,
    ))
}

fn unique_values<'a, T: Ord + 'a>(
    values: impl Iterator<Item = &'a T>,
) -> Result<(), ActorContentError> {
    let mut seen = std::collections::BTreeSet::new();
    for value in values {
        if !seen.insert(value) {
            return Err(ActorContentError::AmbiguousCompatibilityKey);
        }
    }
    Ok(())
}
fn validate_address(bank: u8, offset: u16) -> Result<(), ActorContentError> {
    if matches!(bank, 0x7e | 0x7f) || offset & 0x8000 == 0 {
        Err(ActorContentError::UnsupportedForm)
    } else {
        Ok(())
    }
}
fn validate_entry(
    entry: &PopulationEntry,
    definitions: &ActorDefinitions,
    media: &super::media::ActorMediaDefinitions,
) -> Result<(), ActorContentError> {
    if !definitions.species.contains_key(&entry.species) {
        return Err(ActorContentError::MissingSpecies(entry.species));
    }
    if !definitions.programs.contains_key(&entry.initial_program) {
        return Err(ActorContentError::MissingProgram(entry.initial_program));
    }
    if let Some(appearance) = entry.appearance
        && !media.appearances.contains_key(&appearance)
    {
        return Err(ActorContentError::MissingAppearance(appearance));
    }
    Ok(())
}

// Process-local revision evidence, matching WorldBinding; not a portable file hash.
fn gameplay_revision(
    definitions: &ActorDefinitions,
    keys: &ActorCompatibilityKeys,
) -> GameplayRevision {
    use std::hash::{DefaultHasher, Hash, Hasher};
    let mut revision = [0; 32];
    for (lane, bytes) in revision.as_chunks_mut::<8>().0.iter_mut().enumerate() {
        let mut hash = DefaultHasher::new();
        ("actor-gameplay-v1", lane as u64).hash(&mut hash);
        for (id, species) in &definitions.species {
            (
                id,
                species.initial_health,
                species.contact_damage,
                species.radius.x,
                species.radius.y,
                species.hurt_time,
                species.layer,
                species.behavior,
                species.multipart_count,
            )
                .hash(&mut hash);
        }
        for (id, program) in &definitions.programs {
            id.hash(&mut hash);
            match program {
                ActorProgramDefinition::CompatibleEnemyScript => 0u8.hash(&mut hash),
            }
        }
        for (id, population) in &definitions.populations {
            (id, population.death_quota, population.entries.len()).hash(&mut hash);
            for entry in &population.entries {
                hash_entry(entry, &mut hash);
            }
        }
        for (id, entry) in &definitions.templates {
            id.hash(&mut hash);
            hash_entry(entry, &mut hash);
        }
        for (id, pose) in &definitions.poses {
            (id, pose.collision, pose.parts.len()).hash(&mut hash);
            for part in &pose.parts {
                (part.role, part.offset.x, part.offset.y).hash(&mut hash);
            }
        }
        for (id, collision) in &definitions.collisions {
            id.hash(&mut hash);
            match collision {
                CollisionLayout::Body { radius, active } => {
                    (0u8, radius.x, radius.y, active).hash(&mut hash)
                }
                CollisionLayout::Parts(parts) => {
                    (1u8, parts.len()).hash(&mut hash);
                    for part in parts {
                        (part.offset.x, part.offset.y, part.hitboxes.len()).hash(&mut hash);
                        for hitbox in &part.hitboxes {
                            (
                                hitbox.left,
                                hitbox.top,
                                hitbox.right,
                                hitbox.bottom,
                                hitbox.touch,
                                hitbox.shot,
                            )
                                .hash(&mut hash);
                        }
                    }
                }
            }
        }
        (
            &keys.species,
            &keys.populations,
            &keys.templates,
            &keys.behaviors,
            &keys.programs,
            &keys.callbacks,
            &keys.poses,
            &keys.room_populations,
        )
            .hash(&mut hash);
        bytes.copy_from_slice(&hash.finish().to_le_bytes());
    }
    GameplayRevision(revision)
}
fn hash_entry(entry: &PopulationEntry, hash: &mut impl std::hash::Hasher) {
    use std::hash::Hash;
    (
        entry.id,
        entry.species,
        entry.position.x,
        entry.position.y,
        entry.initial_program,
        entry.properties,
        entry.extra_properties,
        entry.parameters,
    )
        .hash(hash);
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ActorCountWidth {
    Byte,
    NativeWord,
}
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ActorPartTraversal {
    CapturedLinear,
    WrappingReacquire { bank: u8, address: u16 },
}
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ActorPartWord {
    X,
    Y,
    Map,
    Hitboxes,
}
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ActorCollisionWord {
    Left,
    Right,
    Top,
    Bottom,
    Touch,
    Shot,
}

/// Projectile contact captures the offset before effects and samples bank later.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ActorCallbackTarget {
    CompatibleOffset(u16),
    Prepared(ActorRoutine),
}
impl ActorCallbackTarget {
    pub fn bind(self, bank: u8) -> ActorRoutine {
        match self {
            Self::CompatibleOffset(offset) => ActorRoutine::from_bank_offset(bank, offset),
            Self::Prepared(routine) => routine,
        }
    }
}
