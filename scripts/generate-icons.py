#!/usr/bin/env python3
"""Generate a minimal app icon for Tauri."""

from pathlib import Path

try:
    from PIL import Image, ImageDraw
except ImportError:
    print("Install Pillow: pip install Pillow")
    raise

root = Path(__file__).resolve().parent.parent
icons_dir = root / "src-tauri" / "icons"
icons_dir.mkdir(parents=True, exist_ok=True)


def make_icon(size: int) -> Image.Image:
    img = Image.new("RGBA", (size, size), (15, 20, 25, 255))
    draw = ImageDraw.Draw(img)
    margin = size // 10
    draw.rectangle(
        [margin, margin, size - margin, size - margin],
        outline=(20, 163, 168, 255),
        width=max(2, size // 32),
    )
    font_size = size // 3
    text = "GR"
    bbox = draw.textbbox((0, 0), text)
    tw, th = bbox[2] - bbox[0], bbox[3] - bbox[1]
    draw.text(
        ((size - tw) / 2, (size - th) / 2 - size * 0.05),
        text,
        fill=(94, 234, 212, 255),
    )
    return img


for s in (32, 128, 256, 512):
    make_icon(s).save(icons_dir / f"{s}x{s}.png")

make_icon(256).save(icons_dir / "icon.png")
make_icon(256).save(icons_dir / "128x128@2x.png")

# ICO for Windows
make_icon(256).save(icons_dir / "icon.ico", format="ICO", sizes=[(256, 256), (128, 128), (64, 64), (32, 32), (16, 16)])
print(f"Icons written to {icons_dir}")
