//! Asset Management
//!
//! Handles loading EVE ship sprites and powerup icons.

pub mod ship_sprites;
pub mod powerup_icons;

pub use ship_sprites::*;
pub use powerup_icons::*;

use bevy::prelude::*;

/// Asset management plugin
pub struct AssetsPlugin;

impl Plugin for AssetsPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins((ShipSpritesPlugin, PowerupIconsPlugin));
    }
}
