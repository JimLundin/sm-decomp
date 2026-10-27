//! Immutable, validated room tile-table streams and their portable native pack.
#![forbid(unsafe_code)]
use super::{
    content::{AssetRef, AssetSource, SharedAsset},
    world::*,
};
use std::{collections::BTreeMap, sync::Arc};

// SMTT, schema u16LE=1, decoder-command u16LE=1, reference LoROM/RAM profile u32LE=1.
const HEADER: &[u8; 12] = b"SMTT\x01\0\x01\0\x01\0\0\0";
const MAX_STREAM: usize = 262_144;
const MAX_PACK: usize = 16_777_216;
const MAX_TABLES: usize = 1025;
const CRE_SOURCE: u32 = 0xb9a09d;
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
enum TableId {
    Cre,
    Resource(ResourceId),
}
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum PreparedTableError {
    World(WorldDefinitionError),
    Incompatible,
    Malformed,
    Limit,
}
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct PreparedTileTables {
    compatibility: [u8; 32],
    fingerprint: [u8; 32],
    streams: BTreeMap<TableId, Arc<[u8]>>,
}
impl PreparedTileTables {
    /// The caller supplies independent bytes, never a shared mutable RomImage clone.
    pub fn import(
        source: &[u8],
        world: &WorldDefinition,
        keys: &CompatibilityKeys,
    ) -> Result<Self, PreparedTableError> {
        let ids = Self::ids(world, keys)?;
        let mut streams = BTreeMap::new();
        let mut total = 80 + ids.len() * 9;
        for id in ids {
            let address = match id {
                TableId::Cre => CRE_SOURCE,
                TableId::Resource(resource) => match keys.resources[&resource] {
                    ResourceKey::Long(LegacyAddress24([lo, hi, bank])) => {
                        u32::from_le_bytes([lo, hi, bank, 0])
                    }
                    _ => return Err(PreparedTableError::Incompatible),
                },
            };
            let stream = validate(source, Some(address))?;
            total += stream.len();
            if total > MAX_PACK {
                return Err(PreparedTableError::Limit);
            }
            streams.insert(id, Arc::from(stream));
        }
        let mut tables = Self {
            compatibility: compatibility(world, keys),
            fingerprint: [0; 32],
            streams,
        };
        tables.fingerprint = fingerprint(&tables.body());
        Ok(tables)
    }
    fn ids(
        world: &WorldDefinition,
        keys: &CompatibilityKeys,
    ) -> Result<Vec<TableId>, PreparedTableError> {
        WorldBinding::prepared(world.clone(), keys.clone()).map_err(PreparedTableError::World)?;
        let ids: Vec<_> = core::iter::once(TableId::Cre)
            .chain(world.resources.iter().filter_map(|(&id, &kind)| {
                (kind == ResourceKind::TileTable).then_some(TableId::Resource(id))
            }))
            .collect();
        if ids.len() > MAX_TABLES {
            return Err(PreparedTableError::Limit);
        }
        Ok(ids)
    }
    fn body(&self) -> Vec<u8> {
        let mut bytes = HEADER.to_vec();
        bytes.extend_from_slice(&self.compatibility);
        bytes.extend_from_slice(&(self.streams.len() as u32).to_le_bytes());
        for (&id, stream) in &self.streams {
            let (tag, id) = match id {
                TableId::Cre => (0, 0),
                TableId::Resource(id) => (1, id.0),
            };
            bytes.push(tag);
            bytes.extend_from_slice(&id.to_le_bytes());
            bytes.extend_from_slice(&(stream.len() as u32).to_le_bytes());
            bytes.extend_from_slice(stream);
        }
        bytes
    }
    pub fn to_bytes(&self) -> Vec<u8> {
        let mut bytes = self.body();
        bytes.extend_from_slice(&self.fingerprint);
        bytes
    }
    /// Revalidate the canonical record set and command streams before ownership.
    pub fn load(
        bytes: &[u8],
        world: &WorldDefinition,
        keys: &CompatibilityKeys,
    ) -> Result<Self, PreparedTableError> {
        if bytes.len() > MAX_PACK {
            return Err(PreparedTableError::Limit);
        }
        let ids = Self::ids(world, keys)?;
        let mut remaining = bytes;
        if take(&mut remaining, 12)? != HEADER
            || take(&mut remaining, 32)? != compatibility(world, keys)
        {
            return Err(PreparedTableError::Incompatible);
        }
        let count = u32::from_le_bytes(take(&mut remaining, 4)?.try_into().unwrap()) as usize;
        if count > MAX_TABLES {
            return Err(PreparedTableError::Limit);
        }
        if count != ids.len() {
            return Err(PreparedTableError::Malformed);
        }
        let mut streams = BTreeMap::new();
        for expected in ids {
            let tag = take(&mut remaining, 1)?[0];
            let id = u32::from_le_bytes(take(&mut remaining, 4)?.try_into().unwrap());
            let actual = match (tag, id) {
                (0, 0) => TableId::Cre,
                (1, id) => TableId::Resource(ResourceId(id)),
                _ => return Err(PreparedTableError::Malformed),
            };
            if actual != expected {
                return Err(PreparedTableError::Malformed);
            }
            let length = u32::from_le_bytes(take(&mut remaining, 4)?.try_into().unwrap()) as usize;
            if length > MAX_STREAM {
                return Err(PreparedTableError::Limit);
            }
            let stream = take(&mut remaining, length)?;
            let validated = validate(stream, None)?;
            if validated.len() != stream.len() {
                return Err(PreparedTableError::Malformed);
            }
            streams.insert(actual, Arc::from(validated));
        }
        let digest: [u8; 32] = take(&mut remaining, 32)?.try_into().unwrap();
        if !remaining.is_empty() || digest != fingerprint(&bytes[..bytes.len() - 32]) {
            return Err(PreparedTableError::Malformed);
        }
        Ok(Self {
            compatibility: compatibility(world, keys),
            fingerprint: digest,
            streams,
        })
    }
    pub(super) fn matches(&self, world: &WorldDefinition, keys: &CompatibilityKeys) -> bool {
        self.compatibility == compatibility(world, keys)
    }
    pub(super) fn fingerprint(&self) -> [u8; 32] {
        self.fingerprint
    }
    pub(super) fn stream(&self, source: &AssetSource) -> Option<Arc<[u8]>> {
        let id = match source {
            AssetSource::Shared(SharedAsset::CreTileTable) => TableId::Cre,
            AssetSource::Bound(AssetRef::Prepared(id)) => TableId::Resource(*id),
            _ => return None,
        };
        self.streams.get(&id).cloned()
    }
}
fn take<'a>(bytes: &mut &'a [u8], length: usize) -> Result<&'a [u8], PreparedTableError> {
    let result = bytes.get(..length).ok_or(PreparedTableError::Malformed)?;
    *bytes = &bytes[length..];
    Ok(result)
}
struct Validation<'a> {
    source: &'a [u8],
    address: Option<u32>,
    encoded: Vec<u8>,
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
        let at = self.address.map_or(self.encoded.len(), |address| {
            crate::rom::lorom_offset(address) as usize
        });
        let Some(&byte) = self.source.get(at) else {
            self.error = true;
            return 0xff;
        };
        self.encoded.push(byte);
        if let Some(address) = self.address.as_mut() {
            let old = *address;
            *address = old.wrapping_add(1);
            if old & 0xffff == 0xffff {
                *address = address.wrapping_add(0x8000);
            }
        }
        byte
    }
    fn read(&mut self, position: i32) -> u8 {
        // Existing/unproduced destination bytes are valid references at BOTH bases.
        for base in [0xa000, 0xa800] {
            if !(0..0x20000).contains(&(base + i64::from(position))) {
                self.error = true;
            }
        }
        0
    }
    fn write(&mut self, position: i32, _value: u8, _back_reference: bool) {
        self.read(position);
        self.produced += 1;
        if position < 0 || self.produced > 0x15800 {
            self.error = true;
        }
    }
}
fn validate(source: &[u8], address: Option<u32>) -> Result<Vec<u8>, PreparedTableError> {
    let mut output = Validation {
        source,
        address,
        encoded: Vec::new(),
        produced: 0,
        error: false,
    };
    decode(0, &mut output);
    // An error's synthetic ff only completes the bounded current command; never accept it.
    if output.error {
        return Err(PreparedTableError::Malformed);
    }
    Ok(output.encoded)
}
// Four domain-separated FNV-1a64 lanes, fixed u64LE scalar/count framing, not cryptographic.
struct Fingerprint([u64; 4]);
impl Fingerprint {
    fn new() -> Self {
        Self(core::array::from_fn(|lane| {
            0xcbf29ce484222325 ^ lane as u64
        }))
    }
    fn bytes(&mut self, bytes: &[u8]) {
        for &byte in bytes {
            for lane in &mut self.0 {
                *lane = (*lane ^ u64::from(byte)).wrapping_mul(0x100000001b3);
            }
        }
    }
    fn number(&mut self, number: u64) {
        self.bytes(&number.to_le_bytes());
    }
    fn finish(self) -> [u8; 32] {
        let mut result = [0; 32];
        for (chunk, lane) in result.chunks_exact_mut(8).zip(self.0) {
            chunk.copy_from_slice(&lane.to_le_bytes());
        }
        result
    }
}
fn fingerprint(bytes: &[u8]) -> [u8; 32] {
    let mut hash = Fingerprint::new();
    hash.bytes(bytes);
    hash.finish()
}
fn compatibility(world: &WorldDefinition, keys: &CompatibilityKeys) -> [u8; 32] {
    let mut h = Fingerprint::new();
    h.bytes(b"SMTT-world-reference-lorom-ram-v1");
    macro_rules! n { ($($value:expr),* $(,)?) => {{ $(h.number($value as u64);)* }}; }
    macro_rules! option {
        ($value:expr) => {
            match $value {
                None => n!(0),
                Some(id) => n!(1, id.0),
            }
        };
    }
    n!(world.rooms.len());
    for (id, r) in &world.rooms {
        n!(
            id.0,
            r.index,
            r.area,
            r.map_x,
            r.map_y,
            r.width,
            r.height,
            r.up_scroll,
            r.down_scroll,
            r.cre,
            r.doors.0,
            r.selectors.len()
        );
        for s in &r.selectors {
            match s.condition {
                StateCondition::Default => n!(0),
                StateCondition::Event(v) => n!(1, v),
                StateCondition::Boss(v) => n!(2, v),
                StateCondition::BossOne => n!(3),
                StateCondition::MorphAndMissiles => n!(4),
                StateCondition::PowerBombs => n!(5),
            };
            n!(s.state.0);
        }
    }
    n!(world.states.len());
    for (id, s) in &world.states {
        n!(id.0, s.tileset.0, s.level.0, s.music_data, s.music_track);
        option!(s.fx);
        n!(s.enemy_population.0, s.enemy_tilesets.0);
        h.bytes(&s.layer2_policy);
        option!(s.main_code);
        match &s.scrolls {
            ScrollDefinition::Generated(v) => n!(0, *v),
            ScrollDefinition::Cells(v) => {
                n!(1, v.len());
                h.bytes(v);
            }
        }
        n!(s.objects.len());
        for o in &s.objects {
            n!(o.x, o.y, o.definition.0, o.argument);
        }
        n!(s.background.len());
        for b in &s.background {
            use BackgroundCommand::*;
            let target = |t: &BackgroundTarget| match t {
                BackgroundTarget::Main => 0,
                BackgroundTarget::Secondary => 1,
                BackgroundTarget::Fx => 2,
            };
            match b {
                Transfer {
                    source,
                    destination,
                    bytes,
                } => n!(0, source.0, target(destination), *bytes),
                Decompress {
                    source,
                    destination,
                } => n!(1, source.0, target(destination)),
                ClearFx => n!(2),
                ClearBackground => n!(3),
                ClearDualBackground => n!(4),
                TransferAndSetCopyPolicy {
                    source,
                    destination,
                    bytes,
                } => n!(5, source.0, target(destination), *bytes),
                MatchExitTransfer {
                    exit,
                    source,
                    destination,
                    bytes,
                } => n!(6, exit.0, source.0, target(destination), *bytes),
            }
        }
        option!(s.setup);
    }
    n!(world.exits.len());
    for (id, e) in &world.exits {
        let a = e.arrival;
        n!(
            id.0,
            e.destination.0,
            a.flags,
            a.direction,
            a.x,
            a.y,
            a.close_x,
            a.close_y,
            a.distance,
            e.skip_cre_upload
        );
        option!(e.setup);
    }
    n!(world.starts.len());
    for (id, s) in &world.starts {
        n!(
            id.0,
            s.room.0,
            s.exit.0,
            s.camera_x,
            s.camera_y,
            s.samus_x_offset,
            s.samus_y_offset
        );
    }
    n!(world.door_lists.len());
    for (id, v) in &world.door_lists {
        n!(id.0, v.len());
        for id in v {
            n!(id.0);
        }
    }
    n!(world.tilesets.len());
    for (id, t) in &world.tilesets {
        n!(id.0, t.tile_table.0, t.tiles.0, t.palette.0);
    }
    n!(world.resources.len());
    for (id, k) in &world.resources {
        n!(
            id.0,
            match k {
                ResourceKind::Level => 0,
                ResourceKind::TileTable => 1,
                ResourceKind::Tiles => 2,
                ResourceKind::Palette => 3,
                ResourceKind::Fx => 4,
                ResourceKind::EnemyPopulation => 5,
                ResourceKind::EnemyTilesets => 6,
                ResourceKind::ActorSprites => 7,
                ResourceKind::ActorTilemap => 8,
                ResourceKind::RoomObject => 9,
                ResourceKind::DoorSetup => 10,
                ResourceKind::RoomSetup => 11,
                ResourceKind::RoomMain => 12,
                ResourceKind::Background => 13,
            }
        );
    }
    n!(world.closing_doors.len());
    for (direction, id) in &world.closing_doors {
        n!(*direction);
        option!(*id);
    }
    n!(keys.starts.len());
    for (id, s) in &keys.starts {
        n!(id.0, s.area, s.index);
    }
    macro_rules! map {
        ($map:expr) => {
            n!($map.len());
            for (id, key) in &$map {
                n!(id.0, *key);
            }
        };
    }
    map!(keys.rooms);
    map!(keys.states);
    map!(keys.exits);
    map!(keys.door_lists);
    n!(keys.resources.len());
    for (id, key) in &keys.resources {
        n!(id.0);
        match key {
            ResourceKey::Word { bank, offset } => n!(0, *bank, *offset),
            ResourceKey::Long(LegacyAddress24(bytes)) => {
                n!(1);
                h.bytes(bytes);
            }
        }
    }
    h.finish()
}

