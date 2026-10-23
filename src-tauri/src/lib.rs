pub mod engine;

use std::{path::PathBuf, thread, time::Duration};

use tauri::{Emitter, Manager, WebviewWindow};
use xcap::Monitor;

// должны совпадать с ui/index.html (CSS-пиксели)
const BORDER: f64 = 3.0;
const BAR_H: f64 = 60.0;

/// Снимок «дырки» лупы: окно минус рамка и полоса перевода.
fn grab(win: &WebviewWindow) -> anyhow::Result<image::RgbImage> {
    let scale = win.scale_factor()?;
    let pos = win.inner_position()?.to_logical::<f64>(scale);
    let size = win.inner_size()?.to_logical::<f64>(scale);
    let (x, y) = (pos.x + BORDER, pos.y + BORDER);
    let (w, h) = (size.width - 2.0 * BORDER, size.height - BAR_H - BORDER);

    let mon = Monitor::from_point(x as i32, y as i32)?;
    let shot = mon.capture_image()?;
    // monitor в логических точках, снимок — в физических пикселях
    let k = shot.width() as f64 / mon.width()? as f64;
    let cx = ((x - mon.x()? as f64) * k).max(0.0) as u32;
    let cy = ((y - mon.y()? as f64) * k).max(0.0) as u32;
    let cw = ((w * k) as u32).min(shot.width().saturating_sub(cx));
    let ch = ((h * k) as u32).min(shot.height().saturating_sub(cy));
    // ponytail: снимаем весь монитор и режем; если упрёмся в FPS — capture_region
    let crop = image::imageops::crop_imm(&shot, cx, cy, cw, ch).to_image();
    Ok(image::DynamicImage::ImageRgba8(crop).into_rgb8())
}

pub fn run() {
    let pair = std::env::var("SHMOTIX_PAIR").unwrap_or_else(|_| "en_ru".into());

    tauri::Builder::default()
        .setup(move |app| {
            let win = app.get_webview_window("lens").unwrap();
            // модели лежат внутри бандла (bundle.resources), env — чтобы подсунуть свои
            let models = match std::env::var("SHMOTIX_MODELS") {
                Ok(p) => PathBuf::from(p),
                Err(_) => app.path().resource_dir()?.join("models"),
            };
            let mut engine = engine::Engine::new(&models, &pair)
                .map_err(|e| format!("модели не найдены в {models:?} (запусти scripts/fetch-models.sh): {e}"))?;
            thread::spawn(move || loop {
                match grab(&win).and_then(|img| engine.process(&img)) {
                    Ok(t) if !t.is_empty() => { let _ = win.emit("subtitle", t); }
                    Ok(_) => {}
                    Err(e) => eprintln!("frame error: {e}"), // один битый кадр не роняет цикл
                }
                thread::sleep(Duration::from_millis(500));
            });
            Ok(())
        })
        .run(tauri::generate_context!())
        .expect("tauri run failed");
}
