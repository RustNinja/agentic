pub struct DeadHashsetShrinkToFitIterMapItem {
    value: String,
}

impl DeadHashsetShrinkToFitIterMapItem {
    pub fn new(raw: &str) -> Self {
        Self {
            value: raw.to_string(),
        }
    }

    pub fn dead_method(&self) -> String {
        format!("dead-hashset-shrink-to-fit-iter-map:{}", self.value)
    }
}

pub fn dead_hashset_shrink_to_fit_iter_map(raw: &str) -> String {
    DeadHashsetShrinkToFitIterMapItem::new(raw).dead_method()
}
