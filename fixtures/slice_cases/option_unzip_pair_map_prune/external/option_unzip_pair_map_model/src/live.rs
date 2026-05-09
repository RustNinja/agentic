#[derive(Clone)]
pub struct OptionUnzipPairMapLeft {
    value: String,
}

impl OptionUnzipPairMapLeft {
    pub fn new(raw: &str) -> Self {
        Self {
            value: raw.trim().to_string(),
        }
    }

    pub fn render_left(&self) -> String {
        format!("option-unzip-pair-map-left:{}", self.value)
    }

    pub fn dead_method(&self) -> String {
        format!("dead-option-unzip-pair-map-left:{}", self.value)
    }
}

#[derive(Clone)]
pub struct OptionUnzipPairMapRight {
    value: String,
}

impl OptionUnzipPairMapRight {
    pub fn new(raw: &str) -> Self {
        Self {
            value: raw.trim().to_string(),
        }
    }

    pub fn render_right(&self) -> String {
        format!("option-unzip-pair-map-right:{}", self.value)
    }

    pub fn dead_method(&self) -> String {
        format!("dead-option-unzip-pair-map-right:{}", self.value)
    }
}

pub fn selected_option_unzip_pair_map(raw: &str) -> String {
    let pair = Some((
        OptionUnzipPairMapLeft::new(raw),
        OptionUnzipPairMapRight::new(raw),
    ));
    let (left, right) = pair.unzip();
    let left = left
        .map(|payload| payload.render_left())
        .unwrap_or_else(|| format!("option-unzip-pair-map:left-missing"));
    let right = right
        .map(|payload| payload.render_right())
        .unwrap_or_else(|| format!("option-unzip-pair-map:right-missing"));
    format!("{left}|{right}")
}

pub fn dead_live_option_unzip_pair_map(raw: &str) -> String {
    format!(
        "{}|{}",
        OptionUnzipPairMapLeft::new(raw).dead_method(),
        OptionUnzipPairMapRight::new(raw).dead_method()
    )
}
