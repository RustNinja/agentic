mod dead;
mod live;

pub use dead::{dead_projection, DeadProjection};
pub use live::{
    render_projection, ProjectionDto, ProjectionRender, ProjectionResolver, Resolver,
    selected_projection,
};
