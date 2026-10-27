//! Portable room-pack bytes and application selection, before Game construction.
#![forbid(unsafe_code)]
use crate::game::room_entry::{
    PreparedRoomAssets,
    world::{
        ArrivalDefinition, BackgroundCommand, BackgroundTarget, CompatibilityKeys, DoorListId,
        ExitDefinition, ExitId, LegacyAddress24, LegacyStartSite, ResourceId, ResourceKey,
        ResourceKind, RoomDefinition, RoomId, RoomObject, ScrollDefinition, StartDefinition,
        StartId, StateCondition, StateDefinition, StateId, StateSelector, TilesetDefinition,
        TilesetId, WorldBinding, WorldDefinition,
    },
};
use std::{
    collections::BTreeMap,
    fmt,
    fs::File,
    io::Read,
    path::{Path, PathBuf},
    sync::Arc,
};

pub const MAX_PACK_BYTES: usize = 64 * 1024 * 1024;
pub const MAX_ROM_SOURCE_BYTES: usize = 0x400200;
const MAX_METADATA_BYTES: usize = 4 * 1024 * 1024;
const MAX_ITEMS: usize = 65_536;
const MAX_TOTAL_ITEMS: usize = 262_144;
const HEADER_BYTES: usize = 52;
const VERSION: u16 = 1;
const VANILLA_PROFILE: u16 = 1;

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct RoomPackError(String);
impl fmt::Display for RoomPackError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(&self.0)
    }
}
impl std::error::Error for RoomPackError {}
type Result<T> = std::result::Result<T, RoomPackError>;
fn error(message: impl Into<String>) -> RoomPackError {
    RoomPackError(message.into())
}

/// Both native startup and the verifier consume this exact two-token selection.
/// Arguments exclude the executable name. Other arguments retain their order.
pub struct Selection {
    pub world: Option<Arc<WorldBinding>>,
    pub arguments: Vec<String>,
}
/// Select an optional room pack before native startup.
/// Windows desktop room-pack loading requires ASCII ROM path text; automatic
/// directory search for a relative ROM also requires an ASCII current directory.
/// Pack paths and verifier ROM paths retain Rust Unicode file handling.
pub fn select(arguments: &[String]) -> Result<Selection> {
    select_args(arguments, true)
}
/// Verifier argv starts with the ROM and does not enter native SwitchDirectory.
pub fn select_for_verifier(arguments: &[String]) -> Result<Selection> {
    select_args(arguments, false)
}
fn select_args(arguments: &[String], native: bool) -> Result<Selection> {
    let mut path = None;
    let mut retained = Vec::new();
    let mut position = 0;
    while position < arguments.len() {
        if arguments[position] == "--room-pack" {
            if path.is_some() {
                return Err(error("duplicate --room-pack selection"));
            }
            let value = arguments
                .get(position + 1)
                .filter(|value| !value.is_empty() && !value.starts_with('-'))
                .ok_or_else(|| error("--room-pack requires a path"))?;
            path = Some(value);
            position += 2;
        } else if arguments[position].starts_with("--room-pack=") {
            return Err(error("room pack selection requires --room-pack PATH"));
        } else {
            retained.push(arguments[position].clone());
            position += 1;
        }
    }
    let world = path.map(|path| load_file(Path::new(path))).transpose()?;
    if let Some(world) = &world {
        let path = selected_rom_path(&retained, native)?;
        let file = File::open(&path).map_err(|e| error(format!("{}: {e}", path.display())))?;
        let mut source = Vec::new();
        file.take(MAX_ROM_SOURCE_BYTES as u64 + 1)
            .read_to_end(&mut source)
            .map_err(|e| error(format!("{}: {e}", path.display())))?;
        let normalized = normalize_source(&source)?;
        let expected = world
            .prepared_room_source_size()
            .ok_or_else(|| error("selected room pack lacks a complete room catalog"))?;
        if expected != normalized.len() {
            return Err(error(format!(
                "room pack requires {expected} normalized ROM bytes; selected ROM has {}",
                normalized.len(),
            )));
        }
    }
    Ok(Selection {
        world,
        arguments: retained,
    })
}

