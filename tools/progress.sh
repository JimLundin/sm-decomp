#!/usr/bin/env sh
# Counts of transpiler residue left in sm/src. Every number should go down
# over time; none should go up in a commit that does not explain why.
# Run from the repository root: sh tools/progress.sh
set -eu
src=sm/src
count() { grep -rE "$1" "$src" --include='*.rs' | wc -l | tr -d ' '; }

printf '%-34s %8s\n' metric count
printf '%-34s %8s\n' 'raw g_ram offsets'            "$(count 'g_ram as \*mut uint8\)')"
printf '%-34s %8s\n' 'extern "C" blocks'            "$(count '^(pub )?(unsafe )?extern "C" \{')"
printf '%-34s %8s\n' 'extern "C" fns (C ABI exports)' "$(count 'pub unsafe extern "C" fn')"
printf '%-34s %8s\n' '#[no_mangle]'                 "$(count 'no_mangle')"
printf '%-34s %8s\n' 'unsafe fns'                    "$(count 'unsafe (extern "C" )?fn ')"
printf '%-34s %8s\n' 'i32/c_int cast lines'         "$(count 'as (i32|::core::ffi::c_int)([^[:alnum:]_]|$)')"
printf '%-34s %8s\n' 'lines of Rust'                 "$(find "$src" -name '*.rs' -exec cat {} + | wc -l | tr -d ' ')"
