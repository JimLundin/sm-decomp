//! Frame dispatch, ordered gameplay phases and their resumable continuations.
//!
//! Samus preparation includes projectile work; enemy processing and a possible
//! message suspension separate it from movement/pose completion. Callbacks
//! observe live RAM. Only the entry resume point is held across those callbacks.
use crate::actors;
use crate::animated_tiles::execution as animated_tiles;
use crate::constants::game_states::*;
use crate::constants::loading_game_states::*;
use crate::demo_rooms;
use crate::display::preparation::Field as DisplayField;
use crate::display::preparation::Store;
use crate::display::transfer::Source;
use crate::display::video::{Command, Control, apply_mirrored};
use crate::display_refresh;
use crate::enemy_projectiles;
use crate::gameplay_rules::{finish_frame, finish_health_and_time};
use crate::menu_state::MenuState;
use crate::menu_state::{DeathSequence, MenuScreens, OptionsMenu, SaveSelection};
use crate::menus::prelude::CoroutineRet;
use crate::menus::{
    controller_input, file_menu, game_over, map_menu, options_settings, pause_controls,
    reserve_tanks, save_slots,
};
use crate::palette_effects::{execution as palette_effects, fading};
use crate::presentation_workspace::{ContinuationAccess as _, Point, Role};
use crate::room_entry::api::{ArrivalRequest, EntrySignal, StartSite};
use crate::room_entry::content::LoadMode;
use crate::room_scrolls::{advance_camera, debug_position, fill_viewport, update_scrolls};
use crate::runtime_access::{RuntimeAccess, with_session};
use crate::session_access::SessionAccess;
use crate::{escape_timer, hud, message_boxes, samus_death, sprite_drawing};

/// State callbacks the compatibility runtime still implements.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum NativeState {
    OpeningCinematic,
    CeresGoesBoom,
    EndingAndCredits,
}

/// The room hooks run when the pause screen opens.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum PauseHook {
    Draygon,
    Kraid,
}

/// The room hooks run when the pause screen closes; they may yield.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum UnpauseHook {
    Draygon,
    KraidDead,
    KraidAlive,
    KraidSinking,
    MotherBrain,
}

/// What the frame phases need beyond the menu, HUD, scroll and Session hosts:
/// the operations the compatibility runtime still implements.
pub trait Host:
    crate::menus::host::Host + crate::hud::HudHost + crate::display_refresh::Host + RuntimeAccess
{
    /// Present controller 1 to this frame's joypad reads, as $4218 reads it.
    fn latch_controller(&mut self, buttons: u16);
    /// Admit the owned Session for this frame; `fresh` when no pass is resuming.
    fn begin_session_interval(&mut self, fresh: bool);
    /// The gameplay pass resume byte ($7E:0612).
    fn resume_point(&self) -> u8;

    // Frame phase.
    fn resume_boot(&mut self) -> u8;
    /// The original's failure on an unknown frame state byte.
    fn fail_frame_state(&mut self);
    /// The original's warning when a pass leaves continuation bytes set.
    fn report_broken_coroutine(&mut self, state: [u8; 4]);
    fn resume_room_entry(&mut self, signal: EntrySignal) -> u8;
    fn debug_game_over_menu(&mut self);
    /// Emit this frame's queued gameplay sound effects.
    fn advance_sound_effects(&mut self);

    // Room transition phase.
    fn initialize_player(&mut self);

    // Demo phase.
    fn load_title_sequence_graphics(&mut self);
    fn disable_scanline_irq(&mut self);
    fn enable_scanline_irq(&mut self, immediate: bool);

    // Pause phase.
    fn pause_hook(&mut self, hook: PauseHook);
    fn unpause_hook(&mut self, hook: UnpauseHook) -> u8;
}

/// The gameplay pass phases the compatibility runtime runs on a lease the pass
/// already holds.
pub trait PassLease:
    crate::gameplay_context::Access
    + crate::presentation_workspace::ContinuationAccess
    + crate::presentation_workspace::Access
    + crate::presentation_workspace::PaletteAccess
    + crate::sequence_state::Access
    + crate::menus::state::Access
    + crate::menus::save_slots::SlotHost
    + crate::message_boxes::DialogueHost
    + crate::effects::DisplayRefresh
    + crate::effects::SoundEffects
    + crate::session_owners::AudioHost
    + crate::screen_fade::Owner
    + crate::palette_effects::fading::Host
    + crate::display::preparation::Access
    + crate::display::video::Video
    + crate::display_refresh::DisplayOwner
    + crate::hdma_objects::HdmaTable
    + crate::hdma_objects::UpdateHost
    + crate::ram::CompatibilityRam
    + crate::escape_timer::Host
    + crate::samus_death::Lease
{
    /// Samus's own projectiles against her: the bomb jump.
    fn player_projectiles_hit_samus(&mut self);
    /// Samus's frame: the frame handler's beta and gamma work.
    fn advance_player(&mut self);
    /// The room mechanisms' pass; nonzero when a message box suspends it.
    fn run_room_mechanisms(&mut self) -> u8;
    /// The room's main code.
    fn advance_room(&mut self);
    /// Emit this frame's queued gameplay sound effects.
    fn advance_sound_effects(&mut self);
    /// The original's warning when a pass leaves continuation bytes set.
    fn report_broken_coroutine(&mut self, state: [u8; 4]);
    /// A cinematic game state's frame: the title sequence, the intro or the
    /// ending; nonzero when it yields to the next frame.
    fn advance_native_state(&mut self, state: NativeState) -> u8;
    /// A door transition state's frame; nonzero when it yields.
    fn resume_room_entry(&mut self, signal: EntrySignal) -> u8;
    /// The reserve tanks state's low health warning.
    fn advance_low_health_warning(&mut self);
}

#[derive(Clone, Copy)]
enum StateEntry {
    Advance(Entry),
    Native(NativeState),
    DemoReset,
    FileMenu,
    MapMenu,
    PauseFadeIn,
    GameOver,
    BeginDeath,
    DeathPose,
    FlashDeath,
    ExplodeSuit,
    Door(EntrySignal),
    Demo,
    Options,
    Null,
    Unused,
    Debug,
}
use StateEntry::{
    Advance, BeginDeath, DeathPose, Debug, Demo, DemoReset, Door, ExplodeSuit, FileMenu,
    FlashDeath, GameOver, MapMenu, Native, Null, Options, PauseFadeIn, Unused,
};

