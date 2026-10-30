//! Game's implementations of the sm_game host traits whose modules moved.
#![forbid(unsafe_code)]
use crate::game_state::Game;
use sm_game::inventory::{self, Word};
use sm_game::player_position::{PlayerPosition, PlayerWord};

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

impl sm_game::hdma_program::LeaseHdma for Game {
    type Lease<'a> = crate::game::session_runtime::Runtime<'a>;
    fn hdma_lease(&mut self) -> Self::Lease<'_> {
        crate::game::session_runtime::Runtime::new(self)
    }
}

impl sm_game::hdma_program::HdmaLease for crate::game::session_runtime::Runtime<'_> {
    type Game = Game;
    fn foreign<T>(&mut self, action: impl FnOnce(&mut Game) -> T) -> T {
        crate::game::session_runtime::Runtime::foreign(self, action)
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
        crate::game::gameplay_flag_runtime::event_has_happened(event, self)
    }
    fn set_event(&mut self, event: u16) {
        crate::game::gameplay_flag_runtime::mark_event(event, self);
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

/// The compat game reads and stores Samus's words as the named word accesses
/// did: through the projected bytes, low then high.
impl PlayerPosition for Game {
    fn player_word(&self, word: PlayerWord) -> u16 {
        sm_game::projection::word(self, word.address())
    }
    fn set_player_word(&mut self, word: PlayerWord, value: u16) {
        sm_game::projection::set_word(self, word.address(), value)
    }
}

/// A current lease reads Samus's words from `Session::player`, a dirty one the
/// projected word; stores take the lease's byte path (refresh, epoch, alias).
impl PlayerPosition for crate::game::session_runtime::Runtime<'_> {
    fn player_word(&self, word: PlayerWord) -> u16 {
        self.owned_player(|player| word.get(player), word.address())
    }
    fn set_player_word(&mut self, word: PlayerWord, value: u16) {
        self.write_word_le(value, word.address())
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
        tubes::DUST_CLOUD == sm_game::constants::bank_86::ADDR_K_EPROJ_DUST_CLOUD_EXPLOSION as u16
    );
};

impl sm_game::actors::mother_brain_tubes::Host for Game {
    fn end_screen_flash(&mut self) {
        crate::game::sm_a9::end_screen_flashing(self);
    }

    fn start_earthquake(&mut self, kind: u16) {
        crate::game::mother_brain_visuals::start_earthquake(kind, self);
    }
}

// The phase-3 neck functions are the ones `dispatch_neck_function` dispatched on.
const _: () = {
    use crate::game::sm_a9::*;
    use sm_game::actors::mother_brain_neck::{animation as a, function as f};
    const fn at(word: u16) -> u32 {
        0xa9_0000 | word as u32
    }
    assert!(at(f::NORMAL) == 11125552 && at(f::IDLE) == 11125587);
    assert!(f::IDLE == ADDR_LOCRET_A9C353 as u16);
    assert!(at(f::SETUP_RECOIL_RECOVERY) == 11125588 && at(f::RECOIL_RECOVERY) == 11125627);
    assert!(at(f::SETUP_HYPER_BEAM_RECOIL) == 11125653 && at(f::HYPER_BEAM_RECOIL) == 11125709);
    assert!(a::RECOVER == ADDR_K_MOTHER_BRAIN_ILIST_9DBB as u16);
    assert!(a::HYPER_BEAM_HIT == ADDR_K_MOTHER_BRAIN_ILIST_9BE7 as u16);
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
    assert!(
        at(f::DEAD) == 11000637
            && f::NOTHING == crate::game::constants::bank_a7::FNNULLSUB_237 as u16
    );
};

// The named instruction lists are the decompilation's `kKraid_Ilist_CCxx`.
const _: () = {
    use crate::game::constants::bank_a7::*;
    use sm_game::actors::phantoon::animation as a;
    assert!(
        a::BODY_INVULNERABLE == ADDR_K_KRAID_ILIST_CC41 as u16
            && a::BODY_FULL_HITBOX == ADDR_K_KRAID_ILIST_CC47 as u16
    );
    assert!(
        a::BODY_EYE_HITBOX == ADDR_K_KRAID_ILIST_CC4D as u16
            && a::EYE_OPEN == ADDR_K_KRAID_ILIST_CC53 as u16
    );
    assert!(
        a::EYE_CLOSE_AND_PICK_PATTERN == ADDR_K_KRAID_ILIST_CC81 as u16
            && a::EYE_CLOSE == ADDR_K_KRAID_ILIST_CC91 as u16
    );
    assert!(
        a::EYEBALL_CENTRED == ADDR_K_KRAID_ILIST_CC9D as u16
            && a::NOTHING_SPRITEMAP == ADDR_K_SPRITEMAP_NOTHING_A7 as u16
    );
};

impl sm_game::actors::phantoon::Host for Game {
    fn run_function(&mut self, address: u32) {
        crate::game::session_runtime::run_a7_function(address, self);
    }

