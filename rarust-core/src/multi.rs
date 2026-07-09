//! Multi-volume archive handling for rarust-core.
//!
//! Detects `.partN.rar` (RAR5+) and legacy `.r00` (RAR4) naming schemes.

use std::path::{Path, PathBuf};
use std::sync::OnceLock;

use crate::error::{RarustError, Result};

fn part_n_regex() -> &'static regex_lite::Regex {
    static RE: OnceLock<regex_lite::Regex> = OnceLock::new();
    RE.get_or_init(|| {
        regex_lite::Regex::new(r"(?i)^(.+)\.part(\d+)\.rar$").expect("partN.rar regex is valid")
    })
}

/// Detect if a path belongs to a multi-volume set and return ordered volume paths.
pub fn detect_volumes(path: impl AsRef<Path>) -> Vec<PathBuf> {
    let path = path.as_ref();

    if let Some(vols) = detect_part_n_rar(path) {
        return vols;
    }

    if let Some(vols) = detect_r00_rar(path) {
        return vols;
    }

    vec![path.to_owned()]
}

/// Ensure every detected volume path exists on disk.
pub fn ensure_volumes_exist(paths: &[PathBuf]) -> Result<()> {
    for (index, path) in paths.iter().enumerate() {
        if !path.is_file() {
            return Err(RarustError::VolumeMissing {
                path: path.display().to_string(),
                number: index as u32 + 1,
            });
        }
    }
    Ok(())
}

/// Detect `name.part1.rar`, `name.part2.rar`, … (RAR5+ standard).
fn detect_part_n_rar(path: &Path) -> Option<Vec<PathBuf>> {
    let name = path.file_name()?.to_str()?;
    let caps = part_n_regex().captures(name)?;
    let stem = caps.get(1)?.as_str();
    let parent = path.parent()?;

    for padded in [false, true] {
        let volumes = collect_part_volumes(parent, stem, padded);
        if volumes.len() > 1 {
            return Some(volumes);
        }
    }

    None
}

fn collect_part_volumes(parent: &Path, stem: &str, zero_padded: bool) -> Vec<PathBuf> {
    let mut volumes = Vec::new();
    for n in 1..=999 {
        let vol_name = if zero_padded {
            format!("{stem}.part{n:03}.rar")
        } else {
            format!("{stem}.part{n}.rar")
        };
        let vol_path = parent.join(&vol_name);
        if vol_path.is_file() {
            volumes.push(vol_path);
        } else {
            break;
        }
    }
    volumes
}

/// Detect legacy `.rar` + `.r00` + `.r01` naming (RAR4).
fn detect_r00_rar(path: &Path) -> Option<Vec<PathBuf>> {
    let name = path.file_name()?.to_str()?;
    let parent = path.parent()?;

    let base = if let Some(stem) = name.strip_suffix(".rar") {
        stem
    } else if let Some(stem) = name.strip_suffix(".r00") {
        stem
    } else if name.len() >= 4
        && name.as_bytes()[name.len() - 4] == b'.'
        && name.ends_with(|c: char| c.is_ascii_digit())
    {
        // `.r01`, `.r02`, …
        let dot = name.rfind('.')?;
        &name[..dot]
    } else {
        return None;
    };

    let mut volumes = Vec::new();

    let first = parent.join(format!("{base}.rar"));
    if first.is_file() {
        volumes.push(first);
    }

    for n in 0..=999 {
        let vol_path = parent.join(format!("{base}.r{n:02}"));
        if vol_path.is_file() {
            volumes.push(vol_path);
        } else if n > 0 || volumes.is_empty() {
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
    use std::fs;

    #[test]
    fn test_single_file_not_multivol() {
        let vols = detect_volumes("test.rar");
        assert_eq!(vols.len(), 1);
    }

    #[test]
    fn detect_part_n_rar_unpadded_names() {
        let tmp = tempfile::tempdir().expect("tempdir");
        let p1 = tmp.path().join("backup.part1.rar");
        let p2 = tmp.path().join("backup.part2.rar");
        fs::write(&p1, b"vol1").expect("write p1");
        fs::write(&p2, b"vol2").expect("write p2");

        let vols = detect_volumes(&p2);
        assert_eq!(vols.len(), 2);
        assert_eq!(vols[0], p1);
        assert_eq!(vols[1], p2);
    }

    #[test]
    fn detect_legacy_r00_sequence() {
        let tmp = tempfile::tempdir().expect("tempdir");
        let p0 = tmp.path().join("legacy.rar");
        let p1 = tmp.path().join("legacy.r00");
        let p2 = tmp.path().join("legacy.r01");
        fs::write(&p0, b"v0").expect("write v0");
        fs::write(&p1, b"v1").expect("write v1");
        fs::write(&p2, b"v2").expect("write v2");

        let vols = detect_volumes(&p1);
        assert_eq!(vols.len(), 3);
        assert_eq!(vols[0], p0);
        assert_eq!(vols[1], p1);
        assert_eq!(vols[2], p2);
    }

    #[test]
    fn ensure_volumes_exist_reports_missing_part() {
        let tmp = tempfile::tempdir().expect("tempdir");
        let p1 = tmp.path().join("set.part1.rar");
        fs::write(&p1, b"only").expect("write p1");
        let paths = vec![p1, tmp.path().join("set.part2.rar")];

        let err = ensure_volumes_exist(&paths).expect_err("missing volume");
        assert!(matches!(err, RarustError::VolumeMissing { number: 2, .. }));
    }
}
