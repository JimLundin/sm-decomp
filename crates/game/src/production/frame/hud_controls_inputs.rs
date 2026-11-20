//! Early HUD inputs joined with completed physical controls and current inventory.

use super::borrowed_gameplay::{GameplayDispatchReceipt, GameplayRoute, SceneController};
use super::borrowed_player_preparation::PoseInputCompletion;
use canonical::Canonical;
use interface_hud::{Ammo, Ammunition, CancelButton, Equipped, ItemControlInputs, SelectButton};
use types::{Action, Pose};

/// Independently qualified physical item-select binding and early switch permission.
#[derive(Canonical, Clone, Copy, Debug, PartialEq, Eq)]
pub struct Bindings {
    /// Physical item-select button bits, separate from movement actions.
    pub select: u16,
    /// Current source-qualified permission for this bounded early opportunity.
    pub can_switch: bool,
}

/// Current count and carrying capacity supplied by the inventory owner.
#[derive(Canonical, Clone, Copy, Debug, PartialEq, Eq)]
pub struct Stock {
    /// Current count.
    pub count: u16,
    /// Current carrying capacity.
    pub capacity: u16,
}

/// Current inventory, independently admitted from progress rather than the last HUD drawing.
#[derive(Canonical, Clone, Copy, Debug, PartialEq, Eq)]
pub struct CurrentInventory {
    /// Current missiles.
    pub missiles: Stock,
    /// Current super missiles.
    pub super_missiles: Stock,
    /// Current power bombs.
    pub power_bombs: Stock,
    /// Grapple beam equipped.
    pub grapple_beam: bool,
    /// X-ray scope equipped.
    pub x_ray_scope: bool,
}

/// Current actual owner joins at the early controls opportunity.
#[derive(Clone, Copy)]
pub struct CurrentControls<'a> {
    /// Actual completed main-gameplay dispatch.
    pub gameplay: &'a GameplayDispatchReceipt,
    /// Actual current scene owner at this opportunity.
    pub scene: SceneController,
    /// Actual completed physical pose-input dependency and actions.
    pub pose_input: &'a PoseInputCompletion,
    /// Actual starting movement pose, before prospective placement.
    pub pose: Pose,
    /// Actual current progress inventory or independently admitted external current owner.
    pub inventory: &'a CurrentInventory,
}

/// A required current owner or inventory value is outside this bounded opportunity.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Fault {
    /// Actual scene or completed dispatch differs from main gameplay.
    Gameplay,
    /// Starting pose/action differs from the consumed pose-input dependency.
    Pose,
    /// This source-qualified early switch opportunity is unavailable.
    Permission,
    /// Current inventory exceeds the clean HUD domain.
    Inventory,
}

impl CurrentInventory {
    /// Retain exact typed current inventory quantities without copying HUD display history.
    #[must_use]
    pub fn from_current(ammunition: Ammunition, equipped: Equipped) -> Self {
        let stock = |ammo| {
            let stock = ammunition.of(ammo);
            Stock {
                count: stock.count,
                capacity: stock.capacity,
            }
        };
        Self {
            missiles: stock(Ammo::Missiles),
            super_missiles: stock(Ammo::SuperMissiles),
            power_bombs: stock(Ammo::PowerBombs),
            grapple_beam: equipped.grapple_beam,
            x_ray_scope: equipped.x_ray_scope,
        }
    }

    /// Validate the current owner, including quantities restored from checkpoints.
    /// # Errors
    /// Rejects current count or capacity outside the clean HUD domain.
    pub fn ammunition(self) -> Result<Ammunition, Fault> {
        let stock = |stock: Stock| interface_hud::Stock {
            count: stock.count,
            capacity: stock.capacity,
        };
        Ammunition::new(enum_map::enum_map! {
            Ammo::Missiles => stock(self.missiles),
            Ammo::SuperMissiles => stock(self.super_missiles),
            Ammo::PowerBombs => stock(self.power_bombs),
        })
        .ok_or(Fault::Inventory)
    }

    /// Current equipped item meanings consumed by HUD controls.
    #[must_use]
    pub const fn equipped(self) -> Equipped {
        Equipped {
            grapple_beam: self.grapple_beam,
            x_ray_scope: self.x_ray_scope,
        }
    }
}

impl Bindings {
    /// Bind actual completed physical controls to this current gameplay and inventory join.
    /// No prospective pose or translated movement-only button set substitutes for physical bits.
    /// # Errors
    /// Rejects changed current owners, unqualified switch permission or invalid inventory.
    pub fn inputs(self, current: CurrentControls<'_>) -> Result<ItemControlInputs, Fault> {
        if current.scene != SceneController::MainGameplay
            || current.gameplay.scene != current.scene
            || current.gameplay.route != GameplayRoute::MainGameplay
        {
            return Err(Fault::Gameplay);
        }
        let dependency = &current.pose_input.dependency;
        if current.pose != dependency.pose
            || current.pose.action != Action::Running
            || dependency.action != current.pose.action
        {
            return Err(Fault::Pose);
        }
        if !self.can_switch {
            return Err(Fault::Permission);
        }
        let held = dependency.held.0;
        let pressed = dependency.pressed.0;
        let cancel = dependency.bindings.cancel.0;
        Ok(ItemControlInputs {
            select: if pressed & self.select != 0 {
                SelectButton::Pressed
            } else {
                SelectButton::NotPressed
            },
            cancel: if pressed & cancel != 0 {
                CancelButton::Pressed
            } else if held & cancel != 0 {
                CancelButton::Held
            } else {
                CancelButton::NotPressed
            },
            can_switch: self.can_switch,
            ammunition: current.inventory.ammunition()?,
            equipped: current.inventory.equipped(),
        })
    }
}

#[cfg(test)]
#[path = "hud_controls_inputs/tests.rs"]
mod tests;
