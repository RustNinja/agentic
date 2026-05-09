pub struct DeadCowOwnedIntoOwnedMapItem {
    value: String,
}

impl DeadCowOwnedIntoOwnedMapItem {
    pub fn new(raw: &str) -> Self {
        Self {
            value: raw.to_string(),
        }
    }

    pub fn render(&self) -> String {
        format!("dead-cow-owned-into-owned-map:{}", self.value)
    }
}

pub fn dead_cow_owned_into_owned_map(raw: &str) -> String {
    DeadCowOwnedIntoOwnedMapItem::new(raw).render()
}
