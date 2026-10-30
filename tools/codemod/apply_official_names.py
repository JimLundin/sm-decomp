"""Apply tools/official_names.py across the workspace.

    python -I tools/codemod/apply_official_names.py [--root DIR] [--check] [--no-fmt]

One pass: every renamed identifier is looked up in a single map built before
any file is touched, so a rename never feeds another. The map holds:

- FUNCTIONS and IDENTIFIERS entries whose current identifier still occurs
  in the workspace;
- identifiers in the new crates' code (sm_game, sm_data, sm_runtime) whose
  words contain an ENTITIES word sequence, rewritten in the same case style;
- CASE_RULES names (constants.rs labels to SCREAMING_SNAKE_CASE);
- LOCALS, applied only inside their file.

ALLOW_REMOVALS entries drop style lints from `#[allow(..)]` attributes;
ALLOW_REASONS entries give the allows that stay a `reason = ".."`.

Where it replaces:

- Rust code: every whole identifier in the map.
- Rust comments: identifiers inside `backticks`, except provenance such as
  "Was `X`"; PROSE phrases in the new crates' comments, outside backticks.
- Rust strings: identifiers in path-like strings (with '/', '::' or '.rs'),
  which covers `include_bytes!` entries in sm/src/verify_source.rs.
- sm/src/verification/**/*.py: every whole identifier (mutation anchors).

Files and directories whose stem is a renamed identifier move with `git mv`,
in the crates and in sm/src (the re-export shims), so `mod` lines and paths
stay consistent. Files are read and written as bytes, so LF stays LF.
Running it again is a no-op.

--check lists the moves and renames it would make, and any conflict, and
changes nothing. Exit status: 0 nothing to do, 1 changes pending (--check),
2 conflicts. Without --check it refuses to write while a conflict stands,
then runs `cargo fmt --all` unless --no-fmt.
"""
import argparse
import collections
import importlib.util
import keyword
import pathlib
import re
import shutil
import subprocess
import sys

NEW_CRATES = ("sm_game", "sm_data", "sm_runtime")
RUST_KEYWORDS = set("""as async await break const continue crate dyn else enum extern false fn for gen if
impl in let loop match mod move mut pub ref return self Self static struct super trait true type union
unsafe use where while abstract become box do final macro override priv try typeof unsized virtual yield""".split())
ID = re.compile(r"\b[A-Za-z_][A-Za-z0-9_]*\b")
WORD = re.compile(r"[A-Z]+(?=[A-Z][a-z])|[A-Z]?[a-z]+|[A-Z]+|[0-9]+")
PROVENANCE = re.compile(r"\b(?:was|were|formerly|originally|renamed from|previously)\b[^.;]*$", re.IGNORECASE)
LABEL = re.compile(r"[a-z][A-Z]|[A-Z][a-z].*_|_[A-Z][a-z]|^sub_[0-9A-Fa-f]{6}$")


# ---------------------------------------------------------------- lexing

_RAW = re.compile(r'b?r(#*)"')
_CHAR = re.compile(r"'(?:\\(?:x[0-9a-fA-F]{2}|u\{[0-9a-fA-F]+\}|.)|[^\\'\n])'")


def rust_spans(t):
    """Split Rust source into ("code" | "comment" | "string", text) spans."""
    i, n, out, start = 0, len(t), [], 0

    def flush(upto):
        if upto > start:
            out.append(("code", t[start:upto]))

    while i < n:
        c = t[i]
        if c == "/" and t.startswith("//", i):
            flush(i)
            j = t.find("\n", i)
            j = n if j < 0 else j
            out.append(("comment", t[i:j]))
            i = start = j
            continue
        if c == "/" and t.startswith("/*", i):
            flush(i)
            depth, j = 1, i + 2
            while j < n and depth:
                if t.startswith("/*", j):
                    depth, j = depth + 1, j + 2
                elif t.startswith("*/", j):
                    depth, j = depth - 1, j + 2
                else:
                    j += 1
            out.append(("comment", t[i:j]))
            i = start = j
            continue
        if c in "rb" and (i == 0 or not (t[i - 1].isalnum() or t[i - 1] == "_")):
            m = _RAW.match(t, i)
            if m:
                flush(i)
                close = '"' + m.group(1)
                j = t.find(close, m.end())
                j = n if j < 0 else j + len(close)
                out.append(("string", t[i:j]))
                i = start = j
                continue
        if c == '"':
            flush(i)
            j = i + 1
            while j < n and t[j] != '"':
                j += 2 if t[j] == "\\" else 1
            j = min(j + 1, n)
            out.append(("string", t[i:j]))
            i = start = j
            continue
        if c == "'":
            m = _CHAR.match(t, i)
            if m:
                flush(i)
                out.append(("string", m.group(0)))
                i = start = m.end()
                continue
        i += 1
    flush(n)
    return out


