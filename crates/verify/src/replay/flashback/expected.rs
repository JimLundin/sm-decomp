//! Decode encounter state from the original's sprite columns and consumers from their outputs.

use super::{data_failure, word};
use crate::replay::{compare::Observation, readers::ExpectedCut, result::Outcome};
use core::num::Wrapping;
use data::rom::{Lorom, RomAddress};
use game::{
    exact::oam::HIDDEN_Y,
    intent::SpriteDraw,
    production::played::flashback::{Effects, HurtColours, IntroRequest, ShotId},
    state::video::NmiBatch,
};
use oracle_record::FrameRecord;
use std::collections::BTreeSet;
use types::{
    Pair, Parts, Side, fixed,
    flashback::{
        Animation, Brain, BrainPhase, BreathStep, Burst, BurstPlace, BurstStep, DrawRank,
        EncounterState, FlashBeats, HoldBeats, PairSchedule, PendingStart, PriorHits, Ring,
        RingPair, RingPath, RingStep, RoomBackgroundY, ShakeBeats,
    },
};

const BACKGROUND_Y: usize = 0x1997;
const BOSS_FLASH: usize = 0x1add;
const SCRIPT: usize = 0x1b1d;
const HOLD: usize = 0x1b5d;
const PICTURE: usize = 0x1a5d;
const STAGE: usize = 0x1f51;
const DRAW_SAMUS: usize = 0x1a57;
const PAGE_TWO_PENDING: u16 = 0xb35f;
const FLASH_MIRROR: usize = 0xc1e0;
const FLASH_CGRAM: usize = 240;
const OAM_LOW_BYTES: usize = 512;

const BREATH: [BreathStep; 4] = [
    BreathStep::First,
    BreathStep::Second,
    BreathStep::Third,
    BreathStep::Fourth,
];
const RING_STEPS: [RingStep; 7] = [
    RingStep::FormingFirst,
    RingStep::FormingSecond,
    RingStep::FormingThird,
    RingStep::MovingFirst,
    RingStep::MovingSecond,
    RingStep::MovingThird,
    RingStep::MovingFourth,
];
const BURST_STEPS: [BurstStep; 8] = [
    BurstStep::Waiting,
    BurstStep::First,
    BurstStep::Second,
    BurstStep::Third,
    BurstStep::Fourth,
    BurstStep::Fifth,
    BurstStep::Sixth,
    BurstStep::Blank,
];
const RING_PAIRS: [RingPair; 2] = [RingPair::First, RingPair::Second];
const RING_PATHS: [RingPath; 4] = [
    RingPath::FirstToward,
    RingPath::FirstAcross,
    RingPath::SecondToward,
    RingPath::SecondAcross,
];
const BURST_PLACES: [BurstPlace; 8] = [
    BurstPlace::Near,
    BurstPlace::Low,
    BurstPlace::High,
    BurstPlace::Centre,
    BurstPlace::UpperRight,
    BurstPlace::LowerLeft,
    BurstPlace::UpperLeft,
    BurstPlace::LowerRight,
];

const BRAIN_IMAGES: [u16; 4] = [0x8c00, 0x8c2f, 0x8c5e, 0x8c2f];
const RING_IMAGES: [u16; 7] = [0x8c8d, 0x8ca3, 0x8cb9, 0x8ca3, 0x8c8d, 0x8ca3, 0x8cb9];
const SMALL_IMAGES: [u16; 8] = [0, 0x97f7, 0x97fe, 0x9805, 0x981b, 0x9831, 0x9847, 0];
const LARGE_IMAGES: [u16; 8] = [0, 0x985d, 0x9864, 0x987a, 0x9890, 0x98a6, 0x98bc, 0];

pub(in crate::replay) struct Expectations {
    encounter_tiles: BTreeSet<u16>,
}

