use std::collections::HashMap;
pub struct HashmapShrinkToFitValuesMapPayload {
    value: String,
}

impl HashmapShrinkToFitValuesMapPayload {
    pub fn new(raw: &str) -> Self {
        Self {
            value: raw.trim().to_string(),
        }
    }

    pub fn render_label(&self) -> String {
        format!("hashmap-shrink-to-fit-values-map:{}", self.value)
    }

    pub fn bump_and_render(&mut self) -> String {
        self.value.push_str(":mut");
        format!("hashmap-shrink-to-fit-values-map:{}", self.value)
    }

    pub fn unused_label(&self) -> String {
        format!("dead-hashmap-shrink-to-fit-values-map:{}", self.value)
    }
}

pub fn selected_hashmap_shrink_to_fit_values_map(raw: &str) -> String {
    let mut values = HashMap::new();
    values.insert(raw.to_string(), HashmapShrinkToFitValuesMapPayload::new(raw));
    values.shrink_to_fit();
    values
        .values()
        .map(|payload| payload.render_label())
        .next()
        .unwrap_or_default()
}

pub fn dead_live_hashmap_shrink_to_fit_values_map(raw: &str) -> String {
    HashmapShrinkToFitValuesMapPayload::new(raw).unused_label()
}
