//! Spec units and the size budget they set, for `tools spec-units` (MODEL §1 decision 7, §5's
//! "Rules are arms").
//!
//! A domain's budget is k weighted tokens per spec unit, per area; the old code doesn't bound it
//! (Jim, 2026-11-10). The spec holds the rules and the counter counts its units; the budget is the evaluation's, so the spec never states it (MODEL §1 decision 10). k,
//! the areas and their old code are in the `## Budget` section of the domain's decode notes
//! (`docs/behaviours/<domain>.decode.md`), k as "k = N weighted tokens per spec unit" (MODEL §1
//! decision 7: computed once, then frozen). A spec whose notes state no k still has its units
//! counted; its budget is "-" and the findings say so. The units are counted from the spec's
//! structure, so rewording or explaining a rule moves nothing:
//! - an **arm** is a bullet holding `→` (outside backticks): `<conditions> → <outcome> [vectors]`,
//!   under a `### Entry` heading. It is one unit. Arms of one entry with the same outcome count
//!   once. An arm counts only if it cites vectors and each of them exists in the vector file. A
//!   recorded frame, `row:frame` or `row:first-last`, exists when a vector's `source` is that
//!   frame or, for a range, a frame inside it; another vector from the same row doesn't make it
//!   exist. The labels `[original only]` and `[record only: row:frame, ...]` say an arm has no
//!   vector: it counts 0 and is listed. An arm whose outcome is only a reference to other entries
//!   (`` as `Landing` ``) is a cross-reference and counts nothing;
//! - a **cell term** is one term a `Cells:` line names, a third of a unit, counted once per spec;
//! - an **operator** in a backticked formula is a third of a unit. A formula spaces its operators
//!   (`x + d − rx`): a span with no space is a name, a value or a path and counts nothing. A `+`,
//!   `-` or `−` with no operand before it is a sign; an ASCII `-` between two letters or digits
//!   is part of a name (`C-wall`).
//!
//! **Where formulas count.** Only in an entry's counted arms (an arm merged into an earlier one
//! with the same outcome is part of it, so its formulas count) and its **definition bullets**:
//! bullets with no `→` that aren't fields (`kind:`, `summary:`, `params:`, `switch:`,
//! `checked:`) and don't start with an explanation word (`So`, `Thus`, `Hence`, `As`, `E.g.`,
//! `For example`, `That is`, `I.e.`). An arm that counts 0, an explanation bullet and a
//! paragraph count no operator. Within an entry:
//! - a formula **mirrored** to the other axis or side counts once. Two formulas are mirrors when
//!   one becomes the other by swapping `x` and `y` in every name of up to three letters (`rx`,
//!   `ery`), by swapping `+` with `−`, `<` with `>`, `≤` with `≥`, `min` with `max`, `left` with
//!   `right`, `top` with `bottom` and `up` with `down` throughout, or by both. So the four leading
//!   edges `x + d + rx − 1`, `x + d − rx`, `y + d + ry − 1`, `y + d − ry` are two formulas;
//! - a formula that **restates** part of another of the entry's counted formulas counts 0, in
//!   either order in the text: its tokens, or a mirror's, are a whole operand of the longer one, a
//!   run of its tokens bounded by its ends, brackets, commas or operators that bind more loosely
//!   (on the left) or no more tightly (on the right) than the run's loosest. So `y + ry` restates
//!   `gap = (ey − ery) − (y + ry)` and `top − 1` restates `bottom ≥ top − 1`, but `bottom ≥ top`
//!   doesn't. Precedence, loosest first: `|| && | &`; comparisons; shifts; `+ − ±`;
//!   `× ÷ · * / % mod`; `^`.
//!
//! The limits: these are textual. Two formulas that differ only in every `+` and `−` count once
//! even when they aren't mirrors, a mirror written with different names (`w` for `rx`) counts
//! twice, and an explanation that doesn't start with an explanation word counts its formulas;
//! the spec review catches all three.
//!
//! Table rows, parameters, `checked:` bullets, prose and the vector file don't count.
//!
//! **Outside an entry** (MODEL §5's template puts every arm and formula under one), an arm or a
//! definition bullet's formula counts 0 and fails: it is a finding, and `run` returns the report
//! as an error.
//!
//! **Areas.** A line `Area <name> (old <tokens>): <item>, <item>, ...`, as a paragraph or a
//! bullet, in the decode notes' Budget section or in the `--areas` file, which replaces the
//! notes' areas. A spec that still states k or an area is a finding: `tools/cleanroom/
//! move_budget.sh` moves them to the notes. An item names
//! an entry (`SlopesCarry`) or any other heading, by its whole text or its leading number (`1.2`
//! for "1.2 What she remembers"). `(old N)` is the area's old-code weighted score, shown for
//! reference beside the budget; a line `Old code (old N): ...` gives it for the whole domain
//! instead, where the old code isn't split by area. A heading no area names, holding units, is
//! reported unassigned.
//!
//! **Findings**, for the spec review: arms with an earlier arm's outcome, arms that cite no vector
//! or one the file doesn't hold, original-only and record-only arms, cross-references, arms and
//! formulas outside any entry, a bullet with two `→`, entries without a `checked` bullet, area
//! items that name no heading, and the else-lint (still counted: the review decides):
//! - a **complement arm** whose outcome repeats another arm's. An arm is a complement when its
//!   condition is an earlier arm's with `not`, `no` or `unless` in front, or says `otherwise`,
//!   `else` or `any`/`every`/`all other`. A complement with an outcome of its own is a rule;
//!   one that repeats an outcome is the other arm's explanation;
//! - a **null outcome**: an outcome whose first clause starts `nothing`, `none` or `no `, or says
//!   something doesn't change. It belongs in an explanation bullet, with no `→`.
//!
//! **A spec commit's change.** With the parent's spec beside it,
//! `git show HEAD~1:docs/behaviours/D.md > old.md`, the command line
//! `tools spec-units docs/behaviours/D.md scenarios/vectors/D.toml --against old.md` prints the
//! units and the budget before and after; the Budget section's `Budget:` line is the report's.
//! The notes are `--notes FILE`, or the spec's sibling `<domain>.decode.md` when it exists.

use std::collections::hash_map::Entry;
use std::collections::{BTreeSet, HashMap, HashSet};
use std::fmt::{self, Write as _};
use std::fs;

/// The command line `run` takes.
pub const USAGE: &str =
    "tools spec-units SPEC VECTORS [--notes FILE] [--areas FILE] [--against OLD_SPEC]";

/// Words an outcome may hold beside entry names and still be only a cross-reference.
const REFERENCE_WORDS: &[&str] = &[
    "and", "as", "by", "follows", "in", "like", "of", "or", "per", "rule", "rules", "s", "see",
    "the",
];
/// Operators a formula may hold, longest first so the scan takes the longest match.
const OPERATORS: &[&str] = &[
    "<<", ">>", "<=", ">=", "==", "!=", "&&", "||", "≤", "≥", "≠", "±", "×", "÷", "·", "+", "-",
    "−", "*", "/", "%", "<", ">", "=", "&", "|", "^",
];
/// Operators that can also be a sign.
const SIGNS: &[&str] = &["+", "-", "−", "±"];
/// Named operators a formula may use.
const WORD_OPERATORS: &[&str] = &["abs", "max", "min", "mod"];
/// The swaps that mirror a formula to the other side.
const SIDE_SWAPS: &[(&str, &str)] = &[
    ("+", "−"),
    ("<", ">"),
    ("≤", "≥"),
    ("<=", ">="),
    ("min", "max"),
    ("left", "right"),
    ("top", "bottom"),
    ("up", "down"),
];
/// Words that open an explanation bullet, whose formulas count nothing.
const EXPLANATION_WORDS: &[&str] = &["so", "thus", "hence", "as", "e.g.", "i.e."];
/// Phrases that open an explanation bullet.
const EXPLANATION_PHRASES: &[&str] = &["for example", "that is"];
/// The label of an arm only the original shows.
const ORIGINAL_ONLY: &str = "original only";
/// The label of an arm only recorded frames no vector holds show.
const RECORD_ONLY: &str = "record only";
/// Entry fields, whose formulas count nothing.
const FIELDS: &[&str] = &["kind:", "summary:", "params:", "switch:", "checked"];

/// A quantity of spec units, held in thirds so that a cell term or an operator is exact.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, PartialOrd, Ord)]
pub struct Units {
    thirds: u64,
}

impl Units {
    /// The units of `arms` arms, `cells` cell terms and `ops` operators.
    #[must_use]
    pub const fn of(arms: u64, cells: u64, ops: u64) -> Self {
        Self {
            thirds: arms
                .saturating_mul(3)
                .saturating_add(cells)
                .saturating_add(ops),
        }
    }

    /// The units in thirds.
    #[must_use]
    pub const fn thirds(self) -> u64 {
        self.thirds
    }

    /// k × the units, in weighted tokens, rounded down.
    #[must_use]
    pub const fn budget(self, k: PerUnit) -> u64 {
        self.thirds.saturating_mul(k.tenths) / 30
    }
}

impl fmt::Display for Units {
    /// To one decimal, rounded half up.
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let tenths = self.thirds.saturating_mul(20).saturating_add(3) / 6;
        write!(f, "{}.{}", tenths / 10, tenths % 10)
    }
}

/// k, the weighted tokens one spec unit buys (MODEL §1 decision 7), held in tenths: the decode
/// notes state it to one decimal (`k = 37.5`) or whole (`k = 36`).
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub struct PerUnit {
    tenths: u64,
}

impl PerUnit {
    /// k of `tenths` tenths of a token.
    #[must_use]
    pub const fn tenths(tenths: u64) -> Self {
        Self { tenths }
    }