impl Expectations {
    pub(super) fn new(rom: Lorom<'_>) -> Result<Self, Outcome> {
        let bad = || data_failure(None, "flashback observed OAM mask source");
        let mut encounter_tiles = BTreeSet::new();
        for (pictures, palette) in [
            (BRAIN_IMAGES.as_slice(), 7),
            (RING_IMAGES.as_slice(), 7),
            (SMALL_IMAGES.as_slice(), 5),
            (LARGE_IMAGES.as_slice(), 5),
        ] {
            for &picture in pictures.iter().filter(|&&picture| picture != 0) {
                let at = RomAddress::new(0x8c, picture);
                let [lo, hi] =
                    <[u8; 2]>::try_from(rom.read(at, 2).map_err(|_| bad())?).map_err(|_| bad())?;
                let size = usize::from(u16::from_le_bytes([lo, hi])).saturating_mul(5);
                let entries = rom
                    .read(at.advance(2).ok_or_else(bad)?, size)
                    .map_err(|_| bad())?;
                encounter_tiles.extend(
                    entries
                        .as_chunks::<5>()
                        .0
                        .iter()
                        .map(|&[_, _, _, lo, hi]| u16::from_le_bytes([lo, hi]) | (palette << 9)),
                );
            }
        }
        Ok(Self { encounter_tiles })
    }

    pub(super) fn read_record(
        &self,
        previous: &FrameRecord,
        current: &FrameRecord,
        cut: u64,
    ) -> Result<ExpectedCut, Outcome> {
        let bad = || data_failure(Some(cut), "flashback encounter record");
        let pending = word(current, SCRIPT) == 0xcb05 && word(current, PICTURE) == 0;
        let observed = decode(current, pending).ok_or_else(bad)?;
        let finished = word(current, 0x05b6) != word(previous, 0x05b6);
        // A live brain advances its script, picture, hold or elapsed count on each turn.
        // After it retires, a completed empty frame still reports an encounter turn.
        let acted = !pending
            && if word(previous, SCRIPT) != 0 || word(current, SCRIPT) != 0 {
                [SCRIPT, PICTURE, HOLD, 0x1b7d]
                    .into_iter()
                    .any(|addr| word(previous, addr) != word(current, addr))
            } else {
                finished
            };
        let semantics = state(&observed);
        let mut fields = semantics.clone();
        let events = if acted {
            Some(observed_effects(previous, current).ok_or_else(bad)?)
        } else {
            None
        };
        fields.extend(effects(events));

        let colours = if let Some(flash) = events.and_then(|events| events.palette) {
            let mirror = std::array::from_fn(|index| {
                word(
                    current,
                    FLASH_MIRROR.saturating_add(index.saturating_mul(2)),
                )
            });
            let cgram = current.cgram.get(FLASH_CGRAM..).ok_or_else(bad)?;
            if mirror != cgram || (flash == HurtColours::White && mirror != [0x7fff; 16]) {
                return Err(bad());
            }
            Some(mirror)
        } else {
            None
        };
        fields.extend(palette(colours));
        let drawing = if finished {
            Some(self.oam(&current.oam).ok_or_else(bad)?)
        } else {
            None
        };
        fields.push(oam(drawing));
        fields.push(background(
            finished.then(|| word(current, 0x00b3).cast_signed()),
        ));
        Ok(ExpectedCut {
            semantic_count: semantics.len(),
            output_count: fields.len().saturating_sub(semantics.len()),
            fields,
            semantics,
            replies: 0,
        })
    }

    fn oam(&self, oam: &[u8]) -> Option<Vec<u8>> {
        let (low, high) = oam.split_at_checked(OAM_LOW_BYTES)?;
        let bytes = oam_pieces(low.as_chunks::<4>().0.iter().copied(), high, |attrs| {
            self.encounter_tiles.contains(&attrs)
        });
        Some(bytes)
    }
}

fn pending(mut state: EncounterState, word: impl Fn(usize) -> u16) -> Option<EncounterState> {
    const PAIR_SLOT: usize = 28;
    if word(SCRIPT) != 0xcb05
        || word(HOLD) != 1
        || word(SCRIPT + PAIR_SLOT) != 0xce0d
        || word(HOLD + PAIR_SLOT) != 1
        || word(PICTURE) != 0
        || word(PICTURE + PAIR_SLOT) != 0
    {
        return None;
    }
    state.start = PendingStart::Pending;
    Some(state)
}

