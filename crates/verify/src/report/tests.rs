use std::path::Path;

use super::readme::PATH;
use super::{check, current_at};

#[test]
fn committed_readme_is_current() {
    assert_eq!(
        check(Path::new(concat!(env!("CARGO_MANIFEST_DIR"), "/../.."))),
        Ok(())
    );
}

#[test]
fn a_changed_readme_is_stale() {
    let fresh = "# sm-decomp\n\n| a | 1 |\n";
    assert_eq!(current_at(PATH, fresh, fresh), Ok(()));
    assert_eq!(
        current_at(PATH, &fresh.replace('\n', "\r\n"), fresh),
        Ok(())
    );
    assert!(current_at(PATH, "# sm-decomp\n\n| a | 2 |\n", fresh).is_err());
    assert!(current_at(PATH, "", fresh).is_err());
}
