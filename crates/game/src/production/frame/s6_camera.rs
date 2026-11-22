//! Actual empty pre-camera neighbours and the complete following-camera operation.

use super::{Caller, Content, State, camera_stage, drawing_owners};
use crate::production::{ProductionFault, Unsupported, timing};
use canonical::Canonical;
use data::timing::{Profile, Template, TemplateEffect};

pub(super) const BEFORE_CAMERA: timing::WorkId = timing::WorkId(1320);
pub(super) const CAMERA: timing::WorkId = timing::WorkId(1321);

/// Current complete dependencies for the bounded empty neighbours and camera operation.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Contract {
    prefix: Profile,
    inputs: camera_stage::Inputs,
    camera: camera_stage::PhysicalContract,
    adjacent: Option<camera_stage::PhysicalContract>,
}

impl Contract {
    /// Bind the whole pre-camera envelope with typed current camera content.
    /// # Errors
    /// The selected empty neighbours cannot produce any hardware effects.
    pub fn new(
        prefix: &Template,
        inputs: camera_stage::Inputs,
        camera: camera_stage::PhysicalContract,
    ) -> Result<Self, ProductionFault> {
        if prefix
            .runs
            .iter()
            .any(|run| run.effect != TemplateEffect::None)
        {
            return Err(missing());
        }
        Ok(Self {
            prefix: prefix.bind(&[]).map_err(|_| missing())?,
            inputs,
            camera,
            adjacent: None,
        })
    }

    /// Qualify the same physical follow against independently checked at-use operands.
    pub fn admit_adjacent(
        &mut self,
        current: camera_stage::RequiredCentre,
        previous: camera_stage::RequiredCentre,
        view: camera_stage::RequiredView,
        screens: Vec<camera_stage::RequiredScreens>,
    ) {
        let dependencies = self.dependencies(current, previous, view, screens);
        self.adjacent = Some(self.camera.with_dependencies(dependencies));
    }

    /// Qualify another current following opportunity against the same physical rule.
    pub fn admit_recurring(
        &mut self,
        current: camera_stage::RequiredCentre,
        previous: camera_stage::RequiredCentre,
        view: camera_stage::RequiredView,
        screens: Vec<camera_stage::RequiredScreens>,
    ) {
        let dependencies = self.dependencies(current, previous, view, screens);
        if let Some(adjacent) = &mut self.adjacent {
            adjacent.admit(dependencies, camera_stage::Horizontal::Exact);
        } else {
            self.adjacent = Some(self.camera.with_dependencies(dependencies));
        }
    }

    /// Admit varying horizontal centres and viewport fraction in the same running room follow.
    pub fn admit_live_horizontal(
        &mut self,
        current: camera_stage::RequiredCentre,
        previous: camera_stage::RequiredCentre,
        view: camera_stage::RequiredView,
        screens: Vec<camera_stage::RequiredScreens>,
    ) {
        let dependencies = self.dependencies(current, previous, view, screens);
        let adjacent = self.adjacent.get_or_insert_with(|| self.camera.clone());
        adjacent.admit(dependencies, camera_stage::Horizontal::NearRoomEdge);
    }

    /// Admit the crossed backward aim and both rightward edge envelopes.
    #[must_use]
    pub fn admit_edge_profiles(&mut self, profiles: [Template; 3]) -> Option<()> {
        self.adjacent.as_mut()?.admit_edge_profiles(profiles)
    }

    fn dependencies(
        &self,
        current: camera_stage::RequiredCentre,
        previous: camera_stage::RequiredCentre,
        view: camera_stage::RequiredView,
        screens: Vec<camera_stage::RequiredScreens>,
    ) -> camera_stage::RequiredDependencies {
        camera_stage::RequiredDependencies {
            current,
            previous,
            view,
            inputs: Box::new(self.inputs.clone()),
            screens,
        }
    }

    fn physical(&self, state: &State) -> Result<&camera_stage::PhysicalContract, ProductionFault> {
        if state.publications.original_interval() {
            Ok(&self.camera)
        } else {
            self.adjacent.as_ref().ok_or_else(missing)
        }
    }
}

#[derive(Canonical, Clone, Debug, Default, PartialEq, Eq)]
enum Progress {
    #[default]
    Pending,
    PreCamera,
    CameraReady,
    Following(Box<camera_stage::Stage>),
    Complete(Box<camera_stage::Stage>),
}

/// Canonically retained pre-drawing horizon.
#[derive(Canonical, Clone, Debug, Default, PartialEq, Eq)]
pub struct Stage(Progress);

