//! Einbrennen von Greenshot-lite-Overlays in PNG-Kopien.
//!
//! Original-Screenshots in `.steps` bleiben unangetastet; nur Export-Pfade
//! (HTML/Markdown) bekommen eine gerasterte Kopie.

#![allow(
    clippy::cast_possible_truncation,
    clippy::cast_possible_wrap,
    clippy::cast_sign_loss
)]

use std::borrow::Cow;
use std::io::Cursor;

use font8x8::{UnicodeFonts, BASIC_FONTS};
use image::{Rgba, RgbaImage};
use steps_store::limits::MAX_IMAGE_PIXELS;
use steps_store::{marks, Crop, Overlay};

#[derive(Clone, Copy)]
struct Pt {
    x: i32,
    y: i32,
}

/// PNG für den Export. Ein Zuschnitt schneidet eine Kopie; Markierungen
/// werden in dieses Fenster umgerechnet. Die Eingabe-Bytes bleiben gleich.
pub fn present_png<'a>(png: &'a [u8], crop: Option<Crop>, overlays: &[Overlay]) -> Cow<'a, [u8]> {
    let Some(crop) = crop else {
        return if overlays.is_empty() {
            Cow::Borrowed(png)
        } else {
            Cow::Owned(burn_overlays(png, overlays))
        };
    };
    let cropped = crop_png(png, crop);
    if overlays.is_empty() {
        return Cow::Owned(cropped);
    }
    let mapped: Vec<Overlay> = overlays
        .iter()
        .map(|overlay| remap_overlay(overlay, crop))
        .collect();
    Cow::Owned(burn_overlays(&cropped, &mapped))
}

fn crop_png(png: &[u8], crop: Crop) -> Vec<u8> {
    let Ok(image) = image::load_from_memory(png) else {
        return png.to_vec();
    };
    let width = image.width();
    let height = image.height();
    if width < 2 || height < 2 {
        return png.to_vec();
    }
    let width_f = f64::from(width);
    let height_f = f64::from(height);
    let x = (crop.x * width_f).round().clamp(0.0, width_f - 1.0) as u32;
    let y = (crop.y * height_f).round().clamp(0.0, height_f - 1.0) as u32;
    let right = ((crop.x + crop.w) * width_f)
        .round()
        .clamp(f64::from(x + 1), width_f) as u32;
    let bottom = ((crop.y + crop.h) * height_f)
        .round()
        .clamp(f64::from(y + 1), height_f) as u32;
    let cropped = image.crop_imm(x, y, right - x, bottom - y);
    let mut out = Cursor::new(Vec::new());
    if cropped.write_to(&mut out, image::ImageFormat::Png).is_err() {
        return png.to_vec();
    }
    out.into_inner()
}

fn remap_overlay(overlay: &Overlay, crop: Crop) -> Overlay {
    let mut mapped = overlay.clone();
    match &mut mapped {
        Overlay::Rect { x, y, w, h, .. }
        | Overlay::Highlight { x, y, w, h, .. }
        | Overlay::Blur { x, y, w, h, .. } => {
            *x = (*x - crop.x) / crop.w;
            *y = (*y - crop.y) / crop.h;
            *w /= crop.w;
            *h /= crop.h;
        }
        Overlay::Arrow { x1, y1, x2, y2, .. } => {
            *x1 = (*x1 - crop.x) / crop.w;
            *y1 = (*y1 - crop.y) / crop.h;
            *x2 = (*x2 - crop.x) / crop.w;
            *y2 = (*y2 - crop.y) / crop.h;
        }
        Overlay::Pen { points, .. } => {
            for point in points {
                point[0] = (point[0] - crop.x) / crop.w;
                point[1] = (point[1] - crop.y) / crop.h;
            }
        }
        Overlay::Text { x, y, .. } => {
            *x = (*x - crop.x) / crop.w;
            *y = (*y - crop.y) / crop.h;
        }
        Overlay::Circle { cx, cy, r, .. } => {
            *cx = (*cx - crop.x) / crop.w;
            *cy = (*cy - crop.y) / crop.h;
            *r /= crop.w;
        }
    }
    mapped
}

