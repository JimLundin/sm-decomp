//! Spec units and the size budget they set, for `tools spec-units` (MODEL §1 decision 7, §5's
//! "Rules are arms").
//!
//! A domain's budget is k = 24 weighted tokens per spec unit, per area, each area capped at 70% of
//! its old code where that figure is given. The units are counted from the spec's structure, so
//! rewording or explaining a rule moves nothing:
//! - an **arm** is a bullet holding `→` (outside backticks): `<conditions> → <outcome> [vectors]`.
//!   It is one unit. Arms of one entry with the same outcome count once. An arm counts only if it
//!   cites vectors and each of them exists in the vector file; a recorded frame, `row:frame` or
//!   `row:first-last`, exists when a vector comes from that row. An arm whose outcome is only a
//!   reference to other entries (`` as `Landing` ``) is a cross-reference and counts nothing;
//! - a **cell term** is one term a `Cells:` line names, a third of a unit, counted once per spec;
//! - an **operator** in a backticked formula is a third of a unit, each distinct formula counted
//!   once per entry. A formula spaces its operators (`x + d − rx`): a span with no space is a
//!   name, a value or a path and counts nothing. A `+`, `-` or `−` with no operand before it is a
//!   sign; an ASCII `-` between two letters or digits is part of a name (`C-wall`).
//!
//! Table rows, parameters, `checked:` bullets, prose and the vector file don't count.
//!
//! **Areas.** A line `Area <name> (old <tokens>): <item>, <item>, ...`, as a paragraph or a
//! bullet, in the spec's header or in the `--areas` file, which replaces the spec's. An item names
//! an entry (`SlopesCarry`) or any other heading, by its whole text or its leading number (`1.2`
//! for "1.2 What she remembers"). `(old N)` is the area's old-code weighted score, which sets the
//! cap; `--baseline AREA=N` overrides it. A heading no area names, holding units, is reported
//! unassigned.
//!
//! **Findings**, for the spec review: arms with an earlier arm's outcome, arms that cite no vector
//! or one the file doesn't hold, cross-references, an arm whose condition is an earlier one's with
//! `not` in front (still counted: the review decides), a bullet with two `→`, entries without a
//! `checked` bullet, and area items that name no heading.
//!
//! **A spec commit's change.** With the parent's spec beside it,
//! `git show HEAD~1:docs/behaviours/D.md > old.md`, the command line
//! `tools spec-units docs/behaviours/D.md scenarios/vectors/D.toml --against old.md` prints the
//! units and the budget before and after; the header's `Budget:` line is the report's.

use std::collections::{BTreeSet, HashMap, HashSet};
use std::fmt::{self, Write as _};
use std::fs;

/// The command line `run` takes.
pub const USAGE: &str =
    "tools spec-units SPEC VECTORS [--areas FILE] [--baseline AREA=TOKENS]... [--against OLD_SPEC]";

/// Weighted tokens per unit (MODEL §1 decision 7; frozen).
const K: u64 = 24;
/// The cap on an area's budget, in tenths of its old code.
const CAP_TENTHS: u64 = 7;
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

    /// k × the units, in weighted tokens: k per unit is k / 3 per third.
    #[must_use]
    pub const fn budget(self) -> u64 {
        self.thirds.saturating_mul(K / 3)
    }
}

impl fmt::Display for Units {
    /// To one decimal, rounded half up.
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let tenths = self.thirds.saturating_mul(20).saturating_add(3) / 6;
        write!(f, "{}.{}", tenths / 10, tenths % 10)
    }
}

/// What a vector file holds that an arm can cite: vector ids, and the rows recorded vectors come
/// from.
#[derive(Debug, Default)]
pub struct Vectors {
    ids: HashSet<String>,
    rows: HashSet<String>,
}

