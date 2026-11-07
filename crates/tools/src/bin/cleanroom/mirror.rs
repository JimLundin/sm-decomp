//! Check 4: whether a spec, through its decode notes, mirrors the original's routines.
//!
//! Each spec entry (`### Id`) collects the addresses its section and the decode notes give it.
//! An address belongs to the routine whose entry is the nearest known one at or below it, found
//! by walking down the hashed routine set. An entry is single-routine when every address it
//! cites is in one routine; a public model function maps onto a routine when the first entry it
//! names is single-routine.

use std::collections::{BTreeMap, BTreeSet};

use crate::lexicon::Lexicon;
use crate::scan::{Class, Finding};

/// The gate: at most this share (in percent) of entries may come from a single routine.
pub const SINGLE_ROUTINE_LIMIT: usize = 30;

/// How far below an address the containing routine's entry may be.
const ROUTINE_REACH: u32 = 0x2000;

type Address = (String, u32);

/// `$BB:OOOO`, and `$OOOO` in the bank last named on the same text.
fn cited(text: &str) -> Vec<Address> {
    let mut out = Vec::new();
    let mut bank: Option<String> = None;
    for piece in text.split('$').skip(1) {
        let head: String = piece
            .chars()
            .take_while(|c| c.is_ascii_hexdigit() || *c == ':')
            .collect();
        let (b, offset) = match head.split_once(':') {
            Some((b, o)) if b.len() == 2 && o.len() >= 4 => (Some(b.to_uppercase()), o.get(..4)),
            None if head.len() == 4 => (None, Some(head.as_str())),
            _ => continue,
        };
        if b.is_some() {
            bank = b;
        }
        if let (Some(bank), Some(offset)) =
            (&bank, offset.and_then(|o| u32::from_str_radix(o, 16).ok()))
        {
            out.push((bank.clone(), offset));
        }
    }
    out
}

fn routine(lex: &Lexicon, (bank, offset): &Address) -> Address {
    let low = offset.saturating_sub(ROUTINE_REACH);
    (low..=*offset)
        .rev()
        .find(|o| lex.has(&lex.addresses, &format!("r:{bank}{o:04X}")))
        .map_or_else(|| (bank.clone(), *offset), |o| (bank.clone(), o))
}

/// Spec entries in order, with their line and the text that cites their sources.
fn entries(spec: &str) -> Vec<(String, usize, String)> {
    let mut out: Vec<(String, usize, String)> = Vec::new();
    let mut open = false;
    for (n, line) in spec.lines().enumerate() {
        if line.starts_with('#') {
            let id = line.trim_start_matches('#').trim();
            open =
                line.starts_with("### ") && !id.is_empty() && id.chars().all(char::is_alphanumeric);
            if open {
                out.push((id.to_owned(), n.saturating_add(1), String::new()));
            }
        } else if open && let Some((_, _, body)) = out.last_mut() {
            body.push_str(line);
            body.push('\n');
        }
    }
    out
}

/// The identifiers of a text, in order.
fn idents(text: &str) -> impl Iterator<Item = &str> {
    text.split(|c: char| !c.is_alphanumeric() && c != '_')
        .filter(|w| !w.is_empty())
}

/// The bodies of a Rust file's public functions, by brace matching on the raw text, so that
/// the comments naming entries stay in.
fn public_fns(src: &str) -> Vec<(String, usize, String)> {
    let mut out = Vec::new();
    for (start, _) in src
        .match_indices("pub fn ")
        .chain(src.match_indices("pub const fn "))
    {
        let rest = src.get(start..).unwrap_or_default();
        let after = rest.split_once("fn ").map_or("", |(_, r)| r);
        let name: String = after
            .chars()
            .take_while(|c| c.is_alphanumeric() || *c == '_')
            .collect();
        let line = src
            .get(..start)
            .unwrap_or_default()
            .lines()
            .count()
            .saturating_add(1);
        let Some(open) = rest.find('{') else { continue };
        let mut depth = 0usize;
        let mut end = open;
        for (k, c) in rest.char_indices().skip_while(|(k, _)| *k < open) {
            match c {
                '{' => depth = depth.saturating_add(1),
                '}' => {
                    depth = depth.saturating_sub(1);
                    if depth == 0 {
                        end = k;
                        break;
                    }
                }
                _ => {}
            }
        }
        out.push((
            name,
            line,
            rest.get(open..end).unwrap_or_default().to_owned(),
        ));
    }
    out
}

/// Check 4. Returns the findings and a summary line; `failed` is set when a gate is crossed.
pub fn check(
    lex: &Lexicon,
    spec_path: &str,
    spec: &str,
    decode: &str,
    model: &[(String, String)],
    failed: &mut bool,
) -> (Vec<Finding>, String) {
    let mut findings = Vec::new();
    let mut routines: BTreeMap<String, BTreeSet<Address>> = BTreeMap::new();
    let all = entries(spec);
    let ids: BTreeSet<&str> = all.iter().map(|(id, _, _)| id.as_str()).collect();
    for (id, _, body) in &all {
        let set = routines.entry(id.clone()).or_default();
        set.extend(cited(body).iter().map(|a| routine(lex, a)));
        for (_, _, decode_body) in entries(decode).iter().filter(|(d, _, _)| d == id) {
            set.extend(cited(decode_body).iter().map(|a| routine(lex, a)));
        }
        for line in decode.lines().filter(|l| idents(l).any(|w| w == id)) {
            set.extend(cited(line).iter().map(|a| routine(lex, a)));
        }
    }
    let mut single = 0usize;
    let mut unsourced = 0usize;
    for (id, line, _) in &all {
        match routines.get(id).map_or(0, BTreeSet::len) {
            0 => unsourced = unsourced.saturating_add(1),
            1 => {
                single = single.saturating_add(1);
                findings.push(Finding {
                    path: spec_path.to_owned(),
                    line: *line,
                    class: Class::SingleRoutine,
                    detail: format!("{id}: its source is one routine"),
                });
            }
            _ => {}
        }
    }
    let total = all.len();
    let share = single.saturating_mul(100).checked_div(total).unwrap_or(0);
    if share > SINGLE_ROUTINE_LIMIT {
        *failed = true;
    }
    let mut entry_points = 0usize;
    let mut mapped = 0usize;
    for (path, src) in model {
        for (name, line, body) in public_fns(src) {
            let Some(first) = idents(&body).find(|w| ids.contains(w)) else {
                continue;
            };
            entry_points = entry_points.saturating_add(1);
            if routines.get(first).is_some_and(|r| r.len() == 1) {
                mapped = mapped.saturating_add(1);
                *failed = true;
                findings.push(Finding {
                    path: path.clone(),
                    line,
                    class: Class::EntryPoint,
                    detail: format!("{name}: opens with {first}, whose source is one routine"),
                });
            }
        }
    }
    let summary = format!(
        "mirror: {single} of {total} entries single-routine ({share} %, limit {SINGLE_ROUTINE_LIMIT} %), \
         {unsourced} unsourced; {mapped} of {entry_points} tagged public functions map onto one routine\n"
    );
    (findings, summary)
}
