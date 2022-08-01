use sfml::{
    audio::{Sound, SoundSource},
    graphics::{Color, Drawable, RenderTarget, RenderWindow, Sprite, Text, Transformable},
    system::{Vector2f, Vector2u},
};
use thiserror::Error;
use tiled::{objects::ObjectShape, tile::Gid};

use crate::{assets::AssetManager, context::Context, level::camera_transform};

pub trait UiObject<'a>: Drawable {
    fn as_drawable(&self) -> &dyn Drawable;
    fn clone_dyn(&self) -> Box<dyn UiObject<'a> + 'a>;
}

impl<'a, T: Clone + Drawable + 'a> UiObject<'a> for T {
    fn as_drawable(&self) -> &dyn Drawable {
        self
    }

    fn clone_dyn(&self) -> Box<dyn UiObject<'a> + 'a> {
        Box::new(self.clone())
    }
}

impl<'a> Clone for Box<dyn UiObject<'a> + 'a> {
    fn clone(&self) -> Self {
        self.clone_dyn()
    }
}

pub fn get_ui_obj_from_tiled_obj<'s>(
    context: &Context<'s>,
    map: &tiled::map::Map,
    object: &tiled::objects::Object,
) -> anyhow::Result<Box<dyn UiObject<'s> + 's>> {
    let assets = context.assets;

    if let ObjectShape::Text {
        pixel_size,
        halign,
        valign,
        color,
        contents,
        ..
    } = &object.shape
    {
        let contents = if object.name == "level_metrics" {
            let completed_level_count = context.completed_levels.internal_set().len();

            format!("{}/{}", completed_level_count, assets.total_level_count())
        } else {
            contents.clone()
        };
        let mut text = Text::new(&contents, &assets.win_font, *pixel_size as u32);
        text.set_fill_color(Color::rgb(color.red, color.green, color.blue));
        let bounds = text.local_bounds();
        text.set_position(Vector2f::new(object.x, object.y));
        text.move_(Vector2f::new(
            match halign {
                tiled::objects::HorizontalAlignment::Left => -bounds.left,
                tiled::objects::HorizontalAlignment::Center => {
                    object.width / 2. - text.local_bounds().width / 2. - bounds.left
                }
                tiled::objects::HorizontalAlignment::Right => {
                    object.width - text.local_bounds().width - bounds.left
                }
                tiled::objects::HorizontalAlignment::Justify => {
                    unimplemented!("Justified texts are not implemented")
                }
            },
            match valign {
                tiled::objects::VerticalAlignment::Top => -bounds.top,
                tiled::objects::VerticalAlignment::Center => {
                    object.height / 2. - text.local_bounds().height / 2. - bounds.top
                }
                tiled::objects::VerticalAlignment::Bottom => {
                    // FIXME: This is wrong! Bottom alignment should not depend on text bounds
                    // and instead should rely on font baseline and other characteristics.
                    // As SFML does not expose them, we are limited to this hack instead.
                    object.height - bounds.height - bounds.top
                }
            },
        ));

        Ok(Box::new(text))
    } else if object.gid != Gid::EMPTY {
        Ok(Box::new(sprite_from_tiled_obj(
            context.assets,
            map,
            object,
        )?))
    } else {
        Err(anyhow::anyhow!(
            "could not obtain ui object from tiled object {:?}",
            object
        ))
    }
}

#[derive(Debug, Error)]
pub enum SpriteFromTiledObjError {
    #[error("Tiled object has invalid GID `{0:?}`; Does not map to any tileset")]
    InvalidGid(Gid),
    #[error("Object tileset has invalid tilesheet name `{0}`")]
    InvalidTilesheetName(String),
}

pub fn sprite_from_tiled_obj<'s>(
    assets: &'s AssetManager,
    map: &tiled::map::Map,
    object: &tiled::objects::Object,
) -> Result<Sprite<'s>, SpriteFromTiledObjError> {
    let gid_tileset = map
        .tileset_by_gid(object.gid)
        .ok_or(SpriteFromTiledObjError::InvalidGid(object.gid))?;
    let tilesheet = match gid_tileset.name.as_str() {
        "icons" => &assets.icon_tilesheet,
        "Sokoban" => &assets.tilesheet,
        x => return Err(SpriteFromTiledObjError::InvalidTilesheetName(x.to_owned())),
    };
    let mut sprite = tilesheet
        .tile_sprite(Gid(object.gid.0 - gid_tileset.first_gid.0 + 1))
        .expect("invalid gid found in overlay object");
    sprite.set_scale(Vector2f::new(
        object.width / sprite.texture_rect().width as f32,
        object.height / sprite.texture_rect().height as f32,
    ));
    sprite.set_position(Vector2f::new(object.x, object.y));
    sprite.set_rotation(object.rotation);
    Ok(sprite)
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ButtonState {
    Pressed,
    Hovered,
    Inactive,
}

pub fn update_button(ctx: &mut Context, window: &RenderWindow, sprite: &mut Sprite) -> ButtonState {
    let transform = camera_transform(
        window.size(),
        Vector2u::new(
            ctx.assets.play_overlay_map.width * ctx.assets.play_overlay_map.tile_width,
            ctx.assets.play_overlay_map.height * ctx.assets.play_overlay_map.tile_height,
        ),
        0.,
    );

    let mouse_pos = window.mouse_position();
    let mouse_pos = transform
        .inverse()
        .transform_point(Vector2f::new(mouse_pos.x as f32, mouse_pos.y as f32));

    let mut color = sprite.color();

    if sprite.global_bounds().contains(mouse_pos) {
        *color.alpha_mut() = 0xcf;
        sprite.set_color(color);

        if ctx.input.just_released_lmb() {
            let mut sound = Sound::with_buffer(&ctx.assets.ui_click_sound);
            sound.set_volume(60.);
            sound.play();
            ctx.sound.add_sound(sound);
            ButtonState::Pressed
        } else {
            ButtonState::Hovered
        }
    } else {
        *color.alpha_mut() = 0xff;
        sprite.set_color(color);
        ButtonState::Inactive
    }
}
