//! Beam charge input and the complete uncharged, charged and hyper firing sequence.
use crate::game::projectile_initialization::Initialization;
use crate::game::projectile_initialization_ram::initialize;
use crate::game::projectile_flight;
use crate::game::sm_80::QueueSfx1_Max15;
use crate::game::sm_90::{
    fnProjPreInstr_BeamOrIceWave, fnProjPreInstr_Beam_NoWaveBeam, fnProjPreInstr_WavePlasmaEtc,
    kFireChargedBeam_Funcs, kProjectileCooldown_Uncharged, CheckBeamCollByDir, FireSba,
    WaveBeam_CheckColl,
};
use crate::game::sm_91::Samus_LoadSuitPalette;
use crate::game_state::Game;
use crate::ram_map::{checked_word as word, word_arrays as arrays, words};

pub(crate) unsafe fn update_input(game: &mut Game) {
    unsafe {
        word!(game.prev_beam_charge_counter = word!(game.flare_counter));
        if word!(game.hyper_beam_flag) != 0 || (word!(game.equipped_beams) & 0x1000) == 0 {
            if (word!(game.button_config_shoot_x) & word!(game.joypad1_lastkeys)) != 0 {
                fire(false, game);
            }
            return;
        }
        let fire_uncharged: bool = if word!(game.new_projectile_direction_changed_pose) != 0 {
            (word!(game.flare_counter) as i32 - 60i32 & 0x8000 != 0)
        } else if (word!(game.button_config_shoot_x) & word!(game.joypad1_lastkeys)) == 0 {
            if word!(game.flare_counter) == 0 {
                return;
            }
            (word!(game.flare_counter) as i32 - 60i32 & 0x8000 != 0)
        } else {
            if word!(game.flare_counter) as i32 - 120i32 & 0x8000 != 0 {
                word!(game.flare_counter += 1);
                if word!(game.flare_counter) == 1 {
                    clear_animation(game);
                    fire(false, game);
                }
            } else if FireSba(game) & 1 != 0 {
                word!(game.flare_counter = 0);
                clear_animation(game);
                Samus_LoadSuitPalette(game);
            }
            return;
        };
        word!(game.flare_counter = 0);
        clear_animation(game);
        fire(!fire_uncharged, game);
    }
}
#[forbid(unsafe_code)]
pub(crate) fn clear_animation(game: &mut Game) {
    game.g_ram[words::flare_animation_frame..words::flare_fast_sparks_anim_timer + 2].fill(0);
}
#[forbid(unsafe_code)]
fn reserve_beam(game: &mut Game) -> bool {
    if word!(game.projectile_counter).wrapping_sub(5) & 0x8000 == 0
        || word!(game.cooldown_timer) as u8 != 0
    {
        return false;
    }
    word!(game.cooldown_timer = 1);
    word!(game.projectile_counter += 1);
    true
}
unsafe fn fire(charged: bool, game: &mut Game) {
    unsafe {
        let hyper = !charged && word!(game.hyper_beam_flag) != 0;
        if reserve_beam(game) {
            let slot = (0..5)
                .find(|&slot| word!(game.projectile_damage[slot]) == 0)
                .unwrap_or(4);
            let mut projectile = (2 * slot) as u16;
            if crate::game::projectile_placement_ram::place(projectile, game) & 1 == 0 {
                word!(game.projectile_invincibility_timer = 10);
                let sound = if hyper {
                    word!(game.projectile_type[slot] = -28648i32 as u16);
                    kChargedProjectile_Sfx[8]
                } else {
                    word!(game.projectile_timers[slot] = 4);
                    if charged {
                        let beam_type = word!(game.equipped_beams) & 0x100f | 0x8010;
                        word!(game.projectile_type[slot] = beam_type);
                        kChargedProjectile_Sfx[(beam_type as i32 & 0xf) as usize]
                    } else {
                        let sound_index = word!(game.equipped_beams) as i8;
                        word!(game.projectile_type[slot] = word!(game.equipped_beams) | 0x8000);
                        kUnchargedProjectile_Sfx[(sound_index as i32 & 0xf) as usize]
                    }
                };
                QueueSfx1_Max15(sound, game);
                word!(game.play_resume_charging_beam_sfx = 0);
                initialize(projectile, Initialization::Projectile, game);
                if hyper {
                    word!(game.projectile_bomb_x_speed[slot] = 0);
                    word!(game.projectile_bomb_y_speed[slot] = 0);
                    word!(game.projectile_index = projectile);
                    WaveBeam_CheckColl(projectile, game);
                    let projectile = word!(game.projectile_index);
                    prepare_hyper(projectile, game);
                    projectile_flight::initialize_beam_speed(projectile, game);
                    word!(game.cooldown_timer = 21);
                    word!(game.charged_shot_glow_timer = 0x8014);
                    word!(game.flare_animation_frame = 29);
                    word!(game.flare_slow_sparks_anim_frame = 5);
                    word!(game.flare_fast_sparks_anim_frame = 5);
                    word!(game.flare_animation_timer = 3);
                    word!(game.flare_slow_sparks_anim_timer = 3);
                    word!(game.flare_fast_sparks_anim_timer = 3);
                    word!(game.flare_counter = 0x8000);
                } else {
                    let manual_fire = charged
                        || (word!(game.equipped_beams) & 0x1000) != 0
                        || (word!(game.button_config_shoot_x) & word!(game.joypad1_newkeys)) != 0
                        || (word!(game.button_config_shoot_x)
                            & word!(game.joypad1_newinput_samusfilter))
                            != 0;
                    let beam_type = word!(game.projectile_type[slot]);
                    word!(
                        game.cooldown_timer = if manual_fire {
                            kProjectileCooldown_Uncharged[(beam_type as i32 & 0x3f) as usize]
                        } else {
                            kBeamAutoFireCooldowns[(beam_type as i32 & 0x3f) as usize]
                        } as u16
                    );
                    word!(game.projectile_bomb_x_speed[slot] = 0);
                    word!(game.projectile_bomb_y_speed[slot] = 0);
                    word!(game.projectile_index = projectile);
                    let wave = beam_type & 1 != 0;
                    if wave {
                        WaveBeam_CheckColl(projectile, game);
                    } else {
                        CheckBeamCollByDir(projectile, game);
                        projectile = word!(game.projectile_index);
                    }
                    finish_beam(projectile, charged, wave, game);
                    if charged {
                        word!(game.charged_shot_glow_timer = 4);
                    }
                }
                return;
            }
        }
        if !hyper && word!(game.prev_beam_charge_counter) as i32 - 16i32 & 0x8000 == 0 {
            word!(game.play_resume_charging_beam_sfx = 0);
            QueueSfx1_Max15(2, game);
        }
    }
}

