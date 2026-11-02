pub struct RendererInfo {
    pub name: &'static str,
    pub graphics_api: &'static str,
}

impl RendererInfo {
    pub fn new() -> Self {
        Self {
            name: "Horizon Renderer",
            graphics_api: "wgpu",
        }
    }
}