/// Kompositiert Overlays auf eine PNG-Kopie. Ohne Overlays oder bei
/// Decode-Fehlern wird die Eingabe unverändert zurückgegeben.
///
/// Marker (`highlight`) werden zuerst gezeichnet, damit Tinte darüber liegt.
pub fn burn_overlays(png: &[u8], overlays: &[Overlay]) -> Vec<u8> {
    if overlays.is_empty() {
        return png.to_vec();
    }
    let Ok(dyn_img) = image::load_from_memory(png) else {
        return png.to_vec();
    };
    let mut img = dyn_img.to_rgba8();
    let width = img.width();
    let height = img.height();
    if width == 0 || height == 0 {
        return png.to_vec();
    }

    // Paint order matches the UI. Blur, then highlight, then ink. Vectors stay on top.
    for overlay in overlays_in_paint_order(overlays) {
        draw_overlay(&mut img, overlay, width, height);
    }

    let mut out = Cursor::new(Vec::new());
    if image::DynamicImage::ImageRgba8(img)
        .write_to(&mut out, image::ImageFormat::Png)
        .is_err()
    {
        return png.to_vec();
    }
    out.into_inner()
}

/// Stable paint order: Blur, then Highlight, then remaining ink overlays.
fn overlays_in_paint_order(overlays: &[Overlay]) -> impl Iterator<Item = &Overlay> {
    let blur = overlays
        .iter()
        .filter(|o| matches!(o, Overlay::Blur { .. }));
    let highlight = overlays
        .iter()
        .filter(|o| matches!(o, Overlay::Highlight { .. }));
    let ink = overlays
        .iter()
        .filter(|o| !matches!(o, Overlay::Blur { .. } | Overlay::Highlight { .. }));
    blur.chain(highlight).chain(ink)
}

fn to_px(norm: f64, extent: f64) -> i32 {
    (norm * extent).round() as i32
}

fn pt(nx: f64, ny: f64, width: f64, height: f64) -> Pt {
    Pt {
        x: to_px(nx, width),
        y: to_px(ny, height),
    }
}

fn draw_overlay(img: &mut RgbaImage, overlay: &Overlay, width: u32, height: u32) {
    let w = f64::from(width);
    let h = f64::from(height);
    match overlay {
        Overlay::Rect {
            color,
            stroke,
            x,
            y,
            w: rw,
            h: rh,
            ..
        } => {
            draw_rect_stroke(
                img,
                pt(*x, *y, w, h),
                pt(*x + rw, *y + rh, w, h),
                stroke.max(1.0),
                parse_color(color),
            );
        }
        Overlay::Arrow {
            color,
            stroke,
            x1,
            y1,
            x2,
            y2,
            ..
        } => {
            let color = parse_color(color);
            let stroke_px = stroke.max(1.0);
            let from = pt(*x1, *y1, w, h);
            let tip = pt(*x2, *y2, w, h);
            draw_line(img, from, tip, stroke_px, color);
            draw_arrowhead(img, from, tip, stroke_px, color);
        }
        Overlay::Pen {
            color,
            stroke,
            points,
            ..
        } => draw_pen(img, points, stroke.max(1.0), parse_color(color), w, h),
        Overlay::Highlight {
            color,
            opacity,
            x,
            y,
            w: rw,
            h: rh,
            ..
        } => {
            let mut fill = parse_color(color);
            fill.0[3] = ((*opacity).clamp(0.0, 1.0) * 255.0).round() as u8;
            fill_rect(img, pt(*x, *y, w, h), pt(*x + rw, *y + rh, w, h), fill);
        }
        Overlay::Text {
            color,
            size,
            x,
            y,
            text,
            ..
        } => draw_label(
            img,
            pt(*x, *y, w, h),
            text,
            size.max(8.0),
            parse_color(color),
        ),
        Overlay::Circle {
            color,
            stroke,
            cx,
            cy,
            r,
            ..
        } => {
            // `r` is width-relative so circles stay round across aspect ratios.
            draw_circle_stroke(
                img,
                pt(*cx, *cy, w, h),
                (*r * w).max(1.0),
                stroke.max(1.0),
                parse_color(color),
            );
        }
        Overlay::Blur {
            x, y, w: rw, h: rh, ..
        } => hard_blur_region(img, pt(*x, *y, w, h), pt(*x + rw, *y + rh, w, h)),
    }
}