    /// The k at the start of `text`, `37.5` or `36`, and the text after it.
    fn parse(text: &str) -> Option<(Self, &str)> {
        let whole_end = text
            .find(|c: char| !c.is_ascii_digit())
            .unwrap_or(text.len());
        let whole: u64 = text.get(..whole_end)?.parse().ok()?;
        let rest = text.get(whole_end..)?;
        let (tenth, rest) = match rest.strip_prefix('.') {
            Some(fraction) => {
                let digit = fraction.chars().next()?.to_digit(10)?;
                (u64::from(digit), fraction.get(1..)?)
            }
            None => (0, rest),
        };
        Some((
            Self::tenths(whole.checked_mul(10)?.checked_add(tenth)?),
            rest,
        ))
    }
}

impl fmt::Display for PerUnit {
    /// Whole when it is, else to one decimal.
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self.tenths % 10 {
            0 => write!(f, "{}", self.tenths / 10),
            tenth => write!(f, "{}.{tenth}", self.tenths / 10),
        }
    }
}

/// What a vector file holds that an arm can cite: vector ids, and the recorded frames vectors
/// come from, by row.
#[derive(Debug, Default)]
pub struct Vectors {
    ids: HashSet<String>,
    frames: HashMap<String, BTreeSet<u32>>,
}

impl Vectors {
    /// The `id = "..."` and `source = "<row>:<frame>"` lines of a vector file.
    #[must_use]
    pub fn parse(toml: &str) -> Self {
        let mut vectors = Self::default();
        for line in toml.lines() {
            if let Some(id) = string_field(line, "id") {
                vectors.ids.insert(id.to_owned());
            } else if let Some((row, frame)) = string_field(line, "source")
                .and_then(|source| source.rsplit_once(':'))
                .and_then(|(row, frame)| Some((row, frame.parse::<u32>().ok()?)))
            {
                vectors
                    .frames
                    .entry(row.to_owned())
                    .or_default()
                    .insert(frame);
            }
        }
        vectors
    }

    /// How many vectors the file holds.
    #[must_use]
    pub fn len(&self) -> usize {
        self.ids.len()
    }

    /// Whether the file holds no vector.
    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.ids.is_empty()
    }

    /// Whether `cite` is a vector id, or a recorded frame (`row:N`, or `row:N-M` holding one) that
    /// a vector's `source` is.
    fn holds(&self, cite: &str) -> bool {
        if self.ids.contains(cite) {
            return true;
        }
        let Some((row, frames)) = cite.rsplit_once(':') else {
            return false;
        };
        let frame = |part: &str| {
            part.chars()
                .all(|c| c.is_ascii_digit())
                .then(|| part.parse::<u32>().ok())
                .flatten()
        };
        let range = match frames.split_once('-') {
            Some((first, last)) => frame(first).zip(frame(last)),
            None => frame(frames).map(|only| (only, only)),
        };
        range
            .filter(|(first, last)| first <= last)
            .zip(self.frames.get(row))
            .is_some_and(|((first, last), recorded)| recorded.range(first..=last).next().is_some())
    }
}

/// The quoted value of a `key = "value"` line.
fn string_field<'a>(line: &'a str, key: &str) -> Option<&'a str> {
    line.trim()
        .strip_prefix(key)?
        .trim_start()
        .strip_prefix('=')?
        .trim()
        .strip_prefix('"')?
        .strip_suffix('"')
}

/// One area: its name, its old code's weighted score if known, and the headings it holds.
#[derive(Clone, Debug, PartialEq, Eq)]
struct AreaDef {
    name: String,
    old: Option<u64>,
    items: Vec<String>,
}

impl AreaDef {
    /// `Area <name> (old <tokens>): <items>`, the old part optional.
    fn parse(text: &str) -> Option<Self> {
        let (head, items) = text.strip_prefix("Area ")?.split_once(':')?;
        let (name, old) = match head.split_once('(') {
            Some((name, old)) => {
                let figure = old
                    .trim_end()
                    .strip_suffix(')')?
                    .trim()
                    .strip_prefix("old")?;
                (name, Some(old_figure(figure)?))
            }
            None => (head, None),
        };
        let items = items
            .split(',')
            .map(|item| item.trim().trim_end_matches('.').trim_matches('`').trim())
            .filter(|item| !item.is_empty())
            .map(str::to_owned)
            .collect();
        Some(Self {
            name: name.trim().to_owned(),
            old,
            items,
        })
    }

    /// Whether an item of this area names `heading`.
    fn holds(&self, heading: &str) -> bool {
        self.items.iter().any(|item| names(item, heading))
    }
}

/// Whether an area item names a heading: its whole text, or its leading number.
fn names(item: &str, heading: &str) -> bool {
    heading == item
        || heading
            .strip_prefix(item)
            .is_some_and(|rest| rest.starts_with([' ', '.']))
}

/// A logical piece of a block's text: a bullet with its continuation lines, or a paragraph.
struct Piece {
    text: String,
    line: usize,
    bullet: bool,
}

/// The text under one heading.
struct Block {
    heading: String,
    /// A `### Id` heading: a behaviour entry.
    entry: bool,
    pieces: Vec<Piece>,
}

/// Splits a spec into blocks of pieces. Table rows and fenced code are dropped.
fn split_blocks(spec: &str) -> Vec<Block> {
    let mut blocks = vec![Block {
        heading: String::new(),
        entry: false,
        pieces: Vec::new(),
    }];
    let mut open: Option<Piece> = None;
    let mut fenced = false;
    for (number, raw) in (1_usize..).zip(spec.lines()) {
        let line = raw.trim();
        if line.starts_with("```") {
            fenced = !fenced;
            close(&mut blocks, &mut open);
            continue;
        }
        if fenced {
            continue;
        }
        if line.starts_with('#') {
            close(&mut blocks, &mut open);
            let level = line.chars().take_while(|c| *c == '#').count();
            let heading = line.trim_start_matches('#').trim().to_owned();
            let entry = level == 3
                && heading.chars().next().is_some_and(char::is_alphabetic)
                && heading.chars().all(|c| c.is_alphanumeric() || c == '_');
            blocks.push(Block {
                heading,
                entry,
                pieces: Vec::new(),
            });
        } else if line.is_empty() || line.starts_with('|') || line == "---" {
            close(&mut blocks, &mut open);
        } else if let Some(text) = line.strip_prefix("- ").or_else(|| line.strip_prefix("* ")) {
            close(&mut blocks, &mut open);
            open = Some(Piece {
                text: text.to_owned(),
                line: number,
                bullet: true,
            });
        } else if let Some(piece) = open.as_mut().filter(|_| !starts_field(line)) {
            piece.text.push(' ');
            piece.text.push_str(line);
        } else {
            close(&mut blocks, &mut open);
            open = Some(Piece {
                text: line.to_owned(),
                line: number,
                bullet: false,
            });
        }
    }
    close(&mut blocks, &mut open);
    blocks
}

/// Whether a line starts an `Area`, `Cells:` or `Budget:` line, which is a piece of its own even
/// straight after a paragraph.
fn starts_field(line: &str) -> bool {
    ["Area ", "Cells:", "Budget:", "Old code "]
        .iter()
        .any(|field| line.starts_with(field))
}

/// Ends the open piece, adding it to the last block.
fn close(blocks: &mut [Block], open: &mut Option<Piece>) {
    if let (Some(piece), Some(block)) = (open.take(), blocks.last_mut()) {
        block.pieces.push(piece);
    }
}

/// Text outside backticks, with the backticked spans.
fn spans(text: &str) -> impl Iterator<Item = (bool, &str)> {
    text.split('`').scan(true, |outside, part| {
        let this = *outside;
        *outside = !this;
        Some((this, part))
    })
}

/// Lowercase, single-spaced and without trailing punctuation: two outcomes are the same when
/// these are.
fn normal(text: &str) -> String {
    text.split_whitespace()
        .collect::<Vec<_>>()
        .join(" ")
        .to_lowercase()
        .trim_matches(|c: char| c.is_ascii_punctuation() && c != '`')
        .trim()
        .to_owned()
}

/// The operators of one formula.
fn operators(formula: &str) -> u64 {
    let mut rest = formula;
    let mut operand = false;
    let mut count = 0_u64;
    while let Some(c) = rest.chars().next() {
        if c.is_alphanumeric() || c == '_' || c == '.' {
            let (word, tail) = rest.split_at(word_end(rest));
            if WORD_OPERATORS.contains(&word) {
                count = count.saturating_add(1);
                operand = false;
            } else {
                operand = true;
            }
            rest = tail;
            continue;
        }
        if let Some(op) = OPERATORS.iter().find(|op| rest.starts_with(**op)) {
            if operand || !SIGNS.contains(op) {
                count = count.saturating_add(1);
            }
            operand = false;
            rest = rest.get(op.len()..).unwrap_or_default();
            continue;
        }
        match c {
            ')' | ']' => operand = true,
            '(' | '[' | ',' => operand = false,
            _ => {}
        }
        rest = rest.get(c.len_utf8()..).unwrap_or_default();
    }
    count
}

/// Where a name or number at the start of `text` ends. An ASCII `-` between two letters or
/// digits joins a name (`C-wall`); anywhere else it is an operator.
fn word_end(text: &str) -> usize {
    let mut chars = text.char_indices().peekable();
    while let Some((at, c)) = chars.next() {
        let joins = c == '-' && chars.peek().is_some_and(|(_, next)| next.is_alphanumeric());
        if !(c.is_alphanumeric() || c == '_' || c == '.' || joins) {
            return at;
        }
    }
    text.len()
}

