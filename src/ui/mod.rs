//! UI Systems
//!
//! HUD, menus, and visual feedback.

pub mod hud;
pub mod menu;

pub use hud::*;
pub use menu::*;

use bevy::prelude::*;

/// Plugin that registers all UI systems
pub struct UiPlugin;

impl Plugin for UiPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins((
            HudPlugin,
            MenuPlugin,
        ));
    }
}
