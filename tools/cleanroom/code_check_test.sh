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
list=$(mktemp)
trap 'rm -f "$list"' EXIT
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

# The vector runner, the confined implementer's one command, refuses each shape fixture with the
# tag code_check.sh names, and passes the clean one.
shapes=0
for dir in "$here"/code_check_fixtures/clean/ "$here"/code_check_fixtures/{argument-object,layering}__*/; do
  name=$(basename "$dir"); tag=${name%%__*}; shapes=$((shapes + 1))
  out=$(cd "$here" && "${SM_PY:-python3}" -I -c '
import shutil, sys
from pathlib import Path
sys.path.insert(0, ".")
from vector_runner import shape_findings
print("\n".join(shape_findings(shutil.which("awk"), sorted(Path(sys.argv[1]).glob("crates/clean/*/")))))
' "$dir" 2>&1) || { fail "the runner's shape scan did not run on $name: $out"; continue; }
  if [ "$name" = clean ]; then
    [ -z "$out" ] || fail "the runner's shape scan refused the clean fixture: $out"
  else
    grep -q ": $tag: " <<<"$out" || fail "the runner's shape scan did not name $tag in $name: $out"
  fi
done
[ "$bad" -eq 0 ] && echo "CODE CHECK FIXTURES OK: clean passes, $n forbidden fixtures each named; L4's warn list holds; the runner's shape scan names $((shapes - 1)) shape fixtures"
exit $bad
