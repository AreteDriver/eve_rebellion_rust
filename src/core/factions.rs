//! Unified Faction System
//!
//! All 4 EVE factions with ships, colors, doctrines, and lore.

#![allow(dead_code)]

use bevy::prelude::*;

/// The four major factions of New Eden
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum Faction {
    #[default]
    Minmatar,
    Amarr,
    Caldari,
    Gallente,
}

impl Faction {
    /// All factions
    pub fn all() -> &'static [Faction] {
        &[
            Faction::Minmatar,
            Faction::Amarr,
            Faction::Caldari,
            Faction::Gallente,
        ]
    }

    /// Faction display name
    pub fn name(&self) -> &'static str {
        match self {
            Faction::Minmatar => "Minmatar Republic",
            Faction::Amarr => "Amarr Empire",
            Faction::Caldari => "Caldari State",
            Faction::Gallente => "Gallente Federation",
        }
    }

    /// Short name
    pub fn short_name(&self) -> &'static str {
        match self {
            Faction::Minmatar => "MINMATAR",
            Faction::Amarr => "AMARR",
            Faction::Caldari => "CALDARI",
            Faction::Gallente => "GALLENTE",
        }
    }

    /// Faction tagline
    pub fn tagline(&self) -> &'static str {
        match self {
            Faction::Minmatar => "In Rust We Trust",
            Faction::Amarr => "Amarr Victor",
            Faction::Caldari => "The State Provides",
            Faction::Gallente => "Liberty or Death",
        }
    }

    /// Primary color (bright accent)
    pub fn primary_color(&self) -> Color {
        match self {
            Faction::Minmatar => Color::srgb(0.71, 0.39, 0.20), // Rust orange
            Faction::Amarr => Color::srgb(1.0, 0.84, 0.0),      // Gold
            Faction::Caldari => Color::srgb(0.27, 0.51, 0.71),  // Steel blue
            Faction::Gallente => Color::srgb(0.42, 0.56, 0.14), // Olive green
        }
    }

    /// Secondary color (darker)
    pub fn secondary_color(&self) -> Color {
        match self {
            Faction::Minmatar => Color::srgb(0.55, 0.35, 0.17), // Brown
            Faction::Amarr => Color::srgb(0.55, 0.46, 0.0),     // Dark gold
            Faction::Caldari => Color::srgb(0.12, 0.23, 0.37),  // Navy
            Faction::Gallente => Color::srgb(0.18, 0.36, 0.18), // Dark green
        }
    }

    /// Engine trail color
    pub fn engine_color(&self) -> Color {
        match self {
            Faction::Minmatar => Color::srgba(1.0, 0.59, 0.2, 0.9), // Orange
            Faction::Amarr => Color::srgba(0.39, 0.59, 1.0, 0.9),   // Blue
            Faction::Caldari => Color::srgba(0.39, 0.78, 1.0, 0.9), // Cyan
            Faction::Gallente => Color::srgba(0.59, 1.0, 0.59, 0.9), // Green
        }
    }

    /// Weapon doctrine
    pub fn weapon_type(&self) -> WeaponDoctrine {
        match self {
            Faction::Minmatar => WeaponDoctrine::Projectile,
            Faction::Amarr => WeaponDoctrine::Laser,
            Faction::Caldari => WeaponDoctrine::Missile,
            Faction::Gallente => WeaponDoctrine::Hybrid,
        }
    }

    /// Tank doctrine
    pub fn tank_type(&self) -> TankDoctrine {
        match self {
            Faction::Minmatar => TankDoctrine::Speed, // Speed tank
            Faction::Amarr => TankDoctrine::Armor,    // Armor tank
            Faction::Caldari => TankDoctrine::Shield, // Shield tank
            Faction::Gallente => TankDoctrine::Armor, // Armor tank
        }
    }

    /// Default enemy faction
    pub fn rival(&self) -> Faction {
        match self {
            Faction::Minmatar => Faction::Amarr,
            Faction::Amarr => Faction::Minmatar,
            Faction::Caldari => Faction::Gallente,
            Faction::Gallente => Faction::Caldari,
        }
    }

    /// Campaign intro text
    pub fn story_intro(&self) -> &'static str {
        match self {
            Faction::Minmatar => "Your ancestors were enslaved by the Amarr Empire. Generations suffered under their golden heel. But the Minmatar spirit cannot be broken. Today, you strike back.",
            Faction::Amarr => "The Minmatar rebels threaten the divine order of the Empire. As a loyal servant of the Empress, you will crush this insurrection and restore peace through strength.",
            Faction::Caldari => "The Gallente Federation encroaches on State interests. Corporate profits demand action. You are the blade of the megacorporations.",
            Faction::Gallente => "The Caldari State oppresses its workers and threatens Federation sovereignty. Fight for liberty against corporate tyranny.",
        }
    }

    /// Victory text
    pub fn victory_text(&self) -> &'static str {
        match self {
            Faction::Minmatar => "The Amarr fleet lies in ruins. Slaves are free. The Republic stands defiant. You are legend.",
            Faction::Amarr => "The rebellion is crushed. Order is restored. The Empire endures eternal. Glory to Amarr.",
            Faction::Caldari => "Gallente forces are scattered. The trade lanes are secure. The State prospers.",
            Faction::Gallente => "The Caldari fleet retreats. Freedom rings across the stars. Vive la Fédération!",
        }
    }

    /// Get player ships for this faction
    pub fn player_ships(&self) -> &'static [ShipDef] {
        match self {
            Faction::Minmatar => MINMATAR_SHIPS,
            Faction::Amarr => AMARR_SHIPS,
            Faction::Caldari => CALDARI_SHIPS,
            Faction::Gallente => GALLENTE_SHIPS,
        }
    }

    /// Get enemy ships for this faction
    pub fn enemy_ships(&self) -> &'static [EnemyShipDef] {
        match self {
            Faction::Minmatar => MINMATAR_ENEMIES,
            Faction::Amarr => AMARR_ENEMIES,
            Faction::Caldari => CALDARI_ENEMIES,
            Faction::Gallente => GALLENTE_ENEMIES,
        }
    }

    /// Get carrier type_id for this faction (used for wave spawning visuals)
    pub fn carrier_type_id(&self) -> u32 {
        match self {
            Faction::Minmatar => 24483, // Nidhoggur
            Faction::Amarr => 23757,    // Archon
            Faction::Caldari => 23915,  // Chimera
            Faction::Gallente => 23911, // Thanatos
        }
    }
}

