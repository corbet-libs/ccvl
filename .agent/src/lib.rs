//! Deterministic core for ccvl workspaces.

pub mod application;
pub mod check;
pub mod cli;
pub mod content;
pub mod downstream;
pub mod format;
pub mod measure;
pub mod opportunity;
pub mod ownership;
pub mod paper;
pub mod pdf;
pub mod public;
pub mod render;
pub mod review;
pub mod runtime;
mod runtime_source;
pub mod settings;
pub mod skills;
pub mod stations;
pub mod styles;
pub mod workspace;

pub use workspace::Workspace;

mod watch;
