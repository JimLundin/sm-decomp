#!/usr/bin/env bash
# Build the Gate 1 slice (sm_replay and sm_slice) from a copy of the checkout
# with sm/ deleted, so the build fails if anything still reaches into sm/.
# Usage: .github/scripts/gate1_absent.sh COPY_DIRECTORY
# The copy drops "sm" from the workspace members and sm_app's optional
# dependency on it (only the desktop `frontend` feature uses it). Cargo resolves
# offline from what `cargo fetch --locked` downloaded; the only change it may
# make to Cargo.lock is to remove packages.
set -euo pipefail
copy=$1
rm -rf -- "$copy"
mkdir -p -- "$copy"
git ls-files -z | grep -zv '^sm/' | xargs -0 cp --parents -t "$copy" --
cd "$copy"
test ! -e sm
sed -i 's/"sm", //' Cargo.toml
sed -i -e '/^sm = { path = "..\/..\/sm"/d' -e 's/"dep:sm", //' crates/sm_app/Cargo.toml
if grep -nE '^members = .*"sm"|path = "(\.\./)+sm"|"dep:sm"' Cargo.toml crates/*/Cargo.toml; then
  echo "the copy still names the sm package"; exit 1
fi
cp Cargo.lock Cargo.lock.before
cargo build --offline --release -p sm_tools --features replay --bin sm_replay
cargo build --offline --release -p sm_app --no-default-features --features slice --bin sm_slice
diff Cargo.lock.before Cargo.lock || true
if diff Cargo.lock.before Cargo.lock | grep '^>'; then
  echo "the build added to Cargo.lock"; exit 1
fi
echo "GATE1 ABSENT OK"
