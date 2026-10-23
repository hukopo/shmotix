"""Иконки из пиксельного котика SIT в ui/index.html: icons/tray.png и исходник app-иконки.
Запуск: python3 scripts/make-cat.py && npx tauri icon scripts/app-icon.png. Только stdlib."""
import re, struct, zlib

CAT = re.search(r"const SIT = `\n(.*?)`", open("ui/index.html").read(), re.S).group(1).strip().split("\n")
COLORS = {"o": (3, 3, 3), "#": (40, 40, 40), "W": (255, 255, 255), "n": (244, 160, 180)}  # тень g не рисуем


def png(path, pixels):
    h, w = len(pixels), len(pixels[0])
    raw = b"".join(b"\x00" + b"".join(bytes(p) for p in row) for row in pixels)
    chunk = lambda t, d: struct.pack(">I", len(d)) + t + d + struct.pack(">I", zlib.crc32(t + d))
    with open(path, "wb") as f:
        f.write(b"\x89PNG\r\n\x1a\n" + chunk(b"IHDR", struct.pack(">IIBBBBB", w, h, 8, 6, 0, 0, 0))
                + chunk(b"IDAT", zlib.compress(raw)) + chunk(b"IEND", b""))


def render(size, color, bg=(0, 0, 0, 0)):
    """Котик по центру квадрата size×size, целый масштаб пикселя."""
    scale = max(1, size * 3 // 4 // max(len(CAT), len(CAT[0])))
    ox, oy = (size - len(CAT[0]) * scale) // 2, (size - len(CAT) * scale) // 2
    out = [[bg] * size for _ in range(size)]
    for y, row in enumerate(CAT):
        for x, ch in enumerate(row):
            c = color(ch)
            if c:
                for dy in range(scale):
                    out[oy + y * scale + dy][ox + x * scale:ox + (x + 1) * scale] = [c] * scale
    return out


# трей — template-силуэт: macOS сама красит под тему
png("src-tauri/icons/tray.png", render(36, lambda ch: (0, 0, 0, 255) if ch in "o#" else None))  # глаза и нос — дырки
png("scripts/app-icon.png", render(1024, lambda ch: COLORS[ch] + (255,) if ch in COLORS else None, (242, 233, 216, 255)))
