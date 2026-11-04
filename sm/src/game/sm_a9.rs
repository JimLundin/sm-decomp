use crate::game::actors;
pub use crate::game::prelude::*;
use crate::game::session_runtime::Runtime;
use sm_game::actors::corpses;
use sm_game::actors::mother_brain_fight;
use sm_game::actors::{baby_metroid_cutscene, baby_metroid_encounter};
use sm_game::enemy_script::EnemyScript;

pub use crate::game::constants::bank_8d::*;
pub use crate::game::constants::bank_86::*;
pub use crate::game::constants::bank_a6::*;
pub use crate::game::constants::bank_a9::*;

use sm_game::projectile_geometry::SINE_COSINE as kSinCosTable8bit_Sext;

pub use crate::types::ConstsB7;

/// Mother Brain's fight runs in `sm_game::actors::mother_brain_fight`.
pub use sm_game::actors::mother_brain_fight::*;
/// Kept unsafe for the player collision ownership checks, which wrap it in an unsafe block.
pub unsafe fn mother_brain_move_samus_vertically_towards_ceiling_floor(
    a: u16,
    game: &mut Game,
) -> u8 {
    mother_brain_fight::move_samus_towards_ceiling_floor(a, game)
}
/// Kept unsafe for the player collision ownership checks, which wrap it in an unsafe block.
pub unsafe fn mother_brain_move_samus_horiz_towards_wall(a: u16, game: &mut Game) -> u8 {
    mother_brain_fight::move_samus_towards_wall(a, game)
}
crate::forward_calls! { pub fn (a: u16, r18: u16, game: &mut Game) -> u16;
    sin_times => sin_angle_times(r18, a, game);
    cos_times => sin_angle_times((r18).wrapping_add(64), a, game);
}
pub fn sin_angle_times(a: u16, j: u16, game: &mut Game) -> u16 {
    {
        let v2: u16 = kSinCosTable8bit_Sext[((a as i32 & 0xff) + 64) as usize] as u16;
        // Hardware multiply through the PPU's Mode 7 registers.
        unsafe {
            crate::rtl::register_writes! { game;
                byte(M7A as u16, v2 as u8); byte(M7A as u16, v2.to_ne_bytes()[1]);
                byte(M7B as u16, j as u8);
            }
            read_reg_word(MPYM as u16, game)
        }
    }
}

crate::game::callback_dispatch! {
    pub(crate) fn dispatch_enemy_ai(ea: u32, game: &mut Game) => ea;
    call (game); default { crate::rtl::report_unreachable(game); };
    @forward [pub fn (game: &mut Game)];
    [11108367] enemy_grapple_react_cancel_beam_a9 => actors::combat::select_frozen_ai(&mut Runtime::new(game));
    [11108417] enemy_normal_frozen_ai_a9 => actors::scheduling::update_frozen_enemy(&mut Runtime::new(game));
}

/// Bank $A9's instructions still on the game: Mother Brain's body's.
pub(crate) fn dispatch_enemy_instr(ea: u32, j: EnemyScript, game: &mut Game) -> EnemyScript {
    match crate::game::mother_brain_body::dispatch_instruction(ea, j, game) {
        Some(script) => script,
        None => {
            crate::rtl::report_unreachable(game);
            EnemyScript::Stop
        }
    }
}

impl baby_metroid_encounter::Lease for Runtime<'_> {
    fn wait_for_camera(&mut self) {
        self.owned_call(sm_game::actors::baby_metroid::wait_for_camera);
    }
}