/// Windows desktop pack ROM paths are ASCII to match native CRT path spelling.
/// An ASCII cwd is required only for relative ROMs using automatic directory search.
fn selected_rom_path(arguments: &[String], native: bool) -> Result<PathBuf> {
    let desktop = native && arguments.first().map(String::as_str) != Some("verify");
    let config = desktop && arguments.len() >= 2 && arguments[0] == "--config";
    let mut position = if config {
        2
    } else if native && !desktop {
        1
    } else {
        0
    };
    if desktop && arguments.get(position).map(String::as_str) == Some("--debug") {
        position += 1;
    }
    let filename = match arguments.get(position) {
        Some(filename) => filename.as_str(),
        None if desktop => "sm.smc",
        None => return Err(error("selected room pack requires a ROM path")),
    };
    #[cfg(windows)]
    if desktop && !filename.is_ascii() {
        return Err(error(
            "Windows desktop room-pack loading requires an ASCII ROM path",
        ));
    }
    let path = Path::new(filename);
    if !desktop || config || path.is_absolute() {
        return std::path::absolute(path).map_err(|e| error(format!("ROM path: {e}")));
    }
    let current = std::env::current_dir().map_err(|e| error(format!("ROM directory: {e}")))?;
    #[cfg(windows)]
    if !current.as_os_str().as_encoded_bytes().is_ascii() {
        return Err(error(
            "Windows desktop room-pack directory search requires an ASCII current directory",
        ));
    }
    #[cfg(windows)]
    let directory = {
        use std::os::windows::ffi::OsStrExt;
        current.as_os_str().encode_wide().collect::<Vec<_>>()
    };
    #[cfg(unix)]
    let directory = {
        use std::os::unix::ffi::OsStrExt;
        current.as_os_str().as_bytes().to_vec()
    };
    let mut position = directory.len();
    // Native getcwd uses4064 bytes; its three textual prefixes stop at position zero.
    if current.as_os_str().as_encoded_bytes().len() < 4064 {
        for _ in 0..3 {
            if position == 0 {
                break;
            }
            #[cfg(windows)]
            let prefix = {
                use std::os::windows::ffi::OsStringExt;
                std::ffi::OsString::from_wide(&directory[..position])
            };
            #[cfg(unix)]
            let prefix = {
                use std::os::unix::ffi::OsStringExt;
                std::ffi::OsString::from_vec(directory[..position].to_vec())
            };
            let mut config_path = prefix.clone();
            config_path.push("/sm.ini");
            if File::open(config_path).is_ok() {
                return Ok(Path::new(&prefix).join(path));
            }
            position -= 1;
            while position != 0 && directory[position] != 0x2f && directory[position] != 0x5c {
                position -= 1;
            }
        }
    }
    Ok(current.join(path))
}

/// Match native four-candidate header scoring before using its allocation normalizer.
/// Only the native-selected LoROM mapper is compatible with the vanilla catalog.
pub fn normalize_source(source: &[u8]) -> Result<Vec<u8>> {
    if source.len() < 0x8000 || source.len() > MAX_ROM_SOURCE_BYTES {
        return Err(error("room pack ROM source length is out of bounds"));
    }
    let candidates = [
        (0x8000, 0x7fc0),
        (0x8200, 0x81c0),
        (0x10000, 0xffc0),
        (0x10200, 0x101c0),
    ];
    let mut scores = [-50; 4];
    for (score, &(minimum, location)) in scores.iter_mut().zip(&candidates) {
        if source.len() >= minimum {
            *score = header_score(source, location);
        }
    }
    let (mut max, mut used) = (0, 0);
    for (index, score) in scores.into_iter().enumerate() {
        if score > max {
            max = score;
            used = index;
        }
    }
    let cart_type = if candidates[used].1 < 0x9000 { 1 } else { 2 };
    if cart_type != 1 {
        return Err(error(
            "room pack vanilla profile requires native-selected LoROM cartridge",
        ));
    }
    let header = if used & 1 != 0 { 0x200 } else { 0 };
    PreparedRoomAssets::normalize_source(&source[header..])
        .map_err(|e| error(format!("ROM allocation normalization: {e}")))
}

