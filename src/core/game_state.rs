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

/// Game difficulty settings
#[derive(Debug, Clone, Copy, Default, Eq, PartialEq, Hash, Resource)]
pub enum Difficulty {
    Easy,
    #[default]
    Normal,
    Hard,
    Brutal,
}

impl Difficulty {
    pub fn name(&self) -> &'static str {
        match self {
            Difficulty::Easy => "EASY",
            Difficulty::Normal => "NORMAL",
            Difficulty::Hard => "HARD",
            Difficulty::Brutal => "BRUTAL",
        }
    }

    pub fn description(&self) -> &'static str {
        match self {
            Difficulty::Easy => "Relaxed pace, forgiving damage",
            Difficulty::Normal => "Balanced challenge",
            Difficulty::Hard => "Fast enemies, more damage",
            Difficulty::Brutal => "No mercy. Good luck.",
        }
    }

    /// Enemy health multiplier
    pub fn enemy_health_mult(&self) -> f32 {
        match self {
            Difficulty::Easy => 0.7,
            Difficulty::Normal => 1.0,
            Difficulty::Hard => 1.3,
            Difficulty::Brutal => 1.8,
        }
    }

    /// Enemy damage multiplier
    pub fn enemy_damage_mult(&self) -> f32 {
        match self {
            Difficulty::Easy => 0.6,
            Difficulty::Normal => 1.0,
            Difficulty::Hard => 1.4,
            Difficulty::Brutal => 2.0,
        }
    }

    /// Enemy speed multiplier
    pub fn enemy_speed_mult(&self) -> f32 {
        match self {
            Difficulty::Easy => 0.8,
            Difficulty::Normal => 1.0,
            Difficulty::Hard => 1.2,
            Difficulty::Brutal => 1.5,
        }
    }

    /// Spawn rate multiplier
    pub fn spawn_rate_mult(&self) -> f32 {
        match self {
            Difficulty::Easy => 0.7,
            Difficulty::Normal => 1.0,
            Difficulty::Hard => 1.3,
            Difficulty::Brutal => 1.6,
        }
    }

    pub fn color(&self) -> Color {
        match self {
            Difficulty::Easy => Color::srgb(0.3, 0.8, 0.3),   // Green
            Difficulty::Normal => Color::srgb(0.3, 0.6, 1.0), // Blue
            Difficulty::Hard => Color::srgb(1.0, 0.6, 0.2),   // Orange
            Difficulty::Brutal => Color::srgb(1.0, 0.2, 0.2), // Red
        }
    }
}

/// Minmatar ships available for selection
#[derive(Debug, Clone, Copy, Eq, PartialEq, Hash, Resource, Default)]
pub enum MinmatarShip {
    #[default]
    Rifter,     // Balanced fighter
    Slasher,    // Fast interceptor
    Breacher,   // Missile boat
    Probe,      // Utility/drones
}

impl MinmatarShip {
    /// Get EVE type ID for this ship
    pub fn type_id(&self) -> u32 {
        match self {
            MinmatarShip::Rifter => 587,
            MinmatarShip::Slasher => 585,
            MinmatarShip::Breacher => 598,
            MinmatarShip::Probe => 586,
        }
    }

    pub fn name(&self) -> &'static str {
        match self {
            MinmatarShip::Rifter => "RIFTER",
            MinmatarShip::Slasher => "SLASHER",
            MinmatarShip::Breacher => "BREACHER",
            MinmatarShip::Probe => "PROBE",
        }
    }

    pub fn description(&self) -> &'static str {
        match self {
            MinmatarShip::Rifter => "Balanced assault frigate",
            MinmatarShip::Slasher => "Fast interceptor, high speed",
            MinmatarShip::Breacher => "Missile frigate, explosive damage",
            MinmatarShip::Probe => "Utility frigate with drone support",
        }
    }

    /// Base speed multiplier
    pub fn speed_mult(&self) -> f32 {
        match self {
            MinmatarShip::Rifter => 1.0,
            MinmatarShip::Slasher => 1.4,
            MinmatarShip::Breacher => 0.9,
            MinmatarShip::Probe => 1.1,
        }
    }

    /// Base damage multiplier
    pub fn damage_mult(&self) -> f32 {
        match self {
            MinmatarShip::Rifter => 1.0,
            MinmatarShip::Slasher => 0.8,
            MinmatarShip::Breacher => 1.2,
            MinmatarShip::Probe => 0.7,
        }
    }

    /// Base health multiplier
    pub fn health_mult(&self) -> f32 {
        match self {
            MinmatarShip::Rifter => 1.0,
            MinmatarShip::Slasher => 0.7,
            MinmatarShip::Breacher => 1.1,
            MinmatarShip::Probe => 0.9,
        }
    }

    /// Fire rate multiplier
    pub fn fire_rate_mult(&self) -> f32 {
        match self {
            MinmatarShip::Rifter => 1.0,
            MinmatarShip::Slasher => 1.3,
            MinmatarShip::Breacher => 0.7,
            MinmatarShip::Probe => 1.0,
        }
    }

    /// Special ability description
    pub fn special(&self) -> &'static str {
        match self {
            MinmatarShip::Rifter => "Overdrive: +50% speed burst",
            MinmatarShip::Slasher => "Phase Shift: Brief invulnerability",
            MinmatarShip::Breacher => "Missile Salvo: Triple shot",
            MinmatarShip::Probe => "Deploy Drone: Auto-turret ally",
        }
    }

    /// Get all available ships
    pub fn all() -> &'static [MinmatarShip] {
        &[
            MinmatarShip::Rifter,
            MinmatarShip::Slasher,
            MinmatarShip::Breacher,
            MinmatarShip::Probe,
        ]
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
