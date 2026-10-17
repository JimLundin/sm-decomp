//! Independent field mutations re-seal enclosing references so the targeted
//! typed-reader branch, rather than an outer hash mismatch, determines refusal.
use super::*;

pub(super) fn run(receipt: &Path) -> Result<usize> {
    let directory = receipt.parent().ok_or("fixture parent missing")?;
    let baseline = sm_binary_evidence::decode(&sm_binary_evidence::read_bytes(receipt)?)?;
    authenticate(receipt)?;
    let mut sequence = 0;
    let mut count = 0;
    let groups: &[(&[&str], &[(&str, &str)])] = &[
        (
            &[],
            &[
                ("schema", "string field required"),
                ("pre", "required field path missing"),
                ("build", "required field path missing"),
                ("generated", "inventory shards must be an array"),
                ("dependency_info", "dependency info must be an array"),
                ("dll", "required field path missing"),
                ("inputs_unchanged", "receipt input stability missing"),
                ("closure_scope", "string field required"),
            ],
        ),
        (
            &["pre", "@"],
            &[
                ("schema", "string field required"),
                ("inputs", "inventory shards must be an array"),
                ("environment", "pre environment must be a table"),
                ("argv", "argv must be an array"),
                ("graph_argv", "argv must be an array"),
                ("target_initially_absent", "fresh target observation missing"),
                ("cargo", "string field required"),
                ("cwd", "string field required"),
                ("started_unix_ms", "integer field required"),
                ("graph", "required field path missing"),
                ("graph_stderr", "required field path missing"),
            ],
        ),
        (
            &["build", "@"],
            &[
                ("schema", "string field required"),
                ("success", "build success missing"),
                ("exit_code", "exit code must be an array"),
                ("started_unix_ms", "integer field required"),
                ("ended_unix_ms", "integer field required"),
                ("elapsed_seconds", "invalid build elapsed seconds"),
                ("stdout", "required field path missing"),
                ("stderr", "required field path missing"),
            ],
        ),
        (
            &["dll"],
            &[
                ("path", "string field required"),
                ("size", "integer field required"),
                ("sha256", "string field required"),
            ],
        ),
        (
            &["pre", "@", "inputs", "0", "@", "0"],
            &[
                ("path", "string field required"),
                ("snapshot", "string field required"),
                ("size", "integer field required"),
                ("sha256", "string field required"),
            ],
        ),
    ];
    for (prefix, fields) in groups {
        for (name, wrong_error) in *fields {
            let mut path = prefix.to_vec();
            path.push(name);
            for replacement in [None, Some(Value::Integer(-999))] {
                authenticate(receipt)?;
                let wrong = match replacement {
                    Some(_) if matches!(*name, "size" | "started_unix_ms" | "ended_unix_ms") => {
                        Some(Value::Boolean(false))
                    }
                    other => other,
                };
                let expected =
                    if wrong.is_none() { format!("required field {name} missing") } else { wrong_error.to_string() };
                let candidate = mutate(baseline.clone(), &path, wrong, directory, &mut sequence)?;
                reject(candidate, &expected, directory, &mut sequence)?;
                count += 1;
            }
        }
    }
    let cases: &[(&[&str], Value, &str)] = &[
        (&["schema"], Value::String("unsupported".into()), "unsupported receipt schema"),
        (
            &["build", "@", "exit_code"],
            Value::Array(vec![Value::Integer(1)]),
            "successful build exit code must be zero",
        ),
        (&["build", "@", "exit_code"], Value::Array(Vec::new()), "successful build exit code must be zero"),
        (&["build", "@", "ended_unix_ms"], Value::Integer(1), "build timestamps reversed"),
        (&["build", "@", "elapsed_seconds"], Value::Float(-0.1), "invalid build elapsed seconds"),
        (&["dll", "size"], Value::Integer(-1), "negative extent/timestamp"),
        (&["dll", "sha256"], Value::String("bad".into()), "invalid reference identity"),
        (&["dll", "path"], Value::String("relative.dll".into()), "absolute path required"),
        (&["dependency_info"], Value::Array(Vec::new()), "invalid dependency info count"),
        (&["pre", "@", "argv"], Value::Array(Vec::new()), "invalid argv"),
        (&["pre", "@", "inputs"], Value::Array(Vec::new()), "invalid inventory shard count"),
        (&["pre", "@", "inputs", "0", "@"], Value::Array(Vec::new()), "invalid inventory record count"),
    ];
    for (path, replacement, expected) in cases {
        authenticate(receipt)?;
        let candidate = mutate(baseline.clone(), path, Some(replacement.clone()), directory, &mut sequence)?;
        reject(candidate, expected, directory, &mut sequence)?;
        count += 1;
    }
    let outside = canonical(&directory.parent().ok_or("fixture outside parent")?.join("input/two.rs"))?;
    let candidate = mutate(
        baseline.clone(),
        &["dll", "path"],
        Some(outside.to_str().ok_or("fixture path")?.into()),
        directory,
        &mut sequence,
    )?;
    reject(candidate, "reference escapes receipt directory", directory, &mut sequence)?;
    count += 1;
    // Reusing a reference with a conflicting identity must be refused before
    // treating the second occurrence as already authenticated.
    let mut conflict = baseline.get("pre").ok_or("fixture pre")?.clone();
    conflict.as_table_mut().ok_or("fixture ref")?.insert("size".into(), 0_i64.into());
    let candidate = mutate(baseline.clone(), &["dll"], Some(conflict), directory, &mut sequence)?;
    reject(candidate, "conflicting reference identities", directory, &mut sequence)?;
    count += 1;
    let record_path = ["pre", "@", "inputs", "0", "@"];
    let shard_ref = baseline.get("pre").ok_or("fixture pre")?;
    let pre = sm_binary_evidence::decode(&sm_binary_evidence::read_bytes(Path::new(
        shard_ref.get("path").and_then(Value::as_str).ok_or("fixture pre path")?,
    ))?)?;
    let shard = pre.get("inputs").and_then(Value::as_array).and_then(|items| items.first()).ok_or("fixture shard")?;
    let records = sm_binary_evidence::decode(&sm_binary_evidence::read_bytes(Path::new(
        shard.get("path").and_then(Value::as_str).ok_or("fixture shard path")?,
    ))?)?;
    let record = records.as_array().and_then(|records| records.first()).ok_or("fixture record")?.clone();
    let candidate = mutate(
        baseline.clone(),
        &record_path,
        Some(Value::Array(vec![record.clone(), record])),
        directory,
        &mut sequence,
    )?;
    reject(candidate, "duplicate/over-budget inventory path", directory, &mut sequence)?;
    count += 1;
    let limits = [
        (
            &["dependency_info"][..],
            Value::Array(vec![baseline.get("dll").ok_or("fixture dll")?.clone(); 257]),
            "invalid dependency info count",
        ),
        (
            &["generated"][..],
            Value::Array(vec![shard.clone(); MAX_FILES.div_ceil(SHARD_FILES) + 1]),
            "invalid inventory shard count",
        ),
        (
            &record_path[..],
            Value::Array(vec![
                records.as_array().and_then(|records| records.first()).ok_or("fixture record")?.clone();
                SHARD_FILES + 1
            ]),
            "invalid inventory record count",
        ),
        (&["dll", "size"][..], Value::Integer((MAX_TOTAL_BYTES + 1) as i64), "invalid reference identity"),
    ];
    for (path, replacement, expected) in limits {
        authenticate(receipt)?;
        let candidate = mutate(baseline.clone(), path, Some(replacement), directory, &mut sequence)?;
        reject(candidate, expected, directory, &mut sequence)?;
        count += 1;
    }
    authenticate(receipt)?;
    for environment in [
        Map::new(),
        Map::from([("PATH".into(), "literal-tool-path".into()), ("CARGO_HOME".into(), "literal-cargo-home".into())]),
    ] {
        authenticate(receipt)?;
        let candidate = mutate(
            baseline.clone(),
            &["pre", "@", "environment"],
            Some(Value::Table(environment)),
            directory,
            &mut sequence,
        )?;
        sequence += 1;
        let output = directory.join(format!("environment-valid-{sequence:03}.sme"));
        evidence(&output, candidate)?;
        authenticate(&output)?;
        count += 1;
    }
    let environments = [
        (Map::from([("VALID".into(), Value::Integer(1))]), "environment value must be a string"),
        (Map::from([("".into(), "value".into())]), "invalid environment entry"),
        (Map::from([("BAD=KEY".into(), "value".into())]), "invalid environment entry"),
        (Map::from([("BAD\0KEY".into(), "value".into())]), "invalid environment entry"),
        (Map::from([("VALID".into(), "bad\0value".into())]), "invalid environment entry"),
        (
            Map::from([("Path".into(), "one".into()), ("PATH".into(), "two".into())]),
            "ambiguous Windows environment keys",
        ),
        ((0..257).map(|index| (format!("ENTRY{index:03}"), "x".into())).collect(), "environment entry budget exceeded"),
        (Map::from([("VALID".into(), "x".repeat(32768).into())]), "invalid environment entry"),
        (
            (0..34).map(|index| (format!("ENTRY{index:03}"), "x".repeat(32700).into())).collect(),
            "aggregate environment budget exceeded",
        ),
    ];
    for (environment, expected) in environments {
        authenticate(receipt)?;
        let candidate = mutate(
            baseline.clone(),
            &["pre", "@", "environment"],
            Some(Value::Table(environment)),
            directory,
            &mut sequence,
        )?;
        reject(candidate, expected, directory, &mut sequence)?;
        count += 1;
    }
    authenticate(receipt)?;
    Ok(count)
}

