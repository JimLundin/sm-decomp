#!/usr/bin/env bash
# launch.sh EXPORT BRIEF TRANSCRIPT: run a domain's implementer as a headless Claude Code session
# confined to EXPORT (MODEL §5, "The clean room is enforced by construction"). Canaries only show
# a read afterwards; this stops it happening.
#
# How the root runs a clean-room lane, end to end (from a checkout of this repository), with
# ROOT the exports' root, $CLEANROOM_ROOT (default C:/cleanroom on Windows, ~/cleanroom elsewhere):
#   1. bash tools/cleanroom/canary.sh plant              fresh tokens and fingerprints
#   2. bash tools/cleanroom/export.sh DOMAIN             into ROOT/DOMAIN
#   3. bash tools/cleanroom/launch.sh ROOT/DOMAIN BRIEF TRANSCRIPT
#        BRIEF is a text file written for the implementer; TRANSCRIPT lies outside the export.
#        Exits 0 only when the session ended and its transcript and the export are canary-clean.
#   4. bash tools/cleanroom/canary.sh check TRANSCRIPT ROOT/DOMAIN
#        launch.sh already ran it; run it again on anything else the lane produced.
#   5. bash tools/cleanroom/import.sh ROOT/DOMAIN BRANCH
#
# What launch.sh does:
#   - refuses to start unless scan_brief.sh passes on BRIEF, EXPORT is an export (one root commit,
#     no remote) and TRANSCRIPT is outside EXPORT;
#   - makes a lane directory, $CLEANROOM_PRIVATE/lanes/<timestamp>/ (default ~/.cleanroom), outside
#     every repository, holding: brief.txt (the brief, ending with the test command),
#     settings.json (the permission configuration), vector_runner.py, the shape scan it runs
#     (strip_rust.awk, shape_scan.awk), the awk it runs them with (awk.path) and their
#     vector_runner.sha256 (the test runner), argv.txt and env.txt (the exact launch), home/ (the
#     session's empty home and config directory) and tmp/ (its TEMP);
#   - runs the CLI with an emptied environment, HOME, USERPROFILE (Windows) and CLAUDE_CONFIG_DIR pointing at
#     home/, so no user settings, memory, CLAUDE.md, plugins, skills, hooks, MCP servers or session
#     history exist for it to load; and with --bare, --restricted, --setting-sources '' and
#     --strict-mcp-config besides;
#   - confines it with settings.json, which lives in the lane directory and so out of its reach:
#     mode dontAsk (anything not allowed is refused, nothing prompts), the tool set Read, Glob,
#     Grep, Edit, Write and Bash only (no web, agent, notebook, skill or MCP tool exists), Read
#     under EXPORT, Edit and Write under the [import] paths of allowlist.txt, Bash for
#     `cargo check|clippy|fmt|test|build`, `git add|commit|status|diff|log|rm|mv` and the one test
#     command, on Windows `"<python>" -I "<lane>\vector_runner.py"` (Defender blocks the
#     executables `cargo test` links; the runner links the clean crates' tests as DLLs and takes
#     no arguments) and elsewhere `cleanroom-tests` (below), and explicit deny rules for
#     everything the adversary named;
#   - writes the stream-json transcript to TRANSCRIPT and a copy into the lane directory, then
#     exits 1 if settings.json, vector_runner.py, its shape scan or its awk.path changed during the
#     session, or if canary.sh check finds a token or fingerprint in the transcript or the
#     export's history.
#
# A commit message passes the deny rules when each paragraph is its own `-m` and holds none of
# ; & | < > $ ~ `..` ` /` `:/` (`git commit -m "Subject" -m "Body." -m "Role: implementer"`).
# The samus-movement session's commits were refused for a `;` in the body and for `-F -` with a
# heredoc; CLEANROOM.md tells the implementer the form.
#
# Before the first real lane on a new CLI version or a new machine, prove the confinement again:
#   CLEANROOM_ALLOW_MISSING=1 bash tools/cleanroom/export.sh launch-probe
#   bash tools/cleanroom/launch.sh --probe ROOT/launch-probe TRANSCRIPT
# --probe sends launch_probe.txt, which tries each breach the adversary named and then commits one
# file; scan_brief.sh refuses that brief by design, so --probe runs on the dummy domain only. Its
# @NAME@ placeholders become this machine's paths (the temporary folder, the main checkout, the
# root's memory, the canary record, the settings file). Read the denials in TRANSCRIPT ("has been
# denied", "denied by your permission settings"): every CHECK refused, every TASK done.
# CLEANROOM_DRY_RUN=1 writes the lane record and stops before the session (launch_config_test.sh).
#
# Off Windows (Linux) the barrier is the same, built from that machine's paths:
#   - The CLI is `claude` on PATH or CLAUDE_BIN, run by absolute path; nothing goes through cygpath.
#   - Rule paths are the CLI's absolute form, `//` and the path (`//home/me/**` is /home/me).
#   - Reads (Read, Glob, Grep) and writes (Edit, Write) are denied for every entry of every
#     directory on the way from / to the export but the one leading to it: the system folders,
#     the other users' homes, the user's whole home but the exports' root, and the other exports.
#     The Windows rules deny the home outright, but here the exports' root is in the home, and a
#     deny rule wins over an allow, so the home is denied entry by entry around it. On top, by
#     name: the main checkout's folder and its sm-decomp* siblings, every worktree, the temporary
#     folder ($TEMP, else $TMPDIR, else /tmp), CLEANROOM_PRIVATE (the canary record and the lane)
#     and, when the exports' root is outside it, the home itself.
#   - The emptied environment holds HOME, CLAUDE_CONFIG_DIR, the XDG folders and TMPDIR in the
#     lane's home/ and tmp/; PATH is the lane's bin/, cargo's folder and /usr/local/bin:/usr/bin:/bin.
#   - The API key comes from CLEANROOM_AUTH, else the apiKeyHelper and ANTHROPIC_* lines of
#     ~/.claude/settings.json (a helper runs with the lane's HOME, so it names absolute paths),
#     else ANTHROPIC_API_KEY, passed in the environment and redacted in env.txt.
#   - Python is CLEANROOM_PY, else python3.13 or python3 on PATH, 3.11 or later, by absolute path.
#   - The test command is `cleanroom-tests`, a script in the lane's bin/ that runs that Python on
#     the lane's vector_runner.py, which runs the shape scan and then `cargo test` per target. A
#     bare name keeps an absolute path out of the command, which the ` /` rule would refuse;
#     Linux doesn't search the working directory first, so the export can't shadow it; the script
#     is in vector_runner.sha256 with the runner.
# First run on a Linux machine (BUILDING.md, "Setting up on Linux"):
#   1. CLEANROOM_DRY_RUN=1 on the probe export; read settings.json's deny list and env.txt.
#   2. canary.sh plant, then the probe above: every CHECK refused, TASK 7's runner passes, and
#      `LAUNCH OK launch-probe`, whose canary check found nothing.
#   3. canary.sh check on the probe transcript once more, then delete the probe export.
#
# Limits the CLI cannot confine: cargo runs code the implementer writes (tests, build scripts,
# macros) as an ordinary process, which can read any file this user can. Build scripts and cargo
# configuration are not writable here, but a test body is. The canary check after the session is
# what catches a test that read a forbidden file and printed it. On the trunk, code_check.sh, which
# import.sh runs before applying anything, refuses what cargo would run or read at build time
# (include!/include_str! outside the vector file, build scripts, dependencies beyond
# allowed_deps.txt), and import.sh's clean-room clippy run refuses the source that could reach out
# (std::fs, net, process, env, unsafe, include_bytes!) before anything runs it.
# The clean crates' Cargo.toml is not writable here: their dependencies are the integrator's, from
# allowed_deps.txt (export.sh gives a new model crate the vector parser), so a manifest edit is
# never needed. CLI 2.1.316 offers only Read, Edit and Bash (Edit creates files; Glob, Grep and
# Write don't exist); their rules stay for later CLIs.
#
# Overrides: CLAUDE_BIN (default ~/.local/bin/claude.exe on Windows, `claude` on PATH elsewhere),
# CLEANROOM_MODEL (default opus), CLEANROOM_MAX_BUDGET_USD (passed as --max-budget-usd),
# CLEANROOM_AUTH (a file of top-level settings JSON members, `"apiKeyHelper": ..., "env": {...}`;
# default: the apiKeyHelper and the ANTHROPIC_* lines of ~/.claude/settings.json), CLEANROOM_PY
# (the runner's interpreter, default %LOCALAPPDATA%/sm-decomp-tools/python-3.13.7/python.exe on
# Windows), CLEANROOM_PRIVATE, CARGO_HOME, RUSTUP_HOME, CLEANROOM_COMPLETION_SECONDS (output
# deadline after CLI exit, default 240), CLEANROOM_DRY_RUN.
set -euo pipefail
here=$(cd "$(dirname "$0")" && pwd)
. "$here/lib.sh"
case "$(uname -s)" in MINGW* | MSYS* | CYGWIN*) windows=1 ;; *) windows=0 ;; esac

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
if [ "$windows" = 1 ]; then
  claude_bin=${CLAUDE_BIN:-$USERPROFILE/.local/bin/claude.exe}
