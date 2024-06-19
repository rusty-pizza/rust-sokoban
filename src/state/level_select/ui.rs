use sfml::{
    graphics::{Color, FloatRect, Sprite, Transformable},
    system::Vector2f,
};

use crate::context::Context;

/// A button representing a playable level. May be locked.
///
/// Just holds sprites; Doesn't contain any logic
#[derive(Clone)]
pub struct LevelArrayButton<'s> {
    pub sprite: Sprite<'s>,
    pub lock_sprite: Option<Sprite<'s>>,
}

impl LevelArrayButton<'_> {
    pub fn unlocked(&self) -> bool {
        self.lock_sprite.is_none()
    }
}

/// An UI element containing a set of clickable level icons.
///
/// Will render as a horizontal list of buttons, each one corresponding to a different level in a category.
#[derive(Clone)]
pub struct LevelArray<'s> {
    pub category: usize,
    pub sprites: Vec<LevelArrayButton<'s>>,
}

impl<'s> LevelArray<'s> {
    /// Create a new [`LevelArray`] from a target rect to display at and a category index to display the levels from.
    pub fn new(ctx: &Context<'s>, rect: FloatRect, category_idx: usize) -> Self {
        let mut buttons = Vec::new();

        // Setup the level icons to use. We'll clone these for each level in the category
        // We'll use the lock icon over levels that haven't unlocked yet
        let mut level_icon = ctx.assets.icon_tilesheet.tile_sprite(91).unwrap();
        let mut lock_icon = ctx.assets.icon_tilesheet.tile_sprite(115).unwrap();
        let category = &ctx.assets.level_categories[category_idx];
        level_icon.set_position(Vector2f::new(rect.left, rect.top));
        level_icon.set_scale(Vector2f::new(
            rect.height / level_icon.global_bounds().height,
            rect.height / level_icon.global_bounds().height,
        ));
        lock_icon.set_position(Vector2f::new(rect.left, rect.top));
        lock_icon.set_scale(Vector2f::new(
            rect.height / lock_icon.global_bounds().height,
            rect.height / lock_icon.global_bounds().height,
        ));

        let mut completed_previous_level = true;
        for level in category.maps.iter() {
            let completed_level = ctx.completed_levels.internal_set().contains(&level.1);
            let is_unlocked = completed_level || completed_previous_level;
            let color = if is_unlocked {
                Color {
                    a: 50,
                    ..category.color
                }
            } else {
                category.color
            };
            level_icon.set_color(color);
            buttons.push(LevelArrayButton {
                sprite: level_icon.clone(),
                lock_sprite: (!is_unlocked).then_some(lock_icon.clone()),
            });

            // Move to where the next icon will go
            level_icon.move_(Vector2f::new(level_icon.global_bounds().width, 0.));
            lock_icon.move_(Vector2f::new(level_icon.global_bounds().width, 0.));

            completed_previous_level = completed_level;
        }

        Self {
            sprites: buttons,
            category: category_idx,
        }
    }
}
