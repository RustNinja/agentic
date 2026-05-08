pub struct FlatSegment {
    value: String,
}

impl FlatSegment {
    pub fn new(raw: &str) -> Self {
        Self {
            value: raw.trim().to_string(),
        }
    }

    pub fn expand(&self) -> Vec<String> {
        vec![
            format!("flat:{}", self.value),
            format!("flat-len:{}", self.value.len()),
        ]
    }

    pub fn dead_method(&self) -> String {
        format!("dead-flat:{}", self.value)
    }
}

fn build_segments(raw: &str) -> Vec<FlatSegment> {
    raw.split(',').map(FlatSegment::new).collect()
}

pub fn selected_flat_map(raw: &str) -> String {
    build_segments(raw)
        .iter()
        .flat_map(|segment| segment.expand())
        .collect::<Vec<_>>()
        .join("|")
}

pub fn dead_live_flat_map(raw: &str) -> String {
    FlatSegment::new(raw).dead_method()
}
