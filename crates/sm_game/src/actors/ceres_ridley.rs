//! Ceres Ridley (bank $A6:A0F5-$A6:C537): the Ceres station fight's intro
//! fades, hovering, fireballs, lunges and swoops, the fake and real retreats,
//! the mode 7 getaway cutscene and the self destruct sequence it starts.
//!
//! Was `CeresRidley_*` and the self destruct `Ridley_*` functions in the
//! original decompilation. Ceres Ridley is enemy 0 with Norfair Ridley's slot
//! and RAM, so it shares the `Ridley` view and his tail, baby metroid,
//! movement and drawing in `ridley`; the intro (`initial` to
//! `roar_before_flying`) and `init` serve both fights.
#![forbid(unsafe_code)]
use crate::actors::ridley;

/// A lease on a game with Ceres Ridley's effects: Ridley's.
pub use crate::actors::ridley::Lease;

/// What Ceres Ridley's lease needs beyond Ridley's: the boss flags, the mode 7
/// transfers, the getaway push and the escape sequence's tiles.
pub trait Host: ridley::Host + crate::display::preparation::Access {
    /// The boss bits for `area`.
    fn boss_byte(&mut self, area: u16) -> u8;
    /// Queue the bank `bank` mode 7 transfer list at `list`.
    fn queue_mode7_transfer(&mut self, bank: u8, list: u16);
    /// Start Samus's Ceres getaway push.
    fn begin_ceres_push(&mut self);
    /// Queue the next escape timer tile transfer; true once they are done.
    fn queue_escape_timer_tiles(&mut self) -> bool;
    /// Queue the Ceres getaway's tile transfer.
    fn queue_getaway_tiles(&mut self);
    /// Queue the Japanese "self destruct sequence" text tilemaps.
    fn queue_escape_tile_table(&mut self);
}

/// Bank $A6 Ceres Ridley functions, stored in the body's `function` word.
pub mod ceres_function {
    pub const TRANSITION_TO_FLYING: u16 = 0xa354;
    pub const INITIAL: u16 = 0xa35b;
    pub const WAIT_TO_FADE_IN: u16 = 0xa377;
    pub const FADE_IN_EYE: u16 = 0xa389;
    pub const FADE_IN_BODY: u16 = 0xa3df;
    pub const WAIT_TO_ROAR: u16 = 0xa455;
    pub const ROAR_BEFORE_FLYING: u16 = 0xa478;
    pub const LIFTOFF_DECELERATING: u16 = 0xa6c8;
    pub const HOVERING: u16 = 0xa6e8;
    pub const FIREBALLS_MOVE_TO_POSITION: u16 = 0xa782;
    pub const FIREBALLS_SHOOTING: u16 = 0xa7f9;
    pub const LUNGE_SETUP: u16 = 0xa83c;
    pub const LUNGE: u16 = 0xa84e;
    pub const SWOOP_SETUP: u16 = 0xa88d;
    pub const SWOOP_MOVE_TO_POSITION: u16 = 0xa8a4;
    pub const SWOOP_AIMING_DOWN: u16 = 0xa8d4;
    pub const SWOOP_AIMING_LEFT: u16 = 0xa8f8;
    pub const SWOOP_AIMING_UP: u16 = 0xa923;
    pub const SWOOP_AIMING_UP_FASTER: u16 = 0xa947;
    pub const WRITE_PALETTES_AND_SPAWN_WALLS: u16 = 0xa9a0;
    pub const SET_UP_GETAWAY: u16 = 0xaa11;
    /// The RTS while the getaway cutscene runs.
    pub const GETAWAY: u16 = 0xaa4f;
    pub const SELF_DESTRUCT_ACTIVATED: u16 = 0xaa50;
    pub const SELF_DESTRUCT_ACTIVATING: u16 = 0xc04e;
}

/// Bank $A6 instruction lists.
pub mod ceres_instruction_list {
    pub const INITIAL: u16 = 0xe538;
    pub const LUNGE: u16 = 0xe548;
    pub const FIGHT_INTRO_ROAR: u16 = 0xe690;
    pub const TRANSITION_TO_FLYING: u16 = 0xe91d;
    pub const BABY_METROID: u16 = 0xbf31;
}