    fn direction_to_player(&mut self) -> sm_game::actors::api::PlayerDirection {
        crate::game::actors::api::direction_to_player(
            &mut crate::game::session_runtime::Runtime::new(self),
        )
    }

    fn spawn_hdma(&mut self, bank: u8, args: &sm_game::display::hdma_spawn::HdmaObjectDefinition) {
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
    use crate::game::constants::bank_a7::*;
    use sm_game::actors::kraid::animation as a;
    assert!(
        a::FOOT_IDLE == ADDR_K_KRAID_ILIST_86E7 as u16
            && a::FOOT_STAND == ADDR_K_KRAID_ILIST_86ED as u16
    );
    assert!(
        a::FOOT_WALK_FORWARD == ADDR_K_KRAID_ILIST_86F3 as u16
            && a::FOOT_WALK_FORWARD_END == ADDR_OFF_A787BB as u16
    );
    assert!(
        a::FOOT_LUNGE == ADDR_K_KRAID_ILIST_87BD as u16
            && a::FOOT_WALK_BACKWARD_END == ADDR_OFF_A78885 as u16
    );
    assert!(
        a::FOOT_WALK_BACKWARD == ADDR_K_KRAID_ILIST_8887 as u16
            && a::ARM_IDLE == ADDR_K_KRAID_ILIST_89F3 as u16
    );
    assert!(
        a::ARM_SWIPE == ADDR_K_KRAID_ILIST_8A41 as u16
            && a::ARM_RAISED == ADDR_K_KRAID_ILIST_8AA4 as u16
    );
    assert!(
        a::ARM_DYING == ADDR_K_KRAID_ILIST_8AF0 as u16
            && a::LINT_HIDDEN == ADDR_K_KRAID_ILIST_8AFE as u16
    );
    assert!(
        a::LINT_SHOWN == ADDR_K_KRAID_ILIST_8B04 as u16
            && a::FINGERNAIL == ADDR_K_KRAID_ILIST_8B0A as u16
    );
    assert!(
        a::LINT_SPRITEMAP == ADDR_K_KRAID_SPRMAP_8C6C as u16
            && a::LINT_HIDDEN_SPRITEMAP == ADDR_K_KRAID_SPRMAP_A5DF as u16
    );
    assert!(a::SCRIPT_IDLE == ADDR_STRU_A796DA as u16 && a::SCRIPT_GLOW == ADDR_STRU_A7974A as u16);
    assert!(
        a::SCRIPT_DYING == ADDR_STRU_A79764 as u16 && a::ROOM_BG == ADDR_K_KRAIDS_ROOM_BG as u16
    );
    assert!(
        a::TILEMAP_0 == ADDR_K_KRAID_TILEMAPS_0 as u16
            && a::TILEMAP_1 == ADDR_K_KRAID_TILEMAPS_1 as u16
    );
    assert!(
        a::TILEMAP_2 == ADDR_K_KRAID_TILEMAPS_2 as u16
            && a::TILEMAP_3 == ADDR_K_KRAID_TILEMAPS_3 as u16
    );
    assert!(sm_game::actors::kraid::NO_SHOT_AI == FNNULLSUB_170_A7 as u16);
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
        crate::game::session_runtime::set_kraid_shot_ai(enemy_ptr, shot_ai, self);
    }

    fn print_x_undefined(&mut self) {
        crate::game::session_runtime::print_x_undefined();
    }

    #[cfg(sm_verify_dll)]
    fn observe_kraid_flags(&mut self, step: u8) {
        crate::kraid_flags_observer::observe(step, self);
    }
}

// The named bank $A7 functions are the ones `DispatchEnemyPreInstr` dispatches on.
const _: () = {
    use crate::game::constants::bank_a7::*;
    use sm_game::actors::etecoon::function as f;
    const fn at(word: u16) -> i32 {
        0xa7_0000 | word as i32
    }
    assert!(at(f::INITIAL) == FN_ETECOON_FUNC_4 && at(f::START_HOP_BOTTOM) == FN_ETECOON_FUNC_5);
    assert!(
        at(f::HOPPING_BOTTOM) == FN_ETECOON_FUNC_6 && at(f::LOOK_AT_SAMUS) == FN_ETECOON_FUNC_7
    );
    assert!(at(f::RUNNING_LEFT) == FN_ETECOON_FUNC_8 && at(f::RUNNING_RIGHT) == FN_ETECOON_FUNC_9);
    assert!(at(f::JUMPING) == FN_ETECOON_FUNC_10 && at(f::WALL_JUMP) == FN_ETECOON_FUNC_11);
    assert!(
        at(f::LANDED_FROM_JUMP) == FN_ETECOON_FUNC_12
            && at(f::RUN_TO_LEFT_RUN_UP) == FN_ETECOON_FUNC_16
    );
    assert!(
        at(f::RUN_TO_RIGHT_RUN_UP) == FN_ETECOON_FUNC_17
            && at(f::RUNNING_FOR_TUNNEL_JUMP) == FN_ETECOON_FUNC_18
    );
    assert!(
        at(f::TUNNEL_JUMP) == FN_ETECOON_FUNC_19
            && at(f::RUNNING_THROUGH_TUNNEL) == FN_ETECOON_FUNC_20
    );
    assert!(
        at(f::FALLING_FROM_TUNNEL_LEDGE) == FN_ETECOON_FUNC_21
            && at(f::HOPPING_TOP) == FN_ETECOON_FUNC_22
    );
    assert!(
        at(f::START_HOP_TOP) == FN_ETECOON_FUNC_23
            && at(f::HOP_UNTIL_SAMUS_NEAR) == FN_ETECOON_FUNC_24
    );
    assert!(
        at(f::RUNNING_FOR_FAILED_TUNNEL_JUMP) == FN_ETECOON_FUNC_25
            && at(f::FAILED_TUNNEL_JUMP) == FN_ETECOON_FUNC_26
    );
};

