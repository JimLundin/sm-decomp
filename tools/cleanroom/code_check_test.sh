#!/usr/bin/env bash
# code_check_test.sh: code_check.sh passes code_check_fixtures/clean and, on every other fixture
# (named TAG__variant), fails and names TAG; L4 warns in a crate its warn list names, except
# under --import, and refuses a listed crate with none left. Run from anywhere; exit 1 lists what
# went wrong.
here=$(cd "$(dirname "$0")" && pwd)
bad=0
fail() { echo "FAIL $*"; bad=1; }

out=$(bash "$here/code_check.sh" "$here/code_check_fixtures/clean" 2>&1) || fail "clean fixture refused: $out"
n=0
for dir in "$here"/code_check_fixtures/*__*/; do
  name=$(basename "$dir"); tag=${name%%__*}; n=$((n + 1))
  if out=$(bash "$here/code_check.sh" "$dir" 2>&1); then
    fail "$name passed"
  else
    grep -q ": $tag: " <<<"$out" || fail "$name did not report $tag: $out"
  fi
done
[ "$n" -gt 0 ] || fail "no fixtures"

# L4 warns in a crate the warn list names, refuses there under --import, and refuses a listed
# crate with no variant import left.
scratch=$(mktemp -d) || exit 1
trap 'rm -rf "$scratch"' EXIT
list=$scratch/warn-list
echo crates/clean/demo >"$list"
listed=$here/code_check_fixtures/variant-import__listed
out=$(CODE_CHECK_VARIANT_WARN=$list bash "$here/code_check.sh" "$listed" 2>&1) || fail "a listed crate's variant import refused: $out"
grep -q ': warning: .*: variant-import: ' <<<"$out" || fail "a listed crate's variant import not warned: $out"
if CODE_CHECK_VARIANT_WARN=$list bash "$here/code_check.sh" --import "$listed" >/dev/null 2>&1; then
  fail "--import passed a listed crate's variant import"
fi
if out=$(CODE_CHECK_VARIANT_WARN=$list bash "$here/code_check.sh" "$here/code_check_fixtures/clean" 2>&1); then
  fail "a listed crate with no variant import passed"
else
  grep -q ': variant-import: listed in ' <<<"$out" || fail "a stale list line not named: $out"
fi

# An inherited declaration takes its source, version and features from the target
# workspace. The checked-out repository supplies the approvals.
ws=$scratch/workspace
cp -r "$here/code_check_fixtures/clean" "$ws" || exit 1
member=$ws/crates/clean/demo/Cargo.toml
approved_serde=$(sed -n '/^serde = /p' "$here/../../Cargo.toml")
write_workspace() {
  printf '[workspace.package]\nedition = "2024"\n\n[workspace.dependencies]\n%s\n' "$1" >"$ws/Cargo.toml"
}
write_member() {
  printf '[package]\nname = "demo"\nedition.workspace = true\n%s\n\n[lints]\nworkspace = true\n\n[%s]\n%s\n' \
    "${3:-}" "$1" "$2" >"$member"
}
refuses() {
  if out=$(bash "$here/code_check.sh" "$ws" 2>&1); then
    fail "$1 passed"
  else
    grep -q ": $2: " <<<"$out" || fail "$1 did not report $2: $out"
  fi
}
write_workspace "$approved_serde"
write_member dependencies 'serde = { workspace = true, features = ["derive"] }'
out=$(bash "$here/code_check.sh" "$ws" 2>&1) || fail "approved inheritance refused: $out"
write_workspace 'serde = "=0.0.1"'
refuses 'wrong workspace version' dep-registry
write_workspace "serde = { version = ${approved_serde#*= }, features = ['rc'] }"
refuses 'extra workspace feature' dep-registry
write_workspace "$approved_serde"
write_member dependencies 'serde = { workspace = true, features = ["rc"] }'
refuses 'extra member feature' dep-registry
write_member dependencies 'serde.workspace = true' 'workspace = "../.."'
refuses 'member selecting another workspace' dep-workspace
write_member dependencies 'toml.workspace = true'
write_workspace "$(sed -n '/^toml = /p' "$here/../../Cargo.toml")"
refuses 'dev dependency used as normal' dep-registry
write_workspace 'toml = { path = "crates/tools" }'
refuses 'workspace path outside clean crates' dep-path
write_workspace 'toml = { version = "=1", registry = "other" }'
refuses 'alternate workspace registry' dep-git
write_workspace 'toml = {'
refuses 'malformed workspace TOML' manifest

# The vector runner, the confined implementer's one command: a shape fixture fails its run with
# the tag code_check.sh names and the clean one passes; an awk not pinned outside the export
# refuses before any test.
out=$("${SM_PY:-python3}" -I "$here/vector_runner_test.py" 2>&1) || fail "the vector runner's wiring: $out"
[ "$bad" -eq 0 ] && echo "CODE CHECK FIXTURES OK: clean passes, $n forbidden fixtures each named; workspace inheritance checked; L4's warn list holds; the vector runner fails on a shape finding and refuses an unpinned awk"
exit $bad
