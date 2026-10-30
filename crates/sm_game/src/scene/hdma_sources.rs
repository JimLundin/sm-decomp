//! Prepared HDMA definitions and exact bindings to their current source owner.
#![forbid(unsafe_code)]

use super::{
    records::AssetId,
    state::{DisplayError, HdmaState},
};
use std::{collections::BTreeMap, sync::Arc};

pub const HDMA_CONTENT_SCHEMA: u16 = 1;

/// Gameplay identity: these definitions can affect timers, callbacks and state.
/// `definition` is importer provenance, not a substitute for comparing payloads.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct DefinitionRef {
    pub id: AssetId,
    pub gameplay: [u8; 32],
    pub definition: [u8; 32],
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct HdmaTableDefinition {
    pub reference: DefinitionRef,
    pub schema: u16,
    /// Encoded repeat counts, indirect addresses and direct-transfer bytes.
    /// No decoding or execution is performed during construction.
    pub bytes: Arc<[u8]>,
}
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct HdmaIndirectDefinition {
    pub reference: DefinitionRef,
    pub schema: u16,
    pub bytes: Arc<[u8]>,
}
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct HdmaProgramDefinition {
    pub reference: DefinitionRef,
    pub schema: u16,
    /// Original encoded instructions and operands; reads remain lazy.
    pub bytes: Arc<[u8]>,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ContentKind {
    Table,
    Indirect,
    Program,
}
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum HdmaDefinition {
    Table(HdmaTableDefinition),
    Indirect(HdmaIndirectDefinition),
    Program(HdmaProgramDefinition),
}
impl HdmaDefinition {
    pub fn reference(&self) -> DefinitionRef {
        match self {
            Self::Table(v) => v.reference,
            Self::Indirect(v) => v.reference,
            Self::Program(v) => v.reference,
        }
    }
    pub fn kind(&self) -> ContentKind {
        match self {
            Self::Table(_) => ContentKind::Table,
            Self::Indirect(_) => ContentKind::Indirect,
            Self::Program(_) => ContentKind::Program,
        }
    }
    fn schema(&self) -> u16 {
        match self {
            Self::Table(v) => v.schema,
            Self::Indirect(v) => v.schema,
            Self::Program(v) => v.schema,
        }
    }
    fn bytes(&self) -> &[u8] {
        match self {
            Self::Table(v) => &v.bytes,
            Self::Indirect(v) => &v.bytes,
            Self::Program(v) => &v.bytes,
        }
    }
}

/// Immutable GAMEPLAY definitions, separate from SceneMedia pixel resources.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct HdmaCatalog {
    gameplay: [u8; 32],
    definitions: BTreeMap<AssetId, HdmaDefinition>,
}
impl HdmaCatalog {
    pub fn checkpoint_definitions(&self) -> &BTreeMap<AssetId, HdmaDefinition> {
        &self.definitions
    }
    pub fn new(gameplay: [u8; 32], definitions: Vec<HdmaDefinition>) -> Result<Self, DisplayError> {
        if gameplay == [0; 32] {
            return Err(DisplayError::Identity);
        }
        let mut result = Self {
            gameplay,
            definitions: BTreeMap::new(),
        };
        for value in definitions {
            let reference = value.reference();
            if reference.id.0 == 0 || reference.definition == [0; 32] {
                return Err(DisplayError::Identity);
            }
            if reference.gameplay != gameplay {
                return Err(DisplayError::Revision);
            }
            if value.schema() != HDMA_CONTENT_SCHEMA {
                return Err(DisplayError::Source);
            }
            if result.definitions.insert(reference.id, value).is_some() {
                return Err(DisplayError::Identity);
            }
        }
        Ok(result)
    }
    pub fn gameplay(&self) -> [u8; 32] {
        self.gameplay
    }
    /// Expected catalog comes independently from validated G requirements.
    /// Equal fingerprints with changed bytes, kinds, schema or declarations fail.
    pub fn validate_expected(&self, expected: &Self) -> Result<(), DisplayError> {
        if self != expected {
            return Err(DisplayError::Source);
        }
        Ok(())
    }
    pub fn definition(
        &self,
        reference: DefinitionRef,
        kind: ContentKind,
    ) -> Result<&HdmaDefinition, DisplayError> {
        if reference.gameplay != self.gameplay {
            return Err(DisplayError::Revision);
        }
        let value = self
            .definitions
            .get(&reference.id)
            .ok_or(DisplayError::Identity)?;
        if value.reference() != reference {
            return Err(DisplayError::Identity);
        }
        if value.kind() != kind {
            return Err(DisplayError::Source);
        }
        Ok(value)
    }
}

