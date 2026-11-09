#![allow(dead_code)]

#[cfg(proc_macro_span)]
pub(crate) mod proc_macro_span;

#[cfg(feature = "proc-macro")]
pub(crate) mod proc_macro_span_file;

#[cfg(feature = "proc-macro")]
pub(crate) mod proc_macro_span_location;