/// Ein Stiftstrich prüft höchstens so viele Pixel, wie das größte erlaubte
/// Bild hat. Ein Strich, der mehr verlangt, endet vor dem Segment, das die
/// Grenze überschreiten würde.
fn draw_pen(
    img: &mut RgbaImage,
    points: &[[f64; 2]],
    stroke_px: f64,
    color: Rgba<u8>,
    w: f64,
    h: f64,
) {
    let disk_side = u64::from(2 * line_radius(stroke_px).unsigned_abs() + 1);
    let mut budget = MAX_IMAGE_PIXELS;
    for pair in points.windows(2) {
        let a = pt(pair[0][0], pair[0][1], w, h);
        let b = pt(pair[1][0], pair[1][1], w, h);
        let line_steps = u64::from(a.x.abs_diff(b.x).max(a.y.abs_diff(b.y))) + 1;
        let Some(rest) = budget.checked_sub(line_steps * disk_side * disk_side) else {
            return;
        };
        budget = rest;
        draw_line(img, a, b, stroke_px, color);
    }
}

/// Farbe `#RRGGBB`; ungültige Farben zeichnen in der Standardfarbe der
/// Markierungen.
fn parse_color(hex: &str) -> Rgba<u8> {
    hex_rgba(hex)
        .or_else(|| hex_rgba(&marks().color))
        .unwrap_or(Rgba([0; 4]))
}

fn hex_rgba(hex: &str) -> Option<Rgba<u8>> {
    let s = hex.trim().trim_start_matches('#');
    if s.len() != 6 {
        return None;
    }
    let rgb = u32::from_str_radix(s, 16).ok()?;
    Some(Rgba([
        ((rgb >> 16) & 0xFF) as u8,
        ((rgb >> 8) & 0xFF) as u8,
        (rgb & 0xFF) as u8,
        0xFF,
    ]))
}

/// Farbe und Deckkraft des Halos um exportierte Beschriftungen.
fn text_halo() -> Rgba<u8> {
    let halo = &marks().text_halo;
    let mut color = parse_color(&halo.color);
    color.0[3] = (halo.opacity.clamp(0.0, 1.0) * 255.0) as u8;
    color
}

/// Ecken des Rechtecks `a`–`b`, beschnitten auf das Bild; `None`, wenn es das
/// Bild nicht berührt. Flächen-Schleifen laufen nur hierüber, damit eine
/// Markierung höchstens so viele Schritte kostet, wie das Bild Pixel hat.
fn clip_to_image(img: &RgbaImage, a: Pt, b: Pt) -> Option<(Pt, Pt)> {
    let last_x = i32::try_from(img.width().saturating_sub(1)).unwrap_or(0);
    let last_y = i32::try_from(img.height().saturating_sub(1)).unwrap_or(0);
    let top_left = Pt {
        x: a.x.min(b.x).max(0),
        y: a.y.min(b.y).max(0),
    };
    let bottom_right = Pt {
        x: a.x.max(b.x).min(last_x),
        y: a.y.max(b.y).min(last_y),
    };
    (top_left.x <= bottom_right.x && top_left.y <= bottom_right.y)
        .then_some((top_left, bottom_right))
}

fn fill_rect(img: &mut RgbaImage, a: Pt, b: Pt, color: Rgba<u8>) {
    let Some((top_left, bottom_right)) = clip_to_image(img, a, b) else {
        return;
    };
    for y in top_left.y..=bottom_right.y {
        for x in top_left.x..=bottom_right.x {
            blend_pixel(img, x, y, color);
        }
    }
}

