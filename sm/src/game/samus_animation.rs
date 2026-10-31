//! Samus animation, sprite selection, echoes, atmospheric graphics and tile uploads.
#![forbid(unsafe_code)]
use crate::game::scene::samus_position_tiles;
use crate::game::session_runtime::Runtime;
use sm_game::samus_drawing::{self, BoundTable, RoleTable};
use sm_game::scene::owned_samus::SamusSources as _;
use sm_game::scene::samus_body::{self, Body};
use sm_game::scene::samus_content::{PreparedBytes, Registration, Role, RoleLease};

pub use crate::game::prelude::*;
use sm_game::cinematic::project_cinematic_point;

impl samus_body::BottomHost for Game {
    type Error = std::convert::Infallible;
    fn pose(&mut self) -> Result<u16, Self::Error> {
        Ok(self.samus_pose())
    }
    fn animation_frame(&mut self) -> Result<u16, Self::Error> {
        Ok(self.samus_anim_frame())
    }
    fn movement(&mut self) -> Result<u8, Self::Error> {
        Ok(self.samus_movement_type())
    }
    fn clear_bottom(&mut self) -> Result<(), Self::Error> {
        word!(self.SAMUS_BOTTOM_HALF_SPRITEMAP_INDEX = 0);
        Ok(())
    }
    fn cursor(&mut self) -> Result<u16, Self::Error> {
        Ok(word!(self.OAM_NEXT_PTR))
    }
    fn player_x(&mut self) -> Result<u16, Self::Error> {
        Ok(word!(self.SAMUS_X_POS))
    }
    fn player_y(&mut self) -> Result<u16, Self::Error> {
        Ok(word!(self.SAMUS_Y_POS))
    }
    fn camera_x(&mut self) -> Result<u16, Self::Error> {
        Ok(word!(self.LAYER1_X_POS))
    }
    fn camera_y(&mut self) -> Result<u16, Self::Error> {
        Ok(word!(self.LAYER1_Y_POS))
    }
    fn marker_word(&mut self, entry: usize, value: u16) -> Result<(), Self::Error> {
        word_transfers!(self, ascending, 1; native(0x370 + entry) <- value(value));
        Ok(())
    }
    fn publish_cursor(&mut self, value: u16) -> Result<(), Self::Error> {
        word!(self.OAM_NEXT_PTR = value);
        Ok(())
    }
    fn marker_written(&mut self, _: usize, _: u16, _: u16) -> Result<(), Self::Error> {
        Ok(())
    }
}

/// The role table over the native `record`, bound to its role lease when
/// Samus content is bound.
pub(super) fn capture_table(
    game: &Game,
    role: Role,
    source: Registration,
    record: crate::rom::RomRecord,
) -> RoleTable {
    let bound = game.samus_execution().unwrap().map(|execution| {
        let base = game.samus_role_base(role, source).unwrap().unwrap();
        let displacement = record
            .base
            .checked_sub(base)
            .expect("Samus role mapped base");
        match execution {
            crate::game_state::SamusExecution::Prepared => BoundTable::Prepared(
                game.prepared_samus_role(role, source, displacement)
                    .unwrap()
                    .unwrap(),
            ),
            crate::game_state::SamusExecution::RetainedNative => BoundTable::Native(
                game.capture_samus_role(role, source, displacement, &record.image)
                    .unwrap()
                    .unwrap(),
            ),
        }
    });
    RoleTable::captured(record, bound)
}
impl sm_game::samus_death::Host for Game {
    fn draw_samus(&mut self, body: Body) {
        draw_body(self, body);
    }
}

fn draw_body(game: &mut Game, mode: Body) {
    samus_drawing::draw_body(&mut Runtime::new(game), mode);
}

pub fn samus_draw_when_not_animating_or_dying(game: &mut Game) {
    draw_body(game, Body::Frozen);
}

