use crate::readiness_metadata::toml_value_decode;
use crate::readiness_run::identity;
use anyhow::{Context, Result, ensure};
use pipeline_model::timed_pattern::Declaration;
use serde::Deserialize;
use std::{collections::BTreeSet, fs, path::Path};
use toml::Value;
pub const DECLARATION: &str = "crates/recomp/src/timed_patterns.toml";
pub const FIXTURES: &str = "tools/pattern_fixtures.toml";
#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct Fixtures {
    schema_version: u32,
    cases: Vec<Fixture>,
}
#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct Fixture {
    name: String,
    initial: CpuState,
    #[serde(rename = "final")]
    final_state: CpuState,
    cycles: Vec<Cycle>,
    coverage: Vec<String>,
}
use pipeline_model::timed_corpus::{Cycle, State as CpuState};
#[doc = " Native fixtures use explicit cycle tables; absent bus values are omitted."]
#[doc = " The corpus runner's in-memory adapter keeps the external oracle row shape."]
pub fn fixtures(path: &Path) -> Result<Vec<pipeline_model::timed_corpus::Case>> {
    let fixtures: Fixtures = pipeline_model::read(path)?;
    ensure!(fixtures.schema_version == 1 && !fixtures.cases.is_empty(), "independent fixture schema/scope missing");
    let mut names = BTreeSet::new();
    let mut cases = Vec::with_capacity(fixtures.cases.len());
    for fixture in fixtures.cases {
        ensure!(
            !fixture.name.is_empty() && names.insert(fixture.name.clone()) && !fixture.cycles.is_empty(),
            "empty/duplicate independent fixture or cycles"
        );
        for state in [&fixture.initial, &fixture.final_state] {
            ensure!(
                state.e == 0 && state.ram.iter().all(|(address, _)| *address <= 0xffffff),
                "independent fixture mode/address invalid"
            );
            let addresses = state.ram.iter().map(|(address, _)| address).collect::<BTreeSet<_>>();
            ensure!(addresses.len() == state.ram.len(), "duplicate independent fixture memory address");
        }
        ensure!(
            fixture.cycles.iter().all(|cycle| cycle.address <= 0xffffff && !cycle.pins.is_empty()),
            "independent fixture cycle invalid"
        );
        cases.push(pipeline_model::timed_corpus::Case {
            name: fixture.name,
            initial: fixture.initial,
            fin: fixture.final_state,
            cycles: fixture.cycles,
            case_sha256: String::new(),
            opcode: String::new(),
            conditions: fixture.coverage,
            independent_fixture: true,
            body: 0,
            bytes: Vec::new(),
        });
    }
    Ok(cases)
}
pub fn strings(value: &Value) -> Result<Vec<String>> {
    value
        .as_array()
        .context("string list missing")?
        .iter()
        .map(|v| Ok(v.as_str().context("string expected")?.to_owned()))
        .collect()
}
fn set(value: &Value) -> Result<BTreeSet<String>> {
    let values = strings(value)?;
    let set = values.iter().cloned().collect::<BTreeSet<_>>();
    ensure!(set.len() == values.len(), "duplicate declared value");
    Ok(set)
}
pub fn declaration(path: &Path, prospective: Option<&str>) -> Result<Value> {
    let declaration: Declaration = pipeline_model::read(path)?;
    Ok(toml::Value::try_from(declaration.select(prospective)?)?)
}
pub fn load(root: &Path, override_path: Option<&Path>, prospective: Option<&str>) -> Result<Value> {
    let raw = fs::read_to_string(root.join(DECLARATION))?;
    if let Some(path) = override_path {
        ensure!(fs::read_to_string(path)? == raw, "pattern declaration differs from production source declaration");
    }
    let declaration: Declaration = toml::from_str(&raw).context("invalid native timed pattern declaration")?;
    Ok(toml::Value::try_from(declaration.select(prospective)?)?)
}
pub fn validate_declaration(declaration: &Value) -> Result<()> {
    let declaration: Declaration = toml_value_decode(declaration.clone())?;
    declaration.validate()
}
pub fn conditions(case: &pipeline_model::timed_corpus::Case, pattern: &Value) -> Result<Vec<String>> {
    let state = &case.initial;
    let mx = (state.p >> 4) & 3;
    let decimal = state.p & 8 != 0;
    let mut conditions = [
        format!("native-mx-{mx}"),
        if decimal { "decimal" } else { "binary" }.into(),
        if mx & 2 != 0 { "m8" } else { "m16" }.into(),
        if mx & 1 != 0 { "x8" } else { "x16" }.into(),
        "fixed".into(),
        format!("mx{mx}-{}", if decimal { "decimal" } else { "binary" }),
    ]
    .into_iter()
    .collect::<BTreeSet<_>>();
    let length = case
        .cycles
        .iter()
        .filter(|row| row.value.is_some() && (row.pins.starts_with("dp-") || row.pins.starts_with("-p-")))
        .count()
        .max(1);
    if usize::from(state.pc) + length > 0xffff {
        conditions.insert("program-bank-wrap".into());
    }
    conditions.extend(case.conditions.iter().cloned());
    let mut declared = set(&pattern["required_conditions"])?;
    declared.extend(set(&pattern["timing_branches"])?);
    Ok(conditions.intersection(&declared).cloned().collect())
}
pub fn validate_report(report: &Value, timing: bool) -> Result<()> {
    let declaration = &report["declaration"];
    validate_declaration(declaration)?;
    ensure!(
        identity(declaration)? == crate::documents::field(report, "declaration_sha256")?
            && !crate::documents::field(report, "source_sha256")?.is_empty(),
        "pattern declaration/source identity incomplete"
    );
    ensure!(
        ["complete production declaration", "complete prospective declaration"]
            .contains(&crate::documents::field(report, "scope")?),
        "development pattern scope cannot establish acceptance"
    );
    if timing {
        let oracle_names =
            report["oracles"].as_table().context("timing oracles missing")?.keys().cloned().collect::<BTreeSet<_>>();
        ensure!(
            report["returncode"] == crate::readiness_metadata::value(&0).expect("validated native metadata")
                && oracle_names == set(&declaration["timing_oracles"])?
                && report["sources"] == declaration["timing_sources"]
                && report["tests"] == declaration["timing_tests"]
                && report["conditions"]["applicable"] == declaration["hardware_conditions"],
            "declared independent timing scope incomplete"
        );
        return Ok(());
    }
    ensure!(
        report["corpus_verified"] == crate::readiness_metadata::value(&true).expect("validated native metadata")
            && !report.get("corpus_archive_sha256").is_none()
            && report["corpus_archive_sha256"] == report["pinned_dependency"]["sha256"],
        "pattern complete pinned corpus verification missing"
    );
    let patterns = crate::documents::array(declaration, "patterns")?;
    let opcodes = report["opcodes"].as_table().context("opcode counts missing")?;
    ensure!(opcodes.len() == patterns.len(), "pattern declared opcode coverage incomplete");
    let expected = report["expected"].as_integer().and_then(|n| u64::try_from(n).ok()).context("case count missing")?;
    ensure!(
        expected > 0
            && report["run"] == crate::readiness_metadata::value(&expected).expect("validated native metadata")
            && report["passed"] == crate::readiness_metadata::value(&expected).expect("validated native metadata")
            && report["failed"] == crate::readiness_metadata::value(&0).expect("validated native metadata")
            && report["skipped"] == crate::readiness_metadata::value(&0).expect("validated native metadata"),
        "pattern complete case count missing"
    );
    let corpus = crate::documents::array(report, "corpus")?;
    ensure!(corpus.len() == patterns.len(), "pattern complete corpus identities missing");
    let mut total = 0;
    for pattern in patterns {
        let opcode = crate::documents::field(pattern, "opcode")?;
        let item = corpus
            .iter()
            .filter(|c| c["opcode"] == crate::readiness_metadata::value(&opcode).expect("validated native metadata"))
            .collect::<Vec<_>>();
        ensure!(item.len() == 1, "pattern complete corpus identity missing");
        let count =
            item[0]["cases"].as_integer().and_then(|n| u64::try_from(n).ok()).context("corpus cases missing")?
                + crate::documents::array(pattern, "independent_fixtures")?.len() as u64;
        let counts = &report["opcodes"][opcode];
        ensure!(
            count > 0
                && ["expected", "run", "passed"]
                    .iter()
                    .all(|f| counts[f] == crate::readiness_metadata::value(&count).expect("validated native metadata"))
                && counts["failed"] == crate::readiness_metadata::value(&0).expect("validated native metadata")
                && counts["skipped"] == crate::readiness_metadata::value(&0).expect("validated native metadata")
                && !crate::documents::field(item[0], "sha256")?.is_empty(),
            "pattern complete opcode corpus/fixtures missing"
        );
        for condition in
            strings(&pattern["required_conditions"])?.into_iter().chain(strings(&pattern["timing_branches"])?)
        {
            let row = &report["coverage"][opcode][&condition];
            ensure!(
                row["expected"].as_integer().and_then(|n| u64::try_from(n).ok()).is_some_and(|n| n > 0)
                    && row["passed"] == row["expected"],
                "pattern declared condition/branch missing: {opcode}/{condition}"
            );
        }
        ensure!(
            counts["independent_fixtures"] == pattern["independent_fixtures"],
            "pattern independent coverage substitution incomplete"
        );
        total += count;
    }
    ensure!(total == expected, "pattern aggregate count differs");
    Ok(())
}
#[cfg(test)]
mod tests {
    use super::*;
    fn root() -> &'static Path {
        Path::new(env!("CARGO_MANIFEST_DIR")).parent().unwrap().parent().unwrap()
    }
    #[test]
    fn native_declaration_preserves_complete_scope_and_rejects_missing_width_conditions() {
        let production = declaration(&root().join(DECLARATION), None).unwrap();
        assert_eq!(production["patterns"].as_array().unwrap().len(), 7);
        let prospective = declaration(&root().join(DECLARATION), Some("ad")).unwrap();
        assert_eq!(prospective["patterns"].as_array().unwrap().len(), 1);
        assert_eq!(prospective["patterns"][0]["opcode"], toml::Value::try_from("ad").unwrap());
        assert_eq!(
            prospective["timing_tests"],
            toml::Value::try_from(crate::report!(vec!["prospective_lda_timing"])).unwrap()
        );
        assert!(declaration(&root().join(DECLARATION), Some("69")).is_err());
        let mut reduced = production;
        reduced["patterns"][0]["required_conditions"] = crate::report!(vec!["binary", "decimal", "program-bank-wrap"]);
        assert!(validate_declaration(&reduced).is_err());
    }
    #[test]
    fn native_fixtures_preserve_inactive_cycles_and_literal_boundaries() {
        let cases = fixtures(&root().join(FIXTURES)).unwrap();
        assert_eq!(cases.len(), 19);
        let lda = cases.iter().find(|case| case.name == "boundary LDA8 preserves A high across FFFF").unwrap();
        assert_eq!(lda.initial.pc, 65535);
        assert_eq!(lda.fin.pc, 1);
        assert_eq!(lda.fin.a, 43904);
        assert_eq!(lda.cycles[0], Cycle { address: 8454143, value: Some(169), pins: "dp-r---".into() });
        let sep = cases.iter().find(|case| case.name == "boundary SEP truncates X Y and preserves A").unwrap();
        assert_eq!(sep.cycles[2], Cycle { address: 8388608, value: None, pins: "---r---".into() });
    }
}
