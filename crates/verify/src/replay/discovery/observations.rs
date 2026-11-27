//! Independent native sprite-object, queue and visible OAM observations.

use super::{DiscoveryOrder, DiscoveryState, ExpectedCut, FrameRecord, Observation, Pair, word};
use game::{intent::SpriteDraw, state::video::NmiBatch};
use types::discovery::{
    BabyMotion, Coord, DropletStage, Droplets, Egg, FlyingFragment, Fragments, Hatchling, PieceId,
    Pulse, RoomPoint, Slime, SlimeDroplet, Speed,
};

/// Hatch age follows the native timer from its first frame; the release mark follows the actual sound queue.
#[derive(Clone, Default)]
pub(super) struct Native {
    hatch_start: Option<u16>,
    shed: bool,
}

impl Native {
    pub(super) fn compared_state(
        &mut self,
        previous: &FrameRecord,
        current: &FrameRecord,
    ) -> Option<Vec<Observation>> {
        self.semantic(current, &sounds(previous, current)?)
    }

    pub(super) fn at(&mut self, prev: &FrameRecord, r: &FrameRecord) -> Option<ExpectedCut> {
        let finished = crate::replay::played::platform::finished(r);
        let offers = sounds(prev, r)?;
        let semantics = self.semantic(r, &offers)?;
        let mut fields = semantics.clone();
        let cue = word(r, 0x1f51) == 0xb370 && word(prev, 0x1f51) != 0xb370;
        fields.extend(effects(
            offers.into_iter(),
            cue,
            finished && word(prev, 0x1a57) != 0,
        ));
        fields.push(picture(if finished { Some(native_oam(r)?) } else { None }));
        Some(ExpectedCut {
            semantic_count: semantics.len(),
            output_count: fields.len().checked_sub(semantics.len())?,
            fields,
            semantics,
            replies: 0,
        })
    }

    fn semantic(&mut self, r: &FrameRecord, offers: &[(u16, u16)]) -> Option<Vec<Observation>> {
        let completed = word(r, 0x05b6);
        self.shed |= offers.iter().any(|(id, _)| *id == 0x0323);
        let egg = if word(r, 0x1b39) == 0 {
            Egg::Absent
        } else {
            match word(r, 0x1b59) {
                0xa8e8 => Egg::Waiting,
                0x93d9 => Egg::Hatching {
                    age: completed.wrapping_sub(*self.hatch_start.get_or_insert(completed)),
                },
                0xa903 => Egg::Spent,
                _ => return None,
            }
        };
        let baby = if word(r, 0x1b37) == 0 {
            None
        } else {
            let speed = |at| {
                let v = word(r, at).cast_signed();
                if v % 32 == 0 {
                    i8::try_from(v / 32).ok().map(Speed)
                } else {
                    None
                }
            };
            let motion = match word(r, 0x1b57) {
                0xba5e => BabyMotion::Inside,
                0xba73 => BabyMotion::Rising {
                    vertical: speed(0x1a4f)?,
                    slime: if self.shed {
                        Slime::Shed
                    } else {
                        Slime::Clinging
                    },
                },
                0xbb0d => BabyMotion::Hovering {
                    left: u8::try_from(word(r, 0x1a4d)).ok()?,
                },
                0xbb24 => BabyMotion::Following {
                    velocity: Pair {
                        x: speed(0x1a4d)?,
                        y: speed(0x1a4f)?,
                    },
                    cry: u8::try_from(word(r, 0x1b97)).ok()?,
                },
                _ => return None,
            };
            let pulse = match word(r, 0x1b37) {
                0xcc2b => Pulse::Unseen,
                ptr @ (0xcc2f | 0xcc33 | 0xcc37 | 0xcc3b) => Pulse::Age(
                    ((ptr.checked_sub(0xcc2f)? / 4).checked_mul(10)?)
                        .checked_add(10)?
                        .checked_sub(word(r, 0x1b77))?,
                ),
                _ => return None,
            };
            Some(Hatchling {
                place: position(r, 13),
                motion,
                pulse,
            })
        };
        let (fragments, droplets) = particles(r)?;
        let order = match word(r, 0x1a57) {
            0 => DiscoveryOrder::Hidden,
            1 => DiscoveryOrder::SceneFirst,
            0xffff => DiscoveryOrder::SamusFirst,
            _ => return None,
        };
        Some(state(&DiscoveryState {
            egg,
            baby,
            fragments,
            droplets,
            order,
        }))
    }
}

