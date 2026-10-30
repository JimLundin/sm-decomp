//! Owned-Display Samus drawing: the body, position and sprite inputs a
//! persistent Session's Display draws Samus from, over its prepared content.
use crate::external_inputs::ExternalInputs;
use crate::scene::records::{MediaRef, ObjectContext, ObjectSelection, ProducerObjectId};
use crate::scene::samus_body::MarkerPublication;
use crate::scene::samus_content::{PreparedBytes, Registration, Role, RoleLease};
use crate::scene::state::{DisplayError, DisplayState, WordOrder};
use crate::scene::{samus, samus_body, samus_position_tiles as positions};
use crate::session_owners::SamusPublications;
use std::sync::Arc;

/// The bound Samus content a persistent Session draws from.
pub trait SamusSources: crate::cartridge::NativeDiagnostics {
    fn prepared_samus_role(
        &self,
        role: Role,
        registration: Registration,
        displacement: usize,
    ) -> Result<Option<RoleLease<PreparedBytes>>, DisplayError>;
    fn samus_role_base(
        &self,
        role: Role,
        registration: Registration,
    ) -> Result<Option<usize>, DisplayError>;
    fn samus_request(
        &self,
        half: samus_body::Half,
        pointer: u16,
    ) -> Result<Option<positions::TileRequest>, DisplayError>;
    fn samus_spritemaps(&self) -> Result<Option<Arc<samus::SamusCatalog>>, DisplayError>;
}

struct PreparedTable(RoleLease<PreparedBytes>);
impl positions::Table for PreparedTable {
    type Error = DisplayError;
    fn native_word(&self, offset: usize) -> Result<u16, DisplayError> {
        self.0.native_word(offset)
    }
    fn byte(&self, offset: usize) -> Result<u8, DisplayError> {
        self.0.byte(offset)
    }
}
impl samus_body::Table for PreparedTable {
    type Error = DisplayError;
    fn little_word(&self, offset: usize) -> Result<u16, DisplayError> {
        self.0.little_word(offset)
    }
    fn native_word(&self, offset: usize) -> Result<u16, DisplayError> {
        self.0.native_word(offset)
    }
}

