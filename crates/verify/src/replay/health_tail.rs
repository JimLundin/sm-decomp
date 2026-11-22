//! One authenticated Health origin and bounded, independently observed actor ingress.

use std::path::Path;

use game::production::{
    AdvanceInput, Production, ProductionContent, ProductionOutputs, ProductionRules,
    ProductionSeed,
    health_tail::{Content, Context, Input, Output, State},
};
use game::state::owned::OwnedState;
use samus_health::{EnergyAmount, EnergyLoss, ProtectionFrames, Rules, Warning, WholeEnergy};
use serde::Deserialize;

use super::{
    Sources, data_failure, hud,
    index::{Case, Start},
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

pub(super) const ROW: &str = "chain-any-0006-E0B5";
pub(super) const ORIGIN: u64 = 992;
pub(super) const CUTS: [u64; 2] = [993, 994];

const CONTEXT_PIN: &str = "8ca15c6669c31e4605046a10fefd772fed5a2a3f4f842aeef2a7655c791e49cf";
const EXPECTED_PIN: &str = "813b6b8d06f6826b78925ae426278f288d7329f9167c8492d018667ebb9b6884";
const OFFER_PIN: &str = "23b8deabfaa84f7129bf4f46b36f1027c53a3e067b9d2115a12b8e476b7e568c";

#[derive(Deserialize)]
struct Tape<T> {
    row: String,
    opportunity: [T; 2],
}

#[derive(Deserialize)]
struct ContextSource {
    initial_cut: u64,
    initial_wram_sha256: String,
    #[serde(flatten)]
    tape: Tape<ContextAtCut>,
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

#[derive(Clone, Copy)]
struct Opportunity {
    maintenance: Context,
    delivery: Option<u16>,
    hud: hud_inputs::Inputs,
}

pub(super) struct Inputs(Vec<Opportunity>);

fn read<T: serde::de::DeserializeOwned>(
    source: &Path,
    path: &str,
    pin: &str,
) -> Result<T, Outcome> {
    let bytes = std::fs::read(source.join(path))
        .map_err(|_| data_failure(None, "Health tail fixture member"))?;
    if Sha256::of(&bytes).to_string() != pin {
        return Err(data_failure(None, "Health tail fixture member pin"));
    }
    let text = std::str::from_utf8(&bytes)
        .map_err(|_| data_failure(None, "Health tail fixture encoding"))?;
    toml::from_str(text).map_err(|_| data_failure(None, "Health tail fixture schema"))
}

impl Inputs {
    fn load(native: &Path) -> Result<(Self, String), Outcome> {
        let contexts: ContextSource =
            read(native, "ceres-health-tail-v1/borrowed.toml", CONTEXT_PIN)?;
        let offers: OfferSource = read(native, "ceres-tail-offer-v1/actor-offer.toml", OFFER_PIN)?;
        let contexts_at_cut = contexts.tape.opportunity;
        let offers_at_cut = offers.tape.opportunity;
        if contexts.initial_cut != ORIGIN
            || offers.initial_cut != ORIGIN
            || contexts.tape.row != ROW
            || offers.tape.row != ROW
            || contexts_at_cut.each_ref().map(|entry| entry.end_cut) != CUTS
            || offers_at_cut.each_ref().map(|entry| entry.end_cut) != CUTS
            || contexts_at_cut
                .iter()
                .any(|entry| entry.protection_ticks_after_status != 1)
            || offers_at_cut
                .each_ref()
                .map(|entry| entry.delivery.is_some())
                != [true, false]
        {
            return Err(data_failure(None, "Health tail causal ingress schedule"));
        }
        let hud = hud_inputs::load(native, &contexts.initial_wram_sha256)?;
        let opportunities = contexts_at_cut
            .into_iter()
            .zip(offers_at_cut)
            .zip(hud)
            .map(
                |((context, offer), hud)| match offer.actor_protection_access {
                    ProtectionAccess::ReadCurrent => Opportunity {
                        maintenance: Context {
                            time_stopped: context.time_stopped,
                            varia: context.varia_suit,
                            gravity: context.gravity_suit,
                        },
                        delivery: offer.delivery.map(|delivery| delivery.damage_whole),
                        hud,
                    },
                },
            )
            .collect();
        Ok((Self(opportunities), contexts.initial_wram_sha256))
    }

    pub(super) fn advance(
        &self,
        production: &mut Production,
        cut: u64,
    ) -> Result<Vec<Output>, Outcome> {
        let opportunity = CUTS
            .iter()
            .position(|at| *at == cut)
            .and_then(|index| self.0.get(index))
            .ok_or_else(|| data_failure(Some(cut), "Health tail runtime opportunity"))?;
        let steps = Self::steps(*opportunity).collect::<Vec<_>>();
        advance(production, cut, &steps)
    }

    fn steps(opportunity: Opportunity) -> impl Iterator<Item = Input> {
        [
            opportunity.hud.controls,
            Input::Protection(opportunity.maintenance),
        ]
        .into_iter()
        .chain(opportunity.delivery.map(Input::Impact))
        .chain([
            opportunity.hud.map,
            Input::Maintenance,
            Input::FirstDrawingRead,
            Input::SecondDrawingRead,
            opportunity.hud.status,
            Input::ActiveClose,
        ])
    }
}

fn advance(production: &mut Production, cut: u64, steps: &[Input]) -> Result<Vec<Output>, Outcome> {
    let ProductionOutputs::HealthTail(output) = production
        .advance(AdvanceInput::HealthTail(steps))
        .map_err(|fault| production_failure(cut, fault))?
    else {
        return Err(data_failure(Some(cut), "Health tail output scope"));
    };
    Ok(output)
}

fn word(wram: &[u8], at: usize) -> Result<u16, Outcome> {
    wram.get(at..)
        .and_then(<[u8]>::first_chunk::<2>)
        .copied()
        .map(u16::from_le_bytes)
        .ok_or_else(|| data_failure(Some(ORIGIN), "Health tail origin field"))
}

fn whole(wram: &[u8], at: usize) -> Result<WholeEnergy, Outcome> {
    WholeEnergy::new(word(wram, at)?)
        .ok_or_else(|| data_failure(Some(ORIGIN), "Health tail origin energy range"))
}

fn admit_origin(origin_identity: Sha256, wram: &[u8]) -> Result<State, Outcome> {
    let energy = EnergyAmount {
        whole: whole(wram, 0x09c2)?,
        fraction: word(wram, 0x0a4c)?,
    };
    let pending_loss = EnergyLoss {
        whole: whole(wram, 0x0a50)?,
        fraction: word(wram, 0x0a4e)?,
    };
    let warning = match word(wram, 0x0a6a)? {
        0 => Warning::Off,
        1 => Warning::On,
        _ => {
            return Err(data_failure(
                Some(ORIGIN),
                "Health tail origin warning range",
            ));
        }
    };
    let protection = u8::try_from(word(wram, 0x18a8)?)
        .ok()
        .and_then(ProtectionFrames::new)
        .ok_or_else(|| data_failure(Some(ORIGIN), "Health tail origin protection range"))?;
    Ok(State::new(
        origin_identity.0,
        samus_health::State {
            energy,
            pending_loss,
            warning,
            protection,
        },
        hud::decode_state(wram)?,
        word(wram, 0x0a12)?,
    ))
}

pub(super) fn admit(case: &Case, sources: &Sources) -> Result<AdmittedCase, Outcome> {
    let Start::NativeRecord {
        frame: ORIGIN,
        path,
        sha256,
    } = &case.start
    else {
        return Err(data_failure(None, "Health tail origin coordinate"));
    };
    let (inputs, wram_pin) = Inputs::load(&sources.native)?;
    let bytes = std::fs::read(sources.native.join(path))
        .map_err(|_| data_failure(Some(ORIGIN), "Health tail origin record"))?;
    let origin_identity = Sha256::of(&bytes);
    if origin_identity.to_string() != *sha256 {
        return Err(data_failure(Some(ORIGIN), "Health tail origin record pin"));
    }
    let record = oracle_record::lsnes::read(&bytes)
        .map_err(|_| data_failure(Some(ORIGIN), "Health tail origin record format"))?;
    if Sha256::of(&record.wram).to_string() != wram_pin {
        return Err(data_failure(Some(ORIGIN), "Health tail origin WRAM pin"));
    }
    let state = admit_origin(origin_identity, &record.wram)?;
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
    .map_err(|_| data_failure(Some(ORIGIN), "Health tail production admission"))?;
    let expected: Tape<Expected> = read(
        &sources.native,
        "ceres-health-tail-v1/expected.toml",
        EXPECTED_PIN,
    )?;
    if expected.row != ROW
        || expected.opportunity.each_ref().map(|entry| entry.end_cut) != CUTS
        || expected
            .opportunity
            .each_ref()
            .map(Expected::observed_impact)
            != [true, false]
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
        expected: ExpectedReader::HealthTail(Box::new(expected.opportunity)),
        actual: ActualReader::HealthTail,
        schedule: CUTS.to_vec(),
    })
}

fn owner(production: &Production) -> Result<&State, Outcome> {
    let OwnedState::HealthTail(state) = production.state() else {
        return Err(data_failure(None, "Health tail carried owners"));
    };
    Ok(state)
}
