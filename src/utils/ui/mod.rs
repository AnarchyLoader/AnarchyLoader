pub mod grouping;
pub mod intro;
pub mod messages;
pub mod native_theme;
pub mod ui_settings;
pub mod widgets;

// Local replacements for crates that required older egui versions
pub mod modal;
// text animator removed; using plain text color instead
// transition module removed: tab animations fully removed
