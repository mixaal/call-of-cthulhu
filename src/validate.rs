use crate::{config, fs};

static SCREENS_MISSING: [usize; 11] = [5, 13, 16, 27, 38, 49, 61, 71, 82, 93, 104];

pub fn validate_screens(config: &config::Config) -> Result<(), Box<dyn std::error::Error>> {
    let mut errors = Vec::new();
    for screen_no in 0..=111 {
        if SCREENS_MISSING.contains(&screen_no) {
            continue;
        }
        let text_path = format!("{}/text/{}.txt", config.data_path, screen_no);
        let actions_path = format!("{}/actions/{}.json", config.data_path, screen_no);
        let images = fs::get_image_names_for_screen(screen_no, config)?;
        if images.is_empty() {
            errors.push(format!("No images found for screen {}", screen_no));
        }
        if !std::path::Path::new(&text_path).exists() {
            errors.push(format!(
                "Missing text file for screen {}: {}",
                screen_no, text_path
            ));
        }
        if !std::path::Path::new(&actions_path).exists() {
            errors.push(format!(
                "Missing actions file for screen {}: {}",
                screen_no, actions_path
            ));
        }
        for image_path in images {
            if !std::path::Path::new(&image_path).exists() {
                println!(
                    "Missing image file for screen {}: {}",
                    screen_no, image_path
                );
                errors.push(format!(
                    "Missing image file for screen {}: {}",
                    screen_no, image_path
                ));
            }
        }
    }
    if !errors.is_empty() {
        for error in errors {
            eprintln!("{}", error);
        }
        return Err("Validation failed".into());
    }
    Ok(())
}
