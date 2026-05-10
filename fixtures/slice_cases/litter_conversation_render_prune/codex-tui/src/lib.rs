#![allow(dead_code)]

pub mod screens;
pub mod theme;

pub fn dead_tui_entry() -> String {
    theme::dead_palette_name().to_string()
}