fn draw_label(img: &mut RgbaImage, origin: Pt, text: &str, size: f64, color: Rgba<u8>) {
    // font8x8 ist 8×8; auf ~`size` hochskalieren.
    let scale = (size / 8.0).max(1.0);
    let halo = text_halo();
    let mut cursor_x = origin.x;
    let cursor_y = origin.y;
    for ch in text.chars() {
        let glyph = BASIC_FONTS
            .get(ch)
            .unwrap_or_else(|| BASIC_FONTS.get('?').unwrap_or([0; 8]));
        // Halo aus `marks.json` in 8 Richtungen, dann Tinte.
        for &(dx, dy) in &[
            (-1, 0),
            (1, 0),
            (0, -1),
            (0, 1),
            (-1, -1),
            (1, -1),
            (-1, 1),
            (1, 1),
        ] {
            blit_glyph(
                img,
                cursor_x + (f64::from(dx) * scale).round() as i32,
                cursor_y + (f64::from(dy) * scale).round() as i32,
                glyph,
                scale,
                halo,
            );
        }
        blit_glyph(img, cursor_x, cursor_y, glyph, scale, color);
        cursor_x += (8.0 * scale).round() as i32;
    }
}

fn blit_glyph(
    img: &mut RgbaImage,
    origin_x: i32,
    origin_y: i32,
    glyph: [u8; 8],
    scale: f64,
    color: Rgba<u8>,
) {
    let cell = scale.max(1.0).ceil() as i32;
    for (row, bits) in glyph.iter().enumerate() {
        for col in 0..8_i32 {
            if bits & (1 << col) == 0 {
                continue;
            }
            let px = origin_x + (f64::from(col) * scale).round() as i32;
            let py = origin_y + (f64::from(row as i32) * scale).round() as i32;
            for oy in 0..cell {
                for ox in 0..cell {
                    blend_pixel(img, px + ox, py + oy, color);
                }
            }
        }
    }
}

fn draw_rect_stroke(img: &mut RgbaImage, a: Pt, b: Pt, stroke: f64, color: Rgba<u8>) {
    let left = a.x.min(b.x);
    let right = a.x.max(b.x);
    let top = a.y.min(b.y);
    let bottom = a.y.max(b.y);
    let corners = [
        Pt { x: left, y: top },
        Pt { x: right, y: top },
        Pt {
            x: right,
            y: bottom,
        },
        Pt { x: left, y: bottom },
    ];
    for i in 0..4 {
        draw_line(img, corners[i], corners[(i + 1) % 4], stroke, color);
    }
}

fn draw_circle_stroke(img: &mut RgbaImage, center: Pt, radius: f64, stroke: f64, color: Rgba<u8>) {
    let outer = radius + stroke / 2.0;
    let inner = (radius - stroke / 2.0).max(0.0);
    let outer_i = outer.ceil() as i32;
    let corner = |sign: i32| Pt {
        x: center.x + sign * outer_i,
        y: center.y + sign * outer_i,
    };
    let Some((top_left, bottom_right)) = clip_to_image(img, corner(-1), corner(1)) else {
        return;
    };
    let r_out2 = outer * outer;
    let r_in2 = inner * inner;
    for y in top_left.y..=bottom_right.y {
        for x in top_left.x..=bottom_right.x {
            let dx = f64::from(x - center.x);
            let dy = f64::from(y - center.y);
            let d2 = dx * dx + dy * dy;
            if d2 <= r_out2 && d2 >= r_in2 {
                put_pixel(img, x, y, color);
            }
        }
    }
}

