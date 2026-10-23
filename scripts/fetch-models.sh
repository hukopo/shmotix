#!/usr/bin/env bash
# Качает модели в ./models (один раз, ~110 МБ). Дальше всё работает офлайн.
set -euo pipefail
cd "$(dirname "$0")/.."
mkdir -p models && cd models
for f in text-detection.rten text-recognition.rten; do
  [ -f "$f" ] || curl -fL -o "$f" "https://ocrs-models.s3-accelerate.amazonaws.com/$f"
done
if [ ! -d en_ru ]; then
  curl -fL -o en_ru.zip https://argos-net.com/v1/translate-en_ru-1_9.argosmodel
  unzip -q en_ru.zip && mv translate-en_ru-1_9 en_ru && rm en_ru.zip
fi
echo "OK: $(pwd)"
