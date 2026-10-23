"""Окно-лупа: прозрачная рамка, которую ставят на субтитры; перевод — в полосе снизу."""
import argparse
import sys
import threading
import time
import tkinter as tk

import mss
from PIL import Image

from .engine import Engine

BORDER = 3
BAR_H = 60
KEY = "#ff00fe"  # цвет-«дырка» для Windows


class Lens:
    def __init__(self, engine: Engine, interval: float):
        self.engine, self.interval = engine, interval
        self.root = tk.Tk()
        self.root.overrideredirect(True)
        self.root.attributes("-topmost", True)
        self.root.geometry("700x180+200+600")

        if sys.platform == "darwin":
            self.root.attributes("-transparent", True)
            hole = "systemTransparent"
        else:
            self.root.attributes("-transparentcolor", KEY)
            hole = KEY
        self.root.config(bg="#00c853")  # рамка = фон окна, видна по краям

        self.hole = tk.Frame(self.root, bg=hole)
        self.hole.place(x=BORDER, y=BORDER, relwidth=1, relheight=1,
                        width=-2 * BORDER, height=-BAR_H - BORDER)
        self.bar = tk.Label(self.root, text="Наведи рамку на субтитры…", bg="#111", fg="#fff",
                            font=("Helvetica", 16), wraplength=680, justify="center")
        self.bar.place(x=0, rely=1, y=-BAR_H, relwidth=1, height=BAR_H)
        grip = tk.Label(self.root, text="◢", bg="#111", fg="#888", cursor="sizing")
        grip.place(relx=1, rely=1, anchor="se")

        self.bar.bind("<ButtonPress-1>", self._start)
        self.bar.bind("<B1-Motion>", self._drag)
        grip.bind("<ButtonPress-1>", self._start)
        grip.bind("<B1-Motion>", self._resize)
        self.root.bind("<Escape>", lambda e: self.root.destroy())
        self.root.bind("<Configure>", lambda e: self.bar.config(wraplength=self.root.winfo_width() - 20))

        threading.Thread(target=self._loop, daemon=True).start()

    def _start(self, e):
        self._ox, self._oy = e.x_root, e.y_root
        self._geo = (self.root.winfo_x(), self.root.winfo_y(),
                     self.root.winfo_width(), self.root.winfo_height())

    def _drag(self, e):
        x, y, *_ = self._geo
        self.root.geometry(f"+{x + e.x_root - self._ox}+{y + e.y_root - self._oy}")

    def _resize(self, e):
        _, _, w, h = self._geo
        w = max(200, w + e.x_root - self._ox)
        h = max(BAR_H + 40, h + e.y_root - self._oy)
        self.root.geometry(f"{w}x{h}")

    def region(self):
        r = self.root
        return {"left": r.winfo_rootx() + BORDER, "top": r.winfo_rooty() + BORDER,
                "width": r.winfo_width() - 2 * BORDER, "height": r.winfo_height() - BAR_H - BORDER}

    def _loop(self):
        with mss.mss() as sct:
            while True:
                try:
                    shot = sct.grab(self.region())
                    img = Image.frombytes("RGB", shot.size, shot.rgb)
                    out = self.engine.process(img)
                    if out:
                        self.root.after(0, self.bar.config, {"text": out})
                except Exception as ex:  # не роняем цикл из-за одного кадра
                    print("frame error:", ex, file=sys.stderr)
                time.sleep(self.interval)


def main():
    p = argparse.ArgumentParser(description="Shmotix — перевод субтитров с экрана")
    p.add_argument("--src", default="en")
    p.add_argument("--dst", default="ru")
    p.add_argument("--interval", type=float, default=0.5, help="сек между кадрами")
    a = p.parse_args()
    print("Загружаю модели…")
    Lens(Engine(a.src, a.dst), a.interval).root.mainloop()


if __name__ == "__main__":
    main()
