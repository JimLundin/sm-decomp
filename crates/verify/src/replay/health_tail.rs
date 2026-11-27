//! One authenticated Health origin and its independently observed frames, compared at frame cuts.

use std::path::Path;

use game::production::{
    AdvanceInput, Production, ProductionContent, ProductionOutputs, ProductionRules,
    ProductionSeed,
    health_tail::{Content, Context, Frame, Output, State},
};
use game::state::owned::OwnedState;
use samus_health::{
    EnergyAmount, EnergyLoss, Impact, ProtectionFrames, Rules, Warning, WholeEnergy,
};
use serde::{Deserialize, Serialize};

use super::{
    Sources, data_failure, hud,
    index::{Case, FixtureFile, Start, Window},
    production_failure,
    readers::{ActualReader, AdmittedCase, ExpectedReader, InputReader},
    result::Outcome,
};
use crate::lock::Sha256;

mod hud_inputs;
mod observations;
#[cfg(test)]
mod tests;
pub(super) use observations::{Expected, OUTPUTS_PER_CUT, actual, field_count, initial};

#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub struct HealthPacket {
    contexts: FixtureFile,
    offers: FixtureFile,
    hud_operands: FixtureFile,
    expected: FixtureFile,
    offer_cuts: Vec<u64>,
    impact_cuts: Vec<u64>,
}

impl HealthPacket {
    pub(super) fn valid(&self) -> bool {
        let files = [
            &self.contexts,
            &self.offers,
            &self.hud_operands,
            &self.expected,
        ];
        files.iter().enumerate().all(|(index, file)| {
            file.valid()
                && files
                    .iter()
                    .take(index)
                    .all(|other| other.path != file.path && other.sha256 != file.sha256)
        })
    }
}

#[derive(Deserialize)]
struct Tape<T> {
    row: String,
    opportunity: Vec<T>,
}

impl<T> Tape<T> {
    fn frames(self, window: &Window<'_>, at: impl Fn(&T) -> u64) -> Result<Vec<T>, Outcome> {
        if self.row != window.row || self.opportunity.iter().map(at).ne(window.cuts.clone()) {
            return Err(data_failure(None, "Health tail row and schedule"));
        }
        Ok(self.opportunity)
    }
}

#[derive(Deserialize)]
struct WramTape<T> {
    initial_cut: u64,
    initial_wram_sha256: String,
    #[serde(flatten)]
    tape: Tape<T>,
}

impl<T> WramTape<T> {
    fn frames(
        self,
        window: &Window<'_>,
        wram_pin: &str,
        at: impl Fn(&T) -> u64,
    ) -> Result<Vec<T>, Outcome> {
        if self.initial_cut != window.origin || self.initial_wram_sha256 != wram_pin {
            return Err(data_failure(
                Some(window.origin),
                "Health tail origin WRAM and cut",
            ));
        }
        self.tape.frames(window, at)
    }
}

#[derive(Deserialize)]
struct ContextAtCut {
    end_cut: u64,
    gravity_suit: bool,
    varia_suit: bool,
    time_stopped: bool,
    protection_ticks_after_status: u8,
}

#[derive(Deserialize)]
struct OfferSource {
    initial_cut: u64,
    #[serde(flatten)]
    tape: Tape<OfferAtCut>,
}

#[derive(Deserialize)]
struct OfferAtCut {
    end_cut: u64,
    actor_protection_access: ProtectionAccess,
    delivery: Option<Delivery>,
}

#[derive(Deserialize)]
enum ProtectionAccess {
    ReadCurrent,
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct Delivery {
    damage_whole: u16,
}

#[derive(Clone)]
pub(super) struct Inputs {
    first: u64,
    frames: Vec<Frame>,
}

impl Inputs {
    fn load(
        packet: &HealthPacket,
        native: &Path,
        window: &Window<'_>,
        origin_wram: &[u8],
    ) -> Result<Self, Outcome> {
        let wram_pin = Sha256::of(origin_wram).to_string();
        let contexts: WramTape<ContextAtCut> = packet.contexts.read(native)?;
        let contexts_at_cut = contexts.frames(window, &wram_pin, |entry| entry.end_cut)?;
        let offers: OfferSource = packet.offers.read(native)?;
        if offers.initial_cut != window.origin {
            return Err(data_failure(None, "Health tail ingress origin"));
        }
        let offers_at_cut = offers.tape.frames(window, |entry| entry.end_cut)?;
        if contexts_at_cut
            .iter()
            .any(|entry| entry.protection_ticks_after_status != 1)
            || offers_at_cut
                .iter()
                .filter(|entry| entry.delivery.is_some())
                .map(|entry| entry.end_cut)
                .ne(packet.offer_cuts.iter().copied())
        {
            return Err(data_failure(None, "Health tail causal ingress schedule"));
        }
        let hud = hud_inputs::load(&packet.hud_operands, native, window, &wram_pin, origin_wram)?;
        let frames = contexts_at_cut
            .into_iter()
            .zip(offers_at_cut)
            .zip(hud)
            .zip(0..)
            .map(|(((context, offer), hud), played)| {
                let ProtectionAccess::ReadCurrent = offer.actor_protection_access;
                // The hit is worth what the record observes the tail deliver.
                let tail = match offer.delivery {
                    None => Impact::Absent,
                    Some(Delivery { damage_whole }) => Impact::EligibleHit {
                        worth: WholeEnergy::new(damage_whole).ok_or_else(|| {
                            data_failure(Some(offer.end_cut), "Health tail delivery")
                        })?,
                    },
                };
                Ok(Frame {
                    played,
                    context: Context {
                        time_stopped: context.time_stopped,
                        varia: context.varia_suit,
                        gravity: context.gravity_suit,
                    },
                    tail,
                    hud,
                })
            })
            .collect::<Result<_, _>>()?;
        Ok(Self {
            first: *window.cuts.start(),
            frames,
        })
    }

