//! Immutable ordinary power content conversion and sprite presentation.
//! Original content-place associations remain in the verifier's importer edge.

use data::assets::projectile as asset;
use data::assets::samus::map;
use data::pose::PoseId;
use enum_map::EnumMap;
use std::collections::BTreeMap;
use weapons_projectiles as beam;

/// A unsupported content or production capability, without original bytes.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Fault {
    /// Invalid imported or restored content relationship.
    Content,
    /// Unavailable named pose muzzle.
    Muzzle,
    /// Unrecognized picture identity.
    Picture,
    /// Unsupported trail appearance or undelivered request.
    Trail,
    /// Unavailable or unsupported typed collision cell.
    World,
    /// A neighbouring producer or caller lies outside ordinary ownership.
    Applicability,
}

/// Immutable production content; construction establishes the model conversion invariant.
#[derive(Clone, Debug)]
pub struct PowerContent {
    assets: asset::Content,
    model: beam::PowerBeam,
}
impl PartialEq for PowerContent {
    fn eq(&self, other: &Self) -> bool {
        self.assets == other.assets
    }
}
impl Eq for PowerContent {}

/// Convert a typed asset heading into the clean model heading.
#[must_use]
pub const fn heading(value: asset::Heading) -> beam::Heading {
    match value {
        asset::Heading::UpFacingRight => beam::Heading::UpFacingRight,
        asset::Heading::UpRight => beam::Heading::UpRight,
        asset::Heading::Right => beam::Heading::Right,
        asset::Heading::DownRight => beam::Heading::DownRight,
        asset::Heading::DownFacingRight => beam::Heading::DownFacingRight,
        asset::Heading::DownFacingLeft => beam::Heading::DownFacingLeft,
        asset::Heading::DownLeft => beam::Heading::DownLeft,
        asset::Heading::Left => beam::Heading::Left,
        asset::Heading::UpLeft => beam::Heading::UpLeft,
        asset::Heading::UpFacingLeft => beam::Heading::UpFacingLeft,
    }
}

/// Convert a clean model heading into the typed asset heading.
#[must_use]
pub const fn asset_heading(value: beam::Heading) -> asset::Heading {
    match value {
        beam::Heading::UpFacingRight => asset::Heading::UpFacingRight,
        beam::Heading::UpRight => asset::Heading::UpRight,
        beam::Heading::Right => asset::Heading::Right,
        beam::Heading::DownRight => asset::Heading::DownRight,
        beam::Heading::DownFacingRight => asset::Heading::DownFacingRight,
        beam::Heading::DownFacingLeft => asset::Heading::DownFacingLeft,
        beam::Heading::DownLeft => asset::Heading::DownLeft,
        beam::Heading::Left => asset::Heading::Left,
        beam::Heading::UpLeft => asset::Heading::UpLeft,
        beam::Heading::UpFacingLeft => asset::Heading::UpFacingLeft,
    }
}

/// The stable content identity of a named asset clip.
#[must_use]
pub fn clip_id(id: asset::ClipId) -> beam::ClipId {
    beam::ClipId(format!("{id:?}"))
}
/// The stable content identity of an imported sprite.
#[must_use]
pub fn picture(id: u16) -> beam::PictureId {
    beam::PictureId(id.to_string())
}
/// The stable content identity of an imported trail appearance.
#[must_use]
pub fn appearance(id: u16) -> beam::TrailAppearanceId {
    beam::TrailAppearanceId(id.to_string())
}

