//! Non-modal progress dialog for archive operations.

use egui::{Color32, RichText, Ui};

/// Describes the current operation being tracked.
#[derive(Clone, Debug, Default)]
pub struct ProgressState {
    pub title: String,
    pub status: String,
    pub progress: f64, // 0.0 .. 1.0
    pub indeterminate: bool,
    pub total_bytes: u64,
    pub processed_bytes: u64,
    pub speed_bytes_per_sec: f64,
}

/// Non-modal progress overlay window.
pub struct ProgressDialog {
    pub visible: bool,
    pub state: ProgressState,
    cancel_requested: bool,
}

impl ProgressDialog {
    pub fn new() -> Self {
        Self {
            visible: false,
            state: ProgressState::default(),
            cancel_requested: false,
        }
    }

    pub fn show(&mut self, title: &str) {
        self.state.title = title.to_owned();
        self.state.status.clear();
        self.state.progress = 0.0;
        self.state.indeterminate = true;
        self.state.processed_bytes = 0;
        self.state.total_bytes = 0;
        self.state.speed_bytes_per_sec = 0.0;
        self.cancel_requested = false;
        self.visible = true;
    }

    pub fn update(&mut self, f: impl FnOnce(&mut ProgressState)) {
        f(&mut self.state);
    }

    pub fn hide(&mut self) {
        self.visible = false;
    }

    pub fn was_cancelled(&self) -> bool {
        self.cancel_requested
    }

    pub fn render(&mut self, ctx: &egui::Context) {
        if !self.visible {
            return;
        }

        egui::Window::new(&self.state.title)
            .collapsible(false)
            .movable(false)
            .resizable(false)
            .anchor(egui::Align2::CENTER_CENTER, [0.0, 0.0])
            .show(ctx, |ui| {
                ui.set_min_width(420.0);

                ui.label(RichText::new(&self.state.status).strong());
                ui.add_space(6.0);

                if self.state.indeterminate {
                    ui.add(
                        egui::ProgressBar::new(0.5)
                            .animate(true)
                            .desired_width(380.0),
                    );
                } else {
                    let pct = self.state.progress.clamp(0.0, 1.0) as f32;
                    ui.add(
                        egui::ProgressBar::new(pct)
                            .desired_width(380.0)
                            .text(format!("{:.1}%", pct * 100.0)),
                    );
                }

                if self.state.total_bytes > 0 {
                    ui.add_space(4.0);
                    format_speed(
                        ui,
                        self.state.processed_bytes,
                        self.state.total_bytes,
                        self.state.speed_bytes_per_sec,
                    );
                }

                ui.add_space(8.0);
                ui.horizontal(|ui| {
                    ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                        if ui.button("Cancel").clicked() {
                            self.cancel_requested = true;
                        }
                    });
                });
            });
    }
}

impl Default for ProgressDialog {
    fn default() -> Self {
        Self::new()
    }
}

fn format_speed(ui: &mut Ui, processed: u64, total: u64, speed: f64) {
    let p = human_bytes(processed);
    let t = human_bytes(total);
    let s = if speed > 0.0 {
        format!("{}/s", human_bytes(speed as u64))
    } else {
        String::new()
    };
    let text = if !s.is_empty() {
        format!("{p} / {t}  —  {s}")
    } else {
        format!("{p} / {t}")
    };
    ui.label(RichText::new(text).size(12.0).color(Color32::GRAY));
}

fn human_bytes(b: u64) -> String {
    const UNITS: &[&str] = &["B", "KB", "MB", "GB", "TB"];
    let mut size = b as f64;
    let mut unit_idx = 0;
    while size >= 1024.0 && unit_idx < UNITS.len() - 1 {
        size /= 1024.0;
        unit_idx += 1;
    }
    if unit_idx == 0 {
        format!("{b} B")
    } else {
        format!("{size:.1} {}", UNITS[unit_idx])
    }
}
