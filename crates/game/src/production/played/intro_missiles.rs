//! Ordinary missiles in the Zebes flashback. The scene orders work, encounter contact and
//! drawing, and retains the prepared draws until the frame is published.

use crate::intent::SpriteDraw;
use crate::weapons_projectiles_bridge::{Fault, PowerContent, picture};
use data::assets::projectile as asset;
use std::collections::BTreeMap;
use types::Pair;
use weapons_projectiles as w1;

const FLIGHT: &str = "cinematic-missile-flight";
const IMPACT: &str = "cinematic-missile-impact";
const EXHAUST: &str = "cinematic-missile-exhaust";

/// Immutable model content and the corresponding renderer's assets.
#[derive(Clone, Debug)]
pub struct Content {
    art: asset::IntroMissile,
    power: PowerContent,
    model: w1::WeaponContent,
}

impl PartialEq for Content {
    fn eq(&self, other: &Self) -> bool {
        self.art == other.art && self.power == other.power
    }
}

impl Eq for Content {}

impl Content {
    /// Bind the imported assets to the clip and picture identities the model carries.
    ///
    /// # Errors
    /// An animation, picture or exhaust reference is outside the imported assets.
    pub fn new(art: asset::IntroMissile, power: PowerContent) -> Result<Self, Fault> {
        if !matches!(art.flight.end, asset::ClipEnd::RepeatFrom(_))
            || art.impact.end != asset::ClipEnd::Remove
            || art.exhaust.is_empty()
        {
            return Err(Fault::Content);
        }
        let flight_clip = w1::ClipId(FLIGHT.to_owned());
        let impact_clip = w1::ClipId(IMPACT.to_owned());
        let exhaust_id = w1::TrailAppearanceId(EXHAUST.to_owned());
        let exhaust = art
            .exhaust
            .iter()
            .enumerate()
            .map(|(index, frame)| w1::ExhaustFrame {
                hold: frame.hold,
                picture: w1::PictureId(index.to_string()),
            })
            .collect();
        let missile = w1::MissileContent {
            damage: w1::Damage::new(art.damage.points()).ok_or(Fault::Content)?,
            clips: BTreeMap::from([
                (
                    flight_clip.clone(),
                    clip(&art, &art.flight, Some(&exhaust_id))?,
                ),
                (impact_clip.clone(), clip(&art, &art.impact, None)?),
            ]),
            flight_clip,
            impact_clip,
            exhaust: BTreeMap::from([(
                exhaust_id,
                w1::ExhaustAppearance {
                    offset: w1::Offset {
                        x: i64::from(art.exhaust_offset.x),
                        y: i64::from(art.exhaust_offset.y),
                    },
                    frames: exhaust,
                },
            )]),
        };
        let model = w1::WeaponContent {
            beam: power.model().clone(),
            missile,
        };
        Ok(Self { art, power, model })
    }

    /// The muzzle of the pose selected by the cannon caller.
    ///
    /// # Errors
    /// No imported descriptor for the pose.
    pub fn muzzle_for(&self, pose: types::Pose) -> Result<w1::Muzzle, Fault> {
        self.power.muzzle_for(pose)
    }
}

fn clip(
    art: &asset::IntroMissile,
    asset: &asset::Clip,
    exhaust: Option<&w1::TrailAppearanceId>,
) -> Result<w1::Clip, Fault> {
    let frames = asset
        .frames
        .iter()
        .map(|frame| {
            art.sheet.sprite(frame.picture).ok_or(Fault::Picture)?;
            Ok(w1::Frame {
                hold: w1::FrameHold::new(frame.hold.updates()).ok_or(Fault::Content)?,
                picture: picture(frame.picture.0),
                radius: w1::Axes {
                    x: w1::Radius(frame.radius.x),
                    y: w1::Radius(frame.radius.y),
                },
                trail_appearance: exhaust.cloned(),
            })
        })
        .collect::<Result<Vec<_>, Fault>>()?;
    if frames.is_empty() {
        return Err(Fault::Content);
    }
    let ending = match asset.end {
        asset::ClipEnd::Remove => w1::ClipEnding::Remove,
        asset::ClipEnd::RepeatFrom(first) => w1::ClipEnding::RepeatFrom(
            w1::ClipFrame::new(usize::from(first.0), frames.len()).ok_or(Fault::Content)?,
        ),
    };
    Ok(w1::Clip { frames, ending })
}

