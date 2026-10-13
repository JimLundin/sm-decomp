//! Reviewed source correspondence scanner. This measures body presence, never
//! equivalence or gameplay acceptance. Historical snapshots use the same rules.
use super::*;

#[derive(Clone, Debug, PartialEq, Eq)]
struct Token {
    value: String,
    string: bool,
}

fn tokens(source: &str) -> Vec<Token> {
    let bytes = source.as_bytes();
    let mut result = Vec::new();
    let mut i = 0;
    while i < bytes.len() {
        if bytes[i].is_ascii_whitespace() {
            i += 1;
        } else if bytes[i..].starts_with(b"//") {
            while i < bytes.len() && bytes[i] != b'\n' {
                i += 1;
            }
        } else if bytes[i..].starts_with(b"/*") {
            i += 2;
            let mut depth = 1;
            while i < bytes.len() && depth > 0 {
                if bytes[i..].starts_with(b"/*") {
                    depth += 1;
                    i += 2;
                } else if bytes[i..].starts_with(b"*/") {
                    depth -= 1;
                    i += 2;
                } else {
                    i += 1;
                }
            }
        } else if bytes[i] == b'"' {
            i += 1;
            let start = i;
            while i < bytes.len() && bytes[i] != b'"' {
                i += if bytes[i] == b'\\' && i + 1 < bytes.len() { 2 } else { 1 };
            }
            result.push(Token { value: source[start..i].into(), string: true });
            i = (i + 1).min(bytes.len());
        } else {
            let start = i;
            if bytes[i].is_ascii_alphanumeric() || bytes[i] == b'_' {
                i += 1;
                while i < bytes.len() && (bytes[i].is_ascii_alphanumeric() || bytes[i] == b'_') {
                    i += 1;
                }
            } else if bytes[i..].starts_with(b"=>") {
                i += 2;
            } else {
                // Advance by a complete UTF-8 scalar; comments/string contents
                // can contain Unicode without splitting source boundaries.
                i += source[i..].chars().next().unwrap().len_utf8();
            }
            result.push(Token { value: source[start..i].into(), string: false });
        }
    }
    result
}

fn functions(tokens: &[Token]) -> BTreeMap<String, &[Token]> {
    let mut result = BTreeMap::new();
    let mut i = 0;
    while i + 1 < tokens.len() {
        if tokens[i].value == "fn" && !tokens[i].string {
            let name = tokens[i + 1].value.clone();
            let mut start = i + 2;
            while start < tokens.len() && !matches!(tokens[start].value.as_str(), "{" | ";") {
                start += 1;
            }
            if start < tokens.len() && tokens[start].value == "{" {
                let mut end = start + 1;
                let mut depth = 1;
                while end < tokens.len() && depth > 0 {
                    if !tokens[end].string {
                        match tokens[end].value.as_str() {
                            "{" => depth += 1,
                            "}" => depth -= 1,
                            _ => {}
                        }
                    }
                    end += 1;
                }
                result.insert(name, &tokens[start..end]);
                i = end;
                continue;
            }
        }
        i += 1;
    }
    result
}

fn arms<'a>(body: &'a [Token], selector: Option<&str>) -> Vec<&'a [Token]> {
    let mut result = Vec::new();
    for i in 0..body.len() {
        if body[i].string || body[i].value != "match" {
            continue;
        }
        let mut start = i + 1;
        while start < body.len() && (body[start].string || body[start].value != "{") {
            start += 1;
        }
        if start == body.len() || selector.is_some_and(|s| body[i + 1].value != s) {
            continue;
        }
        let mut depth = 1;
        let mut arm = start + 1;
        let mut k = arm;
        while k < body.len() && depth > 0 {
            if !body[k].string {
                match body[k].value.as_str() {
                    "{" => depth += 1,
                    "}" => {
                        depth -= 1;
                        if depth == 1 {
                            arm = k + 1;
                        }
                    }
                    "," if depth == 1 => arm = k + 1,
                    "=>" if depth == 1 => {
                        result.push(&body[arm..k]);
                        arm = k + 1;
                    }
                    _ => {}
                }
            }
            k += 1;
        }
    }
    result
}

fn number(token: &Token) -> Option<u32> {
    if token.string {
        return None;
    }
    let value = token.value.replace('_', "");
    value.strip_prefix("0x").and_then(|v| u32::from_str_radix(v, 16).ok()).or_else(|| value.parse().ok())
}
fn has_number(body: &[Token], value: u32) -> bool {
    body.iter().any(|t| number(t) == Some(value))
}

#[derive(Clone)]
struct Alias {
    module: String,
    function: String,
    symbol: String,
}
#[derive(Clone)]
struct Numeric {
    module: String,
    function: String,
    bank: u32,
    selector: String,
    equality: bool,
}

