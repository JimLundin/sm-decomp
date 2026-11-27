//! Native receipts for the ordinary fight's object-strip transfers.

use super::{Controller, Frame, Run, hud, movement, parity, rows::Rows};
use crate::replay::{compare::Observation, data_failure, result::Outcome};
use crate::replay::{
    index::{
        Case, CaseKind, ContentProvider, ExpectedProvider, InputProvider, ObservationId,
        PlayedWindow, Responsibility, Start,
    },
    readers::ExpectedCut,
    result::CaseResult,
    runner,
};
use crate::source::{VRAM_WRITE_QUEUE, WramWord};
use boss_ceres_ridley::{FightRibs, RibBeat as Beat, RibWait};
use data::assets::ridley_ribs::STRIP_WORDS;
use data::dma::BusAddress;
use game::production::played;
use game::state::video::{Video, VramSource, VramStep, VramTransfer};
use oracle_record::FrameRecord;
use std::ops::Range;

const FIGHT: &str = "chain-any-0006-E0B5";
const STRIP_STARTS: [u16; 2] = [0x7220, 0x7320];
const STRIP_BYTES: u16 = 64;
const NEXT_BEAT: WramWord = WramWord {
    name: "CERES_RIDLEY_RIBS_NEXT_BEAT",
    offset: 0x780a,
};
const BEAT_WAIT: WramWord = WramWord {
    name: "CERES_RIDLEY_RIBS_WAIT",
    offset: 0x780c,
};
const LAST_FIGHT_CUT: u64 = 1614;

const BEAT_FIELD: &str = "ridley.ribs.beat";
const WAIT_FIELD: &str = "ridley.ribs.wait";
const UPLOADS_FIELD: &str = "ridley.ribs.uploads";
const UPPER_FIELD: &str = "ridley.ribs.vram.upper";
const LOWER_FIELD: &str = "ridley.ribs.vram.lower";

pub(in crate::replay) fn decode_clock(wram: &[u8]) -> Result<FightRibs, Outcome> {
    let bad = || data_failure(None, "native ribs beat and wait");
    let beat = match NEXT_BEAT.read(wram).map_err(|_| bad())? {
        0xda71 | 0xda89 => Beat::Closed,
        0xda77 => Beat::Opening,
        0xda7d => Beat::Wide,
        0xda83 => Beat::Closing,
        _ => return Err(bad()),
    };
    let wait = BEAT_WAIT.read(wram).map_err(|_| bad())?;
    let remaining = u8::try_from(wait)
        .ok()
        .and_then(RibWait::new)
        .ok_or_else(bad)?;
    Ok(FightRibs { beat, remaining })
}

