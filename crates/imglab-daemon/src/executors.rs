fn append_log(path: &Path, line: &str) -> DomainResult<()> {
    fs::OpenOptions::new()
        .create(true)
        .append(true)
        .open(path)
        .and_then(|mut file| file.write_all(line.as_bytes()))
        .map_err(|error| io_error(path, error))
}

fn cancel_marker_path(log_path: &Path) -> PathBuf {
    let mut marker = log_path.to_path_buf();
    marker.set_extension("cancel");
    marker
}

fn unix_timestamp_string(add_seconds: u64) -> String {
    let seconds = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|duration| duration.as_secs() + add_seconds)
        .unwrap_or(add_seconds);
    seconds.to_string()
}