else
  # The session's PATH is short, so the CLI goes by absolute path.
  claude_bin=${CLAUDE_BIN:-$(command -v claude || true)}
  case $claude_bin in /*) ;; *) die "no Claude Code CLI on PATH by absolute path; set CLAUDE_BIN" ;; esac
fi
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
# the clean crates' tests through one fixed Python script, copied here out of its reach with the
# shape scan it runs first (code_check.sh's G6 and G7, which import.sh would refuse). The one
# command it may run is that interpreter on that copy, both by absolute Windows path in double
# quotes (a forward-slash path would meet the `:/` and ` /` deny rules below, which win).
if [ "$windows" = 1 ]; then
  python=${CLEANROOM_PY:-${LOCALAPPDATA:-$USERPROFILE/AppData/Local}/sm-decomp-tools/python-3.13.7/python.exe}
  [ -f "$python" ] || die "no Python at $python; set CLEANROOM_PY"
else
  python=${CLEANROOM_PY:-$(command -v python3.13 || command -v python3 || true)}
  case $python in /*) ;; *) die "no Python by absolute path; set CLEANROOM_PY" ;; esac
  [ -x "$python" ] || die "no Python at $python; set CLEANROOM_PY"
  "$python" -c 'import sys; sys.exit(sys.version_info < (3, 11))' || die "$python is older than 3.11"
fi
runner_files=(vector_runner.py strip_rust.awk shape_scan.awk awk.path)
for f in "${runner_files[@]:0:3}"; do cp "$here/$f" "$lane/$f"; done
# The awk the runner runs, by absolute path as the interpreter is: a bare `awk` on Windows
# resolves in the export first.
awk_bin=$(command -v awk) || die "no awk on PATH for the runner's shape scan"
if [ "$windows" = 1 ]; then
  cygpath -w "$awk_bin" >"$lane/awk.path"
  run_tests="\"$(cygpath -w "$python")\" -I \"$(cygpath -w "$lane/vector_runner.py")\""
else
  case $awk_bin in /*) ;; *) die "no awk by absolute path on PATH" ;; esac
  printf '%s\n' "$awk_bin" >"$lane/awk.path"
  # The command is a bare name on the session's PATH: an absolute path in it would meet the
  # ` /` deny rule. The script names its two paths in single quotes, so they hold none.
  case "$python$lane" in *"'"*) die "a quote in $python or $lane" ;; esac
  mkdir -p "$lane/bin"
  printf "#!/bin/sh\nexec '%s' -I '%s' \"\$@\"\n" "$python" "$lane/vector_runner.py" >"$lane/bin/cleanroom-tests"
  chmod 755 "$lane/bin/cleanroom-tests"
  runner_files+=(bin/cleanroom-tests)
  run_tests=cleanroom-tests
fi
runner_digest() { (cd "$lane" && sha256sum "${runner_files[@]}"); }
runner_digest >"$lane/vector_runner.sha256"
runner_sum=$(cat "$lane/vector_runner.sha256")
# The brief ends with the command, as the export cannot know the lane directory; CLEANROOM.md
# says the brief gives it.
printf '\nRun the tests with exactly this Bash command, from the working directory:\n\n    %s\n' \
  "$run_tests" >>"$lane/brief.txt"
# The probe tries the interpreter itself, outside the runner.
if [ "$probe" = 1 ]; then
  if [ "$windows" = 1 ]; then py_shown="\"$(cygpath -w "$python")\""; else py_shown="\"$python\""; fi
  printf '\nPYTHON is %s\n' "$py_shown" >>"$lane/brief.txt"
fi

# Paths as the CLI matches them: `//` and the absolute path, on Windows its drive as a folder
# (`//c/dir/**` is C:/dir).
rule_path() { # DIR -> //c/dir, or //dir
  if [ "$windows" = 1 ]; then
    p=$(cygpath -u "$1" 2>/dev/null || echo "$1")
  else
    p=$1
  fi
  echo "/$p"
}
ex=$(rule_path "$export_dir")
lane_rule=$(rule_path "$lane")
mapfile -t writable < <(allowlist_section import "$domain")
if [ "$windows" = 1 ]; then
  home_rule=$(rule_path "$(cygpath -u "$USERPROFILE" 2>/dev/null || echo "$HOME")")
  outside_reads=(
    "Read(//c/Users/**)" "Glob(//c/Users/**)" "Grep(//c/Users/**)"
    "Read($home_rule/**)" "Glob($home_rule/**)" "Grep($home_rule/**)"
    "Read(//c/Windows/**)" "Read(//c/ProgramData/**)" "Read(//c/scratch/**)"
    "Read(//d/**)" "Read(//e/**)" "Read(//f/**)"
  )
  outside_writes=("Edit($lane_rule/**)" "Write($lane_rule/**)" "Edit($home_rule/**)" "Write($home_rule/**)")
else
  # Every entry beside the export's path, from / down: the export's root is in the home, and a
  # deny wins over an allow, so the home is denied around it rather than whole.
  outside=()
  p=$export_dir
  while [ "$p" != / ]; do
    parent=$(dirname "$p")
    for e in "${parent%/}"/* "${parent%/}"/.[!.]* "${parent%/}"/..?*; do
      [ -e "$e" ] || [ -L "$e" ] || continue
      [ "$e" = "$p" ] && continue
      case $e in *$'\n'*) continue ;; esac
      outside+=("$e")
    done
    p=$parent
  done
  # By name besides: the project's trees and worktrees, the temporary folder, the canary record
  # and this lane, and the home itself when the export isn't in it.
  main=$(cd "$(git -C "$here" rev-parse --path-format=absolute --git-common-dir)/.." && pwd -P)
  for proj in "$(dirname "$main")"/sm-decomp*; do [ -d "$proj" ] && outside+=("$(cd "$proj" && pwd -P)"); done
  while IFS= read -r wt; do
    wt=$(cd "$wt" 2>/dev/null && pwd -P) && outside+=("$wt")
  done < <(git -C "$here" worktree list --porcelain | sed -n 's/^worktree //p')
  tmp_dir=${TEMP:-${TMPDIR:-/tmp}}
  [ -d "$tmp_dir" ] && outside+=("$(cd "$tmp_dir" && pwd -P)")
  outside+=("$(cd "$private" && pwd -P)" "$lane")
  real_home=$(cd "$HOME" && pwd -P)
  case "$export_dir/" in "$real_home"/*) ;; *) outside+=("$real_home") ;; esac
  outside_reads=() outside_writes=()
  for o in "${outside[@]}"; do
    case "$export_dir/" in "$o"/*) die "the export $export_dir is inside $o, which the session must not read" ;; esac
    if [ -d "$o" ]; then r="$(rule_path "$o")/**"; else r=$(rule_path "$o"); fi
    outside_reads+=("Read($r)" "Glob($r)" "Grep($r)")
    outside_writes+=("Edit($r)" "Write($r)")
  done
fi

auth() {
  if [ -n "${CLEANROOM_AUTH:-}" ]; then cat "$CLEANROOM_AUTH"; return; fi
  user=${USERPROFILE:-$HOME}/.claude/settings.json
  if [ "$windows" = 1 ]; then
    [ -f "$user" ] || die "no CLEANROOM_AUTH and no $user to take the API key helper from"
    # The helper runs with the session's short PATH, so a bare powershell.exe gets its full path.
    helper=$(grep -E '^\s*"apiKeyHelper"' "$user" | sed 's/,\s*$//; s#"powershell\.exe #"C:/Windows/System32/WindowsPowerShell/v1.0/powershell.exe #') || true
  elif [ -f "$user" ]; then
    helper=$(grep -E '^\s*"apiKeyHelper"' "$user" | sed 's/,\s*$//') || true
  else
    helper= user=/dev/null
  fi
  [ -n "$helper" ] && printf '  %s,\n' "$helper"
  printf '  "env": {\n'
  { grep -E '^\s*"(ANTHROPIC_[A-Z_]+|CLAUDE_CODE_DISABLE_[A-Z_]+|CLAUDE_CODE_GIT_BASH_PATH)"' "$user" || true; } \
    | sed 's/,\s*$//' | sed '$!s/$/,/'
  printf '  },\n'
}
# Off Windows, without CLEANROOM_AUTH or a helper or key in the user's settings, the key from the
# environment: it goes to the CLI's environment, not into settings.json, and env.txt redacts it.
api_key_env=0
if [ "$windows" = 0 ] && [ -z "${CLEANROOM_AUTH:-}" ] \
  && ! grep -qsE '^\s*"(apiKeyHelper|ANTHROPIC_API_KEY|ANTHROPIC_AUTH_TOKEN)"' "$HOME/.claude/settings.json"; then
  [ -n "${ANTHROPIC_API_KEY:-}" ] \
    || die "no API key: set CLEANROOM_AUTH, an apiKeyHelper in ~/.claude/settings.json, or ANTHROPIC_API_KEY"
  api_key_env=1
fi

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
  "${outside_reads[@]}"
  # The configuration and the lane record, though both are outside the export already.
  "${outside_writes[@]}"
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
# The probe brief names this machine's real paths, so each breach is tried on the real place:
# the temporary folder (as $TEMP spells it and resolved), the main checkout, the root's memory
# (the CLI's project folder: the checkout's path, each character but letters and digits a dash),
# the canary record and the settings file.
if [ "$probe" = 1 ]; then
  main=$(cd "$(git -C "$here" rev-parse --path-format=absolute --git-common-dir)/.." && pwd -P)
  temp_named=${TEMP:-${TMPDIR:-/tmp}}
  if [ "$windows" = 1 ]; then
    host() { cygpath -m "$1"; }
    temp_long=$(cygpath -m -l "$temp_named") project_key=$(cygpath -w "$main")
  else
    host() { printf '%s' "$1"; }
    temp_long=$(cd "$temp_named" && pwd -P) project_key=$main
  fi
  memory="$(host "${USERPROFILE:-$HOME}")/.claude/projects/$(printf '%s' "$project_key" | sed 's/[^A-Za-z0-9]/-/g')/memory/"
  text=$(cat "$lane/brief.txt")
  text=${text//@TEMP_SHORT@/"$(host "$temp_named")"}
  text=${text//@TEMP@/"$temp_long"}
  text=${text//@PROJECT@/"$(host "$main")"}
  text=${text//@MEMORY@/"$memory"}
  text=${text//@PRIVATE@/"$(host "$(cd "$private" && pwd -P)")"}
  text=${text//@SETTINGS@/"$(host "$settings")"}
  ! grep -qE '@[A-Z_]+@' <<<"$text" || die "the probe brief holds a placeholder launch.sh doesn't fill"
  printf '%s\n' "$text" >"$lane/brief.txt"
fi

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
if [ "$windows" = 1 ]; then
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
else
  cargo_bin=$(command -v cargo) || die "no cargo on PATH"
  envv=(
    "PATH=$lane/bin:$(dirname "$cargo_bin"):/usr/local/bin:/usr/bin:/bin"
    "HOME=$lane/home" "CLAUDE_CONFIG_DIR=$lane/home/.claude"
    "XDG_CONFIG_HOME=$lane/home/.config" "XDG_CACHE_HOME=$lane/home/.cache"
    "XDG_DATA_HOME=$lane/home/.local/share" "XDG_STATE_HOME=$lane/home/.local/state"
    "TMPDIR=$lane/tmp" "TEMP=$lane/tmp" "TMP=$lane/tmp"
    "SHELL=$(command -v bash)" "LANG=C.UTF-8"
    "CARGO_HOME=${CARGO_HOME:-$HOME/.cargo}" "RUSTUP_HOME=${RUSTUP_HOME:-$HOME/.rustup}"
    "RUSTUP_TOOLCHAIN_SOURCE="
    "GIT_CONFIG_NOSYSTEM=1" "GIT_CONFIG_GLOBAL=$lane/home/.gitconfig"
    "GIT_AUTHOR_NAME=implementer" "GIT_AUTHOR_EMAIL=implementer@cleanroom.invalid"
    "GIT_COMMITTER_NAME=implementer" "GIT_COMMITTER_EMAIL=implementer@cleanroom.invalid"
    "DISABLE_AUTOUPDATER=1"
  )
  [ "$api_key_env" = 0 ] || envv+=("ANTHROPIC_API_KEY=$ANTHROPIC_API_KEY")
fi
[ -n "${FOUNDRY_LMS_TOKEN:-}" ] && envv+=("FOUNDRY_LMS_TOKEN=$FOUNDRY_LMS_TOKEN")
: >"$lane/home/.gitconfig"
printf '%s\n' "${argv[@]}" >"$lane/argv.txt"
printf '%s\n' "${envv[@]}" | sed -E 's/^(FOUNDRY_LMS_TOKEN|ANTHROPIC_API_KEY)=.*/\1=<redacted>/' >"$lane/env.txt"
printf 'export\t%s\nroot\t%s\nbrief\t%s\ntranscript\t%s\n' "$export_dir" "$root" "$brief" "$transcript" >"$lane/lane.tsv"
echo "launch: $domain in $export_dir; lane record $lane"
if [ "${CLEANROOM_DRY_RUN:-0}" = 1 ]; then
  echo "LAUNCH DRY RUN $domain $lane"
  exit 0
