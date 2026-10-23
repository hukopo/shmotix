"""OCR + перевод. Без GUI — чтобы тестировать отдельно."""
import numpy as np
from rapidocr_onnxruntime import RapidOCR
import argostranslate.package
import argostranslate.translate


def ensure_lang_pair(src: str, dst: str):
    """Скачивает модель перевода src->dst при первом запуске (~100 МБ), дальше всё офлайн."""
    installed = argostranslate.translate.get_installed_languages()
    codes = {l.code: l for l in installed}
    if src in codes and dst in codes and codes[src].get_translation(codes[dst]):
        return codes[src].get_translation(codes[dst])
    argostranslate.package.update_package_index()
    pkg = next((p for p in argostranslate.package.get_available_packages()
                if p.from_code == src and p.to_code == dst), None)
    if pkg is None:
        raise SystemExit(f"Нет прямой модели {src}->{dst} в Argos")
    argostranslate.package.install_from_path(pkg.download())
    return ensure_lang_pair(src, dst)


class Engine:
    def __init__(self, src="en", dst="ru"):
        self.ocr = RapidOCR()
        self.translator = ensure_lang_pair(src, dst)
        self._last_src = ""
        self._last_dst = ""

    def read_text(self, img) -> str:
        result, _ = self.ocr(np.asarray(img.convert("RGB")))
        if not result:
            return ""
        # сортируем строки сверху вниз, склеиваем в одну фразу
        lines = sorted(result, key=lambda r: r[0][0][1])
        return " ".join(r[1] for r in lines).strip()

    def process(self, img) -> str:
        text = self.read_text(img)
        if not text:
            return ""
        if text != self._last_src:  # субтитр тот же — не гоняем переводчик повторно
            self._last_src = text
            self._last_dst = self.translator.translate(text)
        return self._last_dst
