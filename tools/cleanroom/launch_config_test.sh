#!/usr/bin/env bash
# launch_config_test.sh: launch.sh's confinement as written, on this machine's branch, without a
# session: CLEANROOM_DRY_RUN=1 on a throwaway probe export with a fake CLI. Off Windows it checks
# that the deny rules cover the home around the exports' root, the other exports, the temporary
# folder, the canary record, the project's checkout and the system folders, but not the export;
# that the environment is the lane's; that the key from ANTHROPIC_API_KEY stays out of the
# settings and the record; and that the lane's `cleanroom-tests` runs the shape scan and the
# export's tests. On Windows it checks the rules it has always written. The probe's scan builds
# the cleanroom scanner, as score_scan_test.sh does.
set -u
here=$(cd "$(dirname "$0")" && pwd)
repo=$(cd "$here/../.." && pwd -P)
case "$(uname -s)" in
  MINGW* | MSYS* | CYGWIN*) windows=1 py_default=$LOCALAPPDATA/sm-decomp-tools/python-3.13.7/python.exe ;;
  *) windows=0 py_default=python3 ;;
esac
tmp=$(mktemp -d "${TEMP:-${TMPDIR:-/tmp}}/launch-config-XXXXXX")
tmp=$(cd "$tmp" && pwd -P)
trap 'rm -rf "$tmp"' EXIT
bad=0
fail() { echo "FAIL $*"; bad=1; }
python=${CLEANROOM_PY:-${SM_PY:-$py_default}}

# Off Windows the session's home and temporary folder are the test's, so the rules name them.
if [ "$windows" = 0 ]; then
  export CARGO_HOME=${CARGO_HOME:-$HOME/.cargo} RUSTUP_HOME=${RUSTUP_HOME:-$HOME/.rustup}
  export HOME="$tmp/home" TEMP="$tmp/scratch"
  mkdir -p "$HOME/cleanroom/other-domain" "$TEMP/c4a/logs"
  echo private >"$HOME/notes.txt"
  root="$HOME/cleanroom"
else
  root="$tmp/cleanroom"
fi
export CLEANROOM_PRIVATE="$tmp/home/.cleanroom" CLEANROOM_DRY_RUN=1
mkdir -p "$CLEANROOM_PRIVATE" "$tmp/bin"
printf '#!/bin/sh\nexit 3\n' >"$tmp/bin/claude" && chmod 755 "$tmp/bin/claude"
export CLAUDE_BIN="$tmp/bin/claude"

# The export: one commit, no remote, the shared types and the probe's model crate, each with a
# test, so the runner has targets.
ex="$root/launch-probe"
mkdir -p "$ex/crates/clean/types/src" "$ex/crates/clean/launch_probe/src"
printf '[workspace]\nresolver = "3"\nmembers = ["crates/clean/types", "crates/clean/launch_probe"]\n' >"$ex/Cargo.toml"
for c in types launch_probe; do
  printf '[package]\nname = "%s"\nversion = "0.1.0"\nedition = "2024"\npublish = false\n' "$c" \
    >"$ex/crates/clean/$c/Cargo.toml"
  printf '//! A crate.\n#[cfg(test)]\nmod tests {\n    #[test]\n    fn holds() {}\n}\n' \
    >"$ex/crates/clean/$c/src/lib.rs"
done
printf '/target\n/Cargo.lock\n' >"$ex/.gitignore"
git -C "$ex" init -q -b main && git -C "$ex" -c core.autocrlf=false add -A &&
  git -C "$ex" -c user.name=t -c user.email=t@invalid -c commit.gpgsign=false commit -q -m export ||
  { echo "FAIL the test export"; exit 1; }

# launch.sh --probe EXPORT; prints the lane directory.
launch() {
  out=$(cd "$repo" && bash "$here/launch.sh" --probe "$ex" "$tmp/transcripts/t.jsonl" 2>&1) || {
    echo "FAIL launch.sh $*: $out"; return 1; }
  sed -n 's/^LAUNCH DRY RUN launch-probe //p' <<<"$out"
}

printf '  "apiKeyHelper": "echo test-key",\n  "env": {},\n' >"$tmp/auth"
lane=$(CLEANROOM_AUTH="$tmp/auth" launch "with CLEANROOM_AUTH") || exit 1
[ -d "$lane" ] || { echo "FAIL no lane directory"; exit 1; }
"$python" -I -c 'import json, sys; json.load(open(sys.argv[1], encoding="utf-8"))' "$lane/settings.json" ||
  fail "settings.json is not JSON"
grep -q 'test-key' "$lane/settings.json" || fail "CLEANROOM_AUTH is not in the settings"
grep -qE '@[A-Z_]+@' "$lane/brief.txt" && fail "the probe brief keeps a placeholder"
grep -q '^PYTHON is "' "$lane/brief.txt" || fail "the probe brief doesn't name PYTHON"
[ "$(head -1 "$lane/argv.txt")" = "$CLAUDE_BIN" ] || fail "the CLI is not CLAUDE_BIN"

