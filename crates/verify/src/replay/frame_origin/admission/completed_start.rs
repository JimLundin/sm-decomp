//! Physical origin admission before the first scheduled Frame work.

use super::super::{Member, borrowed_receipts, hardware_seed, sender_seed};
use crate::replay::{Sources, data_failure, index::Case, readers::AdmittedCase, result::Outcome};
use data::timing::MasterTicks;
use game::production::{frame, timing::PhysicalPad};
use serde::Deserialize;
use std::path::Path;

const CUT: u64 = 156;
const FIRST: u64 = 157;
const ROW: &str = "chain-any-0003-DFD7";
const WRAM_PIN: &str = "aa0021a536771517434717a5291c1158c77c8c0e529ef8d2dd28bb5c195052be";
const HARDWARE_PIN: &str = "b3280d60c8e7bfbef0eb52551e71d6c68bf69f85fd1c8a7603021d2ede26c345";
const APURAM_PIN: &str = "a4d6d26942942ed370679b7031a3658f3baca148fe4319a5b7850e8494bceda3";
const SENDER_PIN: &str = "a1898ac6651656531a0f4c40274ad302376577b946a1a2dcb3f03123caaa9a34";
const MUSIC_PIN: &str = "79aa686411599925816aac2e4fbcaebb76fb29969459b707dee1983d75242f80";
const RESUME_PIN: &str = "2c17b473a83bed0482b80cd985aae25fa5f0e9929a986cd15ae378ce4af290ff";
const ORIGIN_PIN: &str = "f7d1fb29d2b179f874f41efcacd9f02bdff5663d04c67abc6b4e6484c4c4140e";

#[derive(Deserialize)]
struct Format {
    format: u16,
}

