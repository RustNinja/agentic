#![allow(dead_code)]

pub mod ffi;
pub mod settings;

pub fn dead_mobile_entry() -> String {
    settings::dead_settings().to_string()
}

