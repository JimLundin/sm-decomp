use std::{fs, path::{Path, PathBuf}, process::Command};
const OLD: &str = "/home/jim/sm-decomp/.scratch/decomp-parallel/preparation/readiness-final-20261013/current-root-20261013/ledger-directory-fix-20261013";
const NEW: &str = "/home/jim/sm-decomp/.scratch/decomp-parallel/preparation/readiness-final-20261013/current-root-20261013/ledger-directory-fix-20261013/private-dev-fix-20261013";
const ENGINE: &str = "/home/jim/sm-decomp/.scratch/decomp-parallel/drill-retry/bin/sm-pipeline";
fn hash(path: impl AsRef<Path>) -> String { let r=Command::new("sha256sum").arg(path.as_ref()).output().unwrap(); assert!(r.status.success()); String::from_utf8(r.stdout).unwrap().split_whitespace().next().unwrap().into() }
fn write(name: &str, value: impl AsRef<[u8]>) {fs::write(Path::new(NEW).join(name), value).unwrap();}
fn run(program: &str, args: &[&str], name: &str) {let r=Command::new(program).args(args).output().unwrap();write(&format!("{name}.stdout"),r.stdout);write(&format!("{name}.stderr"),r.stderr);assert!(r.status.success());}
fn visit(path: &Path, files: &mut Vec<PathBuf>) {for e in fs::read_dir(path).unwrap() {let e=e.unwrap();if ["target","package","artifacts.sha256"].iter().any(|n|e.file_name()==*n){continue;} let p=e.path();if p.is_dir(){visit(&p,files)}else{files.push(p);}}}
fn main() {
    assert!(!Path::new(NEW).join("artifacts.sha256").exists());
    assert_eq!(hash(format!("{OLD}/artifacts.sha256")),"52983eb0f05b0eed0a52215b473b6438ebedb73f48444b03130d5330b50a4e26");
    run("sha256sum",&["-c",&format!("{OLD}/artifacts.sha256")],"predecessor-authentication");
    let source=fs::read_to_string(format!("{OLD}/main.rs")).unwrap();
    let marker="command.args([\"--unshare-user\", \"--die-with-parent\", \"--ro-bind\", \"/\", \"/\", \"--tmpfs\", PARENT]);";
    assert_eq!(source.matches(marker).count(),1);
    write("main.rs",source.replace(marker,"command.args([\"--unshare-user\", \"--die-with-parent\", \"--ro-bind\", \"/\", \"/\", \"--dev\", \"/dev\", \"--tmpfs\", PARENT]);").replace(&format!("const READINESS: &str = \"{OLD}\";"),&format!("const READINESS: &str = \"{NEW}\";")));
    for n in ["Cargo.toml","Cargo.lock","prepare-request.toml"]{fs::copy(Path::new(OLD).join(n),Path::new(NEW).join(n)).unwrap();}
    let manifest=format!("{NEW}/Cargo.toml");let target=format!("{NEW}/target");
    run("cargo",&["build","--locked","--offline","--manifest-path",&manifest,"--target-dir",&target],"build");
    fs::create_dir(Path::new(NEW).join("bin")).unwrap();fs::copy(Path::new(NEW).join("target/debug/decomp-readiness"),Path::new(NEW).join("bin/decomp-readiness")).unwrap();
    write("helper-build.toml",format!("schema = \"decomp-readiness-helper-build-v1\"\nsource_sha256 = \"{}\"\nbinary_sha256 = \"{}\"\nactual_caller_required = \"01a13559-e517-72d1-88b5-e554fed61da4\"\nnamespace_validation_run = false\nlimitations = [\"Retained exact strace proves predecessor failed openat /dev/null with EACCES before Git/lock/policy; helper adds namespace dev filesystem only\", \"Same real root identity, sealed preparation observations, frozen source/engine/drill/proof\"]\n[[evidence]]\npath = \"{OLD}/prepare-permission-trace/strace.txt\"\nsha256 = \"{}\"\n",hash(format!("{NEW}/main.rs")),hash(format!("{NEW}/bin/decomp-readiness")),hash(format!("{OLD}/prepare-permission-trace/strace.txt"))));
    for (n,d) in [("cleanup1",false),("cleanup2",true)]{let mut a=vec!["cleanup","--root",NEW];if d{a.push("--deduplicate");}a.push("--apply");run(ENGINE,&a,n);}
    let r=Command::new("du").args(["-sb",&target]).output().unwrap();let bytes=String::from_utf8(r.stdout).unwrap().split_whitespace().next().unwrap().to_owned();
    run("cargo",&["clean","--manifest-path",&manifest,"--target-dir",&target],"cargo-clean");
    let r=Command::new("df").args(["-B1","--output=avail",NEW]).output().unwrap();let avail=String::from_utf8(r.stdout).unwrap().split_whitespace().last().unwrap().to_owned();
    write("cleanup.toml",format!("schema = \"decomp-readiness-package-cleanup-v1\"\nstatus = \"complete\"\nprivate_target_logical_bytes_before_clean = {bytes}\navailable_filesystem_bytes = {avail}\nhelper_binary_retained = true\nnamespace_validation_run = false\n"));
    let mut files=Vec::new();visit(Path::new(NEW),&mut files);files.sort();write("artifacts.sha256",files.iter().map(|p|format!("{}  {}\n",hash(p),p.display())).collect::<String>());
}