fn particles(r: &FrameRecord) -> Option<(Fragments, Droplets)> {
    let mut fragments = Vec::new();
    let mut drops = Vec::new();
    for slot in 0_usize..13 {
        let at = slot.saturating_mul(2);
        let tag = word(r, 0x1b7d_usize.saturating_add(at));
        let identity = PieceId(tag.to_le_bytes()[0]);
        let steps = tag >> 8;
        let place = position(r, slot);
        match word(r, 0x1a5d_usize.saturating_add(at)) {
            0x8f7e | 0x8f85 | 0x8f8c | 0x8f93 | 0x8f9a | 0x8fa1 if steps <= 38 => {
                fragments.push(FlyingFragment {
                    identity,
                    place,
                    steps,
                });
            }
            0x8fa8 => drops.push(SlimeDroplet {
                identity,
                place,
                stage: DropletStage::Falling { steps },
            }),
            ptr @ (0x8faf | 0x8fb6 | 0x8fbd | 0x8fc4) => {
                let step = (ptr.checked_sub(0x8faf)? / 7).checked_mul(10)?;
                let age = step
                    .checked_add(10)?
                    .checked_sub(word(r, 0x1b5d_usize.saturating_add(at)))?;
                drops.push(SlimeDroplet {
                    identity,
                    place,
                    stage: DropletStage::Splashing { age },
                });
            }
            _ => {}
        }
    }
    fragments.sort_by_key(|p| p.identity.0);
    drops.sort_by_key(|p| p.identity.0);
    Some((Fragments::new(fragments)?, Droplets::new(drops)?))
}

fn position(r: &FrameRecord, slot: usize) -> RoomPoint {
    let axis = |high: usize, low: usize| {
        let at = slot.saturating_mul(2);
        Coord(
            (i32::from(word(r, high.saturating_add(at)).cast_signed()) << 16)
                | i32::from(word(r, low.saturating_add(at))),
        )
    };
    Pair {
        x: axis(0x1a7d, 0x1add),
        y: axis(0x1a9d, 0x1afd),
    }
}

fn sounds(prev: &FrameRecord, r: &FrameRecord) -> Option<Vec<(u16, u16)>> {
    let mut events = Vec::new();
    for (end, queue, library, code) in [(0x647, 0x666_usize, 2, 11), (0x648, 0x676_usize, 3, 35)] {
        let mut head = *prev.wram.get(end)?;
        let stop = *r.wram.get(end)?;
        if head >= 16 || stop >= 16 {
            return None;
        }
        while head != stop {
            if *r.wram.get(queue.checked_add(usize::from(head))?)? == code {
                events.push((u16::from_be_bytes([library, code]), 6));
            }
            head = head.wrapping_add(1) % 16;
        }
    }
    Some(events)
}

fn field(name: &str, bytes: Vec<u8>) -> Observation {
    Observation {
        field: format!("discovery.{name}"),
        bytes,
    }
}

fn point(place: RoomPoint, bytes: &mut Vec<u8>) {
    bytes.extend_from_slice(&place.x.0.to_le_bytes());
    bytes.extend_from_slice(&place.y.0.to_le_bytes());
}

const fn rank(order: DiscoveryOrder) -> u8 {
    match order {
        DiscoveryOrder::Hidden => 0,
        DiscoveryOrder::SceneFirst => 1,
        DiscoveryOrder::SamusFirst => 2,
    }
}

