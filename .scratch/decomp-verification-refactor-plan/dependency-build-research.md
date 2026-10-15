# Tickets 03/04: dependency facts and shared preparation

Status: research complete; no implementation or measurement performed here.

Source: `/home/jim/sm-decomp-offline-verifier` at frozen
`8bf8acf610e1a94a1a3bccd5d718dae4a4244857`. Locations below refer to that source.
This continues [the ticket 02 input-flow research](bulk-context-research.md).
Scope is dependency recognition, compiler preparation/keys and the Native
producer constraints; evidence storage and broad scheduling were not explored.

## Minimal safe first implementation

1. Put Rust source facts behind one private `offline-verification` scanner. Use
   the already locked/cached Rust tokenization and parsing libraries, retaining
   exact input hashes. Return owned identifiers, recognized file/environment
   inputs and explicit incompleteness reasons. Consume those facts from affected
   selection and both wrapper-discovery paths.
2. Initially preserve the existing conservative supported subset. Move its
   unsupported-word test onto real code identifiers, excluding comments and
   ordinary literals. Real includes, environment accesses, external modules and
   unreviewed macro expansion stay incomplete unless explicitly modeled. Then
   use the AST to support individual input forms with positive/negative controls.
   Fixing comments must not be described as proving all remaining closures.
3. Extend `tool-execution` preparation to share immutable compiler/toolchain
   configuration inside one compatible batch. Keep per-family cancellation,
   source freshness and publication checks. Continue using the existing
   authenticated cache and key lock.
4. Preserve all Native compiler producer slots. Ordinary pinned C objects already
   share by cache key; generated units and candidate Rust builds need additional
   input/path proofs before they become cacheable. Measure the preparation-only
   change before claiming new compilation savings.

## Exactly why `include` comments are misclassified

`pipeline/src/affected.rs:289–319` reads a source, hashes it again, and then splits
the entire text at every character other than ASCII alphanumeric/underscore.
It rejects any resulting word in:

```text
mod include include_str include_bytes env option_env fs File macro_rules
```

That scan ignores Rust token kinds. In the frozen source,
`crates/port/src/plm.rs:769` contains:

```rust
/// ROM-aware preinstructions include native room projectile spawning.
```

and `crates/port/src/projectile_flight.rs:399` contains:

```rust
/// Trail coordinates include native Ceres Mode7 arithmetic and ROM trail frames.
```

Both generate the unsupported word `include` despite declaring no file input.
The same issue applies to ordinary string literals and identifiers such as a
local variable named `File`. The first two are definite false input signals;
resolving the last requires more syntax/name context.

There are **three owners of raw Rust word traversal**, not just this refusal:

| Location | Purpose |
| --- | --- |
| `pipeline/src/affected.rs:304`, `370` | Reject unsupported inputs, then recursively select any top-level gameplay module whose name occurs as a word. |
| `offline-verification/src/inputs.rs:161–205`, especially 193 | Canonical source closure used by Native preparation and root-level module placement. |
| `offline-verification/src/inputs.rs:236–285`, especially 264 | Ordinary wrapper closure and generated `memory.rs` imports. |

Changing only the planner would leave comments/literals pulling unrelated modules
into generated builds. Share the lexical facts; keep application selection policy
in `pipeline`. Do not put Offline manifest/recipe policy into `decomp-source`.

Current module discovery deliberately overapproximates references. Replacing its
word set with an equally conservative set of actual code identifiers is a safer
first change than assuming every dependency appears as an obvious `crate::foo`
path. Names inside real macro token arguments still matter.

## Existing parser and dependency code to reuse

There is no maintained dependency-grade Rust lexer/parser in these owners.
`pipeline/src/progress/source.rs:11` has a source-correspondence lexer, but its
contract explicitly does not establish equivalence. It handles line/nested block
comments and ordinary quoted strings, lacks a complete Rust literal/attribute
model, and returns tokens even after unterminated constructs. Reusing that
heuristic as an exclusion proof would be inappropriate.

The exact local dependency versions are already available:

- `syn` **3.0.6**, `Cargo.lock:1195`.
- `proc-macro2` **1.0.107**, `Cargo.lock:763`.
- Their source directories are under
  `/home/jim/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/`.