/// Bank $A6 palettes and populations.
pub mod ceres_data {
    /// Initial additional palettes - Ceres Ridley.
    pub const CERES_PALETTES: u16 = 0xe16f;
    /// Initial additional palettes - Norfair Ridley.
    pub const NORFAIR_PALETTES: u16 = 0xe1cf;
    /// The real retreat's palettes.
    pub const RETREAT_SPRITE_PALETTE: u16 = 0xa9e3;
    pub const RETREAT_BACKGROUND_PALETTE: u16 = 0xaa01;
    /// The walls spawned during the getaway cutscene.
    pub const GETAWAY_WALLS: [u16; 2] = [0xaa2f, 0xaa3f];
}

/// The Ceres Ridley palettes by zoom level, 20h bytes each ($A6:B107).
const ZOOM_PALETTES: u16 = 0xb107;
/// Mode 7 with overflowing tiles transparent and no screen flip.
const MODE7_TRANSPARENT_OVERFLOW: u8 = 0x80;
/// The Ceres escape typewriter's tiles.
const ESCAPE_TEXT_TILES: u16 = 0x3582;
/// Ceres status values.
const RIDLEY_ESCAPING: u16 = 1;
const ESCAPE_STARTED: u16 = 2;

/// Initialisation AI - enemy $E13F/$E17F (Ridley) ($A6:A0F5), for both fights;
/// a defeated Ridley is deleted. Was `CeresRidley_Init`.
pub fn init<L: Lease + ?Sized>(game: &mut L) {
    game.foreign(|game| owned::init(&mut crate::runtime_access::RuntimeAccess::runtime(game)))
}

/// Main AI - enemy $E13F (Ceres Ridley) ($A6:A288): immortal; until the
/// getaway it moves, animates and draws as Norfair Ridley does. Was
/// `CeresRidley_Main`.
pub fn main<L: Lease + ?Sized>(game: &mut L) {
    game.foreign(|game| owned::main(&mut crate::runtime_access::RuntimeAccess::runtime(game)))
}

/// Run the bank $A6 Ridley function `function`, Ceres Ridley's or Norfair
/// Ridley's, on this lease. Was `CallRidleyFunc`.
pub fn run_function<L: Lease + ?Sized>(game: &mut L, function: u16) {
    game.foreign(|game| {
        owned::run_function(
            &mut crate::runtime_access::RuntimeAccess::runtime(game),
            function,
        )
    })
}

/// Randomly update Ridley tail curliness ($A6:A2BD): the ideal inter-segment
/// angle, 8 to 17h, one frame in 256. Was `CeresRidley_Func_1`.
pub fn randomize_tail_curliness<L: Lease + ?Sized>(game: &mut L) {
    game.foreign(|game| {
        owned::randomize_tail_curliness(&mut crate::runtime_access::RuntimeAccess::runtime(game))
    })
}

/// Hurt AI - enemy $E13F (Ceres Ridley) ($A6:A2D3): flash and draw. Was
/// `CeresRidley_Hurt`.
pub fn hurt<L: Lease + ?Sized>(game: &mut L) {
    game.foreign(|game| owned::hurt(&mut crate::runtime_access::RuntimeAccess::runtime(game)))
}

/// Ridley function - transition to flying ($A6:A354). Was `CeresRidley_Func_2`.
pub fn transition_to_flying<L: Lease + ?Sized>(game: &mut L) {
    game.foreign(|game| {
        owned::transition_to_flying(&mut crate::runtime_access::RuntimeAccess::runtime(game))
    })
}

/// Ridley function - initial ($A6:A35B): after the door transition, wait to
/// fade in, shorter in Norfair. Was `CeresRidley_Func_3`.
pub fn initial<L: Lease + ?Sized>(game: &mut L) {
    game.foreign(|game| owned::initial(&mut crate::runtime_access::RuntimeAccess::runtime(game)))
}

/// Ridley function - wait to fade in ($A6:A377). Was `CeresRidley_A377`.
pub fn wait_to_fade_in<L: Lease + ?Sized>(game: &mut L) {
    game.foreign(|game| {
        owned::wait_to_fade_in(&mut crate::runtime_access::RuntimeAccess::runtime(game))
    })
}

