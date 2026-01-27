//! File system scanning helpers.

use crate::domain::{FileStats, Result};
use std::cmp::{max, min};
use std::path::Path;

pub fn scan_folder(path: &Path, extension: &str) -> Result<FileStats> {
    if !path.exists() {
        return Ok(Default::default());
    }

    let (count, oldest, newest) = std::fs::read_dir(path)?
        .filter_map(|e| e.ok())
        .filter(|e| {
            e.path()
                .extension()
                .map(|ext| ext.eq_ignore_ascii_case(extension))
                .unwrap_or(false)
        })
        .filter_map(|e| e.metadata().ok()?.modified().ok())
        .fold((0, None, None), |(count, oldest, newest), time| {
            (
                count + 1,
                Some(oldest.map_or(time, |old| min(old, time))),
                Some(newest.map_or(time, |new| max(new, time))),
            )
        });

    Ok(FileStats {
        count,
        oldest,
        newest,
    })
}