impl PowerContent {
    /// Convert checked AF3 assets into the clean model's content.
    /// # Errors
    /// Invalid frame, picture, trail or repetition references.
    pub fn new(assets: asset::Content) -> Result<Self, Fault> {
        let source = &assets.power;
        let mut clips = BTreeMap::new();
        let kinds = asset::Heading::ALL
            .into_iter()
            .map(asset::ClipId::Flight)
            .chain([asset::ClipId::Impact]);
        for kind in kinds {
            let source_clip = source.clip(kind);
            if source_clip.frames.is_empty() {
                return Err(Fault::Content);
            }
            let frames = source_clip
                .frames
                .iter()
                .map(|frame| {
                    source
                        .sheet
                        .sprites
                        .get(usize::from(frame.picture.0))
                        .ok_or(Fault::Picture)?;
                    let trail = source
                        .trails
                        .get(usize::from(frame.trail_appearance.0))
                        .ok_or(Fault::Trail)?;
                    Ok(beam::Frame {
                        hold: beam::FrameHold::new(frame.hold.updates()).ok_or(Fault::Content)?,
                        picture: picture(frame.picture.0),
                        radius: beam::Axes {
                            x: beam::Radius(frame.radius.x),
                            y: beam::Radius(frame.radius.y),
                        },
                        trail_appearance: match trail {
                            asset::TrailAppearance::PowerBeam(_) => {
                                Some(appearance(frame.trail_appearance.0))
                            }
                            asset::TrailAppearance::Inactive => None,
                        },
                    })
                })
                .collect::<Result<Vec<_>, Fault>>()?;
            let ending = match source_clip.end {
                asset::ClipEnd::Remove => beam::ClipEnding::Remove,
                asset::ClipEnd::RepeatFrom(at) => beam::ClipEnding::RepeatFrom(
                    beam::ClipFrame::new(usize::from(at.0), frames.len()).ok_or(Fault::Content)?,
                ),
            };
            clips.insert(clip_id(kind), beam::Clip { frames, ending });
        }
        let model = beam::PowerBeam {
            damage: beam::Damage::new(source.damage.points()).ok_or(Fault::Content)?,
            flying_clips: EnumMap::from_fn(|h| clip_id(asset::ClipId::Flight(asset_heading(h)))),
            impact_clip: clip_id(asset::ClipId::Impact),
            clips,
        };
        Ok(Self { assets, model })
    }
    /// Validate complete restored model references before a State becomes an owner.
    /// # Errors
    /// Unknown clips/pictures/trails, invalid frame cursors or zero-count flying occupants.
    pub fn validate_state(&self, state: &beam::State) -> Result<(), Fault> {
        if state
            .exhaust
            .iter()
            .any(|puff| !matches!(puff, beam::ExhaustSlot::Free))
        {
            return Err(Fault::Applicability);
        }
        let mut flying = false;
        for shot in state.slots.values() {
            let body = match shot {
                beam::ShotState::Free { .. } => continue,
                beam::ShotState::Flying { body, .. } => {
                    flying = true;
                    body
                }
                beam::ShotState::Impact { body } => body,
            };
            if body.kind != beam::ShotKind::Beam {
                return Err(Fault::Applicability);
            }
            let expected_clip = match shot {
                beam::ShotState::Flying { body, .. } => &self.model.flying_clips[body.heading],
                beam::ShotState::Impact { .. } => &self.model.impact_clip,
                beam::ShotState::Free { .. } => return Err(Fault::Content),
            };
            if &body.clip != expected_clip {
                return Err(Fault::Content);
            }
            let clip = self.model.clips.get(&body.clip).ok_or(Fault::Content)?;
            if body.next_frame.index() > clip.frames.len() {
                return Err(Fault::Content);
            }
            if let Some(picture) = &body.picture {
                self.picture(picture)?;
            }
            if let Some(appearance) = &body.trail_appearance {
                self.trail(appearance)?;
            }
        }
        if flying && state.count.0 == 0 {
            return Err(Fault::Applicability);
        }
        Ok(())
    }

    fn picture(&self, id: &beam::PictureId) -> Result<&data::spritemap::Spritemap, Fault> {
        let index = id.0.parse::<usize>().map_err(|_| Fault::Picture)?;
        self.assets
            .power
            .sheet
            .sprites
            .get(index)
            .ok_or(Fault::Picture)
    }
    fn trail(&self, id: &beam::TrailAppearanceId) -> Result<asset::TrailOffsets, Fault> {
        let index = id.0.parse::<usize>().map_err(|_| Fault::Trail)?;
        match self.assets.power.trails.get(index) {
            Some(asset::TrailAppearance::PowerBeam(offsets)) => Ok(*offsets),
            Some(asset::TrailAppearance::Inactive) | None => Err(Fault::Trail),
        }
    }

    /// Resolve one model picture producer into the shared sprite draw order.
    /// # Errors
    /// An unrecognized restored picture identity.
    pub fn draw(
        &self,
        pictures: &[beam::PictureIntent],
    ) -> Result<crate::intent::SpriteDraw, Fault> {
        let mut draw = crate::intent::SpriteDraw::default();
        for picture in pictures {
            draw.place(
                self.picture(&picture.picture)?,
                types::Pair {
                    x: i32::from(picture.screen.0.x.cast_signed()),
                    y: i32::from(picture.screen.0.y.cast_signed()),
                },
            );
        }
        Ok(draw)
    }

    /// The immutable converted clean-model content.
    #[must_use]
    pub const fn model(&self) -> &beam::PowerBeam {
        &self.model
    }

    /// The typed muzzle of this pose, with all heading offsets already imported.
    /// # Errors
    /// No declared muzzle for the named pose.
    pub fn muzzle(&self, pose: PoseId) -> Result<beam::Muzzle, Fault> {
        let key = map::MAP
            .iter()
            .find(|entry| entry.id == pose)
            .ok_or(Fault::Muzzle)?
            .key;
        self.muzzle_for(key)
    }

    /// Resolve a semantic pose's cannon descriptor.
    ///
    /// # Errors
    /// The pose has no supported imported muzzle.
    pub fn muzzle_for(&self, pose: types::Pose) -> Result<beam::Muzzle, Fault> {
        let muzzle = self.assets.muzzle(pose).ok_or(Fault::Muzzle)?;
        let aim = match muzzle.aim {
            asset::MuzzleAim::Forbidden => beam::MuzzleAim::Forbidden,
            asset::MuzzleAim::PreviousAim => beam::MuzzleAim::PreviousAim,
            asset::MuzzleAim::Aimed(h) => beam::MuzzleAim::Aimed(heading(h)),
        };
        let offsets = |values: &asset::Headings<asset::Offset>| {
            EnumMap::from_fn(|h| {
                let offset = values.get(asset_heading(h));
                beam::Offset {
                    x: i64::from(offset.x),
                    y: i64::from(offset.y),
                }
            })
        };
        Ok(beam::Muzzle {
            aim,
            lift: u64::from(muzzle.lift),
            stationary: offsets(&muzzle.stationary),
            moving: offsets(&muzzle.moving),
        })
    }
}
