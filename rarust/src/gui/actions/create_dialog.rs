//! Archive creation dialog — select source files, compression method, and options.

/// Compression methods available for archive creation.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum CompressionMethod {
    Store,
    Fastest,
    Fast,
    Normal,
    Good,
    Best,
    Tqt,
}

impl CompressionMethod {
    pub const ALL: [CompressionMethod; 7] = [
        CompressionMethod::Store,
        CompressionMethod::Fastest,
        CompressionMethod::Fast,
        CompressionMethod::Normal,
        CompressionMethod::Good,
        CompressionMethod::Best,
        CompressionMethod::Tqt,
    ];

    pub fn name(&self) -> &'static str {
        match self {
            CompressionMethod::Store => "Store",
            CompressionMethod::Fastest => "Fastest",
            CompressionMethod::Fast => "Fast",
            CompressionMethod::Normal => "Normal",
            CompressionMethod::Good => "Good",
            CompressionMethod::Best => "Best",
            CompressionMethod::Tqt => "TQT",
        }
    }
}

/// State for the "Create Archive" dialog.
pub struct CreateDialog {
    pub visible: bool,
    pub archive_path: String,
    pub source_paths: Vec<String>,
    pub method: CompressionMethod,
    pub level: u8,
    pub split_mb: u64,
    pub password: String,
    pub confirm_password: String,
    pub encrypt_filenames: bool,
    pub error: Option<String>,
}

impl CreateDialog {
    pub fn new() -> Self {
        Self {
            visible: false,
            archive_path: String::new(),
            source_paths: Vec::new(),
            method: CompressionMethod::Normal,
            level: 5,
            split_mb: 0,
            password: String::new(),
            confirm_password: String::new(),
            encrypt_filenames: false,
            error: None,
        }
    }

    pub fn show(&mut self, ui: &egui::Ui) -> Option<CreateArchiveParams> {
        if !self.visible {
            return None;
        }

        let mut result = None;

        egui::Window::new("Create Archive")
            .collapsible(false)
            .movable(false)
            .resizable(false)
            .anchor(egui::Align2::CENTER_CENTER, [0.0, 0.0])
            .show(ui.ctx(), |ui| {
                ui.set_min_width(480.0);

                // Archive path
                ui.horizontal(|ui| {
                    ui.label("Archive:");
                    ui.add(
                        egui::TextEdit::singleline(&mut self.archive_path)
                            .hint_text("path/to/archive.rar")
                            .desired_width(300.0),
                    );
                    if ui.button("Browse…").clicked() {
                        if let Some(path) = rfd::FileDialog::new()
                            .set_title("Save Archive As")
                            .add_filter("RAR", &["rar"])
                            .add_filter("ZIP", &["zip"])
                            .add_filter("TAR", &["tar", "tar.gz", "tgz"])
                            .save_file()
                        {
                            self.archive_path = path.display().to_string();
                        }
                    }
                });

                ui.add_space(6.0);

                // Source files
                ui.horizontal(|ui| {
                    ui.label("Sources:");
                    if ui.button("Add Files…").clicked() {
                        if let Some(files) = rfd::FileDialog::new()
                            .set_title("Select Files to Add")
                            .pick_files()
                        {
                            for f in files {
                                self.source_paths.push(f.display().to_string());
                            }
                        }
                    }
                    if ui.button("Add Folder…").clicked() {
                        if let Some(dir) = rfd::FileDialog::new()
                            .set_title("Select Folder")
                            .pick_folder()
                        {
                            self.source_paths.push(dir.display().to_string());
                        }
                    }
                });

                if !self.source_paths.is_empty() {
                    ui.add_space(4.0);
                    let mut remove_idx = None;
                    for (i, path) in self.source_paths.iter().enumerate() {
                        ui.horizontal(|ui| {
                            ui.label(
                                egui::RichText::new(format!("  {i}.")).color(egui::Color32::GRAY),
                            );
                            ui.label(
                                egui::RichText::new(truncate_path(
                                    path,
                                    ui.available_width() - 80.0,
                                ))
                                .weak(),
                            );
                            if ui.small_button("✕").clicked() {
                                remove_idx = Some(i);
                            }
                        });
                    }
                    if let Some(idx) = remove_idx {
                        self.source_paths.remove(idx);
                    }
                }

                ui.add_space(8.0);
                ui.separator();
                ui.add_space(4.0);

                // Compression method
                ui.horizontal(|ui| {
                    ui.label("Method:");
                    egui::ComboBox::from_id_salt("create_method")
                        .selected_text(self.method.name())
                        .show_ui(ui, |ui| {
                            for m in CompressionMethod::ALL {
                                ui.selectable_value(&mut self.method, m, m.name());
                            }
                        });
                });

                // Compression level
                ui.horizontal(|ui| {
                    ui.label("Level:");
                    ui.add(egui::Slider::new(&mut self.level, 0..=9).show_value(false));
                    ui.label(format!("{} / 9", self.level));
                });

                // Split volumes
                ui.horizontal(|ui| {
                    ui.label("Split:");
                    ui.add(egui::Slider::new(&mut self.split_mb, 0..=65536).show_value(false));
                    ui.label(if self.split_mb == 0 {
                        "None".to_string()
                    } else {
                        format!("{} MB", self.split_mb)
                    });
                });

                // Password
                ui.horizontal(|ui| {
                    ui.label("Password:");
                    ui.add(
                        egui::TextEdit::singleline(&mut self.password)
                            .password(true)
                            .hint_text("optional")
                            .desired_width(150.0),
                    );
                    if !self.password.is_empty() {
                        ui.label("Confirm:");
                        ui.add(
                            egui::TextEdit::singleline(&mut self.confirm_password)
                                .password(true)
                                .hint_text("retype")
                                .desired_width(150.0),
                        );
                    }
                });

                ui.checkbox(&mut self.encrypt_filenames, "Encrypt filenames");

                if let Some(err) = &self.error {
                    ui.colored_label(egui::Color32::from_rgb(224, 80, 64), err);
                }

                ui.add_space(8.0);
                ui.horizontal(|ui| {
                    if ui.button("Cancel").clicked() {
                        self.visible = false;
                    }
                    if ui.button("Create").clicked() {
                        let validation = self.validate();
                        match validation {
                            Ok(params) => {
                                result = Some(params);
                                self.visible = false;
                            }
                            Err(err) => {
                                self.error = Some(err);
                            }
                        }
                    }
                });
            });

        result
    }

