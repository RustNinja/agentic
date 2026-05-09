pub struct DeadPhantomDataSurfaceMapItem {
    value: String,
}

pub struct DeadPhantomDataSurfaceMapMarker;

impl DeadPhantomDataSurfaceMapItem {
    pub fn new(raw: &str) -> Self {
        Self {
            value: raw.to_string(),
        }
    }

    pub fn render(&self) -> String {
        format!("dead-phantomdata-surface-map:{}", self.value)
    }
}

pub fn dead_phantomdata_surface_map(raw: &str) -> String {
    DeadPhantomDataSurfaceMapItem::new(raw).render()
}