// The named instruction lists are the decompilation's addresses.
const _: () = {
    use crate::game::constants::bank_a7::*;
    use sm_game::actors::etecoon::instruction_list as l;
    assert!(
        l::LOOK_RIGHT_AND_RUN_LEFT == ADDR_K_ETECOON_ILIST_E81E as u16
            && l::RUNNING_LEFT == ADDR_STRU_A7E828 as u16
    );
    assert!(
        l::WALL_JUMP_LEFT == ADDR_K_ETECOON_ILIST_E83C as u16
            && l::HOPPING_FACING_LEFT == ADDR_K_ETECOON_ILIST_E854 as u16
    );
    assert!(
        l::HIT_CEILING == ADDR_STRU_A7E862 as u16
            && l::WALL_JUMP_LEFT_ELIGIBLE == ADDR_K_ETECOON_ILIST_E870 as u16
    );
    assert!(
        l::LOOK_LEFT_AND_RUN_RIGHT == ADDR_K_ETECOON_ILIST_E876 as u16
            && l::RUNNING_RIGHT == ADDR_STRU_A7E880 as u16
    );
    assert!(
        l::WALL_JUMP_RIGHT == ADDR_K_ETECOON_ILIST_E894 as u16
            && l::JUMPING_RIGHT == ADDR_STRU_A7E898 as u16
    );
    assert!(
        l::HOPPING_FACING_RIGHT == ADDR_K_ETECOON_ILIST_E8AC as u16
            && l::WALL_JUMP_RIGHT_ELIGIBLE == ADDR_K_ETECOON_ILIST_E8C8 as u16
    );
    assert!(
        l::INITIAL == ADDR_K_ETECOON_ILIST_E8CE as u16
            && l::FLEXING == ADDR_K_ETECOON_ILIST_E8D6 as u16
    );
    assert!(l::NOTHING_SPRITEMAP == ADDR_K_SPRITEMAP_NOTHING_A7 as u16);
};

impl sm_game::actors::etecoon::Host for Game {
    fn run_function(&mut self, address: u32) {
        crate::game::session_runtime::run_a7_function(address, self);
    }

    fn probe_horizontal(&mut self, k: u16, amount: i32) -> bool {
        crate::game::actors::api::probe_horizontal(
            &mut crate::game::session_runtime::Runtime::new(self),
            sm_game::actors::access::ActorKey::from_legacy(k),
            amount,
        ) == crate::game::actors::api::MovementResult::Blocked
    }
}

// The named bank $A7 functions are the ones `DispatchEnemyPreInstr` dispatches on.
const _: () = {
    use crate::game::constants::bank_a7::*;
    use sm_game::actors::dachora_run::function as f;
    const fn at(word: u16) -> i32 {
        0xa7_0000 | word as i32
    }
    assert!(
        at(f::WAIT_FOR_SAMUS) == FN_DACHORA_FUNC_2 && at(f::START_RUNNING) == FN_DACHORA_FUNC_3
    );
    assert!(at(f::RUNNING_LEFT) == FN_DACHORA_FUNC_4 && at(f::RUNNING_RIGHT) == FN_DACHORA_FUNC_5);
    assert!(
        at(f::ACTIVATE_SHINESPARK) == FN_DACHORA_FUNC_7
            && at(f::SHINESPARKING) == FN_DACHORA_FUNC_8
    );
    assert!(at(f::FALLING) == FN_DACHORA_FUNC_11 && at(f::ECHO) == FN_DACHORA_FUNC_12);
};

