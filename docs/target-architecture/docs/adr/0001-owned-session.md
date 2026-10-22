---
status: accepted
---
# Own gameplay inside one deterministic Session

Super Metroid has tightly coupled, order-sensitive movement, collision, actors, and room transitions. Use a `Session` that exclusively owns authoritative state and exposes open, step, and checkpoint operations; its private domain modules share validated immutable gameplay definitions. This concentrates sequencing, restoration, and determinism behind one interface, at the cost of implementing focused actor storage and inspection instead of adopting an ECS's query and tooling ecosystem.

The private-ECS alternative was considered. The later [owned Rust implementation decision](0004-owned-rust-implementation.md) excludes adopting an external ECS; use the representative encounter to refine our own Actor storage and query helpers while retaining explicit ordering and the Session interface.