/// Ridley function - fade in eye ($A6:A389): every 256th frame the next step
/// of the eye colours, then the body. Was `CeresRidley_Func_4`.
pub fn fade_in_eye<L: Lease + ?Sized>(game: &mut L) {
    game.foreign(|game| {
        owned::fade_in_eye(&mut crate::runtime_access::RuntimeAccess::runtime(game))
    })
}

/// Ridley function - fade in body ($A6:A3DF): every other frame the next 11
/// body colours, into sprite palettes 1 and 7; then tangible and waiting to
/// roar. Was `CeresRidley_Func_5`.
pub fn fade_in_body<L: Lease + ?Sized>(game: &mut L) {
    game.foreign(|game| {
        owned::fade_in_body(&mut crate::runtime_access::RuntimeAccess::runtime(game))
    })
}

/// Ridley function - wait to roar ($A6:A455): the fight intro roar, longer in
/// Ceres. Was `CeresRidley_Func_6`.
pub fn wait_to_roar<L: Lease + ?Sized>(game: &mut L) {
    game.foreign(|game| {
        owned::wait_to_roar(&mut crate::runtime_access::RuntimeAccess::runtime(game))
    })
}

/// Ridley function - roar before fly ($A6:A478): in Norfair the background
/// fades in and the lava rises first. Was `CeresRidley_Func_7`.
pub fn roar_before_flying<L: Lease + ?Sized>(game: &mut L) {
    game.foreign(|game| {
        owned::roar_before_flying(&mut crate::runtime_access::RuntimeAccess::runtime(game))
    })
}

/// Fade in Ridley room background ($A6:A4D6) by step `step`; true once done.
/// Was `CeresRidley_Func_8`.
pub fn fade_in_background<L: Lease + ?Sized>(step: u16, game: &mut L) -> bool {
    game.foreign(|game| {
        owned::fade_in_background(
            step,
            &mut crate::runtime_access::RuntimeAccess::runtime(game),
        )
    })
}

/// Ridley function - liftoff - accelerating ($A6:A6AF). Was
/// `CeresRidley_Func_9`.
pub fn liftoff_accelerating<L: Lease + ?Sized>(game: &mut L) {
    game.foreign(|game| {
        owned::liftoff_accelerating(&mut crate::runtime_access::RuntimeAccess::runtime(game))
    })
}

/// Ridley function - liftoff - decelerating ($A6:A6C8). Was
/// `CeresRidley_Func_10`.
pub fn liftoff_decelerating<L: Lease + ?Sized>(game: &mut L) {
    game.foreign(|game| {
        owned::liftoff_decelerating(&mut crate::runtime_access::RuntimeAccess::runtime(game))
    })
}

/// Ridley function - hovering ($A6:A6E8): after 64h hits the fake retreat,
/// when Samus is below 30 health the real one, otherwise a random attack once
/// in position or after 7Ch frames. Was `CeresRidley_Func_11`.
pub fn hovering<L: Lease + ?Sized>(game: &mut L) {
    game.foreign(|game| owned::hovering(&mut crate::runtime_access::RuntimeAccess::runtime(game)))
}

/// Ceres Ridley hovering movement ($A6:A763): towards (C0h, 64h); false once
/// there. Was `CeresRidley_Func_12`.
pub fn hovering_movement<L: Lease + ?Sized>(game: &mut L) -> bool {
    game.foreign(|game| {
        owned::hovering_movement(&mut crate::runtime_access::RuntimeAccess::runtime(game))
    })
}

/// Ridley function - shoot fireballs - move to position ($A6:A782): rise with
/// at least 80h speed, then shoot from the top. Was `CeresRidley_Func_13`.
pub fn fireballs_move_to_position<L: Lease + ?Sized>(game: &mut L) {
    game.foreign(|game| {
        owned::fireballs_move_to_position(&mut crate::runtime_access::RuntimeAccess::runtime(game))
    })
}

