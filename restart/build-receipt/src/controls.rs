//! Focused filesystem controls shared by ordinary tests and the admitted DLL.
use super::*;

/// Execute controls in a fresh caller-selected directory. This does not invoke
/// Cargo or prove actual build acceptance. Cleanup removes only files created by
/// these controls; the caller retains external logs and DLL identity separately.
pub fn focused_controls(directory: &Path) -> Result<usize> {
    absolute(directory)?;
    if directory.exists() {
        return Err("control directory must be fresh".into());
    }
    let parent = canonical(directory.parent().ok_or("control parent missing")?)?;
    let directory = parent.join(directory.file_name().ok_or("control name missing")?);
    fs::create_dir(&directory).map_err(|e| error(&directory, e))?;
    let resolved = canonical(&directory)?;
    if resolved != directory || resolved.parent() != Some(parent.as_path()) {
        return Err("control directory does not resolve to its fresh owned child".into());
    }
    let result = controls(&directory);
    // The directory is canonical-parent + literal caller filename, fresh at
    // entry, and solely owned here. Never remove an aliased caller input tree.
    let cleanup = if canonical(&directory)? != resolved
        || reparse(&fs::symlink_metadata(&directory).map_err(|e| error(&directory, e))?)
    {
        Err("control directory changed identity before cleanup".into())
    } else {
        fs::remove_dir_all(&resolved).map_err(|e| error(&resolved, e))
    };
    match (result, cleanup) {
        (Ok(count), Ok(())) => Ok(count),
        (Err(e), Ok(())) => Err(e),
        (result, Err(e)) => Err(format!("controls {result:?}; cleanup failed: {e}")),
    }
}

fn require(condition: bool, message: &str) -> Result<()> {
    if condition { Ok(()) } else { Err(format!("control failed: {message}")) }
}

