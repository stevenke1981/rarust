//! Shared utility functions for rarust-core.

/// Format a byte count as a human-readable string.
///
/// # Examples
/// ```
/// assert_eq!(rarust_core::util::format_size(0), "0 B");
/// assert_eq!(rarust_core::util::format_size(1024), "1.0 KB");
/// assert_eq!(rarust_core::util::format_size(1_048_576), "1.0 MB");
/// ```
pub fn format_size(bytes: u64) -> String {
    const UNITS: &[&str] = &["B", "KB", "MB", "GB", "TB", "PB"];
    let mut size = bytes as f64;
    let mut unit_idx = 0;
    while size >= 1024.0 && unit_idx < UNITS.len() - 1 {
        size /= 1024.0;
        unit_idx += 1;
    }
    if unit_idx == 0 {
        format!("{} {}", bytes, UNITS[unit_idx])
    } else {
        format!("{:.1} {}", size, UNITS[unit_idx])
    }
}

/// Format a duration in seconds as a human-readable string.
///
/// # Examples
/// ```
/// assert_eq!(rarust_core::util::format_duration(0), "00:00:00");
/// assert_eq!(rarust_core::util::format_duration(3661), "01:01:01");
/// ```
pub fn format_duration(secs: u64) -> String {
    let h = secs / 3600;
    let m = (secs % 3600) / 60;
    let s = secs % 60;
    format!("{:02}:{:02}:{:02}", h, m, s)
}

/// Format a DOS timestamp to a display string.
pub fn format_dos_time(dos_time: u32) -> String {
    // DOS timestamp format:
    // bits 0-4: seconds/2 (0-29)
    // bits 5-10: minutes (0-59)
    // bits 11-15: hours (0-23)
    // bits 16-20: day (1-31)
    // bits 21-24: month (1-12)
    // bits 25-31: year - 1980 (0-127)
    let seconds = (dos_time & 0x1F) * 2;
    let minutes = (dos_time >> 5) & 0x3F;
    let hours = (dos_time >> 11) & 0x1F;
    let day = (dos_time >> 16) & 0x1F;
    let month = (dos_time >> 21) & 0x0F;
    let year = 1980 + ((dos_time >> 25) & 0x7F);

    format!(
        "{:04}-{:02}-{:02} {:02}:{:02}:{:02}",
        year, month, day, hours, minutes, seconds
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_format_size_bytes() {
        assert_eq!(format_size(0), "0 B");
        assert_eq!(format_size(1), "1 B");
        assert_eq!(format_size(1023), "1023 B");
    }

    #[test]
    fn test_format_size_kb() {
        assert_eq!(format_size(1024), "1.0 KB");
        assert_eq!(format_size(1536), "1.5 KB");
    }

    #[test]
    fn test_format_size_mb() {
        assert_eq!(format_size(1_048_576), "1.0 MB");
    }

    #[test]
    fn test_format_size_gb() {
        assert_eq!(format_size(1_073_741_824), "1.0 GB");
    }

    #[test]
    fn test_format_duration_zero() {
        assert_eq!(format_duration(0), "00:00:00");
    }

    #[test]
    fn test_format_duration_full() {
        assert_eq!(format_duration(3661), "01:01:01");
    }

    #[test]
    fn test_format_duration_large() {
        assert_eq!(format_duration(86399), "23:59:59");
    }

    #[test]
    fn test_dos_time() {
        // 2026-06-15 14:30:00
        let dos = (46 << 25) | (6 << 21) | (15 << 16) | (14 << 11) | (30 << 5);
        let formatted = format_dos_time(dos);
        assert!(formatted.contains("2026"));
        assert!(formatted.contains("06") || formatted.contains("6"));
        assert!(formatted.contains("14:30:00"));
    }
}
