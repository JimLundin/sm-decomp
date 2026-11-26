//! The station cinematic after the elevator's blackout, including its settled drawing outputs.

use canonical::{
    Canonical,
    model::{Model, Pinned},
};
use data::assets::explosion::Station;
use screens_explosion::{BurstFrame, MapRegion, SceneEvent, SpritePicture};
use types::Pair;
use types::explosion::{
    AsteroidField, BackdropFrame, ExplosionStage, ExplosionState, SampleLook, ScreenPoint, SpriteDrawing,
};

use crate::intent::SpriteDraw;
use crate::state::video::{Colour, NmiBatch, Oam};

/// The complete clean scene's checkpoint representation.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ScenePin;
impl Pinned for ScenePin {
    type State = ExplosionState;
    const NAME: &'static str = "production-station-explosion";
    const SHAPE: u64 = 0x915F_7A13_FB92_F53B;
}

/// Independently scheduled completion and the post-service music reply.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Cut {
    /// The scene work is still underway.
    Unfinished,
    /// Work finished, with the audio service's current busy reply.
    Finished {
        /// A song request still waits in the queue.
        music_busy: bool,
    },
}

struct Timing {
    cut: Cut,
    ready: bool,
}
impl screens_explosion::Scheduler for Timing {
    fn frame_finished(&self) -> bool {
        matches!(self.cut, Cut::Finished { .. })
    }
    fn setup_complete(&self) -> bool {
        self.ready
    }
}
impl screens_explosion::MusicService for Timing {
    fn music_busy(&self) -> bool {
        matches!(self.cut, Cut::Finished { music_busy: true })
    }
}

/// The owner the cinematic scheduler dispatches.
#[derive(Canonical, Clone, Copy, Debug, PartialEq, Eq)]
pub enum Flow {
    /// Waiting through the elevator's blank vertical intervals.
    Blank(u8),
    /// The setup transfers can now publish at their next completion.
    Installing,
    /// Running the station picture.
    Station,
    /// The blackout published the flight to Zebes.
    ZebesFlight,
}

/// Destination carried from the elevator through both cinematics.
#[derive(Canonical, Clone, Copy, Debug, PartialEq, Eq)]
pub enum Destination {
    /// The marked station at the Landing Site.
    LandingSite,
    /// The station marking the beginning of Ceres.
    CeresStart,
}

/// The complete scene and the receiving owners carried from the elevator.
#[derive(Canonical, Clone, Debug, PartialEq, Eq)]
pub struct State {
    /// The clean model's whole state.
    pub scene: Model<ScenePin>,
    /// The next scheduler work.
    pub flow: Flow,
    /// Destination selected by the incoming scheduler.
    pub destination: Destination,
    /// Number of completed frames.
    pub finished: u16,
    /// Health crossing the station and flight.
    pub health: Model<crate::production::health_tail::HealthPin>,
    /// Energy capacity.
    pub maximum: u16,
    /// Options selecting the flight's caption.
    pub japanese_subtitles: bool,
    /// The display's colours, including the navigation lights.
    pub palette: [Colour; 256],
    /// The published sprite buffer.
    pub oam: Oam,
}

/// Exact drawing regions published on a settled transfer.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Pages {
    /// Complete mode 7 map and graphics, followed by the installed sprite tiles.
    Station {
        /// The low byte of each mode 7 word.
        map: Vec<u8>,
        /// The high byte of each mode 7 word.
        graphics: Vec<u8>,
        /// Sprite VRAM after the common graphics overlay.
        sprites: Vec<u8>,
    },
    /// One of the two regions replaced at breakup.
    Ship(MapRegion, Vec<u8>),
}

/// Publications during this cut.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Output {
    /// Ordered requests from the scene to its consumers.
    pub events: Vec<SceneEvent>,
    /// The clean model's on-screen sprite placements.
    pub picture: SpritePicture,
    /// The published affine matrix, scroll and pivot.
    pub plane: Option<(screens_flight::PlaneTransform, Pair<i16>, Pair<i16>)>,
    /// Ordered settled writes to drawing memory.
    pub pages: Vec<Pages>,
}

/// An unavailable consumer or invalid admitted state.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Fault {
    /// The following flight must take over the next dispatched cut.
    ZebesFlight,
    /// The consumed drawing or palette is unavailable.
    Artwork,
    /// A restored scene is inconsistent with its scheduler or content.
    State,
}

