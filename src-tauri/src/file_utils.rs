use mime::Mime;
use std::path::Path;

pub struct FileUtils;

impl FileUtils {
    pub fn get_extension(filepath: &Path) -> Option<&str> {
        filepath.extension().and_then(|ext| ext.to_str())
    }

    pub fn get_mime_type(filepath: &Path) -> Option<Mime> {
        let file_extension = match Self::get_extension(filepath) {
            Some(ext) => ext,
            None => return None,
        };
        match file_extension.to_lowercase().as_str() {
            "jpg" | "jpeg" => Some(mime::IMAGE_JPEG),
            "png" => Some(mime::IMAGE_PNG),
            _ => None,
        }
    }

    pub fn get_file_name(filepath: &Path) -> Option<&str> {
        filepath.file_name().and_then(|os| os.to_str())
    }
}
