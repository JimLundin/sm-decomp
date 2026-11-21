//! Actual early HUD controls and cannon readiness share one completed control answer.

use super::{Caller, Content, Dependency, PlayerEntryStage, State};
use crate::production::{ProductionFault, Unsupported, timing};
use canonical::model::Model;
use timing::WorkId;

impl State {
    fn early_hud_frame(&self) -> Option<interface_hud::FrameCount> {
        if self.publications.original_interval() {
            Some(self.hud_display_current.as_ref()?.0.premises.frame)
        } else {
            self.hud_frame_count.map(interface_hud::FrameCount)
        }
    }

    fn early_hud_cannon_ready(&self) -> bool {
        self.publications.original_interval()
            || self
                .appearance
                .cannon
                .as_ref()
                .is_some_and(|cannon| cannon.0.may_follow())
    }

    /// Qualify retained readiness before any later consumer can advance hardware.
    pub(super) fn validate_cannon_history(&self) -> Result<(), ProductionFault> {
        let fault =
            ProductionFault::Unsupported(Unsupported::Frame(Dependency::HudWeaponsPreparation));
        self.appearance.validate().map_err(|_| fault)?;
        if let Caller::PlayerContinuation { controls, .. } = &self.caller {
            self.appearance
                .validate_cannon_completion(interface_hud::Selection::None, &controls.answer.0)
                .map_err(|_| fault)?;
        }
        Ok(())
    }

    pub(super) fn begin_early_hud(&mut self, content: &Content) -> Result<(), ProductionFault> {
        let Caller::Player {
            prepared,
            stage: PlayerEntryStage::HudWeapons,
            charging: false,
            completed,
            completion_work,
        } = &self.caller
        else {
            return Err(ProductionFault::WrongScope);
        };
        let fault =
            ProductionFault::Unsupported(Unsupported::Frame(Dependency::HudWeaponsPreparation));
        if self
            .inside_completed
            .as_ref()
            .is_none_or(|receipt| receipt.work != PlayerEntryStage::Inside.identity())
            || !self.early_hud_cannon_ready()
        {
            return Err(fault);
        }
        let current = super::hud_controls_inputs::CurrentControls {
            gameplay: prepared.gameplay.as_ref().ok_or(fault)?,
            scene: self.scene.ok_or(fault)?,
            pose_input: completed.0.pose_input.as_ref().ok_or(fault)?,
            pose: self.movement.state().pose,
            inventory: self.hud_inventory.as_ref().ok_or(fault)?,
        };
        let mut hud = self.hud.clone();
        if content.late_hud.is_some() {
            let frame = self.early_hud_frame().ok_or(fault)?;
            if !hud.begin_staged(frame) {
                return Err(fault);
            }
        }
        let profile = super::hud_controls_owner::choose(
            &mut hud,
            content.early_hud_controls.as_ref().ok_or(fault)?,
            current,
            content.early_hud_owner.as_ref().ok_or(fault)?,
        )
        .map_err(|_| fault)?;
        let Some(interface_hud::Observation::Chosen(answer)) = hud.controls() else {
            return Err(fault);
        };
        let mut appearance = self.appearance.clone();
        appearance
            .choose_cannon(self.hud.state().selection, answer)
            .map_err(|_| fault)?;
        self.timing
            .begin(PlayerEntryStage::HudWeapons.identity(), profile)
            .map_err(ProductionFault::FrameHardware)?;
        self.hud = hud;
        self.appearance = appearance;
        self.caller = Caller::Player {
            prepared: prepared.clone(),
            stage: PlayerEntryStage::HudWeapons,
            charging: true,
            completed: completed.clone(),
            completion_work: completion_work.clone(),
        };
        Ok(())
    }

    pub(super) fn validate_early_hud_current(
        &self,
        content: &Content,
    ) -> Result<(), ProductionFault> {
        let Caller::Player {
            prepared,
            stage: PlayerEntryStage::HudWeapons,
            charging: true,
            completed,
            ..
        } = &self.caller
        else {
            return Err(ProductionFault::WrongScope);
        };
        let fault =
            ProductionFault::Unsupported(Unsupported::Frame(Dependency::HudWeaponsPreparation));
        if !self.early_hud_cannon_ready() {
            return Err(fault);
        }
        if content.late_hud.is_some() {
            let frame = self.early_hud_frame().ok_or(fault)?;
            if !matches!(&self.hud, super::hud::Hud::Work(work) if work.0.frame_count() == Some(frame))
            {
                return Err(fault);
            }
        }
        let inputs = content
            .early_hud_controls
            .as_ref()
            .ok_or(fault)?
            .inputs(super::hud_controls_inputs::CurrentControls {
                gameplay: prepared.gameplay.as_ref().ok_or(fault)?,
                scene: self.scene.ok_or(fault)?,
                pose_input: completed.0.pose_input.as_ref().ok_or(fault)?,
                pose: self.movement.state().pose,
                inventory: self.hud_inventory.as_ref().ok_or(fault)?,
            })
            .map_err(|_| fault)?;
        let Some(interface_hud::Observation::Chosen(answer)) = self.hud.controls() else {
            return Err(fault);
        };
        self.appearance
            .validate_cannon_choice(self.hud.state().selection, answer)
            .map_err(|_| fault)?;
        if answer.inputs != inputs
            || inputs.select != interface_hud::SelectButton::NotPressed
            || inputs.cancel != interface_hud::CancelButton::NotPressed
            || !inputs.can_switch
            || answer.selection != interface_hud::Selection::None
            || answer.switched.is_some()
            || self.hud.state().selection != interface_hud::Selection::None
        {
            return Err(fault);
        }
        Ok(())
    }

    pub(super) fn complete_early_hud(
        &mut self,
        work: WorkId,
        content: &Content,
    ) -> Result<(), ProductionFault> {
        self.validate_early_hud_current(content)?;
        let Caller::Player {
            prepared,
            stage: PlayerEntryStage::HudWeapons,
            charging: true,
            completed,
            completion_work,
        } = &self.caller
        else {
            return Err(ProductionFault::WrongScope);
        };
        let fault =
            ProductionFault::Unsupported(Unsupported::Frame(Dependency::HudWeaponsPreparation));
        if work != PlayerEntryStage::HudWeapons.identity() {
            return Err(fault);
        }
        let mut hud = self.hud.clone();
        let controls =
            super::hud_stage::complete_controls_receipt(&mut hud, work).map_err(|_| fault)?;
        let mut appearance = self.appearance.clone();
        appearance
            .complete_cannon(&controls.answer.0)
            .map_err(|_| fault)?;
        self.hud = hud;
        self.appearance = appearance;
        self.caller = Caller::PlayerContinuation {
            prepared: Box::new(prepared.clone()),
            completed: completed.clone(),
            completion_work: completion_work.clone(),
            controls,
            stage: super::player_continuation::Stage::Projectiles,
            charging: false,
            remaining: Model(Box::new(
                super::borrowed_player_preparation::PlayerContinuationCompletions::default(),
            )),
            remaining_work: Vec::new(),
            motion_reset: None,
            run: super::movement_caller::Run::Idle,
            ordinary: None,
            support: super::movement_support_contract::Stage::Idle,
            ground: super::movement_ground_contract::Stage::Idle,
            down: super::movement_down_contract::Stage::Idle,
            post_down: super::post_down_caller::Stage::Idle,
        };
        Ok(())
    }
}
