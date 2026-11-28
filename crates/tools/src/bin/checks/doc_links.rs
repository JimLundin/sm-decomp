use std::fs;
use std::path::Path;
use std::process::Command;

fn links(text: &str) -> Vec<&str> {
    let mut found = Vec::new();
    for line in text.lines() {
        let mut rest = line;
        while let Some(start) = rest.find("](") {
            let after = rest.get(start.saturating_add(2)..).unwrap_or_default();
            let path = after.find([')', '#', ' ']).unwrap_or(after.len());
            let end = if after.get(path..).is_some_and(|tail| tail.starts_with('#')) {
                after
                    .get(path..)
                    .unwrap_or_default()
                    .find([')', ' '])
                    .map(|anchor| path.saturating_add(anchor))
            } else {
                Some(path)
            };
            match end.filter(|&end| after.get(end..).is_some_and(|tail| tail.starts_with(')'))) {
                Some(end) => {
                    found.push(after.get(..end).unwrap_or_default());
                    rest = after.get(end.saturating_add(1)..).unwrap_or_default();
                }
                None => rest = rest.get(start.saturating_add(1)..).unwrap_or_default(),
            }
        }
    }
    found
}

fn slug(heading: &str) -> String {
    heading
        .to_ascii_lowercase()
        .chars()
        .filter(|c| c.is_ascii_lowercase() || c.is_ascii_digit() || matches!(c, ' ' | '_' | '-'))
        .map(|c| if c == ' ' { '-' } else { c })
        .collect()
}

fn anchors(text: &str) -> Vec<String> {
    text.lines()
        .filter_map(|line| {
            let level = line.bytes().take_while(|&byte| byte == b'#').count();
            (1..=6)
                .contains(&level)
                .then(|| line.get(level..)?.strip_prefix(' '))
                .flatten()
        })
        .map(slug)
        .collect()
}

fn broken(root: &Path, file: &str, link: &str) -> bool {
    let (path, anchor) = link.split_once('#').unwrap_or((link, ""));
    let source = root.join(file);
    let target = if path.is_empty() {
        source
    } else {
        source.parent().unwrap_or(root).join(path)
    };
    if !target.exists() {
        return true;
    }
    if anchor.is_empty() || !target.is_file() {
        return false;
    }
    fs::read_to_string(&target).map_or(true, |text| {
        !anchors(&text).iter().any(|heading| heading == anchor)
    })
}

fn markdown_files(root: &Path) -> Result<Vec<String>, String> {
    let listed = Command::new("git")
        .current_dir(root)
        .args(["ls-files", "-z", "*.md"])
        .output()
        .map_err(|e| format!("git ls-files: {e}"))?;
    if !listed.status.success() {
        return Err(format!(
            "git ls-files: {}",
            String::from_utf8_lossy(&listed.stderr)
        ));
    }
    String::from_utf8(listed.stdout)
        .map_err(|e| format!("git ls-files: {e}"))
        .map(|names| {
            names
                .split('\0')
                .filter(|name| !name.is_empty())
                .map(str::to_owned)
                .collect()
        })
}

pub fn run(root: &Path, files: &[String]) -> Result<Vec<String>, String> {
    let files = if files.is_empty() {
        markdown_files(root)?
    } else {
        files.to_vec()
    };
    let mut findings = Vec::new();
    for file in &files {
        let text = fs::read_to_string(root.join(file)).map_err(|e| format!("{file}: {e}"))?;
        for link in links(&text) {
            if !link.starts_with("http") && !link.starts_with("mailto:") && broken(root, file, link)
            {
                findings.push(format!("{file}: {link}"));
            }
        }
    }
    Ok(findings)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn links_are_read_as_the_relative_target_and_its_anchor() {
        assert_eq!(
            links("[a](b.md) [c](d.md#e-f) [g](https://x) [h](has space.md) [i](#j)"),
            ["b.md", "d.md#e-f", "https://x", "#j"]
        );
    }

    #[test]
    fn a_heading_anchor_drops_punctuation_and_joins_words_with_dashes() {
        assert_eq!(
            anchors("# A1: B's `c`\n####### Seven\n## Two_words - here\n"),
            ["a1-bs-c", "two_words---here"]
        );
    }

    #[test]
    fn a_missing_file_or_heading_is_broken() {
        let root = std::env::temp_dir().join(format!("doc-links-{}", std::process::id()));
        fs::create_dir_all(&root).unwrap();
        fs::write(root.join("a.md"), "# Here\n").unwrap();
        assert!(!broken(&root, "a.md", "#here"));
        assert!(broken(&root, "a.md", "#there"));
        assert!(broken(&root, "a.md", "b.md"));
        fs::remove_dir_all(&root).unwrap();
    }
}
