//! External phase, live authoring controls and after-flight encounter opportunity.

use super::{data_failure, word};
use crate::replay::{played::platform::finished, readers::CutTape, result::Outcome, weapons};
use data::rom::{Lorom, RomAddress};
use game::production::{
    AdvanceInput, Production, ProductionOutputs,
    intro_missile_scope::{Input, Presentation, Turn},
    played::intro_missiles::Content,
    weapons::QuietBeamWorld,
};
use oracle_record::FrameRecord;
use weapons_projectiles as w1;

const DRAW_GATE: usize = 0x1a57;
const BEAT: usize = 0x1a51;
const DEMO_FIRST: u16 = 0x8694;
const DEMO_ENTRIES: u16 = 17;

pub(super) struct Demo(Vec<[u16; 2]>);

impl Demo {
    pub(super) fn load(rom: Lorom<'_>) -> Result<Self, Outcome> {
        let mut controls = Vec::new();
        for entry in 0..DEMO_ENTRIES {
            let address = RomAddress::new(0x91, DEMO_FIRST.wrapping_add(entry.wrapping_mul(6)));
            let bytes = rom
                .read(address, 6)
                .map_err(|_| data_failure(None, "demo controls ROM"))?;
            let [duration, buttons, presses] = bytes.as_chunks::<2>().0 else {
                return Err(data_failure(None, "demo control entry"));
            };
            let duration = usize::from(u16::from_le_bytes(*duration));
            if !(1..=256).contains(&duration) {
                return Err(data_failure(None, "demo control hold"));
            }
            let held = u16::from_le_bytes(*buttons);
            controls.push([held, u16::from_le_bytes(*presses)]);
            controls.extend(std::iter::repeat_n([held, 0], duration.saturating_sub(1)));
        }
        Ok(Self(controls))
    }

    fn at(&self, beat: usize) -> [u16; 2] {
        self.0.get(beat).copied().unwrap_or([0, 0])
    }
}

pub(super) fn hidden(frame: &FrameRecord) -> Result<bool, Outcome> {
    Ok(word(frame, DRAW_GATE)? == 0)
}

pub(super) fn starts_demo(frame: &FrameRecord) -> Result<bool, Outcome> {
    Ok(word(frame, DRAW_GATE)? != 0
        && word(frame, 0x0a88)? == 0x8000
        && word(frame, 0x0a7e)? == DEMO_FIRST
        && word(frame, 0x0a7c)? == 1)
}

fn opportunity(
    prior: Option<&FrameRecord>,
    frame: &FrameRecord,
) -> Result<(bool, Presentation), Outcome> {
    let Some(prior) = prior else {
        return Ok((false, Presentation::Hold));
    };
    let turn = word(prior, DRAW_GATE)? != 0 && word(prior, BEAT)? != word(frame, BEAT)?;
    let complete = finished(frame);
    let publish = complete && (turn || !finished(prior));
    let drawing = match (turn, complete, word(frame, DRAW_GATE)? != 0) {
        (true, true, true) => Presentation::DrawAndPublish(view(frame)?),
        (true, false, true) => Presentation::Prepare(view(frame)?),
        (true, true, false) => Presentation::Hidden,
        (false, true, _) if publish => Presentation::Publish,
        _ => Presentation::Hold,
    };
    Ok((turn, drawing))
}

#[derive(Clone)]
struct Work {
    update: w1::UpdateInput,
    eligible: bool,
    endures: bool,
}

struct Cut {
    work: Option<Work>,
    drawing: Presentation,
}

pub(in crate::replay) struct Inputs {
    demo: Demo,
    elapsed: usize,
    grid: QuietBeamWorld,
    tape: CutTape<Cut>,
}

impl Inputs {
    pub(super) const fn new(demo: Demo, grid: QuietBeamWorld, first: u64) -> Self {
        Self {
            demo,
            elapsed: 0,
            grid,
            tape: CutTape {
                first,
                values: Vec::new(),
            },
        }
    }

