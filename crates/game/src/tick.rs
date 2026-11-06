//! The main strand's dispatcher (D10): it steps the innermost continuation frame until the
//! strand reaches its beam-sync point (D7) or yields at a wait.
//!
//! Each frame's step runs its routine's phases from where the frame says it stopped and
//! returns a [`Next`]. The phases are in the original's call order; slice 0's are empty until
//! their domain lanes fill them.

use crate::animation;
use crate::collision::{Room, ScanOrder, Surroundings, Touches, inside};
use crate::message_box;
use crate::movement;
use crate::pose::{self, Content};
use crate::scrolling;
use crate::session::{Rules, SessionFault};
use crate::state::samus::SuperSpecial;
use crate::state::strand::{Condition, Frame, GameplayAt, MAIN_STRAND_DEPTH, MainLoopAt, Wait};
use crate::state::video::SamusTileUploads;
use crate::state::{GameMode, NmiTiming, State};

/// What a frame's step asks the dispatcher to do next.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Next {
    /// Step the same frame again from its new position.
    Continue,
    /// Push this frame and step it; the caller resumes where its own position says.
    Call(Frame),
    /// The frame's routine has returned: pop it and resume its caller.
    Return,
    /// The beam-sync point: the scan-out of the previous picture runs here (D7).
    Sync,
    /// The strand yields at a wait until an NMI answers it.
    Yield(Wait),
}

/// How far a run of the dispatcher goes.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Until {
    /// To the innermost frame's beam-sync point, or a yield before it.
    BeamSync,
    /// To the next yield.
    Yield,
}

/// The most frame steps one Tick may take. A strand that hasn't yielded by then is an engine
/// fault, not a hang: one main-loop iteration with slice 0's frames takes fewer than ten.
pub const STEP_BUDGET: u16 = 256;

/// Where the scan-out runs relative to a frame's per-Tick work (D7).
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum BeamSync {
    /// The frame reports [`Next::Sync`] from inside its work.
    Within,
    /// Before the frame does any work this Tick.
    BeforeWork,
}

/// The frame's beam-sync point.
///
/// The main loop's is after the HDMA object handler, because on
/// the console that handler finishes in vblank before line 0; every other frame's is before
/// its work until a measurement places it (D7).
#[must_use]
pub const fn beam_sync(frame: Frame) -> BeamSync {
    match frame {
        Frame::MainLoop(_) => BeamSync::Within,
        Frame::Gameplay(_) | Frame::Plms { .. } | Frame::MessageBox(_) => BeamSync::BeforeWork,
    }
}

/// Whether the main strand runs this Tick: it does unless it waits on a wait that the last
/// vblank's NMI didn't answer (an overrun or a vblank with NMIs off, D8).
#[must_use]
pub const fn resumes(state: &State) -> bool {
    match state.main.waiting {
        None => true,
        Some(_) => matches!(state.timing.last, NmiTiming::OnTime),
    }
}

/// Resume the main strand after an answered wait.
///
/// A `Wait::Until` tests its condition first: while it is false the strand yields again
/// without running a frame, as the original's "wait for NMI, then test" loops do.
///
/// Returns whether the strand is free to step.
pub const fn resume(state: &mut State) -> bool {
    if !resumes(state) {
        return false;
    }
    match state.main.waiting.take() {
        Some(Wait::Until(condition)) if !holds(condition, state) => {
            yield_at(state, Wait::Until(condition));
            false
        }
        Some(Wait::Nmi | Wait::LagFrame | Wait::Until(_)) | None => true,
    }
}

/// Whether a test-each-Tick wait's condition holds.
const fn holds(condition: Condition, _state: &State) -> bool {
    match condition {
        // open: the sound domain's music queue (C11); until it exists the queue is empty.
        Condition::MusicQueueEmpty => true,
    }
}

/// The strand stops at `wait`. A wait for a requested NMI sets the request, as the
/// original's wait-for-NMI routine does before it spins.
const fn yield_at(state: &mut State, wait: Wait) {
    match wait {
        Wait::Nmi | Wait::Until(_) => state.video.nmi_requested = true,
        Wait::LagFrame => {}
    }
    state.main.waiting = Some(wait);
}

