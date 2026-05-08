pub trait Resolver {
    type Output;

    fn resolve(&self, raw: &str) -> Self::Output;
}

pub trait ProjectionRender {
    fn render_projection(self) -> String;
}

pub struct ProjectionResolver;

pub struct ProjectionDto {
    label: String,
}

impl ProjectionDto {
    pub fn dead_method(&self) -> String {
        format!("dead-projection:{}", self.label)
    }
}

impl Resolver for ProjectionResolver {
    type Output = ProjectionDto;

    fn resolve(&self, raw: &str) -> Self::Output {
        ProjectionDto {
            label: raw.trim().to_string(),
        }
    }
}

impl ProjectionRender for ProjectionDto {
    fn render_projection(self) -> String {
        format!("projection:{}", self.label)
    }
}

pub fn render_projection<R>(resolver: &R, raw: &str) -> String
where
    R: Resolver,
    R::Output: ProjectionRender,
{
    resolver.resolve(raw).render_projection()
}

pub fn selected_projection(raw: &str) -> String {
    render_projection(&ProjectionResolver, raw)
}

pub fn dead_live_projection(raw: &str) -> String {
    ProjectionResolver.resolve(raw).dead_method()
}
