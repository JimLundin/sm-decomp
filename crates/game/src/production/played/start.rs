//! The first room of a fresh Ceres file, assembled by the same room owner used in play.

use super::{Fault, State, arrival, frame, samus};
use canonical::model::Model;
use core::num::Wrapping;
use data::assets::samus::map;
use data::assets::start::NewGame;
use data::pose::PoseId;
use types::{Pair, fixed};
use world_camera::{Camera, RememberedPosition, ViewPosition};
use world_doors::{COLOURS, Doors, Palette};

/// The pose she starts in: facing the screen, in her power suit.
const FACING_THE_SCREEN: PoseId = PoseId(0);

impl State {
    /// The first room of a new file, from loaded content and the carried scene and clock.
    ///
    /// # Errors
    /// The new game's pose or fresh energy cannot enter play.
    pub fn new_game(
        start: &NewGame,
        mut started: crate::production::room_loading::Started,
        health: samus_health::State,
        arrival: arrival::State,
        frame: frame::State,
        quake: world_quake::State,
    ) -> Result<Self, Fault> {
        let samus = fresh_at(start.her)?;
        // The gameplay display uses small tiles and keeps the HUD above the room.
        let tiled = crate::state::video::BgMode(9);
        started.room.entry.view.hud = tiled;
        started.room.entry.view.room = tiled;
        let her = samus.movement.0.position;
        let (view_x, view_y) = start.view;
        Ok(Self {
            room_video: super::room_video::RoomVideo::default(),
            play_time: crate::production::played::play_time::ZERO,
            clock_picture: super::clock_picture::ClockPicture::default(),
            samus,
            camera: Model(Camera::restore(
                ViewPosition(Pair {
                    x: fixed(Wrapping(view_x), 0),
                    y: fixed(Wrapping(view_y), 0),
                }),
                RememberedPosition(her),
                types::VerticalFraction(0),
            )),
            health: Model(health),
            hud: Model(blank_hud()?),
            progress: run_progress::Progress::default(),
            passage: super::passage::State {
                doors: Doors::Play,
                doorway: None,
                awaiting_actors: false,
                room: started.room,
                map_room: started.map.0,
            },
            palette: Palette {
                screen: [types::Colour::BLACK; COLOURS],
                goal: started.goal,
            },
            station: super::station::State {
                doors: started.actors.doors,
                slots: started.actors.slots,
                vents: started.actors.vents,
                pad_and_cover: crate::production::shaft::GONE,
                shaft_overlay: started.actors.shaft_overlay,
                ridley: None,
                warning: None,
                stage: boss_ceres_ridley::Stage::Calm,
            },
            quake: Model(quake),
            frame,
            arrival,
        })
    }
}

/// Her State as a new game starts: standing at rest facing the screen, where the start puts her,
/// with no speed and nothing chosen before.
pub(super) fn fresh_at((x, y): (u16, u16)) -> Result<samus::State, Fault> {
    let pose = map::key_of(FACING_THE_SCREEN).ok_or(Fault::NewGame)?;
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
        getaway: samus_movement::Getaway::Idle,
        getaway_release_speed: None,
    };
    Ok(samus::State {
        last_draw_jump: Model(samus_movement::JumpButton::Up),
        travelled: Model(samus_movement::TravelledDistances::default()),
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
pub(super) fn new_file_health(energy: u16) -> Result<samus_health::State, Fault> {
    Ok(samus_health::State {
        energy: samus_health::EnergyAmount {
            whole: samus_health::WholeEnergy::new(energy).ok_or(Fault::NewGame)?,
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
fn blank_hud() -> Result<interface_hud::HudState, Fault> {
    let colours = interface_hud::ColourSet::new(0).ok_or(Fault::HudStartup)?;
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
