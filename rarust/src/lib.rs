//! rarust frontends — CLI (`rarust`) and GUI (`rarust-gui`) for rarust-core.

pub mod cli;
pub mod commands;

#[cfg(feature = "gui")]
pub mod gui;
