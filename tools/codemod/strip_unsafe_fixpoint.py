# Usage (repo root): python3 tools/codemod/strip_unsafe_fixpoint.py
# Windows cached-metadata alternative: --cached-gl PATH --cached-sdl2 PATH.
# Both files must be real compatible dependency metadata; rustc validates them.
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
# Imported aliases are conservatively tainted regardless of scope or target;
# mentions in signatures and module prefixes count, including alias chains.
# Line counts never change before the final fmt, so (file, line) keys are stable.
import argparse, difflib, glob, json, os, re, subprocess, sys, time

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


def source_path(path):
    return os.path.relpath(os.path.abspath(path), ROOT).replace(os.sep, '/')


def write_source(path, text):
    # Rust source in this checkout may use LF or CRLF. Preserve its style so
    # a no-op compiler trial does not rewrite every compiled file's bytes.
    with open(path, 'r', newline='') as original:
        first = original.readline()
    newline = '\r\n' if first.endswith('\r\n') else '\n'
    with open(path, 'w', newline=newline) as output:
        output.write(text)


def dep_files(pattern):
    fs = set()
    for d in glob.glob(pattern):
        for line in open(d):
            # The rule separator has trailing whitespace (or ends the line);
            # a Windows drive colon is part of the path, not the separator.
            rule = re.split(r':(?:\s+|$)', line, maxsplit=1)
            if len(rule) != 2: continue
            for tok in re.findall(r'(?:\\ |[^\s])+', rule[1]):
                tok = tok.replace('\\ ', ' ')
                if tok.endswith('.rs'):
                    p = source_path(tok)
                    if p.startswith('sm/src/'): fs.add(p)
    return fs


DEPS = ['target/debug/deps/sm-*.d', 'target/vcheck/debug/*/sm*.d']
CACHED_FRONTEND = None
ORIGINALS = {}


def files_compiled():
    groups = [dep_files(pattern) for pattern in DEPS]
    if not all(groups): sys.exit('missing compiled source coverage for a configuration')
    return sorted(set().union(*groups))


def direct_command(index, example=False, expand=False):
    # Explicit opt-in for the documented Windows cached-metadata path. These
    # are real rustc checks; never execute frontend dependency build scripts.
    directory = f'target/codemod/{index}'
    os.makedirs(directory, exist_ok=True)
    name = 'sm_verify' if example else 'sm'
    src = 'sm/src/verify_dll.rs' if example else 'sm/src/lib.rs'
    cmd = ['rustc', '--crate-name', name, '--edition=2024', src,
           '--crate-type', 'cdylib' if example else 'lib',
           '--check-cfg', 'cfg(sm_verify_dll)',
           '--check-cfg', 'cfg(feature, values("default", "frontend"))']
    if index:
        cmd += ['--cfg', 'feature="default"', '--cfg', 'feature="frontend"',
                '--cfg', 'sm_verify_dll']
        for crate, path in zip(('gl', 'sdl2_sys'), CACHED_FRONTEND):
            cmd += ['--extern', f'{crate}={path}', '-L', f'dependency={os.path.dirname(path)}']
    if example:
        cmd += ['--extern', f'sm={directory}/libsm.rmeta']
    if expand:
        cmd += ['-Zunpretty=expanded']
    else:
        cmd += ['--emit', f'metadata={directory}/lib{name}.rmeta,dep-info={directory}/{name}.d',
                '--error-format=json']
    return cmd


def run_check(index):
    if CACHED_FRONTEND is None:
        cmd, env = CHECKS[index]
        return subprocess.run(cmd, capture_output=True, text=True, env=dict(os.environ, **env))
    results = []
    for example in ([False, True] if index else [False]):
        cmd = direct_command(index, example)
        r = subprocess.run(cmd, capture_output=True, text=True)
        results.append(r)
        if r.returncode: break
    return subprocess.CompletedProcess(results[-1].args, results[-1].returncode,
                                       ''.join(r.stdout for r in results),
                                       ''.join(r.stderr for r in results))


def diagnostics(result):
    # Cargo wraps diagnostics on stdout; direct rustc emits them on stderr.
    for line in (result.stdout + '\n' + result.stderr).splitlines():
        try: msg = json.loads(line)
        except ValueError: continue
        if msg.get('reason') == 'compiler-message': yield msg['message']
        elif msg.get('$message_type') == 'diagnostic': yield msg


