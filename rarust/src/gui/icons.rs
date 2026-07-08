//! Embedded icon assets for the native GUI.

use std::collections::HashMap;

use egui::{ColorImage, Context, IconData, Image, TextureHandle, TextureOptions, Vec2};

const ICON_SIZE: usize = 32;
const APP_ICON_SIZE: u32 = 256;
const APP_ICON_RGBA: &[u8] = include_bytes!("../../assets/icons/app-256.rgba");

/// Functional icons used by the archive browser UI.
#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub enum UiIcon {
    Open,
    Create,
    Extract,
    Test,
    Refresh,
    Close,
    Password,
    Preview,
    Tab,
    File,
    Folder,
}

/// Lazily uploads embedded icon textures into egui.
#[derive(Default)]
pub struct IconCache {
    textures: HashMap<UiIcon, TextureHandle>,
}

impl IconCache {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn image(&mut self, ctx: &Context, icon: UiIcon) -> Image<'static> {
        let texture = self.textures.entry(icon).or_insert_with(|| {
            let image = ColorImage::from_rgba_unmultiplied([ICON_SIZE, ICON_SIZE], icon_rgba(icon));
            ctx.load_texture(icon_name(icon), image, TextureOptions::LINEAR)
        });

        Image::from_texture((texture.id(), Vec2::splat(18.0)))
    }
}

pub fn app_icon() -> IconData {
    IconData {
        rgba: APP_ICON_RGBA.to_vec(),
        width: APP_ICON_SIZE,
        height: APP_ICON_SIZE,
    }
}

fn icon_name(icon: UiIcon) -> &'static str {
    match icon {
        UiIcon::Open => "rarust-open",
        UiIcon::Create => "rarust-create",
        UiIcon::Extract => "rarust-extract",
        UiIcon::Test => "rarust-test",
        UiIcon::Refresh => "rarust-refresh",
        UiIcon::Close => "rarust-close",
        UiIcon::Password => "rarust-password",
        UiIcon::Preview => "rarust-preview",
        UiIcon::Tab => "rarust-tab",
        UiIcon::File => "rarust-file",
        UiIcon::Folder => "rarust-folder",
    }
}

fn icon_rgba(icon: UiIcon) -> &'static [u8] {
    match icon {
        UiIcon::Open => include_bytes!("../../assets/icons/open-32.rgba"),
        UiIcon::Create => include_bytes!("../../assets/icons/create-32.rgba"),
        UiIcon::Extract => include_bytes!("../../assets/icons/extract-32.rgba"),
        UiIcon::Test => include_bytes!("../../assets/icons/test-32.rgba"),
        UiIcon::Refresh => include_bytes!("../../assets/icons/refresh-32.rgba"),
        UiIcon::Close => include_bytes!("../../assets/icons/close-32.rgba"),
        UiIcon::Password => include_bytes!("../../assets/icons/password-32.rgba"),
        UiIcon::Preview => include_bytes!("../../assets/icons/preview-32.rgba"),
        UiIcon::Tab => include_bytes!("../../assets/icons/tab-32.rgba"),
        UiIcon::File => include_bytes!("../../assets/icons/file-32.rgba"),
        UiIcon::Folder => include_bytes!("../../assets/icons/folder-32.rgba"),
    }
}

#[cfg(test)]
mod tests {
    use super::{APP_ICON_RGBA, APP_ICON_SIZE, ICON_SIZE, UiIcon, icon_rgba};

    #[test]
    fn embedded_icon_assets_have_expected_rgba_sizes() {
        assert_eq!(
            APP_ICON_RGBA.len(),
            APP_ICON_SIZE as usize * APP_ICON_SIZE as usize * 4
        );

        for icon in [
            UiIcon::Open,
            UiIcon::Create,
            UiIcon::Extract,
            UiIcon::Test,
            UiIcon::Refresh,
            UiIcon::Close,
            UiIcon::Password,
            UiIcon::Preview,
            UiIcon::Tab,
            UiIcon::File,
            UiIcon::Folder,
        ] {
            assert_eq!(icon_rgba(icon).len(), ICON_SIZE * ICON_SIZE * 4);
        }
    }
}
