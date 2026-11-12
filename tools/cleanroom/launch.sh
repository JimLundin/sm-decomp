#!/usr/bin/env bash
# launch.sh EXPORT BRIEF TRANSCRIPT: run a domain's implementer as a headless Claude Code session
# confined to EXPORT (MODEL §5, "The clean room is enforced by construction"). Canaries only show
# a read afterwards; this stops it happening.
#
# How the root runs a clean-room lane, end to end (from a checkout of this repository):
#   1. bash tools/cleanroom/canary.sh plant              fresh tokens and fingerprints
#   2. bash tools/cleanroom/export.sh DOMAIN C:/cleanroom/DOMAIN
#   3. bash tools/cleanroom/launch.sh C:/cleanroom/DOMAIN BRIEF TRANSCRIPT
#        BRIEF is a text file written for the implementer; TRANSCRIPT lies outside the export.
#        Exits 0 only when the session ended and its transcript and the export are canary-clean.
#   4. bash tools/cleanroom/canary.sh check TRANSCRIPT C:/cleanroom/DOMAIN
#        launch.sh already ran it; run it again on anything else the lane produced.
#   5. bash tools/cleanroom/import.sh C:/cleanroom/DOMAIN BRANCH
#
# What launch.sh does:
#   - refuses to start unless scan_brief.sh passes on BRIEF, EXPORT is an export (one root commit,
#     no remote) and TRANSCRIPT is outside EXPORT;
#   - makes a lane directory, $CLEANROOM_PRIVATE/lanes/<timestamp>/ (default ~/.cleanroom), outside
#     every repository, holding: brief.txt (the brief, ending with the test command),
#     settings.json (the permission configuration), vector_runner.py and its vector_runner.sha256
#     (the test runner), argv.txt and env.txt (the exact launch), home/ (the session's empty home
#     and config directory) and tmp/ (its TEMP);
#   - runs the CLI with an emptied environment, HOME, USERPROFILE and CLAUDE_CONFIG_DIR pointing at
#     home/, so no user settings, memory, CLAUDE.md, plugins, skills, hooks, MCP servers or session
#     history exist for it to load; and with --bare, --restricted, --setting-sources '' and
#     --strict-mcp-config besides;
#   - confines it with settings.json, which lives in the lane directory and so out of its reach:
#     mode dontAsk (anything not allowed is refused, nothing prompts), the tool set Read, Glob,
#     Grep, Edit, Write and Bash only (no web, agent, notebook, skill or MCP tool exists), Read
#     under EXPORT, Edit and Write under the [import] paths of allowlist.txt, Bash for
#     `cargo check|clippy|fmt|test|build`, `git add|commit|status|diff|log|rm|mv` and the one Python
#     command `"<python>" -I "<lane>\vector_runner.py"` (Defender blocks the executables
#     `cargo test` links; the runner links the clean crates' tests as DLLs and takes no
#     arguments), and explicit deny rules for everything the adversary named;
#   - writes the stream-json transcript to TRANSCRIPT and a copy into the lane directory, then
#     exits 1 if settings.json or vector_runner.py changed during the session, or if canary.sh
#     check finds a token or fingerprint in the transcript or the export's history.
#
# A commit message passes the deny rules when each paragraph is its own `-m` and holds none of
# ; & | < > $ ~ `..` ` /` `:/` (`git commit -m "Subject" -m "Body." -m "Role: implementer"`).
# The samus-movement session's commits were refused for a `;` in the body and for `-F -` with a
# heredoc; CLEANROOM.md tells the implementer the form.
#
# Before the first real lane on a new CLI version, prove the confinement again:
#   CLEANROOM_ALLOW_MISSING=1 bash tools/cleanroom/export.sh launch-probe C:/cleanroom/launch-probe
#   bash tools/cleanroom/launch.sh --probe C:/cleanroom/launch-probe TRANSCRIPT
# --probe sends launch_probe.txt, which tries each breach the adversary named and then commits one
# file; scan_brief.sh refuses that brief by design, so --probe runs on the dummy domain only. Read
# the denials in TRANSCRIPT ("has been denied", "denied by your permission settings").
#
# Limits the CLI cannot confine: cargo runs code the implementer writes (tests, build scripts,
# macros) as an ordinary process, which can read any file this user can. Build scripts and cargo
# configuration are not writable here, but a test body is. The canary check after the session is
# what catches a test that read a forbidden file and printed it, and code_check.sh, which import.sh
# runs before applying anything, refuses the source that could (std::fs, net, process, env,
# unsafe, include_*! outside the vector file, build scripts, dependencies beyond allowed_deps.txt).
# The clean crates' Cargo.toml is not writable here: their dependencies are the integrator's, from
# allowed_deps.txt (export.sh gives a new model crate the vector parser), so a manifest edit is
# never needed. CLI 2.1.316 offers only Read, Edit and Bash (Edit creates files; Glob, Grep and
# Write don't exist); their rules stay for later CLIs.
#
# Overrides: CLAUDE_BIN (default ~/.local/bin/claude.exe), CLEANROOM_MODEL (default opus),
# CLEANROOM_MAX_BUDGET_USD (passed as --max-budget-usd), CLEANROOM_AUTH (a file of top-level
# settings JSON members, `"apiKeyHelper": ..., "env": {...}`; default: the apiKeyHelper and the
# ANTHROPIC_* lines of ~/.claude/settings.json), CLEANROOM_PY (the runner's interpreter, default
# %LOCALAPPDATA%/sm-decomp-tools/python-3.13.7/python.exe), CLEANROOM_PRIVATE, CARGO_HOME,
# RUSTUP_HOME, CLEANROOM_SETTLE_SECONDS (the settle period after the session's result, default 60).
set -euo pipefail
here=$(cd "$(dirname "$0")" && pwd)
. "$here/lib.sh"

