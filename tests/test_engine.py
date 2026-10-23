from PIL import Image, ImageDraw, ImageFont

from shmotix.engine import Engine


def test_ocr_and_translate():
    img = Image.new("RGB", (600, 80), "black")
    font = ImageFont.load_default(size=36)
    ImageDraw.Draw(img).text((10, 20), "Hello my friend", fill="white", font=font)
    eng = Engine("en", "ru")
    assert "hello" in eng.read_text(img).lower()
    ru = eng.process(img)
    assert ru and ru != "Hello my friend"
    assert eng.process(img) == ru  # повтор не переводится заново, тот же результат
