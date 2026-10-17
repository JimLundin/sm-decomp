# Bounded persistence source handoff

Source-only implementation of ticket 24's frozen four-routine interface. Owned
files are exclusively this crate. Root owns workspace registration, Cargo.lock,
Game/native preparation wiring, producers, verification, publication and cleanup.

- Public API: Entry/ALL/address/TryFrom<u32>, Output, Services, execute, WRAM_SIZE,
  SRAM_SIZE. No dependencies; unsafe Rust forbidden.
- `fixture-support` exports `fixtures::{BodyFixture, body_fixtures}` and equivalent
  crate-root reexports. Exactly 30 deterministic full-state body fixtures:
  9 saves, 19 loads and 2 map helpers. No masked-slot-3 actual-C fixtures.
- Twelve semantic tests cover complete payload/service ordering, checksum copy
  alternatives and wrap, exact invalid-load mutations, high-bit slot aliases,
  map boundary sentinels, save/load chain, atomic refusal/error and terminal
  post-mutation service failure. Shared native fixture execution remains required.
- Save/Load slots are admitted iff argument&3 <=2. Save additionally requires
  area<=5. Direct map helpers require argument0. Load preserves arbitrary valid
  saved station/area. All state lengths, consumed ROM slices and required service
  availability are validated before writes. Refusal does not invoke a service.
- ROM sealing checks exactly consumed slices: six slot bytes at812B, six counts
  at8131, twelve packed-offset bytes at8138, twelve pointer bytes at82D6 and the
  six exact index lists. Unused seventh-area words/bytes and padding are not
  dependencies. Fixture-only synthetic ROM initializes those exact slices.
- Runtime copies full065C-byte payload, wraps checksum words, and unpack clears
  exactly0702 bytes. One Save service receives the same mutable WRAM/SRAM pair
  after complete state mutation. Service errors propagate as errors; no fallback.

Source formatting with rustfmt and git diff --check completed. No Cargo command,
build, test, native invocation or gameplay run was performed by this worker.
No generated verification artifacts, disk-save files or caches need cleanup.
Stage: source handoff only, not actual-C verified or live exercised.
