use std::ops::Index;

pub struct IndexItem {
    label: String,
}

impl IndexItem {
    pub fn render(&self) -> String {
        format!("index:{}", self.label)
    }

    pub fn dead_method(&self) -> String {
        format!("dead-index:{}", self.label)
    }
}

pub struct IndexStore {
    items: Vec<IndexItem>,
}

impl IndexStore {
    pub fn new(raw: &str) -> Self {
        Self {
            items: vec![IndexItem {
                label: raw.trim().to_string(),
            }],
        }
    }

    pub fn dead_store_method(&self) -> String {
        self.items[0].dead_method()
    }
}

impl Index<usize> for IndexStore {
    type Output = IndexItem;

    fn index(&self, index: usize) -> &Self::Output {
        &self.items[index]
    }
}

pub fn selected_index(raw: &str) -> String {
    let store = IndexStore::new(raw);
    store[0].render()
}

pub fn dead_live_index(raw: &str) -> String {
    IndexStore::new(raw).dead_store_method()
}
