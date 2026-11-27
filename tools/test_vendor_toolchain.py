"""Exercise the vendor verifier with toolchain proxies and invented build outputs.

The shell runs the production verifier against mock proxies. CI compares the published build scripts.
"""

import os
from pathlib import Path
import shutil
import subprocess
import tempfile
import unittest


SCRIPT = Path(__file__).with_name("vendor_nobuild_check.sh").resolve()
HARNESS = r'''
cd "$FIXTURE_ROOT" || exit 2
FIXTURE_ROOT=$PWD
git() {
  case "$1" in
    rev-parse) printf "%s\n" "$FIXTURE_ROOT" ;;
    diff|ls-files) return 0 ;;
    *) return 2 ;;
  esac
}
rustup() {
  [ "${RESOLUTION_FAIL:-0}" = 0 ] || return 7
  case "$*" in
    "show active-toolchain")
      local selected=${RUSTUP_TOOLCHAIN:-}
      if [ -z "$selected" ]; then
        if [ "$PWD" = "$FIXTURE_ROOT" ]; then
          selected=repository-toolchain
        else
          selected=default-toolchain
        fi
      fi
      printf "%s (active)\n" "$selected" ;;
    "which cargo"|"which rustc")
      printf "/toolchains/%s/%s\n" "${RUSTUP_TOOLCHAIN:-default-toolchain}" "$2" ;;
    *) return 2 ;;
  esac
}
cargo() { proxy cargo "$@"; }
custom_cargo() { proxy custom_cargo "$@"; }
proxy() {
  local invoked=$1
  shift
  case "$1" in
    tree)
      printf "quote v1.0.47|\n" ;;
    check)
      local selected=${RUSTUP_TOOLCHAIN:-default-toolchain}
      printf "%s|%s|%s|%s\n" "$invoked" "$selected" "$(rustup which cargo)" "$(rustup which rustc)" >> "$EVIDENCE"
      local build="$CARGO_TARGET_DIR/debug/build/quote-probe"
      if [ "$selected" = default-toolchain ]; then
        build="$CARGO_TARGET_DIR/new-layout/quote-probe"
      fi
      mkdir -p "$build/out"
      : > "$build/output" ;;
    *) return 2 ;;
  esac
}
# The vendor transformation has no role in toolchain selection.
bash() { [ "$1" = tools/vendor_nobuild.sh ]; }
source "$VERIFIER"
'''


class ToolchainSelectionTests(unittest.TestCase):
    def run_verifier(self, *, toolchain=None, cargo=None, resolution_fail=False):
        shell = os.environ.get("SM_BASH") or shutil.which("bash")
        self.assertIsNotNone(shell)
        with tempfile.TemporaryDirectory(prefix="vendor-toolchain-") as temporary:
            root = Path(temporary)
            vendor = root / "vendor" / "quote-1.0.47"
            vendor.mkdir(parents=True)
            (vendor / "Cargo.toml").write_text(
                'name = "quote"\nversion = "1.0.47"\n'
                '[package.metadata.nobuild]\nout = []\n',
                encoding="utf-8", newline="\n",
            )
            evidence = root / "evidence"
            environment = os.environ.copy()
            environment.pop("RUSTUP_TOOLCHAIN", None)
            environment.pop("CARGO", None)
            environment.update(FIXTURE_ROOT=root.as_posix(), VERIFIER=SCRIPT.as_posix(),
                               EVIDENCE=evidence.as_posix())
            if toolchain is not None:
                environment["RUSTUP_TOOLCHAIN"] = toolchain
            if cargo is not None:
                environment["CARGO"] = cargo
            if resolution_fail:
                environment["RESOLUTION_FAIL"] = "1"
            result = subprocess.run([shell, "-c", HARNESS], env=environment,
                                    capture_output=True, text=True, timeout=30)
            rows = evidence.read_text(encoding="utf-8").splitlines() if evidence.exists() else []
            return result, rows

    def assert_selection(self, toolchain, cargo=None):
        result, rows = self.run_verifier(toolchain=toolchain, cargo=cargo)
        self.assertEqual(result.returncode, 0, result.stdout + result.stderr)
        selected = toolchain or "repository-toolchain"
        invoked = cargo or "cargo"
        self.assertEqual(rows, [f"{invoked}|{selected}|/toolchains/{selected}/cargo|"
                                f"/toolchains/{selected}/rustc"] * 3)

    def test_repository_toolchain_survives_scratch_directory(self):
        self.assert_selection(None)

    def test_cargo_override_keeps_repository_toolchain(self):
        self.assert_selection(None, "custom_cargo")

    def test_explicit_toolchain_and_cargo_overrides_are_preserved(self):
        self.assert_selection("caller-toolchain", "custom_cargo")

    def test_resolution_failure_stops_before_building(self):
        result, rows = self.run_verifier(resolution_fail=True)
        self.assertNotEqual(result.returncode, 0)
        self.assertEqual(rows, [])


if __name__ == "__main__":
    unittest.main()