static GAME_STATE_ENTRIES: [StateEntry; 45] = [
    DemoReset,
    Native(NativeState::OpeningCinematic),
    Options,
    Null,
    FileMenu,
    MapMenu,
    Advance(Entry::Initialize),
    Advance(Entry::FadeIn),
    Advance(Entry::Gameplay),
    Door(EntrySignal::DoorState9),
    Door(EntrySignal::DoorState10),
    Door(EntrySignal::DoorState11),
    Advance(Entry::PauseFadeOut),
    Advance(Entry::PauseSetup),
    PauseFadeIn,
    Advance(Entry::Paused),
    Advance(Entry::UnpauseFadeOut),
    Advance(Entry::UnpauseSetup),
    Advance(Entry::Unpause),
    Advance(Entry::PrepareDeath),
    Advance(Entry::DeathBlackout),
    BeginDeath,
    DeathPose,
    FlashDeath,
    ExplodeSuit,
    Advance(Entry::GameOverFadeOut),
    GameOver,
    Advance(Entry::ReserveTanks),
    Unused,
    Debug,
    Native(NativeState::CeresGoesBoom),
    Advance(Entry::Initialize),
    Advance(Entry::CeresElevator),
    Advance(Entry::CeresBlackout),
    Native(NativeState::CeresGoesBoom),
    Advance(Entry::TimeUp),
    Advance(Entry::TimeUpExit),
    Native(NativeState::CeresGoesBoom),
    Advance(Entry::ZebesEscape),
    Native(NativeState::EndingAndCredits),
    Advance(Entry::Initialize),
    Advance(Entry::DemoStart),
    Demo,
    Advance(Entry::DemoStop),
    Advance(Entry::DemoNext),
];

#[derive(Clone, Copy, PartialEq)]
#[repr(u8)]
enum ResumePoint {
    Start = 0,
    EnemyMessage = 1,
    RoomObjects = 2,
    // Existing bytes 3..=255 skip this pass and reset to Start on completion.
    Skip,
}

impl ResumePoint {
    fn of(byte: u8) -> Self {
        match byte {
            0 => Self::Start,
            1 => Self::EnemyMessage,
            2 => Self::RoomObjects,
            _ => Self::Skip,
        }
    }
}

#[derive(Clone, Copy)]
pub enum Entry {
    Frame,
    Initialize,
    Gameplay,
    FadeIn,
    Unpause,
    DemoStart,
    DemoStop,
    DemoNext,
    PauseSetup,
    Paused,
    UnpauseSetup,
    PauseFadeOut,
    UnpauseFadeOut,
    GameOverFadeOut,
    CeresElevator,
    CeresBlackout,
    ZebesEscape,
    TimeUp,
    TimeUpExit,
    ReserveTanks,
    PrepareDeath,
    DeathBlackout,
}

/// Advances through completion or a suspension, returning the callee's exact
/// yield byte. Nested PLM/message continuations remain owned by those modules.
/// Entry stays fixed even when callbacks change the live game state.
/// One frame's input from the device shell.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct TickInput {
    /// Controller 1, as the joypad register ($4218) reads it.
    pub buttons: u16,
}

/// Run one frame on `input`: the shell's step into the game.
pub fn tick<H: Host>(input: TickInput, game: &mut H) -> CoroutineRet {
    game.latch_controller(input.buttons);
    advance(Entry::Frame, game)
}

/// Run `entry`; a pass state's pass also ends the frame, on its last lease.
pub fn advance<H: Host>(entry: Entry, game: &mut H) -> CoroutineRet {
    match entry {
        Entry::Frame => frame(game),
        Entry::Initialize => initialize(game),
        Entry::DemoStop => stop_demo(game),
        Entry::DemoNext => next_demo(game),
        Entry::PauseSetup => setup_pause(game),
        Entry::Paused => advance_pause(game),
        Entry::UnpauseSetup => setup_unpause(game),
        Entry::UnpauseFadeOut | Entry::GameOverFadeOut => fade_out(entry, game),
        Entry::TimeUpExit => exit_time_up(game),
        Entry::Gameplay
        | Entry::FadeIn
        | Entry::Unpause
        | Entry::DemoStart
        | Entry::PauseFadeOut
        | Entry::CeresElevator
        | Entry::CeresBlackout
        | Entry::ZebesEscape
        | Entry::TimeUp
        | Entry::ReserveTanks
        | Entry::PrepareDeath
        | Entry::DeathBlackout => pass(entry, game),
    }
}

/// The states that run the gameplay pass.
fn runs_pass(entry: Entry) -> bool {
    matches!(
        entry,
        Entry::Gameplay
            | Entry::FadeIn
            | Entry::Unpause
            | Entry::DemoStart
            | Entry::PauseFadeOut
            | Entry::CeresElevator
            | Entry::CeresBlackout
            | Entry::ZebesEscape
            | Entry::TimeUp
            | Entry::ReserveTanks
            | Entry::PrepareDeath
            | Entry::DeathBlackout
    )
}