/// A formula as tokens: names and numbers, operators (an ASCII `-` read as `−`) and the other
/// marks, spaces dropped.
fn tokens(formula: &str) -> Vec<String> {
    let mut out = Vec::new();
    let mut rest = formula;
    while let Some(c) = rest.chars().next() {
        let end = if c.is_alphanumeric() || c == '_' || c == '.' {
            word_end(rest)
        } else if let Some(op) = OPERATORS.iter().find(|op| rest.starts_with(**op)) {
            op.len()
        } else {
            c.len_utf8()
        };
        let (token, tail) = rest.split_at(end.max(c.len_utf8()));
        if !c.is_whitespace() {
            out.push(if token == "-" { "−" } else { token }.to_owned());
        }
        rest = tail;
    }
    out
}

/// A token mirrored to the other axis: `x` and `y` swapped in a name of up to three letters.
fn axis_mirror(token: &str) -> String {
    let short = token.chars().count() <= 3
        && token.chars().all(|c| c.is_ascii_alphabetic())
        && !WORD_OPERATORS.contains(&token);
    if !short {
        return token.to_owned();
    }
    token
        .chars()
        .map(|c| match c {
            'x' => 'y',
            'y' => 'x',
            'X' => 'Y',
            'Y' => 'X',
            other => other,
        })
        .collect()
}

/// A token mirrored to the other side (`SIDE_SWAPS`).
fn side_mirror(token: &str) -> String {
    SIDE_SWAPS
        .iter()
        .find_map(|(a, b)| {
            if token == *a {
                Some(*b)
            } else if token == *b {
                Some(*a)
            } else {
                None
            }
        })
        .unwrap_or(token)
        .to_owned()
}

/// A formula's tokens and its mirrors': as written, to the other axis, the other side, and both.
fn mirrors(tokens: &[String]) -> [Vec<String>; 4] {
    let axis: Vec<String> = tokens.iter().map(|token| axis_mirror(token)).collect();
    let side = |from: &[String]| from.iter().map(|token| side_mirror(token)).collect();
    [tokens.to_vec(), side(&axis), side(tokens), axis]
}

/// The operators an entry's counted formulas hold: each formula once with its mirrors, and none
/// for a formula that restates a run of a longer one.
fn entry_operators(formulas: &[String]) -> u64 {
    let mut distinct: Vec<(&str, [Vec<String>; 4])> = Vec::new();
    for formula in formulas {
        let variants = mirrors(&tokens(formula));
        let [written, ..] = &variants;
        let seen = distinct
            .iter()
            .any(|(_, other)| other.iter().any(|variant| variant == written));
        if !seen {
            distinct.push((formula, variants));
        }
    }
    distinct
        .iter()
        .filter(|(_, variants)| {
            !distinct
                .iter()
                .any(|(_, [longer, ..])| variants.iter().any(|variant| restates(longer, variant)))
        })
        .fold(0_u64, |sum, (formula, _)| {
            sum.saturating_add(operators(formula))
        })
}

/// How tightly an operator token binds, loosest 1; `None` for any other token.
fn precedence(token: &str) -> Option<u8> {
    match token {
        "||" | "&&" | "|" | "&" => Some(1),
        "=" | "==" | "!=" | "≠" | "<" | ">" | "<=" | ">=" | "≤" | "≥" => Some(2),
        "<<" | ">>" => Some(3),
        "+" | "−" | "±" => Some(4),
        "×" | "÷" | "·" | "*" | "/" | "%" | "mod" => Some(5),
        "^" => Some(6),
        _ => None,
    }
}

/// Whether `run` restates a subexpression of the longer formula `longer`: its tokens appear
/// there whole, bounded on the left by the start, an opening mark, a comma or a looser
/// operator, and on the right by the end, a closing mark, a comma or an operator no tighter
/// than the run's loosest. So `top − 1` is part of `bottom ≥ top − 1`, and `bottom ≥ top` isn't.
fn restates(longer: &[String], run: &[String]) -> bool {
    if run.is_empty() || run.len() >= longer.len() {
        return false;
    }
    let mut depth = 0_i32;
    let loosest = run
        .iter()
        .filter_map(|token| {
            match token.as_str() {
                "(" | "[" => depth = depth.saturating_add(1),
                ")" | "]" => depth = depth.saturating_sub(1),
                _ => {}
            }
            // An absolute value's bars group, so they don't set the run's binding.
            (depth == 0 && token != "|")
                .then(|| precedence(token))
                .flatten()
        })
        .min()
        .unwrap_or(u8::MAX);
    longer.windows(run.len()).enumerate().any(|(at, window)| {
        let before = at.checked_sub(1).and_then(|left| longer.get(left));
        let after = longer.get(at.saturating_add(run.len()));
        let left_ok = before.is_none_or(|token| {
            matches!(token.as_str(), "(" | "[" | ",")
                || precedence(token).is_some_and(|binding| binding < loosest)
        });
        let right_ok = after.is_none_or(|token| {
            matches!(token.as_str(), ")" | "]" | ",")
                || precedence(token).is_some_and(|binding| binding <= loosest)
        });
        window == run && left_ok && right_ok
    })
}

/// Whether a bullet with no `→` explains rather than defines: it starts with an explanation
/// word or phrase.
fn explains(text: &str) -> bool {
    let lower = text
        .trim_start_matches(|c: char| !c.is_alphanumeric())
        .to_lowercase();
    let first = lower
        .split(|c: char| c.is_whitespace() || c == ',')
        .next()
        .unwrap_or_default();
    EXPLANATION_WORDS.contains(&first)
        || EXPLANATION_PHRASES
            .iter()
            .any(|phrase| lower.starts_with(phrase))
}

/// Whether a bullet is an entry field.
fn field(text: &str) -> bool {
    let lower = text.trim_start_matches('*').to_lowercase();
    FIELDS.iter().any(|name| lower.starts_with(name))
}

/// The formulas of a text: its backticked spans holding a space, single-spaced.
fn formulas(text: &str) -> impl Iterator<Item = String> + '_ {
    spans(text)
        .filter(|(outside, _)| !outside)
        .map(|(_, span)| span.split_whitespace().collect::<Vec<_>>().join(" "))
        .filter(|formula| formula.contains(' '))
}

/// How an arm was counted.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Verdict {
    /// One unit.
    Counts,
    /// The same outcome as an earlier counted arm of the entry, at that line: nothing.
    SameOutcome(usize),
    /// It cites no vector, as it says: nothing.
    Uncited(Unchecked),
    /// It cites these, which the vector file doesn't hold: nothing.
    Missing(Vec<String>),
    /// Its outcome only names other entries: nothing.
    CrossReference,
    /// It is under no `### Entry` heading: nothing, and the spec fails.
    OutsideEntry,
}

/// Why an arm cites no vector.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Unchecked {
    /// It has no bracket of cites.
    NoCite,
    /// `[original only]`: only the original shows it.
    OriginalOnly,
    /// `[record only: row:frame, ...]`: recorded frames no vector holds.
    RecordOnly(Vec<String>),
}

/// The else-lint's finding on a complement arm: its condition is the complement of the arm at
/// `of`, and its outcome repeats the arm at `repeats`.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Complement {
    /// The arm whose condition it complements.
    pub of: usize,
    /// The arm whose outcome it repeats.
    pub repeats: usize,
}

/// One arm as the spec states it.
#[derive(Clone, Debug)]
pub struct Arm {
    /// Its heading.
    pub block: String,
    /// The spec line its bullet starts on.
    pub line: usize,
    condition: String,
    outcome: String,
    /// How it counted.
    pub verdict: Verdict,
    /// A complement arm repeating another arm's outcome.
    pub complement: Option<Complement>,
    /// Whether its outcome is nothing happening, which belongs in an explanation bullet.
    pub null_outcome: bool,
    /// Whether its bullet holds more than one `→`.
    pub chained: bool,
}

/// What one arm bullet says, before it is judged.
struct Stated<'a> {
    condition: &'a str,
    outcome: &'a str,
    cites: Cites<'a>,
    chained: bool,
}

/// What an arm's trailing bracket cites.
#[derive(Default)]
struct Cites<'a> {
    /// Vector ids and recorded frames, which must exist.
    vectors: Vec<&'a str>,
    /// `original only` is among them.
    original_only: bool,
    /// The frames after `record only:`.
    record_only: Vec<&'a str>,
}

impl<'a> Cites<'a> {
    /// A bracket's text, its parts split at `;`.
    fn parse(bracket: &'a str) -> Self {
        let mut cites = Self::default();
        for part in bracket.split(';').map(str::trim) {
            let lower = part.to_lowercase();
            if lower.starts_with(ORIGINAL_ONLY) {
                cites.original_only = true;
            } else if lower.starts_with(RECORD_ONLY) {
                let frames = part.get(RECORD_ONLY.len()..).unwrap_or_default();
                cites
                    .record_only
                    .extend(split_cites(frames.trim_start_matches([':', ' '])));
            } else {
                cites.vectors.extend(split_cites(part));
            }
        }
        cites
    }
}

/// The cites of a list, split at commas and spaces.
fn split_cites(text: &str) -> impl Iterator<Item = &str> {
    text.split([',', ' '])
        .map(|cite| cite.trim_matches(['(', ')', '`']))
        .filter(|cite| !cite.is_empty())
}

