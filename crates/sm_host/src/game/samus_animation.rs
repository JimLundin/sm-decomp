//! Samus animation, sprite selection, echoes, atmospheric graphics and tile uploads.
#![forbid(unsafe_code)]
use crate::game::session_runtime::Runtime;
use sm_game::samus_drawing;
use sm_game::scene::samus_body::{self, Body};

pub use crate::game::prelude::*;

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
        // The echo draws on the lease's object owner, as the trail does.
        self.draw_objects(|drawing| {
            sm_game::sprite_output::echo(slot, drawing, &mut sm_game::sprite_output::Native)
        });
    }
    fn samus_source_diagnostic(&mut self, address: u32) {
        Runtime::samus_source_diagnostic(self, address);
    }
    fn samus_request_written(&mut self, half: samus_body::Half, pointer: u16) {
        Runtime::samus_request_written(self, half, pointer);
    }
    fn samus_sources(&self) -> sm_game::scene::samus_binding::SamusContent<'_> {
        self.samus_game().samus_content()
    }
}