struct BodyInputs<'a, G: SamusSources + ?Sized> {
    game: &'a mut G,
    player: &'a mut crate::session::Player,
    observations: &'a ExternalInputs,
    publications: Arc<SamusPublications>,
    palette: MediaRef,
    raster: crate::scene::state::RasterState,
}
impl<G: SamusSources + ?Sized> BodyInputs<'_, G> {
    fn source(
        &self,
        role: Role,
        registration: Registration,
        displacement: usize,
    ) -> Result<PreparedTable, DisplayError> {
        self.game
            .prepared_samus_role(role, registration, displacement)?
            .map(PreparedTable)
            .ok_or(DisplayError::Incomplete)
    }
}
impl<G: SamusSources + ?Sized> samus::Inputs for BodyInputs<'_, G> {
    fn content(&mut self, _: &DisplayState) -> Result<Arc<samus::SamusCatalog>, DisplayError> {
        self.game
            .samus_spritemaps()?
            .ok_or(DisplayError::Incomplete)
    }
    fn invalid_map(&mut self, _: &mut DisplayState, cursor: u16) -> Result<(), DisplayError> {
        self.game
            .native_rom_diagnostic(0x920000 | u32::from(cursor));
        Ok(())
    }
    fn object_context(
        &mut self,
        controls: &crate::scene::state::DisplayControls,
        _: &crate::scene::state::EffectiveDisplayDescription,
    ) -> Result<ObjectContext, DisplayError> {
        let atlases = *self
            .publications
            .object_atlases
            .get(&controls.object_layout)
            .ok_or(DisplayError::Incomplete)?;
        Ok(ObjectContext {
            selection: ObjectSelection {
                atlases,
                size_set: controls.object_layout >> 5,
                first_order: self.publications.object_first_order,
                interlaced: self.raster.interlaced,
                even_frame: self.raster.even_frame,
            },
            palette: self.palette,
        })
    }
}
impl<G: SamusSources + ?Sized> positions::Inputs for BodyInputs<'_, G> {
    type Table = PreparedTable;
    fn ceres(&mut self) -> Result<u16, DisplayError> {
        Ok(self.observations.ceres)
    }
    fn player_x(&mut self) -> Result<u16, DisplayError> {
        Ok(self.player.x)
    }
    fn player_y(&mut self) -> Result<u16, DisplayError> {
        Ok(self.player.y)
    }
    fn set_player_x(&mut self, _: &mut DisplayState, value: u16) -> Result<(), DisplayError> {
        self.player.x = value;
        Ok(())
    }
    fn set_player_y(&mut self, _: &mut DisplayState, value: u16) -> Result<(), DisplayError> {
        self.player.y = value;
        Ok(())
    }
    fn project(
        &mut self,
        state: &mut DisplayState,
        x: u16,
        y: u16,
    ) -> Result<(u16, u16), DisplayError> {
        let point = crate::cinematic::project_cinematic_point_reads(
            crate::geometry::Point { x, y },
            |at| state.controls.affine_words[(at - 0x78) / 2],
        );
        Ok((point.x, point.y))
    }
    fn animation_frame(&mut self) -> Result<u16, DisplayError> {
        Ok(self.player.animation_frame())
    }
    fn movement(&mut self) -> Result<u8, DisplayError> {
        Ok(self.player.movement_and_facing.to_le_bytes()[1])
    }
    fn landing_order(&self) -> WordOrder {
        self.source(Role::Landing, Registration::Native, 0)
            .expect("Landing content")
            .0
            .native_order()
    }
    fn landing_offset(&mut self, offset: usize) -> Result<u16, DisplayError> {
        self.source(Role::Landing, Registration::Native, 0)?
            .0
            .native_word(offset)
    }
    fn crouching_offset(&mut self, offset: usize) -> Result<i8, DisplayError> {
        Ok(self
            .source(Role::Crouching, Registration::Native, 0)?
            .0
            .byte(offset)? as i8)
    }
    fn drained_offset(&mut self, offset: usize) -> Result<i8, DisplayError> {
        Ok(self
            .source(Role::Drained, Registration::Native, 0)?
            .0
            .byte(offset)? as i8)
    }
    fn position_source(&mut self) -> Result<PreparedTable, DisplayError> {
        self.source(Role::PositionFallback, Registration::Native, 0)
    }
    fn camera_x(&mut self) -> Result<u16, DisplayError> {
        Ok(self.observations.camera_x)
    }
    fn camera_y(&mut self) -> Result<u16, DisplayError> {
        Ok(self.observations.camera_y)
    }
    fn set_body_x(&mut self, _: &mut DisplayState, value: u16) -> Result<(), DisplayError> {
        self.player.body_position.x = value;
        Ok(())
    }
    fn set_body_y(&mut self, _: &mut DisplayState, value: u16) -> Result<(), DisplayError> {
        self.player.body_position.y = value;
        Ok(())
    }
    fn body_x(&mut self) -> Result<u16, DisplayError> {
        Ok(self.player.body_position.x)
    }
    fn body_y(&mut self) -> Result<u16, DisplayError> {
        Ok(self.player.body_position.y)
    }
    fn pose(&mut self) -> Result<u16, DisplayError> {
        Ok(self.player.pose)
    }
    fn tile_table(&mut self, which: positions::TileTable) -> Result<PreparedTable, DisplayError> {
        let role = match which {
            positions::TileTable::Pose => Role::TilePose,
            positions::TileTable::Top => Role::TileTop,
            positions::TileTable::Bottom => Role::TileBottom,
        };
        self.source(role, Registration::Loaded, 0)
    }
    fn tile_definition(
        &mut self,
        _: &mut DisplayState,
        cursor: u16,
    ) -> Result<PreparedTable, DisplayError> {
        if cursor & 0x8000 == 0 {
            self.game
                .native_rom_diagnostic(0x920000 | u32::from(cursor));
        }
        let mapped = sm_data::cartridge::lorom_offset(0x920000 | u32::from(cursor)) as usize;
        let base = self
            .game
            .samus_role_base(Role::TileDefinition, Registration::Native)?
            .ok_or(DisplayError::Incomplete)?;
        self.source(
            Role::TileDefinition,
            Registration::Native,
            mapped.checked_sub(base).ok_or(DisplayError::Source)?,
        )
    }
    fn request_word(
        &mut self,
        state: &mut DisplayState,
        half: samus_body::Half,
        pointer: u16,
    ) -> Result<(), DisplayError> {
        let slot = match half {
            samus_body::Half::Top => 0,
            samus_body::Half::Bottom => 1,
        };
        let [low, high] = pointer.to_le_bytes();
        let mut bytes = state.samus[slot].pointer.to_le_bytes();
        bytes[0] = low;
        state.samus[slot].pointer = u16::from_le_bytes(bytes);
        bytes[1] = high;
        state.samus[slot].pointer = u16::from_le_bytes(bytes);
        Ok(())
    }
    fn request_flag(
        &mut self,
        state: &mut DisplayState,
        half: samus_body::Half,
    ) -> Result<(), DisplayError> {
        state.samus[match half {
            samus_body::Half::Top => 0,
            samus_body::Half::Bottom => 1,
        }]
        .enabled = 1;
        Ok(())
    }
    fn tile_request(
        &mut self,
        _: &DisplayState,
        half: samus_body::Half,
        pointer: u16,
    ) -> Result<positions::TileRequest, DisplayError> {
        self.game
            .samus_request(half, pointer)?
            .ok_or(DisplayError::Incomplete)
    }
}
impl<G: SamusSources + ?Sized> samus_body::Inputs for BodyInputs<'_, G> {
    type BodyTable = PreparedTable;
    fn knockback(&mut self) -> Result<u16, DisplayError> {
        Ok(self.player.knockback_timer)
    }
    fn invincibility(&mut self) -> Result<u16, DisplayError> {
        Ok(self.player.invincibility)
    }
    fn shine(&mut self) -> Result<u16, DisplayError> {
        Ok(self.player.shine_timer)
    }
    fn frame_counter(&mut self) -> Result<u16, DisplayError> {
        Ok(self.observations.frame)
    }
    fn table(&mut self, half: samus_body::Half) -> Result<PreparedTable, DisplayError> {
        self.source(
            match half {
                samus_body::Half::Top => Role::BodyTop,
                samus_body::Half::Bottom => Role::BodyBottom,
            },
            Registration::Loaded,
            0,
        )
    }
    fn set_sprite(&mut self, half: samus_body::Half, index: u16) -> Result<(), DisplayError> {
        match half {
            samus_body::Half::Top => self.player.top_sprite = index,
            samus_body::Half::Bottom => self.player.bottom_sprite = index,
        };
        Ok(())
    }
}

