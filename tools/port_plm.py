#!/usr/bin/env python3
"""Actual SpawnPLM allocation/amount context against native Rust, full WRAM."""

import argparse
import datetime
import json
import shutil
import subprocess
from pathlib import Path

from port_family import (
    ROOT,
    WRAM_SIZE,
    digest,
    execute,
    load_asset,
    prepare_case,
    verify_source,
)


def verify(manifest_path, source, output):
    run = output.resolve() / datetime.datetime.now(datetime.timezone.utc).strftime(
        "%Y%m%dT%H%M%S.%fZ"
    )
    run.mkdir(parents=True)
    verdict = {
        "status": "rejected",
        "run": str(run),
        "live_exercised": False,
        "original_rom_parity": "not-tested",
        "cases": [],
    }
    try:
        manifest = json.loads(manifest_path.read_text())
        shutil.copyfile(manifest_path, run / "manifest.json")
        verdict["source"] = verify_source(source)
        module = Path(manifest["rust_module"])
        module = module if module.is_absolute() else ROOT / module
        rom = load_asset(manifest["rom"]["file"], manifest["rom"]["sha256"])
        (run / "rom.bin").write_bytes(rom)
        wrapper = run / "wrapper.rs"
        wrapper.write_text(f"""#[path={json.dumps(str(ROOT / "crates/port/src/lib.rs"))}]mod memory;use memory::*;
#[path={json.dumps(str(module))}]mod plm;
#[unsafe(no_mangle)]pub unsafe extern "C" fn native_spawn(ram:*mut u8,len:usize,rom:*const u8,rom_len:usize,header:u16,amount:*mut i32)->i32{{
 if ram.is_null()||len!=WRAM_SIZE||rom.is_null()||rom_len==0||rom_len>0x400000||amount.is_null()||(amount as usize)%std::mem::align_of::<i32>()!=0{{return -1;}}
 let Some(rend)=(ram as usize).checked_add(len)else{{return -1;}};let Some(oend)=(rom as usize).checked_add(rom_len)else{{return -1;}};
 let Some(aend)=(amount as usize).checked_add(4)else{{return -1;}};
 if (ram as usize)<oend&&(rom as usize)<rend||(amount as usize)<rend&&(ram as usize)<aend||(amount as usize)<oend&&(rom as usize)<aend{{return -1;}}
 let state=unsafe{{std::slice::from_raw_parts_mut(ram,len)}};let rom=unsafe{{std::slice::from_raw_parts(rom,rom_len)}};let amount=unsafe{{&mut *amount}};
 let result=std::panic::catch_unwind(std::panic::AssertUnwindSafe(||plm::spawn(&mut Wram::new(state).unwrap(),rom,header,amount)));
 match result{{Ok(Ok(flags))=>flags.into(),_=>-1}}
}}
""")
        host = run / "oracle.c"
        host.write_text(
            """#define main retained_main
"""
            + f"#include {json.dumps(str(ROOT / 'tools/decomp/headless.c'))}\n"
            + """#undef main
#include "funcs.h"
extern int32 *cur_coll_amt32;
extern int32_t native_spawn(uint8_t*,size_t,const uint8_t*,size_t,uint16_t,int32_t*);
int main(int argc,char **argv){
 if(argc!=8||!SnesInit(argv[6]))return 2;g_spc_player=SpcPlayer_Create();SpcPlayer_Initialize(g_spc_player);
 if(!strcmp(argv[1],"abi")){
  _Alignas(int32_t) uint8_t storage[8]={0};int32_t amount=123;
  if(native_spawn(g_ram,sizeof(g_ram),g_rom,g_snes->cart->romSize,0xb72b,(int32_t*)(storage+1))!=-1)return 5;
  if(native_spawn(g_ram,sizeof(g_ram),g_rom,g_snes->cart->romSize,0xb72b,(int32_t*)g_ram)!=-1)return 5;
  if(native_spawn(g_ram,sizeof(g_ram),g_rom,g_snes->cart->romSize,0xb72b,(int32_t*)g_rom)!=-1)return 5;
  if(native_spawn(g_ram,sizeof(g_ram)-1,g_rom,g_snes->cart->romSize,0xb72b,&amount)!=-1)return 5;
  if(native_spawn(g_ram,sizeof(g_ram),g_ram,sizeof(g_ram),0xb72b,&amount)!=-1)return 5;
  if(amount!=123)return 5;puts("5 rejected ABI inputs");return 0;
 }
 FILE*f=fopen(argv[4],"rb");if(!f||fread(g_ram,1,sizeof(g_ram),f)!=sizeof(g_ram)||fgetc(f)!=EOF||fclose(f))return 2;
 uint16_t header=strtoul(argv[2],NULL,10);int32_t amount=(int32_t)strtoll(argv[3],NULL,10);int32_t flags;
 if(!strcmp(argv[1],"rust")){flags=native_spawn(g_ram,sizeof(g_ram),g_rom,g_snes->cart->romSize,header,&amount);if(flags<0)return 4;}
 else{cur_coll_amt32=&amount;flags=SpawnPLM(header);cur_coll_amt32=NULL;}
 f=fopen(argv[5],"wb");if(!f||fwrite(g_ram,1,sizeof(g_ram),f)!=sizeof(g_ram)||fclose(f))return 2;
 printf("{\\"flags\\":%d,\\"amount\\":%d,\\"cpu_fallback_calls\\":%lu,\\"cpu_fallback_instructions\\":%lu}\\n",flags,amount,fallback_calls,fallback_instructions);return 0;
}
"""
        )
        execute(
            [
                "rustc",
                "--edition=2024",
                "-O",
                "--crate-type=staticlib",
                str(wrapper),
                "-o",
                str(run / "libplm.a"),
            ],
            run,
            "rust-build",
        )
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
                str(host),
                str(run / "libplm.a"),
                "-Wl,--gc-sections",
                "-Wl,--wrap=Call",
                "-Wl,--wrap=cpu_runOpcode",
                "-Wl,--wrap=DecompressToMem",
                "-lm",
                "-ldl",
                "-lpthread",
                "-o",
                str(run / "spawn"),
            ],
            run,
            "c-build",
        )
        execute(
            [
                str(run / "spawn"),
                "abi",
                "0",
                "0",
                "reserved",
                "reserved",
                str(run / "rom.bin"),
                "reserved",
            ],
            run,
            "abi-controls",
        )
        verdict["rejected_abi_inputs"] = 5
        dependencies = {
            str(p): digest(p) for p in sorted((ROOT / "crates/port/src").glob("*.rs"))
        }
        dependencies[str(module)] = digest(module)
        for i, p in enumerate(dependencies):
            shutil.copyfile(p, run / f"dependency-{i:03d}.rs")
        verdict["implementation"] = {
            "module": str(module),
            "rust_dependencies": dependencies,
            "harness_sha256": digest(__file__),
            "binary_sha256": digest(run / "spawn"),
        }
        seen = set()
        for case in manifest["cases"]:
            if case["name"] in seen:
                raise ValueError("duplicate case")
            seen.add(case["name"])
            directory = run / case["name"]
            prepare_case(case, directory, ["uint16", "int32"])
            results = {}
            for subject in ("rust", "c"):
                out = execute(
                    [
                        str(run / "spawn"),
                        subject,
                        str(case["a"]),
                        str(case.get("b", 0)),
                        str(directory / "before.wram"),
                        str(directory / (subject + "-after.wram")),
                        str(run / "rom.bin"),
                        "reserved",
                    ],
                    directory,
                    subject,
                )
                results[subject] = json.loads(out.splitlines()[-1])
            record = {"case": case["name"], "results": results}
            verdict["cases"].append(record)
            if results["c"] != results["rust"]:
                raise ValueError("native SpawnPLM flags/amount mismatch")
            if (
                results["rust"]["cpu_fallback_calls"]
                or results["rust"]["cpu_fallback_instructions"]
            ):
                raise ValueError("SpawnPLM CPU fallback")
            if (
                "expected_amount" in case
                and results["rust"]["amount"] != case["expected_amount"]
            ):
                raise ValueError("independent amount mismatch")
            if (
                "expected_flags" in case
                and results["rust"]["flags"] != case["expected_flags"]
            ):
                raise ValueError("independent flags mismatch")
            c, r = [
                (directory / (s + "-after.wram")).read_bytes() for s in ("c", "rust")
            ]
            if len(c) != WRAM_SIZE or len(r) != WRAM_SIZE:
                raise ValueError("full WRAM extent")
            if c != r:
                off = next(i for i in range(WRAM_SIZE) if c[i] != r[i])
                record["first_mismatch"] = {"offset": off, "c": c[off], "rust": r[off]}
                raise ValueError(f"WRAM mismatch at {off:#x}")
        if not seen:
            raise ValueError("no native allocation coverage")
        verdict.update(
            status="offline-verified",
            verified_cases=len(seen),
            native_rust_spawn_calls=len(seen),
            remaining_c_gameplay_callbacks=0,
            scope="declared SpawnPLM headers/input domain; full WRAM plus flags and synchronous amount",
        )
    except (
        OSError,
        ValueError,
        KeyError,
        TypeError,
        subprocess.SubprocessError,
    ) as error:
        verdict["error"] = str(error)
    verdict["artifacts"] = {
        str(p.relative_to(run)): digest(p)
        for p in sorted(run.rglob("*"))
        if p.is_file() and p.name != "verdict.json"
    }
    (run / "verdict.json").write_text(json.dumps(verdict, indent=2) + "\n")
    return verdict


