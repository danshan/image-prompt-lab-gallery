use super::{
    database_error, io_error, LibraryManifest, LocalLibraryService, CURRENT_SCHEMA_VERSION,
    DATABASE_FILE, MANIFEST_FILE, REQUIRED_DIRS,
};
use crate::{DomainError, DomainResult};
use rusqlite::Connection;
use std::fs::{self, File};
use std::io::{Read, Seek, SeekFrom, Write};
use std::path::{Component, Path, PathBuf};
use uuid::Uuid;

const ZIP_LOCAL_FILE_HEADER: u32 = 0x0403_4b50;
const ZIP_CENTRAL_DIRECTORY_HEADER: u32 = 0x0201_4b50;
const ZIP_END_OF_CENTRAL_DIRECTORY: u32 = 0x0605_4b50;
const ZIP_VERSION_NEEDED: u16 = 20;
const ZIP_STORED_METHOD: u16 = 0;

#[derive(Debug, Clone)]
struct ZipEntry {
    name: String,
    crc32: u32,
    size: u32,
    local_header_offset: u32,
    is_dir: bool,
}

#[derive(Debug, Clone)]
struct CentralDirectoryEntry {
    name: String,
    compressed_size: u32,
    uncompressed_size: u32,
    local_header_offset: u32,
    is_dir: bool,
}

pub(super) fn export_backup_zip(root_path: &Path, output_zip_path: &Path) -> DomainResult<()> {
    LocalLibraryService::validate_layout(root_path)?;
    let manifest = LocalLibraryService::read_manifest(root_path)?;
    if manifest.schema_version > CURRENT_SCHEMA_VERSION {
        return Err(DomainError::SchemaMismatch {
            expected: CURRENT_SCHEMA_VERSION,
            found: manifest.schema_version,
        });
    }
    let database_path = LocalLibraryService::database_path(root_path);
    let connection = Connection::open(&database_path).map_err(database_error)?;
    super::migrate_library_database(&connection)?;
    drop(connection);

    if let Some(parent) = output_zip_path.parent() {
        fs::create_dir_all(parent).map_err(|error| io_error(parent, error))?;
    }
    let temp_zip_path = output_zip_path.with_extension(format!(
        "{}.tmp",
        output_zip_path
            .extension()
            .and_then(|value| value.to_str())
            .unwrap_or("zip")
    ));
    if temp_zip_path.exists() {
        fs::remove_file(&temp_zip_path).map_err(|error| io_error(&temp_zip_path, error))?;
    }

    let export_result = write_backup_zip(root_path, &temp_zip_path);
    if export_result.is_err() {
        let _ = fs::remove_file(&temp_zip_path);
    }
    export_result?;
    fs::rename(&temp_zip_path, output_zip_path).map_err(|error| io_error(output_zip_path, error))
}

pub(super) fn import_backup_zip(
    zip_path: &Path,
    destination_path: &Path,
    registry_contains_id: impl Fn(&str) -> DomainResult<bool>,
) -> DomainResult<(LibraryManifest, bool)> {
    if destination_path.exists() && !is_empty_dir(destination_path)? {
        return Err(DomainError::ImportDestinationNotEmpty {
            path: destination_path.display().to_string(),
        });
    }
    if let Some(parent) = destination_path.parent() {
        fs::create_dir_all(parent).map_err(|error| io_error(parent, error))?;
    }

    let staging_path = destination_path.with_file_name(format!(
        ".{}.importing-{}",
        destination_path
            .file_name()
            .and_then(|value| value.to_str())
            .unwrap_or("library"),
        Uuid::new_v4()
    ));
    if staging_path.exists() {
        fs::remove_dir_all(&staging_path).map_err(|error| io_error(&staging_path, error))?;
    }
    fs::create_dir_all(&staging_path).map_err(|error| io_error(&staging_path, error))?;

    let import_result = import_backup_zip_inner(zip_path, &staging_path, registry_contains_id);
    match import_result {
        Ok((manifest, cloned)) => {
            if destination_path.exists() {
                fs::remove_dir(destination_path)
                    .map_err(|error| io_error(destination_path, error))?;
            }
            fs::rename(&staging_path, destination_path)
                .map_err(|error| io_error(destination_path, error))?;
            Ok((manifest, cloned))
        }
        Err(error) => {
            let _ = fs::remove_dir_all(&staging_path);
            Err(error)
        }
    }
}

