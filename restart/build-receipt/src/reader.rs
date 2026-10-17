//! Authentication of retained receipt bytes; external input completeness and
//! sole-writer/process supervision remain independent acceptance requirements.
use super::*;

/// Authenticate every retained receipt reference and decode required fields.
/// This establishes retention integrity, not independent compiler correctness.
pub fn authenticate(receipt: &Path) -> Result<BuildReceipt> {
    let receipt = canonical(receipt)?;
    let directory = receipt.parent().ok_or("receipt parent missing")?;
    let value = sm_binary_evidence::decode(&sm_binary_evidence::read_bytes(&receipt)?)?;
    schema(&value, "sm-build-receipt-v1")?;
    if field(&value, "inputs_unchanged")?.as_bool() != Some(true) {
        return Err("receipt input stability missing".into());
    }
    let mut budget = Budget { count: 0, bytes: 0, paths: BTreeMap::new() };
    let pre_path = reference(field(&value, "pre")?, directory, &mut budget)?;
    let pre = decode(&pre_path)?;
    schema(&pre, "sm-build-inputs-v1")?;
    let environment = field(&pre, "environment")?.as_table().ok_or("pre environment must be a table")?;
    validate_environment(
        environment
            .iter()
            .map(|(key, value)| Ok((key.as_str(), value.as_str().ok_or("environment value must be a string")?))),
    )?;
    for name in ["argv", "graph_argv"] {
        let args = field(&pre, name)?.as_array().ok_or("argv must be an array")?;
        if args.is_empty() || args.iter().any(|value| value.as_str().is_none()) {
            return Err("invalid argv".into());
        }
    }
    if field(&pre, "target_initially_absent")?.as_bool() != Some(true) {
        return Err("fresh target observation missing".into());
    }
    for name in ["cargo", "cwd"] {
        absolute(Path::new(string(field(&pre, name)?)?))?;
    }
    nonnegative(field(&pre, "started_unix_ms")?)?;
    for name in ["graph", "graph_stderr"] {
        reference(field(&pre, name)?, directory, &mut budget)?;
    }
    shards(field(&pre, "inputs")?, directory, &mut budget, true)?;
    let build = decode(&reference(field(&value, "build")?, directory, &mut budget)?)?;
    schema(&build, "sm-build-observation-v1")?;
    if field(&build, "success")?.as_bool() != Some(true) {
        return Err("build success missing".into());
    }
    let codes = field(&build, "exit_code")?.as_array().ok_or("exit code must be an array")?;
    if codes.len() != 1 || codes[0].as_integer() != Some(0) {
        return Err("successful build exit code must be zero".into());
    }
    let start = nonnegative(field(&build, "started_unix_ms")?)?;
    let end = nonnegative(field(&build, "ended_unix_ms")?)?;
    if end < start {
        return Err("build timestamps reversed".into());
    }
    match field(&build, "elapsed_seconds")? {
        Value::Float(seconds) if seconds.is_finite() && *seconds >= 0.0 => (),
        _ => return Err("invalid build elapsed seconds".into()),
    }
    for name in ["stdout", "stderr"] {
        reference(field(&build, name)?, directory, &mut budget)?;
    }
    shards(field(&value, "generated")?, directory, &mut budget, true)?;
    let deps = field(&value, "dependency_info")?.as_array().ok_or("dependency info must be an array")?;
    if deps.is_empty() || deps.len() > 256 {
        return Err("invalid dependency info count".into());
    }
    for dep in deps {
        reference(dep, directory, &mut budget)?;
    }
    let dll = reference(field(&value, "dll")?, directory, &mut budget)?;
    let dll_sha256 = string(field(field(&value, "dll")?, "sha256")?)?.to_owned();
    if dll.file_name().and_then(|name| name.to_str()) != Some(format!("{dll_sha256}.dll").as_str()) {
        return Err("retained DLL filename does not bind its hash".into());
    }
    string(field(&value, "closure_scope")?)?;
    Ok(BuildReceipt { receipt, dll, dll_sha256 })
}

