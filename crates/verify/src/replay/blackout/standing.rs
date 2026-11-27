//! Native standing-body comparisons at the locked room's preparation and NMI cuts.

use super::{ExpectedCut, Outcome, data_failure, word};
use crate::replay::compare::Observation;
use data::pose::PoseId;
use data::pose::animation::AnimationFrame;
use data::pose::tiles::{GraphicsRef, TileDefinition};
use data::rom::{Lorom, RomAddress};
use game::production::Production;
use game::production::blackout::{Output, StandingFrame};
use game::samus_appearance_bridge::{Clips, figure_of};
use game::state::video::{RomWords, StandingStrips};
use oracle_record::{Cut, FrameRecord, NmiKind};
use samus_appearance::{ShaftView, Step};
use types::{Pair, Pose};

pub(in crate::replay) const RULE: &str = "samus-appearance/ElevatorStandingPicture";
const SEMANTIC_FIELDS: usize = 2;
const OUTPUT_FIELDS: usize = 6;

const OTHER_SUITS: u16 = 0x21;
const OAM_START: usize = 17;
const OAM_COUNT: usize = 7;
const OAM_END: usize = OAM_START + OAM_COUNT;

pub(super) struct LockedLook {
    pose: Pose,
    step: Step,
    centre: Pair<i32>,
    pivot: Pair<i32>,
    camera_origin: Pair<i32>,
}

impl LockedLook {
    pub(super) fn read(record: &FrameRecord, rom: Lorom<'_>) -> Result<Self, Outcome> {
        let r = &record.wram;
        let bad = || data_failure(Some(record.index), "shaft standing clip");
        let id = PoseId(word(r, 0xa1c)?);
        let pose = figure_of(id).ok_or_else(bad)?.pose;
        let poses = data::pose::Poses::load(rom).map_err(|_| bad())?;
        let delays =
            data::pose::animation::AnimationDelays::load(rom, &poses).map_err(|_| bad())?;
        let paces = data::pose::animation::RunPaces::load(rom).map_err(|_| bad())?;
        let clips = Clips::import(&delays, &paces).map_err(|_| bad())?;
        let frame = AnimationFrame(word(r, 0xa96)?);
        let step = clips
            .poses()
            .iter()
            .find(|clip| clip.id == id)
            .and_then(|clip| clip.step_of(frame))
            .ok_or_else(bad)?;
        if word(r, 0x18a8)? != 0
            || word(r, 0x18aa)? != 0
            || word(r, 0x0a68)? != 0
            || word(r, 0x09a2)? & OTHER_SUITS != 0
        {
            return Err(bad());
        }
        let point = |x, y| {
            Ok(Pair {
                x: i32::from(word(r, x)?),
                y: i32::from(word(r, y)?),
            })
        };
        Ok(Self {
            pose,
            step,
            centre: point(0xaf6, 0xafa)?,
            pivot: point(0x80, 0x82)?,
            camera_origin: point(0x911, 0x915)?,
        })
    }

    pub(super) fn drawing(&self, early: &FrameRecord) -> Result<StandingFrame, Outcome> {
        let ram = &early.wram;
        Ok(StandingFrame {
            pose: self.pose,
            step: self.step,
            projection: samus_appearance::ElevatorProjection {
                centre: self.centre,
                pivot: self.pivot,
                camera_origin: self.camera_origin,
                carried_view: ShaftView {
                    diagonal: word(ram, 0x78)?.cast_signed(),
                    cross: word(ram, 0x7a)?.cast_signed(),
                },
            },
        })
    }
}

pub(in crate::replay) fn admit_art(
    record: &FrameRecord,
    state: &mut game::production::blackout::State,
    rom: Lorom<'_>,
    pending_publication: bool,
) -> Result<game::samus_appearance_bridge::elevator::Art, Outcome> {
    if pending_publication {
        state.video.batch.standing_strips = pending(record, rom)?;
    }
    game::samus_appearance_bridge::elevator::Art::load(rom)
        .map_err(|_| data_failure(Some(record.index), "standing body art"))
}

fn pending(record: &FrameRecord, rom: Lorom<'_>) -> Result<StandingStrips, Outcome> {
    let ram = &record.wram;
    let half = |flag, pointer| {
        let bad = || data_failure(Some(record.index), "standing body tile descriptor");
        if super::byte(ram, flag)? == 0 {
            return Ok((None, None));
        }
        let descriptor = RomAddress::new(0x92, word(ram, pointer)?);
        let bytes = rom
            .read(descriptor, 7)
            .ok()
            .and_then(|raw| raw.try_into().ok())
            .ok_or_else(bad)?;
        let tile = TileDefinition::decode(bytes);
        let strip = |from: GraphicsRef, size| RomWords {
            from,
            words: size / 2,
        };
        Ok((
            Some(strip(tile.source, tile.first)),
            (tile.second != 0).then(|| strip(tile.second_source(), tile.second)),
        ))
    };
    let (upper_first, upper_second) = half(0x71d, 0x71f)?;
    let (lower_first, lower_second) = half(0x71e, 0x721)?;
    Ok(StandingStrips {
        upper_first,
        upper_second,
        lower_first,
        lower_second,
    })
}

fn strip_bytes(strips: StandingStrips) -> Vec<u8> {
    [
        strips.upper_first,
        strips.upper_second,
        strips.lower_first,
        strips.lower_second,
    ]
    .into_iter()
    .flat_map(|strip| {
        strip.map_or([0; 6], |strip| {
            let [lo, hi] = strip.from.offset.to_le_bytes();
            let [words_lo, words_hi] = strip.words.to_le_bytes();
            [1, strip.from.bank, lo, hi, words_lo, words_hi]
        })
    })
    .collect()
}