// The named instruction lists are the decompilation's addresses.
const _: () = {
    use crate::game::constants::bank_a7::*;
    use sm_game::actors::dachora_run::instruction_list as l;
    assert!(
        l::RUNNING_LEFT == ADDR_K_DACHORA_ILIST_F345 as u16
            && l::IDLING_FACING_LEFT == ADDR_K_DACHORA_ILIST_F399 as u16
    );
    assert!(
        l::BLINKING_FACING_LEFT == ADDR_K_DACHORA_ILIST_F3C9 as u16
            && l::ECHO_FACING_LEFT == ADDR_K_DACHORA_ILIST_F3F7 as u16
    );
    assert!(
        l::FALLING_FACING_LEFT == ADDR_K_DACHORA_ILIST_F3FF as u16
            && l::RUNNING_RIGHT == ADDR_K_DACHORA_ILIST_F407 as u16
    );
    assert!(
        l::IDLING_FACING_RIGHT == ADDR_K_DACHORA_ILIST_F45B as u16
            && l::BLINKING_FACING_RIGHT == ADDR_K_DACHORA_ILIST_F48B as u16
    );
    assert!(
        l::CHARGE_SHINESPARK_FACING_RIGHT == ADDR_K_DACHORA_ILIST_F4B3 as u16
            && l::ECHO_FACING_RIGHT == ADDR_K_DACHORA_ILIST_F4B9 as u16
    );
    assert!(
        l::FALLING_FACING_RIGHT == ADDR_K_DACHORA_ILIST_F4C1 as u16
            && l::PALETTE == ADDR_K_DACHORA_PALETTE as u16
    );
    assert!(l::NOTHING_SPRITEMAP == ADDR_K_SPRITEMAP_NOTHING_A7 as u16);
};

impl sm_game::actors::dachora_run::Host for Game {
    fn run_function(&mut self, address: u32) {
        crate::game::session_runtime::run_a7_function(address, self);
    }
}

// The named bank $A6 functions are the decompilation's; the two RTS
// terminators are its locret labels.
const _: () = {
    use sm_game::actors::ridley::{baby_metroid_function as b, function as f};
    const fn at(word: u16) -> u32 {
        0xa6_0000 | word as u32
    }
    assert!(at(f::LIFTOFF) == 10924787 && at(f::DECIDE_ACTION) == 10924833);
    assert!(at(f::LUNGE_MISSED) == 10925036 && at(f::LUNGE_MISSED_MOVE_TO_POSITION) == 10925048);
    assert!(at(f::SWOOP) == 10925121 && at(f::SWOOP_MOVE_TO_POSITION) == 10925141);
    assert!(at(f::SWOOP_AIMING_DOWN) == 10925203 && at(f::SWOOP_AIMING_HORIZONTAL) == 10925265);
    assert!(at(f::SWOOP_AIMING_UP) == 10925334 && at(f::SWOOP_AIMING_UP_FASTER) == 10925396);
    assert!(at(f::SWOOP_DECELERATE) == 10925460 && at(f::HOVER) == 10925508);
    assert!(at(f::HOVER_TO_WALL_BEHIND) == 10925541 && at(f::HOVER_TO_WALL_IN_FRONT) == 10925587);
    assert!(at(f::POGO_FLY_TO_POSITION) == 10925735 && at(f::POGO_WAIT_AT_POSITION) == 10925789);
    assert!(at(f::POGO_DESCENDING) == 10925838 && at(f::POGO_ASCENDING) == 10926009);
    assert!(at(f::LUNGE) == 10926775 && at(f::GRABBED_SAMUS) == 10926991);
    assert!(at(f::GRABBED_SAMUS_RISING) == 10927044);
    assert!(at(f::GRABBED_SAMUS_RISING_TO_MIDDLE) == 10927089);
    assert!(at(f::GRABBED_SAMUS_RELEASE) == 10927150 && at(f::DODGE_POWER_BOMB) == 10927438);
    assert!(at(f::FAKE_RETREAT_MOVE_TO_POSITION) == 10927514);
    assert!(at(f::FAKE_RETREAT_RISING) == 10927548 && at(f::DROP_BABY_METROID) == 10927602);
    assert!(at(f::RETRIEVE_BABY_METROID) == 10927619 && at(b::HELD) == 10927772);
    assert!(at(b::HELD_BY_FEET) == 10927795 && at(b::START_FALLING) == 10927818);
    assert!(at(b::FALLING) == 10927836 && at(f::DYING_GRABBED_SAMUS) == 10929464);
    assert!(at(f::DEATH_ROAR) == 10929470 && at(f::DEATH_MOVE_TO_DEATH_SPOT) == 10929489);
    assert!(at(f::LOWER_ACID_AND_RELEASE_SAMUS) == 10929544);
    assert!(at(f::START_FALLING_APART) == 10929576 && at(f::FALLING_APART) == 10929608);
    assert!(at(f::STILL_FALLING_APART) == 10929626);
    assert!(
        at(b::NOTHING) == 10927897
            && f::DEAD == crate::game::constants::bank_a6::ADDR_LOCRET_A6C600 as u16
    );
    assert!(at(f::CERES_LIFTOFF_ACCELERATING) == 10921647);
    assert!(at(f::CERES_REAL_RETREAT_RISING) == 10922353);
};

