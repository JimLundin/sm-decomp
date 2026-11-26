//! The encounter's frame: the intro's requests, then on a finished frame the egg, the
//! fragments, the baby, the droplets and the pulse, in that order, each reading the results
//! before it.

use types::Pair;
use types::discovery::{BabyMotion, DiscoveryOrder, DiscoveryState, Egg, Hatchling, Pulse};

use crate::baby::{self, BabyCall};
use crate::drawing::{self, Drawing};
use crate::egg::{self, EggTurn};
use crate::rules::Rules;
use crate::scene::{DiscoveryScene, SoundOffer};
use crate::{pulse, shell, slime};

/// What the intro asks of the encounter this frame.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum IntroRequest {
    /// Nothing.
    None,
    /// `StartDiscovery` places the egg and the baby.
    StartDiscovery,
    /// `FinishDiscovery`: the return dissolve's remaining count reached zero.
    FinishDiscovery,
}

/// Whether the encounter advances a finished frame this cut (T1 says).
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Cut {
    /// A finished frame: the encounter advances.
    Finished,
    /// An unfinished cut: the snapshot is preserved.
    Unfinished,
}

/// The cue that page three may be prepared.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct ReadyForPageThree;

/// What a frame of the encounter reports to audio and the intro.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Report {
    /// The sounds offered to audio, in offer order.
    pub sounds: Vec<SoundOffer>,
    /// `ReadyForPageThree`, on the cut the hatch ends.
    pub cue: Option<ReadyForPageThree>,
}

/// The encounter of the egg and the baby: the caller's content and the encounter's fixed
/// numbers.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Encounter {
    /// The content.
    pub scene: DiscoveryScene,
    /// The numbers. Every entry of the encounter is `Fixed`, so they are always the reference
    /// game's and no caller can replace them.
    rules: Rules,
}

impl Encounter {
    /// The encounter of `scene`, run by the reference game's numbers.
    #[must_use]
    pub const fn new(scene: DiscoveryScene) -> Self {
        Self {
            scene,
            rules: Rules::REFERENCE,
        }
    }

    /// One frame of the encounter, given Samus's whole-pixel centre after her room turn and the
    /// intro's request.
    pub fn step(
        &self,
        state: &mut DiscoveryState,
        samus: Pair<i32>,
        request: IntroRequest,
        cut: Cut,
    ) -> Report {
        let mut report = Report {
            sounds: Vec::new(),
            cue: None,
        };
        match request {
            IntroRequest::FinishDiscovery => {
                hide(state);
                return report;
            }
            IntroRequest::StartDiscovery => self.place(state),
            IntroRequest::None => {}
        }
        if cut == Cut::Finished {
            self.finished_frame(state, samus, &mut report);
        }
        report
    }

    /// The encounter's pictures for E1, front first.
    #[must_use]
    pub fn drawings(&self, state: &DiscoveryState) -> Vec<Drawing> {
        drawing::drawings(state, &self.scene)
    }

    /// Spec: `EggHatches`. `StartDiscovery` on an absent egg places the egg and the baby at home,
    /// waiting and inside, clears the encounter's motion and puts the encounter ahead of Samus.
    /// Pictures begin on the first finished frame after placement.
    fn place(&self, state: &mut DiscoveryState) {
        if state.egg == Egg::Absent {
            *state = DiscoveryState {
                egg: Egg::Waiting,
                baby: Some(Hatchling {
                    place: self.scene.home,
                    motion: BabyMotion::Inside,
                    pulse: Pulse::Unseen,
                }),
                order: DiscoveryOrder::SceneFirst,
                ..DiscoveryState::dormant()
            };
        }
    }

    /// One finished frame of every part, in order.
    fn finished_frame(&self, state: &mut DiscoveryState, samus: Pair<i32>, report: &mut Report) {
        let (scene, rules) = (&self.scene, &self.rules);
        let turn = egg::advance(&mut state.egg, &scene.shell, rules, samus.x);
        let shown = match turn {
            EggTurn::Hatches(age) => Some(age),
            EggTurn::Unchanged | EggTurn::Opens => None,
        };
        if shown == Some(scene.shell.scatter_at) {
            state.fragments = shell::scatter(&scene.fragments);
            report.sounds.push(scene.hatch_sound);
        }
        shell::fly(&mut state.fragments, &scene.fragments, rules.floor_y);
        if let Some(hatchling) = &mut state.baby {
            let released = shown == Some(scene.shell.release_at);
            match baby::turn(hatchling, &mut state.order, released, samus, rules) {
                BabyCall::Silent => {}
                BabyCall::Cries => report.sounds.push(scene.cry_sound),
                BabyCall::Sheds(anchor) => {
                    state.droplets = slime::shed(&scene.droplets, anchor);
                    report.sounds.push(scene.cry_sound);
                }
            }
            hatchling.pulse = pulse::advanced(hatchling.pulse, &scene.baby);
        }
        slime::fall(
            &mut state.droplets,
            &scene.droplets,
            &scene.splash,
            rules.floor_y,
        );
        if turn == EggTurn::Opens {
            report.cue = Some(ReadyForPageThree);
        }
    }
}

/// Spec: `PageThreeApproaches`. `FinishDiscovery` removes the encounter's drawings and hides
/// the encounter, Samus and her projectiles; nothing of the encounter advances on that cut.
fn hide(state: &mut DiscoveryState) {
    *state = DiscoveryState::dormant();
}
