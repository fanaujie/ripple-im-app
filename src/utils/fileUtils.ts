/**
 * File utility functions for handling file types and operations
 */

// Image file extensions (case-insensitive)
const IMAGE_EXTENSIONS = new Set([
  'jpg',
  'jpeg',
  'png',
  'gif',
  'webp',
  'svg',
  'bmp',
  'ico',
]);

/**
 * Check if a file is an image based on its filename
 *
 * @param fileName - The file name or URL to check
 * @returns true if the file is an image, false otherwise
 */
export function isImageFile(fileName: string | null | undefined): boolean {
  if (!fileName) return false;

  // Extract extension from filename or URL
  const parts = fileName.toLowerCase().split('.');
  if (parts.length < 2) return false;

  const extension = parts[parts.length - 1];
  // Handle URL query params (e.g., "image.jpg?v=1")
  const cleanExtension = extension.split('?')[0];

  return IMAGE_EXTENSIONS.has(cleanExtension);
}

/**
 * Get file extension from filename
 *
 * @param fileName - The file name to extract extension from
 * @returns The file extension (lowercase) or empty string if none
 */
export function getFileExtension(fileName: string | null | undefined): string {
  if (!fileName) return '';

  const parts = fileName.split('.');
  if (parts.length < 2) return '';

  const extension = parts[parts.length - 1].toLowerCase();
  // Handle URL query params
  return extension.split('?')[0];
}

/**
 * Get a human-readable file type from extension
 *
 * @param fileName - The file name to get type for
 * @returns Human-readable file type string
 */
export function getFileType(fileName: string | null | undefined): string {
  const ext = getFileExtension(fileName);
  if (!ext) return 'File';

  const typeMap: Record<string, string> = {
    // Documents
    pdf: 'PDF',
    doc: 'Word',
    docx: 'Word',
    xls: 'Excel',
    xlsx: 'Excel',
    ppt: 'PowerPoint',
    pptx: 'PowerPoint',
    txt: 'Text',
    rtf: 'RTF',
    // Archives
    zip: 'ZIP',
    rar: 'RAR',
    '7z': '7-Zip',
    tar: 'TAR',
    gz: 'GZip',
    // Audio
    mp3: 'MP3',
    wav: 'WAV',
    flac: 'FLAC',
    aac: 'AAC',
    ogg: 'OGG',
    // Video
    mp4: 'MP4',
    avi: 'AVI',
    mkv: 'MKV',
    mov: 'MOV',
    wmv: 'WMV',
    webm: 'WebM',
    // Images
    jpg: 'JPEG',
    jpeg: 'JPEG',
    png: 'PNG',
    gif: 'GIF',
    webp: 'WebP',
    svg: 'SVG',
    bmp: 'BMP',
  };

  return typeMap[ext] || ext.toUpperCase();
}

/**
 * Extract filename from a URL or path
 *
 * @param urlOrPath - URL or file path
 * @returns The extracted filename
 */
export function extractFileName(urlOrPath: string | null | undefined): string {
  if (!urlOrPath) return 'file';

  // Handle URLs
  try {
    const url = new URL(urlOrPath);
    const pathParts = url.pathname.split('/');
    const fileName = pathParts[pathParts.length - 1];
    if (fileName) {
      // Decode URL-encoded characters
      return decodeURIComponent(fileName);
    }
  } catch {
    // Not a valid URL, treat as path
  }

  // Handle file paths (both Unix and Windows)
  const parts = urlOrPath.split(/[/\\]/);
  return parts[parts.length - 1] || 'file';
}

/**
 * Download a file from URL with specified filename
 * Shows a save dialog and writes the file to the selected path
 *
 * @param url - The file URL to download
 * @param fileName - Filename for the download (defaults to extracted from URL)
 */
export async function downloadFile(url: string, fileName?: string): Promise<void> {
  const { save } = await import('@tauri-apps/plugin-dialog');
  const { writeFile } = await import('@tauri-apps/plugin-fs');

  const downloadName = fileName || extractFileName(url);

  // Fetch file as blob
  const response = await fetch(url);
  if (!response.ok) {
    throw new Error(`Download failed: ${response.status}`);
  }

  const blob = await response.blob();
  const arrayBuffer = await blob.arrayBuffer();
  const uint8Array = new Uint8Array(arrayBuffer);

  // Show save dialog with default filename
  const filePath = await save({
    defaultPath: downloadName,
  });

  if (filePath) {
    // Write file to selected path
    await writeFile(filePath, uint8Array);
  }
}
