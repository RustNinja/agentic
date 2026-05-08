pub struct TupleSortKey {
    value: String,
}

impl TupleSortKey {
    pub fn new(raw: &str) -> Self {
        Self {
            value: raw.trim().to_string(),
        }
    }

    pub fn rank(&self, value: &TupleSortValue) -> usize {
        self.value.len() + value.render_label().len()
    }

    pub fn render_with(&self, value: &TupleSortValue) -> String {
        format!("tuple-sort:{}:{}", self.value, value.render_label())
    }

    pub fn dead_method(&self) -> String {
        format!("dead-tuple-sort-key:{}", self.value)
    }
}

pub struct TupleSortValue {
    value: String,
}

impl TupleSortValue {
    pub fn new(raw: &str) -> Self {
        Self {
            value: raw.trim().to_string(),
        }
    }

    pub fn render_label(&self) -> String {
        format!("sort-value:{}", self.value)
    }

    pub fn dead_method(&self) -> String {
        format!("dead-tuple-sort-value:{}", self.value)
    }
}

fn tuple_sort_key_entries(raw: &str) -> Vec<(TupleSortKey, TupleSortValue)> {
    raw.split(',')
        .map(|part| (TupleSortKey::new(part), TupleSortValue::new(part)))
        .collect()
}

pub fn selected_tuple_sort_by_key(raw: &str) -> String {
    let mut entries = tuple_sort_key_entries(raw);
    entries.sort_by_key(|(key, value)| key.rank(value));
    entries
        .iter()
        .map(|(key, value)| key.render_with(value))
        .collect::<Vec<_>>()
        .join("|")
}

pub fn dead_live_tuple_sort_by_key(raw: &str) -> String {
    TupleSortKey::new(raw).dead_method()
}
