"""Рисует пиксельного котика: ui/cat.png, icons/tray.png и исходник app-иконки. Только stdlib."""
import struct, sys, zlib

CAT = """
................
...#......#.....
...##....##.....
...########.....
..##########....
..##Y####Y##....
..##########....
...########.....
...########.....
..##########....
..##########..#.
..##########..#.
..##########.##.
..############..
...##.####.##...
................
"""
ROWS = CAT.strip().split("\n")
BLACK, EYE, CLEAR = (17, 17, 17, 255), (255, 214, 0, 255), (0, 0, 0, 0)


def png(path, pixels):
    h, w = len(pixels), len(pixels[0])
    raw = b"".join(b"\x00" + b"".join(bytes(p) for p in row) for row in pixels)
    chunk = lambda t, d: struct.pack(">I", len(d)) + t + d + struct.pack(">I", zlib.crc32(t + d))
    with open(path, "wb") as f:
        f.write(b"\x89PNG\r\n\x1a\n" + chunk(b"IHDR", struct.pack(">IIBBBBB", w, h, 8, 6, 0, 0, 0))
                + chunk(b"IDAT", zlib.compress(raw)) + chunk(b"IEND", b""))


def render(scale, eyes, bg=CLEAR, pad=0):
    color = {"#": BLACK, "Y": EYE if eyes else CLEAR, ".": bg}
    n = 16 * scale + 2 * pad
    out = [[bg] * n for _ in range(n)]
    for y, row in enumerate(ROWS):
        for x, ch in enumerate(row):
            for dy in range(scale):
                for dx in range(scale):
                    out[pad + y * scale + dy][pad + x * scale + dx] = color[ch]
    return out


root = sys.argv[1] if len(sys.argv) > 1 else "."
png(f"{root}/ui/cat.png", render(4, eyes=True))
png(f"{root}/src-tauri/icons/tray.png", render(2, eyes=False))  # template-иконка трея: только силуэт
png(f"{root}/scripts/app-icon.png", render(48, eyes=True, bg=(242, 233, 216, 255), pad=128))