struct Budget {
    count: usize,
    bytes: u64,
    paths: BTreeMap<PathBuf, Identity>,
}

fn decode(path: &Path) -> Result<Value> {
    sm_binary_evidence::decode(&sm_binary_evidence::read_bytes(path)?)
}

fn field<'a>(value: &'a Value, name: &str) -> Result<&'a Value> {
    value.get(name).ok_or_else(|| format!("required field {name} missing"))
}
fn string(value: &Value) -> Result<&str> {
    value.as_str().ok_or("string field required".into())
}
fn nonnegative(value: &Value) -> Result<u64> {
    u64::try_from(value.as_integer().ok_or("integer field required")?).map_err(|_| "negative extent/timestamp".into())
}
fn schema(value: &Value, expected: &str) -> Result<()> {
    if string(field(value, "schema")?)? == expected { Ok(()) } else { Err("unsupported receipt schema".into()) }
}
fn identity(value: &Value) -> Result<Identity> {
    let size = nonnegative(field(value, "size")?)?;
    let sha256 = string(field(value, "sha256")?)?;
    if size > MAX_TOTAL_BYTES
        || sha256.len() != 64
        || !sha256.bytes().all(|byte| byte.is_ascii_digit() || (b'a'..=b'f').contains(&byte))
    {
        return Err("invalid reference identity".into());
    }
    Ok(Identity { size, sha256: sha256.into() })
}

fn referenced_path(value: &Value, key: &str, directory: &Path, budget: &mut Budget) -> Result<PathBuf> {
    let path = Path::new(string(field(value, key)?)?);
    absolute(path)?;
    // Every ancestor in retained output must remain a plain directory.
    let resolved = canonical(path)?;
    if !resolved.starts_with(directory) {
        return Err("reference escapes receipt directory".into());
    }
    if resolved != path {
        return Err("receipt reference must use its canonical path".into());
    }
    let mut ancestor = resolved.as_path();
    while ancestor != directory {
        if reparse(&fs::symlink_metadata(ancestor).map_err(|e| error(ancestor, e))?) {
            return Err("reparse receipt reference".into());
        }
        ancestor = ancestor.parent().ok_or("reference parent missing")?;
    }
    let expected = identity(value)?;
    if let Some(previous) = budget.paths.get(&resolved) {
        if previous != &expected {
            return Err("conflicting reference identities".into());
        }
        return Ok(resolved);
    }
    budget.count += 1;
    budget.bytes = budget.bytes.checked_add(expected.size).ok_or("reference extent overflow")?;
    if budget.count > MAX_VISITS + 1024 || budget.bytes > MAX_TOTAL_BYTES * 3 {
        return Err("receipt reference budget exceeded".into());
    }
    if inspect(&resolved)? != expected {
        return Err(error(&resolved, "reference authentication failed"));
    }
    budget.paths.insert(resolved.clone(), expected);
    Ok(resolved)
}
fn reference(value: &Value, directory: &Path, budget: &mut Budget) -> Result<PathBuf> {
    referenced_path(value, "path", directory, budget)
}
fn shards(value: &Value, directory: &Path, budget: &mut Budget, required: bool) -> Result<()> {
    let shards = value.as_array().ok_or("inventory shards must be an array")?;
    if (required && shards.is_empty()) || shards.len() > MAX_FILES.div_ceil(SHARD_FILES) {
        return Err("invalid inventory shard count".into());
    }
    let mut originals = BTreeSet::new();
    for shard in shards {
        let records = decode(&reference(shard, directory, budget)?)?;
        let records = records.as_array().ok_or("inventory records must be an array")?;
        if records.is_empty() || records.len() > SHARD_FILES {
            return Err("invalid inventory record count".into());
        }
        for record in records {
            let original = Path::new(string(field(record, "path")?)?);
            absolute(original)?;
            if !originals.insert(original.to_path_buf()) || originals.len() > MAX_FILES {
                return Err("duplicate/over-budget inventory path".into());
            }
            referenced_path(record, "snapshot", directory, budget)?;
        }
    }
    Ok(())
}
