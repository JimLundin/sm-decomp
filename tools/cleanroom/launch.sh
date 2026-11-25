#!/usr/bin/env bash
# launch.sh EXPORT BRIEF TRANSCRIPT: run a domain's implementer as a headless Claude Code session
# confined to EXPORT (MODEL §5, "The clean room is enforced by construction"). Canaries only show
# a read afterwards; this stops it happening.
#
# How the root runs a clean-room lane, end to end (from a checkout of this repository), with
# ROOT the exports' root, $CLEANROOM_ROOT (default C:/cleanroom on Windows, /cleanroom elsewhere):
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
# Off Windows (Linux) the session runs in a box, and the rules below are a second layer inside it:
#   - The box is a docker container (`docker run` as this user, its argv in the lane's box.txt)
#     with no network, a read-only root made from nothing (box_relay.py image, which also checks
#     the tag's one layer is that root and its configuration empty before trusting it), no
#     capabilities, no new privileges,
#     and limits: CLEANROOM_BOX_MEMORY (12g), CLEANROOM_BOX_PIDS (4096), CLEANROOM_BOX_TMP (/tmp,
#     2g) and CLEANROOM_SESSION_SECONDS (the CLI's deadline, 21600). Its file system holds only
#     its mounts, each at its own path: the export (written) but its .git's config, info/ and
#     hooks/ (read-only), the lane's home/ and tmp/ (written), and read-only /usr, the
#     /etc/alternatives links, rustup's proxies by name from cargo's bin/ (not cargo's home or
#     what `cargo install` put there), rustup's home, the CLI, and the lane's settings, runner,
#     shape scan and relay. The home, the checkout, the temporary folder, the canary record, the
#     ROM and the record cache are not in it, so a process the session starts (cargo, a test,
#     git) can't read them either, whatever the rules allow; nor can a search from / or the
#     export's parents, and a folder made beside the export during the session is not in it.
#     Bubblewrap and the CLI's own sandbox need unprivileged user namespaces, which Ubuntu's
#     AppArmor restricts; docker needs nothing changed on the host. The box can still see facts
#     about the host that hold no game code or data: the host paths in /proc/self/mountinfo,
#     the DNS search suffix, /proc/cmdline and the machine's DMI ids.
#   - Every git the host runs on an export afterwards (canary.sh check, import.sh, blind_copy.sh
#     and the checks here) reads a mirror of its objects and refs alone (lib.sh export_mirror,
#     capped at CLEANROOM_MIRROR_MAX_MB), never its configuration, attributes, hooks, replace
#     refs, commondir or alternates, with git_isolated turning off what a configuration could
#     still name and replace objects. So nothing the box writes in .git runs on the host or
#     chooses what the canary check sees. After the session, anything in the export's .git or
#     the lane's home/ and tmp/ that a session's git work doesn't leave (commondir,
#     config.worktree, alternates, a replace ref or another branch, a renamed .git, another
#     repository) is a LAUNCH BREACH (lib.sh export_git_odd): a host tool's git would read it.
#   - The credential stays outside: box_relay.py serves the lane's api.sock outside the box and
#     adds the key (an apiKeyHelper's output, ANTHROPIC_API_KEY or ANTHROPIC_AUTH_TOKEN, and
#     ANTHROPIC_CUSTOM_HEADERS) to each request. It passes an allow-list only: GET or POST of an
#     origin-form target under the base path's v1/, plain segments and query, the configured
#     host's own Host header, and the API's request headers. Inside, the box's first process
#     forwards 127.0.0.1:8765 to that socket and runs the CLI, whose settings hold that address,
#     a placeholder key, and only the model and CLAUDE_CODE_DISABLE_* lines. The API is the one
#     place the box can reach.
#   - The relay exits with this script, even when it is killed outright, and an exit this script
#     sees (INT, TERM, HUP) also stops its box. A box left running by a killed launch loses the
#     API and stops at its deadline; the next launch names any box whose launcher is gone. Box
#     and lane names are unique per launch. The docker client runs with this user's
#     DOCKER_CONFIG, never the lane's home.
#   - A launch on Linux needs docker (`docker info` as this user); there is no session without
#     the box.
# The rules are built from that machine's paths:
#   - The CLI is `claude` on PATH or CLAUDE_BIN, run by absolute path; nothing goes through cygpath.
#   - Rule paths are the CLI's absolute form, `//` and the path (`//home/me/**` is /home/me).
#   - The export is outside the home (export.sh and launch.sh refuse one in it): host tools run
#     git in the repositories they find under the home (an editor's source control runs
#     `git status` with the repository's own configuration), and the box writes the export.
#   - Reads (Read, Glob, Grep) and writes (Edit, Write) are denied for every entry of every
#     directory on the way from / to the export but the one leading to it (the system folders,
#     the other exports), and by name: the home whole, the main checkout's folder and its
#     sm-decomp* siblings, every worktree, the temporary folder ($TEMP, else $TMPDIR, else
#     /tmp) and CLEANROOM_PRIVATE (the canary record and the lane). The entries are those
#     present at launch; the box holds none of them anyway. A name holding one of ()[]*?{}!\
#     can't be written as a rule, so it stops the launch, as does a launch from outside a
#     checkout, whose trees the rules then couldn't name.
#   - Bash commands holding `:\` (a drive path) are denied here too; on Windows they can't be,
#     since the runner command names two, and the options that read or write a file they name
#     (--pathspec-from-file, --output, --logfile, --target-dir) are denied on both.
#   - The emptied environment holds HOME, CLAUDE_CONFIG_DIR, the XDG folders and TMPDIR in the
#     lane's home/ and tmp/; PATH is the lane's bin/, cargo's folder and /usr/local/bin:/usr/bin:/bin.
#   - The API key comes from CLEANROOM_AUTH, else the apiKeyHelper and ANTHROPIC_* lines of
#     ~/.claude/settings.json (a helper runs outside the box, as this user), else
#     ANTHROPIC_API_KEY; the relay holds it, and env.txt, the box's environment, holds none.
#   - Python is CLEANROOM_PY, else python3.13 or python3 on PATH, 3.11 or later, by absolute path.
#   - The test command is `cleanroom-tests`, a script in the lane's bin/ that runs that Python on
#     the lane's vector_runner.py, which runs the shape scan and then `cargo test` per target. A
#     bare name keeps an absolute path out of the command, which the ` /` rule would refuse;
#     Linux doesn't search the working directory first, so the export can't shadow it; the script
#     is in vector_runner.sha256 with the runner.
# First run on a Linux machine (BUILDING.md, "Setting up on Linux"):
#   1. CLEANROOM_DRY_RUN=1 on the probe export; read settings.json's deny list, env.txt and
#      box.txt. launch_config_test.sh runs the box with a fake CLI that tries each way out.
#   2. canary.sh plant with the forbidden sources present (canary.sh list shows them), then the
#      probe above: every CHECK refused (29 to 31, searches from / and the home, refused or
#      finding nothing outside the export), TASK 7's runner passes, and `LAUNCH OK launch-probe`,
#      whose canary check found nothing; CHECK 28's write into the lane is a LAUNCH BREACH.
#   3. canary.sh check on the probe transcript once more, then delete the probe export.
#
# Limits the CLI cannot confine: cargo runs code the implementer writes (tests, build scripts,
# macros) as an ordinary process, which on Windows can read any file this user can (on Linux,
# only what the box holds). Build scripts and cargo
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
# The main checkout of the repository launch.sh is in; a launch from anywhere else stops, since the
# rules that name the project's trees come from it.
checkout_main() {
  local common
  common=$(git -C "$here" rev-parse --path-format=absolute --git-common-dir)     || die "launch.sh is not in a checkout of the repository: $here"
  (cd "$common/.." && pwd -P)
}

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
# crate. Its history is read from a mirror of its objects and refs (lib.sh), never with its own
# configuration, which an earlier session on it could have written.
check_dir=$(mktemp -d "${TEMP:-${TMPDIR:-/tmp}}/launch-check-XXXXXX")
export_mirror "$export_dir" "$check_dir/export.git"
root=$(git_isolated "$check_dir/export.git" rev-list --max-parents=0 HEAD) || die "$export_dir is not a repository"
[ "$(echo "$root" | wc -l)" -eq 1 ] || die "$export_dir has more than one root commit"
[ -z "$(export_remotes "$export_dir")" ] || die "$export_dir has a remote"
mapfile -t models < <(git_isolated "$check_dir/export.git" ls-tree -r --name-only "$root" crates/clean/ | model_dirs)
rm -rf "$check_dir"
if [ "$windows" = 0 ]; then
  # Off Windows the exports' root is outside the home: host tools run git in repositories they
  # find there (an editor's source control opens every repository under its workspace and runs
  # `git status` with the repository's own configuration), and the box writes the export.
  real_home=$(cd "$HOME" && pwd -P)
  case "$export_dir/" in
    "$real_home"/*) die "the export $export_dir is in the home; exports go under a root outside it (CLEANROOM_ROOT=/cleanroom), where host tools don't run git" ;;
  esac
  odd=$(export_git_odd "$export_dir")
  [ -z "$odd" ] || die "the export's .git holds what no session's git leaves, so it is not launched: $odd"
fi
[ ${#models[@]} -eq 1 ] || die "the export holds ${#models[@]} model crates, not one"
domain=$(domain_of "${models[0]#crates/clean/}")
check_domain "$domain"
[ "$probe" = 0 ] || [ "$domain" = launch-probe ] || die "--probe runs only on an export of the domain launch-probe"

# 2. The lane directory, outside every repository.
private=${CLEANROOM_PRIVATE:-$HOME/.cleanroom}
# Each launch has its own: two in one second get -2, -3 and so on.
mkdir -p "$private/lanes"
lane_base=$private/lanes/$(date -u +%Y%m%dT%H%M%SZ)-$domain
lane=$lane_base n=1
until mkdir "$lane" 2>/dev/null; do
  n=$((n + 1)) lane=$lane_base-$n
  [ "$n" -lt 100 ] || die "can't make a lane directory at $lane_base"
done
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
  # A drive path after any character, `C:\`, would also refuse the runner command, which names
  # two; the options that read or write a named file are denied instead.
  drive_paths=()
else
  drive_paths=("Bash(*:\\\\*)")
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
  main=$(checkout_main)
  for proj in "$(dirname "$main")"/sm-decomp*; do [ -d "$proj" ] && outside+=("$(cd "$proj" && pwd -P)"); done
  # A git failure here would drop these rules without a word, so it stops the launch.
  worktrees=$(git -C "$here" worktree list --porcelain) || die "git can't list the worktrees of $here"
  while IFS= read -r wt; do
    wt=$(cd "$wt" 2>/dev/null && pwd -P) && outside+=("$wt")
  done < <(sed -n 's/^worktree //p' <<<"$worktrees")
  tmp_dir=${TEMP:-${TMPDIR:-/tmp}}
  [ -d "$tmp_dir" ] && outside+=("$(cd "$tmp_dir" && pwd -P)")
  outside+=("$(cd "$private" && pwd -P)" "$lane")
  real_home=$(cd "$HOME" && pwd -P)
  outside+=("$real_home")
  outside_reads=() outside_writes=()
  for o in "${outside[@]}"; do
    case "$export_dir/" in "$o"/*) die "the export $export_dir is inside $o, which the session must not read" ;; esac
    # A rule is a glob closed by `)`: a name holding one of these would deny something else.
    ! grep -q '[][()*?{}!\\]' <<<"$o" || die "can't write a deny rule for $o: rename it, it holds one of ()[]*?{}!\\"
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
  # Options that read or write a file they name: git prints a pathspec file's contents in its
  # error, and the others write where they're told.
  "Bash(* --pathspec-from-file*)" "Bash(* --pathspec-file-nul*)" "Bash(* --output*)"
  "Bash(* --logfile*)" "Bash(* --target-dir*)" "Bash(* --artifact-dir*)" "Bash(* --out-dir*)"
  # Arguments that name a place outside: parents, absolute paths (after a space or an `=`), the
  # home, variables. The CLI reads `\\` in a rule as one backslash.
  "Bash(*../*)" "Bash(*..)" "Bash(* /*)" "Bash(*=/*)" "Bash(*:/*)" "Bash(*=\\\\*)"
  "Bash(*~*)" "Bash(*\$*)" "Bash(*<*)" "Bash(*>*)" "Bash(*|*)" "Bash(*;*)" "Bash(*&*)"
  "${drive_paths[@]}"
)

# Off Windows the credential stays outside the box: box_relay.py, outside, adds it to each request,
# and the session's settings hold the box's base URL and a placeholder instead.
box_port=8765
if [ "$windows" = 0 ]; then
  auth_json=$(printf '{\n'; auth; printf '  "box": true\n}\n')
  session_auth=$("$python" -I "$here/box_relay.py" settings - "$box_port" <<<"$auth_json") ||
    die "box_relay.py can't read the credential's settings"
fi

settings=$lane/settings.json
{
  printf '{\n'
  if [ "$windows" = 1 ]; then auth; else printf '%s\n' "$session_auth"; fi
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
  main=$(checkout_main)
  temp_named=${TEMP:-${TMPDIR:-/tmp}}
  if [ "$windows" = 1 ]; then
    host() { cygpath -m "$1"; }
    native() { cygpath -w "$1"; }
    temp_long=$(cygpath -m -l "$temp_named") project_key=$(cygpath -w "$main")
  else
    host() { printf '%s' "$1"; }
    native() { printf '%s' "$1"; }
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
  text=${text//@HOME@/"$(host "${USERPROFILE:-$HOME}")"}
  # The two git options that reach a file by a native path: a read of the canary record and a
  # write into the lane, which the launch checks for afterwards.
  text=${text//@TOKENS_NATIVE@/"$(native "$(cd "$private" && pwd -P)/tokens.tsv")"}
  text=${text//@WRITE_NATIVE@/"$(native "$lane/probe-written.txt")"}
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
fi
[ "$windows" = 1 ] && [ -n "${FOUNDRY_LMS_TOKEN:-}" ] && envv+=("FOUNDRY_LMS_TOKEN=$FOUNDRY_LMS_TOKEN")
: >"$lane/home/.gitconfig"

# Off Windows the session runs in a box: a container with no network whose root holds nothing but
# the mounts below, each at its own path so the rules above still name them. Written: the export
# (but its .git's config, info/ and hooks/), the lane's home/ and tmp/. Read-only: /usr (the
# system's programs and libraries), the toolchain, the CLI, the lane's settings, runner and relay.
# The lane's api.sock is the API's way out.
box=()
if [ "$windows" = 0 ]; then
  box_image=sm-cleanroom-root:1
  uid=$(id -u) gid=$(id -g)
  printf 'implementer:x:%s:%s::%s:/bin/sh\n' "$uid" "$gid" "$lane/home" >"$lane/passwd"
  printf 'implementer:x:%s:\n' "$gid" >"$lane/group"
  cp "$here/box_relay.py" "$lane/box_relay.py"
  claude_real=$(readlink -f "$claude_bin")
  argv[0]=$claude_real
  box_name=cleanroom-$(basename "$lane")-$(od -An -tx1 -N4 /dev/urandom | tr -d ' \n')
  # Limits, so a runaway test can't starve the machine: memory, processes, /tmp's size, and a
  # deadline on the CLI inside (box_relay.py box).
  box=(docker run --rm -i --name "$box_name" --label "sm-cleanroom-lane=$lane" --network none
    --read-only --tmpfs "/tmp:rw,nosuid,nodev,mode=1777,size=${CLEANROOM_BOX_TMP:-2g}"
    --memory "${CLEANROOM_BOX_MEMORY:-12g}" --pids-limit "${CLEANROOM_BOX_PIDS:-4096}"
    --cap-drop ALL --security-opt no-new-privileges
    --user "$uid:$gid" --workdir "$export_dir" --env-file "$lane/env.txt")
  box_mount() { # SRC [rw]: SRC at its own path, read-only unless rw
    case $1 in *,* | *$'\n'*) die "can't mount $1: it holds a comma or a newline" ;; esac
    [ -e "$1" ] || die "nothing to mount at $1"
    if [ "${2:-}" = rw ]; then box+=(--mount "type=bind,src=$1,dst=$1"); else box+=(--mount "type=bind,src=$1,dst=$1,readonly"); fi
  }
  box_mount "$export_dir" rw
  # The export's git configuration, attributes and hooks name programs git runs; the session may
  # commit (objects, refs, the index, logs) but not change them. Host-side git reads the export
  # through export_mirror besides, which leaves all of .git but objects and refs behind.
  for g in config info hooks; do
    [ ! -L "$export_dir/.git/$g" ] || die "$export_dir/.git/$g is a symlink"
  done
  [ -f "$export_dir/.git/config" ] || die "$export_dir/.git/config is not a file"
  mkdir -p "$export_dir/.git/info" "$export_dir/.git/hooks"
  for g in config info hooks; do box_mount "$export_dir/.git/$g"; done
  box_mount "$lane/home" rw; box_mount "$lane/tmp" rw
  box_mount /usr; box_mount /etc/alternatives; box_mount /etc/ld.so.cache
  box+=(--mount "type=bind,src=$lane/passwd,dst=/etc/passwd,readonly"
    --mount "type=bind,src=$lane/group,dst=/etc/group,readonly")
  for f in settings.json box_relay.py bin "${runner_files[@]}"; do
    case $f in bin/*) ;; *) box_mount "$lane/$f" ;; esac
  done
  box_mount "$claude_real"
  # The toolchain: rustup's proxies by name (not cargo's home, which can hold a registry token,
  # nor the other programs `cargo install` put beside them) and rustup's toolchains. Anything else
  # the interpreter needs outside /usr is mounted too.
  cargo_dir=$(dirname "$cargo_bin")
  case $cargo_dir/ in
    /usr/*) ;;
    *) for t in cargo rustc rustup rustdoc cargo-clippy clippy-driver cargo-fmt rustfmt; do
         if [ -e "$cargo_dir/$t" ]; then box_mount "$cargo_dir/$t"; fi
       done ;;
  esac
  rustup_home=${RUSTUP_HOME:-$HOME/.rustup}
  if [ -d "$rustup_home" ]; then box_mount "$(cd "$rustup_home" && pwd -P)"; fi
  py_prefix=$("$python" -I -c 'import sys; print(sys.base_prefix)')
  case $py_prefix/ in /usr/*) ;; *) box_mount "$py_prefix" ;; esac
  case $awk_bin in /usr/* | /bin/*) ;; *) die "awk at $awk_bin is outside /usr, which the box holds" ;; esac
  box+=(--mount "type=bind,src=$lane/api.sock,dst=$lane/api.sock" "$box_image"
    "$python" -I "$lane/box_relay.py" box "$box_port" "$lane/api.sock"
    "${CLEANROOM_SESSION_SECONDS:-21600}" --)
  printf '%s\n' "${box[@]}" >"$lane/box.txt"
fi
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
if [ "$windows" = 0 ]; then
  # A box whose launcher is gone is reported, never stopped here: it may be another launch's.
  { docker ps --filter label=sm-cleanroom-lane --format '{{.Names}} {{.Label "sm-cleanroom-lane"}}' 2>/dev/null || true; } |
    while read -r other other_lane; do
      other_pid=$(cat "$other_lane/launch.pid" 2>/dev/null || true)
      if [ -z "$other_pid" ] || ! kill -0 "$other_pid" 2>/dev/null; then
        echo "launch: the box $other runs with no launcher ($other_lane); if it is yours, docker kill $other" >&2
      fi
    done
  echo "$$" >"$lane/launch.pid"
  "$python" -I "$here/box_relay.py" image "$box_image" || die "docker can't make the box's image $box_image"
  : >"$lane/api.sock"
  # The relay exits when this script does, even when it is killed outright; the box and the
  # stream are stopped on any exit this script sees. The stream runs in the background so a
  # signal reaches the trap at once rather than after the session.
  relay_pid= stream_pid=
  stop_box() {
    [ -z "$stream_pid" ] || kill "$stream_pid" 2>/dev/null || true
    [ -z "$relay_pid" ] || kill "$relay_pid" 2>/dev/null || true
    docker kill "$box_name" >/dev/null 2>&1 || true
  }
  trap stop_box EXIT
  trap 'exit 130' INT
  trap 'exit 143' TERM HUP
  "$python" -I "$here/box_relay.py" serve "$lane/api.sock" - "$$" <<<"$auth_json" 2>"$lane/relay.txt" &
  relay_pid=$!
  for _ in $(seq 100); do [ -S "$lane/api.sock" ] && break; sleep 0.1; done
  [ -S "$lane/api.sock" ] || die "the relay didn't start; see $lane/relay.txt"
  # The docker client runs outside with the session's environment, its configuration from this
  # user's (not the lane's home, which the box writes), and DOCKER_HOST if set; inside, the box
  # has only env.txt.
  env -i "${envv[@]}" "DOCKER_CONFIG=${DOCKER_CONFIG:-$HOME/.docker}" ${DOCKER_HOST:+"DOCKER_HOST=$DOCKER_HOST"} \
    "$python" -I "$here/launch_stream.py" \
    --cwd "$export_dir" --transcript "$transcript" --stderr "$lane/stderr.txt" \
    --timeout "${CLEANROOM_COMPLETION_SECONDS:-240}" \
    -- "${box[@]}" "${argv[@]}" <"$lane/brief.txt" &
  stream_pid=$!
  wait "$stream_pid" || status=$?
  stream_pid=
  kill "$relay_pid" 2>/dev/null || true
else
# Inherited stdout handles keep the pipe open while background work reports. EOF proves
# output completion for descendants that inherit the CLI stream. A deadline after CLI exit
# refuses a writer that never closes; detached writers are left running and never certified.
env -i "${envv[@]}" "$python" -I "$here/launch_stream.py" --cwd "$export_dir" \
  --transcript "$transcript" --stderr "$lane/stderr.txt" \
  --timeout "${CLEANROOM_COMPLETION_SECONDS:-240}" \
  -- "${argv[@]}" <"$lane/brief.txt" || status=$?
fi
cp "$transcript" "$lane/transcript.jsonl"
echo "launch: session exited $status; transcript $transcript ($(wc -l <"$transcript") events)"

# The configuration is as it was launched.
[ "$(sha256sum <"$settings")" = "$settings_sum" ] || { echo "LAUNCH BREACH $domain: $settings changed during the session" >&2; exit 1; }
[ "$(runner_digest)" = "$runner_sum" ] || { echo "LAUNCH BREACH $domain: the runner, its shape scan or its awk.path in $lane changed during the session" >&2; exit 1; }

[ "$probe" = 0 ] || [ ! -e "$lane/probe-written.txt" ] || { echo "LAUNCH BREACH $domain: the probe wrote $lane/probe-written.txt" >&2; exit 1; }

# Off Windows, the export's .git and the box's other writable folders hold only what a session's
# git leaves: no commondir, config.worktree, alternates, replace ref, other branch, renamed .git or
# other repository, any of which a host tool's git would read.
if [ "$windows" = 0 ]; then
  odd=$(export_git_odd "$export_dir" "$lane/home" "$lane/tmp")
  if [ -n "$odd" ]; then
    echo "LAUNCH BREACH $domain: the box left what no session's git leaves:" >&2
    sed 's/^/  /' <<<"$odd" >&2
    exit 1
  fi
fi

# 4. The canary check, on the transcript and on the export's history.
if ! bash "$here/canary.sh" check "$transcript" "$export_dir"; then
  echo "LAUNCH BREACH $domain: see $lane" >&2
  exit 1
fi
[ "$status" -eq 0 ] || die "the session failed (exit $status); see $lane/stderr.txt"
echo "LAUNCH OK $domain $lane"
