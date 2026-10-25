# Usage (repo root): python3 tools/codemod/enemy_slots.py sm/src/game/sm_a4.rs [...]
#
# Lane M2: replace raw enemy-record getters `Get_X(slot, game) -> *mut L` with
# checked `slot!` accesses (ram_map.rs). Converted forms:
#   let E = Get_X(ARG, game);  -> let E: i32 = ARG;      (only if every use of E
#                                 in its scope is one of the field forms below)
#   (*E).a.b            read   -> slot!(game, L[E].a.b)
#   (*E).a.b = RHS;     store  -> slot!(game, L[E].a.b = RHS);
#   (*E).a.b OP= RHS;   (+ - | & ^) store of the old value OP RHS, RHS first
#   byte!((*E).a.b; N)  [= RHS] -> slot!(byte game, L[E].a.b[N] [= RHS])
#   (*Get_X(ARG, game)).a.b ... the same, with ARG in place of E.
# Anything else (&raw, macro arguments, whole-struct copies, E passed on) leaves
# that binding or occurrence untouched. Rejected edits are reverted by compiler
# error location until both configurations compile; then unused getters are
# removed and `cargo fmt` runs. Run strip_unsafe_fixpoint.py afterwards.
import os, re, subprocess, sys
sys.path.insert(0, os.path.dirname(__file__))
from strip_unsafe_fixpoint import lex, check, CHECKS

GETTER = re.compile(r'^\s*(Get_\w+)\(o: i32\) -> (\w+) = \[0xf78, o as isize\];\s*$', re.M)
BYTE_FIELDS = {'bank', 'field_2F'}
PLACE_MACROS = {'update_place', 'words'}  # bodies that take places, not values


