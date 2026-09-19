#!/usr/bin/env python3
"""Draw assets/icon.ico (the logo as a Windows icon) with Pillow.

    python scripts/make_icon.py

The mark is the same as assets/logo.svg: a dark rounded hexagon with an N
drawn as a path through four nodes. Sizes 16 to 256 go into one .ico.
"""
import os, math
from PIL import Image, ImageDraw

root = os.path.dirname(os.path.dirname(os.path.abspath(__file__)))
out = os.path.join(root, "assets", "icon.ico")

def render(size):
    s = 8  # supersample for smooth edges
    W = size * s
    img = Image.new("RGBA", (W, W), (0, 0, 0, 0))
    d = ImageDraw.Draw(img)
    cx = cy = W / 2
    r = W * 0.46
    hexagon = [(cx + r * math.cos(math.radians(a)), cy + r * math.sin(math.radians(a))) for a in range(-90, 270, 60)]
    d.polygon(hexagon, fill=(22, 30, 54, 255))
    inner = [(cx + r * 0.86 * math.cos(math.radians(a)), cy + r * 0.86 * math.sin(math.radians(a))) for a in range(-90, 270, 60)]
    d.polygon(inner, outline=(255, 255, 255, 22), width=max(1, W // 128))
    # the N: nodes at the corners of a rectangle
    x0, x1 = cx - r * 0.42, cx + r * 0.42
    y0, y1 = cy - r * 0.40, cy + r * 0.40
    stroke = max(2, int(W * 0.065))
    teal, violet = (79, 209, 197, 255), (139, 124, 246, 255)
    def grad(t):
        return tuple(int(teal[i] + (violet[i] - teal[i]) * t) for i in range(3)) + (255,)
    # draw the N path in segments with a gradient
    path = [(x0, y1), (x0, y0), (x1, y1), (x1, y0)]
    n = 24
    for seg in range(3):
        (ax, ay), (bx, by) = path[seg], path[seg + 1]
        for k in range(n):
            t0, t1 = k / n, (k + 1) / n
            p0 = (ax + (bx - ax) * t0, ay + (by - ay) * t0)
            p1 = (ax + (bx - ax) * t1, ay + (by - ay) * t1)
            d.line([p0, p1], fill=grad((seg + t0) / 3), width=stroke)
    node_r = stroke * 0.85
    for i, (px, py) in enumerate(path):
        col = grad(i / 3)
        d.ellipse([px - node_r, py - node_r, px + node_r, py + node_r], fill=(11, 16, 32, 255), outline=col, width=max(1, stroke // 2))
        d.ellipse([px - node_r * 0.32, py - node_r * 0.32, px + node_r * 0.32, py + node_r * 0.32], fill=col)
    return img.resize((size, size), Image.LANCZOS)

sizes = [16, 24, 32, 48, 64, 128, 256]
images = [render(sz) for sz in sizes]
images[-1].save(out, format="ICO", sizes=[(sz, sz) for sz in sizes], append_images=images[:-1])
png = os.path.join(root, "assets", "icon-256.png")
images[-1].save(png)
print("wrote", out, "and", png)
