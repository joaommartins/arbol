use std::path::{Path, PathBuf};

use walkdir::WalkDir;

use crate::error::Result;

pub(crate) fn discover_crate_roots(root: &Path) -> Result<Vec<PathBuf>> {
    let mut crates = Vec::new();
    for entry in WalkDir::new(root).into_iter().filter_map(|e| e.ok()) {
        if entry.file_type().is_file()
            && entry.file_name() == "Cargo.toml"
            && let Some(parent) = entry.path().parent()
        {
            crates.push(parent.to_path_buf());
        }
    }
    crates.sort_by_key(|b| std::cmp::Reverse(b.as_os_str().len()));
    Ok(crates)
}

pub(crate) fn crate_for_file(file: &Path, crate_roots: &[PathBuf]) -> Option<String> {
    for root in crate_roots {
        if file.starts_with(root) {
            return Some(root.display().to_string());
        }
    }
    None
}
