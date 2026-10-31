//! The bank $8B dispatch: `cinematic_function` and the cinematic sprite
//! objects' pre-instructions, by their original 24-bit addresses.
//!
//! Was `CallCinematicFunction` and `CallCinematicSprPreInstr` in the original
//! decompilation; the addresses stay the dispatch keys.
use crate::cinematic_host::Host;
use crate::intro_pages::{self, Palette, Scene};
use crate::visual_state::CinematicFade;
use crate::{
    cinematic, cinematic_objects, cinematic_states, ending_screens, ending_sequence, intro_objects,
    intro_sequence, title_screen, title_sequence,
};

/// Bank $8B background objects: the intro's text pages 2-5.
const PAGE_2: u16 = 0xcf45;
const PAGE_3: u16 = 0xcf4b;
const PAGE_4: u16 = 0xcf51;
const PAGE_5: u16 = 0xcf57;

/// Runs the cinematic function at `address`. Was `CallCinematicFunction`.
pub fn run_function<H: Host + ?Sized>(address: u32, game: &mut H) {
    match address {
        0x8b_9b68 => title_sequence::setup(game),
        0x8b_9d17 => title_screen::advance_title_pan(0, game),
        0x8b_9d90 => title_screen::advance_title_pan(1, game),
        0x8b_9e12 => title_screen::advance_title_pan(2, game),
        0x8b_9e8b => title_sequence::zooming_out(game),
        0x8b_9f29 => title_screen::advance_title_wait(game),
        0x8b_9f52 => title_sequence::to_file_select(game),
        0x8b_9fae => title_sequence::to_demos(game),
        0x8b_a395 => intro_sequence::initial(game),
        0x8b_a5a7 => intro_pages::wait_for_music(0xa5bd, game),
        0x8b_a5bd => intro_pages::wait_for_fade(CinematicFade::SlowIn, 0xa5f8, game),
        0x8b_a5f8 => intro_sequence::last_metroid_in_captivity(game),
        0x8b_a613 => intro_sequence::galaxy_is_at_peace(game),
        0x8b_a639 => intro_pages::wait_for_music(0xa64c, game),
        0x8b_a64c => intro_sequence::fade_out(game),
        0x8b_a663 => intro_pages::wait_for_fade(CinematicFade::SlowOut, 0xa66f, game),
        0x8b_a66f => intro_sequence::set_up_text_page_1(game),
        0x8b_a82b => intro_pages::wait_for_music(0xa84a, game),
        0x8b_a84a => intro_pages::wait_for_fade(CinematicFade::SlowIn, 0xa391, game),
        0x8b_aeb8 => intro_pages::wait_for_scene(Scene::MotherBrain, game),
        0x8b_af6c => intro_pages::wait_for_scene(Scene::BabyMetroid, game),
        0x8b_b0f2 => intro_pages::wait_for_scene(Scene::ScientistPage3, game),
        0x8b_b123 => intro_pages::wait_for_scene(Scene::ScientistPage4, game),
        0x8b_b1da => intro_pages::wait_for_final_page(game),
        0x8b_b207 => intro_pages::show_final_page(game),
        0x8b_b250 => intro_pages::fade_to_scene(Palette::Gameplay, game),
        0x8b_b2d2 => intro_pages::fade_to_scene(Palette::Scientist, game),
        0x8b_b35f => intro_pages::show_page(PAGE_2, 0xb3f4, game),
        0x8b_b370 => intro_pages::show_page(PAGE_3, 0xb3f4, game),
        0x8b_b381 => intro_pages::show_page(PAGE_4, 0xb458, game),
        0x8b_b392 => intro_pages::show_page(PAGE_5, 0xb458, game),
        0x8b_b3f4 => intro_pages::fade_to_page(Palette::Gameplay, game),
        0x8b_b458 => intro_pages::fade_to_page(Palette::Scientist, game),
        0x8b_b72f => intro_pages::fade_out_to_ceres(true, game),
        0x8b_bca0 => intro_sequence::ceres_initial(game),
        0x8b_bde4 => intro_sequence::ceres_wait_for_music(game),
        0x8b_bdf9 => intro_sequence::ceres_flying_into_camera(game),
        0x8b_bfda => intro_sequence::ceres_flying_into_ceres(game),
        0x8b_c0c5 => intro_pages::fade_out_to_ceres(false, game),
        0x8b_c100 => intro_sequence::start_game_at_ceres(game),
        0x8b_c11b => intro_sequence::boom_initial(game),
        0x8b_c2e4 => intro_sequence::boom_wait_for_music(game),
        0x8b_c2f1 => intro_sequence::boom_small_explosions(game),
        0x8b_c345 => intro_sequence::boom_explosions(game),
        0x8b_c5ca => intro_sequence::boom_gunship_flying_away(game),
        0x8b_c610 => intro_sequence::boom_wait(game),
        0x8b_c627 => intro_sequence::boom_fade_out(game),
        0x8b_c699 => intro_sequence::zebes_initial(game),
        0x8b_c79c => intro_objects::advance_intro_landing_mosaic(true, game),
        0x8b_c7ca => intro_objects::advance_intro_landing_mosaic(false, game),
        0x8b_c9f9 => intro_objects::advance_zebes_flight(1, game),
        0x8b_ca36 => intro_objects::advance_zebes_flight(2, game),
        0x8b_ca85 => intro_objects::advance_zebes_flight(3, game),
        0x8b_cad0 => intro_sequence::zebes_hold(game),
        0x8b_cadf => intro_sequence::zebes_load_game_data(game),
        0x8b_d480 => ending_sequence::setup(game),
        0x8b_d6d7 => ending_sequence::wait_for_music(game),
        0x8b_d6f2 => intro_objects::advance_zebes_destruction_zoom(2, 0xd731, Some(0xd701), game),
        0x8b_d701 => intro_objects::advance_zebes_destruction_zoom(2, 0xd731, None, game),
        0x8b_d731 => ending_sequence::scene_1_setup(game),
        0x8b_d7f8 => intro_objects::advance_zebes_destruction_zoom(3, 0xd837, Some(0xd807), game),
        0x8b_d807 => intro_objects::advance_zebes_destruction_zoom(3, 0xd837, None, game),
        0x8b_d837 => ending_sequence::scene_2_setup(game),
        0x8b_da02 => ending_sequence::scene_2_fading_in(game),
        0x8b_da1a => ending_sequence::scene_2(game),
        0x8b_dad3 => game.advance_ending_tile_upload(),
        0x8b_db9e => ending_sequence::change_music(game),
        0x8b_dbc4 => ending_sequence::gunship_setup(game),
        0x8b_dca5 => {
            ending_screens::advance_ending_gunship_spin(false, game);
        }
        0x8b_dd42 => ending_sequence::gunship_spinning_slow(game),
        0x8b_ddcd => ending_sequence::gunship_fly_into_camera(game),
        0x8b_de64 => ending_sequence::transition_to_credits(game),
        0x8b_de80 => ending_sequence::credits_setup(game),
        0x8b_e0f4 => ending_sequence::blank_screen(game),
        0x8b_e110 => ending_sequence::fade_in_shooting_stars(game),
        0x8b_e158 => ending_sequence::fade_in_samus(game),
        0x8b_e190 => ending_sequence::wavy_samus(game),
        0x8b_e1d2 => ending_screens::advance_postcredits_samus_reveal(0, game),
        0x8b_e265 => ending_screens::advance_postcredits_samus_message(true, game),
        0x8b_e293 => ending_screens::advance_postcredits_samus_message(false, game),
        0x8b_e2dd => ending_sequence::nintendo_1994(game),
        0x8b_e314 => ending_screens::advance_postcredits_samus_reveal(1, game),
        0x8b_e342 => ending_screens::advance_postcredits_samus_reveal(2, game),
        0x8b_e3ae => ending_screens::advance_postcredits_zoom(true, game),
        0x8b_e48a => ending_screens::advance_postcredits_zoom(false, game),
        0x8b_e504 => ending_sequence::fade_from_white(game),
        0x8b_e58a => ending_sequence::grey_out_icon(game),
        0x8b_e797 => ending_sequence::scroll_item_percentage(game),
        0x8b_9a47 | 0x8b_9f28 | 0x8b_a38f | 0x8b_a390 | 0x8b_a391 | 0x8b_cade | 0x8b_db9d
        | 0x8b_de63 | 0x8b_e7ba => {}
        _ => {
            game.rtl_unreachable();
        }
    }
}

