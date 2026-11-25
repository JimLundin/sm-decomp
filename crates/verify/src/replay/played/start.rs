//! State from a new game's start: the run admitted from a seed in the preparation, whose RAM
//! holds the intro's last frame, not the start's.
//!
//! Built from the start: the start room as the room's loading loads it, with its Ceres doors and
//! the shaft's overlay and the colours the fade in heads for; her, fresh, standing at rest
//! facing the screen where the start puts her; the camera at the start's view, remembering her
//! there; her health as a new file starts it; the arrival's State; a blank HUD.
//!
//! From the seed, made before the preparation: the new file's progress, her energy and items,
//! the HUD's held inputs, the escape timer and the station's stage (the held facts of
//! `SEED_HELD`); the sound queue's occupancy and her shot's charge; the screen's brightness and
//! the fade's count, and the platform's counts of finished frames and the random number, which
//! run on through the preparation (`SEED_HELD`).

use super::{seed, word};
use crate::replay::{connected, data_failure, hud, movement, progress, result::Outcome};
use crate::source;
use canonical::model::Model;
use core::num::Wrapping;
use data::assets::samus::map;
use data::assets::start::NewGame;
use data::pose::PoseId;
use game::production::played;
use game::production::room_loading::{self, Rooms};
use types::{Pair, fixed};
use world_camera::{Camera, RememberedPosition, ViewPosition};
use world_doors::{COLOURS, Doors, Palette};

/// The pose she starts in: facing the screen, in her power suit.
const FACING_THE_SCREEN: PoseId = PoseId(0);

/// The held facts and the State a new game's start gives, with the new file's from `wram`.
///
/// The HUD is a blank until the start is ready: the load flow's start-up of it is no owner's
/// here, so the run holds it from the record as she arrives.
pub(super) fn admit(
    wram: &[u8],
    start: &NewGame,
    rooms: &Rooms,
) -> Result<(seed::Room, played::State), Outcome> {
    let bad = || data_failure(None, "played new game's start");
    let progress = progress::seed(wram)?;
    let ceres = match word(wram, source::CERES_STATUS)? {
        0 => boss_ceres_ridley::Stage::Calm,
        _ => return Err(bad()),
    };
    let id = import::world::id(start.room).ok_or_else(bad)?;
    let started = room_loading::start(
        id,
        room_loading::Run {
            progress: &progress,
            ceres,
        },
        rooms,
        &start.her_row,
    )
    .map_err(|_| bad())?;
    // A new game starts with no cinematic, boss or special colours running.
    let inputs = connected::held_inputs(
        wram,
        movement::items(word(wram, source::EQUIPPED_ITEMS)?),
        types::Set::default(),
        started.room.geometry.clone(),
        started.room.camera.clone(),
    )?;
    let held = seed::Room {
        sounds_sent: inputs.sounds_waiting.count() == 0,
        inputs,
        timer_running: word(wram, source::ESCAPE_TIMER_RUNNING)? != 0,
        progress,
        ceres,
        hud: super::hud::held(wram)?,
        map: started.map,
    };
    let samus = fresh(start)?;
    let her = samus.movement.0.position;
    let (view_x, view_y) = start.view;
    let state = played::State {
        samus,
        camera: Model(Camera::restore(
            ViewPosition(Pair {
                x: fixed(Wrapping(view_x), 0),
                y: fixed(Wrapping(view_y), 0),
            }),
            RememberedPosition(her),
            types::VerticalFraction(0),
        )),
        health: Model(new_file_health(held.inputs.energy)?),
        hud: Model(blank_hud()?),
        passage: played::passage::State {
            doors: Doors::Play,
            doorway: None,
            room: started.room,
            map_room: started.map.0,
        },
        palette: Palette {
            screen: [types::Colour::BLACK; COLOURS],
            goal: started.goal,
        },
        station: played::station::State {
            doors: started.actors.doors,
            pad_and_cover: game::production::shaft::GONE,
            shaft_overlay: started.actors.shaft_overlay,
            ridley: None,
            stage: ceres,
        },
        frame: super::frame::seed(wram)?,
        arrival: super::arrival::seed(wram)?,
    };
    Ok((held, state))
}

/// Her State as a new game starts: standing at rest facing the screen, where the start puts her,
/// with no speed and nothing chosen before.
fn fresh(start: &NewGame) -> Result<played::samus::State, Outcome> {
    let pose = map::key_of(FACING_THE_SCREEN)
        .ok_or_else(|| data_failure(None, "played new game's pose"))?;
    let (x, y) = start.her;
    let her = Pair {
        x: fixed(Wrapping(x), 0),
        y: fixed(Wrapping(y), 0),
    };
    let samus = samus_movement::Samus {
        position: her,
        pose,
        run: samus_movement::Run {
            speed: types::ZERO,
            way: types::Side::of(pose.facing),
            carry: samus_movement::Carry::FollowsPad,
            dash: types::ZERO,
            dashing: samus_movement::Dashing::Off,
        },
        vertical: samus_movement::Vertical {
            speed: types::ZERO,
            phase: samus_movement::Phase::Ground,
            bounces: samus_movement::Bounces::None,
        },
        choices: samus_movement::Choices::starting_in(pose),
        seat: samus_movement::Seat::Unseated,
        jump_held: samus_movement::JumpHeld::NONE,
        // No push: the new game clears the side, which reads as left.
        knockback: samus_movement::Knockback::spent(types::Side::Left),
    };
    Ok(played::samus::State {
        movement: Model(samus),
        appearance: Model(samus_appearance::AppearanceState::INITIAL),
        figure: Model(samus_appearance::Figure {
            pose,
            sprite: types::Sprite::Plain,
        }),
    })
}

/// Her health as a new file starts it: the file's energy, and no loss pending, no warning and no
/// protection.
fn new_file_health(energy: u16) -> Result<samus_health::State, Outcome> {
    Ok(samus_health::State {
        energy: samus_health::EnergyAmount {
            whole: samus_health::WholeEnergy::new(energy)
                .ok_or_else(|| data_failure(None, "played new file's energy"))?,
            fraction: 0,
        },
        pending_loss: samus_health::EnergyLoss {
            whole: samus_health::WholeEnergy::ZERO,
            fraction: 0,
        },
        warning: samus_health::Warning::Off,
        protection: samus_health::ProtectionFrames::ZERO,
    })
}

/// The HUD before the load flow puts it up: blank cells, nothing shown or explored, and the
/// mini-map off until her arrival shows it.
fn blank_hud() -> Result<interface_hud::HudState, Outcome> {
    let colours =
        interface_hud::ColourSet::new(0).ok_or_else(|| data_failure(None, "played blank HUD"))?;
    Ok(interface_hud::HudState {
        countdown: interface_hud::Countdown::Dormant,
        grid: interface_hud::HudGrid::filled(interface_hud::Tile::new(
            types::Look::default(),
            colours,
        )),
        shown: interface_hud::Shown::default(),
        selection: interface_hud::Selection::None,
        explored: interface_hud::MapCellSet::default(),
        mini_map: interface_hud::MiniMapPower::Off,
    })
}

/// The HUD as the load flow puts it up for the new file, held from the record at the cut she
/// arrives: no owner here puts it up. The mini-map's power is the arrival's, `shown`.
pub(super) fn hud_put_up(
    wram: &[u8],
    shown: interface_hud::MiniMapPower,
) -> Result<interface_hud::HudState, Outcome> {
    Ok(interface_hud::HudState {
        mini_map: shown,
        ..hud::decode_state(wram)?
    })
}
