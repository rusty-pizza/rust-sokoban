use sfml::system::Vector2i;

use super::{tilemap::LevelTile, Direction, Level};

#[derive(Clone, Copy)]
pub enum Action {
    /// Pushes a crate forwards and moves the player in the direction given.
    /// The player will look in the direction given.
    Push {
        direction: Direction,
        look_direction: Direction,
    },

    /// Pulls or moves backwards in the direction given; e.g. pulling east will move the player to
    /// the east, along with any crate directly to its west.
    /// The player will look in the direction given.
    ///
    /// It will pull out crates from holes!
    Pull {
        direction: Direction,
        look_direction: Direction,
    },
}

impl Action {
    /// Applies this action to the level given, using an application context.
    /// Returns the reciprocal if everything went correctly.
    pub fn apply(self, level: &mut Level) -> Result<Action, ()> {
        match self {
            Action::Push {
                direction,
                look_direction,
            } => {
                let previous_look_direction = level.player.direction();
                let movement: Vector2i = direction.into();

                let cell_to_move_to = level.player.position() + movement;

                if level.is_cell_walkable(cell_to_move_to) {
                    let crate_to_move_idx = level
                        .crates
                        .iter()
                        .enumerate()
                        .find(|(_idx, c)| c.position() == cell_to_move_to && !c.in_hole())
                        .map(|(idx, _ref)| idx);

                    if let Some(crate_to_move_idx) = crate_to_move_idx {
                        let crate_target_position = cell_to_move_to + movement;

                        let is_crate_movable = !level.is_cell_obstructed(crate_target_position);

                        if is_crate_movable {
                            // Can move and we are pushing a crate with ourselves
                            level.player.set_transform(cell_to_move_to, look_direction);
                            level.crates[crate_to_move_idx].set_position(crate_target_position);

                            let target_tile = level.tilemap.get_tile(crate_target_position);
                            if target_tile == Some(LevelTile::Hole) {
                                let is_hole_full = level
                                    .crates
                                    .iter()
                                    .any(|c| c.position() == crate_target_position && c.in_hole());

                                if !is_hole_full {
                                    level.crates[crate_to_move_idx].set_in_hole(true);
                                }
                            }

                            Ok(Action::Pull {
                                direction: direction.inverse(),
                                look_direction: previous_look_direction,
                            })
                        } else {
                            // Can't move, something is on the way after the crate
                            Err(())
                        }
                    } else {
                        // Can move and no obstacle is on the way
                        level.player.set_transform(cell_to_move_to, look_direction);
                        Ok(Action::Push {
                            direction: direction.inverse(),
                            look_direction: previous_look_direction,
                        })
                    }
                } else {
                    // Can't move, something is on the way
                    Err(())
                }
            }
            Action::Pull {
                direction,
                look_direction,
            } => {
                let previous_look_direction = level.player.direction();
                let movement: Vector2i = direction.into();

                let cell_to_pull_from = level.player.position() - movement;
                let cell_to_move_to = level.player.position() + movement;

                if level.is_cell_walkable(cell_to_move_to) {
                    let crate_to_move_idx = level
                        .crates
                        .iter()
                        .enumerate()
                        .find(|(_idx, c)| c.position() == cell_to_pull_from)
                        .map(|(idx, _ref)| idx);

                    if let Some(crate_to_move_idx) = crate_to_move_idx {
                        // Can move and we are pulling a crate with ourselves
                        let crate_target_position = level.player.position();

                        let is_crate_movable = !level.is_cell_obstructed(crate_target_position);

                        if is_crate_movable {
                            level.player.set_transform(cell_to_move_to, look_direction);
                            level.crates[crate_to_move_idx].set_position(crate_target_position);

                            let target_tile = level.tilemap.get_tile(crate_target_position);
                            let is_in_hole = if target_tile == Some(LevelTile::Hole) {
                                let is_hole_full = level
                                    .crates
                                    .iter()
                                    .any(|c| c.position() == crate_target_position && c.in_hole());

                                !is_hole_full
                            } else {
                                false
                            };
                            level.crates[crate_to_move_idx].set_in_hole(is_in_hole);

                            Ok(Action::Push {
                                direction: direction.inverse(),
                                look_direction: previous_look_direction,
                            })
                        } else {
                            // Can't move, something is on the way on the crate target (Should never
                            // happen because the player is there, but checking anyways if we do
                            // more complex mechanics)
                            Err(())
                        }
                    } else {
                        // Can move and no obstacle is on the way
                        level.player.set_transform(cell_to_move_to, look_direction);
                        Ok(Action::Push {
                            direction: direction.inverse(),
                            look_direction: previous_look_direction,
                        })
                    }
                } else {
                    // Can't move, something is on the way
                    Err(())
                }
            }
        }
    }
}