/// Ridley function - shoot fireballs - shooting ($A6:A7F9): jitter around the
/// shooting spot, then hover. Was `CeresRidley_A7F9`.
pub fn fireballs_shooting<L: Lease + ?Sized>(game: &mut L) {
    game.foreign(|game| {
        owned::fireballs_shooting(&mut crate::runtime_access::RuntimeAccess::runtime(game))
    })
}

/// Ridley function - lunge - setup ($A6:A83C). Was `CeresRidley_Func_14`.
pub fn lunge_setup<L: Lease + ?Sized>(game: &mut L) {
    game.foreign(|game| {
        owned::lunge_setup(&mut crate::runtime_access::RuntimeAccess::runtime(game))
    })
}

/// Ridley function - lunge - main ($A6:A84E): at Samus, 44h px above her and
/// no higher than 40h, until there or the timer runs out. Was
/// `CeresRidley_Func_15`.
pub fn lunge<L: Lease + ?Sized>(game: &mut L) {
    game.foreign(|game| owned::lunge(&mut crate::runtime_access::RuntimeAccess::runtime(game)))
}

/// Ridley function - swoop - setup ($A6:A88D), without idle tail whips. Was
/// `CeresRidley_Func_16`.
pub fn swoop_setup<L: Lease + ?Sized>(game: &mut L) {
    game.foreign(|game| {
        owned::swoop_setup(&mut crate::runtime_access::RuntimeAccess::runtime(game))
    })
}

/// Ridley function - swoop - move to position ($A6:A8A4). Was
/// `CeresRidley_Func_17`.
pub fn swoop_move_to_position<L: Lease + ?Sized>(game: &mut L) {
    game.foreign(|game| {
        owned::swoop_move_to_position(&mut crate::runtime_access::RuntimeAccess::runtime(game))
    })
}

/// Ridley function - swoop - descending - aiming down ($A6:A8D4). Was
/// `CeresRidley_Func_18`.
pub fn swoop_aiming_down<L: Lease + ?Sized>(game: &mut L) {
    game.foreign(|game| {
        owned::swoop_aiming_down(&mut crate::runtime_access::RuntimeAccess::runtime(game))
    })
}

/// Ridley function - swoop - descending - aiming left ($A6:A8F8), with a tail
/// whip. Was `CeresRidley_Func_19`.
pub fn swoop_aiming_left<L: Lease + ?Sized>(game: &mut L) {
    game.foreign(|game| {
        owned::swoop_aiming_left(&mut crate::runtime_access::RuntimeAccess::runtime(game))
    })
}

/// Ridley function - swoop - ascending - aiming up ($A6:A923). Was
/// `CeresRidley_Func_20`.
pub fn swoop_aiming_up<L: Lease + ?Sized>(game: &mut L) {
    game.foreign(|game| {
        owned::swoop_aiming_up(&mut crate::runtime_access::RuntimeAccess::runtime(game))
    })
}

/// Ridley function - swoop - ascending - aiming up faster ($A6:A947): back to
/// hovering, with idle tail whips again. Was `CeresRidley_Func_21`.
pub fn swoop_aiming_up_faster<L: Lease + ?Sized>(game: &mut L) {
    game.foreign(|game| {
        owned::swoop_aiming_up_faster(&mut crate::runtime_access::RuntimeAccess::runtime(game))
    })
}

/// Ridley function - real retreat - rising ($A6:A971): off the top with the
/// baby metroid. Was `CeresRidley_Func_22`.
pub fn real_retreat_rising<L: Lease + ?Sized>(game: &mut L) {
    game.foreign(|game| {
        owned::real_retreat_rising(&mut crate::runtime_access::RuntimeAccess::runtime(game))
    })
}

/// The end of the real retreat's rise ($A6:A994). Was `CeresRidley_A994`.
pub fn real_retreat_risen<L: Lease + ?Sized>(game: &mut L) {
    game.foreign(|game| {
        owned::real_retreat_risen(&mut crate::runtime_access::RuntimeAccess::runtime(game))
    })
}

