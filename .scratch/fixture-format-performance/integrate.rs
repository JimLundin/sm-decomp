//! One-time mechanical integration edits, scoped to named source files.
use std::{fs, path::Path};
fn edit(path: &str, f: impl FnOnce(String) -> String) {
    let path = Path::new(path);
    let original = fs::read_to_string(path).unwrap();
    let next = f(original);
    fs::write(path, next).unwrap();
}
fn cut(text: &mut String, start: &str, end: &str) {
    let a = text.find(start).unwrap();
    let b = a + text[a..].find(end).unwrap();
    text.replace_range(a..b, "");
}
fn main() {
    for name in ["family.rs", "translation_native.rs", "translation_native_recipe.rs", "inputs.rs", "live.rs"] {
        edit(&format!("crates/pipeline/src/{name}"), |text| text.replace("\"manifest.toml\"", "\"manifest.smf\""));
    }
    edit("crates/pipeline/src/translation_native.rs", |text| {
        let mut text = text.replace("let original = family::object(&manifest_path)?;", "let original = crate::fixture_format::read(&manifest_path)?;");
        for (declaration, variable) in [("declared", "path"), ("prior_manifest", "prior.run.join(\"manifest.smf\")"),
            ("declared", "verdict.run.join(\"manifest.smf\")"), ("manifest", "manifest_path"),
            ("manifest", "run.join(\"manifest.smf\")"), ("declaration", "run.join(\"manifest.smf\")"),
            ("declared", "checkout.join(path)"), ("original", "checkout.join(path)"), ("manifest", "current.join(manifest_path)")]
        {
            text = text.replace(&format!("let {declaration}: Manifest = family::object(&{variable})?;"),
                &format!("let {declaration} = crate::fixture_format::read(&{variable})?;"));
        }
        text
    });
    edit("crates/pipeline/src/inputs.rs", |text| text.replace("crate::family::object(&run.join(\"manifest.smf\"))?", "crate::fixture_format::read(&run.join(\"manifest.smf\"))?"));
    edit("crates/pipeline/src/live.rs", |text| {
        text.replace("document(&verdict_path.parent()", "crate::fixture_format::read(&verdict_path.parent()")
            .replace("document(&path.parent().context(\"projectile predecessor parent missing\")", "crate::fixture_format::read(&path.parent().context(\"projectile predecessor parent missing\")")
            .replace("document(&Path::new(plm).parent()", "crate::fixture_format::read(&Path::new(plm).parent()")
    });
    edit("crates/pipeline/src/translation_migration.rs", |text| text.replace(
        "let manifest: crate::family_model::Manifest = crate::family::object(&checkout.join(&entry.manifest))?;",
        "let manifest = crate::fixture_format::read(&checkout.join(&entry.manifest))?;"));
    edit("crates/pipeline/src/batch.rs", |mut text| {
        cut(&mut text, "#[derive(Debug, Deserialize, PartialEq)]\nstruct Discovery", "pub fn catalog(");
        text = text.replace("bounded::text(path)", "bounded::read(path)")
            .replace("Result<String>) -> Result<Catalog>", "Result<Vec<u8>>) -> Result<Catalog>")
            .replace("toml::from_str(&read(&checkout.join(\"tools/port/batch.toml\"))?)?", "toml::from_str(std::str::from_utf8(&read(&checkout.join(\"tools/port/batch.toml\"))?)?)?")
            .replace("v != \"toml\"", "v != crate::fixture_format::EXTENSION")
            .replace("discovery(&read(&path)?)", "crate::fixture_format::decode(read(&path)?)")
            .replace("manifest.rust_module.is_none() || manifest.routines.is_none()", "manifest.rust_module.is_empty() || manifest.routines.is_empty()")
            .replace("manifest.routines.unwrap().iter()", "manifest.routines.iter()")
            .replace("tools/port/{name}.toml", "tools/port/{name}.smf")
            .replace("tools/port/discovered.toml", "tools/port/discovered.smf");
        cut(&mut text, "    #[test]\n    fn discovery_matches_full_decode", "    #[test]\n    #[ignore = \"reads the checkout's complete manifest inventory\"]");
        text.replace("fs::write(port.join(\"one.toml\"), \"rust_module='one.rs'\\n[[routines]]\\nsymbol='One'\\n\")?;",
            "crate::fixture_format::write(&port.join(\"one.smf\"), &family_model::Manifest { rust_module: \"one.rs\".into(), ..Default::default() })?;")
    });
    edit("crates/pipeline/src/ridley_subsystem_fixtures.rs", |mut text| {
        cut(&mut text, "fn manifest_toml(", "pub fn generate(");
        let start = text.find("    // Keep fixture writes reviewable").unwrap();
        let end = start + text[start..].find("    ensure!(checkout.join").unwrap();
        text.replace_range(start..end, "    crate::fixture_format::write(output, &manifest)?;\n");
        text
    });
    for name in ["family.rs", "callback.rs", "plm_spawn.rs", "ridley_subsystem_adapter.rs"] {
        edit(&format!("crates/pipeline/src/{name}"), |text| text.replace(
            "(\"family.rs\", include_str!(\"family.rs\")),",
            "(\"family.rs\", include_str!(\"family.rs\")),\n        (\"family_model.rs\", include_str!(\"family_model.rs\")),\n        (\"fixture_format.rs\", include_str!(\"fixture_format.rs\")),\n        (\"fixture_writes.rs\", include_str!(\"fixture_writes.rs\")),"));
    }
    edit("crates/pipeline/src/translation_native.rs", |text| text.replace("    \"family_model.rs\",", "    \"family_model.rs\",\n    \"fixture_format.rs\",\n    \"fixture_writes.rs\",\n    \"fixture_cli.rs\","));
}
