# Add direct bank81 persistence roots to the shared native-exercise contract

Type: task
Status: proposed; coordinator review/publication required
Origin: exact research19 claim; see [Answer](../Answer.md) and [children.toml](../children.toml).
No writable ownership, supported native scope or gameplay acceptance is granted.

Supplement existing ticket21 with the genuine `SaveToSram`818000 and
`LoadFromSram`818085 roots. Their Pack/Unpack closures are separate; a save or
callee hit cannot establish load or caller exercise. Bind actual entry/return,
selected slot and consumed tables, full canonical state windows, complete
original/generated source, definition/inventory/build/binary/reader/ROM/movie
identities and nonzero observations. Reject wrong-root/callee, zero/truncated,
stale identity, slot and selector substitutions independently.

Use the one additive Rust producer/reader under reviewed source-epoch ownership.
Candidate exact files: native_exercise.rs, native_exercise_model.rs,
native_exercise_host.c.in, translation.rs, main.rs and tests/native_exercise.rs
under crates/pipeline. Preserve existing bank86 schema2. Shared registration is
integrator-owned; no second provenance engine.

Proposed unrun gates: `cargo test -p pipeline --locked --offline --test native_exercise`
(60s); `cargo test -p pipeline --locked --offline --test native_exercise --test assignment_queue --bin sm-pipeline`
(3600s). New targets remain unimplemented. After any future run, complete both
project cleanup passes, retain evidence/source/reader/store inputs and record
reclaimed/free bytes. This read-only worker created no run.
