# /// script
# requires-python = ">=3.11"
# dependencies = ["numpy==2.5.5", "pillow==13.0.0", "shapely==2.2.0"]
# ///
import math
import subprocess
import sys
import tempfile
from pathlib import Path

import numpy as np
from PIL import Image

import neuron

HERE = Path(__file__).resolve().parent
ASSETS = HERE.parents[1] / "neuronify-core" / "assets"

BACKGROUND = ("#327dda", "#1b64c2")
INK = ("#ffffff", "#e0ebf9")
SHADOW = "#0a2457"
SHADOW_OPACITY = 0.4
SHADOW_BLUR = 11.2
SHADOW_DROP = 12.5
TILE_RADIUS = 0.225

MACOS_BODY = 824
MACOS_RADIUS = 185.4
MACOS_SMOOTHING = 0.6
MACOS_SHADOW_OPACITY = 0.5
MACOS_SHADOW_BLUR = 14
MACOS_SHADOW_DROP = 12


def styled(drawing, ink=True):
    d, transform = drawing
    defs = (
        f'<linearGradient id="bg" x1="0" y1="0" x2="0" y2="1"><stop offset="0" stop-color="{BACKGROUND[0]}"/><stop offset="1" stop-color="{BACKGROUND[1]}"/></linearGradient>\n'
        f'<linearGradient id="ink" x1="0" y1="0" x2="0" y2="1"><stop offset="0" stop-color="{INK[0]}"/><stop offset="1" stop-color="{INK[1]}"/></linearGradient>\n'
        '<filter id="shadow" x="-10%" y="-10%" width="120%" height="125%" color-interpolation-filters="sRGB">\n'
        f'<feGaussianBlur in="SourceAlpha" stdDeviation="{SHADOW_BLUR:g}"/>\n'
        f'<feOffset dy="{SHADOW_DROP:g}" result="blurred"/>\n'
        f'<feFlood flood-color="{SHADOW}" flood-opacity="{SHADOW_OPACITY:.2f}"/>\n'
        '<feComposite in2="blurred" operator="in" result="shadow"/>\n'
        '<feMerge><feMergeNode in="shadow"/><feMergeNode in="SourceGraphic"/></feMerge>\n'
        "</filter>\n"
    )
    fill = 'fill="url(#ink)" fill-rule="evenodd" filter="url(#shadow)"' if ink else f'fill="{INK[0]}" fill-rule="evenodd"'
    content = f'<rect width="1024" height="1024" fill="url(#bg)"/>\n<path transform="{transform}" {fill} d="{d}"/>\n'
    return defs, content


def document(defs, content):
    return f'<svg xmlns="http://www.w3.org/2000/svg" viewBox="0 0 1024 1024" width="1024" height="1024">\n<defs>\n{defs}</defs>\n{content}</svg>\n'


def squircle(x0, y0, w, r, smoothing=MACOS_SMOOTHING):
    p = (1 + smoothing) * r
    arc = 90 * (1 - smoothing)
    chord = math.sin(math.radians(arc / 2)) * r * math.sqrt(2)
    p34 = r * math.tan(math.radians((90 - arc) / 4))
    beta = 45 * smoothing
    c = p34 * math.cos(math.radians(beta))
    d = c * math.tan(math.radians(beta))
    b = (p - chord - c - d) / 3
    a = 2 * b
    f = lambda *v: " ".join(f"{x:.3f}".rstrip("0").rstrip(".") for x in v)
    return (
        f"M{f(x0 + p, y0)}L{f(x0 + w - p, y0)}"
        f"c{f(a, 0, a + b, 0, a + b + c, d)}a{f(r, r)} 0 0 1 {f(chord, chord)}c{f(d, c, d, b + c, d, a + b + c)}"
        f"L{f(x0 + w, y0 + w - p)}"
        f"c{f(0, a, 0, a + b, -d, a + b + c)}a{f(r, r)} 0 0 1 {f(-chord, chord)}c{f(-c, d, -(b + c), d, -(a + b + c), d)}"
        f"L{f(x0 + p, y0 + w)}"
        f"c{f(-a, 0, -(a + b), 0, -(a + b + c), -d)}a{f(r, r)} 0 0 1 {f(-chord, -chord)}c{f(-d, -c, -d, -(b + c), -d, -(a + b + c))}"
        f"L{f(x0, y0 + p)}"
        f"c{f(0, -a, 0, -(a + b), d, -(a + b + c))}a{f(r, r)} 0 0 1 {f(chord, -chord)}c{f(c, -d, b + c, -d, a + b + c, -d)}Z"
    )


