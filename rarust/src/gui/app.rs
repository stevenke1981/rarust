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

const WINDOW: Color32 = Color32::from_rgb(242, 244, 247);
const CHROME: Color32 = Color32::from_rgb(232, 236, 241);
const CHROME_DARK: Color32 = Color32::from_rgb(216, 222, 230);
const LIST_BG: Color32 = Color32::from_rgb(255, 255, 255);
const HEADER_BG: Color32 = Color32::from_rgb(226, 231, 238);
const ROW_ALT: Color32 = Color32::from_rgb(248, 250, 252);
const BORDER: Color32 = Color32::from_rgb(177, 187, 199);
const TEXT: Color32 = Color32::from_rgb(26, 30, 36);
const MUTED: Color32 = Color32::from_rgb(83, 93, 107);
const ACCENT: Color32 = Color32::from_rgb(0, 103, 192);
const SELECTED: Color32 = Color32::from_rgb(214, 233, 255);
const SUCCESS: Color32 = Color32::from_rgb(38, 128, 67);
const WARNING: Color32 = Color32::from_rgb(154, 103, 0);
const DANGER: Color32 = Color32::from_rgb(184, 40, 31);

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

    fn selected_entry(&self) -> Option<&Entry> {
        let archive = self.archive.as_ref()?;
        let selected = self.selected?;
        archive.entries.get(selected)
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
        Panel::top("classic_menu")
            .frame(
                Frame::NONE
                    .fill(CHROME)
                    .inner_margin(Margin::symmetric(8, 2)),
            )
            .show(ui, |ui| {
                ui.horizontal(|ui| {
                    ui.menu_button("File", |ui| {
                        if ui.button(self.i18n.t(Message::OpenArchive)).clicked() {
                            self.pick_archive_file();
                            ui.close();
                        }
                        if ui.button(self.i18n.t(Message::Refresh)).clicked() {
                            self.reload_archive();
                            ui.close();
                        }
                    });

                    ui.menu_button("Commands", |ui| {
                        if ui
                            .add_enabled(
                                self.selected_entry().is_some(),
                                Button::new(self.i18n.t(Message::Extract)),
                            )
                            .clicked()
                        {
                            if let Some(entry) = self.selected_entry() {
                                self.extract_selected(entry.name.clone(), entry.is_directory);
                            }
                            ui.close();
                        }
                        if ui
                            .add_enabled(
                                self.archive_path.is_some(),
                                Button::new(self.i18n.t(Message::Test)),
                            )
                            .clicked()
                        {
                            self.test_archive();
                            ui.close();
                        }
                    });

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

                    ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                        ui.label(RichText::new(self.i18n.t(Message::AppTitle)).color(MUTED));
                    });
                });
            });
    }

    fn show_toolbar(&mut self, ui: &mut Ui) {
        Panel::top("classic_toolbar")
            .frame(
                Frame::NONE
                    .fill(CHROME_DARK)
                    .inner_margin(Margin::symmetric(8, 6)),
            )
            .show(ui, |ui| {
                ui.horizontal(|ui| {
                    if ui
                        .add_sized(
                            [86.0, 38.0],
                            toolbar_button(self.i18n.t(Message::OpenArchive)),
                        )
                        .clicked()
                    {
                        self.pick_archive_file();
                    }

                    let selected = self.selected_entry().map(|entry| {
                        (
                            entry.name.clone(),
                            entry.is_directory,
                            entry.size,
                            entry.is_encrypted,
                        )
                    });
                    let can_extract = selected.is_some();
                    if ui
                        .add_enabled(
                            can_extract,
                            Button::new(toolbar_label(self.i18n.t(Message::Extract)))
                                .min_size(Vec2::new(86.0, 38.0)),
                        )
                        .clicked()
                        && let Some((name, is_directory, _, _)) = selected.clone()
                    {
                        self.extract_selected(name, is_directory);
                    }

                    if ui
                        .add_enabled(
                            self.archive_path.is_some(),
                            Button::new(toolbar_label(self.i18n.t(Message::Test)))
                                .min_size(Vec2::new(86.0, 38.0)),
                        )
                        .clicked()
                    {
                        self.test_archive();
                    }

                    if ui
                        .add_sized([86.0, 38.0], toolbar_button(self.i18n.t(Message::Refresh)))
                        .clicked()
                    {
                        self.reload_archive();
                    }

                    ui.separator();

                    if let Some((_, _, size, encrypted)) = selected {
                        ui.label(RichText::new(util::format_size(size)).color(MUTED));
                        if encrypted {
                            ui.label(RichText::new(self.i18n.t(Message::Encrypted)).color(WARNING));
                        }
                    } else {
                        ui.label(RichText::new(self.i18n.t(Message::SelectFile)).color(MUTED));
                    }
                });
            });
    }

    fn show_path_bar(&mut self, ui: &mut Ui) {
        Panel::top("classic_path_bar")
            .frame(
                Frame::NONE
                    .fill(CHROME)
                    .inner_margin(Margin::symmetric(8, 6)),
            )
            .show(ui, |ui| {
                ui.horizontal(|ui| {
                    ui.label(RichText::new(self.i18n.t(Message::Archive)).strong());
                    archive_path_field(ui, self.archive_path.as_deref().unwrap_or("-"));
                    ui.add_space(8.0);
                    ui.label(RichText::new(self.i18n.t(Message::SearchPlaceholder)).color(MUTED));
                    ui.add_sized(
                        [220.0, 24.0],
                        TextEdit::singleline(&mut self.search).desired_width(220.0),
                    );
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
                    .fill(Color32::from_rgb(255, 247, 222))
                    .stroke(Stroke::new(1.0, Color32::from_rgb(230, 192, 94)))
                    .inner_margin(Margin::symmetric(8, 5)),
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

    fn show_archive_browser(&mut self, ui: &mut Ui) {
        CentralPanel::default()
            .frame(Frame::NONE.fill(WINDOW).inner_margin(Margin::same(8)))
            .show(ui, |ui| {
                if self.archive_path.is_none() {
                    self.show_welcome_state(ui);
                    return;
                }

                if let Some(err) = &self.load_error {
                    error_panel(ui, self.i18n.t(Message::StatusError), err);
                    ui.add_space(8.0);
                    if ui.button(self.i18n.t(Message::OpenArchive)).clicked() {
                        self.pick_archive_file();
                    }
                    return;
                }

                let Some(loaded) = &self.archive else {
                    loading_panel(ui, self.i18n.t(Message::StatusLoading));
                    return;
                };

                let family_label = loaded.family_label.clone();
                let entry_count = loaded.entries.len();
                let total_size = loaded.entries.iter().map(|entry| entry.size).sum::<u64>();

                Frame::NONE
                    .fill(CHROME)
                    .stroke(Stroke::new(1.0, BORDER))
                    .inner_margin(Margin::symmetric(8, 5))
                    .show(ui, |ui| {
                        ui.horizontal(|ui| {
                            ui.label(RichText::new(self.i18n.t(Message::Format)).strong());
                            ui.label(family_label);
                            ui.separator();
                            ui.label(format!(
                                "{entry_count} {}",
                                self.i18n.t(Message::FilesCount)
                            ));
                            ui.separator();
                            ui.label(format!(
                                "{}: {}",
                                self.i18n.t(Message::TotalSize),
                                util::format_size(total_size)
                            ));
                        });
                    });

                ui.add_space(6.0);
                self.show_entry_table(ui);
            });
    }

    fn show_entry_table(&mut self, ui: &mut Ui) {
        let filtered: Vec<(usize, EntryRow)> = self
            .filtered_entries()
            .into_iter()
            .map(|(idx, entry)| {
                (
                    idx,
                    EntryRow {
                        name: entry.name.clone(),
                        size: util::format_size(entry.size),
                        ratio: format_ratio(entry.ratio),
                        modified: entry
                            .modified
                            .map(util::format_dos_time)
                            .unwrap_or_else(|| "-".to_string()),
                        crc32: entry
                            .crc32
                            .map(|c| format!("{:08X}", c))
                            .unwrap_or_else(|| "-".to_string()),
                        method: entry.method.clone(),
                        attrs: entry_attrs(entry),
                        is_directory: entry.is_directory,
                        is_encrypted: entry.is_encrypted,
                    },
                )
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

        Frame::NONE
            .fill(LIST_BG)
            .stroke(Stroke::new(1.0, BORDER))
            .show(ui, |ui| {
                Grid::new("entry_table_header")
                    .num_columns(7)
                    .spacing([12.0, 0.0])
                    .show(ui, |ui| {
                        table_header(ui, self.i18n.t(Message::Name), 320.0);
                        table_header(ui, self.i18n.t(Message::Size), 92.0);
                        table_header(ui, self.i18n.t(Message::Ratio), 72.0);
                        table_header(ui, self.i18n.t(Message::Modified), 142.0);
                        table_header(ui, self.i18n.t(Message::Crc32), 92.0);
                        table_header(ui, self.i18n.t(Message::Method), 108.0);
                        table_header(ui, "Attr", 60.0);
                        ui.end_row();
                    });

                ScrollArea::vertical()
                    .id_salt("classic_archive_entry_table")
                    .auto_shrink([false, false])
                    .show(ui, |ui| {
                        Grid::new("entry_table_rows")
                            .num_columns(7)
                            .spacing([12.0, 0.0])
                            .striped(false)
                            .show(ui, |ui| {
                                for (row, (idx, entry)) in filtered.iter().enumerate() {
                                    self.entry_row(ui, *idx, entry, row);
                                }
                            });
                    });
            });
    }

    fn entry_row(&mut self, ui: &mut Ui, idx: usize, entry: &EntryRow, row: usize) {
        let selected = self.selected == Some(idx);
        let fill = if selected {
            SELECTED
        } else if row % 2 == 0 {
            LIST_BG
        } else {
            ROW_ALT
        };
        let name = if entry.is_directory {
            format!("[DIR] {}", entry.name)
        } else {
            entry.name.clone()
        };
        let name_color = if entry.is_directory { ACCENT } else { TEXT };

        Frame::NONE
            .fill(fill)
            .inner_margin(Margin::symmetric(4, 1))
            .show(ui, |ui| {
                if ui
                    .add_sized(
                        [320.0, 22.0],
                        Button::selectable(selected, RichText::new(name).color(name_color))
                            .frame(false),
                    )
                    .clicked()
                {
                    self.selected = Some(idx);
                }
            });
        table_cell(ui, &entry.size, 92.0, true);
        table_cell(ui, &entry.ratio, 72.0, true);
        table_cell(ui, &entry.modified, 142.0, false);
        table_cell(ui, &entry.crc32, 92.0, false);
        table_cell(ui, &entry.method, 108.0, false);
        let attrs_color = if entry.is_encrypted { WARNING } else { MUTED };
        table_cell_colored(ui, &entry.attrs, 60.0, attrs_color, false);
        ui.end_row();
    }

    fn show_welcome_state(&mut self, ui: &mut Ui) {
        Frame::NONE
            .fill(LIST_BG)
            .stroke(Stroke::new(1.0, BORDER))
            .inner_margin(Margin::symmetric(18, 16))
            .show(ui, |ui| {
                ui.label(
                    RichText::new(self.i18n.t(Message::Welcome))
                        .heading()
                        .strong()
                        .color(TEXT),
                );
                ui.add_space(8.0);
                ui.label(RichText::new(self.i18n.t(Message::WelcomeDetail)).color(MUTED));
                ui.add_space(14.0);
                if ui.button(self.i18n.t(Message::OpenArchive)).clicked() {
                    self.pick_archive_file();
                }
            });
    }

    fn show_status_bar(&self, ui: &mut Ui) {
        Panel::bottom("classic_status_bar")
            .frame(
                Frame::NONE
                    .fill(CHROME)
                    .stroke(Stroke::new(1.0, BORDER))
                    .inner_margin(Margin::symmetric(8, 4)),
            )
            .show(ui, |ui| {
                ui.horizontal(|ui| {
                    ui.colored_label(status_color(&self.status, &self.i18n), &self.status);
                    if let Some(entry) = self.selected_entry() {
                        ui.separator();
                        ui.label(format!(
                            "{}: {}",
                            self.i18n.t(Message::Name),
                            truncate_middle(&entry.name, 72)
                        ));
                        ui.separator();
                        ui.label(util::format_size(entry.size));
                    }
                    if let Some(source) = &self.font_setup.cjk_source {
                        ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                            ui.label(RichText::new(format!("Font: {source}")).weak().small());
                        });
                    }
                });
            });
    }
}

struct EntryRow {
    name: String,
    size: String,
    ratio: String,
    modified: String,
    crc32: String,
    method: String,
    attrs: String,
    is_directory: bool,
    is_encrypted: bool,
}

impl eframe::App for RarustApp {
    fn logic(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        let title = self.i18n.t(Message::AppTitle);
        ctx.send_viewport_cmd(egui::ViewportCommand::Title(title.to_owned()));
    }

    fn ui(&mut self, ui: &mut Ui, _frame: &mut eframe::Frame) {
        self.show_menu_bar(ui);
        self.show_toolbar(ui);
        self.show_path_bar(ui);
        self.show_font_warning(ui);
        self.show_archive_browser(ui);
        self.show_status_bar(ui);
    }
}

fn apply_theme(ctx: &egui::Context) {
    ctx.set_theme(egui::Theme::Light);
    let mut style = (*ctx.style_of(egui::Theme::Light)).clone();
    style.visuals = Visuals::light();
    style.visuals.override_text_color = Some(TEXT);
    style.visuals.weak_text_color = Some(MUTED);
    style.visuals.panel_fill = CHROME;
    style.visuals.window_fill = WINDOW;
    style.visuals.faint_bg_color = ROW_ALT;
    style.visuals.extreme_bg_color = LIST_BG;
    style.visuals.text_edit_bg_color = Some(LIST_BG);
    style.visuals.window_stroke = Stroke::new(1.0, BORDER);
    style.visuals.window_corner_radius = CornerRadius::same(2);
    style.visuals.menu_corner_radius = CornerRadius::same(2);
    style.visuals.selection.bg_fill = SELECTED;
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
        visuals.corner_radius = CornerRadius::same(2);
        visuals.bg_stroke = Stroke::new(1.0, BORDER);
        visuals.fg_stroke = Stroke::new(1.0, TEXT);
    }
    style.visuals.widgets.noninteractive.bg_fill = CHROME;
    style.visuals.widgets.noninteractive.weak_bg_fill = CHROME;
    style.visuals.widgets.inactive.bg_fill = Color32::from_rgb(244, 247, 250);
    style.visuals.widgets.inactive.weak_bg_fill = Color32::from_rgb(244, 247, 250);
    style.visuals.widgets.hovered.bg_fill = Color32::from_rgb(228, 240, 252);
    style.visuals.widgets.hovered.weak_bg_fill = Color32::from_rgb(228, 240, 252);
    style.visuals.widgets.hovered.bg_stroke = Stroke::new(1.0, ACCENT);
    style.visuals.widgets.active.bg_fill = SELECTED;
    style.visuals.widgets.active.weak_bg_fill = SELECTED;
    style.visuals.widgets.active.bg_stroke = Stroke::new(1.0, ACCENT);

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

fn toolbar_button(label: &str) -> Button<'_> {
    Button::new(toolbar_label(label)).min_size(Vec2::new(86.0, 38.0))
}

fn toolbar_label(label: &str) -> RichText {
    RichText::new(label).strong().color(TEXT)
}

fn archive_path_field(ui: &mut Ui, path: &str) {
    Frame::NONE
        .fill(LIST_BG)
        .stroke(Stroke::new(1.0, BORDER))
        .inner_margin(Margin::symmetric(6, 3))
        .show(ui, |ui| {
            ui.set_min_width(420.0);
            ui.add(Label::new(RichText::new(path).monospace().color(TEXT)).truncate());
        });
}

fn table_header(ui: &mut Ui, text: &str, width: f32) {
    Frame::NONE
        .fill(HEADER_BG)
        .stroke(Stroke::new(1.0, BORDER))
        .inner_margin(Margin::symmetric(5, 3))
        .show(ui, |ui| {
            ui.set_min_width(width);
            ui.label(RichText::new(text).strong().color(TEXT));
        });
}

fn table_cell(ui: &mut Ui, text: &str, width: f32, right_aligned: bool) {
    table_cell_colored(ui, text, width, TEXT, right_aligned);
}

fn table_cell_colored(ui: &mut Ui, text: &str, width: f32, color: Color32, right_aligned: bool) {
    Frame::NONE
        .inner_margin(Margin::symmetric(4, 1))
        .show(ui, |ui| {
            ui.set_min_width(width);
            if right_aligned {
                ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                    ui.add(Label::new(RichText::new(text).color(color)).truncate());
                });
            } else {
                ui.add(Label::new(RichText::new(text).color(color)).truncate());
            }
        });
}

fn error_panel(ui: &mut Ui, title: &str, detail: &str) {
    Frame::NONE
        .fill(Color32::from_rgb(255, 236, 233))
        .stroke(Stroke::new(1.0, DANGER))
        .inner_margin(Margin::symmetric(12, 10))
        .show(ui, |ui| {
            ui.label(RichText::new(title).strong().color(DANGER));
            ui.add_space(4.0);
            ui.label(RichText::new(detail).color(TEXT));
        });
}

fn empty_panel(ui: &mut Ui, title: &str, detail: &str) {
    Frame::NONE
        .fill(LIST_BG)
        .stroke(Stroke::new(1.0, BORDER))
        .inner_margin(Margin::symmetric(12, 10))
        .show(ui, |ui| {
            ui.label(RichText::new(title).strong().color(TEXT));
            ui.add_space(4.0);
            ui.label(RichText::new(detail).color(MUTED));
        });
}

fn loading_panel(ui: &mut Ui, text: &str) {
    Frame::NONE
        .fill(LIST_BG)
        .stroke(Stroke::new(1.0, BORDER))
        .inner_margin(Margin::symmetric(12, 10))
        .show(ui, |ui| {
            ui.spinner();
            ui.label(RichText::new(text).color(MUTED));
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

fn entry_attrs(entry: &Entry) -> String {
    let mut attrs = String::new();
    if entry.is_directory {
        attrs.push('D');
    }
    if entry.is_encrypted {
        attrs.push('E');
    }
    if attrs.is_empty() {
        attrs.push('-');
    }
    attrs
}

fn truncate_middle(value: &str, max_chars: usize) -> String {
    let count = value.chars().count();
    if count <= max_chars || max_chars < 8 {
        return value.to_owned();
    }

    let keep = (max_chars - 3) / 2;
    let start: String = value.chars().take(keep).collect();
    let end: String = value
        .chars()
        .rev()
        .take(max_chars - 3 - keep)
        .collect::<String>()
        .chars()
        .rev()
        .collect();
    format!("{start}...{end}")
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
            .with_inner_size([1120.0, 720.0])
            .with_min_inner_size([820.0, 560.0]),
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
