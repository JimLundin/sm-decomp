"""Draw the progress picture: production Rust by responsibility, split into owned,
partly owned and legacy lines, with four headline numbers and their 24-hour change.

Reads committed Git blobs only (no working tree, no network, standard library).
CI publishes the result to GitHub Pages as treemap.svg; PROGRESS.md embeds it.
Usage: python tools/treemap.py OUT.svg [REVISION]
"""
import datetime, io, re, subprocess, sys, tarfile
from html import escape
from pathlib import PurePosixPath
from zoneinfo import ZoneInfo

W, H, TOP = 960, 520, 132
COLORS = {'owned': '#a9d8b9', 'partial': '#f0d48f', 'legacy': '#c9d3d8'}
NAMES = {'owned': 'Owned', 'partial': 'Partly owned', 'legacy': 'Legacy'}
AREAS = [('movement', 'Samus movement'), ('samus', 'Samus actions'), ('enemies', 'Enemies'),
         ('bosses', 'Bosses'), ('rooms', 'Rooms & doors'), ('projectiles', 'Projectiles'),
         ('hud', 'HUD & scenes'), ('graphics', 'Sprites & palettes'), ('audio', 'Audio'),
         ('engine', 'Engine & devices')]
VERIFY = re.compile(r'/(verification|verify)/|/verify[^/]*[.]rs$')
COMMENTS = re.compile(r'//[^\n]*|/\*[\s\S]*?\*/|"(?:\\.|[^"\\])*"')


def area(path):
    name, low = PurePosixPath(path).name, path.lower()
    if re.fullmatch(r'sm_a[45679]\.rs', name): return 'bosses'
    if any(x in low for x in ('spc', 'audio', 'dsp', 'music', 'sound')): return 'audio'
    if any(x in low for x in ('player_movement', 'player_collision')): return 'movement'
    if any(x in low for x in ('projectile', 'beam_')) or name == 'sm_93.rs': return 'projectiles'
    if any(x in low for x in ('minimap', 'hud', 'menu', 'message_box', 'intro_', 'dialogue')) \
            or name in ('sm_81.rs', 'sm_8b.rs', 'sm_8c.rs', 'sm_8d.rs', 'sm_8e.rs'): return 'hud'
    if any(x in low for x in ('room_', 'door', 'mechanism', 'elevator')) or name in ('sm_82.rs', 'sm_8f.rs'): return 'rooms'
    if any(x in low for x in ('samus', 'player_', 'grapple')) \
            or name in ('sm_90.rs', 'sm_91.rs', 'sm_92.rs', 'sm_94.rs', 'sm_9b.rs'): return 'samus'
    if '/actors/' in low or 'enemy' in name or re.fullmatch(r'sm_(a[0238]|b[23])\.rs', name): return 'enemies'
    if any(x in low for x in ('palette', 'sprite', 'animated_tiles', 'drawing', 'blend', 'hdma', 'mode7',
                              'liquid_effect')) or name in ('sm_83.rs', 'sm_88.rs'): return 'graphics'
    return 'engine'


def status(path, text):
    """Owned: whole-file forbid(unsafe_code), no unsafe, g_ram, Game or foreign calls.
    Legacy: a bank file, or unsafe without a forbid marker. Otherwise partly owned."""
    clean = COMMENTS.sub(' ', text)
    unsafe, raw = re.search(r'\bunsafe\b', clean), re.search(r'\bg_ram\b', clean)
    if re.fullmatch(r'sm_[0-9a-f]{2}\.rs', PurePosixPath(path).name): return 'legacy'
    if re.search(r'^#!\[forbid\(unsafe_code\)\]', text, re.M) and not unsafe and not raw \
            and not re.search(r'\bGame\b|\.foreign\s*\(', clean): return 'owned'
    if unsafe and 'forbid(unsafe_code)' not in text: return 'legacy'
    return 'partial'


def git(*args):
    return subprocess.check_output(['git', *args])