fn reject(candidate: Value, expected: &str, directory: &Path, sequence: &mut usize) -> Result<()> {
    *sequence += 1;
    let path = directory.join(format!("field-candidate-{sequence:03}.sme"));
    evidence(&path, candidate)?;
    let error = authenticate(&path).err().ok_or("field mutation unexpectedly accepted")?;
    if !error.contains(expected) {
        return Err(format!("field control expected {expected:?}, observed {error:?}"));
    }
    Ok(())
}

fn mutate(
    mut value: Value,
    path: &[&str],
    replacement: Option<Value>,
    directory: &Path,
    sequence: &mut usize,
) -> Result<Value> {
    let Some((head, tail)) = path.split_first() else {
        return replacement.ok_or("cannot remove document root".into());
    };
    if *head == "@" {
        let filename = value.get("path").and_then(Value::as_str).ok_or("fixture reference path")?;
        let document = sm_binary_evidence::decode(&sm_binary_evidence::read_bytes(Path::new(filename))?)?;
        let changed = mutate(document, tail, replacement, directory, sequence)?;
        *sequence += 1;
        return evidence(&directory.join(format!("field-document-{sequence:03}.sme")), changed);
    }
    if let Some(table) = value.as_table_mut() {
        if tail.is_empty() {
            match replacement {
                Some(value) => {
                    table.insert((*head).into(), value);
                }
                None => {
                    table.remove(*head);
                }
            }
        } else {
            let child = table.get(*head).ok_or("fixture mutation key missing")?.clone();
            table.insert((*head).into(), mutate(child, tail, replacement, directory, sequence)?);
        }
    } else if let Some(array) = value.as_array_mut() {
        let index = head.parse::<usize>().map_err(|e| e.to_string())?;
        let child = array.get(index).ok_or("fixture mutation index missing")?.clone();
        let changed = mutate(child, tail, replacement, directory, sequence)?;
        *array.get_mut(index).ok_or("fixture mutation index missing")? = changed;
    } else {
        return Err("fixture mutation container missing".into());
    }
    Ok(value)
}
