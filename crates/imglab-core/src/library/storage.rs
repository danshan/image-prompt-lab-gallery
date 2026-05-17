use super::{io_error, CHECKSUM_MD5, CHECKSUM_SHA256, REQUIRED_DIRS};
use crate::{
    hash::{md5_reader, sha256_reader},
    AssetVersionId, DomainError, DomainResult,
};
use std::fs::{self, File};
use std::path::{Component, Path, PathBuf};
use std::time::{SystemTime, UNIX_EPOCH};

pub(super) fn timestamp_string() -> String {
    let millis = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|duration| duration.as_millis())
        .unwrap_or_default();
    millis.to_string()
}

pub(super) fn managed_original_path(
    version_id: &AssetVersionId,
    extension: &str,
    timestamp: &str,
) -> PathBuf {
    let millis = timestamp.parse::<u128>().unwrap_or_default();
    let days = (millis / 86_400_000) as i64;
    let (year, month, _) = civil_from_days(days);
    PathBuf::from("originals")
        .join(format!("{year:04}"))
        .join(format!("{month:02}"))
        .join(format!("{}.{}", version_id.0, extension))
}

fn civil_from_days(days_since_unix_epoch: i64) -> (i32, u32, u32) {
    let z = days_since_unix_epoch + 719_468;
    let era = if z >= 0 { z } else { z - 146_096 } / 146_097;
    let doe = z - era * 146_097;
    let yoe = (doe - doe / 1_460 + doe / 36_524 - doe / 146_096) / 365;
    let y = yoe + era * 400;
    let doy = doe - (365 * yoe + yoe / 4 - yoe / 100);
    let mp = (5 * doy + 2) / 153;
    let day = doy - (153 * mp + 2) / 5 + 1;
    let month = mp + if mp < 10 { 3 } else { -9 };
    let year = y + if month <= 2 { 1 } else { 0 };
    (year as i32, month as u32, day as u32)
}

pub(super) fn file_digest(path: &Path, algorithm: &str) -> DomainResult<String> {
    let file = File::open(path).map_err(|error| io_error(path, error))?;
    match algorithm {
        CHECKSUM_MD5 => md5_reader(file).map_err(|error| io_error(path, error)),
        CHECKSUM_SHA256 => sha256_reader(file).map_err(|error| io_error(path, error)),
        other => Err(DomainError::Database {
            message: format!("unsupported checksum algorithm: {other}"),
        }),
    }
}

pub(super) fn image_dimensions(path: &Path) -> DomainResult<(Option<u32>, Option<u32>)> {
    let bytes = fs::read(path).map_err(|error| io_error(path, error))?;
    Ok(parse_image_dimensions(&bytes).unwrap_or((None, None)))
}

fn parse_image_dimensions(bytes: &[u8]) -> Option<(Option<u32>, Option<u32>)> {
    parse_png_dimensions(bytes)
        .or_else(|| parse_jpeg_dimensions(bytes))
        .or_else(|| parse_webp_dimensions(bytes))
        .map(|(width, height)| (Some(width), Some(height)))
}

fn parse_png_dimensions(bytes: &[u8]) -> Option<(u32, u32)> {
    const PNG_SIGNATURE: &[u8; 8] = b"\x89PNG\r\n\x1a\n";
    if bytes.len() < 24 || &bytes[0..8] != PNG_SIGNATURE || &bytes[12..16] != b"IHDR" {
        return None;
    }
    let width = u32::from_be_bytes(bytes[16..20].try_into().ok()?);
    let height = u32::from_be_bytes(bytes[20..24].try_into().ok()?);
    non_zero_dimensions(width, height)
}

fn parse_jpeg_dimensions(bytes: &[u8]) -> Option<(u32, u32)> {
    if bytes.len() < 4 || bytes[0] != 0xff || bytes[1] != 0xd8 {
        return None;
    }
    let mut index = 2usize;
    while index + 3 < bytes.len() {
        while index < bytes.len() && bytes[index] != 0xff {
            index += 1;
        }
        while index < bytes.len() && bytes[index] == 0xff {
            index += 1;
        }
        if index >= bytes.len() {
            return None;
        }
        let marker = bytes[index];
        index += 1;
        if marker == 0xd9 || marker == 0xda {
            return None;
        }
        if index + 2 > bytes.len() {
            return None;
        }
        let length = u16::from_be_bytes(bytes[index..index + 2].try_into().ok()?) as usize;
        if length < 2 || index + length > bytes.len() {
            return None;
        }
        let payload = index + 2;
        if is_jpeg_sof_marker(marker) {
            if length < 7 {
                return None;
            }
            let height =
                u16::from_be_bytes(bytes[payload + 1..payload + 3].try_into().ok()?) as u32;
            let width = u16::from_be_bytes(bytes[payload + 3..payload + 5].try_into().ok()?) as u32;
            return non_zero_dimensions(width, height);
        }
        index += length;
    }
    None
}

