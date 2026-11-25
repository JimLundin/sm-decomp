#!/usr/bin/env bash
# launch_config_test.sh: launch.sh's confinement as written, on this machine's branch, without a
# session: CLEANROOM_DRY_RUN=1 on a throwaway probe export with a fake CLI. Off Windows it checks
# that the deny rules cover the home around the exports' root, the other exports, the temporary
# folder, the canary record, the project's checkout and the system folders, but not the export;
# that the environment is the lane's; that the key from ANTHROPIC_API_KEY stays out of the
# settings, the record and the box; and that the lane's `cleanroom-tests` runs the shape scan and
# the export's tests. Then it runs the box from the record's box.txt, with the fake CLI trying
# each way out from inside (the home, the canary record, the temporary folder, the checkout, a
# search from /, writes into the lane, the network, the key) and the relay passing a request to
# a fake API with the key added; it needs docker. On Windows it checks the rules it has always
# written. The probe's scan builds the cleanroom scanner, as score_scan_test.sh does.
set -u
here=$(cd "$(dirname "$0")" && pwd)
repo=$(cd "$here/../.." && pwd -P)
case "$(uname -s)" in
  MINGW* | MSYS* | CYGWIN*) windows=1 py_default=$LOCALAPPDATA/sm-decomp-tools/python-3.13.7/python.exe ;;
  *) windows=0 py_default=python3 ;;
esac
tmp=$(mktemp -d "${TEMP:-${TMPDIR:-/tmp}}/launch-config-XXXXXX")
tmp=$(cd "$tmp" && pwd -P)
sibling=
trap 'rm -rf "$tmp" ${sibling:+"$sibling"}' EXIT
bad=0
fail() { echo "FAIL $*"; bad=1; }
python=${CLEANROOM_PY:-${SM_PY:-$py_default}}

# Off Windows the session's home and temporary folder are the test's, so the rules name them.
if [ "$windows" = 0 ]; then
  export CARGO_HOME=${CARGO_HOME:-$HOME/.cargo} RUSTUP_HOME=${RUSTUP_HOME:-$HOME/.rustup}
  export HOME="$tmp/home" TEMP="$tmp/scratch"
  mkdir -p "$HOME/cleanroom/other-domain" "$TEMP/c4a/logs"
  echo private >"$HOME/notes.txt"
  # A dotfile in a folder on the export's path, which only the walk's `.[!.]*` names.
  mkdir -p "$HOME/.ssh" && echo key >"$HOME/.ssh/id"
  # A sibling of the main checkout, which the rules name as sm-decomp*.
  main=$(cd "$(git -C "$repo" rev-parse --path-format=absolute --git-common-dir)/.." && pwd -P)
  candidate="$(dirname "$main")/sm-decomp-launch-test-$$"
  [ -e "$candidate" ] || { mkdir "$candidate" 2>/dev/null && sibling=$candidate; }
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

# launch.sh --probe on $probe_export (default the export); prints the lane directory.
launch() {
  out=$(cd "$repo" && bash "$here/launch.sh" --probe "${probe_export:-$ex}" "$tmp/transcripts/t.jsonl" 2>&1) || {
    echo "FAIL launch.sh $*: $out"; return 1; }
  sed -n 's/^LAUNCH DRY RUN launch-probe //p' <<<"$out"
}

printf '  "apiKeyHelper": "echo test-key",\n  "env": {},\n' >"$tmp/auth"
lane=$(CLEANROOM_AUTH="$tmp/auth" launch "with CLEANROOM_AUTH") || exit 1
[ -d "$lane" ] || { echo "FAIL no lane directory"; exit 1; }
"$python" -I -c 'import json, sys; json.load(open(sys.argv[1], encoding="utf-8"))' "$lane/settings.json" ||
  fail "settings.json is not JSON"
# On Windows the credential is in the settings; elsewhere it stays with the relay outside the box.
if [ "$windows" = 1 ]; then
  grep -q 'test-key' "$lane/settings.json" || fail "CLEANROOM_AUTH is not in the settings"
else
  grep -q 'echo test-key' "$lane/settings.json" && fail "the key helper is in the box's settings"
  grep -q '"ANTHROPIC_API_KEY": "cleanroom-relay"' "$lane/settings.json" || fail "the box's settings hold no placeholder key"
  grep -q '"ANTHROPIC_BASE_URL": "http://127.0.0.1:8765"' "$lane/settings.json" || fail "the box's base URL is not the relay's"