probe=0
if [ "${1:-}" = --probe ]; then
  # The confinement test: the committed probe brief, which tries every breach on purpose and so
  # fails scan_brief.sh by design, on an export of the dummy domain `launch-probe` only.
  [ $# -eq 3 ] || die "usage: launch.sh --probe EXPORT TRANSCRIPT"
  probe=1
  set -- "$2" "$here/launch_probe.txt" "$3"
fi
[ $# -eq 3 ] || die "usage: launch.sh EXPORT BRIEF TRANSCRIPT  |  launch.sh --probe EXPORT TRANSCRIPT"
[ -d "$1" ] || die "no export at $1"
[ -f "$2" ] || die "no brief at $2"
export_dir=$(cd "$1" && pwd -P)
brief=$(cd "$(dirname "$2")" && pwd -P)/$(basename "$2")
mkdir -p "$(dirname "$3")"
transcript=$(cd "$(dirname "$3")" && pwd -P)/$(basename "$3")
case "$transcript/" in "$export_dir"/*) die "the transcript $transcript is inside the export" ;; esac
claude_bin=${CLAUDE_BIN:-$USERPROFILE/.local/bin/claude.exe}
[ -x "$claude_bin" ] || die "no Claude Code CLI at $claude_bin"

# 1. The brief is scanned before anything starts.
if [ "$probe" = 1 ]; then
  ! bash "$here/scan_brief.sh" "$brief" >/dev/null 2>&1 || die "the probe brief passes scan_brief.sh; it must try breaches"
  echo "launch: probe brief; scan_brief.sh refuses it, as it should"
else
  bash "$here/scan_brief.sh" "$brief" || die "scan_brief.sh failed on $brief; not launching"
fi

# The export is one: its own repository, one root commit, no remote. Its domain is its one model
# crate.
root=$(git_isolated "$export_dir" rev-list --max-parents=0 HEAD) || die "$export_dir is not a repository"
[ "$(echo "$root" | wc -l)" -eq 1 ] || die "$export_dir has more than one root commit"
[ -z "$(git_isolated "$export_dir" remote)" ] || die "$export_dir has a remote"
mapfile -t models < <(git_isolated "$export_dir" ls-tree -r --name-only "$root" crates/clean/ | model_dirs)
[ ${#models[@]} -eq 1 ] || die "the export holds ${#models[@]} model crates, not one"
domain=$(domain_of "${models[0]#crates/clean/}")
check_domain "$domain"
[ "$probe" = 0 ] || [ "$domain" = launch-probe ] || die "--probe runs only on an export of the domain launch-probe"

# 2. The lane directory, outside every repository.
private=${CLEANROOM_PRIVATE:-$HOME/.cleanroom}
lane=$private/lanes/$(date -u +%Y%m%dT%H%M%SZ)-$domain
mkdir -p "$lane/home/.claude" "$lane/tmp"
lane=$(cd "$lane" && pwd -P)
if git -C "$lane" rev-parse --git-dir >/dev/null 2>&1; then die "$lane is inside a repository"; fi
cp "$brief" "$lane/brief.txt"

# The test runner: Defender blocks the test executables `cargo test` links, so the session runs
# the clean crates' tests through one fixed Python script, copied here out of its reach. The one
# command it may run is that interpreter on that copy, both by absolute Windows path in double
# quotes (a forward-slash path would meet the `:/` and ` /` deny rules below, which win).
python=${CLEANROOM_PY:-${LOCALAPPDATA:-$USERPROFILE/AppData/Local}/sm-decomp-tools/python-3.13.7/python.exe}
[ -f "$python" ] || die "no Python at $python; set CLEANROOM_PY"
cp "$here/vector_runner.py" "$lane/vector_runner.py"
sha256sum <"$lane/vector_runner.py" >"$lane/vector_runner.sha256"
runner_sum=$(cat "$lane/vector_runner.sha256")
run_tests="\"$(cygpath -w "$python")\" -I \"$(cygpath -w "$lane/vector_runner.py")\""
# The brief ends with the command, as the export cannot know the lane directory; CLEANROOM.md
# says the brief gives it.
printf '\nRun the tests with exactly this Bash command, from the working directory:\n\n    %s\n' \
  "$run_tests" >>"$lane/brief.txt"

# Paths as the CLI matches them on Windows: `//c/dir/**` is the absolute path C:/dir.
rule_path() { # DIR -> //c/dir
  p=$(cygpath -u "$1" 2>/dev/null || echo "$1")
  echo "/$p"
}
ex=$(rule_path "$export_dir")
home_rule=$(rule_path "$(cygpath -u "$USERPROFILE" 2>/dev/null || echo "$HOME")")
lane_rule=$(rule_path "$lane")
mapfile -t writable < <(allowlist_section import "$domain")

auth() {
  if [ -n "${CLEANROOM_AUTH:-}" ]; then cat "$CLEANROOM_AUTH"; return; fi
  user=${USERPROFILE:-$HOME}/.claude/settings.json
  [ -f "$user" ] || die "no CLEANROOM_AUTH and no $user to take the API key helper from"
  # The helper runs with the session's short PATH, so a bare powershell.exe gets its full path.
  helper=$(grep -E '^\s*"apiKeyHelper"' "$user" | sed 's/,\s*$//; s#"powershell\.exe #"C:/Windows/System32/WindowsPowerShell/v1.0/powershell.exe #') || true
  [ -n "$helper" ] && printf '  %s,\n' "$helper"
  printf '  "env": {\n'
  { grep -E '^\s*"(ANTHROPIC_[A-Z_]+|CLAUDE_CODE_DISABLE_[A-Z_]+|CLAUDE_CODE_GIT_BASH_PATH)"' "$user" || true; } \
    | sed 's/,\s*$//' | sed '$!s/$/,/'
  printf '  },\n'
}

q() { # a JSON string: backslashes and quotes escaped
  local bs='\' dq='"' s
  s=${1//"$bs"/"$bs$bs"}
  printf '"%s"' "${s//"$dq"/"$bs$dq"}"
}
rules() { # each argument a rule; prints a JSON array body
  first=1
  for r in "$@"; do
    [ $first = 1 ] || printf ',\n'
    printf '      %s' "$(q "$r")"; first=0
  done
  printf '\n'
}

allow=( "Read($ex/**)" "Glob($ex/**)" "Grep($ex/**)" )
for w in "${writable[@]}"; do allow+=("Edit($ex/$w**)" "Write($ex/$w**)"); done
for c in check clippy fmt test build; do allow+=("Bash(cargo $c)" "Bash(cargo $c *)"); done
for g in add commit status diff log rm mv; do allow+=("Bash(git $g)" "Bash(git $g *)"); done
# The CLI reads a backslash in a rule as an escape, so the rule doubles each one.
bs='\'
allow+=("Bash(${run_tests//"$bs"/"$bs$bs"})")

deny=(
  # Tools that reach outside: the web, agents, notebooks, MCP, skills, background work.
  WebFetch WebSearch Agent Task NotebookEdit Skill "mcp__*" Monitor SendMessage TaskStop
  EnterWorktree ExitWorktree PowerShell
  # Reads outside the export, named for the places the original lives.
  "Read(//c/Users/**)" "Glob(//c/Users/**)" "Grep(//c/Users/**)"
  "Read($home_rule/**)" "Glob($home_rule/**)" "Grep($home_rule/**)"
  "Read(//c/Windows/**)" "Read(//c/ProgramData/**)" "Read(//c/scratch/**)"
  "Read(//d/**)" "Read(//e/**)" "Read(//f/**)"
  # The configuration and the lane record, though both are outside the export already.
  "Edit($lane_rule/**)" "Write($lane_rule/**)" "Edit($home_rule/**)" "Write($home_rule/**)"
  # Inside the export: git internals, cargo configuration, build scripts, toolchain choice.
  "Edit($ex/.git/**)" "Write($ex/.git/**)" "Edit($ex/.claude/**)" "Write($ex/.claude/**)"
  "Edit($ex/**/.cargo/**)" "Write($ex/**/.cargo/**)" "Edit($ex/.cargo/**)" "Write($ex/.cargo/**)"
  "Edit($ex/**/build.rs)" "Write($ex/**/build.rs)"
  "Edit($ex/crates/**/Cargo.toml)" "Write($ex/crates/**/Cargo.toml)"
  "Edit($ex/rust-toolchain.toml)" "Write($ex/rust-toolchain.toml)"
  # Shell: readers, movers, interpreters, network, and the git forms that escape the export.
  "Bash(cat *)" "Bash(head *)" "Bash(tail *)" "Bash(less *)" "Bash(more *)" "Bash(ls)" "Bash(ls *)"
  "Bash(dir *)" "Bash(find *)" "Bash(grep *)" "Bash(rg *)" "Bash(sed *)" "Bash(awk *)"
  "Bash(cd *)" "Bash(cd)" "Bash(pushd *)" "Bash(type *)" "Bash(od *)" "Bash(xxd *)"
  "Bash(strings *)" "Bash(file *)" "Bash(stat *)" "Bash(cp *)" "Bash(mv *)" "Bash(rm *)"
  "Bash(ln *)" "Bash(tar *)" "Bash(echo *)" "Bash(printf *)" "Bash(tee *)" "Bash(xargs *)"
  "Bash(curl *)" "Bash(wget *)" "Bash(nc *)" "Bash(ssh *)" "Bash(scp *)" "Bash(gh *)"
  "Bash(powershell *)" "Bash(powershell.exe *)" "Bash(pwsh *)" "Bash(cmd *)" "Bash(cmd.exe *)"
  "Bash(python3 *)" "Bash(py *)" "Bash(node *)" "Bash(perl *)" "Bash(ruby *)"
  # The CLI matches a command's program by its name, so `python *` would also deny the runner's
  # full-path interpreter; every Python command but the runner's is refused for not being
  # allowed, and these name the forms an implementer would try.
  "Bash(python)" "Bash(python3)" "Bash(py)" "Bash(python -c*)" "Bash(python -m*)"
  "Bash(python -E*)" "Bash(python -X*)" "Bash(python -)" "Bash(python -I -*)"
  "Bash(python.exe *)" "Bash(python3.exe *)" "Bash(py.exe *)" "Bash(*python* -c*)"
  "Bash(*python* -m *)" "Bash(*python* -E*)"
  "Bash(bash *)" "Bash(sh *)" "Bash(env *)" "Bash(export *)" "Bash(source *)" "Bash(eval *)"
  "Bash(exec *)" "Bash(start *)" "Bash(claude *)" "Bash(rustc *)" "Bash(rustup *)"
  "Bash(git -C *)" "Bash(git -c *)" "Bash(git --git-dir*)" "Bash(git --work-tree*)"
  "Bash(git show *)" "Bash(git show)" "Bash(git cat-file *)" "Bash(git config *)"
  "Bash(git remote *)" "Bash(git fetch *)" "Bash(git clone *)" "Bash(git worktree *)"
  "Bash(git archive *)" "Bash(git grep *)" "Bash(git am *)" "Bash(git apply *)"
  "Bash(cargo run *)" "Bash(cargo install *)" "Bash(cargo +*)"
  "Bash(* --manifest-path*)" "Bash(* --config*)" "Bash(* -Z*)" "Bash(* --no-index*)"
  "Bash(* --git-dir*)" "Bash(* --work-tree*)" "Bash(* -F *)" "Bash(* --file*)" "Bash(* --template*)"
  # Arguments that name a place outside: parents, absolute paths, the home, variables.
  "Bash(*../*)" "Bash(*..)" "Bash(* /*)" "Bash(*:/*)" "Bash(*~*)" "Bash(*\$*)" "Bash(*<*)"
  "Bash(*>*)" "Bash(*|*)" "Bash(*;*)" "Bash(*&*)"
)

