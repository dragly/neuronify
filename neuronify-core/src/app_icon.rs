use egui::ColorImage;

const COMPRESSED: &[u8] = include_bytes!(concat!(env!("OUT_DIR"), "/app-icon.deflate"));

pub struct Pixels {
    pub rgba: Vec<u8>,
    pub size: usize,
}

pub fn decode() -> Option<Pixels> {
    let rgba = miniz_oxide::inflate::decompress_to_vec(COMPRESSED).ok()?;
    let size = (rgba.len() / 4).isqrt();
    (size * size * 4 == rgba.len()).then_some(Pixels { rgba, size })
}

pub fn image(size: usize) -> Option<ColorImage> {
    let pixels = decode()?;
    if size >= pixels.size {
        return Some(ColorImage::from_rgba_unmultiplied(
            [pixels.size, pixels.size],
            &pixels.rgba,
        ));
    }
    let span = |index: usize, length: usize| index * length / size..(index + 1) * length / size;
    let mut rgba = Vec::with_capacity(size * size * 4);
    for y in 0..size {
        for x in 0..size {
            let mut sum = [0u64; 4];
            let mut count = 0;
            for source_y in span(y, pixels.size) {
                for source_x in span(x, pixels.size) {
                    let at = 4 * (source_y * pixels.size + source_x);
                    let [r, g, b, a] = [0, 1, 2, 3].map(|c| u64::from(pixels.rgba[at + c]));
                    sum[0] += r * a;
                    sum[1] += g * a;
                    sum[2] += b * a;
                    sum[3] += a * 255;
                    count += 255;
                }
            }
            rgba.extend(sum.map(|total| ((total + count / 2) / count) as u8));
        }
    }
    Some(ColorImage::from_rgba_premultiplied([size, size], &rgba))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_small_image_keeps_the_rounded_corners_and_a_solid_middle() {
        let image = image(64).expect("the icon should decode");
        assert_eq!(image.size, [64, 64]);
        assert_eq!(image.pixels[0].a(), 0);
        assert_eq!(image.pixels[32 * 64 + 32].a(), 255);
    }
}