/// Frame phase: boot continuation, frame start, state dispatch and frame end.
fn frame<H: Host>(game: &mut H) -> CoroutineRet {
    game.begin_session_interval(game.resume_point() == 0);
    let returned = 'frame: {
        let mut st: i32 = game.outer_continuation() as i32;
        let begin_frame: bool = if (1..=3).contains(&st) {
            let returned = game.resume_boot();
            if returned != 0 {
                break 'frame returned;
            }
            true
        } else {
            st < 10 && {
                if st != 0 {
                    game.fail_frame_state();
                }
                true
            }
        };
        if begin_frame {
            game.clear_pass_continuations();
            // The HDMA object handler, with its music queue step, runs on the
            // frame's first lease. The main game loop's PLP restores the carry
            // loading the save slots ($80:8261) set.
            let mut lease = game.runtime();
            crate::hdma_objects::update_hdma(true, &mut lease);
            start_frame::<H>(&mut lease);
            st = game_state(&lease) as i32 + 10;
            lease.set_outer_continuation(st as u8);
            // A pass state runs from here to the frame's end on this lease: its
            // pre-pass work, the pass, an enemy's message box, the state's finish
            // and the frame's end.
            let entry = GAME_STATE_ENTRIES.get(usize::from((st - 10i32) as u16));
            if let Some(&Advance(entry)) = entry
                && runs_pass(entry)
            {
                let returned = run_pass::<H>(&mut lease, PassEnd::Frame(entry));
                drop(lease);
                break 'frame returned;
            }
            // A cinematic, door transition or death state's frame runs on this
            // lease too; the frame ends on Game once it returns.
            if let Some(&entry) = entry
                && runs_on_lease(entry)
            {
                let returned = advance_on_lease::<H>(entry, &mut lease);
                drop(lease);
                if returned != 0 {
                    break 'frame returned;
                }
                break 'frame finish_state(game);
            }
        }
        let returned = match GAME_STATE_ENTRIES[usize::from((st - 10i32) as u16)] {
            // A pass state's frame ends on its pass's last lease.
            Advance(entry) if runs_pass(entry) => break 'frame advance(entry, game),
            Advance(entry) => advance(entry, game),
            // A lease state resuming after a yield opens the frame's lease here.
            entry @ (Native(_) | Door(_) | BeginDeath | DeathPose | FlashDeath | ExplodeSuit) => {
                advance_on_lease::<H>(entry, &mut game.runtime())
            }
            DemoReset => demo_rooms::reset(game),
            FileMenu => file_menu::advance(game),
            MapMenu => map_menu::advance(game),
            PauseFadeIn => pause_controls::fade_in(game),
            GameOver => game_over::advance(game),
            Demo => demo_rooms::play(game, |game| advance_gameplay(PassEnd::Pass, game)),
            Null => 0,
            Unused => {
                game.rtl_unreachable();
                0
            }
            Debug => {
                game.debug_game_over_menu();
                0
            }
            Options => options_settings::advance(game, upload_options_tiles, |game| {
                crate::display_refresh::enable(true, game)
            }),
        };
        if returned != 0 {
            break 'frame returned;
        }
        finish_state(game)
    };
    if returned == 0 {
        game.store_display_control_byte(DisplayField::NmiPending, 1, Store::Named);
    }
    // A requested NMI ends the frame by latching the controller, as on hardware.
    let requested = game.display_control_byte(DisplayField::NmiPending) != 0;
    display_refresh::run(game);
    if requested {
        controller_input::sample_latched(game);
    }
    // A soft reset (the game over and file select quits, the debug reset chord)
    // jumps to $80:8462: the next frame resumes boot there, which clears bank $7E.
    let reset = crate::gameplay_word!(game.GAME_STATE) == SOFT_RESET_REQUESTED;
    if returned == 0 || reset {
        let resume = if reset { RESUME_SOFT_RESET } else { 0 };
        game.set_outer_continuation(resume);
    }
    returned
}

/// The states whose frame runs on the frame's first lease, up to the frame's
/// end.
fn runs_on_lease(entry: StateEntry) -> bool {
    matches!(
        entry,
        Native(_) | Door(_) | BeginDeath | DeathPose | FlashDeath | ExplodeSuit
    )
}

/// Run a lease state's frame on `lease`; nonzero when it yields.
fn advance_on_lease<'a, H: Host + 'a>(entry: StateEntry, lease: &mut H::Runtime<'a>) -> u8 {
    match entry {
        Native(state) => lease.advance_native_state(state),
        Door(signal) => lease.resume_room_entry(signal),
        BeginDeath => samus_death::begin_death(lease),
        DeathPose => samus_death::advance_death_pose(lease),
        FlashDeath => samus_death::flash_death(lease),
        ExplodeSuit => samus_death::explode_suit(lease),
        _ => unreachable!("not a lease state"),
    }
}

/// The frame's end on Game, after a completed state: the sound effects, the
/// sprite table's finish, the joypad filter word and the continuation check.
fn finish_state<H: Host>(game: &mut H) -> CoroutineRet {
    game.advance_sound_effects();
    sprite_drawing::finish_frame(game);
    filter_joypad(game);
    let state = [
        game.resume_point(),
        game.secondary_continuation(),
        game.demo_phase(),
        game.message_continuation(),
    ];
    if broken_coroutine(state) {
        game.report_broken_coroutine(state);
    }
    0
}

/// The frame's start on its first lease: the random number steps, the sprite
/// table's high table and cursor reset, and Samus's drawing requests clear.
fn start_frame<'a, H: Host + 'a>(lease: &mut H::Runtime<'a>) {
    let timing = &mut lease.session_mut().timing;
    timing.random_number = crate::random::next(timing.random_number);
    lease.words_stored();
    sprite_drawing::clear_high_table(lease);
    sprite_drawing::reset_cursor(lease);
    lease.session_mut().presentation.clear_samus_requests();
    lease.words_stored();
}

/// The frame's end, after a completed state, on the main gameplay pass's
/// lease: the sound effects, the sprite table's finish, the joypad filter word
/// and the continuation check.
fn close_frame<'a, H: Host + 'a>(lease: &mut H::Runtime<'a>) {
    lease.advance_sound_effects();
    sprite_drawing::finish_frame(lease);
    filter_joypad(lease);
    let state = [
        lease.screen_primary(),
        lease.secondary_continuation(),
        lease.demo_phase(),
        lease.message_continuation(),
    ];
    if broken_coroutine(state) {
        lease.report_broken_coroutine(state);
    }
}

/// Samus's joypad filter word takes the frame's held buttons.
fn filter_joypad<H: crate::gameplay_context::Access + ?Sized>(host: &mut H) {
    crate::gameplay_word!(
        host.JOYPAD1_INPUT_SAMUSFILTER = crate::gameplay_word!(host.JOYPAD1_LASTKEYS)
    );
}

/// The game state word ($0998).
fn game_state<H: crate::gameplay_context::Access + ?Sized>(host: &H) -> u16 {
    crate::gameplay_word!(host.GAME_STATE)
}

/// Whether a completed frame leaves any pass continuation byte set.
fn broken_coroutine(state: [u8; 4]) -> bool {
    state.iter().fold(0, |bits, byte| bits | byte) != 0
}

/// The pause screen's palette ($B6:F000), which replaces the gameplay colours.
const PAUSE_PALETTE: u32 = 0xb6f000;

