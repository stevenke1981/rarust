//! egui application state and UI rendering.

use egui::{
    Button, CentralPanel, Color32, CornerRadius, FontId, Frame, Grid, Label, Margin, Panel,
    RichText, ScrollArea, Stroke, TextEdit, TextStyle, Ui, Vec2, Visuals,
};
use rarust_core::archive::{OpenOptions, PortableArchive};
use rarust_core::entry::Entry;
use rarust_core::error::RarustError;
use rarust_core::util;

use super::fonts::FontSetup;
use super::i18n::{I18n, Locale, Message};

/// Main egui application for browsing RAR archives.
pub struct RarustApp {
    archive_path: Option<String>,
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

const BG: Color32 = Color32::from_rgb(12, 15, 18);
const PANEL: Color32 = Color32::from_rgb(18, 22, 27);
const PANEL_ALT: Color32 = Color32::from_rgb(24, 29, 35);
const SURFACE: Color32 = Color32::from_rgb(30, 36, 43);
const BORDER: Color32 = Color32::from_rgb(50, 59, 70);
const TEXT: Color32 = Color32::from_rgb(236, 239, 243);
const MUTED: Color32 = Color32::from_rgb(155, 166, 179);
const ACCENT: Color32 = Color32::from_rgb(69, 145, 214);
const ACCENT_SOFT: Color32 = Color32::from_rgb(34, 67, 96);
const SUCCESS: Color32 = Color32::from_rgb(91, 189, 126);
const WARNING: Color32 = Color32::from_rgb(232, 174, 83);
const DANGER: Color32 = Color32::from_rgb(232, 106, 99);

impl RarustApp {
    /// Create a new GUI app, optionally bound to an archive path.
    pub fn new(
        archive_path: Option<String>,
        locale: Locale,
        font_setup: FontSetup,
        password: Option<String>,
    ) -> Self {
        let i18n = I18n::new(locale);
        let has_archive = archive_path.is_some();
        let mut app = Self {
            archive_path,
            i18n,
            font_setup,
            password,
            search: String::new(),
            selected: None,
            archive: None,
            load_error: None,
            status: if has_archive {
                String::new()
            } else {
                I18n::new(locale).t(Message::StatusReady).to_owned()
            },
        };
        if has_archive {
            app.reload_archive();
        }
        app
    }

    fn pick_archive_file(&mut self) {
        let dialog =
            rfd::FileDialog::new().add_filter("Archive", &["rar", "zip", "tar", "gz", "tgz"]);
        if let Some(path) = dialog.pick_file() {
            self.open_archive(path.display().to_string());
        }
    }

    fn open_archive(&mut self, path: String) {
        self.archive_path = Some(path);
        self.reload_archive();
    }

