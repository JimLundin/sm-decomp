//! Bounded diagnostic for the retained Ceres recipe link failure.
//! No Fixture is executed and this probe grants no verification acceptance.
use anyhow::{Context, Result, ensure};
use std::{fs, path::PathBuf};

fn main() -> Result<()> {
    let mut args = std::env::args_os().skip(1);
    let source = fs::canonicalize(PathBuf::from(args.next().context("source root")?))?;
    let oracle = fs::canonicalize(PathBuf::from(args.next().context("retained oracle")?))?;
    let run = PathBuf::from(args.next().context("new output directory")?);
    let mode = args.next().context("retained, minimal, full or full-host")?;
    ensure!(["retained", "minimal", "full", "full-host"].iter().any(|value| mode == *value), "unknown probe mode");
    let minimal = mode == "minimal";
    let full = mode == "full" || mode == "full-host";
    ensure!(args.next().is_none(), "unexpected arguments");
    ensure!(!run.exists(), "diagnostic output already exists");
    fs::create_dir_all(&run)?;
    let run = fs::canonicalize(run)?;
    let generated = run.join("oracle.c");
    if minimal {
        fs::write(
            &generated,
            "#include <stdlib.h>\n#include \"sm_rtl.h\"\n#include \"funcs.h\"\nbool g_fail;\nextern void EprojPreInstr_CeresElevatorPad(uint16);\nint main(int argc, char **argv) { if (argc != 2) return 2; EprojPreInstr_CeresElevatorPad((uint16)strtoul(argv[1], 0, 0)); return 0; }\n",
        )?;
    } else if full {
        let original = fs::read_to_string(&oracle)?;
        ensure!(original.matches("bool g_fail;").count() == 1, "retained oracle globals changed");
        fs::write(&generated, original.replace("bool g_fail;", "extern bool g_fail;"))?;
    } else {
        fs::copy(&oracle, &generated)?;
    }
    let mut command = [
        "/usr/bin/x86_64-linux-gnu-gcc-9",
        "-O2",
        "-fno-strict-aliasing",
        "-ffunction-sections",
        "-fdata-sections",
        "-iquote",
    ]
    .map(String::from)
    .to_vec();
    command.push(source.join("src").display().to_string());
    if full {
        let mut units = fs::read_dir(source.join("src"))?
            .map(|entry| entry.map(|entry| entry.path()))
            .collect::<std::io::Result<Vec<_>>>()?;
        units.retain(|path| {
            path.extension().is_some_and(|value| value == "c")
                && path.file_stem().is_some_and(|value| {
                    value.to_string_lossy().starts_with("sm_")
                        || ["spc_player", "tracing", "util"].contains(&value.to_string_lossy().as_ref())
                })
        });
        for entry in fs::read_dir(source.join("src/snes"))? {
            let path = entry?.path();
            if path.extension().is_some_and(|value| value == "c") {
                units.push(path);
            }
        }
        units.sort();
        command.extend(units.iter().map(|path| path.display().to_string()));
        if mode == "full-host" {
            let host = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../tools/decomp/headless.c");
            let wrapper = run.join("host-support.c");
            fs::write(&wrapper, format!("#define main retained_main\n#include {host:?}\n"))?;
            command.push(wrapper.display().to_string());
        }
        command.push("-lm".into());
    } else {
        for unit in ["sm_86.c", "sm_rtl.c", "sm_90.c"] {
            command.push(source.join("src").join(unit).display().to_string());
        }
    }
    command.extend([
        generated.display().to_string(),
        "-Wl,--gc-sections".into(),
        "-o".into(),
        run.join("c").display().to_string(),
    ]);
    let root = PathBuf::from(std::env::var_os("SM_PIPELINE_ROOT").context("shared pipeline root")?);
    let workers: usize = std::env::var("SM_PIPELINE_WORKERS")?.parse()?;
    ensure!(workers == 4, "diagnostic keeps the shared four-worker capacity");
    let result = tool_execution::builds::execute_timeout(&command, &run, "c-build", &root, workers, 60);
    println!("Diagnostic: {}", run.display());
    result.map(|_| ())
}
