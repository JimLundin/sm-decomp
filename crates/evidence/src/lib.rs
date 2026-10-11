//! Content-addressed evidence, packed independently of its canonical identities.
//!
//! Adapted from the trace-storage worktree's Merkle image and exact burst spike.
//! Storage, bounds, stream framing and run publication are implemented here.
mod canonical;
mod image;
mod pack;
mod run;
mod trace;
mod tree;

pub use canonical::{Id, hex, parse_id};
pub use image::{diff_images, put_image, write_image};
pub use pack::{Store, StoreStats};
pub use run::{Run, put_run, read_run, verify_run};
pub use trace::{Event, StreamInfo, TraceWriter, ingest_trace, read_stream, write_frame, write_stream};
pub use tree::{
    Entry, EntryKind, MemberInfo, ingest_tree, put_file, put_tree, read_tree, tree_inventory, verify_tree, write_entry,
    write_file,
};

pub(crate) const BLOB: u8 = b'B';
pub(crate) const NODE: u8 = b'N';
pub(crate) const IMAGE: u8 = b'I';
pub(crate) const SEGMENT: u8 = b'G';
pub(crate) const STREAM: u8 = b'S';
pub(crate) const TREE: u8 = b'T';
pub(crate) const FILE: u8 = b'F';
pub(crate) const RUN: u8 = b'R';
