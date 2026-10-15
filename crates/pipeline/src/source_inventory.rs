//! CLI publication of typed Recovered C source inventories.
use anyhow::{Context, Result, ensure};
use std::{fs, path::Path};
use tool_execution::bounded;

pub(crate) fn run(root: &Path, source: &Path, output: &Path) -> Result<()> {
    decomp_source::context::limited(|| publish(decomp_source::inventory(root, source)?, output))
}

pub(crate) fn run_until(root: &Path, source: &Path, output: &Path, deadline: u64, max_bytes: u64) -> Result<()> {
    decomp_source::context::until(deadline, max_bytes, || publish(decomp_source::inventory(root, source)?, output))
}

fn publish(report: decomp_source::Inventory, output: &Path) -> Result<()> {
    bounded::safe(output)?;
    fs::create_dir_all(output.parent().context("source inventory output parent missing")?)?;
    ensure!(output.extension().is_some_and(|extension| extension == "toml"), "new pipeline metadata must use .toml");
    bounded::write(output, toml::to_string_pretty(&report)?.as_bytes())?;
    println!("Indexed {} definitions: {}", report.definition_count, output.display());
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn inventory_generation_inherits_expired_budget_before_any_output() -> Result<()> {
        let directory = tool_execution::testing::Directory::new()?;
        let output = directory.0.join("out/inventory.toml");
        let error =
            run_until(&directory.0.join("absent"), &directory.0.join("absent-source"), &output, 0, 1024).unwrap_err();
        assert!(error.to_string().contains("deadline"));
        assert!(!output.parent().unwrap().exists());
        Ok(())
    }
}
