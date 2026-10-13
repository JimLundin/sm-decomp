# Exact bounded integration source context

Status: retained; integration gate and post-cleanup authentication pending.

`/tmp/sm-recovery-bounded-source` contains copies of the frozen crates, tools,
workspace manifests and toolchain, plus the exact `sm-pipeline` executable.
It contains no Git worktree or build tree. The exact ROM is copied; empty
mount-point directories preserve original source/archive and evidence paths
through explicit private namespace binds.

Executable SHA256: `9d91d2aa83942bbc30c8048dfa82db40fc3f5641d10a08355f922ae19fcc0ae1`.
`batch.rs` SHA256: `4aa716117c3cfd0fae0597a13f0fdfac7a0520c0cf99f4ef8905325763201219`.
`main.rs` SHA256: `1c2076de53340f6fd0af0489db696da299134f11d9a07bfd997fc9f10e47b3f4`.

After root source changes, authenticate exact frozen families inside a private
read-only mount namespace binding this context at `/home/jim/sm-decomp` and
binding the original evidence directory back at `/home/jim/sm-decomp/out`, plus
the pinned source/archive directories and read-only Git metadata. Bind `/dev`
with `--dev-bind /dev /dev` and `/tmp` writable for temporary source extraction.
`bwrap --ro-bind / / -- /usr/bin/true` succeeds in this environment. This
preserves absolute candidate/manifest identities without changing old seals
or reverting active shared source. Retain original `/tmp` fixture/snapshot
dependencies, pinned `deps/downloads` and shared packed evidence reader/store.

The earlier camera and broad Ridley controlled evidence retain separate exact
source contexts `/tmp/sm-recovery-camera-source` and
`/tmp/sm-ridley-controlled-source`. Their existing absolute candidate/manifest
files remained exact at snapshot creation. Current camera frozen post-cleanup
authentication passed; Ridley controlled authentication is still running.

The complete namespace command is retained in
`recovery-frozen-authentication-argv.txt`. It succeeds for the current camera
family after cleanup; receipt `recovery-bounded-namespace-authentication.toml`.
This proof required explicit `/dev`, original deps, evidence and Git metadata
binds; direct absolute symlinks inside the overlaid checkout would loop and were
replaced with empty mount points. Old trial diagnostics are retained honestly.
The snapshot requires no alternate Git worktree and has no mutable build tree.