def check(report_failure=False, results=None):
    # A block is unused only if every configuration compiling its file says so.
    errs, per = [], []
    for index, dep in enumerate(DEPS):
        unused = []
        r = run_check(index)
        if results is not None:
            results.append(r)
        if report_failure and r.returncode:
            print(r.stdout, file=sys.stderr, end='')
            print(r.stderr, file=sys.stderr, end='')
        before = len(errs)
        for msg in diagnostics(r):
            lvl = msg['level']
            prim = [sp for sp in msg['spans'] if sp['is_primary']]
            if lvl == 'error':
                for sp in prim:
                    names = set(re.findall(r'\w+', sp.get('text', [{}])[0].get('text', '') if sp.get('text') else ''))
                    names |= set(re.findall(r'`([^`]*)`', msg['message']))
                    while sp.get('expansion'): sp = sp['expansion']['span']
                    errs.append((source_path(sp['file_name']), sp['line_start'], names, msg['message']))
            elif lvl == 'warning':
                code = (msg.get('code') or {}).get('code')
                for sp in prim:
                    if code == 'unused_unsafe' and not sp.get('expansion'):
                        unused.append((source_path(sp['file_name']), sp['line_start'], sp['column_start']))
        if r.returncode != 0 and len(errs) == before:
            errs.append(('?', 0, set(), r.stderr[-2000:]))
        compiled = dep_files(dep)
        expected = {'sm/src/lib.rs'} | ({'sm/src/verify_dll.rs'} if index else set())
        if not r.returncode and not expected <= compiled:
            sys.exit(f'configuration {index}: missing dep-info coverage: {expected - compiled}')
        if not r.returncode and any(fl not in compiled for fl, _, _ in unused):
            sys.exit(f'configuration {index}: warning file absent from dep-info')
        per.append((set(unused), compiled))
    alls = set().union(*[u for u, _ in per])
    ok = [x for x in alls if all(x in u for u, fs in per if x[0] in fs)]
    return errs, sorted(ok)


def warnings_count(result=None):
    r = run_check(0) if result is None else result
    if r.returncode: sys.exit('warning-count compilation failed')
    return sum(m['level'] == 'warning' and bool(m.get('spans')) for m in diagnostics(r))


def changed_lines():
    # Scope cleanup to this invocation, including isolated source snapshots.
    out = set()
    for path, original in ORIGINALS.items():
        current = open(path).read().splitlines()
        for tag, _, _, start, end in difflib.SequenceMatcher(
                a=original.splitlines(), b=current, autojunk=False).get_opcodes():
            if tag != 'equal': out.update((path, n + 1) for n in range(start, end))
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
            write_source(fl, '\n'.join(x for i, x in enumerate(L) if i not in drop)); n += len(drop) // 2
    errs, _ = check()
    assert not errs, errs[:3]
    print(f'flattened {n} fn bodies', flush=True)


def fix_braces():
    # `return unsafe { x }` unwrapped to `return { x }`: apply rustc's own
    # unused_braces suggestion, but only on lines this codemod changed.
    mine = changed_lines()
    r = run_check(0)
    if r.returncode: sys.exit('brace-check compilation failed')
    edits = {}
    for msg in diagnostics(r):
        if (msg.get('code') or {}).get('code') != 'unused_braces': continue
        prim = [sp for sp in msg['spans'] if sp['is_primary']]
        if not prim or prim[0].get('expansion'): continue
        fl = source_path(prim[0]['file_name'])
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
    (['cargo', 'rustc', '-p', 'sm', '--example', 'sm_verify', '--target-dir',
      'target/expand-verify', '--', '-Zunpretty=expanded'],
     {'RUSTFLAGS': '--cfg sm_verify_dll'}),
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


def imported_aliases(txt):
    # Reject every alias spelling, irrespective of scope or target. This also
    # covers grouped imports, `self as module`, re-exports and alias chains.
    # Module aliases must count even in a path prefix normally skipped below.
    aliases = set()
    for k, token in enumerate(txt):
        if token != 'use' and not (token == 'extern' and txt[k + 1:k + 2] == ['crate']): continue
        end = k + 1
        while end < len(txt) and txt[end] != ';': end += 1
        aliases.update(txt[q + 1] for q in range(k + 1, end - 1) if txt[q] == 'as')
    return aliases


