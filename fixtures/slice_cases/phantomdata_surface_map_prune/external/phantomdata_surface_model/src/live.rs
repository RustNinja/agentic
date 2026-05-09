use std::marker::PhantomData;

pub struct PhantomDataSurfaceMapPayload {
    value: String,
}

impl PhantomDataSurfaceMapPayload {
    pub fn new(raw: &str) -> Self {
        Self {
            value: raw.trim().to_string(),
        }
    }

    pub fn render_label(&self) -> String {
        format!("phantomdata-surface-map:{}", self.value)
    }

    pub fn bump_and_render(&mut self) -> String {
        self.value.push_str(":used");
        format!("phantomdata-surface-map:{}", self.value)
    }

    pub fn dead_method(&self) -> String {
        format!("dead-phantomdata-surface-map:{}", self.value)
    }
}

pub struct PhantomDataSurfaceMapMarker;

pub struct PhantomDataSurfaceMapEnvelope<T> {
    payload: PhantomDataSurfaceMapPayload,
    marker: PhantomData<T>,
}

impl<T> PhantomDataSurfaceMapEnvelope<T> {
    pub fn new(raw: &str) -> Self {
        Self {
            payload: PhantomDataSurfaceMapPayload::new(raw),
            marker: PhantomData,
        }
    }

    pub fn render_label(&self) -> String {
        self.payload.render_label()
    }

    pub fn dead_wrapper_method(&self) -> String {
        self.payload.dead_method()
    }
}

pub fn selected_phantomdata_surface_map(raw: &str) -> String {
    PhantomDataSurfaceMapEnvelope::<PhantomDataSurfaceMapMarker>::new(raw).render_label()
}

pub fn dead_live_phantomdata_surface_map(raw: &str) -> String {
    PhantomDataSurfaceMapPayload::new(raw).dead_method()
}