/// Weapon doctrine types
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum WeaponDoctrine {
    Projectile, // Minmatar - autocannons, fast ROF, selectable damage
    Laser,      // Amarr - pulse/beam, instant hit, capacitor hungry
    Missile,    // Caldari - missiles, delayed hit, no tracking issues
    Hybrid,     // Gallente - blasters/rails, high damage, short range
}

impl WeaponDoctrine {
    pub fn name(&self) -> &'static str {
        match self {
            WeaponDoctrine::Projectile => "Autocannons",
            WeaponDoctrine::Laser => "Lasers",
            WeaponDoctrine::Missile => "Missiles",
            WeaponDoctrine::Hybrid => "Blasters",
        }
    }

    /// Projectile color
    pub fn bullet_color(&self) -> Color {
        match self {
            WeaponDoctrine::Projectile => Color::srgb(1.0, 0.8, 0.4), // Yellow-orange tracer
            WeaponDoctrine::Laser => Color::srgb(1.0, 0.9, 0.3),      // Golden beam
            WeaponDoctrine::Missile => Color::srgb(0.8, 0.9, 1.0),    // White-blue exhaust
            WeaponDoctrine::Hybrid => Color::srgb(0.4, 1.0, 0.6),     // Green plasma
        }
    }
}

/// Tank doctrine types
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TankDoctrine {
    Shield, // Caldari - high shield, passive regen
    Armor,  // Amarr/Gallente - high armor, active repair
    Speed,  // Minmatar - low HP, high speed/evasion
}

