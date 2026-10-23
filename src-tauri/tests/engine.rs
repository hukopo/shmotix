use shmotix_lib::engine::Engine;

#[test]
fn ocr_and_translate() {
    let img = image::open("tests/fixtures/hello.png").unwrap().into_rgb8();
    let mut eng = Engine::new("../models".as_ref(), "en_ru").unwrap();
    assert!(eng.read_text(&img).unwrap().to_lowercase().contains("hello"));
    let ru = eng.process(&img).unwrap();
    println!("{ru}");
    assert!(!ru.is_empty() && ru.chars().any(|c| ('а'..='я').contains(&c.to_lowercase().next().unwrap())));
    assert_eq!(eng.process(&img).unwrap(), ru); // кэш повторного субтитра
}
