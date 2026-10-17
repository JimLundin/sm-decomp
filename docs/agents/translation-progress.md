# Publish gameplay translation progress

Read this when assigning, implementing or integrating gameplay translations.
The shared inventory and interactive view live in the main checkout's
`.scratch/decomp-progress/registry.json` and `index.html`.

## Assign and translate

1. Select a coherent Translation scope: a subsystem, bank or complete scheduled
   call boundary and its dependencies. Reuse the recovered C and existing Rust
   modules. Split out individual routines when a differential failure needs
   isolation. Read `docs/agents/assignments.md` for durable claims and ownership.
2. Check the inventory, existing local tickets and live agents before assigning
   mutable work. Keep source ownership disjoint across implementation worktrees;
   the coordinator owns shared registration, verification and integration.
3. Publish a `translating` observation at assignment. Record the source revision,
   exact routine symbols, owner, scope and current limitations. Continue existing
   claims after an interrupted coordinator until an owner-stop check establishes
   otherwise.
4. Before each handoff, publish the stages actually achieved: `translated`,
   `verified`, `integrated`, or `live-exercised`. Record each stage independently
   with its commit and evidence. The integration owner publishes integration and
   live exercise after the corresponding checks and required run cleanup.

## Update the shared inventory

Use the main checkout even when implementation happens in another worktree:

```sh
cargo run -p pipeline --locked --offline --bin sm-pipeline -- progress refresh --root /home/jim/sm-decomp
cargo run -p pipeline --locked --offline --bin sm-pipeline -- progress status --root /home/jim/sm-decomp
cargo run -p pipeline --locked --offline --bin sm-pipeline -- progress update --root /home/jim/sm-decomp --update UPDATE.toml
```

When several stage observations are already ready, publish their TOML files in
order with repeated `--update` arguments (1–64 files, at most one MiB each):

```sh
cargo run -p pipeline --locked --offline --bin sm-pipeline -- progress update --root /home/jim/sm-decomp \
  --update translated.toml --update verified.toml --update integrated.toml
```

Keep each observation's own ID, stage, source identity and evidence. Publish
`translating` at assignment and achieved stages at handoff when ready; batch only
observations that are ready together. A supersession target must already exist
or appear earlier in the argument order.

Copy `source.revision` and `source.inventory_sha256` from `registry.json` into
`source_revision` and `inventory_sha256`. One update has this shape:

```toml
id = "scope-stage-unique-observation"
scope = "Native projectile trail frame"
agent = "/root/independent_gameplay"
stage = "translated"
source_revision = "COPY_FROM_REGISTRY"
inventory_sha256 = "COPY_FROM_REGISTRY"
routines = ["HandleProjectileTrails"]
commit = "EXACT_COMMIT"
limitations = ["Live integration is pending."]

[[evidence]]
path = "/absolute/path/to/retained/evidence.toml"
```

The Rust command prepares all inputs, then validates source identity, routine
membership and evidence hashes under one registry lock. A failed input or
observation rejects the whole proposed batch before publication. JSON and HTML
are each replaced atomically; they are not one atomic pair. If HTML publication
fails after JSON succeeds, retry the same command to repair the dashboard from
the authoritative JSON. Identical retries preserve observation timestamps and
retain intervening valid observations, including when every batch ID is already
published. Reuse an id only for an identical retry; give new observations new ids.
Evidence pointers are hashed records; existing family/live authentication remains
required for admission. List the tested domain and unowned dependencies in
`limitations` so a partial scope is reviewable.

`refresh --source /home/jim/sm-decomp/deps/snesrev-sm` regenerates the lexical
inventory against the pinned source. Routine definitions/calls and C body lines
are inventory, not a full-game completion estimate. Fixture declarations are
shown separately because they may be stale, bounded or unverified; dynamic
instruction dispatch requires an explicit scope observation.

## Inspect progress

Open `.scratch/decomp-progress/index.html` directly for its embedded snapshot,
then choose a newer `registry.json` through the view's file picker if needed.
For automatic refresh, serve the view locally:

```sh
cargo run -p pipeline --locked --offline --bin sm-pipeline -- progress serve --root /home/jim/sm-decomp --bind 127.0.0.1:4185
```

The view filters banks, stage, owner, symbol and address, and exposes each
routine's dependencies and progress evidence. Reported stages stay distinct;
original-ROM parity and behavior outside tested replay domains remain separate
verification requirements.
