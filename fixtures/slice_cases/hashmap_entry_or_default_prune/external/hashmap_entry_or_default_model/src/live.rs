use std::collections::HashMap;

#[derive(Clone, Debug, Eq, PartialEq, Hash)]
pub struct HashmapEntryOrDefaultKey {
    value: &'static str,
}

impl HashmapEntryOrDefaultKey {
    pub fn live() -> Self {
        Self { value: "live" }
    }
}

pub struct HashmapEntryOrDefaultPayload {
    value: String,
    touched: bool,
}

impl HashmapEntryOrDefaultPayload {
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
        format!("hashmap-entry-or-default:{}{}", self.value, suffix)
    }

    pub fn dead_method(&self) -> String {
        format!("dead-hashmap-entry-or-default:{}", self.value)
    }
}

impl Default for HashmapEntryOrDefaultPayload {
    fn default() -> Self {
        Self::new("default")
    }
}

fn hashmap_entry_or_default_entries(
) -> HashMap<HashmapEntryOrDefaultKey, HashmapEntryOrDefaultPayload> {
    HashMap::new()
}

pub fn selected_hashmap_entry_or_default(_raw: &str) -> String {
    let mut entries = hashmap_entry_or_default_entries();
    entries
        .entry(HashmapEntryOrDefaultKey::live())
        .or_default()
        .render_label()
}

pub fn dead_live_hashmap_entry_or_default(raw: &str) -> String {
    HashmapEntryOrDefaultPayload::new(raw).dead_method()
}
