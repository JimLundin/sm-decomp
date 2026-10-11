#!/usr/bin/env python3
"""Actual bank84 dispatcher comparison; uint8 pointers normalized to bank84."""

import argparse
import json
import shutil
from pathlib import Path

import port_family
from port_family import IDENTIFIER, ROOT, WRAM_SIZE, digest, execute, parse_result

from port_inputs import wrapper_memory, compiler_dependencies, verify_dependencies, resolve_source, memory_source


def build(manifest, source, run):
    if manifest.get("projectile_host") and ("rom" not in manifest or
            Path(manifest["rust_module"]).name != "enemy_projectiles.rs"):
        raise ValueError("projectile host requires ROM and allocator module")
    eproj = any(r["symbol"] == "EprojInstruction" for r in manifest["routines"])
    pointer_bank = 0x86 if eproj else 0x84
    (run / "plm-shim.c").write_text(
        '#include "sm_rtl.h"\n#include "variables.h"\n#include "funcs.h"\nextern const uint8 *CallPlmInstr(uint32,const uint8*,uint16);\nextern CoroutineRet PlmHandler_Async(void);\nconst uint16 *PlmInstruction(uint16 slot,const uint16 *script) {\n const uint8 *p=(const uint8*)script;\n return (const uint16*)CallPlmInstr(GET_WORD(p)|0x840000,p+2,slot);\n}\nuint32 PlmControlFrame(void) {return PlmHandler_Async();}\nuint32 PlmFrame(void) {return PlmHandler_Async();}\n'
    )
    if eproj:
        (run / "plm-shim.c").write_text('#include ' + json.dumps(str((source / "src/sm_86.c").resolve())) + '\n')
        with (run / "plm-shim.c").open("a") as shim:
            shim.write('const uint16 *EprojInstruction(uint16 slot,const uint16 *script) {\n'
                       'const uint8 *p=(const uint8*)script;\n'
                       'return (const uint16*)CallEprojInstr(GET_WORD(p)|0x860000,slot,p+2);}\n')
            shim.write('void EprojPreinstruction(uint16 slot,uint16 address) {'
                       'CallEprojPreInstr(address|0x860000,slot);}\n')
    if manifest.get("projectile_host"):
        if not eproj:
            raise ValueError("projectile host requires bank86 dispatcher")
        with (run / "plm-shim.c").open("a") as shim:
            shim.write("void PortHostLoad(uint16 *v){ eproj_spawn_pt=(Point16U){v[0],v[1]}; eproj_spawn_rect=(Rect16U){v[2],v[3],v[4],v[5]}; eproj_spawn_r22=v[6]; eproj_spawn_varE24=v[7]; }\n")
            shim.write("void PortHostSave(uint16 *v){ v[0]=eproj_spawn_pt.x;v[1]=eproj_spawn_pt.y;v[2]=eproj_spawn_rect.x;v[3]=eproj_spawn_rect.y;v[4]=eproj_spawn_rect.w;v[5]=eproj_spawn_rect.h;v[6]=eproj_spawn_r22;v[7]=eproj_spawn_varE24;}\n")
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
            bank = routine.get("pointer_bank", pointer_bank)
            if type(bank) is not int or bank not in (0x84, 0x86, 0xb2):
                raise ValueError("invalid pointer bank")
            result = f"const uint16_t *pointer={call}; result=PortPointerResult(pointer,{bank});"
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
static uint32_t PortPointerResult(const uint16_t *pointer,uint16_t bank) {
 if(!pointer)return 0;uintptr_t p=(uintptr_t)pointer;
 uintptr_t base=(uintptr_t)g_rom+((bank&0x7f)*0x8000)-0x8000;
 if(p<0x10000){if(p<0x8000)exit(3);return ((uint32_t)bank<<16)|p;}
 if(p<base+0x8000||p>=base+0x10000)exit(3);
 return ((uint32_t)bank<<16)|(p-base);
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
    if manifest.get("projectile_host"):
        text = oracle.read_text().replace('int main(int argc, char **argv) {', 'void PortHostLoad(uint16*);void PortHostSave(uint16*);\nint main(int argc, char **argv) {')
        text = text.replace('if(argc!=6 && argc!=7)', 'if(argc!=9)')
        text = text.replace(' int64_t a=', ' uint16 host[8]; FILE *hf=fopen(argv[7],"rb"); if(!hf||fread(host,2,8,hf)!=8||fgetc(hf)!=EOF||fclose(hf))return 2; PortHostLoad(host);\n int64_t a=')
        text = text.replace(' printf("%u\\n",result);', ' PortHostSave(host);hf=fopen(argv[8],"wb");if(!hf||fwrite(host,2,8,hf)!=8||fclose(hf))return 2;\n printf("%u\\n",result);')
        oracle.write_text(text)
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
    memory = wrapper_memory(run, [module], manifest.get("rust_imports", []))
    wrapper = run / "wrapper.rs"
    wrapper.write_text(f"""#[path = {json.dumps(str(memory))}] mod memory;
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
    if module.name == "enemy_projectiles.rs":
        # Allocator and instruction callbacks share ProjectileHost. Compile the
        # allocator once so cross-module calls retain the same Rust type.
        text = wrapper.read_text().replace('#[path = ' + json.dumps(str(module)) + '] mod family;', 'use memory::enemy_projectiles as family;')
        wrapper.write_text(text)
    if manifest.get("projectile_host"):
        text = wrapper.read_text().replace('assert!(args.len()==6 || args.len()==7)', 'assert_eq!(args.len(),9)')
        text = text.replace(' let result=family::dispatch_rom(', ' let raw=std::fs::read(&args[7]).unwrap();assert_eq!(raw.len(),16);let mut words=[0;8];for (i,w) in words.iter_mut().enumerate(){*w=u16::from_le_bytes([raw[i*2],raw[i*2+1]]);}let mut host=family::ProjectileHost::from_words(words);\n let result=family::dispatch_host(')
        text = text.replace('&rom,&args[1]', '&rom,&mut host,&args[1]')
        text = text.replace(' std::fs::write(&args[5]', ' let out:Vec<u8>=host.words().into_iter().flat_map(u16::to_le_bytes).collect();std::fs::write(&args[8],out).unwrap();\n std::fs::write(&args[5]')
        wrapper.write_text(text)
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
    if eproj:
        units = [p for p in units if p != source / "src/sm_86.c"]
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
        ["rustc", "--edition=2024", "-O", "--emit=link,dep-info=" + str(run / "compiler.d"), str(wrapper), "-o", str(run / "rust")],
        run,
        "rust-build",
    )
    shutil.copyfile(module, run / "candidate.rs")
    dependencies = compiler_dependencies(wrapper, run, manifest)
    return {
        "dependency_contract": "compiler-v2", "source_root": str(ROOT),
        "inputs_harness_sha256": digest(ROOT / "tools/port_inputs.py"),
        "rust_dependencies": dependencies,
        "module": str(module),
        "module_sha256": digest(module),
        "memory_sha256": digest(memory_source({"dependency_contract": "compiler-v2"})),
        "harness_sha256": digest(__file__),
        "family_harness_sha256": digest(port_family.__file__),
        "binaries": {name: digest(run / name) for name in ("c", "rust")},
    }


def verify_projectile_host(directory, run, artifacts, words):
    """Host globals must be compared independently even if WRAM matches."""
    paths = [directory / name for name in ("host-before.bin", "c-host-after.bin", "rust-host-after.bin")]
    if any(str(path.relative_to(run)) not in artifacts for path in paths):
        raise ValueError("projectile host artifact missing")
    before, c, rust = [path.read_bytes() for path in paths]
    expected = bytes(value for word in words for value in (word & 255, word >> 8))
    if before != expected or len(c) != 16 or c != rust:
        raise ValueError("projectile host state differs")


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
        (resolve_source(implementation["module"], implementation), implementation["module_sha256"]),
        (memory_source(implementation), implementation["memory_sha256"]),
        (Path(__file__), implementation["harness_sha256"]),
    ]:
        if digest(path) != expected:
            raise ValueError(f"family implementation changed: {path}")
    verify_dependencies(implementation)
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
        if manifest.get("projectile_host"):
            verify_projectile_host(directory, run, artifacts, expected_cases[key]["host_words"])
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