fn clock(state: FightRibs) -> (Beat, u16) {
    (state.beat, u16::from(state.remaining.get()))
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
struct Receipt {
    destination: u16,
    source: BusAddress,
    bytes: u16,
}

fn receipts(record: &FrameRecord) -> Result<Vec<Receipt>, Outcome> {
    // At a serviced cut the NMI clears the tail but leaves the descriptors.
    // Read the leading adjacent rows, which precede the room's other uploads.
    let table = VRAM_WRITE_QUEUE
        .read(&record.wram)
        .map_err(|_| data_failure(Some(record.index), UPLOADS_FIELD))?;
    Ok(table
        .as_chunks::<7>()
        .0
        .iter()
        .zip(STRIP_STARTS)
        .filter_map(|(&[ll, lh, sl, sh, bank, dl, dh], destination)| {
            let from = BusAddress {
                bank,
                offset: u16::from_le_bytes([sl, sh]),
            };
            let to = u16::from_le_bytes([dl, dh]);
            let bytes = u16::from_le_bytes([ll, lh]);
            (to == destination && bytes == STRIP_BYTES && from.bank == 0xb0).then_some(Receipt {
                destination: to,
                source: from,
                bytes,
            })
        })
        .collect())
}

fn span(start: u16) -> Range<usize> {
    let start = usize::from(start);
    start..start + STRIP_WORDS
}

fn observations(
    (beat, wait): (Beat, u16),
    receipts: &[Receipt],
    vram: &[u16],
) -> Result<[Observation; 5], Outcome> {
    let beat = match beat {
        Beat::Closed => 0,
        Beat::Opening => 1,
        Beat::Wide => 2,
        Beat::Closing => 3,
    };
    let upload_bytes = receipts.iter().flat_map(|receipt| {
        receipt
            .bytes
            .to_le_bytes()
            .into_iter()
            .chain(receipt.source.offset.to_le_bytes())
            .chain([receipt.source.bank])
            .chain(receipt.destination.to_le_bytes())
    });
    let strip = |field: &str, start| {
        let words = vram
            .get(span(start))
            .ok_or_else(|| data_failure(None, field))?;
        Ok(Observation {
            field: field.into(),
            bytes: words.iter().flat_map(|word| word.to_le_bytes()).collect(),
        })
    };
    let [upper, lower] = STRIP_STARTS;
    Ok([
        Observation::quantity(BEAT_FIELD, beat),
        Observation::quantity(WAIT_FIELD, i64::from(wait)),
        Observation {
            field: UPLOADS_FIELD.into(),
            bytes: upload_bytes.collect(),
        },
        strip(UPPER_FIELD, upper)?,
        strip(LOWER_FIELD, lower)?,
    ])
}

pub(super) fn admit(video: &mut Video, seed: &FrameRecord) -> Result<(), Outcome> {
    for start in STRIP_STARTS {
        let words = seed
            .vram
            .get(span(start))
            .ok_or_else(|| data_failure(Some(seed.index), "ribs origin VRAM"))?;
        video
            .memory
            .vram
            .get_mut(span(start))
            .ok_or_else(|| data_failure(Some(seed.index), "ribs origin destination"))?
            .copy_from_slice(words);
    }
    Ok(())
}

pub(super) fn compare(
    frame: Frame<'_>,
    state: &played::State,
    serviced: &[VramTransfer],
    (expected, actual): (&mut Vec<Observation>, &mut Vec<Observation>),
) -> Result<(), Outcome> {
    if !witness(frame.place) {
        return Ok(());
    }
    let record = frame.current;
    let native = receipts(record)?;
    let ridley = state
        .station
        .ridley
        .ok_or_else(|| data_failure(Some(frame.cut), BEAT_FIELD))?;
    let observed = production_receipts(serviced);
    expected.extend(observations(
        clock(decode_clock(&record.wram)?),
        &native,
        &record.vram,
    )?);
    actual.extend(observations(
        clock(ridley.ribs),
        &observed,
        &state.room_video.video.memory.vram,
    )?);
    Ok(())
}

fn witness(place: (&str, u64)) -> bool {
    const MOVIE_FIGHT_START: u64 = 10_162;
    const MOVIE_FIGHT_END: u64 = 11_776;
    match place {
        (FIGHT, cut) => cut <= LAST_FIGHT_CUT,
        ("any-power-on", cut) => (MOVIE_FIGHT_START..=MOVIE_FIGHT_END).contains(&cut),
        _ => false,
    }
}

fn production_receipts(writes: &[VramTransfer]) -> Vec<Receipt> {
    writes
        .iter()
        .filter_map(|write| {
            let VramSource::Words { from, words } = &write.source else {
                return None;
            };
            if from.bank != 0xb0 {
                return None;
            }
            let destination = match write.step {
                VramStep::Row => write.to.0,
                VramStep::Column => write.to.0 | 0x8000,
            };
            Some(Receipt {
                destination,
                source: *from,
                bytes: u16::try_from(words.len().saturating_mul(2)).unwrap_or(u16::MAX),
            })
        })
        .collect()
}

const FIELDS: &[ObservationId] = &[ObservationId::CeresRibsBeat, ObservationId::CeresRibsWait];
const OUTPUTS: &[ObservationId] = &[
    ObservationId::CeresRibsUploads,
    ObservationId::CeresRibsUpper,
    ObservationId::CeresRibsLower,
];

pub(in crate::replay) fn is_case(case: &Case) -> bool {
    case.expectations.fields == FIELDS
}

pub(super) fn valid(case: &Case) -> bool {
    case.kind == CaseKind::Integrated
        && case.row == FIGHT
        && matches!(case.start, Start::CacheCut { .. })
        && case.end.cut_inclusive <= LAST_FIGHT_CUT
        && case.responsibilities == [Responsibility::Ceres]
        && case.content == ContentProvider::PlayerRomImport
        && case.inputs == InputProvider::PlayedCeresInputs
        && case.expectations.provider == ExpectedProvider::PinnedRecordCache
        && case.expectations.outputs == OUTPUTS
        && case.expectations.output_cut.is_none()
        && match (&case.expectations.played, &case.start) {
            (Some(PlayedWindow::RibsReveal { cut }), Start::CacheCut { frame }) => {
                case.checkpoints && frame < cut && *cut < case.end.cut_inclusive
            }
            (None, _) => !case.checkpoints,
            _ => false,
        }
}

pub(super) fn run(
    run: &mut Run<'_>,
    rows: &mut Rows<'_>,
    mut previous: FrameRecord,
    mut controller: Controller,
    case: &Case,
    result: &mut CaseResult,
) -> Result<(), Outcome> {
    let mut digest = Vec::new();
    let mut execute = || -> Result<(), Outcome> {
        for cut in previous.index.saturating_add(1)..=case.end.cut_inclusive {
            let current = rows.record(cut)?;
            if current.video_frame != previous.video_frame.wrapping_add(1) {
                return Err(data_failure(Some(cut), "played rows join"));
            }
            let held = matches!(case.expectations.played,
                Some(PlayedWindow::RibsReveal { cut: reveal }) if reveal == cut)
            .then(|| run.clone());
            let null = observations(
                clock(carried(run, previous.index)?),
                &[],
                &run.state.room_video.video.memory.vram,
            )?;
            let fields = run.step(controller, &previous, &current, cut, (FIGHT, cut))?;
            record_fields(fields, cut, result, &mut digest, &null)?;
            if let Some(origin) = held {
                pending_control(
                    origin,
                    controller,
                    &previous,
                    &current,
                    &rows.record(
                        cut.checked_add(1)
                            .ok_or_else(|| data_failure(Some(cut), "ribs release cut"))?,
                    )?,
                )?;
                result.counts.restorations = 1;
                result.counts.pending_boundaries = 1;
                result.counts.pending_restorations = 1;
            }
            controller = controller.read(current.pad.0, super::polled(&current));
            previous = current;
        }
        Ok(())
    };
    let outcome = execute();
    if !digest.is_empty() {
        result.observation_sha256 = crate::lock::Sha256::of(&digest).to_string();
    }
    outcome
}

fn record_fields(
    mut fields: Vec<Observation>,
    cut: u64,
    result: &mut CaseResult,
    digest: &mut Vec<u8>,
    null: &[Observation],
) -> Result<(), Outcome> {
    fields.retain(|field| field.field.starts_with("ridley.ribs."));
    let [beat, wait, uploads, _, _] = fields.as_slice() else {
        return Err(data_failure(Some(cut), "ribs replay observations"));
    };
    if !uploads.bytes.is_empty() {
        result.counts.publications = result.counts.publications.saturating_add(1);
    }
    let observed = ExpectedCut {
        semantic_count: 2,
        output_count: 3,
        semantics: vec![beat.clone(), wait.clone()],
        fields,
        replies: 0,
    };
    runner::record_cut(result, digest, &observed, null, cut)
}

fn carried(run: &Run<'_>, cut: u64) -> Result<FightRibs, Outcome> {
    run.state
        .station
        .ridley
        .map(|ridley| ridley.ribs)
        .ok_or_else(|| data_failure(Some(cut), BEAT_FIELD))
}

fn work(
    run: &mut Run<'_>,
    controller: Controller,
    previous: &FrameRecord,
    current: &FrameRecord,
    hold_publication: bool,
) -> Result<played::Outputs, Outcome> {
    let mut inputs = run.room.inputs.clone();
    inputs.held = movement::pad(controller.held);
    inputs.pressed = movement::pad(controller.pressed);
    inputs.parity = parity(run.state.frame.finished_frames);
    let mut passing = run.room_work(previous, current, (FIGHT, current.index), false)?;
    if !passing.publishes {
        return Err(data_failure(
            Some(current.index),
            "ribs control needs a requested publication",
        ));
    }
    passing.publishes = !hold_publication;
    run.state
        .step(
            &inputs,
            &passing,
            &hud::frame(&run.room.hud, controller, None),
            &run.played,
            &run.rules,
        )
        .map_err(|fault| Outcome::Unsupported {
            cut: Some(current.index),
            dependency: format!("played frame {fault:?}"),
        })
}

fn pending_control(
    mut direct: Run<'_>,
    controller: Controller,
    previous: &FrameRecord,
    first: &FrameRecord,
    after: &FrameRecord,
) -> Result<(), Outcome> {
    use canonical::model::{Model, pin_holds};
    use game::state::owned::FightRibsPin;

    let held = work(&mut direct, controller, previous, first, true)?;
    let native = receipts(first)?;
    let video = &direct.state.room_video.video;
    let pending = video
        .batch
        .vram
        .get(..native.len())
        .map(production_receipts);
    let differs = |field: &str| Outcome::Difference {
        cut: first.index,
        field: field.into(),
    };
    if !held.serviced_vram.is_empty() || native.len() != 2 || pending.as_ref() != Some(&native) {
        return Err(differs("ridley.ribs.pending-uploads"));
    }
    let held_clock = carried(&direct, first.index)?;
    if held_clock != decode_clock(&first.wram)? {
        return Err(differs(BEAT_FIELD));
    }
    for start in STRIP_STARTS {
        let words = video
            .memory
            .vram
            .get(span(start))
            .ok_or_else(|| differs("ridley.ribs.held-vram"))?;
        if Some(words) != previous.vram.get(span(start))
            || Some(words) != first.vram.get(span(start))
        {
            return Err(differs("ridley.ribs.held-vram"));
        }
    }

    let failed = || Outcome::CheckpointFailure { cut: after.index };
    pin_holds::<FightRibsPin>().map_err(|_| failed())?;
    let saved_clock = canonical::to_bytes(&Model::<FightRibsPin>(held_clock));
    let saved_picture = direct.state.room_video.picture_checkpoint();
    let mut restored = direct.clone();
    restored
        .state
        .station
        .ridley
        .as_mut()
        .ok_or_else(failed)?
        .ribs = canonical::from_bytes::<Model<FightRibsPin>>(&saved_clock)
        .map_err(|_| failed())?
        .0;
    *restored.state.room_video.video = Video::default();
    restored
        .state
        .room_video
        .restore_picture(&saved_picture)
        .map_err(|_| failed())?;

    let ready = controller.read(first.pad.0, super::polled(first));
    let direct_output = work(&mut direct, ready, first, after, false)?;
    let restored_output = work(&mut restored, ready, first, after, false)?;
    if direct_output != restored_output
        || direct.state != restored.state
        || production_receipts(&restored_output.serviced_vram) != native
        || !restored.state.room_video.video.batch.vram.is_empty()
        || carried(&restored, after.index)? != decode_clock(&after.wram)?
    {
        return Err(failed());
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    const FIRST_REVEAL: u64 = 577;
    const RELEASE_CUT: u64 = 578;
    use super::super::{open_rows, read_lock, tests};
    use super::*;
    use crate::source::VRAM_WRITE_QUEUE_TAIL;
    use data::assets::ridley_ribs::{Art, Strips};
    use data::{pose::tiles::Graphics, rom::Rom};
    use game::exact::ridley_ribs::queue;

    use super::super::{Stop, appearance, connected, seed_controller};

    fn content(sources: &crate::replay::Sources) -> connected::Content {
        connected::Content {
            movement: movement::Content::load(&sources.rom).unwrap(),
            appearance: appearance::Content::load(&sources.rom).unwrap(),
        }
    }

    #[test]
    #[ignore = "pinned record cache and player ROM"]
    fn production_ribs_advance_all_thirty_reveals() {
        let sources = tests::sources();
        let lock = read_lock(&sources).unwrap();
        let mut rows = open_rows(&sources, &lock, FIGHT).unwrap();
        let seed = rows.record(0).unwrap();
        let controller = seed_controller(&seed).unwrap();
        let content = content(&sources);
        let mut run = Run::admit(&seed, &content, &sources.rom, FIGHT).unwrap();
        let mut reveals = 0_usize;
        let (held, stop) = run
            .witness_through(
                &mut rows,
                seed,
                controller,
                1..=LAST_FIGHT_CUT,
                |_, _, fields| {
                    if fields
                        .iter()
                        .any(|field| field.field == UPLOADS_FIELD && !field.bytes.is_empty())
                    {
                        reveals = reveals.saturating_add(1);
                    }
                    Ok(())
                },
            )
            .unwrap();
        assert_eq!((held, stop), (LAST_FIGHT_CUT, Stop::End));
        assert_eq!(reveals, 30);
        tests::diagnostic(format_args!(
            "{held} fight cuts with {reveals} production ribs reveals"
        ));
    }

    #[test]
    #[ignore = "pinned record cache and player ROM"]
    fn production_ribs_restore_the_held_first_release() {
        let sources = tests::sources();
        let lock = read_lock(&sources).unwrap();
        let mut rows = open_rows(&sources, &lock, FIGHT).unwrap();
        let before = rows.record(576).unwrap();
        let first = rows.record(FIRST_REVEAL).unwrap();
        let after = rows.record(RELEASE_CUT).unwrap();
        let content = content(&sources);
        let run = Run::admit(&before, &content, &sources.rom, FIGHT).unwrap();
        pending_control(
            run,
            seed_controller(&before).unwrap(),
            &before,
            &first,
            &after,
        )
        .unwrap();
        tests::diagnostic(format_args!(
            "c577 held both transfers over the same picture and restored their c578 publication"
        ));
    }

    fn selected_art(art: &Art, native: &[Receipt]) -> Option<Strips> {
        [art.closed, art.partly_open, art.wide]
            .into_iter()
            .find(|look| {
                look.iter()
                    .map(|strip| strip.source)
                    .eq(native.iter().map(|write| write.source))
            })
    }

    #[test]
    #[ignore = "pinned record cache and player ROM"]
    fn ordinary_ribs_object_strips_match_native_reveals() {
        let sources = tests::sources();
        let lock = read_lock(&sources).unwrap();
        let mut rows = open_rows(&sources, &lock, FIGHT).unwrap();
        let rom = Rom::new(std::fs::read(&sources.rom).unwrap()).unwrap();
        let art = import::ridley_ribs::import(rom.lorom()).unwrap();
        let mut video = Video::default();
        for (start, strip) in STRIP_STARTS.into_iter().zip(art.closed) {
            video.memory.vram[span(start)].copy_from_slice(&strip.words);
        }

        let mut held = None;
        let mut null_cuts = 0_usize;
        let mut null_fields = 0_usize;
        let mut reveals = 0;
        let mut first_omitted_difference = None;
        let mut first_held_clock_difference = None;
        for cut in 0..=LAST_FIGHT_CUT {
            let record = rows.record(cut).unwrap();
            let clock = clock(decode_clock(&record.wram).unwrap());
            if clock != (Beat::Closed, 1) {
                first_held_clock_difference.get_or_insert(cut);
            }
            if cut == 1156 {
                assert_eq!(clock, (Beat::Opening, 1));
            }
            if cut >= 1157 {
                assert_eq!(clock, (Beat::Wide, 20), "ribs clock at fight cut {cut}");
            }
            let native = receipts(&record).unwrap();
            if let Some(previous) = &held {
                let expected = observations(clock, &native, &record.vram).unwrap();
                null_cuts = null_cuts.saturating_add(usize::from(&expected == previous));
                null_fields = null_fields.saturating_add(
                    expected[..2]
                        .iter()
                        .zip(previous)
                        .filter(|(a, b)| a == b)
                        .count(),
                );
            }
            held = Some(observations(clock, &[], &record.vram).unwrap());
            if !native.is_empty() {
                assert_eq!(
                    native.len(),
                    2,
                    "two native ribs descriptors at fight cut {cut}"
                );
                assert_eq!(VRAM_WRITE_QUEUE_TAIL.read(&record.wram).unwrap(), 0);
                let strips = selected_art(&art, &native).unwrap();
                if reveals == 0 {
                    assert_eq!(cut, FIRST_REVEAL);
                    let closed = strips == art.closed;
                    assert!(closed, "the first receipt names the closed art");
                    for start in STRIP_STARTS {
                        assert!(
                            video.memory.vram[span(start)] == record.vram[span(start)],
                            "doorway picture already shows the first reveal at {start:#06x}"
                        );
                    }
                }

                queue(&mut video.batch, &strips);
                if cut == FIRST_REVEAL {
                    check_publication_controls(&video, &native);
                }
                assert!(
                    restored_receipts(&mut video) == native,
                    "ribs upload receipt at fight cut {cut}"
                );
                reveals += 1;
            }
            for (start, closed) in STRIP_STARTS.into_iter().zip(art.closed) {
                assert!(
                    video.memory.vram[span(start)] == record.vram[span(start)],
                    "ribs strip at fight cut {cut}, VRAM {start:#06x}"
                );
                if closed.words != record.vram[span(start)] {
                    first_omitted_difference.get_or_insert(cut);
                }
            }
        }
        assert_eq!(reveals, 30);
        assert_eq!((null_cuts, null_fields), (1033, 2617));
        assert_eq!(first_held_clock_difference, Some(FIRST_REVEAL));
        assert_eq!(first_omitted_difference, Some(597));
        tests::diagnostic(format_args!(
            "{reveals} native receipts restored through c{LAST_FIGHT_CUT}. \
             Omitted and shifted receipts differ at c{FIRST_REVEAL}. \
             The omitted picture first differs at c597."
        ));
    }

    fn restored_receipts(video: &mut Video) -> Vec<Receipt> {
        *video = canonical::from_bytes(&canonical::to_bytes(video)).unwrap();
        let serviced = video.process_vram_writes(&Graphics::default());
        assert_eq!(video.batch.vram.len(), 0);
        production_receipts(&serviced)
    }

    fn check_publication_controls(video: &Video, native: &[Receipt]) {
        let mut omitted = video.clone();
        omitted.batch.vram.clear();
        assert_eq!(restored_receipts(&mut omitted).len(), 0);
        assert!(
            !native.is_empty(),
            "an omitted first reveal loses both receipts"
        );

        let mut shifted = video.clone();
        let destination = &mut shifted.batch.vram[0].to.0;
        *destination = destination.checked_add(1).unwrap();
        let receipts = restored_receipts(&mut shifted);
        assert!(
            receipts != native,
            "a shifted upper row differs at the first receipt"
        );
        assert_eq!(receipts[1].destination, STRIP_STARTS[1]);
    }
}
