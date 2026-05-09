use std::cmp::Ordering;

#[derive(Clone, Debug)]
pub struct CollectResultVecSource {
    value: String,
}

impl CollectResultVecSource {
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
        format!("unused-source-collect-result-vec:{}", self.value)
    }
}

#[derive(Clone, Debug)]
pub struct CollectResultVecItem {
    value: String,
}

impl CollectResultVecItem {
    pub fn from_source(source: &CollectResultVecSource) -> Self {
        Self {
            value: source.value_seed(),
        }
    }

    pub fn render_label(&self) -> String {
        format!("collect-result-vec:{}", self.value)
    }

    pub fn compare(&self, other: &Self) -> Ordering {
        self.value.cmp(&other.value)
    }

    pub fn unused_helper(&self) -> String {
        format!("unused-collect-result-vec:{}", self.value)
    }

    pub fn dead_method(&self) -> String {
        format!("dead-collect-result-vec:{}", self.value)
    }
}

#[derive(Clone, Debug)]
pub struct CollectResultVecError {
    value: String,
}

impl CollectResultVecError {
    pub fn from_source(source: &CollectResultVecSource) -> Self {
        Self {
            value: source.key_seed(),
        }
    }

    pub fn render_error(&self) -> String {
        format!("error-collect-result-vec:{}", self.value)
    }

    pub fn dead_error_method(&self) -> String {
        format!("dead-error-collect-result-vec:{}", self.value)
    }
}

fn collect_result_vec_sources(raw: &str) -> Vec<CollectResultVecSource> {
    vec![CollectResultVecSource::live(), CollectResultVecSource::new(raw)]
}

fn collect_result_vec_convert(source: CollectResultVecSource) -> Result<CollectResultVecItem, CollectResultVecError> {
    if source.is_live_source() {
        Ok(CollectResultVecItem::from_source(&source))
    } else {
        Err(CollectResultVecError::from_source(&source))
    }
}

pub fn selected_collect_result_vec(raw: &str) -> String {
    collect_result_vec_sources(raw)
        .into_iter()
        .map(collect_result_vec_convert)
        .collect::<Result<Vec<CollectResultVecItem>, CollectResultVecError>>()
        .map(|items| items.into_iter().map(|item| item.render_label()).collect::<Vec<_>>().join("|"))
        .unwrap_or_else(|err| err.render_error())
}

pub fn dead_live_collect_result_vec(raw: &str) -> String {
    CollectResultVecItem::from_source(&CollectResultVecSource::new(raw)).dead_method()
}