# ---------------------------------------------------------------- entity rules

def split_words(ident):
    """Word spans (start, end, text) of an identifier."""
    return [(m.start(), m.end(), m.group(0)) for m in WORD.finditer(ident)]


def render(words, sample):
    """Render lowercase `words` in the case style of the original text `sample`."""
    if sample.isupper() or (sample.replace("_", "").isdigit()):
        return "_".join(w.upper() for w in words)
    if "_" in sample or sample.islower():
        if sample.islower():
            return "_".join(words)
        return "_".join(w.capitalize() for w in words)
    camel = "".join(w.capitalize() for w in words)
    if sample[:1].islower():
        camel = camel[:1].lower() + camel[1:]
    return camel


def apply_entities(ident, rules):
    """Rewrite entity word sequences in `ident`; None when nothing matches."""
    spans = split_words(ident)
    lowered = [w.lower() for _, _, w in spans]
    out, pos, i, changed = [], 0, 0, False
    while i < len(spans):
        for old, new in rules:
            k = len(old)
            if tuple(lowered[i:i + k]) != old:
                continue
            start, end = spans[i][0], spans[i + k - 1][1]
            sample = ident[start:end]
            seps = {ident[spans[j][1]:spans[j + 1][0]] for j in range(i, i + k - 1)}
            if len(seps) > 1 or (seps and seps != {"_"} and seps != {""}):
                continue
            # Only whole words: no lowercase word continuing a camel hump, etc.
            out.append(ident[pos:start])
            out.append(render(new, sample))
            pos, i, changed = end, i + k, True
            break
        else:
            i += 1
    if not changed:
        return None
    out.append(ident[pos:])
    return "".join(out)


# ---------------------------------------------------------------- workspace

def load_table(root):
    path = root / "tools" / "official_names.py"
    spec = importlib.util.spec_from_file_location("official_names", path)
    module = importlib.util.module_from_spec(spec)
    spec.loader.exec_module(module)
    return module


def rust_files(root):
    files = []
    for base in (root / "crates", root / "sm" / "src"):
        for p in base.rglob("*.rs"):
            if "target" in p.relative_to(root).parts:
                continue
            files.append(p)
    return sorted(files)


def anchor_files(root):
    base = root / "sm" / "src" / "verification"
    return sorted(base.rglob("*.py")) if base.is_dir() else []


def is_new_crate(root, path):
    rel = path.relative_to(root).parts
    return len(rel) > 2 and rel[0] == "crates" and rel[1] in NEW_CRATES


def decode(data):
    return data.decode("utf-8", errors="surrogateescape")


def encode(text):
    return text.encode("utf-8", errors="surrogateescape")


class Workspace:
    def __init__(self, root):
        self.root = root
        self.rust = {p: decode(p.read_bytes()) for p in rust_files(root)}
        self.anchors = {p: decode(p.read_bytes()) for p in anchor_files(root)}
        self.spans = {p: rust_spans(t) for p, t in self.rust.items()}
        self.tokens = {}
        self.code_tokens = {}
        for p, spans in self.spans.items():
            every, code = set(), set()
            for kind, text in spans:
                found = set(ID.findall(text))
                every |= found
                if kind == "code":
                    code |= found
            self.tokens[p] = every
            self.code_tokens[p] = code
        for p, t in self.anchors.items():
            self.tokens[p] = set(ID.findall(t))
        self.all_tokens = set().union(*self.tokens.values()) if self.tokens else set()
        self.all_code = set().union(*self.code_tokens.values()) if self.code_tokens else set()
        self.new_crate_code = set().union(*(self.code_tokens[p] for p in self.rust if is_new_crate(root, p)))


