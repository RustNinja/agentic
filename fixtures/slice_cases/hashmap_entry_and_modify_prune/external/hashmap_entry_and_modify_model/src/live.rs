use std::collections::HashMap;

#[derive(Clone, Debug, Eq, PartialEq, Hash)]
pub struct HashmapEntryAndModifyKey {
    value: &'static str,
}

impl HashmapEntryAndModifyKey {
    pub fn live() -> Self {
        Self { value: "live" }
    }
}

pub struct HashmapEntryAndModifyPayload {
    value: String,
    touched: bool,
}

impl HashmapEntryAndModifyPayload {
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
        format!("hashmap-entry-and-modify:{}{}", self.value, suffix)
    }

    pub fn dead_method(&self) -> String {
        format!("dead-hashmap-entry-and-modify:{}", self.value)
    }
}

fn hashmap_entry_and_modify_entries(
) -> HashMap<HashmapEntryAndModifyKey, HashmapEntryAndModifyPayload> {
    HashMap::new()
}

pub fn selected_hashmap_entry_and_modify(raw: &str) -> String {
    let mut entries = hashmap_entry_and_modify_entries();
    entries.insert(
        HashmapEntryAndModifyKey::live(),
        HashmapEntryAndModifyPayload::new(raw),
    );
    entries
        .entry(HashmapEntryAndModifyKey::live())
        .and_modify(|payload| {
            payload.mark_live();
        })
        .or_insert(HashmapEntryAndModifyPayload::new("fallback"))
        .render_label()
}

pub fn dead_live_hashmap_entry_and_modify(raw: &str) -> String {
    HashmapEntryAndModifyPayload::new(raw).dead_method()
}
