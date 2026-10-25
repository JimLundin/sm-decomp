# Usage (repo root): python3 tools/codemod/strip_unsafe_fixpoint.py
#
# Lane M1a: sound fixpoint strip of `unsafe fn` over sm/src.
# A fn loses `unsafe` only when (a) its signature names no raw pointer,
# (b) it is not extern/no_mangle/export_name/cfg-gated or inside macro_rules,
# (c) its body contains no `unsafe {}` block (fn-pointer types are fine), and (d) both compile
# configurations (no-default lib; verifier lib+example with sm_verify_dll)
# still build. With crate-root deny(unsafe_op_in_unsafe_fn), (c)+(d) mean the
# body performs no unsafe operation. After each accepted round, `unused_unsafe`
# blocks reported by the compiler are unwrapped (`unsafe {` -> `{`), which can
# make callers eligible; repeat to a fixpoint, then `cargo fmt`.
# (e) Transitive: the fn's name must not reach, through any same-named item in
# the macro-expanded crate, an item holding an `unsafe {}` block. A safe wrapper
# around unchecked hardware/ROM/RAM access must not gain new safe callers
# (docs/M1A-REVIEW.md). Name collisions only exclude more.
# Line counts never change before the final fmt, so (file, line) keys are stable.
import glob, json, os, re, subprocess, sys, time

ROOT = os.getcwd()
CHECKS = [
    (['cargo', 'check', '-p', 'sm', '--no-default-features', '--lib',
      '--message-format', 'json'], {}),
    (['cargo', 'check', '-p', 'sm', '--lib', '--example', 'sm_verify',
      '--target-dir', 'target/vcheck', '--message-format', 'json'],
     {'RUSTFLAGS': '--cfg sm_verify_dll'}),
]
RAW = re.compile(r'\*\s*(const|mut)\b|\bSDL_GLContext\b')
BAD_ATTR = re.compile(r'no_mangle|export_name|\bcfg\s*\(|extern')


TOKEN = re.compile(r"""
    (?P<ws>\s+)
  | (?P<lc>//[^\n]*)
  | (?P<bc>/\*)
  | (?P<raw>b?r(?P<h>\#*)")
  | (?P<str>b?"(?:\\.|[^"\\])*")
  | (?P<chr>b?'(?:\\.[^']*|[^\\'])')
  | (?P<life>'\w+)
  | (?P<id>[A-Za-z_]\w*)
  | (?P<p>.)
""", re.X | re.S)


def lex(s):
    """Return (kind, start, end) for code tokens, skipping comments/strings/chars."""
    i, n, out = 0, len(s), []
    while i < n:
        m = TOKEN.match(s, i)
        k = m.lastgroup if m.lastgroup != 'h' else 'raw'
        if m.group('raw') is not None:
            close = '"' + m.group('h')
            j = s.find(close, m.end())
            out.append(('str', i, j + len(close))); i = j + len(close); continue
        if m.group('bc') is not None:
            d, j = 1, i + 2
            while j < n and d:
                if s.startswith('/*', j): d += 1; j += 2
                elif s.startswith('*/', j): d -= 1; j += 2
                else: j += 1
            i = j; continue
        if m.group('ws') is None and m.group('lc') is None and m.group('life') is None:
            kind = 'id' if m.group('id') is not None else ('str' if m.group('str') is not None else ('chr' if m.group('chr') is not None else 'p'))
            out.append((kind, i, m.end()))
        i = m.end()
    return out