fi
grep -qE '@[A-Z_]+@' "$lane/brief.txt" && fail "the probe brief keeps a placeholder"
grep -q '^PYTHON is "' "$lane/brief.txt" || fail "the probe brief doesn't name PYTHON"
[ "$(head -1 "$lane/argv.txt")" = "$CLAUDE_BIN" ] || fail "the CLI is not CLAUDE_BIN"
grep -q -- '--pathspec-from-file=.*tokens.tsv' "$lane/brief.txt" || fail "the probe brief's pathspec read names no file"
# The Bash rules for absolute paths after `=` and the options that read or write a named file.
"$python" -I -c 'import json, sys; print("\n".join(json.load(open(sys.argv[1], encoding="utf-8"))["permissions"]["deny"]))' \
  "$lane/settings.json" | tr -d '\r' >"$tmp/deny.txt" || fail "the deny list can't be read"
for rule in 'Bash(*=/*)' 'Bash(*=\\*)' 'Bash(* --pathspec-from-file*)' 'Bash(* --output*)' \
  'Bash(* --logfile*)' 'Bash(* --target-dir*)' 'Bash(* /*)'; do
  grep -qxF "$rule" "$tmp/deny.txt" || fail "no deny rule $rule"
done

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
  "$repo/AGENTS.md" /etc/passwd "$HOME/.ssh/id" /proc/self/environ <<'PY'
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
  "$TEMP/c4a/logs/b90.txt" "$repo/AGENTS.md" /etc/passwd "$HOME/.ssh/id" /proc/self/environ; do
  grep -qx "$p Read,Glob,Grep,Edit,Write" <<<"$covered" || fail "not denied every tool: $(grep "^$p " <<<"$covered")"
done
grep -qxF 'Bash(*:\\*)' "$tmp/deny.txt" || fail "no deny rule for a drive path off Windows"
[ -z "$sibling" ] || grep -qxF "Read(/$sibling/**)" "$tmp/deny.txt" || fail "the checkout's sibling $sibling is not denied by name"
grep -qx 'Bash(cleanroom-tests)' < <("$python" -I -c 'import json, sys; print("\n".join(json.load(open(sys.argv[1], encoding="utf-8"))["permissions"]["allow"]))' "$lane/settings.json") ||
  fail "the runner command is not allowed"
grep -qx "HOME=$lane/home" "$lane/env.txt" || fail "HOME is not the lane's home"
grep -qx "CLAUDE_CONFIG_DIR=$lane/home/.claude" "$lane/env.txt" || fail "the CLI's config is not the lane's"
grep -qx "TMPDIR=$lane/tmp" "$lane/env.txt" || fail "TMPDIR is not the lane's"
grep -q "^PATH=$lane/bin:" "$lane/env.txt" || fail "the lane's bin/ does not lead PATH"
grep -q '^USERPROFILE=' "$lane/env.txt" && fail "a Windows variable off Windows"
grep -q 'bin/cleanroom-tests' "$lane/vector_runner.sha256" || fail "the runner script is not in the digest"
[ "$(tail -1 "$lane/awk.path" | cut -c1)" = / ] || fail "awk.path is not absolute"

# The key from the environment: for the relay outside the box, never in the box's environment,
# its settings or the record.
lane2=$(ANTHROPIC_API_KEY=sk-test-launch-secret launch "with ANTHROPIC_API_KEY") || exit 1
grep -q 'sk-test-launch-secret' "$lane2/settings.json" "$lane2/env.txt" "$lane2/argv.txt" "$lane2/box.txt" &&
  fail "the API key is written down"
grep -q '^ANTHROPIC_API_KEY=' "$lane2/env.txt" && fail "the API key is in the box's environment"
(unset ANTHROPIC_API_KEY; launch "without a key" >/dev/null 2>&1) && fail "a launch without a key goes on"