fn import_backup_zip_inner(
    zip_path: &Path,
    staging_path: &Path,
    registry_contains_id: impl Fn(&str) -> DomainResult<bool>,
) -> DomainResult<(LibraryManifest, bool)> {
    extract_backup_zip(zip_path, staging_path)?;
    LocalLibraryService::validate_layout(staging_path)?;
    let mut manifest = LocalLibraryService::read_manifest(staging_path)?;
    if manifest.schema_version > CURRENT_SCHEMA_VERSION {
        return Err(DomainError::SchemaMismatch {
            expected: CURRENT_SCHEMA_VERSION,
            found: manifest.schema_version,
        });
    }
    let database_path = LocalLibraryService::database_path(staging_path);
    let connection = Connection::open(&database_path).map_err(database_error)?;
    super::migrate_library_database(&connection)?;
    drop(connection);

    let cloned = registry_contains_id(&manifest.id)?;
    if cloned {
        manifest.id = Uuid::new_v4().to_string();
        manifest.name = cloned_library_name(&manifest.name);
        LocalLibraryService::write_manifest(staging_path, &manifest)?;
    }

    Ok((manifest, cloned))
}

fn write_backup_zip(root_path: &Path, temp_zip_path: &Path) -> DomainResult<()> {
    let mut entries = Vec::new();
    collect_entries(root_path, root_path, &mut entries)?;
    entries.sort();

    let mut writer = File::create(temp_zip_path).map_err(|error| io_error(temp_zip_path, error))?;
    let mut zip_entries = Vec::new();

    for relative_path in entries {
        let source_path = root_path.join(&relative_path);
        let name = zip_name(&relative_path)?;
        if source_path.is_dir() {
            let entry = write_zip_entry(&mut writer, &format!("{name}/"), &[], true)?;
            zip_entries.push(entry);
            continue;
        }

        let mut bytes = Vec::new();
        File::open(&source_path)
            .and_then(|mut file| file.read_to_end(&mut bytes))
            .map_err(|error| io_error(&source_path, error))?;
        let entry = write_zip_entry(&mut writer, &name, &bytes, false)?;
        zip_entries.push(entry);
    }

    write_central_directory(&mut writer, &zip_entries)
        .map_err(|error| zip_io_error(temp_zip_path, error.to_string()))
}

fn collect_entries(
    root_path: &Path,
    current_path: &Path,
    entries: &mut Vec<PathBuf>,
) -> DomainResult<()> {
    for entry in fs::read_dir(current_path).map_err(|error| io_error(current_path, error))? {
        let entry = entry.map_err(|error| io_error(current_path, error))?;
        let path = entry.path();
        let relative = path
            .strip_prefix(root_path)
            .map_err(|error| zip_io_error(&path, error.to_string()))?
            .to_path_buf();
        entries.push(relative.clone());
        if path.is_dir() {
            collect_entries(root_path, &path, entries)?;
        }
    }
    Ok(())
}

fn write_zip_entry<W: Write + Seek>(
    writer: &mut W,
    name: &str,
    bytes: &[u8],
    is_dir: bool,
) -> DomainResult<ZipEntry> {
    let local_header_offset = checked_u32(
        writer
            .stream_position()
            .map_err(|error| zip_io_error(Path::new(name), error.to_string()))?,
        name,
    )?;
    let crc32 = if is_dir { 0 } else { crc32(bytes) };
    let size = checked_u32(bytes.len() as u64, name)?;
    let name_bytes = name.as_bytes();
    let name_len = checked_u16(name_bytes.len(), name)?;

    write_u32_zip(writer, ZIP_LOCAL_FILE_HEADER, name)?;
    write_u16_zip(writer, ZIP_VERSION_NEEDED, name)?;
    write_u16_zip(writer, 0, name)?;
    write_u16_zip(writer, ZIP_STORED_METHOD, name)?;
    write_u16_zip(writer, 0, name)?;
    write_u16_zip(writer, 0, name)?;
    write_u32_zip(writer, crc32, name)?;
    write_u32_zip(writer, size, name)?;
    write_u32_zip(writer, size, name)?;
    write_u16_zip(writer, name_len, name)?;
    write_u16_zip(writer, 0, name)?;
    writer
        .write_all(name_bytes)
        .map_err(|error| zip_io_error(Path::new(name), error.to_string()))?;
    if !is_dir {
        writer
            .write_all(bytes)
            .map_err(|error| zip_io_error(Path::new(name), error.to_string()))?;
    }

    Ok(ZipEntry {
        name: name.to_string(),
        crc32,
        size,
        local_header_offset,
        is_dir,
    })
}

