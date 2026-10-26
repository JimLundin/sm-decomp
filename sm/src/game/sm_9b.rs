use crate::game::actors;
use crate::game::combat_rules::GrappleOutcome;
use crate::game::display_refresh::queue_native_transfer;
use crate::game::grapple_swing::{self, advance_swing, change_length, place_rope_origin};
pub use crate::game::prelude::*;
use crate::game::session_runtime::Runtime;
use crate::game::{
    player_collision::operations as player_collision, session_runtime::Runtime as CollisionRuntime,
};

pub use crate::game::constants::bank_9b::*;

use crate::game::grapple_swing::{
    extend_beam as BlockCollGrappleBeam, react_beam as BlockReactGrapple,
};
use crate::game::music::QueueSfx1_Max6;
use crate::game::session_runtime::player_bottom;

crate::game::constants::typed_constants! { i32;
    fnSamus_MoveHandler_ReleaseFromGrapple = 0x90946e, fnSamus_MovementHandler_Normal = 0x90a337,
    fnSamusDrawHandler_Default = 0x90eb52, fnsub_90EB86 = 0x90eb86,
    fnGrappleBeamFunc_Inactive = 0x9bc4f0,
    fnGrappleBeamFunc_Firing = 0x9bc703, fnGrappleBeamFunc_ConnectedLockedInPlace = 0x9bc77e,
    fnGrappleBeamFunc_Cancel = 0x9bc856,
    fnGrappleBeam_Func2 = 0x9bc8c5,
}
pub fn CheckIfGrappleIsConnectedToBlock(game: &mut Game) -> u8 {
    {
        word! { game, 0;
            grapple_beam_extension_x_velocity, grapple_beam_extension_y_velocity,
        }
        if (BlockReactGrapple(game) & 1) == 0 {
            return 0;
        }
        word!(game.samus_grapple_flags = 1);
        1
    }
}