def build_map(ws, table):
    problems = []
    notes = []
    explicit, kind = {}, {}
    for group, entries in (("function", table.FUNCTIONS), ("identifier", getattr(table, "IDENTIFIERS", ()))):
        for _, old, new, _ in entries:
            if old in explicit and explicit[old] != new:
                problems.append(f"{old}: two targets {explicit[old]} and {new}")
            explicit[old], kind[old] = new, group
    rules = sorted(((tuple(old), tuple(new)) for _, old, new, _ in table.ENTITIES), key=lambda r: -len(r[0]))
    for old, new in case_rule_names(ws, table).items():
        if old not in explicit:
            # Entity words first, then the case: addr_kEproj_LavaSeahorseFireball -> ADDR_K_EPROJ_DRAGON_FIREBALL.
            named = apply_entities(old, rules)
            explicit[old], kind[old] = (screaming(named) if named else new), "case"
    mapping, origin = {}, {}
    for old, new in explicit.items():
        if old in ws.all_code:
            mapping[old], origin[old] = new, kind[old]
    derived = {}
    for ident in sorted(ws.new_crate_code):
        if ident in explicit:
            continue
        new = apply_entities(ident, rules)
        if new and new != ident:
            derived[ident] = new
    for old, new in derived.items():
        mapping[old], origin[old] = new, "entity"
        if new in ws.all_tokens and new not in mapping:
            # Legal for distinct items (another enum's variant); file_conflicts() rejects real clashes.
            notes.append(f"{old} -> {new}: {new} already names something elsewhere")
    for old, new in mapping.items():
        if not re.fullmatch(r"[A-Za-z_][A-Za-z0-9_]*", new) or new in RUST_KEYWORDS or keyword.iskeyword(new):
            problems.append(f"{old} -> {new}: not a usable identifier")
        if new in mapping:
            problems.append(f"{old} -> {new}: {new} is itself renamed (chained rename)")
    return mapping, origin, problems, notes


METHOD_DECL = re.compile(r"\bfn\s+([A-Za-z_]\w*)\s*(?:<[^>{]*>)?\s*\(\s*(?:&\s*(?:'\w+\s+)?)?(?:mut\s+)?self\b")


def free_tokens(code):
    """Identifiers that name something in this file's own namespace.

    Skips method calls (`.name`), path tails (`a::name`) outside `use` lines and
    method declarations, which a free function of the same name cannot clash with.
    """
    methods = set(METHOD_DECL.findall(code))
    free = set()
    for m in ID.finditer(code):
        name = m.group(0)
        before = code[: m.start()].rstrip(" \t")
        if before.endswith("."):
            continue
        if before.endswith("::"):
            line = code.rfind("\n", 0, m.start()) + 1
            if not re.match(r"\s*(?:pub(?:\([^)]*\))?\s+)?use\b", code[line: m.start()]):
                continue
        if name in methods and re.search(r"\bfn\s+" + re.escape(name) + r"\b", code):
            # Only the declaration(s) of a method; a later free use still counts below.
            if re.match(r"\s*(?:<[^>{]*>)?\s*\(\s*(?:&\s*(?:'\w+\s+)?)?(?:mut\s+)?self\b", code[m.end():]):
                continue
        free.add(name)
    return free


def file_conflicts(ws, mapping):
    problems = []
    for p in list(ws.rust) + list(ws.anchors):
        if p in ws.rust:
            code = "".join(t if k == "code" else " " for k, t in ws.spans[p])
            olds = [t for t in free_tokens(code) if t in mapping]
            present = free_tokens(code)
        else:
            olds = [t for t in ws.tokens[p] if t in mapping]
            present = ws.tokens[p]
        if not olds:
            continue
        rel = p.relative_to(ws.root).as_posix()
        targets = collections.defaultdict(list)
        for o in olds:
            targets[mapping[o]].append(o)
        for new, sources in targets.items():
            if len(sources) > 1:
                problems.append(f"{rel}: {', '.join(sorted(sources))} would all become {new}")
            if new in present and new not in mapping:
                problems.append(f"{rel}: {sources[0]} -> {new}, but {new} already occurs in this file")
    return problems


def planned_moves(ws, mapping):
    moves = []
    seen = set()
    for p in sorted(ws.rust):
        rel = p.relative_to(ws.root)
        parts = list(rel.parts)
        for i, part in enumerate(parts):
            stem = part[:-3] if part.endswith(".rs") and i == len(parts) - 1 else part
            if i < 2 or stem not in mapping or stem in ("mod", "lib", "main"):
                continue
            src = ws.root.joinpath(*parts[: i + 1])
            if src in seen:
                break
            seen.add(src)
            newpart = mapping[stem] + (".rs" if part.endswith(".rs") and i == len(parts) - 1 else "")
            moves.append((src, src.with_name(newpart)))
            break
    return moves