fn controls(directory: &Path) -> Result<usize> {
    #[cfg(windows)]
    for (input, expected) in [
        (r"\\?\C:\source with spaces\target", r"C:\source with spaces\target"),
        (r"\\?\UNC\server\share\target", r"\\server\share\target"),
        (r"C:\ordinary\target", r"C:\ordinary\target"),
        (r"\\?\C:\Unicode å\target", r"C:\Unicode å\target"),
    ] {
        require(cargo_path(Path::new(input))? == expected, "Cargo path spelling")?;
    }
    #[cfg(windows)]
    require(cargo_path(Path::new(r"\\?\Volume{123}\target")).is_err(), "unsupported verbatim Cargo path")?;
    let input = directory.join("input");
    fs::create_dir(&input).map_err(|e| error(&input, e))?;
    let first = input.join("one file.rs");
    let second = input.join("two.rs");
    immutable_bytes(&first, b"first literal")?;
    immutable_bytes(&second, b"different literal")?;
    let roots = vec![InputRoot { path: input.clone(), excluded: Vec::new() }];
    let original = inventory(&roots)?;
    require(original.len() == 2 && original[&first] != original[&second], "distinct literal identities")?;
    require(inventory(&roots)? == original, "stable inventory")?;
    fs::write(&first, b"other literal").map_err(|e| error(&first, e))?;
    require(inventory(&roots)? != original, "same-size content mutation")?;
    fs::write(&first, b"first literal").map_err(|e| error(&first, e))?;
    let added = input.join("added.rs");
    immutable_bytes(&added, b"added")?;
    require(inventory(&roots)? != original, "added member")?;
    fs::remove_file(&added).map_err(|e| error(&added, e))?;
    fs::remove_file(&second).map_err(|e| error(&second, e))?;
    require(inventory(&roots)? != original, "deleted member")?;
    immutable_bytes(&second, b"different literal")?;
    require(immutable_bytes(&first, b"overwrite").is_err(), "immutable retry")?;
    let snapshot = directory.join("snapshot.bin");
    retain(&first, &snapshot, &original[&first])?;
    let snapshot_inventory = BTreeMap::from([(snapshot.clone(), original[&first].clone())]);
    unchanged(&snapshot_inventory)?;
    fs::write(&snapshot, b"tampered").map_err(|e| error(&snapshot, e))?;
    require(unchanged(&snapshot_inventory).is_err(), "retained snapshot mutation")?;
    let mut escaped = first.to_str().ok_or("nonUTF8 fixture")?.replace('\\', "\\\\").replace(' ', "\\ ");
    escaped.push_str(" \\\n");
    escaped.push_str(&second.to_str().ok_or("nonUTF8 fixture")?.replace('\\', "\\\\"));
    let rule = format!("target: {escaped}\n");
    require(
        dependency_paths(rule.as_bytes(), directory)? == BTreeSet::from([canonical(&first)?, canonical(&second)?]),
        "spaces/backslashes/continuation",
    )?;
    require(dependency_paths(b"not a rule", directory).is_err(), "malformed rule")?;
    require(dependency_paths(b"target: \n", directory).is_err(), "empty dependency list")?;
    require(dependency_paths(b"target: nonexistent.rs", directory).is_err(), "missing dependency")?;
    require(dependency_paths(b"target: one\\", directory).is_err(), "trailing escape")?;
    let dollar = input.join("cash$#.rs");
    immutable_bytes(&dollar, b"dollar hash literal")?;
    require(
        dependency_paths(b"target: input/cash$$\\#.rs", directory)? == BTreeSet::from([canonical(&dollar)?]),
        "dollar and hash escapes",
    )?;
    require(
        dependency_paths(b"literal-target: input/one\\ file.rs \\\r\n input/two.rs", directory)?
            == BTreeSet::from([canonical(&first)?, canonical(&second)?]),
        "independent escaped-space CRLF literal",
    )?;
    for rule in
        [b"target: input/one\\ file.rs\\\ninput/two.rs".as_slice(), b"target: input/one\\ file.rs\\\r\ninput/two.rs"]
    {
        require(
            dependency_paths(rule, directory)? == BTreeSet::from([canonical(&first)?, canonical(&second)?]),
            "continuation separates adjacent tokens",
        )?;
    }
    immutable_bytes(&input.join("left"), b"split left decoy")?;
    immutable_bytes(&directory.join("right.rs"), b"split right decoy")?;
    for space in ['\u{00a0}', '\u{2003}'] {
        let unicode = input.join(format!("left{space}right.rs"));
        immutable_bytes(&unicode, b"whole Unicode filename")?;
        require(
            dependency_paths(format!("target: input/left{space}right.rs").as_bytes(), directory)?
                == BTreeSet::from([canonical(&unicode)?]),
            "Unicode filename is one dependency",
        )?;
    }
    let mut visits = MAX_VISITS;
    require(walk(&roots[0], &input, 0, &mut BTreeSet::new(), &mut visits).is_err(), "entry budget")?;
    require(walk(&roots[0], &input, MAX_DEPTH + 1, &mut BTreeSet::new(), &mut 0).is_err(), "depth budget")?;
    let request = BuildRequest {
        checkout: directory.into(),
        output: directory.join("output"),
        cargo: first,
        environment: BTreeMap::new(),
        inputs: roots,
    };
    validate(&request)?;
    let mut wrong = request.clone();
    wrong.inputs.push(wrong.inputs[0].clone());
    require(validate(&wrong).is_err(), "duplicate root")?;
    wrong = request.clone();
    wrong.output = input.join("receipt");
    require(validate(&wrong).is_err(), "output/input overlap")?;
    wrong = request.clone();
    wrong.environment.insert("Path".into(), "a".into());
    wrong.environment.insert("PATH".into(), "b".into());
    require(validate(&wrong).is_err(), "ambiguous Windows environment")?;
    wrong = request;
    wrong.environment.insert("BAD=KEY".into(), "value".into());
    require(validate(&wrong).is_err(), "invalid environment name")?;
    let encoded = sm_binary_evidence::encode(&Value::Table(Map::from([("count".into(), 20_i64.into())])))?;
    require(
        sm_binary_evidence::decode(&encoded)?.get("count").and_then(Value::as_integer) == Some(20),
        "maintained codec roundtrip",
    )?;
    let reader_count = reader_controls(directory)?;
    producer_controls(directory)?;
    let measurement_count = measurement_controls(directory)?;
    independent_hash_controls(directory)?;
    let streaming_count = streaming_hash_controls(directory)?;
    let request_count = request_controls(directory)?;
    Ok(40 + reader_count + streaming_count + request_count + measurement_count + if cfg!(windows) { 5 } else { 0 })
}

