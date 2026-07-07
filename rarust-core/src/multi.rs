//! Multi-volume archive handling for rarust-core.
//!
//! Provides helpers to detect and manage multi-volume RAR archives
//! (both `.partN.rar` and legacy `.r00` naming).

use std::path::{Path, PathBuf};

/// Detect if a path looks like a multi-volume archive and return the
/// ordered list of volume paths.
pub fn detect_volumes(path: impl AsRef<Path>) -> Vec<PathBuf> {
    let path = path.as_ref();

    // Check for .partN.rar naming (RAR5+ standard)
    if let Some(vols) = detect_part_n_rar(path) {
        return vols;
    }

    // Check for legacy .r00, .r01, ... naming (RAR4)
    if let Some(vols) = detect_r00_rar(path) {
        return vols;
    }

    // Single volume
    vec![path.to_owned()]
}

/// Detect `.part1.rar`, `.part2.rar`, ... naming.
fn detect_part_n_rar(path: &Path) -> Option<Vec<PathBuf>> {
    let name = path.file_name()?.to_str()?;

    // Match pattern: anything ending in .partN.rar
    let re = regex_lite::Regex::new(r"\.part(\d+)\.rar$").ok()?;
    let caps = re.captures(name)?;
    let first_num: u32 = caps.get(1)?.as_str().parse().ok()?;

    if first_num != 1 {
        // Not starting from part1; return single volume
        return None;
    }

    let stem = re.replace(name, "").to_string();
    let parent = path.parent()?;

    let mut volumes = Vec::new();
    for n in first_num..=first_num + 999 {
        let vol_name = format!("{}.part{:03}.rar", stem, n);
        let vol_path = parent.join(&vol_name);
        if vol_path.exists() {
            volumes.push(vol_path);
        } else {
            break;
        }
    }

    if volumes.len() > 1 {
        Some(volumes)
    } else {
        None
    }
}

/// Detect legacy `.r00`, `.r01`, `.rar` naming.
fn detect_r00_rar(path: &Path) -> Option<Vec<PathBuf>> {
    let name = path.file_name()?.to_str()?;
    let parent = path.parent()?;

    // Legacy archives have .rar as the first part, then .r00, .r01, ...
    // Or the .r00 could be the first file given
    let base = if name.ends_with(".rar") {
        name.strip_suffix(".rar")?
    } else if name.ends_with(".r00") {
        name.strip_suffix(".r00")?
    } else {
        return None;
    };

    let mut volumes = Vec::new();

    // First volume is .rar
    let first = parent.join(format!("{}.rar", base));
    if first.exists() {
        volumes.push(first);
    }

    // Then .r00, .r01, ...
    for n in 0..=999 {
        let vol_name = format!("{}.r{:02}", base, n);
        let vol_path = parent.join(&vol_name);
        if vol_path.exists() {
            volumes.push(vol_path);
        } else {
            break;
        }
    }

    if volumes.len() > 1 {
        Some(volumes)
    } else {
        None
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_single_file_not_multivol() {
        let vols = detect_volumes("test.rar");
        assert_eq!(vols.len(), 1);
    }
}