pub(super) fn decode(record: &FrameRecord, starting: bool) -> Option<EncounterState> {
    let w = |address| word(record, address);
    let mut state = EncounterState {
        start: PendingStart::Clear,
        background_y: RoomBackgroundY(w(BACKGROUND_Y).cast_signed()),
        brain: None,
        next_pair: None,
        rings: enum_map::EnumMap::default(),
        bursts: enum_map::EnumMap::default(),
    };
    if starting {
        return pending(state, w);
    }

    for slot in 0..15_u8 {
        let at = |base: usize| w(base.saturating_add(usize::from(slot).saturating_mul(2)));
        let pointer = at(SCRIPT);
        if pointer == 0 {
            continue;
        }
        let hold = HoldBeats::new(at(HOLD))?;
        let progress = at(0x1b7d);
        let routine = at(0x1b3d);
        let rank = DrawRank(14_u8.saturating_sub(slot));
        let art = at(PICTURE);
        let point = Pair {
            x: Wrapping(at(0x1a7d)),
            y: Wrapping(at(0x1a9d)),
        };
        match (slot, pointer) {
            (0, _) => state.brain = Some(brain(at, hold)?),
            (_, 0xce11 | 0xce17) if rank == DrawRank::FOREMOST => {
                state.next_pair = Some(PairSchedule {
                    pair: if pointer == 0xce11 {
                        RingPair::First
                    } else {
                        RingPair::Second
                    },
                    hold,
                });
            }
            (_, 0xcdef..=0xce09) => {
                let path = *RING_PATHS.get(usize::from(progress))?;
                let moving_offset = if pointer >= 0xcdfd { 2 } else { 0 };
                let index = pointer.checked_sub(0xcdef_u16.saturating_add(moving_offset))? / 4;
                let index = usize::from(index);
                if RING_IMAGES.get(index) != Some(&art) || at(0x1abd) != 0xe00 {
                    return None;
                }
                state.rings[path] = Some(Ring {
                    position: Pair {
                        x: fixed(point.x, at(0x1add)),
                        y: fixed(point.y, at(0x1afd)),
                    },
                    clip: Animation {
                        step: *RING_STEPS.get(index)?,
                        hold,
                    },
                    rank,
                });
            }
            _ if routine == 0xba0f => {
                let (start, places, images) = if pointer >= 0xcdcb {
                    (0xcdcb, 0, SMALL_IMAGES)
                } else {
                    (0xcdab, 3, LARGE_IMAGES)
                };
                let index = usize::from(pointer.checked_sub(start)? / 4);
                let place = *BURST_PLACES.get(usize::from(progress).saturating_add(places))?;
                if images.get(index) != Some(&art) || at(0x1abd) != 0xa00 {
                    return None;
                }
                state.bursts[place] = Some(Burst {
                    at: point,
                    clip: Animation {
                        step: *BURST_STEPS.get(index)?,
                        hold,
                    },
                    rank,
                });
            }
            _ => return None,
        }
    }
    Some(state)
}

fn brain(at: impl Fn(usize) -> u16, hold: HoldBeats) -> Option<Brain> {
    let progress = at(0x1b7d);
    if (at(0x1a7d), at(0x1a9d), at(0x1abd)) != (56, 111, 0xe00) {
        return None;
    }
    let phase = match at(0x1b3d) {
        0xb786 => BrainPhase::Endures {
            hits: PriorHits(u8::try_from(progress).ok()?),
        },
        0xb80f => BrainPhase::Shaking {
            elapsed: ShakeBeats(progress),
        },
        0xb82e => BrainPhase::Returning,
        _ => return None,
    };
    let base = if phase == BrainPhase::Returning {
        0xcb23
    } else {
        0xcb09
    };
    let index = usize::from(at(SCRIPT).checked_sub(base)? / 4);
    if BRAIN_IMAGES.get(index) != Some(&at(PICTURE)) {
        return None;
    }
    Some(Brain {
        phase,
        flash: FlashBeats::new(u8::try_from(at(BOSS_FLASH)).ok()?)?,
        breath: Animation {
            step: *BREATH.get(index)?,
            hold,
        },
    })
}