/// Starke Pixelate-Maske: Text/PII unlesbar (kein soft wash).
fn hard_blur_region(img: &mut RgbaImage, a: Pt, b: Pt) {
    let Some((top_left, bottom_right)) = clip_to_image(img, a, b) else {
        return;
    };
    let (left, top) = (top_left.x, top_left.y);
    let (right, bottom) = (bottom_right.x, bottom_right.y);
    if right <= left || bottom <= top {
        return;
    }
    let region_w = (right - left + 1) as u32;
    let region_h = (bottom - top + 1) as u32;
    let block = marks().blur_block.size(region_w.min(region_h)) as i32;
    let mut y = top;
    while y <= bottom {
        let y2 = (y + block - 1).min(bottom);
        let mut x = left;
        while x <= right {
            let x2 = (x + block - 1).min(right);
            let avg = average_block(img, x, y, x2, y2);
            for py in y..=y2 {
                for px in x..=x2 {
                    put_pixel(img, px, py, avg);
                }
            }
            x += block;
        }
        y += block;
    }
}

fn average_block(img: &RgbaImage, x0: i32, y0: i32, x1: i32, y1: i32) -> Rgba<u8> {
    let mut sum = [0u64; 4];
    let mut n = 0u64;
    for y in y0..=y1 {
        for x in x0..=x1 {
            if let (Ok(ux), Ok(uy)) = (u32::try_from(x), u32::try_from(y)) {
                if ux < img.width() && uy < img.height() {
                    let p = img.get_pixel(ux, uy).0;
                    for i in 0..4 {
                        sum[i] += u64::from(p[i]);
                    }
                    n += 1;
                }
            }
        }
    }
    if n == 0 {
        return Rgba([0, 0, 0, 255]);
    }
    Rgba([
        (sum[0] / n) as u8,
        (sum[1] / n) as u8,
        (sum[2] / n) as u8,
        255,
    ])
}

fn draw_arrowhead(img: &mut RgbaImage, from: Pt, tip: Pt, stroke: f64, color: Rgba<u8>) {
    let dx = f64::from(tip.x - from.x);
    let dy = f64::from(tip.y - from.y);
    let len = (dx * dx + dy * dy).sqrt();
    if len < 1.0 {
        return;
    }
    let ux = dx / len;
    let uy = dy / len;
    let (head_len, head_width) = marks().arrow_head.size(stroke);
    let base_x = f64::from(tip.x) - ux * head_len;
    let base_y = f64::from(tip.y) - uy * head_len;
    let px = -uy;
    let py = ux;
    let left = Pt {
        x: (base_x + px * head_width).round() as i32,
        y: (base_y + py * head_width).round() as i32,
    };
    let right = Pt {
        x: (base_x - px * head_width).round() as i32,
        y: (base_y - py * head_width).round() as i32,
    };
    fill_triangle(img, tip, left, right, color);
}

fn fill_triangle(img: &mut RgbaImage, a: Pt, b: Pt, c: Pt, color: Rgba<u8>) {
    let min_x = a.x.min(b.x).min(c.x);
    let max_x = a.x.max(b.x).max(c.x);
    let min_y = a.y.min(b.y).min(c.y);
    let max_y = a.y.max(b.y).max(c.y);
    for y in min_y..=max_y {
        for x in min_x..=max_x {
            if point_in_triangle(Pt { x, y }, a, b, c) {
                put_pixel(img, x, y, color);
            }
        }
    }
}

fn point_in_triangle(p: Pt, a: Pt, b: Pt, c: Pt) -> bool {
    let d1 = sign(p, a, b);
    let d2 = sign(p, b, c);
    let d3 = sign(p, c, a);
    let has_neg = (d1 < 0) || (d2 < 0) || (d3 < 0);
    let has_pos = (d1 > 0) || (d2 > 0) || (d3 > 0);
    !(has_neg && has_pos)
}

fn sign(p: Pt, a: Pt, b: Pt) -> i32 {
    (p.x - b.x) * (a.y - b.y) - (a.x - b.x) * (p.y - b.y)
}

fn line_radius(stroke: f64) -> i32 {
    ((stroke / 2.0).ceil() as i32).max(1)
}

