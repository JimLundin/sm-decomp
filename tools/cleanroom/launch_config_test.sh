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
. "$here/lib.sh"
# Outside the home, where host tools would open and fetch the repositories this makes.
tmp=$(mktemp -d "$(outside_home_temp)/launch-config-XXXXXX") || exit 1
tmp=$(cd "$tmp" && pwd -P) || exit 1
sibling=
trap 'rm -rf "$tmp" ${sibling:+"$sibling"}' EXIT
bad=0
fail() { echo "FAIL $*"; bad=1; }
python=${CLEANROOM_PY:-${SM_PY:-$py_default}}

# Off Windows the session's home and temporary folder are the test's, so the rules name them.
if [ "$windows" = 0 ]; then
  export CARGO_HOME=${CARGO_HOME:-$HOME/.cargo} RUSTUP_HOME=${RUSTUP_HOME:-$HOME/.rustup}
  export HOME="$tmp/home" TEMP="$tmp/scratch"
  # The exports' root is outside the home, as launch.sh demands, with another export in it.
  mkdir -p "$HOME" "$tmp/root/other-domain" "$TEMP/c4a/logs"
  echo private >"$HOME/notes.txt"
  mkdir -p "$HOME/.ssh" && echo key >"$HOME/.ssh/id"
  # A dotfile in a folder on the export's path, which only the walk's `.[!.]*` names.
  mkdir -p "$tmp/.dot" && echo key >"$tmp/.dot/id"
  # A sibling of the main checkout, which the rules name as sm-decomp*.
  main=$(cd "$(git -C "$repo" rev-parse --path-format=absolute --git-common-dir)/.." && pwd -P)
  candidate="$(dirname "$main")/sm-decomp-launch-test-$$"
  [ -e "$candidate" ] || { mkdir "$candidate" 2>/dev/null && sibling=$candidate; }
  root="$tmp/root"
  quoted_sibling="$root/"'a"b'
  mkdir "$quoted_sibling" || exit 1
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
  "$CLEANROOM_PRIVATE/tokens.tsv" "$tmp/root/other-domain/x" "$TEMP/c4a/logs/b90.txt" \
  "$repo/AGENTS.md" /etc/passwd "$HOME/.ssh/id" "$tmp/.dot/id" /proc/self/environ <<'PY'
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
for p in "$HOME/notes.txt" "$CLEANROOM_PRIVATE/tokens.tsv" "$tmp/root/other-domain/x" \
  "$TEMP/c4a/logs/b90.txt" "$repo/AGENTS.md" /etc/passwd "$HOME/.ssh/id" "$tmp/.dot/id" /proc/self/environ; do
  grep -qx "$p Read,Glob,Grep,Edit,Write" <<<"$covered" || fail "not denied every tool: $(grep "^$p " <<<"$covered")"
done
grep -qxF 'Bash(*:\\*)' "$tmp/deny.txt" || fail "no deny rule for a drive path off Windows"
grep -qxF "Read(/$quoted_sibling/**)" "$tmp/deny.txt" || fail "no deny rule for the sibling with a double quote"
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

# A name the rules can't hold stops the launch; the home is denied whole; an export in the home,
# or one whose .git holds what no session's git leaves, is not launched.
mkdir "$tmp/Foo (2)"
CLEANROOM_AUTH="$tmp/auth" launch "with a folder named Foo (2)" >/dev/null 2>&1 && fail "a folder named Foo (2) beside the export's path goes into a rule"
rmdir "$tmp/Foo (2)"
backslash_sibling="$root/"'a\b'
mkdir "$backslash_sibling" || exit 1
CLEANROOM_AUTH="$tmp/auth" launch "with a backslash in a sibling name" >/dev/null 2>&1 &&
  fail "a sibling with a backslash in its name goes into a rule"