fn header_score(source: &[u8], location: usize) -> i32 {
    let field = |offset| source[location + offset];
    let (speed, kind) = (field(0x15) >> 4, field(0x15) & 0xf);
    let (coprocessor, chips) = (field(0x16) >> 4, field(0x16) & 0xf);
    let mut score = if matches!(speed, 2 | 3) { 5 } else { -4 };
    score += if kind <= 3 || kind == 5 { 5 } else { -2 };
    score += if coprocessor <= 5 || coprocessor >= 0xe {
        5
    } else {
        -2
    };
    score += if chips <= 6 || matches!(chips, 9 | 0xa) {
        5
    } else {
        -2
    };
    score += if field(0x19) <= 0x14 { 5 } else { -2 };
    let checksum = u16::from_le_bytes([field(0x1e), field(0x1f)]);
    let complement = u16::from_le_bytes([field(0x1c), field(0x1d)]);
    score += if u32::from(checksum) + u32::from(complement) == 0xffff {
        8
    } else {
        -6
    };
    let reset = u16::from_le_bytes([field(0x3c), field(0x3d)]);
    score += if reset >= 0x8000 { 8 } else { -20 };
    let opcode_at = location + 0x40 - 0x8000 + usize::from(reset & 0x7fff);
    let opcode = match source.get(opcode_at) {
        Some(&opcode) => opcode,
        None => {
            score -= 14;
            0xff
        }
    };
    if matches!(opcode, 0x78 | 0x18) {
        score += 6;
    }
    if matches!(opcode, 0x4c | 0x5c | 0x9c) {
        score += 3;
    }
    if matches!(opcode, 0 | 0xff | 0xdb) {
        score -= 6;
    }
    score
}

/// Read once at the caller's current directory, before the native host changes it.
pub fn load_file(path: &Path) -> Result<Arc<WorldBinding>> {
    let path = std::path::absolute(path).map_err(|e| error(format!("room pack path: {e}")))?;
    let file = File::open(&path).map_err(|e| error(format!("{}: {e}", path.display())))?;
    let mut bytes = Vec::new();
    file.take(MAX_PACK_BYTES as u64 + 1)
        .read_to_end(&mut bytes)
        .map_err(|e| error(format!("{}: {e}", path.display())))?;
    load(&bytes).map_err(|e| error(format!("{}: {e}", path.display())))
}

/// Import caller-owned immutable source bytes through the complete room catalog.
pub fn import(source: &[u8], world: &WorldDefinition, keys: &CompatibilityKeys) -> Result<Vec<u8>> {
    let metadata = metadata(world, keys)?;
    let normalized = normalize_source(source)?;
    let assets = PreparedRoomAssets::import(&normalized, world, keys)
        .map_err(|e| error(format!("room assets: {e}")))?;
    encode(&metadata, &assets.to_bytes())
}

/// Write a generic supplied world/catalog; no fixture or ROM allocation is retained.
pub fn to_bytes(
    world: &WorldDefinition,
    keys: &CompatibilityKeys,
    assets: &PreparedRoomAssets,
) -> Result<Vec<u8>> {
    let metadata = metadata(world, keys)?;
    let catalog = assets.to_bytes();
    PreparedRoomAssets::load(&catalog, world, keys)
        .map_err(|e| error(format!("room assets: {e}")))?;
    encode(&metadata, &catalog)
}

pub fn load(bytes: &[u8]) -> Result<Arc<WorldBinding>> {
    if bytes.len() < HEADER_BYTES || bytes.len() > MAX_PACK_BYTES {
        return Err(error("room pack length is out of bounds"));
    }
    let mut header = Reader::new(&bytes[..20]);
    if header.take(4)? != b"SMRP" || header.u16()? != VERSION {
        return Err(error("unsupported room pack schema"));
    }
    if header.u16()? != VANILLA_PROFILE || header.u32()? != 0 {
        return Err(error("unsupported room pack profile/options"));
    }
    let metadata_length = header.u32()? as usize;
    let catalog_length = header.u32()? as usize;
    let catalog_start = HEADER_BYTES
        .checked_add(metadata_length)
        .ok_or_else(|| error("room pack length overflow"))?;
    if metadata_length > MAX_METADATA_BYTES
        || catalog_start.checked_add(catalog_length) != Some(bytes.len())
    {
        return Err(error(
            "room pack sections are missing, trailing or out of bounds",
        ));
    }
    if bytes[20..HEADER_BYTES] != content_fingerprint(&bytes[..20], &bytes[HEADER_BYTES..]) {
        return Err(error("room pack content fingerprint differs"));
    }
    let mut reader = Reader::new(&bytes[HEADER_BYTES..catalog_start]);
    let world = reader.world()?;
    let keys = reader.keys()?;
    if reader.position != reader.bytes.len() {
        return Err(error("trailing world/key records"));
    }
    validate(&world, &keys)?;
    let assets = PreparedRoomAssets::load(&bytes[catalog_start..], &world, &keys)
        .map_err(|e| error(format!("room assets: {e}")))?;
    WorldBinding::prepared_with_room_assets(world, keys, assets)
        .map_err(|e| error(format!("room binding: {e}")))
}

