use sfml::{
    graphics::{Drawable, IntRect, Sprite, Texture, Transformable},
    system::Vector2f,
};

#[derive(Debug)]
pub struct SpriteAtlas<'t> {
    frames: Vec<IntRect>,
    current_frame: usize,
    sprite: Sprite<'t>,
}

impl<'t> SpriteAtlas<'t> {
    pub fn with_texture_and_frames(texture: &'t Texture, frames: &[IntRect]) -> Self {
        Self {
            current_frame: 0,
            frames: Vec::from(frames),
            sprite: if let Some(first_frame) = frames.get(0) {
                Sprite::with_texture_and_rect(texture, &first_frame)
            } else {
                Sprite::with_texture(texture)
            },
        }
    }

    pub fn add_frame(&mut self, frame: IntRect) {
        self.frames.push(frame);
    }

    pub fn current_frame(&self) -> usize {
        self.current_frame
    }

    pub fn set_frame(&mut self, frame: usize) -> Result<(), ()> {
        if let Some(rect) = self.frames.get(frame) {
            self.current_frame = frame;
            self.sprite.set_texture_rect(rect);
            Ok(())
        } else {
            Err(())
        }
    }
}

impl Transformable for SpriteAtlas<'_> {
    fn set_position<P: Into<Vector2f>>(&mut self, position: P) {
        self.sprite.set_position(position);
    }

    fn set_rotation(&mut self, angle: f32) {
        self.sprite.set_rotation(angle);
    }

    fn set_scale<S: Into<Vector2f>>(&mut self, scale: S) {
        self.sprite.set_scale(scale);
    }

    fn set_origin<O: Into<Vector2f>>(&mut self, origin: O) {
        self.sprite.set_origin(origin);
    }

    fn position(&self) -> Vector2f {
        self.sprite.position()
    }

    fn rotation(&self) -> f32 {
        self.sprite.rotation()
    }

    fn get_scale(&self) -> Vector2f {
        self.sprite.get_scale()
    }

    fn origin(&self) -> Vector2f {
        self.sprite.origin()
    }

    fn move_<O: Into<Vector2f>>(&mut self, offset: O) {
        self.sprite.move_(offset);
    }

    fn rotate(&mut self, angle: f32) {
        self.sprite.rotate(angle);
    }

    fn scale<F: Into<Vector2f>>(&mut self, factors: F) {
        self.sprite.scale(factors);
    }

    fn transform(&self) -> sfml::graphics::Transform {
        self.sprite.transform()
    }

    fn inverse_transform(&self) -> sfml::graphics::Transform {
        self.sprite.inverse_transform()
    }
}

impl Drawable for SpriteAtlas<'_> {
    fn draw<'a: 'shader, 'texture, 'shader, 'shader_texture>(
        &'a self,
        target: &mut dyn sfml::graphics::RenderTarget,
        states: &sfml::graphics::RenderStates<'texture, 'shader, 'shader_texture>,
    ) {
        target.draw_sprite(&self.sprite, states);
    }
}