fn write_central_directory<W: Write + Seek>(
    writer: &mut W,
    entries: &[ZipEntry],
) -> std::io::Result<()> {
    let central_directory_offset = writer.stream_position()?;

    for entry in entries {
        let name_bytes = entry.name.as_bytes();
        write_u32(writer, ZIP_CENTRAL_DIRECTORY_HEADER)?;
        write_u16(writer, ZIP_VERSION_NEEDED)?;
        write_u16(writer, ZIP_VERSION_NEEDED)?;
        write_u16(writer, 0)?;
        write_u16(writer, ZIP_STORED_METHOD)?;
        write_u16(writer, 0)?;
        write_u16(writer, 0)?;
        write_u32(writer, entry.crc32)?;
        write_u32(writer, entry.size)?;
        write_u32(writer, entry.size)?;
        write_u16(writer, name_bytes.len() as u16)?;
        write_u16(writer, 0)?;
        write_u16(writer, 0)?;
        write_u16(writer, 0)?;
        write_u16(writer, 0)?;
        write_u32(writer, if entry.is_dir { 0x10 } else { 0 })?;
        write_u32(writer, entry.local_header_offset)?;
        writer.write_all(name_bytes)?;
    }

    let central_directory_size = writer.stream_position()? - central_directory_offset;
    write_u32(writer, ZIP_END_OF_CENTRAL_DIRECTORY)?;
    write_u16(writer, 0)?;
    write_u16(writer, 0)?;
    write_u16(writer, entries.len() as u16)?;
    write_u16(writer, entries.len() as u16)?;
    write_u32(writer, central_directory_size as u32)?;
    write_u32(writer, central_directory_offset as u32)?;
    write_u16(writer, 0)?;
    Ok(())
}

fn extract_backup_zip(zip_path: &Path, destination_path: &Path) -> DomainResult<()> {
    let mut file =
        File::open(zip_path).map_err(|error| zip_io_error(zip_path, error.to_string()))?;
    let entries = read_central_directory(&mut file, zip_path)?;
    for entry in entries {
        let relative_path = safe_zip_path(&entry.name)?;
        let output_path = destination_path.join(relative_path);
        if entry.is_dir || entry.name.ends_with('/') {
            fs::create_dir_all(&output_path).map_err(|error| io_error(&output_path, error))?;
            continue;
        }
        if let Some(parent) = output_path.parent() {
            fs::create_dir_all(parent).map_err(|error| io_error(parent, error))?;
        }
        let bytes = read_zip_file_entry(&mut file, zip_path, &entry)?;
        fs::write(&output_path, bytes).map_err(|error| io_error(&output_path, error))?;
    }

    for required in REQUIRED_DIRS {
        if !destination_path.join(required).is_dir() {
            return Err(DomainError::InvalidLibraryBackup {
                message: format!("required directory is missing: {required}"),
            });
        }
    }
    if !destination_path.join(MANIFEST_FILE).is_file() {
        return Err(DomainError::InvalidLibraryBackup {
            message: "manifest.json is missing".to_string(),
        });
    }
    if !destination_path.join(DATABASE_FILE).is_file() {
        return Err(DomainError::InvalidLibraryBackup {
            message: "library.sqlite is missing".to_string(),
        });
    }
    Ok(())
}