def plan(s, getters):
    toks = lex(s); txt = [s[a:b] for _, a, b in toks]; n = len(toks)
    T = lambda j: txt[j] if 0 <= j < n else ''
    def close(j):  # index of token closing the bracket opened at j
        d = 0
        for e in range(j, n):
            if txt[e] in '([{': d += 1
            elif txt[e] in ')]}':
                d -= 1
                if d == 0: return e
        return None
    def stmt_end(j):  # first ';' at depth 0 from j
        d = 0
        for e in range(j, n):
            if txt[e] in '([{': d += 1
            elif txt[e] in ')]}':
                d -= 1
                if d < 0: return None
            elif d == 0 and txt[e] == ';': return e
        return None
    def split(a, b):  # top-level comma-separated token ranges in [a, b)
        out, d, st = [], 0, a
        for e in range(a, b):
            if txt[e] in '([{': d += 1
            elif txt[e] in ')]}': d -= 1
            elif d == 0 and txt[e] == ',':
                out.append((st, e)); st = e + 1
        if st < b: out.append((st, b))
        return out
    span = lambda a, b: (toks[a][1], toks[b - 1][2])  # source span of tokens [a, b)
    def field_path(j):  # tokens from j: . f (. f)*
        path, e = [], j
        while e + 1 < n and txt[e] == '.' and toks[e + 1][0] == 'id' and T(e + 2) != '(':
            path.append(txt[e + 1]); e += 2
        return path, e  # e: index after path
    stmt_start = lambda j: T(j - 1) in (';', '{', '}')
    def enclosing_macro(j):  # name of the innermost macro invocation around token j
        d = 0
        for e in range(j - 1, -1, -1):
            if txt[e] in ')]}': d += 1
            elif txt[e] in '([{':
                if d == 0:
                    if T(e - 1) == '!' and toks[e - 2][0] == 'id': return T(e - 2)
                    if txt[e] == '{' and T(e - 1) in ('before', 'prefix'): continue
                    if txt[e] == '{': return None
                else: d -= 1
        return None

    def store(target, byte_field, value, lane=None):
        if byte_field or lane is not None:
            return [f'slot!(byte game, {target}[{lane or 0}] = ', value, ')']
        return [f'slot!(game, {target} = ', value, ')']

    def access(open_paren, inner_end, layout, slot):
        """`( * X )` at tokens open_paren..inner_end; returns (start, end, parts) or None."""
        path, after = field_path(inner_end + 1)
        start = toks[open_paren][1]
        base = f'{layout}[{slot}]'
        # macro receivers: name! {[(*E)] ...} / name! {[(*E).base] ...}
        if T(open_paren - 1) == '[' and T(open_paren - 2) == '{' and T(open_paren - 3) == '!':
            name = T(open_paren - 4)
            if T(after) != ']' or path not in ([], ['base']): return None
            body_close = close(open_paren - 2)
            prefix = '.'.join(path + [''])
            a0, end = toks[open_paren - 4][1], toks[body_close][2]
            parts = []
            def done(parts):  # statements; keep one `;` before an existing one
                parts = parts[:-1]
                return (a0, end, parts + ([] if T(body_close + 1) == ';' else [';']))
            stores = []  # (field, value span) in order
            if name == 'enemy_instruction' and path == ['base'] and T(after + 1) == ';':
                fields = {'instruction': 'current_instruction', 'timer': 'instruction_timer'}
                for x, y in split(after + 2, body_close):
                    if y - x < 3 or T(x + 1) != '=' or T(x) not in fields: return None
                    stores.append((fields[T(x)], span(x + 2, y)))
            elif name == 'set_fields' and T(after + 1) == ';':
                for x, y in split(after + 2, body_close):
                    if y - x < 3 or T(x + 1) != '=' or toks[x][0] != 'id': return None
                    stores.append((T(x), span(x + 2, y)))
            elif name == 'set_fields' and T(after + 1) == ',':
                semi = next((e for e in range(after + 2, body_close) if T(e) == ';'), None)
                if semi is None or split(after + 2, semi) != [(after + 2, semi)]: return None
                for x, y in split(semi + 1, body_close):
                    if y - x != 1 or toks[x][0] != 'id': return None
                    stores.append((T(x), span(after + 2, semi)))
            if not stores: return None
            if any(f in BYTE_FIELDS for f, _ in stores):
                for f, v in stores:
                    parts += store(f'{base}.{prefix}{f}', f in BYTE_FIELDS, v) + ['; ']
                return done(parts)
            # one ordered multi-store, values evaluated as the macro pasted them
            parts.append(f'slot!(game, {base}' + ''.join('.' + x for x in path) + '; ')
            for i, (f, v) in enumerate(stores):
                parts += [f'{", " if i else ""}{f} = ', v]
            parts.append(')')
            return (a0, end, parts + ([] if T(body_close + 1) == ';' else [';']))
        # words!([i32,] (*E), HI, LO [+= V | = V]) split-word pairs
        w = open_paren - 1
        signed = T(w) == ',' and T(w - 1) == 'i32'
        if signed: w -= 2
        if path in ([], ['base']) and T(w) == '(' and T(w - 1) == '!' and T(w - 2) == 'words':
            pre = ''.join('.' + x for x in path)
            mclose = close(w)
            if T(after) != ',' or toks[after + 1][0] != 'id' or T(after + 2) != ',' or toks[after + 3][0] != 'id':
                return None
            hi, lo, e = T(after + 1), T(after + 3), after + 4
            q = w - 2
            while T(q - 1) == ':' and T(q - 2) == ':' and toks[q - 3][0] == 'id': q -= 3
            a0, end = toks[q][1], toks[mclose][2]
            if e == mclose:
                kind = 'i32, ' if signed else ''
                return (a0, end, [f'crate::rtl::words!({kind}(slot!(game, {base}{pre}.{hi}), slot!(game, {base}{pre}.{lo})))'])
            if signed: return None
            if T(e) == '+' and T(e + 1) == '=': op, v = '+=', e + 2
            elif T(e) == '=' and T(e + 1) != '=': op, v = '=', e + 1
            else: return None
            return (a0, end, [f'slot!(game, {base}{pre}({hi}, {lo}) {op} ', span(v, mclose), ')'])
        if not path or path[-1] == 'base': return None
        target = f'{base}.' + '.'.join(path)
        # crate::update_place! { previous [mut] NAME [: TY] = (*E).path => VALUE; }
        if T(open_paren - 1) == '=':
            j = open_paren - 2
            if T(j - 1) == ':' and T(j - 2) != ':':
                j -= 2  # `NAME: TY` with a one-token type
            jn = j
            j -= 1 if T(j - 1) == 'mut' else 0
            if T(j - 1) == 'previous' and T(j - 2) == '{' and T(j - 4) == 'update_place':
                mclose = close(j - 2)
                if path[-1] in BYTE_FIELDS or T(after) != '=' or T(after + 1) != '>' or T(mclose - 1) != ';': return None
                q = j - 4
                while T(q - 1) == ':' and T(q - 2) == ':' and toks[q - 3][0] == 'id': q -= 3
                binding = s[toks[j][1]:toks[open_paren - 1][1]].strip()
                return (toks[q][1], toks[mclose][2], [f'let {binding} = slot!(game, {target}); ',
                        f'slot!(game, {target} = ', span(after + 2, mclose - 1), ');'])
        byte_field = path[-1] in BYTE_FIELDS
        # byte!((*E).path; LANE) [= RHS;]
        if T(open_paren - 1) == '(' and T(open_paren - 2) == '!' and T(open_paren - 3) == 'byte':
            if T(after) != ';': return None
            mac_close = close(open_paren - 1)
            lane = s[toks[after + 1][1]:toks[mac_close][1]].strip()
            a0 = toks[open_paren - 3][1]
            if T(mac_close + 1) == '=' and T(mac_close + 2) != '=':
                if not stmt_start(open_paren - 3): return None
                end = stmt_end(mac_close + 2)
                if end is None: return None
                return (a0, toks[end][1], store(target, False, span(mac_close + 2, end), lane))
            return (a0, toks[mac_close][2], [f'slot!(byte game, {target}[{lane}])'])
        # crate::update_place! { reference F = *(&raw mut (*E).path as *mut u8)[.offset(N)] => EXPR; }
        if (T(open_paren - 1) == 'mut' and T(open_paren - 2) == 'raw' and T(open_paren - 3) == '&'
                and T(open_paren - 4) == '(' and T(open_paren - 5) == '*' and T(open_paren - 6) == '='
                and T(open_paren - 8) == 'reference' and T(open_paren - 9) == '{' and T(open_paren - 11) == 'update_place'):
            fresh, mclose = T(open_paren - 7), close(open_paren - 9)
            if T(after) != 'as' or T(after + 1) != '*' or T(after + 2) != 'mut' or T(after + 3) != 'u8' or T(after + 4) != ')':
                return None
            e, lane = after + 5, '0'
            if T(e) == '.' and T(e + 1) == 'offset' and T(e + 2) == '(':
                c = close(e + 2); lane = s[toks[e + 3][1]:toks[c][1]].strip(); e = c + 1
            if T(e) != '=' or T(e + 1) != '>' or T(mclose - 1) != ';': return None
            expr = txt[e + 2:mclose - 1]
            if 'game' in expr: return None
            uses = [q for q in range(e + 2, mclose - 1) if txt[q] == fresh]
            if any(not (T(q - 2) == '(' and T(q - 1) == '*' and T(q + 1) == ')') for q in uses): return None
            text = s[toks[e + 2][1]:toks[mclose - 2][2]].replace(f'(*{fresh})', fresh)
            j = open_paren - 11  # back over a `path::` prefix of `update_place`
            while T(j - 1) == ':' and T(j - 2) == ':' and toks[j - 3][0] == 'id': j -= 3
            a0 = toks[j][1]
            old = f'slot!(byte game, {target}[{lane}])'
            return (a0, toks[mclose][2], [f'slot!(byte game, {target}[{lane}] = match {old} {{ {fresh} => {text} }});'])
        # let F = &raw mut (*E).path; F.write_unaligned((F.read_unaligned()) OP V);
        if T(open_paren - 1) == 'mut' and T(open_paren - 2) == 'raw' and T(open_paren - 3) == '&' and T(open_paren - 4) == '=' and T(open_paren - 6) == 'let':
            fresh = T(open_paren - 5)
            if byte_field or T(after) != ';': return None
            rest = s[toks[after][2]:]
            m = re.match(r'\s*%s\.write_unaligned\(\(%s\.read_unaligned\(\)\) (\||&|\^) \(([^;]*)\)\);' % (fresh, fresh), rest)
            if not m or 'game' in m.group(2) or re.search(r'\b%s\b' % fresh, rest[m.end():m.end() + 4000]): return None
            old = f'slot!(game, {target})'
            return (toks[open_paren - 6][1], toks[after][2] + m.end(),
                    [f'slot!(game, {target} = {old} {m.group(1)} ({m.group(2)}));'])
        if enclosing_macro(open_paren) in PLACE_MACROS: return None
        if T(open_paren - 1) in ('[', '&', 'mut'): return None
        form, lane = ('byte game', '[0]') if byte_field else ('game', '')
        t, nt = T(after), T(after + 1)
        if t == '=' and nt not in ('=', '>'):
            if not stmt_start(open_paren): return None
            end = stmt_end(after + 1)
            if end is None: return None
            # `X = (X) OP ..` whose value starts by reading X: read-modify-write form
            r = after + 1
            while T(r) == '(' and T(r + 1) == '(': r += 1
            if not byte_field and T(r) == '(' and T(r + 1) == '*' and 'old' not in txt[after + 1:end]:
                rc = close(r)
                own = s[toks[open_paren][1]:toks[inner_end][2]]
                simple = re.fullmatch(r'\w+', slot) is not None
                if rc and s[toks[r][1]:toks[rc][2]] == own and simple:
                    rpath, rafter = field_path(rc + 1)
                    if rpath == path and rafter < end:
                        return (start, toks[end][1], [f'slot!(game, {target}, |old| ',
                                (toks[after + 1][1], toks[r][1]), 'old', (toks[rafter - 1][2], toks[end][1]), ')'])
            return (start, toks[end][1], store(target, byte_field, span(after + 1, end)))
        if t in ('+', '-', '|', '&', '^') and nt == '=' and toks[after + 1][1] == toks[after][2]:
            if byte_field or not stmt_start(open_paren): return None
            end = stmt_end(after + 2)
            if end is None: return None
            old = f'slot!(game, {target})'
            return (start, toks[end][1], [f'slot!(game, {target} = match ', span(after + 2, end),
                                          f' {{ rhs => {old} {t} rhs }})'])
        if nt == '=' and t in ('*', '/', '%', '<', '>'): return None
        return (start, toks[after - 1][2], [f'slot!({form}, {target}{lane})'])

    edits = []  # (start, end, parts, region_lo_line, region_hi_line, group)
    line = lambda off: s.count('\n', 0, off) + 1
    for k in range(n - 6):  # bindings: let [mut] E = Get_X(ARG, game);
        m = 1 if T(k + 1) == 'mut' else 0
        if txt[k] != 'let' or toks[k + 1 + m][0] != 'id' or T(k + 2 + m) != '=' or T(k + 3 + m) not in getters or T(k + 4 + m) != '(':
            continue
        var, getter = txt[k + 1 + m], txt[k + 3 + m]
        gclose = close(k + 4 + m)
        if gclose is None or T(gclose + 1) != ';' or T(gclose - 1) != 'game' or T(gclose - 2) != ',': continue
        arg = s[toks[k + 5 + m][1]:toks[gclose - 2][1]].strip()
        prefix = T(k - 1) == '{' and (T(k - 2) == 'prefix' and T(k - 3) == '@' or T(k - 2) == 'before')
        if prefix:
            # `@prefix { let E = ..; };` / countdown `before { let E = ..; .. }` are
            # pasted into each following callback body up to the next `@` section
            # or the end of the invocation.
            pclose = close(k - 1)
            if pclose is None or (T(k - 2) == 'prefix' and pclose != gclose + 2): continue
            d, e = 0, pclose + 1
            while e < n:
                if txt[e] in '([{': d += 1
                elif txt[e] in ')]}':
                    d -= 1
                    if d < 0: break
                elif d == 0 and txt[e] == '@': break
                e += 1
            first = gclose + 2
        else:
            d, e = 0, gclose + 2  # scope: to the end of the enclosing block
            while e < n:
                if txt[e] in '([{': d += 1
                elif txt[e] in ')]}':
                    d -= 1
                    if d < 0: break
                e += 1
            first = gclose + 2
        group, ok = [(toks[k][1], toks[gclose + 1][2], [f'let {var}: i32 = {arg};'])], True
        for q in range(first, e):
            if txt[q] != var: continue
            if T(q - 1) == '.' or (T(q - 1) == ':' and T(q - 2) == ':'): continue  # field/path
            if T(q - 1) == 'let': ok = False; break  # shadowing
            if T(q - 1) == '*' and T(q - 2) == '(' and T(q + 1) == ')':
                ed = access(q - 2, q + 1, getter[4:], var)
                if ed is None: ok = False; break
                group.append(ed)
            else:
                ok = False; break
        # a binding without uses in its own block feeds macro-pasted bodies (@prefix)
        if ok and len(group) > 1:
            lo, hi = line(toks[k][1]), line(toks[e - 1][2])
            for ed in group: edits.append(ed + (lo, hi, ('b', k)))
    for k in range(n - 6):  # inline (*Get_X(ARG, game)).path
        if txt[k] == '(' and txt[k + 1] == '*' and txt[k + 2] in getters and txt[k + 3] == '(':
            gclose = close(k + 3)
            if gclose is None or T(gclose + 1) != ')' or T(gclose - 1) != 'game' or T(gclose - 2) != ',': continue
            arg = s[toks[k + 4][1]:toks[gclose - 2][1]].strip()
            ed = access(k, gclose + 1, txt[k + 2][4:], arg)
            if ed: edits.append(ed + (line(ed[0]), line(ed[1]), ('i', k)))
    return edits