/// Splits an arm bullet at its first `→` outside backticks, and takes its trailing `[...]`.
fn stated(text: &str) -> Option<Stated<'_>> {
    let mut arrows = Vec::new();
    let mut offset = 0_usize;
    for (outside, part) in spans(text) {
        if outside {
            arrows.extend(
                part.match_indices('→')
                    .map(|(at, _)| offset.saturating_add(at)),
            );
        }
        offset = offset.saturating_add(part.len()).saturating_add(1);
    }
    let first = *arrows.first()?;
    let (condition, rest) = text.split_at(first);
    let rest = rest.strip_prefix('→').unwrap_or(rest);
    let trimmed = rest.trim_end().trim_end_matches(['.', ';', ',']).trim_end();
    let (outcome, cites) = match trimmed.strip_suffix(']').zip(trimmed.rfind('[')) {
        Some((inner, open)) => {
            let (outcome, bracket) = inner.split_at(open);
            (outcome, Cites::parse(bracket.trim_start_matches('[')))
        }
        None => (trimmed, Cites::default()),
    };
    Some(Stated {
        condition: condition.trim(),
        outcome: outcome.trim(),
        cites,
        chained: arrows.len() > 1,
    })
}

/// Whether an outcome names only other entries, with joining words.
fn cross_reference(outcome: &str, entries: &HashSet<&str>) -> bool {
    let mut named = false;
    for (outside, part) in spans(outcome) {
        if outside {
            let filler = part
                .split(|c: char| !c.is_alphanumeric())
                .filter(|word| !word.is_empty())
                .all(|word| REFERENCE_WORDS.contains(&word.to_lowercase().as_str()));
            if !filler {
                return false;
            }
        } else if entries.contains(part.trim()) {
            named = true;
        } else {
            return false;
        }
    }
    named
}

/// The arms, cell terms and operators of one area or entry.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct Tally {
    /// Arms that count.
    pub arms: u64,
    /// Cell terms first named here.
    pub cells: u64,
    /// Formula operators.
    pub ops: u64,
}

impl Tally {
    /// Its units.
    #[must_use]
    pub const fn units(self) -> Units {
        Units::of(self.arms, self.cells, self.ops)
    }

    const fn add(self, other: Self) -> Self {
        Self {
            arms: self.arms.saturating_add(other.arms),
            cells: self.cells.saturating_add(other.cells),
            ops: self.ops.saturating_add(other.ops),
        }
    }
}

/// One heading's count.
#[derive(Clone, Debug)]
pub struct BlockCount {
    /// Its heading.
    pub heading: String,
    /// Whether it is a `### Id` entry.
    pub entry: bool,
    /// Whether it has a `checked` bullet.
    pub checked: bool,
    /// Its area's index in [`Count::areas`], if an area names it.
    pub area: Option<usize>,
    /// What it holds.
    pub tally: Tally,
}

/// One area's count and budget.
#[derive(Clone, Debug)]
pub struct AreaCount {
    /// Its name.
    pub name: String,
    /// Its old code's weighted score, if given.
    pub old: Option<u64>,
    /// How many entries it holds.
    pub entries: usize,
    /// What it holds.
    pub tally: Tally,
    /// The domain's k, if its decode notes' Budget section states one.
    pub k: Option<PerUnit>,
}

impl AreaCount {
    /// Its budget: k × its units, if the notes state k.
    #[must_use]
    pub fn budget(&self) -> Option<u64> {
        self.k.map(|k| self.tally.units().budget(k))
    }
}

/// A spec's units, by area and heading, its arms and its findings.
#[derive(Clone, Debug)]
pub struct Count {
    /// The areas, in the order they are declared, then "unassigned" if a heading with units has
    /// none.
    pub areas: Vec<AreaCount>,
    /// Every heading, in spec order.
    pub blocks: Vec<BlockCount>,
    /// Every arm.
    pub arms: Vec<Arm>,
    /// Area items that name no heading: (area, item).
    pub unknown_items: Vec<(String, String)>,
    /// The "unassigned" area's index, when an entry or a heading with units has no area.
    pub unassigned: Option<usize>,
    /// Formulas outside any entry.
    pub strays: Vec<StrayFormula>,
    /// The domain's k, from its decode notes' Budget section, "k = N weighted tokens per spec
    /// unit".
    pub k: Option<PerUnit>,
    /// Whether the spec itself states k or an area, which belong in the decode notes.
    pub spec_states_budget: bool,
    /// The old code's weighted score for the whole domain, where the notes give it whole
    /// rather than by area: "Old code (old N): ...".
    pub old: Option<u64>,
}

impl Count {
    /// Counts `spec` against `vectors`. k and the areas come from `budget`, the decode notes'
    /// Budget section ([`budget_section`]); areas from `areas` instead when given.
    #[must_use]
    pub fn of(spec: &str, vectors: &Vectors, budget: Option<&str>, areas: Option<&str>) -> Self {
        let blocks = split_blocks(spec);
        let spec_states_budget = stated_k(&blocks).is_some() || !area_defs(&blocks).is_empty();
        let budget = split_blocks(budget.unwrap_or_default());
        let k = stated_k(&budget);
        let old = stated_old(&budget);
        let defs = areas.map_or_else(|| area_defs(&budget), |file| area_defs(&split_blocks(file)));
        let entries: HashSet<&str> = blocks
            .iter()
            .filter(|block| block.entry)
            .map(|block| block.heading.as_str())
            .collect();
        let mut seen_cells = BTreeSet::new();
        let mut arms = Vec::new();
        let mut strays = Vec::new();
        let mut counts: Vec<BlockCount> = blocks
            .iter()
            .map(|block| {
                let (tally, checked) = count_block(
                    block,
                    vectors,
                    &entries,
                    &mut seen_cells,
                    &mut arms,
                    &mut strays,
                );
                BlockCount {
                    heading: block.heading.clone(),
                    entry: block.entry,
                    checked,
                    area: defs.iter().position(|def| def.holds(&block.heading)),
                    tally,
                }
            })
            .collect();
        let unknown_items = defs
            .iter()
            .flat_map(|def| {
                def.items
                    .iter()
                    .filter(|item| !blocks.iter().any(|block| names(item, &block.heading)))
                    .map(|item| (def.name.clone(), item.clone()))
            })
            .collect();
        let mut areas: Vec<AreaCount> = defs
            .iter()
            .map(|def| AreaCount {
                name: def.name.clone(),
                old: def.old,
                entries: 0,
                tally: Tally::default(),
                k,
            })
            .collect();
        let mut unassigned = None;
        for block in &mut counts {
            if block.area.is_none() && (block.entry || block.tally.units() > Units::default()) {
                block.area = Some(*unassigned.get_or_insert_with(|| {
                    areas.push(AreaCount {
                        name: String::from("unassigned"),
                        old: None,
                        entries: 0,
                        tally: Tally::default(),
                        k,
                    });
                    areas.len().saturating_sub(1)
                }));
            }
            if let Some(area) = block.area.and_then(|area| areas.get_mut(area)) {
                area.tally = area.tally.add(block.tally);
                if block.entry {
                    area.entries = area.entries.saturating_add(1);
                }
            }
        }
        Self {
            areas,
            blocks: counts,
            arms,
            unknown_items,
            unassigned,
            strays,
            k,
            spec_states_budget,
            old,
        }
    }

    /// Whether the spec fails the template: an arm or a formula outside any entry.
    #[must_use]
    pub fn fails(&self) -> bool {
        !self.strays.is_empty()
            || self
                .arms
                .iter()
                .any(|arm| arm.verdict == Verdict::OutsideEntry)
    }

    /// Everything the spec holds.
    #[must_use]
    pub fn total(&self) -> Tally {
        self.areas
            .iter()
            .fold(Tally::default(), |sum, area| sum.add(area.tally))
    }

    /// The domain's budget, each area's summed, if the notes state k.
    #[must_use]
    pub fn budget(&self) -> Option<u64> {
        self.areas.iter().try_fold(0_u64, |sum, area| {
            area.budget().map(|budget| sum.saturating_add(budget))
        })
    }

    /// The budget line, as the decode notes' Budget section holds it.
    #[must_use]
    pub fn budget_line(&self) -> String {
        let (Some(k), Some(budget)) = (self.k, self.budget()) else {
            return format!(
                "Budget: no k in the decode notes' Budget section (MODEL §1 decision 7), {} units",
                self.total().units()
            );
        };
        let split: Vec<String> = self
            .areas
            .iter()
            .map(|area| format!("{} {}", area.name, or_dash(area.budget())))
            .collect();
        format!(
            "Budget: {k} × {} units = {} ({})",
            self.total().units(),
            thousands(budget),
            split.join(", ")
        )
    }

    /// The report: units and budget by area, the budget line, the findings, and units by
    /// heading.
    #[must_use]
    pub fn report(&self) -> String {
        let mut out =
            String::from("area                 entries  arms cells   ops   units budget    old\n");
        for area in &self.areas {
            let _ = writeln!(
                out,
                "{:<20} {:>7} {:>5} {:>5} {:>5} {:>7} {:>6} {:>6}",
                area.name,
                area.entries,
                area.tally.arms,
                area.tally.cells,
                area.tally.ops,
                area.tally.units().to_string(),
                or_dash(area.budget()),
                or_dash(area.old)
            );
        }
        let total = self.total();
        let entries = self.areas.iter().map(|area| area.entries).sum::<usize>();
        let _ = writeln!(
            out,
            "{:<20} {:>7} {:>5} {:>5} {:>5} {:>7} {:>6} {:>6}\n\n{}\n",
            "total",
            entries,
            total.arms,
            total.cells,
            total.ops,
            total.units().to_string(),
            or_dash(self.budget()),
            "",
            self.budget_line()
        );
        out.push_str(&self.findings());
        out.push_str("\nheading                         area          arms cells   ops   units\n");
        for block in self.blocks.iter().filter(|block| block.area.is_some()) {
            let area = block
                .area
                .and_then(|area| self.areas.get(area))
                .map_or("", |area| area.name.as_str());
            let _ = writeln!(
                out,
                "{:<31} {:<13} {:>4} {:>5} {:>5} {:>7}",
                block.heading,
                area,
                block.tally.arms,
                block.tally.cells,
                block.tally.ops,
                block.tally.units().to_string()
            );
        }
        out
    }

