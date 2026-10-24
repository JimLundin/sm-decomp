//! State and tileset reads occur at each call on the retained capture.
#![forbid(unsafe_code)]
use super::content::{
    AssetRef, Capture, LoadMode, ObjectCursor, ScrollCursor, ScrollDefinition, SetupAction,
    SetupKind, StateScalar, TilesetAsset, TilesetSelection,
};
use super::world::{ResourceKind, ScrollDefinition as AuthoredScroll, StateId, TilesetId};

impl Capture<StateId> {
    pub(crate) fn scalar(&self, field: StateScalar) -> u16 {
        match self {
            Self::Legacy { .. } => match field {
                StateScalar::MusicData => u16::from(self.raw_byte(4)),
                StateScalar::MusicTrack => u16::from(self.raw_byte(5)),
                StateScalar::Fx => self.raw_word(6),
                StateScalar::EnemyPopulation => self.raw_word(8),
                StateScalar::EnemyTilesets => self.raw_word(10),
                StateScalar::Layer2Policy => self.raw_word(12),
                StateScalar::MainCode => self.raw_word(18),
            },
            Self::Prepared { world, position } => {
                let state = &world.definition().expect("prepared state").states[position];
                match field {
                    StateScalar::MusicData => u16::from(state.music_data),
                    StateScalar::MusicTrack => u16::from(state.music_track),
                    StateScalar::Fx => state
                        .fx
                        .map_or(0, |id| world.resource_word(id, ResourceKind::Fx, 0x83)),
                    StateScalar::EnemyPopulation => world.resource_word(
                        state.enemy_population,
                        ResourceKind::EnemyPopulation,
                        0xa1,
                    ),
                    StateScalar::EnemyTilesets => {
                        world.resource_word(state.enemy_tilesets, ResourceKind::EnemyTilesets, 0xb4)
                    }
                    StateScalar::Layer2Policy => u16::from_le_bytes(state.layer2_policy),
                    StateScalar::MainCode => state.main_code.map_or(0, |id| {
                        world.resource_word(id, ResourceKind::RoomMain, 0x8f)
                    }),
                }
            }
        }
    }

    pub(crate) fn level(&self) -> AssetRef {
        match self {
            Self::Legacy { .. } => legacy_asset(self, 0),
            Self::Prepared { world, position } => AssetRef::Prepared(
                world.definition().expect("prepared state").states[position].level,
            ),
        }
    }

    pub(crate) fn tileset(&self) -> TilesetSelection {
        match self {
            Self::Legacy { .. } => TilesetSelection::Legacy(self.raw_byte(3)),
            Self::Prepared { world, position } => TilesetSelection::Prepared(
                world.definition().expect("prepared state").states[position].tileset,
            ),
        }
    }

    pub(crate) fn setup(&self) -> Option<SetupAction> {
        match self {
            Self::Legacy { .. } => {
                let code = self.raw_word(24);
                (code != 0).then_some(SetupAction::Legacy {
                    kind: SetupKind::Room,
                    code,
                })
            }
            Self::Prepared { world, position } => {
                world.definition().expect("prepared state").states[position]
                    .setup
                    .map(SetupAction::Prepared)
            }
        }
    }

    pub(crate) fn scrolls(&self) -> ScrollDefinition {
        match self {
            Self::Legacy { .. } => {
                let cursor = self.raw_word(14);
                if cursor & 0x8000 == 0 {
                    ScrollDefinition::Generated(cursor as u8)
                } else {
                    ScrollDefinition::Records(ScrollCursor::Legacy(cursor))
                }
            }
            Self::Prepared { world, position } => {
                match &world.definition().expect("prepared state").states[position].scrolls {
                    AuthoredScroll::Generated(color) => ScrollDefinition::Generated(*color),
                    AuthoredScroll::Cells(_) => ScrollDefinition::Records(ScrollCursor::Prepared {
                        state: *position,
                        index: 0,
                    }),
                }
            }
        }
    }

    pub(crate) fn objects(&self, mode: LoadMode) -> Option<ObjectCursor> {
        match self {
            Self::Legacy { .. } => {
                let cursor = self.raw_word(20);
                if cursor == 0 {
                    return None;
                }
                Some(match mode {
                    LoadMode::Initial => ObjectCursor::LegacyInitial(cursor),
                    LoadMode::Door => ObjectCursor::LegacyDoor(i32::from(cursor)),
                })
            }
            Self::Prepared { world, position } => {
                let state = &world.definition().expect("prepared state").states[position];
                (!state.objects.is_empty()).then_some(ObjectCursor::Prepared {
                    state: *position,
                    index: 0,
                })
            }
        }
    }
}

impl Capture<TilesetId> {
    pub(crate) fn asset(&self, asset: TilesetAsset) -> AssetRef {
        match self {
            Self::Legacy { .. } => legacy_asset(
                self,
                match asset {
                    TilesetAsset::TileTable => 0,
                    TilesetAsset::Tiles => 3,
                    TilesetAsset::Palette => 6,
                },
            ),
            Self::Prepared { world, position } => {
                let tileset = &world.definition().expect("prepared tileset").tilesets[position];
                AssetRef::Prepared(match asset {
                    TilesetAsset::TileTable => tileset.tile_table,
                    TilesetAsset::Tiles => tileset.tiles,
                    TilesetAsset::Palette => tileset.palette,
                })
            }
        }
    }
}

// RomRecord::long reads a native word, then its bank byte. The RAM projection
// stores that numeric word little endian; this is not a three-byte raw copy.
fn legacy_asset<P>(capture: &Capture<P>, offset: usize) -> AssetRef {
    let [lo, hi] = capture.raw_word(offset).to_le_bytes();
    let bank = capture.raw_byte(offset + 2);
    AssetRef::Legacy([lo, hi, bank])
}
