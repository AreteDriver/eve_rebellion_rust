//! EVE Rebellion Enhanced Scoring System
//!
//! Adds to existing BerserkSystem:
//! - Combo: Consecutive kill chains with timer
//! - Heat: Weapon overheating for bonus multiplier

#![allow(dead_code)]

use bevy::prelude::*;

/// Combo timeout in seconds
pub const COMBO_TIMEOUT: f32 = 2.0;

/// Combo bonus thresholds
pub const COMBO_TIER_1: u32 = 5; // 1.2x
pub const COMBO_TIER_2: u32 = 10; // 1.5x
pub const COMBO_TIER_3: u32 = 20; // 2.0x
pub const COMBO_TIER_4: u32 = 50; // 3.0x

/// Heat level classification
/// Heat system matches Python EVE Rebellion:
/// - Overheat at 100, exit at 50
/// - Doesn't block firing, just slows fire rate by 30% when overheated
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum HeatLevel {
    #[default]
    Cool, // 0-50%, normal operation
    Warm,       // 50-75%, score bonus
    Hot,        // 75-100%, higher score bonus
    Overheated, // Entered overheat - slower fire rate until heat drops to 50
}

impl HeatLevel {
    pub fn from_heat(heat: f32, was_overheated: bool) -> Self {
        // Hysteresis: once overheated, stay overheated until heat < 50
        if (was_overheated && heat > 50.0) || heat >= 100.0 {
            HeatLevel::Overheated
        } else if heat >= 75.0 {
            HeatLevel::Hot
        } else if heat >= 50.0 {
            HeatLevel::Warm
        } else {
            HeatLevel::Cool
        }
    }

    pub fn score_multiplier(&self) -> f32 {
        match self {
            HeatLevel::Cool => 1.0,
            HeatLevel::Warm => 1.25,
            HeatLevel::Hot => 1.5,
            HeatLevel::Overheated => 2.0, // High risk, high reward
        }
    }

    /// Fire rate multiplier (1.0 = normal, <1.0 = slower)
    pub fn fire_rate_mult(&self) -> f32 {
        match self {
            HeatLevel::Cool => 1.0,
            HeatLevel::Warm => 1.0,
            HeatLevel::Hot => 1.0,
            HeatLevel::Overheated => 0.7, // 30% slower when overheated
        }
    }

    pub fn color(&self) -> Color {
        match self {
            HeatLevel::Cool => Color::srgb(0.3, 0.5, 1.0),
            HeatLevel::Warm => Color::srgb(1.0, 1.0, 0.0),
            HeatLevel::Hot => Color::srgb(1.0, 0.5, 0.0),
            HeatLevel::Overheated => Color::srgb(1.0, 0.0, 0.0),
        }
    }
}

/// Combo and Heat tracking (works with existing BerserkSystem)
/// Heat values match Python EVE Rebellion
#[derive(Resource, Debug)]
pub struct ComboHeatSystem {
    // === Combo System ===
    pub combo_count: u32,
    pub combo_timer: f32,
    pub max_combo: u32,

    // === Heat System (matches Python game) ===
    pub heat: f32,
    pub heat_per_shot: f32,   // 2.0 per shot
    pub heat_decay_rate: f32, // ~72/second (1.2/frame at 60fps)
    pub heat_level: HeatLevel,

    // === Stats ===
    pub total_kills: u32,
    pub souls_liberated: u32,
}

impl Default for ComboHeatSystem {
    fn default() -> Self {
        Self {
            combo_count: 0,
            combo_timer: 0.0,
            max_combo: 0,
            heat: 0.0,
            heat_per_shot: 2.0,    // Python: 2.0 per shot
            heat_decay_rate: 72.0, // Python: 1.2/frame * 60fps
            heat_level: HeatLevel::Cool,
            total_kills: 0,
            souls_liberated: 0,
        }
    }
}

impl ComboHeatSystem {
    /// Reset for new stage
    pub fn reset(&mut self) {
        self.combo_count = 0;
        self.combo_timer = 0.0;
        self.heat = 0.0;
        self.heat_level = HeatLevel::Cool;
        self.total_kills = 0;
        self.souls_liberated = 0;
    }

