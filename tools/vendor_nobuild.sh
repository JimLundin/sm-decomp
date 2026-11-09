#!/usr/bin/env bash
# vendor_nobuild.sh CRATE VERSION: write vendor/CRATE-VERSION/, the published crate with its build
# script removed and what that script emits on the pinned toolchain baked in, so nothing in the
# build runs a freshly built executable (Defender blocks those on the development machine). The
# root Cargo.toml's [patch.crates-io] points at the copy; tools/vendor_nobuild_check.sh holds the
# baked values to the real build scripts' output in CI. Rerun it on an upgrade, with a recipe
# below for any crate whose build script changed.
#
# The transformation, from the .crate file whose SHA-256 the crates.io index lists:
#   - build.rs, Cargo.toml.orig, Cargo.lock, .cargo_vcs_info.json, .cargo-ok, .github,
#     .gitignore and rust-toolchain.toml are dropped; the licences, README, sources and tests stay.
#   - The manifest's `build = "build.rs"` becomes `build = false`.
#   - A cfg the script sets (`cargo:rustc-cfg=NAME`) is replaced in src/ by the predicate under
#     which the script sets it: `all()` when always, `feature = "x"` when only with that feature.
#     A cfg it never sets on this toolchain is left as written: an unset cfg is false.
#   - The cfgs it declares (`cargo:rustc-check-cfg`) move to the manifest's
#     [lints.rust.unexpected_cfgs] check-cfg, cargo's own place for them.
#   - A file it writes to OUT_DIR is written to out/ in the copy, and src/ includes it from
#     CARGO_MANIFEST_DIR/out instead.
#   - [package.metadata.nobuild] records the cfgs and the out/ files, for the CI check.
# Needs curl and sha256sum, or the crate already in cargo's download cache.
set -euo pipefail
die() { echo "vendor_nobuild: $*" >&2; exit 1; }
[ $# -eq 2 ] || die "usage: vendor_nobuild.sh CRATE VERSION"
crate=$1 version=$2
root=$(git rev-parse --show-toplevel)
dest=$root/vendor/$crate-$version
patch=${version#*.}; patch=${patch#*.}; patch=${patch%%[+-]*}

# The recipe: what the crate's build.rs emits on rustc 1.80 or later (every check-cfg branch
# taken), on a stable compiler without RUSTC_BOOTSTRAP. `cfg NAME PREDICATE` for a cfg it sets,
# `out FILE TEXT` for a file it writes. Read the new build.rs before changing a version here.
recipe() {
  case $crate-$version in
    proc-macro2-1.0.107)
      # Without `procmacro2_semver_exempt` in RUSTFLAGS: wrap the compiler's proc_macro, and use
      # Span::start/end and Span::file, whose probes compile on stable 1.88 and later.
      cfg wrap_proc_macro 'feature = "proc-macro"'
      cfg proc_macro_span_location 'feature = "proc-macro"'
      cfg proc_macro_span_file 'feature = "proc-macro"'
      cfg span_locations 'feature = "span-locations"'
      ;;
    quote-1.0.47) ;; # `no_diagnostic_namespace` only before 1.78.
    serde_core-1.0.229)
      out private.rs "#[doc(hidden)]
pub mod __private$patch {
    #[doc(hidden)]
    pub use crate::private::*;
}
"
      ;;
    serde-1.0.229)
      cfg if_docsrs_then_no_serde_core 'all()'
      out private.rs "#[doc(hidden)]
pub mod __private$patch {
    #[doc(hidden)]
    pub use crate::private::*;
}
use serde_core::__private$patch as serde_core_private;
"
      ;;
    *) die "no recipe for $crate $version: read its build.rs and add one" ;;
  esac
}

