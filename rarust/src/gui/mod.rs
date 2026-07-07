//! Graphical user interface (egui) for rarust.
//!
//! Feature-gated behind `gui`. Supports multi-language UI with CJK font loading
//! for correct Chinese character rendering.

mod app;
mod fonts;
mod i18n;

pub use app::run_gui;
pub use i18n::Locale;