pub(super) fn state(owner: &DiscoveryState) -> Vec<Observation> {
    let egg = match owner.egg {
        Egg::Absent => vec![0],
        Egg::Waiting => vec![1],
        Egg::Hatching { age } => {
            let [lo, hi] = age.to_le_bytes();
            vec![2, lo, hi]
        }
        Egg::Spent => vec![3],
    };
    let mut baby = Vec::new();
    let mut pulse = Vec::new();
    if let Some(h) = owner.baby {
        point(h.place, &mut baby);
        match h.motion {
            BabyMotion::Inside => baby.push(0),
            BabyMotion::Rising { vertical, slime } => baby.extend([
                1,
                vertical.0.cast_unsigned(),
                u8::from(slime == Slime::Shed),
            ]),
            BabyMotion::Hovering { left } => baby.extend([2, left]),
            BabyMotion::Following { velocity, cry } => baby.extend([
                3,
                velocity.x.0.cast_unsigned(),
                velocity.y.0.cast_unsigned(),
                cry,
            ]),
        }
        if let Pulse::Age(age) = h.pulse {
            pulse.extend(age.to_le_bytes());
        }
    }
    let mut fragments = Vec::new();
    for p in owner.fragments.as_slice() {
        fragments.push(p.identity.0);
        point(p.place, &mut fragments);
        fragments.extend(p.steps.to_le_bytes());
    }
    let mut drops = Vec::new();
    for p in owner.droplets.as_slice() {
        drops.push(p.identity.0);
        point(p.place, &mut drops);
        let (phase, age) = match p.stage {
            DropletStage::Falling { steps } => (0, steps),
            DropletStage::Splashing { age } => (1, age),
        };
        drops.push(phase);
        drops.extend(age.to_le_bytes());
    }
    vec![
        field("egg", egg),
        field("baby", baby),
        field("pulse", pulse),
        field("fragments", fragments),
        field("droplets", drops),
        field("order", vec![rank(owner.order)]),
    ]
}

pub(super) fn effects(
    sounds: impl Iterator<Item = (u16, u16)>,
    cue: bool,
    room_ran: bool,
) -> [Observation; 3] {
    let mut offers = Vec::new();
    for (sound, allowance) in sounds {
        offers.extend(sound.to_le_bytes());
        offers.extend(allowance.to_le_bytes());
    }
    [
        field("sounds", offers),
        field("page-three", vec![u8::from(cue)]),
        field("room-turn", vec![u8::from(room_ran)]),
    ]
}

pub(super) fn picture(oam: Option<(DiscoveryOrder, Vec<u8>)>) -> Observation {
    let mut bytes = Vec::new();
    if let Some((order, pieces)) = oam {
        bytes.push(rank(order));
        bytes.extend(pieces);
    }
    field("oam", bytes)
}

fn native_oam(r: &FrameRecord) -> Option<(DiscoveryOrder, Vec<u8>)> {
    let low = r.wram.get(0x370..0x570)?.as_chunks::<4>().0;
    visible(low.iter().copied(), r.wram.get(0x570..0x590)?)
}

fn visible(
    entries: impl Iterator<Item = [u8; 4]>,
    high: &[u8],
) -> Option<(DiscoveryOrder, Vec<u8>)> {
    let mut pieces = Vec::new();
    let mut last_scene = None;
    let mut first_scene = None;
    let mut first_room = None;
    let mut last_room = None;
    for (i, sprite) in entries.enumerate() {
        let [_, y, _, attr] = sprite;
        if matches!(y, 224 | 240) {
            continue;
        }
        match (attr >> 1) & 7 {
            7 => {
                pieces.extend(sprite);
                let upper = high.get(i / 4)? >> ((i % 4).checked_mul(2)?);
                pieces.push(upper & 3);
                first_scene.get_or_insert(i);
                last_scene = Some(i);
            }
            6 => {}
            _ => {
                first_room.get_or_insert(i);
                last_room = Some(i);
            }
        }
    }
    let order = match (first_scene, last_scene, first_room, last_room) {
        (None, None, None, None) => DiscoveryOrder::Hidden,
        (Some(_), Some(scene), Some(room), Some(_)) if scene < room => DiscoveryOrder::SceneFirst,
        (Some(scene), Some(_), Some(_), Some(room)) if room < scene => DiscoveryOrder::SamusFirst,
        _ => return None,
    };
    Some((order, pieces))
}

/// An invented room piece qualifies relative precedence. Its art is outside this scope.
pub(super) fn draw(
    output: &super::Publication,
) -> Result<Option<(DiscoveryOrder, Vec<u8>)>, super::Outcome> {
    let mut room = SpriteDraw::default();
    room.lone(
        Pair { x: 0, y: 0 },
        data::spritemap::SpriteAttributes::default(),
    );
    let Some(draw) = output.sprites(room) else {
        return Ok(None);
    };
    let mut batch = NmiBatch::default();
    batch.draw_sprites(&draw);
    batch.finalise_oam();
    let entries = batch
        .oam
        .low
        .iter()
        .map(|e| [e.x, e.y, e.tile, e.attributes]);
    visible(entries, &batch.oam.high)
        .map(Some)
        .ok_or_else(|| super::data_failure(None, "discovery sprite groups"))
}