rmdir "$backslash_sibling" || exit 1
grep -qF "\"Read(/$HOME/**)\"" "$lane/settings.json" || fail "the home is not denied whole"
mkdir -p "$HOME/cleanroom" && cp -a "$ex" "$HOME/cleanroom/launch-probe"
probe_export="$HOME/cleanroom/launch-probe" CLEANROOM_AUTH="$tmp/auth" launch "in the home" >/dev/null 2>&1 &&
  fail "an export in the home is launched"
rm -rf "$HOME/cleanroom"
echo "$tmp/elsewhere.git" >"$ex/.git/commondir"
CLEANROOM_AUTH="$tmp/auth" launch "with a commondir" >/dev/null 2>&1 && fail "an export with a .git/commondir is launched"
rm "$ex/.git/commondir"

# What export_git_odd reports, each on an export's copy: everything a host tool's git would read
# besides objects, refs/heads/main and the configuration, and nothing for a clean export.
odd() { (. "$here/lib.sh"; export_git_odd "$@"); }
[ -z "$(odd "$ex")" ] || fail "a clean export is odd: $(odd "$ex")"
plant() { # NAME COMMAND: COMMAND run in a fresh copy of the export, then the copy must be odd
  rm -rf "$tmp/odd" && cp -a "$ex" "$tmp/odd"
  (cd "$tmp/odd" && eval "$2") >/dev/null 2>&1
  [ -n "$(odd "$tmp/odd")" ] || fail "export_git_odd misses $1"
}
gx() { git -c user.name=t -c user.email=t@invalid "$@"; }
plant commondir 'echo /elsewhere >.git/commondir'
plant config.worktree 'echo "[core]" >.git/config.worktree'
plant gitdir 'echo /elsewhere >.git/gitdir'
plant worktrees 'mkdir -p .git/worktrees/x'
plant alternates 'mkdir -p .git/objects/info && echo /elsewhere >.git/objects/info/alternates'
plant replace-ref 'gx replace -f HEAD "$(gx commit-tree HEAD^{tree} -m other)"'
plant packed-replace 'gx replace -f HEAD "$(gx commit-tree HEAD^{tree} -m other)" && gx pack-refs --all'
plant branch 'gx branch other'
plant renamed-git 'cp -a .git .git-old'
plant nested-git 'mkdir -p crates/x && gx init -q crates/x'
plant git-file 'mv .git .git-real && echo "gitdir: $PWD/.git-real" >.git'
plant git-symlink 'mv .git .git-real && ln -s .git-real .git'
plant branches 'mkdir -p .git/branches && echo /elsewhere >.git/branches/origin'
plant symlink-head 'mkdir -p crates/r && ln -s ../../.git/HEAD crates/r/HEAD'
rm -rf "$tmp/odd"

# The test command as the session runs it: the lane's environment alone, from the export.
mapfile -t envv <"$lane/env.txt"
run=$(cd "$ex" && env -i "${envv[@]}" cleanroom-tests 2>&1) || fail "cleanroom-tests failed: $run"
grep -q 'vector_runner: all targets passed' <<<"$run" || fail "the runner did not pass: $run"
grep -q 'test result: ok. 1 passed' <<<"$run" || fail "the export's tests did not run: $run"
# Only the session's PATH finds the script, and an argument is refused.
(cd "$ex" && env -i "${envv[@]}" cleanroom-tests --help >/dev/null 2>&1) && fail "the runner takes an argument"

# The box: the lane record's docker command, with network none, a read-only root, no
# capabilities, limits, and exactly these mounts: the export, the lane's home/, tmp/ and socket
# written; the export's .git config, info/ and hooks/, /usr, the lane's settings and runner, the
# CLI, rustup's proxies by name and rustup's home read-only. Anything else (cargo's home, any
# other path of the home) fails the test.
mapfile -t box <"$lane/box.txt"
pairs=$(paste -d' ' <(sed '$d' "$lane/box.txt") <(sed 1d "$lane/box.txt"))
for flag in '--network none' '--cap-drop ALL' '--security-opt no-new-privileges' '--memory 12g' '--pids-limit 4096'; do
  grep -qxF -- "$flag" <<<"$pairs" || fail "the box has no $flag"
