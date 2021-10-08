use sfml::{graphics::{Color, FloatRect, PrimitiveType, Vertex, VertexArray}, system::Vector2f};


pub struct QuadArray {
    inner: VertexArray
}

impl QuadArray {
    pub fn new(quad_count: usize) -> Self {
        Self { inner: VertexArray::new(PrimitiveType::QUADS, quad_count * 4)}
    }

    pub fn add_quad(&mut self, position: Vector2f, size: f32, uv: FloatRect) {
        self.inner.append(&Vertex::new(position, Color::WHITE, Vector2f::new(uv.left, uv.top)));
        self.inner.append(&Vertex::new(position + Vector2f::new(size, 0f32), Color::WHITE, Vector2f::new(uv.left+uv.width, uv.top)));
        self.inner.append(&Vertex::new(position + Vector2f::new(size, size), Color::WHITE, Vector2f::new(uv.left+uv.width, uv.top+uv.height)));
        self.inner.append(&Vertex::new(position + Vector2f::new(0f32, size), Color::WHITE, Vector2f::new(uv.left, uv.top+uv.height)));
    }    

    pub fn result(self) -> VertexArray {
        self.inner
    }
}