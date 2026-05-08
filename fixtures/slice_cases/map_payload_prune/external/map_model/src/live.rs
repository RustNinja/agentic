use std::collections::BTreeMap;

pub struct MapPayload {
    values: BTreeMap<String, MapEntry>,
}

impl MapPayload {
    pub fn render(&self) -> String {
        self.values
            .values()
            .map(MapEntry::render)
            .collect::<Vec<_>>()
            .join(",")
    }

    pub fn dead_method(&self) -> String {
        format!("dead-map:{}", self.values.len())
    }
}

pub struct MapEntry {
    label: String,
}

impl MapEntry {
    pub fn new(raw: &str) -> Self {
        Self {
            label: raw.trim().to_string(),
        }
    }

    pub fn render(&self) -> String {
        format!("entry:{}", self.label)
    }
}

pub fn selected_map(raw: &str) -> String {
    let mut values = BTreeMap::new();
    values.insert("live".to_string(), MapEntry::new(raw));
    MapPayload { values }.render()
}

pub fn dead_live_map(raw: &str) -> String {
    let mut values = BTreeMap::new();
    values.insert("dead".to_string(), MapEntry::new(raw));
    MapPayload { values }.dead_method()
}
