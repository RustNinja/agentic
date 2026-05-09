use std::cmp::Ordering;

#[derive(Clone, Debug)]
pub struct CollectAnnotatedOptionVecSource {
    value: String,
}

impl CollectAnnotatedOptionVecSource {
    pub fn new(raw: &str) -> Self {
        Self { value: raw.trim().to_string() }
    }

    pub fn live() -> Self {
        Self::new("live")
    }

    pub fn key_seed(&self) -> String {
        format!("key:{}", self.value)
    }

    pub fn value_seed(&self) -> String {
        format!("value:{}", self.value)
    }

    pub fn is_live_source(&self) -> bool {
        self.value.contains("live")
    }

    pub fn unused_source_helper(&self) -> String {
        format!("unused-source-collect-annotated-option-vec:{}", self.value)
    }
}

#[derive(Clone, Debug)]
pub struct CollectAnnotatedOptionVecItem {
    value: String,
}

impl CollectAnnotatedOptionVecItem {
    pub fn from_source(source: &CollectAnnotatedOptionVecSource) -> Self {
        Self { value: source.value_seed() }
    }

    pub fn render_label(&self) -> String {
        format!("collect-annotated-option-vec:{}", self.value)
    }

    pub fn compare(&self, other: &Self) -> Ordering {
        self.value.cmp(&other.value)
    }

    pub fn unused_helper(&self) -> String {
        format!("unused-collect-annotated-option-vec:{}", self.value)
    }

    pub fn dead_method(&self) -> String {
        format!("dead-collect-annotated-option-vec:{}", self.value)
    }
}

fn collect_annotated_option_vec_sources(raw: &str) -> Vec<CollectAnnotatedOptionVecSource> {
    vec![CollectAnnotatedOptionVecSource::live(), CollectAnnotatedOptionVecSource::new(raw)]
}

fn collect_annotated_option_vec_convert(source: CollectAnnotatedOptionVecSource) -> Option<CollectAnnotatedOptionVecItem> {
    source.is_live_source().then(|| CollectAnnotatedOptionVecItem::from_source(&source))
}

pub fn selected_collect_annotated_option_vec(raw: &str) -> String {
    let collected: Option<Vec<CollectAnnotatedOptionVecItem>> = collect_annotated_option_vec_sources(raw)
        .into_iter()
        .map(collect_annotated_option_vec_convert)
        .collect();
    collected
        .map(|items| items.into_iter().map(|item| item.render_label()).collect::<Vec<_>>().join("|"))
        .unwrap_or_else(|| String::from("missing"))
}

pub fn dead_live_collect_annotated_option_vec(raw: &str) -> String {
    CollectAnnotatedOptionVecItem::from_source(&CollectAnnotatedOptionVecSource::new(raw)).dead_method()
}
