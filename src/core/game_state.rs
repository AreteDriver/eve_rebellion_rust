//! Game State Machine
//!
//! Defines all possible game states and Minmatar-focused enums.

use bevy::prelude::*;

/// Main game state - controls which systems run and what's displayed
#[derive(Debug, Clone, Copy, Default, Eq, PartialEq, Hash, States)]
pub enum GameState {
    #[default]
    Loading,
    MainMenu,
    ModuleSelect,    // Choose game module (Elder Fleet, Caldari vs Gallente, etc.)
    FactionSelect,   // Choose faction (for Caldari/Gallente module)
    DifficultySelect,
    ShipSelect,
    Playing,
    UpgradeShop,
    BossIntro,
    BossFight,
    StageComplete,
    GameOver,
    Victory,
    Paused,
}

/// Game difficulty settings - EVE-themed
/// (Wraps DifficultyLevel from resources.rs for backwards compatibility)
#[derive(Debug, Clone, Copy, Default, Eq, PartialEq, Hash, Resource)]
pub enum Difficulty {
    /// Carebear - High-sec living, relaxed gameplay
    Carebear,
    /// Newbro - Balanced for new pilots
    #[default]
    Newbro,
    /// Bitter Vet - Punishing for experienced pilots
    BitterVet,
    /// Triglavian - Nightmare mode, no mercy
    Triglavian,
}

impl Difficulty {
    pub fn name(&self) -> &'static str {
        match self {
            Difficulty::Carebear => "CAREBEAR",
            Difficulty::Newbro => "NEWBRO",
            Difficulty::BitterVet => "BITTER VET",
            Difficulty::Triglavian => "TRIGLAVIAN",
        }
    }

    pub fn tagline(&self) -> &'static str {
        match self {
            Difficulty::Carebear => "High-sec living",
            Difficulty::Newbro => "Welcome to New Eden",
            Difficulty::BitterVet => "I remember when...",
            Difficulty::Triglavian => "Clade proving grounds",
        }
    }

    pub fn description(&self) -> &'static str {
        match self {
            Difficulty::Carebear => "Relaxed gameplay with generous shields and forgiving combat.",
            Difficulty::Newbro => "Balanced experience for new pilots. Fair challenge with room to learn.",
            Difficulty::BitterVet => "Punishing difficulty for experienced pilots. Enemies hit hard.",
            Difficulty::Triglavian => "Nightmare mode. One-shot kills, relentless enemies, no mercy.",
        }
    }

    /// Enemy health multiplier
    pub fn enemy_health_mult(&self) -> f32 {
        match self {
            Difficulty::Carebear => 0.7,
            Difficulty::Newbro => 1.0,
            Difficulty::BitterVet => 1.3,
            Difficulty::Triglavian => 1.5,
        }
    }

    /// Enemy damage multiplier
    pub fn enemy_damage_mult(&self) -> f32 {
        match self {
            Difficulty::Carebear => 0.5,
            Difficulty::Newbro => 1.0,
            Difficulty::BitterVet => 1.5,
            Difficulty::Triglavian => 3.0,
        }
    }

    /// Enemy speed multiplier
    pub fn enemy_speed_mult(&self) -> f32 {
        match self {
            Difficulty::Carebear => 0.85,
            Difficulty::Newbro => 1.0,
            Difficulty::BitterVet => 1.15,
            Difficulty::Triglavian => 1.3,
        }
    }

    /// Spawn rate multiplier
    pub fn spawn_rate_mult(&self) -> f32 {
        match self {
            Difficulty::Carebear => 0.8,
            Difficulty::Newbro => 1.0,
            Difficulty::BitterVet => 1.2,
            Difficulty::Triglavian => 1.5,
        }
    }

    /// Enemy fire rate multiplier
    pub fn enemy_fire_rate_mult(&self) -> f32 {
        match self {
            Difficulty::Carebear => 0.7,
            Difficulty::Newbro => 1.0,
            Difficulty::BitterVet => 1.3,
            Difficulty::Triglavian => 1.5,
        }
    }

    /// Player shield multiplier
    pub fn player_shield_mult(&self) -> f32 {
        match self {
            Difficulty::Carebear => 2.0,
            Difficulty::Newbro => 1.0,
            Difficulty::BitterVet => 0.8,
            Difficulty::Triglavian => 0.5,
        }
    }

    /// Score multiplier for this difficulty
    pub fn score_mult(&self) -> f32 {
        match self {
            Difficulty::Carebear => 0.5,
            Difficulty::Newbro => 1.0,
            Difficulty::BitterVet => 1.5,
            Difficulty::Triglavian => 3.0,
        }
    }

    pub fn color(&self) -> Color {
        match self {
            Difficulty::Carebear => Color::srgb(0.4, 0.8, 0.4),    // Green
            Difficulty::Newbro => Color::srgb(0.4, 0.6, 1.0),      // Blue
            Difficulty::BitterVet => Color::srgb(1.0, 0.6, 0.2),   // Orange
            Difficulty::Triglavian => Color::srgb(0.8, 0.2, 0.2),  // Red
        }
    }

    /// Get all difficulty levels
    pub fn all() -> &'static [Difficulty] {
        &[
            Difficulty::Carebear,
            Difficulty::Newbro,
            Difficulty::BitterVet,
            Difficulty::Triglavian,
        ]
    }

    /// Get the next difficulty (wraps around)
    pub fn next(&self) -> Difficulty {
        match self {
            Difficulty::Carebear => Difficulty::Newbro,
            Difficulty::Newbro => Difficulty::BitterVet,
            Difficulty::BitterVet => Difficulty::Triglavian,
            Difficulty::Triglavian => Difficulty::Carebear,
        }
    }

    /// Get the previous difficulty (wraps around)
    pub fn prev(&self) -> Difficulty {
        match self {
            Difficulty::Carebear => Difficulty::Triglavian,
            Difficulty::Newbro => Difficulty::Carebear,
            Difficulty::BitterVet => Difficulty::Newbro,
            Difficulty::Triglavian => Difficulty::BitterVet,
        }
    }
}