/// Ridley function - real retreat - write palettes and spawn walls ($A6:A9A0).
/// Was `CeresRidley_Func_23`.
pub fn write_palettes_and_spawn_walls<L: Lease + ?Sized>(game: &mut L) {
    game.foreign(|game| {
        owned::write_palettes_and_spawn_walls(&mut crate::runtime_access::RuntimeAccess::runtime(
            game,
        ))
    })
}

/// Ridley function - real retreat - set up getaway cutscene ($A6:AA11). Was
/// `CeresRidley_Func_24`.
pub fn set_up_getaway<L: Lease + ?Sized>(game: &mut L) {
    game.foreign(|game| {
        owned::set_up_getaway(&mut crate::runtime_access::RuntimeAccess::runtime(game))
    })
}

/// Spawn walls during Ceres Ridley getaway cutscene ($A6:AA20). Was
/// `CeresRidley_Func_25`.
pub fn spawn_getaway_walls<L: Lease + ?Sized>(game: &mut L) {
    game.foreign(|game| {
        owned::spawn_getaway_walls(&mut crate::runtime_access::RuntimeAccess::runtime(game))
    })
}

/// Ridley function - real retreat - activated self destruct sequence
/// ($A6:AA50): the emergency text glows. Was `CeresRidley_Func_26`.
pub fn self_destruct_activated<L: Lease + ?Sized>(game: &mut L) {
    game.foreign(|game| {
        owned::self_destruct_activated(&mut crate::runtime_access::RuntimeAccess::runtime(game))
    })
}

/// Set up Ceres Ridley mode 7 ($A6:AA54). Was `CeresRidley_Func_27`.
pub fn set_up_mode7<L: Lease + ?Sized>(game: &mut L) {
    game.foreign(|game| {
        owned::set_up_mode7(&mut crate::runtime_access::RuntimeAccess::runtime(game))
    })
}

/// Handle Ceres Ridley getaway cutscene if active ($A6:AAAF). Was
/// `HandleCeresRidleyGetawayCutscene`.
pub fn handle_getaway_if_active<L: Lease + ?Sized>(game: &mut L) {
    game.foreign(|game| {
        owned::handle_getaway_if_active(&mut crate::runtime_access::RuntimeAccess::runtime(game))
    })
}

/// Handle Ceres Ridley getaway cutscene ($A6:AABD): Ridley flies off in mode 7
/// along the zoom and velocity tables; the push and quake start at 208, and
/// the self destruct sequence when the table ends. Was `CeresRidley_Func_28`.
pub fn handle_getaway<L: Lease + ?Sized>(game: &mut L) {
    game.foreign(|game| {
        owned::handle_getaway(&mut crate::runtime_access::RuntimeAccess::runtime(game))
    })
}

/// Set Ceres Ridley mode 7 rotation matrix ($A6:AB5F): rotation by the high
/// byte of the mode 7 angle, scaled by the zoom. Was `CeresRidley_Func_29`.
pub fn set_mode7_rotation<L: Lease + ?Sized>(game: &mut L) {
    game.foreign(|game| {
        owned::set_mode7_rotation(&mut crate::runtime_access::RuntimeAccess::runtime(game))
    })
}

/// 8-bit sine multiplication ($A6:AC30): `amplitude` times the sine of
/// `angle`, over 100h. Was `CeresRidley_AC30`.
pub fn sine_multiply(angle: u16, amplitude: i16) -> i16 {
    owned::sine_multiply(angle, amplitude)
}

/// Animate Ceres Ridley mode 7 baby metroid ($A6:ACBC): four frames, one
/// every fourth frame. Was `CeresRidley_Func_33`.
pub fn animate_mode7_baby_metroid<L: Lease + ?Sized>(game: &mut L) {
    game.foreign(|game| {
        owned::animate_mode7_baby_metroid(&mut crate::runtime_access::RuntimeAccess::runtime(game))
    })
}

/// Animate Ceres Ridley mode 7 wings ($A6:AD27): two frames, one every
/// eighth frame. Was `CeresRidley_Func_34`.
pub fn animate_mode7_wings<L: Lease + ?Sized>(game: &mut L) {
    game.foreign(|game| {
        owned::animate_mode7_wings(&mut crate::runtime_access::RuntimeAccess::runtime(game))
    })
}

