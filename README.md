# Shmotix

Экранная «лупа»-переводчик: ставишь прозрачную рамку на субтитры (игра, видео, что угодно) —
внизу появляется перевод. Всё локально, без интернета и облаков.

## Запуск (macOS / Windows)

Нужны: Rust, Node.js, CMake (`brew install cmake` / `winget install Kitware.CMake`).

```bash
./scripts/fetch-models.sh   # ~110 МБ моделей в ./models, один раз
npm i
npm run dev                 # окно-лупа
cd src-tauri && cargo test --release   # проверка OCR+перевода на картинке
```

Управление: тащить за полосу перевода, ресайз — уголок ◢, выход — Esc или ✕.
macOS попросит разрешение «Запись экрана» для терминала/приложения.

Переменные: `SHMOTIX_MODELS` (папка моделей), `SHMOTIX_PAIR` (по умолчанию `en_ru`).

Подробности — [SPEC.md](SPEC.md).