impl Stage {
    /// Only the actually returned viewport may be consumed by Drawing.
    #[must_use]
    pub fn following(&self) -> Option<&camera_stage::Stage> {
        let Progress::Complete(camera) = &self.0 else {
            return None;
        };
        Some(camera)
    }
}

const fn missing() -> ProductionFault {
    ProductionFault::Unsupported(Unsupported::Frame(super::Dependency::CameraPreparation))
}

impl State {
    fn qualified_camera_owners(
        &self,
        contract: &Contract,
    ) -> Result<camera_stage::Previous, ProductionFault> {
        let Caller::PlayerContinuation {
            motion_reset: Some(reset),
            remaining,
            ..
        } = &self.caller
        else {
            return Err(missing());
        };
        let origins = self.drawing_origins.as_ref().ok_or_else(missing)?;
        let actors = remaining.0.actors.as_ref().ok_or_else(missing)?;
        let empty = drawing_owners::Membership::Absent;
        let collections = &origins.collections;
        if self.maintenance.energy().is_none()
            || !origins.collections.enemy_projectiles_enabled
            || !origins.collections.plm_enabled
            || !origins.collections.animated_tiles_enabled
            || origins.projectile_population != 0
            || origins.actors.sprite_interactions_disabled
            || origins.power_bomb_radius != 0
            || origins.invincibility.0 != actors.dependency.contact.samus_invincibility
            || origins.invincibility.0 != 0
            || actors.dependency.contact.damage_mode
                != super::borrowed_player_preparation::ContactDamageDisposition::Ordinary
            || collections
                .enemy_projectiles
                .iter()
                .chain(&collections.plm)
                .chain(&collections.animated_tiles)
                .any(|member| *member != empty)
            || contract.inputs.knockback != camera_stage::Activity::Inactive
            || contract.inputs.time_stopped != origins.frozen
            || !matches!(
                self.interruption,
                super::interruption_caller::Stage::Complete(_)
            )
            || self.movement.finished().is_none()
        {
            return Err(ProductionFault::Unsupported(Unsupported::Frame(
                super::Dependency::CameraNeighbourOwners,
            )));
        }
        Ok(camera_stage::Previous(reset.previous))
    }

    pub(super) fn validate_completed_camera(&self, content: &Content) -> Result<(), super::Fault> {
        let Some(camera) = self.s6_camera.following() else {
            return Ok(());
        };
        let invalid = || super::Fault::InvalidPlayerEntry;
        let camera_stage::Stage::Completed(receipt) = camera else {
            return Err(invalid());
        };
        let contract = content.s6_camera.as_ref().ok_or_else(invalid)?;
        let previous = self
            .qualified_camera_owners(contract)
            .map_err(|_| invalid())?;
        if receipt.previous != previous
            || receipt.movement != self.movement
            || receipt.inputs != contract.inputs
            || receipt.follow().map_err(|_| invalid())? != self.camera
        {
            return Err(invalid());
        }
        Ok(())
    }