pub(super) fn recognizes(text: &str) -> Result<bool, Outcome> {
    let format: Format =
        toml::from_str(text).map_err(|_| data_failure(None, "frame origin format"))?;
    Ok(format.format == 2)
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct Donor {
    #[serde(rename = "format")]
    _format: u16,
    cut: u64,
    phase: OriginPhase,
    origin: Member,
    hardware: Member,
    apuram: Member,
    sender_origin: Member,
    music_completion: Member,
    profiles: Option<[Member; 6]>,
}

#[derive(Deserialize, PartialEq, Eq)]
#[serde(rename_all = "kebab-case")]
enum OriginPhase {
    RtiPrefetch,
    Resume,
}

#[derive(Deserialize)]
struct SenderClock {
    #[serde(alias = "format")]
    schema: u16,
    cut: u64,
    service_pending: bool,
    frame_origin_master_from_locked_load: u64,
    previous_contact_end_relative: i64,
    next_contact_begin_relative: i64,
    sender_now_relative: u64,
}

impl SenderClock {
    fn validate(&self, timing: MasterTicks) -> Result<(), Outcome> {
        if self.schema != 1
            || self.cut != CUT
            || self.service_pending
            || self.previous_contact_end_relative >= 0
            || self.next_contact_begin_relative <= 0
            || self.previous_contact_end_relative.unsigned_abs()
                > self.frame_origin_master_from_locked_load
            || self.sender_now_relative != 0
            || timing != MasterTicks(0)
        {
            return Err(data_failure(Some(CUT), "completed sender clock join"));
        }
        Ok(())
    }
}

struct Origins {
    hardware: game::production::timing::State,
    senders: frame::Senders,
    music: frame::borrowed_receipts::MusicCompletionReceipt,
    profiles: Option<[Member; 6]>,
}

impl Donor {
    fn load(sources: &Sources, root: &Path, text: &str) -> Result<Origins, Outcome> {
        let mut donor: Self = toml::from_str(text)
            .map_err(|_| data_failure(Some(CUT), "completed frame donor manifest"))?;
        if donor.cut != CUT || donor.phase != OriginPhase::RtiPrefetch {
            return Err(data_failure(Some(CUT), "completed frame donor cut"));
        }
        for member in [
            &mut donor.origin,
            &mut donor.hardware,
            &mut donor.apuram,
            &mut donor.sender_origin,
            &mut donor.music_completion,
        ] {
            member.resolve(root);
        }
        if let Some(members) = &mut donor.profiles {
            for member in members {
                member.resolve(root);
            }
        }
        let bytes = donor.origin.read()?;
        let mut record = oracle_record::lsnes::blank();
        oracle_record::lsnes::decode(&bytes, &mut record)
            .map_err(|_| data_failure(Some(CUT), "completed native donor record"))?;
        if !record
            .nmi()
            .is_some_and(|end| end.kind == oracle_record::NmiKind::Requested)
        {
            return Err(data_failure(Some(CUT), "completed requested NMI"));
        }
        let raw_hardware = donor.hardware.read()?;
        let hardware = hardware_seed::decode(
            &raw_hardware,
            data::timing::IrqPolicy::Eligible,
            PhysicalPad(record.pad.0),
        )
        .map_err(|_| data_failure(Some(CUT), "completed-return physical origin"))?;
        if hardware.hardware.controller.words.first().copied() != Some(record.pad.0) {
            return Err(data_failure(Some(CUT), "completed controller sample"));
        }
        let senders = contact_origin(
            sources,
            &record.wram,
            &raw_hardware,
            hardware.hardware.now,
            &donor.apuram,
            &donor.sender_origin,
            &donor.music_completion,
        )?;
        let music = borrowed_receipts::qualified_music_completion(
            &record.wram,
            &donor.music_completion.text()?,
        )
        .map_err(|_| data_failure(Some(CUT), "completed frame Music receipt"))?;
        Ok(Origins {
            hardware,
            senders,
            music,
            profiles: donor.profiles,
        })
    }
}

fn contact_origin(
    sources: &Sources,
    wram: &[u8],
    hardware: &[u8],
    now: MasterTicks,
    apuram: &Member,
    sender: &Member,
    music: &Member,
) -> Result<frame::Senders, Outcome> {
    let pin_matches = |bytes: &[u8], pin: &str| {
        crate::lock::Sha256::try_from(pin).ok() == Some(crate::lock::Sha256::of(bytes))
    };
    if !pin_matches(wram, WRAM_PIN)
        || !pin_matches(hardware, HARDWARE_PIN)
        || apuram.sha256 != APURAM_PIN
        || sender.sha256 != SENDER_PIN
        || music.sha256 != MUSIC_PIN
    {
        return Err(data_failure(Some(CUT), "completed sender same-origin pins"));
    }
    let clock: SenderClock = toml::from_str(&sender.text()?)
        .map_err(|_| data_failure(Some(CUT), "completed sender clock source"))?;
    clock.validate(now)?;
    current_senders(sources, wram, &apuram.read()?)
}

fn current_senders(
    sources: &Sources,
    wram: &[u8],
    apuram: &[u8],
) -> Result<frame::Senders, Outcome> {
    let resident_room = crate::source::ROOM_PTR
        .read(wram)
        .ok()
        .and_then(|pointer| import::world::id(data::room::RoomKey::from_room_pointer(pointer)))
        .ok_or_else(|| data_failure(Some(CUT), "completed donor room identity"))?;
    let image =
        std::fs::read(&sources.rom).map_err(|_| data_failure(Some(CUT), "completed donor ROM"))?;
    let cartridge = data::rom::Rom::new(image)
        .map_err(|_| data_failure(Some(CUT), "completed donor ROM pin"))?;
    let resident = import::audio::resident_ceres(&cartridge, resident_room)
        .map_err(|_| data_failure(Some(CUT), "completed donor resident cue aliases"))?;
    sender_seed::completed_contact(wram, apuram, &resident.aliases)
        .map_err(|_| data_failure(Some(CUT), "completed frame sender origins"))
}

#[derive(Deserialize)]
struct Resume {
    phase: OriginPhase,
    wram: Member,
    hardware: Member,
    apuram: Member,
    ack_music_template: Member,
}

fn read_resume(source: &Member) -> Result<(Resume, SenderClock), Outcome> {
    if source.sha256 != RESUME_PIN {
        return Err(data_failure(
            Some(CUT),
            "reviewed true-resume origin identity",
        ));
    }
    let text = source.text()?;
    let resume: Resume =
        toml::from_str(&text).map_err(|_| data_failure(Some(CUT), "resume owner manifest"))?;
    let clock: SenderClock =
        toml::from_str(&text).map_err(|_| data_failure(Some(CUT), "resume sender clock source"))?;
    if resume.phase != OriginPhase::Resume || resume.wram.sha256 != WRAM_PIN {
        return Err(data_failure(Some(CUT), "same-cut true-resume WRAM"));
    }
    Ok((resume, clock))
}

pub(super) fn resume_profiles(source: &Member) -> Result<[data::timing::Template; 2], Outcome> {
    let (resume, _) = read_resume(source)?;
    let mut member = resume.ack_music_template;
    let root = source
        .path
        .parent()
        .ok_or_else(|| data_failure(Some(CUT), "true-resume profile directory"))?;
    member.resolve(root);
    super::quiet_profile::acknowledgment_and_music(&member.text()?)
        .map_err(|_| data_failure(Some(CUT), "resume acknowledgment and Music profiles"))
}

pub(super) fn resume_senders(
    sources: &Sources,
    wram: &[u8],
    hardware: &[u8],
    timing: &game::production::timing::State,
    apuram: &Member,
    source: &Member,
    music: &Member,
) -> Result<frame::Senders, Outcome> {
    let (resume, clock) = read_resume(source)?;
    let matches = |bytes: &[u8], member: &Member| {
        crate::lock::Sha256::try_from(member.sha256.as_str()).ok()
            == Some(crate::lock::Sha256::of(bytes))
    };
    let controller = &timing.hardware.controller;
    if !matches(wram, &resume.wram)
        || !matches(hardware, &resume.hardware)
        || apuram.sha256 != resume.apuram.sha256
        || music.sha256 != MUSIC_PIN
        || controller.words.first().copied() != Some(controller.committed_pad.0)
    {
        return Err(data_failure(
            Some(CUT),
            "true-resume same-origin pins and controller",
        ));
    }
    clock.validate(timing.hardware.now)?;
    current_senders(sources, wram, &apuram.read()?)
}

pub(super) fn admit(
    case: &Case,
    sources: &Sources,
    root: &Path,
    text: &str,
) -> Result<AdmittedCase, Outcome> {
    if case.row != ROW || case.first_cut() != Some(FIRST) {
        return Err(data_failure(
            Some(CUT),
            "completed frame origin coordinates",
        ));
    }
    if crate::lock::Sha256::try_from(ORIGIN_PIN).ok()
        != Some(crate::lock::Sha256::of(text.as_bytes()))
    {
        return Err(data_failure(Some(CUT), "completed frame origin identity"));
    }
    let Origins {
        hardware,
        senders,
        music,
        profiles,
    } = Donor::load(sources, root, text)?;
    // The prefetch clock is local to the physical import. Frame still needs same-origin work.
    if hardware.hardware.now != MasterTicks(0)
        || senders.validate().is_err()
        || music.dependency != music.terminal
    {
        return Err(data_failure(Some(CUT), "completed frame owner relation"));
    }
    super::seed::templates(profiles.as_ref())?;
    Err(data_failure(None, "RTI-to-resume hardware continuation"))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::replay::index::{Index, Start};
    use audio_transport::{CpuMasterTicks, CueId, Request, SenderProgress, SoundGroup};
    use data::assets::audio::CueAlias;
    use game::production::{Production, ProductionContent, ProductionRules, ProductionSeed};

    #[test]
    #[ignore = "requires the reviewed native completed-frame donor and player ROM"]
    fn shot_frame_completed_origin_keeps_held_contact_and_reports_first_boundary() {
        let sources = Sources {
            native: std::env::var_os("SM_AUDIO_NATIVE").unwrap().into(),
            rom: std::env::var_os("SM_ROM").unwrap().into(),
            ..Sources::default()
        };
        let path = Path::new("shot-frame-origin-v1/origin.toml");
        let mut case = Index::read(Path::new("../../scenarios/replay/index.toml"))
            .unwrap()
            .cases
            .into_iter()
            .find(|case| case.id.0 == "frame/ordinary-requested-nmi-return")
            .unwrap();
        case.start = Start::NativeSeed {
            path: path.into(),
            sha256: ORIGIN_PIN.to_owned(),
        };
        case.end.cut_inclusive = FIRST;

        let admitted = crate::replay::run(&case, &sources);
        assert!(matches!(
            admitted.outcome,
            Outcome::DataFailure { cut: None, .. }
        ));
        assert_eq!(admitted.counts.cuts, 0);
        case.end.cut_inclusive = CUT;
        assert!(matches!(
            crate::replay::run(&case, &sources).outcome,
            Outcome::DataFailure { .. }
        ));

        let manifest = sources.native.join(path);
        let root = manifest.parent().unwrap();
        let text = std::fs::read_to_string(&manifest).unwrap();
        let origins = Donor::load(&sources, root, &text).unwrap();
        assert!(origins.profiles.is_none());
        assert!(super::super::seed::templates(origins.profiles.as_ref()).is_err());
        let contact = origins.senders.get(SoundGroup::Contact);
        assert!(contact.queue.is_empty() && contact.pending.is_none());
        assert_eq!(contact.progress, SenderProgress::AwaitClear);
        assert_eq!(contact.reply, Request::Cue(CueId::Footstep));
        assert_eq!(contact.now, CpuMasterTicks(0));
        let content = ProductionContent::AudioSender(SoundGroup::Contact);
        let rules = ProductionRules::AudioSender;
        let direct = Production::admit(
            ProductionSeed::AudioSender(Box::new(contact.clone())),
            content.clone(),
            rules.clone(),
        )
        .unwrap();
        let saved = direct.save().unwrap();
        let restored = Production::restore(&saved, content, rules).unwrap();
        assert!(restored.eq(&direct), "restored sender differs");
        controls(root, &text);
    }

    fn controls(root: &Path, text: &str) {
        let donor: Donor = toml::from_str(text).unwrap();
        let mut clock: SenderClock =
            toml::from_str(&std::fs::read_to_string(root.join(donor.sender_origin.path)).unwrap())
                .unwrap();
        clock.service_pending = true;
        assert!(clock.validate(MasterTicks(0)).is_err());
        clock.service_pending = false;
        clock.cut = FIRST;
        assert!(clock.validate(MasterTicks(0)).is_err());
        clock.cut = CUT;
        assert!(clock.validate(MasterTicks(1)).is_err());

        let bytes = std::fs::read(root.join(donor.origin.path)).unwrap();
        let mut record = oracle_record::lsnes::blank();
        oracle_record::lsnes::decode(&bytes, &mut record).unwrap();
        let mut apuram = std::fs::read(root.join(donor.apuram.path)).unwrap();
        let held = apuram[0xf7];
        let alias = CueAlias {
            group: SoundGroup::Contact,
            byte: held,
            cue: CueId::Footstep,
        };
        assert!(sender_seed::completed_contact(&record.wram, &apuram, &[alias]).is_ok());
        apuram[0xf7] = 0;
        assert_eq!(
            sender_seed::completed_contact(&record.wram, &apuram, &[alias]),
            Err(sender_seed::SeedFault::Reply),
        );
        apuram[0xf7] = held;
        record.wram[0x064b] = 0;
        assert_eq!(
            sender_seed::completed_contact(&record.wram, &apuram, &[alias]),
            Err(sender_seed::SeedFault::Progress),
        );
    }
}

#[cfg(test)]
mod resume_test {
    use super::*;
    use frame::opening_prelude::{Opening, Profiles};
    use game::production::{
        AdmissionFault, AdvanceInput, Production, ProductionContent, ProductionOutputs,
        ProductionRules, ProductionSeed,
    };
    use game::state::owned::OwnedState;

    fn open(sources: &Sources, prefetch: Origins) -> Result<(Opening, Profiles), Outcome> {
        let root = sources.native.join("shot-frame-resume-v1");
        let source = Member {
            path: root.join("resume.toml"),
            sha256: RESUME_PIN.to_owned(),
        };
        let (mut donor, _) = read_resume(&source)?;
        for member in [&mut donor.wram, &mut donor.hardware, &mut donor.apuram] {
            member.resolve(&root);
        }
        let pad = prefetch.hardware.hardware.controller.committed_pad;
        let hardware = donor.hardware.read()?;
        let timing = hardware_seed::decode(&hardware, data::timing::IrqPolicy::Eligible, pad)
            .map_err(|_| data_failure(Some(CUT), "true-resume physical origin"))?;
        if timing.hardware.beam.position == prefetch.hardware.hardware.beam.position {
            return Err(data_failure(Some(CUT), "true-resume hardware phase"));
        }
        let wram = donor.wram.read()?;
        let music_source = Member {
            path: sources
                .native
                .join("shot-frame-origin-v1/music-completion.toml"),
            sha256: MUSIC_PIN.to_owned(),
        };
        let senders = resume_senders(
            sources,
            &wram,
            &hardware,
            &timing,
            &donor.apuram,
            &source,
            &music_source,
        )?;
        if senders != prefetch.senders {
            return Err(data_failure(Some(CUT), "resume sender ownership"));
        }
        let current_music = borrowed_receipts::music(&wram)
            .map_err(|_| data_failure(Some(CUT), "resume current Music owner"))?;
        let receipt = borrowed_receipts::publication(&wram)
            .map_err(|_| data_failure(Some(CUT), "resume software acknowledgment guard"))?;
        let [acknowledgment, music] = resume_profiles(&source)?;
        let opening = Opening::new(timing, senders, receipt, current_music, prefetch.music)
            .map_err(|_| data_failure(Some(CUT), "production opening owner admission"))?;
        Ok((
            opening,
            Profiles {
                acknowledgment,
                music,
            },
        ))
    }

    fn admit(opening: Opening, profiles: Profiles) -> (Production, ProductionContent) {
        let content = ProductionContent::FrameOpening(Box::new(profiles));
        let prod = Production::admit(
            ProductionSeed::FrameOpening(Box::new(opening)),
            content.clone(),
            ProductionRules::FrameOpening,
        )
        .unwrap();
        (prod, content)
    }

    fn owner(production: &Production) -> &Opening {
        let OwnedState::FrameOpening(owner) = production.state() else {
            panic!("Frame opening owner absent");
        };
        owner
    }

    fn step(production: &mut Production, budget: u64) -> ProductionOutputs {
        let outputs = production
            .advance(AdvanceInput::Frame {
                budget: MasterTicks(budget),
            })
            .unwrap();
        let ProductionOutputs::FrameOpening(_, events) = &outputs else {
            panic!("Frame opening outputs absent");
        };
        assert_eq!(events.len(), 0);
        outputs
    }

    fn reach_music(production: &mut Production) {
        for _ in 0..1024 {
            step(production, 8);
            if matches!(owner(production).caller(), frame::Caller::Music { .. }) {
                return;
            }
        }
        panic!("opening did not finish acknowledgment");
    }

    fn shortened_acknowledgment_is_observable(opening: &Opening, profiles: &Profiles) {
        let mut altered = profiles.clone();
        let last = altered.acknowledgment.runs.last_mut().unwrap();
        last.count = last.count.saturating_sub(1);
        if last.count == 0 {
            altered.acknowledgment.runs.pop();
        }
        let (mut mutant, _) = admit(opening.clone(), altered);
        reach_music(&mut mutant);
        assert!(owner(&mutant).timing().hardware.now < MasterTicks(232));
    }

    #[test]
    #[ignore = "requires independently reviewed true-resume and prelude fixtures"]
    fn shot_actual_resume_opening_restores_through_music() {
        let sources = Sources {
            native: std::env::var_os("SM_AUDIO_NATIVE").unwrap().into(),
            rom: std::env::var_os("SM_ROM").unwrap().into(),
            ..Sources::default()
        };
        let root = sources.native.join("shot-frame-origin-v1");
        let prefetch = Donor::load(
            &sources,
            &root,
            &std::fs::read_to_string(root.join("origin.toml")).unwrap(),
        )
        .unwrap();
        let (opening, profiles) = open(&sources, prefetch).unwrap();
        shortened_acknowledgment_is_observable(&opening, &profiles);
        let mut changed = profiles.clone();
        changed.music.runs.pop();
        let (mut direct, content) = admit(opening, profiles);
        reach_music(&mut direct);
        assert_eq!(owner(&direct).timing().hardware.now, MasterTicks(232));
        for _ in 0..32 {
            step(&mut direct, 1);
            if owner(&direct).timing().hardware.charge_debt != 0 {
                break;
            }
        }
        assert!(matches!(
            owner(&direct).caller(),
            frame::Caller::MusicRunning { .. }
        ));
        assert!(owner(&direct).timing().hardware.charge_debt != 0);
        assert!(owner(&direct).timing().hardware.now > MasterTicks(232));
        let saved = direct.save().unwrap();
        let rules = ProductionRules::FrameOpening;
        assert!(matches!(
            Production::restore(
                &saved,
                ProductionContent::FrameOpening(Box::new(changed)),
                rules.clone()
            ),
            Err(AdmissionFault::ContentChanged),
        ));
        let mut restored = Production::restore(&saved, content, rules).unwrap();
        for _ in 0..1024 {
            let result = step(&mut direct, 8);
            let resumed = step(&mut restored, 8);
            assert!(result.eq(&resumed), "restored Music publication prefix");
            if matches!(owner(&direct).caller(), frame::Caller::Haze { .. }) {
                break;
            }
        }
        let frame::Caller::Haze { prelude, .. } = owner(&direct).caller() else {
            panic!("opening did not complete Music");
        };
        assert!(prelude.origin().unwrap().music_completion.is_none());
        assert_eq!(owner(&direct).timing().hardware.now, MasterTicks(738));
        assert!(direct.eq(&restored), "restored Music completion");
        let saved = direct.save().unwrap();
        let refusal = direct.advance(AdvanceInput::Frame {
            budget: MasterTicks(8),
        });
        assert!(matches!(
            refusal,
            Err(game::production::ProductionFault::Unsupported(
                game::production::Unsupported::Frame(frame::Dependency::HazePreparation),
            ))
        ));
        assert!(
            saved.eq(&direct.save().unwrap()),
            "Haze refusal preserves state"
        );
    }
}
