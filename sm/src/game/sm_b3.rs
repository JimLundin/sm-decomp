use crate::game::actors;
pub use crate::game::prelude::*;

pub use crate::types::Consts_B3;

crate::game::constants::typed_constants! { Consts_B3;
    addr_kUnusedSpinningTurtleEye_Ilist_86A7 = 34471,
}
crate::ram_map::ram_pointer_functions! {
    gEnemyData(o: i32) -> EnemyData = [0xf78, o as isize];
}
pub fn UnusedSpinningTurtleEye_Main() {}
crate::game::callback_dispatch! {
    #[allow(unused_unsafe)]
    pub unsafe fn CallBugInstr(ea: u32, _k: u16, game: &mut Game) => ea;
    call (game); default { Unreachable(game); };
    @external {
        11766733 => actors::norfair_pipe_bug::await_group;
        11766783 => actors::norfair_pipe_bug::await_player;
        11766950 => actors::norfair_pipe_bug::rise;
        11767039 => actors::norfair_pipe_bug::settle;
        11767052 => [actors::norfair_pipe_bug::move_to_limit(-16, game)];
        11767118 => [actors::norfair_pipe_bug::move_to_limit(-32, game)];
        11767184 => [actors::norfair_pipe_bug::move_to_limit(16, game)];
        11767250 => [actors::norfair_pipe_bug::move_to_limit(32, game)];
        11767316 => [actors::norfair_pipe_bug::fly(2, game)];
        11767349 => [actors::norfair_pipe_bug::fly(0, game)];
        11767386 => actors::norfair_pipe_bug::start_flight;
        11767733 => actors::yellow_pipe_bug::detect_player;
        11767797 => [actors::yellow_pipe_bug::launch_delay(word!(game.cur_enemy_index), game)];
        11767848 => [actors::yellow_pipe_bug::fly(true, game)];
        11767997 => [actors::yellow_pipe_bug::fly(false, game)];
        11768154 => [actors::yellow_pipe_bug::turn(true, game)];
        11768280 => [actors::yellow_pipe_bug::turn(false, game)];
    }
}

crate::game::callback_dispatch! {
    pub(crate) unsafe fn DispatchEnemyAi(ea: u32, game: &mut Game) => ea;
    call (game); default { crate::rtl::Unreachable(game); };
    @functions [pub unsafe fn (game: &mut Game)];
    [11765499] UnusedSpinningTurtleEye_Init [] {
        let v0 = gEnemyData(word!(game.cur_enemy_index) as i32, game);
        (*v0).properties = ((*v0).properties) | ((kEnemyProps_DisableSamusColl) as u16);
        (*v0).current_instruction = addr_kUnusedSpinningTurtleEye_Ilist_86A7 as u16;
    };

}
