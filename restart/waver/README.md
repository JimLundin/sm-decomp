# Waver actor

Pure Rust translation of the four Waver definitions at recovered C revision
578f90b: Init A386ED, Main A3874C, instruction A386E3 and animation helper A387FE.
The native enemy scheduler, instruction interpreter, terrain, sine, contact,
damage/death/drop and drawing remain external. This crate has no runtime deps
and forbids unsafe Rust. Upstream licensing is retained in LICENSE.upstream.txt.

`execute` takes full canonical WRAM, the runtime ROM and three synchronous
native services. Main's native capability must be available before any write;
unavailable capability and unsupported animation selectors return unchanged
`None`. Invalid canonical memory/slots/continuation extents are errors. Any
service error after a request is fatal to the invocation, never a C retry.

Current enemy slot is WRAM E54, aligned64 and below800. Instruction validates
its explicit k separately but writes the current slot's E field; it returns
exact jp unchanged. Bank-A3 jp is a byte offset: real86C5/86D9 values are odd.
Noninstruction entries ignore k/jp. Canonical Main selectors F0..3 are admitted;
helper equal F==C is a no-op even outside that range and before any ROM lookup.
Init overwrites irrelevant old F. Animation words are read from runtime ROM,
not replaced by constants; Game owns pristine/runtime ROM authentication.

Native movement services publish/copy back complete WRAM. Main retains the
entry record offset like the C E pointer, but rereads its fields after services.
Subsequent movement k and local helper current slot come from canonical E54.
No Rust write precedes horizontal/sine/vertical service calls; post-service
animation/event updates follow exact source order. Native services own terrain
admission, including the finite spike15/D094 continuation, and transport guards.

With `fixture-support`, public `BodyFixture` and `body_fixtures()` are available
at crate root and `fixtures`. Their24 full-state inputs feed root's actual-C
controls. Fifteen semantic tests use scripted foreign outcomes to check actor
semantics and state preservation; they are not a Rust terrain approximation or
native execution evidence. Build/verification/publication belongs to root.