/// Set Ceres Ridley palette according to zoom level ($A6:B0EF): the palette
/// by the zoom's high byte. Was `CeresRidley_Func_35`.
pub fn palette_by_zoom<L: Lease + ?Sized>(zoom: u16, game: &mut L) {
    game.foreign(|game| {
        owned::palette_by_zoom(
            zoom,
            &mut crate::runtime_access::RuntimeAccess::runtime(game),
        )
    })
}

/// Ridley function - real retreat - activating self destruct sequence
/// ($A6:C04E): the sequence step in the function timer. Was `Ridley_Func_54`.
pub fn self_destruct_activating<L: Lease + ?Sized>(game: &mut L) {
    game.foreign(|game| {
        owned::self_destruct_activating(&mut crate::runtime_access::RuntimeAccess::runtime(game))
    })
}

/// Self destruct sequence function index 0 - load text colours ($A6:C062).
/// Was `Ridley_Func_55`.
pub fn load_text_colours<L: Lease + ?Sized>(game: &mut L) {
    game.foreign(|game| {
        owned::load_text_colours(&mut crate::runtime_access::RuntimeAccess::runtime(game))
    })
}

/// Self destruct sequence function index 2 - loading escape timer sprite
/// tiles ($A6:C08E). Was `Ridley_C08E`.
pub fn load_escape_timer_tiles<L: Lease + ?Sized>(game: &mut L) {
    game.foreign(|game| {
        owned::load_escape_timer_tiles(&mut crate::runtime_access::RuntimeAccess::runtime(game))
    })
}

/// Self destruct sequence function index 4 - loading Ceres escape timer BG1/2
/// tiles ($A6:C09F). Was `Ridley_C09F`.
pub fn load_escape_background_tiles<L: Lease + ?Sized>(game: &mut L) {
    game.foreign(|game| {
        owned::load_escape_background_tiles(&mut crate::runtime_access::RuntimeAccess::runtime(
            game,
        ))
    })
}

/// Self destruct sequence function index 6 - "emergency" text appears
/// ($A6:C0BB): then the Japanese text, or straight to typing. Was
/// `Ridley_Func_56`.
pub fn emergency_text_appears<L: Lease + ?Sized>(game: &mut L) {
    game.foreign(|game| {
        owned::emergency_text_appears(&mut crate::runtime_access::RuntimeAccess::runtime(game))
    })
}

/// Self destruct sequence function index 8 - Japanese text appears ($A6:C0F5).
/// Was `Ridley_Func_57`.
pub fn japanese_text_appears<L: Lease + ?Sized>(game: &mut L) {
    game.foreign(|game| {
        owned::japanese_text_appears(&mut crate::runtime_access::RuntimeAccess::runtime(game))
    })
}

/// Self destruct sequence function index Ah - typing "self destruct sequence"
/// text ($A6:C104). Was `Ridley_C104`.
pub fn type_self_destruct_text<L: Lease + ?Sized>(game: &mut L) {
    game.foreign(|game| {
        owned::type_self_destruct_text(&mut crate::runtime_access::RuntimeAccess::runtime(game))
    })
}

/// Self destruct sequence function index Ch - start escape sequence
/// ($A6:C117): the timer starts and Ceres's boss is marked defeated. Was
/// `Ridley_Func_58`.
pub fn start_escape<L: Lease + ?Sized>(game: &mut L) {
    game.foreign(|game| {
        owned::start_escape(&mut crate::runtime_access::RuntimeAccess::runtime(game))
    })
}

// The named Ceres Ridley functions are the decompilation's; the getaway's
// RTS is its locret label.
const _: () = {
    use crate::actors::ceres_ridley::ceres_function as f;
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

#[path = "ceres_ridley_owned.rs"]
pub mod owned;

pub fn clear_initial_palettes<H: crate::session_access::SessionAccess + ?Sized>(
    norfair: bool,
    game: &mut H,
) {
    for index in 0..15 {
        if norfair {
            game.session_mut().presentation.visual_mut().targets[113 + index] = 0;
        }
        game.session_mut().presentation.visual_mut().targets[241 + index] = 0;
    }
}
