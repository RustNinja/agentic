pub struct DeadVecShrinkToFitLastMapItem {
    value: String,
}

impl DeadVecShrinkToFitLastMapItem {
    pub fn new(raw: &str) -> Self {
        Self {
            value: raw.to_string(),
        }
    }

    pub fn dead_method(&self) -> String {
        format!("dead-vec-shrink-to-fit-last-map:{}", self.value)
    }
}

pub fn dead_vec_shrink_to_fit_last_map(raw: &str) -> String {
    DeadVecShrinkToFitLastMapItem::new(raw).dead_method()
}
