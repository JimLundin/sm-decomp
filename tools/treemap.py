"""Draw a mobile-readable progress picture: production Rust by responsibility,
split into owned, partly owned and legacy lines, with current totals and 24h change.

Reads committed Git blobs only (no working tree, no network, standard library).
CI publishes the result to GitHub Pages as treemap.svg; README.md embeds it.
Usage: python tools/treemap.py OUT.svg [REVISION]
"""
import datetime, io, re, subprocess, sys, tarfile
from html import escape
from pathlib import PurePosixPath

W, PAD, TOP, ROW_H, GAP = 360, 16, 338, 92, 10
COLORS = {'owned': '#164739', 'partial': '#594319', 'legacy': '#273b54'}
ACCENTS = {'owned': '#67e8b3', 'partial': '#ffd080', 'legacy': '#a5bcd8'}
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
        if re.fullmatch(r'sm/src/game/sm_[0-9a-f]{2}\.rs', path):
            m['legacy_bank'] += lines
        m['rows'][area(path)][status(path, text)] += lines
    m['totals'] = {k: sum(r[k] for r in m['rows'].values()) for k in COLORS}
    return m


def short(n):
    return f'{n / 1000:.1f}k' if abs(n) >= 1000 else str(n)


def card(x, y, label, value, delta):
    """One headline number; delta is (text, sign) where sign>0 means better."""
    text, sign = delta
    tone = 'good' if sign > 0 else 'bad' if sign < 0 else 'neutral'
    return (f'<rect x="{x}" y="{y}" width="159" height="94" rx="10" fill="#131f2e" stroke="#293a50"/>'
            f'<text x="{x + 12}" y="{y + 23}" class="l">{escape(label)}</text>'
            f'<text x="{x + 12}" y="{y + 55}" class="v">{escape(value)}</text>'
            f'<text x="{x + 12}" y="{y + 79}" class="d {tone}">{escape(text)}</text>')


def change(now, then, lower_is_better, fmt=short):
    if then is None:
        return ('', 0)
    d = now - then
    sign = 0 if d == 0 else (-1 if (d > 0) == lower_is_better else 1)
    return (f'{"+" if d > 0 else "-" if d < 0 else "+/-"}{fmt(abs(d))} in 24h', sign)


