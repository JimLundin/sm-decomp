//! The shaft handoff on one seed, plus a second seed inside the final unfinished room batch.
pub(super) mod hud;
pub(super) mod observe;
pub(super) mod seed;
pub(super) mod standing;
use super::borrowed::escape::Origin;
use super::index::{Case, Start};
use super::readers::{
    ActualReader, AdmittedCase, ExpectedCut, ExpectedReader, InputReader, Tape, record,
};
use super::{Sources, data_failure, production_failure, result::Outcome};
use data::rom::Rom;
use game::production::{
    self, AdvanceInput, Production, ProductionContent, ProductionOutputs, ProductionRules,
    ProductionSeed, blackout,
};
pub(super) use observe::actual;
use oracle_record::{Cut, NmiKind};

pub(super) const MEMORY: &str = "blackout.memory";
/// The names a handoff case declares. The standing body cases declare their own.
pub(super) const NAMES: [&str; 6] = [
    "blackout.scene",
    "blackout.room",
    "blackout.timer-drawing",
    "blackout.battery",
    "blackout.requests",
    MEMORY,
];

fn standing_body(case: &Case) -> bool {
    case.rules.iter().any(|rule| rule == standing::RULE)
}

impl Tape<blackout::Cut> {
    pub(super) fn advance(
        &self,
        production: &mut Production,
        cut: u64,
    ) -> Result<ProductionOutputs, Outcome> {
        production
            .advance(AdvanceInput::Blackout(*self.at(cut)?))
            .map_err(|why| production_failure(cut, why))
    }
}

fn origin(case: &Case, record: &oracle_record::FrameRecord) -> Result<Origin, Outcome> {
    let bad = || data_failure(Some(record.index), "blackout elevator origin");
    if !elevator_room(record)? {
        return Err(bad());
    }
    let memory = !standing_body(case) && case.declares(&NAMES, MEMORY)?;
    if word(&record.wram, 0x998)? == 8
        && word(&record.wram, 0x93f)? & 0x8000 != 0
        && matches!(record.cut, Cut::NmiReturn(nmi) if nmi.kind == NmiKind::Requested)
        && !standing_body(case)
        && !memory
    {
        return Ok(Origin::Reach);
    }
    let late = super::played::explosion::shaft_interrupted(record)?;
    if !super::played::explosion::shaft_starts(record)? || (!standing_body(case) && memory != late)
    {
        return Err(bad());
    }
    Ok(match (standing_body(case), late) {
        (true, true) => Origin::SuspendedStanding,
        (true, false) => Origin::Standing,
        (false, true) => Origin::Suspended,
        (false, false) => Origin::Locked,
    })
}

pub(super) fn admit(case: &Case, sources: &Sources) -> Result<AdmittedCase, Outcome> {
    let Start::CacheCut { frame } = case.start else {
        return Err(data_failure(None, "blackout seed"));
    };
    let cache = super::row_cache(sources, &case.row, case.end.cut_inclusive)?;
    let rom =
        Rom::new(std::fs::read(&sources.rom).map_err(|_| data_failure(None, "blackout ROM file"))?)
            .map_err(|_| data_failure(None, "blackout ROM pin"))?;
    let format = game::exact::save_files::Format::load(rom.lorom())
        .map_err(|_| data_failure(None, "cartridge format"))?;
    let figures =
        import::countdown::import(rom.lorom()).map_err(|_| data_failure(None, "timer art"))?;
    let mut previous = record(&cache, frame)?;
    let body = standing_body(case);
    let origin = origin(case, &previous)?;
    origin.check(case, frame)?;
    let late = matches!(origin, Origin::Suspended | Origin::SuspendedStanding);
    let mut state = seed::read(&previous, rom.lorom(), &format, late)?;
    let locked_look = if body && !late {
        Some(standing::LockedLook::read(&previous, rom.lorom())?)
    } else {
        None
    };
    let art = body
        .then(|| standing::admit_art(&previous, &mut state, rom.lorom(), late))
        .transpose()?;
    let content = ProductionContent::Blackout(Box::new(blackout::Content {
        figures,
        format,
        graphics: data::pose::tiles::Graphics::default(),
        standing: art,
        hud: None,
    }));
    let rules = ProductionRules::Blackout;
    let production = Production::admit(
        ProductionSeed::Blackout(Box::new(state)),
        content.clone(),
        rules.clone(),
    )
    .map_err(|why| data_failure(Some(frame), &format!("blackout admission {why:?}")))?;
    let schedule: Vec<_> = (frame.saturating_add(1)..=case.end.cut_inclusive).collect();
    let mut inputs = Vec::new();
    let mut expected = Vec::new();
    for &cut in &schedule {
        let current = record(&cache, cut)?;
        let mut input = work(
            &previous,
            &current,
            origin == Origin::Reach && cut == frame.saturating_add(1),
        )?;
        if matches!(input.work, blackout::Work::Fresh { .. })
            && let Some(look) = &locked_look
        {
            input.standing = Some(look.drawing(&previous)?);
        }
        inputs.push(input);
        expected.push(if body {
            standing::expected(&current, rom.lorom())?
        } else {
            observe::expected(&previous, &current, late)?
        });
        previous = current;
    }
    let first = frame.saturating_add(1);
    Ok(AdmittedCase {
        production,
        content,
        rules,
        schedule,
        inputs: InputReader::Blackout(Tape::new(first, inputs)),
        expected: ExpectedReader::Blackout(Tape::new(first, expected)),
        actual: if body {
            ActualReader::StandingBody
        } else {
            ActualReader::Blackout(late)
        },
    })
}

