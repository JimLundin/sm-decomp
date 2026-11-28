//! The `cleanroom` binary cargo built for this run holds no route marker.

mod routes;
#[path = "../../src/bin/cleanroom/salted.rs"]
mod salted;

use routes::PLAIN_ROUTES;

#[test]
fn the_binary_holds_no_route_in_plain_text() {
    let binary = std::fs::read(env!("CARGO_BIN_EXE_cleanroom"))
        .unwrap()
        .to_ascii_lowercase();
    let text = String::from_utf8_lossy(&binary);
    for plain in PLAIN_ROUTES {
        assert_eq!(*plain, plain.to_ascii_lowercase(), "markers are lowercase");
        assert!(!text.contains(plain), "the binary holds {plain}");
    }
}

/// Neither the lexicon's hash of a marker nor its hash under the salt the binary once held, in
/// either byte order.
#[test]
fn the_binary_holds_no_route_hash() {
    let salt = std::fs::read_to_string(concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/../../tools/cleanroom/scan_lexicon/salt"
    ))
    .unwrap();
    let mut hashes: Vec<u64> = PLAIN_ROUTES
        .iter()
        .flat_map(|plain| {
            [
                salted::hash(salt.trim(), plain),
                salted::hash("route", plain),
            ]
        })
        .flat_map(|hash| [hash, hash.swap_bytes()])
        .collect();
    hashes.sort_unstable();
    let binary = std::fs::read(env!("CARGO_BIN_EXE_cleanroom")).unwrap();
    let held = binary
        .windows(8)
        .filter_map(|window| window.try_into().ok().map(u64::from_le_bytes))
        .find(|word| hashes.binary_search(word).is_ok());
    assert_eq!(held, None, "the binary holds a route marker's hash");
}