// ============================================================================
// SHIP DEFINITIONS
// ============================================================================

/// Player ship definition
#[derive(Debug, Clone, Copy)]
pub struct ShipDef {
    pub type_id: u32,
    pub name: &'static str,
    pub class: ShipClass,
    pub role: &'static str,
    pub health: f32,
    pub speed: f32,
    pub fire_rate: f32,
    pub damage: f32,
    pub special: &'static str,
    pub unlock_stage: u32, // 0 = always available
}

/// Enemy ship definition
#[derive(Debug, Clone, Copy)]
pub struct EnemyShipDef {
    pub type_id: u32,
    pub name: &'static str,
    pub class: ShipClass,
    pub health: f32,
    pub speed: f32,
    pub damage: f32,
    pub spawn_weight: u32,
    pub score: u32,
}

/// Ship class
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ShipClass {
    Frigate,
    AssaultFrigate,
    Interceptor,
    Destroyer,
    TacticalDestroyer,
    Cruiser,
    Battlecruiser,
    Battleship,
}

impl ShipClass {
    pub fn name(&self) -> &'static str {
        match self {
            ShipClass::Frigate => "Frigate",
            ShipClass::AssaultFrigate => "Assault Frigate",
            ShipClass::Interceptor => "Interceptor",
            ShipClass::Destroyer => "Destroyer",
            ShipClass::TacticalDestroyer => "Tactical Destroyer",
            ShipClass::Cruiser => "Cruiser",
            ShipClass::Battlecruiser => "Battlecruiser",
            ShipClass::Battleship => "Battleship",
        }
    }
}

// ============================================================================
// MINMATAR SHIPS
// ============================================================================

pub const MINMATAR_SHIPS: &[ShipDef] = &[
    ShipDef {
        type_id: 587,
        name: "Rifter",
        class: ShipClass::Frigate,
        role: "Balanced Brawler",
        health: 100.0,
        speed: 350.0,
        fire_rate: 8.0,
        damage: 10.0,
        special: "Overdrive: +50% speed burst",
        unlock_stage: 0,
    },
    ShipDef {
        type_id: 585,
        name: "Slasher",
        class: ShipClass::Frigate,
        role: "Fast Interceptor",
        health: 70.0,
        speed: 420.0,
        fire_rate: 10.0,
        damage: 7.0,
        special: "Afterburner: Invulnerable dash",
        unlock_stage: 0,
    },
    ShipDef {
        type_id: 598,
        name: "Breacher",
        class: ShipClass::Frigate,
        role: "Rocket Specialist",
        health: 110.0,
        speed: 320.0,
        fire_rate: 4.0,
        damage: 18.0,
        special: "Rocket Barrage: Triple spread",
        unlock_stage: 0,
    },
    ShipDef {
        type_id: 11371,
        name: "Wolf",
        class: ShipClass::AssaultFrigate,
        role: "Heavy Autocannon",
        health: 150.0,
        speed: 340.0,
        fire_rate: 12.0,
        damage: 15.0,
        special: "Gyrostabilizer: +100% fire rate",
        unlock_stage: 2,
    },
    ShipDef {
        type_id: 11400,
        name: "Jaguar",
        class: ShipClass::AssaultFrigate,
        role: "Rocket Swarm",
        health: 140.0,
        speed: 380.0,
        fire_rate: 3.0,
        damage: 25.0,
        special: "Rocket Swarm: Tracking missiles",
        unlock_stage: 3,
    },
];

pub const MINMATAR_ENEMIES: &[EnemyShipDef] = &[
    EnemyShipDef {
        type_id: 587,
        name: "Rifter",
        class: ShipClass::Frigate,
        health: 50.0,
        speed: 180.0,
        damage: 8.0,
        spawn_weight: 30,
        score: 100,
    },
    EnemyShipDef {
        type_id: 585,
        name: "Slasher",
        class: ShipClass::Frigate,
        health: 35.0,
        speed: 220.0,
        damage: 5.0,
        spawn_weight: 25,
        score: 75,
    },
    EnemyShipDef {
        type_id: 598,
        name: "Breacher",
        class: ShipClass::Frigate,
        health: 60.0,
        speed: 150.0,
        damage: 12.0,
        spawn_weight: 20,
        score: 125,
    },
];

