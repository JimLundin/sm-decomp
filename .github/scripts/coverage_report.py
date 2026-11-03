#!/usr/bin/env python3
"""Merge a ci.yml coverage run's profiles and write coverage.md."""
import os, pathlib, subprocess, sys

toolchain = os.environ["RUST_VERSION"]
subprocess.run(["rustup", "toolchain", "install", toolchain, "--profile", "minimal", "--component", "llvm-tools"], check=True)
sysroot = subprocess.check_output(["rustup", "run", toolchain, "rustc", "--print", "sysroot"], text=True).strip()
llvm = pathlib.Path(sysroot, "lib/rustlib/x86_64-unknown-linux-gnu/bin")


def merge(paths: list[pathlib.Path], output: str) -> None:
    listing = pathlib.Path(output + ".txt")
    listing.write_text("".join(f"{path}\n" for path in paths))
    subprocess.run([str(llvm / "llvm-profdata"), "merge", "-sparse", "-f", str(listing), "-o", output], check=True)


root = pathlib.Path("all-profiles")
raw = [path for path in root.glob("profiles-*/**/*.profraw") if not path.parts[1].startswith("profiles-route-")]
route = list(root.glob("profiles-route-*/**/*.profraw"))
if not raw:
    raise SystemExit("No coverage profiles were produced")
print(f"Merging {len(raw)} corpus and {len(route)} route profiles")
merge(raw, "corpus.profdata")
arguments: list[str] = []
if route:
    merge(route, "route.profdata")
    arguments.append("route.profdata")
unit_root = pathlib.Path("target/coverage-unit")
unit = list((unit_root / "profiles").glob("**/*.profraw"))
if unit:
    merge(unit, "unit.profdata")
    subprocess.run([str(llvm / "llvm-profdata"), "merge", "-sparse", "corpus.profdata", "unit.profdata",
                    "-o", "combined.profdata"], check=True)
    arguments += ["--unit", "unit.profdata", "--combined", "combined.profdata"]
    for name in (unit_root / "objects.txt").read_text().split():
        arguments += ["--unit-object", str(unit_root / "bin" / name)]
else:
    print("::warning::No unit-test coverage profiles were produced; the report omits the unit tests")
verifier = "target/verify-host/sm_verify"
if not pathlib.Path(verifier).is_file():
    raise SystemExit("expected headless verifier executable is missing")
subprocess.run([sys.executable, "-B", "tools/coverage_summary.py", str(llvm / "llvm-cov"), "corpus.profdata",
                verifier, "coverage.md", *arguments], check=True)
with open(os.environ["GITHUB_STEP_SUMMARY"], "a") as out:
    out.write("".join(pathlib.Path("coverage.md").read_text().splitlines(True)[:80]))