impl Vectors {
    /// The `id = "..."` and `source = "<row>:<frame>"` lines of a vector file.
    #[must_use]
    pub fn parse(toml: &str) -> Self {
        let mut vectors = Self::default();
        for line in toml.lines() {
            if let Some(id) = string_field(line, "id") {
                vectors.ids.insert(id.to_owned());
            } else if let Some((row, _)) =
                string_field(line, "source").and_then(|source| source.split_once(':'))
            {
                vectors.rows.insert(row.to_owned());
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

    /// Whether `cite` is a vector id, or a frame (`row:N` or `row:N-M`) of a recorded row.
    fn holds(&self, cite: &str) -> bool {
        if self.ids.contains(cite) {
            return true;
        }
        cite.rsplit_once(':').is_some_and(|(row, frames)| {
            let digits = |part: &str| !part.is_empty() && part.chars().all(|c| c.is_ascii_digit());
            let frames_ok = match frames.split_once('-') {
                Some((first, last)) => digits(first) && digits(last),
                None => digits(frames),
            };
            frames_ok && self.rows.contains(row)
        })
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
                let digits: String = old
                    .trim_end()
                    .strip_suffix(')')?
                    .trim()
                    .strip_prefix("old")?
                    .chars()
                    .filter(|c| *c != ',' && !c.is_whitespace())
                    .collect();
                (name, Some(digits.parse().ok()?))
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
    ["Area ", "Cells:", "Budget:"]
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

/// How an arm was counted.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Verdict {
    /// One unit.
    Counts,
    /// The same outcome as an earlier counted arm of the entry, at that line: nothing.
    SameOutcome(usize),
    /// It cites no vector: nothing.
    Uncited,
    /// It cites these, which the vector file doesn't hold: nothing.
    Missing(Vec<String>),
    /// Its outcome only names other entries: nothing.
    CrossReference,
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
    /// The line of an earlier arm of its entry whose condition this one negates.
    pub negates: Option<usize>,
    /// Whether its bullet holds more than one `→`.
    pub chained: bool,
}

/// What one arm bullet says, before it is judged.
struct Stated<'a> {
    condition: &'a str,
    outcome: &'a str,
    cites: Vec<&'a str>,
    chained: bool,
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
            let cites = bracket
                .trim_start_matches('[')
                .split([',', ';', ' '])
                .map(|cite| cite.trim_matches(['(', ')', '`']))
                .filter(|cite| !cite.is_empty())
                .collect();
            (outcome, cites)
        }
        None => (trimmed, Vec::new()),
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
}

impl AreaCount {
    /// k × its units, before the cap.
    #[must_use]
    pub const fn uncapped(&self) -> u64 {
        self.tally.units().budget()
    }

    /// 70% of its old code, if given.
    #[must_use]
    pub fn cap(&self) -> Option<u64> {
        self.old.map(|old| old.saturating_mul(CAP_TENTHS) / 10)
    }

    /// Its budget: k × its units, capped.
    #[must_use]
    pub fn budget(&self) -> u64 {
        let uncapped = self.uncapped();
        self.cap().map_or(uncapped, |cap| cap.min(uncapped))
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
}

impl Count {
    /// Counts `spec` against `vectors`. Areas come from `areas` when given, else from the spec's
    /// own `Area` lines; `baselines` override their old-code figures.
    #[must_use]
    pub fn of(
        spec: &str,
        vectors: &Vectors,
        areas: Option<&str>,
        baselines: &[(String, u64)],
    ) -> Self {
        let blocks = split_blocks(spec);
        let mut defs =
            areas.map_or_else(|| area_defs(&blocks), |file| area_defs(&split_blocks(file)));
        for (name, old) in baselines {
            if let Some(def) = defs.iter_mut().find(|def| def.name == *name) {
                def.old = Some(*old);
            } else {
                defs.push(AreaDef {
                    name: name.clone(),
                    old: Some(*old),
                    items: Vec::new(),
                });
            }
        }
        let entries: HashSet<&str> = blocks
            .iter()
            .filter(|block| block.entry)
            .map(|block| block.heading.as_str())
            .collect();
        let mut seen_cells = BTreeSet::new();
        let mut arms = Vec::new();
        let mut counts: Vec<BlockCount> = blocks
            .iter()
            .map(|block| {
                let (tally, checked) =
                    count_block(block, vectors, &entries, &mut seen_cells, &mut arms);
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
        }
    }

    /// Everything the spec holds.
    #[must_use]
    pub fn total(&self) -> Tally {
        self.areas
            .iter()
            .fold(Tally::default(), |sum, area| sum.add(area.tally))
    }

    /// The domain's budget: each area's, summed.
    #[must_use]
    pub fn budget(&self) -> u64 {
        self.areas
            .iter()
            .fold(0, |sum, area| sum.saturating_add(area.budget()))
    }

    /// The header's budget line.
    #[must_use]
    pub fn budget_line(&self) -> String {
        let capped = self
            .areas
            .iter()
            .any(|area| area.budget() < area.uncapped());
        let split: Vec<String> = self
            .areas
            .iter()
            .map(|area| {
                let cap = if area.budget() < area.uncapped() {
                    " capped"
                } else {
                    ""
                };
                format!("{} {}{cap}", area.name, thousands(area.budget()))
            })
            .collect();
        format!(
            "Budget: {K} × {} units{} = {} ({})",
            self.total().units(),
            if capped { ", capped at 70% of old" } else { "" },
            thousands(self.budget()),
            split.join(", ")
        )
    }

    /// The report: units and budget by area, the budget line, the findings, and units by
    /// heading.
    #[must_use]
    pub fn report(&self) -> String {
        let mut out = String::from(
            "area                 entries  arms cells   ops   units    k×U    cap budget\n",
        );
        for area in &self.areas {
            let cap = area.cap().map_or_else(|| String::from("-"), thousands);
            let _ = writeln!(
                out,
                "{:<20} {:>7} {:>5} {:>5} {:>5} {:>7} {:>6} {:>6} {:>6}",
                area.name,
                area.entries,
                area.tally.arms,
                area.tally.cells,
                area.tally.ops,
                area.tally.units().to_string(),
                thousands(area.uncapped()),
                cap,
                thousands(area.budget())
            );
        }
        let total = self.total();
        let entries = self.areas.iter().map(|area| area.entries).sum::<usize>();
        let _ = writeln!(
            out,
            "{:<20} {:>7} {:>5} {:>5} {:>5} {:>7} {:>6} {:>6} {:>6}\n\n{}\n",
            "total",
            entries,
            total.arms,
            total.cells,
            total.ops,
            total.units().to_string(),
            thousands(total.units().budget()),
            "",
            thousands(self.budget()),
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
                Verdict::Uncited => {
                    let _ = writeln!(out, "{at}: cites no vector, counts 0");
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
            if let Some(earlier) = arm.negates {
                let _ = writeln!(out, "{at}: negates the arm at line {earlier}");
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
        for (area, item) in &self.unknown_items {
            let _ = writeln!(out, "  area {area}: {item} names no heading");
        }
        if out.len() == start {
            out.push_str("  none\n");
        }
        out
    }
}

/// The `Area` lines among the blocks' pieces.
fn area_defs(blocks: &[Block]) -> Vec<AreaDef> {
    blocks
        .iter()
        .flat_map(|block| &block.pieces)
        .filter_map(|piece| AreaDef::parse(&piece.text))
        .collect()
}

/// One block's tally and whether it has a `checked` bullet; its arms go to `arms`.
fn count_block(
    block: &Block,
    vectors: &Vectors,
    entries: &HashSet<&str>,
    seen_cells: &mut BTreeSet<String>,
    arms: &mut Vec<Arm>,
) -> (Tally, bool) {
    let mut tally = Tally::default();
    let mut checked = false;
    let mut formulas = HashSet::new();
    let mut outcomes: HashMap<String, usize> = HashMap::new();
    let mut conditions: Vec<(String, usize)> = Vec::new();
    for piece in &block.pieces {
        let text = piece.text.trim_start_matches("**");
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
        if piece.bullet && text.to_lowercase().starts_with("checked") {
            checked = true;
            continue;
        }
        for (outside, formula) in spans(text) {
            let formula = formula.split_whitespace().collect::<Vec<_>>().join(" ");
            // A span without a space is a name, a value or a path, never a formula.
            if !outside && formula.contains(' ') && formulas.insert(formula.clone()) {
                tally.ops = tally.ops.saturating_add(operators(&formula));
            }
        }
        let Some(arm) = piece.bullet.then(|| stated(text)).flatten() else {
            continue;
        };
        let condition = normal(arm.condition);
        let outcome = normal(arm.outcome);
        let missing: Vec<String> = arm
            .cites
            .iter()
            .filter(|cite| !vectors.holds(cite))
            .map(|cite| (*cite).to_owned())
            .collect();
        let verdict = if cross_reference(arm.outcome, entries) {
            Verdict::CrossReference
        } else if arm.cites.is_empty() {
            Verdict::Uncited
        } else if !missing.is_empty() {
            Verdict::Missing(missing)
        } else if let Some(first) = outcomes.get(&outcome) {
            Verdict::SameOutcome(*first)
        } else {
            outcomes.insert(outcome.clone(), piece.line);
            tally.arms = tally.arms.saturating_add(1);
            Verdict::Counts
        };
        let negates = conditions
            .iter()
            .find(|(earlier, _)| negation(&condition, earlier))
            .map(|(_, line)| *line);
        conditions.push((condition.clone(), piece.line));
        arms.push(Arm {
            block: block.heading.clone(),
            line: piece.line,
            condition,
            outcome,
            verdict,
            negates,
            chained: arm.chained,
        });
    }
    (tally, checked)
}

/// Whether one condition is the other with `not` or `no` in front.
fn negation(a: &str, b: &str) -> bool {
    let negated = |x: &str, y: &str| {
        ["not ", "no ", "otherwise not ", "unless "]
            .iter()
            .any(|word| x.strip_prefix(word) == Some(y))
    };
    negated(a, b) || negated(b, a)
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

/// `--areas FILE`, `--baseline AREA=N` (repeatable) and `--against OLD_SPEC`.
#[derive(Default)]
struct Options {
    areas: Option<String>,
    baselines: Vec<(String, u64)>,
    against: Option<String>,
}

fn options(args: &[String]) -> Result<Options, String> {
    let mut options = Options::default();
    let mut rest = args;
    while let [flag, value, tail @ ..] = rest {
        match flag.as_str() {
            "--areas" => options.areas = Some(read(value)?),
            "--against" => options.against = Some(read(value)?),
            "--baseline" => {
                let (name, old) = value.rsplit_once('=').ok_or_else(|| USAGE.to_owned())?;
                let old = old
                    .replace(',', "")
                    .parse()
                    .map_err(|e| format!("{value}: {e}"))?;
                options.baselines.push((name.to_owned(), old));
            }
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
/// The usage on a wrong command line, or the path and the error when a file can't be read.
pub fn run(args: &[String]) -> Result<String, String> {
    let [spec, vectors, rest @ ..] = args else {
        return Err(USAGE.to_owned());
    };
    let options = options(rest)?;
    let vectors = Vectors::parse(&read(vectors)?);
    let count = Count::of(
        &read(spec)?,
        &vectors,
        options.areas.as_deref(),
        &options.baselines,
    );
    let mut out = format!(
        "spec units: {spec}, {} vectors (k = {K}, cap 70% of old)\n\n{}",
        vectors.len(),
        count.report()
    );
    if let Some(old) = options.against {
        let before = Count::of(&old, &vectors, options.areas.as_deref(), &options.baselines);
        let (was, now) = (before.total().units(), count.total().units());
        let units = delta(was.thirds(), now.thirds(), |thirds| Units { thirds });
        let _ = writeln!(
            out,
            "\nchange: units {was} → {now} ({units}), budget {} → {} ({})",
            thousands(before.budget()),
            thousands(count.budget()),
            delta(before.budget(), count.budget(), thousands)
        );
    }
    Ok(out)
}

#[cfg(test)]
mod tests {
    use super::{Count, Units, Vectors, Verdict, delta, operators, thousands};

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

    const SPEC: &str = "# Demo

## Header

Area moving (old 1,000): Walk, 1.2
Area falling: `Fall`

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
- in the air → gravity adds `0.5 × t` [rec:12]
- not in the air → she stays put [V2]
- checked: constructed

### Unnamed
- a rule nobody placed → something [V1]
";

    fn count(spec: &str) -> Count {
        Count::of(spec, &Vectors::parse(VECTORS), None, &[])
    }

    fn thirds(spec: &str) -> u64 {
        count(spec).total().units().thirds()
    }

    #[test]
    fn arms_cells_and_operators_make_the_units() {
        let count = count(SPEC);
        let [moving, falling, unassigned] = count.areas.as_slice() else {
            panic!("three areas: {:?}", count.areas);
        };
        assert_eq!(
            (moving.tally.arms, moving.tally.cells, moving.tally.ops),
            (2, 2, 3)
        );
        assert_eq!(moving.tally.units().thirds(), 11);
        assert_eq!(moving.uncapped(), 88);
        assert_eq!(moving.cap(), Some(700));
        assert_eq!(moving.entries, 1);
        assert_eq!((falling.tally.arms, falling.tally.ops), (2, 1));
        assert_eq!(falling.budget(), 56);
        assert_eq!(unassigned.tally.arms, 1);
        assert_eq!(count.unassigned, Some(2));
        assert_eq!(count.budget(), 88 + 56 + 24);
    }

    #[test]
    fn verdicts_say_why_an_arm_counts_nothing() {
        let count = count(SPEC);
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
                Verdict::SameOutcome(13),
                Verdict::Missing(vec![String::from("Nope")]),
                Verdict::Uncited,
                Verdict::CrossReference,
            ]
        );
        let negation = count.arms.iter().find(|arm| arm.line == 29);
        assert_eq!(negation.and_then(|arm| arm.negates), Some(28));
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
    fn recorded_frames_count_when_their_row_is_recorded() {
        let spec = "### A\n- a → b [rec:3-9]\n- c → d [rec:7]\n- e → f [other:7]\n- g → h [rec]\n";
        assert_eq!(count(spec).total().arms, 2);
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
    fn an_areas_file_replaces_the_spec_areas_and_baselines_cap() {
        let spec = "Area one: A\n### A\n- a → b [V1]\n### B\n- c → d [V1]\n";
        let vectors = Vectors::parse(VECTORS);
        let areas = "- Area left (old 20): A, B\n";
        let baselines = [(String::from("left"), 30)];
        let count = Count::of(spec, &vectors, Some(areas), &baselines);
        let names: Vec<_> = count.areas.iter().map(|area| area.name.as_str()).collect();
        assert_eq!(names, ["left"]);
        assert_eq!(count.budget(), 21);
        assert_eq!(
            count.budget_line(),
            "Budget: 24 × 2.0 units, capped at 70% of old = 21 (left 21 capped)"
        );
    }

    #[test]
    fn units_print_to_a_tenth() {
        assert_eq!(Units::of(0, 1, 0).to_string(), "0.3");
        assert_eq!(Units::of(1, 1, 1).to_string(), "1.7");
        assert_eq!(Units::of(271, 0, 0).to_string(), "271.0");
        assert_eq!(Units::of(271, 0, 0).budget(), 6504);
    }

    #[test]
    fn the_budget_line_splits_by_area() {
        assert_eq!(
            count(SPEC).budget_line(),
            "Budget: 24 × 7.0 units = 168 (moving 88, falling 56, unassigned 24)"
        );
    }

    #[test]
    fn entries_without_checked_and_unknown_items_are_found() {
        let spec = "Area one: A, Gone\n### A\n- a → b [V1]\n- checked: oracle\n### B\n";
        let count = count(spec);
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
