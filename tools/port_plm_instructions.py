#!/usr/bin/env python3
"""Actual bank84 dispatcher comparison; uint8 pointers normalized to bank84."""

import argparse
import json
import shutil
from pathlib import Path

import port_family
from port_family import IDENTIFIER, ROOT, WRAM_SIZE, digest, execute, parse_result


def build(manifest, source, run):
    (run / "plm-shim.c").write_text(
        '#include "sm_rtl.h"\n#include "variables.h"\n#include "funcs.h"\nextern const uint8 *CallPlmInstr(uint32,const uint8*,uint16);\nextern CoroutineRet PlmHandler_Async(void);\nconst uint16 *PlmInstruction(uint16 slot,const uint16 *script) {\n const uint8 *p=(const uint8*)script;\n return (const uint16*)CallPlmInstr(GET_WORD(p)|0x840000,p+2,slot);\n}\nuint32 PlmControlFrame(void) {return PlmHandler_Async();}\n'
    )
    host_support = run / "host-support.c"
    host_support.write_text(
        "#define main retained_main\n#include "
        + json.dumps(str(ROOT / "tools/decomp/headless.c"))
        + "\n"
    )
    prototypes, branches = [], []
    for routine in manifest["routines"]:
        symbol, return_type = routine["symbol"], routine["return"]
        types = routine.get("arg_types", ["uint16"] * routine.get("args", 0))
        count = len(types)
        if (
            not IDENTIFIER.fullmatch(symbol)
            or count not in (0, 1, 2)
            or return_type
            not in ("void", "uint8", "uint16", "uint32", "int32", "rom_ptr")
            or any(
                t not in ("uint8", "uint16", "uint32", "int32", "rom_ptr")
                for t in types
            )
        ):
            raise ValueError("invalid routine signature")
        c_type = {
            "void": "void",
            "uint8": "uint8_t",
            "uint16": "uint16_t",
            "uint32": "uint32_t",
            "int32": "int32_t",
            "rom_ptr": "const uint16_t *",
        }[return_type]
        prototypes.append(
            f"extern {c_type} {symbol}({', '.join('const uint16_t *' if t == 'rom_ptr' else t + '_t' for t in types) or 'void'});"
        )
        call = f"{symbol}({', '.join('(const uint16_t*)(g_rom+((((uint32_t)' + arg + ' >>16)<<15)|((uint32_t)' + arg + '&0x7fff))%0x400000)' if t == 'rom_ptr' else '(' + t + '_t)' + arg for t, arg in zip(types, ['a', 'b']))})"
        result = f"{call}; result=0;" if return_type == "void" else f"result={call};"
        if "rom_ptr" in types and "rom" not in manifest:
            raise ValueError("ROM pointer argument requires ROM asset")
        if return_type == "rom_ptr":
            result = f"const uint16_t *pointer={call}; result=PortPointerResult(pointer,(uint16_t)a);"
        branches.append(f'if (!strcmp(argv[1], "{symbol}")) {{ {result} }}')
    support = manifest.get("c_support", [])
    uses_rtl = "sm_rtl.c" in support or manifest["bank"] == "sm_rtl.c"
    globals_c = (
        "extern uint8_t g_ram[0x20000]; extern const uint8_t *g_rom;"
        if uses_rtl
        else "uint8_t g_ram[0x20000]; const uint8_t *g_rom;"
    )
    rom_setup = (
        """FILE *rf=fopen(argv[6],"rb"); if(!rf)return 2; if(fseek(rf,0,SEEK_END))return 2; long n=ftell(rf); rewind(rf); uint8_t *rom=malloc(n); if(!rom||fread(rom,1,n,rf)!=(size_t)n||fclose(rf))return 2; g_rom=rom;"""
        if "rom" in manifest
        else ""
    )
    oracle = run / "oracle.c"
    oracle.write_text(
        """#include <stdint.h>
#include <stdio.h>
#include <stdlib.h>
#include <string.h>
#include "sm_rtl.h"
#include "funcs.h"
"""
        + globals_c
        + """
static uint32_t PortPointerResult(const uint16_t *pointer,uint16_t unused) {
 if(!pointer)return 0;uintptr_t p=(uintptr_t)pointer;
 uintptr_t base=(uintptr_t)g_rom+0x18000;
 if(p<0x10000){if(p<0x8000)exit(3);return 0x840000|p;}
 if(p<base+0x8000||p>=base+0x10000)exit(3);
 return 0x840000|(p-base);
}
"""
        + "\n".join(prototypes)
        + """
int main(int argc, char **argv) {
 if(argc!=6 && argc!=7) return 2;
 int64_t a=strtoll(argv[2],0,10), b=strtoll(argv[3],0,10); uint32_t result=0;
 """
        + rom_setup
        + """
 FILE *f=fopen(argv[4],"rb");
 if(!f || fread(g_ram,1,sizeof(g_ram),f)!=sizeof(g_ram) || fgetc(f)!=EOF || fclose(f)) return 2;
 """
        + " else ".join(branches)
        + """ else return 2;
 f=fopen(argv[5],"wb");
 if(!f || fwrite(g_ram,1,sizeof(g_ram),f)!=sizeof(g_ram) || fclose(f)) return 2;
 printf("%u\\n",result); return 0;
}
"""
    )
    module = Path(manifest["rust_module"])
    if not module.is_absolute():
        module = ROOT / module
    signed = any("arg_types" in r for r in manifest["routines"])
    if "rom" in manifest:
        rust_setup = "let rom=std::fs::read(&args[6]).unwrap();"
        rust_call = "family::dispatch_rom(&mut Wram::new(&mut bytes).unwrap(),&rom,&args[1],args[2].parse::<i64>().unwrap() as i32,args[3].parse::<i64>().unwrap() as i32)"
    elif signed:
        rust_setup = ""
        rust_call = "family::dispatch_i32(&mut Wram::new(&mut bytes).unwrap(),&args[1],args[2].parse::<i64>().unwrap() as i32,args[3].parse::<i64>().unwrap() as i32)"
    else:
        rust_setup = ""
        rust_call = "family::dispatch(&mut Wram::new(&mut bytes).unwrap(),&args[1],args[2].parse().unwrap(),args[3].parse().unwrap())"
    imports = manifest.get("rust_imports", [])
    if any(not IDENTIFIER.fullmatch(name) for name in imports):
        raise ValueError("invalid Rust module import")
    root_imports = "".join(f"use memory::{name};\n" for name in imports)
    wrapper = run / "wrapper.rs"
    wrapper.write_text(f"""#[path = {json.dumps(str(ROOT / "crates/port/src/lib.rs"))}] mod memory;
use memory::*;
{root_imports}#[path = {json.dumps(str(module))}] mod family;
fn main() {{
 let args:Vec<String>=std::env::args().collect(); assert!(args.len()==6 || args.len()==7);
 let mut bytes=std::fs::read(&args[4]).unwrap();
 {rust_setup}
 let result={rust_call}.unwrap();
 std::fs::write(&args[5],bytes).unwrap(); println!("{{}}",result);
}}
""")
    bank = source / "src" / manifest["bank"]
    if (
        bank.parent != source / "src"
        or not bank.name.startswith("sm_")
        or bank.suffix != ".c"
    ):
        raise ValueError("invalid source bank")
    extra = []
    for name in support:
        path = source / "src" / name
        if path.parent != source / "src" or path.suffix != ".c":
            raise ValueError("invalid C support source")
        if path != bank:
            extra.append(str(path))
    units = (
        sorted((source / "src").glob("sm_*.c"))
        + [source / "src" / n for n in ("spc_player.c", "tracing.c", "util.c")]
        + sorted((source / "src/snes").glob("*.c"))
    )
    execute(
        [
            "cc",
            "-O2",
            "-fno-strict-aliasing",
            "-ffunction-sections",
            "-fdata-sections",
            "-iquote",
            str(source / "src"),
            *map(str, units),
            str(run / "plm-shim.c"),
            str(host_support),
            str(oracle),
            "-Wl,--gc-sections",
            "-lm",
            "-o",
            str(run / "c"),
        ],
        run,
        "c-build",
    )
    execute(
        ["rustc", "--edition=2024", "-O", str(wrapper), "-o", str(run / "rust")],
        run,
        "rust-build",
    )
    shutil.copyfile(module, run / "candidate.rs")
    shutil.copyfile(ROOT / "crates/port/src/lib.rs", run / "memory.rs")
    dependency_paths = {p.resolve() for p in (ROOT / "crates/port/src").glob("*.rs")}
    for value in manifest.get("rust_dependencies", []):
        path = Path(value)
        dependency_paths.add((path if path.is_absolute() else ROOT / path).resolve())
    dependencies = {str(path): digest(path) for path in sorted(dependency_paths)}
    for index, path in enumerate(sorted(dependency_paths)):
        shutil.copyfile(path, run / f"dependency-{index:03d}.rs")
    return {
        "rust_dependencies": dependencies,
        "module": str(module),
        "module_sha256": digest(module),
        "memory_sha256": digest(ROOT / "crates/port/src/lib.rs"),
        "harness_sha256": digest(__file__),
        "family_harness_sha256": digest(port_family.__file__),
        "binaries": {name: digest(run / name) for name in ("c", "rust")},
    }