impl Runtime<'_> {
    /// Bank $A9's AI, reactions and draw hooks that run on this lease: Mother
    /// Brain's, the tubes', the baby Metroid's and the corpses'; false when
    /// `routine` is not one of them.
    pub(crate) fn bank_a9_routine(&mut self, routine: u32) -> bool {
        let action: fn(&mut Self) = match routine {
            11110023 => mother_brain_fight::initialize_body,
            11110149 => mother_brain_fight::initialize_brain,
            11110206 => mother_brain_fight::body_main_ai,
            11110279 => mother_brain_fight::power_bomb_reaction,
            11110283 => mother_brain_fight::brain_main_ai,
            11111221 => sm_game::actors::mother_brain_tubes::initialize,
            11110345 => mother_brain_fight::draw_brain_and_neck_hook,
            11110365 => mother_brain_fight::draw_brain_hook,
            11121923 => mother_brain_fight::body_shot,
            11121927 => mother_brain_fight::brain_shot,
            11122118 => mother_brain_fight::brain_touch,
            11126544 => baby_metroid_cutscene::initialize,
            11126649 => baby_metroid_cutscene::main_ai,
            11128579 => baby_metroid_cutscene::touch,
            11136823 => baby_metroid_encounter::initialize,
            11136954 => baby_metroid_encounter::power_bomb_reaction,
            11136965 => baby_metroid_encounter::main_ai,
            11138953 => baby_metroid_encounter::touch,
            11139138 => baby_metroid_encounter::shot,
            11129608 => corpses::initialize_torizo_corpse,
            11129704 => corpses::torizo_corpse_main_ai,
            11129754 => corpses::torizo_corpse_graphics_drawn_hook,
            11129898 => corpses::torizo_corpse_power_bomb_reaction,
            11129907 => corpses::torizo_corpse_touch_or_shot,
            11130806 => corpses::initialize_sidehopper_corpse,
            11130953 => corpses::initialize_geemer_corpse,
            11130998 => corpses::initialize_ripper_corpse,
            11131039 => corpses::initialize_skree_corpse,
            11131084 => corpses::sidehopper_corpse_power_bomb_reaction,
            11131099 => corpses::corpse_main_ai,
            11132141 => corpses::power_bomb_reaction_geemer_corpse,
            11132152 => corpses::touch_or_shot_geemer_corpse,
            11132157 => corpses::ripper_corpse_power_bomb_reaction,
            11132168 => corpses::ripper_corpse_touch_or_shot,
            11132173 => corpses::skree_corpse_power_bomb_reaction,
            11132184 => corpses::skree_corpse_touch_or_shot,
            11132189 => corpses::sidehopper_corpse_shot,
            11132228 => corpses::sidehopper_corpse_touch,
            _ => return false,
        };
        self.owned_call(action);
        true
    }

    /// Bank $A9's instructions that run on this lease, for `actor` at `script`;
    /// the script back, unrun, when `routine` is not one of them.
    pub(crate) fn bank_a9_instruction(
        &mut self,
        routine: u32,
        actor: u16,
        script: EnemyScript,
    ) -> Result<EnemyScript, EnemyScript> {
        let instruction: fn(u16, EnemyScript, &mut Self) -> EnemyScript = match routine {
            11115208 => mother_brain_fight::spawn_dust_cloud_at_offset,
            11115247 => mother_brain_fight::spawn_hand_beam_projectile,
            11115269 => mother_brain_fight::increment_hand_beam_attack_phase,
            11139476 => baby_metroid_encounter::branch_or_queue_remorse_sound,
            11136208 => corpses::sidehopper_corpse_end_hop,
            _ => return Err(script),
        };
        Ok(self.owned_call(|runtime| instruction(actor, script, runtime)))
    }
}

// On the game the Session-owned words are their RAM words, read and stored
// as word! does.
impl sm_game::sprite_drawing::DisplayObjects for Game {}

impl sm_game::actors::mother_brain_fight::OwnedWords for Game {
    fn owned(&self, word: sm_game::actors::mother_brain_fight::OwnedWord) -> u16 {
        let at = word.address();
        u16::from_le_bytes([self.compatibility_byte(at), self.compatibility_byte(at + 1)])
    }
    fn set_owned(&mut self, word: sm_game::actors::mother_brain_fight::OwnedWord, value: u16) {
        let at = word.address();
        let [low, high] = value.to_le_bytes();
        self.compatibility_write_byte(at, low);
        self.compatibility_write_byte(at + 1, high);
    }
}

impl sm_game::actors::mother_brain_fight::Host for Game {
    type Lease<'a> = crate::game::session_runtime::Runtime<'a>;
    fn lease(&mut self) -> crate::game::session_runtime::Runtime<'_> {
        crate::game::session_runtime::Runtime::new(self)
    }
}