pub fn UpdateGrappleBeamTilesAndIncrFlameCtr(game: &mut Game) {
    if (word!(game.grapple_point_anim_timer -= 1, reread) & 0x8000) != 0 {
        word!(game.grapple_point_anim_timer = 5);
        word!(game.grapple_point_anim_ptr += 512);
        if ((word!(game.grapple_point_anim_ptr)) as i16)
            .wrapping_sub((crate::rtl::rom_table_word!(game, 0x9bc344, 0)) as i16)
            >= 0
        {
            word!(game.grapple_point_anim_ptr = crate::rtl::rom_table_word!(game, 0x9bc342, 0));
        }
    }
    let cursor = queue_native_transfer(
        word!(game.vram_write_queue_tail),
        32,
        |game| word!(game.grapple_point_anim_ptr),
        0x9a,
        |_| 0x6200,
        game,
    );
    let index = (game.g_ram[0xcfb] >> 1 & 0xfe) >> 1;
    queue_native_transfer(
        cursor,
        128,
        |game| crate::rtl::rom_table_word!(game, 0x9bc346, isize::from(index)),
        0x9a,
        |_| 0x6210,
        game,
    );
    if word!(game.flare_counter) as i32 - 120i32 & 0x8000 != 0 {
        word!(game.flare_counter += 1);
    }
}
pub unsafe fn GrappleBeamHandler(game: &mut Game) {
    unsafe {
        let mut r18: u16 = 0;
        if word!(game.grapple_varCF6) != 0 {
            word!(game.grapple_varCF6 -= 1);
        }
        word!(game.samus_grapple_flags &= !(1));
        grapple_swing::cancel_incompatible_pose(game);
        CallGrappleBeamFunc(
            ((word!(game.grapple_beam_function)) as u32) | (0x9b0000u32),
            game,
        );
        if word!(game.grapple_beam_function) as i32 != 0x9bc4f0 & 0xffff
            && word!(game.grapple_beam_function) as i32 - (0x9bc856 & 0xffff) & 0x8000 != 0
            && (word!(game.samus_suit_palette_index) & 4) == 0
            && word!(game.fx_type) != 0
            && {
                r18 = player_bottom(game);
                (word!(game.fx_y_pos) & 0x8000) == 0
            }
            && word!(game.fx_y_pos) as i32 - r18 as i32 & 0x8000 != 0
        {
            word!(game.grapple_beam_flags |= (1u16));
        } else {
            word!(game.grapple_beam_flags &= !(1));
        };
    }
}
crate::game::callback_dispatch! {
    pub unsafe fn CallGrappleBeamFunc(ea: u32, game: &mut Game) => ea;
    call (game); default { Unreachable(game); };
    @functions [pub unsafe fn (game: &mut Game)];




    [10208496] GrappleBeamFunc_Inactive [] {
        if (word!(game.button_config_shoot_x) & word!(game.joypad1_newkeys)) != 0
            || (word!(game.button_config_shoot_x) & word!(game.joypad1_newinput_samusfilter)) != 0
        {
            grapple_swing::start_firing(game);
        } else if word!(game.flare_counter) != 0 {
            word! { game, 0;
                flare_counter, flare_animation_frame, flare_slow_sparks_anim_frame,
                flare_fast_sparks_anim_frame, flare_animation_timer, flare_slow_sparks_anim_timer,
                flare_fast_sparks_anim_timer,
            }
            crate::game::samus_palette::load_suit(&mut Runtime::new(game), crate::game::visual_state::PaletteBuffer::Current);
        }
    };
    [10209027] GrappleBeamFunc_Firing [] {
        if (word!(game.button_config_shoot_x) & word!(game.joypad1_lastkeys)) == 0 {
            word!(game.grapple_beam_function = (0x9bc856i32) as u16);
            return;
        }
        word!(game.grapple_beam_length += word!(game.grapple_beam_length_delta));
        if word!(game.grapple_beam_length) as i32 - 128i32 & 0x8000 == 0 {
            word!(game.grapple_beam_function = (0x9bc856i32) as u16);
            return;
        }
        let contact = actors::contacts::grapple_contact(&mut Runtime::new(game));
        let outcome = match contact {
            Some(contact) => actors::combat::resolve_grapple(&mut Runtime::new(game), contact),
            None => GrappleOutcome::Miss,
        };
        match outcome {
            GrappleOutcome::Attach => {},
            GrappleOutcome::Cancel => {
                word!(game.grapple_beam_function = (0x9bc856i32) as u16);
                return;
            }
            GrappleOutcome::Miss => {
                let v2: u8 = BlockCollGrappleBeam(game);
                if (v2 & 1) == 0 {
                    return;
                }
                if (v2 & 0x40) == 0 {
                    word!(game.grapple_beam_function = (0x9bc856i32) as u16);
                    return;
                }
            }
        }
        QueueSfx1_Max6(6, game);
        grapple_swing::connect_beam(game);
        word!(game.grapple_beam_length_delta = -8i32 as u16);
        word!(game.samus_grapple_flags |= (1u16));
    };
    [10209150] GrappleBeamFunc_ConnectedLockedInPlace [] {
        if !((word!(game.button_config_shoot_x) & word!(game.joypad1_lastkeys)) != 0
            && (actors::contacts::grapple_contact(&mut Runtime::new(game)).is_some_and(|contact| contact.has_reaction())
                || CheckIfGrappleIsConnectedToBlock(game) != 0))
        {
            word!(game.grapple_beam_function = (0x9bc856i32) as u16);
        }
    };
    [10209181] GrappleBeamFunc_Connected_Swinging [] {
        if (word!(game.button_config_shoot_x) & word!(game.joypad1_lastkeys)) != 0 {
            grapple_swing::apply_input(game);
            if word!(game.grapple_beam_length_delta) != 0 {
                change_length(game);
            }
            grapple_swing::apply_gravity(game);
            grapple_swing::accelerate(game);
            grapple_swing::rebound_jump(game);
            advance_swing(game);
            if (word!(game.grapple_beam_unkD36) & 0x8000) != 0
                && grapple_swing::apply_special_angle(game) & 1 != 0
            {
                return;
            }
            let keep_swinging: bool = if actors::contacts::grapple_contact(&mut Runtime::new(game)).is_some_and(|contact| contact.has_reaction()) {
                word!(game.grapple_beam_flags |= (0x8000u16));
                false
            } else {
                CheckIfGrappleIsConnectedToBlock(game) & 1 == 0
            };
            if keep_swinging {
            } else {
                place_rope_origin(game);
                grapple_swing::position_swing(game);
                return;
            }
        }
        if word!(game.grapple_beam_unkD26) != 0 || word!(game.grapple_beam_end_angle16) != 0x8000 {
            grapple_swing::launch_from_swing(game);
            word!(game.grapple_beam_function = (0x9bcb8bi32) as u16);
            word!(game.samus_movement_handler = (0x90946ei32) as u16);
        } else {
            word!(game.grapple_beam_function = (0x9bc8c5i32) as u16);
        };
    };
    [10209300] GrappleBeamFunc_Wallgrab [] {
        if !((word!(game.button_config_shoot_x) & word!(game.joypad1_lastkeys)) != 0
            && CheckIfGrappleIsConnectedToBlock(game) & 1 != 0)
        {
            word!(game.grapple_walljump_timer = 30);
            word!(game.grapple_beam_function = (0x9bc832i32) as u16);
        }
    };
    [10209330] GrappleBeamFunc_C832 [] {
        if (word!(game.grapple_walljump_timer -= 1, reread) & 0x8000) == 0 {
            if { let collision_request = player_collision::JumpRequest { kind: player_collision::JumpKind::Grapple, amount: (16i16 as i32) << 16 }; player_collision::jump_admission(&mut CollisionRuntime::new(game), collision_request) } {
                word!(game.grapple_beam_function = (0x9bc9cei32) as u16);
            }
        } else {
            word!(game.grapple_beam_function = (0x9bc8c5i32) as u16);
        };
    };
    @external {
        10206613 => grapple_swing::position_swing;
        10206872 => grapple_swing::clamp_swing_position;
        10206955 => grapple_swing::position_anchored;
        10207003 => grapple_swing::restore_start_positions;
        10208542 => grapple_swing::start_firing;
        10209366 => grapple_swing::cancel;
        10209477 => grapple_swing::release_stationary;
        10209742 => grapple_swing::release_walljump;
        10210187 => grapple_swing::release_swing;
        10206052 => grapple_swing::apply_input;
        10206239 => grapple_swing::apply_gravity;
        10206463 => grapple_swing::accelerate;
        10206532 => grapple_swing::rebound_jump;
    }
}
