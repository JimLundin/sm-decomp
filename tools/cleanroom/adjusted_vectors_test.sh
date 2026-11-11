#!/usr/bin/env bash
# adjusted_vectors_test.sh: adjusted_vectors.sh, on adjusted_vectors_fixtures, names the one
# Parameter entry no vector adjusts and no other, passes without --gate and fails with it.
here=$(cd "$(dirname "$0")" && pwd)
bad=0
fail() { echo "FAIL $*"; bad=1; }
fixture=$here/adjusted_vectors_fixtures
out=$(bash "$here/adjusted_vectors.sh" -C "$fixture" demo 2>&1) || fail "advisory run failed: $out"
[ "$(grep -c 'has no adjusted vector' <<<"$out")" -eq 1 ] || fail "not exactly one entry named: $out"
grep -qx 'adjusted: demo: Missing has no adjusted vector' <<<"$out" || fail "Missing not named: $out"
grep -qx 'adjusted: demo: 2 of 3 Parameter entries have an adjusted vector' <<<"$out" || fail "wrong count: $out"
bash "$here/adjusted_vectors.sh" --gate -C "$fixture" demo >/dev/null 2>&1 && fail "--gate passed an entry without one"
[ "$bad" -eq 0 ] && echo "ADJUSTED VECTORS OK: the one entry without an adjusted vector named; --gate refuses it"
exit $bad