/// Minmatar ships available for selection
#[derive(Debug, Clone, Copy, Eq, PartialEq, Hash, Resource, Default)]
pub enum MinmatarShip {
    // Frigates (always available)
    #[default]
    Rifter,     // Balanced fighter
    Slasher,    // Fast interceptor
    Breacher,   // Missile boat
    Probe,      // Utility/drones
    // Assault Frigates (unlockable)
    Wolf,       // Assault Frigate - Act 2 unlock (autocannon)
    Jaguar,     // Assault Frigate - Act 3 unlock (rockets)
}

impl MinmatarShip {
    /// Get EVE type ID for this ship
    pub fn type_id(&self) -> u32 {
        match self {
            MinmatarShip::Rifter => 587,
            MinmatarShip::Slasher => 585,
            MinmatarShip::Breacher => 598,
            MinmatarShip::Probe => 586,
            MinmatarShip::Wolf => 11371,
            MinmatarShip::Jaguar => 11377,
        }
    }

    pub fn name(&self) -> &'static str {
        match self {
            MinmatarShip::Rifter => "RIFTER",
            MinmatarShip::Slasher => "SLASHER",
            MinmatarShip::Breacher => "BREACHER",
            MinmatarShip::Probe => "PROBE",
            MinmatarShip::Wolf => "WOLF",
            MinmatarShip::Jaguar => "JAGUAR",
        }
    }

    pub fn description(&self) -> &'static str {
        match self {
            MinmatarShip::Rifter => "Balanced assault frigate",
            MinmatarShip::Slasher => "Fast interceptor, high speed",
            MinmatarShip::Breacher => "Missile frigate, explosive damage",
            MinmatarShip::Probe => "Utility frigate with drone support",
            MinmatarShip::Wolf => "Assault frigate, heavy autocannons",
            MinmatarShip::Jaguar => "Assault frigate, rocket swarm",
        }
    }

    /// Ship class name
    pub fn ship_class(&self) -> &'static str {
        match self {
            MinmatarShip::Rifter | MinmatarShip::Slasher |
            MinmatarShip::Breacher | MinmatarShip::Probe => "Frigate",
            MinmatarShip::Wolf | MinmatarShip::Jaguar => "Assault Frigate",
        }
    }

    /// Base speed multiplier
    pub fn speed_mult(&self) -> f32 {
        match self {
            MinmatarShip::Rifter => 1.0,
            MinmatarShip::Slasher => 1.4,
            MinmatarShip::Breacher => 0.9,
            MinmatarShip::Probe => 1.1,
            MinmatarShip::Wolf => 1.13,    // 340/300 base
            MinmatarShip::Jaguar => 1.27,  // 380/300 base
        }
    }

    /// Base damage multiplier
    pub fn damage_mult(&self) -> f32 {
        match self {
            MinmatarShip::Rifter => 1.0,
            MinmatarShip::Slasher => 0.8,
            MinmatarShip::Breacher => 1.2,
            MinmatarShip::Probe => 0.7,
            MinmatarShip::Wolf => 1.5,     // Heavy autocannons
            MinmatarShip::Jaguar => 1.8,   // Rocket swarm
        }
    }

    /// Base health multiplier
    pub fn health_mult(&self) -> f32 {
        match self {
            MinmatarShip::Rifter => 1.0,
            MinmatarShip::Slasher => 0.7,
            MinmatarShip::Breacher => 1.1,
            MinmatarShip::Probe => 0.9,
            MinmatarShip::Wolf => 1.5,     // 150 hull
            MinmatarShip::Jaguar => 1.4,   // 140 hull
        }
    }

    /// Fire rate multiplier
    pub fn fire_rate_mult(&self) -> f32 {
        match self {
            MinmatarShip::Rifter => 1.0,
            MinmatarShip::Slasher => 1.3,
            MinmatarShip::Breacher => 0.7,
            MinmatarShip::Probe => 1.0,
            MinmatarShip::Wolf => 1.6,     // 8 fire rate
            MinmatarShip::Jaguar => 0.5,   // 2.5 fire rate, but high damage
        }
    }

    /// Special ability description
    pub fn special(&self) -> &'static str {
        match self {
            MinmatarShip::Rifter => "Overdrive: +50% speed burst",
            MinmatarShip::Slasher => "Phase Shift: Brief invulnerability",
            MinmatarShip::Breacher => "Missile Salvo: Triple shot",
            MinmatarShip::Probe => "Deploy Drone: Auto-turret ally",
            MinmatarShip::Wolf => "Gyrostabilizer: +100% fire rate burst",
            MinmatarShip::Jaguar => "Rocket Swarm: Triple tracking rockets",
        }
    }

    /// Whether this ship requires an unlock
    pub fn requires_unlock(&self) -> bool {
        matches!(self, MinmatarShip::Wolf | MinmatarShip::Jaguar)
    }

    /// Which act unlocks this ship (0 = always available)
    pub fn unlock_act(&self) -> u32 {
        match self {
            MinmatarShip::Rifter | MinmatarShip::Slasher |
            MinmatarShip::Breacher | MinmatarShip::Probe => 0,
            MinmatarShip::Wolf => 2,
            MinmatarShip::Jaguar => 3,
        }
    }

    /// Get all base frigate ships (always available)
    pub fn all() -> &'static [MinmatarShip] {
        &[
            MinmatarShip::Rifter,
            MinmatarShip::Slasher,
            MinmatarShip::Breacher,
            MinmatarShip::Probe,
        ]
    }

    /// Get all ships including unlockables
    pub fn all_including_unlocks() -> &'static [MinmatarShip] {
        &[
            MinmatarShip::Rifter,
            MinmatarShip::Slasher,
            MinmatarShip::Breacher,
            MinmatarShip::Probe,
            MinmatarShip::Wolf,
            MinmatarShip::Jaguar,
        ]
    }
}

