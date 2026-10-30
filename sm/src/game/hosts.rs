//! Game's implementations of the sm_game host traits whose modules moved.
#![forbid(unsafe_code)]
use crate::game_state::Game;

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
    fn runtime_admissible(&self) -> bool {
        self.session_lease_admissible()
    }
}

impl sm_game::grapple_swing::Host for Game {
    type Lease<'a> = crate::game::session_runtime::Runtime<'a>;
    fn with_lease<'s, R>(&'s mut self, action: impl FnOnce(&mut Self::Lease<'s>) -> R) -> R {
        action(&mut crate::game::session_runtime::Runtime::new(self))
    }
    fn grapple_unreachable(&mut self) -> bool {
        sm_game::menus::host::MenuRuntime::rtl_unreachable(self)
    }
    fn grapple_pose_command(&mut self, command: u16) -> u16 {
        sm_game::menus::host::MenuRuntime::player_command(self, command)
    }
    fn multiply_with_scratch(&mut self, a: u16, b: u16) -> u32 {
        Game::multiply_with_scratch(self, a, b)
    }
    fn player_bottom(&self) -> u16 {
        crate::game::session_runtime::player_bottom(self)
    }
    fn settle_grapple(&mut self) {
        crate::game::session_runtime::settle_grapple(self)
    }
    fn block_behavior_word(&self, index: usize) -> i16 {
        let at = 0x16402 + index;
        i16::from_ne_bytes([self.ram().read_byte(at), self.ram().read_byte(at + 1)])
    }
    fn periodic_damage(&self) -> (u16, u16) {
        let low = u16::from_ne_bytes([self.ram().read_byte(0xa4e), self.ram().read_byte(0xa4f)]);
        let high = u16::from_ne_bytes([self.ram().read_byte(0xa50), self.ram().read_byte(0xa51)]);
        (low, high)
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

// The named bank $A7 functions are the ones `DispatchEnemyPreInstr` dispatches on.
const _: () = {
    use sm_game::actors::phantoon::function as f;
    const fn at(word: u16) -> u32 {
        0xa7_0000 | word as u32
    }
    assert!(
        at(f::SPAWN_STARTING_FLAMES) == 10998953 && at(f::WAIT_BEFORE_SPINNING_FLAMES) == 10999022
    );
    assert!(at(f::SPAWN_FLAMES_BEFORE_FIGHT) == 10999048 && at(f::WAVY_FADE_IN) == 10999114);
    assert!(at(f::PICK_ROUND_1_PATTERN) == 10999190 && at(f::FIGURE_EIGHT) == 10999271);
    assert!(at(f::EYE_FOLLOWS_SAMUS) == 10999309 && at(f::BECOME_SOLID_AND_SWOOP) == 10999388);
    assert!(at(f::SWOOPING) == 10999416 && at(f::FADE_OUT_WITH_SWOOP) == 10999481);
    assert!(at(f::WAIT_AFTER_FADE_OUT) == 10999508 && at(f::REAPPEAR) == 10999522);
    assert!(
        at(f::FADE_IN_BEFORE_FIGURE_EIGHT) == 10999597
            && at(f::BECOME_SOLID_AFTER_FLAME_RAIN) == 10999615
    );
    assert!(
        at(f::FADE_IN_DURING_FLAME_RAIN) == 10999655
            && at(f::FOLLOW_SAMUS_DURING_FLAME_RAIN) == 10999688
    );
    assert!(at(f::FADE_OUT_DURING_FLAME_RAIN) == 10999765 && at(f::SPAWN_FLAME_RAIN) == 10999799);
    assert!(
        at(f::FADE_OUT_BEFORE_FIRST_FLAME_RAIN) == 10999850
            && at(f::FADE_OUT_BEFORE_ENRAGE) == 10999900
    );
    assert!(at(f::MOVE_TO_TOP_CENTRE) == 10999924 && at(f::FADE_IN_ENRAGED) == 10999953);
    assert!(at(f::ENRAGED) == 10999980 && at(f::FADE_OUT_AFTER_ENRAGE) == 11000086);
    assert!(
        at(f::FINISH_SWOOP_AFTER_FATAL_SHOT) == 11000110 && at(f::DYING_FADE_IN_OUT) == 11000136
    );
    assert!(at(f::DYING_EXPLOSIONS) == 11000203 && at(f::WAVY_DYING_CRY) == 11000401);
    assert!(at(f::DYING_FADE_OUT) == 11000454 && at(f::ALMOST_DEAD) == 11000535);
    assert!(at(f::DEAD) == 11000637 && f::NOTHING == crate::game::sm_a7::fnnullsub_237 as u16);
};

// The named instruction lists are the decompilation's `kKraid_Ilist_CCxx`.
const _: () = {
    use crate::game::sm_a7::*;
    use sm_game::actors::phantoon::animation as a;
    assert!(
        a::BODY_INVULNERABLE == addr_kKraid_Ilist_CC41 as u16
            && a::BODY_FULL_HITBOX == addr_kKraid_Ilist_CC47 as u16
    );
    assert!(
        a::BODY_EYE_HITBOX == addr_kKraid_Ilist_CC4D as u16
            && a::EYE_OPEN == addr_kKraid_Ilist_CC53 as u16
    );
    assert!(
        a::EYE_CLOSE_AND_PICK_PATTERN == addr_kKraid_Ilist_CC81 as u16
            && a::EYE_CLOSE == addr_kKraid_Ilist_CC91 as u16
    );
    assert!(
        a::EYEBALL_CENTRED == addr_kKraid_Ilist_CC9D as u16
            && a::NOTHING_SPRITEMAP == addr_kSpritemap_Nothing_A7 as u16
    );
};

impl sm_game::actors::phantoon::Host for Game {
    fn run_function(&mut self, address: u32) {
        crate::game::sm_a7::run_a7_function(address, self);
    }

    fn direction_to_player(&mut self) -> sm_game::actors::api::PlayerDirection {
        crate::game::actors::api::direction_to_player(
            &mut crate::game::session_runtime::Runtime::new(self),
        )
    }

    fn spawn_hdma(&mut self, bank: u8, args: &sm_game::display::hdma_spawn::SpawnHdmaObject_Args) {
        crate::game::visual_runtime::spawn_hdma(bank, args, self);
    }

    fn start_wave(&mut self, direction: u16, speed: u16) {
        crate::game::visual_runtime::initialize_phantoon_motion(direction, speed, self);
    }
}

// The named bank $A7 functions are the ones the original `CallKraidFunc` dispatched on.
const _: () = {
    use sm_game::actors::kraid::function as f;
    const fn at(word: u16) -> u32 {
        0xa7_0000 | word as u32
    }
    assert!(
        at(f::GETS_BIG_BREAK_CEILING) == 10988621 && at(f::GETS_BIG_SET_BG2_PRIORITY) == 10988858
    );
    assert!(
        at(f::GETS_BIG_FINISH_BG2_TILEMAP) == 10988897 && at(f::GETS_BIG_DRAW_ROOM_BG) == 10988942
    );
    assert!(at(f::GETS_BIG_FADE_IN_ROOM_BG) == 10989091 && at(f::MAIN_LOOP_THINKING) == 10989220);
    assert!(at(f::GETS_BIG_THINKING) == 10989252 && at(f::MOUTH_OPEN) == 10989284);
    assert!(at(f::INIT_EYE_GLOW) == 10991295 && at(f::GLOW_EYE) == 10991319);
    assert!(at(f::UNGLOW_EYE) == 10991421 && at(f::NOTHING) == 10991665);
    assert!(at(f::LINT_PRODUCE) == 10991666 && at(f::LINT_CHARGE) == 10991720);
    assert!(at(f::LINT_FIRE) == 10991771 && at(f::FINGERNAIL_WAIT_FOR_LINT) == 10991879);
    assert!(at(f::ALIGN_TO_KRAID) == 10991907 && at(f::HANDLE_FUNCTION_TIMER) == 10991917);
    assert!(
        at(f::DECREMENT_FUNCTION_TIMER) == 10991935 && at(f::FOOT_FIRST_PHASE_THINKING) == 10991968
    );
    assert!(at(f::PROCESS_SCRIPT_AND_TIMER) == 10991973 && at(f::FOOT_IDLE) == 10992173);
    assert!(
        at(f::FOOT_SECOND_PHASE_THINKING) == 10992174 && at(f::FOOT_WALK_BACKWARDS) == 10992453
    );
    assert!(at(f::FOOT_WALK_TO_START) == 10992494 && at(f::FOOT_SECOND_PHASE_INIT) == 10992548);
    assert!(at(f::FOOT_WALK_FORWARD) == 10992558 && at(f::ATTACK_WITH_MOUTH_OPEN) == 10992618);
    assert!(at(f::FINGERNAIL_INIT) == 10992992 && at(f::FINGERNAIL_FIRE) == 10993294);
    assert!(at(f::FOOT_PREPARE_LUNGE) == 10993453 && at(f::FOOT_LUNGE_FORWARD) == 10993501);
    assert!(
        at(f::FOOT_RETREAT_FROM_LUNGE) == 10993579 && at(f::GETS_BIG_RELEASE_CAMERA) == 10993825
    );
    assert!(at(f::DEATH_INIT) == 10994528 && at(f::DEATH_FADE_OUT) == 10994681);
    assert!(at(f::DEATH_BG2_TOP_HALF) == 10994852 && at(f::DEATH_BG2_BOTTOM_HALF) == 10994888);
    assert!(
        at(f::DEATH_SINK_THROUGH_FLOOR) == 10994999 && at(f::FADE_IN_CLEAR_BG2_TOP) == 10995477
    );
    assert!(
        at(f::FADE_IN_CLEAR_BG2_BOTTOM) == 10995537 && at(f::FADE_IN_LOAD_BG3_TILES_1) == 10995575
    );
    assert!(
        at(f::FADE_IN_LOAD_BG3_TILES_2) == 10995619 && at(f::FADE_IN_LOAD_BG3_TILES_3) == 10995657
    );
    assert!(at(f::FADE_IN_LOAD_BG3_TILES_4) == 10995695 && at(f::FADE_IN_BG6) == 10995733);
    assert!(at(f::FADE_IN_DEAD_WAS_ALIVE) == 10995779 && at(f::FADE_IN_DEAD_WAS_DEAD) == 10995793);
    assert!(at(f::RESTRICT_SAMUS_X) == 10995813 && at(f::RAISE_THROUGH_FLOOR) == 10995819);
    assert!(
        at(f::RAISE_LOAD_TILEMAP_BOTTOM) == 10995866 && at(f::RAISE_QUAKE_EVERY_16) == 10995936
    );
    assert!(at(f::RAISE_QUAKE_EVERY_8) == 10995970 && at(f::RAISE) == 10996004);
};

// The named instruction lists, spritemaps and scripts are the decompilation's addresses.
const _: () = {
    use crate::game::sm_a7::*;
    use sm_game::actors::kraid::animation as a;
    assert!(
        a::FOOT_IDLE == addr_kKraid_Ilist_86E7 as u16
            && a::FOOT_STAND == addr_kKraid_Ilist_86ED as u16
    );
    assert!(
        a::FOOT_WALK_FORWARD == addr_kKraid_Ilist_86F3 as u16
            && a::FOOT_WALK_FORWARD_END == addr_off_A787BB as u16
    );
    assert!(
        a::FOOT_LUNGE == addr_kKraid_Ilist_87BD as u16
            && a::FOOT_WALK_BACKWARD_END == addr_off_A78885 as u16
    );
    assert!(
        a::FOOT_WALK_BACKWARD == addr_kKraid_Ilist_8887 as u16
            && a::ARM_IDLE == addr_kKraid_Ilist_89F3 as u16
    );
    assert!(
        a::ARM_SWIPE == addr_kKraid_Ilist_8A41 as u16
            && a::ARM_RAISED == addr_kKraid_Ilist_8AA4 as u16
    );
    assert!(
        a::ARM_DYING == addr_kKraid_Ilist_8AF0 as u16
            && a::LINT_HIDDEN == addr_kKraid_Ilist_8AFE as u16
    );
    assert!(
        a::LINT_SHOWN == addr_kKraid_Ilist_8B04 as u16
            && a::FINGERNAIL == addr_kKraid_Ilist_8B0A as u16
    );
    assert!(
        a::LINT_SPRITEMAP == addr_kKraid_Sprmap_8C6C as u16
            && a::LINT_HIDDEN_SPRITEMAP == addr_kKraid_Sprmap_A5DF as u16
    );
    assert!(a::SCRIPT_IDLE == addr_stru_A796DA as u16 && a::SCRIPT_GLOW == addr_stru_A7974A as u16);
    assert!(a::SCRIPT_DYING == addr_stru_A79764 as u16 && a::ROOM_BG == addr_kKraidsRoomBg as u16);
    assert!(
        a::TILEMAP_0 == addr_kKraidTilemaps_0 as u16
            && a::TILEMAP_1 == addr_kKraidTilemaps_1 as u16
    );
    assert!(
        a::TILEMAP_2 == addr_kKraidTilemaps_2 as u16
            && a::TILEMAP_3 == addr_kKraidTilemaps_3 as u16
    );
    assert!(sm_game::actors::kraid::NO_SHOT_AI == fnnullsub_170_A7 as u16);
};

impl sm_game::actors::kraid::Host for Game {
    fn queue_kraid_tilemap(&mut self, tail: u16, source: u16) {
        // The two fixed tail reads have no intervening effects and select the same entry.
        crate::game::display_refresh::queue_native_transfer(
            tail,
            704,
            |_| source,
            0xa7,
            |game| ((game.ram().read_byte(0x59) as i32 & 0xfc) << 8) as u16,
            self,
        );
    }

    fn touches_player_from_above(&mut self, k: u16) -> bool {
        crate::game::actors::api::touches_player_from_above(
            &crate::game::session_runtime::Runtime::new(self),
            sm_game::actors::access::ActorKey::from_legacy(k),
        )
    }

    fn boss_word_native(&mut self, area: u16) -> u16 {
        crate::game::gameplay_flag_runtime::boss_word_at(
            area,
            sm_game::gameplay_flag_host::BossWordOrder::Native,
            self,
        )
    }

    fn set_boss_word_native(&mut self, area: u16, value: u16) {
        crate::game::gameplay_flag_runtime::set_boss_word_native(area, value, self);
    }

    fn set_shot_ai(&mut self, enemy_ptr: u16, shot_ai: u16) {
        crate::game::sm_a7::set_kraid_shot_ai(enemy_ptr, shot_ai, self);
    }

    fn print_x_undefined(&mut self) {
        crate::game::sm_a7::print_x_undefined();
    }

    #[cfg(sm_verify_dll)]
    fn observe_kraid_flags(&mut self, step: u8) {
        crate::kraid_flags_observer::observe(step, self);
    }
}

// The named bank $A7 functions are the ones `DispatchEnemyPreInstr` dispatches on.
const _: () = {
    use crate::game::sm_a7::*;
    use sm_game::actors::etecoon::function as f;
    const fn at(word: u16) -> i32 {
        0xa7_0000 | word as i32
    }
    assert!(at(f::INITIAL) == fnEtecoon_Func_4 && at(f::START_HOP_BOTTOM) == fnEtecoon_Func_5);
    assert!(at(f::HOPPING_BOTTOM) == fnEtecoon_Func_6 && at(f::LOOK_AT_SAMUS) == fnEtecoon_Func_7);
    assert!(at(f::RUNNING_LEFT) == fnEtecoon_Func_8 && at(f::RUNNING_RIGHT) == fnEtecoon_Func_9);
    assert!(at(f::JUMPING) == fnEtecoon_Func_10 && at(f::WALL_JUMP) == fnEtecoon_Func_11);
    assert!(
        at(f::LANDED_FROM_JUMP) == fnEtecoon_Func_12
            && at(f::RUN_TO_LEFT_RUN_UP) == fnEtecoon_Func_16
    );
    assert!(
        at(f::RUN_TO_RIGHT_RUN_UP) == fnEtecoon_Func_17
            && at(f::RUNNING_FOR_TUNNEL_JUMP) == fnEtecoon_Func_18
    );
    assert!(
        at(f::TUNNEL_JUMP) == fnEtecoon_Func_19
            && at(f::RUNNING_THROUGH_TUNNEL) == fnEtecoon_Func_20
    );
    assert!(
        at(f::FALLING_FROM_TUNNEL_LEDGE) == fnEtecoon_Func_21
            && at(f::HOPPING_TOP) == fnEtecoon_Func_22
    );
    assert!(
        at(f::START_HOP_TOP) == fnEtecoon_Func_23
            && at(f::HOP_UNTIL_SAMUS_NEAR) == fnEtecoon_Func_24
    );
    assert!(
        at(f::RUNNING_FOR_FAILED_TUNNEL_JUMP) == fnEtecoon_Func_25
            && at(f::FAILED_TUNNEL_JUMP) == fnEtecoon_Func_26
    );
};

// The named instruction lists are the decompilation's addresses.
const _: () = {
    use crate::game::sm_a7::*;
    use sm_game::actors::etecoon::instruction_list as l;
    assert!(
        l::LOOK_RIGHT_AND_RUN_LEFT == addr_kEtecoon_Ilist_E81E as u16
            && l::RUNNING_LEFT == addr_stru_A7E828 as u16
    );
    assert!(
        l::WALL_JUMP_LEFT == addr_kEtecoon_Ilist_E83C as u16
            && l::HOPPING_FACING_LEFT == addr_kEtecoon_Ilist_E854 as u16
    );
    assert!(
        l::HIT_CEILING == addr_stru_A7E862 as u16
            && l::WALL_JUMP_LEFT_ELIGIBLE == addr_kEtecoon_Ilist_E870 as u16
    );
    assert!(
        l::LOOK_LEFT_AND_RUN_RIGHT == addr_kEtecoon_Ilist_E876 as u16
            && l::RUNNING_RIGHT == addr_stru_A7E880 as u16
    );
    assert!(
        l::WALL_JUMP_RIGHT == addr_kEtecoon_Ilist_E894 as u16
            && l::JUMPING_RIGHT == addr_stru_A7E898 as u16
    );
    assert!(
        l::HOPPING_FACING_RIGHT == addr_kEtecoon_Ilist_E8AC as u16
            && l::WALL_JUMP_RIGHT_ELIGIBLE == addr_kEtecoon_Ilist_E8C8 as u16
    );
    assert!(
        l::INITIAL == addr_kEtecoon_Ilist_E8CE as u16
            && l::FLEXING == addr_kEtecoon_Ilist_E8D6 as u16
    );
    assert!(l::NOTHING_SPRITEMAP == addr_kSpritemap_Nothing_A7 as u16);
};

impl sm_game::actors::etecoon::Host for Game {
    fn run_function(&mut self, address: u32) {
        crate::game::sm_a7::run_a7_function(address, self);
    }

    fn probe_horizontal(&mut self, k: u16, amount: i32) -> bool {
        crate::game::actors::api::probe_horizontal(
            &mut crate::game::session_runtime::Runtime::new(self),
            sm_game::actors::access::ActorKey::from_legacy(k),
            amount,
        ) == crate::game::actors::api::MovementResult::Blocked
    }

    fn direction_to_player(&mut self) -> sm_game::actors::api::PlayerDirection {
        crate::game::actors::api::direction_to_player(
            &mut crate::game::session_runtime::Runtime::new(self),
        )
    }
}
