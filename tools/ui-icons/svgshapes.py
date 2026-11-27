"""Reads an SVG's shapes as flattened outlines in the root's user units.

Each shape: its id, its ancestors' ids, its subpaths (point lists, closed or
not), its fill and stroke (colour strings or None), stroke width (root units),
fill rule and opacity. Text is left out. Transforms, `use`, and inherited
styles are applied.
"""
import math
import re
import xml.etree.ElementTree as ET

NS = '{http://www.w3.org/2000/svg}'
XLINK = '{http://www.w3.org/1999/xlink}href'

IDENTITY = (1.0, 0.0, 0.0, 1.0, 0.0, 0.0)


def mul(m, n):
    a, b, c, d, e, f = m
    A, B, C, D, E, F = n
    return (a * A + c * B, b * A + d * B, a * C + c * D, b * C + d * D, a * E + c * F + e, b * E + d * F + f)


def apply(m, p):
    a, b, c, d, e, f = m
    return (a * p[0] + c * p[1] + e, b * p[0] + d * p[1] + f)


def parse_transform(s):
    m = IDENTITY
    for name, args in re.findall(r'(\w+)\s*\(([^)]*)\)', s or ''):
        v = [float(x) for x in re.findall(r'[-+]?(?:\d*\.\d+|\d+\.?)(?:[eE][-+]?\d+)?', args)]
        if name == 'matrix':
            t = tuple(v)
        elif name == 'translate':
            t = (1, 0, 0, 1, v[0], v[1] if len(v) > 1 else 0)
        elif name == 'scale':
            t = (v[0], 0, 0, v[1] if len(v) > 1 else v[0], 0, 0)
        elif name == 'rotate':
            a = math.radians(v[0])
            r = (math.cos(a), math.sin(a), -math.sin(a), math.cos(a), 0, 0)
            if len(v) == 3:
                t = mul(mul((1, 0, 0, 1, v[1], v[2]), r), (1, 0, 0, 1, -v[1], -v[2]))
            else:
                t = r
        elif name == 'skewX':
            t = (1, 0, math.tan(math.radians(v[0])), 1, 0, 0)
        elif name == 'skewY':
            t = (1, math.tan(math.radians(v[0])), 0, 1, 0, 0)
        else:
            continue
        m = mul(m, t)
    return m


INHERITED = ('fill', 'stroke', 'stroke-width', 'fill-rule', 'fill-opacity', 'stroke-opacity', 'visibility')


def own_style(el):
    st = {}
    for k in INHERITED + ('display', 'opacity'):
        if k in el.attrib:
            st[k] = el.attrib[k]
    for part in (el.attrib.get('style') or '').split(';'):
        if ':' in part:
            k, v = part.split(':', 1)
            st[k.strip()] = v.strip()
    return st


NUM = re.compile(r'[-+]?(?:\d*\.\d+|\d+\.?)(?:[eE][-+]?\d+)?')
TOKEN = re.compile(r'([MmLlHhVvCcSsQqTtAaZz])|([-+]?(?:\d*\.\d+|\d+\.?)(?:[eE][-+]?\d+)?)')


def arc_points(p0, rx, ry, phi, large, sweep, p1, steps_per_turn=48):
    # SVG arc endpoint parameterisation to centre form (SVG spec F.6.5)
    if rx == 0 or ry == 0:
        return [p1]
    rx, ry = abs(rx), abs(ry)
    cp, sp = math.cos(math.radians(phi)), math.sin(math.radians(phi))
    dx, dy = (p0[0] - p1[0]) / 2, (p0[1] - p1[1]) / 2
    x1, y1 = cp * dx + sp * dy, -sp * dx + cp * dy
    lam = (x1 * x1) / (rx * rx) + (y1 * y1) / (ry * ry)
    if lam > 1:
        rx, ry = rx * math.sqrt(lam), ry * math.sqrt(lam)
    num = rx * rx * ry * ry - rx * rx * y1 * y1 - ry * ry * x1 * x1
    den = rx * rx * y1 * y1 + ry * ry * x1 * x1
    co = math.sqrt(max(0, num / den)) if den else 0
    if large == sweep:
        co = -co
    cx1, cy1 = co * rx * y1 / ry, -co * ry * x1 / rx
    cx = cp * cx1 - sp * cy1 + (p0[0] + p1[0]) / 2
    cy = sp * cx1 + cp * cy1 + (p0[1] + p1[1]) / 2

    def ang(ux, uy, vx, vy):
        a = math.atan2(ux * vy - uy * vx, ux * vx + uy * vy)
        return a
    t1 = ang(1, 0, (x1 - cx1) / rx, (y1 - cy1) / ry)
    dt = ang((x1 - cx1) / rx, (y1 - cy1) / ry, (-x1 - cx1) / rx, (-y1 - cy1) / ry)
    if not sweep and dt > 0:
        dt -= 2 * math.pi
    elif sweep and dt < 0:
        dt += 2 * math.pi
    n = max(2, int(abs(dt) / (2 * math.pi) * steps_per_turn) + 1)
    pts = []
    for i in range(1, n + 1):
        t = t1 + dt * i / n
        x, y = rx * math.cos(t), ry * math.sin(t)
        pts.append((cp * x - sp * y + cx, sp * x + cp * y + cy))
    return pts