/// Step the main strand until `until`, charging each step to `budget`.
///
/// # Errors
///
/// A [`SessionFault`] when the strand empties, nests deeper than [`MAIN_STRAND_DEPTH`] or
/// overruns [`STEP_BUDGET`].
pub fn run(
    state: &mut State,
    rules: Rules,
    content: &Content,
    until: Until,
    budget: &mut u16,
) -> Result<(), SessionFault> {
    if state.main.waiting.is_some() {
        return Ok(());
    }
    loop {
        let top = *state.main.frames.last().ok_or(SessionFault::StrandEmpty)?;
        if until == Until::BeamSync && beam_sync(top) == BeamSync::BeforeWork {
            return Ok(());
        }
        *budget = budget
            .checked_sub(1)
            .ok_or(SessionFault::StepBudgetExceeded)?;
        match step_once(state, rules, content)? {
            Next::Sync if until == Until::BeamSync => return Ok(()),
            Next::Yield(_) => return Ok(()),
            Next::Continue | Next::Call(_) | Next::Return | Next::Sync => {}
        }
    }
}

/// Step the innermost frame once and apply its [`Next`] to the stack.
///
/// # Errors
///
/// A [`SessionFault`] when the strand is empty or a call would nest it too deep.
pub fn step_once(state: &mut State, rules: Rules, content: &Content) -> Result<Next, SessionFault> {
    let top = *state.main.frames.last().ok_or(SessionFault::StrandEmpty)?;
    let (at, next) = step(top, state, rules, content)?;
    if let Some(innermost) = state.main.frames.last_mut() {
        *innermost = at;
    }
    match next {
        Next::Continue | Next::Sync => {}
        Next::Call(frame) => {
            if state.main.frames.len() >= MAIN_STRAND_DEPTH {
                return Err(SessionFault::StrandTooDeep);
            }
            state.main.frames.push(frame);
        }
        Next::Return => {
            state.main.frames.pop();
            if state.main.frames.is_empty() {
                return Err(SessionFault::StrandEmpty);
            }
        }
        Next::Yield(wait) => yield_at(state, wait),
    }
    Ok(next)
}

/// One step of `frame`: the frame's position afterwards and what comes next.
fn step(
    frame: Frame,
    state: &mut State,
    rules: Rules,
    content: &Content,
) -> Result<(Frame, Next), SessionFault> {
    Ok(match frame {
        Frame::MainLoop(at) => {
            let (at, next) = main_loop(at, state, rules);
            (Frame::MainLoop(at), next)
        }
        Frame::Gameplay(at) => {
            let (at, next) = gameplay(at, state, content)?;
            (Frame::Gameplay(at), next)
        }
        Frame::MessageBox(at) => {
            let (at, next) = message_box::step(at, state);
            (Frame::MessageBox(at), next)
        }
        // The PLM pass, called out of by an item's pickup.
        Frame::Plms { slot } => {
            let population = content.room_plms.of(state.room.room);
            let (slot, next) = crate::plm::resume(state, population, slot);
            (Frame::Plms { slot }, next)
        }
    })
}

/// A phase of the main game loop, in the original's order.
///
/// Bank `$82` log, main game loop `$82:8948`: the HDMA object handler (`$88:84B9`, which runs
/// the music queue first), the RNG (`$80:8111`), the high-OAM clear (`$80:8B1A`), the OAM stack
/// pointer and Samus tile transfer reset, the game state dispatch through the table at
/// `$82:8981`, the sound effect handler (`$82:89EF`), the OAM finaliser (`$80:896E`), the spare
/// CPU display and controller input update (`$82:8AB0`), then wait for NMI (`$80:8338`).
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum MainLoopPhase {
    /// The HDMA object handler and the music queue.
    HdmaObjects,
    /// One RNG call.
    Rng,
    /// Clear high OAM.
    ClearHighOam,
    /// Reset the OAM stack and the Samus tile transfer flags.
    ResetOamAndSamusTransfers,
    /// The sound effect handler.
    SoundEffects,
    /// Finalise OAM.
    FinaliseOam,
    /// The spare-CPU display and the controller input update.
    SpareCpuAndInput,
}

/// The phases between the beam-sync point and the game state dispatch.
const BEFORE_GAME_STATE: [MainLoopPhase; 3] = [
    MainLoopPhase::Rng,
    MainLoopPhase::ClearHighOam,
    MainLoopPhase::ResetOamAndSamusTransfers,
];

