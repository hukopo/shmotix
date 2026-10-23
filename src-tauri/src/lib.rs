pub mod engine;

use std::{path::PathBuf, sync::Mutex, thread, time::Duration};

use tauri::image::Image;
use tauri::menu::{CheckMenuItem, Menu, MenuItem};
use tauri::tray::TrayIconBuilder;
use tauri::{Emitter, Manager, State, WebviewWindow};
use xcap::Monitor;

/// Область захвата в CSS-пикселях относительно окна; присылает UI (ширина 0 — котик свёрнут).
struct Region(Mutex<[f64; 4]>);

#[tauri::command]
fn set_region(region: State<Region>, x: f64, y: f64, w: f64, h: f64) {
    *region.0.lock().unwrap() = [x, y, w, h];
}

/// Снимок области под рамкой. Само окно в снимок не попадает (contentProtected),
/// поэтому перевод можно рисовать прямо поверх субтитров.
fn grab(win: &WebviewWindow, [rx, ry, w, h]: [f64; 4]) -> anyhow::Result<image::RgbImage> {
    let scale = win.scale_factor()?;
    let pos = win.inner_position()?.to_logical::<f64>(scale);
    let (x, y) = (pos.x + rx, pos.y + ry);

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
        .manage(Region(Mutex::new([0.0; 4])))
        .invoke_handler(tauri::generate_handler![set_region])
        .setup(move |app| {
            #[cfg(target_os = "macos")]
            app.set_activation_policy(tauri::ActivationPolicy::Accessory); // только трей, без Dock

            let win = app.get_webview_window("lens").unwrap();
            // модели лежат внутри бандла (bundle.resources), env — чтобы подсунуть свои
            let models = match std::env::var("SHMOTIX_MODELS") {
                Ok(p) => PathBuf::from(p),
                Err(_) => app.path().resource_dir()?.join("models"),
            };
            let mut engine = engine::Engine::new(&models, &pair)
                .map_err(|e| format!("модели не найдены в {models:?} (запусти scripts/fetch-models.sh): {e}"))?;

            let toggle = MenuItem::with_id(app, "toggle", "Скрыть / показать", true, None::<&str>)?;
            let cat = CheckMenuItem::with_id(app, "cat", "Котик", true, true, None::<&str>)?;
            let overlay = CheckMenuItem::with_id(app, "overlay", "Перевод поверх субтитров", true, true, None::<&str>)?;
            let quit = MenuItem::with_id(app, "quit", "Выход", true, None::<&str>)?;
            let menu = Menu::with_items(app, &[&toggle, &cat, &overlay, &quit])?;
            TrayIconBuilder::new()
                .icon(Image::from_bytes(include_bytes!("../icons/tray.png"))?)
                .icon_as_template(true)
                .menu(&menu)
                .on_menu_event(move |app, e| {
                    let w = app.get_webview_window("lens").unwrap();
                    match e.id.as_ref() {
                        "toggle" if w.is_visible().unwrap_or(true) => { let _ = w.hide(); }
                        "toggle" => { let _ = w.show(); }
                        "quit" => app.exit(0),
                        _ => {
                            let _ = w.emit("settings", serde_json::json!({
                                "cat": cat.is_checked().unwrap_or(true),
                                "overlay": overlay.is_checked().unwrap_or(true),
                            }));
                        }
                    }
                })
                .build(app)?;

            thread::spawn(move || loop {
                thread::sleep(Duration::from_millis(500));
                let region = *win.state::<Region>().0.lock().unwrap();
                if region[2] < 10.0 || !win.is_visible().unwrap_or(false) {
                    continue; // скрыта в трей или свёрнута в котика — не жжём CPU
                }
                // пустая строка тоже уходит в UI: субтитр пропал — убираем перевод
                match grab(&win, region).and_then(|img| engine.process(&img)) {
                    Ok(t) => { let _ = win.emit("subtitle", t); }
                    Err(e) => eprintln!("frame error: {e}"), // один битый кадр не роняет цикл
                }
            });
            Ok(())
        })
        .run(tauri::generate_context!())
        .expect("tauri run failed");
}
