// SPDX-License-Identifier: MIT
// Copyright (c) 2026 Otávio Cordeiro

//! Card rendering: Fredoka text in a purple-to-cyan gradient on a transparent canvas.

use swash::FontRef;
use swash::NormalizedCoord;
use swash::scale::{Render, ScaleContext, Source};
use swash::zeno::{Format, Vector};

const FONT_DATA: &[u8] = include_bytes!("../assets/Fredoka.ttf");

pub const WIDTH: u32 = 1800;
pub const HEIGHT: u32 = 1000;

const WEIGHT: f32 = 600.0;
const MARGIN_FRAC: f32 = 0.10;
const HEIGHT_FRAC: f32 = 0.60;
const MAX_FONT_SIZE: u32 = 300;
const MIN_FONT_SIZE: u32 = 20;

const START_COLOR: [u8; 3] = [0xAA, 0x5C, 0xC3];
const END_COLOR: [u8; 3] = [0x00, 0xA4, 0xDC];

#[derive(Debug)]
pub struct RenderError(pub String);

impl std::fmt::Display for RenderError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(&self.0)
    }
}

fn font() -> FontRef<'static> {
    FontRef::from_index(FONT_DATA, 0).expect("embedded Fredoka.ttf is a valid font")
}

fn coords(font: &FontRef) -> Vec<NormalizedCoord> {
    font.variations().normalized_coords(&[("wght", WEIGHT)]).collect()
}

fn vertical_metrics(font: &FontRef, coords: &[NormalizedCoord], size: f32) -> (i32, i32) {
    let metrics = font.metrics(coords).scale(size);
    (metrics.ascent.ceil() as i32, metrics.descent.abs().ceil() as i32)
}

fn advance_width(font: &FontRef, coords: &[NormalizedCoord], text: &str, size: f32) -> f32 {
    let charmap = font.charmap();
    let metrics = font.glyph_metrics(coords).scale(size);
    text.chars().map(|ch| metrics.advance_width(charmap.map(ch))).sum()
}

pub fn fit_font_size(text: &str) -> Result<u32, RenderError> {
    let font = font();
    let coords = coords(&font);

    let max_width = WIDTH as f32 - 2.0 * (WIDTH as f32 * MARGIN_FRAC).trunc();
    let max_height = (HEIGHT as f32 * HEIGHT_FRAC) as i32;

    let (mut lo, mut hi) = (MIN_FONT_SIZE, MAX_FONT_SIZE);
    let mut best = None;
    while lo <= hi {
        let size = (lo + hi) / 2;
        let (ascent, descent) = vertical_metrics(&font, &coords, size as f32);
        if advance_width(&font, &coords, text, size as f32) <= max_width && ascent + descent <= max_height {
            best = Some(size);
            lo = size + 1;
        } else {
            hi = size - 1;
        }
    }

    best.ok_or_else(|| RenderError(format!("{text:?} does not fit the card at any size >= {MIN_FONT_SIZE}")))
}

fn text_mask(text: &str, size: f32) -> Vec<u8> {
    let font = font();
    let coords = coords(&font);
    let charmap = font.charmap();
    let glyph_metrics = font.glyph_metrics(&coords).scale(size);

    let (ascent, descent) = vertical_metrics(&font, &coords, size);
    let baseline = (HEIGHT as i32 / 2) - (ascent + descent) / 2 + ascent;
    let mut pen = WIDTH as f32 / 2.0 - advance_width(&font, &coords, text, size) / 2.0;

    let mut context = ScaleContext::new();
    let mut scaler = context.builder(font).size(size).hint(false).normalized_coords(&coords).build();

    let mut mask = vec![0u8; (WIDTH * HEIGHT) as usize];
    for ch in text.chars() {
        let glyph_id = charmap.map(ch);
        let pen_x = pen.floor();
        let image = Render::new(&[Source::Outline])
            .format(Format::Alpha)
            .offset(Vector::new(pen - pen_x, 0.0))
            .render(&mut scaler, glyph_id);

        if let Some(image) = image {
            let x = pen_x as i32 + image.placement.left;
            let y = baseline - image.placement.top;
            blit(&mut mask, &image.data, image.placement.width, x, y);
        }

        pen += glyph_metrics.advance_width(glyph_id);
    }
    mask
}