// ============================================================================
// AMARR SHIPS
// ============================================================================

pub const AMARR_SHIPS: &[ShipDef] = &[
    ShipDef {
        type_id: 589,
        name: "Executioner",
        class: ShipClass::Frigate,
        role: "Laser Interceptor",
        health: 90.0,
        speed: 380.0,
        fire_rate: 6.0,
        damage: 12.0,
        special: "Scorch: Extended laser range",
        unlock_stage: 0,
    },
    ShipDef {
        type_id: 597,
        name: "Punisher",
        class: ShipClass::Frigate,
        role: "Armored Brawler",
        health: 140.0,
        speed: 300.0,
        fire_rate: 5.0,
        damage: 14.0,
        special: "Armor Hardener: -50% damage",
        unlock_stage: 0,
    },
    ShipDef {
        type_id: 591,
        name: "Tormentor",
        class: ShipClass::Frigate,
        role: "Drone Support",
        health: 100.0,
        speed: 340.0,
        fire_rate: 7.0,
        damage: 10.0,
        special: "Deploy Drone: Autonomous fighter",
        unlock_stage: 0,
    },
    ShipDef {
        type_id: 11186,
        name: "Crusader",
        class: ShipClass::Interceptor,
        role: "Fast Strike",
        health: 80.0,
        speed: 450.0,
        fire_rate: 10.0,
        damage: 8.0,
        special: "Microwarpdrive: Extreme speed",
        unlock_stage: 2,
    },
    ShipDef {
        type_id: 11184,
        name: "Malediction",
        class: ShipClass::Interceptor,
        role: "Rocket Interceptor",
        health: 75.0,
        speed: 440.0,
        fire_rate: 5.0,
        damage: 15.0,
        special: "Tackle: Slow enemies on hit",
        unlock_stage: 3,
    },
];

pub const AMARR_ENEMIES: &[EnemyShipDef] = &[
    EnemyShipDef {
        type_id: 589,
        name: "Executioner",
        class: ShipClass::Frigate,
        health: 45.0,
        speed: 200.0,
        damage: 10.0,
        spawn_weight: 30,
        score: 100,
    },
    EnemyShipDef {
        type_id: 597,
        name: "Punisher",
        class: ShipClass::Frigate,
        health: 80.0,
        speed: 140.0,
        damage: 12.0,
        spawn_weight: 25,
        score: 150,
    },
    EnemyShipDef {
        type_id: 591,
        name: "Tormentor",
        class: ShipClass::Frigate,
        health: 55.0,
        speed: 170.0,
        damage: 8.0,
        spawn_weight: 20,
        score: 100,
    },
    EnemyShipDef {
        type_id: 16236,
        name: "Coercer",
        class: ShipClass::Destroyer,
        health: 120.0,
        speed: 120.0,
        damage: 18.0,
        spawn_weight: 15,
        score: 250,
    },
    EnemyShipDef {
        type_id: 24690,
        name: "Harbinger",
        class: ShipClass::Battlecruiser,
        health: 400.0,
        speed: 80.0,
        damage: 30.0,
        spawn_weight: 5,
        score: 500,
    },
];

// ============================================================================
// CALDARI SHIPS
// ============================================================================