settings=$lane/settings.json
{
  printf '{\n'
  auth
  printf '  "model": %s,\n' "$(q "${CLEANROOM_MODEL:-opus}")"
  printf '  "disableAllHooks": true,\n  "autoMemoryEnabled": false,\n  "includeCoAuthoredBy": false,\n'
  printf '  "enableAllProjectMcpServers": false,\n  "cleanupPeriodDays": 99999,\n'
  # The built-in plugins --bare leaves on: one loads AGENTS.md files, the other authors plugins.
  printf '  "enabledPlugins": { "cc-plugin-agents-md@builtin": false, "cc-plugin-plugin-authoring@builtin": false },\n'
  printf '  "permissions": {\n    "defaultMode": "dontAsk",\n    "disableBypassPermissionsMode": "disable",\n'
  printf '    "additionalDirectories": [],\n'
  printf '    "allow": [\n'; rules "${allow[@]}"; printf '    ],\n'
  printf '    "deny": [\n'; rules "${deny[@]}"; printf '    ]\n'
  printf '  }\n}\n'
} >"$settings"
settings_sum=$(sha256sum <"$settings")
# The probe brief names the real settings file, so its edit is tried on the real path.
[ "$probe" = 0 ] || sed -i "s#@SETTINGS@#$(cygpath -m "$settings")#" "$lane/brief.txt"