    /// The minimality findings and the spec's gaps in the convention, one line each.
    #[must_use]
    pub fn findings(&self) -> String {
        let mut out = String::from("findings\n");
        let start = out.len();
        if self.k.is_none() {
            out.push_str(
                "  the decode notes' Budget section states no k (\"k = N weighted tokens per spec unit\"), so no budget\n",
            );
        }
        if self.spec_states_budget {
            out.push_str(
                "  the spec states k or an area: they go in the decode notes' Budget section (tools/cleanroom/move_budget.sh)\n",
            );
        }
        for arm in &self.arms {
            let at = format!("  {}:{} `{}`", arm.block, arm.line, arm.condition);
            match &arm.verdict {
                Verdict::Counts => {}
                Verdict::SameOutcome(first) => {
                    let _ = writeln!(
                        out,
                        "{at}: same outcome as line {first} (\"{}\"), counted once",
                        arm.outcome
                    );
                }
                Verdict::Uncited(Unchecked::NoCite) => {
                    let _ = writeln!(out, "{at}: cites no vector, counts 0");
                }
                Verdict::Uncited(Unchecked::OriginalOnly) => {
                    let _ = writeln!(out, "{at}: original only, counts 0");
                }
                Verdict::Uncited(Unchecked::RecordOnly(frames)) => {
                    let _ = writeln!(out, "{at}: record only ({}), counts 0", frames.join(", "));
                }
                Verdict::OutsideEntry => {
                    let _ = writeln!(out, "{at}: an arm outside any entry, counts 0 and fails");
                }
                Verdict::Missing(cites) => {
                    let _ = writeln!(
                        out,
                        "{at}: cites {} the vectors don't hold, counts 0",
                        cites.join(", ")
                    );
                }
                Verdict::CrossReference => {
                    let _ = writeln!(out, "{at}: a cross-reference, counts 0");
                }
            }
            if let Some(Complement { of, repeats }) = arm.complement {
                let _ = writeln!(
                    out,
                    "{at}: the complement of line {of} with line {repeats}'s outcome, an                      explanation"
                );
            }
            if arm.null_outcome {
                let _ = writeln!(
                    out,
                    "{at}: a null outcome (\"{}\"), an explanation bullet with no →",
                    arm.outcome
                );
            }
            if arm.chained {
                let _ = writeln!(
                    out,
                    "{at}: more than one → in one bullet, counted as one arm"
                );
            }
        }
        for block in &self.blocks {
            if block.area.is_some() && block.area == self.unassigned {
                let _ = writeln!(out, "  {}: no area names it", block.heading);
            }
            if block.entry && !block.checked {
                let _ = writeln!(out, "  {}: no `checked` bullet", block.heading);
            }
        }
        for stray in &self.strays {
            let _ = writeln!(
                out,
                "  {}:{} `{}`: a formula outside any entry, counts 0 and fails",
                stray.block, stray.line, stray.formula
            );
        }
        for (area, item) in &self.unknown_items {
            let _ = writeln!(out, "  area {area}: {item} names no heading");
        }
        if out.len() == start {
            out.push_str("  none\n");
        }
        out
    }
}

/// The k a spec's text states, "k = N weighted tokens per spec unit" (N whole or to one
/// decimal), the first one found.
fn stated_k(blocks: &[Block]) -> Option<PerUnit> {
    const AFTER: &str = " weighted tokens per spec unit";
    blocks
        .iter()
        .flat_map(|block| &block.pieces)
        .find_map(|piece| {
            piece.text.match_indices("k = ").find_map(|(at, start)| {
                let rest = piece.text.get(at.saturating_add(start.len())..)?;
                let (k, after) = PerUnit::parse(rest)?;
                after.starts_with(AFTER).then_some(k)
            })
        })
}

/// The old code's figure for the whole domain: a piece `Old code (old N): ...`, for old code
/// the notes don't split by area.
fn stated_old(blocks: &[Block]) -> Option<u64> {
    blocks
        .iter()
        .flat_map(|block| &block.pieces)
        .find_map(|piece| {
            let (head, _) = piece
                .text
                .strip_prefix("Old code (old ")?
                .split_once("):")?;
            old_figure(head)
        })
}

/// An old-code figure as the notes write it, `1,443` or `1,443.25`, in whole tokens: `size`
/// gives quarters, and a fraction of a half or more rounds up.
fn old_figure(text: &str) -> Option<u64> {
    let digits: String = text
        .chars()
        .filter(|c| *c != ',' && !c.is_whitespace())
        .collect();
    let (whole, fraction) = digits.split_once('.').unwrap_or((&digits, ""));
    let whole: u64 = whole.parse().ok()?;
    if !fraction.chars().all(|c| c.is_ascii_digit()) {
        return None;
    }
    let up = fraction.chars().next().is_some_and(|tenths| tenths >= '5');
    Some(whole.saturating_add(u64::from(up)))
}

/// A budget figure, or "-" when the notes state no k.
fn or_dash(figure: Option<u64>) -> String {
    figure.map_or_else(|| String::from("-"), thousands)
}

/// The decode notes' `## Budget` section: from its heading to the next `## ` heading, or ""
/// when the notes have none.
#[must_use]
pub fn budget_section(notes: &str) -> &str {
    let mut at = 0_usize;
    let mut start = None;
    for line in notes.split_inclusive('\n') {
        let heading = line.trim_end();
        match start {
            None if heading == "## Budget" => start = Some(at),
            Some(_) if heading.starts_with("## ") => break,
            _ => {}
        }
        at = at.saturating_add(line.len());
    }
    start
        .and_then(|start| notes.get(start..at))
        .unwrap_or_default()
}

/// The `Area` lines among the blocks' pieces.
fn area_defs(blocks: &[Block]) -> Vec<AreaDef> {
    blocks
        .iter()
        .flat_map(|block| &block.pieces)
        .filter_map(|piece| AreaDef::parse(&piece.text))
        .collect()
}

/// A formula outside any entry, which counts nothing and fails the spec.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct StrayFormula {
    /// Its heading.
    pub block: String,
    /// The spec line its bullet starts on.
    pub line: usize,
    /// The formula.
    pub formula: String,
}

/// One block's tally and whether it has a `checked` bullet; its arms go to `arms`, and formulas
/// outside an entry to `strays`.
fn count_block(
    block: &Block,
    vectors: &Vectors,
    entries: &HashSet<&str>,
    seen_cells: &mut BTreeSet<String>,
    arms: &mut Vec<Arm>,
    strays: &mut Vec<StrayFormula>,
) -> (Tally, bool) {
    let mut tally = Tally::default();
    let mut checked = false;
    let mut counted_formulas = Vec::new();
    let mut outcomes: HashMap<String, usize> = HashMap::new();
    let mut conditions: Vec<(String, usize)> = Vec::new();
    let first_arm = arms.len();
    for piece in &block.pieces {
        let text = unlabelled(&piece.text);
        if AreaDef::parse(text).is_some() || text.starts_with("Budget:") {
            continue;
        }
        if let Some(terms) = text.strip_prefix("Cells:") {
            for term in terms.split([',', ';']) {
                let term = normal(&term.replace('`', ""));
                if !term.is_empty() && seen_cells.insert(term) {
                    tally.cells = tally.cells.saturating_add(1);
                }
            }
            continue;
        }
        if !piece.bullet {
            continue;
        }
        if text.to_lowercase().starts_with("checked") {
            checked = true;
            continue;
        }
        let Some(arm) = stated(text) else {
            if field(text) || explains(text) {
                continue;
            }
            if block.entry {
                counted_formulas.extend(formulas(text));
            } else {
                strays.extend(formulas(text).filter(|formula| operators(formula) > 0).map(
                    |formula| StrayFormula {
                        block: block.heading.clone(),
                        line: piece.line,
                        formula,
                    },
                ));
            }
            continue;
        };
        let condition = normal(arm.condition);
        let outcome = normal(arm.outcome);
        let verdict = judge(&arm, block.entry, vectors, entries, || {
            match outcomes.entry(outcome.clone()) {
                Entry::Occupied(first) => Verdict::SameOutcome(*first.get()),
                Entry::Vacant(slot) => {
                    slot.insert(piece.line);
                    tally.arms = tally.arms.saturating_add(1);
                    Verdict::Counts
                }
            }
        });
        if matches!(verdict, Verdict::Counts | Verdict::SameOutcome(_)) {
            counted_formulas.extend(formulas(text));
        }
        let complements = conditions
            .iter()
            .find(|(earlier, _)| negation(&condition, earlier))
            .or_else(|| conditions.first().filter(|_| otherwise(&condition)))
            .map(|(_, line)| *line);
        conditions.push((condition.clone(), piece.line));
        arms.push(Arm {
            block: block.heading.clone(),
            line: piece.line,
            null_outcome: null_outcome(&outcome),
            condition,
            outcome,
            verdict,
            complement: complements.map(|of| Complement { of, repeats: of }),
            chained: arm.chained,
        });
    }
    else_lint(arms.get_mut(first_arm..).unwrap_or_default());
    tally.ops = entry_operators(&counted_formulas);
    (tally, checked)
}