fn observed_effects(previous: &FrameRecord, current: &FrameRecord) -> Option<Effects> {
    let p = |address| word(previous, address);
    let c = |address| word(current, address);
    let impact = |kind: u16| kind & 0x0fff == 0x0800;
    let mut hits = (0..5_u8).filter(|&slot| {
        let address = 0x0c18_usize.saturating_add(usize::from(slot).saturating_mul(2));
        !impact(p(address)) && impact(c(address))
    });
    let missile_impact = hits.next().map(ShotId);
    if hits.next().is_some() {
        return None;
    }

    let ring_knockback = if c(0x18aa) == 11 {
        if c(0x18a8) != 11 || c(0x0a54) != 1 || p(0x09c2) != c(0x09c2) || p(0x09c6) != c(0x09c6) {
            return None;
        }
        Some(game::production::played::flashback::RingKnockback {
            direction: Side::Right,
            invincible_beats: 11,
            knockback_beats: 11,
        })
    } else {
        None
    };
    let palette = if c(SCRIPT) != 0 && (1..8).contains(&c(BOSS_FLASH)) {
        Some(if c(BOSS_FLASH) & 1 == 1 {
            HurtColours::White
        } else {
            HurtColours::Normal
        })
    } else if p(SCRIPT) != 0 && p(BOSS_FLASH) == 1 && c(SCRIPT) != 0 {
        Some(HurtColours::Normal)
    } else {
        None
    };

    let cue = c(STAGE) == PAGE_TWO_PENDING && p(STAGE) != PAGE_TWO_PENDING;
    let hide = p(SCRIPT) != 0 && p(DRAW_SAMUS) != 0 && c(DRAW_SAMUS) == 0;
    let intro = match (cue, hide) {
        (true, false) => Some(IntroRequest::CuePageTwo),
        (false, true) => Some(IntroRequest::HideSamus),
        (false, false) => None,
        (true, true) => return None,
    };
    Some(Effects {
        missile_impact,
        ring_knockback,
        palette,
        intro,
    })
}

fn field(name: &str, bytes: Vec<u8>) -> Observation {
    Observation {
        field: name.to_owned(),
        bytes,
    }
}

fn ordinal<T: PartialEq>(values: &[T], element: &T) -> u16 {
    values
        .iter()
        .position(|value| value == element)
        .and_then(|index| u16::try_from(index).ok())
        .unwrap_or(u16::MAX)
}

fn words(values: impl IntoIterator<Item = u16>) -> Vec<u8> {
    values.into_iter().flat_map(u16::to_le_bytes).collect()
}

pub(super) fn state(state: &EncounterState) -> Vec<Observation> {
    let brain = state
        .brain
        .map(|brain| {
            let (phase, count) = match brain.phase {
                BrainPhase::Endures { hits } => (0, u16::from(hits.0)),
                BrainPhase::Shaking { elapsed } => (1, elapsed.0),
                BrainPhase::Returning => (2, 0),
            };
            words([
                phase,
                count,
                u16::from(brain.flash.get()),
                ordinal(&BREATH, &brain.breath.step),
                brain.breath.hold.get(),
            ])
        })
        .unwrap_or_default();
    let pair = state
        .next_pair
        .map(|pair| words([ordinal(&RING_PAIRS, &pair.pair), pair.hold.get()]))
        .unwrap_or_default();
    let rings = words(state.rings.values().flat_map(|ring| {
        ring.map_or([0; 8], |ring| {
            let (x, fx) = ring.position.x.words();
            let (y, fy) = ring.position.y.words();
            [
                1,
                x,
                fx,
                y,
                fy,
                ordinal(&RING_STEPS, &ring.clip.step),
                ring.clip.hold.get(),
                u16::from(ring.rank.0),
            ]
        })
    }));
    let bursts = words(state.bursts.values().flat_map(|burst| {
        burst.map_or([0; 6], |burst| {
            [
                1,
                burst.at.x.0,
                burst.at.y.0,
                ordinal(&BURST_STEPS, &burst.clip.step),
                burst.clip.hold.get(),
                u16::from(burst.rank.0),
            ]
        })
    }));
    vec![
        Observation::quantity(
            "flashback.pending-start",
            i64::from(state.start == PendingStart::Pending),
        ),
        Observation::quantity("flashback.background-y", i64::from(state.background_y.0)),
        field("flashback.brain", brain),
        field("flashback.pair-schedule", pair),
        field("flashback.rings", rings),
        field("flashback.bursts", bursts),
    ]
}