/// Clear the projectile table words $198D..$1C1E: the table's flags, zoom,
/// selection and spawn words and every slot's words through `behavior_c`.
fn clear_projectile_table(table: &mut crate::session::EnemyProjectiles) {
    table.flags = 0;
    table.zoom = 0;
    table.selected = 0;
    table.spawn.primary = 0;
    table.spawn.secondary = 0;
    for slot in &mut table.slots {
        slot.definition = 0;
        slot.graphics = 0;
        slot.counter = 0;
        slot.pre_action = 0;
        slot.x.fraction = 0;
        slot.x.whole = 0;
        slot.y.fraction = 0;
        clear_slot_motion(slot);
        slot.spritemap = 0;
        slot.delay = 0;
        slot.radius = 0;
        slot.properties = 0;
        slot.behavior_c = 0;
    }
}

/// Clear the projectile words $1A8D..$1B8C: the last three slots' vertical
/// subpixels through the first seventeen slots' spritemaps.
fn clear_projectile_motion(slots: &mut [crate::session::EnemyProjectile; 18]) {
    for (index, slot) in slots.iter_mut().enumerate() {
        if index >= 15 {
            slot.y.fraction = 0;
        }
        clear_slot_motion(slot);
        if index < 17 {
            slot.spritemap = 0;
        }
    }
}

/// A slot's vertical position, velocity, behaviour words and program.
fn clear_slot_motion(slot: &mut crate::session::EnemyProjectile) {
    slot.y.whole = 0;
    slot.velocity.x = 0;
    slot.velocity.y = 0;
    slot.behavior_a = 0;
    slot.behavior_b = 0;
    slot.program = 0;
}

/// The game state a soft-reset request leaves for the frame's end.
const SOFT_RESET_REQUESTED: u16 = 0xffff;
/// The frame resume byte that enters boot at the soft reset.
const RESUME_SOFT_RESET: u8 = 3;

/// Room transition phase: initial room load and actor tile staging.
fn initialize<H: Host>(game: &mut H) -> CoroutineRet {
    crate::probe::hit("coroutine.init");
    let mut stage = game.screen_primary();
    let mut waiting = stage >= 2;
    if stage == 0 {
        let demo =
            u32::from(crate::gameplay_word!(game.GAME_STATE)) == K_GAME_STATE_40_TRANSITION_TO_DEMO;
        if !demo {
            crate::menus::state::with(game, |game, selection: &mut SaveSelection| {
                let loading = u32::from(selection.loading_game_state);
                if loading == K_LOADING_GAME_STATE_1F_STARTING_AT_CERES {
                    game.runtime().session_mut().room.area = 6;
                    selection.load_station = 0;
                    escape_timer::clear(game);
                } else if loading == K_LOADING_GAME_STATE_22_ESCAPING_CERES {
                    game.runtime().session_mut().room.area = 0;
                    selection.load_station = 18;
                    save_slots::load_area_map(selection, game);
                }
            });
        }
        display_refresh::initialize_gameplay(game);
        game.initialize_player();
        if demo {
            demo_rooms::load(game);
        } else {
            crate::room_entry::selection::begin(
                &mut game.runtime(),
                ArrivalRequest::Start(StartSite::CurrentLoadStation),
            );
        }
    }
    if stage <= 1 {
        let yielded = game.resume_room_entry(EntrySignal::Initial);
        if yielded != 0 {
            game.screen_set_primary(1);
            return yielded;
        }
        hud::initialize(game);
        with_session(game, |session| {
            let visual = session.presentation.visual_mut();
            visual.targets[192..208].copy_from_slice(&visual.colors[192..208]);
        });
        crate::screen_fade::store(game, 1, 1);
        crate::display_refresh::enable(true, game);
        // The enables and the liquid physics run on one lease.
        let mut lease = game.runtime();
        enemy_projectiles::lifecycle::set_enabled(&mut lease, true);
        set_mechanisms_enabled(&mut lease, true);
        palette_effects::set_enabled(lease.session_mut(), true);
        crate::hdma_objects::set_hdma_enabled(true, &mut lease);
        animated_tiles::set_enabled(lease.session_mut(), true);
        crate::player_movement::speed::set_liquid_physics(&mut lease);
        drop(lease);
        stage = if u32::from(crate::gameplay_word!(game.GAME_STATE))
            == K_GAME_STATE_40_TRANSITION_TO_DEMO
        {
            2
        } else if u32::from(crate::menus::state::load::<SaveSelection, _>(game).loading_game_state)
            == K_LOADING_GAME_STATE_22_ESCAPING_CERES
        {
            crate::music_requests::queue_delayed_8(5, game);
            3
        } else {
            4
        };
        with_session(game, |session| {
            session.wavy_samus.speed = if stage == 3 { 15 } else { 6 };
        });
    }
    if matches!(stage, 2..=4) {
        crate::probe::hit(match stage {
            2 => "coroutine.init.demo",
            3 => "coroutine.init.escape",
            _ => "coroutine.init.normal",
        });
        loop {
            if !waiting {
                actors::staging::transfer_actor_tiles_and_initialize(&mut game.runtime());
            }
            let yielded = crate::display_refresh::wait(true, game);
            if yielded != 0 {
                game.screen_set_primary(stage);
                return yielded;
            }
            let remaining = with_session(game, |session| {
                let counter = &mut session.wavy_samus.speed;
                *counter = counter.wrapping_sub(1);
                *counter
            });
            if (remaining & 0x8000) != 0 {
                break;
            }
            waiting = false;
        }
        if stage == 2 {
            demo_rooms::finish_setup(game);
        }
        if stage == 4 {
            crate::gameplay_word!(game.GAME_STATE = K_GAME_STATE_7_MAIN_GAMEPLAY_FADE_IN as u16);
        } else {
            crate::gameplay_word!(game.GAME_STATE += 1);
        }
        with_session(game, |session| {
            let visual = session.presentation.visual_mut();
            for (color, &target) in visual.colors.iter_mut().zip(&visual.targets) {
                *color = crate::word_transfers::stored_native(target);
            }
        });
        if stage == 4 {
            if u32::from(crate::menus::state::load::<SaveSelection, _>(game).loading_game_state)
                == K_LOADING_GAME_STATE_1F_STARTING_AT_CERES
            {
                with_session(game, |session| {
                    session.presentation.visual_mut().colors[223] = 0;
                });
                game.player_command(8);
            } else {
                game.player_command(9);
            }
        }
    }
    game.screen_set_primary(0);
    0
}

