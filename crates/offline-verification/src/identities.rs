//! Required computation attributes projected from existing records.
//!
//! Content hashes and catalog labels are lookup hints. The complete admitted
//! family already binds source bytes, generated/compiler inputs, binaries,
//! ordered fixtures, comparisons and producer provenance. This additional
//! projection binds the current registered recipe to that admitted computation;
//! it is neither a build-cache key nor a new receipt or attempt authority.
use crate::{family::AdmittedFamily, family_model::Implementation, recipe::Recipe};
use anyhow::{Context, Result, ensure};

#[derive(Debug, PartialEq, Eq)]
struct RecipeComputation<'a> {
    dependency_contract: &'a str,
    callback_adapter: Option<&'a str>,
}

impl RecipeComputation<'_> {
    fn required(recipe: Option<&Recipe>) -> RecipeComputation<'static> {
        let (dependency_contract, callback_adapter) = match recipe {
            Some(Recipe::Callback) => ("rust-callback-v2", Some("callback")),
            Some(Recipe::PowerBomb) => ("rust-callback-v2", Some("power-bomb")),
            Some(Recipe::GameHost) => ("rust-callback-v2", Some("game-host")),
            Some(Recipe::Plm) => ("rust-plm-v2", None),
            Some(Recipe::Ridley) => (crate::ridley_subsystem_adapter::CONTRACT, Some("ridley-subsystem")),
            _ => ("rust-family-v2", None),
        };
        RecipeComputation { dependency_contract, callback_adapter }
    }

    fn admitted(implementation: &Implementation) -> RecipeComputation<'_> {
        RecipeComputation {
            dependency_contract: &implementation.dependency_contract,
            callback_adapter: implementation.callback_adapter.as_deref(),
        }
    }
}

pub(crate) fn check_registered_computation(kind: &str, admitted: &AdmittedFamily) -> Result<()> {
    ensure!(
        [
            "family",
            "offline-family",
            "dispatcher",
            "plm-instructions",
            "projectile",
            "offline-projectile",
            "callback",
            "plm",
            "power-bomb",
            "ridley-subsystem"
        ]
        .contains(&kind),
        "unknown registered computation: {kind}"
    );
    let recipe = crate::recipe::for_adapter(kind, &admitted.original)?;
    let required = RecipeComputation::required(recipe.as_ref());
    let actual = RecipeComputation::admitted(
        admitted.report.implementation.as_ref().context("admitted implementation missing")?,
    );
    ensure!(
        actual == required,
        "registered computation differs from admitted recipe: required {required:?}, actual {actual:?}"
    );
    Ok(())
}
