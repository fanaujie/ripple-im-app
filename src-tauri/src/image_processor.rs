use crate::file_utils::FileUtils;
use image::ImageReader;
use mime::Mime;
use std::path::Path;

pub struct ImageProcessor;

impl ImageProcessor {
    pub fn new() -> Self {
        ImageProcessor
    }

    pub fn crop_image(
        &self,
        filepath: &Path,
        x: u32,
        y: u32,
        width: u32,
        height: u32,
    ) -> anyhow::Result<(Mime, Vec<u8>)> {
        let mine =
            FileUtils::get_mime_type(filepath).ok_or(anyhow::anyhow!("Failed to get MIME type"))?;
        let image_format =
            Self::image_format(&mine).ok_or(anyhow::anyhow!("Not supported MIME type"))?;
        let mut img = ImageReader::open(filepath)?.decode()?;
        let crop_img = img.crop(x, y, width, height);
        let mut bytes: Vec<u8> = Vec::new();
        crop_img.write_to(&mut std::io::Cursor::new(&mut bytes), image_format)?;
        Ok((mine, bytes))
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
