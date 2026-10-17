//! Bounded discovery of the installed compiler and linker confinement inputs.
use crate::confinement::{self, ReadOnlyBind};
use anyhow::{Context, Result, ensure};
use std::{fs, path::Path};
use tool_execution::bounded;

/// Read-only files needed by the host's installed compiler/linker. This exposes
/// neither a host checkout nor a controller executable. Runtime imports are
/// frozen by the durable operation runner before executing rewritten Rust.
pub fn discover(rustc: &Path) -> Result<Vec<ReadOnlyBind>> {
    bounded::limited(|| discover_bounded(rustc))
}

pub fn discover_until(rustc: &Path, deadline: u64, max_bytes: u64) -> Result<Vec<ReadOnlyBind>> {
    bounded::until(deadline, max_bytes, || discover_bounded(rustc))
}

fn discover_bounded(rustc: &Path) -> Result<Vec<ReadOnlyBind>> {
    let rustc = fs::canonicalize(rustc)?;
    let toolchain = rustc.parent().and_then(Path::parent).context("compiler toolchain root missing")?;
    ensure!(toolchain.join("bin/rustc") == rustc && toolchain.join("lib/rustlib").is_dir(), "installed rustc required");
    let mut bindings = confinement::helper_runtime()?;
    for (source, destination) in [
        ("/usr/bin/x86_64-linux-gnu-gcc-9", "/usr/bin/cc"),
        ("/usr/bin/x86_64-linux-gnu-as", "/usr/bin/as"),
        ("/usr/bin/x86_64-linux-gnu-ld.bfd", "/usr/bin/ld"),
        ("/usr/lib/gcc/x86_64-linux-gnu/9/cc1", "/usr/lib/gcc/x86_64-linux-gnu/9/cc1"),
        ("/usr/lib/gcc/x86_64-linux-gnu/9/collect2", "/usr/lib/gcc/x86_64-linux-gnu/9/collect2"),
        ("/usr/lib/gcc/x86_64-linux-gnu/9/include", "/usr/lib/gcc/x86_64-linux-gnu/9/include"),
        ("/usr/include", "/usr/include"),
    ] {
        bounded::check()?;
        bindings.push(ReadOnlyBind { source: fs::canonicalize(source)?, destination: destination.into() });
    }
    // GCC's non-LTO link closure; enumerate files so unrelated installed shared
    // libraries, system programs and package caches are inaccessible.
    for directory in ["/usr/lib/gcc/x86_64-linux-gnu/9", "/usr/lib/x86_64-linux-gnu"] {
        for path in bounded::entries(Path::new(directory))? {
            let name = path.file_name().context("runtime filename missing")?.to_string_lossy();
            if (name.starts_with("crt") && name.ends_with(".o"))
                || ["Scrt1.o", "rcrt1.o"].contains(&name.as_ref())
                || [
                    "libgcc.a",
                    "libgcc_eh.a",
                    "libgcc_s.so",
                    "liblto_plugin.so",
                    "libc.so",
                    "libc.a",
                    "libc_nonshared.a",
                    "libm.so",
                    "libm.a",
                    "libmvec.so",
                    "libmvec.a",
                    "libpthread.so",
                    "libpthread.a",
                    "libpthread_nonshared.a",
                    "libdl.so",
                    "libdl.a",
                    "librt.so",
                    "librt.a",
                    "libutil.so",
                    "libutil.a",
                ]
                .contains(&name.as_ref())
            {
                bindings.push(ReadOnlyBind { source: fs::canonicalize(&path)?, destination: path });
            }
        }
    }
    // Dynamic linker/search aliases referenced by libc and gcc's link scripts.
    for path in [
        "/usr/lib/x86_64-linux-gnu/libgcc_s.so.1",
        "/lib/x86_64-linux-gnu/libc.so.6",
        "/lib/x86_64-linux-gnu/libm.so.6",
        "/lib/x86_64-linux-gnu/libmvec.so.1",
        "/lib/x86_64-linux-gnu/libpthread.so.0",
        "/lib/x86_64-linux-gnu/libdl.so.2",
        "/lib/x86_64-linux-gnu/librt.so.1",
        "/lib/x86_64-linux-gnu/libutil.so.1",
        "/lib/x86_64-linux-gnu/libz.so.1",
        "/lib/x86_64-linux-gnu/libmpfr.so.6",
        "/lib/x86_64-linux-gnu/libmpc.so.3",
        "/lib/x86_64-linux-gnu/libgmp.so.10",
        "/lib/x86_64-linux-gnu/libisl.so.22",
        "/lib/x86_64-linux-gnu/libexpat.so.1",
        "/lib/x86_64-linux-gnu/libzstd.so.1",
        "/lib/x86_64-linux-gnu/libstdc++.so.6",
        "/lib/x86_64-linux-gnu/libtinfo.so.6",
        "/lib/x86_64-linux-gnu/libxml2.so.2",
        "/lib/x86_64-linux-gnu/liblzma.so.5",
    ] {
        bounded::check()?;
        if Path::new(path).exists() && !bindings.iter().any(|binding| binding.destination == Path::new(path)) {
            bindings.push(ReadOnlyBind { source: fs::canonicalize(path)?, destination: path.into() });
        }
    }
    for path in [
        "/lib/x86_64-linux-gnu/libopcodes-2.34-system.so",
        "/lib/x86_64-linux-gnu/libbfd-2.34-system.so",
        "/lib/x86_64-linux-gnu/libctf.so.0",
    ] {
        bounded::check()?;
        bindings.push(ReadOnlyBind { source: fs::canonicalize(path)?, destination: path.into() });
    }
    bindings.push(ReadOnlyBind {
        source: fs::canonicalize("/lib/x86_64-linux-gnu/ld-linux-x86-64.so.2")?,
        destination: "/lib/x86_64-linux-gnu/ld-linux-x86-64.so.2".into(),
    });
    bindings.push(ReadOnlyBind { source: toolchain.join("lib"), destination: "/toolchain/lib".into() });
    ensure!(bindings.len() <= 1024, "installed compiler runtime binding inventory exceeded");
    bounded::count(bindings.len())?;
    Ok(bindings)
}

pub const IMPLEMENTATION_SOURCES: &[(&str, &str)] = &[
    ("crates/translation-execution/Cargo.toml", include_str!("../Cargo.toml")),
    ("crates/translation-execution/src/lib.rs", include_str!("lib.rs")),
    ("crates/translation-execution/src/compiler_runtime.rs", include_str!("compiler_runtime.rs")),
];
