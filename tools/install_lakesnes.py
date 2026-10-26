#!/usr/bin/env python3
"""Install pinned LakeSnes CPU, backend or SPC archives using Zig on Windows/Linux."""
import argparse
import hashlib
import os
from pathlib import Path
import subprocess
import urllib.request

SOURCES = {
    "cpu": {
        "repository": "dinkc64",
        "revision": "048a0d72568668f74e0cdf371dcd9e8efd614965",
        "files": {
            "snes/cpu.c": "45200e56488392a25f2664caedb4edb7782ac78197a28972ccf5612b0088a470",
            "snes/cpu.h": "4dcce50ce511cf44e115c9384bcb5cf0e057d2158581b69f490c12a613941b19",
            "snes/statehandler.c": "2bf94ba80e0c7ca8363ab2a1f7d3e7ecb076278c9339990a73141641b772f1c0",
            "snes/statehandler.h": "6e1b3affd60dd242bfa2dd5b25c67edfb8663d28f6c2b89e8db1a15b7b7d7249",
            "LICENSE.txt": "27568d73a4ce17ced1bf37e86bfee6fcb9b0f6a27de517ddbc16c946c5c509fd"
        }
    },
    "backend": {
        "repository": "angelo-wf",
        "revision": "28b8fbc15ad15afff003165ff4f2a8adb1c93d18",
        "files": {
            "snes/ppu.c": "1713d72737ce85996da96359fe141f472845a5b374408c30b866a724cb25824a",
            "snes/ppu.h": "bc7a46cafd27876fb196eced73479af796a3699df6469ac13985b7e4066d63d0",
            "snes/snes.h": "47f15f42667bb749263022244575b053f116bc75f4d45b9fd32d46e00466795e",
            "snes/dsp.c": "a2710134d7b648ce26e720af8ff56347cab537ddc4ce121894b761624db2fb0b",
            "snes/dsp.h": "9b4b0c6e554e5bf8dfde03be0da90f691270f9bfdeb9c32ca414a54345add6ba",
            "snes/apu.h": "1e63389e7e308fff401fe0c62b15623ac5aba4ca983669a67d728ab2192f4fc0",
            "LICENSE.txt": "1e88db7a44bec196f647c7b0e281517600797f4db5f8460bb02510c5219b257c"
        }
    },
    "spc": {
        "repository": "elzo-d",
        "revision": "1da16a22db4d0f2e7f6960a9846ef4e1f3acbd29",
        "files": {
            "snes/spc.c": "fc9dfd730114d5c42c7cbde884091ff53e2e64b4237861f48d56b83d9ff123b7",
            "snes/apu.h": "1e63389e7e308fff401fe0c62b15623ac5aba4ca983669a67d728ab2192f4fc0",
            "snes/cart.h": "9723504d49ed9c6b9dc6e36068e0302c7428692676383159c3ceafaedf72a75e",
            "snes/cpu.h": "6ee238c3272ee781ccbf92ddecbe8d857548665ab15d06852f16048ab47ddea1",
            "snes/dma.h": "b70680d974f0776d2aaa14fc5c96ebc55a9cae6a051838ad6db04e90c5766e5d",
            "snes/dsp.h": "9b4b0c6e554e5bf8dfde03be0da90f691270f9bfdeb9c32ca414a54345add6ba",
            "snes/input.h": "b5f70fdd1817f4f2a62095a647b6b96b1d9cbb6b19d48cf68c3ada2871d9103b",
            "snes/ppu.h": "bc7a46cafd27876fb196eced73479af796a3699df6469ac13985b7e4066d63d0",
            "snes/snes.h": "d23d09ba03d058e35aa5ce4947855a39ad36b80fd8b1ef537c51c3a14f4ea81f",
            "snes/spc.h": "fe42e827ea5447fc53ef7175b6bfd0471f4fa55e2bdbba15d857ad52629d4f2b",
            "LICENSE.txt": "1e88db7a44bec196f647c7b0e281517600797f4db5f8460bb02510c5219b257c"
        }
    }
}


def default_prefix(component):
    if os.name == "nt":
        cache = Path(os.environ["LOCALAPPDATA"])
    else:
        cache = Path(os.environ.get("XDG_CACHE_HOME", Path.home() / ".cache"))
    return cache / "sm-decomp-tools" / f"lakesnes-{component}-{SOURCES[component]['revision'][:7]}"


def install(component, zig, target, prefix, library):
    spec = SOURCES[component]
    source = prefix / "source"
    library.mkdir(parents=True, exist_ok=True)
    for name, expected in spec["files"].items():
        # Preserve existing cache layouts, including the CPU/SPC license locations.
        path = (prefix if name == "LICENSE.txt" and component != "backend" else source) / name
        path.parent.mkdir(parents=True, exist_ok=True)
        if not path.exists():
            url = f"https://raw.githubusercontent.com/{spec['repository']}/LakeSnes/{spec['revision']}/{name}"
            with urllib.request.urlopen(url) as response:
                path.write_bytes(response.read())
        data = path.read_bytes()
        # Only backend accepts CRLF caches; never rewrite compiler input bytes.
        if component == "backend":
            data = data.replace(b"\r\n", b"\n")
        if hashlib.sha256(data).hexdigest() != expected:
            raise ValueError(f"Upstream source checksum mismatch: {name}")

    flags = ["-target", target, "-O2", "-std=c11"]
    if target == "x86_64-linux-gnu":
        flags += ["-fPIC", "-D_DEFAULT_SOURCE"]
    if component == "backend":
        bridge = Path(__file__).resolve().parents[1] / "sm/src/snes/backend.c"
        stamp = library / "backend_source.h"
        digest = hashlib.sha256(bridge.read_bytes()).hexdigest()
        # Match Set-Content's ASCII header bytes on each existing host.
        stamp.write_bytes(f'#define SM_BACKEND_SOURCE_HASH "{digest}"{os.linesep}'.encode("ascii"))
        flags += ["-fexceptions", "-ffunction-sections", "-fdata-sections",
                  "-include", str(stamp), "-I", str(source / "snes")]
        units = [(bridge, "backend.o")]
    elif component == "cpu":
        units = [(source / "snes" / f"{name}.c", f"{name}.o") for name in ("cpu", "statehandler")]
    else:
        units = [(source / "snes/spc.c", "lakesnes_spc.o")]
    objects = []
    for path, name in units:
        obj = library / name
        subprocess.run([zig, "cc", *flags, "-c", str(path), "-o", str(obj)], check=True)
        objects.append(str(obj))
    subprocess.run([zig, "ar", "rcs", str(library / f"liblakesnes_{component}.a"), *objects], check=True)
    print(f"Installed pinned LakeSnes {component} library in {library}")
    print("Add the library directory to LIBRARY_PATH for the normal Cargo linker.")


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("component", choices=SOURCES)
    parser.add_argument("--zig", required=True)
    parser.add_argument("--target", required=True, choices=("x86_64-windows-gnu", "x86_64-linux-gnu"))
    parser.add_argument("--prefix", type=Path)
    parser.add_argument("--library-directory", type=Path)
    args = parser.parse_args()
    if args.component == "spc" and args.library_directory is None:
        parser.error("spc requires --library-directory")
    prefix = (args.prefix or default_prefix(args.component)).resolve()
    library = (args.library_directory or prefix / "lib").resolve()
    install(args.component, args.zig, args.target, prefix, library)


if __name__ == "__main__":
    main()