def fns(s):
    """Return list of dicts for `unsafe fn NAME ... { body }` items with bodies."""
    toks = lex(s)
    txt = [s[a:b] for _, a, b in toks]
    # macro_rules! bodies are excluded
    macro_spans = []
    for k in range(len(toks) - 2):
        if txt[k] == 'macro_rules' and txt[k + 1] == '!':
            j = k + 3
            while j < len(toks) and txt[j] not in '{([': j += 1
            d, e = 0, j
            while e < len(toks):
                if txt[e] in '{([': d += 1
                elif txt[e] in '})]': d -= 1
                if d == 0: break
                e += 1
            macro_spans.append((toks[j][1], toks[min(e, len(toks) - 1)][2]))
    out = []
    for k in range(len(toks) - 2):
        if not (txt[k] == 'unsafe' and txt[k + 1] == 'fn' and toks[k + 2][0] == 'id'):
            continue
        start = toks[k][1]
        if any(a <= start < b for a, b in macro_spans): continue
        # signature: until '{' or ';' at paren/bracket depth 0
        j, d = k + 3, 0
        while j < len(toks):
            t = txt[j]
            if t in '([': d += 1
            elif t in ')]': d -= 1
            elif d == 0 and t in '{;': break
            elif d == 0 and t == '=' and j + 1 < len(toks) and txt[j + 1] == '>': break
            j += 1
        if j >= len(toks) or txt[j] != '{': continue
        sig = s[start:toks[j][1]]
        e, d = j, 0
        while e < len(toks):
            if txt[e] == '{': d += 1
            elif txt[e] == '}':
                d -= 1
                if d == 0: break
            e += 1
        body_toks = txt[j:e + 1]
        # preceding qualifiers/attributes on the same item
        ls = s.rfind('\n', 0, start) + 1
        prefix = s[ls:start]
        attrs = []
        p = ls
        while p > 0:
            pl = s.rfind('\n', 0, p - 1) + 1
            line = s[pl:p - 1].strip()
            if line.startswith('#[') or line.startswith('///') or line.startswith('//') or (line.endswith(']') and not line.endswith('];')):
                attrs.append(line); p = pl
            else:
                break
        out.append(dict(name=txt[k + 2], kw=start, line=s.count('\n', 0, start) + 1,
                        sig=sig, prefix=prefix, attrs=' '.join(attrs),
                        body_unsafe=any(a == 'unsafe' and b in ('{', 'impl') for a, b in zip(body_toks, body_toks[1:])),
                        lo=s.count('\n', 0, start) + 1, hi=s.count('\n', 0, toks[e][2]) + 1))
    return out


def files_compiled():
    fs = set()
    for d in glob.glob('target/debug/deps/sm-*.d') + glob.glob('target/vcheck/debug/deps/sm*.d') + glob.glob('target/vcheck/debug/examples/sm_verify*.d'):
        for line in open(d):
            if ':' not in line: continue
            for tok in line.split(':', 1)[1].split():
                tok = tok.rstrip('\\')
                if tok.endswith('.rs'):
                    p = os.path.relpath(os.path.abspath(tok), ROOT)
                    if p.startswith('sm/src/'): fs.add(p)
    return sorted(fs)


def dep_files(pattern):
    fs = set()
    for d in glob.glob(pattern):
        for line in open(d):
            if ':' not in line: continue
            for tok in line.split(':', 1)[1].split():
                tok = tok.rstrip('\\')
                if tok.endswith('.rs'):
                    fs.add(os.path.relpath(os.path.abspath(tok), ROOT))
    return fs


DEPS = ['target/debug/deps/sm-*.d', 'target/vcheck/debug/*/sm*.d']


def check():
    # A block is unused only if every configuration compiling its file says so.
    errs, per = [], []
    for (cmd, env), dep in zip(CHECKS, DEPS):
        unused = []
        r = subprocess.run(cmd, capture_output=True, text=True, env=dict(os.environ, **env))
        for line in r.stdout.split('\n'):
            try: m = json.loads(line)
            except Exception: continue
            if m.get('reason') != 'compiler-message': continue
            msg = m['message']
            lvl = msg['level']
            prim = [sp for sp in msg['spans'] if sp['is_primary']]
            if lvl == 'error':
                for sp in prim:
                    names = set(re.findall(r'\w+', sp.get('text', [{}])[0].get('text', '') if sp.get('text') else ''))
                    names |= set(re.findall(r'`([^`]*)`', msg['message']))
                    while sp.get('expansion'): sp = sp['expansion']['span']
                    errs.append((os.path.relpath(os.path.abspath(sp['file_name']), ROOT), sp['line_start'], names, msg['message']))
            elif lvl == 'warning':
                code = (msg.get('code') or {}).get('code')
                for sp in prim:
                    if code == 'unused_unsafe' and not sp.get('expansion'):
                        unused.append((os.path.relpath(os.path.abspath(sp['file_name']), ROOT), sp['line_start'], sp['column_start']))
        if r.returncode != 0 and not errs:
            errs.append(('?', 0, set(), r.stderr[-2000:]))
        per.append((set(unused), dep_files(dep)))
    alls = set().union(*[u for u, _ in per])
    ok = [x for x in alls if all(x in u for u, fs in per if x[0] in fs)]
    return errs, sorted(ok)