/// An arm's verdict; `cited` judges an entry's arm whose cites all exist, by its outcome.
fn judge(
    arm: &Stated<'_>,
    in_entry: bool,
    vectors: &Vectors,
    entries: &HashSet<&str>,
    cited: impl FnOnce() -> Verdict,
) -> Verdict {
    let missing: Vec<String> = arm
        .cites
        .vectors
        .iter()
        .filter(|cite| !vectors.holds(cite))
        .map(|cite| (*cite).to_owned())
        .collect();
    if !in_entry {
        Verdict::OutsideEntry
    } else if cross_reference(arm.outcome, entries) {
        Verdict::CrossReference
    } else if !arm.cites.vectors.is_empty() {
        if missing.is_empty() {
            cited()
        } else {
            Verdict::Missing(missing)
        }
    } else if !arm.cites.record_only.is_empty() {
        Verdict::Uncited(Unchecked::RecordOnly(
            arm.cites
                .record_only
                .iter()
                .map(|&frame| frame.to_owned())
                .collect(),
        ))
    } else if arm.cites.original_only {
        Verdict::Uncited(Unchecked::OriginalOnly)
    } else {
        Verdict::Uncited(Unchecked::NoCite)
    }
}

/// The else-lint, once an entry's arms are all known (a complement can repeat a later arm's
/// outcome): a complement keeps its finding only when its outcome repeats another arm's.
fn else_lint(block_arms: &mut [Arm]) {
    let stated_outcomes: Vec<(String, usize)> = block_arms
        .iter()
        .map(|arm| (arm.outcome.clone(), arm.line))
        .collect();
    for arm in block_arms {
        arm.complement = arm.complement.and_then(|complement| {
            stated_outcomes
                .iter()
                .find(|(outcome, line)| *line != arm.line && *outcome == arm.outcome)
                .map(|(_, repeats)| Complement {
                    of: complement.of,
                    repeats: *repeats,
                })
        });
    }
}

/// A piece's text without a leading bold label (`**Ceiling.** Then, ...`) or bold marks.
fn unlabelled(text: &str) -> &str {
    let Some(bold) = text.strip_prefix("**") else {
        return text;
    };
    bold.split_once("**")
        .filter(|(label, _)| label.ends_with(['.', ':']))
        .map_or(bold, |(_, rest)| rest.trim_start())
}

/// Whether one condition is the other with `not`, `no` or `unless` in front.
fn negation(a: &str, b: &str) -> bool {
    let negated = |x: &str, y: &str| {
        ["not ", "no ", "otherwise not ", "unless "]
            .iter()
            .any(|word| x.strip_prefix(word) == Some(y))
    };
    negated(a, b) || negated(b, a)
}

/// Whether a condition is every case the entry's other arms leave: `otherwise`, `else`, or
/// `any`, `every` or `all` before `other`.
fn otherwise(condition: &str) -> bool {
    let words: Vec<&str> = condition
        .split(|c: char| !c.is_alphanumeric())
        .filter(|word| !word.is_empty())
        .collect();
    words
        .iter()
        .any(|word| matches!(*word, "otherwise" | "else"))
        || words
            .windows(2)
            .any(|pair| matches!(pair, ["any" | "every" | "all", "other"]))
}

/// Whether an outcome is nothing happening: its first clause starts `nothing`, `none` or `no `,
/// or says something doesn't change.
fn null_outcome(outcome: &str) -> bool {
    let clause = outcome
        .split([':', ';', ','])
        .next()
        .unwrap_or_default()
        .trim();
    ["nothing", "none", "no "]
        .iter()
        .any(|start| clause.starts_with(start))
        || ["n't change", "not change", "unchanged"]
            .iter()
            .any(|phrase| clause.contains(phrase))
}

/// `n` with a comma between each three digits.
fn thousands(n: u64) -> String {
    let digits = n.to_string();
    let mut out = String::new();
    for (left, c) in (0..digits.len()).rev().zip(digits.chars()) {
        out.push(c);
        if left > 0 && left % 3 == 0 {
            out.push(',');
        }
    }
    out
}

/// `--notes FILE`, `--areas FILE` and `--against OLD_SPEC`.
#[derive(Default)]
struct Options {
    notes: Option<String>,
    areas: Option<String>,
    against: Option<String>,
}

fn options(args: &[String]) -> Result<Options, String> {
    let mut options = Options::default();
    let mut rest = args;
    while let [flag, value, tail @ ..] = rest {
        match flag.as_str() {
            "--notes" => options.notes = Some(read(value)?),
            "--areas" => options.areas = Some(read(value)?),
            "--against" => options.against = Some(read(value)?),
            _ => return Err(USAGE.to_owned()),
        }
        rest = tail;
    }
    if rest.is_empty() {
        Ok(options)
    } else {
        Err(USAGE.to_owned())
    }
}

fn read(path: &str) -> Result<String, String> {
    fs::read_to_string(path).map_err(|e| format!("{path}: {e}"))
}

/// The decode notes beside `spec` (`D.md` beside `D.decode.md`), if they exist.
fn sibling_notes(spec: &str) -> Result<Option<String>, String> {
    let Some(stem) = spec.strip_suffix(".md") else {
        return Ok(None);
    };
    let notes = format!("{stem}.decode.md");
    if fs::exists(&notes).map_err(|e| format!("{notes}: {e}"))? {
        read(&notes).map(Some)
    } else {
        Ok(None)
    }
}

/// The change from `old` to `new`, signed.
fn delta<T: fmt::Display>(old: u64, new: u64, show: impl Fn(u64) -> T) -> String {
    if new >= old {
        format!("+{}", show(new.saturating_sub(old)))
    } else {
        format!("-{}", show(old.saturating_sub(new)))
    }
}

/// Counts a spec's units and prints its budget; with `--against`, the change from an older spec.
///
/// # Errors
///
/// The usage on a wrong command line, the path and the error when a file can't be read, or the
/// whole report when the spec fails (an arm or a formula outside any entry).
pub fn run(args: &[String]) -> Result<String, String> {
    let [spec, vectors, rest @ ..] = args else {
        return Err(USAGE.to_owned());
    };
    let options = options(rest)?;
    let vectors = Vectors::parse(&read(vectors)?);
    let notes = match options.notes {
        Some(notes) => Some(notes),
        None => sibling_notes(spec)?,
    };
    let budget = notes.as_deref().map(budget_section);
    let count = Count::of(&read(spec)?, &vectors, budget, options.areas.as_deref());
    let mut out = format!(
        "spec units: {spec}, {} vectors (k = {})\n\n{}",
        vectors.len(),
        count.k.map_or_else(|| String::from("-"), |k| k.to_string()),
        count.report()
    );
    if let Some(old) = options.against {
        let before = Count::of(&old, &vectors, budget, options.areas.as_deref());
        let (was, now) = (before.total().units(), count.total().units());
        let units = delta(was.thirds(), now.thirds(), |thirds| Units { thirds });
        let _ = writeln!(
            out,
            "\nchange: units {was} → {now} ({units}), budget {} → {} ({})",
            or_dash(before.budget()),
            or_dash(count.budget()),
            match (before.budget(), count.budget()) {
                (Some(was), Some(now)) => delta(was, now, thousands),
                _ => String::from("-"),
            }
        );
    }
    if count.fails() {
        out.push_str(
            "
fails: an arm or a formula is outside any entry (see the findings)
",
        );
        return Err(out);
    }
    Ok(out)
}

#[cfg(test)]
mod tests {
    use super::{
        Complement, Count, PerUnit, StrayFormula, Unchecked, Units, Vectors, Verdict,
        budget_section, delta, entry_operators, operators, thousands,
    };

    const VECTORS: &str = r#"
[[vector]]
id = "V1"
source = "constructed"

[[vector]]
id = "V2"
source = "rec:10"

[[vector]]
id = "V3"
source = "constructed"
"#;

    /// The demo's decode notes: its Budget section, between two other sections.
    const NOTES: &str = "# Demo: decode notes

## Sources

Area ignored (old 5): Walk

## Budget

k = 24 weighted tokens per spec unit, frozen.

Area moving (old 1,000): Walk, 1.2
Area falling: `Fall`

## Later

k = 99 weighted tokens per spec unit
";

    const SPEC: &str = "# Demo

## Header

### 1.2 Memory
- carrying on with the pad let go → she keeps building [V1]

### Walk
- Mechanic. How the step builds.
- holding a side → she gains the step [V1, V2]
- holding a side with run held → she gains the step. [V3]
- facing the screen → she doesn't move [Nope]
- standing → she halts
- landing → as `Fall`
- Her leading edge is `x + d + rx − 1`, and again `x + d + rx − 1`.
- checked: oracle (V1 → V2)

| She is | Next pose |
| --- | --- |
| running | kept |

Cells: kept, gun drawn, `kept`

### Fall
- in the air → gravity adds `0.5 × t` [rec:10]
- not in the air → she stays put [V2]
- checked: constructed

### Unnamed
- a rule nobody placed → something [V1]
";

    fn count(spec: &str) -> Count {
        count_with(spec, "")
    }

    /// `spec` counted with `budget` as its notes' Budget section.
    fn count_with(spec: &str, budget: &str) -> Count {
        Count::of(spec, &Vectors::parse(VECTORS), Some(budget), None)
    }

    fn demo() -> Count {
        count_with(SPEC, budget_section(NOTES))
    }

    fn thirds(spec: &str) -> u64 {
        count(spec).total().units().thirds()
    }