def cubic(p0, p1, p2, p3, n=16):
    out = []
    for i in range(1, n + 1):
        t = i / n
        u = 1 - t
        out.append((u ** 3 * p0[0] + 3 * u * u * t * p1[0] + 3 * u * t * t * p2[0] + t ** 3 * p3[0],
                    u ** 3 * p0[1] + 3 * u * u * t * p1[1] + 3 * u * t * t * p2[1] + t ** 3 * p3[1]))
    return out


def quad(p0, p1, p2, n=12):
    out = []
    for i in range(1, n + 1):
        t = i / n
        u = 1 - t
        out.append((u * u * p0[0] + 2 * u * t * p1[0] + t * t * p2[0], u * u * p0[1] + 2 * u * t * p1[1] + t * t * p2[1]))
    return out


def parse_path(d):
    """Subpaths of a path's d attribute: a list of (points, closed)."""
    toks = []
    for cmd, num in TOKEN.findall(d or ''):
        toks.append(cmd if cmd else float(num))
    subs, cur, start = [], [], (0.0, 0.0)
    pos = (0.0, 0.0)
    last_ctrl, last_cmd = None, ''
    i, cmd = 0, None

    def take(k):
        nonlocal i
        v = toks[i:i + k]
        i += k
        return v

    def flush(closed):
        nonlocal cur
        if len(cur) > 1:
            subs.append((cur, closed))
        cur = []

    while i < len(toks):
        if isinstance(toks[i], str):
            cmd = toks[i]
            i += 1
            if cmd in 'Zz':
                flush(True)
                pos = start
                last_cmd = 'Z'
                continue
        elif cmd is None:
            break
        rel = cmd.islower()
        C = cmd.upper()
        ox, oy = pos if rel else (0.0, 0.0)
        if C == 'M':
            x, y = take(2)
            flush(False)
            pos = (ox + x, oy + y)
            start = pos
            cur = [pos]
            cmd = 'l' if rel else 'L'   # further pairs are lines
            last_ctrl = None
        elif C == 'L':
            x, y = take(2)
            pos = (ox + x, oy + y)
            if not cur:
                cur = [start]
            cur.append(pos)
            last_ctrl = None
        elif C == 'H':
            (x,) = take(1)
            pos = (ox + x if rel else x, pos[1])
            if not cur:
                cur = [start]
            cur.append(pos)
            last_ctrl = None
        elif C == 'V':
            (y,) = take(1)
            pos = (pos[0], oy + y if rel else y)
            if not cur:
                cur = [start]
            cur.append(pos)
            last_ctrl = None
        elif C == 'C':
            x1, y1, x2, y2, x, y = take(6)
            p1, p2, p3 = (ox + x1, oy + y1), (ox + x2, oy + y2), (ox + x, oy + y)
            if not cur:
                cur = [start]
            cur += cubic(pos, p1, p2, p3)
            last_ctrl, pos = p2, p3
        elif C == 'S':
            x2, y2, x, y = take(4)
            p1 = (2 * pos[0] - last_ctrl[0], 2 * pos[1] - last_ctrl[1]) if last_cmd in 'CS' and last_ctrl else pos
            p2, p3 = (ox + x2, oy + y2), (ox + x, oy + y)
            if not cur:
                cur = [start]
            cur += cubic(pos, p1, p2, p3)
            last_ctrl, pos = p2, p3
        elif C == 'Q':
            x1, y1, x, y = take(4)
            p1, p2 = (ox + x1, oy + y1), (ox + x, oy + y)
            if not cur:
                cur = [start]
            cur += quad(pos, p1, p2)
            last_ctrl, pos = p1, p2
        elif C == 'T':
            x, y = take(2)
            p1 = (2 * pos[0] - last_ctrl[0], 2 * pos[1] - last_ctrl[1]) if last_cmd in 'QT' and last_ctrl else pos
            p2 = (ox + x, oy + y)
            if not cur:
                cur = [start]
            cur += quad(pos, p1, p2)
            last_ctrl, pos = p1, p2
        elif C == 'A':
            rx, ry, phi, large, sweep, x, y = take(7)
            p1 = (ox + x, oy + y)
            if not cur:
                cur = [start]
            cur += arc_points(pos, rx, ry, phi, int(large), int(sweep), p1)
            pos = p1
            last_ctrl = None
        last_cmd = C
    flush(False)
    return subs


