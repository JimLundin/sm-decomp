//! Game's implementations of the sm_game host traits whose modules moved.
#![forbid(unsafe_code)]
use crate::game_state::Game;
use sm_game::display::transfer::ScrollingLayout;
use sm_game::player_collision::operations::Axis;

impl sm_game::room_scrolls::Host for Game {
    fn unreachable(&mut self) -> bool {
        crate::rtl::Unreachable(self)
    }
    fn record_camera_distance(&mut self, axis: Axis) {
        sm_game::player_movement::speed::record_camera_distance(
            crate::game::session_runtime::Runtime::new(self).session_mut(),
            axis,
        );
    }
    fn queue_scrolling_tiles(&mut self, k: u16, layout: ScrollingLayout) {
        crate::game::display_refresh::queue_scrolling_tiles(k, layout, self);
    }
}

impl sm_game::explosion_window::Host for Game {
    fn initialize_crystal_flash(&mut self) -> bool {
        sm_game::player_movement::forced::initialize_crystal_flash(
            &mut crate::game::session_runtime::Runtime::new(self),
        )
    }
    fn window_row(&self) -> u16 {
        self.k_out
    }
    fn set_window_row(&mut self, row: u16) {
        self.k_out = row;
    }
}

impl sm_game::runtime_access::RuntimeAccess for Game {
    type Runtime<'a> = crate::game::session_runtime::Runtime<'a>;
    fn runtime(&mut self) -> Self::Runtime<'_> {
        crate::game::session_runtime::Runtime::new(self)
    }
}

impl sm_game::grapple_swing::Host for Game {
    fn multiply_with_scratch(&mut self, a: u16, b: u16) -> u32 {
        Game::multiply_with_scratch(self, a, b)
    }
    fn player_bottom(&self) -> u16 {
        crate::game::session_runtime::player_bottom(self)
    }
    fn settle_grapple(&mut self) {
        crate::game::session_runtime::settle_grapple(self)
    }
}

impl sm_game::runtime_access::SessionRam for crate::game::session_runtime::Runtime<'_> {
    fn byte(&self, at: usize) -> u8 {
        crate::game::session_runtime::Runtime::byte(self, at)
    }
    fn set_byte(&mut self, at: usize, value: u8) {
        crate::game::session_runtime::Runtime::set_byte(self, at, value)
    }
    fn write_slice(&mut self, at: usize, bytes: &[u8]) {
        crate::game::session_runtime::Runtime::write_slice(self, at, bytes)
    }
}

impl sm_game::actors::boulder::Host for Game {
    fn enemy_speed32(&self, offset: usize) -> u32 {
        crate::game::enemy_speeds::word32(&self.enemy_speeds32, &self.capture_native_rom(), offset)
    }
}

impl sm_game::actors::deaths::Deaths for Game {
    fn normal_death(&mut self, animation: u16) {
        crate::game::actors::combat::die(
            &mut crate::game::session_runtime::Runtime::new(self),
            crate::game::actors::api::DeathKind::Normal { animation },
        );
    }
    fn scatter_enemy_drop(
        &mut self,
        source: sm_game::enemy_projectiles::drops::DropSource,
        index: u16,
    ) {
        crate::game::enemy_projectiles::scatter_drop(source, index, self);
    }
}

impl sm_game::actors::zebetites::Host for Game {
    fn event_happened(&mut self, event: u16) -> u16 {
        crate::game::gameplay_flag_runtime::CheckEventHappened(event, self)
    }
    fn set_event(&mut self, event: u16) {
        crate::game::gameplay_flag_runtime::SetEventHappened(event, self);
    }
    fn clear_event(&mut self, event: u16) {
        crate::game::gameplay_flag_runtime::ClearEventHappened(event, self);
    }
    fn spawn_population(&mut self, bank: u8, population: u16) -> u16 {
        crate::game::actors::population::spawn_legacy(
            &mut crate::game::session_runtime::Runtime::new(self),
            bank,
            population,
        )
    }
}

