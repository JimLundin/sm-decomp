"""Install the pinned configuration/input dependency with native SM compatibility."""
import argparse
import hashlib
import os
from pathlib import Path
import re
import subprocess
import urllib.request

REVISION = "45a149d8b4bb8098f53d1009f76e76e7d8b79bbf"
FILES = {
    "src/config.c": "d7c4f90eb30050919c329b1aba21f07c122ffa5220edeb059c3d9e3e7364b111",
    "src/util.c": "60e519bd41049f1c09e158471021d4449cb418b560b6318e510b47cace1a049f",
    "src/config.h": "f428417ba93dd8df89fa0b1ecce7f15be1c14d50efaa9341fd6bac863b5dfc94",
    "src/types.h": "af3bbac047a9a6db0707f9241aa9e19767147ab3783b523cb22cfd55b0409e23",
    "LICENSE.txt": "78b243fa858f4bcb49f8ccca14fec262f62fcee4e44a1ce7565840bdb4ca2217",
}
NATIVE_INPUTS = ("sm/src/config.rs", "sm/src/native_config.c", "sm/src/native_config.h",
                 "sm/src/native_util.h", "sm/src/types.rs", "tools/install_native_config.py")
UTILITY_INPUTS = ("sm/src/util.rs", "sm/src/native_util.c", "sm/src/native_util.h",
                  "sm/src/types.rs", "tools/install_native_config.py")
SYMBOLS = {
    "g_config": "sm_config_state", "ParseConfigFile": "sm_config_parse_file",
    "FindCmdForSdlKey": "sm_config_find_key", "FindCmdForGamepadButton": "sm_config_find_button",
    "ParseBool": "sm_config_parse_bool", "ReadWholeFile": "sm_util_ReadWholeFile",
    "NextLineStripComments": "sm_util_NextLineStripComments", "NextDelim": "sm_util_NextDelim",
    "NextPossiblyQuotedString": "sm_util_NextPossiblyQuotedString",
    "ReplaceFilenameWithNewPath": "sm_util_ReplaceFilenameWithNewPath", "SkipPrefix": "sm_util_SkipPrefix",
    "SplitKeyValue": "sm_util_SplitKeyValue", "StringEqualsNoCase": "sm_util_StringEqualsNoCase",
    "StringStartsWithNoCase": "sm_util_StringStartsWithNoCase", "Die": "sm_util_die", "atoi": "sm_config_atoi",
}


UTILITY_NAMES = """NextDelim StringEqualsNoCase StringStartsWithNoCase ReadWholeFile
    NextLineStripComments NextPossiblyQuotedString ReplaceFilenameWithNewPath
    SplitKeyValue SkipPrefix StrSet ByteArray_Resize ByteArray_Destroy
    ByteArray_AppendData ByteArray_AppendByte""".split()


def utility_source(source):
    selected = []
    for name in ["ToLower"] + UTILITY_NAMES:
        match = re.search(r"^(?:static inline )?[^\n]+\b" + name +
                          r"\([^\n]*\) \{\n.*?^\}", source, re.M | re.S)
        if not match:
            raise ValueError(f"Missing pinned utility function: {name}")
        selected.append(match[0])
    source = "\n\n".join(selected) + "\n"
    for old, new in (
        ("fread(buffer, 1, size, f) != size",
         "(size_t)(unsigned long)fread(buffer, 1, size, f) != size"),
        ("memchr(p, '#', eol - p)", "memchr(p, '#', (size_t)(long)(eol - p))"),
    ):
        if source.count(old) != 1:
            raise ValueError(f"Pinned utility patch no longer matches: {old}")
        source = source.replace(old, new)
    symbols = {name: "sm_util_" + name for name in UTILITY_NAMES}
    symbols["Die"] = "sm_util_die"
    return re.sub(r"\b(?:" + "|".join(symbols) + r")\b", lambda m: symbols[m[0]], source)


def rename_symbols(source):
    return re.sub(r"\b(?:" + "|".join(SYMBOLS) + r")\b", lambda m: SYMBOLS[m[0]], source)


def replace(source, old, new):
    if source.count(old) != 1:
        raise ValueError(f"Pinned configuration patch no longer matches: {old}")
    return source.replace(old, new)


