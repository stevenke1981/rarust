//! egui application state and UI rendering.

use egui::{CentralPanel, MenuBar, Panel, RichText, ScrollArea, Ui};
use rarust_core::archive::{OpenOptions, RarArchive};
use rarust_core::entry::Entry;
use rarust_core::error::RarustError;
use rarust_core::util;
use rarust_core::ArchiveFamily;

use super::fonts::FontSetup;
use super::i18n::{I18n, Locale, Message};

/// Main egui application for browsing RAR archives.
pub struct RarustApp {
    archive_path: String,
    i18n: I18n,
    font_setup: FontSetup,
    password: Option<String>,
    search: String,
    selected: Option<usize>,
    archive: Option<LoadedArchive>,
    load_error: Option<String>,
    status: String,
}

struct LoadedArchive {
    family_label: String,
    entries: Vec<Entry>,
}

impl RarustApp {
    /// Create a new GUI app for the given archive path.
    pub fn new(
        archive_path: impl Into<String>,
        locale: Locale,
        font_setup: FontSetup,
        password: Option<String>,
    ) -> Self {
        let i18n = I18n::new(locale);
        let mut app = Self {
            archive_path: archive_path.into(),
            i18n,
            font_setup,
            password,
            search: String::new(),
            selected: None,
            archive: None,
            load_error: None,
            status: String::new(),
        };
        app.reload_archive();
        app
    }

    fn reload_archive(&mut self) {
        self.status = self.i18n.t(Message::StatusLoading).to_owned();
        self.load_error = None;
        self.selected = None;

        let options = OpenOptions {
            password: self.password.clone(),
            ..OpenOptions::default()
        };

        match RarArchive::open_with_options(&self.archive_path, &options) {
            Ok(archive) => {
                let family_label = format_family(archive.family());
                match archive.list() {
                    Ok(entries) => {
                        self.archive = Some(LoadedArchive {
                            family_label,
                            entries,
                        });
                        self.status = self.i18n.t(Message::StatusReady).to_owned();
                    }
                    Err(e) => {
                        self.archive = None;
                        self.load_error = Some(e.to_string());
                        self.status = self.i18n.t(Message::StatusError).to_owned();
                    }
                }
            }
            Err(e) => {
                self.archive = None;
                self.load_error = Some(e.to_string());
                self.status = self.i18n.t(Message::StatusError).to_owned();
            }
        }
    }

    fn filtered_entries(&self) -> Vec<(usize, &Entry)> {
        let Some(loaded) = &self.archive else {
            return Vec::new();
        };
        let query = self.search.trim().to_ascii_lowercase();
        loaded
            .entries
            .iter()
            .enumerate()
            .filter(|(_, e)| query.is_empty() || e.name.to_ascii_lowercase().contains(&query))
            .collect()
    }

    fn show_menu_bar(&mut self, ui: &mut Ui) {
        Panel::top("menu_bar").show(ui, |ui| {
            MenuBar::new().ui(ui, |ui| {
                ui.menu_button(self.i18n.t(Message::Language), |ui| {
                    for locale in Locale::ALL {
                        let selected = self.i18n.locale() == locale;
                        if ui.selectable_label(selected, locale.display_name()).clicked() {
                            self.i18n.set_locale(locale);
                            self.status = self.i18n.t(Message::StatusReady).to_owned();
                            ui.close();
                        }
                    }
                });

                if ui.button(self.i18n.t(Message::Refresh)).clicked() {
                    self.reload_archive();
                }
            });
        });
    }

    fn show_font_warning(&self, ui: &mut Ui) {
        if self.font_setup.cjk_loaded || !self.i18n.locale().needs_cjk_font() {
            return;
        }
        egui::Window::new(self.i18n.t(Message::FontWarning))
            .collapsible(true)
            .resizable(false)
            .show(ui.ctx(), |ui| {
                ui.label(self.i18n.t(Message::FontWarningDetail));
            });
    }

    fn show_file_list(&mut self, ui: &mut Ui) {
        Panel::left("file_list")
            .default_size(360.0)
            .resizable(true)
            .show(ui, |ui| {
                ui.heading(self.i18n.t(Message::Entries));
                ui.add_space(4.0);
                ui.text_edit_singleline(&mut self.search);
                ui.label(
                    RichText::new(self.i18n.t(Message::SearchPlaceholder))
                        .weak()
                        .small(),
                );
                ui.separator();

                if let Some(err) = &self.load_error {
                    ui.colored_label(egui::Color32::RED, err);
                    return;
                }

                let filtered: Vec<(usize, String)> = self
                    .filtered_entries()
                    .into_iter()
                    .map(|(idx, entry)| {
                        let label = if entry.is_directory {
                            format!("📁 {}/", entry.name)
                        } else {
                            format!("📄 {}", entry.name)
                        };
                        (idx, label)
                    })
                    .collect();

                if filtered.is_empty() {
                    ui.label(self.i18n.t(Message::EmptyArchive));
                    return;
                }

                ScrollArea::vertical().auto_shrink([false, false]).show(ui, |ui| {
                    for (idx, label) in filtered {
                        let selected = self.selected == Some(idx);
                        if ui.selectable_label(selected, label).clicked() {
                            self.selected = Some(idx);
                        }
                    }
                });
            });
    }

