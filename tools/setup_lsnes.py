#!/usr/bin/env python3
"""Build the pinned TAS recording core locally on Ubuntu 20.04 x86_64."""

import argparse
import ctypes
import ctypes.util
import json
import os
import platform
import re
import shutil
import subprocess
import sys
import tarfile
from pathlib import Path

from setup import ROOT, digest


def run(*command, **kwargs):
    subprocess.run([str(arg) for arg in command], check=True, **kwargs)


def fetch(record, downloads, offline):
    archive = downloads / record["filename"]
    if not archive.exists():
        if offline:
            raise RuntimeError("missing cached download: {}".format(archive))
        partial = archive.with_suffix(archive.suffix + ".part")
        run("curl", "-L", "--fail", "--silent", "--show-error", "--retry", "3",
            "--connect-timeout", "30", record["url"], "-o", partial)
        if digest(partial) != record["sha256"]:
            raise RuntimeError("checksum mismatch: {}".format(partial))
        partial.replace(archive)
    if digest(archive) != record["sha256"]:
        raise RuntimeError("checksum mismatch: {}".format(archive))
    return archive


def extract_source_tar(archive, destination):
    """Read only sources.tar from the official 7z using the host libarchive."""
    library = ctypes.util.find_library("archive")
    if not library:
        raise RuntimeError("libarchive is required to read the official 7z")
    lib = ctypes.CDLL(library)
    pointer = ctypes.c_void_p
    lib.archive_read_new.restype = pointer
    for name in ("archive_read_support_format_7zip", "archive_read_support_filter_all", "archive_read_free"):
        getattr(lib, name).argtypes = [pointer]
    lib.archive_read_open_filename.argtypes = [pointer, ctypes.c_char_p, ctypes.c_size_t]
    lib.archive_read_next_header.argtypes = [pointer, ctypes.POINTER(pointer)]
    lib.archive_entry_pathname.argtypes = [pointer]
    lib.archive_entry_pathname.restype = ctypes.c_char_p
    lib.archive_read_data.argtypes = [pointer, pointer, ctypes.c_size_t]
    lib.archive_read_data.restype = ctypes.c_ssize_t
    lib.archive_error_string.argtypes = [pointer]
    lib.archive_error_string.restype = ctypes.c_char_p
    reader = lib.archive_read_new()
    if not reader:
        raise RuntimeError("could not allocate libarchive reader")

    def checked(status):
        if status < 0:
            message = lib.archive_error_string(reader)
            raise RuntimeError("libarchive: " + (message.decode() if message else "read failed"))
        return status

    try:
        checked(lib.archive_read_support_filter_all(reader))
        checked(lib.archive_read_support_format_7zip(reader))
        checked(lib.archive_read_open_filename(reader, os.fsencode(archive), 65536))
        entry = pointer()
        while checked(lib.archive_read_next_header(reader, ctypes.byref(entry))) != 1:
            if Path(os.fsdecode(lib.archive_entry_pathname(entry))).name != "sources.tar":
                continue
            buffer = ctypes.create_string_buffer(65536)
            with destination.open("wb") as out:
                while True:
                    size = checked(lib.archive_read_data(reader, buffer, len(buffer)))
                    if size == 0:
                        return
                    out.write(buffer.raw[:size])
        raise RuntimeError("official archive contains no sources.tar")
    finally:
        lib.archive_read_free(reader)


def write_if_changed(path, contents):
    if not path.exists() or path.read_text() != contents:
        path.write_text(contents)