// The named instruction lists and tables are the decompilation's addresses.
const _: () = {
    use crate::game::constants::bank_a6::*;
    use sm_game::actors::ridley::{
        actions, explosion_instruction_list as e, instruction_list as l, tail_data as t,
    };
    assert!(
        l::RETRIEVE_BABY_METROID == ADDR_K_RIDLEY_ILIST_E658 as u16
            && l::DEATH_ROAR == ADDR_K_RIDLEY_ILIST_E6C8 as u16
    );
    assert!(
        l::TURN_TO_FACE_RIGHT == ADDR_K_RIDLEY_ILIST_E6F0 as u16
            && l::TURN_TO_FACE_LEFT == ADDR_K_RIDLEY_ILIST_E706 as u16
    );
    assert!(l::SHOOT_FIREBALLS == ADDR_K_RIDLEY_ILIST_E73A as u16);
    assert!(
        e::TAIL_SEGMENTS_0_1 == ADDR_K_RIDLEYS_EXPLOSION_ILIST_CA47 as u16
            && e::TAIL_SEGMENTS_2_3 == ADDR_K_RIDLEYS_EXPLOSION_ILIST_CA4D as u16
    );
    assert!(e::TAIL_SEGMENTS_4_5 == ADDR_K_RIDLEYS_EXPLOSION_ILIST_CA53 as u16);
    assert!(
        actions::POGO_OR_SWOOP == ADDR_OFF_A6B38C as u16
            && actions::SWOOP_OR_POGO == ADDR_OFF_A6B39C as u16
    );
    assert!(
        actions::LUNGE_OR_POGO == ADDR_OFF_A6B3AC as u16 && actions::POGO == ADDR_OFF_A6B3BC as u16
    );
    assert!(actions::HOVER == ADDR_OFF_A6B3CC as u16 && actions::LUNGE == ADDR_OFF_A6B3DC as u16);
    assert!(t::DIRECTIONS == ADDR_WORD_A6D36E as u16 && t::DISTANCES == ADDR_WORD_A6D37C as u16);
    assert!(t::ANGLES == ADDR_WORD_A6D38A as u16 && t::X_POSITIONS == ADDR_WORD_A6D398 as u16);
    assert!(t::Y_POSITIONS == ADDR_WORD_A6D3A6 as u16);
};

impl sm_game::actors::ridley::Host for Game {
    fn run_function(&mut self, function: u16) {
        sm_game::actors::ceres_ridley::run_function(self, function);
    }

    fn spawn_population(&mut self, bank: u8, population: u16) -> u16 {
        crate::game::actors::population::spawn_legacy(
            &mut crate::game::session_runtime::Runtime::new(self),
            bank,
            population,
        )
    }

    fn set_boss_bit(&mut self, mask: u16) {
        crate::game::gameplay_flag_runtime::set_boss_bits_for_current_area(mask, self);
    }

    fn hardware_sine(&mut self, amplitude: u16, angle: u16) -> u16 {
        crate::game::sm_a9::ComputeSinMult(amplitude, angle, self)
    }

    fn hardware_cosine(&mut self, amplitude: u16, angle: u16) -> u16 {
        crate::game::sm_a9::ComputeCosMult(amplitude, angle, self)
    }

    fn warn_x_undefined(&mut self) {
        crate::c_library::print(c"Warning: X undefined\n");
    }

    fn move_liquid(&mut self, target: u16, velocity: u16, delay: u16) {
        let mut runtime = crate::game::session_runtime::Runtime::new(self);
        let room = &mut runtime.session_mut().room;
        room.liquid_target = target;
        room.liquid_velocity = velocity;
        room.liquid_delay = delay;
    }
}

// The named Ceres Ridley functions are the decompilation's; the getaway's
// RTS is its locret label.
const _: () = {
    use sm_game::actors::ceres_ridley::ceres_function as f;
    const fn at(word: u16) -> u32 {
        0xa6_0000 | word as u32
    }
    assert!(at(f::TRANSITION_TO_FLYING) == 10920788 && at(f::INITIAL) == 10920795);
    assert!(at(f::WAIT_TO_FADE_IN) == 10920823 && at(f::FADE_IN_EYE) == 10920841);
    assert!(at(f::FADE_IN_BODY) == 10920927 && at(f::WAIT_TO_ROAR) == 10921045);
    assert!(at(f::ROAR_BEFORE_FLYING) == 10921080 && at(f::LIFTOFF_DECELERATING) == 10921672);
    assert!(at(f::HOVERING) == 10921704 && at(f::FIREBALLS_MOVE_TO_POSITION) == 10921858);
    assert!(at(f::FIREBALLS_SHOOTING) == 10921977 && at(f::LUNGE_SETUP) == 10922044);
    assert!(at(f::LUNGE) == 10922062 && at(f::SWOOP_SETUP) == 10922125);
    assert!(at(f::SWOOP_MOVE_TO_POSITION) == 10922148 && at(f::SWOOP_AIMING_DOWN) == 10922196);
    assert!(at(f::SWOOP_AIMING_LEFT) == 10922232 && at(f::SWOOP_AIMING_UP) == 10922275);
    assert!(at(f::SWOOP_AIMING_UP_FASTER) == 10922311);
    assert!(at(f::WRITE_PALETTES_AND_SPAWN_WALLS) == 10922400 && at(f::SET_UP_GETAWAY) == 10922513);
    assert!(at(f::SELF_DESTRUCT_ACTIVATED) == 10922576);
    assert!(at(f::SELF_DESTRUCT_ACTIVATING) == 10928206);
    assert!(at(f::GETAWAY) == 10922575);
};