def warnings_count():
    r = subprocess.run(CHECKS[0][0][:-2], capture_output=True, text=True)
    m = re.search(r'generated (\d+) warnings? \((\d+) duplicates\)', r.stderr) or re.search(r'generated (\d+) warnings?', r.stderr)
    return (int(m.group(1)) - (int(m.group(2)) if m.lastindex == 2 else 0)) if m else 0


def changed_lines():
    out, cur = set(), None
    for line in subprocess.run(['git', 'diff', '-U0', '--', 'sm/src'], capture_output=True, text=True).stdout.split('\n'):
        if line.startswith('+++ b/'): cur = line[6:]
        m = re.match(r'@@ -\S+ \+(\d+)(?:,(\d+))? @@', line)
        if m and cur:
            a, n = int(m.group(1)), int(m.group(2) or 1)
            out |= {(cur, k) for k in range(a, a + n)}
    return out


def body_blocks(s):
    # (open_line, close_line) of a fn body's sole inner block: `fn f() { { .. } }`.
    toks = lex(s); txt = [s[a:b] for _, a, b in toks]
    out = []
    for k in range(len(toks) - 1):
        if txt[k] != 'fn' or toks[k + 1][0] != 'id': continue
        j, d = k + 2, 0
        while j < len(toks):
            if txt[j] in '([': d += 1
            elif txt[j] in ')]': d -= 1
            elif d == 0 and txt[j] in '{;': break
            j += 1
        if j + 1 >= len(toks) or txt[j] != '{' or txt[j + 1] != '{': continue
        e, d = j + 1, 0
        while e < len(toks):
            if txt[e] == '{': d += 1
            elif txt[e] == '}':
                d -= 1
                if d == 0: break
            e += 1
        if e + 1 < len(toks) and txt[e + 1] == '}':
            out.append((s.count('\n', 0, toks[j + 1][1]) + 1, s.count('\n', 0, toks[e][1]) + 1))
    return out


def flatten(files):
    # An unwrapped whole-body `unsafe {` leaves `fn f() { { .. } }`; drop the
    # inner braces when they sit alone on lines this codemod changed.
    mine = changed_lines(); n = 0
    for fl in files:
        s = open(fl).read(); L = s.split('\n'); drop = set()
        for a, b in body_blocks(s):
            if (fl, a) in mine and L[a - 1].strip() == '{' and L[b - 1].strip() == '}':
                drop |= {a - 1, b - 1}
        if drop:
            open(fl, 'w').write('\n'.join(x for i, x in enumerate(L) if i not in drop)); n += len(drop) // 2
    errs, _ = check()
    assert not errs, errs[:3]
    print(f'flattened {n} fn bodies', flush=True)


def fix_braces():
    # `return unsafe { x }` unwrapped to `return { x }`: apply rustc's own
    # unused_braces suggestion, but only on lines this codemod changed.
    mine = changed_lines()
    r = subprocess.run(CHECKS[0][0], capture_output=True, text=True)
    edits = {}
    for line in r.stdout.split('\n'):
        try: m = json.loads(line)
        except Exception: continue
        if m.get('reason') != 'compiler-message': continue
        msg = m['message']
        if (msg.get('code') or {}).get('code') != 'unused_braces': continue
        prim = [sp for sp in msg['spans'] if sp['is_primary']]
        if not prim or prim[0].get('expansion'): continue
        fl = os.path.relpath(os.path.abspath(prim[0]['file_name']), ROOT)
        if (fl, prim[0]['line_start']) not in mine: continue
        for ch in msg['children']:
            for sp in ch['spans']:
                if sp.get('suggested_replacement') is not None:
                    edits.setdefault(fl, set()).add((sp['byte_start'], sp['byte_end'], sp['suggested_replacement']))
    for fl, es in edits.items():
        b = open(fl, 'rb').read()
        for a, e, rep in sorted(es, reverse=True):
            b = b[:a] + rep.encode() + b[e:]
        open(fl, 'wb').write(b)
    print(f'unused_braces fixed in {sum(len(v) for v in edits.values())} spans', flush=True)