    pub(super) fn advance_before_drawing(
        &mut self,
        content: &Content,
    ) -> Result<(), ProductionFault> {
        let contract = content.s6_camera.as_ref().ok_or_else(missing)?;
        let previous = self.qualified_camera_owners(contract)?;
        match &self.s6_camera.0 {
            Progress::Pending => {
                self.timing
                    .begin(BEFORE_CAMERA, contract.prefix.clone())
                    .map_err(ProductionFault::FrameHardware)?;
                self.s6_camera.0 = Progress::PreCamera;
            }
            Progress::PreCamera => self.validate_camera_profile(BEFORE_CAMERA, &contract.prefix)?,
            Progress::CameraReady => {
                let mut progress = camera_stage::Stage::Idle;
                progress
                    .choose_from_current_history(
                        &self.camera,
                        &previous,
                        &self.movement,
                        &contract.inputs,
                    )
                    .map_err(|fault| {
                        let dependency = match fault {
                            camera_stage::Fault::Previous => {
                                super::Dependency::CameraPreviousCentre
                            }
                            camera_stage::Fault::Unsupported => {
                                super::Dependency::CameraNeighbourOwners
                            }
                            camera_stage::Fault::Room(_) => super::Dependency::CameraRoom,
                            camera_stage::Fault::Order
                            | camera_stage::Fault::Movement
                            | camera_stage::Fault::Completion
                            | camera_stage::Fault::Profile
                            | camera_stage::Fault::Dependencies
                            | camera_stage::Fault::CurrentCentre
                            | camera_stage::Fault::PreviousCentre
                            | camera_stage::Fault::Samples => super::Dependency::CameraPhysical,
                        };
                        ProductionFault::Unsupported(Unsupported::Frame(dependency))
                    })?;
                let profile = progress
                    .physical(contract.physical(self)?)
                    .map_err(|fault| {
                        let dependency = match fault {
                            camera_stage::Fault::CurrentCentre => {
                                super::Dependency::CameraCurrentCentre
                            }
                            camera_stage::Fault::PreviousCentre => {
                                super::Dependency::CameraPreviousCentre
                            }
                            camera_stage::Fault::Order
                            | camera_stage::Fault::Movement
                            | camera_stage::Fault::Previous
                            | camera_stage::Fault::Room(_)
                            | camera_stage::Fault::Completion
                            | camera_stage::Fault::Profile
                            | camera_stage::Fault::Dependencies
                            | camera_stage::Fault::Samples
                            | camera_stage::Fault::Unsupported => super::Dependency::CameraPhysical,
                        };
                        ProductionFault::Unsupported(Unsupported::Frame(dependency))
                    })?;
                progress.bind(CAMERA).map_err(|_| {
                    ProductionFault::Unsupported(Unsupported::Frame(
                        super::Dependency::CameraPhysical,
                    ))
                })?;
                self.timing
                    .begin(CAMERA, profile)
                    .map_err(ProductionFault::FrameHardware)?;
                self.s6_camera.0 = Progress::Following(Box::new(progress));
            }
            Progress::Following(camera) => {
                let camera_stage::Stage::Charging { receipt, .. } = camera.as_ref() else {
                    return Err(missing());
                };
                if receipt.initial != self.camera
                    || receipt.previous != previous
                    || receipt.movement != self.movement
                    || receipt.inputs != contract.inputs
                {
                    return Err(ProductionFault::Unsupported(Unsupported::Frame(
                        super::Dependency::CameraHeldInput,
                    )));
                }
                let profile = camera.physical(contract.physical(self)?).map_err(|_| {
                    ProductionFault::Unsupported(Unsupported::Frame(
                        super::Dependency::CameraPhysical,
                    ))
                })?;
                self.validate_camera_profile(CAMERA, &profile)?;
            }
            Progress::Complete(_) => return Err(missing()),
        }
        Ok(())
    }

    fn validate_camera_profile(
        &self,
        identity: timing::WorkId,
        profile: &Profile,
    ) -> Result<(), ProductionFault> {
        let mut active = self
            .timing
            .work
            .iter()
            .filter(|work| work.kind == timing::WorkKind::Semantic(identity));
        if !active.next().is_some_and(|work| &work.profile == profile) || active.next().is_some() {
            return Err(ProductionFault::Unsupported(Unsupported::Frame(
                super::Dependency::CameraHeldProfile,
            )));
        }
        Ok(())
    }

    pub(super) fn sample_current_camera(
        &mut self,
        events: &[timing::Event],
    ) -> Result<(), ProductionFault> {
        let Progress::Following(camera) = &mut self.s6_camera.0 else {
            return Err(missing());
        };
        for event in events {
            if let timing::EventKind::Sample(
                sample @ (data::timing::Sample::ProductLow | data::timing::Sample::ProductHigh),
                timing::SampleValue::Byte(value),
            ) = &event.kind
            {
                camera.sample(CAMERA, *sample, *value).map_err(|_| {
                    ProductionFault::Unsupported(Unsupported::Frame(
                        super::Dependency::CameraLiveSample,
                    ))
                })?;
            }
        }
        Ok(())
    }

    pub(super) fn complete_camera_work(
        &mut self,
        work: timing::WorkId,
        content: &Content,
    ) -> Result<(), ProductionFault> {
        let contract = content.s6_camera.as_ref().ok_or_else(missing)?;
        self.qualified_camera_owners(contract)?;
        match (&mut self.s6_camera.0, work) {
            (Progress::PreCamera, BEFORE_CAMERA) => self.s6_camera.0 = Progress::CameraReady,
            (Progress::Following(camera), CAMERA) => {
                camera.complete(&mut self.camera, work).map_err(|fault| {
                    let dependency = if fault == camera_stage::Fault::Samples {
                        super::Dependency::CameraLiveSample
                    } else {
                        super::Dependency::CameraPhysical
                    };
                    ProductionFault::Unsupported(Unsupported::Frame(dependency))
                })?;
                self.s6_camera.0 = Progress::Complete(camera.clone());
            }
            _ => return Err(missing()),
        }
        Ok(())
    }
}
