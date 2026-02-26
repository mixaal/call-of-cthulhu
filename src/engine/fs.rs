use image::{Rgb, RgbImage, imageops::FilterType};

use std::{
    fs::File,
    io::{BufReader, Read},
};

use crate::{engine::config, screens::play};

pub(crate) fn read_bitmap(file_path: &str) -> std::io::Result<(u16, u16, Vec<Vec<(u8, u8, u8)>>)> {
    let mut file = File::open(file_path)?;
    let mut width_buf = [0; 2];
    let mut height_buf = [0; 2];
    file.read_exact(&mut width_buf)?;
    file.read_exact(&mut height_buf)?;

    let width = u16::from_le_bytes(width_buf);
    let height = u16::from_le_bytes(height_buf);

    let mut pixels = Vec::new();
    let mut pixel_buf = [0; 3];
    while file.read_exact(&mut pixel_buf).is_ok() {
        pixels.push((pixel_buf[0], pixel_buf[1], pixel_buf[2]));
    }

    let mut screen = vec![vec![(0, 0, 0); width as usize]; height as usize];
    for y in 0..height {
        for x in 0..width {
            let pixel = pixels[(y * width + x) as usize];
            screen[y as usize][x as usize] = pixel;
        }
    }

    Ok((width, height, screen))
}

fn read_png(file_path: &str) -> Result<(u16, u16, Vec<(u8, u8, u8)>), Box<dyn std::error::Error>> {
    let file = File::open(file_path)?;
    let decoder = png::Decoder::new(BufReader::new(file));
    let mut reader = decoder.read_info()?;
    let buffer_sz = reader
        .output_buffer_size()
        .expect("can't determine buffer size for png file: {file_path}");
    let mut buf = vec![0; buffer_sz];
    let info = reader.next_frame(&mut buf)?;
    if info.color_type != png::ColorType::Rgb {
        // we do not support RGBA images
        return Err(format!("Unsupported color type in PNG file: {:?}", info.color_type).into());
    }

    let width = info.width as u16;
    let height = info.height as u16;

    let mut pixels = Vec::new();
    let bytes = &buf[..info.buffer_size()];
    for chunk in bytes.chunks(3) {
        if chunk.len() == 3 {
            pixels.push((chunk[0], chunk[1], chunk[2]));
        }
    }

    Ok((width, height, pixels))
}

fn quality_scale_image(
    image: &[(u8, u8, u8)],
    width: u16,
    height: u16,
    new_width: u16,
    new_height: u16,
    keep_aspect_ratio: bool,
) -> Vec<Vec<(u8, u8, u8)>> {
    // Convert the input image to an RgbImage
    let mut input_image = RgbImage::new(width as u32, height as u32);
    let image_len = image.len();
    for (i, pixel) in image.iter().enumerate() {
        let x = (i as u32) % width as u32;
        let y = (i as u32) / width as u32;
        //println!("i={i}, x={x}, y={y}, w={width}, h={height}");
        if x < width as u32 && y < height as u32 {
            // this should not happen but in case of RGBA image we might get in here
            // in that case the image is not displayed correctly
            input_image.put_pixel(x, y, Rgb([pixel.0, pixel.1, pixel.2]));
        }
    }
    // input_image
    //     .save("input.png")
    //     .expect("Failed to save the input image");

    // Calculate aspect ratio and scaled dimensions
    let aspect_ratio = width as f32 / height as f32;
    let new_aspect_ratio = new_width as f32 / new_height as f32;

    let (mut scaled_width, mut scaled_height) = if new_aspect_ratio > aspect_ratio {
        // Fit to height
        let scaled_width = (new_height as f32 * aspect_ratio).round() as u32;
        ((2 * scaled_width).min(new_width as u32), new_height as u32)
    } else {
        // Fit to width
        let scaled_height = (new_width as f32 / aspect_ratio).round() as u32;
        (new_width as u32, scaled_height)
    };

    if !keep_aspect_ratio {
        // If we don't care about aspect ratio, just use the new dimensions directly
        scaled_width = new_width as u32;
        scaled_height = new_height as u32;
    }

    // Resize the image using a cubic filter
    let resized_image = image::imageops::resize(
        &input_image,
        scaled_width,
        scaled_height,
        FilterType::CatmullRom,
    );

    // resized_image
    //     .save("scaled.png")
    //     .expect("Failed to save the scaled image");

    // Convert the output image back to the Vec<Vec<(u8, u8, u8)>> format
    let mut scaled = vec![vec![(0, 0, 0); new_width as usize]; new_height as usize];
    let x_offset = (new_width as usize - scaled_width as usize) / 2;
    let y_offset = (new_height as usize - scaled_height as usize) / 2;
    for y in 0..scaled_height {
        for x in 0..scaled_width {
            let pixel = resized_image.get_pixel(x as u32, y as u32);
            scaled[y_offset + y as usize][x_offset + x as usize] = (pixel[0], pixel[1], pixel[2]);
        }
    }

    scaled
}

