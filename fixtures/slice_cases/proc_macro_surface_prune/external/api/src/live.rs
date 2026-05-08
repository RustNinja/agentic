use macro_support::{surface_attr, DeadRecord, SurfaceRecord};
use model::LiveWire;

#[derive(Clone, SurfaceRecord)]
#[surface_helper(path = "model::wire_tag")]
pub struct ApiRecord {
    wire: LiveWire,
}

impl ApiRecord {
    pub fn new(raw: &str) -> Self {
        Self {
            wire: LiveWire::new(raw),
        }
    }

    pub fn render(self) -> String {
        format!("api:{}", self.wire.render())
    }

    pub fn dead_method(self) -> String {
        format!("dead-api:{}", self.wire.dead_method())
    }
}

#[surface_attr(model::wire_tag)]
pub fn selected_wire(raw: &str) -> String {
    ApiRecord::new(raw).render()
}

pub fn dead_live_wire(raw: &str) -> String {
    ApiRecord::new(raw).dead_method()
}
