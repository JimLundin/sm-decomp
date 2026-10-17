use sm_native_host::prepare;
use std::fs;
use std::path::{Path, PathBuf};

struct PrivateCopy(PathBuf);
impl Drop for PrivateCopy {
    fn drop(&mut self) {
        fs::remove_dir_all(&self.0).expect("remove private integrity-control copy");
    }
}

fn copy_directory(source: &Path, destination: &Path) {
    fs::create_dir(destination).unwrap();
    for entry in fs::read_dir(source).unwrap() {
        let entry = entry.unwrap();
        let target = destination.join(entry.file_name());
        if entry.file_type().unwrap().is_dir() {
            copy_directory(&entry.path(), &target);
        } else {
            assert!(entry.file_type().unwrap().is_file());
            fs::copy(entry.path(), target).unwrap();
        }
    }
}

/// A real prepared-library control, deliberately excluded from asset-free Cargo
/// tests. The copied files are private; the accepted library remains untouched.
#[cfg_attr(test, test)]
#[cfg_attr(test, ignore = "requires SM_NATIVE_ARCHIVE and SM_NATIVE_PREPARED or SM_NATIVE_PREPARED_ROOT")]
pub(crate) fn rejects_modified_library_and_modified_retained_source_on_cache_hit() {
    let archive = PathBuf::from(std::env::var_os("SM_NATIVE_ARCHIVE").expect("SM_NATIVE_ARCHIVE"));
    let prepared = if let Some(path) = std::env::var_os("SM_NATIVE_PREPARED") {
        PathBuf::from(path)
    } else {
        let root = PathBuf::from(std::env::var_os("SM_NATIVE_PREPARED_ROOT").expect("SM_NATIVE_PREPARED_ROOT"));
        let subjects: Vec<_> = fs::read_dir(root)
            .unwrap()
            .map(|entry| entry.unwrap().path())
            .filter(|path| path.join("library.sme").is_file())
            .collect();
        assert_eq!(subjects.len(), 1, "integrity control needs exactly one prepared subject");
        subjects.into_iter().next().unwrap()
    };
    let scratch = PrivateCopy(std::env::temp_dir().join(format!("sm-native-integrity-{}", std::process::id())));
    fs::create_dir(&scratch.0).unwrap();
    let copy = scratch.0.join(prepared.file_name().unwrap());
    copy_directory(&prepared, &copy);
    let prepared_copy = prepare(&archive, &scratch.0).unwrap();
    assert!(prepared_copy.reused);

    let library = prepared_copy.library;
    let original = fs::read(&library).unwrap();
    let mut changed = original.clone();
    changed[0] ^= 1;
    fs::write(&library, &changed).unwrap();
    assert!(prepare(&archive, &scratch.0).unwrap_err().contains("library SHA-256 mismatch"));
    fs::write(&library, original).unwrap();

    let source = copy.join("source/restart_shim.c");
    let mut changed = fs::read(&source).unwrap();
    changed.extend_from_slice(b"\n/* deliberate retained-source mutation */\n");
    fs::write(source, changed).unwrap();
    assert!(prepare(&archive, &scratch.0).unwrap_err().contains("source identity mismatch"));
}