/// Runs cinematic sprite object `object`'s pre-instruction at `address`. Was
/// `CallCinematicSprPreInstr`.
pub fn run_sprite_pre_instruction<H: Host + ?Sized>(address: u32, object: u16, game: &mut H) {
    match address {
        0x8b_9ccf => title_sequence::scrolling_1994(object, game),
        0x8b_a8e8 => intro_pages::advance_scene_sprite(object, 0, game),
        0x8b_a903 => intro_pages::wait_for_scene_end(object, true, game),
        0x8b_a994 => cinematic_states::egg_particle(object, game),
        0x8b_aab3 => cinematic_states::slime_drop(object, game),
        0x8b_ad68 => intro_pages::advance_laboratory_baby(object, false, game),
        0x8b_ada6 => intro_pages::advance_laboratory_baby(object, true, game),
        0x8b_ae1a => intro_pages::advance_scene_sprite(object, 1, game),
        0x8b_b786 => cinematic_objects::advance_intro_mother_brain(object, 0, game),
        0x8b_b80f => {
            cinematic_objects::advance_intro_mother_brain(object, 1, game);
        }
        0x8b_b82e => {
            cinematic_objects::advance_intro_mother_brain(object, 2, game);
        }
        0x8b_b8d8 => intro_objects::advance_intro_rinka(object, false, game),
        0x8b_b93b => intro_objects::advance_intro_rinka(object, true, game),
        0x8b_ba0f => intro_pages::wait_for_scene_end(object, false, game),
        0x8b_ba5e => intro_sequence::baby_waiting(object, game),
        0x8b_ba73 => intro_sequence::baby_hatched(object, game),
        0x8b_bb0d => intro_sequence::baby_idling(object, game),
        0x8b_bb24 => intro_objects::advance_intro_baby_pursuit(object, game),
        0x8b_beb5 => intro_objects::advance_ceres_station(object, game),
        0x8b_bf35 => cinematic::drift_cinematic_sprite(object, 0x4000, None, true, game),
        0x8b_bf5f => cinematic::drift_cinematic_sprite(object, 4096, None, true, game),
        0x8b_bf89 => cinematic::drift_cinematic_sprite(object, 2048, None, true, game),
        0x8b_bfc6 => {
            cinematic::drift_cinematic_sprite(object, -0x2000i32 as u32, None, false, game)
        }
        0x8b_c489 => intro_sequence::explosion_spawner(object, game),
        0x8b_c582 => {
            cinematic::drift_cinematic_sprite(object, 0x4000, Some(-4096i32 as u32), false, game)
        }
        0x8b_c84e => intro_objects::wait_cinematic_scene(object, 0xcade, Some(0xc85d), game),
        0x8b_c85d => intro_objects::launch_cinematic_sprite(object, 64, false, game),
        0x8b_c8aa => intro_objects::wait_cinematic_scene(object, 0xcade, Some(0xc8b9), game),
        0x8b_c8b9 => intro_objects::launch_cinematic_sprite(object, 32, true, game),
        0x8b_c8f9 => intro_objects::wait_cinematic_scene(object, 0xcade, Some(0xc908), game),
        0x8b_c908 => intro_objects::launch_cinematic_sprite(object, 32, false, game),
        0x8b_efb2 => ending_screens::update_ending_critters(object, false, game),
        0x8b_f1e7 => ending_sequence::icon_s_top(object, game),
        0x8b_f227 => {
            ending_screens::advance_ending_sprite_pair(usize::from(object >> 1), false, game);
        }
        0x8b_f2a5 => intro_pages::advance_scene_sprite(object, 2, game),
        0x8b_f35a => intro_objects::advance_zebes_stars(object, false, game),
        0x8b_f375 => intro_objects::advance_zebes_stars(object, true, game),
        0x8b_f39b => intro_objects::wait_cinematic_scene(object, 0xde63, None, game),
        0x8b_f3b9 => intro_objects::wait_cinematic_scene(object, 0xde80, None, game),
        0x8b_f455 => ending_screens::wait_ending_cloud(object, true, 0xf464, game),
        0x8b_f464 => ending_screens::move_ending_cloud(object, (0, 1), game),
        0x8b_f478 => ending_screens::wait_ending_cloud(object, true, 0xf487, game),
        0x8b_f487 => ending_screens::move_ending_cloud(object, (0, -1), game),
        0x8b_f49b => ending_screens::wait_ending_cloud(object, false, 0xf4aa, game),
        0x8b_f4aa => ending_screens::move_ending_cloud(object, (-1, -2), game),
        0x8b_f4d1 => ending_screens::wait_ending_cloud(object, false, 0xf4e0, game),
        0x8b_f4e0 => ending_screens::move_ending_cloud(object, (1, 2), game),
        0x8b_f507 => intro_objects::wait_cinematic_scene(object, 0xdb9d, None, game),
        0x8b_f528 => ending_screens::advance_ending_samus_jump(object, 0, game),
        0x8b_f57f => ending_screens::advance_ending_samus_jump(object, 1, game),
        0x8b_f5dd => ending_screens::advance_ending_samus_jump(object, 2, game),
        0x8b_93d9 | 0x8b_945f | 0x8b_a8d4 | 0x8b_bfd9 | 0x8b_f51c => {}
        _ => {
            game.rtl_unreachable();
        }
    }
}

