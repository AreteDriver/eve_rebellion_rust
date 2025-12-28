//! Asset Management
//!
//! Handles loading EVE ship sprites, 3D models, and powerup icons.

pub mod powerup_icons;
pub mod ship_models;
pub mod ship_sprites;

pub use powerup_icons::*;
pub use ship_models::*;
pub use ship_sprites::*;

use bevy::prelude::*;

/// Asset management plugin
pub struct AssetsPlugin;

impl Plugin for AssetsPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins((ShipSpritesPlugin, ShipModelsPlugin, PowerupIconsPlugin));
    }
}
