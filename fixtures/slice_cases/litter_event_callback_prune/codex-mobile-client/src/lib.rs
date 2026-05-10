#![allow(dead_code)]

pub mod diagnostics;
pub mod ffi;

pub fn dead_mobile_event_entry() -> String {
    diagnostics::dead_diagnostics_panel().to_string()
}