argv=(
  "$claude_bin" -p --verbose --output-format stream-json
  --bare --restricted --setting-sources "" --settings "$settings"
  --strict-mcp-config --mcp-config '{"mcpServers":{}}' --disable-slash-commands
  --tools "Read,Glob,Grep,Edit,Write,Bash"
  --disallowedTools "WebFetch,WebSearch,Agent,Task,NotebookEdit,Skill,mcp__*"
  --permission-mode dontAsk --permission-prompts none
  --model "${CLEANROOM_MODEL:-opus}" --no-session-persistence
)
[ -n "${CLEANROOM_MAX_BUDGET_USD:-}" ] && argv+=(--max-budget-usd "$CLEANROOM_MAX_BUDGET_USD")

# An emptied environment: nothing of the parent session (its messaging socket, its session ids)
# and a home with nothing in it. Cargo and rustup keep their real homes so the toolchain resolves.
win_home=$(cygpath -w "$lane/home")
cargo_home=${CARGO_HOME:-$(cygpath -w "${USERPROFILE:-$HOME}/.cargo")}
rustup_home=${RUSTUP_HOME:-$(cygpath -w "${USERPROFILE:-$HOME}/.rustup")}
envv=(
  "PATH=$(cygpath -u "$cargo_home")/bin:/mingw64/bin:/usr/bin:/c/Windows/system32:/c/Windows"
  "HOME=$lane/home" "USERPROFILE=$win_home" "CLAUDE_CONFIG_DIR=$win_home\\.claude"
  "APPDATA=$win_home\\AppData\\Roaming" "LOCALAPPDATA=$win_home\\AppData\\Local"
  "TEMP=$(cygpath -w "$lane/tmp")" "TMP=$(cygpath -w "$lane/tmp")"
  "SYSTEMROOT=${SYSTEMROOT:-C:\\Windows}" "WINDIR=${WINDIR:-C:\\Windows}"
  "COMSPEC=${COMSPEC:-C:\\Windows\\system32\\cmd.exe}" "PATHEXT=${PATHEXT:-.COM;.EXE;.BAT;.CMD}"
  "CARGO_HOME=$cargo_home" "RUSTUP_HOME=$rustup_home" "RUSTUP_TOOLCHAIN_SOURCE="
  "GIT_CONFIG_NOSYSTEM=1" "GIT_CONFIG_GLOBAL=$(cygpath -w "$lane/home")\\.gitconfig"
  "GIT_AUTHOR_NAME=implementer" "GIT_AUTHOR_EMAIL=implementer@cleanroom.invalid"
  "GIT_COMMITTER_NAME=implementer" "GIT_COMMITTER_EMAIL=implementer@cleanroom.invalid"
  "DISABLE_AUTOUPDATER=1" "MSYS=winsymlinks:native"
)
[ -n "${FOUNDRY_LMS_TOKEN:-}" ] && envv+=("FOUNDRY_LMS_TOKEN=$FOUNDRY_LMS_TOKEN")
: >"$lane/home/.gitconfig"
printf '%s\n' "${argv[@]}" >"$lane/argv.txt"
printf '%s\n' "${envv[@]}" | sed 's/^\(FOUNDRY_LMS_TOKEN\)=.*/\1=<redacted>/' >"$lane/env.txt"
printf 'export\t%s\nroot\t%s\nbrief\t%s\ntranscript\t%s\n' "$export_dir" "$root" "$brief" "$transcript" >"$lane/lane.tsv"
echo "launch: $domain in $export_dir; lane record $lane"