fn validate(world: &WorldDefinition, keys: &CompatibilityKeys) -> Result<()> {
    for site in [
        LegacyStartSite { area: 6, index: 0 },
        LegacyStartSite { area: 0, index: 18 },
    ] {
        if !keys.starts.values().any(|value| *value == site) {
            return Err(error(format!(
                "room pack lacks CurrentLoadStation site {}/{}",
                site.area, site.index
            )));
        }
    }
    WorldBinding::prepared(world.clone(), keys.clone())
        .map_err(|e| error(format!("world/key definitions: {e:?}")))?;
    Ok(())
}
fn metadata(world: &WorldDefinition, keys: &CompatibilityKeys) -> Result<Vec<u8>> {
    let mut writer = Writer::new();
    writer.world(world)?;
    writer.keys(keys)?;
    validate(world, keys)?;
    Ok(writer.bytes)
}
fn encode(metadata: &[u8], catalog: &[u8]) -> Result<Vec<u8>> {
    let length = HEADER_BYTES
        .checked_add(metadata.len())
        .and_then(|n| n.checked_add(catalog.len()))
        .filter(|n| *n <= MAX_PACK_BYTES)
        .ok_or_else(|| error("room pack exceeds its byte limit"))?;
    let mut bytes = Vec::with_capacity(length);
    bytes.extend_from_slice(b"SMRP");
    bytes.extend_from_slice(&VERSION.to_le_bytes());
    bytes.extend_from_slice(&VANILLA_PROFILE.to_le_bytes());
    bytes.extend_from_slice(&0u32.to_le_bytes());
    bytes.extend_from_slice(&(metadata.len() as u32).to_le_bytes());
    bytes.extend_from_slice(&(catalog.len() as u32).to_le_bytes());
    bytes.extend_from_slice(&[0; 32]);
    bytes.extend_from_slice(metadata);
    bytes.extend_from_slice(catalog);
    let fingerprint = content_fingerprint(&bytes[..20], &bytes[HEADER_BYTES..]);
    bytes[20..HEADER_BYTES].copy_from_slice(&fingerprint);
    Ok(bytes)
}

/// Stable SMRP identity, including schema/profile/lengths and exact world/catalog data.
/// The checksum slot itself is excluded. This follows the catalog's fixed FNV lanes.
pub fn content_fingerprint(header: &[u8], payload: &[u8]) -> [u8; 32] {
    let mut lanes = core::array::from_fn::<_, 4, _>(|lane| 0xcbf29ce484222325u64 ^ lane as u64);
    for part in [b"SMRP-world-catalog-vanilla-v1".as_slice(), header, payload] {
        for &byte in part {
            for lane in &mut lanes {
                *lane = (*lane ^ u64::from(byte)).wrapping_mul(0x100000001b3);
            }
        }
    }
    let mut result = [0; 32];
    for (bytes, lane) in result.chunks_exact_mut(8).zip(lanes) {
        bytes.copy_from_slice(&lane.to_le_bytes());
    }
    result
}

