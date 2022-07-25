use sfml::{
    graphics::{Color, FloatRect, Vertex},
    system::Vector2f,
};

/// Represents a container which can be expanded with quads made out of vertices.
pub trait QuadMeshable {
    fn add_quad(&mut self, position: Vector2f, size: Vector2f, uv: FloatRect);
}

impl QuadMeshable for Vec<Vertex> {
    fn add_quad(&mut self, position: Vector2f, size: Vector2f, uv: FloatRect) {
        self.push(Vertex::new(
            position,
            Color::WHITE,
            Vector2f::new(uv.left, uv.top),
        ));
        self.push(Vertex::new(
            position + Vector2f::new(size.x, 0f32),
            Color::WHITE,
            Vector2f::new(uv.left + uv.width, uv.top),
        ));
        self.push(Vertex::new(
            position + size,
            Color::WHITE,
            Vector2f::new(uv.left + uv.width, uv.top + uv.height),
        ));
        self.push(Vertex::new(
            position + Vector2f::new(0f32, size.y),
            Color::WHITE,
            Vector2f::new(uv.left, uv.top + uv.height),
        ));
    }
}
