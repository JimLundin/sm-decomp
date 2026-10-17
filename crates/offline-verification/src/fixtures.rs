//! Current Fixture values, binary codec, and explicit Fixture authoring.
pub use crate::family_model::{Asset, Case, Generation, Manifest, PlmGeneration, Routine, Write, Writes};
pub use crate::fixture_format::{
    EXTENSION, RUN_MANIFEST, case_identity, decode, encode, identity, read, read_authenticated, write,
};
pub use crate::plm_fixtures::generate as generate_plm;
use anyhow::{Result, ensure};

impl Routine {
    pub fn argument_types(&self) -> Result<Vec<String>> {
        if let Some(values) = &self.arg_types {
            return Ok(values.clone());
        }
        ensure!(self.args <= 2, "too many arguments");
        Ok(vec!["uint16".into(); self.args])
    }

    pub fn has_valid_symbol(&self) -> bool {
        crate::family::identifier(&self.symbol)
    }
}

impl Manifest {
    pub fn uses_dispatcher(&self) -> bool {
        self.projectile_host
            || self.projectile_rendering
            || self.routines.iter().any(|routine| {
                routine.pointer_bank.is_some()
                    || matches!(
                        routine.symbol.as_str(),
                        "EprojInstruction" | "EprojPreinstruction" | "PlmInstruction" | "PlmFrame" | "PlmControlFrame"
                    )
            })
    }
}

/// Generate the maintained callback ABI source for the explicit authoring CLI.
pub fn callback_abi(manifest: &Manifest, module: &std::path::Path) -> Result<String> {
    crate::callback::abi(manifest, module)
}