// The named Ceres instruction lists, palettes and populations are the
// decompilation's addresses.
const _: () = {
    use crate::game::constants::bank_a6::*;
    use sm_game::actors::ceres_ridley::{ceres_data as d, ceres_instruction_list as l};
    assert!(
        l::INITIAL == ADDR_K_RIDLEY_ILIST_E538 as u16
            && l::LUNGE == ADDR_K_RIDLEY_ILIST_E548 as u16
    );
    assert!(
        l::FIGHT_INTRO_ROAR == ADDR_K_RIDLEY_ILIST_E690 as u16
            && l::TRANSITION_TO_FLYING == ADDR_K_RIDLEY_ILIST_E91D as u16
    );
    assert!(l::BABY_METROID == ADDR_K_BABY_METROID_ILIST_BF31 as u16);
    assert!(
        d::CERES_PALETTES == ADDR_WORD_A6E16F as u16
            && d::NORFAIR_PALETTES == ADDR_WORD_A6E1CF as u16
    );
    assert!(
        d::RETREAT_SPRITE_PALETTE == ADDR_WORD_A6A9E3 as u16
            && d::RETREAT_BACKGROUND_PALETTE == ADDR_WORD_A6AA01 as u16
    );
    assert!(
        d::GETAWAY_WALLS[0] == ADDR_STRU_A6AA2F as u16
            && d::GETAWAY_WALLS[1] == ADDR_STRU_A6AA3F as u16
    );
};

impl sm_game::actors::ceres_ridley::Host for Game {
    fn boss_byte(&mut self, area: u16) -> u8 {
        crate::game::gameplay_flag_runtime::boss_byte_at(area, self)
    }

    fn queue_mode7_transfer(&mut self, bank: u8, list: u16) {
        crate::game::mode7_transfers::enqueue_rom(bank, list, self);
    }

    fn begin_ceres_push(&mut self) {
        sm_game::player_movement::forced::begin_ceres_push(
            crate::game::session_runtime::Runtime::new(self).session_mut(),
        );
    }

    fn queue_escape_timer_tiles(&mut self) -> bool {
        crate::game::rom_vram_entries::queue_escape_timer_tiles(self) & 1 != 0
    }

    fn queue_getaway_tiles(&mut self) {
        crate::game::rom_vram_entries::queue_getaway_tiles(self);
    }

    fn queue_escape_tile_table(&mut self) {
        crate::game::rom_vram_entries::queue_escape_tile_table(self);
    }

    fn type_escape_text(&mut self, tiles: u16) -> bool {
        crate::game::escape_typewriter::type_escape_text(tiles, self) & 1 != 0
    }
}

// The fire arc's functions and data are its dispatch IDs and the
// decompilation's addresses.
const _: () = {
    use crate::game::constants::{bank_86::*, bank_a6::*};
    use sm_game::actors::fire_arc::{self as a, function as f};
    assert!((0xa6_0000 | f::INACTIVE as u32) == 10917397);
    assert!((0xa6_0000 | f::ACTIVE as u32) == 10917506);
    assert!(a::INSTRUCTION_LIST == ADDR_K_FIRE_ARC_ILIST_9490 as u16);
    assert!(a::BODY_PROJECTILE == ADDR_K_EPROJ_FIRE_ARC_BODY as u16);
};

impl sm_game::actors::fire_arc::Host for Game {
    fn advance_orbit(&mut self) {
        crate::game::enemy_projectiles::advance_fire_arc_orbit(self);
    }
}

// The door functions, palettes and spritemap are the dispatch IDs and the
// decompilation's addresses; the steam's hidden state is the doors' one.
const _: () = {
    use crate::game::constants::bank_a6::*;
    use sm_game::actors::ceres_doors::{self as c, door_function as f};
    assert!((0xa6_0000 | f::RUMBLING_AND_EXPLOSIONS as u32) == 10942428);
    assert!((0xa6_0000 | f::ELEVATOR_ANIMATIONS as u32) == 10942544);
    assert!(c::DOOR_PALETTE == ADDR_K_CERES_DOOR_PALETTE as u16);
    assert!(c::ESCAPE_DOOR_PALETTE == ADDR_WORD_A6F50C as u16);
    assert!(c::DOOR_SPRITEMAP == ADDR_K_CERES_DOOR_SPRMAP_FAC7 as u16);
    assert!(c::VANISH == crate::game::ceres_doors::TANGIBLE | crate::game::ceres_doors::INVISIBLE);
};

impl sm_game::actors::ceres_doors::Host for Game {
    fn change_properties(&mut self, k: u16, set: u16, clear: u16) {
        crate::game::ceres_doors::change_properties(
            k,
            set,
            clear,
            sm_game::enemy_script::EnemyScript::Stop,
            self,
        );
    }