// Output implementations retain their byte/write protocol. VRAM also needs
// the back-reference flag to preserve its address-register writes.
pub(in crate::game) trait Output {
    fn next_byte(&mut self) -> u8;
    fn read(&mut self, position: i32) -> u8;
    fn write(&mut self, position: i32, value: u8, back_reference: bool);
}

pub(in crate::game) fn decode(start: i32, output: &mut impl Output) {
    let mut destination = start;
    loop {
        let header = output.next_byte();
        if header == 0xff {
            return;
        }
        let (command, length) = if header & 0xe0 == 0xe0 {
            (
                (header << 3) & 0xe0,
                (i32::from(header & 3) << 8 | i32::from(output.next_byte())) + 1,
            )
        } else {
            (header & 0xe0, i32::from(header & 0x1f) + 1)
        };
        // Read each command's operands before its first output byte. Absolute
        // references are relative to the initial destination, even in VRAM.
        let (mut source, first, second) = match command {
            0x20 | 0x60 => (0, output.next_byte(), 0),
            0x40 => (0, output.next_byte(), output.next_byte()),
            0x80 | 0xa0 => {
                let low = i32::from(output.next_byte());
                (low + i32::from(output.next_byte()) * 256 + start, 0, 0)
            }
            0xc0 | 0xe0 => (destination - i32::from(output.next_byte()), 0, 0),
            _ => (0, 0, 0),
        };
        for index in 0..length {
            let value = match command {
                0x20 => first,
                0x40 => {
                    if index & 1 == 0 {
                        first
                    } else {
                        second
                    }
                }
                0x60 => first.wrapping_add(index as u8),
                0x80 | 0xa0 | 0xc0 | 0xe0 => {
                    let value = output.read(source) ^ if command & 0x20 != 0 { 0xff } else { 0 };
                    source += 1;
                    value
                }
                _ => output.next_byte(),
            };
            // Read and write each reference byte separately: overlapping copies
            // propagate earlier writes instead of copying a frozen source range.
            output.write(destination, value, command & 0x80 != 0);
            destination += 1;
        }
    }
}