def measure(rev):
    """Counts at REV; rules match tools/progress.sh."""
    tar = tarfile.open(fileobj=io.BytesIO(git('archive', rev, 'sm/src')))
    m = dict(rows={k: dict.fromkeys(COLORS, 0) for k, _ in AREAS}, unsafe=0, production=0, legacy_bank=0)
    for member in tar:
        path = member.name
        if not member.isfile() or not path.endswith('.rs'):
            continue
        text = tar.extractfile(member).read().decode('utf-8', 'replace')
        lines = len(text.splitlines())
        m['unsafe'] += sum('unsafe' in line for line in text.splitlines())
        if VERIFY.search(path):
            continue
        m['production'] += lines
        m['rows'][area(path)][status(path, text)] += lines
    m['totals'] = {k: sum(r[k] for r in m['rows'].values()) for k in COLORS}
    # Checks come from the agents' Score lines; take the best of the latest few,
    # since a rebased commit can carry a stale count.
    log = git('log', '-60', '--format=%B', rev).decode('utf-8', 'replace')
    scores = [(int(a), int(b)) for a, b in re.findall(r'^Score: checks (\d+)/(\d+)', log, re.M)[:10]]
    m['checks'] = max(scores) if scores else None
    return m


def small_partition(items, x, y, w, h):
    if len(items) == 1:
        return [dict(items[0], x=x, y=y, w=w, h=h)]
    best, score, total = None, float('inf'), sum(r['lines'] for r in items)
    for split in range(1, len(items)):
        f = sum(r['lines'] for r in items[:split]) / total
        for vertical in (True, False):
            a = (x, y, w * f, h) if vertical else (x, y, w, h * f)
            b = (x + w * f, y, w * (1 - f), h) if vertical else (x, y + h * f, w, h * (1 - f))
            tiles = small_partition(items[:split], *a) + small_partition(items[split:], *b)
            cost = max(max(r['w'] / r['h'], r['h'] / r['w'], 132 / r['w'] if r['w'] < 44 else 0) for r in tiles)
            if cost < score:
                best, score = tiles, cost
    return best


def partition(items, x, y, w, h):
    """Squarified treemap; the last three tiles are placed to avoid slivers."""
    scale = w * h / sum(r['lines'] for r in items)
    pending, out = [dict(r, area=r['lines'] * scale) for r in items], []

    def worst(row, side):
        areas = [r['area'] for r in row]
        s = sum(areas)
        return max(side * side * max(areas) / (s * s), s * s / (side * side * min(areas)))

    while pending:
        if len(pending) <= 3:
            out += small_partition(pending, x, y, w, h)
            break
        side, row = min(w, h), [pending.pop(0)]
        while pending and len(pending) != 3 and worst(row + [pending[0]], side) <= worst(row, side):
            row.append(pending.pop(0))
        depth, offset = sum(r['area'] for r in row) / side, 0
        for r in row:
            span = r['area'] / depth
            out.append(dict(r, x=x, y=y + offset, w=depth, h=span) if w >= h else
                       dict(r, x=x + offset, y=y, w=span, h=depth))
            offset += span
        if w >= h:
            x, w = x + depth, w - depth
        else:
            y, h = y + depth, h - depth
    return out


def short(n):
    return f'{n / 1000:.1f}k' if abs(n) >= 1000 else str(n)


def card(x, label, value, delta):
    """One headline number; delta is (text, sign) where sign>0 means better."""
    text, sign = delta
    color = '#237343' if sign > 0 else '#a23b2a' if sign < 0 else '#6b7f86'
    return (f'<rect x="{x}" y="40" width="232" height="56" rx="7" fill="#fff" stroke="#d6e0e2"/>'
            f'<text x="{x + 12}" y="59" class="l">{escape(label)}</text>'
            f'<text x="{x + 12}" y="86" class="v">{escape(value)}</text>'
            f'<text x="{x + 220}" y="86" class="d" text-anchor="end" fill="{color}">{escape(text)}</text>')


def change(now, then, lower_is_better, fmt=short):
    if then is None:
        return ('', 0)
    d = now - then
    sign = 0 if d == 0 else (-1 if (d > 0) == lower_is_better else 1)
    return (f'{"+" if d > 0 else "−" if d < 0 else "±"}{fmt(abs(d))} in 24h', sign)