pub(super) struct Scanner {
    native: BTreeSet<String>,
    address: BTreeMap<u32, Vec<String>>,
    aliases: Vec<Alias>,
    numeric: Vec<Numeric>,
    forwards: BTreeSet<(String, String)>,
}

pub(super) const INPUTS: [&str; 4] = [
    "tools/progress/source-aliases.tsv",
    "tools/progress/source-numeric-scopes.tsv",
    "tools/progress/source-forwarders.tsv",
    "crates/pipeline/src/progress/source.rs",
];

impl Scanner {
    pub(super) fn new(root: &Path, routines: &[Routine]) -> Result<Self> {
        let read = |path: &str| -> Result<Vec<Vec<String>>> {
            Ok(bounded_text(&root.join(path), MAX_UPDATE)?
                .lines()
                .filter(|line| !line.trim().is_empty() && !line.starts_with('#'))
                .map(|line| line.split('\t').map(str::to_owned).collect())
                .collect())
        };
        let mut address = BTreeMap::<u32, Vec<String>>::new();
        for r in routines {
            if let Some(v) =
                r.original_address.as_deref().and_then(|a| u32::from_str_radix(a.trim_start_matches("0x"), 16).ok())
            {
                address.entry(v).or_default().push(r.symbol.clone());
            }
        }
        let mut aliases = Vec::new();
        for fields in read(INPUTS[0])? {
            ensure!(fields.len() >= 3, "invalid reviewed source alias");
            ensure!(routines.iter().any(|r| r.symbol == fields[2]), "unknown reviewed alias {}", fields[2]);
            aliases.push(Alias { module: fields[0].clone(), function: fields[1].clone(), symbol: fields[2].clone() });
        }
        let mut numeric = Vec::new();
        for fields in read(INPUTS[1])? {
            ensure!(fields.len() >= 4, "invalid reviewed numeric scope");
            numeric.push(Numeric {
                module: fields[0].clone(),
                function: fields[1].clone(),
                bank: u32::from_str_radix(&fields[2], 16)?,
                selector: fields[3].clone(),
                equality: fields.get(4).is_some_and(|s| s == "if"),
            });
        }
        let mut forwards = BTreeSet::new();
        for fields in read(INPUTS[2])? {
            ensure!(fields.len() >= 3, "invalid reviewed forwarder");
            forwards.insert((fields[0].clone(), fields[1].clone()));
        }
        Ok(Self { native: routines.iter().map(|r| r.symbol.clone()).collect(), address, aliases, numeric, forwards })
    }