/// Dicke Linie als Kapsel aus Kreisen entlang Bresenham.
fn draw_line(img: &mut RgbaImage, a: Pt, b: Pt, stroke: f64, color: Rgba<u8>) {
    let radius = line_radius(stroke);
    let mut x = a.x;
    let mut y = a.y;
    let dx = (b.x - a.x).abs();
    let dy = -(b.y - a.y).abs();
    let sx = if a.x < b.x { 1 } else { -1 };
    let sy = if a.y < b.y { 1 } else { -1 };
    let mut err = dx + dy;
    loop {
        fill_disk(img, x, y, radius, color);
        if x == b.x && y == b.y {
            break;
        }
        let e2 = 2 * err;
        if e2 >= dy {
            err += dy;
            x += sx;
        }
        if e2 <= dx {
            err += dx;
            y += sy;
        }
    }
}

fn fill_disk(img: &mut RgbaImage, cx: i32, cy: i32, radius: i32, color: Rgba<u8>) {
    let r2 = radius * radius;
    for dy in -radius..=radius {
        for dx in -radius..=radius {
            if dx * dx + dy * dy <= r2 {
                put_pixel(img, cx + dx, cy + dy, color);
            }
        }
    }
}

fn put_pixel(img: &mut RgbaImage, x: i32, y: i32, color: Rgba<u8>) {
    if let (Ok(ux), Ok(uy)) = (u32::try_from(x), u32::try_from(y)) {
        if ux < img.width() && uy < img.height() {
            img.put_pixel(ux, uy, color);
        }
    }
}

fn blend_pixel(img: &mut RgbaImage, x: i32, y: i32, src: Rgba<u8>) {
    if let (Ok(ux), Ok(uy)) = (u32::try_from(x), u32::try_from(y)) {
        if ux >= img.width() || uy >= img.height() {
            return;
        }
        let dst = img.get_pixel(ux, uy).0;
        let a = f32::from(src[3]) / 255.0;
        let inv = 1.0 - a;
        let blended = Rgba([
            (f32::from(src[0]) * a + f32::from(dst[0]) * inv).round() as u8,
            (f32::from(src[1]) * a + f32::from(dst[1]) * inv).round() as u8,
            (f32::from(src[2]) * a + f32::from(dst[2]) * inv).round() as u8,
            255,
        ]);
        img.put_pixel(ux, uy, blended);
    }
}

#[cfg(test)]
mod tests {
    use std::sync::mpsc;
    use std::thread;
    use std::time::Duration;

    use super::*;
    use image::{GenericImageView, ImageBuffer};
    use steps_store::limits::{MAX_IMAGE_SIDE, MAX_OVERLAY_STROKE, MAX_PEN_POINTS};

    fn solid_png(width: u32, height: u32, color: [u8; 4]) -> Vec<u8> {
        let img: RgbaImage = ImageBuffer::from_pixel(width, height, Rgba(color));
        let mut out = Cursor::new(Vec::new());
        image::DynamicImage::ImageRgba8(img)
            .write_to(&mut out, image::ImageFormat::Png)
            .expect("encode");
        out.into_inner()
    }

    #[test]
    fn halo_und_weichzeichner_kommen_aus_marks_json() {
        assert_eq!(text_halo(), Rgba([255, 255, 255, 178]));
        let blur = marks().blur_block;
        assert_eq!(blur.size(12), blur.min);
        assert_eq!(blur.size(10_000), blur.max);
    }

    #[test]
    fn ohne_overlays_bleibt_png_identisch() {
        let png = solid_png(8, 8, [255, 255, 255, 255]);
        let burned = burn_overlays(&png, &[]);
        assert_eq!(burned, png);
    }

    #[test]
    fn zuschnitt_nimmt_die_rechte_haelfte_und_laesst_die_quelle() {
        let mut img: RgbaImage = ImageBuffer::from_pixel(10, 10, Rgba([255, 0, 0, 255]));
        for (x, _y, pixel) in img.enumerate_pixels_mut() {
            if x >= 5 {
                *pixel = Rgba([0, 0, 255, 255]);
            }
        }
        let mut out = Cursor::new(Vec::new());
        image::DynamicImage::ImageRgba8(img)
            .write_to(&mut out, image::ImageFormat::Png)
            .expect("encode");
        let png = out.into_inner();
        let crop = Crop {
            x: 0.5,
            y: 0.0,
            w: 0.5,
            h: 1.0,
        };
        let presented = present_png(&png, Some(crop), &[]);
        assert_eq!(&png[..8], b"\x89PNG\r\n\x1a\n");
        let decoded = image::load_from_memory(presented.as_ref()).expect("decode");
        assert_eq!((decoded.width(), decoded.height()), (5, 10));
        assert_eq!(decoded.get_pixel(0, 0), Rgba([0, 0, 255, 255]));
        assert_eq!(decoded.get_pixel(4, 9), Rgba([0, 0, 255, 255]));
    }

