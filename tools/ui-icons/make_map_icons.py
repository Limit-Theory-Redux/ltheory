"""Renders the game map icons from game-map-icons.svg into textures the HUD tints.

Each icon is a list of the SVG's shapes, by element or group id (ICONS below). It is cropped to those
shapes and written to res/tex2d/map-icon/ twice, white with the shape in the alpha channel:
  <name>.png       128 x 128, the icon's longer side 120 pixels (ICON_FILL), centred
  <name>-glow.png  64 x 64, the icon's longer side 32 pixels (GLOW_FILL), blurred: a soft glow to draw
                   under the icon
Fills (nonzero and evenodd), strokes and radial gradients on circles are drawn; text is not.

Run from the repository root: python -I tools/ui-icons/make_map_icons.py   (needs numpy and Pillow)
"""
import math
import os
import sys

import numpy as np
from PIL import Image, ImageFilter

HERE = os.path.dirname(os.path.abspath(__file__))
sys.path.insert(0, HERE)
import svgshapes  # noqa: E402

SVG = os.path.join(HERE, 'game-map-icons.svg')
OUT = os.path.join(HERE, '..', '..', 'res', 'tex2d', 'map-icon')

ICON_SIZE, ICON_FILL = 128, 120
GLOW_SIZE, GLOW_FILL, GLOW_BLUR = 64, 32, 5
SS = 4  # samples per pixel along each axis

# name -> the ids of its shapes in the SVG ("Game Map Icons" page); a group id takes every shape in it
ICONS = {
    'station': ['path5893-7-3-5'],
    'warp-gate': ['circle20', 'path33', 'path34', 'path84-5-9'],
    'wormhole': ['path84-5'],
    'star': ['path1-4-5-6-0'],
    'planet-gas': ['path399'],
    'planet-rocky': ['path405', 'circle23'],
    'beacon': ['circle40-9', 'path85', 'path86'],
    'relay-station': ['path5893-7-3-2', 'rect94', 'path95', 'rect92', 'path101'],
    'colony': ['g429'],
    'asteroid-field': ['g52'],
    'asteroid': ['path48'],  # the field's largest rock, for a single asteroid
    'refinery': ['rect72'],
    'factory': ['g68'],
    'research-lab': ['g79'],
    'drone': ['circle407'],
    'fighter': ['path406'],
    'corvette': ['rect10'],
    'capital-ship': ['path413'],
}


def shapes_for(shapes, ids):
    picked = [s for s in shapes if s['id'] in ids or any(a in ids for a in s['ancestors'])]
    missing = [i for i in ids if not any(s['id'] == i or i in s['ancestors'] for s in shapes)]
    if missing:
        sys.exit(f'not in the SVG: {", ".join(missing)}')
    return picked


def bounds(shapes):
    x0 = y0 = math.inf
    x1 = y1 = -math.inf
    for s in shapes:
        pad = s['stroke_width'] / 2 if s['stroke'] != 'none' else 0
        for pts, _ in s['subpaths']:
            for x, y in pts:
                x0, y0, x1, y1 = min(x0, x - pad), min(y0, y - pad), max(x1, x + pad), max(y1, y + pad)
    return x0, y0, x1, y1


def fill_coverage(subpaths, rule, n):
    """Coverage (0/1 per sample) of closed subpaths on an n x n sample grid."""
    edges = []
    for pts, _ in subpaths:
        for i in range(len(pts)):
            (ax, ay), (bx, by) = pts[i], pts[(i + 1) % len(pts)]
            if ay != by:
                edges.append((ax, ay, bx, by))
    out = np.zeros((n, n), dtype=np.float32)
    if not edges:
        return out
    e = np.array(edges, dtype=np.float64)
    ax, ay, bx, by = e[:, 0], e[:, 1], e[:, 2], e[:, 3]
    xs = np.arange(n) + 0.5
    for row in range(n):
        y = row + 0.5
        hit = ((ay <= y) & (by > y)) | ((by <= y) & (ay > y))
        if not hit.any():
            continue
        cx = ax[hit] + (y - ay[hit]) * (bx[hit] - ax[hit]) / (by[hit] - ay[hit])
        wind = np.where(by[hit] > ay[hit], 1, -1)
        order = np.argsort(cx)
        cx, wind = cx[order], wind[order]
        total = np.cumsum(wind)
        inside = (total != 0) if rule == 'nonzero' else (np.cumsum(np.ones_like(wind)) % 2 == 1)
        for k in range(len(cx) - 1):
            if inside[k]:
                out[row, (xs >= cx[k]) & (xs < cx[k + 1])] = 1
    return out