/// Demo phase: leave a finished demo.
fn stop_demo<H: Host>(game: &mut H) -> CoroutineRet {
    if crate::session_words::current(game).substate != 1 {
        demo_rooms::advance_set(game);
    }
    crate::display_refresh::enable(true, game);
    crate::gameplay_word!(game.GAME_STATE += 1);
    crate::screen_fade::store(game, 0, 0);
    crate::hdma_objects::clear_hdma(game);
    game.disable_scanline_irq();
    crate::sequence_word!(game.FX_LAYER_BLENDING_CONFIG_A = 0);
    {
        crate::frame_display_word!(game.CurrentInterrupt = 0);
        crate::frame_display_word!(game.PendingInterrupt = 0);
    }
    // The palette effects and the enemy projectile table clear on one lease.
    let mut lease = game.runtime();
    palette_effects::set_enabled(lease.session_mut(), false);
    palette_effects::clear(lease.session_mut());
    clear_projectile_table(&mut lease.session_mut().enemy_projectiles);
    lease.words_stored();
    drop(lease);
    // The original clears the block a word at a time, from the top down.
    for index in (0..crate::demo_clear::WORDS).rev() {
        game.set_view_word(crate::projection::Field::DemoClear(index), 0);
    }
    display_refresh::reset_layer_mirrors(game);
    0
}

/// Demo phase: choose the next demo or title step.
fn next_demo<H: Host>(game: &mut H) -> CoroutineRet {
    crate::gameplay_word!(game.GAME_STATE = K_GAME_STATE_1_OPENING_CINEMATIC as u16);
    let substate = crate::session_words::current(game).substate;
    if (substate & 0x8000) != 0 {
        crate::gameplay_word!(game.GAME_STATE = K_GAME_STATE_40_TRANSITION_TO_DEMO as u16);
    } else if substate != 0 {
        game.load_title_sequence_graphics();
        // The title reuses slot 4's X position as its skip-to-title index.
        crate::enemy_projectiles::table::with_slots(game, |slots| slots[4].x.whole = 2);
        crate::sequence_state::with_flow(game, |_, flow: &mut crate::flow::Flow| {
            flow.cinematic_function = 0x9a47
        });
    } else {
        crate::music_requests::queue_delayed_8(0, game);
        crate::sequence_state::with_words(
            game,
            |_, words: &mut crate::session_words::SessionWords| {
                words.debug_disable_sounds = 0;
            },
        );
        crate::sequence_state::with_flow(game, |_, flow: &mut crate::flow::Flow| {
            flow.cinematic_function = 0x9b68
        });
    }
    0
}

/// Pause phase: enter the pause screen.
fn setup_pause<H: Host>(game: &mut H) -> CoroutineRet {
    crate::hdma_objects::set_hdma_enabled(false, game);
    apply_mirrored(
        game,
        "pause",
        Control::HdmaChannels(0),
        DisplayField::HdmaEnable,
        true,
    );
    animated_tiles::set_enabled(game.runtime().session_mut(), false);
    display_refresh::capture_pause_background(game);
    match pause_controls::hook_address(true, game) {
        8946657 => {}
        9423094 => game.pause_hook(PauseHook::Draygon),
        10994469 => game.pause_hook(PauseHook::Kraid),
        _ => {
            game.rtl_unreachable();
        }
    }
    cancel_sound_effects(game, 6);
    pause_controls::save_graphics(game);
    display_refresh::load_pause_assets(game);
    display_refresh::upload_pause_map(game);
    for index in 0..256 {
        let value = game.palette_word(false, index * 2);
        game.screen_write_word(Point::new(Role::Pause, (384 + index) * 2), value);
    }
    let palette: [u16; 256] = crate::word_transfers::rom_words(game, PAUSE_PALETTE);
    with_session(game, |session| {
        session.presentation.visual_mut().colors = palette
    });
    display_refresh::pause_graphics(true, game);
    crate::menus::state::with(game, |game, menu: &mut MenuState| {
        pause_controls::initialize(menu, game)
    });
    0
}

/// Pause phase: one pause menu frame.
fn advance_pause<H: Host>(game: &mut H) -> CoroutineRet {
    crate::menus::state::with(game, |game, menu: &mut MenuState| {
        controller_input::filter(3, game);
        // Preserve the eight-entry bounds failure before any tab work.
        let tab = [0, 1, 2, 3, 4, 5, 6, 7][usize::from(menu.menu_index)];
        match tab {
            0 => pause_controls::update_map(menu, game),
            1 => pause_controls::update_equipment(menu, game),
            2 | 5 => pause_controls::fade_out_tab(menu, game, tab == 5, |game| {
                crate::display_refresh::enable(true, game);
            }),
            3 | 6 => pause_controls::load_tab(menu, game, tab == 3, |game| {
                if tab == 3 {
                    game.load_vram_words(0x7e3800, 0x3000, 0x800);
                } else {
                    display_refresh::upload_pause_map(game);
                }
            }),
            4 => pause_controls::fade_tab::<true, _>(menu, game),
            _ => pause_controls::fade_tab::<false, _>(menu, game),
        }
        hud::advance(game);
        map_menu::animate_palette(menu, game);
        0
    })
}

/// Pause phase: restore the room after the pause screen.
fn setup_unpause<H: Host>(game: &mut H) -> CoroutineRet {
    let resume = game.screen_primary();
    if resume == 0 {
        game.load_vram_words(0x9ad200, 0x6000, 0x1000);
        for index in 0..256 {
            let value = game.screen_word(Point::new(Role::Pause, (384 + index) * 2));
            game.palette_write_word(false, index * 2, value);
        }
        display_refresh::pause_graphics(false, game);
        pause_controls::resume(game);
        game.apply("unpause", Command::StopDma);
        crate::display_refresh::enable(false, game);
        game.disable_scanline_irq();
        // The room's tiles, palette, background, viewport and mechanism
        // graphics reload on one lease.
        let mut lease = game.runtime();
        crate::room_entry::installation::load_tiles_and_palette(&mut lease);
        crate::room_entry::background::load(&mut lease, LoadMode::Initial);
        fill_viewport(&mut lease);
        crate::room_mechanisms::graphics::reload_graphics(&mut lease);
        drop(lease);
        crate::display_refresh::enable(true, game);
        game.enable_scanline_irq(false);
        pause_controls::restore_graphics(game);
        game.load_vram_words(
            0x7edf5c,
            u16::from(
                game.display_control_byte(crate::display::preparation::Field::Screens1) & 0xfc,
            ) << 8,
            0x1000,
        );
        crate::screen_fade::store(game, 1, 1);
    }
    if resume <= 1 {
        let yielded = match pause_controls::hook_address(false, game) {
            8946657 => 0,
            9423100 => game.unpause_hook(UnpauseHook::Draygon),
            10994171 => game.unpause_hook(UnpauseHook::KraidDead),
            10994254 => game.unpause_hook(UnpauseHook::KraidAlive),
            10994336 => game.unpause_hook(UnpauseHook::KraidSinking),
            11110243 => game.unpause_hook(UnpauseHook::MotherBrain),
            _ => {
                game.rtl_unreachable();
                0
            }
        };
        if yielded != 0 {
            game.screen_set_primary(1);
            return yielded;
        }
        // HDMA, the animated tiles and Samus's movement sound resume on one lease.
        let mut lease = game.runtime();
        crate::hdma_objects::set_hdma_enabled(true, &mut lease);
        animated_tiles::set_enabled(lease.session_mut(), true);
        crate::player_projectiles::feedback::restore_sound(&mut lease);
        drop(lease);
        crate::gameplay_word!(game.GAME_STATE += 1);
    }
    game.screen_set_primary(0);
    0
}

