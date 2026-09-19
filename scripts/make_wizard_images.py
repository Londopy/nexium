#!/usr/bin/env python3
"""Draw the Windows installer's wizard images with Pillow.

    python scripts/make_wizard_images.py

Inno Setup shows WizardImageFile (164x314) on the welcome and finish pages and
WizardSmallImageFile (55x58) in the header of every other page. Both are the
mark from assets/icon-256.png on the banner's gradient, written as 24-bit BMP
into installers/windows/.
"""
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


def large():
    w, h = 164, 314
    img = gradient(w, h)
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
