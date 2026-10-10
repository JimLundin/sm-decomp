//! The recompiled game as a library: generated code, overrides, and hardware glue.

pub mod generated {
    include!(concat!(env!("OUT_DIR"), "/generated.rs"));
}
pub mod hw;
pub mod overrides;
pub mod steps {
    #![allow(non_snake_case, unused_imports)]
    include!(concat!(env!("OUT_DIR"), "/steps.rs"));
}

use runtime::{Fallback, Machine, TableEntry};

/// Generated functions first, then overrides. Overrides win on duplicate keys.
pub fn table() -> Vec<TableEntry> {
    let mut t: Vec<TableEntry> = generated::TABLE.to_vec();
    t.extend_from_slice(overrides::TABLE);
    t
}

pub fn machine(bus: Box<dyn runtime::Bus>, fallback: Option<Box<dyn Fallback>>) -> Machine {
    Machine::new(bus, &table(), fallback)
}