/// The inputs an owned Display draws Samus with.
fn inputs<'a, G: SamusSources + ?Sized>(
    owner: &crate::scene::commit::OwnedDisplay,
    game: &'a mut G,
    player: &'a mut crate::session::Player,
    observations: &'a ExternalInputs,
    publications: Arc<SamusPublications>,
) -> BodyInputs<'a, G> {
    BodyInputs {
        game,
        player,
        observations,
        publications,
        palette: owner.image().palette,
        raster: owner.state.raster,
    }
}

pub fn draw_body<G: SamusSources + ?Sized>(
    owner: &mut crate::scene::commit::OwnedDisplay,
    game: &mut G,
    player: &mut crate::session::Player,
    observations: &ExternalInputs,
    publications: Arc<SamusPublications>,
    marker: &MarkerPublication,
    mode: samus_body::Body,
) -> Result<(), DisplayError> {
    let media = owner.media().clone();
    let producer = publications.body;
    let mut inputs = inputs(owner, game, player, observations, publications);
    samus_body::draw_owned(
        &mut owner.state,
        &media,
        producer,
        marker,
        mode,
        &mut inputs,
    )
}
pub fn bottom_visible<G: SamusSources + ?Sized>(
    owner: &mut crate::scene::commit::OwnedDisplay,
    game: &mut G,
    player: &mut crate::session::Player,
    observations: &ExternalInputs,
    publications: Arc<SamusPublications>,
    marker: &MarkerPublication,
) -> Result<bool, DisplayError> {
    let media = owner.media().clone();
    let producer = publications.body;
    let mut inputs = inputs(owner, game, player, observations, publications);
    samus_body::bottom_owned(&mut owner.state, &media, producer, marker, &mut inputs)
}
/// One Samus sprite: its producer, spritemap index and screen position.
pub struct Sprite {
    pub producer: ProducerObjectId,
    pub index: u16,
    pub position: (u16, u16),
}
pub fn draw_sprite<G: SamusSources + ?Sized>(
    owner: &mut crate::scene::commit::OwnedDisplay,
    game: &mut G,
    player: &mut crate::session::Player,
    observations: &ExternalInputs,
    publications: Arc<SamusPublications>,
    sprite: Sprite,
) -> Result<(), DisplayError> {
    let media = owner.media().clone();
    let mut inputs = inputs(owner, game, player, observations, publications);
    let Sprite {
        producer,
        index,
        position,
    } = sprite;
    samus::draw_owned(
        &mut owner.state,
        &media,
        producer,
        index,
        position,
        &mut inputs,
    )
}
