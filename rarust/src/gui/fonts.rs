//! CJK font loading for egui.
//!
//! egui's bundled default font only covers Latin glyphs. For Chinese (and other
//! CJK scripts) we prepend a system or user-specified font to the Proportional
//! and Monospace families.

use std::path::Path;

use egui::{Context, FontData, FontDefinitions, FontFamily};

const CJK_FONT_KEY: &str = "rarust_cjk";

/// Result of installing fonts into an egui context.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct FontSetup {
    /// Whether a CJK-capable font was successfully loaded.
    pub cjk_loaded: bool,
    /// Path or source description used for the CJK font, if any.
    pub cjk_source: Option<String>,
}

/// Install fonts into `ctx`, prioritizing CJK coverage for Chinese UI text.
pub fn setup_fonts(ctx: &Context) -> FontSetup {
    let mut fonts = FontDefinitions::default();

    let (cjk_bytes, cjk_source) = load_cjk_font();
    let cjk_loaded = if let Some(bytes) = cjk_bytes {
        fonts
            .font_data
            .insert(CJK_FONT_KEY.to_owned(), FontData::from_owned(bytes).into());
        prepend_family(&mut fonts, FontFamily::Proportional, CJK_FONT_KEY);
        prepend_family(&mut fonts, FontFamily::Monospace, CJK_FONT_KEY);
        true
    } else {
        false
    };

    ctx.set_fonts(fonts);

    FontSetup {
        cjk_loaded,
        cjk_source,
    }
}

fn prepend_family(fonts: &mut FontDefinitions, family: FontFamily, font_key: &str) {
    fonts
        .families
        .entry(family)
        .or_default()
        .insert(0, font_key.to_owned());
}

/// Load CJK font bytes from `RARUST_FONT` or known system font paths.
fn load_cjk_font() -> (Option<Vec<u8>>, Option<String>) {
    if let Ok(path) = std::env::var("RARUST_FONT")
        && let Some(bytes) = read_font_file(Path::new(&path))
    {
        return (Some(bytes), Some(path));
    }

    for path in system_cjk_font_paths() {
        if let Some(bytes) = read_font_file(path) {
            return (Some(bytes), Some(path.display().to_string()));
        }
    }

    (None, None)
}

fn read_font_file(path: &Path) -> Option<Vec<u8>> {
    if !path.is_file() {
        return None;
    }
    std::fs::read(path).ok().filter(|data| !data.is_empty())
}

/// Platform-specific CJK font search paths (most common first).
fn system_cjk_font_paths() -> Vec<&'static Path> {
    #[cfg(windows)]
    {
        vec![
            Path::new(r"C:\Windows\Fonts\msyh.ttc"), // Microsoft YaHei
            Path::new(r"C:\Windows\Fonts\msyhbd.ttc"),
            Path::new(r"C:\Windows\Fonts\simhei.ttf"), // SimHei
            Path::new(r"C:\Windows\Fonts\simsun.ttc"), // SimSun
            Path::new(r"C:\Windows\Fonts\mingliu.ttc"), // MingLiU (Traditional)
        ]
    }

    #[cfg(target_os = "macos")]
    {
        vec![
            Path::new("/System/Library/Fonts/PingFang.ttc"),
            Path::new("/System/Library/Fonts/STHeiti Light.ttc"),
            Path::new("/System/Library/Fonts/Supplemental/Songti.ttc"),
            Path::new("/Library/Fonts/Arial Unicode.ttf"),
        ]
    }

    #[cfg(target_os = "linux")]
    {
        vec![
            Path::new("/usr/share/fonts/opentype/noto/NotoSansCJK-Regular.ttc"),
            Path::new("/usr/share/fonts/noto-cjk/NotoSansCJK-Regular.ttc"),
            Path::new("/usr/share/fonts/truetype/noto/NotoSansCJK-Regular.ttc"),
            Path::new("/usr/share/fonts/wenquanyi/wqy-microhei/wqy-microhei.ttc"),
            Path::new("/usr/share/fonts/truetype/wqy/wqy-microhei.ttc"),
        ]
    }

    #[cfg(not(any(windows, target_os = "macos", target_os = "linux")))]
    {
        vec![]
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn system_paths_are_non_empty_on_supported_platforms() {
        #[cfg(any(windows, target_os = "macos", target_os = "linux"))]
        assert!(!system_cjk_font_paths().is_empty());
    }

    #[test]
    fn read_font_file_returns_none_for_missing_path() {
        assert!(read_font_file(Path::new("/nonexistent/rarust-font-test.ttf")).is_none());
    }
}
