//! Command and file contracts shared by execution and receipt authentication.
use super::Recipe;
use crate::family_model::{Case, Manifest, Routine};
use anyhow::{Context, Result, ensure};
use std::path::{Path, PathBuf};
use translation_execution::confinement::{self, Program, ReadOnlyBind};

pub(crate) enum FixtureSelection<'a> {
    Case { routine: &'a Routine, fixture: &'a Case, index: usize },
    AbiControls,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum FixtureKind {
    Family,
    Callback,
    Plm,
    Ridley,
    PlmAbi,
    RidleyAbi,
}

#[derive(Debug, PartialEq, Eq)]
pub(crate) struct FixtureCommand {
    pub command: Vec<String>,
    pub kind: FixtureKind,
}

pub(crate) fn fixture_command(
    recipe: Option<&Recipe>,
    manifest: &Manifest,
    selection: FixtureSelection<'_>,
    run: &Path,
    label: &str,
) -> Result<FixtureCommand> {
    let FixtureSelection::Case { routine, fixture, index } = selection else {
        let (command, kind) = match recipe {
            Some(Recipe::Plm) => (
                vec![
                    run.join("spawn").display().to_string(),
                    "abi".into(),
                    "0".into(),
                    "0".into(),
                    "reserved".into(),
                    "reserved".into(),
                    run.join("rom.bin").display().to_string(),
                    "reserved".into(),
                ],
                FixtureKind::PlmAbi,
            ),
            Some(Recipe::Ridley) => {
                (vec![run.join("ridley").display().to_string(), "--abi-controls".into()], FixtureKind::RidleyAbi)
            }
            _ => anyhow::bail!("unexpected native ABI operation"),
        };
        return Ok(FixtureCommand { command, kind });
    };
    let directory = run.join(&fixture.name);
    let text = |path: PathBuf| path.display().to_string();
    let before = text(directory.join("before.wram"));
    let after = text(directory.join(format!("{label}-after.wram")));
    let rom = text(run.join("rom.bin"));
    let a = fixture.a.to_string();
    let b = fixture.b.to_string();
    let (command, kind) = match recipe {
        Some(Recipe::Callback | Recipe::PowerBomb | Recipe::GameHost) => (
            vec![
                text(run.join("collision")),
                label.into(),
                routine.symbol.clone(),
                a,
                b,
                before,
                after,
                index.to_string(),
                rom,
                text(directory.join(label)),
            ],
            FixtureKind::Callback,
        ),
        Some(Recipe::Plm) => (
            vec![
                text(run.join("spawn")),
                label.into(),
                a,
                b,
                before,
                after,
                rom,
                match routine.symbol.as_str() {
                    "terrain" => "0",
                    "room" => "1",
                    "hardcoded" => "2",
                    _ => anyhow::bail!("invalid allocator"),
                }
                .into(),
            ],
            FixtureKind::Plm,
        ),
        Some(Recipe::Ridley) => (
            vec![
                text(run.join(label)),
                label.into(),
                routine.symbol.clone(),
                a,
                b,
                before,
                after,
                "0".into(),
                rom,
                text(directory.join("host-before.bin")),
                text(directory.join(format!("{label}-host-after.bin"))),
                text(directory.join(format!("{label}-effects.log"))),
                text(directory.join(format!("{label}-ppu.bin"))),
                text(directory.join(format!("{label}-services.bin"))),
                text(directory.join("ppu-before.bin")),
            ],
            FixtureKind::Ridley,
        ),
        _ => {
            let mut command = vec![text(run.join(label)), routine.symbol.clone(), a, b, before, after];
            if manifest.rom.is_some() {
                command.push(rom);
            }
            if manifest.projectile_host {
                command.extend([
                    text(directory.join("host-before.bin")),
                    text(directory.join(format!("{label}-host-after.bin"))),
                ]);
            }
            (command, FixtureKind::Family)
        }
    };
    Ok(FixtureCommand { command, kind })
}

pub(crate) fn fixture_program(
    recipe: Option<&Recipe>,
    manifest: &Manifest,
    selection: FixtureSelection<'_>,
    run: &Path,
    label: &str,
    helper: &Path,
) -> Result<Program> {
    let fixture = fixture_command(recipe, manifest, selection, run, label)?;
    confined_fixture_program(helper, &fixture.command, fixture.kind)
}