def stroke_coverage(subpaths, width, n):
    """Coverage of strokes `width` samples wide along the subpaths."""
    out = np.zeros((n, n), dtype=np.float32)
    half = width / 2
    gy, gx = np.mgrid[0:n, 0:n] + 0.5
    for pts, closed in subpaths:
        segs = list(zip(pts, pts[1:] + (pts[:1] if closed else [])))
        for (ax, ay), (bx, by) in segs:
            lo_x, hi_x = int(max(0, min(ax, bx) - half - 1)), int(min(n, max(ax, bx) + half + 2))
            lo_y, hi_y = int(max(0, min(ay, by) - half - 1)), int(min(n, max(ay, by) + half + 2))
            if lo_x >= hi_x or lo_y >= hi_y:
                continue
            px, py = gx[lo_y:hi_y, lo_x:hi_x], gy[lo_y:hi_y, lo_x:hi_x]
            dx, dy = bx - ax, by - ay
            ll = dx * dx + dy * dy
            t = np.clip(((px - ax) * dx + (py - ay) * dy) / ll, 0, 1) if ll > 0 else 0
            d = np.hypot(px - (ax + t * dx), py - (ay + t * dy))
            out[lo_y:hi_y, lo_x:hi_x] = np.maximum(out[lo_y:hi_y, lo_x:hi_x], (d <= half).astype(np.float32))
    return out


def gradient_coverage(shape, stops, n):
    """A radial gradient on a circle: opacity by distance from its centre, from the gradient's stops."""
    pts = shape['subpaths'][0][0]
    cx, cy = sum(p[0] for p in pts) / len(pts), sum(p[1] for p in pts) / len(pts)
    r = max(math.hypot(p[0] - cx, p[1] - cy) for p in pts)
    gy, gx = np.mgrid[0:n, 0:n] + 0.5
    u = np.clip(np.hypot(gx - cx, gy - cy) / r, 0, 1)
    offs, alphas = zip(*stops)
    return np.interp(u, offs, alphas).astype(np.float32)


def gradient_stops(root, url):
    """(offset, opacity) stops of a gradient fill url(#id), following xlink:href."""
    by_id = {el.attrib['id']: el for el in root.iter() if 'id' in el.attrib}
    el = by_id.get(url[url.find('#') + 1:url.rfind(')')])
    while el is not None and not list(el.iter(svgshapes.NS + 'stop')):
        el = by_id.get((el.attrib.get(svgshapes.XLINK) or '').lstrip('#'))
    stops = []
    for st in (el.iter(svgshapes.NS + 'stop') if el is not None else []):
        style = svgshapes.own_style(st)
        stops.append((float(st.attrib.get('offset', 0)), float(style.get('stop-opacity', 1))))
    return stops or [(0, 1), (1, 1)]


def render(root, shapes, size, fill):
    """The icon's alpha, size x size, its longer side `fill` pixels, centred."""
    x0, y0, x1, y1 = bounds(shapes)
    n = size * SS
    scale = fill * SS / max(x1 - x0, y1 - y0)
    ox, oy = (n - (x1 - x0) * scale) / 2 - x0 * scale, (n - (y1 - y0) * scale) / 2 - y0 * scale

    def to_grid(pts):
        return [(x * scale + ox, y * scale + oy) for x, y in pts]

    alpha = np.zeros((n, n), dtype=np.float32)
    for s in shapes:
        subs = [(to_grid(p), c) for p, c in s['subpaths']]
        cover = np.zeros((n, n), dtype=np.float32)
        if s['fill'].startswith('url('):
            cover = gradient_coverage({'subpaths': subs}, gradient_stops(root, s['fill']), n)
        elif s['fill'] != 'none':
            cover = fill_coverage(subs, s['fill_rule'], n) * s['fill_opacity']
        if s['stroke'] != 'none':
            cover = np.maximum(cover, stroke_coverage(subs, s['stroke_width'] * scale, n))
        cover *= s['opacity']
        alpha = alpha + cover * (1 - alpha)   # each shape over the ones before it
    return alpha.reshape(size, SS, size, SS).mean(axis=(1, 3))


def save(alpha, path, blur=0):
    a = Image.fromarray(np.clip(alpha * 255 + 0.5, 0, 255).astype(np.uint8), 'L')
    if blur:
        a = a.filter(ImageFilter.GaussianBlur(blur))
    img = Image.new('RGBA', a.size, (255, 255, 255, 0))
    img.putalpha(a)
    img.save(path, optimize=True)


def main():
    root, shapes = svgshapes.read(SVG)
    os.makedirs(OUT, exist_ok=True)
    for name, ids in ICONS.items():
        picked = shapes_for(shapes, ids)
        save(render(root, picked, ICON_SIZE, ICON_FILL), os.path.join(OUT, f'{name}.png'))
        save(render(root, picked, GLOW_SIZE, GLOW_FILL), os.path.join(OUT, f'{name}-glow.png'), GLOW_BLUR)
        print(name)


if __name__ == '__main__':
    main()
