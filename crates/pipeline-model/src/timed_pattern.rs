//! Source-owned timed instruction scope shared by generation and verification.
use anyhow::{Context, Result, ensure};
use serde::{Deserialize, Serialize};
use std::collections::BTreeSet;

#[derive(Clone, Debug, Deserialize, Serialize, PartialEq, Eq)]
#[serde(deny_unknown_fields)]
pub struct Declaration {
    pub schema_version: u32,
    pub execution_modes: Vec<String>,
    pub decimal_states: Vec<bool>,
    pub timing_tests: Vec<String>,
    pub timing_oracles: Vec<String>,
    pub timing_sources: Vec<String>,
    pub hardware_conditions: Vec<String>,
    pub exclusions: Vec<String>,
    pub patterns: Vec<Pattern>,
    #[serde(default)]
    pub prospective_patterns: Vec<Pattern>,
}

#[derive(Clone, Debug, Deserialize, Serialize, PartialEq, Eq)]
#[serde(deny_unknown_fields)]
pub struct Pattern {
    pub opcode: String,
    pub mnemonic: String,
    pub mode: String,
    pub addressing: String,
    pub mx: Vec<u8>,
    pub corpus: String,
    pub required_conditions: Vec<String>,
    pub independent_fixtures: Vec<String>,
    pub timing_branches: Vec<String>,
    pub inapplicable: Vec<String>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub timing_tests: Vec<String>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub timing_oracles: Vec<String>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub hardware_conditions: Vec<String>,
}

fn declared_set(values: &[String], field: &str) -> Result<BTreeSet<String>> {
    let set = values.iter().cloned().collect::<BTreeSet<_>>();
    ensure!(
        !set.is_empty() && set.len() == values.len() && values.iter().all(|value| !value.is_empty()),
        "empty or duplicate declaration values: {field}"
    );
    Ok(set)
}

impl Pattern {
    pub fn opcode_byte(&self) -> Result<u8> {
        ensure!(self.opcode.len() == 2, "invalid declared opcode");
        u8::from_str_radix(&self.opcode, 16).context("invalid declared opcode")
    }

    fn validate(&self) -> Result<()> {
        self.opcode_byte()?;
        ensure!(
            [&self.mnemonic, &self.mode, &self.addressing].iter().all(|value| !value.is_empty()),
            "pattern description incomplete: {}",
            self.opcode
        );
        let widths = self.mx.iter().copied().collect::<BTreeSet<_>>();
        ensure!(
            !widths.is_empty() && widths.len() == self.mx.len() && widths.iter().all(|mx| *mx < 4),
            "invalid declared M/X widths"
        );
        ensure!(
            self.corpus == format!("{}.n.json", self.opcode),
            "declared corpus must cover complete available native opcode file"
        );
        let conditions = declared_set(&self.required_conditions, "required_conditions")?;
        declared_set(&self.independent_fixtures, "independent_fixtures")?;
        declared_set(&self.inapplicable, "inapplicable")?;
        let branches = declared_set(&self.timing_branches, "timing_branches")?;
        let mandatory = widths
            .iter()
            .map(|mx| format!("native-mx-{mx}"))
            .chain(["binary", "decimal", "program-bank-wrap"].map(str::to_owned))
            .collect::<BTreeSet<_>>();
        ensure!(mandatory.is_subset(&conditions), "pattern declaration omits applicable mode/boundary conditions");
        ensure!(
            !branches.contains("m8") || branches.contains("m16") || widths.iter().all(|mx| *mx >= 2),
            "pattern declaration omits M16 timing branch"
        );
        ensure!(
            !branches.contains("x8") || branches.contains("x16") || widths.iter().all(|mx| mx % 2 == 1),
            "pattern declaration omits X16 timing branch"
        );
        Ok(())
    }
}

impl Declaration {
    pub fn validate(&self) -> Result<()> {
        ensure!(self.schema_version == 2 && !self.patterns.is_empty(), "pattern declaration version/scope missing");
        ensure!(
            self.execution_modes == ["native"]
                && self.decimal_states.len() == 2
                && self.decimal_states.contains(&false)
                && self.decimal_states.contains(&true),
            "pattern declaration execution modes incomplete"
        );
        for (name, values) in [
            ("timing_tests", &self.timing_tests),
            ("timing_oracles", &self.timing_oracles),
            ("timing_sources", &self.timing_sources),
            ("hardware_conditions", &self.hardware_conditions),
            ("exclusions", &self.exclusions),
        ] {
            declared_set(values, name)?;
        }
        for patterns in [&self.patterns, &self.prospective_patterns] {
            let mut seen = BTreeSet::new();
            for pattern in patterns {
                pattern.validate()?;
                ensure!(seen.insert(pattern.opcode_byte()?), "duplicate declared opcode");
            }
        }
        for pattern in &self.prospective_patterns {
            declared_set(&pattern.timing_tests, "prospective timing_tests")?;
            declared_set(&pattern.timing_oracles, "prospective timing_oracles")?;
            declared_set(&pattern.hardware_conditions, "prospective hardware_conditions")?;
        }
        Ok(())
    }

    pub fn select(mut self, prospective: Option<&str>) -> Result<Self> {
        self.validate()?;
        if let Some(opcode) = prospective {
            let candidates =
                self.prospective_patterns.iter().filter(|pattern| pattern.opcode == opcode).collect::<Vec<_>>();
            ensure!(candidates.len() == 1, "unknown prospective pattern; independent implementation decision required");
            let selected = candidates[0].clone();
            self.timing_tests = selected.timing_tests.clone();
            self.timing_oracles = selected.timing_oracles.clone();
            self.hardware_conditions = selected.hardware_conditions.clone();
            self.patterns = vec![selected];
        }
        self.validate()?;
        Ok(self)
    }
}