pub(super) fn effects(events: Option<Effects>) -> Vec<Observation> {
    let shot = events
        .and_then(|event| event.missile_impact)
        .map_or_else(Vec::new, |id| vec![id.0]);
    let push = events
        .and_then(|event| event.ring_knockback)
        .map_or_else(Vec::new, |push| {
            vec![
                u8::from(push.direction == Side::Right),
                push.invincible_beats,
                push.knockback_beats,
            ]
        });
    let flash = match events.and_then(|event| event.palette) {
        None => 0,
        Some(HurtColours::White) => 1,
        Some(HurtColours::Normal) => 2,
    };
    let intro = events.and_then(|event| event.intro);
    vec![
        Observation::quantity("flashback.turn", i64::from(events.is_some())),
        field("flashback.missile-impact", shot),
        field("flashback.ring-knockback", push),
        Observation::quantity("flashback.flash", flash),
        Observation::quantity(
            "flashback.page-two",
            i64::from(intro == Some(IntroRequest::CuePageTwo)),
        ),
        Observation::quantity(
            "flashback.hide",
            i64::from(intro == Some(IntroRequest::HideSamus)),
        ),
    ]
}

pub(super) fn palette(row: Option<[u16; 16]>) -> [Observation; 2] {
    let bytes = row.map_or_else(Vec::new, words);
    // A single write goes into the palette producer's mirror and to CGRAM at the publication.
    [
        field("flashback.palette-mirror", bytes.clone()),
        field("flashback.cgram", bytes),
    ]
}

pub(super) fn oam(drawn: Option<Vec<u8>>) -> Observation {
    // An empty encounter list at a completed publication still differs from no publication.
    let bytes = drawn.map_or_else(Vec::new, |pieces| {
        let mut bytes = vec![1];
        bytes.extend(pieces);
        bytes
    });
    field("flashback.oam", bytes)
}

pub(super) fn background(y: Option<i16>) -> Observation {
    field(
        "flashback.bg1-y",
        y.map_or_else(Vec::new, |y| y.to_le_bytes().to_vec()),
    )
}

pub(super) fn draw(draw: &SpriteDraw) -> Vec<u8> {
    let mut batch = NmiBatch::default();
    batch.draw_sprites(draw);
    batch.finalise_oam();
    let entries = batch
        .oam
        .low
        .into_iter()
        .map(|entry| [entry.x, entry.y, entry.tile, entry.attributes]);
    oam_pieces(entries, &batch.oam.high, |_| true)
}

fn oam_pieces(
    entries: impl Iterator<Item = [u8; 4]>,
    high: &[u8],
    included: impl Fn(u16) -> bool,
) -> Vec<u8> {
    entries
        .enumerate()
        .filter_map(|(index, [x, y, tile, attrs])| {
            if y == HIDDEN_Y || !included(u16::from_le_bytes([tile, attrs])) {
                return None;
            }
            let bits = high.get(index / 4)? >> ((index % 4).saturating_mul(2));
            Some([x, y, tile, attrs, bits & 3])
        })
        .flatten()
        .collect()
}