impl State {
    /// Accept the elevator's completed save and blackout for a Landing Site destination.
    #[must_use]
    pub fn start(
        finished: u16,
        health: samus_health::State,
        maximum: u16,
        destination: Destination,
        japanese_subtitles: bool,
        palette: [Colour; 256],
        oam: Oam,
    ) -> Self {
        Self {
            scene: Model(ExplosionState::pending()),
            flow: Flow::Blank(9),
            finished,
            health: Model(health),
            maximum,
            destination,
            japanese_subtitles,
            palette,
            oam,
        }
    }

    pub(crate) fn validate(&self, content: &Station) -> Result<(), Fault> {
        let phase = self.scene.0.stage;
        if self.destination != Destination::LandingSite
            || self.health.0.energy.whole.get() > self.maximum
            || self.maximum > i16::MAX.cast_unsigned()
            || (phase == ExplosionStage::Complete) != (self.flow == Flow::ZebesFlight)
            || (phase == ExplosionStage::Pending)
                != matches!(self.flow, Flow::Blank(1..=9) | Flow::Installing)
            || matches!(self.flow, Flow::Blank(0 | 10..))
            || content.lights.is_empty()
            || content
                .light_first
                .checked_add(2)
                .is_none_or(|end| end > self.palette.len())
        {
            return Err(Fault::State);
        }
        Ok(())
    }

    pub(crate) fn advance(
        &mut self,
        content: &Station,
        rules: screens_explosion::Rules,
        cut: Cut,
    ) -> Result<Output, Fault> {
        if self.flow == Flow::ZebesFlight {
            return Err(Fault::ZebesFlight);
        }
        let timing = Timing {
            cut,
            ready: self.flow == Flow::Installing,
        };
        let events =
            screens_explosion::step(&mut self.scene.0, &timing, &timing, &content.scene, rules);
        let picture = screens_explosion::sprites(&self.scene.0, &content.scene);
        let mut pages = Vec::new();
        if matches!(cut, Cut::Finished { .. }) {
            // Re-enabling NMI inside setup services the waiting interval before the
            // first settled return. Both completed frames are visible at publication.
            self.finished = self.finished.wrapping_add(if timing.ready { 2 } else { 1 });
            if let Flow::Blank(left) = self.flow {
                self.flow = if left == 1 {
                    Flow::Installing
                } else {
                    Flow::Blank(left.saturating_sub(1))
                };
            }
            for event in &events {
                self.receive(content, event, &mut pages)?;
            }
            if self.scene.0.stage != ExplosionStage::Pending {
                if timing.ready {
                    // Setup publishes two draws. The longer first backdrop leaves its
                    // unused slots in OAM when the second backdrop hides them by Y.
                    let mut placement = picture.clone();
                    placement.backdrop = BackdropFrame::Second;
                    placement.large_asteroids.x = placement.large_asteroids.x.minus(content.scene.sky.asteroids[AsteroidField::Large].speed);
                    placement.small_asteroids.x = placement.small_asteroids.x.minus(content.scene.sky.asteroids[AsteroidField::Small].speed);
                    self.draw(content, &placement)?;
                }
                self.colour_lights(content)?;
                self.draw(content, &picture)?;
            }
        }
        let plane = if self.scene.0.stage == ExplosionStage::Pending {
            None
        } else {
            use types::flight::{Angle, Plane, Zoom, whole_pixels};
            let pose = self.scene.0.pose;
            let plane = Plane {
                angle: Angle(pose.angle.0),
                zoom: Zoom::new(pose.zoom.0).ok_or(Fault::State)?,
                position: Pair {
                    x: std::num::Wrapping(pose.scroll.x.0.cast_unsigned()),
                    y: std::num::Wrapping(pose.scroll.y.0.cast_unsigned()),
                },
            };
            Some((
                screens_flight::transform(&plane),
                whole_pixels(plane.position),
                content.scene.pivot,
            ))
        };
        Ok(Output {
            events,
            picture,
            plane,
            pages,
        })
    }