/// Pause and death phases: fade to black before unpausing or game over.
fn fade_out<H: Host>(entry: Entry, game: &mut H) -> CoroutineRet {
    if matches!(entry, Entry::UnpauseFadeOut) {
        crate::menus::state::with(game, |game, menu: &mut MenuState| {
            pause_controls::draw_unpause(menu, game)
        });
    }
    game.fade(false);
    if game.display_control_byte(crate::display::preparation::Field::Brightness) == 0x80 {
        crate::display_refresh::enable(true, game);
        crate::screen_fade::store(game, 0, 0);
        crate::gameplay_word!(game.GAME_STATE += 1);
        if matches!(entry, Entry::GameOverFadeOut) {
            crate::menus::state::with(game, |_, menu: &mut MenuState| menu.menu_index = 0);
            crate::sequence_state::with_words(
                game,
                |_, words: &mut crate::session_words::SessionWords| {
                    words.debug_disable_sounds = 0;
                },
            );
        }
    }
    0
}

/// Death phase: leave the time-up whiteout.
fn exit_time_up<H: Host>(game: &mut H) -> CoroutineRet {
    game.fade(false);
    if game.display_control_byte(crate::display::preparation::Field::Brightness) == 0x80 {
        crate::display_refresh::enable(true, game);
        finish_gameplay(true, game);
        crate::screen_fade::store(game, 0, 0);
        crate::sequence_state::with_words(
            game,
            |_, words: &mut crate::session_words::SessionWords| {
                words.ceres_status = 0;
                words.timer_status = 0;
            },
        );
        cancel_sound_effects(game, 15);
        if crate::progress::event_happened(&mut game.runtime(), 0xe) {
            crate::menus::state::with(game, |_, options: &mut OptionsMenu| {
                options.screen_index = 0
            });
            crate::menus::state::with(game, |_, menu: &mut MenuState| menu.menu_index = 0);
            crate::enemy_projectiles::table::with_slots(game, clear_projectile_motion);
            crate::gameplay_word!(game.GAME_STATE = K_GAME_STATE_25_SAMUS_NO_HEALTH as u16);
        } else {
            crate::gameplay_word!(
                game.GAME_STATE = K_GAME_STATE_37_CERES_GOES_BOOM_WITH_SAMUS as u16
            );
            crate::sequence_state::with_flow(game, |_, flow: &mut crate::flow::Flow| {
                flow.cinematic_function = 0xc11b
            });
        }
    }
    0
}

/// What a completed pass's lease runs after the pass's own bookkeeping.
#[derive(Clone, Copy)]
enum PassEnd {
    /// Nothing more: the demo's play loop continues its frame.
    Pass,
    /// The state's per-state finish, then the frame's end.
    Frame(Entry),
}

/// States that run the gameplay pass: their pre-pass work, the pass, their
/// per-state finish and the frame's end, on one lease.
fn pass<H: Host>(entry: Entry, game: &mut H) -> CoroutineRet {
    advance_gameplay(PassEnd::Frame(entry), game)
}

/// Frame phase: the resumable gameplay pass shared by playing states, on one
/// lease.
fn advance_gameplay<H: Host>(end: PassEnd, game: &mut H) -> CoroutineRet {
    run_pass::<H>(&mut game.runtime(), end)
}

/// The gameplay pass on `lease`: a pass state's pre-pass work, then the pass
/// from the resume point, then `end`.
fn run_pass<'a, H: Host + 'a>(lease: &mut H::Runtime<'a>, end: PassEnd) -> CoroutineRet {
    if let PassEnd::Frame(entry) = end {
        prepare_pass::<H>(entry, lease);
    }
    crate::probe::hit("coroutine.gameplay");
    let resume = ResumePoint::of(lease.screen_primary());
    pass_on::<H>(lease, resume, end)
}

/// A pass state's work before its pass: the Ceres escape timer, and the
/// reserve tanks' automatic refill.
fn prepare_pass<'a, H: Host + 'a>(entry: Entry, lease: &mut H::Runtime<'a>) {
    if matches!(entry, Entry::CeresElevator | Entry::CeresBlackout)
        && crate::session_words::current(lease).timer_status != 0
    {
        escape_timer::draw(lease);
    }
    if matches!(entry, Entry::ReserveTanks)
        && lease.screen_primary() == 0
        && reserve_tanks::refill_auto(lease) != 0
    {
        crate::sequence_word!(lease.TIME_IS_FROZEN_FLAG = 0);
        crate::gameplay_word!(lease.GAME_STATE = K_GAME_STATE_8_MAIN_GAMEPLAY as u16);
        crate::player_transitions::command(
            lease,
            0x10,
            crate::player_transitions::commands::finish_ceres_arrival,
        );
    }
}