# 3. The session, its prompt the brief, its working directory the export.
status=0
(cd "$export_dir" && env -i "${envv[@]}" "${argv[@]}" <"$lane/brief.txt") >"$transcript" 2>"$lane/stderr.txt" || status=$?
# The CLI's process can return while the session goes on writing the same stream: a session that
# started background work ends its turn with a result event, and resumes when that work reports.
# So the session has ended only once the stream's last event is a result and the stream has not
# grown for three settle periods (a minute each by default). Until then nothing is checked: a check
# of a part transcript would pass a session still at work.
settled=0 last=-1
until [ "$settled" -ge 3 ]; do
  size=$(stat -c %s "$transcript")
  if tail -n 1 "$transcript" | grep -q '"type":"result","duration_ms":' && [ "$size" -eq "$last" ]; then
    settled=$((settled + 1))
  else
    settled=0
  fi
  last=$size
  sleep "${CLEANROOM_SETTLE_SECONDS:-60}"
done
cp "$transcript" "$lane/transcript.jsonl"
echo "launch: session exited $status; transcript $transcript ($(wc -l <"$transcript") events)"

# The configuration is as it was launched.
[ "$(sha256sum <"$settings")" = "$settings_sum" ] || { echo "LAUNCH BREACH $domain: $settings changed during the session" >&2; exit 1; }
[ "$(sha256sum <"$lane/vector_runner.py")" = "$runner_sum" ] || { echo "LAUNCH BREACH $domain: $lane/vector_runner.py changed during the session" >&2; exit 1; }

# 4. The canary check, on the transcript and on the export's history.
if ! bash "$here/canary.sh" check "$transcript" "$export_dir"; then
  echo "LAUNCH BREACH $domain: see $lane" >&2
  exit 1
fi
[ "$status" -eq 0 ] || die "the session failed (exit $status); see $lane/stderr.txt"
echo "LAUNCH OK $domain $lane"