/// Implements `PlayerPosition` over a compatibility-RAM receiver with the
/// original named-word reads and stores.
macro_rules! compatibility_player_position {
    ($($ty:ty),+ $(,)?) => {$(
        impl sm_game::player_position::PlayerPosition for $ty {
            fn player_x(&self) -> u16 { let game = self; sm_game::ram_word!(game.samus_x_pos) }
            fn player_y(&self) -> u16 { let game = self; sm_game::ram_word!(game.samus_y_pos) }
            fn player_x_fraction(&self) -> u16 { let game = self; sm_game::ram_word!(game.samus_x_subpos) }
            fn player_y_fraction(&self) -> u16 { let game = self; sm_game::ram_word!(game.samus_y_subpos) }
            fn previous_player_x(&self) -> u16 { let game = self; sm_game::ram_word!(game.samus_prev_x_pos) }
            fn previous_player_y(&self) -> u16 { let game = self; sm_game::ram_word!(game.samus_prev_y_pos) }
            fn previous_player_x_fraction(&self) -> u16 { let game = self; sm_game::ram_word!(game.samus_prev_x_subpos) }
            fn previous_player_y_fraction(&self) -> u16 { let game = self; sm_game::ram_word!(game.samus_prev_y_subpos) }
            fn set_player_x(&mut self, x: u16) { let game = self; sm_game::ram_word!(game.samus_x_pos = x) }
            fn set_player_y(&mut self, y: u16) { let game = self; sm_game::ram_word!(game.samus_y_pos = y) }
            fn set_player_x_fraction(&mut self, fraction: u16) { let game = self; sm_game::ram_word!(game.samus_x_subpos = fraction) }
            fn set_player_y_fraction(&mut self, fraction: u16) { let game = self; sm_game::ram_word!(game.samus_y_subpos = fraction) }
            fn set_previous_player_x(&mut self, x: u16) { let game = self; sm_game::ram_word!(game.samus_prev_x_pos = x) }
            fn set_previous_player_y(&mut self, y: u16) { let game = self; sm_game::ram_word!(game.samus_prev_y_pos = y) }
            fn set_previous_player_x_fraction(&mut self, fraction: u16) { let game = self; sm_game::ram_word!(game.samus_prev_x_subpos = fraction) }
            fn set_previous_player_y_fraction(&mut self, fraction: u16) { let game = self; sm_game::ram_word!(game.samus_prev_y_subpos = fraction) }
        }
    )+};
}

compatibility_player_position!(Game);

/// Inside a lease, Samus's position is Session-owned: while the lease is current
/// the reads come straight from `Session::player`, exactly as the projection
/// would serve them. A dirty lease falls back to the projected word reads, and
/// stores go through the lease's own byte path so observation refresh, epoch
/// and semantic-counterpart handling are unchanged.
impl sm_game::player_position::PlayerPosition for crate::game::session_runtime::Runtime<'_> {
    fn player_x(&self) -> u16 {
        self.owned_player(|player| player.x, sm_game::ram::words::samus_x_pos)
    }
    fn player_y(&self) -> u16 {
        self.owned_player(|player| player.y, sm_game::ram::words::samus_y_pos)
    }
    fn player_x_fraction(&self) -> u16 {
        self.owned_player(
            |player| player.x_fraction,
            sm_game::ram::words::samus_x_subpos,
        )
    }
    fn player_y_fraction(&self) -> u16 {
        self.owned_player(
            |player| player.y_fraction,
            sm_game::ram::words::samus_y_subpos,
        )
    }
    fn previous_player_x(&self) -> u16 {
        self.owned_player(|player| player.previous_position.x.whole, 0xb10)
    }
    fn previous_player_y(&self) -> u16 {
        self.owned_player(|player| player.previous_position.y.whole, 0xb14)
    }
    fn previous_player_x_fraction(&self) -> u16 {
        self.owned_player(|player| player.previous_position.x.fraction, 0xb12)
    }
    fn previous_player_y_fraction(&self) -> u16 {
        self.owned_player(|player| player.previous_position.y.fraction, 0xb16)
    }
    fn set_player_x(&mut self, x: u16) {
        self.write_word_le(x, sm_game::ram::words::samus_x_pos)
    }
    fn set_player_y(&mut self, y: u16) {
        self.write_word_le(y, sm_game::ram::words::samus_y_pos)
    }
    fn set_player_x_fraction(&mut self, fraction: u16) {
        self.write_word_le(fraction, sm_game::ram::words::samus_x_subpos)
    }
    fn set_player_y_fraction(&mut self, fraction: u16) {
        self.write_word_le(fraction, sm_game::ram::words::samus_y_subpos)
    }
    fn set_previous_player_x(&mut self, x: u16) {
        self.write_word_le(x, 0xb10)
    }
    fn set_previous_player_y(&mut self, y: u16) {
        self.write_word_le(y, 0xb14)
    }
    fn set_previous_player_x_fraction(&mut self, fraction: u16) {
        self.write_word_le(fraction, 0xb12)
    }
    fn set_previous_player_y_fraction(&mut self, fraction: u16) {
        self.write_word_le(fraction, 0xb16)
    }
}

