//! Backend-free composition of the actual animation and palette cores.
#![forbid(unsafe_code)]
// Match lib.rs's existing policy for imported legacy symbols and translated locals.
#![allow(non_camel_case_types)]
#![allow(non_snake_case)]
#![allow(non_upper_case_globals)]
// The detached harness replaces the compat Game; the DLL verifier's adapters
// (session_runtime, game_state) only exist in the sm library.
#[cfg(sm_verify_dll)]
compile_error!(
    "player_detached is built without --cfg sm_verify_dll; that cfg selects the DLL verifier"
);
include!("shared.rs");

fn main() {
    let args: Vec<String> = std::env::args().collect();
    if args.get(1).map(String::as_str) == Some("--visual-mutation") {
        let case = args.get(2).expect("visual case name required");
        visual_checks::run_selected(case);
        println!("visual composition case passed: {case}");
        return;
    }
    if args.get(1).map(String::as_str) == Some("--sprite-mutation") {
        let case = args.get(2).expect("sprite case name required");
        sprite_checks::run_selected(case);
        println!("sprite composition case passed: {case}");
        return;
    }
    if args.get(1).map(String::as_str) == Some("--flags-mutation") {
        let case = args.get(2).expect("flags case name required");
        flag_checks::run_selected(case);
        println!("flags composition case passed: {case}");
        return;
    }
    checks::run();
    composition_checks::run();
    enemy_checks::run();
    visual_checks::run();
    sprite_checks::run();
    flag_checks::run();
    palette_independent_checks::run();
    animation_independent_checks::run();
    println!("player/animation/palette composed core checks passed");
}
