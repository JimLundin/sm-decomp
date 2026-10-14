use std::fs;
fn edit(path: &str, f: impl FnOnce(String) -> String) {
    fs::write(path, f(fs::read_to_string(path).unwrap())).unwrap();
}
fn main() {
    for file in ["family.rs", "callback.rs", "plm_spawn.rs", "affected.rs"] {
        edit(&format!("crates/pipeline/tests/{file}"), |mut text| {
            let after = text.find('\n').unwrap()+1;
            text.insert_str(after, "#[path = \"support/binary_fixture.rs\"]\nmod binary_fixture;\n");
            for name in ["fixture", "other", "one-native", "discovered", "a", "b"] {
                text = text.replace(&format!("{name}.toml"), &format!("{name}.smf"));
            }
            text = text.replace("\"manifest.toml\"", "\"manifest.smf\"");
            if file == "family.rs" {
                text = text.replace("    fs::write(p, bytes).unwrap();", "    if p.extension().is_some_and(|e| e == \"smf\") { binary_fixture::write(&p, bytes).unwrap(); } else { fs::write(p, bytes).unwrap(); }")
                    .replace("fs::read_to_string(&path)", "binary_fixture::read(&path)")
                    .replace("String::from_utf8(original.clone())", "binary_fixture::read(&path)")
                    .replace("fs::read_to_string(&first_manifest)", "binary_fixture::read(&first_manifest)")
                    .replace("fs::read_to_string(output.join(\"tools/port/fixture.smf\"))", "binary_fixture::read(&output.join(\"tools/port/fixture.smf\"))")
                    .replace("fs::read_to_string(checkout.join(\"tools/port/observed-projectile-initializers.smf\"))", "binary_fixture::read(&checkout.join(\"tools/port/observed-projectile-initializers.smf\"))")
                    .replace("    let verdict = project.verify_mode(true, \"fresh\");\n    assert_eq!(verdict[\"verified_cases\"].as_integer(), Some(2));\n    assert_eq!(fs::read_to_string(project.0.join(\"tools/port/fixture.smf\")).unwrap(), manifest);", "    let binary = fs::read(project.0.join(\"tools/port/fixture.smf\")).unwrap();\n    let verdict = project.verify_mode(true, \"fresh\");\n    assert_eq!(verdict[\"verified_cases\"].as_integer(), Some(2));\n    assert_eq!(fs::read(project.0.join(\"tools/port/fixture.smf\")).unwrap(), binary);");
            } else if file == "affected.rs" {
                text = text.replace("        fs::write(path, text).unwrap();", "        if path.extension().is_some_and(|e| e == \"smf\") { binary_fixture::write(&path, text).unwrap(); } else { fs::write(path, text).unwrap(); }");
            } else {
                text = text.replace("    fs::write(\n        &manifest,", "    binary_fixture::write(\n        &manifest,");
            }
            text
        });
    }
    edit("crates/pipeline/src/translation_native.rs", |text| {
        text.replace("tools/port/alias.toml", "tools/port/alias.smf")
            .replace("fs::write(checkout.join(\"tools/port/alias.smf\"), toml::to_string(&manifest)?)?;", "crate::fixture_format::write(&checkout.join(\"tools/port/alias.smf\"), &manifest)?;")
            .replace("fs::write(checkout.join(\"manifest.smf\"), b\"family='test'\")?;", "crate::fixture_format::write(&checkout.join(\"manifest.smf\"), &Manifest { family: \"test\".into(), ..Default::default() })?;")
            .replace("fs::write(checkout.join(\"manifest.smf\"), toml::to_string(&manifest)?)?;", "crate::fixture_format::write(&checkout.join(\"manifest.smf\"), &manifest)?;")
            .replace("(\"family_model.rs\", include_str!(\"family_model.rs\")),", "(\"family_model.rs\", include_str!(\"family_model.rs\")),\n            (\"fixture_format.rs\", include_str!(\"fixture_format.rs\")),\n            (\"fixture_writes.rs\", include_str!(\"fixture_writes.rs\")),\n            (\"fixture_cli.rs\", include_str!(\"fixture_cli.rs\")),")
    });
    edit("crates/pipeline/src/translation_managed_tests.rs", |text| text.replace("tools/port/managed-diagnostic.toml", "tools/port/managed-diagnostic.smf"));
    edit("crates/pipeline/src/affected.rs", |text| {
        text.replace("root.join(format!(\"tools/port/{name}.smf\")),\n                format!(\"family='{name}'\\nbank='sm_80.c'\\nrust_module='crates/port/src/{name}.rs'\\nroutines=[]\\n\"),",
            "root.join(format!(\"tools/port/{name}.smf\")),\n                crate::fixture_format::encode(&family_model::Manifest { family: name.into(), bank: \"sm_80.c\".into(), rust_module: format!(\"crates/port/src/{name}.rs\"), ..Default::default() })?,")
            .replace("tools/port/a.toml", "tools/port/a.smf")
    });
    edit("crates/pipeline/src/ridley_instruction_fixtures.rs", |text| {
        text.replace("use crate::{atomic_toml, digest};", "use crate::digest;")
            .replace("atomic_toml(output, &manifest)?", "crate::fixture_format::write(output, &manifest)?")
            .replace("writes: base(64, 0, 99, 0),", "writes: base(64, 0, 99, 0).into(),")
            .replace("writes: base(64, orientation, health, area),", "writes: base(64, orientation, health, area).into(),")
            .replace("                writes,", "                writes: writes.into(),")
            .replace("            writes,", "            writes: writes.into(),")
    });
}
