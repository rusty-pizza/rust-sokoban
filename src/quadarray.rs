use sfml::{
    graphics::{Color, FloatRect, Vertex},
    system::Vector2f,
};

pub struct QuadMeshBuilder {
    inner: Vec<Vertex>,
}

impl QuadMeshBuilder {
    pub fn new() -> Self {
        Self { inner: Vec::new() }
    }

    pub fn add_quad(&mut self, position: Vector2f, size: f32, uv: FloatRect) {
        self.inner.push(Vertex::new(
            position,
            Color::WHITE,
            Vector2f::new(uv.left, uv.top),
        ));
        self.inner.push(Vertex::new(
            position + Vector2f::new(size, 0f32),
            Color::WHITE,
            Vector2f::new(uv.left + uv.width, uv.top),
        ));
        self.inner.push(Vertex::new(
            position + Vector2f::new(size, size),
            Color::WHITE,
            Vector2f::new(uv.left + uv.width, uv.top + uv.height),
        ));
        self.inner.push(Vertex::new(
            position + Vector2f::new(0f32, size),
            Color::WHITE,
            Vector2f::new(uv.left, uv.top + uv.height),
        ));
    }

    pub fn result(self) -> Vec<Vertex> {
        self.inner
    }
}
