use std::fs;
use std::path::Path;

use crate::errors::AppError;

/// Writes JSON output to a file path when analyst want to save reports
/// instead of printing directly to the terminal.
pub fn write_output(path: impl AsRef<Path>, content: &str) -> Result<(), AppError> {
    let path = path.as_ref();

    if let Some(parent) = path.parent() {
        if !parent.as_os_str().is_empty() {
            fs::create_dir_all(parent)?;
        }
    }

    fs::write(path, content)?;
    Ok(())
}