struct Reader<'a> {
    bytes: &'a [u8],
    position: usize,
    items: usize,
}
impl<'a> Reader<'a> {
    fn new(bytes: &'a [u8]) -> Self {
        Self {
            bytes,
            position: 0,
            items: 0,
        }
    }
    fn take(&mut self, length: usize) -> Result<&'a [u8]> {
        let end = self
            .position
            .checked_add(length)
            .filter(|end| *end <= self.bytes.len())
            .ok_or_else(|| error("truncated world/key record"))?;
        let bytes = &self.bytes[self.position..end];
        self.position = end;
        Ok(bytes)
    }
    fn u8(&mut self) -> Result<u8> {
        Ok(self.take(1)?[0])
    }
    fn u16(&mut self) -> Result<u16> {
        let bytes = self.take(2)?;
        Ok(u16::from_le_bytes([bytes[0], bytes[1]]))
    }
    fn u32(&mut self) -> Result<u32> {
        let bytes = self.take(4)?;
        Ok(u32::from_le_bytes([bytes[0], bytes[1], bytes[2], bytes[3]]))
    }
    fn count(&mut self) -> Result<usize> {
        let count = self.u32()? as usize;
        self.items = self
            .items
            .checked_add(count)
            .filter(|n| *n <= MAX_TOTAL_ITEMS)
            .ok_or_else(|| error("world/key work limit exceeded"))?;
        if count > MAX_ITEMS || count > self.bytes.len() - self.position {
            return Err(error("world/key count is out of bounds"));
        }
        Ok(count)
    }
    fn list<T>(&mut self, mut read: impl FnMut(&mut Self) -> Result<T>) -> Result<Vec<T>> {
        let count = self.count()?;
        let mut values = Vec::with_capacity(count);
        for _ in 0..count {
            values.push(read(self)?);
        }
        Ok(values)
    }
    fn map<K: Copy + Ord, V>(
        &mut self,
        mut key: impl FnMut(&mut Self) -> Result<K>,
        mut value: impl FnMut(&mut Self) -> Result<V>,
    ) -> Result<BTreeMap<K, V>> {
        let count = self.count()?;
        let mut values = BTreeMap::new();
        let mut previous = None;
        for _ in 0..count {
            let id = key(self)?;
            if previous.is_some_and(|last| last >= id) {
                return Err(error("duplicate or noncanonical world/key ID"));
            }
            previous = Some(id);
            values.insert(id, value(self)?);
        }
        Ok(values)
    }
    fn option(&mut self) -> Result<Option<ResourceId>> {
        match self.u8()? {
            0 => Ok(None),
            1 => Ok(Some(ResourceId(self.u32()?))),
            _ => Err(error("unknown optional resource tag")),
        }
    }
    fn world(&mut self) -> Result<WorldDefinition> {
        Ok(WorldDefinition {
            rooms: self.map(|r| Ok(RoomId(r.u32()?)), Self::room)?,
            states: self.map(|r| Ok(StateId(r.u32()?)), Self::state)?,
            exits: self.map(|r| Ok(ExitId(r.u32()?)), Self::exit)?,
            starts: self.map(|r| Ok(StartId(r.u32()?)), Self::start)?,
            door_lists: self.map(
                |r| Ok(DoorListId(r.u32()?)),
                |r| r.list(|r| Ok(ExitId(r.u32()?))),
            )?,
            tilesets: self.map(
                |r| Ok(TilesetId(r.u32()?)),
                |r| {
                    Ok(TilesetDefinition {
                        tile_table: ResourceId(r.u32()?),
                        tiles: ResourceId(r.u32()?),
                        palette: ResourceId(r.u32()?),
                    })
                },
            )?,
            resources: self.map(|r| Ok(ResourceId(r.u32()?)), Self::kind)?,
            closing_doors: self.map(Self::u16, Self::option)?,
        })
    }
    fn room(&mut self) -> Result<RoomDefinition> {
        let fields = self.take(9)?;
        Ok(RoomDefinition {
            index: fields[0],
            area: fields[1],
            map_x: fields[2],
            map_y: fields[3],
            width: fields[4],
            height: fields[5],
            up_scroll: fields[6],
            down_scroll: fields[7],
            cre: fields[8],
            doors: DoorListId(self.u32()?),
            selectors: self.list(|r| {
                let condition = match r.u8()? {
                    0 => StateCondition::Default,
                    1 => StateCondition::Event(r.u8()?),
                    2 => StateCondition::Boss(r.u8()?),
                    3 => StateCondition::BossOne,
                    4 => StateCondition::MorphAndMissiles,
                    5 => StateCondition::PowerBombs,
                    _ => return Err(error("unknown room selector tag")),
                };
                Ok(StateSelector {
                    condition,
                    state: StateId(r.u32()?),
                })
            })?,
        })
    }
    fn state(&mut self) -> Result<StateDefinition> {
        let tileset = TilesetId(self.u32()?);
        let level = ResourceId(self.u32()?);
        let music_data = self.u8()?;
        let music_track = self.u8()?;
        let fx = self.option()?;
        let enemy_population = ResourceId(self.u32()?);
        let enemy_tilesets = ResourceId(self.u32()?);
        let layer2_policy = [self.u8()?, self.u8()?];
        let main_code = self.option()?;
        let scrolls = match self.u8()? {
            0 => ScrollDefinition::Generated(self.u8()?),
            1 => ScrollDefinition::Cells(self.list(Self::u8)?),
            _ => return Err(error("unknown scroll definition tag")),
        };
        let objects = self.list(|r| {
            Ok(RoomObject {
                x: r.u8()?,
                y: r.u8()?,
                definition: ResourceId(r.u32()?),
                argument: r.u16()?,
            })
        })?;
        let background = self.list(Self::background)?;
        Ok(StateDefinition {
            tileset,
            level,
            music_data,
            music_track,
            fx,
            enemy_population,
            enemy_tilesets,
            layer2_policy,
            main_code,
            scrolls,
            objects,
            background,
            setup: self.option()?,
        })
    }
    fn target(&mut self) -> Result<BackgroundTarget> {
        match self.u8()? {
            0 => Ok(BackgroundTarget::Main),
            1 => Ok(BackgroundTarget::Secondary),
            2 => Ok(BackgroundTarget::Fx),
            _ => Err(error("unknown background target tag")),
        }
    }
    fn background(&mut self) -> Result<BackgroundCommand> {
        use BackgroundCommand::*;
        Ok(match self.u8()? {
            0 => Transfer {
                source: ResourceId(self.u32()?),
                destination: self.target()?,
                bytes: self.u16()?,
            },
            1 => Decompress {
                source: ResourceId(self.u32()?),
                destination: self.target()?,
            },
            2 => ClearFx,
            3 => ClearBackground,
            4 => ClearDualBackground,
            5 => TransferAndSetCopyPolicy {
                source: ResourceId(self.u32()?),
                destination: self.target()?,
                bytes: self.u16()?,
            },
            6 => MatchExitTransfer {
                exit: ExitId(self.u32()?),
                source: ResourceId(self.u32()?),
                destination: self.target()?,
                bytes: self.u16()?,
            },
            _ => return Err(error("unknown background command tag")),
        })
    }
    fn exit(&mut self) -> Result<ExitDefinition> {
        let destination = RoomId(self.u32()?);
        let fields = self.take(6)?;
        let distance = self.u16()? as i16;
        let setup = self.option()?;
        let skip_cre_upload = match self.u8()? {
            0 => false,
            1 => true,
            _ => return Err(error("invalid CRE upload boolean")),
        };
        Ok(ExitDefinition {
            destination,
            arrival: ArrivalDefinition {
                flags: fields[0],
                direction: fields[1],
                x: fields[2],
                y: fields[3],
                close_x: fields[4],
                close_y: fields[5],
                distance,
            },
            setup,
            skip_cre_upload,
        })
    }
    fn start(&mut self) -> Result<StartDefinition> {
        Ok(StartDefinition {
            room: RoomId(self.u32()?),
            exit: ExitId(self.u32()?),
            camera_x: self.u16()?,
            camera_y: self.u16()?,
            samus_x_offset: self.u16()?,
            samus_y_offset: self.u16()?,
        })
    }
    fn kind(&mut self) -> Result<ResourceKind> {
        use ResourceKind::*;
        match self.u8()? {
            0 => Ok(Level),
            1 => Ok(TileTable),
            2 => Ok(Tiles),
            3 => Ok(Palette),
            4 => Ok(Fx),
            5 => Ok(EnemyPopulation),
            6 => Ok(EnemyTilesets),
            7 => Ok(ActorSprites),
            8 => Ok(ActorTilemap),
            9 => Ok(RoomObject),
            10 => Ok(DoorSetup),
            11 => Ok(RoomSetup),
            12 => Ok(RoomMain),
            13 => Ok(Background),
            _ => Err(error("unknown resource kind tag")),
        }
    }
    fn keys(&mut self) -> Result<CompatibilityKeys> {
        Ok(CompatibilityKeys {
            starts: self.map(
                |r| Ok(StartId(r.u32()?)),
                |r| {
                    Ok(LegacyStartSite {
                        area: r.u16()?,
                        index: r.u16()?,
                    })
                },
            )?,
            rooms: self.map(|r| Ok(RoomId(r.u32()?)), Self::u16)?,
            states: self.map(|r| Ok(StateId(r.u32()?)), Self::u16)?,
            exits: self.map(|r| Ok(ExitId(r.u32()?)), Self::u16)?,
            door_lists: self.map(|r| Ok(DoorListId(r.u32()?)), Self::u16)?,
            resources: self.map(
                |r| Ok(ResourceId(r.u32()?)),
                |r| match r.u8()? {
                    0 => Ok(ResourceKey::Word {
                        bank: r.u8()?,
                        offset: r.u16()?,
                    }),
                    1 => Ok(ResourceKey::Long(LegacyAddress24([
                        r.u8()?,
                        r.u8()?,
                        r.u8()?,
                    ]))),
                    _ => Err(error("unknown compatibility resource tag")),
                },
            )?,
        })
    }
}

