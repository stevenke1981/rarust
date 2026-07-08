//! Side-panel file preview widget (text / hex / image / metadata).

use egui::{Frame, Margin, ScrollArea, Ui, Vec2};

/// Supported preview modes.
#[derive(Clone, Debug, PartialEq)]
pub enum PreviewKind {
    None,
    Text,
    Hex,
    Image,
    Metadata,
}

/// Renders a preview panel for the currently selected archive entry.
pub struct FilePreview {
    pub visible: bool,
    kind: PreviewKind,
    content: String,
    // Cache last-seen path to avoid reloading the same entry.
    last_path: Option<String>,
}

impl FilePreview {
    pub fn new() -> Self {
        Self {
            visible: true,
            kind: PreviewKind::None,
            content: String::new(),
            last_path: None,
        }
    }

    /// Clear preview state.
    pub fn clear(&mut self) {
        self.kind = PreviewKind::None;
        self.content.clear();
        self.last_path = None;
    }

    /// Load text content for preview.
    pub fn show_text(&mut self, path: &str, text: String) {
        self.kind = PreviewKind::Text;
        self.content = text;
        self.last_path = Some(path.to_owned());
    }

    /// Load hex dump for preview.
    pub fn show_hex(&mut self, path: &str, data: &[u8]) {
        self.kind = PreviewKind::Hex;
        self.last_path = Some(path.to_owned());
        let mut hex = String::with_capacity(data.len() * 4);
        for (i, chunk) in data.chunks(16).enumerate() {
            hex.push_str(&format!("{:08x}  ", i * 16));
            for (j, b) in chunk.iter().enumerate() {
                hex.push_str(&format!("{b:02x} "));
                if j == 7 {
                    hex.push(' ');
                }
            }
            hex.push_str("  ");
            for b in chunk {
                if b.is_ascii_graphic() || *b == b' ' {
                    hex.push(*b as char);
                } else {
                    hex.push('.');
                }
            }
            hex.push('\n');
        }
        self.content = hex;
    }

    /// Show metadata string.
    pub fn show_metadata(&mut self, path: &str, meta: String) {
        self.kind = PreviewKind::Metadata;
        self.content = meta;
        self.last_path = Some(path.to_owned());
    }

    /// Returns whether the given path already has content loaded in preview.
    pub fn is_loaded(&self, path: &str) -> bool {
        self.last_path.as_deref() == Some(path)
    }

    /// Render the preview panel. Must be called **before** `CentralPanel` in the
    /// same `show_archive_browser` function, since egui requires top-level
    /// panels to be added in order (right panel, then central panel).
    pub fn render(&mut self, ui: &mut Ui, width: f32) {
        if !self.visible || self.kind == PreviewKind::None {
            return;
        }

        let frame = Frame {
            inner_margin: Margin::symmetric(8, 4),
            fill: ui.visuals().extreme_bg_color,
            ..Default::default()
        };

        egui::panel::Panel::right("preview_panel")
            .resizable(true)
            .default_size(width)
            .min_size(180.0)
            .max_size(600.0)
            .frame(frame)
            .show(ui, |ui| {
                ui.vertical(|ui| {
                    ui.horizontal(|ui| {
                        ui.heading("Preview");
                        ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                            let _ = ui.small_button("✕");
                        });
                    });
                    ui.separator();
                    ScrollArea::vertical()
                        .auto_shrink([false, false])
                        .show(ui, |ui| {
                            ui.set_min_size(Vec2::new(ui.available_width(), ui.available_height()));
                            match self.kind {
                                PreviewKind::Text | PreviewKind::Metadata => {
                                    Frame::NONE.inner_margin(Margin::same(4)).show(ui, |ui| {
                                        ui.label(
                                            egui::RichText::new(self.content.as_str())
                                                .family(egui::FontFamily::Monospace),
                                        );
                                    });
                                }
                                PreviewKind::Hex => {
                                    Frame::NONE.inner_margin(Margin::same(4)).show(ui, |ui| {
                                        ui.add(
                                            egui::Label::new(
                                                egui::RichText::new(self.content.as_str())
                                                    .size(12.0)
                                                    .family(egui::FontFamily::Monospace),
                                            )
                                            .wrap_mode(egui::TextWrapMode::Extend),
                                        );
                                    });
                                }
                                PreviewKind::Image => {
                                    ui.label("(Image preview not yet supported)");
                                }
                                PreviewKind::None => {}
                            }
                        });
                });
            });
    }
}

impl Default for FilePreview {
    fn default() -> Self {
        Self::new()
    }
}