// The intro's local copies of these bank $8B definitions must match.
const _: () = {
    use crate::constants::bank_8b as bank;
    use crate::intro_pages as intro;
    assert!(
        intro::ADDR_CINEMATIC_BG_OBJECT_DEF_8BCF3F == bank::ADDR_K_CINEMATIC_BG_OBJECT_DEF_8BCF3F
    );
    assert!(
        intro::ADDR_CINEMATIC_BG_OBJECT_DEF_8BCF5D == bank::ADDR_K_CINEMATIC_BG_OBJECT_DEF_8BCF5D
    );
    assert!(
        intro::ADDR_CINEMATIC_SPRITE_OBJECT_DEF_8BCE5B
            == bank::ADDR_K_CINEMATIC_SPRITE_OBJECT_DEF_8BCE5B
    );
    assert!(
        intro::ADDR_CINEMATIC_SPRITE_OBJECT_DEF_8BCE55
            == bank::ADDR_K_CINEMATIC_SPRITE_OBJECT_DEF_8BCE55
    );
    assert!(
        intro::ADDR_CINEMATIC_SPRITE_OBJECT_DEF_8BCE61
            == bank::ADDR_K_CINEMATIC_SPRITE_OBJECT_DEF_8BCE61
    );
    assert!(
        intro::ADDR_CINEMATIC_SPRITE_OBJECT_DEF_8BCE67
            == bank::ADDR_K_CINEMATIC_SPRITE_OBJECT_DEF_8BCE67
    );
    assert!(
        intro::ADDR_CINEMATIC_SPRITE_OBJECT_DEF_8BCE73
            == bank::ADDR_K_CINEMATIC_SPRITE_OBJECT_DEF_8BCE73
    );
    assert!(
        intro::ADDR_CINEMATIC_SPRITE_OBJECT_DEF_8BCE79
            == bank::ADDR_K_CINEMATIC_SPRITE_OBJECT_DEF_8BCE79
    );
    assert!(
        intro::ADDR_CINEMATIC_SPRITE_OBJECT_DEF_8BCF27
            == bank::ADDR_K_CINEMATIC_SPRITE_OBJECT_DEF_8BCF27
    );
};