EXPAND = [
    (['cargo', 'rustc', '-p', 'sm', '--lib', '--no-default-features', '--target-dir',
      'target/expand', '--', '-Zunpretty=expanded'], {}),
    (['cargo', 'rustc', '-p', 'sm', '--lib', '--target-dir', 'target/expand-verify', '--',
      '-Zunpretty=expanded'], {'RUSTFLAGS': '--cfg sm_verify_dll'}),
]


IMPLICIT = {'Drop', 'Deref', 'DerefMut', 'Index', 'IndexMut', 'Iterator', 'IntoIterator',
            'PartialEq', 'Eq', 'PartialOrd', 'Ord', 'Display', 'Debug', 'Hash', 'Default',
            'Add', 'Sub', 'Mul', 'Div', 'Rem', 'Neg', 'Not', 'BitAnd', 'BitOr', 'BitXor', 'Shl',
            'Shr', 'AddAssign', 'SubAssign', 'MulAssign', 'DivAssign', 'RemAssign',
            'BitAndAssign', 'BitOrAssign', 'BitXorAssign', 'ShlAssign', 'ShrAssign',
            'Fn', 'FnMut', 'FnOnce', 'Future', 'From', 'Into', 'TryFrom', 'Write'}


PRIMS = {'u8', 'u16', 'u32', 'u64', 'u128', 'usize', 'i8', 'i16', 'i32', 'i64', 'i128',
         'isize', 'bool', 'char', 'str', 'f32', 'f64', 'self', 'Self', 'crate', 'super'}


def bindings(txt, a, b, sig=None):
    # Lowercase names bound by let/for/closure/match patterns and fn params.
    out = set()
    def take(lo, hi):
        for q in range(lo, hi):
            x = txt[q]
            if x[:1].islower() or x[:1] == '_':
                if q + 1 < hi and txt[q + 1] in ('(', '{') and q > lo and txt[q - 1] != '|': continue
                if q + 1 < hi and txt[q + 1] == ':' and q + 2 < hi and txt[q + 2] == ':': continue
                if q > lo and txt[q - 1] == ':' and q > lo + 1 and txt[q - 2] == ':': continue
                out.add(x)
    if sig:
        lo, hi = sig
        d = 0
        for q in range(lo, hi):
            if txt[q] == '(':
                d += 1
            elif txt[q] == ')':
                d -= 1
            elif d == 1 and txt[q + 1] == ':' and txt[q + 2] != ':' and (txt[q - 1] in '(,' or txt[q - 1] == 'mut'):
                out.add(txt[q])
    q = a
    while q < b:
        t = txt[q]
        if t == 'let' or t == 'for':
            stop = '=' if t == 'let' else 'in'
            e, d = q + 1, 0
            while e < b:
                if txt[e] in '([{': d += 1
                elif txt[e] in ')]}': d -= 1
                elif d == 0 and (txt[e] == stop or txt[e] == ';' or (t == 'let' and txt[e] == ':')): break
                e += 1
            take(q + 1, e)
        elif t == '|' and q + 1 < b and (txt[q - 1] in ('(', ',', '=', 'move', '{', ';') ):
            e = q + 1
            while e < b and txt[e] != '|': e += 1
            take(q + 1, e)
        elif t == '=' and q + 1 < b and txt[q + 1] == '>':
            e, d = q - 1, 0
            while e > a:
                if txt[e] in ')]}': d += 1
                elif txt[e] in '([{':
                    if d == 0: break
                    d -= 1
                elif d == 0 and txt[e] == ',': break
                e -= 1
            take(e + 1, q)
        q += 1
    return out


def idents(txt, a, b, sig=None):
    # Identifiers that may name an item. Skipped as name collisions only: field
    # accesses `.x` (no call), field names `x:`, primitive types, and bare uses
    # of names the body binds locally, and module/type path prefixes `x::`.
    # Calls `x(`, methods `.x(`, last path segments and bare names count.
    bound = bindings(txt, a, b, sig)
    out = set()
    for q in range(a, b):
        x = txt[q]
        if not (x[:1].isalpha() or x[:1] == '_'): continue
        nxt = txt[q + 1] if q + 1 < len(txt) else ''
        prev = txt[q - 1] if q > 0 else ''
        call = nxt == '(' or (nxt == ':' and q + 3 < len(txt) and txt[q + 2] == ':' and txt[q + 3] == '<')
        path = prev == ':' and q > 1 and txt[q - 2] == ':'
        if prev == '.' and nxt != '(' and nxt != ':': continue
        if nxt == ':' and q + 2 < len(txt) and txt[q + 2] == ':' and not call: continue  # path prefix
        if nxt == ':' and not (q + 2 < len(txt) and txt[q + 2] == ':') and prev in ('{', ','): continue
        if x in PRIMS and not ((prev == '.' or path) and nxt == '('): continue
        if x in bound and not call and not path and prev != '.': continue
        out.add(x)
    return out


