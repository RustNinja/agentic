use std::cmp::Ordering;

#[derive(Clone, Debug)]
pub struct CollectAnnotatedResultVecSource {
    value: String,
}

impl CollectAnnotatedResultVecSource {
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
        format!("unused-source-collect-annotated-result-vec:{}", self.value)
    }
}

#[derive(Clone, Debug)]
pub struct CollectAnnotatedResultVecItem {
    value: String,
}

impl CollectAnnotatedResultVecItem {
    pub fn from_source(source: &CollectAnnotatedResultVecSource) -> Self {
        Self { value: source.value_seed() }
    }

    pub fn render_label(&self) -> String {
        format!("collect-annotated-result-vec:{}", self.value)
    }

    pub fn compare(&self, other: &Self) -> Ordering {
        self.value.cmp(&other.value)
    }

    pub fn unused_helper(&self) -> String {
        format!("unused-collect-annotated-result-vec:{}", self.value)
    }

    pub fn dead_method(&self) -> String {
        format!("dead-collect-annotated-result-vec:{}", self.value)
    }
}

#[derive(Clone, Debug)]
pub struct CollectAnnotatedResultVecError {
    value: String,
}

impl CollectAnnotatedResultVecError {
    pub fn from_source(source: &CollectAnnotatedResultVecSource) -> Self {
        Self { value: source.key_seed() }
    }

    pub fn render_error(&self) -> String {
        format!("error-collect-annotated-result-vec:{}", self.value)
    }

    pub fn dead_error_method(&self) -> String {
        format!("dead-error-collect-annotated-result-vec:{}", self.value)
    }
}

fn collect_annotated_result_vec_sources(raw: &str) -> Vec<CollectAnnotatedResultVecSource> {
    vec![CollectAnnotatedResultVecSource::live(), CollectAnnotatedResultVecSource::new(raw)]
}

fn collect_annotated_result_vec_convert(source: CollectAnnotatedResultVecSource) -> Result<CollectAnnotatedResultVecItem, CollectAnnotatedResultVecError> {
    if source.is_live_source() {
        Ok(CollectAnnotatedResultVecItem::from_source(&source))
    } else {
        Err(CollectAnnotatedResultVecError::from_source(&source))
    }
}

pub fn selected_collect_annotated_result_vec(raw: &str) -> String {
    let collected: Result<Vec<CollectAnnotatedResultVecItem>, CollectAnnotatedResultVecError> = collect_annotated_result_vec_sources(raw)
        .into_iter()
        .map(collect_annotated_result_vec_convert)
        .collect();
    collected
        .map(|items| items.into_iter().map(|item| item.render_label()).collect::<Vec<_>>().join("|"))
        .unwrap_or_else(|err| err.render_error())
}

pub fn dead_live_collect_annotated_result_vec(raw: &str) -> String {
    CollectAnnotatedResultVecItem::from_source(&CollectAnnotatedResultVecSource::new(raw)).dead_method()
}