// The cinematic engine's named bank $8B definitions must match.
const _: () = {
    use crate::constants::bank_8b as bank;
    use crate::ending_screens::samus as s;
    use crate::ending_sequence::object as e;
    use crate::intro_sequence::{self as i, mode7 as m};
    use crate::title_sequence::object as t;
    assert!(t::COPYRIGHT_FADE_IN == bank::ADDR_K_CINEMATIC_SPRITE_OBJECT_DEF_8BA113 as u16);
    assert!(
        i::object::PLANET_ZEBES_JAPANESE_TEXT
            == bank::ADDR_K_CINEMATIC_SPRITE_OBJECT_DEF_8BCEB5 as u16
    );
    assert!(i::ZEBES_TEXT_FADE_OUT == bank::ADDR_LOC_8BE1B4 as u16);
    assert!(m::JAPANESE_TEXT_PAGE_1 == bank::ADDR_K_MODE7_OBJECT_DEF_8BD401 as u16);
    assert!(m::JAPANESE_TEXT_PAGE_2 == bank::ADDR_K_MODE7_OBJECT_DEF_8BD407 as u16);
    assert!(m::JAPANESE_TEXT_PAGE_3 == bank::ADDR_K_MODE7_OBJECT_DEF_8BD413 as u16);
    assert!(m::JAPANESE_TEXT_PAGE_4 == bank::ADDR_K_MODE7_OBJECT_DEF_8BD41F as u16);
    assert!(m::JAPANESE_TEXT_PAGE_5 == bank::ADDR_K_MODE7_OBJECT_DEF_8BD42B as u16);
    assert!(m::JAPANESE_TEXT_PAGE_6 == bank::ADDR_K_MODE7_OBJECT_DEF_8BD437 as u16);
    assert!(e::ZEBES_SILHOUETTE == bank::ADDR_K_CINEMATIC_SPRITE_OBJECT_DEF_8BEEB5 as u16);
    assert!(e::ZEBES_STARS_RIGHT == bank::ADDR_K_CINEMATIC_SPRITE_OBJECT_DEF_8BEEBB as u16);
    assert!(e::ZEBES_AFTERGLOW == bank::ADDR_K_CINEMATIC_SPRITE_OBJECT_DEF_8BEEC1 as u16);
    assert!(e::COMPLETED_SUCCESSFULLY == bank::ADDR_K_CINEMATIC_SPRITE_OBJECT_DEF_8BEECD as u16);
    assert!(e::ZEBES_STARS_LEFT == bank::ADDR_K_CINEMATIC_SPRITE_OBJECT_DEF_8BEEF7 as u16);
    assert!(e::CLEAR_TIME == bank::ADDR_K_CINEMATIC_SPRITE_OBJECT_DEF_8BEEFD as u16);
    assert!(e::CLEAR_TIME_DIGITS == bank::ADDR_K_CINEMATIC_SPRITE_OBJECT_DEF_8BEF03 as u16);
    assert!(s::SUITLESS_IDLE == bank::ADDR_K_CINEMATIC_SPRITE_OBJECT_DEF_8BEF27 as u16);
    assert!(s::SUITLESS_IDLE_LEGS == bank::ADDR_K_CINEMATIC_SPRITE_OBJECT_DEF_8BEF2D as u16);
    assert!(
        s::SUITLESS_LETTING_HAIR_DOWN == bank::ADDR_K_CINEMATIC_SPRITE_OBJECT_DEF_8BEF33 as u16
    );
    assert!(
        s::SUITLESS_LETTING_HAIR_DOWN_LEGS
            == bank::ADDR_K_CINEMATIC_SPRITE_OBJECT_DEF_8BEF39 as u16
    );
    assert!(s::SUITLESS_JUMP == bank::ADDR_K_CINEMATIC_SPRITE_OBJECT_DEF_8BEF3F as u16);
    assert!(s::SUITED_IDLE_BODY == bank::ADDR_K_CINEMATIC_SPRITE_OBJECT_DEF_8BEF45 as u16);
    assert!(
        s::SUITED_IDLE_HEAD_WITH_HELMET == bank::ADDR_K_CINEMATIC_SPRITE_OBJECT_DEF_8BEF4B as u16
    );
    assert!(
        s::SUITED_IDLE_HEAD_WITHOUT_HELMET
            == bank::ADDR_K_CINEMATIC_SPRITE_OBJECT_DEF_8BEF51 as u16
    );
    assert!(s::SUITED_THUMBS_UP_BODY == bank::ADDR_K_CINEMATIC_SPRITE_OBJECT_DEF_8BEF57 as u16);
    assert!(s::SUITED_THUMBS_UP_ARM == bank::ADDR_K_CINEMATIC_SPRITE_OBJECT_DEF_8BEF5D as u16);
    assert!(
        s::SUITED_THUMBS_UP_HEAD_WITH_HELMET
            == bank::ADDR_K_CINEMATIC_SPRITE_OBJECT_DEF_8BEF63 as u16
    );
    assert!(
        s::SUITED_THUMBS_UP_HEAD_WITHOUT_HELMET
            == bank::ADDR_K_CINEMATIC_SPRITE_OBJECT_DEF_8BEF69 as u16
    );
    assert!(s::SUITED_JUMP_BODY == bank::ADDR_K_CINEMATIC_SPRITE_OBJECT_DEF_8BEF6F as u16);
    assert!(
        s::SUITED_JUMP_HEAD_WITH_HELMET == bank::ADDR_K_CINEMATIC_SPRITE_OBJECT_DEF_8BEF75 as u16
    );
    assert!(
        s::SUITED_JUMP_HEAD_WITHOUT_HELMET
            == bank::ADDR_K_CINEMATIC_SPRITE_OBJECT_DEF_8BEF7B as u16
    );
};

