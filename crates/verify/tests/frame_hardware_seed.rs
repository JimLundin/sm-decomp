//! Development evidence for the independent source hardware edge.
#[path = "../src/replay/frame_hardware_seed.rs"]
mod frame_hardware_seed;

use data::timing::IrqPolicy;
use frame_hardware_seed::{SeedFault, decode, pads};

#[test]
fn hexadecimal_physical_actions() {
    assert_eq!(pads("0080", "0100"), Ok((0x80, 0x100)));
    assert_eq!(pads("10", "20"), Ok((16, 32)));
    for invalid in ["", "-1", "0x10", "10000", " 10"] {
        assert!(pads(invalid, "0").is_err());
    }
}

#[test]
fn incomplete_context_rejected() {
    for length in [0, 82, 330, 493, 495] {
        assert_eq!(
            decode(&vec![0; length], IrqPolicy::Eligible, 0, 0),
            Err(SeedFault::Length)
        );
    }
}

#[test]
#[ignore = "requires independently admitted private source packet and physical action rows"]
fn qualified_source_origin_and_mutations() {
    let directory =
        std::env::var_os("SM_FRAME_HARDWARE_PACKET").expect("private packet path required");
    let bytes =
        std::fs::read(std::path::Path::new(&directory).join("initial-context.bin")).unwrap();
    let pad_path =
        std::env::var_os("SM_FRAME_HARDWARE_PADS").expect("independent pad rows required");
    let rows = std::fs::read_to_string(pad_path).unwrap();
    let mut lines = rows.lines();
    let (current, preview) = pads(lines.next().unwrap(), lines.next().unwrap()).unwrap();
    let state = decode(&bytes, IrqPolicy::Eligible, current, preview).unwrap();
    state.validate().unwrap();
    assert_eq!(state.work.len(), 0);
    assert_eq!(state.hardware.now.0, 0);
    for (offset, value, fault) in [
        (83, 2, SeedFault::Boolean),
        (314, 1, SeedFault::Unsupported),
        (466, 1, SeedFault::Unsupported),
        (462, 1, SeedFault::Unsupported),
        (378, 1, SeedFault::Unsupported),
        (82, 1, SeedFault::Unsupported),
        (86, 1, SeedFault::Unsupported),
    ] {
        let mut mutated = bytes.clone();
        mutated[offset] = value;
        assert!(decode(&mutated, IrqPolicy::Eligible, 0, 0).is_err_and(|actual| actual == fault));
    }
}
