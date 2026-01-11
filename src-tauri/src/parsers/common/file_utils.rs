use std::path::Path;
use walkdir::WalkDir;
use crate::models::SourceFile;

/// Scan directory for files with specific extensions
pub fn scan_directory(
    root_path: &Path,
    extensions: &[&str],
    exclude_dirs: &[&str],
) -> Vec<SourceFile> {
    let mut files = Vec::new();

    for entry in WalkDir::new(root_path)
        .follow_links(false)
        .into_iter()
        .filter_entry(|e| {
            // Skip excluded directories
            if e.file_type().is_dir() {
                let name = e.file_name().to_str().unwrap_or("");
                return !exclude_dirs.contains(&name);
            }
            true
        })
        .filter_map(|e| e.ok())
    {
        let path = entry.path();

        if path.is_file() {
            if let Some(ext) = path.extension().and_then(|e| e.to_str()) {
                let ext_lower = ext.to_lowercase();
                if extensions.iter().any(|e| e.eq_ignore_ascii_case(&ext_lower)) {
                    if let Some(file) = create_source_file(path, root_path) {
                        files.push(file);
                    }
                }
            }
        }
    }

    files
}

/// Create a SourceFile from a path
pub fn create_source_file(path: &Path, root_path: &Path) -> Option<SourceFile> {
    let name = path.file_name()?.to_str()?.to_string();
    let absolute_path = path.to_str()?.to_string();
    let relative_path = path
        .strip_prefix(root_path)
        .ok()?
        .to_str()?
        .to_string();

    let metadata = std::fs::metadata(path).ok()?;
    let size = metadata.len();

    Some(
        SourceFile::new(name, relative_path, absolute_path)
            .with_size(size)
    )
}

/// Check if file has specific extension
pub fn has_extension(path: &Path, ext: &str) -> bool {
    path.extension()
        .and_then(|e| e.to_str())
        .map(|e| e.eq_ignore_ascii_case(ext))
        .unwrap_or(false)
}

/// Get file name without extension
pub fn file_stem(path: &Path) -> Option<String> {
    path.file_stem()
        .and_then(|s| s.to_str())
        .map(|s| s.to_string())
}