def main(out_path, rev='HEAD'):
    commit = git('rev-parse', rev).decode().strip()
    when = git('log', '-1', '--format=%cI', commit).decode().strip()
    day_before = (datetime.datetime.fromisoformat(when) - datetime.timedelta(days=1)).isoformat()
    base = git('rev-list', '-1', f'--before={day_before}', commit).decode().strip() or None
    now, then = measure(commit), (measure(base) if base else None)
    t = now['totals']
    old = (lambda k: then[k]) if then else (lambda k: None)
    checks = now['checks'] or (0, 0)
    old_checks = then['checks'][0] if then and then['checks'] else None
    cards = [
        ('Required checks passing', f'{checks[0]} / {checks[1]}', change(checks[0], old_checks, False, str)),
        ('Legacy lines', short(t['legacy']), change(t['legacy'], then and then['totals']['legacy'], True)),
        ('Lines containing unsafe', f'{now["unsafe"]:,}', change(now['unsafe'], old('unsafe'), True, lambda n: f'{n:,}')),
        ('Production lines', short(now['production']), change(now['production'], old('production'), True)),
    ]
    rows = sorted(({**dict(id=k, title=n), 'counts': now['rows'][k], 'lines': sum(now['rows'][k].values())}
                   for k, n in AREAS if sum(now['rows'][k].values())), key=lambda r: -r['lines'])
    stamp = datetime.datetime.fromisoformat(when).astimezone(ZoneInfo('Europe/Stockholm'))
    p = [f'<svg xmlns="http://www.w3.org/2000/svg" width="{W}" height="{H + TOP}" viewBox="0 0 {W} {H + TOP}" role="img">',
         f'<title>Engine migration at {commit[:7]}: {checks[0]}/{checks[1]} checks, {t["legacy"]:,} legacy, '
         f'{now["unsafe"]:,} unsafe, {now["production"]:,} production lines</title>',
         '<style>text{font-family:system-ui,Segoe UI,sans-serif;fill:#17343e}.h{font-size:20px;font-weight:650}'
         '.l{font-size:13px;fill:#526873}.v{font-size:22px;font-weight:650}.d{font-size:13px;font-weight:600}'
         '.k{font-size:13px}.t{font-size:15px;font-weight:650}.s{font-size:12px}</style>',
         f'<rect width="{W}" height="{H + TOP}" fill="#f5f7f7"/>',
         '<text x="4" y="26" class="h">Super Metroid engine migration</text>',
         f'<text x="{W - 4}" y="26" class="l" text-anchor="end">{commit[:7]} &#183; {stamp:%Y-%m-%d %H:%M %Z}</text>']
    for i, (label, value, delta) in enumerate(cards):
        p.append(card(4 + i * 240, label, value, delta))
    x = 4
    for k, name in NAMES.items():
        label = f'{name} {short(t[k])}'
        p.append(f'<rect x="{x}" y="108" width="12" height="12" rx="2" fill="{COLORS[k]}"/>'
                 f'<text x="{x + 17}" y="119" class="k">{label}</text>')
        x += 30 + 7.5 * len(label)
    for r in partition(rows, 0, 0, W, H):
        x0, y0, w, h, c = r['x'], r['y'] + TOP, r['w'], r['h'], r['counts']
        p.append(f'<clipPath id="c{r["id"]}"><rect x="{x0 + 2:.1f}" y="{y0 + 2:.1f}" width="{max(w - 4, 0):.1f}" '
                 f'height="{max(h - 4, 0):.1f}" rx="7"/></clipPath><g clip-path="url(#c{r["id"]})">')
        bx = x0
        for k in COLORS:
            bw = w * c[k] / r['lines']
            if bw:
                p.append(f'<rect x="{bx:.1f}" y="{y0:.1f}" width="{bw:.1f}" height="{h:.1f}" fill="{COLORS[k]}"/>')
                bx += bw
        room = w - 16
        title = next((s for s in (r['title'], r['title'].split(' ')[0]) if 8.6 * len(s) <= room), None)
        if title and h >= 34:
            p.append(f'<text x="{x0 + 9:.1f}" y="{y0 + h / 2 - (8 if h >= 60 else -5):.1f}" class="t">{escape(title)}</text>')
        legacy, owned = round(c['legacy'] / r['lines'] * 100), round(c['owned'] / r['lines'] * 100)
        detail = next((s for s in (f'{legacy}% legacy · {owned}% owned · {short(r["lines"])}',
                                   f'{legacy}% legacy · {short(r["lines"])}', f'{legacy}% legacy')
                       if 6.8 * len(s) <= room), None)
        if title and detail and h >= 60:
            p.append(f'<text x="{x0 + 9:.1f}" y="{y0 + h / 2 + 10:.1f}" class="s">{escape(detail)}</text>')
        p.append('</g>')
    p.append('</svg>')
    open(out_path, 'w', encoding='utf-8').write('\n'.join(p) + '\n')


if __name__ == '__main__':
    main(*sys.argv[1:3])