    fn rom_script(&mut self, bank: u8, address: u16) -> sm_game::enemy_script::EnemyScript {
        crate::game::session_runtime::steam_script(bank, address, self)
    }

    fn project_cinematic_point(&mut self, x: u16, y: u16) -> (u16, u16) {
        let point =
            sm_game::cinematic::project_cinematic_point(sm_game::geometry::Point { x, y }, self);
        (point.x, point.y)
    }

    fn queue_mode7_transfer(&mut self, bank: u8, list: u16) {
        crate::game::mode7_transfers::enqueue_rom(bank, list, self);
    }
}

// The named bank $AA functions and instruction lists are the decompilation's.
const _: () = {
    use crate::game::constants::bank_aa::*;
    use sm_game::actors::statues::{function as f, instruction_list as l};
    const fn at(word: u16) -> i32 {
        0xaa_0000 | word as i32
    }
    assert!(at(f::LOWER_NORFAIR) == 11199557 && at(f::WRECKED_SHIP) == 11200430);
    assert!(at(f::NOTHING) == FNNULLSUB_276);
    assert!(l::LOWER_NORFAIR_ACTIVATED == ADDR_K_SHAKTOOL_ILIST_E3A7 as u16);
    assert!(l::WRECKED_SHIP_ACTIVATED == ADDR_K_SHAKTOOL_ILIST_E461 as u16);
    assert!(l::NOTHING_SPRITEMAP == ADDR_K_SPRITEMAP_NOTHING_AA as u16);
};

impl sm_game::actors::statues::Host for Game {
    fn run_function(&mut self, address: u32) {
        crate::game::session_runtime::run_aa_function(address, self);
    }

    fn boss_word_native(&mut self, area: u16) -> u16 {
        crate::game::gameplay_flag_runtime::boss_word_at(
            area,
            sm_game::gameplay_flag_host::BossWordOrder::Native,
            self,
        )
    }

    fn start_lowering_acid(&mut self) {
        let mut runtime = crate::game::session_runtime::Runtime::new(self);
        let room = &mut runtime.session_mut().room;
        room.liquid_delay = 32;
        room.liquid_velocity = 64;
    }

    fn lower_acid(&mut self) {
        crate::game::room_mechanisms::lower_acid(self);
    }

    fn select_block_at(&mut self, x: u16, y: u16) {
        crate::game::actors::api::select_block_at(
            &mut crate::game::session_runtime::Runtime::new(self),
            sm_game::geometry::Point { x, y },
        );
    }

    fn spawn_collision_plm(&mut self, header: u16) {
        crate::game::session_runtime::spawn_collision_plm(header, self);
    }
}

impl sm_game::lease::Lease for crate::game::session_runtime::Runtime<'_> {
    fn owned_call<R>(&mut self, action: impl FnOnce(&mut Self) -> R) -> R {
        crate::game::session_runtime::Runtime::owned_call(self, action)
    }
    fn refresh_player_observations(&mut self) {
        crate::game::session_runtime::Runtime::refresh_player_observations(self)
    }
    fn owned_player_call<R>(&mut self, action: impl FnOnce(&mut Self) -> R) -> R {
        crate::game::session_runtime::Runtime::owned_player_call(self, action)
    }
    fn has_owned_display(&self) -> bool {
        crate::game::session_runtime::Runtime::has_owned_display(self)
    }
    fn draw_owned_samus(&mut self, index: u16, x: u16, y: u16) {
        let producer = self
            .session()
            .presentation
            .publications()
            .expect("Samus publications")
            .body;
        crate::game::session_runtime::Runtime::draw_owned_samus(self, producer, index, (x, y));
    }
    fn native_samus_sprite(&mut self, index: u16, x: u16, y: u16) {
        use sm_game::sprite_output;
        self.foreign(|game| sprite_output::samus(index, (x, y), game, &mut sprite_output::Native));
    }
    fn native_music_delayed8(&mut self, track: u16) {
        self.foreign(|game| crate::game::music::queue_music_delayed_8(track, game));
    }
    fn native_player_command(&mut self, command: u16) {
        self.foreign(|game| {
            crate::game::session_runtime::player_command(command, game);
        });
    }
    fn native_upload(&mut self, request: sm_game::visual_state::Upload) {
        self.foreign(|game| crate::game::animated_tiles::native::upload(request, game));
    }
    fn native_unreachable(&mut self) {
        self.foreign(|game| {
            crate::rtl::Unreachable(game);
        });
    }
    fn native_animation_unreachable(&mut self) -> u16 {
        self.foreign(crate::game::animated_tiles::native::unreachable)
    }
}