`syn::parse_file` is present at local `syn-3.0.6/src/lib.rs:1108`; `visit` is exposed
at 875. Its Cargo features include `full`, `parsing` and `visit`. Neither library
is currently a direct dependency of `offline-verification`, so adding it requires
the normal Cargo.toml/lock update, not another package download. The first lexical
slice can use `proc_macro2::TokenStream` directly; the same owned fact interface
supports `syn` for precise item/attribute/macro classification.

Do not retain `TokenStream`/AST values in a cross-worker `Arc` cache:
`proc-macro2/src/lib.rs:82–84` documents thread-local, non-`Sync` types. Parse once
in its owning thread, extract ordinary `String`/`PathBuf`/enum facts, then discard
the AST. The immutable source bytes and these facts can be shared safely.

Bound source bytes, token/nesting inventory and total scanner work. Check the
current deadline before and after parsing, and during fact traversal. A parser
error or unsupported syntax must produce an explicit incomplete closure; exhausted
budget must remain a hard error. Do not catch all parser/resource errors and turn
them into an apparently complete empty dependency set.

Existing compiler dependency machinery is useful but has narrower guarantees:

- `tool-execution/src/builds.rs:521–552`, `dep_paths`: bounded Make-style first-rule
  parsing, escaped spaces/continuations, path count. It is a path parser, not
  proof that the dependency file includes every kind of build input.
- `builds.rs:147–162`: canonicalize emitted paths and compare complete expected
  versus actual emitted closure.
- `builds.rs:102–138`: standalone Rust recipes always perform `--emit=dep-info`,
  reject `# env-dep:` entries, build with emitted dependencies on cache misses,
  and compare the resulting closure. Reuse this behavior for eligible candidate
  Rust builds rather than wrapping raw `execute` in a cache.
- `builds.rs:431–453`: pinned C units ask the real compiler for `-M` dependencies,
  including system headers, before looking up an object. Cache misses compile
  with `-MD -MF` and compare actual dependencies.
- `offline-verification/src/inputs.rs:318–393`: ingests `compiler.d`, validates
  `/inputs/rust` and `/inputs/generated` mappings for Native, adds explicitly
  declared inputs, snapshots/hashes source, and retains implementation stamps.
  Unlike standalone helpers, this function currently does not inspect
  `# env-dep:` lines. Fresh ordinary Rust is currently uncached; adding a cache
  without addressing this difference would enlarge the accepted input contract.
- `family_model.rs:96–110`: existing `Implementation` has dependency contract,
  Rust dependency/harness maps, module/memory/binary hashes and original manifest
  identity. `decomp-source::Source` supplies the complete pinned C identity.

## Supported versus incomplete closure rules