/// Inventory and incoming-shot protection carried by the cinematic caller.
#[derive(Clone, Copy, Debug, PartialEq, Eq, canonical::Canonical)]
pub struct Supplies {
    /// Remaining missile ammo.
    pub ammo: u16,
    /// Ordinary missiles selected.
    pub selected: bool,
    /// Whether a release cancels selection.
    pub cancel_after_shot: bool,
    /// Incoming-shot protection, held by this scene between releases.
    pub protection: u16,
}

/// Apply the authorized cannon and shot turn and deliver its inventory reports.
///
/// The caller applies `PendingAimConsumed` to the same candidate Samus State. Supply
/// the demo's fresh input and the preceding movie fresh input through the cannon fields.
///
/// # Errors
/// An inventory report would debit unavailable ammunition.
pub fn update(
    state: &mut w1::State,
    input: &w1::UpdateInput,
    supplies: &mut Supplies,
    eligible: bool,
    content: &Content,
    rules: w1::Rules,
    world: &mut impl w1::World,
) -> Result<w1::IntroFightUpdate, Fault> {
    let mut next = state.clone();
    let mut inventory = *supplies;
    let supply = w1::MissileSupply {
        eligible,
        selected: inventory.selected,
        ammo: inventory.ammo,
        cancel_after_shot: inventory.cancel_after_shot,
    };
    let result = w1::update_intro_fight(&mut next, input, supply, &content.model, rules, world);
    for report in &result.events {
        match report {
            w1::Event::MissileDebit => {
                inventory.ammo = inventory.ammo.checked_sub(1).ok_or(Fault::Applicability)?;
            }
            w1::Event::ClearSelection => {
                inventory.selected = false;
                inventory.cancel_after_shot = false;
            }
            w1::Event::Protection(hold) => inventory.protection = hold.0,
            w1::Event::PendingAimConsumed
            | w1::Event::PuffRequest { .. }
            | w1::Event::Trail { .. }
            | w1::Event::FireSound(_)
            | w1::Event::ImpactSound(_) => {}
        }
    }
    *state = next;
    *supplies = inventory;
    Ok(result)
}

/// Apply the encounter's chosen slot after flight and before either drawing visit.
pub fn hit(state: &mut w1::State, slot: w1::ShotSlot, content: &Content) -> bool {
    w1::encounter_hit(state, slot, &content.model.missile)
}

/// The encounter's logical slot index.
#[must_use]
pub const fn slot_id(slot: w1::ShotSlot) -> u8 {
    match slot {
        w1::ShotSlot::Slot0 => 0,
        w1::ShotSlot::Slot1 => 1,
        w1::ShotSlot::Slot2 => 2,
        w1::ShotSlot::Slot3 => 3,
        w1::ShotSlot::Slot4 => 4,
    }
}

/// Recover the weapons identity in an encounter reply.
#[must_use]
pub const fn slot_from_id(slot: u8) -> Option<w1::ShotSlot> {
    match slot {
        0 => Some(w1::ShotSlot::Slot0),
        1 => Some(w1::ShotSlot::Slot1),
        2 => Some(w1::ShotSlot::Slot2),
        3 => Some(w1::ShotSlot::Slot3),
        4 => Some(w1::ShotSlot::Slot4),
        _ => None,
    }
}

/// Prepare both distinct drawing visits, only after the encounter's visibility gate.
///
/// # Errors
/// A carried picture cannot be resolved in the imported assets.
pub fn draw_twice(
    state: &mut w1::State,
    view: w1::ScreenOrigin,
    content: &Content,
) -> Result<[SpriteDraw; 2], Fault> {
    let mut next = state.clone();
    let first = draw(&mut next, view, content)?;
    let second = draw(&mut next, view, content)?;
    *state = next;
    Ok([first, second])
}

fn draw(
    state: &mut w1::State,
    view: w1::ScreenOrigin,
    content: &Content,
) -> Result<SpriteDraw, Fault> {
    let pictures = w1::intro_pictures(state, view, &content.model.missile);
    let mut draw = SpriteDraw::default();
    let point = |screen: w1::ScreenCentre| Pair {
        x: i32::from(screen.0.x.cast_signed()),
        y: i32::from(screen.0.y.cast_signed()),
    };
    for intent in pictures.missiles {
        let index = intent
            .picture
            .0
            .parse::<usize>()
            .map_err(|_| Fault::Picture)?;
        let map = content.art.sheet.sprites.get(index).ok_or(Fault::Picture)?;
        draw.place(map, point(intent.screen));
    }
    for puff in pictures.exhaust {
        let index = puff
            .picture
            .0
            .parse::<usize>()
            .map_err(|_| Fault::Picture)?;
        let frame = content.art.exhaust.get(index).ok_or(Fault::Picture)?;
        draw.lone(point(puff.screen), frame.look);
    }
    Ok(draw)
}