impl sm_game::player_drawing::Host for crate::game::session_runtime::Runtime<'_> {
    fn draw_flare(&mut self) {
        self.foreign(crate::game::player_projectiles::flare);
    }
    fn draw_grapple_flare(&mut self) {
        self.foreign(crate::game::player_projectiles::grapple_flare);
    }
    fn restore_grapple_start_positions(&mut self) {
        self.owned_call(|runtime| sm_game::grapple_swing::restore_start_positions(runtime));
    }
    fn update_grapple_tiles(&mut self) {
        self.foreign(sm_game::grapple_swing::update_tiles);
    }
    fn draw_grapple_rope(&mut self) {
        self.owned_player_call(|runtime| sm_game::grapple_rope::draw(runtime));
    }
    fn draw_projectiles(&mut self) {
        self.foreign(crate::game::player_projectiles::draw_projectiles);
    }
    fn finish_player_frame(&mut self) {
        self.owned_call(crate::game::session_runtime::finish_player_frame);
    }
}

/// Queue the active beam's tiles and write its palette on a fresh lease.
pub(crate) fn update_beam_tiles_and_palette(game: &mut Game) {
    sm_game::player_drawing::update_beam_tiles_and_palette(
        &mut crate::game::session_runtime::Runtime::new(game),
    );
}

/// Draw Samus's frame on a fresh lease, inside the player driver's owned call.
pub(crate) fn draw_player_frame(game: &mut Game) {
    crate::game::session_runtime::Runtime::new(game)
        .owned_player_call(sm_game::player_drawing::draw_player_frame);
}

/// The HUD's values on the game: the inventory view over the projected bytes,
/// and the named words for the HUD items and grapple, read the same way.
impl sm_game::hud::HudPlayer for Game {
    fn health(&self) -> u16 {
        inventory::get(self, Word::Health)
    }
    fn max_health(&self) -> u16 {
        inventory::get(self, Word::MaxHealth)
    }
    fn ammunition(&self, index: usize) -> u16 {
        inventory::get(self, Word::ammunition(index).0)
    }
    fn max_ammunition(&self, index: usize) -> u16 {
        inventory::get(self, Word::ammunition(index).1)
    }
    fn reserve_health(&self) -> u16 {
        inventory::get(self, Word::ReserveHealth)
    }
    fn reserve_mode(&self) -> u16 {
        inventory::get(self, Word::ReserveMode)
    }
    fn equipped_items(&self) -> u16 {
        inventory::get(self, Word::EquippedItems)
    }
    fn selected_item(&self) -> u16 {
        let game = self;
        sm_game::ram_word!(game.HUD_ITEM_INDEX)
    }
    fn automatic_item(&self) -> u16 {
        let game = self;
        sm_game::ram_word!(game.SAMUS_AUTO_CANCEL_HUD_ITEM_INDEX)
    }
    fn grapple_function(&self) -> u16 {
        let game = self;
        sm_game::ram_word!(game.GRAPPLE_BEAM_FUNCTION)
    }
}

impl sm_game::hud::HudFrameHost for Game {
    fn raw_frame_counter_byte(&self) -> u8 {
        self.ram()
            .read_byte(sm_game::ram::bytes::NMI_FRAME_COUNTER_BYTE)
    }
}

/// On a lease the HUD reads Samus's values from the owned Session.
impl sm_game::hud::HudPlayer for crate::game::session_runtime::Runtime<'_> {
    fn health(&self) -> u16 {
        self.owned_session(|s| s.inventory.health.current, 0x9c2)
    }
    fn max_health(&self) -> u16 {
        self.owned_session(|s| s.inventory.health.maximum, 0x9c4)
    }
    fn ammunition(&self, index: usize) -> u16 {
        let at = [0x9c6, 0x9ca, 0x9ce][index];
        self.owned_session(|s| s.inventory.ammunition[index].current, at)
    }
    fn max_ammunition(&self, index: usize) -> u16 {
        let at = [0x9c8, 0x9cc, 0x9d0][index];
        self.owned_session(|s| s.inventory.ammunition[index].maximum, at)
    }
    fn reserve_health(&self) -> u16 {
        self.owned_session(|s| s.inventory.reserve_health, 0x9d6)
    }
    fn reserve_mode(&self) -> u16 {
        self.owned_session(|s| s.inventory.reserve_mode, 0x9c0)
    }
    fn equipped_items(&self) -> u16 {
        self.owned_session(|s| s.inventory.equipment.active, 0x9a2)
    }
    fn selected_item(&self) -> u16 {
        self.owned_session(|s| s.projectiles.controls.selected_weapon, 0x9d2)
    }
    fn automatic_item(&self) -> u16 {
        self.owned_session(|s| s.projectiles.controls.automatic_weapon, 0xa04)
    }
    fn grapple_function(&self) -> u16 {
        self.owned_session(|s| s.player.grapple, 0xd32)
    }
}

impl sm_game::hud::HudFrameHost for crate::game::session_runtime::Runtime<'_> {
    fn raw_frame_counter_byte(&self) -> u8 {
        // As on the game: the raw work-RAM byte, not the Display projection.
        self.samus_game()
            .ram()
            .read_byte(sm_game::ram::bytes::NMI_FRAME_COUNTER_BYTE)
    }
}