// The named bank $8B functions, objects and lists are the decompilation's.
const _: () = {
    use crate::constants::bank_8b::*;
    use crate::intro_sequence as i;
    use crate::title_sequence as t;
    const fn at(word: u16) -> u32 {
        0x8b_0000 | word as u32
    }
    assert!(at(t::function::NOTHING) == 0x8b9a47);
    assert!(at(t::function::SETUP) == 0x8b9b68);
    assert!(at(t::function::ZOOMING_OUT) == 9150091 && at(t::function::TO_FILE_SELECT) == 9150290);
    assert!(at(t::function::TO_DEMOS) == 9150382 && at(t::function::WAITING) == 9150248);
    assert!(at(t::function::TITLE_SCREEN) == 9150249 && at(t::NO_PRE_INSTRUCTION) == 9147353);
    assert!(t::object::SCROLLING_1994 == ADDR_K_CINEMATIC_SPRITE_OBJECT_DEF_8BA0EF as u16);
    assert!(t::object::TITLE_LOGO == ADDR_K_CINEMATIC_SPRITE_OBJECT_DEF_8BA107 as u16);
    assert!(t::object::COPYRIGHT == ADDR_K_CINEMATIC_SPRITE_OBJECT_DEF_8BA119 as u16);
    assert!(t::object::DEMO_LOGO == ADDR_K_CINEMATIC_SPRITE_OBJECT_DEF_8BA125 as u16);
    assert!(t::object::BABY_METROID == ADDR_K_MODE7_OBJ_BABY_METROID_IN_TITLE as u16);
    assert!(
        at(i::function::WAIT_FOR_MUSIC_AND_FADE_IN) == 0x8ba5a7
            && at(i::function::PAGE_6_DRAWN) == 0x8ba38f
    );
    assert!(
        at(i::function::GALAXY_IS_AT_PEACE) == 9152019
            && at(i::function::WAIT_240_FRAMES) == 9152057
    );
    assert!(
        at(i::function::WAIT_FOR_FADE_OUT) == 9152099
            && at(i::function::SPAWN_TEXT_PAGE_1) == 9152555
    );
    assert!(
        at(i::function::CERES_WAIT_FOR_MUSIC) == 9158116
            && at(i::function::CERES_FLYING_INTO_CAMERA) == 9158137
    );
    assert!(
        at(i::function::CERES_FLYING_INTO_CERES) == 9158618
            && at(i::function::BOOM_WAIT_FOR_MUSIC) == 9159396
    );
    assert!(
        at(i::function::BOOM_SMALL_EXPLOSIONS) == 9159409
            && at(i::function::BOOM_EXPLOSIONS) == 9159493
    );
    assert!(
        at(i::function::BOOM_GUNSHIP_FLYING_AWAY) == 9160138
            && at(i::function::BOOM_WAIT_C0H_FRAMES) == 9160208
    );
    assert!(at(i::function::BOOM_FADE_OUT) == 9160231 && at(i::function::ZEBES_INITIAL) == 9160345);
    assert!(
        at(i::function::ZEBES_FADING_IN) == 9160604
            && at(i::function::ZEBES_SLIDE_SCENE_AWAY) == 9161438
    );
    assert!(
        at(i::pre_instruction::BABY_HATCHED) == 9157235
            && at(i::pre_instruction::BABY_IDLING) == 9157389
    );
    assert!(
        at(i::pre_instruction::BABY_DANCING) == 9157412
            && at(i::pre_instruction::NOTHING) == 9147353
    );
    assert!(i::object::TEXT_CARET == ADDR_K_CINEMATIC_SPRITE_OBJECT_DEF_8BCE6D as u16);
    assert!(i::object::CERES_GUNSHIP == ADDR_K_CINEMATIC_SPRITE_OBJECT_DEF_8BCE7F as u16);
    assert!(i::object::CERES_UNDER_ATTACK == ADDR_K_CINEMATIC_SPRITE_OBJECT_DEF_8BCE85 as u16);
    assert!(i::object::CERES_SMALL_ASTEROIDS == ADDR_K_CINEMATIC_SPRITE_OBJECT_DEF_8BCE8B as u16);
    assert!(i::object::CERES_VORTEX == ADDR_K_CINEMATIC_SPRITE_OBJECT_DEF_8BCE91 as u16);
    assert!(i::object::CERES_EXPLOSION_2 == ADDR_K_CINEMATIC_SPRITE_OBJECT_DEF_8BCEC1 as u16);
    assert!(i::object::SLIME_DROPS == ADDR_K_CINEMATIC_SPRITE_OBJECT_DEF_8BCEF1 as u16);
    assert!(i::object::CERES_STARS == ADDR_K_CINEMATIC_SPRITE_OBJECT_DEF_8BCF0F as u16);
    assert!(i::object::CERES_EXPLOSION_GUNSHIP == ADDR_K_CINEMATIC_SPRITE_OBJECT_DEF_8BCF2D as u16);
    assert!(i::object::CERES_EXPLOSION_SPAWNER == ADDR_K_CINEMATIC_SPRITE_OBJECT_DEF_8BCF33 as u16);
    assert!(i::object::CERES_LARGE_ASTEROIDS == ADDR_K_CINEMATIC_SPRITE_OBJECT_DEF_8BCF39 as u16);
    assert!(i::object::SAMUS_BLINKING == ADDR_K_CINEMATIC_BG_OBJECT_DEF_8BCF63 as u16);
    assert!(i::object::CERES_BG == ADDR_K_CINEMATIC_BG_OBJECT_DEF_8BCF69 as u16);
    assert!(i::mode7::FLYING_INTO_CERES == ADDR_K_CINEMATIC_FUNCTION_INTRO_FUNC56_M7 as u16);
    assert!(i::mode7::BOOM == ADDR_K_CINEMATIC_FUNCTION_INTRO_FUNC76_M7_0 as u16);
    assert!(i::mode7::BOOM_GUNSHIP == ADDR_K_CINEMATIC_FUNCTION_INTRO_FUNC76_M7_1 as u16);
    assert!(i::mode7::BOOM_WITH_SAMUS == ADDR_K_CINEMATIC_FUNCTION_INTRO_FUNC76_M7_2 as u16);
};

