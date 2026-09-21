#!/usr/bin/env python3
"""Draw the Windows installer's wizard images with Pillow.

    python scripts/make_wizard_images.py

Inno Setup shows WizardImageFile (164x314) on the welcome and finish pages and
WizardSmallImageFile (55x58) in the header of every other page. Both are the
mark from assets/icon-256.png on the banner's gradient, the large one over the
social preview's topographic contours, written as 24-bit BMP into
installers/windows/.
"""
import math
import os
from PIL import Image, ImageDraw, ImageFont

root = os.path.dirname(os.path.dirname(os.path.abspath(__file__)))
icon = Image.open(os.path.join(root, "assets", "icon-256.png")).convert("RGBA")
out_dir = os.path.join(root, "installers", "windows")

TOP, BOTTOM = (27, 31, 59), (14, 42, 63)  # assets/banner.svg's gradient


def gradient(w, h):
    img = Image.new("RGB", (w, h))
    px = img.load()
    for y in range(h):
        for x in range(w):
            t = (x / max(w - 1, 1) + y / max(h - 1, 1)) / 2
            px[x, y] = tuple(int(a + (b - a) * t) for a, b in zip(TOP, BOTTOM))
    return img


def height(x, y, w, h):
    """The social preview's ground: a summit behind the mark, a ridge upper
    right, a rise lower left, and a gentle swell so the lines wander."""
    peaks = [(0.5 * w, 0.36 * h, 1.0, 0.62 * w), (1.05 * w, 0.06 * h, 0.7, 0.7 * w), (0.0, 0.92 * h, 0.55, 0.6 * w)]
    v = 0.0
    for px, py, a, sp in peaks:
        dx, dy = (x - px) / sp, (y - py) / (sp * 0.82)
        v += a * math.exp(-(dx * dx + dy * dy))
    v += 0.06 * math.sin(x / (w * 0.15) + 0.8) * math.cos(y / (h * 0.12) - 0.4)
    v += 0.03 * math.sin(x / (w * 0.05)) * math.sin(y / (h * 0.04) + 1.1)
    return v


def contours(img, levels=14, scale=4, alpha=0.11):
    """Contour lines of the height field over the gradient, drawn at `scale`
    times the size and shrunk for smooth edges, as on the social preview."""
    w, h = img.size
    W, H = w * scale, h * scale
    lvl = [[int(height(x / scale, y / scale, w, h) * levels) for x in range(W)] for y in range(H)]
    layer = Image.new("L", (W, H), 0)
    px = layer.load()
    for y in range(H - 1):
        row, below = lvl[y], lvl[y + 1]
        for x in range(W - 1):
            if row[x] != row[x + 1] or row[x] != below[x]:
                px[x, y] = 255
                px[x + 1, y] = 255
                px[x, y + 1] = 255
    layer = layer.resize((w, h), Image.LANCZOS)
    tint = Image.new("RGB", (w, h), (235, 240, 255))
    img.paste(tint, (0, 0), layer.point(lambda v: int(v * alpha)))
    return img


def large():
    w, h = 164, 314
    img = contours(gradient(w, h))
    mark = icon.resize((104, 104), Image.LANCZOS)
    img.paste(mark, ((w - 104) // 2, 70), mark)
    d = ImageDraw.Draw(img)
    try:
        font = ImageFont.truetype("segoeuib.ttf", 26)
        small = ImageFont.truetype("segoeui.ttf", 12)
    except OSError:
        font = small = ImageFont.load_default()
    tw = d.textlength("Nexium", font=font)
    d.text(((w - tw) / 2, 190), "Nexium", fill=(232, 236, 255), font=font)
    line = "high-level by default"
    tw = d.textlength(line, font=small)
    d.text(((w - tw) / 2, 228), line, fill=(160, 170, 200), font=small)
    line = "low-level when needed"
    tw = d.textlength(line, font=small)
    d.text(((w - tw) / 2, 246), line, fill=(160, 170, 200), font=small)
    img.save(os.path.join(out_dir, "wizard-large.bmp"))


def small():
    w, h = 55, 58
    img = gradient(w, h)
    mark = icon.resize((48, 48), Image.LANCZOS)
    img.paste(mark, ((w - 48) // 2, (h - 48) // 2), mark)
    img.save(os.path.join(out_dir, "wizard-small.bmp"))


large()
small()
print("wrote installers/windows/wizard-large.bmp and wizard-small.bmp")
