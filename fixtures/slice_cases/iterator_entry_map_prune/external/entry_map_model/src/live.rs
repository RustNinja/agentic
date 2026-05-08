use std::collections::BTreeMap;

#[derive(Eq, Ord, PartialEq, PartialOrd)]
pub struct EntryMapKey {
    value: String,
}

impl EntryMapKey {
    pub fn new(raw: &str) -> Self {
        Self {
            value: raw.trim().to_string(),
        }
    }

    pub fn render_entry(&self, value: &EntryMapValue) -> String {
        format!("entry-map:{}:{}", self.value, value.render_label())
    }

    pub fn dead_method(&self) -> String {
        format!("dead-entry-map-key:{}", self.value)
    }
}

pub struct EntryMapValue {
    value: String,
}

impl EntryMapValue {
    pub fn new(raw: &str) -> Self {
        Self {
            value: raw.trim().to_string(),
        }
    }

    pub fn render_label(&self) -> String {
        format!("entry-value:{}", self.value)
    }

    pub fn dead_method(&self) -> String {
        format!("dead-entry-map-value:{}", self.value)
    }
}

fn entry_map_entries(raw: &str) -> BTreeMap<EntryMapKey, EntryMapValue> {
    raw.split(',')
        .map(|part| (EntryMapKey::new(part), EntryMapValue::new(part)))
        .collect()
}

pub fn selected_entry_map(raw: &str) -> String {
    let entries = entry_map_entries(raw);
    entries
        .iter()
        .map(|(key, value)| key.render_entry(value))
        .collect::<Vec<_>>()
        .join("|")
}

pub fn dead_live_entry_map(raw: &str) -> String {
    EntryMapKey::new(raw).dead_method()
}