const _: () = {
    use crate::constants::bank_8b::*;
    use crate::ending_sequence::{function as f, object as o};
    const fn at(word: u16) -> u32 {
        0x8b_0000 | word as u32
    }
    assert!(at(f::WAIT_FOR_MUSIC) == 9164503 && at(f::SCENE_0_FADING_IN) == 9164530);
    assert!(at(f::SCENE_1_FADING_IN) == 9164792 && at(f::SCENE_2_FADING_IN) == 9165314);
    assert!(at(f::SCENE_2) == 9165338 && at(f::LOAD_GUNSHIP_BG) == 9165523);
    assert!(at(f::CREDITS) == 9165725 && at(f::GUNSHIP_SETUP) == 9165764);
    assert!(at(f::GUNSHIP_SPINNING_FAST) == 9165989 && at(f::GUNSHIP_FREEZE_FRAME) == 9166435);
    assert!(at(f::CREDITS_SETUP) == 9166464 && at(f::FADE_IN_SHOOTING_STARS) == 9167120);
    assert!(at(f::FADE_IN_SAMUS) == 9167192 && at(f::WAVY_SAMUS) == 9167248);
    assert!(at(f::DEER_FORCE) == 9167314 && at(f::IDLE_SAMUS_2_CROSS_FADE) == 9167636);
    assert!(at(f::IDLE_SAMUS_2) == 9167682 && at(f::FINISHED) == 9168826);
    assert!(o::EXPLODING_ZEBES == ADDR_K_CINEMATIC_SPRITE_OBJECT_DEF_8BEE9D as u16);
    assert!(o::PURPLE_GLOW == ADDR_K_CINEMATIC_SPRITE_OBJECT_DEF_8BEEA3 as u16);
    assert!(o::STARS == ADDR_K_CINEMATIC_SPRITE_OBJECT_DEF_8BEEA9 as u16);
    assert!(o::LAVA == ADDR_K_CINEMATIC_SPRITE_OBJECT_DEF_8BEEAF as u16);
    assert!(o::GUNSHIP_TEXT == ADDR_K_CINEMATIC_SPRITE_OBJECT_DEF_8BEEC7 as u16);
    assert!(o::ITEM_PERCENTAGE == ADDR_K_CINEMATIC_BG_OBJECT_DEF_8BF748 as u16);
    assert!(
        o::ICON == ADDR_K_CINEMATIC_BG_OBJECT_DEF_8BF74E as u16
            && o::CREDITS == ADDR_STRU_8BF6F8 as u16
    );
};
