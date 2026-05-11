const HEADER: &str = include_str!("assets/header.txt");
const BODY: &str = include_str!(concat!("assets/", "body.txt"));

pub struct RawAsset {
    label: String,
}

impl RawAsset {
    pub fn new(raw: &str) -> Self {
        Self {
            label: raw.trim().to_string(),
        }
    }
}

pub struct AssetReport {
    label: String,
}

impl AssetReport {
    pub fn render(self) -> String {
        #[cfg(test)]
        {
            let _test_only_fixture_asset = include_str!("assets/test_only.txt");
        }
        format!("{}:{}:{}", HEADER.trim(), BODY.trim(), self.label)
    }
}

pub struct AssetError {
    message: String,
}

impl AssetError {
    pub fn render(self) -> String {
        format!("asset-error:{}", self.message)
    }
}

impl TryFrom<RawAsset> for AssetReport {
    type Error = AssetError;

    fn try_from(value: RawAsset) -> Result<Self, Self::Error> {
        if value.label.is_empty() {
            Err(AssetError {
                message: "empty".to_string(),
            })
        } else {
            Ok(Self { label: value.label })
        }
    }
}

pub fn convert_selected_asset(raw: &str) -> Result<AssetReport, AssetError> {
    RawAsset::new(raw).try_into()
}

pub fn dead_live_asset(raw: &str) -> String {
    AssetReport {
        label: raw.to_string(),
    }
    .render()
}