/// Bind only the named inputs and outputs admitted by the selected recipe.
pub(crate) fn confined_fixture_program(helper: &Path, command: &[String], kind: FixtureKind) -> Result<Program> {
    let mut program = Program {
        executable: PathBuf::from(command.first().context("empty fixture command")?),
        arguments: command[1..].to_vec(),
        inputs: vec![],
        runtime: confinement::helper_runtime()?,
        helper: helper.into(),
        outputs: vec![],
    };
    let (lengths, inputs, outputs): (&[usize], Vec<(usize, &str)>, Vec<(usize, &str)>) = match kind {
        FixtureKind::Family => (
            &[6, 7, 9],
            [(4, "before.wram"), (6, "rom.bin"), (7, "host-before.bin")]
                .into_iter()
                .filter(|(i, _)| *i < command.len())
                .collect(),
            [(5, "after.wram"), (8, "host-after.bin")].into_iter().filter(|(i, _)| *i < command.len()).collect(),
        ),
        FixtureKind::Callback => (&[10], vec![(5, "before.wram"), (8, "rom.bin")], vec![(6, "after.wram")]),
        FixtureKind::Plm => (&[8], vec![(4, "before.wram"), (6, "rom.bin")], vec![(5, "after.wram")]),
        FixtureKind::Ridley => (
            &[15],
            vec![(5, "before.wram"), (8, "rom.bin"), (9, "host-before.bin"), (14, "ppu-before.bin")],
            vec![(6, "after.wram"), (10, "host-after.bin"), (11, "effects.log"), (12, "ppu.bin"), (13, "services.bin")],
        ),
        FixtureKind::PlmAbi => (&[8], vec![(6, "rom.bin")], vec![]),
        FixtureKind::RidleyAbi => (&[2], vec![], vec![]),
    };
    ensure!(lengths.contains(&command.len()), "confined fixture argument contract differs");
    for (index, name) in inputs {
        let destination = Path::new("/inputs").join(name);
        program.inputs.push(ReadOnlyBind { source: PathBuf::from(&command[index]), destination: destination.clone() });
        program.arguments[index - 1] = destination.display().to_string();
    }
    for (index, name) in outputs {
        program.arguments[index - 1] = Path::new("/work").join(name).display().to_string();
        program.outputs.push(name.into());
    }
    if matches!(kind, FixtureKind::Callback) {
        program.arguments[8] = "/work".into();
        program.outputs.extend(["callbacks.txt".into(), "callback-states.bin".into()]);
    }
    Ok(program)
}

pub(crate) fn fixture_output_path(run: &Path, label: &str, name: &Path) -> PathBuf {
    if name == Path::new("callbacks.txt") {
        run.join(label).join(name)
    } else if ["after.wram", "host-after.bin", "effects.log", "ppu.bin", "services.bin", "callback-states.bin"]
        .iter()
        .any(|n| name == Path::new(n))
    {
        run.join(format!("{label}-{}", name.display()))
    } else {
        run.join(name)
    }
}

#[derive(Debug, PartialEq, Eq)]
pub(crate) struct CompilerCommand {
    pub command: Vec<String>,
    pub generated: Vec<&'static str>,
    pub outputs: Vec<&'static str>,
    pub full_c: bool,
}

pub(crate) fn compiler_files(recipe: Option<&Recipe>, rust: bool) -> (Vec<&'static str>, Vec<&'static str>, bool) {
    if rust {
        let mut generated = vec!["wrapper.rs", "memory.rs"];
        let output = match recipe {
            Some(Recipe::Callback | Recipe::PowerBomb | Recipe::GameHost) => "libcollision.a",
            Some(Recipe::Plm) => "libplm.a",
            Some(Recipe::Ridley) => {
                generated.push("fixture-worker.rs");
                "libridley.a"
            }
            _ => "rust",
        };
        return (generated, vec![output, "compiler.d"], false);
    }
    let (generated, binary) = match recipe {
        Some(Recipe::Dispatcher | Recipe::GeneratedPlm { .. }) => {
            (vec!["oracle.c", "plm-shim.c", "host-support.c"], "c")
        }
        Some(Recipe::Callback) => (vec!["oracle.c", "libcollision.a"], "collision"),
        Some(Recipe::PowerBomb) => (vec!["oracle.c", "libcollision.a", "sm_88-allocation-seam.c"], "collision"),
        Some(Recipe::GameHost) => {
            (vec!["oracle.c", "libcollision.a", "sm_88-allocation-seam.c", "sm_80-vram-seam.c"], "collision")
        }
        Some(Recipe::Plm) => (vec!["oracle.c", "libplm.a"], "spawn"),
        Some(Recipe::Ridley) => (
            vec![
                "oracle.c",
                "libridley.a",
                "ridley-services-sm_80.c",
                "ridley-services-sm_86.c",
                "ridley-services-sm_90.c",
                "ridley-services-sm_a0.c",
            ],
            "ridley",
        ),
        _ => (vec!["oracle.c"], "c"),
    };
    (generated, vec![binary], super::full_c(recipe))
}

pub(crate) fn rust_compiler_command(recipe: Option<&Recipe>, run: &Path) -> CompilerCommand {
    let (generated, outputs, full_c) = compiler_files(recipe, true);
    let mut command = vec!["rustc".into(), "--edition=2024".into(), "-O".into()];
    if outputs[0] != "rust" {
        command.push("--crate-type=staticlib".into());
    }
    command.extend([
        format!("--emit=link,dep-info={}", run.join("compiler.d").display()),
        run.join("wrapper.rs").display().to_string(),
        "-o".into(),
        run.join(outputs[0]).display().to_string(),
    ]);
    CompilerCommand { command, generated, outputs, full_c }
}