    fn reload_archive(&mut self) {
        let Some(path) = self.archive_path.as_deref() else {
            self.archive = None;
            self.load_error = None;
            self.selected = None;
            self.status = self.i18n.t(Message::StatusReady).to_owned();
            return;
        };

        self.status = self.i18n.t(Message::StatusLoading).to_owned();
        self.load_error = None;
        self.selected = None;

        let options = OpenOptions {
            password: self.password.clone(),
            ..OpenOptions::default()
        };

        match PortableArchive::open_with_options(path, &options) {
            Ok(archive) => {
                let family_label = archive.format_name().to_string();
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

    fn extract_selected(&mut self, entry_name: String, is_directory: bool) {
        let Some(archive_path) = self.archive_path.clone() else {
            return;
        };
        let Some(dest) = rfd::FileDialog::new().pick_folder() else {
            return;
        };

        let options = OpenOptions {
            password: self.password.clone(),
            ..OpenOptions::default()
        };
        let result =
            PortableArchive::open_with_options(&archive_path, &options).and_then(|archive| {
                let prefix = if is_directory && !entry_name.ends_with('/') {
                    format!("{entry_name}/")
                } else {
                    entry_name.clone()
                };
                archive.extract_with_filter(&dest, |entry| {
                    if is_directory {
                        entry.name == entry_name || entry.name.starts_with(&prefix)
                    } else {
                        entry.name == entry_name
                    }
                })
            });

        match result {
            Ok(summary) => {
                self.status = format!(
                    "Extracted {} entries to {}",
                    summary.extracted,
                    dest.display()
                );
            }
            Err(error) => {
                self.status = format!("{}: {}", self.i18n.t(Message::StatusError), error);
            }
        }
    }

    fn test_archive(&mut self) {
        let Some(archive_path) = self.archive_path.clone() else {
            return;
        };
        let options = OpenOptions {
            password: self.password.clone(),
            ..OpenOptions::default()
        };
        match PortableArchive::open_with_options(&archive_path, &options)
            .and_then(|archive| archive.test_all())
        {
            Ok(summary) => {
                self.status = format!(
                    "Test completed: {} entries OK, {} failed",
                    summary.tested, summary.failed
                );
            }
            Err(error) => {
                self.status = format!("{}: {}", self.i18n.t(Message::StatusError), error);
            }
        }
    }

    fn show_menu_bar(&mut self, ui: &mut Ui) {
        Panel::top("menu_bar")
            .frame(
                Frame::NONE
                    .fill(PANEL)
                    .inner_margin(Margin::symmetric(16, 10)),
            )
            .show(ui, |ui| {
                ui.horizontal(|ui| {
                    ui.label(
                        RichText::new(self.i18n.t(Message::AppTitle))
                            .heading()
                            .strong()
                            .color(TEXT),
                    );
                    ui.add_space(12.0);
                    status_pill(ui, &self.status, status_color(&self.status, &self.i18n));

                    ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                        if ui
                            .add(secondary_button(self.i18n.t(Message::Refresh)))
                            .clicked()
                        {
                            self.reload_archive();
                        }

                        ui.menu_button(self.i18n.t(Message::Language), |ui| {
                            for locale in Locale::ALL {
                                let selected = self.i18n.locale() == locale;
                                if ui
                                    .selectable_label(selected, locale.display_name())
                                    .clicked()
                                {
                                    self.i18n.set_locale(locale);
                                    self.status = self.i18n.t(Message::StatusReady).to_owned();
                                    ui.close();
                                }
                            }
                        });

                        if ui
                            .add(primary_button(self.i18n.t(Message::OpenArchive)))
                            .clicked()
                        {
                            self.pick_archive_file();
                        }
                    });
                });
            });
    }

    fn show_font_warning(&self, ui: &mut Ui) {
        if self.font_setup.cjk_loaded || !self.i18n.locale().needs_cjk_font() {
            return;
        }
        Panel::top("font_warning")
            .frame(
                Frame::NONE
                    .fill(Color32::from_rgb(52, 39, 18))
                    .inner_margin(Margin::symmetric(16, 8)),
            )
            .show(ui, |ui| {
                ui.colored_label(
                    WARNING,
                    format!(
                        "{}: {}",
                        self.i18n.t(Message::FontWarning),
                        self.i18n.t(Message::FontWarningDetail)
                    ),
                );
            });
    }

    fn show_file_list(&mut self, ui: &mut Ui) {
        Panel::left("file_list")
            .default_size(360.0)
            .min_size(280.0)
            .resizable(true)
            .frame(
                Frame::NONE
                    .fill(PANEL_ALT)
                    .inner_margin(Margin::symmetric(14, 14)),
            )
            .show(ui, |ui| {
                ui.label(
                    RichText::new(self.i18n.t(Message::Entries))
                        .strong()
                        .color(TEXT),
                );
                ui.add_space(8.0);
                ui.add(
                    TextEdit::singleline(&mut self.search)
                        .hint_text(self.i18n.t(Message::SearchPlaceholder))
                        .desired_width(f32::INFINITY),
                );
                ui.add_space(12.0);

                if self.archive_path.is_none() {
                    empty_panel(
                        ui,
                        self.i18n.t(Message::Welcome),
                        self.i18n.t(Message::WelcomeDetail),
                    );
                    ui.add_space(12.0);
                    if ui
                        .add_sized(
                            [ui.available_width(), 36.0],
                            primary_button(self.i18n.t(Message::OpenArchive)),
                        )
                        .clicked()
                    {
                        self.pick_archive_file();
                    }
                    return;
                }

                if let Some(err) = &self.load_error {
                    error_panel(ui, self.i18n.t(Message::StatusError), err);
                    return;
                }

                if let Some(loaded) = &self.archive {
                    let total_size = loaded.entries.iter().map(|entry| entry.size).sum::<u64>();
                    stat_strip(
                        ui,
                        &loaded.entries.len().to_string(),
                        self.i18n.t(Message::FilesCount),
                        self.i18n.t(Message::TotalSize),
                        &util::format_size(total_size),
                    );
                    ui.add_space(12.0);
                }

                let filtered: Vec<(usize, String, bool)> = self
                    .filtered_entries()
                    .into_iter()
                    .map(|(idx, entry)| {
                        let label = if entry.is_directory {
                            format!("[DIR] {}/", entry.name)
                        } else {
                            format!("[FILE] {}", entry.name)
                        };
                        (idx, label, entry.is_directory)
                    })
                    .collect();

                if filtered.is_empty() {
                    empty_panel(
                        ui,
                        self.i18n.t(Message::EmptyArchive),
                        self.i18n.t(Message::SearchPlaceholder),
                    );
                    return;
                }

                ScrollArea::vertical()
                    .id_salt("archive_entry_list")
                    .auto_shrink([false, false])
                    .show(ui, |ui| {
                        for (idx, label, is_directory) in filtered {
                            let selected = self.selected == Some(idx);
                            let text_color = if selected {
                                TEXT
                            } else if is_directory {
                                Color32::from_rgb(216, 191, 115)
                            } else {
                                TEXT
                            };
                            let response = ui.add_sized(
                                [ui.available_width(), 28.0],
                                Button::selectable(
                                    selected,
                                    RichText::new(label).color(text_color).monospace(),
                                ),
                            );
                            if response.clicked() {
                                self.selected = Some(idx);
                            }
                        }
                    });
            });
    }

    fn show_details(&mut self, ui: &mut Ui) {
        CentralPanel::default()
            .frame(Frame::NONE.fill(BG).inner_margin(Margin::symmetric(24, 20)))
            .show(ui, |ui| {
                if self.archive_path.is_none() {
                    self.show_welcome_state(ui);
                    return;
                }

                if let Some(err) = &self.load_error {
                    error_panel(ui, self.i18n.t(Message::StatusError), err);
                    ui.add_space(12.0);
                    if ui
                        .add(primary_button(self.i18n.t(Message::OpenArchive)))
                        .clicked()
                    {
                        self.pick_archive_file();
                    }
                    return;
                }

                let Some(loaded) = &self.archive else {
                    loading_panel(ui, self.i18n.t(Message::StatusLoading));
                    return;
                };

                let archive_path = self.archive_path.as_deref().unwrap_or("-").to_owned();
                let entry_count = loaded.entries.len();
                let total_size = loaded.entries.iter().map(|entry| entry.size).sum::<u64>();
                let family_label = loaded.family_label.clone();

                Frame::NONE
                    .fill(PANEL)
                    .stroke(Stroke::new(1.0, BORDER))
                    .corner_radius(CornerRadius::same(8))
                    .inner_margin(Margin::symmetric(18, 16))
                    .show(ui, |ui| {
                        ui.label(
                            RichText::new(self.i18n.t(Message::Archive))
                                .small()
                                .color(MUTED),
                        );
                        ui.add(
                            Label::new(RichText::new(archive_path).strong().color(TEXT)).truncate(),
                        );
                        ui.add_space(12.0);
                        Grid::new("archive_summary_grid")
                            .num_columns(3)
                            .spacing([24.0, 6.0])
                            .show(ui, |ui| {
                                metric(ui, self.i18n.t(Message::Format), &family_label);
                                metric(
                                    ui,
                                    self.i18n.t(Message::Entries),
                                    &format!("{entry_count} {}", self.i18n.t(Message::FilesCount)),
                                );
                                metric(
                                    ui,
                                    self.i18n.t(Message::Size),
                                    &util::format_size(total_size),
                                );
                                ui.end_row();
                            });
                    });

                ui.add_space(16.0);

                let Some(idx) = self.selected else {
                    empty_panel(
                        ui,
                        self.i18n.t(Message::SelectFile),
                        self.i18n.t(Message::SelectFileDetail),
                    );
                    return;
                };
                let Some(entry) = loaded.entries.get(idx) else {
                    empty_panel(
                        ui,
                        self.i18n.t(Message::SelectFile),
                        self.i18n.t(Message::SelectFileDetail),
                    );
                    return;
                };

                let selected_entry_name = entry.name.clone();
                let selected_entry_is_directory = entry.is_directory;
                let selected_entry_is_encrypted = entry.is_encrypted;
                let modified = entry
                    .modified
                    .map(util::format_dos_time)
                    .unwrap_or_else(|| "-".to_string());
                let crc32 = entry
                    .crc32
                    .map(|c| format!("{:08X}", c))
                    .unwrap_or_else(|| "-".to_string());
                let size = util::format_size(entry.size);
                let ratio = format_ratio(entry.ratio);
                let method = entry.method.clone();
                let directory = if entry.is_directory {
                    self.i18n.t(Message::Yes)
                } else {
                    self.i18n.t(Message::No)
                };
                let encrypted = if entry.is_encrypted {
                    self.i18n.t(Message::Yes)
                } else {
                    self.i18n.t(Message::No)
                };

                Frame::NONE
                    .fill(PANEL)
                    .stroke(Stroke::new(1.0, BORDER))
                    .corner_radius(CornerRadius::same(8))
                    .inner_margin(Margin::symmetric(18, 16))
                    .show(ui, |ui| {
                        ui.horizontal(|ui| {
                            ui.label(
                                RichText::new(self.i18n.t(Message::Name))
                                    .small()
                                    .color(MUTED),
                            );
                            if selected_entry_is_directory {
                                status_pill(ui, self.i18n.t(Message::Directory), WARNING);
                            }
                            if selected_entry_is_encrypted {
                                status_pill(ui, self.i18n.t(Message::Encrypted), ACCENT);
                            }
                        });
                        ui.add(
                            Label::new(
                                RichText::new(&selected_entry_name)
                                    .heading()
                                    .strong()
                                    .color(TEXT),
                            )
                            .truncate(),
                        );
                        ui.add_space(16.0);

                        Grid::new("selected_entry_details")
                            .num_columns(2)
                            .spacing([24.0, 10.0])
                            .striped(true)
                            .show(ui, |ui| {
                                detail_row(ui, self.i18n.t(Message::Size), &size);
                                detail_row(ui, self.i18n.t(Message::Ratio), &ratio);
                                detail_row(ui, self.i18n.t(Message::Modified), &modified);
                                detail_row(ui, self.i18n.t(Message::Crc32), &crc32);
                                detail_row(ui, self.i18n.t(Message::Method), &method);
                                detail_row(ui, self.i18n.t(Message::Directory), directory);
                                detail_row(ui, self.i18n.t(Message::Encrypted), encrypted);
                            });

                        ui.add_space(18.0);
                        ui.horizontal(|ui| {
                            if ui
                                .add_sized(
                                    [128.0, 36.0],
                                    primary_button(self.i18n.t(Message::Extract)),
                                )
                                .clicked()
                            {
                                self.extract_selected(
                                    selected_entry_name.clone(),
                                    selected_entry_is_directory,
                                );
                            }
                            if ui
                                .add_sized(
                                    [112.0, 36.0],
                                    secondary_button(self.i18n.t(Message::Test)),
                                )
                                .clicked()
                            {
                                self.test_archive();
                            }
                        });
                    });
            });
    }

    fn show_welcome_state(&mut self, ui: &mut Ui) {
        Frame::NONE
            .fill(PANEL)
            .stroke(Stroke::new(1.0, BORDER))
            .corner_radius(CornerRadius::same(8))
            .inner_margin(Margin::symmetric(28, 24))
            .show(ui, |ui| {
                ui.label(
                    RichText::new(self.i18n.t(Message::Welcome))
                        .heading()
                        .strong()
                        .color(TEXT),
                );
                ui.add_space(8.0);
                ui.label(RichText::new(self.i18n.t(Message::WelcomeDetail)).color(MUTED));
                ui.add_space(18.0);
                if ui
                    .add_sized(
                        [160.0, 38.0],
                        primary_button(self.i18n.t(Message::OpenArchive)),
                    )
                    .clicked()
                {
                    self.pick_archive_file();
                }
            });
    }

    fn show_status_bar(&self, ui: &mut Ui) {
        Panel::bottom("status_bar")
            .frame(
                Frame::NONE
                    .fill(PANEL)
                    .inner_margin(Margin::symmetric(14, 8)),
            )
            .show(ui, |ui| {
                ui.horizontal(|ui| {
                    status_pill(ui, &self.status, status_color(&self.status, &self.i18n));
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

fn apply_theme(ctx: &egui::Context) {
    ctx.set_theme(egui::Theme::Dark);
    let mut style = (*ctx.style_of(egui::Theme::Dark)).clone();
    style.visuals = Visuals::dark();
    style.visuals.override_text_color = Some(TEXT);
    style.visuals.weak_text_color = Some(MUTED);
    style.visuals.panel_fill = PANEL;
    style.visuals.window_fill = BG;
    style.visuals.faint_bg_color = PANEL_ALT;
    style.visuals.extreme_bg_color = SURFACE;
    style.visuals.text_edit_bg_color = Some(Color32::from_rgb(14, 18, 22));
    style.visuals.window_stroke = Stroke::new(1.0, BORDER);
    style.visuals.window_corner_radius = CornerRadius::same(8);
    style.visuals.menu_corner_radius = CornerRadius::same(6);
    style.visuals.selection.bg_fill = ACCENT_SOFT;
    style.visuals.selection.stroke = Stroke::new(1.0, ACCENT);
    style.visuals.hyperlink_color = ACCENT;
    style.visuals.warn_fg_color = WARNING;
    style.visuals.error_fg_color = DANGER;
    style.visuals.button_frame = true;
    style.visuals.interact_cursor = Some(egui::CursorIcon::PointingHand);

    for visuals in [
        &mut style.visuals.widgets.noninteractive,
        &mut style.visuals.widgets.inactive,
        &mut style.visuals.widgets.hovered,
        &mut style.visuals.widgets.active,
        &mut style.visuals.widgets.open,
    ] {
        visuals.corner_radius = CornerRadius::same(6);
        visuals.bg_stroke = Stroke::new(1.0, BORDER);
    }
    style.visuals.widgets.noninteractive.bg_fill = PANEL;
    style.visuals.widgets.noninteractive.weak_bg_fill = PANEL_ALT;
    style.visuals.widgets.inactive.bg_fill = SURFACE;
    style.visuals.widgets.inactive.weak_bg_fill = SURFACE;
    style.visuals.widgets.hovered.bg_fill = Color32::from_rgb(38, 47, 56);
    style.visuals.widgets.hovered.weak_bg_fill = Color32::from_rgb(38, 47, 56);
    style.visuals.widgets.hovered.bg_stroke = Stroke::new(1.0, ACCENT);
    style.visuals.widgets.active.bg_fill = ACCENT_SOFT;
    style.visuals.widgets.active.weak_bg_fill = ACCENT_SOFT;
    style.visuals.widgets.active.bg_stroke = Stroke::new(1.0, ACCENT);

    style.spacing.item_spacing = Vec2::new(10.0, 8.0);
    style.spacing.button_padding = Vec2::new(14.0, 8.0);
    style.spacing.interact_size = Vec2::new(36.0, 32.0);
    style.spacing.window_margin = Margin::same(16);
    style.spacing.menu_margin = Margin::symmetric(10, 8);

    style
        .text_styles
        .insert(TextStyle::Heading, FontId::proportional(22.0));
    style
        .text_styles
        .insert(TextStyle::Body, FontId::proportional(14.0));
    style
        .text_styles
        .insert(TextStyle::Small, FontId::proportional(12.0));

    ctx.set_style_of(egui::Theme::Dark, style);
}

fn primary_button(label: &str) -> Button<'_> {
    Button::new(RichText::new(label).strong().color(TEXT))
        .fill(ACCENT_SOFT)
        .stroke(Stroke::new(1.0, ACCENT))
        .corner_radius(CornerRadius::same(6))
}

fn secondary_button(label: &str) -> Button<'_> {
    Button::new(RichText::new(label).color(TEXT))
        .fill(SURFACE)
        .stroke(Stroke::new(1.0, BORDER))
        .corner_radius(CornerRadius::same(6))
}

fn status_pill(ui: &mut Ui, text: &str, color: Color32) {
    Frame::NONE
        .fill(Color32::from_rgba_premultiplied(
            color.r(),
            color.g(),
            color.b(),
            44,
        ))
        .stroke(Stroke::new(1.0, color))
        .corner_radius(CornerRadius::same(128))
        .inner_margin(Margin::symmetric(8, 3))
        .show(ui, |ui| {
            ui.label(RichText::new(text).small().strong().color(TEXT));
        });
}

fn status_color(status: &str, i18n: &I18n) -> Color32 {
    if status.starts_with(i18n.t(Message::StatusError)) {
        DANGER
    } else if status == i18n.t(Message::StatusLoading) {
        WARNING
    } else if status == i18n.t(Message::StatusReady) || status.starts_with("Extracted") {
        SUCCESS
    } else {
        ACCENT
    }
}

fn empty_panel(ui: &mut Ui, title: &str, detail: &str) {
    Frame::NONE
        .fill(PANEL)
        .stroke(Stroke::new(1.0, BORDER))
        .corner_radius(CornerRadius::same(8))
        .inner_margin(Margin::symmetric(16, 14))
        .show(ui, |ui| {
            ui.label(RichText::new(title).strong().color(TEXT));
            ui.add_space(4.0);
            ui.label(RichText::new(detail).small().color(MUTED));
        });
}

fn error_panel(ui: &mut Ui, title: &str, detail: &str) {
    Frame::NONE
        .fill(Color32::from_rgb(45, 26, 26))
        .stroke(Stroke::new(1.0, DANGER))
        .corner_radius(CornerRadius::same(8))
        .inner_margin(Margin::symmetric(16, 14))
        .show(ui, |ui| {
            ui.label(RichText::new(title).strong().color(TEXT));
            ui.add_space(4.0);
            ui.label(RichText::new(detail).color(Color32::from_rgb(255, 205, 201)));
        });
}

fn loading_panel(ui: &mut Ui, text: &str) {
    Frame::NONE
        .fill(PANEL)
        .stroke(Stroke::new(1.0, BORDER))
        .corner_radius(CornerRadius::same(8))
        .inner_margin(Margin::symmetric(16, 14))
        .show(ui, |ui| {
            ui.spinner();
            ui.label(RichText::new(text).color(MUTED));
        });
}

fn stat_strip(ui: &mut Ui, count: &str, count_label: &str, total_label: &str, total_size: &str) {
    Frame::NONE
        .fill(PANEL)
        .stroke(Stroke::new(1.0, BORDER))
        .corner_radius(CornerRadius::same(8))
        .inner_margin(Margin::symmetric(12, 10))
        .show(ui, |ui| {
            ui.horizontal(|ui| {
                metric(ui, count_label, count);
                ui.separator();
                metric(ui, total_label, total_size);
            });
        });
}

fn metric(ui: &mut Ui, label: &str, value: &str) {
    ui.vertical(|ui| {
        ui.label(RichText::new(label).small().color(MUTED));
        ui.label(RichText::new(value).strong().color(TEXT));
    });
}

fn detail_row(ui: &mut Ui, label: &str, value: &str) {
    ui.label(RichText::new(label).small().color(MUTED));
    ui.add(Label::new(RichText::new(value).color(TEXT)).truncate());
    ui.end_row();
}

fn format_ratio(ratio: f64) -> String {
    if ratio > 0.0 {
        format!("{:.0}%", ratio * 100.0)
    } else {
        "-".to_string()
    }
}

/// Launch the native egui archive browser.
///
/// When `archive_path` is `None`, the GUI opens with a file picker (default).
pub fn run_gui(
    archive_path: Option<&str>,
    locale: Option<Locale>,
    password: Option<String>,
) -> Result<(), RarustError> {
    let locale = locale.unwrap_or_else(Locale::detect);

    let native_options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default()
            .with_inner_size([1040.0, 700.0])
            .with_min_inner_size([720.0, 520.0]),
        ..Default::default()
    };

    let archive = archive_path.map(str::to_owned);
    let pwd = password;

    eframe::run_native(
        "rarust-gui",
        native_options,
        Box::new(move |cc| {
            let font_setup = super::fonts::setup_fonts(&cc.egui_ctx);
            apply_theme(&cc.egui_ctx);
            Ok(Box::new(RarustApp::new(archive, locale, font_setup, pwd)))
        }),
    )
    .map_err(|e| RarustError::Unsupported(format!("GUI failed to start: {e}")))
}