# The .crate file: cargo's download cache, else crates.io, checked against the index's SHA-256.
work=$(mktemp -d)
trap 'rm -rf -- "$work"' EXIT
file=$crate-$version.crate
cached=$(ls "${CARGO_HOME:-$HOME/.cargo}"/registry/cache/*/"$file" 2>/dev/null | head -n 1 || true)
case ${#crate} in
  1) index=1/$crate ;; 2) index=2/$crate ;; 3) index=3/${crate:0:1}/$crate ;;
  *) index=${crate:0:2}/${crate:2:2}/$crate ;;
esac
want=$(curl -fsSL "https://index.crates.io/$index" 2>/dev/null \
  | grep -F "\"vers\":\"$version\"" | sed -n 's/.*"cksum":"\([0-9a-f]\{64\}\)".*/\1/p' || true)
if [ -n "$cached" ]; then
  cp "$cached" "$work/$file"
else
  curl -fsSL -o "$work/$file" "https://static.crates.io/crates/$crate/$file" || die "can't download $file"
fi
got=$(sha256sum "$work/$file" | cut -d' ' -f1)
if [ -n "$want" ]; then
  [ "$got" = "$want" ] || die "$file has SHA-256 $got, the index lists $want"
elif [ -z "$cached" ]; then
  die "can't read the index's checksum for $crate $version"
fi
tar -xzf "$work/$file" -C "$work"
src=$work/$crate-$version
[ -f "$src/build.rs" ] || die "$crate $version has no build.rs; depend on it from crates.io"

rm -rf -- "$src/build.rs" "$src/Cargo.toml.orig" "$src/Cargo.lock" "$src/.cargo_vcs_info.json" \
  "$src/.cargo-ok" "$src/.github" "$src/.gitignore" "$src/rust-toolchain.toml"
grep -qx 'build = "build.rs"' "$src/Cargo.toml" || die "no \`build = \"build.rs\"\` line in the manifest"
grep -q '^\[lints' "$src/Cargo.toml" && die "the manifest has [lints] already; merge the check-cfgs by hand"
sed -i 's/^build = "build.rs"$/build = false/' "$src/Cargo.toml"

# The check-cfgs, from the build script's println! lines (escaped quotes unescaped).
mapfile -t checks < <(tar -xzOf "$work/$file" "$crate-$version/build.rs" \
  | sed -n 's/.*"cargo::\{0,1\}rustc-check-cfg=\(.*\)");.*/\1/p' | sed 's/\\"/"/g')

cfgs=() outs=()
cfg() { # NAME PREDICATE: the cfg's uses in src/, outside the build script's probes.
  local f n=0 before
  while IFS= read -r f; do
    before=$(sha256sum <"$f")
    awk -v name="$1" -v pred="$2" -f "$root/tools/vendor_nobuild_cfg.awk" "$f" >"$work/edit"
    cp "$work/edit" "$f"
    [ "$(sha256sum <"$f")" = "$before" ] || n=$((n + 1))
  done < <(grep -rlw --include='*.rs' --exclude-dir=probe -- "$1" "$src/src" || true)
  [ "$n" -gt 0 ] || die "the recipe's cfg $1 is in no cfg predicate in src/"
  cfgs+=("$1 = '$2'")
}
out() { # FILE TEXT
  local hits
  mkdir -p "$src/out"
  printf '%s' "$2" >"$src/out/$1"
  hits=$(grep -rlF --include='*.rs' -- "env!(\"OUT_DIR\"), \"/$1\"" "$src/src" || true)
  [ -n "$hits" ] || die "nothing in src/ includes OUT_DIR/$1"
  # shellcheck disable=SC2086
  sed -i "s#env!(\"OUT_DIR\"), \"/$1\"#env!(\"CARGO_MANIFEST_DIR\"), \"/out/$1\"#g" $hits
  outs+=("\"$1\"")
}
recipe
! grep -rqF --include='*.rs' 'OUT_DIR' "$src/src" || die "src/ still reads OUT_DIR; add an \`out\` to the recipe"

{
  printf '\n[lints.rust.unexpected_cfgs]\nlevel = "warn"\n'
  printf '# What the removed build script declared with cargo:rustc-check-cfg.\ncheck-cfg = [\n'
  for c in "${checks[@]}"; do printf "    '%s',\n" "$c"; done
  printf ']\n\n'
  printf '# Written by tools/vendor_nobuild.sh: the cfgs the removed build script sets, each with the\n'
  printf '# predicate it is set under, now written in src/; and the files it wrote to OUT_DIR, now in\n'
  printf '# out/. tools/vendor_nobuild_check.sh compares them with the real script in CI.\n'
  printf '[package.metadata.nobuild]\nout = [%s]\n\n[package.metadata.nobuild.rustc-cfg]\n' \
    "$(IFS=,; echo "${outs[*]:-}" | sed 's/,/, /g')"
  for c in "${cfgs[@]}"; do printf '%s\n' "$c"; done
} >>"$src/Cargo.toml"

rm -rf -- "$dest"
mkdir -p "$root/vendor"
mv "$src" "$dest"
echo "vendor_nobuild: $crate $version into vendor/$crate-$version (sha256 $got)"
