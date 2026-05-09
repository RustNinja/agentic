#[derive(Clone)]
pub struct IteratorUnzipPairMapLeft {
    value: String,
}

impl IteratorUnzipPairMapLeft {
    pub fn new(raw: &str) -> Self {
        Self {
            value: raw.trim().to_string(),
        }
    }

    pub fn render_left(&self) -> String {
        format!("iterator-unzip-pair-map-left:{}", self.value)
    }

    pub fn dead_method(&self) -> String {
        format!("dead-iterator-unzip-pair-map-left:{}", self.value)
    }
}

#[derive(Clone)]
pub struct IteratorUnzipPairMapRight {
    value: String,
}

impl IteratorUnzipPairMapRight {
    pub fn new(raw: &str) -> Self {
        Self {
            value: raw.trim().to_string(),
        }
    }

    pub fn render_right(&self) -> String {
        format!("iterator-unzip-pair-map-right:{}", self.value)
    }

    pub fn dead_method(&self) -> String {
        format!("dead-iterator-unzip-pair-map-right:{}", self.value)
    }
}

pub fn selected_iterator_unzip_pair_map(raw: &str) -> String {
    let pairs = vec![(
        IteratorUnzipPairMapLeft::new(raw),
        IteratorUnzipPairMapRight::new(raw),
    )];
    let (left, right): (Vec<_>, Vec<_>) = pairs.into_iter().unzip();
    let left = left
        .into_iter()
        .map(|payload| payload.render_left())
        .next()
        .unwrap_or_else(|| format!("iterator-unzip-pair-map:left-missing"));
    let right = right
        .into_iter()
        .map(|payload| payload.render_right())
        .next()
        .unwrap_or_else(|| format!("iterator-unzip-pair-map:right-missing"));
    format!("{left}|{right}")
}

pub fn dead_live_iterator_unzip_pair_map(raw: &str) -> String {
    format!(
        "{}|{}",
        IteratorUnzipPairMapLeft::new(raw).dead_method(),
        IteratorUnzipPairMapRight::new(raw).dead_method()
    )
}
