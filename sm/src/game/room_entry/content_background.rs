//! Pure retained background reads and compatibility resource resolution.
#![forbid(unsafe_code)]
use super::{
    content::*,
    effects::{StagingSource, VramSlot},
    world::{BackgroundCommand, BackgroundTarget, ExitId, Identity, ResourceKind, WorldBinding},
};
fn command<'a>(world: &'a WorldBinding, position: &BackgroundPosition) -> &'a BackgroundCommand {
    &world.definition().expect("prepared background").states[&position.state].background
        [position.index]
}
fn transfer(command: &BackgroundCommand) -> (super::world::ResourceId, BackgroundTarget, u16) {
    match *command {
        BackgroundCommand::Transfer {
            source,
            destination,
            bytes,
        }
        | BackgroundCommand::TransferAndSetCopyPolicy {
            source,
            destination,
            bytes,
        }
        | BackgroundCommand::MatchExitTransfer {
            source,
            destination,
            bytes,
            exit: _,
        } => (source, destination, bytes),
        BackgroundCommand::Decompress {
            source,
            destination,
        } => (source, destination, 0),
        _ => panic!("background command has no transfer operands"),
    }
}
impl<P> Capture<P> {
    // Background records validate the complete read before observing any byte.
    // Ordinary records intentionally retain their separate access policy.
    pub(super) fn background_bytes<const N: usize>(&self, offset: usize) -> [u8; N] {
        let Capture::Legacy { image, base, .. } = self else {
            panic!("legacy background capture required");
        };
        let start = base + offset;
        assert!(
            start + N <= image.len(),
            "ROM read outside loaded allocation"
        );
        std::array::from_fn(|i| image.read(start + i))
    }
    pub(super) fn background_word(&self, offset: usize) -> u16 {
        let Capture::Legacy { access, .. } = self else {
            panic!("legacy background capture required");
        };
        let bytes = self.background_bytes(offset);
        match access {
            RecordAccess::Native => u16::from_ne_bytes(bytes),
            RecordAccess::LittleEndian => u16::from_le_bytes(bytes),
        }
    }
}
impl BackgroundCapture {
    pub fn word(&self, field: BackgroundWord) -> u16 {
        match self {
            Capture::Legacy { .. } => self.background_word(match field {
                BackgroundWord::Opcode | BackgroundWord::SourceLow => 0,
                BackgroundWord::SourceBankWord => 2,
                BackgroundWord::ByteCount => 5,
            }),
            Capture::Prepared { world, position } => {
                let command = command(world, position);
                if matches!(field, BackgroundWord::Opcode) {
                    assert_eq!(position.part, BackgroundPart::Opcode);
                    return match command {
                        BackgroundCommand::Transfer { .. } => 2,
                        BackgroundCommand::Decompress { .. } => 4,
                        BackgroundCommand::ClearFx => 6,
                        BackgroundCommand::TransferAndSetCopyPolicy { .. } => 8,
                        BackgroundCommand::ClearBackground => 10,
                        BackgroundCommand::ClearDualBackground => 12,
                        BackgroundCommand::MatchExitTransfer { .. } => 14,
                    };
                }
                let (source, destination, count) = transfer(command);
                match field {
                    BackgroundWord::SourceLow => {
                        world.resource_address(source, ResourceKind::Background) as u16
                    }
                    BackgroundWord::SourceBankWord => {
                        let source = world.resource_address(source, ResourceKind::Background);
                        let destination =
                            VramDestination::Prepared(destination).compatibility_word();
                        u16::from_ne_bytes([(source >> 16) as u8, destination as u8])
                    }
                    BackgroundWord::ByteCount => count,
                    BackgroundWord::Opcode => unreachable!(),
                }
            }
        }
    }
    pub fn source(&self) -> AssetSource {
        match self {
            Capture::Legacy { access, .. } => {
                let value = match access {
                    RecordAccess::Native => u32::from_ne_bytes(self.background_bytes(0)) & 0xffffff,
                    RecordAccess::LittleEndian => {
                        u32::from(self.background_word(0))
                            | (u32::from(self.background_bytes::<1>(2)[0]) << 16)
                    }
                };
                let [a, b, c, _] = value.to_le_bytes();
                AssetSource::Bound(AssetRef::Legacy([a, b, c]))
            }
            Capture::Prepared { world, position } => {
                AssetSource::Bound(AssetRef::Prepared(transfer(command(world, position)).0))
            }
        }
    }
    pub fn match_exit(&self) -> Identity<ExitId> {
        match self {
            Capture::Legacy { world, .. } => world.exit_identity(self.background_word(0)),
            Capture::Prepared { world, position } => match command(world, position) {
                BackgroundCommand::MatchExitTransfer { exit, .. } => Identity::Known(*exit),
                _ => panic!("background command has no exit match"),
            },
        }
    }
    pub fn destination(&self) -> BackgroundDestination {
        match self {
            Capture::Legacy { .. } => BackgroundDestination::Legacy(self.background_word(3)),
            Capture::Prepared { world, position } => {
                BackgroundDestination::Prepared(transfer(command(world, position)).1)
            }
        }
    }
    pub fn vram(&self) -> VramDestination {
        match self {
            Capture::Legacy { .. } => VramDestination::Legacy(self.background_word(3)),
            Capture::Prepared { world, position } => {
                VramDestination::Prepared(transfer(command(world, position)).1)
            }
        }
    }
}
impl ClosingHeaderCapture {
    pub fn word(&self) -> u16 {
        match self {
            Capture::Legacy { .. } => self.raw_word(0),
            Capture::Prepared { world, position } => world
                .definition()
                .expect("prepared closing door")
                .closing_doors
                .get(&position.direction)
                .copied()
                .flatten()
                .map_or(0, |id| {
                    world.resource_word(id, ResourceKind::RoomObject, 0x84)
                }),
        }
    }
}
impl BackgroundCursor {
    pub fn is_end(&self) -> bool {
        matches!(self, Self::Legacy(0))
    }
    pub fn advance(self, width: BackgroundWidth, world: &WorldBinding) -> Option<Self> {
        match self {
            Self::Legacy(cursor) => Some(Self::Legacy(cursor.wrapping_add(match width {
                BackgroundWidth::Opcode => 2,
                BackgroundWidth::Transfer => 7,
                BackgroundWidth::Decompress => 5,
                BackgroundWidth::MatchAndTransfer => 9,
            }))),
            Self::Prepared(mut position) => {
                let finish = match width {
                    BackgroundWidth::Opcode => match position.part {
                        BackgroundPart::Opcode => match command(world, &position) {
                            BackgroundCommand::ClearFx
                            | BackgroundCommand::ClearBackground
                            | BackgroundCommand::ClearDualBackground => true,
                            _ => {
                                position.part = BackgroundPart::Operand;
                                false
                            }
                        },
                        BackgroundPart::Operand => {
                            assert!(matches!(
                                command(world, &position),
                                BackgroundCommand::MatchExitTransfer { .. }
                            ));
                            position.part = BackgroundPart::MatchedTransfer;
                            false
                        }
                        BackgroundPart::MatchedTransfer => {
                            panic!("unexpected background opcode advance")
                        }
                    },
                    BackgroundWidth::Transfer
                    | BackgroundWidth::Decompress
                    | BackgroundWidth::MatchAndTransfer => true,
                };
                if finish {
                    position.index += 1;
                    position.part = BackgroundPart::Opcode;
                }
                (position.index
                    < world.definition().expect("prepared background").states[&position.state]
                        .background
                        .len())
                .then_some(Self::Prepared(position))
            }
        }
    }
}
impl AssetSource {
    pub(crate) fn compatibility_address(&self, world: &WorldBinding) -> u32 {
        match self {
            Self::Bound(asset) => {
                let [lo, hi, bank] = world.asset_bytes(asset);
                u32::from_le_bytes([lo, hi, bank, 0])
            }
            Self::Shared(asset) => match asset {
                SharedAsset::CreTiles => 0xb98000,
                SharedAsset::CreTileTable => 0xb9a09d,
                SharedAsset::BackgroundBlankLow | SharedAsset::BackgroundBlankHigh => 0x80a29a,
                SharedAsset::FxBlankLow => 0x80a2f7,
                SharedAsset::FxBlankHigh => 0x80a2f8,
                SharedAsset::TransitionBackground => 0x9ab200,
            },
            Self::Staging(source) => match source {
                StagingSource::TilesFirst => 0x7e2000,
                StagingSource::TilesSecond | StagingSource::Background => 0x7e4000,
                StagingSource::TilesThird => 0x7e6000,
                StagingSource::CreFirst => 0x7e7000,
                StagingSource::CreSecond => 0x7e8000,
            },
            Self::FxTilemap(offset) => 0x8a0000 | u32::from(*offset),
        }
    }
}
impl BackgroundDestination {
    pub(crate) fn compatibility_offset(&self) -> usize {
        match self {
            Self::Legacy(offset) => usize::from(*offset),
            Self::Prepared(target) => match target {
                BackgroundTarget::Main | BackgroundTarget::Fx => 0x4000,
                BackgroundTarget::Secondary => 0x5000,
            },
        }
    }
}
impl VramDestination {
    pub(crate) fn compatibility_word(&self) -> u16 {
        match self {
            Self::Legacy(word) => *word,
            Self::Slot(slot) => match slot {
                VramSlot::MainTiles => 0,
                VramSlot::TilesSecond => 0x1000,
                VramSlot::TilesThird => 0x2000,
                VramSlot::CreFirst => 0x2800,
                VramSlot::CreSecond => 0x3000,
                VramSlot::BackgroundFirst => 0x4000,
                VramSlot::BackgroundSecond => 0x4800,
                VramSlot::Fx => 0x5880,
                VramSlot::FxTilemap => 0x5be0,
            },
            Self::Prepared(target) => match target {
                BackgroundTarget::Main => 0x4000,
                BackgroundTarget::Secondary => 0x4800,
                BackgroundTarget::Fx => 0x5880,
            },
        }
    }
}
impl SetupAction {
    pub(crate) fn compatibility_code(&self, world: &WorldBinding) -> (SetupKind, u16) {
        match self {
            Self::Legacy { kind, code } => (*kind, *code),
            Self::Prepared(id) => {
                let resource = world.resource_kind(*id);
                let kind = match resource {
                    ResourceKind::DoorSetup => SetupKind::Door,
                    ResourceKind::RoomSetup => SetupKind::Room,
                    _ => panic!("resource is not a setup action"),
                };
                (kind, world.resource_word(*id, resource, 0x8f))
            }
        }
    }
}
pub(crate) enum SpawnObject {
    Legacy(u16),
    Prepared {
        x: u8,
        y: u8,
        definition: u16,
        argument: u16,
    },
}
impl ObjectCursor {
    pub(crate) fn spawn_object(&self, world: &WorldBinding) -> SpawnObject {
        match self {
            Self::LegacyInitial(address) => SpawnObject::Legacy(*address),
            Self::LegacyDoor(address) => SpawnObject::Legacy(*address as u16),
            Self::Prepared { state, index } => {
                let object = &world.definition().expect("prepared room object").states[state]
                    .objects[*index];
                SpawnObject::Prepared {
                    x: object.x,
                    y: object.y,
                    definition: world.resource_word(
                        object.definition,
                        ResourceKind::RoomObject,
                        0x84,
                    ),
                    argument: object.argument,
                }
            }
        }
    }
}
