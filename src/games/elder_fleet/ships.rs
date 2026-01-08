//! Elder Fleet Ship Pools
//!
//! Minmatar and Amarr ship definitions for the Elder Fleet campaign.
//! Wraps the core faction ship definitions for module-specific use.

use crate::core::factions::{EnemyShipDef, Faction, ShipDef};
use crate::games::{EnemyShipDef as ModuleEnemyShipDef, FactionShipPool, ModuleShip};
use bevy::prelude::*;

/// Ship pools for the Elder Fleet module
#[derive(Resource, Debug, Clone)]
pub struct ElderFleetShips {
    pub minmatar: FactionShipPool,
    pub amarr: FactionShipPool,
}

impl Default for ElderFleetShips {
    fn default() -> Self {
        Self::new()
    }
}

impl ElderFleetShips {
    pub fn new() -> Self {
        Self {
            minmatar: Self::build_minmatar_pool(),
            amarr: Self::build_amarr_pool(),
        }
    }

    fn build_minmatar_pool() -> FactionShipPool {
        let core_ships = Faction::Minmatar.player_ships();
        let core_enemies = Faction::Minmatar.enemy_ships();

        FactionShipPool {
            player_ships: core_ships.iter().map(Self::convert_ship).collect(),
            enemy_ships: core_enemies.iter().map(Self::convert_enemy).collect(),
        }
    }

    fn build_amarr_pool() -> FactionShipPool {
        let core_ships = Faction::Amarr.player_ships();
        let core_enemies = Faction::Amarr.enemy_ships();

        FactionShipPool {
            player_ships: core_ships.iter().map(Self::convert_ship).collect(),
            enemy_ships: core_enemies.iter().map(Self::convert_enemy).collect(),
        }
    }

    fn convert_ship(def: &ShipDef) -> ModuleShip {
        ModuleShip {
            type_id: def.type_id,
            name: def.name,
            class: def.class.name(),
            role: def.role,
            health: def.health,
            speed: def.speed,
            fire_rate: def.fire_rate,
            damage: def.damage,
            unlimited_thrust: false, // Determined by ship special ability
            unlocked: def.unlock_stage == 0,
            unlock_mission: if def.unlock_stage > 0 {
                Some(def.unlock_stage)
            } else {
                None
            },
        }
    }

    fn convert_enemy(def: &EnemyShipDef) -> ModuleEnemyShipDef {
        ModuleEnemyShipDef {
            type_id: def.type_id,
            name: def.name,
            class: def.class.name(),
            spawn_weight: def.spawn_weight,
        }
    }

    /// Get player ships for a faction
    pub fn player_ships(&self, faction: &str) -> &[ModuleShip] {
        match faction {
            "minmatar" => &self.minmatar.player_ships,
            "amarr" => &self.amarr.player_ships,
            _ => &self.minmatar.player_ships, // Default to Minmatar
        }
    }

    /// Get enemy ships for a faction
    pub fn enemy_ships(&self, faction: &str) -> &[ModuleEnemyShipDef] {
        match faction {
            "minmatar" => &self.minmatar.enemy_ships,
            "amarr" => &self.amarr.enemy_ships,
            _ => &self.amarr.enemy_ships, // Default to Amarr enemies
        }
    }

    /// Get random enemy type ID for a faction
    pub fn random_enemy_type(&self, faction: &str) -> u32 {
        let enemies = self.enemy_ships(faction);
        if enemies.is_empty() {
            return 597; // Punisher fallback
        }

        // Weighted random selection
        let total_weight: u32 = enemies.iter().map(|e| e.spawn_weight).sum();
        let mut roll = fastrand::u32(0..total_weight);

        for enemy in enemies {
            if roll < enemy.spawn_weight {
                return enemy.type_id;
            }
            roll -= enemy.spawn_weight;
        }

        enemies[0].type_id
    }
}