pub(in crate::replay) fn packed_oam(raw: &[u8], slots: core::ops::Range<usize>) -> Option<Vec<u8>> {
    let (low, high) = raw.split_at_checked(512)?;
    let entries = low
        .get(slots.start.wrapping_mul(4)..slots.end.wrapping_mul(4))?
        .as_chunks::<4>()
        .0;
    entries
        .iter()
        .zip(slots)
        .flat_map(|(entry, slot)| {
            let bits = high
                .get(slot / 4)
                .map(|byte| (byte >> ((slot % 4).wrapping_mul(2))) & 3);
            entry.iter().copied().map(Some).chain([bits])
        })
        .collect()
}

fn observe(
    prepared: &[u8],
    published: &[u8],
    pending: StandingStrips,
    serviced: StandingStrips,
    vram: &[u16],
) -> Result<Vec<Observation>, Outcome> {
    let bad = || data_failure(None, "standing body video");
    let field = |name, bytes| Observation {
        field: format!("standing-body.{name}"),
        bytes,
    };
    let mut observations = vec![
        field(
            "pending-oam",
            packed_oam(prepared, OAM_START..OAM_END).ok_or_else(bad)?,
        ),
        field("pending-strips", strip_bytes(pending)),
        field(
            "published-oam",
            packed_oam(published, OAM_START..OAM_END).ok_or_else(bad)?,
        ),
        field("publication", strip_bytes(serviced)),
    ];
    for (name, start) in [
        ("upper-first", 0x6000_usize),
        ("upper-second", 0x6100),
        ("lower-first", 0x6080),
        ("lower-second", 0x6180),
    ] {
        let row = vram.get(start..start.wrapping_add(0x80)).ok_or_else(bad)?;
        observations.push(field(
            name,
            row.iter().flat_map(|word| word.to_le_bytes()).collect(),
        ));
    }
    Ok(observations)
}

pub(in crate::replay) fn actual(
    prod: &Production,
    out: &Output,
) -> Result<Vec<Observation>, Outcome> {
    let state = prod
        .blackout()
        .ok_or_else(|| data_failure(None, "standing body continuation"))?;
    observe(
        &super::observe::oam_bytes(&state.video.batch.oam),
        &super::observe::oam_bytes(&state.video.memory.oam),
        state.video.batch.standing_strips,
        out.standing_publication,
        &state.video.memory.vram,
    )
}

pub(super) fn expected(record: &FrameRecord, rom: Lorom<'_>) -> Result<ExpectedCut, Outcome> {
    let pending = pending(record, rom)?;
    let serviced = if matches!(record.cut, Cut::NmiReturn(nmi) if nmi.kind == NmiKind::Requested) {
        pending
    } else {
        StandingStrips::default()
    };
    let fields = observe(
        crate::source::OAM
            .read(&record.wram)
            .map_err(|_| data_failure(Some(record.index), "standing body OAM buffer"))?,
        &record.oam,
        pending,
        serviced,
        &record.vram,
    )?;
    Ok(ExpectedCut {
        semantics: fields.iter().take(SEMANTIC_FIELDS).cloned().collect(),
        fields,
        semantic_count: SEMANTIC_FIELDS,
        output_count: OUTPUT_FIELDS,
        replies: 0,
    })
}

#[cfg(test)]
pub(in crate::replay) fn compare_live(
    video: &game::state::video::Video,
    record: &FrameRecord,
    rom: Lorom<'_>,
    cut: u64,
) -> Result<(), Outcome> {
    let bad = || data_failure(Some(cut), "live locked OAM");
    let shadow = crate::source::OAM.read(&record.wram).map_err(|_| bad())?;
    let batch = super::observe::oam_bytes(&video.batch.oam);
    let live = super::observe::oam_bytes(&video.memory.oam);
    let different = |field: &str| Outcome::Difference {
        cut,
        field: field.into(),
    };
    for (stage, expected, actual) in [
        ("pending", shadow, batch.as_slice()),
        ("published", &record.oam[..], live.as_slice()),
    ] {
        for (draw, slots) in [("timer", 0..OAM_START), ("standing", OAM_START..OAM_END)] {
            if packed_oam(expected, slots.clone()).ok_or_else(bad)?
                != packed_oam(actual, slots).ok_or_else(bad)?
            {
                return Err(different(&format!("live-shaft.{draw}-{stage}-oam")));
            }
        }
    }
    if super::word(&record.wram, 0x0590)? != video.batch.sprites.0.saturating_mul(4) {
        return Err(different("live-shaft.oam-cursor"));
    }
    let strips = pending(record, rom)?;
    if strips != video.batch.standing_strips {
        return Err(different("live-shaft.standing-strips"));
    }
    for (name, start, strip) in [
        ("upper-first", 0x6000_usize, strips.upper_first),
        ("upper-second", 0x6100, strips.upper_second),
        ("lower-first", 0x6080, strips.lower_first),
        ("lower-second", 0x6180, strips.lower_second),
    ] {
        if let Some(strip) = strip {
            let end = start.wrapping_add(usize::from(strip.words));
            let range = start..end;
            if video.memory.vram.get(range.clone()) != record.vram.get(range) {
                return Err(different(&format!("live-shaft.standing-{name}")));
            }
        }
    }
    let hud_rows = 0x5820..0x5880;
    if video.memory.vram.get(hud_rows.clone()) != record.vram.get(hud_rows) {
        return Err(different("live-shaft.hud-tilemap"));
    }
    Ok(())
}