def tainted_names():
    # Transitive closure over macro-expanded source, by name (conservative):
    # a name is tainted if any item so named contains an `unsafe {}` block or
    # mentions a tainted name. Items: fns (incl. methods/trait impls), statics
    # and consts (fn tables). A type name links to every method implemented on
    # it through an implicitly called std trait (Drop, Deref, Index, operators,
    # formatting, comparison, conversion); other methods are called by name.
    # Expansion needs RUSTC_BOOTSTRAP for -Zunpretty; it is analysis only.
    refs, seeds = {}, set()
    for cmd, env in EXPAND:
        r = subprocess.run(cmd, capture_output=True, text=True,
                           env=dict(os.environ, RUSTC_BOOTSTRAP='1', **env))
        if r.returncode != 0: sys.exit('expansion failed: ' + r.stderr[-2000:])
        src = r.stdout; toks = lex(src); txt = [src[a:b] for _, a, b in toks]
        n = len(toks)
        def block_end(j):
            d = 0
            for e in range(j, n):
                if txt[e] == '{': d += 1
                elif txt[e] == '}':
                    d -= 1
                    if d == 0: return e
            return n - 1
        for k in range(n - 1):
            t = txt[k]
            if t == 'fn' and toks[k + 1][0] == 'id':
                j, d = k + 2, 0
                while j < n:
                    if txt[j] in '([': d += 1
                    elif txt[j] in ')]': d -= 1
                    elif d == 0 and txt[j] in '{;': break
                    j += 1
                if j >= n or txt[j] != '{': continue
                e = block_end(j); body = txt[j:e + 1]; name = txt[k + 1]
                refs.setdefault(name, set()).update(idents(txt, j, e + 1, (k + 2, j)))
                if any(a == 'unsafe' and b == '{' for a, b in zip(body, body[1:])): seeds.add(name)
            elif t in ('static', 'const') and toks[k + 1][0] == 'id' and txt[k + 1] not in ('fn', 'unsafe', 'extern'):
                j = k + 1
                if txt[j] == 'mut': j += 1
                name = txt[j]; e = j
                d = 0
                while e < n:
                    if txt[e] in '([{': d += 1
                    elif txt[e] in ')]}': d -= 1
                    elif d == 0 and txt[e] == ';': break
                    if d < 0: break
                    e += 1
                body = txt[j + 1:e]
                refs.setdefault(name, set()).update(idents(txt, j + 1, e))
                if any(a == 'unsafe' and b == '{' for a, b in zip(body, body[1:])): seeds.add(name)
            elif t == 'impl':
                j = k + 1
                while j < n and txt[j] != '{' and txt[j] != ';': j += 1
                if j >= n or txt[j] != '{': continue
                head = [x for x in txt[k + 1:j] if x[:1].isalpha() or x[:1] == '_']
                if 'for' not in head: continue  # inherent methods are called by name
                if not IMPLICIT & set(head[:head.index('for')]): continue
                head = head[head.index('for') + 1:]
                head = [x for x in head if x not in ('where', 'dyn', 'mut', 'const')]
                if not head: continue
                ty = head[0]
                e = block_end(j)
                meths = {txt[q + 1] for q in range(j, e) if txt[q] == 'fn' and toks[q + 1][0] == 'id'}
                refs.setdefault(ty, set()).update(meths)
    bad = set(seeds); rev = {}
    for a, bs in refs.items():
        for b in bs: rev.setdefault(b, set()).add(a)
    stack = list(bad)
    while stack:
        b = stack.pop()
        for a in rev.get(b, ()):
            if a not in bad: bad.add(a); stack.append(a)
    return bad


