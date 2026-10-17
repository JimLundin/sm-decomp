# Prospective DLL build receipt

This Rust library owns one local build transaction. Integration calls `build`
from its admitted DLL launcher. Python is allowed for that DLL host launcher;
project tools, orchestration and tests remain Rust.
It freezes declared local, registry, toolchain, linker and configuration roots,
queries Cargo's selected normal/build graph and resolved features, executes the
fixed restart DLL command in a fully explicit environment, checks complete input
membership and hashes, verifies every fresh-target dep-info against frozen
files, and publishes a receipt only for a successful build. Output uses the maintained SME
codec; immutable inventory shards stay within its size/node limits.

The caller must declare the complete relevant external input roots. The receipt
reports that declared scope; it cannot discover an undeclared dynamic native
library or environment dependency.
Independent review must compare it with the selected graph and compiler/linker
inputs. Rust sysroot, selected registry sources including build scripts, linker
libraries and effective Cargo configuration are mandatory caller responsibilities.
The command selects an explicit Windows GNU target and a fresh target directory
under receipt output. Every produced target file is retained as generated output;
all emitted dep-info is checked, including the final DLL-to-dep-info association.
This intentionally requires a fresh build. Independent review still compares
selected graph packages/build scripts with the emitted dependency inventory.
Fresh output provenance assumes integration's sole-writer producer lease and
supervised process-tree completion. Pre/post hashes cannot detect transient
modifications restored before observation.

Each output directory must be fresh and outside captured roots. Failed builds
retain diagnostics and snapshots, with no success receipt. Cleanup belongs to
integration: preserve referenced snapshots, inventories, logs and DLL; remove
only identified unsealed intermediates and reauthenticate afterward.

No build, test execution or Clippy result is claimed by authored source. The
integration owner runs formatting, warnings-denied Clippy, meaningful tests and
the local DLL acceptance build on frozen reviewed source. This library uses a
separate workspace so its checks must select this manifest explicitly.

`focused_controls` shares filesystem refusal controls with ordinary tests and
the admitted DLL. Its count establishes only those controls; it does not execute
Cargo or establish build/graph completeness. Real compiler execution and
independent selected-input completeness remain integration gates.
`authenticate` checks typed required fields and every retained reference. It is
an integrity reader; it does not certify external graph/input completeness.
The temporary `receipt-pending.sme` is authenticated before `receipt.sme` is
published and can be removed as an identified intermediate after completion.

The package also builds a DLL with the single-use `sm_restart_run` boundary.
Pass exactly two UTF-8 arguments: `controls DIRECTORY`, `authenticate RECEIPT`
or `build REQUEST`. The existing Python DLL host launches the Rust operation.

REQUEST is bounded SME using schema `sm-build-request-v1`. Exact fields are
`schema`, absolute UTF-8 strings `checkout`, `output`, `cargo`, a string-valued
`environment` table, and `inputs` array. Each input is exactly a table with
absolute string `path` and absolute-string array `excluded`. Unknown fields,
wrong types, invalid environment entries, excessive roots/exclusions and lexical
path overlap refuse before output creation. Admission resolves existing paths,
rechecks overlap and requires the request itself in a nonexcluded captured root.
`encode_request` emits this format from typed Rust inputs through the same
validator. Integration owns the selected source/tool/config root inventory and
explicit environment; it must freeze the request and all inputs for the entire
supervised build. Cargo arguments remain the library's fixed recipe.