    /// Update per frame
    pub fn update(&mut self, dt: f32) {
        // Update combo timer
        if self.combo_count > 0 {
            self.combo_timer -= dt;
            if self.combo_timer <= 0.0 {
                if self.combo_count > self.max_combo {
                    self.max_combo = self.combo_count;
                }
                self.combo_count = 0;
            }
        }

        // Decay heat
        let was_overheated = self.heat_level == HeatLevel::Overheated;
        if self.heat > 0.0 {
            self.heat = (self.heat - self.heat_decay_rate * dt).max(0.0);
        }
        // Update heat level with hysteresis
        self.heat_level = HeatLevel::from_heat(self.heat, was_overheated);
    }

    /// Called when player fires weapon - returns true if can fire
    /// Note: firing is NEVER blocked, just slowed when overheated
    pub fn on_fire(&mut self) {
        let was_overheated = self.heat_level == HeatLevel::Overheated;
        self.heat = (self.heat + self.heat_per_shot).min(100.0);
        self.heat_level = HeatLevel::from_heat(self.heat, was_overheated);
    }

    /// Get fire rate multiplier (1.0 = normal, 0.7 = overheated)
    pub fn fire_rate_mult(&self) -> f32 {
        self.heat_level.fire_rate_mult()
    }

    /// Check if currently overheated
    pub fn is_overheated(&self) -> bool {
        self.heat_level == HeatLevel::Overheated
    }

    /// Called when enemy is killed - returns score multiplier
    pub fn on_kill(&mut self) -> f32 {
        self.total_kills += 1;
        self.combo_count += 1;
        self.combo_timer = COMBO_TIMEOUT;

        self.combo_multiplier() * self.heat_level.score_multiplier()
    }

    /// Get current combo multiplier
    pub fn combo_multiplier(&self) -> f32 {
        if self.combo_count >= COMBO_TIER_4 {
            3.0
        } else if self.combo_count >= COMBO_TIER_3 {
            2.0
        } else if self.combo_count >= COMBO_TIER_2 {
            1.5
        } else if self.combo_count >= COMBO_TIER_1 {
            1.2
        } else {
            1.0
        }
    }

    /// Get combo tier name
    pub fn combo_tier_name(&self) -> Option<&'static str> {
        if self.combo_count >= 100 {
            Some("GODLIKE!")
        } else if self.combo_count >= COMBO_TIER_4 {
            Some("UNSTOPPABLE!")
        } else if self.combo_count >= 30 {
            Some("RAMPAGE!")
        } else if self.combo_count >= COMBO_TIER_3 {
            Some("DOMINATING!")
        } else if self.combo_count >= COMBO_TIER_2 {
            Some("KILLING SPREE!")
        } else if self.combo_count >= COMBO_TIER_1 {
            Some("COMBO!")
        } else {
            None
        }
    }

    /// Reduce heat (from nanite powerup)
    pub fn reduce_heat(&mut self, amount: f32) {
        let was_overheated = self.heat_level == HeatLevel::Overheated;
        self.heat = (self.heat - amount).max(0.0);
        self.heat_level = HeatLevel::from_heat(self.heat, was_overheated);
    }

    /// Get heat percentage (0.0 - 1.0)
    pub fn heat_percent(&self) -> f32 {
        self.heat / 100.0
    }

    /// Get combo timer percentage remaining (0.0 - 1.0)
    pub fn combo_timer_percent(&self) -> f32 {
        if self.combo_count > 0 {
            self.combo_timer / COMBO_TIMEOUT
        } else {
            0.0
        }
    }
}

/// Plugin to register combo/heat system
pub struct ScoringSystemPlugin;

impl Plugin for ScoringSystemPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<ComboHeatSystem>()
            .add_systems(Update, update_combo_heat_system);
    }
}

fn update_combo_heat_system(time: Res<Time>, mut system: ResMut<ComboHeatSystem>) {
    system.update(time.delta_secs());
}
