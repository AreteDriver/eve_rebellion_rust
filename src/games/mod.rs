//! Game Modules System
//!
//! Supports multiple game modules (campaigns) that share the core engine.
//! Each module defines its own factions, ships, missions, and theme.

#![allow(dead_code)]

use bevy::prelude::*;

pub mod caldari_gallente;

/// Game modules plugin - registers all available game modules
pub struct GameModulesPlugin;

impl Plugin for GameModulesPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<ModuleRegistry>()
            .init_resource::<ActiveModule>()
            .add_plugins(caldari_gallente::CaldariGallentePlugin);
    }
}

/// Registry of all available game modules
#[derive(Resource, Default)]
pub struct ModuleRegistry {
    pub modules: Vec<GameModuleInfo>,
}

impl ModuleRegistry {
    pub fn register(&mut self, module: GameModuleInfo) {
        self.modules.push(module);
    }

    pub fn get(&self, id: &str) -> Option<&GameModuleInfo> {
        self.modules.iter().find(|m| m.id == id)
    }
}

/// Information about a game module
#[derive(Clone, Debug)]
pub struct GameModuleInfo {
    pub id: &'static str,
    pub display_name: &'static str,
    pub subtitle: &'static str,
    pub description: &'static str,
    pub factions: Vec<FactionInfo>,
}

/// Faction information
#[derive(Clone, Debug)]
pub struct FactionInfo {
    pub id: &'static str,
    pub name: &'static str,
    pub primary_color: Color,
    pub secondary_color: Color,
    pub accent_color: Color,
    pub doctrine: Vec<&'static str>,
    pub description: &'static str,
}

/// Currently active game module and selected faction
#[derive(Resource, Default)]
pub struct ActiveModule {
    pub module_id: Option<String>,
    pub player_faction: Option<String>,
    pub enemy_faction: Option<String>,
}

impl ActiveModule {
    pub fn set_module(&mut self, module_id: &str) {
        self.module_id = Some(module_id.to_string());
        self.player_faction = None;
        self.enemy_faction = None;
    }

    pub fn set_faction(&mut self, player: &str, enemy: &str) {
        self.player_faction = Some(player.to_string());
        self.enemy_faction = Some(enemy.to_string());
    }

    pub fn is_caldari_gallente(&self) -> bool {
        self.module_id.as_deref() == Some("caldari_gallente")
    }

    pub fn is_elder_fleet(&self) -> bool {
        self.module_id.is_none() || self.module_id.as_deref() == Some("elder_fleet")
    }
}

/// Ship definition for a module
#[derive(Clone, Debug)]
pub struct ModuleShip {
    pub type_id: u32,
    pub name: &'static str,
    pub class: &'static str,
    pub role: &'static str,
    pub health: f32,
    pub speed: f32,
    pub fire_rate: f32,
    pub damage: f32,
    pub unlimited_thrust: bool,
    pub unlocked: bool,
    pub unlock_mission: Option<u32>,
}

/// Ship pools for a faction
#[derive(Clone, Debug, Default)]
pub struct FactionShipPool {
    pub player_ships: Vec<ModuleShip>,
    pub enemy_ships: Vec<EnemyShipDef>,
}

/// Enemy ship definition
#[derive(Clone, Debug)]
pub struct EnemyShipDef {
    pub type_id: u32,
    pub name: &'static str,
    pub class: &'static str,
    pub spawn_weight: u32,
}