done
grep -qxF -- --read-only "$lane/box.txt" || fail "the box's root is writable"
grep -q -- '^/tmp:.*,size=' "$lane/box.txt" || fail "the box's /tmp has no size"
box_mounts() { sed -n 's/^type=bind,src=\([^,]*\),dst=[^,]*\(,readonly\)\{0,1\}$/\1\2/p' "$1/box.txt" | sort; }
expected_mounts() { # LANE EXPORT
  printf '%s\n' "$2" "$1/home" "$1/tmp" "$1/api.sock"
  printf '%s,readonly\n' "$2/.git/config" "$2/.git/info" "$2/.git/hooks" /usr /etc/alternatives \
    /etc/ld.so.cache "$1/passwd" "$1/group" "$1/settings.json" "$1/box_relay.py" "$1/bin" \
    "$1/vector_runner.py" "$1/strip_rust.awk" "$1/shape_scan.awk" "$1/awk.path" "$(readlink -f "$CLAUDE_BIN")" \
    "$(cd "$RUSTUP_HOME" && pwd -P)"
  cargo_dir=$(dirname "$(command -v cargo)")
  case $cargo_dir/ in /usr/*) ;; *)
    for t in cargo rustc rustup rustdoc cargo-clippy clippy-driver cargo-fmt rustfmt; do
      [ ! -e "$cargo_dir/$t" ] || printf '%s,readonly\n' "$cargo_dir/$t"
    done ;;
  esac
  py_prefix=$("$python" -I -c 'import sys; print(sys.base_prefix)')
  case $py_prefix/ in /usr/*) ;; *) printf '%s,readonly\n' "$py_prefix" ;; esac
}
[ "$(box_mounts "$lane")" = "$(expected_mounts "$lane" "$ex" | sort)" ] ||
  fail "the box's mounts are not the expected ones: $(diff <(expected_mounts "$lane" "$ex" | sort) <(box_mounts "$lane"))"

# Host-side git on an export runs nothing the export's .git names: a textconv driver planted in
# its config and attributes, hiding a token committed and then removed, neither runs nor hides it.
# (No fsmonitor: only commands that read the work tree run it, and git_isolated turns it off.)
token=crtestcanarytoken0123456789
printf '%s\t%s\t2026-01-01\n' "$token" "$TEMP/c4a/logs/b90.txt" >"$CLEANROOM_PRIVATE/tokens.tsv"
echo "$token" >"$TEMP/c4a/logs/b90.txt"
blind="$tmp/blind-export"
mkdir -p "$blind" && git -C "$blind" init -q && echo a >"$blind/f.rs"
gc() { git -C "$blind" -c user.name=t -c user.email=t@invalid -c commit.gpgsign=false "$@"; }
gc add . && gc commit -qm root && echo "$token" >>"$blind/f.rs" && gc commit -qam leak &&
  sed -i '$d' "$blind/f.rs" && gc commit -qam unleak
printf '#!/bin/sh\ntouch "%s"\ngrep -v crtest "$1"\n' "$tmp/textconv-ran" >"$tmp/conv.sh"
chmod +x "$tmp/conv.sh"
git -C "$blind" config diff.pwn.textconv "$tmp/conv.sh"
git -C "$blind" config log.showSignature true
git -C "$blind" config gpg.program "$tmp/conv.sh"
echo '* diff=pwn' >"$blind/.git/info/attributes"
cp "$tmp/conv.sh" "$blind/.git/hooks/post-checkout"
: >"$tmp/empty.jsonl"
check=$(bash "$here/canary.sh" check "$tmp/empty.jsonl" "$blind" 2>&1)
grep -q "^BREACH token .*(history)" <<<"$check" || fail "a textconv driver in the export's .git hides a token from canary.sh: $check"
[ ! -e "$tmp/textconv-ran" ] || fail "canary.sh ran a program the export's .git names"
# A replace ref that swaps the leaking commit for a clean one doesn't hide it either.
gc replace "$(gc rev-parse HEAD~1)" "$(gc commit-tree "$(gc rev-parse 'HEAD~2^{tree}')" -p HEAD~2 -m leak)"
check=$(bash "$here/canary.sh" check "$tmp/empty.jsonl" "$blind" 2>&1)
grep -q "^BREACH token .*(history)" <<<"$check" || fail "a replace ref hides a token from canary.sh: $check"
# A .git that is a file, or holds a symlink, stops the check.
ln -s /etc/passwd "$blind/.git/refs/heads/odd"
bash "$here/canary.sh" check "$tmp/empty.jsonl" "$blind" >/dev/null 2>&1 && fail "canary.sh reads a .git holding a symlink"
rm "$blind/.git/refs/heads/odd"

# The box run, through launch.sh itself: its relay, its box, its canary check and its cleanup.
# The fake CLI tries each way out from inside, then commits the token and removes it and tries
# to plant a textconv driver for the check; the check must still find the token, and nothing it
# planted may run on the host.
if ! docker info >/dev/null 2>&1; then
  fail "no docker: launch.sh confines a session on Linux in a docker box"
else
  # The root image is trusted only when it is exactly the empty root: a tag with the same layer
  # and an environment of its own (LD_PRELOAD, say) is made again.
  evil=sm-cleanroom-root:test-$$
  "$python" -I -c 'import sys; sys.path.insert(0, sys.argv[1]); import box_relay; sys.stdout.buffer.write(box_relay.root_tar())' "$here" |
    docker import --change 'ENV LD_PRELOAD=/x' - "$evil" >/dev/null || fail "docker can't import a test image"
  "$python" -I "$here/box_relay.py" image "$evil" || fail "box_relay.py image failed"
  [ "$(docker image inspect --format '{{json .Config.Env}}' "$evil")" = null ] ||
    fail "box_relay.py image trusts a root with its own environment"
  docker rmi "$evil" >/dev/null 2>&1
  port_file="$tmp/api.port"
  "$python" -I - "$port_file" "$tmp/api.seen" >"$tmp/api.log" 2>&1 <<'PY' &
import http.server, sys
class H(http.server.BaseHTTPRequestHandler):
    def do_POST(self):
        self.rfile.read(int(self.headers.get("content-length") or 0))
        with open(sys.argv[2], "a") as f:
            f.write(f"{self.requestline} host={self.headers.get('host')}\n")
        body = f"key={self.headers.get('x-api-key')} path={self.path}".encode()
        self.send_response(200); self.send_header("content-length", str(len(body))); self.end_headers()
        self.wfile.write(body)
    do_GET = do_POST
s = http.server.HTTPServer(("127.0.0.1", 0), H)
open(sys.argv[1], "w").write(str(s.server_port))
s.serve_forever()
PY
  api_pid=$!
  for _ in $(seq 50); do [ -s "$port_file" ] && break; sleep 0.1; done
  printf '  "env": {"ANTHROPIC_BASE_URL": "http://127.0.0.1:%s/api", "ANTHROPIC_API_KEY": "sk-test-box-secret"},\n' \
    "$(cat "$port_file")" >"$tmp/box-auth"
  cat >"$CLAUDE_BIN" <<EOF
#!/bin/sh
# Each line is JSON, as the CLI's stream is.
try() { name=\$1; shift; if "\$@" >/dev/null 2>&1; then r=OPEN; else r=SHUT; fi; echo "{\"type\":\"probe\",\"result\":\"\$r \$name\"}"; }
# A raw request line to the relay; it passes when the relay answers 200.
cat >"\$TMPDIR/raw.py" <<'PYEOF'
import socket, sys
s = socket.create_connection(("127.0.0.1", 8765))
s.sendall(sys.argv[1].encode() + b" HTTP/1.1\r\nHost: evil.example\r\nContent-Length: 0\r\n\r\n")
sys.exit(0 if s.recv(64).split()[1] == b"200" else 1)
PYEOF
try home-file cat '$HOME/notes.txt'
try home-dotfile cat '$HOME/.ssh/id'
try canary-record cat '$CLEANROOM_PRIVATE/tokens.tsv'
try temp-source cat '$TEMP/c4a/logs/b90.txt'
try checkout cat '$repo/AGENTS.md'
try checkout-git git -C '$repo' show HEAD:AGENTS.md
try other-export cat '$tmp/root/other-domain/x'
try export-parent-lists sh -c "ls -A '$tmp/root' | grep -vqx launch-probe"
try search-from-root sh -c "grep -rIl crtestcanarytoke[n] / --exclude-dir=proc --exclude-dir=sys --exclude-dir=dev --exclude-dir=usr --exclude-dir=target 2>/dev/null | grep -vx '$(readlink -f "$CLAUDE_BIN")' | grep -q ."
try cargo-home sh -c "ls -A '$CARGO_HOME' | grep -vqx bin"
try cargo-installs sh -c "ls -A '$CARGO_HOME/bin' | grep -vqxE 'cargo|rustc|rustup|rustdoc|cargo-clippy|clippy-driver|cargo-fmt|rustfmt'"
try docker-socket sh -c "test -e /var/run/docker.sock || test -e /run/docker.sock"
try root-write sh -c "touch /etc/box-written || touch /box-written"
try lane-write sh -c "echo x >\"\\\$(dirname \"\\\$HOME\")/probe-written.txt\""
try settings-write sh -c "echo {} >\"\\\$(dirname \"\\\$HOME\")/settings.json\""
try runner-write sh -c "echo x >\"\\\$(dirname \"\\\$HOME\")/vector_runner.py\""
try network python3 -I -c "import socket; socket.create_connection(('1.1.1.1', 443), timeout=5)"
try key-in-box sh -c "grep -rqs sk-test-box-secre[t] /proc/[0-9]*/environ /proc/[0-9]*/cmdline \"\\\$(dirname \"\\\$HOME\")/settings.json\" '$ex'"
try git-config-write sh -c "echo '[diff \"pwn\"]' >>'$ex/.git/config'"
try git-attributes-write sh -c "echo '* diff=pwn' >'$ex/.git/info/attributes'"
try git-hook-write sh -c "echo 'exit 0' >'$ex/.git/hooks/post-commit'"
try export-write sh -c "echo x >'$ex/box-written' && rm '$ex/box-written'"
try relay sh -c "python3 -I -c \"import urllib.request; print(urllib.request.urlopen(urllib.request.Request('http://127.0.0.1:8765/api/v1/messages', data=b'{}', headers={'x-api-key': 'cleanroom-relay'})).read().decode())\" | grep -qx 'key=sk-test-box-secret path=/api/v1/messages'"
try relay-elsewhere python3 -I -c "import urllib.request; urllib.request.urlopen(urllib.request.Request('http://127.0.0.1:8765/other', data=b'{}'))"
try relay-raw-absolute python3 -I "\$TMPDIR/raw.py" "POST http://evil.example/api/v1/messages"
try relay-raw-semicolon python3 -I "\$TMPDIR/raw.py" "POST /api/v1/..;/..;/admin"
try relay-raw-backslash python3 -I "\$TMPDIR/raw.py" 'POST /api/v1/..\\..\\admin'
try relay-raw-encoded python3 -I "\$TMPDIR/raw.py" "POST /api/v1/%2e%2e/admin"
try relay-raw-allowed python3 -I "\$TMPDIR/raw.py" "POST /api/v1/messages"
try runner sh -c "cleanroom-tests | grep -q 'vector_runner: all targets passed'"
# The leak and the cover-ups a host tool's git would read: a token committed and removed, a
# replace ref, a textconv driver through a commondir in the lane's tmp/, a nested repository and
# a renamed .git with a fresh one in its place.
try commit-token sh -c "echo $token >leak.txt && git add leak.txt && git commit -qm leak && git rm -q leak.txt && git commit -qm unleak"
try replace-ref sh -c "git replace HEAD~1 \"\\\$(git commit-tree 'HEAD~2^{tree}' -p HEAD~2 -m leak)\""
printf '#!/bin/sh\ntouch "%s"\ngrep -v crtest "\$1"\n' '$tmp/textconv-ran-box' >conv.sh; chmod +x conv.sh
try commondir sh -c "git init -q --bare \"\\\$TMPDIR/evil.git\" && git -C \"\\\$TMPDIR/evil.git\" config diff.pwn.textconv '$ex/conv.sh' && echo '* diff=pwn' >\"\\\$TMPDIR/evil.git/info/attributes\" && echo \"\\\$TMPDIR/evil.git\" >.git/commondir"
try nested-repo git init -q crates/nested
try rename-git sh -c "mv .git .git-old && git init -q"
echo '{"type":"result","subtype":"success"}'
EOF
  out=$(cd "$repo" && CLEANROOM_AUTH="$tmp/box-auth" CLEANROOM_DRY_RUN=0 CLEANROOM_COMPLETION_SECONDS=20 \
    timeout 600 bash "$here/launch.sh" --probe "$ex" "$tmp/transcripts/box.jsonl" 2>&1)
  lane4=$(sed -n 's/^launch: launch-probe in .*; lane record //p' <<<"$out")
  result=$(sed -n 's/^{"type":"probe","result":"\(.*\)"}$/\1/p' "$tmp/transcripts/box.jsonl" 2>/dev/null)
  kill "$api_pid" 2>/dev/null
  for open in export-write relay relay-raw-allowed runner commit-token replace-ref commondir nested-repo rename-git; do
    grep -qx "OPEN $open" <<<"$result" || fail "the box refuses what the session needs, $open: $result"
  done
  for shut in home-file home-dotfile canary-record temp-source checkout checkout-git other-export \
    export-parent-lists search-from-root cargo-home cargo-installs docker-socket root-write lane-write \
    settings-write runner-write network key-in-box git-config-write git-attributes-write git-hook-write \
    relay-elsewhere relay-raw-absolute relay-raw-semicolon relay-raw-backslash relay-raw-encoded; do
    grep -qx "SHUT $shut" <<<"$result" || fail "the box lets $shut through: $result"
  done
  # The export's .git and the lane: each thing a host tool's git would read is named a breach.
  grep -q "^LAUNCH BREACH launch-probe: the box left what no session's git leaves" <<<"$out" ||
    fail "launch.sh didn't report the box's git leftovers: $out"
  for left in "$ex/.git-old" "$ex/crates/nested/.git" "$lane4/tmp/evil.git"; do
    grep -qxF "  $left" <<<"$out" || fail "launch.sh didn't name $left: $out"
  done
  [ ! -e "$tmp/textconv-ran-box" ] || fail "a program the box planted ran on the host"
  grep -v " host=127.0.0.1:$(cat "$port_file")\$" "$tmp/api.seen" | grep -q . &&
    fail "the fake API saw another Host: $(cat "$tmp/api.seen")"
  grep -vq '^POST /api/v1/messages HTTP' "$tmp/api.seen" && fail "the fake API saw another target: $(cat "$tmp/api.seen")"
  # launch.sh's cleanup: no relay or box of this lane left.
  [ -n "$lane4" ] || fail "no lane for the box run: $out"
  pgrep -f "box_relay.py serve $lane4/api.sock" >/dev/null && fail "the relay outlived launch.sh"
  [ -z "$(docker ps -q --filter "label=sm-cleanroom-lane=$lane4")" ] || fail "the box outlived launch.sh"
fi

# The Linux branch under shellcheck where it is installed (CI's runner has it).
if command -v shellcheck >/dev/null; then
  shellcheck -S error "$here/launch.sh" "$here/export.sh" "$here/../lane/worktree.sh" >"$tmp/sc" 2>&1 ||
    fail "shellcheck: $(cat "$tmp/sc")"
fi
[ "$bad" -eq 0 ] && echo "LAUNCH CONFIG TEST OK"
exit "$bad"