pub(super) fn elevator_room(record: &oracle_record::FrameRecord) -> Result<bool, Outcome> {
    let room = data::room::RoomKey::from_room_pointer(word(&record.wram, 0x079b)?);
    Ok(import::world::id(room) == Some(data::assets::world::rooms::CERES_ELEVATOR_ROOM))
}

const FINAL_ROOM_WAIT: u32 = 0x0080_82ce;

/// A new room turn finished its work before the NMI suspended publication.
pub(super) fn final_room_overrun(
    previous: &oracle_record::FrameRecord,
    current: &oracle_record::FrameRecord,
) -> bool {
    matches!(previous.cut, Cut::NmiReturn(nmi) if nmi.kind == NmiKind::Requested)
        && matches!(current.cut, Cut::NmiReturn(nmi)
            if nmi.kind == NmiKind::Overrun && nmi.interrupted_pc == FINAL_ROOM_WAIT)
}

pub(super) fn work(
    previous: &oracle_record::FrameRecord,
    current: &oracle_record::FrameRecord,
    reached: bool,
) -> Result<blackout::Cut, Outcome> {
    let Cut::NmiReturn(nmi) = current.cut else {
        return Err(data_failure(Some(current.index), "blackout NMI"));
    };
    let work = match previous.cut {
        Cut::NmiReturn(prior) => match prior.interrupted_pc {
            FINAL_ROOM_WAIT => blackout::Work::VerticalEnded,
            0x0081_8066 if nmi.kind == NmiKind::Requested => blackout::Work::SaveCompleted,
            _ if prior.kind == NmiKind::Requested => blackout::Work::Fresh { reached },
            _ => blackout::Work::Interrupted,
        },
        Cut::VblankWithoutNmi | Cut::NmiStillRunning { .. } => blackout::Work::Interrupted,
    };
    Ok(blackout::Cut {
        work,
        publish: nmi.kind == NmiKind::Requested,
        standing: None,
    })
}

fn byte(r: &[u8], at: usize) -> Result<u8, Outcome> {
    r.get(at)
        .copied()
        .ok_or_else(|| data_failure(None, "blackout record byte"))
}
pub(in crate::replay) fn word(r: &[u8], at: usize) -> Result<u16, Outcome> {
    r.get(at..)
        .and_then(<[_]>::first_chunk::<2>)
        .map(|b| u16::from_le_bytes(*b))
        .ok_or_else(|| data_failure(None, "blackout record word"))
}

fn digits(r: &[u8]) -> Result<u32, Outcome> {
    let decimal = |at| {
        r.get(at)
            .copied()
            .map(|n: u8| {
                u32::from(n >> 4)
                    .saturating_mul(10)
                    .saturating_add(u32::from(n & 15))
            })
            .ok_or_else(|| data_failure(None, "timer digits"))
    };
    Ok(decimal(0x947)?
        .saturating_mul(6000)
        .saturating_add(decimal(0x946)?.saturating_mul(100))
        .saturating_add(decimal(0x945)?))
}

#[cfg(test)]
mod owned_standing {
    use super::*;
    use crate::replay::{appearance, camera, compared, played};
    use canonical::model::Model;
    use game::production::blackout::StandingFrame;
    use game::state::video::Mode7;
    use types::Parts as _;