    pub(super) fn read(
        &mut self,
        prior: Option<&FrameRecord>,
        frame: &FrameRecord,
        content: &Content,
    ) -> Result<(), Outcome> {
        let (turn, drawing) = opportunity(prior, frame)?;
        let work = if turn {
            let prior =
                prior.ok_or_else(|| data_failure(Some(frame.index), "weapon work origin"))?;
            let [held, fresh] = self.demo.at(self.elapsed);
            self.elapsed = self.elapsed.saturating_add(1);
            if [held, fresh] != [word(frame, 0x0a84)?, word(frame, 0x0a86)?] {
                return Err(data_failure(Some(frame.index), "authored missile controls"));
            }
            let (update, eligible) = cannon(prior, held, fresh, content)?;
            Some(Work {
                update,
                eligible,
                endures: word(prior, 0x1b3d)? == 0xb786,
            })
        } else {
            None
        };
        self.tape.values.push(Cut { work, drawing });
        Ok(())
    }

    pub(in crate::replay) fn advance(
        &self,
        production: &mut Production,
        cut: u64,
    ) -> Result<ProductionOutputs, Outcome> {
        let input = self.tape.at(cut)?;
        let contact = |candidates: &[w1::MissileCandidate]| {
            let work = input.work.as_ref()?;
            // The boss phase is external to W1. Its contact rule reads the first ordered
            // after-flight candidate, so this callback has no access to the observed impact.
            candidates
                .first()
                .filter(|shot| work.endures && shot.whole_x.cast_signed() < 84)
                .map(|shot| shot.slot)
        };
        let turn = input.work.as_ref().map(|work| Turn {
            update: &work.update,
            eligible: work.eligible,
            world: &self.grid,
            contact: &contact,
        });
        production
            .advance(AdvanceInput::IntroMissiles(Input {
                turn,
                presentation: input.drawing,
            }))
            .map_err(|fault| crate::replay::production_failure(cut, fault))
    }
}

fn view(record: &FrameRecord) -> Result<w1::ScreenOrigin, Outcome> {
    Ok(w1::ScreenOrigin(w1::Axes {
        x: word(record, 0x0911)?,
        y: word(record, 0x0915)?,
    }))
}

fn cannon(
    prior: &FrameRecord,
    held: u16,
    fresh: u16,
    content: &Content,
) -> Result<(w1::UpdateInput, bool), Outcome> {
    let at = |address| word(prior, address);
    let mode = match at(0x0a42)? {
        0xe6c9 => w1::UpdateMode::CannonAndShots,
        0xe713 => w1::UpdateMode::ShotsOnly,
        _ => return Err(data_failure(Some(prior.index), "cinematic cannon mode")),
    };
    let lookup = |raw| {
        let pose = data::assets::samus::map::MAP
            .iter()
            .find(|entry| entry.id.0 == raw)
            .ok_or_else(|| data_failure(Some(prior.index), "cinematic cannon pose"))?;
        content
            .muzzle_for(pose.key)
            .map_err(|_| data_failure(Some(prior.index), "cinematic muzzle content"))
    };
    let binding = at(0x09b2)?;
    let pending = at(0x0b5e)?;
    let [movement, _] = at(0x0a1f)?.to_le_bytes();
    let distance = |address: usize| -> Result<w1::MovementDistance, Outcome> {
        Ok(w1::MovementDistance {
            whole: at(address)?,
            fraction: w1::PixelFraction(at(address.saturating_add(2))?),
        })
    };
    let update = w1::UpdateInput {
        mode,
        time_stopped: at(0x0a78)? != 0,
        cannon: w1::CannonInput {
            shoot: held & binding != 0,
            new_press: fresh & binding != 0,
            previous_new_press: at(0x0e00)? & binding != 0,
            muzzle: lookup(at(0x0a1c)?)?,
            previous_muzzle: lookup(at(0x0a24)?)?.aim,
            moving_muzzle: movement == 1,
            pending_aim: if pending == 0 {
                None
            } else {
                Some(weapons::heading(pending & 255)?)
            },
            centre: w1::SamusCentre(w1::Axes {
                x: at(0x0af6)?,
                y: at(0x0afa)?,
            }),
        },
        inheritance: w1::Inheritance {
            left: distance(0x0daa)?,
            right: distance(0x0dae)?,
            up: distance(0x0db2)?,
            down: distance(0x0db6)?,
            camera_vertical_fraction: w1::PixelFraction(at(0x0da8)?),
        },
        camera: view(prior)?,
        room: w1::RoomScreens(w1::Axes {
            x: at(0x07a9)?,
            y: at(0x07ab)?,
        }),
    };
    Ok((update, movement == 0))
}