/// The raw work-RAM byte reads these facts replaced, unchanged.
impl sm_game::player_position::PlayerFacts for Game {
    fn facing(&self) -> u8 {
        self.ram().read_byte(0xa1e)
    }
    fn movement_type(&self) -> u8 {
        self.ram().read_byte(0xa1f)
    }
    fn grapple_angle(&self) -> u8 {
        self.ram().read_byte(0xcfb)
    }
    fn grapple_mirrored_angle(&self) -> u8 {
        self.ram().read_byte(0xcfd)
    }
}

// The falling tubes' bank $A9 functions are the ones `CallMotherBrainsTubesFalling` dispatched on.
const _: () = {
    use sm_game::actors::mother_brain_tubes::{self as tubes, function as f};
    const fn at(word: u16) -> u32 {
        0xa9_0000 | word as u32
    }
    assert!(at(f::FALL) == 11111304 && at(f::WAIT_TO_FALL) == 11111371);
    assert!(at(f::FALL_WITH_BRAIN) == 11111382);
    // MotherBrainBody_FakeDeath_Ascent_0_DrawBG1Row23.
    assert!(at(tubes::BODY_ASCENT) == 11111559);
    assert!(
        tubes::DUST_CLOUD == sm_game::constants::bank_86::addr_kEproj_DustCloudExplosion as u16
    );
};

impl sm_game::actors::mother_brain_tubes::Host for Game {
    fn end_screen_flash(&mut self) {
        crate::game::sm_a9::MotherBrain_Pal_EndScreenFlash(self);
    }

    fn start_earthquake(&mut self, kind: u16) {
        crate::game::mother_brain_visuals::start_earthquake(kind, self);
    }

    fn setup_neck_for_fake_ascent(&mut self) {
        sm_game::actors::mother_brain_neck::setup_for_fake_ascent(self);
    }
}

// The phase-3 neck functions are the ones `CallMotherBrainNeckFunc` dispatched on.
const _: () = {
    use crate::game::sm_a9::*;
    use sm_game::actors::mother_brain_neck::{animation as a, function as f};
    const fn at(word: u16) -> u32 {
        0xa9_0000 | word as u32
    }
    assert!(at(f::NORMAL) == 11125552 && at(f::IDLE) == 11125587);
    assert!(f::IDLE == addr_locret_A9C353 as u16);
    assert!(at(f::SETUP_RECOIL_RECOVERY) == 11125588 && at(f::RECOIL_RECOVERY) == 11125627);
    assert!(at(f::SETUP_HYPER_BEAM_RECOIL) == 11125653 && at(f::HYPER_BEAM_RECOIL) == 11125709);
    assert!(a::RECOVER == addr_kMotherBrain_Ilist_9DBB as u16);
    assert!(a::HYPER_BEAM_HIT == addr_kMotherBrain_Ilist_9BE7 as u16);
};

impl sm_game::actors::mother_brain_neck::Host for Game {
    fn sine_times(&mut self, length: u16, angle: u16) -> u16 {
        crate::game::sm_a9::ComputeSinMult(length, angle, self)
    }

    fn cosine_times(&mut self, length: u16, angle: u16) -> u16 {
        crate::game::sm_a9::ComputeCosMult(length, angle, self)
    }
}