    #[test]
    fn burn_in_aendert_pixel() {
        let png = solid_png(64, 64, [255, 255, 255, 255]);
        let marks = marks();
        let overlays = vec![
            Overlay::Highlight {
                id: "h1".into(),
                color: marks.highlight_color.clone(),
                opacity: marks.highlight_opacity,
                x: 0.1,
                y: 0.1,
                w: 0.5,
                h: 0.2,
            },
            Overlay::Rect {
                id: "o1".into(),
                color: marks.color.clone(),
                stroke: marks.stroke,
                x: 0.1,
                y: 0.1,
                w: 0.4,
                h: 0.3,
            },
            Overlay::Arrow {
                id: "o2".into(),
                color: marks.color.clone(),
                stroke: marks.stroke,
                x1: 0.2,
                y1: 0.8,
                x2: 0.8,
                y2: 0.2,
            },
            Overlay::Pen {
                id: "o3".into(),
                color: marks.color.clone(),
                stroke: marks.pen_stroke(marks.stroke),
                points: vec![[0.05, 0.5], [0.3, 0.55], [0.5, 0.45]],
            },
            Overlay::Text {
                id: "t1".into(),
                color: marks.color.clone(),
                size: marks.text_size,
                x: 0.15,
                y: 0.6,
                text: "Hier".into(),
            },
            Overlay::Circle {
                id: "c1".into(),
                color: marks.color.clone(),
                stroke: marks.stroke,
                cx: 0.7,
                cy: 0.7,
                r: 0.12,
            },
            Overlay::Blur {
                id: "b1".into(),
                x: 0.55,
                y: 0.05,
                w: 0.35,
                h: 0.2,
            },
        ];
        let burned = burn_overlays(&png, &overlays);
        assert_ne!(burned, png);
        let img = image::load_from_memory(&burned).unwrap().to_rgba8();
        let accent = parse_color(&marks.color);
        let has_accent = img.pixels().any(|p| *p == accent);
        assert!(has_accent, "eingebranntes PNG sollte Akzentfarbe enthalten");
    }

    #[test]
    fn riesiger_kreis_auf_breitem_bild_zeichnet_nur_im_bild() {
        let white = Rgba([255, 255, 255, 255]);
        let png = solid_png(MAX_IMAGE_SIDE, 2, white.0);
        let marks = marks();
        let circle = Overlay::Circle {
            id: "c1".into(),
            color: marks.color.clone(),
            stroke: marks.stroke,
            cx: -1.49,
            cy: 0.5,
            r: 1.99,
        };
        let (done, finished) = mpsc::channel();
        thread::spawn(move || done.send(burn_overlays(&png, &[circle])));
        let burned = finished
            .recv_timeout(Duration::from_secs(2))
            .expect("Kreis kostet höchstens so viele Schritte, wie das Bild Pixel hat");
        let img = image::load_from_memory(&burned).unwrap().to_rgba8();
        assert_eq!(*img.get_pixel(8192, 0), parse_color(&marks.color));
        assert_eq!(*img.get_pixel(0, 0), white);
        assert_eq!(*img.get_pixel(MAX_IMAGE_SIDE - 1, 1), white);
    }