| Construct/input | Safe rule for the first implementation | Requirement before more precise exclusion/reuse |
| --- | --- | --- |
| Line, block and doc comments | Preserve their source hash; no dependency names from their prose. | Use Rust tokenization, including nested block comments. |
| Ordinary/raw/byte/character literals | No module/input edge from text alone. | Treat a literal used by a recognized input macro differently from ordinary data. |
| Code identifier matching a known gameplay module | Retain a conservative possible edge. | Resolve imports/aliases/reexports before narrowing to only syntactic paths. |
| Direct module references/imports and cycles | Traverse current namespace, preserve explicit imports, deduplicate by logical path and checked identity. | Cover `use` groups, aliases, `self`/`super`/`crate`, raw identifiers and module rename/delete cases. |
| Inline `mod { ... }` | AST traversal can support it without adding a filesystem path. | Visit contained items/macros/attributes; do not omit a conditional subtree without a bound cfg context. |
| External `mod name;`, `#[path]` | Incomplete until module-resolution semantics and namespace are bound. | Resolve `name.rs` versus `name/mod.rs`, nested module base paths, literal `path`, aliases and additions that change resolution. |
| `include!`, `include_str!`, `include_bytes!` | Actual invocation remains an input/incompleteness reason; comment words do not. | Literal paths must be resolved relative to their containing source, read/hash/capture, recursively scan Rust includes, and be checked against emitted dependencies. Dynamic expressions require a separately modeled generator or stay incomplete. |
| `env!`, `option_env!` | Remain incomplete, or reject artifact reuse, until named environment inputs are modeled. | Bind name and value/state (unset differs from empty), include in computation keys, recheck after waits, and validate compiler `# env-dep:` records. |
| `cfg`/features/target | Use a conservative union of source dependencies; do not assume inactive code is absent. | Bind compiler target, edition, flags/features and environment. A changed effective cfg context cannot reuse a prior result merely because Git reports no source edit. |
| `cfg_attr`, macro-valued attributes | Incomplete unless the nested attribute/input forms are understood. | Inspect dynamic `path`, derive/attribute expansion, and `#[doc = include_str!(...)]`; discarding every `doc` attribute would hide a real input. |
| Declarative/custom/procedural macros | Keep an explicit incompleteness reason unless a reviewed macro rule covers them. | Builtin macro allowlists must account for shadowing/imports; traverse their token arguments. Rustc dep-info alone does not prove arbitrary proc-macro filesystem/environment behavior absent. |
| `std::fs`, `File`, environment/process/FFI/runtime access | Preserve conservative uncertainty. | Alias-aware source facts plus declared runtime input/capability contracts; do not infer no runtime I/O from the absence of two spellings. |
| Generated PLM | Keep the existing generated-input requirement (`affected.rs:343`). | Bind generator implementation, generator source inputs, output declaration, ROM and complete generated coverage. |
| Captured Ridley and external captures | Preserve the reviewed specialized closure and limits; unknown planner adapter stays affected. | Retain every declared capture, large event-file identity, adapter, original source and ROM; do not drop difficult external paths. |
| C source/headers/runtime | Keep full pinned Recovered C requirements and real compiler closure. | Narrow only with authenticated compiler/configuration facts; object, link and runtime inputs are different closures. |
| Missing/ignored/external/redirected input | Incomplete or hard rejection according to existing boundary; never proof of independence. | Bind safe path, membership, existence and content; preserve Git-history and symlink/alias checks. |

The token-only first slice can keep the old unsupported identifier list, making
the definite comment/literal correction without broadening supported syntax.
Supporting inline modules and literal includes is an explicit next step through
the same fact interface. Do not accidentally treat a `let include = ...` as an
include macro once AST classification is in place.

For execution, an incomplete *planning* closure still means the family must be
verified. Do not use a narrower parsed module set to remove formerly supplied
compiler inputs for unmodeled macros. Either retain a validated conservative
execution closure or refuse that unsupported binding explicitly. The actual
compiler's new dependency must be checked against captured inputs, not silently
added after the candidate has already compiled.

Retained dependency records can avoid a new compiler scan for **planning** only
when their source/configuration/namespace provenance is authenticated and still
matches. A stale `compiler.d`, an arbitrary declared map, or a prior success flag
cannot establish completeness. A changed root/manifest/configuration or newly
possible dependency makes the family affected. No need to compile the whole
catalog just to answer selection.

## Identity integration points

Keep the four identities from the plan separate:

- Captured logical input/content closure: ticket 02 bytes, names, sizes,
  namespace, source pin and declared external assets.
- Build computation: exact recipe-generated sources, compiler/toolchain/runtime,
  target/flags, environment and complete compiler/link input closure.
- Verification computation: builds plus complete Fixture declarations/order,
  initial/host/capture state, comparison implementation and fresh/reuse policy.
- Admission binding: required family set, candidate, original source,
  registration/provenance, installed contract and attempt/operation identity.

Use existing records rather than a second result ledger. Candidate lookup may
identify eligible evidence; existing authentication still establishes acceptance.
`ordinary.rs:57–69` currently gates reuse on manifest/kind then authenticates the
complete family. `family.rs:864–876` and 901–915 check current manifest, pinned C
and source/harness maps. Preserve these broad seals until narrower dependency
proofs are verified. A new scanner implementation must be included in
`offline-verification/src/sources.rs`; parser Cargo/lock identity also matters.

Native already separates dependency and verification hashes
(`native.rs:860–861`), approved candidate-path binding (709–729), current candidate
digest (1009–1015), and immutable operation context. Do not fold them all into one
cross-family artifact key or let a build cache hit stand in for an attempt receipt.

## What compiler work shares today

`tool-execution/src/build_cache.rs:107–155` prepares repository identity, selected
compiler path, `--version`, compiler executable hash, optional `rustc -vV`, and
compiler environment. That work repeats whenever `Preparation::new` is called.

`Preparation` then:

- freezes roots/headers (`160`), caching observed hashes (`190`);
- constructs `CacheIdentity` and a SHA-256 key (`208–245`);
- waits on that exact key's lock without a producer lease (`248–254`);
- rehashes original inputs after the wait (`253–255`);
- authenticates cache metadata and artifact bytes (`257–267`);
- checks compiler/environment before a miss, inputs/compiler after compilation
  (`283–295`), and atomically publishes (`298–303`);
- copies the artifact and verifies it against the already authenticated seal
  (`311–320`);
- rechecks every observed source, compiler and environment at `finish` (`325–331`).

There is already one build per identical eligible key under contention. The
opportunity is sharing preparation and eligible requests, not inventing another
object cache.

| Present caller/work | Current sharing | Concrete extension |
| --- | --- | --- |
| Pinned C object units, `builds.rs:368–458` | One `Preparation` per family C build; objects under canonical `checkout/deps` use the common authenticated cache. Identical source/flags/header/environment objects already hit across families. | Reuse compatible compiler preparation and input facts across families; retain final checks and real closure validation. |
| Generated C units, `builds.rs:454–456` | Always compiled directly; no cache entry. | Cache only after generated-byte/logical-path and actual dependency proofs. |
| C link, `builds.rs:503–507` | Always runs. | A link cache would need the complete object/archive/linker/runtime closure; outside the minimal preparation change. |
| Ordinary candidate Rust, `family.rs:444–450` and equivalent recipes | Direct `builds::execute`, no candidate artifact cache. | Adapt the existing standalone Rust scan/build/closure-validation pipeline once generated-path identity is proven. |
| Pristine Fixture worker, `offline-verification/builds.rs:697–718` | Calls `standalone_builds` for the same source/flags; final artifact normally shares, but toolchain selection/discovery runs again. | Prepare once per batch, authenticate/reference the same eligible artifact for every family, preserve required source/tool facts. |
| Standalone helpers, `tool-execution/builds.rs:74–144` | Up to three outputs already share one preparation. | Reuse its selected toolchain and preparation through the batch owner instead of repeated independent helper requests. |
| Cargo artifacts, `builds.rs:304–350` | Existing package-specific artifact cache; inputs supplied by caller. | Reuse compiler preparation only for matching tool/context; do not broaden caller input assumptions. |
| Native C/Rust, `offline-verification/builds.rs:219–241` | Runs exact confined compiler producers. It does **not** use ordinary `c_build`'s pinned-unit cache. | Share trusted input/runtime preparation where bound; retain both compiler producers until a new Native contract explicitly permits shared artifacts. |

### Sharing `Preparation` has a cancellation trap

`Preparation` contains one `AtomicBool aborted` (`build_cache.rs:96`). Any failing
C unit calls `preparation.abort()` (`builds.rs:475–478`), stopping siblings in that
family. Ordinary batch behavior lets other families finish and retain their
diagnostics. Passing one unmodified `Preparation` to all families would expand
that cancellation domain silently.

A minimal safe seam is an immutable shared compiler/toolchain/environment context
plus separate per-family preparation/abort state, or an equivalent scoped abort
parameter. Shared captured input identities can sit above both. Revalidate the
family's own observed closure before its publication; do not make unrelated
family output determine whether its compiler identity may be used. Retain global
batch cancellation independently. This change removes repeated compiler probes
without changing failure semantics.

Dependency scan coalescing is a later guarded part of the same ticket: `-M` and
Rust dep-info scans must be keyed by input bytes, namespace/include search paths,
toolchain, flags/configuration and environment. Without ticket 02's captured
namespace, a cache hit could hide a newly created header/include. Keep fresh scans
until that proof is implemented; current code deliberately scans even on warm
artifact hits.

## Which keys and generated bytes differ by run path

`CacheIdentity` version 4 (`build_cache.rs:38–47`, 230–244) includes normalized
command, normalized input path-to-digest map, compiler hash/version, selected
environment hash, host architecture/OS and optional Rust version details.
Only checkout/repository strings are replaced in arguments at 235–236; the
destination is not independently a key field. Callers use `{artifact}` as the
output placeholder, which is already the correct way to keep the physical output
destination out of an eligible object's identity.

Run paths under checkout still remain as `{checkout}/out/.../<run>/...`; paths
outside it stay absolute. More importantly, input bytes themselves can contain
those paths and their hashes remain different even if arguments are normalized.