/// The phases after the game state dispatch, before the wait for NMI.
const AFTER_GAME_STATE: [MainLoopPhase; 3] = [
    MainLoopPhase::SoundEffects,
    MainLoopPhase::FinaliseOam,
    MainLoopPhase::SpareCpuAndInput,
];

/// One step of the main loop from `at`.
fn main_loop(at: MainLoopAt, state: &mut State, rules: Rules) -> (MainLoopAt, Next) {
    match at {
        MainLoopAt::Start => {
            main_loop_phase(MainLoopPhase::HdmaObjects, state, rules);
            (MainLoopAt::AfterHdmaObjects, Next::Sync)
        }
        MainLoopAt::AfterHdmaObjects => {
            for phase in BEFORE_GAME_STATE {
                main_loop_phase(phase, state, rules);
            }
            (
                MainLoopAt::AfterGameState,
                Next::Call(game_state(state.mode)),
            )
        }
        MainLoopAt::AfterGameState => {
            for phase in AFTER_GAME_STATE {
                main_loop_phase(phase, state, rules);
            }
            (MainLoopAt::Start, Next::Yield(Wait::Nmi))
        }
    }
}

/// The frame the game state dispatch calls.
const fn game_state(mode: GameMode) -> Frame {
    match mode {
        GameMode::Gameplay => Frame::Gameplay(GameplayAt::Start),
    }
}

/// A main-loop phase's work.
fn main_loop_phase(phase: MainLoopPhase, state: &mut State, _rules: Rules) {
    match phase {
        MainLoopPhase::Rng => {
            state.rng.advance();
        }
        MainLoopPhase::ClearHighOam => state.video.batch.clear_high_oam(),
        MainLoopPhase::ResetOamAndSamusTransfers => {
            state.video.batch.restart_sprites();
            state.video.batch.samus_tiles = SamusTileUploads::default();
        }
        MainLoopPhase::FinaliseOam => state.video.batch.finalise_oam(),
        // open: the HDMA objects and music queue (B6, C11), the sound effects (C11), and the
        // spare-CPU display with its previous-pad update.
        MainLoopPhase::HdmaObjects
        | MainLoopPhase::SoundEffects
        | MainLoopPhase::SpareCpuAndInput => {}
    }
}

/// A phase of game state 8's gameplay pass, in the original's order.
///
/// Bank `$82` log, main gameplay `$82:8B44`: enemy processing selection (`$A0:8EB6`), the
/// debug handler (`$B4:9809`, dead in vanilla), palette FX (`$8D:C527`), Samus's current state
/// (`$90:E692`), Samus and projectile interaction (`$A0:9785`), the enemies (`$A0:8FD4`),
/// Samus's new state (`$90:E722`), enemy projectiles (`$86:8104`), PLMs (`$84:85B4`), animated
/// tiles (`$87:8064`), enemy projectile and Samus collision (`$A0:9894`), enemy projectile and
/// projectile collision (`$A0:996C`), enemy power bomb interaction (`$A0:A306`), scrolling
/// (`$90:94EC`), drawing (`$A0:884D`), the enemy BG2 tilemap queue (`$A0:9726`), the HUD
/// (`$80:9B44`), the layer 2 and BG scroll (`$80:A3AB`), the room's main code (`$8F:E8BD`),
/// health and game time (`$82:DB69`), room shaking (`$A0:8687`) and the hurt timers
/// (`$A0:9169`). The three interaction phases run only while sprite interactions are enabled.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum GameplayPhase {
    /// Choose which enemies process.
    EnemySelection,
    /// Palette FX.
    PaletteFx,
    /// Samus's current state.
    SamusCurrentState,
    /// Samus and projectile interaction with enemies.
    SamusProjectileInteraction,
    /// The enemies.
    Enemies,
    /// Samus's new state.
    SamusNewState,
    /// Enemy projectiles.
    EnemyProjectiles,
    /// The PLM pass.
    Plms,
    /// Animated tiles.
    AnimatedTiles,
    /// Enemy projectile and Samus collision.
    EnemyProjectileSamus,
    /// Enemy projectile and projectile collision.
    EnemyProjectileProjectile,
    /// Enemy power bomb interaction.
    EnemyPowerBomb,
    /// The main scrolling routine.
    Scrolling,
    /// Draw Samus, projectiles, enemies and enemy projectiles.
    Draw,
    /// The enemy BG2 tilemap transfer queue.
    EnemyBg2Tilemap,
    /// The HUD tilemap.
    Hud,
    /// Layer 2 position and BG scroll.
    BgScroll,
    /// The room's main code.
    RoomMain,
    /// Health depletion and game time.
    HealthAndTime,
    /// Room shaking.
    RoomShaking,
    /// Hurt timers and the active enemy index.
    HurtTimers,
}