    #[test]
    fn zickzack_stift_mit_hoechstwerten_endet_rechtzeitig() {
        let white = Rgba([255, 255, 255, 255]);
        let png = solid_png(MAX_IMAGE_SIDE, 2, white.0);
        let marks = marks();
        let pen = Overlay::Pen {
            id: "p1".into(),
            color: marks.color.clone(),
            stroke: MAX_OVERLAY_STROKE,
            points: [[0.0, 0.5], [1.0, 0.5]]
                .into_iter()
                .cycle()
                .take(MAX_PEN_POINTS)
                .collect(),
        };
        let (done, finished) = mpsc::channel();
        thread::spawn(move || done.send(burn_overlays(&png, &[pen])));
        let burned = finished
            .recv_timeout(Duration::from_secs(5))
            .expect("Stiftstrich kostet höchstens 50 Millionen Schritte");
        let img = image::load_from_memory(&burned).unwrap().to_rgba8();
        assert_eq!(*img.get_pixel(0, 1), parse_color(&marks.color));
        assert_eq!(
            *img.get_pixel(MAX_IMAGE_SIDE - 1, 1),
            parse_color(&marks.color)
        );
    }

    #[test]
    fn stift_aus_der_app_zeichnet_bis_zum_letzten_punkt() {
        let white = Rgba([255, 255, 255, 255]);
        let png = solid_png(MAX_IMAGE_SIDE, 64, white.0);
        let marks = marks();
        let mut points: Vec<[f64; 2]> = (0..2_000)
            .map(|i| {
                let along = f64::from(i % 500) * 0.002;
                let x = if (i / 500) % 2 == 0 {
                    along
                } else {
                    1.0 - along
                };
                [x, 0.25]
            })
            .collect();
        points.push([0.0, 0.9]);
        let pen = Overlay::Pen {
            id: "p1".into(),
            color: marks.color.clone(),
            stroke: marks.pen_stroke(8.0),
            points,
        };
        let burned = burn_overlays(&png, &[pen]);
        let img = image::load_from_memory(&burned).unwrap().to_rgba8();
        assert_eq!(
            *img.get_pixel(MAX_IMAGE_SIDE / 2, 16),
            parse_color(&marks.color)
        );
        assert_eq!(*img.get_pixel(0, 58), parse_color(&marks.color));
        assert_eq!(*img.get_pixel(MAX_IMAGE_SIDE - 1, 58), white);
    }

    #[test]
    fn ungueltige_farbe_zeichnet_in_der_standardfarbe() {
        let standard = hex_rgba(&marks().color).expect("marks.json color");
        assert_eq!(parse_color("rot"), standard);
        assert_eq!(parse_color("#12345"), standard);
        assert_eq!(parse_color(" #2563eb "), Rgba([0x25, 0x63, 0xEB, 0xFF]));
    }

    #[test]
    fn blur_maske_macht_region_unlesbar() {
        // Schachbrett mit feinem Kontrast. Nach dem Weichzeichnen sollten
        // Nachbarpixel innerhalb eines Blocks nah beieinander liegen.
        let mut img: RgbaImage = ImageBuffer::from_pixel(64, 64, Rgba([255, 255, 255, 255]));
        for y in 0..64 {
            for x in 0..64 {
                let on = ((x / 2) + (y / 2)) % 2 == 0;
                img.put_pixel(
                    x,
                    y,
                    if on {
                        Rgba([0, 0, 0, 255])
                    } else {
                        Rgba([255, 255, 255, 255])
                    },
                );
            }
        }
        let mut out = Cursor::new(Vec::new());
        image::DynamicImage::ImageRgba8(img)
            .write_to(&mut out, image::ImageFormat::Png)
            .expect("encode");
        let png = out.into_inner();
        let burned = burn_overlays(
            &png,
            &[Overlay::Blur {
                id: "b1".into(),
                x: 0.1,
                y: 0.1,
                w: 0.5,
                h: 0.5,
            }],
        );
        let result = image::load_from_memory(&burned).unwrap().to_rgba8();
        let a = result.get_pixel(16, 16).0;
        let b = result.get_pixel(17, 16).0;
        let delta = i32::from(a[0]).abs_diff(i32::from(b[0]));
        assert!(
            delta < 40,
            "Blur-Maske sollte feinen Kontrast glätten, delta={delta}"
        );
    }
}