/// A compatibility address window, not a payload-offset approximation of RAM.
/// Windows never cross 10000; a wrapping range is represented by two windows.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct BankWindow {
    pub bank: u8,
    pub address: u16,
    pub offset: usize,
    pub length: usize,
}
impl BankWindow {
    fn validate(&self, source_length: usize) -> Result<(), DisplayError> {
        if self.length == 0
            || self.length > 65_536 - usize::from(self.address)
            || self
                .offset
                .checked_add(self.length)
                .ok_or(DisplayError::Source)?
                > source_length
        {
            return Err(DisplayError::Source);
        }
        Ok(())
    }
    fn resolve(&self, bank: u8, address: u16) -> Option<usize> {
        if bank != self.bank {
            return None;
        }
        let displacement = usize::from(address).checked_sub(usize::from(self.address))?;
        if displacement >= self.length {
            return None;
        }
        self.offset.checked_add(displacement)
    }
}

/// Existing mutable G buffers, never an immutable captured RAM image.
/// Numbered tables/windows are the explicit ram_map owners at9800/9d00/9e00/9e10.
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub enum LiveOwner {
    MessageDescriptor,
    MessageScanlines,
    HdmaTable1,
    HdmaTable2,
    WindowLeft,
    WindowRight,
    SkyBg2,
    CrocomireBg2,
    ObjectVariable { object: u8, variable: HdmaVariable },
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub enum HdmaVariable {
    A,
    B,
    C,
    D,
    E,
    F,
}
impl HdmaVariable {
    fn index(self) -> usize {
        match self {
            Self::A => 0,
            Self::B => 1,
            Self::C => 2,
            Self::D => 3,
            Self::E => 4,
            Self::F => 5,
        }
    }
}

/// One-way starting ownership transfer; bytes are current domain-buffer data.
/// Aliases are regions into this allocation, not additional copied buffers.
pub struct HdmaOwnedBuffer {
    pub owner: LiveOwner,
    pub bytes: Vec<u8>,
}
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum HdmaRegionSource {
    Buffer {
        storage: LiveOwner,
        offset: usize,
    },
    /// This subtype aliases the existing HdmaState owner, including byte writes.
    ObjectVariable {
        object: u8,
        variable: HdmaVariable,
    },
}
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct HdmaLiveRegion {
    pub owner: LiveOwner,
    pub source: HdmaRegionSource,
    pub length: usize,
}

/// All live HDMA buffers needed by sealed construction, checkpoint and reset.
/// Clone snapshots each allocation once and preserves region alias topology.
/// Reset consumes that snapshot; it needs no external source lifetime or host.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct HdmaLiveDescription {
    buffers: BTreeMap<LiveOwner, Vec<u8>>,
    regions: BTreeMap<LiveOwner, HdmaLiveRegion>,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum CanonicalOwner {
    Buffer(LiveOwner),
    ObjectVariable { object: u8, variable: HdmaVariable },
}

#[derive(Clone, Copy)]
struct CanonicalWindow {
    window: BankWindow,
    /// None retains independent immutable prepared-definition provenance.
    live: Option<(CanonicalOwner, usize)>,
}

impl HdmaLiveDescription {
    pub fn checkpoint_parts(
        &self,
    ) -> (
        &BTreeMap<LiveOwner, Vec<u8>>,
        &BTreeMap<LiveOwner, HdmaLiveRegion>,
    ) {
        (&self.buffers, &self.regions)
    }
    pub fn new(
        buffers: Vec<HdmaOwnedBuffer>,
        regions: Vec<HdmaLiveRegion>,
    ) -> Result<Self, DisplayError> {
        let mut result = Self {
            buffers: BTreeMap::new(),
            regions: BTreeMap::new(),
        };
        for buffer in buffers {
            if matches!(buffer.owner, LiveOwner::ObjectVariable { .. })
                || buffer.bytes.is_empty()
                || buffer.bytes.len() > 65_536
            {
                return Err(DisplayError::Source);
            }
            if result.buffers.insert(buffer.owner, buffer.bytes).is_some() {
                return Err(DisplayError::Identity);
            }
        }
        for region in regions {
            if result.regions.insert(region.owner, region).is_some() {
                return Err(DisplayError::Identity);
            }
        }
        result.validate()?;
        Ok(result)
    }
    pub fn validate(&self) -> Result<(), DisplayError> {
        for region in self.regions.values() {
            if region.length == 0 {
                return Err(DisplayError::Source);
            }
            match region.source {
                HdmaRegionSource::Buffer { storage, offset } => {
                    if matches!(region.owner, LiveOwner::ObjectVariable { .. }) {
                        return Err(DisplayError::Source);
                    }
                    let bytes = self.buffers.get(&storage).ok_or(DisplayError::Source)?;
                    if offset
                        .checked_add(region.length)
                        .ok_or(DisplayError::Source)?
                        > bytes.len()
                    {
                        return Err(DisplayError::Source);
                    }
                }
                HdmaRegionSource::ObjectVariable { object, variable } => {
                    if object >= 6
                        || region.length != 2
                        || region.owner != (LiveOwner::ObjectVariable { object, variable })
                    {
                        return Err(DisplayError::Source);
                    }
                }
            }
        }
        // Each storage has a complete, same-owner region. Additional overlapping
        // views must resolve back to it rather than manufacture independent data.
        for (owner, bytes) in &self.buffers {
            let region = self.regions.get(owner).ok_or(DisplayError::Source)?;
            if region.length != bytes.len()
                || region.source
                    != (HdmaRegionSource::Buffer {
                        storage: *owner,
                        offset: 0,
                    })
            {
                return Err(DisplayError::Source);
            }
        }
        Ok(())
    }
    /// Compare declared physical aliases without reading execution bytes.
    /// Caller validates individual definitions/windows before this cross-binding check.
    pub fn validate_canonical_aliases(&self, state: &HdmaState) -> Result<(), DisplayError> {
        let bindings = state
            .channels
            .iter()
            .flat_map(|channel| [&channel.table, &channel.source])
            .chain(state.objects.iter().map(|object| &object.instruction));
        let mut mapped: Vec<CanonicalWindow> = Vec::new();
        for binding in bindings {
            if matches!(binding, SourceBinding::Unbound) {
                continue;
            }
            for window in binding.windows()? {
                let live = match binding {
                    SourceBinding::LiveOwner { owner, .. } => {
                        let region = self.region(*owner, window.offset)?;
                        Some(match region.source {
                            HdmaRegionSource::Buffer { storage, offset } => (
                                CanonicalOwner::Buffer(storage),
                                offset
                                    .checked_add(window.offset)
                                    .ok_or(DisplayError::Source)?,
                            ),
                            HdmaRegionSource::ObjectVariable { object, variable } => (
                                CanonicalOwner::ObjectVariable { object, variable },
                                window.offset,
                            ),
                        })
                    }
                    SourceBinding::Prepared { .. } => None,
                    SourceBinding::Unbound => unreachable!(),
                };
                for prior in &mapped {
                    if window.bank != prior.window.bank {
                        continue;
                    }
                    let start = usize::from(window.address).max(usize::from(prior.window.address));
                    let end = (usize::from(window.address) + window.length)
                        .min(usize::from(prior.window.address) + prior.window.length);
                    if start >= end {
                        continue;
                    }
                    match (live, prior.live) {
                        (Some((owner, offset)), Some((prior_owner, prior_offset))) => {
                            let at = offset
                                .checked_add(start - usize::from(window.address))
                                .ok_or(DisplayError::Source)?;
                            let prior_at = prior_offset
                                .checked_add(start - usize::from(prior.window.address))
                                .ok_or(DisplayError::Source)?;
                            // Both mappings advance one backing byte per physical byte.
                            if owner != prior_owner || at != prior_at {
                                return Err(DisplayError::Source);
                            }
                        }
                        (None, None) => {}
                        // An immutable payload cannot share an independently mutable byte.
                        _ => return Err(DisplayError::Source),
                    }
                }
                mapped.push(CanonicalWindow {
                    window: *window,
                    live,
                });
            }
        }
        Ok(())
    }
    fn region(&self, owner: LiveOwner, offset: usize) -> Result<HdmaLiveRegion, DisplayError> {
        let region = self.regions.get(&owner).ok_or(DisplayError::Source)?;
        if offset >= region.length {
            return Err(DisplayError::Alias { offset });
        }
        Ok(*region)
    }
    /// G producers use these same methods at the original byte publications.
    pub fn read(
        &self,
        state: &HdmaState,
        owner: LiveOwner,
        offset: usize,
    ) -> Result<u8, DisplayError> {
        match self.region(owner, offset)?.source {
            HdmaRegionSource::Buffer {
                storage,
                offset: first,
            } => {
                let at = first
                    .checked_add(offset)
                    .ok_or(DisplayError::Alias { offset })?;
                self.buffers
                    .get(&storage)
                    .and_then(|bytes| bytes.get(at))
                    .copied()
                    .ok_or(DisplayError::Alias { offset: at })
            }
            HdmaRegionSource::ObjectVariable { object, variable } => {
                let object = state
                    .objects
                    .get(usize::from(object))
                    .ok_or(DisplayError::Source)?;
                object.variables[variable.index()]
                    .to_le_bytes()
                    .get(offset)
                    .copied()
                    .ok_or(DisplayError::Alias { offset })
            }
        }
    }
    pub fn write(
        &mut self,
        state: &mut HdmaState,
        owner: LiveOwner,
        offset: usize,
        value: u8,
    ) -> Result<(), DisplayError> {
        match self.region(owner, offset)?.source {
            HdmaRegionSource::Buffer {
                storage,
                offset: first,
            } => {
                let at = first
                    .checked_add(offset)
                    .ok_or(DisplayError::Alias { offset })?;
                *self
                    .buffers
                    .get_mut(&storage)
                    .and_then(|bytes| bytes.get_mut(at))
                    .ok_or(DisplayError::Alias { offset: at })? = value;
            }
            HdmaRegionSource::ObjectVariable { object, variable } => {
                let object = state
                    .objects
                    .get_mut(usize::from(object))
                    .ok_or(DisplayError::Source)?;
                let word = &mut object.variables[variable.index()];
                let mut bytes = word.to_le_bytes();
                *bytes
                    .get_mut(offset)
                    .ok_or(DisplayError::Alias { offset })? = value;
                *word = u16::from_le_bytes(bytes);
            }
        }
        Ok(())
    }
}
impl LiveSources for HdmaLiveDescription {
    fn length(&self, owner: LiveOwner) -> Result<usize, DisplayError> {
        self.regions
            .get(&owner)
            .map(|region| region.length)
            .ok_or(DisplayError::Source)
    }
    fn read(
        &mut self,
        state: &mut HdmaState,
        owner: LiveOwner,
        offset: usize,
    ) -> Result<u8, DisplayError> {
        HdmaLiveDescription::read(self, state, owner, offset)
    }
    fn write(
        &mut self,
        state: &mut HdmaState,
        owner: LiveOwner,
        offset: usize,
        value: u8,
    ) -> Result<(), DisplayError> {
        HdmaLiveDescription::write(self, state, owner, offset, value)
    }
}

/// An adapter borrows the actual owners. Reads/writes may encounter aliases to
/// HDMA fields, so receive the same state the byte algorithm has just published.
/// `length` is an admission query only; it must not execute or capture bytes.
pub trait LiveSources {
    fn length(&self, owner: LiveOwner) -> Result<usize, DisplayError>;
    fn read(
        &mut self,
        state: &mut HdmaState,
        owner: LiveOwner,
        offset: usize,
    ) -> Result<u8, DisplayError>;
    fn write(
        &mut self,
        state: &mut HdmaState,
        owner: LiveOwner,
        offset: usize,
        value: u8,
    ) -> Result<(), DisplayError>;
}

/// For a nonaliased actual byte buffer, both operations use this same borrow.
/// Word/structured or HDMA-field aliases need a LiveSources implementation that
/// preserves their original byte order and partial publications.
pub struct LiveBuffer<'a> {
    pub owner: LiveOwner,
    pub bytes: &'a mut [u8],
}
impl LiveBuffer<'_> {
    pub fn read(&self, owner: LiveOwner, offset: usize) -> Result<u8, DisplayError> {
        if owner != self.owner {
            return Err(DisplayError::Source);
        }
        self.bytes
            .get(offset)
            .copied()
            .ok_or(DisplayError::Alias { offset })
    }
    pub fn write(
        &mut self,
        owner: LiveOwner,
        offset: usize,
        value: u8,
    ) -> Result<(), DisplayError> {
        if owner != self.owner {
            return Err(DisplayError::Source);
        }
        *self
            .bytes
            .get_mut(offset)
            .ok_or(DisplayError::Alias { offset })? = value;
        Ok(())
    }
}
impl LiveSources for LiveBuffer<'_> {
    fn length(&self, owner: LiveOwner) -> Result<usize, DisplayError> {
        if owner != self.owner {
            return Err(DisplayError::Source);
        }
        Ok(self.bytes.len())
    }
    fn read(
        &mut self,
        _state: &mut HdmaState,
        owner: LiveOwner,
        offset: usize,
    ) -> Result<u8, DisplayError> {
        LiveBuffer::read(self, owner, offset)
    }
    fn write(
        &mut self,
        _state: &mut HdmaState,
        owner: LiveOwner,
        offset: usize,
        value: u8,
    ) -> Result<(), DisplayError> {
        LiveBuffer::write(self, owner, offset, value)
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum SourceBinding {
    Prepared {
        reference: DefinitionRef,
        windows: Arc<[BankWindow]>,
    },
    LiveOwner {
        owner: LiveOwner,
        windows: Arc<[BankWindow]>,
    },
    /// Retained dormant/reset cursor provenance. Actual reads still fault.
    Unbound,
}
impl SourceBinding {
    fn windows(&self) -> Result<&[BankWindow], DisplayError> {
        match self {
            Self::Prepared { windows, .. } | Self::LiveOwner { windows, .. } => Ok(windows),
            Self::Unbound => Err(DisplayError::Source),
        }
    }
    fn offset(&self, bank: u8, address: u16) -> Result<usize, DisplayError> {
        let mut found = None;
        for window in self.windows()? {
            if let Some(offset) = window.resolve(bank, address)
                && found.replace(offset).is_some()
            {
                return Err(DisplayError::Source);
            }
        }
        found.ok_or(DisplayError::Source)
    }
    /// Validate declarations and mapping bounds, never preflight execution.
    pub fn validate(
        &self,
        catalog: &HdmaCatalog,
        live: &impl LiveSources,
        kind: ContentKind,
        required: bool,
    ) -> Result<(), DisplayError> {
        let length = match self {
            Self::Prepared { reference, .. } => catalog.definition(*reference, kind)?.bytes().len(),
            Self::LiveOwner { owner, .. } => {
                if kind == ContentKind::Program {
                    return Err(DisplayError::Source);
                }
                live.length(*owner)?
            }
            Self::Unbound => {
                return if required {
                    Err(DisplayError::Incomplete)
                } else {
                    Ok(())
                };
            }
        };
        let windows = self.windows()?;
        if windows.is_empty() {
            return Err(DisplayError::Source);
        }
        for (index, window) in windows.iter().enumerate() {
            window.validate(length)?;
            for other in &windows[..index] {
                if window.bank == other.bank
                    && usize::from(window.address) < usize::from(other.address) + other.length
                    && usize::from(other.address) < usize::from(window.address) + window.length
                {
                    return Err(DisplayError::Source);
                }
            }
        }
        Ok(())
    }
    /// Exactly one source access, using the bank/address supplied by hdma.rs.
    /// Do not infer addresses from a source payload or reacquire channel fields.
    pub fn read(
        &self,
        catalog: &HdmaCatalog,
        live: &mut impl LiveSources,
        state: &mut HdmaState,
        kind: ContentKind,
        bank: u8,
        address: u16,
    ) -> Result<u8, DisplayError> {
        let offset = self.offset(bank, address)?;
        match self {
            Self::Prepared { reference, .. } => catalog
                .definition(*reference, kind)?
                .bytes()
                .get(offset)
                .copied()
                .ok_or(DisplayError::Source),
            Self::LiveOwner { owner, .. } if kind != ContentKind::Program => {
                live.read(state, *owner, offset)
            }
            _ => Err(DisplayError::Source),
        }
    }
    /// Reverse transfers publish to the actual mutable owner after the prior
    /// destination read. Prepared definitions cannot acquire a mutable shadow.
    pub fn write(
        &self,
        live: &mut impl LiveSources,
        state: &mut HdmaState,
        bank: u8,
        address: u16,
        value: u8,
    ) -> Result<(), DisplayError> {
        let offset = self.offset(bank, address)?;
        match self {
            Self::LiveOwner { owner, .. } => live.write(state, *owner, offset, value),
            _ => Err(DisplayError::Source),
        }
    }
    pub fn validate_channel(
        table: &Self,
        indirect: &Self,
        catalog: &HdmaCatalog,
        live: &impl LiveSources,
        state: &HdmaState,
        channel: usize,
    ) -> Result<(), DisplayError> {
        let channel = state.channels.get(channel).ok_or(DisplayError::Source)?;
        // initialize reads every active channel even if previously terminated.
        // General DMA can also consume these owners while HDMA is disabled.
        let required = channel.active || channel.dma_active;
        table.validate(catalog, live, ContentKind::Table, required)?;
        indirect.validate(
            catalog,
            live,
            ContentKind::Indirect,
            required && channel.indirect,
        )
    }
    pub fn validate_program(
        &self,
        catalog: &HdmaCatalog,
        live: &impl LiveSources,
        state: &HdmaState,
        object: usize,
    ) -> Result<(), DisplayError> {
        let binding = state.bindings.get(object).ok_or(DisplayError::Source)?;
        // Only explicit object disable or global HDMA-object disable permits
        // missing program provenance. Timer/sleep is not proof of inactivity.
        self.validate(
            catalog,
            live,
            ContentKind::Program,
            state.flags & 0x8000 != 0 && binding.enable as u8 != 0,
        )
    }
}