fi

# 3. The session, its prompt the brief, its working directory the export.
status=0
# Inherited stdout handles keep the pipe open while background work reports. EOF proves
# output completion for descendants that inherit the CLI stream. A deadline after CLI exit
# refuses a writer that never closes; detached writers are left running and never certified.
env -i "${envv[@]}" "$python" -I "$here/launch_stream.py" --cwd "$export_dir" \
  --transcript "$transcript" --stderr "$lane/stderr.txt" \
  --timeout "${CLEANROOM_COMPLETION_SECONDS:-240}" \
  -- "${argv[@]}" <"$lane/brief.txt" || status=$?
cp "$transcript" "$lane/transcript.jsonl"
echo "launch: session exited $status; transcript $transcript ($(wc -l <"$transcript") events)"

# The configuration is as it was launched.
[ "$(sha256sum <"$settings")" = "$settings_sum" ] || { echo "LAUNCH BREACH $domain: $settings changed during the session" >&2; exit 1; }
[ "$(runner_digest)" = "$runner_sum" ] || { echo "LAUNCH BREACH $domain: the runner, its shape scan or its awk.path in $lane changed during the session" >&2; exit 1; }

# 4. The canary check, on the transcript and on the export's history.
if ! bash "$here/canary.sh" check "$transcript" "$export_dir"; then
  echo "LAUNCH BREACH $domain: see $lane" >&2
  exit 1
fi
[ "$status" -eq 0 ] || die "the session failed (exit $status); see $lane/stderr.txt"
echo "LAUNCH OK $domain $lane"
