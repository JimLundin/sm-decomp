"""Write the production treemap (the map's overview tiles) as a static SVG.

Reads the atlas-data.json that generate_atlas.py writes beside index.html and
uses the same layout as the page. Pages publishes the result as treemap.svg,
which PROGRESS.md embeds, so neither needs hand updates.
Usage: python docs/codebase-map/treemap_svg.py atlas-data.json treemap.svg
"""
import json, sys
from html import escape

W, H, TOP = 960, 520, 64
COLORS = {'owned': '#a9d8b9', 'partial': '#f0d48f', 'legacy': '#c9d3d8'}
NAMES = {'owned': 'Owned', 'partial': 'Partly owned', 'legacy': 'Legacy'}


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
    """Squarified layout, ported from atlas.template.html's partition()."""
    scale = w * h / sum(r['lines'] for r in items)
    pending = [dict(r, area=r['lines'] * scale) for r in items]
    out = []

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
        depth = sum(r['area'] for r in row) / side
        offset = 0
        for r in row:
            span = r['area'] / depth
            if w >= h:
                out.append(dict(r, x=x, y=y + offset, w=depth, h=span))
            else:
                out.append(dict(r, x=x + offset, y=y, w=span, h=depth))
            offset += span
        if w >= h:
            x, w = x + depth, w - depth
        else:
            y, h = y + depth, h - depth
    return out


def short(n):
    return f'{n / 1000:.1f}k' if n >= 1000 else str(n)


def main(data_path, out_path):
    data = json.load(open(data_path, encoding='utf-8'))
    rows = []
    for r in data['responsibilities']:
        counts = {k: 0 for k in COLORS}
        for f in data['files']:
            if f['production'] and f['responsibility'] == r['id']:
                counts[f['status']] += f['lines'] or 0
        lines = sum(counts.values())
        if lines:
            rows.append(dict(r, counts=counts, lines=lines))
    rows.sort(key=lambda r: -r['lines'])
    total = sum(data['production'].values())
    p = [f'<svg xmlns="http://www.w3.org/2000/svg" width="{W}" height="{H + TOP}" viewBox="0 0 {W} {H + TOP}" role="img">',
         f'<title>{total:,} production lines by responsibility at {data["commit"][:7]}</title>',
         '<style>text{font-family:system-ui,Segoe UI,sans-serif;fill:#17343e}.h{font-size:20px;font-weight:650}'
         '.l{font-size:14px}.t{font-size:15px;font-weight:650}.s{font-size:12px}</style>',
         f'<rect width="{W}" height="{H + TOP}" fill="#f5f7f7"/>',
         f'<text x="4" y="24" class="h">{total:,} production lines</text>']
    x = 4
    for k, name in NAMES.items():
        label = f'{name} {short(data["production"][k])}'
        p.append(f'<rect x="{x}" y="38" width="12" height="12" rx="2" fill="{COLORS[k]}"/>'
                 f'<text x="{x + 17}" y="49" class="l">{label}</text>')
        x += 30 + 8 * len(label)
    p.append(f'<text x="{W - 4}" y="24" class="l" text-anchor="end">{data["commit"][:7]} &#183; {data["generated"][:10]}</text>')
    for r in partition(rows, 0, 0, W, H):
        x0, y0, w, h = r['x'], r['y'] + TOP, r['w'], r['h']
        clip = f'c{r["id"]}'
        p.append(f'<clipPath id="{clip}"><rect x="{x0 + 2:.1f}" y="{y0 + 2:.1f}" width="{max(w - 4, 0):.1f}" height="{max(h - 4, 0):.1f}" rx="7"/></clipPath><g clip-path="url(#{clip})">')
        bx = x0
        for k in COLORS:
            bw = w * r['counts'][k] / r['lines']
            if bw:
                p.append(f'<rect x="{bx:.1f}" y="{y0:.1f}" width="{bw:.1f}" height="{h:.1f}" fill="{COLORS[k]}"/>')
                bx += bw
        c = r['counts']
        room = w - 16
        title = next((t for t in (r['title'], r['title'].split(' ')[0]) if 8.6 * len(t) <= room), None)
        if title and h >= 34:
            p.append(f'<text x="{x0 + 9:.1f}" y="{y0 + h / 2 - (8 if h >= 60 else -5):.1f}" class="t">{escape(title)}</text>')
        legacy, owned = round(c['legacy'] / r['lines'] * 100), round(c['owned'] / r['lines'] * 100)
        detail = next((t for t in (f'{legacy}% legacy · {owned}% owned · {short(r["lines"])}',
                                   f'{legacy}% legacy · {short(r["lines"])}', f'{legacy}% legacy')
                       if 6.8 * len(t) <= room), None)
        if title and detail and h >= 60:
            p.append(f'<text x="{x0 + 9:.1f}" y="{y0 + h / 2 + 10:.1f}" class="s">{escape(detail)}</text>')
        p.append('</g>')
    p.append('</svg>')
    open(out_path, 'w', encoding='utf-8').write('\n'.join(p) + '\n')


if __name__ == '__main__':
    main(*sys.argv[1:3])