def verify_plm_instruction_evidence(verdict_path):
    """Authenticate retained offline evidence before using it as a live input.

    Integrity is relative to this local verdict and current pinned implementation,
    not an external cryptographic signer. Rechecks actual output files and coverage.
    """
    verdict_path = Path(verdict_path)
    verdict = json.loads(verdict_path.read_text())
    run = verdict_path.parent
    if verdict.get("status") != "offline-verified" or not verdict.get("cases"):
        raise ValueError("family evidence is not nonempty offline verification")
    artifacts = verdict.get("artifacts")
    if not artifacts:
        raise ValueError("family evidence missing artifact inventory; regenerate")
    for name, expected in artifacts.items():
        path = run / name
        if run.resolve() not in path.resolve().parents or digest(path) != expected:
            raise ValueError(f"family artifact changed: {name}")
    required = {
        "manifest.json",
        "c",
        "rust",
        "candidate.rs",
        "memory.rs",
        "oracle.c",
        "wrapper.rs",
        "plm-shim.c",
        "host-support.c",
    }
    if not required <= artifacts.keys():
        raise ValueError("family artifacts incomplete")
    manifest = json.loads((run / "manifest.json").read_text())
    pin = next(
        p
        for p in json.loads((ROOT / "tools/dependencies.json").read_text())
        if p["name"] == "snesrev-sm"
    )
    if (
        verdict["source"]["revision"] != pin["revision"]
        or verdict["source"]["archive_sha256"] != pin["sha256"]
    ):
        raise ValueError("family pinned source identity changed")
    implementation = verdict["implementation"]
    for path, expected in [
        (Path(implementation["module"]), implementation["module_sha256"]),
        (ROOT / "crates/port/src/lib.rs", implementation["memory_sha256"]),
        (Path(__file__), implementation["harness_sha256"]),
    ]:
        if digest(path) != expected:
            raise ValueError(f"family implementation changed: {path}")
    for dependency, expected in implementation.get("rust_dependencies", {}).items():
        if digest(dependency) != expected:
            raise ValueError(f"family Rust dependency changed: {dependency}")
    for binary in ("c", "rust"):
        if digest(run / binary) != implementation["binaries"][binary]:
            raise ValueError(f"family binary changed: {binary}")
    expected_cases = {
        (r["symbol"], c["name"]): c for r in manifest["routines"] for c in r["cases"]
    }
    if not expected_cases or len(expected_cases) != sum(
        len(r["cases"]) for r in manifest["routines"]
    ):
        raise ValueError("family manifest has empty or duplicate cases")
    observed = set()
    for case in verdict["cases"]:
        key = (case["routine"], case["case"])
        if key not in expected_cases or key in observed:
            raise ValueError("family coverage changed")
        observed.add(key)
        directory = run / case["case"]
        paths = [
            directory / "before.wram",
            directory / "c-after.wram",
            directory / "rust-after.wram",
        ]
        mandatory = paths + [directory / "c.stdout", directory / "rust.stdout"]
        if any(str(path.relative_to(run)) not in artifacts for path in mandatory):
            raise ValueError("family case artifacts incomplete")
        before, c, rust = [path.read_bytes() for path in paths]
        if any(len(data) != WRAM_SIZE for data in (before, c, rust)) or c != rust:
            raise ValueError("family actual full WRAM outputs differ")
        results = {
            subject: parse_result((directory / (subject + ".stdout")).read_bytes())
            for subject in ("c", "rust")
        }
        if results != case["return"] or results["c"] != results["rust"]:
            raise ValueError("family actual return outputs differ")
        config = expected_cases[key]
        if "expected_return" in config and results["c"] != config["expected_return"]:
            raise ValueError("family expected return changed")
        if (
            "expected_after_file" in config
            and str((directory / "expected-after.wram").relative_to(run))
            not in artifacts
        ):
            raise ValueError("family capture artifact missing")
        if (
            "expected_after_file" in config
            and c != (directory / "expected-after.wram").read_bytes()
        ):
            raise ValueError("family actual captured output differs")
    if (
        observed != expected_cases.keys()
        or verdict.get("verified_cases") != len(observed)
        or verdict.get("verified_routines") != len(manifest["routines"])
    ):
        raise ValueError("family coverage incomplete")
    if digest(port_family.__file__) != implementation["family_harness_sha256"]:
        raise ValueError("PLM family harness changed")
    return verdict


if __name__ == "__main__":
    parser = argparse.ArgumentParser()
    parser.add_argument("--manifest", type=Path, required=True)
    parser.add_argument("--source", type=Path, required=True)
    parser.add_argument("--output", type=Path, required=True)
    args = parser.parse_args()
    port_family.build = build
    v = port_family.verify(args.manifest, args.source, args.output)
    print(json.dumps({k: v[k] for k in ("status", "run", "error") if k in v}))
    raise SystemExit(v["status"] != "offline-verified")
