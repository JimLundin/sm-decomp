//! Keep maintained tooling native. External sources and immutable evidence are excluded.
use std::{fs, path::Path};
fn inspect(path: &Path, root: &Path, violations: &mut Vec<String>) {
    if path.is_symlink() {
        return;
    }
    if path.is_dir() {
        if [".git", "deps", "out", "target", ".scratch", ".claude", ".codex", ".agents"]
            .contains(&path.file_name().unwrap().to_str().unwrap_or(""))
        {
            return;
        }
        for entry in fs::read_dir(path).unwrap() {
            inspect(&entry.unwrap().path(), root, violations);
        }
    } else {
        let relative = path.strip_prefix(root).unwrap().to_string_lossy();
        if path.extension().is_some_and(|e| e == "py" || e == "pyc") {
            violations.push(relative.into_owned());
            return;
        }
        if relative == "crates/pipeline/tests/no_python.rs" {
            return;
        }
        if (path.file_name().is_some_and(|n| n == "justfile")
            || path.extension().is_some_and(|e| ["rs", "sh", "yml", "yaml"].contains(&e.to_str().unwrap_or(""))))
            && let Ok(text) = fs::read_to_string(path)
        {
            for (index, line) in text.lines().enumerate() {
                let line = line.trim();
                if line.starts_with("//") || line.starts_with('#') {
                    continue;
                }
                if line.contains("python3")
                    || line.contains("Command::new(\"python\")")
                    || line.contains("/usr/bin/env python")
                {
                    violations.push(format!("{relative}:{}: Python runtime invocation", index + 1));
                }
            }
        }
    }
}
#[test]
fn project_sources_recipes_and_ci_have_no_python_dependency() {
    let root = Path::new(env!("CARGO_MANIFEST_DIR")).parent().unwrap().parent().unwrap();
    let mut violations = Vec::new();
    inspect(root, root, &mut violations);
    assert!(violations.is_empty(), "maintained Python sources/runtime calls remain:\n{}", violations.join("\n"));
}

#[test]
fn maintained_configuration_has_no_json_files() {
    fn configurations(path: &Path, root: &Path, violations: &mut Vec<String>) {
        if path.is_symlink() {
            return;
        }
        if path.is_dir() {
            if path.file_name().is_some_and(|n| n == "target") {
                return;
            }
            for entry in fs::read_dir(path).unwrap() {
                configurations(&entry.unwrap().path(), root, violations);
            }
        } else if path.extension().is_some_and(|e| e == "json" || e == "jsonl") {
            violations.push(path.strip_prefix(root).unwrap().display().to_string());
        }
    }
    let root = Path::new(env!("CARGO_MANIFEST_DIR")).parent().unwrap().parent().unwrap();
    let mut violations = Vec::new();
    for directory in ["tools", "movies", "docs", ".github"] {
        configurations(&root.join(directory), root, &mut violations);
    }
    assert!(violations.is_empty(), "maintained JSON configuration remains:\n{}", violations.join("\n"));
}