    fn validate(&self) -> Result<CreateArchiveParams, String> {
        if self.archive_path.trim().is_empty() {
            return Err("Archive path is required".to_string());
        }
        if self.source_paths.is_empty() {
            return Err("No source files selected".to_string());
        }
        if !self.password.is_empty() && self.password != self.confirm_password {
            return Err("Passwords do not match".to_string());
        }
        Ok(CreateArchiveParams {
            archive_path: self.archive_path.trim().to_owned(),
            source_paths: self.source_paths.clone(),
            password: if self.password.is_empty() {
                None
            } else {
                Some(self.password.clone())
            },
            encrypt_filenames: self.encrypt_filenames,
            split_mb: self.split_mb,
            method: self.method,
        })
    }
}

impl Default for CreateDialog {
    fn default() -> Self {
        Self::new()
    }
}

/// Parameters returned when the user confirms archive creation.
pub struct CreateArchiveParams {
    pub archive_path: String,
    pub source_paths: Vec<String>,
    pub password: Option<String>,
    pub encrypt_filenames: bool,
    pub split_mb: u64,
    pub method: CompressionMethod,
}

fn truncate_path(path: &str, max_width: f32) -> String {
    // Rough heuristic: ~7 chars per em-width at 13px
    let max_chars = (max_width / 7.5) as usize;
    if path.chars().count() <= max_chars || max_chars < 20 {
        return path.to_owned();
    }
    let keep_front = max_chars.saturating_sub(3) * 2 / 3;
    let keep_back = max_chars.saturating_sub(3) - keep_front;
    let front: String = path.chars().take(keep_front).collect();
    let back: String = path
        .chars()
        .rev()
        .take(keep_back)
        .collect::<String>()
        .chars()
        .rev()
        .collect();
    format!("{front}...{back}")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn validate_rejects_empty_archive_path() {
        let dlg = CreateDialog::new();
        let result = dlg.validate();
        assert!(result.is_err());
    }

    #[test]
    fn validate_rejects_no_source_files() {
        let mut dlg = CreateDialog::new();
        dlg.archive_path = "/tmp/test.rar".to_string();
        let result = dlg.validate();
        assert!(result.is_err());
    }

    #[test]
    fn validate_rejects_mismatched_passwords() {
        let mut dlg = CreateDialog::new();
        dlg.archive_path = "/tmp/test.rar".to_string();
        dlg.source_paths.push("/tmp/file.txt".to_string());
        dlg.password = "secret".to_string();
        dlg.confirm_password = "different".to_string();
        let result = dlg.validate();
        assert!(result.is_err());
    }

    #[test]
    fn validate_succeeds_with_valid_input() {
        let mut dlg = CreateDialog::new();
        dlg.archive_path = "/tmp/test.rar".to_string();
        dlg.source_paths.push("/tmp/file.txt".to_string());
        let result = dlg.validate();
        assert!(result.is_ok());
        let params = result.unwrap();
        assert_eq!(params.archive_path, "/tmp/test.rar");
        assert_eq!(params.source_paths.len(), 1);
        assert!(params.password.is_none());
    }

    #[test]
    fn compression_method_names() {
        assert_eq!(CompressionMethod::Store.name(), "Store");
        assert_eq!(CompressionMethod::Normal.name(), "Normal");
        assert_eq!(CompressionMethod::Best.name(), "Best");
    }

    #[test]
    fn default_dialog_is_hidden() {
        let dlg = CreateDialog::new();
        assert!(!dlg.visible);
        assert!(dlg.archive_path.is_empty());
        assert_eq!(dlg.method, CompressionMethod::Normal);
    }
}