    pub(super) fn scan(&self, path: &str, source: &str) -> BTreeSet<String> {
        let module = Path::new(path).file_stem().unwrap().to_string_lossy();
        let tokens = tokens(source);
        let functions = functions(&tokens);
        let mut result = BTreeSet::new();
        for (name, body) in &functions {
            let execution = name.starts_with("dispatch")
                || matches!(
                    name.as_str(),
                    "execute" | "pure" | "render" | "select" | "instruction" | "callback" | "door_scroll_writes"
                )
                || (module == "camera_scroll" && name == "prepare");
            if execution {
                for arm in arms(body, None) {
                    for t in arm.iter().filter(|t| t.string && self.native.contains(&t.value)) {
                        result.insert(t.value.clone());
                    }
                }
            }
        }
        let find = |name: &str| -> Option<&[Token]> {
            functions.get(name).copied().or_else(|| {
                // Renames in retained source history, not new implementation.
                let old = match name {
                    "instruction_context" => "instruction",
                    "preinstruction_context" => "preinstruction",
                    "run_all_context" => "run_all",
                    "spawn_context" => "spawn",
                    _ => return None,
                };
                functions.get(old).copied()
            })
        };
        for rule in self.numeric.iter().filter(|r| r.module == module) {
            let Some(body) = find(&rule.function) else {
                continue;
            };
            let mut numbers = BTreeSet::new();
            for arm in arms(body, Some(&rule.selector)) {
                for token in arm.iter().take_while(|t| t.value != "if") {
                    if let Some(v) = number(token).filter(|v| (0x8000..=0xffff).contains(v)) {
                        numbers.insert(v);
                    }
                }
            }
            if rule.equality {
                for i in 0..body.len().saturating_sub(3) {
                    if body[i].value == rule.selector && body[i + 1].value == "=" && body[i + 2].value == "=" {
                        if let Some(n) = number(&body[i + 3]) {
                            numbers.insert(n);
                        }
                    }
                }
                for i in 3..body.len() {
                    if body[i].value == rule.selector
                        && body[i - 3].value == "matches"
                        && body[i - 2].value == "!"
                        && body[i - 1].value == "("
                    {
                        for t in body[i + 1..].iter().take_while(|t| t.value != ")") {
                            if let Some(v) = number(t) {
                                numbers.insert(v);
                            }
                        }
                    }
                }
            }
            if module == "plm_instructions"
                && rule.function == "instruction"
                && has_number(body, 0x8c07)
                && has_number(body, 0x8c85)
            {
                numbers.extend((0x8c07..=0x8c85).step_by(9));
            }
            for v in numbers {
                if let Some(symbols) = self.address.get(&((rule.bank << 16) | v)) {
                    result.extend(symbols.iter().cloned());
                }
            }
        }
        for rule in self.aliases.iter().filter(|r| r.module == module) {
            let Some(body) = find(&rule.function) else {
                continue;
            };
            // Multi-branch helper families must contain the specific branch;
            // the existence of a growing dispatcher is insufficient evidence.
            let supported = if module == "enemy_projectile_instructions" && rule.function == "pickup" {
                body.len() > 50 && body.iter().any(|t| t.value == "0x9c2")
            } else if module == "collision" && rule.function == "handler" {
                rule.symbol
                    .split('_')
                    .nth(2)
                    .and_then(|s| u32::from_str_radix(s, 16).ok())
                    .is_some_and(|n| has_number(body, n))
            } else if module == "enemy_jump" && rule.function == "state" {
                rule.symbol.rsplit('_').next().and_then(|s| s.parse::<u32>().ok()).is_some_and(|n| has_number(body, n))
            } else if module == "samus_control" && rule.function == "animation_command" {
                let code = rule
                    .symbol
                    .split('_')
                    .nth(if rule.symbol.starts_with("UNUSED_") { 3 } else { 2 })
                    .and_then(|s| s.parse::<u32>().ok());
                code.is_none_or(|n| has_number(body, n))
            } else {
                body.len() > 4
            };
            if supported {
                result.insert(rule.symbol.clone());
            }
        }
        for symbol in &self.native {
            let dynamic = if module == "audio" {
                symbol.strip_prefix("QueueSfx").and_then(|s| s.split_once('_')).is_some_and(|(prefix, suffix)| {
                    tokens.iter().any(|t| t.string && t.value == format!("QueueSfx{prefix}"))
                        && tokens.iter().any(|t| t.string && t.value == suffix)
                })
            } else if module == "enemy_rio" {
                ["NorfairRio_", "LowerNorfairRio_"].iter().any(|prefix| {
                    symbol.strip_prefix(prefix).is_some_and(|suffix| {
                        tokens.iter().any(|t| t.string && t.value == *prefix)
                            && (matches!(suffix, "Init" | "Main")
                                || suffix
                                    .strip_prefix("Func_")
                                    .and_then(|s| s.parse::<u8>().ok())
                                    .is_some_and(|n| (1..=7).contains(&n)))
                    })
                })
            } else if module == "projectile_flight" {
                [
                    "ProjPreInstr_Dir",
                    "ProjPreInstr_Missile_Func",
                    "ProjPreInstr_WavePlasmaEtc_",
                    "ProjPreInstr_SuperMissile_Func",
                    "CheckBeamCollByDir_",
                    "WaveBeam_CheckColl_",
                ]
                .iter()
                .any(|prefix| symbol.starts_with(prefix) && tokens.iter().any(|t| t.string && t.value == *prefix))
                    && ["0459", "1368", "27", "_2", "_7"].iter().any(|suffix| symbol.ends_with(suffix))
            } else {
                false
            };
            if dynamic {
                result.insert(symbol.clone());
            }
        }
        result.retain(|symbol| !self.forwards.contains(&(module.to_string(), symbol.clone())));
        result
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn scanner_requires_executable_selectors_and_ignores_comments_and_support_lists() {
        let scanner = Scanner {
            native: ["First".into(), "Second".into()].into(),
            address: BTreeMap::new(),
            aliases: Vec::new(),
            numeric: Vec::new(),
            forwards: BTreeSet::new(),
        };
        let source = r#"
            const SUPPORT: &[&str] = &["Second"];
            // fn dispatch() { match symbol { "Second" => {} } }
            /* ignored /* nested */ "Second" */
            fn accepts(symbol: &str) { matches!(symbol, "Second") }
            fn dispatch(symbol: &str) { match symbol { "First" => {}, _ => {} } }
        "#;
        assert_eq!(scanner.scan("example.rs", source), ["First".into()].into());
    }
    #[test]
    fn numeric_branches_do_not_count_unselected_constants_or_forwarded_bodies() {
        let scanner = Scanner {
            native: ["First".into(), "Second".into()].into(),
            address: [(0x868001, vec!["First".into()]), (0x868002, vec!["Second".into()])].into(),
            aliases: Vec::new(),
            numeric: vec![Numeric {
                module: "example".into(),
                function: "instruction_context".into(),
                bank: 0x86,
                selector: "opcode".into(),
                equality: false,
            }],
            forwards: [("example".into(), "Second".into())].into(),
        };
        assert_eq!(
            scanner.scan(
                "example.rs",
                "fn instruction(opcode: u16) { match opcode { 0x8001 => { let v = 0x8002; }, _ => {} } }"
            ),
            ["First".into()].into()
        );
    }
}
