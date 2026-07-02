import os

import pytesseract
from PIL import Image, ImageFilter, ImageOps

from engine import paths

_tessdata = paths.tessdata_dir()
if _tessdata:
    os.environ.setdefault("TESSDATA_PREFIX", _tessdata)

pytesseract.pytesseract.tesseract_cmd = paths.tesseract_exe()


def _preprocess(img: Image.Image) -> Image.Image:
    w, h = img.size
    img = img.resize((w * 2, h * 2), Image.LANCZOS)
    img = img.convert("L")
    avg = sum(img.getdata()) / (img.width * img.height)
    if avg < 128:
        img = ImageOps.invert(img)
    img = img.point(lambda p: 255 if p > 140 else 0)
    img = img.filter(ImageFilter.SHARPEN)
    return img


def run_ocr(img: Image.Image) -> str:
    processed = _preprocess(img)
    text = pytesseract.image_to_string(processed, config="--psm 6 --oem 3")
    return text.strip()
