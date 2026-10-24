#!/usr/bin/env sh
# Counts of transpiler residue left in sm/src. Every number should go down
# over time; none should go up in a commit that does not explain why.
# Run from the repository root: sh tools/progress.sh
set -eu
src=sm/src
count() { grep -rE "$1" "$src" --include='*.rs' | wc -l | tr -d ' '; }
legacy=$(find "$src/game" -maxdepth 1 -type f -name 'sm_*.rs' -exec cat {} + | wc -l | tr -d ' ')

# Markdown is stdout only; keep the hand-maintained compact graph above it.
# Supply current local measurements explicitly, never cached/CI values:
# sh tools/progress.sh --markdown [WARNINGS [PASSED/TOTAL]]
case "${1-}" in
  '') ;;
  --markdown)
    [ "$#" -le 3 ] || { echo 'Expected --markdown [WARNINGS [PASSED/TOTAL]]' >&2; exit 2; }
    warning_count=${2:-not measured}
    checkpoint_counts=${3:-not measured}
    case "$warning_count" in
      'not measured') ;;
      *[!0-9]*) echo 'WARNINGS must be a nonnegative count or "not measured"' >&2; exit 2 ;;
    esac
    if [ "$checkpoint_counts" != 'not measured' ]; then
      passed=${checkpoint_counts%/*}
      total=${checkpoint_counts#*/}
      case "$passed:$total" in
        *[!0-9:]*|:*|*:) echo 'Checkpoint counts must be PASSED/TOTAL' >&2; exit 2 ;;
      esac
      [ "$checkpoint_counts" = "$passed/$total" ] && [ "$total" -gt 0 ] && [ "$passed" -le "$total" ] || {
        echo 'Checkpoint counts require 0 <= PASSED <= TOTAL and TOTAL > 0' >&2; exit 2;
      }
    fi
    # awk counts physical lines, including an unterminated final line. Passing
    # filenames via find -exec preserves spaces and avoids shell word splitting.
    metrics=$(find "$src" -type f -name '*.rs' -exec awk '
      {
        if (FILENAME ~ /\/(verification|verify)\// || FILENAME ~ /\/verify[^/]*[.]rs$/)
          verification++
        else production++
        if (index($0, "unsafe")) unsafe_lines++
      }
      END { print production+0, verification+0, unsafe_lines+0 }
    ' {} +)
    printf '%s\n' "$metrics" | awk -v warnings="$warning_count" -v segments="$checkpoint_counts" -v legacy="$legacy" '
      { production += $1; verification += $2; unsafe_lines += $3 }
      END {
        print "| Measure | Count |"
        print "| --- | ---: |"
        print "| Production Rust lines* | " production " |"
        print "| Legacy bank Rust lines | " legacy " |"
        print "| Verification Rust lines | " verification " |"
        print "| Total Rust | " production + verification " |"
        print "| Lines containing `unsafe` | " unsafe_lines " |"
        print "| Warnings (no-default library) | " warnings " |"
        print "| Required checkpoint checks passing | " segments " |"
        print ""
        print "*Physical lines by path; inline verification hooks remain in production. Verification: `verification/`, `verify/`, or filenames starting `verify`."
      }
    '
    exit ;;
  *) echo 'Usage: progress.sh [--markdown [WARNINGS [PASSED/TOTAL]]]' >&2; exit 2 ;;
esac

printf '%-34s %8s\n' metric count
printf '%-34s %8s\n' 'raw g_ram offsets'            "$(count 'g_ram as \*mut (uint8|uint8_t|__uint8_t|u8)\)')"
printf '%-34s %8s\n' 'extern "C" blocks'            "$(count '^(pub )?(unsafe )?extern "C" \{')"
printf '%-34s %8s\n' 'extern "C" fns (C ABI exports)' "$(count 'pub unsafe extern "C" fn')"
printf '%-34s %8s\n' '#[no_mangle]'                 "$(count 'no_mangle')"
printf '%-34s %8s\n' 'unsafe fns'                    "$(count 'unsafe (extern "C" )?fn ')"
printf '%-34s %8s\n' 'i32/c_int cast lines'         "$(count 'as (i32|::core::ffi::c_int)([^[:alnum:]_]|$)')"
printf '%-34s %8s\n' 'legacy bank Rust lines'       "$legacy"
printf '%-34s %8s\n' 'lines of Rust'                 "$(find "$src" -name '*.rs' -exec cat {} + | wc -l | tr -d ' ')"