def config_sources(source, header):
    source = replace(source, '#include "features.h"\n', '')
    source = replace(source, '#include "util.h"', '#include "native_config.h"')
    header = replace(header, '  const char *language;\n', '')
    header = replace(header, '  kOutputMethod_OpenGL_ES,\n', '')
    header = replace(header, '  kKeys_CheatWalkThroughWalls,\n', '')
    source = replace(source, '  _(SDLK_w), _(SDLK_o), S(SDLK_w), C(SDLK_e),',
                     '  _(SDLK_w), _(SDLK_q), S(SDLK_r),')
    source = replace(source, 'S(CheatLife), S(CheatKeys), S(CheatEquipment), S(CheatWalkThroughWalls),',
                     'S(CheatLife), S(CheatJump), S(ToggleWhichFrame),')
    for old, new in (("CheatKeys", "CheatJump"), ("CheatEquipment", "ToggleWhichFrame")):
        header = replace(header, "  kKeys_" + old + ",", "  kKeys_" + new + ",")
    reference_keys = "SDLK_1 SDLK_2 SDLK_3 SDLK_4 SDLK_5 SDLK_6 SDLK_7 SDLK_8 SDLK_9 SDLK_0 SDLK_MINUS SDLK_EQUALS SDLK_BACKSPACE".split()
    for title, modifier in (("Load Ref State", "_"), ("Replay Ref State", "C")):
        old = f"  // {title}\n  " + ", ".join(["N"] * 20) + ","
        new = f"  // {title}\n  " + ", ".join(f"{modifier}({key})" for key in reference_keys) + ", " + ", ".join(["N"] * 7) + ","
        source = replace(source, old, new)
    first = source.index("static bool ParseBoolBit(")
    last = source.index("static bool HandleIniConfig(", first)
    source = source[:first] + source[last:]
    source = replace(source, '                               StringEqualsNoCase(value, "OpenGL ES") ? kOutputMethod_OpenGL_ES :\n', '')
    source = replace(source, '    } else if (StringEqualsNoCase(key, "DimFlashes")) {\n      return ParseBoolBit(value, &g_config.features0, kFeatures0_DimFlashes);\n', '')
    first = source.index('    } else if (StringEqualsNoCase(key, "ExtendedAspectRatio")) {')
    last = source.index('    } else if (StringEqualsNoCase(key, "DisplayPerfInTitle")) {', first)
    source = source[:first] + source[last:]
    source = replace(source, '    } else if (StringEqualsNoCase(key, "Language")) {\n      g_config.language = value;\n', '')
    first = source.index('  } else if (section == 4) {')
    last = source.index('  }\n  return false;\n}', first)
    source = source[:first] + source[last:]
    source = replace(source, '"zelda3.user.ini"', '"sm.user.ini"')
    source = replace(source, '"zelda3.ini"', '"sm.ini"')
    # Use the same writable FILE pointer exposed by the native frontend.
    source = source.replace('fprintf(stderr,', 'fprintf(sm_config_stderr(),')
    return rename_symbols(source), rename_symbols(header)


def write_stamp(root, path, prefix, inputs):
    blobs = [(root / name).read_bytes() for name in inputs]
    declarations = [f"static const unsigned char {prefix}_source_{i}[] = {{" +
                    ",".join(str(byte) for byte in data) + "};" for i, data in enumerate(blobs)]
    declarations.append(f"static const unsigned char *const {prefix}_source_bytes[] = {{" +
                        ",".join(f"{prefix}_source_{i}" for i in range(len(blobs))) + "};")
    declarations.append(f"static const size_t {prefix}_source_lengths[] = {{" +
                        ",".join(str(len(data)) for data in blobs) + "};")
    path.write_text("\n".join(declarations) + "\n", encoding="ascii")


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--zig", required=True, type=Path)
    parser.add_argument("--sdl-include", required=True, type=Path)
    parser.add_argument("--prefix", type=Path, default=Path(os.environ["LOCALAPPDATA"]) / "sm-decomp-tools/native-config-45a149d8")
    parser.add_argument("--library-directory", type=Path)
    parser.add_argument("--emit-only", action="store_true")
    args = parser.parse_args()
    root = Path(__file__).resolve().parents[1]
    source = args.prefix / "source"
    library = args.library_directory or args.prefix / "lib"
    library.mkdir(parents=True, exist_ok=True)
    for name, expected in FILES.items():
        path = source / name
        path.parent.mkdir(parents=True, exist_ok=True)
        if not path.exists():
            urllib.request.urlretrieve(f"https://raw.githubusercontent.com/snesrev/zelda3/{REVISION}/{name}", path)
        if hashlib.sha256(path.read_bytes()).hexdigest() != expected:
            raise ValueError(f"Upstream source checksum mismatch: {name}")
    text, header = config_sources((source / "src/config.c").read_text(), (source / "src/config.h").read_text())
    (library / "native_config_selected.c").write_text(text, encoding="utf-8")
    (library / "config.h").write_text(header, encoding="utf-8")
    (library / "native_util_selected.c").write_text(utility_source((source / "src/util.c").read_text()), encoding="utf-8")
    write_stamp(root, library / "native_config_sources.h", "sm_config", NATIVE_INPUTS)
    write_stamp(root, library / "native_util_sources.h", "sm_util", UTILITY_INPUTS)
    if args.emit_only:
        return
    objects = []
    for owner in ("config", "util"):
        obj = library / f"native_{owner}.o"
        subprocess.run([str(args.zig), "cc", "-target", "x86_64-windows-gnu", "-O2", "-std=c11",
                        "-fexceptions", "-fwrapv", "-fno-strict-aliasing", "-ffunction-sections", "-fdata-sections",
                        "-I", str(library), "-I", str(source / "src"), "-I", str(root / "sm/src"),
                        "-I", str(args.sdl_include), "-c", str(root / f"sm/src/native_{owner}.c"), "-o", str(obj)], check=True)
        objects.append(str(obj))
    subprocess.run([str(args.zig), "ar", "rcs", str(library / "libsm_config.a"), *objects], check=True)
    print(f"Installed native configuration library in {library}; add this directory to LIBRARY_PATH.")


if __name__ == "__main__":
    main()