pub const CALDARI_SHIPS: &[ShipDef] = &[
    ShipDef {
        type_id: 602,
        name: "Kestrel",
        class: ShipClass::Frigate,
        role: "Missile Boat",
        health: 95.0,
        speed: 340.0,
        fire_rate: 4.0,
        damage: 16.0,
        special: "Salvo: 4 missiles at once",
        unlock_stage: 0,
    },
    ShipDef {
        type_id: 603,
        name: "Merlin",
        class: ShipClass::Frigate,
        role: "Shield Brawler",
        health: 120.0,
        speed: 310.0,
        fire_rate: 6.0,
        damage: 11.0,
        special: "Shield Boost: Instant regen",
        unlock_stage: 0,
    },
    ShipDef {
        type_id: 583,
        name: "Condor",
        class: ShipClass::Frigate,
        role: "Fast Tackler",
        health: 70.0,
        speed: 420.0,
        fire_rate: 5.0,
        damage: 12.0,
        special: "Warp Disruptor: Slow enemies",
        unlock_stage: 0,
    },
    ShipDef {
        type_id: 11381,
        name: "Hawk",
        class: ShipClass::AssaultFrigate,
        role: "Assault Missile",
        health: 130.0,
        speed: 330.0,
        fire_rate: 5.0,
        damage: 20.0,
        special: "Assault Launchers: +50% damage",
        unlock_stage: 2,
    },
    ShipDef {
        type_id: 11387,
        name: "Harpy",
        class: ShipClass::AssaultFrigate,
        role: "Railgun Sniper",
        health: 110.0,
        speed: 350.0,
        fire_rate: 3.0,
        damage: 28.0,
        special: "Optimal Range: Bonus at distance",
        unlock_stage: 2,
    },
    ShipDef {
        type_id: 35683,
        name: "Jackdaw",
        class: ShipClass::TacticalDestroyer,
        role: "Mode Switcher",
        health: 180.0,
        speed: 300.0,
        fire_rate: 6.0,
        damage: 22.0,
        special: "Mode Switch: Defense/Speed/Sniper",
        unlock_stage: 4,
    },
];

pub const CALDARI_ENEMIES: &[EnemyShipDef] = &[
    EnemyShipDef {
        type_id: 602,
        name: "Kestrel",
        class: ShipClass::Frigate,
        health: 50.0,
        speed: 170.0,
        damage: 12.0,
        spawn_weight: 30,
        score: 100,
    },
    EnemyShipDef {
        type_id: 603,
        name: "Merlin",
        class: ShipClass::Frigate,
        health: 70.0,
        speed: 150.0,
        damage: 9.0,
        spawn_weight: 25,
        score: 125,
    },
    EnemyShipDef {
        type_id: 583,
        name: "Condor",
        class: ShipClass::Frigate,
        health: 40.0,
        speed: 220.0,
        damage: 8.0,
        spawn_weight: 25,
        score: 75,
    },
    EnemyShipDef {
        type_id: 16238,
        name: "Cormorant",
        class: ShipClass::Destroyer,
        health: 100.0,
        speed: 130.0,
        damage: 15.0,
        spawn_weight: 12,
        score: 200,
    },
    EnemyShipDef {
        type_id: 24688,
        name: "Drake",
        class: ShipClass::Battlecruiser,
        health: 450.0,
        speed: 70.0,
        damage: 25.0,
        spawn_weight: 5,
        score: 500,
    },
];

// ============================================================================
// GALLENTE SHIPS
// ============================================================================