def rewrite_comment(text, mapping, prose, previous=""):
    """`previous` is the comment line just above, so "Was" at a line end covers the next line."""
    pieces = re.split(r"(`[^`\n]*`)", text)
    out = []
    for idx, piece in enumerate(pieces):
        if idx % 2 == 1:
            before = "".join(pieces[:idx])
            line = before.rsplit("\n", 1)[-1]
            if PROVENANCE.search(line) or (not line.strip("/! ") and PROVENANCE.search(previous)):
                out.append(piece)
            else:
                out.append(ID.sub(lambda m: mapping.get(m.group(0), m.group(0)), piece))
        else:
            if prose and not PROVENANCE.search(piece):
                for old, new in prose:
                    piece = re.sub(r"(?<![A-Za-z0-9_])" + re.escape(old) + r"(?![A-Za-z0-9_])", new, piece)
            out.append(piece)
    return "".join(out)


def rewrite_rust(ws, p, mapping, prose, local=None, lints=(), reasons=None):
    out = []
    names = dict(mapping, **local) if local else mapping
    swap = lambda m: names.get(m.group(0), m.group(0))
    use_prose = prose if is_new_crate(ws.root, p) else ()
    previous = ""
    for kind, text in ws.spans[p]:
        if kind == "code":
            if text.strip():
                previous = ""
            out.append(ID.sub(swap, text))
        elif kind == "comment":
            out.append(rewrite_comment(text, names, use_prose, previous))
            previous = text
        elif "/" in text or "::" in text or ".rs" in text:
            out.append(ID.sub(swap, text))
        else:
            out.append(text)
    text = "".join(out)
    text = remove_allows(text, lints) if lints else text
    return add_reasons(text, reasons) if reasons else text


def add_reasons(text, reasons):
    """Give `#[allow(lint)]` attributes for each lint in `reasons` a `reason = ".."`, once."""
    def fix(m):
        items = [x.strip() for x in m.group(3).split(",") if x.strip()]
        if any(x.startswith("reason") for x in items):
            return m.group(0)
        hit = [reasons[x] for x in items if x in reasons]
        if not hit:
            return m.group(0)
        return f'{m.group(1)}#{m.group(2)}[allow({", ".join(items)}, reason = "{hit[0]}")]\n'
    return ALLOW.sub(fix, text)


ALLOW = re.compile(r"(?m)^([ \t]*)#(!?)\[allow\(([^()\]]*)\)\][ \t]*\n")


def remove_allows(text, lints):
    """Drop `lints` from #[allow(..)] / #![allow(..)] attributes; drop an attribute left empty."""
    def fix(m):
        items = [x.strip() for x in m.group(3).split(",") if x.strip()]
        keep = [x for x in items if x not in lints]
        if keep == items:
            return m.group(0)
        if not keep:
            return ""
        return f"{m.group(1)}#{m.group(2)}[allow({', '.join(keep)})]\n"
    text = ALLOW.sub(fix, text)
    # Multi-line allow lists: one lint per line inside #[allow(\n ... \n)].
    def fix_block(m):
        items = [x.strip() for x in m.group(3).replace("\n", " ").split(",") if x.strip()]
        keep = [x for x in items if x not in lints]
        if keep == items:
            return m.group(0)
        if not keep:
            return ""
        inner = "".join(f"{m.group(1)}    {x},\n" for x in keep)
        return f"{m.group(1)}#{m.group(2)}[allow(\n{inner}{m.group(1)})]\n"
    return re.sub(r"(?m)^([ \t]*)#(!?)\[allow\(\n((?:[ \t]*[\w:]+,?\n)+)[ \t]*\)\][ \t]*\n", fix_block, text)


def screaming(name):
    """SCREAMING_SNAKE_CASE that keeps hex and acronym parts whole (addr_kEproj_86A7 -> ADDR_K_EPROJ_86A7)."""
    parts = []
    for part in name.split("_"):
        if not re.search(r"[a-z]", part):
            parts.append(part)
        else:
            parts += re.split(r"(?<=[a-z0-9])(?=[A-Z])|(?<=[A-Z])(?=[A-Z][a-z])", part)
    return "_".join(x.upper() for x in parts if x)


def case_rule_names(ws, table):
    """Names declared by each CASE_RULES macro in its file, mapped to the rule's case."""
    out = {}
    for rel, macro, case in getattr(table, "CASE_RULES", ()):
        path = ws.root / rel
        if path not in ws.rust or case != "SCREAMING_SNAKE_CASE":
            continue
        code = "".join(t if k == "code" else " " for k, t in ws.spans[path])
        name = re.escape(macro.rstrip("!"))
        for m in re.finditer(name + r"!\s*\{[^;{}]*;([^{}]*)\}", code):
            for n in re.findall(r"\b([A-Za-z_]\w*)\s*=", m.group(1)):
                if screaming(n) != n:
                    out[n] = screaming(n)
    return out


