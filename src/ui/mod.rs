//! UI Systems
//!
//! HUD, menus, and visual feedback.

pub mod hud;
pub mod menu;
pub mod capacitor;

pub use hud::*;
pub use menu::*;
pub use capacitor::*;

use bevy::prelude::*;

/// Plugin that registers all UI systems
pub struct UiPlugin;

impl Plugin for UiPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins((
            HudPlugin,
            MenuPlugin,
            CapacitorWheelPlugin,
        ));
    }
}
