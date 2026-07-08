//! Graphical user interface (egui) for rarust.
//!
//! Feature-gated behind `gui`. Supports multi-language UI with CJK font loading
//! for correct Chinese character rendering.

pub mod actions;
mod app;
mod fonts;
mod i18n;
mod icons;
mod jobs;
pub mod theme;
pub mod widgets;

pub use app::run_gui;
pub use i18n::Locale;