# A name the rules can't hold stops the launch; an export outside the home denies the home whole.
mkdir "$HOME/Foo (2)"
CLEANROOM_AUTH="$tmp/auth" launch "with a folder named Foo (2)" >/dev/null 2>&1 && fail "a folder named Foo (2) beside the export's path goes into a rule"
rmdir "$HOME/Foo (2)"
mkdir -p "$tmp/outside" && cp -a "$ex" "$tmp/outside/launch-probe"
lane3=$(probe_export="$tmp/outside/launch-probe" CLEANROOM_AUTH="$tmp/auth" launch "outside the home") || exit 1
grep -qF "\"Read(/$HOME/**)\"" "$lane3/settings.json" || fail "the home is not denied whole for an export outside it"

# The test command as the session runs it: the lane's environment alone, from the export.
mapfile -t envv <"$lane/env.txt"
run=$(cd "$ex" && env -i "${envv[@]}" cleanroom-tests 2>&1) || fail "cleanroom-tests failed: $run"
grep -q 'vector_runner: all targets passed' <<<"$run" || fail "the runner did not pass: $run"
grep -q 'test result: ok. 1 passed' <<<"$run" || fail "the export's tests did not run: $run"
# Only the session's PATH finds the script, and an argument is refused.
(cd "$ex" && env -i "${envv[@]}" cleanroom-tests --help >/dev/null 2>&1) && fail "the runner takes an argument"

# The box: the lane record's docker command, with network none, a read-only root, no
# capabilities, and mounts of the export (written), the lane's home/ and tmp/ (written) and
# read-only toolchain, settings and runner; nothing else of the home.
mapfile -t box <"$lane/box.txt"
pairs=$(paste -d' ' <(sed '$d' "$lane/box.txt") <(sed 1d "$lane/box.txt"))
for flag in '--network none' '--cap-drop ALL' '--security-opt no-new-privileges'; do
  grep -qxF -- "$flag" <<<"$pairs" || fail "the box has no $flag"
done
grep -qxF -- --read-only "$lane/box.txt" || fail "the box's root is writable"
mounts=$(sed -n 's/^type=bind,src=\([^,]*\),dst=\([^,]*\)\(,readonly\)\{0,1\}$/\1 \3/p' "$lane/box.txt")
written=$(grep -v ',readonly$' <<<"$mounts" | sed 's/ $//' | sort)
[ "$written" = "$(printf '%s\n' "$ex" "$lane/api.sock" "$lane/home" "$lane/tmp" | sort)" ] ||
  fail "the box writes other than the export and the lane's home/, tmp/ and socket: $written"
