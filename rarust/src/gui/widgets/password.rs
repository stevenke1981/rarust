//! Password prompt dialog with session-scoped caching.

use std::collections::HashMap;

use egui::{Button, Color32, RichText, TextEdit};

/// Modal password prompt for encrypted archives.
pub struct PasswordDialog {
    pub visible: bool,
    password: String,
    show_password: bool,
    remember_session: bool,
    error: Option<String>,
    /// Cache: archive_path → password (session scope).
    cache: HashMap<String, String>,
}

impl PasswordDialog {
    /// Create a new hidden password dialog.
    pub fn new() -> Self {
        Self {
            visible: false,
            password: String::new(),
            show_password: false,
            remember_session: false,
            error: None,
            cache: HashMap::new(),
        }
    }

    /// Return cached password for an archive, if any.
    pub fn cached(&self, archive_path: &str) -> Option<String> {
        self.cache.get(archive_path).cloned()
    }

    /// Prompt the user for a password. If cached, does nothing (caller must check `cached()` first).
    pub fn prompt(&mut self, archive_path: &str) {
        if self.cache.contains_key(archive_path) {
            return;
        }
        self.visible = true;
        self.password.clear();
        self.error = None;
    }

    /// Render the modal dialog. Returns `Some(password)` when user confirms.
    pub fn show(&mut self, ctx: &egui::Context, archive_path: &str) -> Option<String> {
        if !self.visible {
            return None;
        }

        let mut result = None;

        egui::Window::new("Archive Password")
            .collapsible(false)
            .movable(false)
            .resizable(false)
            .anchor(egui::Align2::CENTER_CENTER, [0.0, 0.0])
            .show(ctx, |ui| {
                ui.set_min_width(360.0);
                ui.label(RichText::new(format!("Enter password for:\n{archive_path}")).strong());

                ui.add_space(8.0);

                let resp = ui.add_sized(
                    [320.0, 28.0],
                    TextEdit::singleline(&mut self.password)
                        .password(!self.show_password)
                        .hint_text("Password"),
                );
                if !resp.has_focus() && self.password.is_empty() {
                    resp.request_focus();
                }

                ui.checkbox(&mut self.show_password, "Show password");
                ui.checkbox(&mut self.remember_session, "Remember for this session");

                if let Some(err) = &self.error {
                    ui.colored_label(Color32::from_rgb(224, 80, 64), err);
                }

                ui.add_space(8.0);
                ui.horizontal(|ui| {
                    if ui.button("Cancel").clicked() {
                        self.visible = false;
                    }
                    if ui.add_sized([100.0, 28.0], Button::new("OK")).clicked()
                        || (ui.input(|i| i.key_pressed(egui::Key::Enter))
                            && !self.password.is_empty())
                    {
                        if self.password.is_empty() {
                            self.error = Some("Password cannot be empty".to_string());
                        } else {
                            let pwd = self.password.clone();
                            if self.remember_session {
                                self.cache.insert(archive_path.to_owned(), pwd.clone());
                            }
                            result = Some(pwd);
                            self.visible = false;
                        }
                    }
                });
            });

        result
    }

    /// Clear the password cache (e.g., on "Close Archive").
    pub fn clear_cache(&mut self) {
        self.cache.clear();
    }
}

impl Default for PasswordDialog {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn cache_hit_returns_stored_password() {
        let mut dlg = PasswordDialog::new();
        // Simulate entering a password with remember_session
        dlg.password = "secret".to_string();
        dlg.remember_session = true;
        // Simulate the OK flow
        let pwd = dlg.password.clone();
        dlg.cache.insert("/path/to/archive.rar".to_string(), pwd);

        assert_eq!(
            dlg.cached("/path/to/archive.rar"),
            Some("secret".to_string())
        );
    }

    #[test]
    fn cache_miss_returns_none() {
        let dlg = PasswordDialog::new();
        assert_eq!(dlg.cached("/unknown.rar"), None);
    }

    #[test]
    fn prompt_does_not_override_cache() {
        let mut dlg = PasswordDialog::new();
        dlg.cache
            .insert("/cached.rar".to_string(), "existing".to_string());
        dlg.prompt("/cached.rar");
        assert!(!dlg.visible);
    }

    #[test]
    fn clear_cache_empties_all_entries() {
        let mut dlg = PasswordDialog::new();
        dlg.cache.insert("/a.rar".to_string(), "p1".to_string());
        dlg.cache.insert("/b.rar".to_string(), "p2".to_string());
        dlg.clear_cache();
        assert!(dlg.cache.is_empty());
    }
}
