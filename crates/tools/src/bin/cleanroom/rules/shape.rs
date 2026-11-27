use std::collections::{BTreeMap, BTreeSet};

use super::lex::{Kind, Toks};

pub struct Shape {
    pub file: usize,
    pub tag: &'static str,
    pub detail: String,
}

#[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
enum Role {
    Field,
    Trait,
    Const,
    Impl,
    Return,
}

#[derive(Default)]
struct Struct {
    file: usize,
    line: usize,
    tuple: bool,
    takers: usize,
    taker: String,
    calls: usize,
    others: usize,
    callees: BTreeSet<String>,
    roles: BTreeSet<Role>,
}

pub fn module_of(path: &str) -> String {
    let stem = path.strip_suffix(".rs").unwrap_or(path);
    let stem = ["lib", "main", "mod"]
        .iter()
        .find_map(|leaf| {
            stem.strip_suffix(leaf)
                .filter(|rest| rest.is_empty() || rest.ends_with('/'))
        })
        .map_or(stem, |rest| rest.trim_end_matches('/'));
    if stem.is_empty() {
        "crate".to_owned()
    } else {
        format!("crate::{}", stem.replace('/', "::"))
    }
}

fn parent(module: &str) -> String {
    module
        .rsplit_once("::")
        .map_or_else(|| module.to_owned(), |(up, _)| up.to_owned())
}

pub fn check(toks: Toks<'_>, modules: &[String]) -> Vec<Shape> {
    let skip = test_items(toks);
    let mut out = argument_objects(toks, &skip);
    out.extend(layering(toks, &skip, modules));
    out
}

fn test_items(toks: Toks<'_>) -> Vec<bool> {
    let mut skip = vec![false; toks.len()];
    let mut a = 0usize;
    while a < toks.len() {
        if toks.cfg_test_at(a) {
            let end = toks.after_item(a.saturating_add(7));
            for slot in skip.iter_mut().take(end).skip(a) {
                *slot = true;
            }
            a = end;
        } else {
            a = a.saturating_add(1);
        }
    }
    skip
}

fn skipped(skip: &[bool], i: usize) -> bool {
    skip.get(i).copied().unwrap_or(false)
}

fn scan_to(toks: Toks<'_>, from: usize, stops: &[&str]) -> usize {
    (from..toks.len())
        .find(|k| stops.contains(&toks.text(*k)))
        .unwrap_or(toks.len())
}

fn built(toks: Toks<'_>, at: usize, end: usize) -> bool {
    let before = |n: usize| toks.text(at.wrapping_sub(n));
    let opener = matches!(
        before(1),
        "(" | "{" | "[" | "," | ";" | ":" | "=" | "&" | "!" | "=>" | "return" | "mut" | "else"
    );
    if !opener && !toks.is(at.wrapping_sub(1), Kind::Close) {
        return false;
    }
    if before(1) == "&" && before(2) == "->" {
        return false;
    }
    if before(1) == "mut" && before(2) == "&" && before(3) == "->" {
        return false;
    }
    !matches!(toks.text(end), "=" | "=>" | "|")
}

fn call_of(toks: Toks<'_>, from: usize, end: usize) -> Option<&str> {
    let mut a = from;
    if toks.text(a.wrapping_sub(1)) == "&" {
        a = a.wrapping_sub(1);
    } else if toks.text(a.wrapping_sub(1)) == "mut" && toks.text(a.wrapping_sub(2)) == "&" {
        a = a.wrapping_sub(2);
    }
    if !matches!(toks.text(a.wrapping_sub(1)), "(" | ",") || !matches!(toks.text(end), "," | ")") {
        return None;
    }
    let mut depth = 0usize;
    let mut k = a.checked_sub(1)?;
    loop {
        match toks.kind(k) {
            Some(Kind::Close) => depth = depth.saturating_add(1),
            Some(Kind::Open) if depth == 0 => break,
            Some(Kind::Open) => depth = depth.saturating_sub(1),
            _ => {}
        }
        k = k.checked_sub(1)?;
    }
    let name = toks.text(k.wrapping_sub(1));
    let callable = toks.text(k) == "("
        && toks.is(k.wrapping_sub(1), Kind::Word)
        && name.starts_with(|c: char| c.is_ascii_lowercase() || c == '_')
        && !matches!(
            name,
            "if" | "while" | "match" | "return" | "in" | "for" | "let" | "fn"
        )
        && toks.text(k.wrapping_sub(2)) != "fn";
    callable.then_some(name)
}

