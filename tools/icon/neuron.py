import math

from shapely.geometry import LineString, MultiPolygon, Point, Polygon
from shapely.geometry.polygon import orient
from shapely.ops import unary_union

SOMA = [(437, 360), (545, 440), (537, 558), (420, 621), (310, 548), (318, 422)]
DENDRITES = [
    (0, (433, 272), [(372, 209), (491, 208)]),
    (5, (238, 390), [(199, 285), (142, 429)]),
    (1, (635, 407), [(697, 320), (742, 434)]),
    (4, (237, 575), [(140, 546), (196, 677)]),
    (3, (407, 700), [(328, 792), (461, 792)]),
]
AXON = [
    ((537, 558), (722, 672)),
    ((722, 672), (815, 622)),
    ((815, 622), (885, 683)),
    ((722, 672), (744, 756)),
    ((744, 756), (696, 836)),
    ((744, 756), (841, 801)),
]
AXON_BEND = [(815, 622), (856, 541), (937, 538)]
AXON_FORKS = [
    ((722, 672), [(815, 622), (744, 756)]),
    ((815, 622), [(856, 541), (885, 683)]),
    ((744, 756), [(696, 836), (841, 801)]),
]
SMALL_AXON = [
    ((537, 558), (722, 672)),
    ((722, 672), (815, 622)),
    ((722, 672), (744, 756)),
]
SMALL_AXON_FORKS = [((722, 672), [(815, 622), (744, 756)])]

STROKE = 27.8
NUCLEUS = 62
CORNERS = {
    "crotch": 10,
    "fork_side": 12,
    "soma_join": 14,
    "hole": 15,
    "bend_inner": 10,
    "bend_outer": 23,
    "tip": 12,
    "nucleus": 13,
}
SAFE_ZONE = 0.8

SMALL_STROKE = 73
SMALL_NUCLEUS = 50
SMALL_TRUNK = 1.3
SMALL_FILL = 0.8


def lerp(a, b, t):
    return (a[0] + (b[0] - a[0]) * t, a[1] + (b[1] - a[1]) * t)


def skeleton(full):
    segments, joints, forks = [], list(SOMA), []
    trunk = 1.0 if full else SMALL_TRUNK
    for corner, fork, tips in DENDRITES:
        start = SOMA[corner]
        end = lerp(start, fork, trunk)
        segments.append((start, end))
        if full:
            ends = [(end[0] + x - fork[0], end[1] + y - fork[1]) for x, y in tips]
            segments += [(end, tip) for tip in ends]
            joints.append(end)
            forks.append((end, ends))
    axon_forks = AXON_FORKS if full else SMALL_AXON_FORKS
    segments += AXON if full else SMALL_AXON
    joints += [point for point, _ in axon_forks]
    forks += axon_forks
    return segments, joints, forks


def outline(stroke, full):
    segments, joints, forks = skeleton(full)
    half = stroke / 2
    parts = []
    if full:
        parts.append(LineString(AXON_BEND).buffer(half, cap_style="flat", join_style="mitre", mitre_limit=10))
    parts += [LineString(segment).buffer(half, cap_style="flat") for segment in segments]
    parts.append(Polygon(SOMA).exterior.buffer(half, join_style="mitre", mitre_limit=10))
    parts += [Point(joint).buffer(half * 0.98, quad_segs=16) for joint in joints]
    return unary_union(parts), forks


def clean_ring(coords, min_turn=0.5, min_edge=0.3):
    points = [tuple(p) for p in coords[:-1]]
    changed = True
    while changed:
        changed = False
        kept = []
        for i, p in enumerate(points):
            a, b = points[i - 1], points[(i + 1) % len(points)]
            if math.dist(a, p) < min_edge or abs(turn_at(a, p, b)) < min_turn:
                changed = True
                continue
            kept.append(p)
        points = kept
    return points


def turn_at(a, p, b):
    u = (p[0] - a[0], p[1] - a[1])
    v = (b[0] - p[0], b[1] - p[1])
    return math.degrees(math.atan2(u[0] * v[1] - u[1] * v[0], u[0] * v[0] + u[1] * v[1]))


def unit(a, b):
    d = math.dist(a, b)
    return ((b[0] - a[0]) / d, (b[1] - a[1]) / d)


def between(v, d1, d2):
    c1 = d1[0] * v[1] - d1[1] * v[0]
    c2 = v[0] * d2[1] - v[1] * d2[0]
    c12 = d1[0] * d2[1] - d1[1] * d2[0]
    return (c1 >= 0) == (c12 >= 0) and (c2 >= 0) == (c12 >= 0)


