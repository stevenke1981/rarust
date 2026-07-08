//! Tab bar widget for multi-archive browsing.

/// State for a single archive tab.
#[derive(Clone)]
pub struct TabInfo {
    /// Unique identifier (incrementing counter).
    pub id: u64,
    /// Label shown in the tab bar.
    pub label: String,
    /// Path to the archive, if loaded.
    pub archive_path: Option<String>,
}

/// Horizontal tab bar for switching between open archives.
pub struct TabBar {
    /// All open tabs.
    pub tabs: Vec<TabInfo>,
    /// Index of the currently active tab.
    pub active: usize,
    next_id: u64,
}

impl TabBar {
    pub fn new() -> Self {
        Self {
            tabs: Vec::new(),
            active: 0,
            next_id: 1,
        }
    }

    /// Open a new tab with the given archive path.
    /// Returns the new tab ID.
    pub fn open_tab(&mut self, archive_path: Option<String>, label: String) -> u64 {
        let id = self.next_id;
        self.next_id += 1;
        let ti = TabInfo {
            id,
            label,
            archive_path,
        };
        let idx = self.tabs.len();
        self.tabs.push(ti);
        self.active = idx;
        id
    }

    // Close the active tab, return its index if it was open.
    pub fn close_active(&mut self) -> Option<usize> {
        if self.tabs.is_empty() {
            return None;
        }
        let idx = self.active;
        self.tabs.remove(idx);
        if !self.tabs.is_empty() {
            self.active = self.active.min(self.tabs.len() - 1);
        } else {
            self.active = 0;
        }
        Some(idx)
    }

    /// Close a tab by index. Returns true if the tab existed.
    pub fn close_tab(&mut self, idx: usize) -> bool {
        if idx >= self.tabs.len() {
            return false;
        }
        self.tabs.remove(idx);
        if !self.tabs.is_empty() {
            self.active = self.active.saturating_sub(1).min(self.tabs.len() - 1);
        } else {
            self.active = 0;
        }
        true
    }

    /// Switch to a different tab.
    pub fn switch_to(&mut self, idx: usize) {
        if idx < self.tabs.len() {
            self.active = idx;
        }
    }

    /// Render the tab bar. Returns actions the caller should apply.
    pub fn render(&mut self, ui: &mut egui::Ui) -> TabBarAction {
        if self.tabs.is_empty() {
            return TabBarAction::None;
        }

        let mut action = TabBarAction::None;

        ui.horizontal(|ui| {
            // "New Tab" button
            if ui.button("+").clicked() {
                action = TabBarAction::NewTab;
            }

            let mut close_idx = None;
            let mut switch_idx = None;

            for (i, tab) in self.tabs.iter().enumerate() {
                let is_active = i == self.active;
                let text: String = if tab.archive_path.is_some() {
                    truncate_label(&tab.label, 28)
                } else {
                    "+".to_string()
                };

                let resp = ui.selectable_label(is_active, text);
                if resp.clicked() {
                    switch_idx = Some(i);
                }

                // Close button in the tab
                let close_resp = ui.put(
                    egui::Rect::from_min_size(
                        resp.rect.max - egui::Vec2::new(16.0, 16.0),
                        egui::Vec2::splat(16.0),
                    ),
                    egui::Button::new("✕").frame(false),
                );
                if close_resp.clicked() {
                    close_idx = Some(i);
                }
            }

            if let Some(idx) = switch_idx {
                self.switch_to(idx);
            }
            if let Some(idx) = close_idx {
                action = TabBarAction::CloseTab(idx);
            }
        });

        action
    }
}

impl Default for TabBar {
    fn default() -> Self {
        Self::new()
    }
}

/// Action returned from TabBar::render() for the caller to process.
#[derive(Debug)]
pub enum TabBarAction {
    None,
    NewTab,
    CloseTab(usize),
}

fn truncate_label(label: &str, max: usize) -> String {
    if label.chars().count() <= max {
        label.to_owned()
    } else {
        let front: String = label.chars().take(max.saturating_sub(3)).collect();
        format!("{front}...")
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn new_tab_bar_is_empty() {
        let bar = TabBar::new();
        assert!(bar.tabs.is_empty());
        assert_eq!(bar.active, 0);
    }

    #[test]
    fn open_tab_adds_and_activates() {
        let mut bar = TabBar::new();
        let id = bar.open_tab(Some("/test.rar".into()), "test.rar".into());
        assert_eq!(bar.tabs.len(), 1);
        assert_eq!(bar.active, 0);
        assert_eq!(bar.tabs[0].id, id);
        assert_eq!(bar.tabs[0].archive_path.as_deref(), Some("/test.rar"));
    }

    #[test]
    fn open_multiple_tabs_switches_to_last() {
        let mut bar = TabBar::new();
        bar.open_tab(Some("/a.rar".into()), "A".into());
        bar.open_tab(Some("/b.rar".into()), "B".into());
        assert_eq!(bar.tabs.len(), 2);
        assert_eq!(bar.active, 1);
    }

    #[test]
    fn switch_to_valid_tab() {
        let mut bar = TabBar::new();
        bar.open_tab(Some("/a.rar".into()), "A".into());
        bar.open_tab(Some("/b.rar".into()), "B".into());
        bar.switch_to(0);
        assert_eq!(bar.active, 0);
    }

    #[test]
    fn close_tab_removes_and_readjusts_active() {
        let mut bar = TabBar::new();
        bar.open_tab(Some("/a.rar".into()), "A".into());
        bar.open_tab(Some("/b.rar".into()), "B".into());
        bar.close_tab(0);
        assert_eq!(bar.tabs.len(), 1);
        assert_eq!(bar.tabs[0].label, "B");
        assert_eq!(bar.active, 0);
    }

    #[test]
    fn close_active_tab_when_last() {
        let mut bar = TabBar::new();
        bar.open_tab(Some("/a.rar".into()), "A".into());
        bar.close_active();
        assert!(bar.tabs.is_empty());
    }

    #[test]
    fn close_nonexistent_tab_returns_false() {
        let mut bar = TabBar::new();
        assert!(!bar.close_tab(0));
    }

    #[test]
    fn truncate_short_label() {
        assert_eq!(truncate_label("hello", 10), "hello");
    }

    #[test]
    fn truncate_long_label() {
        let long = "abcdefghijklmnopqrstuvwxyz"; // 26 chars
        let t = truncate_label(long, 10);
        assert!(t.chars().count() <= 10);
        assert!(t.ends_with("..."));
    }
}
