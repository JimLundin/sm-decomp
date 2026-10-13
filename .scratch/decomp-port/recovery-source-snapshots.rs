use std::{collections::BTreeMap, fs, path::{Path, PathBuf}, process::Command};

fn hash(path: &Path) -> String {
    let output = Command::new("sha256sum").arg(path).output().unwrap();
    assert!(output.status.success(), "cannot hash {}", path.display());
    String::from_utf8(output.stdout).unwrap().split_whitespace().next().unwrap().to_owned()
}
fn quoted(value: &str) -> String { assert!(value.starts_with('"') && value.ends_with('"')); value[1..value.len()-1].to_owned() }
fn table(text: &str, name: &str) -> BTreeMap<String, String> {
    let mut result = BTreeMap::new();
    let mut selected = false;
    for line in text.lines() {
        if line.starts_with('[') { selected = line == format!("[{name}]"); continue; }
        if selected && let Some((key,value)) = line.split_once(" = ") && value.starts_with('"') {
            result.insert(if key.starts_with('"') { quoted(key) } else { key.to_owned() }, quoted(value));
        }
    }
    result
}
fn main() {
    let args: Vec<String> = std::env::args().collect();
    let verdict = PathBuf::from(&args[1]);
    let target = PathBuf::from(&args[2]);
    let repo = PathBuf::from(&args[3]);
    let v = fs::read_to_string(&verdict).unwrap();
    let run = verdict.parent().unwrap();
    let implementation = table(&v, "implementation");
    let manifest = table(&v, "implementation.manifest");
    let mut files = BTreeMap::new();
    let artifacts = table(&v, "artifacts");
    let mut by_hash = BTreeMap::new();
    for (name, digest) in artifacts {
        if name.contains('/') { continue; }
        by_hash.entry(digest).or_insert(run.join(name));
    }
    for map in ["rust_dependencies", "harness_sources"] {
        for (name, digest) in table(&v, &format!("implementation.{map}")) {
            let expected = digest.as_str();
            let destination = if Path::new(&name).is_absolute() { PathBuf::from(&name) } else { target.join(&name) };
            let source = by_hash.get(expected).unwrap_or_else(|| panic!("unsealed dependency {name} {expected}"));
            assert_eq!(hash(source), expected, "retained artifact changed: {}", source.display());
            fs::create_dir_all(destination.parent().unwrap()).unwrap();
            if destination.exists() { assert_eq!(hash(&destination), expected, "restoration collision {}", destination.display()); }
            else { fs::copy(source, &destination).unwrap(); }
            files.insert(destination.display().to_string(), (expected.to_owned(), source.display().to_string()));
        }
    }
    for (destination, expected, sealed) in [
        (implementation["module"].as_str(), implementation["module_sha256"].as_str(), "candidate.rs"),
        (manifest["file"].as_str(), manifest["sha256"].as_str(), "manifest.toml")
    ] {
        let destination = PathBuf::from(destination);
        let source = run.join(sealed);
        assert_eq!(hash(&source), expected);
        fs::create_dir_all(destination.parent().unwrap()).unwrap();
        if destination.exists() { assert_eq!(hash(&destination), expected, "restoration collision {}", destination.display()); }
        else { fs::copy(&source, &destination).unwrap(); }
        files.insert(destination.display().to_string(), (expected.to_owned(), source.display().to_string()));
    }
    let destination = target.join("tools/dependencies.toml");
    fs::create_dir_all(destination.parent().unwrap()).unwrap();
    fs::copy(repo.join("tools/dependencies.toml"), &destination).unwrap();
    files.insert(destination.display().to_string(), (hash(&destination), repo.join("tools/dependencies.toml").display().to_string()));
    fs::create_dir_all(target.join("deps")).unwrap();
    for name in ["snesrev-sm", "downloads"] {
        let link = target.join("deps").join(name);
        if !link.exists() { std::os::unix::fs::symlink(repo.join("deps").join(name), link).unwrap(); }
    }
    let mut text = format!("status = \"recovered-from-sealed-artifacts\"\nverdict = {:?}\nverdict_sha256 = {:?}\nsource_snapshot = {:?}\n", verdict.display().to_string(), hash(&verdict), target.display().to_string());
    for (path, (sha256, source)) in files {
        text.push_str(&format!("\n[[files]]\npath = {path:?}\nsha256 = {sha256:?}\nsealed_source = {source:?}\n"));
    }
    fs::write(target.join("retention.toml"), &text).unwrap();
    print!("{text}");
}
