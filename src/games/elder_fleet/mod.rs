//! Elder Fleet Invasion Module
//!
//! Minmatar Republic vs Amarr Empire campaign.
//! The original EVE Rebellion campaign - 13 missions across 3 acts.

use super::{ActiveModule, FactionInfo, GameModuleInfo, ModuleRegistry};
use bevy::prelude::*;

pub mod ships;

pub use ships::ElderFleetShips;

/// Elder Fleet module plugin
pub struct ElderFleetPlugin;

impl Plugin for ElderFleetPlugin {
    fn build(&self, app: &mut App) {
        // Register module at startup
        app.add_systems(Startup, register_module);

        // Initialize resources
        app.init_resource::<ElderFleetShips>();
    }
}

/// Register the Elder Fleet module with the registry
fn register_module(mut registry: ResMut<ModuleRegistry>) {
    registry.register(GameModuleInfo {
        id: "elder_fleet",
        display_name: "Elder Fleet Invasion",
        subtitle: "Minmatar vs Amarr",
        description:
            "The Minmatar Rebellion rises. Strike back against centuries of Amarr oppression.",
        factions: vec![
            FactionInfo {
                id: "minmatar",
                name: "Minmatar Republic",
                primary_color: Color::srgb(0.71, 0.39, 0.20), // Rust orange
                secondary_color: Color::srgb(0.55, 0.35, 0.17), // Brown
                accent_color: Color::srgba(1.0, 0.59, 0.2, 0.9), // Orange glow
                doctrine: vec!["Projectiles", "Speed", "Guerrilla"],
                description:
                    "In Rust We Trust. Your ancestors were enslaved. Today, you strike back.",
            },
            FactionInfo {
                id: "amarr",
                name: "Amarr Empire",
                primary_color: Color::srgb(1.0, 0.84, 0.0), // Gold
                secondary_color: Color::srgb(0.8, 0.6, 0.2), // Dark gold
                accent_color: Color::srgba(1.0, 0.95, 0.6, 0.9), // Golden glow
                doctrine: vec!["Lasers", "Armor", "Divine Right"],
                description: "Crush the rebel insurrection. Restore order through strength.",
            },
        ],
    });
}

/// Run condition: is Elder Fleet module active?
pub fn is_elder_fleet(active_module: Res<ActiveModule>) -> bool {
    active_module.is_elder_fleet()
}

/// Run condition: is Elder Fleet module NOT active?
pub fn not_elder_fleet(active_module: Res<ActiveModule>) -> bool {
    !active_module.is_elder_fleet()
}