def render(s, edits, rejected, a=0, b=None):
    """Source [a, b) with every admitted edit applied, nested edits inside stored values too."""
    b = len(s) if b is None else b
    live = sorted((e for e in edits if e[5] not in rejected and a <= e[0] and e[1] <= b),
                  key=lambda e: (e[0], -e[1]))
    out, pos = [], a
    for e in live:
        if e[0] < pos: continue  # nested inside an earlier edit; rendered by it
        out.append(s[pos:e[0]])
        text = ''.join(part if isinstance(part, str) else render(s, edits, rejected, part[0], part[1])
                       for part in e[2])
        # keep line numbers: compiler errors map back to planned regions by line
        out.append(text + '\n' * max(0, s.count('\n', e[0], e[1]) - text.count('\n')))
        pos = e[1]
    out.append(s[pos:b])
    return ''.join(out)


def main():
    files = sys.argv[1:]
    orig = {p: open(p).read() for p in files}
    getters = {}
    for p, s in orig.items():
        for m in GETTER.finditer(s): getters[m.group(1)] = m.group(2)
    for p, s in orig.items():  # converted forms name the macro unqualified
        if getters and 'use crate::ram_map::slot;' not in s:
            first = re.search(r'^use .*;$', s, re.M)
            s = s[:first.start()] + 'use crate::ram_map::slot;\n' + s[first.start():]
        # each getter Get_X names its layout through a file-local alias X
        for g, layout in getters.items():
            if f'type {g[4:]} = {layout};' in s: continue
            for m in re.finditer(r'^crate::ram_map::ram_pointer_functions! \{.*?^\}\n', s, re.M | re.S):
                if re.search(r'\b%s\(' % g, m.group(0)):
                    s = s[:m.end()] + f'type {g[4:]} = {layout};\n' + s[m.end():]
                    break
        open(p, 'w').write(s)
    total = 0
    # Repeat planning on the edited text: nested occurrences inside a stored RHS
    # are converted on the next pass.
    for rnd in range(6):
        cur = {p: open(p).read() for p in files}
        plans = {p: plan(s, getters) for p, s in cur.items()}
        if not any(plans.values()): break
        rejected = {p: set() for p in files}
        for it in range(30):
            for p in files: open(p, 'w').write(render(cur[p], plans[p], rejected[p]))
            errs, _ = check()
            if not errs: break
            hit = False
            for fl, ln, _, msg in errs:
                if fl not in plans: continue
                for ed in plans[fl]:
                    if ed[3] <= ln <= ed[4] and ed[5] not in rejected[fl]:
                        rejected[fl].add(ed[5]); hit = True
            if not hit:
                print('unattributed errors; reverting round', errs[:3], flush=True)
                for p in files: open(p, 'w').write(cur[p])
                plans = {p: [] for p in files}
                break
            print(f'  round {rnd} iter {it}: {len(errs)} errors, rejected {sum(map(len, rejected.values()))};',
                  errs[0][0], errs[0][1], errs[0][3][:120], flush=True)
        done = sum(len([e for e in v if e[5] not in rejected[p]]) for p, v in plans.items())
        total += done
        print(f'round {rnd}: {done} edits', flush=True)
        if done == 0: break
    # remove getters no longer referenced anywhere in the crate
    for g in getters:
        r = subprocess.run(['grep', '-rlw', g, 'sm/src'], capture_output=True, text=True).stdout.split()
        uses = sum(len(re.findall(r'\b%s\b' % g, open(f).read())) for f in r)
        if uses == 1:
            for p in files:
                s = open(p).read()
                s2 = re.sub(r'^\s*%s\(o: i32\) -> \w+ = \[0xf78, o as isize\];\n' % g, '', s, flags=re.M)
                if s2 != s: open(p, 'w').write(s2); print('removed getter', g)
    for p in files:  # drop aliases and the import when nothing was converted
        s = open(p).read()
        for g, layout in getters.items():
            if not re.search(r'\b%s\[' % g[4:], s):
                s = s.replace(f'type {g[4:]} = {layout};\n', '')
        if 'slot!(' not in s: s = s.replace('use crate::ram_map::slot;\n', '')
        open(p, 'w').write(s)
    errs, _ = check(); assert not errs, errs[:3]
    subprocess.run(['cargo', 'fmt'], check=True)
    print(f'total edits {total}')


if __name__ == '__main__':
    main()
