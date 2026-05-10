#![allow(dead_code)]

pub mod panels;
pub mod wire;

pub fn dead_tui_codegen_entry() -> String {
    panels::dead_codegen_panel().to_string()
}

