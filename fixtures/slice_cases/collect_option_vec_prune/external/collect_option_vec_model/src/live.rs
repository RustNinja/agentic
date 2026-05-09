use std::cmp::Ordering;

#[derive(Clone, Debug)]
pub struct CollectOptionVecSource {
    value: String,
}

impl CollectOptionVecSource {
    pub fn new(raw: &str) -> Self {
        Self {
            value: raw.trim().to_string(),
        }
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
        format!("unused-source-collect-option-vec:{}", self.value)
    }
}

#[derive(Clone, Debug)]
pub struct CollectOptionVecItem {
    value: String,
}

impl CollectOptionVecItem {
    pub fn from_source(source: &CollectOptionVecSource) -> Self {
        Self {
            value: source.value_seed(),
        }
    }

    pub fn render_label(&self) -> String {
        format!("collect-option-vec:{}", self.value)
    }

    pub fn compare(&self, other: &Self) -> Ordering {
        self.value.cmp(&other.value)
    }

    pub fn unused_helper(&self) -> String {
        format!("unused-collect-option-vec:{}", self.value)
    }

    pub fn dead_method(&self) -> String {
        format!("dead-collect-option-vec:{}", self.value)
    }
}

fn collect_option_vec_sources(raw: &str) -> Vec<CollectOptionVecSource> {
    vec![CollectOptionVecSource::live(), CollectOptionVecSource::new(raw)]
}

fn collect_option_vec_convert(source: CollectOptionVecSource) -> Option<CollectOptionVecItem> {
    source.is_live_source().then(|| CollectOptionVecItem::from_source(&source))
}

pub fn selected_collect_option_vec(raw: &str) -> String {
    collect_option_vec_sources(raw)
        .into_iter()
        .map(collect_option_vec_convert)
        .collect::<Option<Vec<CollectOptionVecItem>>>()
        .map(|items| items.into_iter().map(|item| item.render_label()).collect::<Vec<_>>().join("|"))
        .unwrap_or_else(|| String::from("missing"))
}

pub fn dead_live_collect_option_vec(raw: &str) -> String {
    CollectOptionVecItem::from_source(&CollectOptionVecSource::new(raw)).dead_method()
}
