#![allow(unused_braces, unused_imports, unused_variables)]
use runtime::{Cpu, Machine, ea, flag, ops};

mod support {
    use std::collections::HashMap;

    use runtime::{Bus, Cpu, Machine};

    struct Sparse(HashMap<u32, u8>);
    impl Bus for Sparse {
        fn read(&mut self, a: u32) -> u8 {
            *self.0.get(&a).unwrap_or(&0)
        }
        fn write(&mut self, a: u32, v: u8) {
            self.0.insert(a, v);
        }
    }

    pub fn machine(ram: &[(u32, u8)]) -> Machine {
        Machine::new(Box::new(Sparse(ram.iter().copied().collect())), &[], None)
    }

    #[allow(dead_code)] // Used when the external corpus has generated cases.
    pub fn check(m: &mut Machine, want: Cpu, ram: &[(u32, u8)], name: &str) {
        // PC is not modelled by generated code; compare everything else.
        assert_eq!(m.cpu, want, "registers: {name}");
        for &(a, v) in ram {
            assert_eq!(m.read8(a), v, "ram ${a:06X}: {name}");
        }
    }
}

include!(concat!(env!("OUT_DIR"), "/cases.rs"));

#[test]
fn harness_compiles() {
    let m = support::machine(&[]);
    let _ = m.cpu.clone();
}