    fn receive(
        &mut self,
        content: &Station,
        event: &SceneEvent,
        pages: &mut Vec<Pages>,
    ) -> Result<(), Fault> {
        match event {
            SceneEvent::InstallArt(id) if id == &content.scene.art => {
                self.flow = Flow::Station;
                pages.push(Pages::Station {
                    map: content.station_map.clone(),
                    graphics: content.plane_tiles.clone(),
                    sprites: content.sprite_tiles.clone(),
                });
            }
            SceneEvent::ReplaceSpacePalette(id) if id == &content.scene.space_palette => {
                self.palette = content.palette.map(Colour);
            }
            SceneEvent::InstallMapDrawing {
                drawing,
                region: MapRegion::Upper,
            } if drawing == &content.scene.ship_front => {
                pages.push(Pages::Ship(MapRegion::Upper, content.ship_front.clone()));
            }
            SceneEvent::InstallMapDrawing {
                drawing,
                region: MapRegion::Lower,
            } if drawing == &content.scene.empty_drawing => {
                pages.push(Pages::Ship(MapRegion::Lower, content.empty_map.clone()));
            }
            SceneEvent::StartStationFlash { style, .. } if style == &content.scene.flash_style => {}
            SceneEvent::RequestMusic { .. } => {}
            SceneEvent::BeginZebesFlight => self.flow = Flow::ZebesFlight,
            SceneEvent::InstallArt(_)
            | SceneEvent::ReplaceSpacePalette(_)
            | SceneEvent::InstallMapDrawing { .. }
            | SceneEvent::StartStationFlash { .. } => return Err(Fault::Artwork),
        }
        Ok(())
    }

    fn colour_lights(&mut self, content: &Station) -> Result<(), Fault> {
        let duration: u32 = content
            .lights
            .iter()
            .map(|sample| u32::from(sample.frames))
            .sum();
        let mut age = self
            .scene
            .0
            .age
            .0
            .saturating_sub(1)
            .checked_rem(duration)
            .ok_or(Fault::Artwork)?;
        let sample = content
            .lights
            .iter()
            .find(|sample| {
                if age < u32::from(sample.frames) {
                    true
                } else {
                    age = age.saturating_sub(u32::from(sample.frames));
                    false
                }
            })
            .ok_or(Fault::Artwork)?;
        let lights = self
            .palette
            .get_mut(content.light_first..content.light_first.saturating_add(2))
            .ok_or(Fault::Artwork)?;
        lights.copy_from_slice(&sample.colours.map(Colour));
        Ok(())
    }

    fn draw(&mut self, content: &Station, picture: &SpritePicture) -> Result<(), Fault> {
        let mut batch = NmiBatch {
            oam: self.oam,
            ..NmiBatch::default()
        };
        batch.clear_high_oam();
        let sky = &content.scene.sky;
        let mut draw = |id: &SpriteDrawing, position: ScreenPoint, palette| {
            let (_, map) = content
                .drawings
                .iter()
                .find(|(reference, _)| reference == id)
                .ok_or(Fault::Artwork)?;
            let mut sprite = SpriteDraw::default();
            sprite.place(
                map,
                Pair {
                    x: position.x.0 >> 16,
                    y: position.y.0 >> 16,
                },
            );
            for piece in &mut sprite.pieces {
                piece.look.palette = palette;
                if !(-32..224).contains(&piece.at.y) {
                    piece.at = Pair { x: 384, y: 224 };
                }
            }
            batch.draw_sprites(&sprite);
            Ok(())
        };
        draw(
            &sky.asteroids[AsteroidField::Large].drawing,
            picture.large_asteroids,
            content.sky_palette,
        )?;
        for burst in &picture.bursts {
            if let BurstFrame::Showing(index) = burst.frame {
                let Some(SampleLook::Drawing(id)) = content.scene.tracks[burst.id.group]
                    .samples
                    .get(index)
                    .map(|s| &s.look)
                else {
                    return Err(Fault::Artwork);
                };
                draw(id, burst.position, content.burst_palette)?;
            }
        }
        draw(
            &sky.asteroids[AsteroidField::Small].drawing,
            picture.small_asteroids,
            content.sky_palette,
        )?;
        draw(
            &sky.backdrop.drawings[picture.backdrop],
            ScreenPoint::at_pixels(sky.backdrop.place),
            content.sky_palette,
        )?;
        batch.finalise_oam();
        self.oam = batch.oam;
        Ok(())
    }
}