/// The caller supplies the trusted, sorted original C units and writes any
/// generated seams. This builder owns the complete compiler argument contract.
pub(crate) fn c_compiler_command(
    recipe: Option<&Recipe>,
    manifest: &Manifest,
    source: &Path,
    run: &Path,
    c_units: &[PathBuf],
    fork: bool,
) -> Result<CompilerCommand> {
    let (generated, outputs, full_c) = compiler_files(recipe, false);
    let mut command = vec![
        "cc".into(),
        if matches!(recipe, Some(Recipe::PowerBomb | Recipe::GameHost | Recipe::Plm)) { "-O0" } else { "-O2" }.into(),
        "-fno-strict-aliasing".into(),
        "-ffunction-sections".into(),
        "-fdata-sections".into(),
        "-iquote".into(),
        source.join("src").display().to_string(),
    ];
    match recipe {
        Some(Recipe::Callback | Recipe::PowerBomb | Recipe::GameHost) => {
            let mut units = c_units.to_vec();
            if matches!(recipe, Some(Recipe::PowerBomb | Recipe::GameHost)) {
                let bank = units
                    .iter_mut()
                    .find(|p| p.file_name().is_some_and(|n| n == "sm_88.c"))
                    .context("bank88 missing")?;
                *bank = run.join("sm_88-allocation-seam.c");
            }
            if matches!(recipe, Some(Recipe::GameHost)) {
                let bank = units
                    .iter_mut()
                    .find(|p| p.file_name().is_some_and(|n| n == "sm_80.c"))
                    .context("bank80 missing")?;
                *bank = run.join("sm_80-vram-seam.c");
            }
            command.extend(units.iter().map(|path| path.display().to_string()));
            if matches!(recipe, Some(Recipe::GameHost)) {
                command.extend(
                    ["WriteReg", "ppu_write", "snes_read", "snes_write"].map(|name| format!("-Wl,--wrap={name}")),
                );
            }
            command.extend([
                run.join("oracle.c").display().to_string(),
                run.join("libcollision.a").display().to_string(),
                "-Wl,--gc-sections".into(),
            ]);
            command.extend(
                ["SpawnPLM", "Call", "cpu_runOpcode", "DecompressToMem"].map(|name| format!("-Wl,--wrap={name}")),
            );
            command.extend(["-lm", "-ldl", "-lpthread"].map(String::from));
        }
        Some(Recipe::Plm) => {
            command.extend(c_units.iter().map(|path| path.display().to_string()));
            command.extend([
                run.join("oracle.c").display().to_string(),
                run.join("libplm.a").display().to_string(),
                "-Wl,--gc-sections".into(),
            ]);
            command.extend(["Call", "cpu_runOpcode", "DecompressToMem"].map(|name| format!("-Wl,--wrap={name}")));
            command.extend(["-lm", "-ldl", "-lpthread"].map(String::from));
        }
        Some(Recipe::Ridley) => {
            command.extend(c_units.iter().map(|path| {
                let bank = path.file_name().and_then(|name| name.to_str()).unwrap_or("");
                if ["sm_80.c", "sm_86.c", "sm_90.c", "sm_a0.c"].contains(&bank) {
                    run.join(format!("ridley-services-{bank}")).display().to_string()
                } else {
                    path.display().to_string()
                }
            }));
            command.extend([
                run.join("oracle.c").display().to_string(),
                run.join("libridley.a").display().to_string(),
                "-Wl,--gc-sections".into(),
            ]);
            command.extend(
                ["Call", "cpu_runOpcode", "DecompressToMem", "WriteReg", "ReadReg", "snes_read", "snes_write"]
                    .map(|name| format!("-Wl,--wrap={name}")),
            );
            command.extend(["-lm", "-ldl", "-lpthread"].map(String::from));
        }
        _ => {
            for name in std::iter::once(&manifest.bank).chain(&manifest.c_support) {
                ensure!(
                    super::relative(Path::new(name))
                        && !name.contains('/')
                        && name.starts_with("sm_")
                        && name.ends_with(".c"),
                    "invalid source bank"
                );
                let path = source.join("src").join(name).display().to_string();
                if !command.contains(&path) {
                    command.push(path);
                }
            }
            if matches!(recipe, Some(Recipe::Dispatcher | Recipe::GeneratedPlm { .. })) {
                command.truncate(7);
                let eproj = manifest.projectile_host
                    || manifest.projectile_rendering
                    || manifest.routines.iter().any(|routine| routine.symbol == "EprojInstruction");
                command.extend(
                    c_units
                        .iter()
                        .filter(|path| !eproj || path.file_name().is_none_or(|name| name != "sm_86.c"))
                        .map(|path| path.display().to_string()),
                );
                command.extend([
                    run.join("plm-shim.c").display().to_string(),
                    run.join("host-support.c").display().to_string(),
                    "-lm".into(),
                ]);
            }
            command.extend([run.join("oracle.c").display().to_string(), "-Wl,--gc-sections".into()]);
            if fork {
                command.extend([
                    run.join("libfixture_worker.a").display().to_string(),
                    "-ldl".into(),
                    "-lpthread".into(),
                    "-lm".into(),
                ]);
            }
        }
    }
    command.extend(["-o".into(), run.join(outputs[0]).display().to_string()]);
    Ok(CompilerCommand { command, generated, outputs, full_c })
}
