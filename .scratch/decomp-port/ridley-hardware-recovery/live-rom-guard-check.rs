use std::{fs, path::Path, process::Command};
fn main() -> Result<(), Box<dyn std::error::Error>> {
    let root = Path::new("/home/jim/sm-decomp/.scratch/decomp-port/ridley-hardware-recovery");
    let template =
        fs::read_to_string("/tmp/sm-ridley-whole-live-source/crates/pipeline/src/live_ridley_subsystem.rs.in")?;
    let start = template.find("fn execution_matches(").ok_or("missing exact execution guard")?;
    let end = template[start..].find("\nconst ROOTS").ok_or("missing guard end")? + start;
    let guard = &template[start..end];
    let pristine = fs::read_to_string("/tmp/sm-ridley-whole-live-probe-9000-r5/families.rs")?;
    if !pristine.contains("if unsafe{std::slice::from_raw_parts(rom,rom_len)}!=EXECUTION{return -1;}") {
        return Err("r5 strict guard changed".into());
    }
    let original = fs::read("/tmp/sm-ridley-whole-live-probe-9000-r5/ridley-execution.rom")?;
    let native = fs::read("/tmp/sm-ridley-whole-live-probe-9000-r5/mixed-0/ridley-error-runtime.rom")?;
    let differences = original
        .iter()
        .zip(&native)
        .enumerate()
        .filter(|(_, (a, b))| a != b)
        .map(|(i, (a, b))| (i, *a, *b))
        .collect::<Vec<_>>();
    if differences != vec![(0x2965a, 0x8f, 0x8b)] || original.len() != native.len() {
        return Err("native retained symptom differs".into());
    }
    fs::write(
        root.join("live-rom-guard-pristine-red.log"),
        format!(
            "pristine r5 strict runtime guard returns -1 before support: expected guard acceptance fails on native image; exact differences {differences:?}\n"
        ),
    )?;
    let red_source = "const EXECUTION:&[u8]=include_bytes!(\"/tmp/sm-ridley-whole-live-probe-9000-r5/ridley-execution.rom\");fn main(){let actual=std::fs::read(\"/tmp/sm-ridley-whole-live-probe-9000-r5/mixed-0/ridley-error-runtime.rom\").unwrap();let status=if actual.as_slice()!=EXECUTION{-1}else{0};assert_eq!(status,0,\"pristine exact retained native patch falsely rejected before gameplay support\");}\n";
    fs::write(root.join("live-rom-guard-pristine-red.rs"), red_source)?;
    let compile = Command::new("rustc")
        .args(["--edition=2024", "-O"])
        .arg(root.join("live-rom-guard-pristine-red.rs"))
        .arg("-o")
        .arg(root.join("live-rom-guard-pristine-red"))
        .output()?;
    if !compile.status.success() {
        return Err("pristine guard compile failed".into());
    }
    let red = Command::new(root.join("live-rom-guard-pristine-red")).output()?;
    fs::write(root.join("live-rom-guard-pristine-red.stderr"), &red.stderr)?;
    fs::remove_file(root.join("live-rom-guard-pristine-red"))?;
    if red.status.success() {
        return Err("pristine symptom regression was not red".into());
    }
    let source = format!(
        "const EXECUTION:&[u8]=include_bytes!(\"/tmp/sm-ridley-whole-live-probe-9000-r5/ridley-execution.rom\");\n{guard}\nfn main(){{\n let native=std::fs::read(\"/tmp/sm-ridley-whole-live-probe-9000-r5/mixed-0/ridley-error-runtime.rom\").unwrap();\n assert!(execution_matches(EXECUTION));assert!(execution_matches(&native));\n for offset in [0,0x29659,0x2965b,0x130000,EXECUTION.len()-1]{{let mut changed=native.clone();changed[offset]^=1;assert!(!execution_matches(&changed),\"accepted unknown mutation at {{offset:x}}\");}}\n let mut operand=native.clone();operand[0x2965a]=0x8a;assert!(!execution_matches(&operand));\n assert!(!execution_matches(&native[..native.len()-1]));\n println!(\"exact current guard accepts retained native bugfix byte and refuses all unrelated/operand/length mutations\");\n}}\n"
    );
    fs::write(root.join("live-rom-guard-probe.rs"), source)?;
    let output = Command::new("rustc")
        .args(["--edition=2024", "-O"])
        .arg(root.join("live-rom-guard-probe.rs"))
        .arg("-o")
        .arg(root.join("live-rom-guard-probe"))
        .output()?;
    fs::write(root.join("live-rom-guard-build.stderr"), &output.stderr)?;
    if !output.status.success() {
        return Err(String::from_utf8_lossy(&output.stderr).to_string().into());
    }
    let output = Command::new(root.join("live-rom-guard-probe")).output()?;
    fs::write(root.join("live-rom-guard-probe.log"), &output.stdout)?;
    fs::write(root.join("live-rom-guard-probe.stderr"), &output.stderr)?;
    fs::remove_file(root.join("live-rom-guard-probe"))?;
    if !output.status.success() {
        return Err(format!("native guard failed {:?}", output.status.code()).into());
    }
    println!("pristine r5 guard fails exact retained native image as expected");
    print!("{}", String::from_utf8_lossy(&output.stdout));
    Ok(())
}
