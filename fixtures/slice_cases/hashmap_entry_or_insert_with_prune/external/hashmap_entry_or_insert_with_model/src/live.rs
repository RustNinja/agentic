use std::collections::HashMap;

#[derive(Clone, Debug, Eq, PartialEq, Hash)]
pub struct HashmapEntryOrInsertWithKey {
    value: &'static str,
}

impl HashmapEntryOrInsertWithKey {
    pub fn live() -> Self {
        Self { value: "live" }
    }
}

pub struct HashmapEntryOrInsertWithPayload {
    value: String,
    touched: bool,
}

impl HashmapEntryOrInsertWithPayload {
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
        format!("hashmap-entry-or-insert-with:{}{}", self.value, suffix)
    }

    pub fn dead_method(&self) -> String {
        format!("dead-hashmap-entry-or-insert-with:{}", self.value)
    }
}

fn hashmap_entry_or_insert_with_entries(
) -> HashMap<HashmapEntryOrInsertWithKey, HashmapEntryOrInsertWithPayload> {
    HashMap::new()
}

pub fn selected_hashmap_entry_or_insert_with(raw: &str) -> String {
    let mut entries = hashmap_entry_or_insert_with_entries();
    entries
        .entry(HashmapEntryOrInsertWithKey::live())
        .or_insert_with(|| HashmapEntryOrInsertWithPayload::new(raw))
        .render_label()
}

pub fn dead_live_hashmap_entry_or_insert_with(raw: &str) -> String {
    HashmapEntryOrInsertWithPayload::new(raw).dead_method()
}
