#![allow(dead_code)]

pub use external_leaf::prelude::*;

pub fn selected_leaf_report(raw: &str) -> String {
    let record = LeafRecord {
        label: raw.trim().to_string(),
    };
    format!("leaf:{}", record.label)
}

pub fn dead_leaf_report(raw: &str) -> String {
    let record = DeadLeaf {
        label: raw.trim().to_string(),
    };
    format!("dead:{}", record.label)
}
