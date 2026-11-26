//! The discovery room's encounter, immediately after the demo's room turn.

use crate::intent::SpriteDraw;
use core::convert::Infallible;
use data::assets::discovery as assets;
use data::spritemap::Spritemap;
use screens_discovery::Encounter;
use screens_discovery::scene::{
    Animation, DiscoveryScene, DrawingRef, DropletContent, FragmentContent, PaletteRef,
    ShellPictures, Shown, SoundOffer, SoundRef, SpriteRef, Track,
};
pub use screens_discovery::{Cut, IntroRequest, ReadyForPageThree};
pub use types::discovery::{DiscoveryOrder, DiscoveryState};
use types::{Pair, QueueAllowance};

/// The encounter and the pictures its clean drawing identities name.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Discovery {
    encounter: Encounter,
    maps: Vec<Spritemap>,
}

impl Discovery {
    /// Bind the imported pictures and particle tracks to the simulation.
    ///
    /// # Errors
    /// Rejects an empty particle track or an unavailable sprite map.
    pub fn new(scene: assets::Scene) -> Result<Self, &'static str> {
        let sprite = |picture| {
            scene
                .drawings
                .get(usize::from(picture))
                .ok_or("discovery picture")?;
            Ok(SpriteRef {
                drawing: DrawingRef(picture),
                palette: PaletteRef(u16::from(scene.palette)),
            })
        };
        let shown = |frame: assets::Frame| -> Result<_, &'static str> {
            Ok(Shown {
                sprite: sprite(frame.picture)?,
                hold: frame.hold,
            })
        };
        let animation = |clip: assets::Clip| -> Result<_, &'static str> {
            Ok(Animation::new(
                shown(clip.first)?,
                clip.rest.into_iter().map(shown).collect::<Result<_, _>>()?,
            ))
        };
        let flight = |flight: assets::Flight| -> Result<_, &'static str> {
            Ok((
                sprite(flight.picture)?,
                flight.drift,
                Track::new(flight.path).ok_or("discovery motion")?,
            ))
        };
        let fragments = scene
            .fragments
            .into_iter()
            .map(|(anchor, f)| {
                let (sprite, drift, track) = flight(f)?;
                Ok(FragmentContent {
                    sprite,
                    anchor,
                    drift,
                    track,
                })
            })
            .collect::<Result<_, &'static str>>()?;
        let droplets = scene
            .drops
            .into_iter()
            .map(|f| {
                let (sprite, drift, track) = flight(f)?;
                Ok(DropletContent {
                    sprite,
                    drift,
                    track,
                })
            })
            .collect::<Result<_, &'static str>>()?;
        // The cue identity includes the library. Shells and cries use distinct queues.
        let offer = |library, code| SoundOffer {
            sound: SoundRef(u16::from_be_bytes([library, code])),
            allowance: QueueAllowance(6),
        };
        let content = DiscoveryScene {
            home: scene.home,
            shell: ShellPictures {
                closed: sprite(scene.closed)?,
                hatch: animation(scene.hatch)?,
                spent: sprite(scene.spent)?,
                scatter_at: scene.release_at,
                release_at: scene.release_at,
            },
            baby: animation(scene.pulse)?,
            fragments,
            droplets,
            splash: animation(scene.splash)?,
            hatch_sound: offer(2, scene.hatch_sound),
            cry_sound: offer(3, scene.cry),
        };
        Ok(Self {
            encounter: Encounter::new(content),
            maps: scene.drawings,
        })
    }

    /// Perform this cut in scene order. Setup can place the pair at an interrupted cut.
    /// A finished, visible room runs before the return can hide it.
    ///
    /// # Errors
    /// A failed room turn leaves the encounter as it was on entry.
    pub fn turn<E>(
        &self,
        state: &mut DiscoveryState,
        cut: Cut,
        request: IntroRequest,
        room: impl FnOnce() -> Result<RoomView, E>,
    ) -> Result<Publication, E> {
        let mut next = state.clone();
        let origin = Pair { x: 0, y: 0 };
        if request == IntroRequest::StartDiscovery {
            self.encounter
                .step(&mut next, origin, request, Cut::Unfinished);
        }
        let room_runs = cut == Cut::Finished && next.order != DiscoveryOrder::Hidden;
        let view = if room_runs {
            room()?
        } else {
            RoomView {
                centre: origin,
                scroll: origin,
            }
        };
        let late_request = if cut == Cut::Finished && request == IntroRequest::FinishDiscovery {
            request
        } else {
            IntroRequest::None
        };
        let result = self
            .encounter
            .step(&mut next, view.centre, late_request, cut);
        let draw = (cut == Cut::Finished).then(|| self.draw(&next, view.scroll));
        let output = Publication {
            room_ran: room_runs,
            sounds: result.sounds,
            cue: result.cue,
            draw,
            order: next.order,
        };
        *state = next;
        Ok(output)
    }

    fn draw(&self, state: &DiscoveryState, scroll: Pair<i32>) -> SpriteDraw {
        let mut out = SpriteDraw::default();
        for figure in self.encounter.drawings(state) {
            if let Some(map) = self.maps.get(usize::from(figure.sprite.drawing.0)) {
                let start = out.pieces.len();
                out.place(map, figure.on_screen(scroll));
                for piece in out.pieces.iter_mut().skip(start) {
                    piece.look.palette = figure.sprite.palette.0.to_le_bytes()[0];
                }
            }
        }
        out
    }

    pub(crate) fn scoped(&self, state: &mut DiscoveryState, frame: Frame) -> Publication {
        let result = self.turn(state, frame.cut, frame.request, || {
            Ok::<_, Infallible>(frame.room)
        });
        let Ok(output) = result;
        output
    }
}

/// Samus's resulting centre and camera after this room turn, whole pixels.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct RoomView {
    /// Samus's whole position in the room.
    pub centre: Pair<i32>,
    /// The camera's room position.
    pub scroll: Pair<i32>,
}

/// The scene and room dependencies of one cut.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Frame {
    /// Whether this game frame finished.
    pub cut: Cut,
    /// The intro's start or late finish request.
    pub request: IntroRequest,
    /// Room result after movement.
    pub room: RoomView,
}

/// A cut's scene publication. The drawing is absent on interrupted cuts.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Publication {
    /// Whether the room took its turn before the encounter.
    pub room_ran: bool,
    /// The offers for this cut in emission order.
    pub sounds: Vec<SoundOffer>,
    /// The hatch has finished and page three can begin.
    pub cue: Option<ReadyForPageThree>,
    /// Sprites for the finished cut, in foreground order.
    pub draw: Option<SpriteDraw>,
    /// Placement of the scene against Samus and her shots.
    pub order: DiscoveryOrder,
}

impl Publication {
    /// Place the encounter and the room's figures in foreground order for E1.
    /// A hidden room contributes neither group, even after its last turn.
    #[must_use]
    pub fn sprites(&self, mut room: SpriteDraw) -> Option<SpriteDraw> {
        let scene = self.draw.as_ref()?;
        Some(match self.order {
            DiscoveryOrder::SceneFirst => {
                let mut draw = scene.clone();
                draw.pieces.append(&mut room.pieces);
                draw
            }
            DiscoveryOrder::SamusFirst => {
                room.pieces.extend_from_slice(&scene.pieces);
                room
            }
            DiscoveryOrder::Hidden => SpriteDraw::default(),
        })
    }
}