    #[test]
    fn arms_cells_and_operators_make_the_units() {
        let count = demo();
        let [moving, falling, unassigned] = count.areas.as_slice() else {
            panic!("three areas: {:?}", count.areas);
        };
        assert_eq!(
            (moving.tally.arms, moving.tally.cells, moving.tally.ops),
            (1, 2, 3)
        );
        // `1.2 Memory` is no entry: its arm counts 0 and fails the spec.
        assert!(count.fails());
        assert_eq!(moving.tally.units().thirds(), 8);
        // Its old code is shown for reference and doesn't bound the budget.
        assert_eq!(moving.old, Some(1000));
        assert_eq!(moving.budget(), Some(64));
        assert_eq!(moving.entries, 1);
        assert_eq!((falling.tally.arms, falling.tally.ops), (2, 1));
        assert_eq!(falling.budget(), Some(56));
        assert_eq!(unassigned.tally.arms, 1);
        assert_eq!(count.unassigned, Some(2));
        assert_eq!(count.budget(), Some(64 + 56 + 24));
    }

    #[test]
    fn the_old_code_can_be_given_for_the_whole_domain() {
        assert_eq!(demo().old, None);
        let whole = NOTES.replace(
            "Area moving (old 1,000)",
            "Old code (old 2,345): the old module, not split by area.

Area moving",
        );
        let count = count_with(SPEC, budget_section(&whole));
        assert_eq!(count.old, Some(2345));
        assert_eq!(count.areas.first().and_then(|area| area.old), None);
        let quarters = whole.replace("2,345)", "1,443.25)");
        assert_eq!(count_with(SPEC, budget_section(&quarters)).old, Some(1443));
        let half = whole.replace("2,345)", "1,443.5)");
        assert_eq!(count_with(SPEC, budget_section(&half)).old, Some(1444));
    }

    #[test]
    fn verdicts_say_why_an_arm_counts_nothing() {
        let count = demo();
        let verdicts: Vec<_> = count
            .arms
            .iter()
            .filter(|arm| arm.block == "Walk")
            .map(|arm| arm.verdict.clone())
            .collect();
        assert_eq!(
            verdicts,
            [
                Verdict::Counts,
                Verdict::SameOutcome(10),
                Verdict::Missing(vec![String::from("Nope")]),
                Verdict::Uncited(Unchecked::NoCite),
                Verdict::CrossReference,
            ]
        );
        let negation = count.arms.iter().find(|arm| arm.line == 26);
        // A complement with an outcome of its own is a rule: no lint.
        assert_eq!(negation.and_then(|arm| arm.complement), None);
        assert_eq!(negation.map(|arm| &arm.verdict), Some(&Verdict::Counts));
    }

    #[test]
    fn a_missing_vector_fails_the_whole_arm_and_a_later_same_outcome_counts() {
        let spec = "### A\n- x → y [V1, V9]\n- z → y [V2]\n";
        let count = count(spec);
        assert_eq!(count.total().arms, 1);
        let verdicts: Vec<_> = count.arms.iter().map(|arm| arm.verdict.clone()).collect();
        assert_eq!(
            verdicts,
            [Verdict::Missing(vec![String::from("V9")]), Verdict::Counts]
        );
    }

    #[test]
    fn a_recorded_frame_counts_only_when_a_vector_comes_from_it() {
        // V2 comes from rec:10: another frame of the row, or a range missing it, holds nothing.
        let spec = "### A\n- a → b [rec:10]\n- c → d [rec:7-12]\n- e → f [rec:11]\n- g → h \
                    [rec:3-9]\n- i → j [other:10]\n- k → l [rec]\n- m → n [rec:12-7]\n";
        let count = count(spec);
        assert_eq!(count.total().arms, 2);
        let missing: Vec<_> = count
            .arms
            .iter()
            .filter_map(|arm| {
                if let Verdict::Missing(cites) = &arm.verdict {
                    Some(cites.join(","))
                } else {
                    None
                }
            })
            .collect();
        assert_eq!(
            missing,
            ["rec:11", "rec:3-9", "other:10", "rec", "rec:12-7"]
        );
    }

    #[test]
    fn original_only_and_record_only_are_uncited_and_listed() {
        let spec = "### A\n- a → b [original only]\n- c → d [record only: r02:37-38, \
                    r07:263]\n- e → f [V1; original only]\n";
        let count = count(spec);
        let verdicts: Vec<_> = count.arms.iter().map(|arm| arm.verdict.clone()).collect();
        assert_eq!(
            verdicts,
            [
                Verdict::Uncited(Unchecked::OriginalOnly),
                Verdict::Uncited(Unchecked::RecordOnly(vec![
                    String::from("r02:37-38"),
                    String::from("r07:263")
                ])),
                Verdict::Counts,
            ]
        );
        let findings = count.findings();
        assert!(
            findings.contains("A:2 `a`: original only, counts 0"),
            "{findings}"
        );
        assert!(
            findings.contains("A:3 `c`: record only (r02:37-38, r07:263), counts 0"),
            "{findings}"
        );
        assert_eq!(count.total().arms, 1);
    }

    #[test]
    fn operators_count_only_in_counted_arms_and_definitions() {
        assert_eq!(count("### A\n- `v < 5` → b [V1]\n").total().ops, 1);
        // An arm that counts 0, for any reason, counts no operator.
        for zero in [
            "### A\n- `v < 5` → b [original only]\n",
            "### A\n- `v < 5` → b\n",
            "### A\n- `v < 5` → b [Nope]\n",
            "### A\n- `v < 5` → as `B`\n### B\n",
        ] {
            assert_eq!(count(zero).total().ops, 0, "{zero}");
        }
        // An arm merged into one with the same outcome is part of it: its formulas count.
        let merged = "### A\n- `v < 5` → b [V1]\n- `w > 6` → b [V2]\n";
        assert_eq!(count(merged).total().ops, 2);
        // A definition bullet counts; an explanation, a field or a paragraph doesn't.
        assert_eq!(count("### A\n- Her edge is `x + rx`.\n").total().ops, 1);
        for explanation in [
            "### A\n- So her edge is `x + rx`.\n",
            "### A\n- As in `x + rx`.\n",
            "### A\n- For example `x + rx`.\n",
            "### A\n- params: `x + rx`.\n",
            "### A\n\nHer edge is `x + rx`.\n",
        ] {
            assert_eq!(count(explanation).total().ops, 0, "{explanation}");
        }
    }

    #[test]
    fn a_mirrored_formula_counts_once() {
        let edges = [
            "x + d + rx − 1",
            "x + d − rx",
            "y + d + ry − 1",
            "y + d - ry",
        ]
        .map(String::from);
        // Right and down are one, left and up the other.
        assert_eq!(entry_operators(&edges), 3 + 2);
        // The bars count as operators, as `operators` reads them.
        let axes = ["|x − ex| < rx + erx", "|y − ey| < ry + ery"].map(String::from);
        assert_eq!(entry_operators(&axes), 5);
        let sides = ["a + b ≤ top", "a − b ≥ bottom"].map(String::from);
        assert_eq!(entry_operators(&sides), 2);
        let different = ["x + 1", "x × 2"].map(String::from);
        assert_eq!(entry_operators(&different), 2);
        // `max` is an operator, not a name to mirror.
        let named = ["max(x, y) + 1", "min(y, x) − 1"].map(String::from);
        assert_eq!(entry_operators(&named), 2);
    }

    #[test]
    fn a_restated_subexpression_counts_nothing() {
        // Before or after the formula it restates, and also as a mirror (`x + rx`).
        let gap = [
            "y + ry",
            "ey − ery",
            "gap = (ey − ery) − (y + ry)",
            "x + rx",
            "ex − erx",
        ]
        .map(String::from);
        assert_eq!(entry_operators(&gap), 4);
        let after = ["gap = (ey − ery) − (y + ry)", "y + ry"].map(String::from);
        assert_eq!(entry_operators(&after), 4);
        // Only within one entry.
        let spec = "### A\n- `gap = (ey − ery) − (y + ry)` → b [V1]\n### B\n- `y + ry` → c [V1]\n";
        assert_eq!(count(spec).total().ops, 5);
        // Tokens, not text: `x + 10` doesn't restate `x + 1`.
        let digits = ["x + 1", "x + 10"].map(String::from);
        assert_eq!(entry_operators(&digits), 2);
        // Only a whole operand restates: `bottom ≥ top` is no part of `bottom ≥ top − 1`, and
        // `a + b` no part of `a + b × c`, but `a − b` is part of `a − b + c`.
        let partial = ["bottom ≥ top", "bottom ≥ top − 1", "top − 1"].map(String::from);
        assert_eq!(entry_operators(&partial), 3);
        let tighter = ["a + b", "a + b × c"].map(String::from);
        assert_eq!(entry_operators(&tighter), 3);
        let looser = ["a − b", "b + c", "a − b + c"].map(String::from);
        assert_eq!(entry_operators(&looser), 3);
    }

    #[test]
    fn arms_and_formulas_outside_an_entry_fail() {
        let spec = "# Spec\n## 1. Intro\n- a frame → `speed + dash × way` [V1]\n- So `a + b`.\n\
                    ### 1.2 Memory\n- `p − 1` is her last.\n### A\n- c → d [V1]\n";
        let count = count(spec);
        assert!(count.fails());
        assert_eq!(count.total().units().thirds(), 3);
        assert_eq!(
            count.arms.first().map(|arm| &arm.verdict),
            Some(&Verdict::OutsideEntry)
        );
        assert_eq!(
            count.strays,
            [StrayFormula {
                block: String::from("1.2 Memory"),
                line: 6,
                formula: String::from("p − 1"),
            }]
        );
        let findings = count.findings();
        assert!(findings.contains("an arm outside any entry"), "{findings}");
        assert!(
            findings.contains("1.2 Memory:6 `p − 1`: a formula outside"),
            "{findings}"
        );
        assert!(!self::count("### A\n- c → d [V1]\n").fails());
    }