fn read_central_directory(
    file: &mut File,
    zip_path: &Path,
) -> DomainResult<Vec<CentralDirectoryEntry>> {
    let file_len = file
        .metadata()
        .map_err(|error| zip_io_error(zip_path, error.to_string()))?
        .len();
    if file_len < 22 {
        return Err(invalid_backup("zip file is too small"));
    }
    let search_len = file_len.min(66_000) as usize;
    file.seek(SeekFrom::End(-(search_len as i64)))
        .map_err(|error| zip_io_error(zip_path, error.to_string()))?;
    let mut tail = vec![0; search_len];
    file.read_exact(&mut tail)
        .map_err(|error| zip_io_error(zip_path, error.to_string()))?;
    let eocd_index = tail
        .windows(4)
        .rposition(|window| window == ZIP_END_OF_CENTRAL_DIRECTORY.to_le_bytes())
        .ok_or_else(|| invalid_backup("zip end of central directory is missing"))?;
    if eocd_index + 22 > tail.len() {
        return Err(invalid_backup("zip end of central directory is truncated"));
    }
    let entry_count = read_u16_from(&tail, eocd_index + 10) as usize;
    let central_directory_offset = read_u32_from(&tail, eocd_index + 16) as u64;
    file.seek(SeekFrom::Start(central_directory_offset))
        .map_err(|error| zip_io_error(zip_path, error.to_string()))?;

    let mut entries = Vec::with_capacity(entry_count);
    for _ in 0..entry_count {
        let signature = read_u32(file, zip_path)?;
        if signature != ZIP_CENTRAL_DIRECTORY_HEADER {
            return Err(invalid_backup(
                "central directory entry has invalid signature",
            ));
        }
        skip(file, 6, zip_path)?;
        let compression_method = read_u16(file, zip_path)?;
        skip(file, 4, zip_path)?;
        let _crc32 = read_u32(file, zip_path)?;
        let compressed_size = read_u32(file, zip_path)?;
        let uncompressed_size = read_u32(file, zip_path)?;
        let name_len = read_u16(file, zip_path)? as usize;
        let extra_len = read_u16(file, zip_path)? as usize;
        let comment_len = read_u16(file, zip_path)? as usize;
        skip(file, 4, zip_path)?;
        let external_attributes = read_u32(file, zip_path)?;
        let local_header_offset = read_u32(file, zip_path)?;
        if compression_method != ZIP_STORED_METHOD {
            return Err(invalid_backup("only stored zip entries are supported"));
        }
        let mut name_bytes = vec![0; name_len];
        file.read_exact(&mut name_bytes)
            .map_err(|error| zip_io_error(zip_path, error.to_string()))?;
        skip(file, extra_len + comment_len, zip_path)?;
        let name = String::from_utf8(name_bytes)
            .map_err(|_| invalid_backup("zip entry name is not utf-8"))?;
        entries.push(CentralDirectoryEntry {
            name,
            compressed_size,
            uncompressed_size,
            local_header_offset,
            is_dir: external_attributes & 0x10 != 0,
        });
    }
    Ok(entries)
}

fn read_zip_file_entry(
    file: &mut File,
    zip_path: &Path,
    entry: &CentralDirectoryEntry,
) -> DomainResult<Vec<u8>> {
    if entry.compressed_size != entry.uncompressed_size {
        return Err(invalid_backup(
            "compressed and uncompressed sizes differ for stored entry",
        ));
    }
    file.seek(SeekFrom::Start(entry.local_header_offset as u64))
        .map_err(|error| zip_io_error(zip_path, error.to_string()))?;
    if read_u32(file, zip_path)? != ZIP_LOCAL_FILE_HEADER {
        return Err(invalid_backup("local file header has invalid signature"));
    }
    skip(file, 22, zip_path)?;
    let name_len = read_u16(file, zip_path)? as u64;
    let extra_len = read_u16(file, zip_path)? as u64;
    file.seek(SeekFrom::Current((name_len + extra_len) as i64))
        .map_err(|error| zip_io_error(zip_path, error.to_string()))?;
    let mut bytes = vec![0; entry.uncompressed_size as usize];
    file.read_exact(&mut bytes)
        .map_err(|error| zip_io_error(zip_path, error.to_string()))?;
    Ok(bytes)
}

fn is_empty_dir(path: &Path) -> DomainResult<bool> {
    if !path.is_dir() {
        return Ok(false);
    }
    let mut entries = fs::read_dir(path).map_err(|error| io_error(path, error))?;
    Ok(entries.next().is_none())
}