if [ "$windows" = 1 ]; then
  for rule in 'Read(//c/Users/**)' 'Grep(//c/Users/**)' 'Read(//d/**)'; do
    grep -qF "\"$rule\"" "$lane/settings.json" || fail "no deny rule $rule"
  done
  grep -q 'vector_runner.py' "$lane/brief.txt" || fail "the brief has no runner command"
  [ "$bad" -eq 0 ] && echo "LAUNCH CONFIG TEST OK (Windows)"
  exit "$bad"
fi

# Which paths the deny rules cover: a rule `Read(//P)` the path P, `Read(//P/**)` all under P.
covered=$("$python" -I - "$lane/settings.json" "$ex/Cargo.toml" "$HOME/notes.txt" \
  "$CLEANROOM_PRIVATE/tokens.tsv" "$HOME/cleanroom/other-domain/x" "$TEMP/c4a/logs/b90.txt" \
  "$repo/AGENTS.md" /etc/passwd <<'PY'
import json, sys
deny = json.load(open(sys.argv[1], encoding="utf-8"))["permissions"]["deny"]
def covers(tool, path):
    for rule in deny:
        if not rule.startswith(tool + "(//"):
            continue
        target = rule[len(tool) + 2:-1]
        if target == path or (target.endswith("/**") and path.startswith(target[:-2])):
            return True
    return False
for path in sys.argv[2:]:
    tools = [t for t in ("Read", "Glob", "Grep", "Edit", "Write") if covers(t, path)]
    print(path, ",".join(tools) or "none")
PY
) || fail "the deny rules can't be read"
grep -qx "$ex/Cargo.toml none" <<<"$covered" || fail "the export is denied: $(grep "^$ex/" <<<"$covered")"
for p in "$HOME/notes.txt" "$CLEANROOM_PRIVATE/tokens.tsv" "$HOME/cleanroom/other-domain/x" \
  "$TEMP/c4a/logs/b90.txt" "$repo/AGENTS.md" /etc/passwd; do
  grep -qx "$p Read,Glob,Grep,Edit,Write" <<<"$covered" || fail "not denied every tool: $(grep "^$p " <<<"$covered")"
done
grep -qx 'Bash(cleanroom-tests)' < <("$python" -I -c 'import json, sys; print("\n".join(json.load(open(sys.argv[1], encoding="utf-8"))["permissions"]["allow"]))' "$lane/settings.json") ||
  fail "the runner command is not allowed"
grep -qx "HOME=$lane/home" "$lane/env.txt" || fail "HOME is not the lane's home"
grep -qx "CLAUDE_CONFIG_DIR=$lane/home/.claude" "$lane/env.txt" || fail "the CLI's config is not the lane's"
grep -qx "TMPDIR=$lane/tmp" "$lane/env.txt" || fail "TMPDIR is not the lane's"
grep -q "^PATH=$lane/bin:" "$lane/env.txt" || fail "the lane's bin/ does not lead PATH"
grep -q '^USERPROFILE=' "$lane/env.txt" && fail "a Windows variable off Windows"
grep -q 'bin/cleanroom-tests' "$lane/vector_runner.sha256" || fail "the runner script is not in the digest"
[ "$(tail -1 "$lane/awk.path" | cut -c1)" = / ] || fail "awk.path is not absolute"

# The key from the environment: in the CLI's environment, never in the settings or the record.
lane2=$(ANTHROPIC_API_KEY=sk-test-launch-secret launch "with ANTHROPIC_API_KEY") || exit 1
grep -q 'sk-test-launch-secret' "$lane2/settings.json" "$lane2/env.txt" "$lane2/argv.txt" &&
  fail "the API key is written down"
grep -qx 'ANTHROPIC_API_KEY=<redacted>' "$lane2/env.txt" || fail "the API key is not passed"
(unset ANTHROPIC_API_KEY; launch "without a key" >/dev/null 2>&1) && fail "a launch without a key goes on"

# The test command as the session runs it: the lane's environment alone, from the export.
mapfile -t envv <"$lane/env.txt"
run=$(cd "$ex" && env -i "${envv[@]}" cleanroom-tests 2>&1) || fail "cleanroom-tests failed: $run"
grep -q 'vector_runner: all targets passed' <<<"$run" || fail "the runner did not pass: $run"
grep -q 'test result: ok. 1 passed' <<<"$run" || fail "the export's tests did not run: $run"
# Only the session's PATH finds the script, and an argument is refused.
(cd "$ex" && env -i "${envv[@]}" cleanroom-tests --help >/dev/null 2>&1) && fail "the runner takes an argument"

# The Linux branch under shellcheck where it is installed (CI's runner has it).
if command -v shellcheck >/dev/null; then
  shellcheck -S error "$here/launch.sh" "$here/export.sh" "$here/../lane/worktree.sh" >"$tmp/sc" 2>&1 ||
    fail "shellcheck: $(cat "$tmp/sc")"
fi
[ "$bad" -eq 0 ] && echo "LAUNCH CONFIG TEST OK"
exit "$bad"