def corner_kind(p, turn, outer, forks):
    if not outer:
        return "hole"
    if turn < 0:
        return "tip" if abs(abs(turn) - 90) < 15 else "bend_outer"
    if math.dist(p, AXON_BEND[1]) < 40:
        return "bend_inner"
    fork, ends = min(forks, key=lambda f: math.dist(p, f[0]))
    if min(math.dist(p, s) for s in SOMA) < math.dist(p, fork):
        return "soma_join"
    if between(unit(fork, p), unit(fork, ends[0]), unit(fork, ends[1])):
        return "crotch"
    return "fork_side"


def rounded_path(points, radius_of):
    plan = []
    for i, p in enumerate(points):
        a, b = points[i - 1], points[(i + 1) % len(points)]
        turn = turn_at(a, p, b)
        r = radius_of(p, turn)
        ua, ub = unit(p, a), unit(p, b)
        theta = math.acos(max(-1, min(1, ua[0] * ub[0] + ua[1] * ub[1])))
        t = r / math.tan(theta / 2) if r > 0 else 0
        limit = 0.5 * min(math.dist(a, p), math.dist(b, p))
        if t > limit:
            r *= limit / t
            t = limit
        t1 = (p[0] + ua[0] * t, p[1] + ua[1] * t)
        t2 = (p[0] + ub[0] * t, p[1] + ub[1] * t)
        plan.append((p, t1, t2, r, 1 if turn > 0 else 0))
    d = [f"M{plan[-1][2][0]:.2f},{plan[-1][2][1]:.2f}"]
    for p, t1, t2, r, sweep in plan:
        if r > 0:
            d.append(f"L{t1[0]:.2f},{t1[1]:.2f}A{r:.2f},{r:.2f} 0 0 {sweep} {t2[0]:.2f},{t2[1]:.2f}")
        else:
            d.append(f"L{p[0]:.2f},{p[1]:.2f}")
    return "".join(d) + "Z"


def nucleus_points(size):
    soma = Polygon(SOMA)
    c = soma.centroid
    apothems = []
    for a, b in zip(SOMA, SOMA[1:] + SOMA[:1]):
        ex, ey = b[0] - a[0], b[1] - a[1]
        apothems.append(abs((c.x - a[0]) * ey - (c.y - a[1]) * ex) / math.hypot(ex, ey))
    inset = sum(apothems) / len(apothems) - size * math.cos(math.radians(30))
    return clean_ring(list(soma.buffer(-inset, join_style="mitre").exterior.coords))


def path(stroke, corners, nucleus, full):
    shape, forks = outline(stroke, full)
    polygons = list(shape.geoms) if isinstance(shape, MultiPolygon) else [shape]
    d = []
    for polygon in polygons:
        polygon = orient(polygon, -1.0)
        for k, ring in enumerate([polygon.exterior, *polygon.interiors]):
            d.append(rounded_path(clean_ring(list(ring.coords)), lambda p, turn, outer=(k == 0): corners[corner_kind(p, turn, outer, forks)]))
    d.append(rounded_path(nucleus_points(nucleus), lambda p, turn: corners["nucleus"]))
    return "".join(d)


def placed(stroke, corners, nucleus, full, fit):
    scale = 1.0
    for _ in range(4):
        shape, _ = outline(stroke / scale, full)
        minx, miny, maxx, maxy = shape.bounds
        centre = ((minx + maxx) / 2, (miny + maxy) / 2)
        far = max(math.dist(centre, p) for p in shape.exterior.coords)
        scale = fit(far, max(maxx - minx, maxy - miny))
    d = path(stroke / scale, {k: v / scale for k, v in corners.items()}, nucleus, full)
    transform = f"translate(512 512) scale({scale:.5f}) translate({-centre[0]:.2f} {-centre[1]:.2f})"
    return d, transform


def master():
    return placed(STROKE, CORNERS, NUCLEUS, True, lambda far, size: SAFE_ZONE / 2 * 1024 / far)


def small():
    corners = {k: v * SMALL_STROKE / STROKE for k, v in CORNERS.items()}
    corners["nucleus"] = CORNERS["nucleus"] * SMALL_NUCLEUS / NUCLEUS
    return placed(SMALL_STROKE, corners, SMALL_NUCLEUS, False, lambda far, size: SMALL_FILL * 1024 / size)