/// The gameplay pass up to and including the PLM pass, whose instructions can call out (the
/// item's message box) and so end a step.
const THROUGH_PLMS: [GameplayPhase; 8] = [
    GameplayPhase::EnemySelection,
    GameplayPhase::PaletteFx,
    GameplayPhase::SamusCurrentState,
    GameplayPhase::SamusProjectileInteraction,
    GameplayPhase::Enemies,
    GameplayPhase::SamusNewState,
    GameplayPhase::EnemyProjectiles,
    GameplayPhase::Plms,
];

/// The gameplay pass after the PLM pass.
const AFTER_PLMS: [GameplayPhase; 13] = [
    GameplayPhase::AnimatedTiles,
    GameplayPhase::EnemyProjectileSamus,
    GameplayPhase::EnemyProjectileProjectile,
    GameplayPhase::EnemyPowerBomb,
    GameplayPhase::Scrolling,
    GameplayPhase::Draw,
    GameplayPhase::EnemyBg2Tilemap,
    GameplayPhase::Hud,
    GameplayPhase::BgScroll,
    GameplayPhase::RoomMain,
    GameplayPhase::HealthAndTime,
    GameplayPhase::RoomShaking,
    GameplayPhase::HurtTimers,
];

/// Blocks a screen across.
const SCREEN_BLOCKS: u16 = 16;

/// What one gameplay pass hands from phase to phase: nothing outlives the step.
#[derive(Default)]
struct Pass {
    /// Samus's pose requests, from her current state to her new state.
    requests: pose::PoseRequests,
    /// The enemies processing this frame, from the selection to the main enemy routine.
    active: crate::enemy::Active,
    /// The PLM whose instruction called out of the PLM pass, which ends the step.
    callout: Option<crate::state::slots::SlotIndex<crate::state::room::Plm>>,
}

/// One step of the gameplay pass from `at`.
fn gameplay(
    at: GameplayAt,
    state: &mut State,
    content: &Content,
) -> Result<(GameplayAt, Next), SessionFault> {
    Ok(match at {
        GameplayAt::Start => {
            let mut pass = Pass::default();
            for phase in THROUGH_PLMS {
                gameplay_phase(phase, state, content, &mut pass)?;
            }
            // An item's pickup called out of the PLM pass.
            let next = pass
                .callout
                .map_or(Next::Continue, |slot| Next::Call(Frame::Plms { slot }));
            (GameplayAt::AfterPlms, next)
        }
        GameplayAt::AfterPlms => {
            // The pose requests are consumed before the PLM pass, and the active enemy list
            // only the enemy phases before it read.
            let mut pass = Pass::default();
            for phase in AFTER_PLMS {
                gameplay_phase(phase, state, content, &mut pass)?;
            }
            (GameplayAt::Start, Next::Return)
        }
    })
}