pub const GALLENTE_SHIPS: &[ShipDef] = &[
    ShipDef {
        type_id: 593,
        name: "Tristan",
        class: ShipClass::Frigate,
        role: "Drone Boat",
        health: 100.0,
        speed: 340.0,
        fire_rate: 6.0,
        damage: 8.0,
        special: "Drones: 2 autonomous fighters",
        unlock_stage: 0,
    },
    ShipDef {
        type_id: 594,
        name: "Incursus",
        class: ShipClass::Frigate,
        role: "Armor Brawler",
        health: 130.0,
        speed: 320.0,
        fire_rate: 8.0,
        damage: 10.0,
        special: "Armor Repair: Heal over time",
        unlock_stage: 0,
    },
    ShipDef {
        type_id: 592,
        name: "Atron",
        class: ShipClass::Frigate,
        role: "Blaster Interceptor",
        health: 75.0,
        speed: 410.0,
        fire_rate: 12.0,
        damage: 6.0,
        special: "Close Range: +100% damage in melee",
        unlock_stage: 0,
    },
    ShipDef {
        type_id: 11373,
        name: "Enyo",
        class: ShipClass::AssaultFrigate,
        role: "Heavy Blaster",
        health: 150.0,
        speed: 310.0,
        fire_rate: 10.0,
        damage: 14.0,
        special: "Void: Maximum damage ammo",
        unlock_stage: 2,
    },
    ShipDef {
        type_id: 11377,
        name: "Ishkur",
        class: ShipClass::AssaultFrigate,
        role: "Assault Drones",
        health: 140.0,
        speed: 330.0,
        fire_rate: 5.0,
        damage: 10.0,
        special: "Heavy Drones: 3 strong fighters",
        unlock_stage: 2,
    },
    ShipDef {
        type_id: 35685,
        name: "Hecate",
        class: ShipClass::TacticalDestroyer,
        role: "Mode Switcher",
        health: 160.0,
        speed: 320.0,
        fire_rate: 10.0,
        damage: 18.0,
        special: "Mode Switch: Defense/Speed/Sniper",
        unlock_stage: 4,
    },
];

pub const GALLENTE_ENEMIES: &[EnemyShipDef] = &[
    EnemyShipDef {
        type_id: 593,
        name: "Tristan",
        class: ShipClass::Frigate,
        health: 55.0,
        speed: 170.0,
        damage: 7.0,
        spawn_weight: 30,
        score: 100,
    },
    EnemyShipDef {
        type_id: 594,
        name: "Incursus",
        class: ShipClass::Frigate,
        health: 75.0,
        speed: 160.0,
        damage: 9.0,
        spawn_weight: 25,
        score: 125,
    },
    EnemyShipDef {
        type_id: 592,
        name: "Atron",
        class: ShipClass::Frigate,
        health: 40.0,
        speed: 220.0,
        damage: 6.0,
        spawn_weight: 25,
        score: 75,
    },
    EnemyShipDef {
        type_id: 16242,
        name: "Catalyst",
        class: ShipClass::Destroyer,
        health: 90.0,
        speed: 140.0,
        damage: 20.0,
        spawn_weight: 12,
        score: 200,
    },
    EnemyShipDef {
        type_id: 24700,
        name: "Myrmidon",
        class: ShipClass::Battlecruiser,
        health: 380.0,
        speed: 85.0,
        damage: 22.0,
        spawn_weight: 5,
        score: 450,
    },
];

// ============================================================================
// ACTIVE GAME STATE
// ============================================================================

/// Current game session state - which factions are in play
#[derive(Resource, Default, Clone)]
pub struct GameSession {
    pub player_faction: Faction,
    pub enemy_faction: Faction,
    pub selected_ship_index: usize,
}

impl GameSession {
    pub fn new(player: Faction, enemy: Faction) -> Self {
        Self {
            player_faction: player,
            enemy_faction: enemy,
            selected_ship_index: 0,
        }
    }

    pub fn player_ships(&self) -> &'static [ShipDef] {
        self.player_faction.player_ships()
    }

    pub fn enemy_ships(&self) -> &'static [EnemyShipDef] {
        self.enemy_faction.enemy_ships()
    }

    pub fn selected_ship(&self) -> &'static ShipDef {
        let ships = self.player_ships();
        &ships[self.selected_ship_index.min(ships.len() - 1)]
    }

    /// Get a random enemy based on spawn weights
    pub fn random_enemy(&self) -> &'static EnemyShipDef {
        let enemies = self.enemy_ships();
        let total_weight: u32 = enemies.iter().map(|e| e.spawn_weight).sum();
        let roll = fastrand::u32(0..total_weight);

        let mut cumulative = 0;
        for enemy in enemies {
            cumulative += enemy.spawn_weight;
            if roll < cumulative {
                return enemy;
            }
        }
        &enemies[0]
    }
}