def configure(source, original_tar, libraries):
    # Changes are build compatibility only; no emulator logic is patched.
    for relative, header in (("include/library/httpreq.hpp", "functional"),
                             ("include/library/loadlib.hpp", "functional"),
                             ("bsnes/snes/snes.hpp", "vector")):
        path = source / relative
        include = "#include <{}>\n".format(header)
        if include not in path.read_text():
            write_if_changed(path, include + path.read_text())
    path = source / "bsnes/Makefile"
    write_if_changed(path, path.read_text().replace("-std=gnu++0x", "-std=gnu++14"))
    path = source / "Makefile"
    if "\nheadless:" not in path.read_text():
        write_if_changed(path, path.read_text() + """
.PHONY: headless
headless: src/core/version.cpp buildaux/mkdeps$(DOT_EXECUTABLE_SUFFIX) buildaux/txt2cstr$(DOT_EXECUTABLE_SUFFIX)
	$(MAKE) -C src precheck
	$(MAKE) -C src util/lsnes-dumpavi.util
""")
    include = libraries / "usr/include"
    lib = libraries / "usr/lib/x86_64-linux-gnu"
    linker = "-L{} -Wl,-rpath,{}".format(lib, lib)
    settings = {
        "CC": "g++", "LD": "ld", "RANLIB": "ranlib", "CROSS_PREFIX": "",
        "HOSTCC": "$(CC) $(USER_HOSTCCFLAGS) -DNATIVE_THREADS",
        "USER_HOSTCCFLAGS": "-I{} {}".format(include, linker),
        "USER_CFLAGS": "-std=gnu++14 -O2 -g -Wno-deprecated -I{} -I{}/lua5.2 -I{}/x86_64-linux-gnu".format(include, include, include),
        "USER_LDFLAGS": linker,
        "LUA": "lua5.2", "REGEX": "NATIVE", "THREADS": "NATIVE",
        "GRAPHICS": "DUMMY", "SOUND": "DUMMY", "JOYSTICK": "DUMMY",
        "BUILD_BSNES": "yes", "BSNES_IS_COMPAT": "yes",
    }
    for key in ("BUILD_GAMBATTE", "SECRET_RABBIT_CODE", "OPUS_CODEC", "OPUS_CODEC_SUPPORTS_SURROUND",
                "USE_LIBGCRYPT", "USE_LIBLZMA", "BOOST_NEEDS_MT", "HOST_BOOST_NEEDS_MT"):
        settings[key] = ""
    with tarfile.open(original_tar) as archive:
        options = archive.extractfile("options.build").read().decode()
    for key, value in settings.items():
        options, count = re.subn(r"^" + key + r"=.*$", lambda _: key + "=" + value,
                                 options, flags=re.MULTILINE)
        if count != 1:
            raise RuntimeError("unexpected original build setting: " + key)
    write_if_changed(source / "options.build", options)


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--offline", action="store_true")
    parser.add_argument("--jobs", type=int, default=min(os.cpu_count() or 1, 4))
    args = parser.parse_args()
    if (platform.system(), platform.machine()) != ("Linux", "x86_64"):
        parser.error("these private development packages require Linux x86_64")
    if args.jobs < 1:
        parser.error("--jobs must be positive")
    deps = ROOT / "deps"
    # Absolute paths are embedded in legacy Make flags.
    if re.search(r"[\s$#]", str(deps)):
        parser.error("legacy build requires a repository path without whitespace, $ or #")
    required = ("g++", "make", "pkg-config", "curl", "tar", "dpkg-deb")
    missing = [tool for tool in required if not shutil.which(tool)]
    if missing:
        raise RuntimeError("install host build tools first: " + ", ".join(missing))
    if not Path("/usr/include/zlib.h").exists():
        raise RuntimeError("host zlib development headers are required")
    curl_runtime = Path("/usr/lib/x86_64-linux-gnu/libcurl.so.4")
    if not curl_runtime.exists():
        raise RuntimeError("host libcurl OpenSSL runtime is required")
    manifest = json.loads((ROOT / "docs/lsnes-dependencies.json").read_text())
    downloads = deps / "downloads"
    downloads.mkdir(parents=True, exist_ok=True)
    official = fetch(manifest["archive"], downloads, args.offline)
    local = deps / "lsnes"
    local.mkdir(parents=True, exist_ok=True)
    original_tar = local / "sources.tar"
    if not original_tar.exists():
        extract_source_tar(official, original_tar)
    if digest(original_tar) != manifest["archive"]["source_tar_sha256"]:
        raise RuntimeError("original source tar checksum mismatch")
    source = local / "source"
    if not (source / "VERSION").exists():
        source.mkdir(parents=True, exist_ok=True)
        run("tar", "-xf", original_tar, "-C", source)
    libraries = deps / "lsnes-build-deps"
    libraries.mkdir(parents=True, exist_ok=True)
    for record in manifest["packages"]:
        archive = fetch(record, downloads, args.offline)
        stamp = libraries / (".setup-" + record["filename"])
        if not stamp.exists() or stamp.read_text().strip() != record["sha256"]:
            run("dpkg-deb", "-x", archive, libraries)
            stamp.write_text(record["sha256"] + "\n")
    for alias, target in ((libraries / "usr/bin/lua", "lua5.2"),
                          (libraries / "usr/bin/luac", "luac5.2"),
                          (libraries / "usr/lib/x86_64-linux-gnu/libcurl.so", str(curl_runtime))):
        if alias.is_symlink():
            if os.readlink(alias) == target:
                continue
            alias.unlink()
        elif alias.exists():
            raise RuntimeError("unexpected file at private symlink: {}".format(alias))
        alias.symlink_to(target)
    configure(source, original_tar, libraries)
    env = os.environ.copy()
    env["PATH"] = str(libraries / "usr/bin") + os.pathsep + env.get("PATH", "")
    env["PKG_CONFIG_PATH"] = str(libraries / "usr/lib/x86_64-linux-gnu/pkgconfig")
    log = local / "build.log"
    print("Building lsnes rr2-beta23 / bsnes v085 Compatibility; log: {}".format(log), flush=True)
    with log.open("w") as out:
        run("make", "-j{}".format(args.jobs), "headless", cwd=source, env=env,
            stdout=out, stderr=subprocess.STDOUT)
    print("Ready. Run `just sync-smoke` or `just sync-check`.")


if __name__ == "__main__":
    try:
        main()
    except (RuntimeError, OSError, subprocess.CalledProcessError) as error:
        print("setup-lsnes: {}".format(error), file=sys.stderr)
        sys.exit(1)
