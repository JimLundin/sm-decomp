//! Declared compatibility build edits; no recording-core semantic changes.
use anyhow::{Result, ensure};
use std::{fs, path::Path, process::Command};
fn write(path: &Path, text: &str) -> Result<()> {
    fs::write(path, text)?;
    Ok(())
}
pub fn configure(source: &Path, original: &Path, libraries: &Path) -> Result<()> {
    let libraries = fs::canonicalize(libraries)?;
    for (relative, header) in [
        ("include/library/httpreq.hpp", "functional"),
        ("include/library/loadlib.hpp", "functional"),
        ("bsnes/snes/snes.hpp", "vector"),
    ] {
        let path = source.join(relative);
        let text = fs::read_to_string(&path)?;
        let include = format!("#include <{header}>\n");
        if !text.contains(&include) {
            write(&path, &format!("{include}{text}"))?;
        }
    }
    let path = source.join("bsnes/Makefile");
    write(&path, &fs::read_to_string(&path)?.replace("-std=gnu++0x", "-std=gnu++14"))?;
    let path = source.join("Makefile");
    let text = fs::read_to_string(&path)?;
    if !text.contains("\nheadless:") {
        write(
            &path,
            &format!(
                "{text}\n.PHONY: headless\nheadless: src/core/version.cpp buildaux/mkdeps$(DOT_EXECUTABLE_SUFFIX) buildaux/txt2cstr$(DOT_EXECUTABLE_SUFFIX)\n\t$(MAKE) -C src precheck\n\t$(MAKE) -C src util/lsnes-dumpavi.util\n"
            ),
        )?;
    }
    let include = libraries.join("usr/include");
    let lib = libraries.join("usr/lib/x86_64-linux-gnu");
    let linker = format!("-L{} -Wl,-rpath,{}", lib.display(), lib.display());
    let mut settings = std::collections::BTreeMap::from([
        ("CC", "g++".to_owned()),
        ("LD", "ld".into()),
        ("RANLIB", "ranlib".into()),
        ("CROSS_PREFIX", "".into()),
        ("HOSTCC", "$(CC) $(USER_HOSTCCFLAGS) -DNATIVE_THREADS".into()),
        ("USER_HOSTCCFLAGS", format!("-I{} {linker}", include.display())),
        (
            "USER_CFLAGS",
            format!(
                "-std=gnu++14 -O2 -g -Wno-deprecated -I{} -I{}/lua5.2 -I{}/x86_64-linux-gnu",
                include.display(),
                include.display(),
                include.display()
            ),
        ),
        ("USER_LDFLAGS", linker),
        ("LUA", "lua5.2".into()),
        ("REGEX", "NATIVE".into()),
        ("THREADS", "NATIVE".into()),
        ("GRAPHICS", "DUMMY".into()),
        ("SOUND", "DUMMY".into()),
        ("JOYSTICK", "DUMMY".into()),
        ("BUILD_BSNES", "yes".into()),
        ("BSNES_IS_COMPAT", "yes".into()),
    ]);
    for key in [
        "BUILD_GAMBATTE",
        "SECRET_RABBIT_CODE",
        "OPUS_CODEC",
        "OPUS_CODEC_SUPPORTS_SURROUND",
        "USE_LIBGCRYPT",
        "USE_LIBLZMA",
        "BOOST_NEEDS_MT",
        "HOST_BOOST_NEEDS_MT",
    ] {
        settings.insert(key, String::new());
    }
    let output = Command::new("tar").arg("-xOf").arg(original).arg("options.build").output()?;
    ensure!(output.status.success(), "cannot read official build options");
    let mut options = String::from_utf8(output.stdout)?;
    for (key, value) in settings {
        let prefix = format!("{key}=");
        ensure!(
            options.lines().filter(|line| line.starts_with(&prefix)).count() == 1,
            "unexpected original build setting: {key}"
        );
        options = options
            .split_inclusive('\n')
            .map(|line| {
                if line.starts_with(&prefix) {
                    format!("{prefix}{value}{}", if line.ends_with('\n') { "\n" } else { "" })
                } else {
                    line.to_owned()
                }
            })
            .collect();
    }
    write(&source.join("options.build"), &options)
}
