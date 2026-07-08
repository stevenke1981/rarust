//! egui application state and UI rendering.

use std::collections::HashSet;

use egui::{
    Button, CentralPanel, Color32, Frame, Label, Margin, Panel, RichText, ScrollArea, Stroke,
    TextEdit, Ui, Vec2,
};
use rarust_core::archive::{OpenOptions, PortableArchive};
use rarust_core::entry::Entry;
use rarust_core::error::RarustError;
use rarust_core::util;

use super::actions::create_dialog::{CompressionMethod, CreateArchiveParams, CreateDialog};
use super::fonts::FontSetup;
use super::i18n::{I18n, Locale, Message};
use super::theme::Theme;
use super::widgets::password::PasswordDialog;
use super::widgets::preview::FilePreview;
use super::widgets::progress::ProgressDialog;
use super::widgets::tab_bar::{TabBar, TabBarAction};

/// Sortable column in the entry table.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum SortBy {
    Name,
    Size,
    Ratio,
    Modified,
    Crc32,
    Method,
}

/// Main egui application for browsing RAR archives.
pub struct RarustApp {
    archive_path: Option<String>,
    i18n: I18n,
    font_setup: FontSetup,
    password: Option<String>,
    search: String,
    theme: Theme,
    selected: HashSet<usize>,
    sort_by: SortBy,
    sort_ascending: bool,
    archive: Option<LoadedArchive>,
    load_error: Option<String>,
    status: String,

    // Widgets
    password_dialog: PasswordDialog,
    progress_dialog: ProgressDialog,
    preview: FilePreview,
    create_dialog: CreateDialog,
    tab_bar: TabBar,

    /// Set when the opened archive needs a password.
    needs_password: bool,
}