fn is_jpeg_sof_marker(marker: u8) -> bool {
    matches!(
        marker,
        0xc0 | 0xc1 | 0xc2 | 0xc3 | 0xc5 | 0xc6 | 0xc7 | 0xc9 | 0xca | 0xcb | 0xcd | 0xce | 0xcf
    )
}

fn parse_webp_dimensions(bytes: &[u8]) -> Option<(u32, u32)> {
    if bytes.len() < 30 || &bytes[0..4] != b"RIFF" || &bytes[8..12] != b"WEBP" {
        return None;
    }
    match &bytes[12..16] {
        b"VP8X" if bytes.len() >= 30 => {
            let width = 1 + u32::from_le_bytes([bytes[24], bytes[25], bytes[26], 0]);
            let height = 1 + u32::from_le_bytes([bytes[27], bytes[28], bytes[29], 0]);
            non_zero_dimensions(width, height)
        }
        b"VP8L" if bytes.len() >= 25 && bytes[20] == 0x2f => {
            let b1 = bytes[21] as u32;
            let b2 = bytes[22] as u32;
            let b3 = bytes[23] as u32;
            let b4 = bytes[24] as u32;
            let width = 1 + (((b2 & 0x3f) << 8) | b1);
            let height = 1 + (((b4 & 0x0f) << 10) | (b3 << 2) | ((b2 & 0xc0) >> 6));
            non_zero_dimensions(width, height)
        }
        b"VP8 " if bytes.len() >= 30 && bytes[23..26] == [0x9d, 0x01, 0x2a] => {
            let width = (u16::from_le_bytes(bytes[26..28].try_into().ok()?) & 0x3fff) as u32;
            let height = (u16::from_le_bytes(bytes[28..30].try_into().ok()?) & 0x3fff) as u32;
            non_zero_dimensions(width, height)
        }
        _ => None,
    }
}

fn non_zero_dimensions(width: u32, height: u32) -> Option<(u32, u32)> {
    if width == 0 || height == 0 {
        None
    } else {
        Some((width, height))
    }
}

pub(super) fn is_safe_relative_path(path: &Path) -> bool {
    path.components()
        .all(|component| matches!(component, Component::Normal(_)))
}

pub(super) fn managed_storage_size(root_path: &Path) -> DomainResult<u64> {
    REQUIRED_DIRS
        .iter()
        .filter(|relative| !relative.contains('/'))
        .try_fold(0u64, |total, relative| {
            directory_size(&root_path.join(relative)).map(|size| total + size)
        })
}

fn directory_size(path: &Path) -> DomainResult<u64> {
    if !path.exists() {
        return Ok(0);
    }
    let mut total = 0u64;
    for entry in fs::read_dir(path).map_err(|error| io_error(path, error))? {
        let entry = entry.map_err(|error| io_error(path, error))?;
        let metadata = entry
            .metadata()
            .map_err(|error| io_error(&entry.path(), error))?;
        if metadata.is_dir() {
            total += directory_size(&entry.path())?;
        } else if metadata.is_file() {
            total += metadata.len();
        }
    }
    Ok(total)
}

pub(super) fn normalized_extension(path: &Path) -> String {
    path.extension()
        .and_then(|extension| extension.to_str())
        .map(|extension| extension.to_ascii_lowercase())
        .filter(|extension| !extension.is_empty())
        .unwrap_or_else(|| "bin".to_string())
}

pub(super) fn mime_type_for_extension(extension: &str) -> &'static str {
    match extension {
        "jpg" | "jpeg" => "image/jpeg",
        "png" => "image/png",
        "webp" => "image/webp",
        "gif" => "image/gif",
        "avif" => "image/avif",
        _ => "application/octet-stream",
    }
}

pub(super) fn extension_for_mime_type(mime_type: &str) -> &'static str {
    match mime_type {
        "image/jpeg" => "jpg",
        "image/webp" => "webp",
        "image/gif" => "gif",
        "image/avif" => "avif",
        _ => "png",
    }
}
