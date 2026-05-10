pub struct DeadHashmapShrinkToFitValuesMapItem {
    value: String,
}

impl DeadHashmapShrinkToFitValuesMapItem {
    pub fn new(raw: &str) -> Self {
        Self {
            value: raw.to_string(),
        }
    }

    pub fn dead_method(&self) -> String {
        format!("dead-hashmap-shrink-to-fit-values-map:{}", self.value)
    }
}

pub fn dead_hashmap_shrink_to_fit_values_map(raw: &str) -> String {
    DeadHashmapShrinkToFitValuesMapItem::new(raw).dead_method()
}