fn scale_image(
    image: &[(u8, u8, u8)],
    width: u16,
    height: u16,
    new_width: u16,
    new_height: u16,
    keep_aspect_ratio: bool,
) -> Vec<Vec<(u8, u8, u8)>> {
    let aspect_ratio = width as f32 / height as f32;
    let new_aspect_ratio = new_width as f32 / new_height as f32;

    let (mut scaled_width, mut scaled_height) = if new_aspect_ratio > aspect_ratio {
        // Fit to height
        let scaled_width = (new_height as f32 * aspect_ratio).round() as u16;
        ((2 * scaled_width).min(new_width), new_height)
    } else {
        // Fit to width
        let scaled_height = (new_width as f32 / aspect_ratio).round() as u16;
        (new_width, scaled_height)
    };

    if !keep_aspect_ratio {
        // If we don't care about aspect ratio, just use the new dimensions directly
        scaled_width = new_width;
        scaled_height = new_height;
    }

    let x_offset = (new_width - scaled_width) / 2;
    let y_offset = (new_height - scaled_height) / 2;

    let mut scaled = vec![vec![(0, 0, 0); new_width as usize]; new_height as usize];

    for y in 0..scaled_height {
        for x in 0..scaled_width {
            let kx = x as f32 * width as f32 / scaled_width as f32;
            let ky = y as f32 * height as f32 / scaled_height as f32;
            let src_x = kx as usize;
            let src_y = ky as usize;
            let pixel = image[src_y * width as usize + src_x];
            scaled[(y + y_offset) as usize][(x + x_offset) as usize] = pixel;
        }
    }

    scaled
}

pub(crate) fn get_image_names_for_screen(
    screen_no: usize,
    config: &config::Config,
) -> Result<Vec<String>, Box<dyn std::error::Error>> {
    let mut names = Vec::new();
    let main_image = format!("{}/images/{}.png", config.data_path, screen_no);
    if std::path::Path::new(&main_image).exists() {
        names.push(main_image);
    }
    let pattern = format!("{}/images/{}_*.png", config.data_path, screen_no);
    let base_path = glob::glob(&pattern)?.collect::<Vec<_>>();
    for entry in base_path {
        if let Ok(path) = entry {
            names.push(path.to_string_lossy().into_owned());
        }
    }

    Ok(names)
}

pub(crate) fn read_image(
    file_path: &str,
    term_width: u16,
    term_height: u16,
    quality_scale: bool,
    keep_aspect_ratio: bool,
) -> std::io::Result<(u16, u16, Vec<Vec<(u8, u8, u8)>>)> {
    let (width, height, pixels) = read_png(file_path).map_err(|e| {
        std::io::Error::new(
            std::io::ErrorKind::Other,
            format!("Error reading PNG file: {}", e),
        )
    })?;
    let scaled = if quality_scale {
        quality_scale_image(
            &pixels,
            width,
            height,
            term_width,
            term_height,
            keep_aspect_ratio,
        )
    } else {
        scale_image(
            &pixels,
            width,
            height,
            term_width,
            term_height,
            keep_aspect_ratio,
        )
    };
    Ok((width, height, scaled))
}

fn read_screen_text(screen_no: usize, config: &config::Config) -> std::io::Result<String> {
    let file_path = format!("{}/text/{}.txt", config.data_path, screen_no);
    let mut file = File::open(file_path)?;
    let mut contents = String::new();
    file.read_to_string(&mut contents)?;
    Ok(contents)
}

pub(crate) fn read_text(screen_no: usize, config: &config::Config) -> std::io::Result<String> {
    let screen_text = read_screen_text(screen_no, config)?;

    Ok(screen_text)
}

pub(crate) fn read_actions(
    screen_no: usize,
    config: &config::Config,
) -> std::io::Result<play::GameActions> {
    let file_path = format!("{}/actions/{}.json", config.data_path, screen_no);
    let mut file = File::open(file_path)?;
    let mut contents = String::new();
    file.read_to_string(&mut contents)?;
    let actions = serde_json::from_str(&contents)?;
    Ok(actions)
}

pub(crate) fn load_intro_screen_image(
    term_width: u16,
    term_height: u16,
    config: &config::Config,
) -> Result<Vec<Vec<(u8, u8, u8)>>, Box<dyn std::error::Error>> {
    let intro_image = format!("{}/images/intro.png", config.data_path);
    let intro_screen = read_image(
        &intro_image,
        term_width,
        term_height,
        config.scale_quality,
        false,
    )?;
    Ok(intro_screen.2)
}

pub(crate) fn load_achievements_screen_image(
    term_width: u16,
    term_height: u16,
    config: &config::Config,
) -> Result<Vec<Vec<(u8, u8, u8)>>, Box<dyn std::error::Error>> {
    let intro_image = format!("{}/images/achievements.png", config.data_path);
    let intro_screen = read_image(
        &intro_image,
        term_width,
        term_height,
        config.scale_quality,
        false,
    )?;
    Ok(intro_screen.2)
}
