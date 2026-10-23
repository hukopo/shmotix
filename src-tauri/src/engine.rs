//! OCR (ocrs) + перевод (CTranslate2, модель Argos/Opus-MT). Без GUI — тестируется отдельно.
use std::path::Path;

use anyhow::Result;
use ct2rs::tokenizers::sentencepiece::Tokenizer;
use ct2rs::{Config, TranslationOptions, Translator};
use image::RgbImage;
use ocrs::{ImageSource, OcrEngine, OcrEngineParams};
use rten::Model;

pub struct Engine {
    ocr: OcrEngine,
    tr: Translator<Tokenizer>,
    last_src: String,
    last_dst: String,
}

impl Engine {
    /// `models` — папка из scripts/fetch-models.sh
    pub fn new(models: &Path, pair: &str) -> Result<Self> {
        let ocr = OcrEngine::new(OcrEngineParams {
            detection_model: Some(Model::load_file(models.join("text-detection.rten"))?),
            recognition_model: Some(Model::load_file(models.join("text-recognition.rten"))?),
            ..Default::default()
        })?;
        let dir = models.join(pair);
        let spm = dir.join("sentencepiece.model");
        let tr = Translator::with_tokenizer(dir.join("model"), Tokenizer::from_file(&spm, &spm)?, &Config::default())?;
        Ok(Self { ocr, tr, last_src: String::new(), last_dst: String::new() })
    }

    pub fn read_text(&self, img: &RgbImage) -> Result<String> {
        let input = self.ocr.prepare_input(ImageSource::from_bytes(img.as_raw(), img.dimensions())?)?;
        // строки субтитра склеиваем в одну фразу
        Ok(self.ocr.get_text(&input)?.split_whitespace().collect::<Vec<_>>().join(" "))
    }

    /// Пустая строка — текста в кадре нет.
    pub fn process(&mut self, img: &RgbImage) -> Result<String> {
        let text = self.read_text(img)?;
        // шум с фона (камни, иконки) OCR читает как «x O.» — такое не переводим
        if text.chars().filter(|c| c.is_alphabetic()).count() < 3 {
            return Ok(String::new());
        }
        if text != self.last_src {
            // субтитр тот же — переводчик не дёргаем
            let out = self.tr.translate_batch(&[text.clone()], &TranslationOptions::default(), None)?;
            self.last_dst = out.into_iter().next().map(|r| r.0).unwrap_or_default();
            eprintln!("{text} → {}", self.last_dst);
            self.last_src = text;
        }
        Ok(self.last_dst.clone())
    }
}