| Location | Path dependence / legitimate differences |
| --- | --- |
| `recipe_commands.rs:256–268` | Rust command embeds `<run>/compiler.d`, `<run>/wrapper.rs` and output binary/staticlib path. Merely caching this command makes every run distinct. |
| `family.rs:350–352` | Ordinary wrapper embeds the absolute generated `memory.rs` path and original candidate module path. |
| `inputs.rs:272–285` | Generated `memory.rs` contains absolute original gameplay source paths; equivalent checkouts can still produce different bytes. |
| `callback.rs:102–113`, `plm_spawn.rs:28–42`, `ridley_subsystem_adapter.rs:211–227` | Ordinary wrappers also embed generated-memory and module paths. Callback ABI additionally embeds the declared routine-name set (`callback.rs:13–31`), which is a legitimate semantic difference. |
| `family.rs:383`, `401–427` | Fixture-worker and C shim/headless includes can embed checkout/pinned-source paths. |
| `recipe_commands.rs:273–397` | C uses fixed source/include flags plus family output/seam/oracle/staticlib paths. C object splitting replaces only `-o`; generated units bypass the cache. |
| `recipe_commands.rs:284` | PowerBomb/GameHost/PLM compile at `-O0`; other recipes at `-O2`. These are intentionally different object keys. |
| `recipe_commands.rs:294–306`, `335–339` | Some recipes replace specific C banks with different generated seams. Those inputs cannot share just because their original bank name matches. |
| `offline-verification/builds.rs:734–804` | Native already maps recipe paths to `/inputs/rust`, `/inputs/c`, `/inputs/generated` and `/work/compiler.d`; generated bytes use canonical layout. The `Program` still binds exact host sources and operation/producer identity. |

A safe path improvement is recipe-owned relative/logical generated input naming,
not replacing arbitrary substrings in compiler commands. For example, a generated
Rust wrapper can potentially refer to sibling `memory.rs` relatively, but the
compiler source argument, dep-info path, embedded user source paths and failure
diagnostics still need an explicit logical-path contract.

Before two physical directories can share, test `file!`, include resolution,
panic/source locations, `__FILE__`/`__BASE_FILE__`, debug paths and toolchain
selection. Link/library paths and `__DATE__`/`__TIME__` can also affect artifact
bytes. Equal source function names do not prove equal builds. Generated Rust for
different families may or may not be identical; inspect actual recipe bytes and
typed flags, never use family names as the computation key.

The cache's environment allowlist (`build_cache.rs:54–80`) covers `CARGO_*`,
`RUST*`, `CC_*` and named compiler/search variables. It does not cover arbitrary
variables referenced by `env!` or runtime code. Compiler binary/version alone also
does not prove an independently modified sysroot/linker library is unchanged.
Expanding cached linked-artifact use requires binding those actual inputs.

## Exact Native barriers to dropping shared compiler work

1. `native.rs:1452–1462`, `operation_slots`, unconditionally requires one
   `c-build` and one `rust-build` in every family run, plus optional ABI controls
   and both actual subjects for every Fixture.
2. `native.rs:1390–1419` reconstructs the expected slot set, rejects unexpected
   or duplicate receipts, checks each producer, validates reservations, and
   requires all slots for a passing family.
3. `native.rs:1522–1528` derives the producer operation key from the family
   operation key, directory and label, then checks the saved context and request
   hash. An operation from another family is not the same producer.
4. `native.rs:1530–1542` checks exact frozen compiler/Fixture limits; shortening
   timeout is allowed only within that original contract.
5. `native.rs:1552–1624` checks installed compiler/helper/runtime, reconstructs
   the exact recipe `CompilerCommand` and confined `Program`, and compares its
   arguments, input capabilities and outputs. Supplying objects or a different
   cache helper instead of the compiler fails that check.
6. `native.rs:1648–1678` authenticates the exact durable operation result,
   references, logs and copied outputs. Matching final artifact bytes alone
   cannot replace producer attribution.
7. `builds.rs:254–343` creates per-producer `StrictInvocation`, reserves its
   output, saves immutable request/context/limits, and allows same-attempt
   reconciliation without re-executing a completed producer. That is distinct
   from cross-family or cross-attempt compilation reuse.

