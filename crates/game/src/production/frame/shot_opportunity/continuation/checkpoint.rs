//! Save only the carried owners. Restore uses the same independently admitted room and content.

use super::{Audio, Continuation, Origin, missing};
use crate::weapons_projectiles_bridge::PowerContent;
use crate::{
    production::{
        Checkpoint, ProductionFault,
        frame::{inputs::MovementInputsPin, random::RandomSeed},
    },
    state::{
        owned::{CameraPin, MovementPin, PowerPin},
        video::{NmiBatch, OAM_SPRITES},
    },
};
use boss_ceres_ridley::{Phase, Quarter, Wait};
use canonical::{Canonical, DecodeError, Schema, model::Model};
use data::{
    assets::samus::{key::Dress, map},
    pose::PoseId,
};
use weapons_projectiles as beam;

#[derive(Canonical, Clone, Debug, PartialEq, Eq)]
struct Carried {
    player: Model<MovementPin>,
    prior_pose: u16,
    camera: Model<CameraPin>,
    projectiles: Model<PowerPin>,
    doors: [DoorState; 2],
    random: RandomSeed,
    pad: [u16; 2],
    inheritance: [[u16; 2]; 4],
    pending_aim: Option<Heading>,
    stimuli: Model<MovementInputsPin>,
    batch: Box<NmiBatch>,
}

#[derive(Canonical, Clone, Debug, PartialEq, Eq)]
struct WithAudio {
    shot: Carried,
    audio: Audio,
}

impl Continuation {
    /// Encode the complete carried state at the boundary between ordinary opportunities.
    ///
    /// # Errors
    /// Rejects a prior pose that has no exact identity in this declared ordinary scope.
    pub fn save(&self, dress: Dress) -> Result<Vec<u8>, ProductionFault> {
        let prior_pose = map::id_of(self.origin.prior_pose, dress)
            .filter(|&id| map::key_of(id) == Some(self.origin.prior_pose))
            .ok_or_else(missing)?;
        let inheritance = self.origin.weapon.inheritance;
        let state = Carried {
            player: Model(self.player),
            prior_pose: prior_pose.0,
            camera: Model(self.camera),
            projectiles: Model(Box::new(self.origin.shots.clone())),
            doors: self.origin.doors.map(|door| DoorState::from(door.phase)),
            random: self.origin.random,
            pad: self.origin.previous_pad,
            inheritance: [
                inheritance.left,
                inheritance.right,
                inheritance.up,
                inheritance.down,
            ]
            .map(|distance| [distance.whole, distance.fraction.0]),
            pending_aim: self.origin.weapon.cannon.pending_aim.map(Heading::from),
            stimuli: Model(self.stimuli),
            batch: Box::new(self.batch.clone()),
        };
        Ok(match &self.audio {
            Some(audio) => canonical::to_bytes(&Checkpoint {
                schema: Schema::new(),
                state: WithAudio {
                    shot: state,
                    audio: audio.clone(),
                },
            }),
            None => canonical::to_bytes(&Checkpoint {
                schema: Schema::new(),
                state,
            }),
        })
    }

