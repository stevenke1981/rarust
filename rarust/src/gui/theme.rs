//! Theme system for rarust-gui — Light and Dark color palettes.
//!
//! Provides a [`Theme`] struct with all UI color tokens and an `apply()`
//! method that rewrites egui's [`Style`] for consistent theming.

use egui::{Color32, CornerRadius, FontId, Margin, Stroke, TextStyle, Vec2, Visuals};

/// Complete color palette for one theme variant.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Theme {
    pub name: &'static str,
    pub window: Color32,
    pub chrome: Color32,
    pub chrome_dark: Color32,
    pub list_bg: Color32,
    pub header_bg: Color32,
    pub row_alt: Color32,
    pub border: Color32,
    pub text: Color32,
    pub muted: Color32,
    pub accent: Color32,
    pub selected: Color32,
    pub success: Color32,
    pub warning: Color32,
    pub danger: Color32,
}

impl Theme {
    /// Light theme — clean, professional palette (default).
    pub const LIGHT: Theme = Theme {
        name: "Light",
        window: Color32::from_rgb(242, 244, 247),
        chrome: Color32::from_rgb(232, 236, 241),
        chrome_dark: Color32::from_rgb(216, 222, 230),
        list_bg: Color32::from_rgb(255, 255, 255),
        header_bg: Color32::from_rgb(226, 231, 238),
        row_alt: Color32::from_rgb(248, 250, 252),
        border: Color32::from_rgb(177, 187, 199),
        text: Color32::from_rgb(26, 30, 36),
        muted: Color32::from_rgb(83, 93, 107),
        accent: Color32::from_rgb(0, 103, 192),
        selected: Color32::from_rgb(214, 233, 255),
        success: Color32::from_rgb(38, 128, 67),
        warning: Color32::from_rgb(154, 103, 0),
        danger: Color32::from_rgb(184, 40, 31),
    };

    /// Dark theme — reduced eye strain, modern look.
    pub const DARK: Theme = Theme {
        name: "Dark",
        window: Color32::from_rgb(30, 30, 46),
        chrome: Color32::from_rgb(43, 43, 61),
        chrome_dark: Color32::from_rgb(36, 36, 54),
        list_bg: Color32::from_rgb(37, 37, 64),
        header_bg: Color32::from_rgb(54, 54, 80),
        row_alt: Color32::from_rgb(46, 46, 72),
        border: Color32::from_rgb(68, 68, 102),
        text: Color32::from_rgb(224, 224, 240),
        muted: Color32::from_rgb(144, 144, 176),
        accent: Color32::from_rgb(112, 160, 255),
        selected: Color32::from_rgb(58, 58, 106),
        success: Color32::from_rgb(60, 200, 100),
        warning: Color32::from_rgb(230, 180, 60),
        danger: Color32::from_rgb(224, 80, 64),
    };

    /// Apply this theme to the egui context, rewriting all style tokens.
    pub fn apply(&self, ctx: &egui::Context) {
        ctx.set_theme(egui::Theme::Light);
        let mut style = (*ctx.style_of(egui::Theme::Light)).clone();
        style.visuals = Visuals::light();
        style.visuals.override_text_color = Some(self.text);
        style.visuals.weak_text_color = Some(self.muted);
        style.visuals.panel_fill = self.chrome;
        style.visuals.window_fill = self.window;
        style.visuals.faint_bg_color = self.row_alt;
        style.visuals.extreme_bg_color = self.list_bg;
        style.visuals.text_edit_bg_color = Some(self.list_bg);
        style.visuals.window_stroke = Stroke::new(1.0, self.border);
        style.visuals.window_corner_radius = CornerRadius::same(2);
        style.visuals.menu_corner_radius = CornerRadius::same(2);
        style.visuals.selection.bg_fill = self.selected;
        style.visuals.selection.stroke = Stroke::new(1.0, self.accent);
        style.visuals.hyperlink_color = self.accent;
        style.visuals.warn_fg_color = self.warning;
        style.visuals.error_fg_color = self.danger;
        style.visuals.button_frame = true;
        style.visuals.interact_cursor = Some(egui::CursorIcon::PointingHand);

        for visuals in [
            &mut style.visuals.widgets.noninteractive,
            &mut style.visuals.widgets.inactive,
            &mut style.visuals.widgets.hovered,
            &mut style.visuals.widgets.active,
            &mut style.visuals.widgets.open,
        ] {
            visuals.corner_radius = CornerRadius::same(2);
            visuals.bg_stroke = Stroke::new(1.0, self.border);
            visuals.fg_stroke = Stroke::new(1.0, self.text);
        }
        style.visuals.widgets.noninteractive.bg_fill = self.chrome;
        style.visuals.widgets.noninteractive.weak_bg_fill = self.chrome;
        style.visuals.widgets.inactive.bg_fill = Color32::from_rgb(244, 247, 250);
        style.visuals.widgets.inactive.weak_bg_fill = Color32::from_rgb(244, 247, 250);
        style.visuals.widgets.hovered.bg_fill = Color32::from_rgb(228, 240, 252);
        style.visuals.widgets.hovered.weak_bg_fill = Color32::from_rgb(228, 240, 252);
        style.visuals.widgets.hovered.bg_stroke = Stroke::new(1.0, self.accent);
        style.visuals.widgets.active.bg_fill = self.selected;
        style.visuals.widgets.active.weak_bg_fill = self.selected;
        style.visuals.widgets.active.bg_stroke = Stroke::new(1.0, self.accent);

        style.spacing.item_spacing = Vec2::new(6.0, 4.0);
        style.spacing.button_padding = Vec2::new(10.0, 5.0);
        style.spacing.interact_size = Vec2::new(32.0, 26.0);
        style.spacing.window_margin = Margin::same(8);
        style.spacing.menu_margin = Margin::symmetric(8, 6);
        style
            .text_styles
            .insert(TextStyle::Heading, FontId::proportional(18.0));
        style
            .text_styles
            .insert(TextStyle::Body, FontId::proportional(13.0));
        style
            .text_styles
            .insert(TextStyle::Small, FontId::proportional(12.0));

        ctx.set_style_of(egui::Theme::Light, style);
    }

    /// Load saved theme name from config dir. Returns `None` on first run.
    pub fn load_saved() -> Option<String> {
        let mut path = dirs::config_dir()?;
        path.push("rarust");
        path.push("gui-theme");
        std::fs::read_to_string(path)
            .ok()
            .map(|s| s.trim().to_owned())
    }

    /// Persist theme choice to config dir.
    pub fn save(name: &str) {
        if let Some(mut path) = dirs::config_dir() {
            path.push("rarust");
            let _ = std::fs::create_dir_all(&path);
            path.push("gui-theme");
            let _ = std::fs::write(path, name);
        }
    }
}
