//! Samus animation, sprite selection, echoes, atmospheric graphics and tile uploads.
#![forbid(unsafe_code)]
use crate::game::session_runtime::Runtime;
use sm_game::samus_drawing::{self, BoundTable, RoleTable};
use sm_game::scene::owned_samus::SamusSources as _;
use sm_game::scene::samus_body::{self, Body};
use sm_game::scene::samus_content::{Registration, Role};

pub use crate::game::prelude::*;

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
    fn native_offsets(&self, role: Role) -> Option<RoleTable> {
        self.samus_game()
            .prepared_samus_role(role, Registration::Native, 0)
            .unwrap()
            .map(RoleTable::prepared)
    }
}