fn request_controls(directory: &Path) -> Result<usize> {
    let root = directory.join("request-input");
    fs::create_dir(&root).map_err(|e| error(&root, e))?;
    let output = directory.join("request-output");
    let text = |path: &Path| -> Result<Value> { Ok(path.to_str().ok_or("fixture path")?.into()) };
    let baseline = Map::from([
        ("schema".into(), "sm-build-request-v1".into()),
        ("checkout".into(), text(directory)?),
        ("output".into(), text(&output)?),
        ("cargo".into(), text(&root.join("cargo.exe"))?),
        ("environment".into(), Value::Table(Map::from([("PATH".into(), "literal-path".into())]))),
        (
            "inputs".into(),
            Value::Array(vec![Value::Table(Map::from([
                ("path".into(), text(&root)?),
                ("excluded".into(), Value::Array(Vec::new())),
            ]))]),
        ),
    ]);
    let decode =
        |table: Map| -> Result<BuildRequest> { decode_request(&sm_binary_evidence::encode(&Value::Table(table))?) };
    let valid = decode(baseline.clone())?;
    require(
        valid.output == output
            && valid.inputs.len() == 1
            && valid.environment.get(&OsString::from("PATH")) == Some(&OsString::from("literal-path")),
        "valid typed request preserves values",
    )?;
    let mut count = 1;
    let encoded = encode_request(&valid)?;
    require(decode_request(&encoded)?.output == output, "typed request encoder roundtrip")?;
    count += 1;
    immutable_bytes(&root.join("cargo.exe"), b"request fake Cargo identity")?;
    let request_path = root.join("request.sme");
    immutable_bytes(&request_path, &encoded)?;
    require(read_request(&request_path)?.output == output, "included request admission")?;
    count += 1;
    let outside = directory.join("outside-request.sme");
    immutable_bytes(&outside, &encoded)?;
    require(
        read_request(&outside)
            .err()
            .is_some_and(|error| error.contains("build request file must be included in frozen input roots")),
        "undeclared request refuses",
    )?;
    count += 1;
    let excluded_directory = root.join("excluded");
    fs::create_dir(&excluded_directory).map_err(|e| error(&excluded_directory, e))?;
    let mut excluded_request = valid.clone();
    excluded_request.inputs[0].excluded.push(excluded_directory.clone());
    let excluded_path = excluded_directory.join("request.sme");
    immutable_bytes(&excluded_path, &encode_request(&excluded_request)?)?;
    require(
        read_request(&excluded_path)
            .err()
            .is_some_and(|error| error.contains("build request file must be included in frozen input roots")),
        "excluded request refuses",
    )?;
    count += 1;
    for name in ["schema", "checkout", "output", "cargo", "environment", "inputs"] {
        for missing in [true, false] {
            let mut candidate = baseline.clone();
            if missing {
                candidate.remove(name);
            } else {
                candidate.insert(name.into(), Value::Integer(1));
            }
            let expected = if missing {
                format!("request field {name} missing")
            } else {
                match name {
                    "schema" => "request schema must be a string",
                    "environment" => "request environment must be a table",
                    "inputs" => "request inputs must be an array",
                    _ => "request path must be a string",
                }
                .into()
            };
            require(
                decode(candidate).err().is_some_and(|error| error.contains(&expected)),
                "targeted request missing/type refusal",
            )?;
            require(!output.exists(), "request decode does not create outputs")?;
            count += 1;
        }
    }
    let cases = [
        ("schema", "unknown".into(), "unsupported build request schema"),
        ("argv", Value::Array(Vec::new()), "unknown build request field"),
        ("output", "relative".into(), "absolute path required"),
        ("output", format!("{}/../escape", directory.display()).into(), "parent components are forbidden"),
        (
            "environment",
            Value::Table(Map::from([("PATH".into(), Value::Integer(1))])),
            "request environment value must be a string",
        ),
        (
            "environment",
            Value::Table(Map::from([("Path".into(), "one".into()), ("PATH".into(), "two".into())])),
            "ambiguous Windows environment keys",
        ),
        ("inputs", Value::Array(Vec::new()), "input root count must be 1..64"),
        ("inputs", Value::Array(vec![Value::Integer(1)]), "request input root must be a table"),
        (
            "inputs",
            Value::Array(vec![Value::Table(Map::from([("path".into(), text(&root)?)]))]),
            "request field excluded missing",
        ),
        (
            "inputs",
            Value::Array(vec![Value::Table(Map::from([
                ("path".into(), text(&root)?),
                ("excluded".into(), Value::Integer(1)),
            ]))]),
            "request exclusions must be an array",
        ),
        ("output", text(&root.join("receipt"))?, "receipt output overlaps a captured root"),
    ];
    for (name, value, expected) in cases {
        let mut candidate = baseline.clone();
        candidate.insert(name.into(), value);
        let error = decode(candidate).err().ok_or_else(|| format!("request {name} unexpectedly accepted"))?;
        require(error.contains(expected), &format!("request {name} expected {expected:?}, observed {error:?}"))?;
        count += 1;
    }
    Ok(count)
}