    /// Restore directly using the immutable neighbours from the original admission.
    ///
    /// # Errors
    /// Rejects an invalid encoded continuation or a missing current owner relation.
    pub fn restore(
        bytes: &[u8],
        mut admitted: Origin,
        dress: Dress,
        power: &PowerContent,
    ) -> Result<Self, ProductionFault> {
        let (state, audio) = match canonical::from_bytes::<Checkpoint<Carried>>(bytes) {
            Ok(checkpoint) => (checkpoint.state, None),
            Err(DecodeError::SchemaMismatch { .. }) => {
                let checkpoint: Checkpoint<WithAudio> =
                    canonical::from_bytes(bytes).map_err(|_| missing())?;
                let WithAudio { shot, audio } = checkpoint.state;
                audio.sounds().validate().map_err(|_| missing())?;
                (shot, Some(audio))
            }
            Err(_) => return Err(missing()),
        };
        let player = state.player.0;
        let camera = state.camera.0;
        let batch = *state.batch;
        if usize::from(batch.sprites.0) > OAM_SPRITES {
            return Err(missing());
        }
        admitted.prior_pose = map::key_of(PoseId(state.prior_pose)).ok_or_else(missing)?;
        let muzzles = super::super::muzzles(player.pose, admitted.prior_pose, dress, power)?;
        super::super::refresh(&mut admitted, &player, &camera, muzzles);
        admitted.shots = *state.projectiles.0;
        admitted.random = state.random;
        admitted.previous_pad = state.pad;
        for (door, phase) in admitted.doors.iter_mut().zip(state.doors) {
            door.phase = phase.restore().ok_or_else(missing)?;
        }
        let [left, right, up, down] =
            state
                .inheritance
                .map(|[whole, fraction]| beam::MovementDistance {
                    whole,
                    fraction: beam::PixelFraction(fraction),
                });
        admitted.weapon.inheritance = beam::Inheritance {
            left,
            right,
            up,
            down,
            camera_vertical_fraction: beam::PixelFraction(camera.vertical_fraction().0),
        };
        admitted.weapon.cannon.pending_aim = state.pending_aim.map(beam::Heading::from);
        let mut complete = Self::new(admitted, player, camera, state.stimuli.0, batch);
        complete.audio = audio;
        Ok(complete)
    }
}

#[derive(Canonical, Clone, Copy, Debug, PartialEq, Eq)]
enum DoorState {
    Closed(u16),
    Closing(QuarterState, u16),
    Open(u16),
    Opening(QuarterState, u16),
    Placed,
    Starting(u16),
    Wall,
}

impl From<Phase> for DoorState {
    fn from(phase: Phase) -> Self {
        match phase {
            Phase::Closed { wait } => Self::Closed(wait.frames()),
            Phase::Closing { quarter, wait } => Self::Closing(quarter.into(), wait.frames()),
            Phase::Open { wait } => Self::Open(wait.frames()),
            Phase::Opening { quarter, wait } => Self::Opening(quarter.into(), wait.frames()),
            Phase::Placed => Self::Placed,
            Phase::Starting { wait } => Self::Starting(wait.frames()),
            Phase::Wall => Self::Wall,
        }
    }
}

impl DoorState {
    fn restore(self) -> Option<Phase> {
        Some(match self {
            Self::Closed(wait) => Phase::Closed {
                wait: Wait::new(wait)?,
            },
            Self::Closing(quarter, wait) => Phase::Closing {
                quarter: quarter.into(),
                wait: Wait::new(wait)?,
            },
            Self::Open(wait) => Phase::Open {
                wait: Wait::new(wait)?,
            },
            Self::Opening(quarter, wait) => Phase::Opening {
                quarter: quarter.into(),
                wait: Wait::new(wait)?,
            },
            Self::Placed => Phase::Placed,
            Self::Starting(wait) => Phase::Starting {
                wait: Wait::new(wait)?,
            },
            Self::Wall => Phase::Wall,
        })
    }
}

macro_rules! saved_enum {
    ($name:ident, $model:path, [$($variant:ident),+]) => {
        #[derive(Canonical, Clone, Copy, Debug, PartialEq, Eq)]
        enum $name { $($variant),+ }
        impl From<$model> for $name {
            fn from(value: $model) -> Self {
                use $model as ModelValue;
                match value { $(ModelValue::$variant => Self::$variant),+ }
            }
        }
        impl From<$name> for $model {
            fn from(value: $name) -> Self {
                match value { $($name::$variant => Self::$variant),+ }
            }
        }
    };
}

saved_enum!(QuarterState, Quarter, [First, Second, Third, Fourth]);
saved_enum!(
    Heading,
    beam::Heading,
    [
        DownLeft,
        DownRight,
        DownFacingLeft,
        DownFacingRight,
        Left,
        Right,
        UpLeft,
        UpRight,
        UpFacingLeft,
        UpFacingRight
    ]
);
