use std::collections::HashMap;

#[derive(Clone, Debug, Eq, PartialEq, Hash)]
pub struct HashmapEntryOrInsertWithKeyKey {
    value: &'static str,
}

impl HashmapEntryOrInsertWithKeyKey {
    pub fn live() -> Self {
        Self { value: "live" }
    }
}

pub struct HashmapEntryOrInsertWithKeyPayload {
    value: String,
    touched: bool,
}

impl HashmapEntryOrInsertWithKeyPayload {
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
        format!("hashmap-entry-or-insert-with-key:{}{}", self.value, suffix)
    }

    pub fn dead_method(&self) -> String {
        format!("dead-hashmap-entry-or-insert-with-key:{}", self.value)
    }
}

fn hashmap_entry_or_insert_with_key_entries(
) -> HashMap<HashmapEntryOrInsertWithKeyKey, HashmapEntryOrInsertWithKeyPayload> {
    HashMap::new()
}

pub fn selected_hashmap_entry_or_insert_with_key(raw: &str) -> String {
    let mut entries = hashmap_entry_or_insert_with_key_entries();
    entries
        .entry(HashmapEntryOrInsertWithKeyKey::live())
        .or_insert_with_key(|_| HashmapEntryOrInsertWithKeyPayload::new(raw))
        .render_label()
}

pub fn dead_live_hashmap_entry_or_insert_with_key(raw: &str) -> String {
    HashmapEntryOrInsertWithKeyPayload::new(raw).dead_method()
}