struct Writer {
    bytes: Vec<u8>,
    items: usize,
}
impl Writer {
    fn new() -> Self {
        Self {
            bytes: Vec::new(),
            items: 0,
        }
    }
    fn bytes(&mut self, bytes: &[u8]) -> Result<()> {
        if self
            .bytes
            .len()
            .checked_add(bytes.len())
            .is_none_or(|n| n > MAX_METADATA_BYTES)
        {
            return Err(error("world/key byte limit exceeded"));
        }
        self.bytes.extend_from_slice(bytes);
        Ok(())
    }
    fn u8(&mut self, value: u8) -> Result<()> {
        self.bytes(&[value])
    }
    fn u16(&mut self, value: u16) -> Result<()> {
        self.bytes(&value.to_le_bytes())
    }
    fn u32(&mut self, value: u32) -> Result<()> {
        self.bytes(&value.to_le_bytes())
    }
    fn count(&mut self, count: usize) -> Result<()> {
        self.items = self
            .items
            .checked_add(count)
            .filter(|n| *n <= MAX_TOTAL_ITEMS)
            .ok_or_else(|| error("world/key work limit exceeded"))?;
        if count > MAX_ITEMS {
            return Err(error("world/key count is out of bounds"));
        }
        self.u32(count as u32)
    }
    fn list<T>(
        &mut self,
        values: &[T],
        mut write: impl FnMut(&mut Self, &T) -> Result<()>,
    ) -> Result<()> {
        self.count(values.len())?;
        for value in values {
            write(self, value)?;
        }
        Ok(())
    }
    fn map<K, V>(
        &mut self,
        values: &BTreeMap<K, V>,
        mut write: impl FnMut(&mut Self, &K, &V) -> Result<()>,
    ) -> Result<()> {
        self.count(values.len())?;
        for (key, value) in values {
            write(self, key, value)?;
        }
        Ok(())
    }
    fn option(&mut self, value: Option<ResourceId>) -> Result<()> {
        match value {
            None => self.u8(0),
            Some(id) => {
                self.u8(1)?;
                self.u32(id.0)
            }
        }
    }
    fn world(&mut self, world: &WorldDefinition) -> Result<()> {
        self.map(&world.rooms, |w, id, r| {
            w.u32(id.0)?;
            w.bytes(&[
                r.index,
                r.area,
                r.map_x,
                r.map_y,
                r.width,
                r.height,
                r.up_scroll,
                r.down_scroll,
                r.cre,
            ])?;
            w.u32(r.doors.0)?;
            w.list(&r.selectors, |w, s| {
                use StateCondition::*;
                match s.condition {
                    Default => w.u8(0)?,
                    Event(n) => {
                        w.u8(1)?;
                        w.u8(n)?;
                    }
                    Boss(n) => {
                        w.u8(2)?;
                        w.u8(n)?;
                    }
                    BossOne => w.u8(3)?,
                    MorphAndMissiles => w.u8(4)?,
                    PowerBombs => w.u8(5)?,
                }
                w.u32(s.state.0)
            })
        })?;
        self.map(&world.states, |w, id, s| {
            w.u32(id.0)?;
            w.u32(s.tileset.0)?;
            w.u32(s.level.0)?;
            w.bytes(&[s.music_data, s.music_track])?;
            w.option(s.fx)?;
            w.u32(s.enemy_population.0)?;
            w.u32(s.enemy_tilesets.0)?;
            w.bytes(&s.layer2_policy)?;
            w.option(s.main_code)?;
            match &s.scrolls {
                ScrollDefinition::Generated(n) => {
                    w.u8(0)?;
                    w.u8(*n)?;
                }
                ScrollDefinition::Cells(v) => {
                    w.u8(1)?;
                    w.list(v, |w, n| w.u8(*n))?;
                }
            }
            w.list(&s.objects, |w, o| {
                w.bytes(&[o.x, o.y])?;
                w.u32(o.definition.0)?;
                w.u16(o.argument)
            })?;
            w.list(&s.background, Self::background)?;
            w.option(s.setup)
        })?;
        self.map(&world.exits, |w, id, e| {
            let a = e.arrival;
            w.u32(id.0)?;
            w.u32(e.destination.0)?;
            w.bytes(&[a.flags, a.direction, a.x, a.y, a.close_x, a.close_y])?;
            w.u16(a.distance as u16)?;
            w.option(e.setup)?;
            w.u8(u8::from(e.skip_cre_upload))
        })?;
        self.map(&world.starts, |w, id, s| {
            w.u32(id.0)?;
            w.u32(s.room.0)?;
            w.u32(s.exit.0)?;
            w.u16(s.camera_x)?;
            w.u16(s.camera_y)?;
            w.u16(s.samus_x_offset)?;
            w.u16(s.samus_y_offset)
        })?;
        self.map(&world.door_lists, |w, id, v| {
            w.u32(id.0)?;
            w.list(v, |w, id| w.u32(id.0))
        })?;
        self.map(&world.tilesets, |w, id, t| {
            w.u32(id.0)?;
            w.u32(t.tile_table.0)?;
            w.u32(t.tiles.0)?;
            w.u32(t.palette.0)
        })?;
        self.map(&world.resources, |w, id, kind| {
            use ResourceKind::*;
            w.u32(id.0)?;
            w.u8(match kind {
                Level => 0,
                TileTable => 1,
                Tiles => 2,
                Palette => 3,
                Fx => 4,
                EnemyPopulation => 5,
                EnemyTilesets => 6,
                ActorSprites => 7,
                ActorTilemap => 8,
                RoomObject => 9,
                DoorSetup => 10,
                RoomSetup => 11,
                RoomMain => 12,
                Background => 13,
            })
        })?;
        self.map(&world.closing_doors, |w, direction, id| {
            w.u16(*direction)?;
            w.option(*id)
        })
    }
    fn target(&mut self, target: BackgroundTarget) -> Result<()> {
        self.u8(match target {
            BackgroundTarget::Main => 0,
            BackgroundTarget::Secondary => 1,
            BackgroundTarget::Fx => 2,
        })
    }
    fn background(&mut self, command: &BackgroundCommand) -> Result<()> {
        use BackgroundCommand::*;
        match *command {
            Transfer {
                source,
                destination,
                bytes,
            } => {
                self.u8(0)?;
                self.u32(source.0)?;
                self.target(destination)?;
                self.u16(bytes)
            }
            Decompress {
                source,
                destination,
            } => {
                self.u8(1)?;
                self.u32(source.0)?;
                self.target(destination)
            }
            ClearFx => self.u8(2),
            ClearBackground => self.u8(3),
            ClearDualBackground => self.u8(4),
            TransferAndSetCopyPolicy {
                source,
                destination,
                bytes,
            } => {
                self.u8(5)?;
                self.u32(source.0)?;
                self.target(destination)?;
                self.u16(bytes)
            }
            MatchExitTransfer {
                exit,
                source,
                destination,
                bytes,
            } => {
                self.u8(6)?;
                self.u32(exit.0)?;
                self.u32(source.0)?;
                self.target(destination)?;
                self.u16(bytes)
            }
        }
    }
    fn keys(&mut self, keys: &CompatibilityKeys) -> Result<()> {
        self.map(&keys.starts, |w, id, s| {
            w.u32(id.0)?;
            w.u16(s.area)?;
            w.u16(s.index)
        })?;
        self.map(&keys.rooms, |w, id, n| {
            w.u32(id.0)?;
            w.u16(*n)
        })?;
        self.map(&keys.states, |w, id, n| {
            w.u32(id.0)?;
            w.u16(*n)
        })?;
        self.map(&keys.exits, |w, id, n| {
            w.u32(id.0)?;
            w.u16(*n)
        })?;
        self.map(&keys.door_lists, |w, id, n| {
            w.u32(id.0)?;
            w.u16(*n)
        })?;
        self.map(&keys.resources, |w, id, key| {
            w.u32(id.0)?;
            match *key {
                ResourceKey::Word { bank, offset } => {
                    w.u8(0)?;
                    w.u8(bank)?;
                    w.u16(offset)
                }
                ResourceKey::Long(LegacyAddress24(bytes)) => {
                    w.u8(1)?;
                    w.bytes(&bytes)
                }
            }
        })
    }
}
