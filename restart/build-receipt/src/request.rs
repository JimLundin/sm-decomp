//! Bounded launch input for the fixed prospective build transaction.
use super::*;

/// Read an SME request without creating outputs or invoking a producer.
/// Schema `sm-build-request-v1` requires checkout/output/cargo absolute strings,
/// environment string table and inputs array of {path, excluded string array}.
/// Unknown fields refuse; no caller-controlled Cargo argv is admitted.
pub fn read_request(path: &Path) -> Result<BuildRequest> {
    absolute(path)?;
    let bytes = sm_binary_evidence::read_bytes(path)?;
    let request = normalized(&decode_request(&bytes)?)?;
    validate(&request)?;
    let resolved = canonical(path)?;
    if !request
        .inputs
        .iter()
        .any(|root| resolved.starts_with(&root.path) && !root.excluded.iter().any(|skip| resolved.starts_with(skip)))
    {
        return Err("build request file must be included in frozen input roots".into());
    }
    if sm_binary_evidence::read_bytes(&resolved)? != bytes {
        return Err("build request changed during admission".into());
    }
    Ok(request)
}

/// Encode a typed request through the same validator as the launch decoder.
/// The caller writes it immutably under a declared input root before admission.
pub fn encode_request(request: &BuildRequest) -> Result<Vec<u8>> {
    validate(request)?;
    let text = |path: &Path| -> Result<Value> { Ok(path.to_str().ok_or("non-UTF8 request path")?.into()) };
    let environment = request
        .environment
        .iter()
        .map(|(key, value)| {
            Ok((
                key.to_str().ok_or("non-UTF8 environment key")?.into(),
                value.to_str().ok_or("non-UTF8 environment value")?.into(),
            ))
        })
        .collect::<Result<Map>>()?;
    let inputs = request
        .inputs
        .iter()
        .map(|root| {
            Ok(Value::Table(Map::from([
                ("path".into(), text(&root.path)?),
                (
                    "excluded".into(),
                    Value::Array(root.excluded.iter().map(|path| text(path)).collect::<Result<Vec<_>>>()?),
                ),
            ])))
        })
        .collect::<Result<Vec<_>>>()?;
    let value = Value::Table(Map::from([
        ("schema".into(), "sm-build-request-v1".into()),
        ("checkout".into(), text(&request.checkout)?),
        ("output".into(), text(&request.output)?),
        ("cargo".into(), text(&request.cargo)?),
        ("environment".into(), environment.into()),
        ("inputs".into(), inputs.into()),
    ]));
    let bytes = sm_binary_evidence::encode(&value)?;
    decode_request(&bytes)?;
    Ok(bytes)
}

pub fn decode_request(bytes: &[u8]) -> Result<BuildRequest> {
    let value = sm_binary_evidence::decode(bytes)?;
    let table = object(&value, "request")?;
    exact(table, &["schema", "checkout", "output", "cargo", "environment", "inputs"])?;
    if string(field(table, "schema")?, "schema")? != "sm-build-request-v1" {
        return Err("unsupported build request schema".into());
    }
    let environment = object(field(table, "environment")?, "environment")?;
    validate_environment(
        environment.iter().map(|(key, value)| Ok((key.as_str(), string(value, "environment value")?))),
    )?;
    let inputs = field(table, "inputs")?.as_array().ok_or("request inputs must be an array")?;
    if inputs.is_empty() || inputs.len() > 64 {
        return Err("input root count must be 1..64".into());
    }
    let roots = inputs
        .iter()
        .map(|value| {
            let root = object(value, "input root")?;
            exact(root, &["path", "excluded"])?;
            let excluded = field(root, "excluded")?.as_array().ok_or("request exclusions must be an array")?;
            if excluded.len() > 64 {
                return Err("exclusion count budget exceeded".into());
            }
            Ok(InputRoot {
                path: path(field(root, "path")?)?,
                excluded: excluded.iter().map(path).collect::<Result<Vec<_>>>()?,
            })
        })
        .collect::<Result<Vec<_>>>()?;
    let request = BuildRequest {
        checkout: path(field(table, "checkout")?)?,
        output: path(field(table, "output")?)?,
        cargo: path(field(table, "cargo")?)?,
        environment: environment
            .iter()
            .map(|(key, value)| Ok((OsString::from(key), OsString::from(string(value, "environment value")?))))
            .collect::<Result<_>>()?,
        inputs: roots,
    };
    // Lexical validation is pure; build() subsequently resolves paths and repeats
    // overlap checks before creating its fresh output directory.
    validate(&request)?;
    Ok(request)
}

fn object<'a>(value: &'a Value, name: &str) -> Result<&'a Map> {
    value.as_table().ok_or_else(|| format!("request {name} must be a table"))
}
fn field<'a>(table: &'a Map, name: &str) -> Result<&'a Value> {
    table.get(name).ok_or_else(|| format!("request field {name} missing"))
}
fn string<'a>(value: &'a Value, name: &str) -> Result<&'a str> {
    value.as_str().ok_or_else(|| format!("request {name} must be a string"))
}
fn path(value: &Value) -> Result<PathBuf> {
    let path = PathBuf::from(string(value, "path")?);
    absolute(&path)?;
    Ok(path)
}
fn exact(table: &Map, names: &[&str]) -> Result<()> {
    if table.keys().any(|key| !names.contains(&key.as_str())) {
        return Err("unknown build request field".into());
    }
    for name in names {
        field(table, name)?;
    }
    Ok(())
}