    fn show_details(&mut self, ui: &mut Ui) {
        CentralPanel::default().show(ui, |ui| {
            if let Some(err) = &self.load_error {
                ui.colored_label(egui::Color32::RED, err);
                return;
            }

            let Some(loaded) = &self.archive else {
                ui.label(self.i18n.t(Message::StatusLoading));
                return;
            };

            ui.horizontal(|ui| {
                ui.label(RichText::new(self.i18n.t(Message::Archive)).strong());
                ui.label(&self.archive_path);
            });
            ui.horizontal(|ui| {
                ui.label(RichText::new(self.i18n.t(Message::Format)).strong());
                ui.label(&loaded.family_label);
                ui.separator();
                ui.label(format!(
                    "{} {}",
                    loaded.entries.len(),
                    self.i18n.t(Message::FilesCount)
                ));
            });
            ui.separator();

            let Some(idx) = self.selected else {
                ui.label(self.i18n.t(Message::SelectFile));
                return;
            };
            let Some(entry) = loaded.entries.get(idx) else {
                ui.label(self.i18n.t(Message::SelectFile));
                return;
            };

            detail_row(ui, self.i18n.t(Message::Name), &entry.name);
            detail_row(
                ui,
                self.i18n.t(Message::Size),
                &util::format_size(entry.size),
            );
            detail_row(ui, self.i18n.t(Message::Ratio), &format_ratio(entry.ratio));
            detail_row(
                ui,
                self.i18n.t(Message::Modified),
                &entry
                    .modified
                    .map(util::format_dos_time)
                    .unwrap_or_else(|| "-".to_string()),
            );
            detail_row(
                ui,
                self.i18n.t(Message::Crc32),
                &entry
                    .crc32
                    .map(|c| format!("{:08X}", c))
                    .unwrap_or_else(|| "-".to_string()),
            );
            detail_row(ui, self.i18n.t(Message::Method), &entry.method);
            detail_row(
                ui,
                self.i18n.t(Message::Directory),
                if entry.is_directory {
                    self.i18n.t(Message::Yes)
                } else {
                    self.i18n.t(Message::No)
                },
            );
            detail_row(
                ui,
                self.i18n.t(Message::Encrypted),
                if entry.is_encrypted {
                    self.i18n.t(Message::Yes)
                } else {
                    self.i18n.t(Message::No)
                },
            );

            ui.add_space(12.0);
            ui.horizontal(|ui| {
                if ui.button(self.i18n.t(Message::Extract)).clicked() {
                    self.status = format!(
                        "{}: {} (CLI: rarust extract …)",
                        self.i18n.t(Message::Extract),
                        entry.name
                    );
                }
                if ui.button(self.i18n.t(Message::Test)).clicked() {
                    self.status = format!(
                        "{}: {}",
                        self.i18n.t(Message::Test),
                        self.archive_path
                    );
                }
            });
        });
    }

    fn show_status_bar(&self, ui: &mut Ui) {
        Panel::bottom("status_bar").show(ui, |ui| {
            ui.horizontal(|ui| {
                ui.label(&self.status);
                if let Some(source) = &self.font_setup.cjk_source {
                    ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                        ui.label(RichText::new(format!("Font: {source}")).weak().small());
                    });
                }
            });
        });
    }
}

impl eframe::App for RarustApp {
    fn logic(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        let title = self.i18n.t(Message::AppTitle);
        ctx.send_viewport_cmd(egui::ViewportCommand::Title(title.to_owned()));
    }

    fn ui(&mut self, ui: &mut Ui, _frame: &mut eframe::Frame) {
        self.show_font_warning(ui);
        self.show_menu_bar(ui);
        self.show_file_list(ui);
        self.show_details(ui);
        self.show_status_bar(ui);
    }
}

fn detail_row(ui: &mut Ui, label: &str, value: &str) {
    ui.horizontal(|ui| {
        ui.label(RichText::new(label).strong());
        ui.label(value);
    });
}

fn format_family(family: ArchiveFamily) -> String {
    match family {
        ArchiveFamily::Rar50Plus => "RAR5".to_string(),
        ArchiveFamily::Rar15To40 => "RAR4".to_string(),
        ArchiveFamily::Rar13 => "RAR1.3".to_string(),
        _ => format!("{family:?}"),
    }
}

fn format_ratio(ratio: f64) -> String {
    if ratio > 0.0 {
        format!("{:.0}%", ratio * 100.0)
    } else {
        "-".to_string()
    }
}

/// Launch the native egui window for an archive.
pub fn run_gui(
    archive_path: &str,
    locale: Option<Locale>,
    password: Option<String>,
) -> Result<(), RarustError> {
    let locale = locale.unwrap_or_else(Locale::detect);

    let native_options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default()
            .with_inner_size([960.0, 640.0])
            .with_min_inner_size([640.0, 480.0]),
        ..Default::default()
    };

    let archive = archive_path.to_owned();
    let pwd = password;

    eframe::run_native(
        "rarust",
        native_options,
        Box::new(|cc| {
            let font_setup = super::fonts::setup_fonts(&cc.egui_ctx);
            Ok(Box::new(RarustApp::new(archive, locale, font_setup, pwd)))
        }),
    )
    .map_err(|e| RarustError::Unsupported(format!("GUI failed to start: {e}")))
}