/// The pass on `lease` from `resume`: from its start, after an enemy's message
/// box, or from the room mechanisms. Once it completes, the frame's health,
/// time and timer bookkeeping, the resume point and `end`. Nonzero when the
/// message box or the mechanisms yield.
fn pass_on<'a, H: Host + 'a>(
    lease: &mut H::Runtime<'a>,
    resume: ResumePoint,
    end: PassEnd,
) -> CoroutineRet {
    let player = match resume {
        ResumePoint::Skip => {
            lease.screen_set_primary(ResumePoint::Start as u8);
            end_pass::<H>(end, lease);
            return 0;
        }
        ResumePoint::Start => {
            begin_pass::<H>(lease);
            // Read the queue after enemy callbacks, even when they changed it this pass.
            if lease.session().messages.queued == 0 {
                return complete_pass::<H>(lease, true, end);
            }
            true
        }
        ResumePoint::EnemyMessage => true,
        ResumePoint::RoomObjects => false,
    };
    if player {
        let message = crate::session_words::current(lease).queued_message_box_index;
        let yielded = message_boxes::display(message, lease);
        if yielded != 0 {
            lease.screen_set_primary(ResumePoint::EnemyMessage as u8);
            return yielded;
        }
        crate::sequence_state::with_words(
            lease,
            |_, words: &mut crate::session_words::SessionWords| {
                words.queued_message_box_index = 0;
            },
        );
    }
    complete_pass::<H>(lease, player, end)
}

/// The rest of the pass and, once it completes, the frame's health, time and
/// timer bookkeeping, the resume point and `end`, on one lease. Nonzero when
/// the mechanisms yield.
fn complete_pass<'a, H: Host + 'a>(
    lease: &mut H::Runtime<'a>,
    player: bool,
    end: PassEnd,
) -> CoroutineRet {
    let yielded = continue_pass::<H>(lease, player);
    if yielded != 0 {
        lease.screen_set_primary(ResumePoint::RoomObjects as u8);
        return yielded;
    }
    finish_health_and_time(lease);
    finish_frame(lease);
    lease.screen_set_primary(ResumePoint::Start as u8);
    end_pass::<H>(end, lease);
    0
}

/// A completed pass's `end`: the state's finish and the frame's end.
fn end_pass<'a, H: Host + 'a>(end: PassEnd, lease: &mut H::Runtime<'a>) {
    if let PassEnd::Frame(entry) = end {
        finish_pass(entry, lease);
        close_frame::<H>(lease);
    }
}

/// The pass's first phases: actor selection, the palette effects, Samus's
/// projectiles and the enemies.
fn begin_pass<'a, H: Host + 'a>(lease: &mut H::Runtime<'a>) {
    actors::scheduling::select(lease);
    palette_effects::advance(lease);
    prepare_player_projectiles(lease);
    if lease.session().actors.diagnostics.sprite_contacts_disabled == 0 {
        lease.player_projectiles_hit_samus();
    }
    actors::scheduling::advance_selected(lease);
}

/// The rest of the pass on one lease: Samus's frame and the enemy projectiles
/// when `player`, the room mechanisms, and the animated tiles, contacts, camera,
/// scene, HUD, scrolls and room code after them. Nonzero when the mechanisms
/// yield.
fn continue_pass<'a, H: Host + 'a>(lease: &mut H::Runtime<'a>, player: bool) -> CoroutineRet {
    if player {
        lease.advance_player();
        enemy_projectiles::lifecycle::advance(lease);
    }
    let yielded = lease.run_room_mechanisms();
    if yielded != 0 {
        return yielded;
    }
    animated_tiles::advance(lease);
    // This is a new read: earlier callbacks may change interaction policy.
    if lease.session().actors.diagnostics.sprite_contacts_disabled == 0 {
        enemy_projectiles::contact::enemy_projectiles_hit_samus(lease);
        enemy_projectiles::contact::shoot_enemy_projectiles(lease);
        actors::contacts::apply_power_bomb_to_enemies(lease);
    }
    advance_camera(lease);
    let debug_scrolling_enabled: i32 = 0;
    if debug_scrolling_enabled != 0 {
        debug_position(lease);
    }
    actors::drawing::draw_scene(lease);
    // The HUD reads Samus's owned values after the actor tiles.
    actors::staging::queue_actor_tiles(lease);
    hud::advance(lease);
    update_scrolls(lease);
    lease.advance_room();
    0
}

/// Prepare Samus's projectiles through the frame handler's entry: normal,
/// demo, projectiles only or inactive.
fn prepare_player_projectiles(lease: &mut impl crate::runtime_access::PlayerLease) {
    use crate::session::PrepareEntry;
    let handler = lease.session().player.transitions.frame_alpha.to_raw();
    let entry = match 0x900000 | u32::from(handler) {
        0x90e695 => PrepareEntry::Normal,
        0x90e6c9 => PrepareEntry::Demo,
        0x90e713 => PrepareEntry::ProjectilesOnly,
        0x90e8cd => PrepareEntry::Inactive,
        _ => {
            crate::player_projectiles::host::Host::unknown(lease);
            return;
        }
    };
    crate::player_projectiles::preparation::prepare(lease, entry);
}

/// Upload the options screen tiles.
fn upload_options_tiles<H: Host>(game: &mut H) {
    // The original also stores the VRAM address's low byte (0) to the colour
    // math mirror, before the port.
    game.store_display_control_byte(DisplayField::ColorMath, 0, Store::Named);
    game.apply(
        "options tiles",
        Command::TilesNowBytePort {
            address: 0x5800,
            source: Source {
                address: 0xdc00,
                bank: 0x8e,
            },
            bytes: 0x800,
        },
    );
}

/// Clear HDMA and the scanline IRQ, then reset the gameplay layers.
fn finish_gameplay<H>(clear_current_irq: bool, game: &mut H)
where
    H: crate::display::video::Video
        + crate::display::preparation::Access
        + crate::display_refresh::DisplayOwner
        + crate::hdma_objects::HdmaTable
        + crate::ram::CompatibilityRam
        + crate::sequence_state::Access
        + ?Sized,
{
    crate::hdma_objects::clear_hdma(game);
    display_refresh::disable_scanline_irq(game);
    display_refresh::reset_gameplay_layers(clear_current_irq, game);
}

/// Cancel the three sound channels, keeping at most `max_queued` queued.
fn cancel_sound_effects<H: crate::effects::SoundEffects + ?Sized>(game: &mut H, max_queued: u8) {
    for (channel, sound) in [2, 0x71, 1].into_iter().enumerate() {
        game.queue_sound(channel, sound, max_queued);
    }
}

fn set_mechanisms_enabled(lease: &mut impl SessionAccess, enabled: bool) {
    let flags = lease.session().mechanisms.flags;
    lease.session_mut().mechanisms.flags = if enabled {
        flags | 0x8000
    } else {
        flags & !0x8000
    };
}