fn declared(toks: Toks<'_>, skip: &[bool]) -> BTreeMap<String, Struct> {
    let mut structs = BTreeMap::new();
    for a in 0..toks.len() {
        if skipped(skip, a) || toks.text(a) != "struct" || !toks.is(a.saturating_add(1), Kind::Word)
        {
            continue;
        }
        let mut j = a.saturating_add(2);
        if toks.text(j) == "<" {
            j = toks.after_angles(j);
        }
        let token = toks.get(a);
        structs.insert(
            toks.text(a.saturating_add(1)).to_owned(),
            Struct {
                file: token.map_or(0, |t| t.file),
                line: token.map_or(0, |t| t.line),
                tuple: toks.text(j) == "(",
                ..Struct::default()
            },
        );
    }
    structs
}

fn mark_roles(
    toks: Toks<'_>,
    structs: &mut BTreeMap<String, Struct>,
    from: usize,
    to: usize,
    role: Role,
) {
    for k in from..to {
        if toks.is(k, Kind::Word)
            && let Some(s) = structs.get_mut(toks.text(k))
        {
            s.roles.insert(role);
        }
    }
}

fn signature(toks: Toks<'_>, structs: &mut BTreeMap<String, Struct>, at: usize) {
    let name = toks.text(at.saturating_add(1));
    let mut j = at.saturating_add(2);
    if toks.text(j) == "<" {
        j = toks.after_angles(j);
    }
    if toks.text(j) != "(" {
        return;
    }
    let end = toks.after_group(j);
    let mut seen = BTreeSet::new();
    for k in j.saturating_add(1)..end.saturating_sub(1) {
        let taken = toks.text(k);
        if toks.is(k, Kind::Word)
            && seen.insert(taken)
            && let Some(s) = structs.get_mut(taken)
        {
            s.takers = s.takers.saturating_add(1);
            name.clone_into(&mut s.taker);
        }
    }
    let body = scan_to(toks, end, &["{", ";"]);
    if let Some(arrow) = (end..body).find(|k| toks.text(*k) == "->") {
        let stop = scan_to(toks, arrow, &["{", ";", "where"]);
        mark_roles(toks, structs, arrow.saturating_add(1), stop, Role::Return);
    }
}

fn construction(toks: Toks<'_>, structs: &mut BTreeMap<String, Struct>, at: usize) {
    let word = toks.text(at);
    let Some(tuple) = structs.get(word).map(|s| s.tuple) else {
        return;
    };
    let next = at.saturating_add(1);
    let opener = toks.text(next);
    if !((opener == "{" && !tuple) || (opener == "(" && tuple)) {
        return;
    }
    let end = toks.after_group(next);
    if !built(toks, at, end) {
        return;
    }
    let callee = call_of(toks, at, end).map(str::to_owned);
    if let Some(s) = structs.get_mut(word) {
        match callee {
            Some(f) => {
                s.calls = s.calls.saturating_add(1);
                s.callees.insert(f);
            }
            None => s.others = s.others.saturating_add(1),
        }
    }
}

fn argument_objects(toks: Toks<'_>, skip: &[bool]) -> Vec<Shape> {
    let mut structs = declared(toks, skip);
    let mut a = 0usize;
    while a < toks.len() {
        let next = a.saturating_add(1);
        let word = toks.text(a);
        if skipped(skip, a) || !toks.is(a, Kind::Word) {
            a = next;
            continue;
        }
        let named = toks.is(next, Kind::Word);
        let item = match word {
            "struct" | "enum" | "union" | "type"
                if named && toks.text(a.wrapping_sub(1)) != "::" =>
            {
                Some(Role::Field)
            }
            "trait" if named => Some(Role::Trait),
            _ => None,
        };
        if let Some(role) = item {
            let end = toks.after_item(a);
            mark_roles(toks, &mut structs, a.saturating_add(2), end, role);
            a = end;
            continue;
        }
        if matches!(word, "const" | "static") && named && toks.text(next) != "fn" {
            let end = scan_to(toks, next, &["=", ";"]);
            mark_roles(toks, &mut structs, next, end, Role::Const);
        } else if word == "impl" {
            let end = scan_to(toks, next, &["{", ";"]);
            mark_roles(toks, &mut structs, next, end, Role::Impl);
        } else if word == "fn" && named {
            signature(toks, &mut structs, a);
        } else {
            construction(toks, &mut structs, a);
        }
        a = next;
    }
    structs
        .into_iter()
        .filter(|(_, s)| {
            s.takers == 1
                && s.calls >= 1
                && s.others == 0
                && s.roles.is_empty()
                && s.callees.len() == 1
                && s.callees.contains(&s.taker)
        })
        .map(|(name, s)| Shape {
            file: s.file,
            tag: "argument-object",
            detail: format!(
                "line {}: struct {name} is taken only by fn {} and built only in its arguments ({} calls): pass the values, or give it a role",
                s.line, s.taker, s.calls
            ),
        })
        .collect()
}

