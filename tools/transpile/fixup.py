#!/usr/bin/env python3
"""One-time fixups that make c2rust 0.22.1 output of snesrev/sm build on
nightly-2026-09-29. Each fix is mechanical and behaviour-preserving.

Usage: fixup.py <crate root containing src/>
"""
import re
import sys
from pathlib import Path

root = Path(sys.argv[1])
changed = {}


def edit(path, fn):
    text = path.read_text()
    new = fn(text)
    if new != text:
        path.write_text(new)
        changed[str(path.relative_to(root))] = changed.get(str(path.relative_to(root)), 0) + 1


# 1. c_variadic: VaListImpl/as_va_list() were folded into VaList.
def varargs(text):
    text = text.replace(
        "let mut va: ::core::ffi::VaListImpl;\n    va = args.clone();",
        "let mut va = args.clone();",
    )
    return text.replace("va.as_va_list()", "va")


# 2. `(int expr) as bool` is not a valid cast in Rust; C means `!= 0`.
BOOL_CAST = re.compile(r"as ::core::ffi::c_int\) as bool;")


def bool_casts(text):
    return BOOL_CAST.sub("as ::core::ffi::c_int) != 0;", text)


# 3. `let ref mut freshN = <place>.<field>;` takes a reference to a field of a
#    #[repr(packed)] struct, which may be unaligned. Turn it into a raw pointer
#    and use read_unaligned/write_unaligned for every use until the end of the
#    enclosing top-level function.
LET_REF = re.compile(r"let ref mut (fresh\d+) = ")


def packed_refs(text, packed_lines):
    lines = text.split("\n")
    for start in sorted(packed_lines, reverse=True):
        i = start - 1
        m = LET_REF.search(lines[i])
        if not m:
            raise SystemExit(f"no `let ref mut` at line {start}")
        name = m.group(1)
        lines[i] = lines[i].replace(f"let ref mut {name} = ", f"let {name} = &raw mut ", 1)
        # scope: until the next line that closes a top-level fn
        end = i + 1
        while end < len(lines) and lines[end] != "}":
            end += 1
        body = "\n".join(lines[i + 1 : end])
        # writes: `*freshN = expr;` (expr may span lines, ends at the first `;`
        # at the same nesting depth)
        out, pos = [], 0
        write = re.compile(r"\*" + name + r" = ")
        while True:
            w = write.search(body, pos)
            if not w:
                out.append(body[pos:])
                break
            out.append(body[pos : w.start()])
            j, depth = w.end(), 0
            while True:
                c = body[j]
                if c in "([{":
                    depth += 1
                elif c in ")]}":
                    depth -= 1
                elif c == ";" and depth == 0:
                    break
                j += 1
            expr = body[w.end() : j]
            out.append(f"{name}.write_unaligned({expr})")
            pos = j
        body = "".join(out)
        # remaining reads: `(*freshN)` or bare `*freshN`
        body = body.replace(f"(*{name})", f"{name}.read_unaligned()")
        body = re.sub(r"\*" + name + r"\b", f"{name}.read_unaligned()", body)
        lines[i + 1 : end] = body.split("\n")
    return "\n".join(lines)


# rustc E0793 locations, collected from the first build.
PACKED = {
    "src/src/sm_81.rs": [6260],
    "src/src/sm_82.rs": [8724, 8797],
    "src/src/sm_86.rs": [8917, 9230, 9451, 9486],
    "src/src/sm_8f.rs": [2590, 2631, 2686],
    "src/src/sm_a4.rs": [1823, 2096],
    "src/src/sm_a7.rs": [3553, 3571],
    "src/src/sm_a9.rs": [8983],
}

edit(root / "src/src/util.rs", varargs)
for f in sorted(root.glob("src/**/*.rs")):
    edit(f, bool_casts)
for f, ls in PACKED.items():
    edit(root / f, lambda t, ls=ls: packed_refs(t, ls))

for f, n in sorted(changed.items()):
    print(f"fixed {f}")