def tainted_names():
    # Transitive closure over macro-expanded source, by name (conservative):
    # a name is tainted if any item so named contains an `unsafe {}` block or
    # mentions a tainted name. Items: fns (incl. methods/trait impls), statics
    # and consts (fn tables). A type name links to every method implemented on
    # it through an implicitly called std trait (Drop, Deref, Index, operators,
    # formatting, comparison, conversion); other methods are called by name.
    # Expansion needs RUSTC_BOOTSTRAP for -Zunpretty; it is analysis only.
    refs, seeds = {}, set()
    expansions = EXPAND
    if CACHED_FRONTEND is not None:
        expansions = [(direct_command(i, example, expand=True), {})
                      for i, example in ((0, False), (1, False), (1, True))]
    for cmd, env in expansions:
        r = subprocess.run(cmd, capture_output=True, text=True,
                           env=dict(os.environ, RUSTC_BOOTSTRAP='1', **env))
        if r.returncode != 0: sys.exit('expansion failed: ' + r.stderr[-2000:])
        src = r.stdout; toks = lex(src); txt = [src[a:b] for _, a, b in toks]
        aliases = imported_aliases(txt)
        seeds.update(aliases)
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
                refs[name].update(aliases.intersection(txt[k + 2:e + 1]))
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
                refs[name].update(aliases.intersection(txt[j + 1:e]))
                if any(a == 'unsafe' and b == '{' for a, b in zip(body, body[1:])): seeds.add(name)
            elif t == 'impl':
                j = k + 1
                while j < n and txt[j] != '{' and txt[j] != ';': j += 1
                if j >= n or txt[j] != '{': continue
                head = [x for x in txt[k + 1:j] if x[:1].isalpha() or x[:1] == '_']
                if 'for' not in head: continue  # inherent methods are called by name
                if not (IMPLICIT | aliases) & set(head[:head.index('for')]): continue
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
    global CACHED_FRONTEND, DEPS
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument('--cached-gl', help='existing gl rmeta for direct metadata checks')
    parser.add_argument('--cached-sdl2', help='matching sdl2_sys rmeta; no dependency rebuild')
    args = parser.parse_args()
    if bool(args.cached_gl) != bool(args.cached_sdl2): parser.error('both cached metadata paths are required')
    if args.cached_gl:
        CACHED_FRONTEND = tuple(os.path.abspath(p) for p in (args.cached_gl, args.cached_sdl2))
        if not all(os.path.isfile(p) for p in CACHED_FRONTEND): parser.error('cached metadata file missing')
        DEPS = ['target/codemod/0/*.d', 'target/codemod/1/*.d']
    check_results = []
    errs, _ = check(report_failure=True, results=check_results)
    if errs: sys.exit(f'baseline does not compile: {errs[:3]}')
    files = files_compiled()
    ORIGINALS.update({p: open(p).read() for p in files})
    base_warn = warnings_count(check_results[0])
    del check_results
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
                    write_source(p, s2)
                t = time.time(); errs, unused = check()
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
        # The accepted trial already checked this source in both configurations.
        if not n:
            errs, unused = check()
        # unwrap unused unsafe blocks
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
            write_source(fl, '\n'.join(lines))
        if by_file:  # Empty means no source writes were attempted.
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
                    write_source(fl, s0)
                    fixed -= sum(1 for a, b in zip(cur, s0.split('\n')) if a != b)
            errs, _ = check()
        assert not errs, errs[:3]
        print(f'round {rnd}: stripped {ok}, unwrapped {fixed} unused unsafe blocks', flush=True)
        if ok == 0 and fixed == 0: break
    flatten(files)
    fix_braces()
    subprocess.run(['cargo', 'fmt'], check=True)
    check_results = []
    errs, _ = check(results=check_results); assert not errs, errs[:3]
    print(f'total fns made safe: {total_stripped}; rejected: {len(keep)}; warnings {base_warn} -> {warnings_count(check_results[0])}')


if __name__ == '__main__':
    main()