def main():
    errs, _ = check()
    if errs: sys.exit(f'baseline does not compile: {errs[:3]}')
    files = files_compiled()
    base_warn = warnings_count()
    print(f'{len(files)} compiled files; baseline warnings {base_warn}', flush=True)
    keep = set()  # (file, line) of rejected fns
    total_stripped = 0
    for rnd in range(60):
        texts = {p: open(p).read() for p in files}
        bad = tainted_names()
        cands = {}
        for p, s in texts.items():
            for f in fns(s):
                if (p, f['line']) in keep or f['body_unsafe'] or f['name'] in bad: continue
                if RAW.search(f['sig']) or BAD_ATTR.search(f['attrs']) or 'extern' in f['prefix'] or 'extern' in f['sig']:
                    continue
                cands.setdefault(p, []).append(f)
        n = sum(len(v) for v in cands.values())
        print(f'round {rnd}: {n} candidates', flush=True)
        stripped = {}
        if n:
            stripped = {p: list(v) for p, v in cands.items()}
            for it in range(40):
                for p, s in texts.items():
                    s2 = s
                    for f in sorted(stripped.get(p, []), key=lambda f: -f['kw']):
                        assert s2.startswith('unsafe fn', f['kw'])
                        s2 = s2[:f['kw']] + s2[f['kw'] + 7:]
                    open(p, 'w').write(s2)
                t = time.time(); errs, _ = check()
                if not errs: break
                rej = set()
                for fl, ln, names, msg in errs:
                    hit = [f for f in stripped.get(fl, []) if f['lo'] <= ln <= f['hi']]
                    if not hit:
                        hit = [f for v in stripped.values() for f in v if f['name'] in names]
                    if not hit and fl in stripped:
                        hit = list(stripped[fl])
                    if not hit:
                        hit = [f for v in stripped.values() for f in v]
                        print('  unattributed error, rejecting all:', fl, ln, msg[:200], flush=True)
                    for f in hit: rej.add(id(f))
                for p in list(stripped):
                    for f in stripped[p]:
                        if id(f) in rej: keep.add((p, f['line']))
                    stripped[p] = [f for f in stripped[p] if id(f) not in rej]
                print(f'  iter {it}: {len(errs)} errors, rejected {len(rej)}, {time.time() - t:.0f}s', flush=True)
            else:
                sys.exit('no convergence')
        ok = sum(len(v) for v in stripped.values())
        total_stripped += ok
        # unwrap unused unsafe blocks
        errs, unused = check()
        assert not errs, errs[:3]
        fixed = 0
        by_file = {}
        for fl, ln, col in unused:
            by_file.setdefault(fl, []).append((ln, col))
        for fl, locs in by_file.items():
            if fl not in texts: continue
            lines = open(fl).read().split('\n')
            for ln, col in locs:
                L = lines[ln - 1]; c = col - 1
                if L[c:c + 6] == 'unsafe':
                    rest = L[c + 6:]
                    lines[ln - 1] = L[:c] + rest.lstrip(' ') if rest.lstrip(' ').startswith('{') else L
                    fixed += lines[ln - 1] != L
            open(fl, 'w').write('\n'.join(lines))
        errs, _ = check()
        if errs:  # revert unwraps in files with errors
            bad = {e[0] for e in errs}
            print('  unwrap errors in', bad, flush=True)
            for fl in bad:
                if fl in texts:
                    cur = open(fl).read().split('\n')
                    orig = texts[fl].split('\n')
                    # keep fn strips, restore blocks: reapply strips on original text
                    s0 = texts[fl]
                    for f in sorted(stripped.get(fl, []), key=lambda f: -f['kw']):
                        s0 = s0[:f['kw']] + s0[f['kw'] + 7:]
                    open(fl, 'w').write(s0)
                    fixed -= sum(1 for a, b in zip(cur, s0.split('\n')) if a != b)
            errs, _ = check()
        assert not errs, errs[:3]
        print(f'round {rnd}: stripped {ok}, unwrapped {fixed} unused unsafe blocks', flush=True)
        if ok == 0 and fixed == 0: break
    flatten(files)
    fix_braces()
    subprocess.run(['cargo', 'fmt'], check=True)
    errs, _ = check(); assert not errs, errs[:3]
    print(f'total fns made safe: {total_stripped}; rejected: {len(keep)}; warnings {base_warn} -> {warnings_count()}')


if __name__ == '__main__':
    main()