fn streaming_hash_controls(directory: &Path) -> Result<usize> {
    let vectors = [
        (0, "e3b0c44298fc1c149afbf4c8996fb92427ae41e4649b934ca495991b7852b855"),
        (1, "084fed08b978af4d7d196a7446a86b58009e636b611db16211b65a9aadff29c5"),
        (65535, "4c32fbc9c2bad24dbe35c62f9aafd3e7dd6d2419f0b4b0a064db2fd79a56b6ab"),
        (65536, "f779dbe3aeaa609beb32212579f3aea441f244621d13c977cb8b52d7ec3016d9"),
        (65537, "5ea07e7659785552529c03843e59b785822a318d92800a44ed2aa1dc4df61400"),
        (131089, "ee4c95c36dbe4e78f0146daf77eb9feaa14f13cc173cab30cc4824724101f75e"),
    ];
    for (size, expected) in vectors {
        let bytes: Vec<u8> = (0..size).map(|index| ((index * 17 + 3) % 256) as u8).collect();
        let path = directory.join(format!("independent-streaming-{size}.bin"));
        immutable_bytes(&path, &bytes)?;
        let identity = inspect(&path)?;
        require(
            identity.size == size as u64 && identity.sha256 == expected,
            "independent streaming SHA256 and lowercase fixed-width hex",
        )?;
    }
    Ok(vectors.len())
}

fn independent_hash_controls(directory: &Path) -> Result<()> {
    let local = directory.join("same-local");
    let dependency = directory.join("same-dependency");
    fs::create_dir(&local).map_err(|e| error(&local, e))?;
    fs::create_dir(&dependency).map_err(|e| error(&dependency, e))?;
    let left = local.join("same.bin");
    let right = dependency.join("same.bin");
    immutable_bytes(&left, b"DLL-A")?;
    immutable_bytes(&right, b"DLL-B")?;
    let entries = inventory(&[
        InputRoot { path: local, excluded: Vec::new() },
        InputRoot { path: dependency, excluded: Vec::new() },
    ])?;
    let expected_left =
        Identity { size: 5, sha256: "bf5798c6113b5e439346d76c48c3e00976cffa490e5724d86bb46f5b4dc86a68".into() };
    let expected_right =
        Identity { size: 5, sha256: "a4d28393ba5565e0fd65c5c881bc6a6241a693c0ed089ab26edbe21d8bb4f9e5".into() };
    require(
        entries.len() == 2
            && entries.get(&left) == Some(&expected_left)
            && entries.get(&right) == Some(&expected_right),
        "independent same-basename literal digest oracle",
    )?;
    let retained = directory.join("literal-dll-retained.bin");
    retain(&left, &retained, &expected_left)?;
    require(inspect(&retained)? == expected_left, "retained independent literal DLL identity")
}

fn status(success: bool) -> std::process::ExitStatus {
    #[cfg(windows)]
    {
        use std::os::windows::process::ExitStatusExt;
        std::process::ExitStatus::from_raw(u32::from(!success))
    }
    #[cfg(not(windows))]
    {
        use std::os::unix::process::ExitStatusExt;
        std::process::ExitStatus::from_raw(if success { 0 } else { 256 })
    }
}