    #[test]
    fn the_else_lint_flags_repeated_complements_and_null_outcomes() {
        let spec = "### A\n- running → she gains [V1]\n- not running → she keeps [V2]\n- any \
                    other speed → she gains [V3]\n- standing → nothing happens [V1]\n- sloped \
                    → no slope shortens the move [V1]\n- shortened → her speeds don't change \
                    [V2]\n- landing → no push: she grows [V3]\n";
        let count = count(spec);
        let complements: Vec<_> = count.arms.iter().map(|arm| arm.complement).collect();
        assert_eq!(
            complements,
            [
                None,
                None,
                Some(Complement { of: 2, repeats: 2 }),
                None,
                None,
                None,
                None
            ]
        );
        let nulls: Vec<_> = count.arms.iter().map(|arm| arm.null_outcome).collect();
        assert_eq!(nulls, [false, false, false, true, true, true, true]);
        let findings = count.findings();
        assert!(
            findings.contains("the complement of line 2 with line 2's outcome"),
            "{findings}"
        );
        assert!(
            findings.contains("a null outcome (\"nothing happens\")"),
            "{findings}"
        );
        // A complement with an outcome of its own is a rule (`WordwiseTopSpeed`'s "any other
        // speed"), as is a negation with its own outcome.
        let own = self::count(
            "### A\n- `wordwise`, small → keeps [V1]\n- any other speed → takes the top [V2]\n",
        );
        assert!(own.arms.iter().all(|arm| arm.complement.is_none()));
    }

    #[test]
    fn a_bold_label_is_not_part_of_the_condition() {
        let count = count(
            "### A
- **Ceiling.** Then, her top edge → she is pushed down [V1]
",
        );
        let conditions: Vec<_> = count
            .arms
            .iter()
            .map(|arm| arm.condition.as_str())
            .collect();
        assert_eq!(conditions, ["then, her top edge"]);
    }

    #[test]
    fn rewording_and_explaining_move_nothing() {
        let plain = "### A\n- she runs → she speeds up [V1]\n- checked: oracle (V1)\n";
        let explained = "### A\n\nWhy she speeds up: the pad.\n\n- Mechanic. A long note about \
                         running, with no arrow.\n- running with the pad held → she speeds up\n  \
                         by the step. [V1]\n- So she reaches her top speed.\n- checked: oracle \
                         (V1) and more words\n";
        assert_eq!(thirds(plain), 3);
        assert_eq!(thirds(explained), 3);
    }

    #[test]
    fn splitting_an_arm_with_one_outcome_moves_nothing() {
        let joined = "### A\n- standing or crouched → she halts [V1, V2]\n";
        let split = "### A\n- standing → she halts [V1]\n- crouched → she halts [V2]\n";
        assert_eq!(thirds(joined), thirds(split));
        let other_entry =
            "### A\n- standing → she halts [V1]\n### B\n- crouched → she halts [V2]\n";
        assert_eq!(thirds(other_entry), 6);
    }

    #[test]
    fn cell_terms_count_once_per_spec() {
        let spec = "### A\nCells: kept, gun drawn\n### B\n- Cells: `kept`; striding, Gun  drawn.\n";
        let count = count(spec);
        let cells: Vec<_> = count.blocks.iter().map(|block| block.tally.cells).collect();
        assert_eq!(cells, [0, 2, 1]);
    }

    #[test]
    fn operators_skip_signs_and_hyphenated_names() {
        assert_eq!(operators("x + d + rx − 1"), 3);
        assert_eq!(operators("-4.E000"), 0);
        assert_eq!(operators("C-wall"), 0);
        assert_eq!(operators("min(a, b) + 1"), 2);
        assert_eq!(operators("a − -b"), 1);
        assert_eq!(operators("(y + 1) >> 4 ≤ top"), 3);
        assert_eq!(operators("x - 1"), 1);
    }

    #[test]
    fn names_values_and_paths_are_not_formulas() {
        let spec = "### A\n- See `scenarios/vectors/a.toml`, `-4.E000` and `x+1`; `y + 1` is.\n";
        assert_eq!(count(spec).total().ops, 1);
    }

    #[test]
    fn arrows_inside_formulas_and_table_rows_are_not_arms() {
        let spec = "### A\n- the map `a → b` holds\n| x → y | z |\n";
        assert_eq!(count(spec).arms.len(), 0);
    }

    #[test]
    fn an_areas_file_replaces_the_notes_areas_and_old_code_doesnt_bound_the_budget() {
        let spec = "### A\n- a → b [V1]\n### B\n- c → d [V1]\n";
        let budget = "k = 24 weighted tokens per spec unit\nArea one: A\n";
        let vectors = Vectors::parse(VECTORS);
        let areas = "- Area left (old 20): A, B\n";
        let count = Count::of(spec, &vectors, Some(budget), Some(areas));
        let names: Vec<_> = count.areas.iter().map(|area| area.name.as_str()).collect();
        assert_eq!(names, ["left"]);
        assert_eq!(count.areas.first().and_then(|area| area.old), Some(20));
        assert_eq!(count.budget(), Some(48));
        assert_eq!(count.budget_line(), "Budget: 24 × 2.0 units = 48 (left 48)");
    }

    #[test]
    fn units_print_to_a_tenth() {
        assert_eq!(Units::of(0, 1, 0).to_string(), "0.3");
        assert_eq!(Units::of(1, 1, 1).to_string(), "1.7");
        assert_eq!(Units::of(271, 0, 0).to_string(), "271.0");
        assert_eq!(Units::of(271, 0, 0).budget(PerUnit::tenths(240)), 6504);
        assert_eq!(Units::of(193, 1, 0).budget(PerUnit::tenths(360)), 6960);
    }

    #[test]
    fn k_and_the_areas_are_read_from_the_notes_budget_section() {
        let count = demo();
        assert_eq!(count.k, Some(PerUnit::tenths(240)));
        let names: Vec<_> = count.areas.iter().map(|area| area.name.as_str()).collect();
        assert_eq!(names, ["moving", "falling", "unassigned"]);
        assert!(!count.spec_states_budget);
        let budget = "## Budget\n\nk = 36 weighted tokens per spec unit, frozen.\n\nArea a: A\n";
        let count = count_with("### A\n- x → y [V1]\n", budget);
        assert_eq!(count.k, Some(PerUnit::tenths(360)));
        assert_eq!(count.budget(), Some(36));
        assert_eq!(count.budget_line(), "Budget: 36 × 1.0 units = 36 (a 36)");
        // k may be stated to one decimal, and the budget rounds down.
        let decimal = "## Budget\n\nk = 37.5 weighted tokens per spec unit, frozen.\n\nArea a: A\n";
        let count = count_with("### A\n- x → y [V1]\n", decimal);
        assert_eq!(count.k, Some(PerUnit::tenths(375)));
        assert_eq!(count.budget_line(), "Budget: 37.5 × 1.0 units = 37 (a 37)");
        // Another number written as "k = " is not the budget's k.
        assert_eq!(count_with("### A\n- x → y [V1]\n", "k = 3 cells\n").k, None);
    }

    #[test]
    fn the_budget_section_runs_to_the_next_section() {
        let section = budget_section(NOTES);
        assert!(section.starts_with("## Budget\n"));
        assert!(section.contains("Area falling"));
        assert!(!section.contains("ignored") && !section.contains("99"));
        assert_eq!(budget_section("# Notes\n\n## Sources\n"), "");
        assert_eq!(budget_section("## Budgets\n"), "");
    }

    #[test]
    fn a_spec_stating_k_or_an_area_is_a_finding_and_sets_nothing() {
        let spec =
            "## Header\n\nk = 36 weighted tokens per spec unit\n\nArea a: A\n### A\n- x → y [V1]\n";
        let count = count(spec);
        assert!(count.spec_states_budget);
        assert_eq!(count.k, None);
        let names: Vec<_> = count.areas.iter().map(|area| area.name.as_str()).collect();
        assert_eq!(names, ["unassigned"]);
        // The same area in the notes counts.
        let count = count_with("### A\n- x → y [V1]\n", "Area a: A\n");
        assert!(!count.spec_states_budget);
        assert_eq!(count.areas.len(), 1);
    }

    #[test]
    fn without_k_the_units_count_and_the_budget_is_a_dash() {
        let count = count_with("### A\n- x → y [V1]\n", "Area a: A\n");
        assert_eq!(count.k, None);
        assert_eq!(count.budget(), None);
        assert_eq!(count.areas.first().and_then(super::AreaCount::budget), None);
        assert_eq!(
            count.budget_line(),
            "Budget: no k in the decode notes' Budget section (MODEL §1 decision 7), 1.0 units"
        );
    }

    #[test]
    fn the_budget_line_splits_by_area() {
        assert_eq!(
            demo().budget_line(),
            "Budget: 24 × 6.0 units = 144 (moving 64, falling 56, unassigned 24)"
        );
    }

    #[test]
    fn entries_without_checked_and_unknown_items_are_found() {
        let spec = "### A\n- a → b [V1]\n- checked: oracle\n### B\n";
        let count = count_with(spec, "Area one: A, Gone\n");
        let unchecked: Vec<_> = count
            .blocks
            .iter()
            .filter(|block| block.entry && !block.checked)
            .map(|block| block.heading.as_str())
            .collect();
        assert_eq!(unchecked, ["B"]);
        assert_eq!(
            count.unknown_items,
            [(String::from("one"), String::from("Gone"))]
        );
        assert_eq!(count.unassigned, Some(1));
    }

    #[test]
    fn a_change_prints_signed() {
        assert_eq!(delta(14, 23, |thirds| Units { thirds }), "+3.0");
        assert_eq!(delta(7_000, 6_504, thousands), "-496");
        assert_eq!(thousands(1_234_567), "1,234,567");
    }
}