    #[test]
    #[ignore = "pinned record cache and player ROM"]
    fn first_locked_turn_reads_the_carried_look() -> Result<(), Outcome> {
        let sources = played::tests::sources();
        let cache = crate::replay::row_cache(&sources, "chain-any-0011-DF45", 165).unwrap();
        let prior = record(&cache, 164)?;
        let current = record(&cache, 165)?;
        let image = Rom::new(std::fs::read(&sources.rom).unwrap()).unwrap();
        let rom = image.lorom();
        let format = game::exact::save_files::Format::load(rom).unwrap();
        let samus = played::samus::seed(&prior.wram, &appearance::Content::load(&sources.rom)?)?;
        let (state, content) = first_owners(&prior, rom, format, samus.movement.0.position)?;
        let mut production = Production::admit(
            ProductionSeed::Blackout(Box::new(state)),
            ProductionContent::Blackout(Box::new(content)),
            ProductionRules::Blackout,
        )
        .unwrap();
        let camera = camera::seed(&prior.wram, &camera::Content)?;
        let view = Mode7 {
            centre: [
                word(&prior.wram, 0x80)?.cast_signed(),
                word(&prior.wram, 0x82)?.cast_signed(),
            ],
            matrix: [
                word(&prior.wram, 0x78)?.cast_signed(),
                word(&prior.wram, 0x7a)?.cast_signed(),
                0,
                0,
            ],
            ..Mode7::default()
        };
        let standing = StandingFrame::from_locked(&samus, camera.position(), view);
        assert_eq!(
            standing,
            standing::LockedLook::read(&prior, rom)?.drawing(&prior)?
        );
        let cut = blackout::Cut {
            standing: Some(standing),
            ..work(&prior, &current, false)?
        };
        let ProductionOutputs::Blackout(output) =
            production
                .advance(AdvanceInput::Blackout(cut))
                .map_err(|fault| crate::replay::production_failure(165, fault))?
        else {
            return Err(data_failure(Some(165), "locked body output"));
        };
        compared(
            165,
            &standing::expected(&current, rom)?.fields,
            &standing::actual(&production, &output)?,
        )?;
        compared(
            165,
            &hud::expected(&prior, &current)?,
            &hud::actual(&production, &output.regular_publication)?,
        )
    }

    fn first_owners(
        prior: &oracle_record::FrameRecord,
        rom: data::rom::Lorom<'_>,
        format: game::exact::save_files::Format,
        position: types::Position,
    ) -> Result<(blackout::State, blackout::Content), Outcome> {
        let mut state = seed::read(prior, rom, &format, false)?;
        let hud = carried_hud(prior, rom, &mut state, position)?;
        let standing = standing::admit_art(prior, &mut state, rom, false)?;
        Ok((
            state,
            blackout::Content {
                figures: import::countdown::import(rom).unwrap(),
                graphics: data::pose::tiles::Graphics::default(),
                format,
                standing: Some(standing),
                hud: Some(hud),
            },
        ))
    }

    fn carried_hud(
        prior: &oracle_record::FrameRecord,
        rom: data::rom::Lorom<'_>,
        state: &mut blackout::State,
        position: types::Position,
    ) -> Result<blackout::HudContent, Outcome> {
        let wram = &prior.wram;
        let assets = crate::replay::hud::Content::from_rom(rom)?;
        let map = assets.map(wram)?.clone();
        let map_room = crate::replay::hud::room_from_words(
            [word(wram, 0x07a1)?, word(wram, 0x07a3)?],
            [word(wram, 0x07a5)?, word(wram, 0x07a7)?],
        );
        let ordinary = played::hud::frame(
            &played::hud::held(wram)?,
            played::platform::Controller::seeded(wram)?,
            None,
        );
        let (model, assets) = blackout::LockedHud::from_ordinary(
            &Model(crate::replay::hud::decode_state(wram)?),
            crate::replay::hud::frame_count(wram)?.0,
            (assets.pictures, map, map_room),
            types::Pair {
                x: position.x.pixel().0,
                y: position.y.pixel().0,
            },
            state.owners.0.health.energy.whole.get(),
            &ordinary,
        );
        state.hud = Some(model);
        state.owners.0.countdown = interface_hud::Countdown::Dormant;
        Ok(assets)
    }
}