/// Tracks which ships and acts the player has unlocked
#[derive(Debug, Clone, Resource)]
pub struct ShipUnlocks {
    /// Highest act completed
    pub highest_act_completed: u32,
    /// Specific ships that have been unlocked
    pub unlocked_ships: Vec<MinmatarShip>,
}

impl Default for ShipUnlocks {
    fn default() -> Self {
        Self {
            highest_act_completed: 0,
            unlocked_ships: vec![
                MinmatarShip::Rifter,
                MinmatarShip::Slasher,
                MinmatarShip::Breacher,
                MinmatarShip::Probe,
            ],
        }
    }
}

impl ShipUnlocks {
    /// Check if a ship is unlocked
    pub fn is_unlocked(&self, ship: MinmatarShip) -> bool {
        if !ship.requires_unlock() {
            return true;
        }
        self.unlocked_ships.contains(&ship) || self.highest_act_completed >= ship.unlock_act()
    }

    /// Unlock a specific ship
    pub fn unlock_ship(&mut self, ship: MinmatarShip) {
        if !self.unlocked_ships.contains(&ship) {
            self.unlocked_ships.push(ship);
        }
    }

    /// Complete an act and unlock associated ships
    pub fn complete_act(&mut self, act: u32) {
        if act > self.highest_act_completed {
            self.highest_act_completed = act;

            // Unlock ships associated with this act
            match act {
                2 => self.unlock_ship(MinmatarShip::Wolf),
                3 => self.unlock_ship(MinmatarShip::Jaguar),
                _ => {}
            }
        }
    }

    /// Get list of available ships (unlocked only)
    pub fn available_ships(&self) -> Vec<MinmatarShip> {
        MinmatarShip::all_including_unlocks()
            .iter()
            .filter(|s| self.is_unlocked(**s))
            .copied()
            .collect()
    }
}

/// Selected ship for current run
#[derive(Debug, Clone, Resource, Default)]
pub struct SelectedShip {
    pub ship: MinmatarShip,
}

/// Current stage/level being played
#[derive(Debug, Clone, Resource)]
pub struct CurrentStage {
    pub stage_number: u32,
    pub wave_number: u32,
    pub total_waves: u32,
    pub is_boss_stage: bool,
}

impl Default for CurrentStage {
    fn default() -> Self {
        Self {
            stage_number: 1,
            wave_number: 1,
            total_waves: 5,
            is_boss_stage: false,
        }
    }
}
