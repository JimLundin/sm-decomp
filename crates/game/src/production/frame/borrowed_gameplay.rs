//! Actual current scene controller and deferred gameplay-dispatch dependency.
use super::borrowed_preparation::BuilderCompletionReceipt;
use canonical::Canonical;

/// Actual current scene owner, independent of simulation or Haze preparation.
#[derive(Canonical, Clone, Copy, Debug, PartialEq, Eq)]
pub enum SceneController {
    /// Current scene owner offers ordinary main gameplay.
    MainGameplay,
    /// A different current scene; no route for it is admitted here.
    Other,
}
/// Route offered by the actual completed scene-controller operation.
#[derive(Canonical, Clone, Copy, Debug, PartialEq, Eq)]
pub enum GameplayRoute {
    /// Ordinary actor/debug/palette preparation followed by gameplay input.
    MainGameplay,
}
/// Deferred external current-owner offer, retained until actual dispatch completes.
#[derive(Canonical, Clone, Debug, PartialEq, Eq)]
pub struct GameplayDispatchOffer {
    /// Actual current scene dependency to revalidate at dispatch.
    pub dependency: SceneController,
    /// Actual offered route.
    pub route: GameplayRoute,
}
/// Actual dispatch completion after current builder reset and scene revalidation.
#[derive(Canonical, Clone, Debug, PartialEq, Eq)]
pub struct GameplayDispatchReceipt {
    /// Actual current scene owner that completed dispatch.
    pub scene: SceneController,
    /// Actual completed route.
    pub route: GameplayRoute,
}
/// Missing actual reset completion or changed current scene dependency.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum GameplayDependencyFault {
    /// The actual current scene is outside the offered route.
    Scene,
    /// The supplied actual completed builder is not the required reset owner.
    Builder,
}
impl GameplayDispatchOffer {
    /// Consume once only after the actual dispatch's full physical completion.
    /// `builder` must be the actual completed reset, not an initial or future drawing owner.
    /// The caller retains its actual opportunity identity and complete physical progress.
    /// # Errors
    /// Rejects changed or unsupported scenes and missing required builder-reset facts.
    pub fn complete(
        self,
        current: SceneController,
        builder: &BuilderCompletionReceipt,
    ) -> Result<GameplayDispatchReceipt, GameplayDependencyFault> {
        if current != self.dependency {
            return Err(GameplayDependencyFault::Scene);
        }
        GameplayDispatchReceipt::dispatch(current, builder)
    }
}

impl GameplayDispatchReceipt {
    /// Select the current scene's route after the live builder reset completes.
    /// # Errors
    /// Rejects other scenes and invalid builder reset results.
    pub fn dispatch(
        current: SceneController,
        builder: &BuilderCompletionReceipt,
    ) -> Result<Self, GameplayDependencyFault> {
        if current != SceneController::MainGameplay {
            return Err(GameplayDependencyFault::Scene);
        }
        let terminal = &builder.terminal;
        if terminal.cursor != 0
            || terminal.prepared_flags != 0
            || terminal.prepared_top != 0
            || terminal.prepared_bottom != 0
            || !terminal
                .backing
                .iter()
                .take(512)
                .eq(builder.dependency.backing.iter().take(512))
            || terminal.backing.iter().skip(512).any(|byte| *byte != 0)
        {
            return Err(GameplayDependencyFault::Builder);
        }
        Ok(Self {
            scene: current,
            route: GameplayRoute::MainGameplay,
        })
    }
}