struct Edge {
    from: String,
    to: String,
    file: usize,
    line: usize,
}

fn references(
    toks: Toks<'_>,
    modules: &BTreeSet<String>,
    at: usize,
    current: &str,
    from: &str,
    edges: &mut Vec<Edge>,
) {
    let start = at;
    let mut k = at;
    let mut cur = current.to_owned();
    while k < toks.len() && toks.is(k, Kind::Word) {
        match toks.text(k) {
            "super" => cur = parent(&cur),
            "crate" => "crate".clone_into(&mut cur),
            "self" => {}
            name => {
                let child = format!("{cur}::{name}");
                if !modules.contains(&child) {
                    break;
                }
                cur = child;
            }
        }
        if toks.text(k.saturating_add(1)) != "::" {
            break;
        }
        k = k.saturating_add(2);
    }
    if toks.text(k) == "{" {
        let end = toks.after_group(k);
        let mut depth = 0usize;
        for j in k.saturating_add(1)..end.saturating_sub(1) {
            if depth == 0 && (j == k.saturating_add(1) || toks.text(j.wrapping_sub(1)) == ",") {
                references(toks, modules, j, &cur, from, edges);
            }
            match toks.kind(j) {
                Some(Kind::Open) => depth = depth.saturating_add(1),
                Some(Kind::Close) => depth = depth.saturating_sub(1),
                _ => {}
            }
        }
        return;
    }
    if cur != from && !edges.iter().any(|e| e.from == from && e.to == cur) {
        let token = toks.get(start);
        edges.push(Edge {
            from: from.to_owned(),
            to: cur,
            file: token.map_or(0, |t| t.file),
            line: token.map_or(0, |t| t.line),
        });
    }
}

fn layering(toks: Toks<'_>, skip: &[bool], file_modules: &[String]) -> Vec<Shape> {
    let modules: BTreeSet<String> = file_modules.iter().cloned().collect();
    let mut edges = Vec::new();
    for a in 0..toks.len() {
        let word = toks.text(a);
        if skipped(skip, a)
            || !toks.is(a, Kind::Word)
            || toks.text(a.saturating_add(1)) != "::"
            || matches!(toks.text(a.wrapping_sub(1)), "::" | ".")
        {
            continue;
        }
        let Some(from) = toks.get(a).and_then(|t| file_modules.get(t.file)) else {
            continue;
        };
        if matches!(word, "crate" | "super" | "self")
            || modules.contains(&format!("{from}::{word}"))
        {
            references(toks, &modules, a, from, from, &mut edges);
        }
    }
    let reaches = |from: &str, to: &str| {
        let mut seen = BTreeSet::from([from.to_owned()]);
        let mut frontier = vec![from.to_owned()];
        while let Some(m) = frontier.pop() {
            for e in edges.iter().filter(|e| e.from == m) {
                if e.to == to {
                    return true;
                }
                if seen.insert(e.to.clone()) {
                    frontier.push(e.to.clone());
                }
            }
        }
        false
    };
    edges
        .iter()
        .filter(|e| reaches(&e.to, &e.from))
        .map(|e| Shape {
            file: e.file,
            tag: "layering",
            detail: format!(
                "line {}: {} names {}, and {} reaches {}: modules import in one direction only",
                e.line, e.from, e.to, e.to, e.from
            ),
        })
        .collect()
}

#[cfg(test)]
mod tests {
    use super::module_of;

    #[test]
    fn a_file_is_its_module() {
        assert_eq!(module_of("lib.rs"), "crate");
        assert_eq!(module_of("a/mod.rs"), "crate::a");
        assert_eq!(module_of("a/b.rs"), "crate::a::b");
        assert_eq!(module_of("library.rs"), "crate::library");
    }
}