def macos_body(size):
    if size >= 1024:
        return (1024 - MACOS_BODY) / 2, float(MACOS_BODY)
    body = 2 * round(MACOS_BODY / 1024 * size / 2) if size > 16 else 14
    return (size - body) / 2 * 1024 / size, body * 1024 / size


def macos(defs, content, size):
    x, w = macos_body(size)
    outline = squircle(x, x, w, MACOS_RADIUS / MACOS_BODY * w)
    defs += (
        f'<clipPath id="body"><path d="{outline}"/></clipPath>\n'
        '<filter id="drop" x="-20%" y="-20%" width="140%" height="150%" color-interpolation-filters="sRGB">\n'
        f'<feGaussianBlur in="SourceAlpha" stdDeviation="{MACOS_SHADOW_BLUR}"/>\n'
        f'<feOffset dy="{MACOS_SHADOW_DROP}"/>\n'
        f'<feComponentTransfer><feFuncA type="linear" slope="{MACOS_SHADOW_OPACITY}"/></feComponentTransfer>\n'
        "</filter>\n"
    )
    content = (
        f'<path d="{outline}" fill="#000" filter="url(#drop)"/>\n'
        f'<g clip-path="url(#body)"><g transform="translate({x:.3f} {x:.3f}) scale({w / 1024:.6f})">{content}</g></g>\n'
    )
    return document(defs, content)


def tile_alpha(size, supersampling=4):
    s = size * supersampling
    y, x = np.mgrid[0:s, 0:s] + 0.5
    r = TILE_RADIUS * s
    inside = (np.hypot(x - np.clip(x, r, s - r), y - np.clip(y, r, s - r)) <= r).astype(float)
    return inside.reshape(size, supersampling, size, supersampling).mean(axis=(1, 3))


class Renderer:
    def __init__(self, directory):
        self.svg = Path(directory) / "render.svg"
        self.png = Path(directory) / "render.png"

    def __call__(self, svg, size):
        self.svg.write_text(svg)
        subprocess.run(
            ["inkscape", str(self.svg), "--export-type=png", f"--export-filename={self.png}", "-w", str(size), "-h", str(size)],
            check=True,
            capture_output=True,
        )
        with Image.open(self.png) as image:
            return image.convert("RGBA")

    def tile(self, svg, size):
        rgb = np.asarray(self(svg, size).convert("RGB"))
        alpha = (tile_alpha(size) * 255).astype(np.uint8)
        return Image.fromarray(np.dstack([rgb, alpha]), "RGBA")


def main():
    out = Path(sys.argv[1]) if len(sys.argv) > 1 else ASSETS
    iconset = out / "macos.iconset"
    iconset.mkdir(parents=True, exist_ok=True)
    master = styled(neuron.master())
    small = neuron.small()
    small_sizes = {16: styled(small, ink=False), 32: styled(small)}
    (out / "icon.svg").write_text(document(*master))
    with tempfile.TemporaryDirectory() as directory:
        render = Renderer(directory)
        render.tile(document(*master), 256).save(out / "icon.png")
        pixel_art = [Image.open(HERE / f"pixel-{size}.png") for size in (16, 32)]
        render.tile(document(*master), 48).save(out / "favicon.ico", format="ICO", sizes=[(48, 48), (32, 32), (16, 16)], append_images=pixel_art)
        images = {size: render(macos(*small_sizes.get(size, master), size), size) for size in (16, 32, 64, 128, 256, 512, 1024)}
    for size in (16, 32, 128, 256, 512):
        images[size].save(iconset / f"icon_{size}x{size}.png", optimize=True)
        images[size * 2].save(iconset / f"icon_{size}x{size}@2x.png", optimize=True)


if __name__ == "__main__":
    main()
