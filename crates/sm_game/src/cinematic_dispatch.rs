//! The bank $8B dispatch: `cinematic_function` and the cinematic sprite
//! objects' pre-instructions, by their original 24-bit addresses.
//!
//! Was `CallCinematicFunction` and `CallCinematicSprPreInstr` in the original
//! decompilation; the addresses stay the dispatch keys.
#![forbid(unsafe_code)]
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