/// Start the death sequence's progress: its first phase after a three-frame
/// delay, with the death pose's flash timer at `flash_timer`.
fn begin_death_sequence<H: crate::menus::state::Access + ?Sized>(game: &mut H, flash_timer: u16) {
    crate::menus::state::with(game, |_, screens: &mut MenuScreens| {
        screens.options_menu.screen_index = 3;
        screens.death = DeathSequence {
            index: 0,
            counter: 0,
            flash_timer,
        };
    });
}

/// Per-state work after a completed gameplay pass, on the pass's lease.
fn finish_pass<L: PassLease + SessionAccess + ?Sized>(entry: Entry, game: &mut L) {
    match entry {
        Entry::FadeIn | Entry::Unpause => {
            game.fade(true);
            if game.display_control_byte(crate::display::preparation::Field::Brightness) == 15 {
                crate::screen_fade::store(game, 0, 0);
                if matches!(entry, Entry::FadeIn) {
                    crate::gameplay_word!(game.GAME_STATE += 1);
                } else {
                    crate::gameplay_word!(game.GAME_STATE = K_GAME_STATE_8_MAIN_GAMEPLAY as u16);
                }
            }
        }
        Entry::CeresElevator => {
            let timer = crate::sequence_state::with_words(
                game,
                |_, words: &mut crate::session_words::SessionWords| {
                    let timer = &mut words.reached_ceres_elevator_fade_timer;
                    *timer = timer.wrapping_sub(1);
                    *timer
                },
            );
            if timer == 0 || timer & 0x8000 != 0 {
                crate::gameplay_word!(game.GAME_STATE += 1);
                crate::screen_fade::store(game, 0, 0);
            }
        }
        Entry::TimeUp | Entry::DeathBlackout => {
            let death = matches!(entry, Entry::DeathBlackout);
            if fading::all(if death { 6 } else { 8 }, game) != 0 {
                if death {
                    finish_gameplay(true, game);
                    begin_death_sequence(game, 16);
                    crate::screen_fade::store(game, 0, 0);
                    clear_projectile_motion(&mut game.session_mut().enemy_projectiles.slots);
                    crate::gameplay_word!(game.GAME_STATE += 1);
                    game.session_mut().projectiles.power_bomb.explosion_status = 0;
                    game.words_stored();
                    cancel_sound_effects(game, 15);
                    crate::music_requests::queue_delayed_8(0, game);
                    crate::music_requests::queue_delayed_8(0xff39, game);
                    crate::music_requests::queue_delayed(5, 0xe, game);
                } else {
                    crate::gameplay_word!(
                        game.GAME_STATE = K_GAME_STATE_36_WHITING_OUT_FROM_TIME_UP as u16
                    );
                    crate::screen_fade::store(game, 0, 0);
                }
            }
        }
        Entry::ReserveTanks => {
            game.advance_low_health_warning();
        }
        Entry::PrepareDeath => {
            for index in (0..256).rev() {
                let value = crate::presentation_workspace::PaletteAccess::palette_word(
                    game,
                    false,
                    index * 2,
                );
                game.screen_write_word(Point::new(Role::Pause, (384 + index) * 2), value);
            }
            // Keep palette 6's sprite colours as the death sequence's targets.
            let visual = game.session_mut().presentation.visual_mut();
            for (index, target) in visual.targets.iter_mut().enumerate() {
                *target = if (192..208).contains(&index) {
                    visual.colors[index]
                } else {
                    0
                };
            }
            game.words_stored();
            begin_death_sequence(game, 0);
            crate::sequence_state::with_words(
                game,
                |_, words: &mut crate::session_words::SessionWords| {
                    words.hud_item_index = 0;
                    words.samus_auto_cancel_hud_item_index = 0;
                    words.samus_invincibility_timer = 0;
                    words.samus_knockback_timer = 0;
                },
            );
            crate::gameplay_word!(game.GAME_STATE += 1);
        }
        Entry::CeresBlackout | Entry::ZebesEscape => {
            game.fade(false);
            if game.display_control_byte(crate::display::preparation::Field::Brightness) == 0x80 {
                let ceres = matches!(entry, Entry::CeresBlackout);
                crate::display_refresh::enable(true, game);
                finish_gameplay(ceres, game);
                crate::screen_fade::store(game, 0, 0);
                if ceres {
                    let slot =
                        crate::menus::state::with(game, |_, selection: &mut SaveSelection| {
                            selection.loading_game_state =
                                K_LOADING_GAME_STATE_22_ESCAPING_CERES as u16;
                            selection.selected_slot
                        });
                    crate::gameplay_word!(game.GAME_STATE = 34);
                    save_slots::save(slot, game);
                    crate::sequence_state::with_flow(game, |_, flow: &mut crate::flow::Flow| {
                        flow.cinematic_function = 0xc11b
                    });
                    crate::sequence_state::with_words(
                        game,
                        |_, words: &mut crate::session_words::SessionWords| {
                            words.ceres_status = 0;
                            words.timer_status = 0;
                        },
                    );
                } else {
                    crate::gameplay_word!(
                        game.GAME_STATE = K_GAME_STATE_39_ENDING_AND_CREDITS as u16
                    );
                    crate::sequence_state::with_flow(game, |_, flow: &mut crate::flow::Flow| {
                        flow.cinematic_function = 0xd480
                    });
                    crate::sequence_state::with_words(
                        game,
                        |_, words: &mut crate::session_words::SessionWords| {
                            words.timer_status = 0;
                        },
                    );
                }
                crate::music_requests::queue_delayed_8(0, game);
                cancel_sound_effects(game, 15);
            }
        }
        Entry::PauseFadeOut => {
            game.fade(false);
            if game.display_control_byte(crate::display::preparation::Field::Brightness) & 0xf == 0
            {
                crate::display_refresh::enable(true, game);
                crate::screen_fade::store(game, 0, 0);
                crate::gameplay_word!(game.GAME_STATE += 1);
            }
        }
        Entry::DemoStart => {
            crate::hdma_objects::update_hdma(true, game);
            crate::gameplay_word!(game.GAME_STATE += 1);
            game.store_display_control_byte(DisplayField::Brightness, 15, Store::Named);
        }
        Entry::Frame
        | Entry::DemoStop
        | Entry::DemoNext
        | Entry::PauseSetup
        | Entry::Paused
        | Entry::UnpauseSetup
        | Entry::TimeUpExit
        | Entry::UnpauseFadeOut
        | Entry::GameOverFadeOut
        | Entry::Initialize
        | Entry::Gameplay => {}
    }
}