struct LoadedArchive {
    family_label: String,
    entries: Vec<Entry>,
}

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
        let theme = match Theme::load_saved().as_deref() {
            Some("Dark") => Theme::DARK,
            _ => Theme::LIGHT,
        };
        let mut app = Self {
            archive_path,
            i18n,
            font_setup,
            password,
            search: String::new(),
            theme,
            selected: HashSet::new(),
            sort_by: SortBy::Name,
            sort_ascending: true,
            archive: None,
            load_error: None,
            status: if has_archive {
                String::new()
            } else {
                I18n::new(locale).t(Message::StatusReady).to_owned()
            },
            password_dialog: PasswordDialog::new(),
            progress_dialog: ProgressDialog::new(),
            preview: FilePreview::new(),
            create_dialog: CreateDialog::new(),
            tab_bar: TabBar::new(),
            needs_password: false,
        };
        if has_archive {
            app.reload_archive();
            let label = app
                .archive_path
                .clone()
                .map(|p| {
                    std::path::Path::new(&p)
                        .file_name()
                        .map(|n| n.to_string_lossy().to_string())
                        .unwrap_or(p)
                })
                .unwrap_or_else(|| "Archive".to_string());
            app.tab_bar
                .open_tab(app.archive_path.clone(), label);
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
        let path = match self.archive_path.as_deref() {
            Some(p) => p.to_owned(),
            None => {
                self.archive = None;
                self.load_error = None;
                self.selected.clear();
                self.status = self.i18n.t(Message::StatusReady).to_owned();
                return;
            }
        };

        self.status = self.i18n.t(Message::StatusLoading).to_owned();
        self.load_error = None;
        self.selected.clear();

        // Check password dialog cache first
        let pwd = self
            .password_dialog
            .cached(&path)
            .or_else(|| self.password.clone());

        let options = OpenOptions {
            password: pwd,
            ..OpenOptions::default()
        };

        match PortableArchive::open_with_options(&path, &options) {
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
                // Flag for password dialog (handled in show_archive_browser)
                if e.to_string().to_ascii_lowercase().contains("password") {
                    self.needs_password = true;
                }
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
        let mut result: Vec<(usize, &Entry)> = loaded
            .entries
            .iter()
            .enumerate()
            .filter(|(_, e)| query.is_empty() || e.name.to_ascii_lowercase().contains(&query))
            .collect();
        result.sort_by(|(_, a), (_, b)| {
            let cmp = match self.sort_by {
                SortBy::Name => a.name.cmp(&b.name),
                SortBy::Size => a.size.cmp(&b.size),
                SortBy::Ratio => a.ratio.partial_cmp(&b.ratio).unwrap_or(std::cmp::Ordering::Equal),
                SortBy::Modified => a.modified.cmp(&b.modified),
                SortBy::Crc32 => a.crc32.cmp(&b.crc32),
                SortBy::Method => a.method.cmp(&b.method),
            };
            if self.sort_ascending { cmp } else { cmp.reverse() }
        });
        result
    }

    fn selected_entries(&self) -> Vec<&Entry> {
        let archive = match self.archive.as_ref() {
            Some(a) => a,
            None => return Vec::new(),
        };
        self.selected
            .iter()
            .filter_map(|idx| archive.entries.get(*idx))
            .collect()
    }

    fn selected_entry(&self) -> Option<&Entry> {
        self.selected.iter().next().and_then(|idx| {
            self.archive.as_ref()?.entries.get(*idx)
        })
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
                    .fill(self.theme.chrome)
                    .inner_margin(Margin::symmetric(8, 2)),
            )
            .show(ui, |ui| {
                ui.horizontal(|ui| {
                    ui.menu_button("File", |ui| {
                        if ui.button(self.i18n.t(Message::OpenArchive)).clicked() {
                            self.pick_archive_file();
                            ui.close();
                        }
                        if ui.button(self.i18n.t(Message::CloseArchive)).clicked() {
                            self.close_archive();
                            ui.close();
                        }
                        if ui.button(self.i18n.t(Message::Refresh)).clicked() {
                            self.reload_archive();
                            ui.close();
                        }
                        if ui.button(self.i18n.t(Message::CreateArchive)).clicked() {
                            self.create_dialog.visible = true;
                            ui.close();
                        }
                    });

                    ui.menu_button("Commands", |ui| {
                        if ui
                            .add_enabled(
                                !self.selected.is_empty(),
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

                    ui.menu_button(self.i18n.t(Message::Theme), |ui| {
                        if ui
                            .selectable_label(self.theme.name == "Light", self.i18n.t(Message::LightTheme))
                            .clicked()
                        {
                            self.theme = Theme::LIGHT;
                            Theme::save("Light");
                            ui.close();
                        }
                        if ui
                            .selectable_label(self.theme.name == "Dark", self.i18n.t(Message::DarkTheme))
                            .clicked()
                        {
                            self.theme = Theme::DARK;
                            Theme::save("Dark");
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
                        ui.label(RichText::new(self.i18n.t(Message::AppTitle)).color(self.theme.muted));
                    });
                });
            });
    }

    fn close_archive(&mut self) {
        self.archive_path = None;
        self.archive = None;
        self.load_error = None;
        self.selected.clear();
        self.search.clear();
        self.status = self.i18n.t(Message::StatusReady).to_owned();
    }

    fn show_toolbar(&mut self, ui: &mut Ui) {
        Panel::top("classic_toolbar")
            .frame(
                Frame::NONE
                    .fill(self.theme.chrome_dark)
                    .inner_margin(Margin::symmetric(8, 6)),
            )
            .show(ui, |ui| {
                ui.horizontal(|ui| {
                    if ui
                        .add_sized(
                            [86.0, 38.0],
                            toolbar_button(self.i18n.t(Message::OpenArchive), self.theme),
                        )
                        .clicked()
                    {
                        self.pick_archive_file();
                    }

                    let can_extract = !self.selected.is_empty();
                    if ui
                        .add_enabled(
                            can_extract,
                            Button::new(toolbar_label(self.i18n.t(Message::Extract), self.theme))
                                .min_size(Vec2::new(86.0, 38.0)),
                        )
                        .clicked()
                        && let Some(entry) = self.selected_entry().map(|e| (e.name.clone(), e.is_directory))
                    {
                        self.extract_selected(entry.0, entry.1);
                    }

                    if ui
                        .add_enabled(
                            self.archive_path.is_some(),
                            Button::new(toolbar_label(self.i18n.t(Message::Test), self.theme))
                                .min_size(Vec2::new(86.0, 38.0)),
                        )
                        .clicked()
                    {
                        self.test_archive();
                    }

                    if ui
                        .add_sized(
                            [86.0, 38.0],
                            toolbar_button(self.i18n.t(Message::Refresh), self.theme),
                        )
                        .clicked()
                    {
                        self.reload_archive();
                    }

                    ui.separator();

                    if let Some(entry) = self.selected_entry() {
                        ui.label(RichText::new(util::format_size(entry.size)).color(self.theme.muted));
                        if entry.is_encrypted {
                            ui.label(
                                RichText::new(self.i18n.t(Message::Encrypted)).color(self.theme.warning),
                            );
                        }
                    } else {
                        ui.label(RichText::new(self.i18n.t(Message::SelectFile)).color(self.theme.muted));
                    }
                });
            });
    }

    fn show_path_bar(&mut self, ui: &mut Ui) {
        Panel::top("classic_path_bar")
            .frame(
                Frame::NONE
                    .fill(self.theme.chrome)
                    .inner_margin(Margin::symmetric(8, 6)),
            )
            .show(ui, |ui| {
                ui.horizontal(|ui| {
                    ui.label(RichText::new(self.i18n.t(Message::Archive)).strong());
                    archive_path_field(ui, self.archive_path.as_deref().unwrap_or("-"), self.theme);
                    ui.add_space(8.0);
                    ui.label(
                        RichText::new(self.i18n.t(Message::SearchPlaceholder)).color(self.theme.muted),
                    );
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
                    self.theme.warning,
                    format!(
                        "{}: {}",
                        self.i18n.t(Message::FontWarning),
                        self.i18n.t(Message::FontWarningDetail)
                    ),
                );
            });
    }

    fn show_archive_browser(&mut self, ui: &mut Ui) {
        // Password dialog (modal)
        if let Some(path) = self.archive_path.clone() {
            if self.needs_password && self.password_dialog.cached(&path).is_none() {
                self.password_dialog.prompt(&path);
                self.needs_password = false;
            }
            if let Some(pwd) = self.password_dialog.show(ui.ctx(), &path) {
                self.password = Some(pwd);
                self.reload_archive();
            }
        }

        // Progress dialog
        self.progress_dialog.render(ui.ctx());

        // Preview panel (before CentralPanel per panel ordering)
        self.preview.render(ui, 320.0);

        // Tab bar
        let action = self.tab_bar.render(ui);
        match action {
            TabBarAction::NewTab => {
                self.pick_archive_file();
            }
            TabBarAction::CloseTab(idx) => {
                if idx == self.tab_bar.active {
                    // Close current tab
                    self.archive_path = None;
                    self.archive = None;
                    self.load_error = None;
                    self.selected.clear();
                    self.status = self.i18n.t(Message::StatusReady).to_owned();
                }
            }
            TabBarAction::None => {}
        }

        CentralPanel::default()
            .frame(
                Frame::NONE
                    .fill(self.theme.window)
                    .inner_margin(Margin::same(8)),
            )
            .show(ui, |ui| {
                if self.archive_path.is_none() {
                    self.show_welcome_state(ui);
                    return;
                }

                if let Some(err) = &self.load_error {
                    error_panel(ui, self.i18n.t(Message::StatusError), err, self.theme);
                    ui.add_space(8.0);
                    if ui.button(self.i18n.t(Message::OpenArchive)).clicked() {
                        self.pick_archive_file();
                    }
                    return;
                }

                let Some(loaded) = &self.archive else {
                    loading_panel(ui, self.i18n.t(Message::StatusLoading), self.theme);
                    return;
                };

                let family_label = loaded.family_label.clone();
                let entry_count = loaded.entries.len();
                let total_size = loaded.entries.iter().map(|entry| entry.size).sum::<u64>();

                Frame::NONE
                    .fill(self.theme.chrome)
                    .stroke(Stroke::new(1.0, self.theme.border))
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
                self.theme,
            );
            return;
        }

        Frame::NONE
            .fill(self.theme.list_bg)
            .stroke(Stroke::new(1.0, self.theme.border))
            .show(ui, |ui| {
                // Sortable header row
                ui.horizontal(|ui| {
                    header_clickable(ui, self.i18n.t(Message::Name), 320.0, || {
                        self.toggle_sort(SortBy::Name);
                    });
                    header_clickable(ui, self.i18n.t(Message::Size), 92.0, || {
                        self.toggle_sort(SortBy::Size);
                    });
                    header_clickable(ui, self.i18n.t(Message::Ratio), 72.0, || {
                        self.toggle_sort(SortBy::Ratio);
                    });
                    header_clickable(ui, self.i18n.t(Message::Modified), 142.0, || {
                        self.toggle_sort(SortBy::Modified);
                    });
                    header_clickable(ui, self.i18n.t(Message::Crc32), 92.0, || {
                        self.toggle_sort(SortBy::Crc32);
                    });
                    header_clickable(ui, self.i18n.t(Message::Method), 108.0, || {
                        self.toggle_sort(SortBy::Method);
                    });
                    header_clickable(ui, "Attr", 60.0, || {});
                });
                ui.end_row();

                ScrollArea::vertical()
                    .id_salt("classic_archive_entry_table")
                    .auto_shrink([false, false])
                    .show(ui, |ui| {
                        for (row, (idx, entry)) in filtered.iter().enumerate() {
                            self.entry_row(ui, *idx, entry, row);
                        }
                    });
            });

        // Create dialog (modal)
        let params = self.create_dialog.show(ui);
        if let Some(params) = params {
            self.create_archive(params);
        }
    }

    fn create_archive(&mut self, params: CreateArchiveParams) {
        self.status = self.i18n.t(Message::ProgressCreating).to_owned();
        self.progress_dialog.show(&self.i18n.t(Message::ProgressCreating));

        use rarust_core::archive::ArchiveBuilder;
        let method = match params.method {
            CompressionMethod::Store => rarust_core::archive::CompressionMethod::Store,
            CompressionMethod::Fastest => rarust_core::archive::CompressionMethod::Fastest,
            CompressionMethod::Fast => rarust_core::archive::CompressionMethod::Fast,
            CompressionMethod::Normal => rarust_core::archive::CompressionMethod::Normal,
            CompressionMethod::Good => rarust_core::archive::CompressionMethod::Good,
            CompressionMethod::Best => rarust_core::archive::CompressionMethod::Best,
            CompressionMethod::Tqt => rarust_core::archive::CompressionMethod::Normal,
        };

        let mut builder = ArchiveBuilder::new();
        builder = builder.with_method(method);
        if let Some(pwd) = &params.password {
            builder = builder.with_password(pwd.clone());
            if params.encrypt_filenames {
                builder = builder.with_header_encrypt(true);
            }
        }
        if params.split_mb > 0 {
            builder = builder.with_volume_size(params.split_mb * 1024 * 1024);
        }
        for src in &params.source_paths {
            let p = std::path::Path::new(src);
            if p.is_dir() {
                builder = builder.add_file(src);
            } else {
                builder = builder.add_file(src);
            }
        }

        let result = builder.build(&params.archive_path);
        self.progress_dialog.hide();
        match result {
            Ok(()) => {
                self.status = "Archive created successfully".to_string();
                self.open_archive(params.archive_path);
            }
            Err(e) => {
                self.status = format!("{}: {}", self.i18n.t(Message::StatusError), e);
            }
        }
    }

    fn toggle_sort(&mut self, column: SortBy) {
        if self.sort_by == column {
            self.sort_ascending = !self.sort_ascending;
        } else {
            self.sort_by = column;
            self.sort_ascending = true;
        }
    }

    fn entry_row(&mut self, ui: &mut Ui, idx: usize, entry: &EntryRow, row: usize) {
        let selected = self.selected.contains(&idx);
        let fill = if selected {
            self.theme.selected
        } else if row % 2 == 0 {
            self.theme.list_bg
        } else {
            self.theme.row_alt
        };
        let name = if entry.is_directory {
            format!("[DIR] {}", entry.name)
        } else {
            entry.name.clone()
        };
        let name_color = if entry.is_directory {
            self.theme.accent
        } else {
            self.theme.text
        };

        Frame::NONE
            .fill(fill)
            .inner_margin(Margin::symmetric(4, 1))
            .show(ui, |ui| {
                let resp = ui
                    .add_sized(
                        [320.0, 22.0],
                        Button::selectable(selected, RichText::new(name).color(name_color))
                            .frame(false),
                    );

                // Multi-select: Ctrl/Cmd+click toggles, Shift+click extends
                let shift = ui.input(|i| i.modifiers.shift);
                let ctrl = ui.input(|i| i.modifiers.ctrl || i.modifiers.mac_cmd);
                if resp.clicked() {
                    if ctrl {
                        if selected {
                            self.selected.remove(&idx);
                        } else {
                            self.selected.insert(idx);
                        }
                    } else if shift {
                        if let Some(last) = self.selected.iter().max() {
                            let range: HashSet<usize> = if idx > *last {
                                (*last..=idx).collect()
                            } else {
                                (idx..=*last).collect()
                            };
                            self.selected.extend(range);
                        } else {
                            self.selected.insert(idx);
                        }
                    } else {
                        self.selected.clear();
                        self.selected.insert(idx);
                    }
                }

                // Context menu on right-click
                resp.context_menu(|ui| {
                    if ui.button(self.i18n.t(Message::Extract)).clicked() {
                        if let Some(e) = self.selected_entry() {
                            self.extract_selected(e.name.clone(), e.is_directory);
                        }
                        ui.close();
                    }
                    if ui.button(self.i18n.t(Message::CopyPath)).clicked() {
                        ui.ctx().copy_text(entry.name.clone());
                        ui.close();
                    }
                    if ui.button(self.i18n.t(Message::SelectAll)).clicked() {
                        let entries = self.filtered_entries();
                        self.selected = entries.into_iter().map(|(i, _)| i).collect();
                        ui.close();
                    }
                });
            });
        table_cell(ui, &entry.size, 92.0, true, self.theme);
        table_cell(ui, &entry.ratio, 72.0, true, self.theme);
        table_cell(ui, &entry.modified, 142.0, false, self.theme);
        table_cell(ui, &entry.crc32, 92.0, false, self.theme);
        table_cell(ui, &entry.method, 108.0, false, self.theme);
        let attrs_color = if entry.is_encrypted {
            self.theme.warning
        } else {
            self.theme.muted
        };
        table_cell_colored(ui, &entry.attrs, 60.0, attrs_color, false);
        ui.end_row();
    }

    fn show_welcome_state(&mut self, ui: &mut Ui) {
        Frame::NONE
            .fill(self.theme.list_bg)
            .stroke(Stroke::new(1.0, self.theme.border))
            .inner_margin(Margin::symmetric(18, 16))
            .show(ui, |ui| {
                ui.label(
                    RichText::new(self.i18n.t(Message::Welcome))
                        .heading()
                        .strong()
                        .color(self.theme.text),
                );
                ui.add_space(8.0);
                ui.label(
                    RichText::new(self.i18n.t(Message::WelcomeDetail)).color(self.theme.muted),
                );
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
                    .fill(self.theme.chrome)
                    .stroke(Stroke::new(1.0, self.theme.border))
                    .inner_margin(Margin::symmetric(8, 4)),
            )
            .show(ui, |ui| {
                ui.horizontal(|ui| {
                    ui.colored_label(
                        status_color(&self.status, &self.i18n, self.theme),
                        &self.status,
                    );
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
        // Apply theme every frame (handles runtime switching)
        self.theme.apply(ctx);
        // Accept dropped files
        ctx.input(|i| {
            if !i.raw.dropped_files.is_empty() {
                for file in &i.raw.dropped_files {
                    if let Some(_path) = &file.path {
                        // Will be wired as tab open in Task 8
                    }
                }
            }
        });
        // Reload theme from saved config
        if let Some(saved) = Theme::load_saved() {
            let desired = if saved == "Dark" { Theme::DARK } else { Theme::LIGHT };
            if self.theme.name != desired.name {
                self.theme = if saved == "Dark" { Theme::DARK } else { Theme::LIGHT };
            }
        }
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

fn toolbar_button<'a>(label: &'a str, theme: Theme) -> Button<'a> {
    Button::new(toolbar_label(label, theme)).min_size(Vec2::new(86.0, 38.0))
}

fn toolbar_label(label: &str, theme: Theme) -> RichText {
    RichText::new(label).strong().color(theme.text)
}

fn archive_path_field(ui: &mut Ui, path: &str, theme: Theme) {
    Frame::NONE
        .fill(theme.list_bg)
        .stroke(Stroke::new(1.0, theme.border))
        .inner_margin(Margin::symmetric(6, 3))
        .show(ui, |ui| {
            ui.set_min_width(420.0);
            ui.add(
                Label::new(RichText::new(path).monospace().color(theme.text)).truncate(),
            );
        });
}

fn header_clickable(ui: &mut Ui, text: &str, width: f32, on_click: impl FnOnce()) {
    Frame::NONE
        .fill(ui.visuals().extreme_bg_color)
        .stroke(Stroke::new(1.0, ui.visuals().widgets.noninteractive.bg_stroke.color))
        .inner_margin(Margin::symmetric(5, 3))
        .show(ui, |ui| {
            ui.set_min_width(width);
            if ui
                .add(Label::new(RichText::new(text).strong()).sense(egui::Sense::click()))
                .clicked()
            {
                on_click();
            }
        });
}

fn table_cell(ui: &mut Ui, text: &str, width: f32, right_aligned: bool, theme: Theme) {
    table_cell_colored(ui, text, width, theme.text, right_aligned);
}

fn table_cell_colored(
    ui: &mut Ui,
    text: &str,
    width: f32,
    color: Color32,
    right_aligned: bool,
) {
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

fn error_panel(ui: &mut Ui, title: &str, detail: &str, theme: Theme) {
    Frame::NONE
        .fill(Color32::from_rgb(255, 236, 233))
        .stroke(Stroke::new(1.0, theme.danger))
        .inner_margin(Margin::symmetric(12, 10))
        .show(ui, |ui| {
            ui.label(RichText::new(title).strong().color(theme.danger));
            ui.add_space(4.0);
            ui.label(RichText::new(detail).color(theme.text));
        });
}

fn empty_panel(ui: &mut Ui, title: &str, detail: &str, theme: Theme) {
    Frame::NONE
        .fill(theme.list_bg)
        .stroke(Stroke::new(1.0, theme.border))
        .inner_margin(Margin::symmetric(12, 10))
        .show(ui, |ui| {
            ui.label(RichText::new(title).strong().color(theme.text));
            ui.add_space(4.0);
            ui.label(RichText::new(detail).color(theme.muted));
        });
}

fn loading_panel(ui: &mut Ui, text: &str, theme: Theme) {
    Frame::NONE
        .fill(theme.list_bg)
        .stroke(Stroke::new(1.0, theme.border))
        .inner_margin(Margin::symmetric(12, 10))
        .show(ui, |ui| {
            ui.spinner();
            ui.label(RichText::new(text).color(theme.muted));
        });
}

fn status_color(status: &str, i18n: &I18n, theme: Theme) -> Color32 {
    if status.starts_with(i18n.t(Message::StatusError)) {
        theme.danger
    } else if status == i18n.t(Message::StatusLoading) {
        theme.warning
    } else if status == i18n.t(Message::StatusReady) || status.starts_with("Extracted") {
        theme.success
    } else {
        theme.accent
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
            Ok(Box::new(RarustApp::new(archive, locale, font_setup, pwd)))
        }),
    )
    .map_err(|e| RarustError::Unsupported(format!("GUI failed to start: {e}")))
}
