use crate::file_utils::FileUtils;
use image::imageops::FilterType;
use image::ImageReader;
use mime::Mime;
use std::path::Path;

pub struct ImageProcessor;

impl ImageProcessor {
    pub fn new() -> Self {
        ImageProcessor
    }

    /// Resize image to a square of target_size x target_size pixels.
    /// Scales proportionally by shorter side, then crops based on crop_ratio.
    /// crop_ratio: 0.0 = crop from top, 1.0 = crop from bottom, 0.5 = center
    pub fn resize_to_square(
        &self,
        filepath: &Path,
        target_size: u32,
        crop_ratio: f64,
    ) -> anyhow::Result<(Mime, Vec<u8>)> {
        let mime =
            FileUtils::get_mime_type(filepath).ok_or(anyhow::anyhow!("Failed to get MIME type"))?;
        let image_format =
            Self::image_format(&mime).ok_or(anyhow::anyhow!("Not supported MIME type"))?;

        let img = ImageReader::open(filepath)?.decode()?;
        let (orig_width, orig_height) = (img.width(), img.height());

        // Scale proportionally by shorter side to ensure both dimensions >= target_size
        let scale = if orig_width < orig_height {
            target_size as f64 / orig_width as f64
        } else {
            target_size as f64 / orig_height as f64
        };

        let new_width = (orig_width as f64 * scale).ceil() as u32;
        let new_height = (orig_height as f64 * scale).ceil() as u32;

        // Resize with high-quality Lanczos3 filter
        let resized = img.resize_exact(new_width, new_height, FilterType::Lanczos3);

        // Crop: width centered, height based on crop_ratio
        let x_offset = (new_width - target_size) / 2;
        let max_y_offset = new_height.saturating_sub(target_size);
        let y_offset = ((max_y_offset as f64) * crop_ratio.clamp(0.0, 1.0)).round() as u32;

        let cropped = resized.crop_imm(x_offset, y_offset, target_size, target_size);

        let mut bytes: Vec<u8> = Vec::new();
        cropped.write_to(&mut std::io::Cursor::new(&mut bytes), image_format)?;
        Ok((mime, bytes))
    }

    fn image_format(mime: &Mime) -> Option<image::ImageFormat> {
        match mime.type_() {
            mime::IMAGE => match mime.subtype() {
                mime::JPEG => Some(image::ImageFormat::Jpeg),
                mime::PNG => Some(image::ImageFormat::Png),
                _ => None,
            },
            _ => None,
        }
    }
}