def verify_plm_evidence(verdict_path):
    """Authenticate native allocation evidence before admitting selected headers."""
    path = Path(verdict_path)
    run = path.parent
    v = json.loads(path.read_text())
    if v.get("status") != "offline-verified" or not v.get("cases"):
        raise ValueError("native PLM evidence not verified")
    artifacts = v.get("artifacts", {})
    for name, expected in artifacts.items():
        artifact = run / name
        if (
            run.resolve() not in artifact.resolve().parents
            or digest(artifact) != expected
        ):
            raise ValueError(f"PLM artifact changed: {name}")
    if (
        not {"spawn", "libplm.a", "manifest.json", "rom.bin", "oracle.c", "wrapper.rs"}
        <= artifacts.keys()
    ):
        raise ValueError("PLM artifact inventory incomplete")
    if digest(__file__) != v["implementation"]["harness_sha256"]:
        raise ValueError("PLM harness changed")
    dependencies = v["implementation"]["rust_dependencies"]
    required_dependencies = {str(p) for p in (ROOT / "crates/port/src").glob("*.rs")}
    required_dependencies.add(v["implementation"]["module"])
    if not required_dependencies <= dependencies.keys():
        raise ValueError("PLM Rust dependency inventory incomplete")
    for dependency, expected in dependencies.items():
        if digest(dependency) != expected:
            raise ValueError(f"PLM dependency changed: {dependency}")
    pin = next(
        p
        for p in json.loads((ROOT / "tools/dependencies.json").read_text())
        if p["name"] == "snesrev-sm"
    )
    if (
        v["source"]["revision"] != pin["revision"]
        or v["source"]["archive_sha256"] != pin["sha256"]
    ):
        raise ValueError("PLM pinned source identity changed")
    if digest(run / "spawn") != v["implementation"]["binary_sha256"]:
        raise ValueError("PLM binary changed")
    if (
        v.get("rejected_abi_inputs") != 5
        or "abi-controls.stdout" not in artifacts
        or (run / "abi-controls.stdout").read_text().splitlines()[-1]
        != "5 rejected ABI inputs"
    ):
        raise ValueError("PLM ABI rejection evidence incomplete")
    manifest = json.loads((run / "manifest.json").read_text())
    expected = {c["name"]: c for c in manifest["cases"]}
    observed = set()
    if len(expected) != len(manifest["cases"]):
        raise ValueError("duplicate PLM cases")
    for record in v["cases"]:
        name = record["case"]
        if name not in expected or name in observed:
            raise ValueError("PLM coverage changed")
        observed.add(name)
        directory = run / name
        if any(
            str((directory / n).relative_to(run)) not in artifacts
            for n in (
                "before.wram",
                "c-after.wram",
                "rust-after.wram",
                "c.stdout",
                "rust.stdout",
            )
        ):
            raise ValueError("PLM case artifacts incomplete")
        states = [
            (directory / n).read_bytes()
            for n in ("before.wram", "c-after.wram", "rust-after.wram")
        ]
        if any(len(s) != WRAM_SIZE for s in states) or states[1] != states[2]:
            raise ValueError("PLM actual WRAM mismatch")
        results = {
            subject: json.loads(
                (directory / (subject + ".stdout")).read_bytes().splitlines()[-1]
            )
            for subject in ("c", "rust")
        }
        if results != record["results"] or results["c"] != results["rust"]:
            raise ValueError("PLM actual amount/flags mismatch")
        if (
            results["rust"]["cpu_fallback_calls"]
            or results["rust"]["cpu_fallback_instructions"]
        ):
            raise ValueError("PLM CPU fallback")
        for field in ("amount", "flags"):
            if (
                "expected_" + field in expected[name]
                and results["rust"][field] != expected[name]["expected_" + field]
            ):
                raise ValueError("PLM expected output changed")
    if observed != expected.keys() or v["verified_cases"] != len(observed):
        raise ValueError("PLM coverage incomplete")
    return v


if __name__ == "__main__":
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--manifest", type=Path, required=True)
    parser.add_argument("--source", type=Path, default=ROOT / "deps/snesrev-sm")
    parser.add_argument("--output", type=Path, required=True)
    args = parser.parse_args()
    verdict = verify(args.manifest, args.source.resolve(), args.output)
    print(
        json.dumps({k: verdict[k] for k in ("status", "run", "error") if k in verdict})
    )
    raise SystemExit(0 if verdict["status"] == "offline-verified" else 1)