def main(out_path, rev='HEAD'):
    commit = git('rev-parse', rev).decode().strip()
    when = git('log', '-1', '--format=%cI', commit).decode().strip()
    day_before = (datetime.datetime.fromisoformat(when) - datetime.timedelta(days=1)).isoformat()
    base = git('rev-list', '-1', f'--before={day_before}', commit).decode().strip() or None
    now, then = measure(commit), (measure(base) if base else None)
    t = now['totals']
    old = (lambda k: then[k]) if then else (lambda k: None)
    cards = [
        ('Production lines', f'{now["production"]:,}', change(now['production'], old('production'), True)),
        ('Owned code', f'{t["owned"]:,}', change(t['owned'], then['totals']['owned'] if then else None, False)),
        ('Legacy bank lines', f'{now["legacy_bank"]:,}', change(now['legacy_bank'], old('legacy_bank'), True)),
        ('Unsafe lines', f'{now["unsafe"]:,}', change(now['unsafe'], old('unsafe'), True, lambda n: f'{n:,}')),
    ]
    rows = sorted(({**dict(id=k, title=n), 'counts': now['rows'][k], 'lines': sum(now['rows'][k].values())}
                   for k, n in AREAS if sum(now['rows'][k].values())), key=lambda r: -r['lines'])
    stamp = datetime.datetime.fromisoformat(when)
    height = TOP + len(rows) * (ROW_H + GAP) + 92
    p = [f'<svg xmlns="http://www.w3.org/2000/svg" width="{W}" height="{height}" viewBox="0 0 {W} {height}" role="img">',
         f'<title>Super Metroid codebase at {commit[:7]}: {now["production"]:,} production, '
         f'{t["owned"]:,} owned code, {now["legacy_bank"]:,} legacy bank and {now["unsafe"]:,} unsafe lines</title>',
         '<desc>Each row is one responsibility; its number is the production Rust line count. '
         'The stacked bar shows the fraction of those lines classified as owned (green), partly owned (amber) '
         'or legacy (blue). The three numbers below the bar are their line counts in that order. '
         'Rows have equal height; their size does not represent source volume. '
         'These are whole-file source classifications, not proof that a subsystem is complete. Owned requires '
         'forbid(unsafe_code), no unsafe operations, g_ram, Game or direct foreign calls. Bank files and '
         'unsafe files without that marker are legacy; other files are partly owned. '
         'The legacy bank headline is only a subset of the blue legacy total. '
         'Unsafe counts all Rust, including verification; other totals exclude verification. '
         'Headline changes compare the source with the latest ancestor at least 24 hours earlier.</desc>',
         '<style>text{font-family:system-ui,Segoe UI,sans-serif;fill:#edf4fc}.h{font-size:22px;font-weight:650}'
         '.l{font-size:14px;fill:#b0c2d8}.v{font-size:26px;font-weight:650}.d{font-size:14px;font-weight:600}'
         '.good{fill:#67e8b3}.bad{fill:#ff9e9e}.neutral{fill:#b0c2d8}'
         '.owned{fill:#67e8b3}.partial{fill:#ffd080}.legacy{fill:#a5bcd8}'
         '.k{font-size:14px;font-weight:600}.t{font-size:17px;font-weight:650}'
         '.s{font-size:15px;font-variant-numeric:tabular-nums}.section{font-size:18px;font-weight:650}</style>',
         f'<rect width="{W}" height="{height}" rx="12" fill="#0b121c"/>',
         f'<text x="{PAD}" y="32" class="h">Super Metroid &#183; Codebase</text>',
         f'<text x="{PAD}" y="55" class="l">{commit[:7]} &#183; {stamp:%Y-%m-%d %H:%M %z}</text>']
    for i, (label, value, delta) in enumerate(cards):
        p.append(card(PAD + (i % 2) * 169, 74 + (i // 2) * 104, label, value, delta))
    p.append(f'<text x="{PAD}" y="304" class="section">Production Rust by area</text>')
    for i, (k, name) in enumerate(NAMES.items()):
        p.append(f'<text x="{PAD + 12 + i * 104}" y="327" class="k {k}">{name}</text>')
    for i, r in enumerate(rows):
        y, c = TOP + i * (ROW_H + GAP), r['counts']
        detail = ', '.join(f'{NAMES[k]} {c[k]:,}' for k in COLORS)
        p.append(f'<g><title>{escape(r["title"])}: {r["lines"]:,} production lines; {detail}</title>'
                 f'<rect x="{PAD}" y="{y}" width="{W - 2 * PAD}" height="{ROW_H}" rx="10" '
                 'fill="#101b28" stroke="#293a50"/>'
                 f'<text x="{PAD + 12}" y="{y + 26}" class="t">{escape(r["title"])}</text>'
                 f'<text x="{W - PAD - 12}" y="{y + 26}" class="s" text-anchor="end">{r["lines"]:,}</text>'
                 f'<clipPath id="c{r["id"]}"><rect x="{PAD + 12}" y="{y + 42}" '
                 f'width="{W - 2 * PAD - 24}" height="14" rx="5"/></clipPath>'
                 f'<g clip-path="url(#c{r["id"]})">')
        bx = PAD + 12
        for k in COLORS:
            bw = (W - 2 * PAD - 24) * c[k] / r['lines']
            if bw:
                p.append(f'<rect x="{bx:.4f}" y="{y + 42}" width="{bw:.4f}" height="14" fill="{ACCENTS[k]}"/>')
                bx += bw
        p.append('</g>')
        for j, k in enumerate(COLORS):
            p.append(f'<text x="{PAD + 12 + j * 104}" y="{y + 80}" class="s {k}">{c[k]:,}</text>')
        p.append('</g>')
    footer = TOP + len(rows) * (ROW_H + GAP)
    for i, line in enumerate(('Bars show the mix within each area.',
                              'Legacy bank lines are part of blue legacy.',
                              'Unsafe also counts verification Rust.')):
        p.append(f'<text x="{PAD}" y="{footer + 20 + 23 * i}" class="l">{line}</text>')
    p.append('</svg>')
    open(out_path, 'w', encoding='utf-8').write('\n'.join(p) + '\n')


if __name__ == '__main__':
    main(*sys.argv[1:3])
