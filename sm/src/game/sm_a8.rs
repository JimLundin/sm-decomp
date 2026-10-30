use crate::game::actors;
use crate::game::session_runtime::Runtime;
use crate::game_state::Game;

// Native diagnostic retained in the bank under the current migration policy.
pub(crate) fn warn_lava_height(value: i32) {
    {
        (unsafe { crate::c_library::printf(c"Y undefined %d!\n".as_ptr(), value) });
    }
}
crate::game::callback_dispatch! {
    pub(crate) fn DispatchEnemyAi(ea: u32, game: &mut Game) => ea;
    call (game); default { crate::rtl::Unreachable(game); };
    @forward [pub fn (game: &mut Game)];
    [11042826] Enemy_GrappleReact_KillEnemy_A8 => actors::combat::die(&mut Runtime::new(game), actors::api::DeathKind::Grapple);
    [11042831] Enemy_GrappleReact_CancelBeam_A8 => actors::combat::select_frozen_ai(&mut Runtime::new(game));
    [11042846] Enemy_GrappleReact_HurtSamus_A8 => actors::combat::select_frozen_ai(&mut Runtime::new(game));
    [11042851] Enemy_NormalTouchAI_A8 => actors::combat::contact_reaction(&mut Runtime::new(game), actors::api::DeathHandling::Finalize);
    [11042861] Enemy_NormalShotAI_A8 => actors::combat::shot_reaction(&mut Runtime::new(game), actors::api::ShotHandling::Finalize);
    [11042881] Enemy_NormalFrozenAI_A8 => actors::scheduling::update_frozen_enemy(&mut Runtime::new(game));
}