Thus the first Native improvement can share trusted preparation facts, not erase
compiler slots. `translation-execution/src/compiler_runtime.rs:18–112` repeatedly
discovers installed runtime bind paths; it does not cache authenticated runtime
contents. Sharing its path inventory needs retained namespace checks, while the
operation runner must still authenticate each actual input image. Keep the early
retained-receipt branch before runtime discovery (`native.rs:1049–1067`).

If measurements justify Native artifact sharing, introduce a current contract
with explicit authenticated shared-build references, producer ownership,
consumer dependency bindings, and peak/retained reservations. Revise expected
slots, exact Program reconstruction and installed source/policy together. Do not
create synthetic `c-build`/`rust-build` success receipts for a restored artifact.
This requires fresh frozen integration/installation; existing production policy
is not edited in place.

## Tests to implement or extend

Dependency controls through public selection/input facts:

- Both observed `include` comments, ordinary/raw/byte strings and nested comments
  cause no edge/refusal; exact source hashes still change when prose changes.
- Real literal include macros create captured dependencies or explicit
  uncertainty. Nested/dynamic includes, `env!`, `option_env!`, proc macros and
  macro-valued attributes never become complete empty closures.
- Raw identifiers, grouped/aliased imports, inline/external modules, cfg branches,
  missing/ignored inputs, newly created modules/headers and renames/deletions.
- Root/manifest/source change during discovery or before publication; fake/stale
  dep-info and wrong compiler/configuration identity cannot exclude/reuse.
- Compare the parsed source facts used by planning and wrapper generation. Keep
  established module placement/type identity and actual-C Fixture observations.

Use `pipeline/tests/affected.rs:84–206`, existing source-context tests, and adapt
`offline-verification/src/inputs.rs:418` to valid Rust if the new parser rejects
its current synthetic repeated bare identifier text.

Preparation/cache controls through the bulk/public build interface:

- Two compatible families cause one compiler identity preparation; incompatible
  toolchain/environment/flags do not share. Count actual version/discovery/build
  processes rather than assert noisy wall-clock ratios.
- Existing equal-key requests compile once under contention and authenticate
  each restored artifact. Test corrupt cache bytes and changed sources while
  waiting, including a newly introduced transitive include.
- A failing unit cancels its family siblings while another family continues and
  retains diagnostics; batch cancellation still stops all owned descendants.
- Matching logical generated inputs in different physical directories share only
  after path-sensitive controls pass; differing routine sets, ABI/seams or
  optimization flags remain distinct.
- Fresh C/Rust calls remain complete when builds hit. Reused results, fresh
  verification with warm builds, and same-attempt Native replay remain distinct.
- Existing Native tests must reject removed/duplicated/foreign compiler slots and
  changed runtime/Program/limits/reservations, and keep independent admission
  working after cleanup without compiler/runtime discovery.

Existing concrete build tests are in `pipeline/tests/build_preparation.rs`:
154 (one preparation plus independent units), 182 (mutation during key wait),
251 (discovered Rust inputs/corruption/env-dep refusal), 407 (transitive closure
changes before publication), 485 (failed-unit cancellation), 503 (SIGTERM reaping),
and 528 (checkout toolchain versus run override). Extend these behavioral controls
for batch-scoped sharing rather than replace them with internal key-shape tests.

## Unresolved proof obligations and handoff

- Actual corpus completeness after removing comment false positives must be
  measured; subsequent unknowns are not resolved by this research.
- Ticket 02's immutable captured input/namespace implementation is prerequisite
  to coalescing dependency discovery or trusting old closure facts.
- Exact generated-code groups, physical-path equivalence and complete linked
  runtime identity require controls before extending the artifact cache.
- A shared compiler context needs an explicit compatible key and separate family
  abort/finish behavior; existing `Preparation` cannot simply become global.
- Native shared artifacts need a new current producer contract; Native preparation
  sharing alone must retain the existing slot, runtime and replay requirements.
- Existing compiler environment/sysroot behavior must remain conservative until
  its actual closure is represented. No absence of macro/env/generated inputs
  has been asserted.

Only this note was written in the authorized main `.scratch` directory. No
source edits, builds, producers, verification runs or worktrees were created.
No speedup or passing verification is claimed; run cleanup is inapplicable to
this read-only research. Leave the frozen checkout with its independent owner.