fn safe_zip_path(name: &str) -> DomainResult<PathBuf> {
    let path = Path::new(name);
    let mut safe = PathBuf::new();
    for component in path.components() {
        match component {
            Component::Normal(value) => safe.push(value),
            Component::CurDir => {}
            _ => {
                return Err(DomainError::InvalidLibraryBackup {
                    message: format!("unsafe zip entry path: {name}"),
                });
            }
        }
    }
    Ok(safe)
}

fn zip_name(path: &Path) -> DomainResult<String> {
    let mut parts = Vec::new();
    for component in path.components() {
        match component {
            Component::Normal(value) => parts.push(value.to_string_lossy().to_string()),
            _ => {
                return Err(DomainError::InvalidLibraryBackup {
                    message: format!("unsupported backup path: {}", path.display()),
                });
            }
        }
    }
    Ok(parts.join("/"))
}

fn cloned_library_name(name: &str) -> String {
    if name.trim().is_empty() {
        "Imported Library Copy".to_string()
    } else {
        format!("{name} Copy")
    }
}

fn checked_u16(value: usize, name: &str) -> DomainResult<u16> {
    u16::try_from(value).map_err(|_| DomainError::InvalidLibraryBackup {
        message: format!("zip entry name is too long: {name}"),
    })
}

fn checked_u32(value: u64, name: &str) -> DomainResult<u32> {
    u32::try_from(value).map_err(|_| DomainError::InvalidLibraryBackup {
        message: format!("zip entry is too large for backup format: {name}"),
    })
}

fn invalid_backup(message: &str) -> DomainError {
    DomainError::InvalidLibraryBackup {
        message: message.to_string(),
    }
}

fn zip_io_error(path: &Path, message: String) -> DomainError {
    DomainError::ZipIoError {
        path: path.display().to_string(),
        message,
    }
}

fn skip(file: &mut File, bytes: usize, zip_path: &Path) -> DomainResult<()> {
    file.seek(SeekFrom::Current(bytes as i64))
        .map(|_| ())
        .map_err(|error| zip_io_error(zip_path, error.to_string()))
}

fn write_u16<W: Write>(writer: &mut W, value: u16) -> std::io::Result<()> {
    writer.write_all(&value.to_le_bytes())
}

fn write_u32<W: Write>(writer: &mut W, value: u32) -> std::io::Result<()> {
    writer.write_all(&value.to_le_bytes())
}

fn write_u16_zip<W: Write>(writer: &mut W, value: u16, name: &str) -> DomainResult<()> {
    write_u16(writer, value).map_err(|error| zip_io_error(Path::new(name), error.to_string()))
}

fn write_u32_zip<W: Write>(writer: &mut W, value: u32, name: &str) -> DomainResult<()> {
    write_u32(writer, value).map_err(|error| zip_io_error(Path::new(name), error.to_string()))
}

fn read_u16(file: &mut File, zip_path: &Path) -> DomainResult<u16> {
    let mut bytes = [0; 2];
    file.read_exact(&mut bytes)
        .map_err(|error| zip_io_error(zip_path, error.to_string()))?;
    Ok(u16::from_le_bytes(bytes))
}

fn read_u32(file: &mut File, zip_path: &Path) -> DomainResult<u32> {
    let mut bytes = [0; 4];
    file.read_exact(&mut bytes)
        .map_err(|error| zip_io_error(zip_path, error.to_string()))?;
    Ok(u32::from_le_bytes(bytes))
}

fn read_u16_from(bytes: &[u8], offset: usize) -> u16 {
    u16::from_le_bytes([bytes[offset], bytes[offset + 1]])
}

fn read_u32_from(bytes: &[u8], offset: usize) -> u32 {
    u32::from_le_bytes([
        bytes[offset],
        bytes[offset + 1],
        bytes[offset + 2],
        bytes[offset + 3],
    ])
}

fn crc32(bytes: &[u8]) -> u32 {
    let mut crc = 0xffff_ffff;
    for byte in bytes {
        crc ^= u32::from(*byte);
        for _ in 0..8 {
            let mask = 0u32.wrapping_sub(crc & 1);
            crc = (crc >> 1) ^ (0xedb8_8320 & mask);
        }
    }
    !crc
}