    pub(super) fn advance(&self, production: &mut Production, cut: u64) -> Result<Output, Outcome> {
        let frame = cut
            .checked_sub(self.first)
            .and_then(|offset| usize::try_from(offset).ok())
            .and_then(|index| self.frames.get(index))
            .ok_or_else(|| data_failure(Some(cut), "Health tail frame"))?;
        advance(production, cut, frame)
    }
}

fn advance(production: &mut Production, cut: u64, frame: &Frame) -> Result<Output, Outcome> {
    let ProductionOutputs::HealthTail(output) = production
        .advance(AdvanceInput::HealthTail(frame))
        .map_err(|fault| production_failure(cut, fault))?
    else {
        return Err(data_failure(Some(cut), "Health tail output scope"));
    };
    Ok(output)
}

fn word(wram: &[u8], at: usize, cut: Option<u64>) -> Result<u16, Outcome> {
    wram.get(at..)
        .and_then(<[u8]>::first_chunk::<2>)
        .copied()
        .map(u16::from_le_bytes)
        .ok_or_else(|| data_failure(cut, "Health record field"))
}

fn whole(wram: &[u8], at: usize, cut: Option<u64>) -> Result<WholeEnergy, Outcome> {
    WholeEnergy::new(word(wram, at, cut)?)
        .ok_or_else(|| data_failure(cut, "Health record energy range"))
}

/// Health's complete State as the record at `cut` holds it.
pub(super) fn decode(wram: &[u8], cut: Option<u64>) -> Result<samus_health::State, Outcome> {
    let energy = EnergyAmount {
        whole: whole(wram, 0x09c2, cut)?,
        fraction: word(wram, 0x0a4c, cut)?,
    };
    let pending_loss = EnergyLoss {
        whole: whole(wram, 0x0a50, cut)?,
        fraction: word(wram, 0x0a4e, cut)?,
    };
    let warning = match word(wram, 0x0a6a, cut)? {
        0 => Warning::Off,
        1 => Warning::On,
        _ => return Err(data_failure(cut, "Health record warning range")),
    };
    let protection = u8::try_from(word(wram, 0x18a8, cut)?)
        .ok()
        .and_then(ProtectionFrames::new)
        .ok_or_else(|| data_failure(cut, "Health record protection range"))?;
    Ok(samus_health::State {
        energy,
        pending_loss,
        warning,
        protection,
    })
}

fn admit_origin(origin_identity: Sha256, wram: &[u8], cut: u64) -> Result<State, Outcome> {
    Ok(State::new(
        origin_identity.0,
        decode(wram, Some(cut))?,
        hud::decode_state(wram)?,
        word(wram, 0x0a12, Some(cut))?,
    ))
}

pub(super) fn admit(case: &Case, sources: &Sources) -> Result<AdmittedCase, Outcome> {
    let Start::NativeRecord {
        frame,
        path,
        sha256,
    } = &case.start
    else {
        return Err(data_failure(None, "Health tail origin coordinate"));
    };
    let window = Window::after(case, *frame)?;
    let packet = case.health_packet()?;
    let bytes = std::fs::read(sources.native.join(path))
        .map_err(|_| data_failure(Some(*frame), "Health tail origin record"))?;
    let origin_identity = Sha256::of(&bytes);
    if origin_identity.to_string() != *sha256 {
        return Err(data_failure(Some(*frame), "Health tail origin record pin"));
    }
    let record = oracle_record::lsnes::read(&bytes)
        .map_err(|_| data_failure(Some(*frame), "Health tail origin record format"))?;
    let inputs = Inputs::load(packet, &sources.native, &window, &record.wram)?;
    let state = admit_origin(origin_identity, &record.wram, *frame)?;
    let hud = hud::Content::load(&sources.rom)?;
    let content = ProductionContent::HealthTail(Box::new(Content {
        origin_identity: origin_identity.0,
        pictures: hud.pictures,
        map: hud.map(&record.wram)?.clone(),
    }));
    let rules = ProductionRules::HealthTail(Rules::REFERENCE);
    let production = Production::admit(
        ProductionSeed::HealthTail(Box::new(state)),
        content.clone(),
        rules.clone(),
    )
    .map_err(|_| data_failure(Some(*frame), "Health tail production admission"))?;
    let expected: Tape<Expected> = packet.expected.read(&sources.native)?;
    let expected = expected.frames(&window, |entry| entry.end_cut)?;
    if expected
        .iter()
        .filter(|entry| entry.observed_impact())
        .map(|entry| entry.end_cut)
        .ne(packet.impact_cuts.iter().copied())
    {
        return Err(data_failure(
            None,
            "Health tail expected observation schedule",
        ));
    }
    Ok(AdmittedCase {
        production,
        content,
        rules,
        inputs: InputReader::HealthTail(inputs),
        expected: ExpectedReader::HealthTail(expected),
        actual: ActualReader::HealthTail,
        schedule: window.cuts.collect(),
    })
}

fn owner(production: &Production) -> Result<&State, Outcome> {
    let OwnedState::HealthTail(state) = production.state() else {
        return Err(data_failure(None, "Health tail carried owners"));
    };
    Ok(state)
}
