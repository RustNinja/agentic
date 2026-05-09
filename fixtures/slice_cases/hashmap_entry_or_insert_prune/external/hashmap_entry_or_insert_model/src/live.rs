use std::collections::HashMap;

#[derive(Clone, Debug, Eq, PartialEq, Hash)]
pub struct HashmapEntryOrInsertKey {
    value: &'static str,
}

impl HashmapEntryOrInsertKey {
    pub fn live() -> Self {
        Self { value: "live" }
    }
}

pub struct HashmapEntryOrInsertPayload {
    value: String,
    touched: bool,
}

impl HashmapEntryOrInsertPayload {
    pub fn new(raw: &str) -> Self {
        Self {
            value: raw.trim().to_string(),
            touched: false,
        }
    }

    pub fn mark_live(&mut self) -> &mut Self {
        self.touched = true;
        self
    }

    pub fn render_label(&self) -> String {
        let suffix = if self.touched { ":touched" } else { "" };
        format!("hashmap-entry-or-insert:{}{}", self.value, suffix)
    }

    pub fn dead_method(&self) -> String {
        format!("dead-hashmap-entry-or-insert:{}", self.value)
    }
}

fn hashmap_entry_or_insert_entries() -> HashMap<HashmapEntryOrInsertKey, HashmapEntryOrInsertPayload>
{
    HashMap::new()
}

pub fn selected_hashmap_entry_or_insert(raw: &str) -> String {
    let mut entries = hashmap_entry_or_insert_entries();
    entries
        .entry(HashmapEntryOrInsertKey::live())
        .or_insert(HashmapEntryOrInsertPayload::new(raw))
        .render_label()
}

pub fn dead_live_hashmap_entry_or_insert(raw: &str) -> String {
    HashmapEntryOrInsertPayload::new(raw).dead_method()
}
