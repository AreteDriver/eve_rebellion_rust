//! Caldari/Gallente Ship Pools
//!
//! Ship definitions for both factions - player and enemy rosters.

#![allow(dead_code)]

use crate::games::{EnemyShipDef, FactionShipPool, ModuleShip};
use bevy::prelude::*;

/// Ship pools for both Caldari and Gallente factions
#[derive(Resource, Default)]
pub struct CaldariGallenteShips {
    pub caldari: FactionShipPool,
    pub gallente: FactionShipPool,
}

impl CaldariGallenteShips {
    pub fn new() -> Self {
        Self {
            caldari: build_caldari_pool(),
            gallente: build_gallente_pool(),
        }
    }

    /// Get player ships for the given faction
    pub fn player_ships(&self, faction: &str) -> &[ModuleShip] {
        match faction {
            "caldari" => &self.caldari.player_ships,
            "gallente" => &self.gallente.player_ships,
            _ => &[],
        }
    }

    /// Get enemy ships for the given faction
    pub fn enemy_ships(&self, faction: &str) -> &[EnemyShipDef] {
        match faction {
            "caldari" => &self.caldari.enemy_ships,
            "gallente" => &self.gallente.enemy_ships,
            _ => &[],
        }
    }

    /// Get a random enemy type ID based on spawn weights
    pub fn random_enemy_type(&self, faction: &str) -> u32 {
        let enemies = self.enemy_ships(faction);
        if enemies.is_empty() {
            return 603; // Fallback to Merlin
        }

        let total_weight: u32 = enemies.iter().map(|e| e.spawn_weight).sum();
        let roll = fastrand::u32(0..total_weight);

        let mut cumulative = 0;
        for enemy in enemies {
            cumulative += enemy.spawn_weight;
            if roll < cumulative {
                return enemy.type_id;
            }
        }

        enemies[0].type_id
    }
}

fn build_caldari_pool() -> FactionShipPool {
    FactionShipPool {
        player_ships: vec![
            // Hawk - Assault Frigate, Missile Boat
            ModuleShip {
                type_id: 11381,
                name: "Hawk",
                class: "Assault Frigate",
                role: "Missile Boat",
                health: 100.0,
                speed: 120.0,
                fire_rate: 0.3,
                damage: 18.0,
                unlimited_thrust: false,
                unlocked: true,
                unlock_mission: None,
            },
            // Harpy - Assault Frigate, Railgun Platform
            ModuleShip {
                type_id: 11387,
                name: "Harpy",
                class: "Assault Frigate",
                role: "Railgun Platform",
                health: 90.0,
                speed: 130.0,
                fire_rate: 0.25,
                damage: 22.0,
                unlimited_thrust: false,
                unlocked: true,
                unlock_mission: None,
            },
            // Jackdaw - T3 Tactical Destroyer (unlockable)
            ModuleShip {
                type_id: 35683,
                name: "Jackdaw",
                class: "Tactical Destroyer",
                role: "Mode-Switching Platform",
                health: 150.0,
                speed: 100.0,
                fire_rate: 0.35,
                damage: 25.0,
                unlimited_thrust: false,
                unlocked: false,
                unlock_mission: Some(4),
            },
        ],
        enemy_ships: vec![
            // Gallente enemies when playing Caldari
            EnemyShipDef {
                type_id: 11373, // Enyo
                name: "Enyo",
                class: "Assault Frigate",
                spawn_weight: 30,
            },
            EnemyShipDef {
                type_id: 11371, // Ishkur
                name: "Ishkur",
                class: "Assault Frigate",
                spawn_weight: 30,
            },
            EnemyShipDef {
                type_id: 594, // Incursus
                name: "Incursus",
                class: "Frigate",
                spawn_weight: 25,
            },
            EnemyShipDef {
                type_id: 593, // Tristan
                name: "Tristan",
                class: "Frigate",
                spawn_weight: 15,
            },
        ],
    }
}

fn build_gallente_pool() -> FactionShipPool {
    FactionShipPool {
        player_ships: vec![
            // Enyo - Assault Frigate, Blaster Brawler
            ModuleShip {
                type_id: 11373,
                name: "Enyo",
                class: "Assault Frigate",
                role: "Blaster Brawler",
                health: 110.0,
                speed: 110.0,
                fire_rate: 0.2,
                damage: 20.0,
                unlimited_thrust: false,
                unlocked: true,
                unlock_mission: None,
            },
            // Ishkur - Assault Frigate, Drone Boat
            ModuleShip {
                type_id: 11371,
                name: "Ishkur",
                class: "Assault Frigate",
                role: "Drone Boat",
                health: 95.0,
                speed: 115.0,
                fire_rate: 0.35,
                damage: 16.0,
                unlimited_thrust: false,
                unlocked: true,
                unlock_mission: None,
            },
            // Hecate - T3 Tactical Destroyer (unlockable)
            ModuleShip {
                type_id: 35685,
                name: "Hecate",
                class: "Tactical Destroyer",
                role: "Mode-Switching Platform",
                health: 140.0,
                speed: 105.0,
                fire_rate: 0.22,
                damage: 28.0,
                unlimited_thrust: false,
                unlocked: false,
                unlock_mission: Some(4),
            },
        ],
        enemy_ships: vec![
            // Caldari enemies when playing Gallente
            EnemyShipDef {
                type_id: 11381, // Hawk
                name: "Hawk",
                class: "Assault Frigate",
                spawn_weight: 30,
            },
            EnemyShipDef {
                type_id: 11387, // Harpy
                name: "Harpy",
                class: "Assault Frigate",
                spawn_weight: 30,
            },
            EnemyShipDef {
                type_id: 603, // Merlin
                name: "Merlin",
                class: "Frigate",
                spawn_weight: 25,
            },
            EnemyShipDef {
                type_id: 602, // Kestrel
                name: "Kestrel",
                class: "Frigate",
                spawn_weight: 15,
            },
        ],
    }
}

/// Initialize ship pools on startup
pub fn init_ship_pools(mut ships: ResMut<CaldariGallenteShips>) {
    *ships = CaldariGallenteShips::new();
}