def unmapped_labels(ws, mapping):
    labels = set()
    for p, spans in ws.spans.items():
        if not is_new_crate(ws.root, p):
            continue
        code = "".join(t for k, t in spans if k == "code")
        for m in re.finditer(r"\bfn\s+([A-Za-z_]\w*)", code):
            name = m.group(1)
            if name not in mapping and LABEL.search(name) and not name.isupper():
                labels.add(name)
    return sorted(labels)


def git_mv(root, src, dst):
    if shutil.which("git"):
        r = subprocess.run(["git", "-C", str(root), "mv", str(src), str(dst)], capture_output=True, text=True)
        if r.returncode == 0:
            return
    src.rename(dst)


def main(argv=None):
    ap = argparse.ArgumentParser(description=__doc__.split("\n\n")[0])
    ap.add_argument("--root", type=pathlib.Path, default=pathlib.Path(__file__).resolve().parents[2])
    ap.add_argument("--check", action="store_true", help="list what would change; change nothing")
    ap.add_argument("--no-fmt", action="store_true", help="skip cargo fmt after applying")
    ap.add_argument("--verbose", action="store_true", help="list every rename")
    args = ap.parse_args(argv)
    root = args.root.resolve()
    table = load_table(root)
    ws = Workspace(root)
    mapping, origin, problems, notes = build_map(ws, table)
    problems += file_conflicts(ws, mapping)
    moves = planned_moves(ws, mapping)
    prose = tuple((old, new) for old, new, _ in table.PROSE)

    locals_by_file = collections.defaultdict(dict)
    for rel, old, new in getattr(table, "LOCALS", ()):
        path = root / rel
        if path in ws.code_tokens and old in ws.code_tokens[path]:
            locals_by_file[path][old] = new
            if new in ws.code_tokens[path] and new not in mapping:
                problems.append(f"{rel}: local {old} -> {new}, but {new} already occurs in this file")
    allows = collections.defaultdict(set)
    for rel, lint in getattr(table, "ALLOW_REMOVALS", ()):
        allows[root / rel].add(lint)
    reasons = collections.defaultdict(dict)
    for rel, lint, reason in getattr(table, "ALLOW_REASONS", ()):
        reasons[root / rel][lint] = reason
    changed = {}
    for p in ws.rust:
        new = rewrite_rust(ws, p, mapping, prose, locals_by_file.get(p), allows.get(p, ()), reasons.get(p))
        if new != ws.rust[p]:
            changed[p] = new
    for p, t in ws.anchors.items():
        new = ID.sub(lambda m: mapping.get(m.group(0), m.group(0)), t)
        if new != t:
            changed[p] = new

    kinds = collections.Counter(origin.values())
    print(f"renames: {len(mapping)} ({kinds['function']} functions, {kinds['identifier']} identifiers, "
          f"{kinds['case']} case-rule constants, {kinds['entity']} entity-derived identifiers); "
          f"locals: {sum(len(v) for v in locals_by_file.values())} in {len(locals_by_file)} files")
    print(f"moves: {len(moves)}; files to rewrite: {len(changed)}")
    for src, dst in moves:
        print(f"  move {src.relative_to(root).as_posix()} -> {dst.relative_to(root).as_posix()}")
    if args.verbose or args.check:
        for old in sorted(mapping):
            print(f"  {origin[old]:8} {old} -> {mapping[old]}")
    for name in unmapped_labels(ws, mapping):
        print(f"  unmapped decompiler label: {name}")
    for line in notes:
        print(f"note {line}")
    for line in problems:
        print(f"CONFLICT {line}")
    if problems:
        return 2
    if args.check:
        return 1 if (changed or moves) else 0
    for p, text in changed.items():
        p.write_bytes(encode(text))
    for src, dst in sorted(moves, key=lambda m: -len(m[0].parts)):
        git_mv(root, src, dst)
    if (changed or moves) and not args.no_fmt:
        cargo = shutil.which("cargo")
        if cargo:
            # A renamed call can need a second pass once its line has been rewrapped.
            for _ in range(3):
                subprocess.run([cargo, "fmt", "--all"], cwd=root, check=False)
                if subprocess.run([cargo, "fmt", "--all", "--check"], cwd=root,
                                  capture_output=True).returncode == 0:
                    break
        else:
            print("cargo not found; run `cargo fmt --all`")
    return 0


if __name__ == "__main__":
    sys.exit(main())