fn producer_controls(directory: &Path) -> Result<()> {
    let checkout = directory.join("checkout");
    fs::create_dir_all(checkout.join("restart")).map_err(|e| error(&checkout, e))?;
    immutable_bytes(&checkout.join("restart/Cargo.toml"), b"literal manifest fixture")?;
    immutable_bytes(&checkout.join("restart/Cargo.lock"), b"literal lock fixture")?;
    let cargo = checkout.join("cargo-fixture.bin");
    immutable_bytes(&cargo, b"not an executable: fake producer seam")?;
    for (label, graph_success, build_success) in
        [("graph-failed", false, false), ("build-failed", true, false), ("no-dll", true, true)]
    {
        let output = directory.join(label);
        let request = BuildRequest {
            checkout: checkout.clone(),
            output: output.clone(),
            cargo: cargo.clone(),
            environment: BTreeMap::new(),
            inputs: vec![InputRoot { path: checkout.clone(), excluded: Vec::new() }],
        };
        let mut calls = 0;
        let result = build_with(&request, |_, argv, stdout, stderr| {
            calls += 1;
            immutable_bytes(stdout, b"independent fake producer stdout")?;
            immutable_bytes(stderr, b"independent fake producer stderr")?;
            match argv.first().map(String::as_str) {
                Some("tree") => Ok(status(graph_success)),
                Some("rustc") => {
                    require(
                        argv.windows(2).any(|pair| pair == ["--target", "x86_64-pc-windows-gnu"]),
                        "actual explicit target argv",
                    )?;
                    if build_success {
                        let target_index =
                            argv.iter().position(|arg| arg == "--target-dir").ok_or("target arg missing")?;
                        let target = PathBuf::from(argv.get(target_index + 1).ok_or("target arg value missing")?);
                        fs::create_dir(&target).map_err(|e| error(&target, e))?;
                        immutable_bytes(&target.join("some-generated.bin"), b"no DLL despite successful exit")?;
                    }
                    Ok(status(build_success))
                }
                _ => Err("unexpected producer operation".into()),
            }
        });
        let expected = match label {
            "graph-failed" => "Cargo graph inspection failed",
            "build-failed" => "Cargo build failed",
            _ => "produced dep-info count",
        };
        require(
            result.as_ref().err().is_some_and(|error| error.contains(expected)) && !output.join("receipt.sme").exists(),
            label,
        )?;
        require(calls == if graph_success { 2 } else { 1 }, "actual producer call count")?;
        if graph_success {
            require(output.join("build.sme").is_file(), "failed/success producer observation retained")?;
            let observation = sm_binary_evidence::decode(&sm_binary_evidence::read_bytes(&output.join("build.sme"))?)?;
            require(
                observation.get("success").and_then(Value::as_bool) == Some(build_success),
                "observed producer success flag",
            )?;
            let codes = observation.get("exit_code").and_then(Value::as_array).ok_or("observed exit codes missing")?;
            require(
                codes.len() == 1 && codes[0].as_integer() == Some(i64::from(!build_success)),
                "observed producer exit code",
            )?;
            for name in ["stdout", "stderr"] {
                let reference = observation.get(name).ok_or("observed log missing")?;
                let path = Path::new(reference.get("path").and_then(Value::as_str).ok_or("observed log path missing")?);
                let actual = inspect(path)?;
                require(
                    reference.get("sha256").and_then(Value::as_str) == Some(actual.sha256.as_str())
                        && reference.get("size").and_then(Value::as_integer) == Some(actual.size as i64),
                    "observed log identity",
                )?;
            }
        }
    }
    let outside = directory.join("omitted-source.rs");
    immutable_bytes(&outside, b"undeclared dependency literal")?;
    for label in ["valid-build", "omitted-input", "wrong-final-target", "changed-input", "tree-tampered-snapshot"] {
        let output = directory.join(label);
        let request = BuildRequest {
            checkout: checkout.clone(),
            output: output.clone(),
            cargo: cargo.clone(),
            environment: BTreeMap::new(),
            inputs: vec![InputRoot { path: checkout.clone(), excluded: Vec::new() }],
        };
        let result = build_with(&request, |transaction, argv, stdout, stderr| {
            immutable_bytes(stdout, b"fake build output")?;
            immutable_bytes(stderr, b"fake build diagnostic")?;
            if label == "tree-tampered-snapshot" && argv.first().map(String::as_str) == Some("tree") {
                fs::write(transaction.output.join("inputs/00000.bin"), b"snapshot changed while tree ran")
                    .map_err(|e| error(&transaction.output, e))?;
            }
            if argv.first().map(String::as_str) == Some("rustc") {
                let index = argv.iter().position(|arg| arg == "--target-dir").ok_or("target arg missing")?;
                let target = PathBuf::from(argv.get(index + 1).ok_or("target arg missing")?);
                require(
                    !target.to_string_lossy().starts_with(r"\\?\"),
                    "external Cargo target uses ordinary absolute spelling",
                )?;
                let dll = target.join("x86_64-pc-windows-gnu/debug/sm_restart_dll_host.dll");
                fs::create_dir_all(dll.parent().ok_or("DLL parent missing")?).map_err(|e| error(&dll, e))?;
                require(canonical(&target)? == transaction.output.join("target"), "Cargo target identity preserved")?;
                immutable_bytes(&dll, b"fresh produced DLL literal")?;
                let dependency = if label == "omitted-input" { outside.clone() } else { cargo.clone() };
                let target_file = if label == "wrong-final-target" {
                    let wrong = target.join("wrong.dll");
                    immutable_bytes(&wrong, b"other artifact literal")?;
                    wrong
                } else {
                    dll.clone()
                };
                // Fake producer emission; parser oracle has separate raw
                // literals above rather than using this escaping as its oracle.
                let escape = |path: &Path| path.to_string_lossy().replace('\\', "\\\\").replace(' ', "\\ ");
                immutable_bytes(
                    &dll.with_extension("d"),
                    format!(
                        "{}: {}\n",
                        escape(&target_file),
                        if label == "valid-build" { "Cargo.toml".to_owned() } else { escape(&dependency) }
                    )
                    .as_bytes(),
                )?;
                if label == "changed-input" {
                    fs::write(&cargo, b"mutated producer input").map_err(|e| error(&cargo, e))?;
                }
            }
            Ok(status(true))
        });
        if label == "valid-build" {
            let receipt = result?;
            require(
                authenticate(&receipt.receipt)?.dll_sha256 == receipt.dll_sha256,
                "fresh fake build accepted with authentic receipt",
            )?;
        } else {
            let expected = match label {
                "omitted-input" => "compiled dep-info input omitted from inventory",
                "wrong-final-target" => "final dep-info does not identify produced DLL",
                "changed-input" => "build input membership or content changed",
                _ => "retained file changed",
            };
            require(
                result.as_ref().err().is_some_and(|error| error.contains(expected))
                    && !output.join("receipt.sme").exists(),
                label,
            )?;
        }
        if label == "changed-input" {
            fs::write(&cargo, b"not an executable: fake producer seam").map_err(|e| error(&cargo, e))?;
        }
    }
    Ok(())
}

fn measurement_controls(directory: &Path) -> Result<usize> {
    require(Measurements::disabled().start().is_none(), "disabled measurements do not start timers")?;
    let mut rejected = |_: PhaseObservation| Err("independent observer rejection".into());
    let missing = InputRoot { path: directory.join("missing-measurement-root"), excluded: Vec::new() };
    let result = inventory_observed(&[missing], ["membership", "hash"], &mut Measurements::enabled(&mut rejected));
    require(
        result.err().is_some_and(|error| {
            error.contains("missing-measurement-root") && error.contains("independent observer rejection")
        }),
        "membership and observer failures both survive",
    )?;

    let checkout = directory.join("measurement-checkout");
    fs::create_dir_all(checkout.join("restart")).map_err(|e| error(&checkout, e))?;
    immutable_bytes(&checkout.join("restart/Cargo.toml"), b"manifest")?;
    immutable_bytes(&checkout.join("restart/Cargo.lock"), b"lock")?;
    let cargo = checkout.join("cargo.bin");
    immutable_bytes(&cargo, b"cargo")?;
    let mut count = 2;
    for label in ["observed-success", "observer-error", "observer-panic", "late-error", "paired-build-error"] {
        let output = directory.join(label);
        let request = BuildRequest {
            checkout: checkout.clone(),
            output: output.clone(),
            cargo: cargo.clone(),
            environment: BTreeMap::new(),
            inputs: vec![InputRoot { path: checkout.clone(), excluded: Vec::new() }],
        };
        let mut records = Vec::new();
        let mut observer = |record: PhaseObservation| -> Result<()> {
            require(!output.join("receipt.sme").exists(), "no observer call after publication")?;
            let phase = record.phase;
            records.push(record);
            if label == "observer-error" && phase == "cargo_graph" {
                return Err("deliberate graph rejection".into());
            }
            if label == "observer-panic" && phase == "cargo_graph" {
                panic!("deliberate observer panic");
            }
            if label == "late-error" && phase == "publication_ready" {
                return Err("deliberate publication rejection".into());
            }
            if label == "paired-build-error" && phase == "cargo_build" {
                return Err("deliberate build rejection".into());
            }
            Ok(())
        };
        let result = build_with_observed(
            &request,
            |_, argv, stdout, stderr| {
                immutable_bytes(stdout, b"measurement producer stdout")?;
                immutable_bytes(stderr, b"measurement producer stderr")?;
                if argv.first().map(String::as_str) == Some("rustc") {
                    if label == "paired-build-error" {
                        return Ok(status(false));
                    }
                    let index = argv.iter().position(|arg| arg == "--target-dir").ok_or("target arg missing")?;
                    let target = PathBuf::from(argv.get(index + 1).ok_or("target value missing")?);
                    let dll = target.join("x86_64-pc-windows-gnu/debug/sm_restart_dll_host.dll");
                    fs::create_dir_all(dll.parent().ok_or("DLL parent missing")?).map_err(|e| error(&dll, e))?;
                    immutable_bytes(&dll, b"measurement DLL")?;
                    let escaped = dll.to_string_lossy().replace('\\', "\\\\").replace(' ', "\\ ");
                    immutable_bytes(&dll.with_extension("d"), format!("{escaped}: Cargo.toml\n").as_bytes())?;
                }
                Ok(status(true))
            },
            &mut Measurements::enabled(&mut observer),
        );
        require(
            records.iter().all(|record| record.elapsed_seconds.is_finite() && record.elapsed_seconds >= 0.0),
            "finite completed phase times",
        )?;
        if label == "observed-success" {
            let receipt = result?;
            authenticate(&receipt.receipt)?;
            require(
                records.first().is_some_and(|record| record.phase == "normalization_admission")
                    && records.last().is_some_and(|record| record.phase == "publication_ready"),
                "stable phase boundaries",
            )?;
            let expected = [
                "normalization_admission",
                "initial_membership",
                "initial_hash",
                "input_copy",
                "input_sync",
                "input_destination_hash",
                "input_source_hash",
                "input_shard_metadata",
                "frozen_snapshots_membership",
                "frozen_snapshots_hash",
                "cargo_graph",
                "prebuild_membership",
                "prebuild_hash",
                "frozen_output_membership",
                "frozen_output_hash",
                "cargo_build",
                "postbuild_membership",
                "postbuild_hash",
                "generated_membership",
                "generated_hash",
                "generated_copy",
                "generated_sync",
                "generated_destination_hash",
                "generated_source_hash",
                "generated_shard_metadata",
                "retained_generated_membership",
                "retained_generated_hash",
                "dependency_info",
                "dll_hash",
                "dll_retention",
                "final_frozen_output",
                "final_frozen_snapshots",
                "final_observed_files",
                "final_retained_dependencies",
                "final_inputs_membership",
                "final_inputs_hash",
                "final_generated_membership",
                "final_generated_hash",
                "final_retained_membership",
                "final_retained_hash",
                "pending_authentication",
                "publication_ready",
            ];
            require(records.iter().map(|record| record.phase).eq(expected), "complete independent phase order")?;
            let input = |phase| records.iter().find(|record| record.phase == phase).ok_or("input observation missing");
            // Independent literal oracle: manifest (8) + lock (4) + cargo (5).
            require(
                input("input_copy")?.operations == 3 && input("input_copy")?.bytes == 17,
                "input copy exact independent literals",
            )?;
            require(
                input("input_sync")?.operations == 3 && input("input_sync")?.bytes == 0,
                "input durability call count",
            )?;
            require(
                input("input_destination_hash")?.bytes == 17 && input("input_source_hash")?.bytes == 17,
                "both authentication byte counts",
            )?;
            require(records.iter().all(|record| record.success), "successful phases report success")?;
            require(records.iter().filter(|record| record.phase == "cargo_build").count() == 1, "single build phase")?;
        } else {
            let expected = match label {
                "observer-error" => "deliberate graph rejection",
                "observer-panic" => "phase observer panicked at cargo_graph",
                "late-error" => "deliberate publication rejection",
                _ => "Cargo build failed",
            };
            let error = result.err().ok_or("observer failure accepted")?;
            require(
                error.contains(expected) && !output.join("receipt.sme").exists(),
                "terminal observer failure refuses publication",
            )?;
            if label == "paired-build-error" {
                require(
                    error.contains("deliberate build rejection") && output.join("build.sme").is_file(),
                    "build diagnostic and observer rejection survive",
                )?;
                require(
                    records.last().is_some_and(|record| record.phase == "cargo_build" && !record.success),
                    "failed build observation",
                )?;
            }
            if label == "late-error" {
                require(output.join("receipt-pending.sme").is_file(), "late rejection after pending authentication")?;
            }
        }
        count += 1;
    }
    Ok(count)
}

fn reader_controls(directory: &Path) -> Result<usize> {
    let output = directory.join("reader");
    fs::create_dir(&output).map_err(|e| error(&output, e))?;
    let input = directory.join("input/two.rs");
    let entries = BTreeMap::from([(input.clone(), inspect(&input)?)]);
    let inputs = snapshots(&output, "inputs", &entries)?;
    let generated_input = directory.join("generated-literal.bin");
    immutable_bytes(&generated_input, b"different generated literal")?;
    let generated =
        snapshots(&output, "generated", &BTreeMap::from([(generated_input.clone(), inspect(&generated_input)?)]))?;
    let log = output.join("log.txt");
    immutable_bytes(&log, b"literal diagnostics")?;
    let graph = output.join("graph.txt");
    immutable_bytes(&graph, b"literal graph package features")?;
    let graph_error = output.join("graph.stderr");
    immutable_bytes(&graph_error, b"graph diagnostic literal")?;
    let stderr = output.join("build.stderr");
    immutable_bytes(&stderr, b"build diagnostic literal")?;
    let dep_info = output.join("literal.d");
    immutable_bytes(&dep_info, b"target: literal-dependency")?;
    let pre = evidence(
        &output.join("pre.sme"),
        Value::Table(Map::from([
            ("schema".into(), "sm-build-inputs-v1".into()),
            ("inputs".into(), inputs),
            ("environment".into(), Value::Table(Map::new())),
            ("argv".into(), vec!["rustc".to_owned()].into()),
            ("graph_argv".into(), vec!["tree".to_owned()].into()),
            ("target_initially_absent".into(), true.into()),
            ("cargo".into(), input.to_str().ok_or("fixture path")?.into()),
            ("cwd".into(), directory.to_str().ok_or("fixture path")?.into()),
            ("started_unix_ms".into(), 100_i64.into()),
            ("graph".into(), file_reference(&graph)?),
            ("graph_stderr".into(), file_reference(&graph_error)?),
        ])),
    )?;
    let observation = Value::Table(Map::from([
        ("schema".into(), "sm-build-observation-v1".into()),
        ("success".into(), true.into()),
        ("exit_code".into(), vec![Value::Integer(0)].into()),
        ("started_unix_ms".into(), 100_i64.into()),
        ("ended_unix_ms".into(), 101_i64.into()),
        ("elapsed_seconds".into(), 0.001_f64.into()),
        ("stdout".into(), file_reference(&log)?),
        ("stderr".into(), file_reference(&stderr)?),
    ]));
    let build = evidence(&output.join("build.sme"), observation.clone())?;
    let dll_source = directory.join("dll-literal.bin");
    immutable_bytes(&dll_source, b"distinct DLL fixture literal")?;
    let dll_identity = inspect(&dll_source)?;
    let dll = output.join(format!("{}.dll", dll_identity.sha256));
    retain(&dll_source, &dll, &dll_identity)?;
    let receipt_value = Value::Table(Map::from([
        ("schema".into(), "sm-build-receipt-v1".into()),
        ("pre".into(), pre),
        ("build".into(), build),
        ("generated".into(), generated),
        ("dependency_info".into(), vec![file_reference(&dep_info)?].into()),
        ("dll".into(), file_reference(&dll)?),
        ("inputs_unchanged".into(), true.into()),
        ("closure_scope".into(), "literal reader fixture; not a real build".into()),
    ]));
    let receipt = output.join("receipt.sme");
    evidence(&receipt, receipt_value.clone())?;
    require(
        authenticate(&receipt)?.dll_sha256 == dll_identity.sha256,
        "reader authenticates distinct retained fixture",
    )?;
    let field_count = super::field_controls::run(&receipt)?;
    let mut malformed = receipt_value.clone();
    malformed.as_table_mut().ok_or("fixture table")?.remove("build");
    let bad = output.join("missing.sme");
    evidence(&bad, malformed)?;
    require(authenticate(&bad).is_err(), "reader refuses missing required field")?;
    let mut bad_observation = observation;
    bad_observation.as_table_mut().ok_or("fixture table")?.insert("success".into(), false.into());
    let bad_build = evidence(&output.join("failed-build.sme"), bad_observation)?;
    let mut failed_receipt = receipt_value.clone();
    failed_receipt.as_table_mut().ok_or("fixture table")?.insert("build".into(), bad_build);
    let failed = output.join("failed.sme");
    evidence(&failed, failed_receipt)?;
    require(authenticate(&failed).is_err(), "reader refuses failed observation")?;
    fs::write(&dll, b"wrong dll bytes").map_err(|e| error(&dll, e))?;
    require(authenticate(&receipt).is_err(), "reader refuses DLL corruption")?;
    fs::write(&dll, b"distinct DLL fixture literal").map_err(|e| error(&dll, e))?;
    authenticate(&receipt)?;
    fs::write(output.join("inputs/00000.bin"), b"bad snapshot").map_err(|e| error(&output, e))?;
    require(authenticate(&receipt).is_err(), "reader refuses snapshot corruption")?;
    fs::write(output.join("inputs/00000.bin"), b"different literal").map_err(|e| error(&output, e))?;
    authenticate(&receipt)?;
    fs::write(&log, b"changed diagnostics").map_err(|e| error(&log, e))?;
    require(authenticate(&receipt).is_err(), "reader refuses retained log corruption")?;
    Ok(field_count)
}

#[cfg(test)]
mod tests {
    #[test]
    fn focused_refusal_controls() {
        let path = std::env::temp_dir().join(format!(
            "sm-build-receipt-{}-{}",
            std::process::id(),
            super::super::timestamp().unwrap().as_integer().unwrap()
        ));
        assert_eq!(super::focused_controls(&path).unwrap(), 179 + if cfg!(windows) { 5 } else { 0 });
        assert!(!path.exists());
    }
}