impl samus_drawing::Authority for Runtime<'_> {
    fn frame_counter(&self) -> u16 {
        self.inputs().frame
    }
    fn camera(&self) -> (u16, u16) {
        (self.inputs().camera_x, self.inputs().camera_y)
    }
    fn has_owned_display(&self) -> bool {
        Runtime::has_owned_display(self)
    }
    fn draw_owned_body(&mut self, mode: Body) {
        Runtime::draw_owned_body(self, mode);
    }
    fn draw_owned_echo(&mut self, slot: u16) {
        Runtime::draw_owned_echo(self, slot);
    }
    fn owned_bottom_visible(&mut self) -> bool {
        Runtime::owned_bottom_visible(self)
    }
    fn draw_echo_samus_sprite(&mut self, slot: u16, index: u16, x: u16, y: u16) {
        Runtime::draw_echo_samus_sprite(self, slot, index, x, y);
    }
    fn draw_native_echo(&mut self, slot: u16) {
        self.foreign(|game| {
            sm_game::sprite_output::echo(slot, game, &mut sm_game::sprite_output::Native)
        });
    }
    fn samus_source_diagnostic(&mut self, address: u32) {
        Runtime::samus_source_diagnostic(self, address);
    }
    fn samus_request_written(&mut self, half: samus_body::Half, pointer: u16) {
        Runtime::samus_request_written(self, half, pointer);
    }
    fn prepared_role(
        &self,
        role: Role,
        registration: Registration,
        displacement: usize,
    ) -> RoleTable {
        let lease = self
            .samus_game()
            .prepared_samus_role(role, registration, displacement)
            .expect("Samus prepared role")
            .expect("supplied Samus role");
        RoleTable::prepared(lease)
    }
    fn captured_role(
        &self,
        role: Role,
        registration: Registration,
        record: crate::rom::RomRecord,
    ) -> RoleTable {
        capture_table(self.samus_game(), role, registration, record)
    }
    fn role_base(&self, role: Role, registration: Registration) -> usize {
        self.samus_game()
            .samus_role_base(role, registration)
            .unwrap()
            .unwrap()
    }
    fn sprite_position(&mut self, doubled: u16) -> (u16, u16) {
        self.foreign(|game| samus_position_tiles::sprite_position(game, doubled).unwrap())
    }
    fn bottom_visible(&mut self) -> bool {
        self.foreign(|game| samus_body::bottom_is_visible(game).unwrap())
    }
}

impl samus_position_tiles::PositionHost for Game {
    type Error = std::convert::Infallible;
    fn ceres(&mut self) -> Result<u16, Self::Error> {
        Ok(word!(self.CERES_STATUS))
    }
    fn player_x(&mut self) -> Result<u16, Self::Error> {
        Ok(word!(self.SAMUS_X_POS))
    }
    fn player_y(&mut self) -> Result<u16, Self::Error> {
        Ok(word!(self.SAMUS_Y_POS))
    }
    fn set_player_x(&mut self, value: u16) -> Result<(), Self::Error> {
        word!(self.SAMUS_X_POS = value);
        Ok(())
    }
    fn set_player_y(&mut self, value: u16) -> Result<(), Self::Error> {
        word!(self.SAMUS_Y_POS = value);
        Ok(())
    }
    fn project(&mut self, x: u16, y: u16) -> Result<(u16, u16), Self::Error> {
        let point = project_cinematic_point(sm_game::geometry::Point { x, y }, self);
        Ok((point.x, point.y))
    }
    fn frame(&mut self) -> Result<u16, Self::Error> {
        Ok(self.samus_anim_frame())
    }
    fn movement(&mut self) -> Result<u8, Self::Error> {
        Ok(self.samus_movement_type())
    }
    fn landing_word(&self, bytes: [u8; 2]) -> u16 {
        u16::from_ne_bytes(bytes)
    }
    fn landing_offset(&mut self, offset: usize) -> Result<u16, Self::Error> {
        Ok(match self.native_offsets(Role::Landing) {
            Some(table) => table.native_word(offset).unwrap(),
            None => self.landing_word(samus_position_tiles::default_landing_bytes(offset)),
        })
    }
    fn crouching_offset(&mut self, offset: usize) -> Result<i8, Self::Error> {
        Ok(match self.native_offsets(Role::Crouching) {
            Some(table) => table.byte(offset).unwrap() as i8,
            None => samus_position_tiles::default_crouching_byte(offset),
        })
    }
    fn drained_offset(&mut self, offset: usize) -> Result<i8, Self::Error> {
        Ok(match self.native_offsets(Role::Drained) {
            Some(table) => table.byte(offset).unwrap() as i8,
            None => samus_position_tiles::default_drained_byte(offset),
        })
    }
    fn fallback_byte(&mut self, offset: usize) -> Result<u8, Self::Error> {
        let image = self.capture_native_rom();
        let record = crate::rom::RomRecord {
            image,
            base: crate::rtl::lorom_offset(0x91b629) as usize,
        };
        Ok(capture_table(self, Role::PositionFallback, Registration::Native, record).byte(offset))
    }
    fn camera_x(&mut self) -> Result<u16, Self::Error> {
        Ok(word!(self.LAYER1_X_POS))
    }
    fn camera_y(&mut self) -> Result<u16, Self::Error> {
        Ok(word!(self.LAYER1_Y_POS))
    }
    fn set_body_x(&mut self, value: u16) -> Result<(), Self::Error> {
        word!(self.SAMUS_SPRITEMAP_X_POS = value);
        Ok(())
    }
    fn set_body_y(&mut self, value: u16) -> Result<(), Self::Error> {
        word!(self.SAMUS_SPRITEMAP_Y_POS = value);
        Ok(())
    }
}
impl Game {
    /// A prepared native Samus offset table, when the prepared content supplies it.
    fn native_offsets(&self, role: Role) -> Option<RoleLease<PreparedBytes>> {
        self.prepared_samus_role(role, Registration::Native, 0)
            .unwrap()
    }
}
