//! Full fresh/reused preparation control shared by CLI and local DLL checks.
use crate::{authenticate_library, prepare};
use sha2::{Digest, Sha256};
use sm_binary_evidence::{Map, Value};
use std::io::Read;
use std::path::Path;

pub fn producer_bytes(path: &Path) -> Result<Vec<u8>, String> {
    // Local debug DLLs are below128MiB; the same bound admits the CLI artifact
    // while preventing input metadata from controlling unbounded allocation.
    const LIMIT: u64 = 128 * 1024 * 1024;
    let file = std::fs::File::open(path).map_err(|error| error.to_string())?;
    if file.metadata().map_err(|error| error.to_string())?.len() > LIMIT {
        return Err("control artifact exceeds128MiB".into());
    }
    let mut bytes = Vec::new();
    file.take(LIMIT + 1).read_to_end(&mut bytes).map_err(|error| error.to_string())?;
    if bytes.len() as u64 > LIMIT {
        return Err("control artifact exceeds128MiB".into());
    }
    Ok(bytes)
}

pub fn run(archive: &Path, output: &Path, runtime: &Path, producer: &Path) -> Result<(), String> {
    if output.exists() {
        return Err("preparation control requires a fresh output directory".into());
    }
    // In DLL mode current_exe identifies Python; the caller names the Rust DLL.
    let runtime_bytes = producer_bytes(runtime)?;
    let producer_bytes = producer_bytes(producer)?;
    let started = std::time::Instant::now();
    let first = prepare(archive, output)?;
    if first.reused {
        return Err("fresh preparation unexpectedly reused a library".into());
    }
    authenticate_library(&first.library)?;
    let second = prepare(archive, output)?;
    if !second.reused || first.library != second.library || first.manifest != second.manifest {
        return Err("authenticated preparation reuse changed its subject".into());
    }
    let library_hash = authenticate_library(&second.library)?;
    // SAFETY: authentication checked the recipe/library/sources. The scoped
    // handle resolves opaque addresses without calling or dereferencing them.
    let library = unsafe { libloading::Library::new(&second.library) }.map_err(|error| error.to_string())?;
    for name in [b"sm_native_boot\0".as_slice(), b"sm_native_step\0", b"sm_native_invoke_rio\0"] {
        // SAFETY: opaque addresses are never invoked or dereferenced.
        let _symbol = unsafe { library.get::<*const ()>(name) }.map_err(|error| error.to_string())?;
    }
    std::fs::write(output.join("public-runtime"), &runtime_bytes).map_err(|error| error.to_string())?;
    std::fs::write(output.join("prepare-control"), &producer_bytes).map_err(|error| error.to_string())?;
    std::fs::write(output.join("preparation_control.rs"), include_bytes!("preparation_control.rs"))
        .map_err(|error| error.to_string())?;
    let receipt = Value::Table(Map::from([
        ("schema".into(), "production-prepare-control-v2".into()),
        ("runtime_sha256".into(), format!("{:x}", Sha256::digest(&runtime_bytes)).into()),
        ("producer_sha256".into(), format!("{:x}", Sha256::digest(&producer_bytes)).into()),
        (
            "producer_source_sha256".into(),
            format!("{:x}", Sha256::digest(include_bytes!("preparation_control.rs"))).into(),
        ),
        ("library_sha256".into(), library_hash.into()),
        ("library".into(), second.library.to_string_lossy().as_ref().into()),
        ("manifest".into(), second.manifest.to_string_lossy().as_ref().into()),
        ("fresh_reused".into(), false.into()),
        ("second_reused".into(), true.into()),
        ("elapsed_seconds".into(), started.elapsed().as_secs_f64().into()),
        ("game_booted".into(), false.into()),
        ("native_exports_invoked".into(), false.into()),
    ]));
    std::fs::write(output.join("control-receipt.sme"), sm_binary_evidence::encode(&receipt)?)
        .map_err(|error| error.to_string())?;
    println!("production preparation and authenticated reuse passed: {}", second.library.display());
    Ok(())
}

#[cfg(any(test, feature = "private-checks"))]
#[cfg_attr(test, test)]
pub(crate) fn producer_inputs_are_bounded_before_preparation() {
    let nonce = std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH).unwrap().as_nanos();
    let root = std::env::temp_dir().join(format!("sm-producer-control-{}-{nonce}", std::process::id()));
    std::fs::create_dir(&root).unwrap();
    let outcome = std::panic::catch_unwind(|| {
        let valid = root.join("valid");
        std::fs::write(&valid, b"literalproducer").unwrap();
        assert_eq!(producer_bytes(&valid).unwrap(), b"literalproducer");
        assert!(producer_bytes(&root.join("missing")).is_err());
        let large = root.join("large");
        std::fs::File::create(&large).unwrap().set_len(128 * 1024 * 1024 + 1).unwrap();
        assert_eq!(producer_bytes(&large).unwrap_err(), "control artifact exceeds128MiB");
        for (runtime, producer) in [(&large, &valid), (&valid, &large)] {
            let output = root.join("neverprepared");
            assert_eq!(
                run(&root.join("missingarchive"), &output, runtime, producer).unwrap_err(),
                "control artifact exceeds128MiB"
            );
            assert!(!output.exists());
        }
    });
    std::fs::remove_dir_all(&root).unwrap_or_else(|error| panic!("remove {}: {error}", root.display()));
    if let Err(payload) = outcome {
        std::panic::resume_unwind(payload);
    }
}
