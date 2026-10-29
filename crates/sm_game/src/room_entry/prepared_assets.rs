//! Immutable encoded room assets and exact mapped ROM windows for synchronous DMA.
#![forbid(unsafe_code)]
use super::{
    content::{AssetRef, AssetSource, SharedAsset},
    prepared_tables::{self, Output, PreparedTableError, PreparedTileTables},
    world::*,
};
use std::{
    collections::{BTreeMap, BTreeSet},
    sync::Arc,
};

// SMRA: schema1, decoder1, vanilla reference profile1, options0. Integers are LE.
const HEADER: &[u8; 16] = b"SMRA\x01\0\x01\0\x01\0\0\0\0\0\0\0";
const MAX_PACK: usize = 33_554_432;
const MAX_STREAM: usize = 262_144;
const MAX_RECORDS: usize = 4096;
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum PreparedRoomError {
    World(WorldDefinitionError),
    Tables(PreparedTableError),
    Incompatible,
    Malformed,
    Limit,
    MissingPayload,
}
impl core::fmt::Display for PreparedRoomError {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        match self {
            Self::World(error) => write!(f, "room definition: {error:?}"),
            Self::Tables(error) => write!(f, "room tile tables: {error:?}"),
            Self::Incompatible => f.write_str("incompatible prepared room assets"),
            Self::Malformed => f.write_str("malformed prepared room assets"),
            Self::Limit => f.write_str("prepared room assets exceed limits"),
            Self::MissingPayload => f.write_str("prepared room payload is unavailable"),
        }
    }
}
impl std::error::Error for PreparedRoomError {}
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
enum StreamId {
    Cre,
    Resource(ResourceId),
}
#[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
enum Target {
    Ram(usize),
    Vram(i32),
}
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct PreparedRoomAssets {
    compatibility: [u8; 32],
    fingerprint: [u8; 32],
    tables: PreparedTileTables,
    streams: BTreeMap<StreamId, Arc<[u8]>>,
    // One enclosing window contains exact caller bytes, including gaps. It starts
    // at mapped index0 so the cartridge's original size mask remains covered.
    raw: Arc<[u8]>,
    source_size: usize,
}
fn address(key: ResourceKey) -> u32 {
    match key {
        ResourceKey::Word { bank, offset } => u32::from(bank) << 16 | u32::from(offset),
        ResourceKey::Long(LegacyAddress24([lo, hi, bank])) => u32::from_le_bytes([lo, hi, bank, 0]),
    }
}
fn raw_end(source: u32) -> usize {
    // DMA keeps the bank and wraps its16-bit address; the special pre-read's t+1
    // instead carries linearly. Include both aliased halves and that extra byte.
    (sm_data::cartridge::lorom_offset(source & 0xff0000) as usize + 0x8001).min(0x400000)
}
fn specifications(
    world: &WorldDefinition,
    keys: &CompatibilityKeys,
) -> Result<(BTreeMap<StreamId, BTreeSet<Target>>, usize), PreparedRoomError> {
    WorldBinding::prepared(world.clone(), keys.clone()).map_err(PreparedRoomError::World)?;
    let mut streams = BTreeMap::<_, BTreeSet<_>>::new();
    streams.insert(
        StreamId::Cre,
        BTreeSet::from([Target::Ram(0x7000), Target::Vram(0x5000)]),
    );
    for (&id, &kind) in &world.resources {
        let targets = match kind {
            ResourceKind::Level => vec![Target::Ram(0x10000)],
            ResourceKind::Tiles => vec![Target::Ram(0x2000), Target::Vram(0)],
            ResourceKind::Palette
                if world.tilesets.values().any(|tileset| tileset.palette == id) =>
            {
                vec![Target::Ram(0xc200)]
            }
            _ => Vec::new(),
        };
        if !targets.is_empty() {
            streams.insert(StreamId::Resource(id), targets.into_iter().collect());
        }
    }
    // FX native records/selector slots and area spawn lists live in83; blend in89,
    // FX tilemaps in8a, blanks in80 and the transition upload in9a. Exact caller
    // windows cover full/odd native offsets and all8 indexed slots without zeros.
    let mut end = [0x830000, 0x89aa02, 0x8a0000, 0x80a29a, 0x9ab200]
        .into_iter()
        .map(raw_end)
        .max()
        .unwrap();
    // Native FX records may straddle the end of a mapped bank by15 bytes.
    end = end.max(sm_data::cartridge::lorom_offset(0x838000) as usize + 0x8010);
    for state in world.states.values() {
        for command in &state.background {
            match command {
                BackgroundCommand::Decompress {
                    source,
                    destination,
                } => {
                    let base = match destination {
                        BackgroundTarget::Main | BackgroundTarget::Fx => 0x4000,
                        BackgroundTarget::Secondary => 0x5000,
                    };
                    streams
                        .entry(StreamId::Resource(*source))
                        .or_default()
                        .insert(Target::Ram(base));
                }
                BackgroundCommand::Transfer { source, .. }
                | BackgroundCommand::TransferAndSetCopyPolicy { source, .. }
                | BackgroundCommand::MatchExitTransfer { source, .. } => {
                    let source = address(keys.resources[source]);
                    if !matches!((source >> 16) as u8, 0x7e | 0x7f) {
                        end = end.max(raw_end(source));
                    }
                }
                _ => {}
            }
        }
    }
    if streams.len() > MAX_RECORDS || end > 0x400001 {
        return Err(PreparedRoomError::Limit);
    }
    Ok((streams, end))
}
impl PreparedRoomAssets {
    /// Input has the native-selected copier header removed. Mirror the final blocks
    /// exactly like snes_other::snes_loadRom; no backend or mutable image is needed.
    pub fn normalize_source(source: &[u8]) -> Result<Vec<u8>, PreparedRoomError> {
        if source.len() < 0x8000 || source.len() > 0x400000 {
            return Err(PreparedRoomError::Limit);
        }
        let mut length = 0x8000usize;
        while source.len() > length {
            length = length.checked_mul(2).ok_or(PreparedRoomError::Limit)?;
        }
        let mut bytes = Vec::with_capacity(length);
        bytes.extend_from_slice(source);
        let mut test = 1usize;
        while bytes.len() != length {
            if bytes.len() & test != 0 {
                let start = bytes
                    .len()
                    .checked_sub(test)
                    .ok_or(PreparedRoomError::Malformed)?;
                bytes.extend_from_within(start..);
            }
            test = test.checked_mul(2).ok_or(PreparedRoomError::Limit)?;
        }
        Ok(bytes)
    }
    /// Source is the independent normalized cart allocation, including mirrors.
    pub fn import(
        source: &[u8],
        world: &WorldDefinition,
        keys: &CompatibilityKeys,
    ) -> Result<Self, PreparedRoomError> {
        let (specs, end) = specifications(world, keys)?;
        let source_size = source.len();
        if source_size < 0xd8001 || source_size > 0x400000 {
            return Err(PreparedRoomError::Limit);
        }
        if !source_size.is_power_of_two() {
            return Err(PreparedRoomError::Incompatible);
        }
        let end = end.min(source_size);
        let tables =
            PreparedTileTables::import(source, world, keys).map_err(PreparedRoomError::Tables)?;
        let mut streams = BTreeMap::new();
        let mut total = end + tables.to_bytes().len() + 96 + specs.len() * 9;
        if total > MAX_PACK {
            return Err(PreparedRoomError::Limit);
        }
        for (&id, targets) in &specs {
            let source_address = match id {
                StreamId::Cre => 0xb98000,
                StreamId::Resource(id) => address(keys.resources[&id]),
            };
            let stream = validate(source, Some(source_address), targets)?;
            total += stream.len();
            if total > MAX_PACK {
                return Err(PreparedRoomError::Limit);
            }
            streams.insert(id, Arc::from(stream));
        }
        let raw = Arc::from(source.get(..end).ok_or(PreparedRoomError::MissingPayload)?);
        let mut result = Self {
            compatibility: prepared_tables::compatibility(world, keys),
            fingerprint: [0; 32],
            tables,
            streams,
            raw,
            source_size,
        };
        result.validate_fx(world, keys)?;
        result.fingerprint = Self::checksum(&result.body());
        Ok(result)
    }
    fn body(&self) -> Vec<u8> {
        let mut bytes = HEADER.to_vec();
        bytes.extend_from_slice(&self.compatibility);
        let tables = self.tables.to_bytes();
        bytes.extend_from_slice(&(tables.len() as u32).to_le_bytes());
        bytes.extend_from_slice(&tables);
        bytes.extend_from_slice(&(self.streams.len() as u32).to_le_bytes());
        for (&id, stream) in &self.streams {
            let (tag, id) = match id {
                StreamId::Cre => (0, 0),
                StreamId::Resource(id) => (1, id.0),
            };
            bytes.push(tag);
            bytes.extend_from_slice(&id.to_le_bytes());
            bytes.extend_from_slice(&(stream.len() as u32).to_le_bytes());
            bytes.extend_from_slice(stream);
        }
        bytes.extend_from_slice(&(self.source_size as u32).to_le_bytes());
        bytes.extend_from_slice(&(self.raw.len() as u32).to_le_bytes());
        bytes.extend_from_slice(&self.raw);
        bytes
    }
    pub fn to_bytes(&self) -> Vec<u8> {
        let mut bytes = self.body();
        bytes.extend_from_slice(&self.fingerprint);
        bytes
    }
    pub fn load(
        bytes: &[u8],
        world: &WorldDefinition,
        keys: &CompatibilityKeys,
    ) -> Result<Self, PreparedRoomError> {
        if bytes.len() > MAX_PACK {
            return Err(PreparedRoomError::Limit);
        }
        let (specs, end) = specifications(world, keys)?;
        let compatibility = prepared_tables::compatibility(world, keys);
        let mut rest = bytes;
        if take(&mut rest, 16)? != HEADER || take(&mut rest, 32)? != compatibility {
            return Err(PreparedRoomError::Incompatible);
        }
        let count = number(&mut rest)?;
        if count > MAX_PACK {
            return Err(PreparedRoomError::Limit);
        }
        let tables = PreparedTileTables::load(take(&mut rest, count)?, world, keys)
            .map_err(PreparedRoomError::Tables)?;
        if number(&mut rest)? != specs.len() {
            return Err(PreparedRoomError::Malformed);
        }
        let mut streams = BTreeMap::new();
        for (expected, targets) in specs {
            let tag = take(&mut rest, 1)?[0];
            let id = number(&mut rest)? as u32;
            let actual = match (tag, id) {
                (0, 0) => StreamId::Cre,
                (1, id) => StreamId::Resource(ResourceId(id)),
                _ => return Err(PreparedRoomError::Malformed),
            };
            if actual != expected {
                return Err(PreparedRoomError::Malformed);
            }
            let length = number(&mut rest)?;
            if length > MAX_STREAM {
                return Err(PreparedRoomError::Limit);
            }
            let stream = take(&mut rest, length)?;
            let validated = validate(stream, None, &targets)?;
            if validated.len() != length {
                return Err(PreparedRoomError::Malformed);
            }
            streams.insert(actual, Arc::from(validated));
        }
        let source_size = number(&mut rest)?;
        if source_size < 0xd8001 || source_size > 0x400000 {
            return Err(PreparedRoomError::Limit);
        }
        if !source_size.is_power_of_two() {
            return Err(PreparedRoomError::Incompatible);
        }
        let end = end.min(source_size);
        if number(&mut rest)? != end {
            return Err(PreparedRoomError::Malformed);
        }
        let raw = Arc::from(take(&mut rest, end)?);
        let fingerprint: [u8; 32] = take(&mut rest, 32)?.try_into().unwrap();
        if !rest.is_empty() || fingerprint != Self::checksum(&bytes[..bytes.len() - 32]) {
            return Err(PreparedRoomError::Malformed);
        }
        let result = Self {
            compatibility,
            fingerprint,
            tables,
            streams,
            raw,
            source_size,
        };
        result.validate_fx(world, keys)?;
        Ok(result)
    }
    fn validate_fx(
        &self,
        world: &WorldDefinition,
        keys: &CompatibilityKeys,
    ) -> Result<(), PreparedRoomError> {
        for (&id, &kind) in &world.resources {
            if kind != ResourceKind::Fx {
                continue;
            }
            let mut at = address(keys.resources[&id]) as u16;
            let mut terminated = false;
            for _ in 0..4096 {
                let mapped = sm_data::cartridge::lorom_offset(0x830000 | u32::from(at)) as usize;
                let bytes = self
                    .raw
                    .get(mapped..mapped + 16)
                    .ok_or(PreparedRoomError::MissingPayload)?;
                let door = u16::from_ne_bytes([bytes[0], bytes[1]]);
                if door == 0 || door == 0xffff {
                    terminated = true;
                    break;
                }
                at = at.wrapping_add(16);
            }
            if !terminated {
                return Err(PreparedRoomError::Malformed);
            }
        }
        Ok(())
    }
    pub fn matches(&self, world: &WorldDefinition, keys: &CompatibilityKeys) -> bool {
        self.compatibility == prepared_tables::compatibility(world, keys)
    }
    pub fn checksum(bytes: &[u8]) -> [u8; 32] {
        prepared_tables::fingerprint(bytes)
    }
    pub fn fingerprint(&self) -> [u8; 32] {
        self.fingerprint
    }
    pub fn tables(&self) -> &PreparedTileTables {
        &self.tables
    }
    pub fn stream(&self, source: &AssetSource) -> Option<Arc<[u8]>> {
        let id = match source {
            AssetSource::Shared(SharedAsset::CreTiles) => StreamId::Cre,
            AssetSource::Bound(AssetRef::Prepared(id)) => StreamId::Resource(*id),
            _ => return None,
        };
        self.streams.get(&id).cloned()
    }
    pub fn rom_byte(&self, mapped: usize) -> u8 {
        *self
            .raw
            .get(mapped)
            .expect("prepared room ROM window is unavailable")
    }
    pub fn source_size(&self) -> usize {
        self.source_size
    }
    pub fn raw_window(&self) -> (usize, &[u8]) {
        (0, &self.raw)
    }
}
fn take<'a>(bytes: &mut &'a [u8], length: usize) -> Result<&'a [u8], PreparedRoomError> {
    let value = bytes.get(..length).ok_or(PreparedRoomError::Malformed)?;
    *bytes = &bytes[length..];
    Ok(value)
}
fn number(bytes: &mut &[u8]) -> Result<usize, PreparedRoomError> {
    Ok(u32::from_le_bytes(take(bytes, 4)?.try_into().unwrap()) as usize)
}
struct Validation<'a> {
    source: &'a [u8],
    address: Option<u32>,
    encoded: Vec<u8>,
    target: Target,
    produced: usize,
    error: bool,
}
impl Output for Validation<'_> {
    fn next_byte(&mut self) -> u8 {
        if self.error {
            return 0xff;
        }
        if self.encoded.len() == MAX_STREAM {
            self.error = true;
            return 0xff;
        }
        let at = self.address.map_or(self.encoded.len(), |at| {
            sm_data::cartridge::lorom_offset(at) as usize
        });
        let Some(&byte) = self.source.get(at) else {
            self.error = true;
            return 0xff;
        };
        self.encoded.push(byte);
        if let Some(at) = self.address.as_mut() {
            let old = *at;
            *at = old.wrapping_add(1);
            if old & 0xffff == 0xffff {
                *at = at.wrapping_add(0x8000);
            }
        }
        byte
    }
    fn read(&mut self, position: i32) -> u8 {
        if let Target::Ram(base) = self.target {
            if !(0..0x20000).contains(&(base as i64 + i64::from(position))) {
                self.error = true;
            }
        }
        // VRAM references retain the actual u16 byte-address wrap and live PPU read.
        0
    }
    fn write(&mut self, position: i32, _value: u8, _reference: bool) {
        self.read(position);
        self.produced += 1;
        if self.produced > MAX_STREAM {
            self.error = true;
        }
    }
}
fn validate(
    source: &[u8],
    address: Option<u32>,
    targets: &BTreeSet<Target>,
) -> Result<Vec<u8>, PreparedRoomError> {
    let mut encoded = Vec::new();
    for &target in targets {
        let mut output = Validation {
            source,
            address,
            encoded: Vec::new(),
            target,
            produced: 0,
            error: false,
        };
        prepared_tables::decode(
            match target {
                Target::Ram(_) => 0,
                Target::Vram(start) => start,
            },
            &mut output,
        );
        if output.error {
            return Err(PreparedRoomError::Malformed);
        }
        encoded = output.encoded;
    }
    Ok(encoded)
}