fn blit(mask: &mut [u8], data: &[u8], glyph_width: u32, x0: i32, y0: i32) {
    if glyph_width == 0 {
        return;
    }
    for (index, &coverage) in data.iter().enumerate() {
        let x = x0 + (index as u32 % glyph_width) as i32;
        let y = y0 + (index as u32 / glyph_width) as i32;
        if x < 0 || y < 0 || x >= WIDTH as i32 || y >= HEIGHT as i32 {
            continue;
        }
        let pixel = &mut mask[(y as u32 * WIDTH + x as u32) as usize];
        *pixel = (*pixel).max(coverage);
    }
}

fn ink_bbox(mask: &[u8]) -> Option<(u32, u32, u32, u32)> {
    let (mut x0, mut y0, mut x1, mut y1) = (WIDTH, HEIGHT, 0u32, 0u32);
    for y in 0..HEIGHT {
        for x in 0..WIDTH {
            if mask[(y * WIDTH + x) as usize] != 0 {
                x0 = x0.min(x);
                y0 = y0.min(y);
                x1 = x1.max(x + 1);
                y1 = y1.max(y + 1);
            }
        }
    }
    (x1 > x0).then_some((x0, y0, x1, y1))
}

fn gradient_color(t: f32) -> [u8; 3] {
    let mix = |start: u8, end: u8| (start as f32 + (end as f32 - start as f32) * t).round() as u8;
    [mix(START_COLOR[0], END_COLOR[0]), mix(START_COLOR[1], END_COLOR[1]), mix(START_COLOR[2], END_COLOR[2])]
}

pub fn render(text: &str) -> Result<Vec<u8>, RenderError> {
    let size = fit_font_size(text)?;
    let mask = text_mask(text, size as f32);
    let (x0, _, x1, _) = ink_bbox(&mask).ok_or_else(|| RenderError(format!("{text:?} renders no visible glyphs")))?;

    let span = (x1 - x0 - 1).max(1) as f32;
    let mut rgba = vec![0u8; (WIDTH * HEIGHT * 4) as usize];
    for y in 0..HEIGHT {
        for x in x0..x1 {
            let alpha = mask[(y * WIDTH + x) as usize];
            let [r, g, b] = gradient_color((x - x0) as f32 / span);
            let index = ((y * WIDTH + x) * 4) as usize;
            rgba[index..index + 4].copy_from_slice(&[r, g, b, alpha]);
        }
    }
    Ok(rgba)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn shares_one_size_until_a_name_is_too_long() {
        assert_eq!(fit_font_size("Movies").unwrap(), MAX_FONT_SIZE);
        assert_eq!(fit_font_size("Kids").unwrap(), MAX_FONT_SIZE);
        assert_eq!(fit_font_size("Collections").unwrap(), 283);
        assert_eq!(fit_font_size("Documentaries").unwrap(), 210);
    }

    #[test]
    fn paints_a_transparent_canvas_with_a_purple_to_cyan_text_gradient() {
        let rgba = render("Movies").unwrap();
        assert_eq!(rgba.len(), (WIDTH * HEIGHT * 4) as usize);

        let pixel = |x: u32, y: u32| {
            let index = ((y * WIDTH + x) * 4) as usize;
            ([rgba[index], rgba[index + 1], rgba[index + 2]], rgba[index + 3])
        };
        assert_eq!(pixel(0, 0).1, 0);
        assert_eq!(pixel(WIDTH - 1, HEIGHT - 1).1, 0);

        let mask = text_mask("Movies", fit_font_size("Movies").unwrap() as f32);
        let (x0, y0, x1, _) = ink_bbox(&mask).unwrap();
        assert_eq!(pixel(x0, y0 + 1).0, START_COLOR);
        assert_eq!(pixel(x1 - 1, y0 + 1).0, END_COLOR);
    }

    #[test]
    fn rejects_text_with_no_glyphs() {
        assert!(render(" ").is_err());
    }
}