while read -r src _; do
  case $src in "$ex" | "$lane"/* | /usr | /etc/* | "$CARGO_HOME"/bin | "$RUSTUP_HOME" | "$(readlink -f "$RUSTUP_HOME")" | "$(readlink -f "$CLAUDE_BIN")") ;;
    *) case $src in "$HOME" | "$HOME"/* | "$tmp" | "$tmp"/*) fail "the box mounts $src" ;; esac ;;
  esac
done <<<"$mounts"

# The box run: the fake CLI tries each way out the probe names, from inside, and the relay adds
# the key on the way to a fake API outside. Every read, write and connection outside must fail.
if ! docker info >/dev/null 2>&1; then
  fail "no docker: launch.sh confines a session on Linux in a docker box"
else
  echo "token-in-record" >"$CLEANROOM_PRIVATE/tokens.tsv"
  echo "forbidden-source" >"$TEMP/c4a/logs/b90.txt"
  port_file="$tmp/api.port"
  "$python" -I - "$port_file" >"$tmp/api.log" 2>&1 <<'PY' &
import http.server, sys
class H(http.server.BaseHTTPRequestHandler):
    def do_POST(self):
        self.rfile.read(int(self.headers.get("content-length") or 0))
        body = f"key={self.headers.get('x-api-key')} path={self.path}".encode()
        self.send_response(200); self.send_header("content-length", str(len(body))); self.end_headers()
        self.wfile.write(body)
s = http.server.HTTPServer(("127.0.0.1", 0), H)
open(sys.argv[1], "w").write(str(s.server_port))
s.serve_forever()
PY
  api_pid=$!
  for _ in $(seq 50); do [ -s "$port_file" ] && break; sleep 0.1; done
  printf '  "env": {"ANTHROPIC_BASE_URL": "http://127.0.0.1:%s/api", "ANTHROPIC_API_KEY": "sk-test-box-secret"},\n' \
    "$(cat "$port_file")" >"$tmp/box-auth"
  lane4=$(CLEANROOM_AUTH="$tmp/box-auth" launch "for the box run") || exit 1
  mapfile -t box <"$lane4/box.txt"
  # The fake CLI: each attempt prints OPEN or SHUT and its name.
  cat >"$CLAUDE_BIN" <<EOF
#!/bin/sh
try() { name=\$1; shift; if "\$@" >/dev/null 2>&1; then echo "OPEN \$name"; else echo "SHUT \$name"; fi; }
try home-file cat '$HOME/notes.txt'
try home-dotfile cat '$HOME/.ssh/id'
try canary-record cat '$CLEANROOM_PRIVATE/tokens.tsv'
try temp-source cat '$TEMP/c4a/logs/b90.txt'
try checkout cat '$repo/AGENTS.md'
try checkout-git git -C '$repo' show HEAD:AGENTS.md
try other-export cat '$HOME/cleanroom/other-domain/x'
try export-parent-lists sh -c "ls -A '$HOME/cleanroom' | grep -vqx launch-probe"
try search-from-root sh -c "grep -rIl forbidden-sourc[e] / --exclude-dir=proc --exclude-dir=sys --exclude-dir=dev --exclude-dir=usr --exclude-dir=target 2>/dev/null | grep -q ."
try lane-write sh -c "echo x >'$lane4/probe-written.txt'"
try settings-write sh -c "echo {} >'$lane4/settings.json'"
try runner-write sh -c "echo x >'$lane4/vector_runner.py'"
try network python3 -I -c "import socket; socket.create_connection(('1.1.1.1', 443), timeout=5)"
try key-in-box sh -c "grep -rqs sk-test-box-secret /proc/[0-9]*/environ '$lane4' '$ex'"
try export-write sh -c "echo x >'$ex/box-written' && rm '$ex/box-written'"
try relay sh -c "python3 -I -c \"import urllib.request; print(urllib.request.urlopen(urllib.request.Request('http://127.0.0.1:8765/api/v1/messages', data=b'{}', headers={'x-api-key': 'cleanroom-relay'})).read().decode())\" | grep -qx 'key=sk-test-box-secret path=/api/v1/messages'"
try relay-elsewhere python3 -I -c "import urllib.request; urllib.request.urlopen(urllib.request.Request('http://127.0.0.1:8765/other', data=b'{}'))"
try runner sh -c "cleanroom-tests | grep -q 'vector_runner: all targets passed'"
EOF
  : >"$lane4/api.sock"
  ANTHROPIC_API_KEY= "$python" -I "$here/box_relay.py" serve "$lane4/api.sock" - <<<"{ $(sed 's/,$//' "$tmp/box-auth") }" 2>"$tmp/relay.log" &
  relay_pid=$!
  for _ in $(seq 50); do [ -S "$lane4/api.sock" ] && break; sleep 0.1; done
  "$python" -I "$here/box_relay.py" image sm-cleanroom-root:1 || fail "docker can't make the box's image"
  result=$(timeout 300 "${box[@]}" "$CLAUDE_BIN" </dev/null 2>&1)
  kill "$relay_pid" "$api_pid" 2>/dev/null
  docker rm -f "cleanroom-$(basename "$lane4")" >/dev/null 2>&1
  for open in export-write relay runner; do
    grep -qx "OPEN $open" <<<"$result" || fail "the box refuses what the session needs, $open: $result"
  done
  for shut in home-file home-dotfile canary-record temp-source checkout checkout-git other-export \
    export-parent-lists search-from-root lane-write settings-write runner-write network key-in-box relay-elsewhere; do
    grep -qx "SHUT $shut" <<<"$result" || fail "the box lets $shut through: $result"
  done
  [ ! -e "$lane4/probe-written.txt" ] || fail "the box wrote into the lane"
fi

# The Linux branch under shellcheck where it is installed (CI's runner has it).
if command -v shellcheck >/dev/null; then
  shellcheck -S error "$here/launch.sh" "$here/export.sh" "$here/../lane/worktree.sh" >"$tmp/sc" 2>&1 ||
    fail "shellcheck: $(cat "$tmp/sc")"
fi
[ "$bad" -eq 0 ] && echo "LAUNCH CONFIG TEST OK"
exit "$bad"
