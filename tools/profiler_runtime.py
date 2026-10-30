#!/usr/bin/env python3
"""Build Rust's profiler runtime for a toolchain that does not ship one.

The pinned x86_64-pc-windows-gnu toolchain has no `profiler_builtins`, so
`-C instrument-coverage` fails with E0463. This compiles LLVM's profile runtime
from the exact rust-lang/llvm-project commit the compiler was built from (as
library/profiler_builtins/build.rs does) with Zig, wraps it in a
`profiler_builtins` rlib, and prints the directory to pass as
`-L DIR` (crate and dependency lookup) next to `-C instrument-coverage`.

Usage: profiler_runtime.py --rustc RUSTC --zig ZIG --out DIR
"""
import argparse, json, os, pathlib, subprocess, sys, urllib.request

# library/profiler_builtins/build.rs source list; target-specific files compile empty.
SOURCES = (
    "GCDAProfiling.c", "InstrProfiling.c", "InstrProfilingBuffer.c", "InstrProfilingFile.c",
    "InstrProfilingInternal.c", "InstrProfilingMerge.c", "InstrProfilingMergeFile.c",
    "InstrProfilingNameVar.c", "InstrProfilingPlatformAIX.c", "InstrProfilingPlatformDarwin.c",
    "InstrProfilingPlatformFuchsia.c", "InstrProfilingPlatformLinux.c", "InstrProfilingPlatformOther.c",
    "InstrProfilingPlatformWindows.c", "InstrProfilingRuntime.cpp", "InstrProfilingUtil.c",
    "InstrProfilingValue.c", "InstrProfilingVersionVar.c", "InstrProfilingWriter.c", "WindowsMMap.c",
)
CRATE = """#![allow(internal_features)]
#![feature(no_core)]
#![feature(profiler_runtime)]
#![no_core]
#![profiler_runtime]
"""
TARGETS = {"x86_64-pc-windows-gnu": "x86_64-windows-gnu", "x86_64-unknown-linux-gnu": "x86_64-linux-gnu"}


def run(*command, **kwargs):
    subprocess.run(command, check=True, **kwargs)


def llvm_commit(rust_commit):
    request = urllib.request.Request(
        f"https://api.github.com/repos/rust-lang/rust/contents/src/llvm-project?ref={rust_commit}",
        headers={"Accept": "application/vnd.github+json", "User-Agent": "sm-decomp-ci",
                 **({"Authorization": f"Bearer {os.environ['GITHUB_TOKEN']}"} if os.environ.get("GITHUB_TOKEN") else {})})
    with urllib.request.urlopen(request, timeout=60) as response:
        return json.load(response)["sha"]


def fetch(commit, directory):
    git = ["git", "-C", str(directory)]
    run("git", "init", "-q", str(directory))
    run(*git, "remote", "add", "origin", "https://github.com/rust-lang/llvm-project")
    run(*git, "sparse-checkout", "set", "--no-cone", "/compiler-rt/lib/profile/", "/compiler-rt/include/profile/",
        env=dict(os.environ, MSYS_NO_PATHCONV="1"))
    run(*git, "fetch", "-q", "--depth", "1", "--filter=blob:none", "origin", commit)
    run(*git, "checkout", "-q", "FETCH_HEAD")
    return directory / "compiler-rt"


def main():
    parser = argparse.ArgumentParser()
    parser.add_argument("--rustc", default="rustc")
    parser.add_argument("--zig", required=True)
    parser.add_argument("--out", type=pathlib.Path, required=True)
    args = parser.parse_args()
    info = dict(line.split(": ", 1) for line in subprocess.check_output(
        [args.rustc, "-vV"], text=True).splitlines() if ": " in line)
    host = info["host"]
    sysroot = pathlib.Path(subprocess.check_output([args.rustc, "--print", "sysroot"], text=True).strip())
    if any((sysroot / "lib/rustlib" / host / "lib").glob("libprofiler_builtins-*.rlib")):
        print(f"{host} ships profiler_builtins; nothing to build")
        return
    out = args.out.resolve()
    out.mkdir(parents=True, exist_ok=True)
    commit = llvm_commit(info["commit-hash"])
    print(f"rustc {info['release']} (LLVM {info['LLVM version']}): rust-lang/llvm-project {commit}", flush=True)
    root = fetch(commit, out / "llvm-project")
    target = TARGETS[host]
    objects = []
    for name in SOURCES:
        source = root / "lib/profile" / name
        obj = out / (name + ".o")
        cpp = name.endswith(".cpp")
        flags = ["-fno-exceptions", "-fno-rtti"] if cpp else []
        unix = [] if "windows" in target else ["-fvisibility=hidden", "-DCOMPILER_RT_HAS_UNAME=1", "-DCOMPILER_RT_HAS_FCNTL_LCK=1"]
        run(args.zig, "c++" if cpp else "cc", "-target", target, "-O2", "-fno-builtin", "-fomit-frame-pointer",
            "-DVISIBILITY_HIDDEN", "-DCOMPILER_RT_HAS_ATOMICS=1", *unix, *flags, "-w",
            "-I", str(root / "include"), "-c", str(source), "-o", str(obj))
        objects.append(str(obj))
    archive = out / "libprofiler-rt.a"
    archive.unlink(missing_ok=True)
    run(args.zig, "ar", "rcs", str(archive), *objects)
    crate = out / "profiler_builtins.rs"
    crate.write_text(CRATE)
    run(args.rustc, "--edition=2021", "--crate-type", "rlib", "--crate-name", "profiler_builtins",
        "-C", "opt-level=3", "-C", "metadata=sm-decomp", "-L", f"native={out}", "-l", "static=profiler-rt",
        str(crate), "-o", str(out / "libprofiler_builtins-sm_decomp.rlib"),
        env=dict(os.environ, RUSTC_BOOTSTRAP="1"))
    print(out)


if __name__ == "__main__":
    main()