#[forbid(unsafe_code)]
pub(crate) fn prepare_hyper(projectile: u16, game: &mut Game) {
    let slot = usize::from(projectile >> 1);
    word!(game.projectile_damage[slot] = 1000);
    word!(game.projectile_bomb_pre_instructions[slot] = 0xb159);
}

/// Wave retains its caller's slot and launches unconditionally. Other beams
/// use the selector captured after collision and test its native type word.
pub(crate) unsafe fn finish_beam(projectile: u16, charged: bool, wave: bool, game: &mut Game) {
    let slot = usize::from(projectile >> 1);
    let at = arrays::projectile_type + 2 * slot;
    if !wave && u16::from_ne_bytes([game.g_ram[at], game.g_ram[at + 1]]) & 0xf00 != 0 {
        return;
    }
    let instructions = if charged {
        &kFireChargedBeam_Funcs
    } else {
        &kProjectileBombPreInstr
    };
    word!(
        game.projectile_bomb_pre_instructions[slot] =
            instructions[usize::from(word!(game.projectile_type[slot]) & 15)]
    );
    unsafe { projectile_flight::initialize_beam_speed(projectile, game) };
}

static kUnchargedProjectile_Sfx: [u16; 12] = [
    0xb, 0xd, 0xc, 0xe, 0xf, 0x12, 0x10, 0x11, 0x13, 0x16, 0x14, 0x15,
];
static kChargedProjectile_Sfx: [u16; 12] = [
    0x17, 0x19, 0x18, 0x1a, 0x1b, 0x1e, 0x1c, 0x1d, 0x1f, 0x22, 0x20, 0x21,
];
static kBeamAutoFireCooldowns: [u8; 12] = [0x19; 12];
static kProjectileBombPreInstr: [u16; 12] = [
    fnProjPreInstr_Beam_NoWaveBeam as u16,
    fnProjPreInstr_BeamOrIceWave as u16,
    fnProjPreInstr_Beam_NoWaveBeam as u16,
    fnProjPreInstr_BeamOrIceWave as u16,
    fnProjPreInstr_Beam_NoWaveBeam as u16,
    fnProjPreInstr_WavePlasmaEtc as u16,
    fnProjPreInstr_Beam_NoWaveBeam as u16,
    fnProjPreInstr_WavePlasmaEtc as u16,
    fnProjPreInstr_Beam_NoWaveBeam as u16,
    fnProjPreInstr_WavePlasmaEtc as u16,
    fnProjPreInstr_Beam_NoWaveBeam as u16,
    fnProjPreInstr_WavePlasmaEtc as u16,
];