/// A gameplay phase's work.
// open: each empty arm fills in with its C lane; the PLM pass ends the step with
// `Next::Call(Frame::Plms { .. })` once an instruction can call out (B7, C12).
fn gameplay_phase(
    phase: GameplayPhase,
    state: &mut State,
    content: &Content,
    pass: &mut Pass,
) -> Result<(), SessionFault> {
    match phase {
        // The frame starts with no super special request (`$90:E695`).
        GameplayPhase::SamusCurrentState => {
            state.samus.super_special = SuperSpecial::default();
            // While the elevator carries her, the locked current state handler (`$90:E713`,
            // from `$91:E44D`) neither sets her radius nor runs the pose input handler, which
            // would return at once anyway (`$91:805C`).
            pass.requests = if movement::carried(&state.samus, state.elevator) {
                pose::PoseRequests::default()
            } else {
                movement::set_radius(&mut state.samus, content);
                let requests = pose::input(&state.samus, &state.controller, content);
                // open: the suit palette index and the Y acceleration (`$90:E6B6`,
                // `$90:E6B9`) run between the input handler and the inside handling.
                inside::handle(&mut state.room, &state.samus).map_err(SessionFault::Plm)?;
                requests
            };
        }
        // The movement handler, the animation, the pose the moves ask for (`$91:E8B6`), then
        // the pose update (`$90:E725`).
        // open: the movement types beyond standing, running and turning join with C3b and C4.
        GameplayPhase::SamusNewState => {
            let surroundings = Surroundings {
                room: Room {
                    blocks: &state.room.blocks,
                    width: state.room.width,
                    slopes: Some(&content.slopes),
                },
                order: ScanOrder::of(state.clock.counters.completed.0),
            };
            let mut touches = Touches::default();
            let contacts = movement::run(
                &mut state.samus,
                &state.controller,
                content,
                surroundings,
                &mut touches,
            );
            movement::ride(&mut state.samus, state.elevator, surroundings, &mut touches);
            mini_map(
                &mut state.hud,
                state.samus.position,
                &state.room,
                state.clock.counters.requested_byte,
                content,
            );
            animation::animate(&mut state.samus, &state.controller, pass.requests, content);
            let requests = pose::collision_request(
                &state.samus,
                contacts.vertical,
                &state.controller,
                content,
                pass.requests,
            );
            let collisions = pose::Collisions {
                contacts,
                surroundings,
            };
            pose::change(
                &mut state.samus,
                requests,
                collisions,
                &mut touches,
                content,
            );
            // The touch PLMs the moves spawned, which change nothing the moves read.
            touches.apply(&mut state.room).map_err(SessionFault::Plm)?;
            crate::samus_palette::handle(state, &content.visor_colours);
        }
        // open: running out of health (the reserve tanks' refill and the death sequence) joins
        // with those game states. In game state 8 the clock still counts on that frame; it
        // stops only if an earlier phase this frame already left gameplay.
        GameplayPhase::HealthAndTime => state.clock.game_time.tick(),
        GameplayPhase::Hud => crate::hud::phase(state),
        GameplayPhase::EnemySelection => pass.active = crate::enemy::select(state),
        GameplayPhase::Enemies => crate::enemy::run(state, &pass.active, content),
        // `$84:85B4`.
        GameplayPhase::Plms => {
            let population = content.room_plms.of(state.room.room);
            pass.callout =
                crate::plm::process(&mut state.room, &mut state.collected_items, population);
        }
        GameplayPhase::Scrolling => {
            let room = &mut state.room;
            let width = room.width.0 / SCREEN_BLOCKS;
            let height = u16::try_from(room.scroll_areas.len())
                .unwrap_or(u16::MAX)
                .checked_div(width)
                .unwrap_or(0);
            let areas = scrolling::ScrollAreas {
                areas: &room.scroll_areas,
                stale: &room.stale_scrolls,
                width,
                height,
            };
            scrolling::follow(&mut room.camera, &mut state.samus, areas);
        }
        GameplayPhase::Draw => crate::enemy::drawing::draw(state, content),
        GameplayPhase::BgScroll => {
            scrolling::scroll_backgrounds(&mut state.room.camera, &mut state.video.batch.registers);
        }
        GameplayPhase::PaletteFx
        | GameplayPhase::SamusProjectileInteraction
        | GameplayPhase::EnemyProjectiles
        | GameplayPhase::AnimatedTiles
        | GameplayPhase::EnemyProjectileSamus
        | GameplayPhase::EnemyProjectileProjectile
        | GameplayPhase::EnemyPowerBomb
        | GameplayPhase::EnemyBg2Tilemap
        | GameplayPhase::RoomMain
        | GameplayPhase::RoomShaking
        | GameplayPhase::HurtTimers => {}
    }
    Ok(())
}

/// The mini-map's update after the movement handler (`$90:E734`).
fn mini_map(
    hud: &mut crate::state::hud::Hud,
    samus: crate::state::samus::Point,
    room: &crate::state::room::RoomScoped,
    requested: u8,
    content: &Content,
) {
    let width = room.width.0;
    let blocks = u16::try_from(room.blocks.len()).unwrap_or(u16::MAX);
    let room = crate::mini_map::RoomBlocks {
        width,
        height: blocks.checked_div(width).unwrap_or(0),
    };
    crate::mini_map::update(
        &mut hud.mini_map,
        &mut hud.tilemap,
        samus,
        room,
        requested,
        &content.area_maps,
    );
}