def ellipse_points(cx, cy, rx, ry, n=48):
    return [(cx + rx * math.cos(2 * math.pi * i / n), cy + ry * math.sin(2 * math.pi * i / n)) for i in range(n)]


def f(el, k, default=0.0):
    v = el.attrib.get(k)
    if v is None:
        return default
    m = NUM.match(v.strip())
    return float(m.group(0)) if m else default


def element_subpaths(el, tag):
    if tag == 'path':
        return parse_path(el.attrib.get('d'))
    if tag == 'rect':
        x, y, w, h = f(el, 'x'), f(el, 'y'), f(el, 'width'), f(el, 'height')
        rx = f(el, 'rx', None) if 'rx' in el.attrib else None
        ry = f(el, 'ry', None) if 'ry' in el.attrib else None
        rx = rx if rx is not None else (ry or 0)
        ry = ry if ry is not None else rx
        rx, ry = min(rx, w / 2), min(ry, h / 2)
        if rx > 0 and ry > 0:
            pts = []
            for cx, cy, a0 in ((x + w - rx, y + ry, -90), (x + w - rx, y + h - ry, 0), (x + rx, y + h - ry, 90), (x + rx, y + ry, 180)):
                for i in range(9):
                    a = math.radians(a0 + 90 * i / 8)
                    pts.append((cx + rx * math.cos(a), cy + ry * math.sin(a)))
            return [(pts, True)]
        return [([(x, y), (x + w, y), (x + w, y + h), (x, y + h)], True)]
    if tag == 'circle':
        r = f(el, 'r')
        return [(ellipse_points(f(el, 'cx'), f(el, 'cy'), r, r), True)]
    if tag == 'ellipse':
        return [(ellipse_points(f(el, 'cx'), f(el, 'cy'), f(el, 'rx'), f(el, 'ry')), True)]
    if tag in ('polygon', 'polyline'):
        v = [float(x) for x in NUM.findall(el.attrib.get('points', ''))]
        pts = list(zip(v[0::2], v[1::2]))
        return [(pts, tag == 'polygon')]
    if tag == 'line':
        return [([(f(el, 'x1'), f(el, 'y1')), (f(el, 'x2'), f(el, 'y2'))], False)]
    return []


def read(path):
    root = ET.parse(path).getroot()
    by_id = {el.attrib['id']: el for el in root.iter() if 'id' in el.attrib}
    shapes = []

    def walk(el, m, style, ancestors, in_defs):
        tag = el.tag.replace(NS, '')
        if tag in ('defs', 'symbol', 'title', 'desc', 'metadata', 'text', 'flowRoot', 'clipPath', 'mask', 'linearGradient',
                   'radialGradient', 'filter', 'pattern', 'marker') and not in_defs:
            return
        st = dict(style)
        own = own_style(el)
        if own.get('display') == 'none':
            return
        for k in INHERITED:
            if k in own:
                st[k] = own[k]
        st['opacity'] = style.get('opacity', 1.0) * float(own.get('opacity', 1) or 1)
        m = mul(m, parse_transform(el.attrib.get('transform')))
        ids = ancestors + [el.attrib.get('id', '')]
        if tag == 'use':
            ref = by_id.get((el.attrib.get(XLINK) or el.attrib.get('href') or '').lstrip('#'))
            if ref is not None:
                mm = mul(m, (1, 0, 0, 1, f(el, 'x'), f(el, 'y')))
                if ref.tag.replace(NS, '') == 'symbol':
                    for ch in ref:
                        walk(ch, mm, st, ids, True)
                else:
                    walk(ref, mm, st, ids, True)
            return
        subs = element_subpaths(el, tag)
        if subs:
            scale = math.sqrt(abs(m[0] * m[3] - m[1] * m[2]))
            sw = st.get('stroke-width', '1')
            mw = NUM.match(str(sw))
            shapes.append({
                'id': el.attrib.get('id', ''), 'ancestors': ancestors, 'tag': tag,
                'subpaths': [([apply(m, p) for p in pts], closed) for pts, closed in subs],
                'fill': st.get('fill', '#000000'), 'stroke': st.get('stroke', 'none'),
                'stroke_width': (float(mw.group(0)) if mw else 1.0) * scale,
                'fill_rule': st.get('fill-rule', 'nonzero'), 'opacity': st['opacity'],
                'fill_opacity': float(st.get('fill-opacity', 1) or 1),
            })
        for ch in el:
            walk(ch, m, st, ids, in_defs)

    walk(root, IDENTITY, {'opacity': 1.0}, [], False)
    return root, shapes
