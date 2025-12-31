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

#[cfg(test)]
#[allow(clippy::field_reassign_with_default)]
mod tests {
    use super::*;

    // ==================== HeatLevel Tests ====================

    #[test]
    fn heat_level_cool_range() {
        assert_eq!(HeatLevel::from_heat(0.0, false), HeatLevel::Cool);
        assert_eq!(HeatLevel::from_heat(25.0, false), HeatLevel::Cool);
        assert_eq!(HeatLevel::from_heat(49.9, false), HeatLevel::Cool);
    }

    #[test]
    fn heat_level_warm_range() {
        assert_eq!(HeatLevel::from_heat(50.0, false), HeatLevel::Warm);
        assert_eq!(HeatLevel::from_heat(60.0, false), HeatLevel::Warm);
        assert_eq!(HeatLevel::from_heat(74.9, false), HeatLevel::Warm);
    }

    #[test]
    fn heat_level_hot_range() {
        assert_eq!(HeatLevel::from_heat(75.0, false), HeatLevel::Hot);
        assert_eq!(HeatLevel::from_heat(85.0, false), HeatLevel::Hot);
        assert_eq!(HeatLevel::from_heat(99.9, false), HeatLevel::Hot);
    }

    #[test]
    fn heat_level_overheated_at_100() {
        assert_eq!(HeatLevel::from_heat(100.0, false), HeatLevel::Overheated);
    }

    #[test]
    fn heat_level_hysteresis_stays_overheated_above_50() {
        // Once overheated, stay overheated until heat drops below 50
        assert_eq!(HeatLevel::from_heat(80.0, true), HeatLevel::Overheated);
        assert_eq!(HeatLevel::from_heat(60.0, true), HeatLevel::Overheated);
        assert_eq!(HeatLevel::from_heat(51.0, true), HeatLevel::Overheated);
    }

    #[test]
    fn heat_level_hysteresis_exits_at_50() {
        // Exit overheated when heat drops to/below 50
        assert_eq!(HeatLevel::from_heat(50.0, true), HeatLevel::Warm);
        assert_eq!(HeatLevel::from_heat(40.0, true), HeatLevel::Cool);
    }

    #[test]
    fn heat_level_score_multipliers() {
        assert_eq!(HeatLevel::Cool.score_multiplier(), 1.0);
        assert_eq!(HeatLevel::Warm.score_multiplier(), 1.25);
        assert_eq!(HeatLevel::Hot.score_multiplier(), 1.5);
        assert_eq!(HeatLevel::Overheated.score_multiplier(), 2.0);
    }

    #[test]
    fn heat_level_fire_rate_multipliers() {
        assert_eq!(HeatLevel::Cool.fire_rate_mult(), 1.0);
        assert_eq!(HeatLevel::Warm.fire_rate_mult(), 1.0);
        assert_eq!(HeatLevel::Hot.fire_rate_mult(), 1.0);
        assert_eq!(HeatLevel::Overheated.fire_rate_mult(), 0.7); // 30% slower
    }

    // ==================== ComboHeatSystem Tests ====================

    #[test]
    fn combo_heat_default_values() {
        let c = ComboHeatSystem::default();
        assert_eq!(c.combo_count, 0);
        assert_eq!(c.heat, 0.0);
        assert_eq!(c.heat_per_shot, 2.0);
        assert_eq!(c.heat_decay_rate, 72.0);
        assert_eq!(c.heat_level, HeatLevel::Cool);
    }

    #[test]
    fn combo_heat_on_kill_increments_combo() {
        let mut c = ComboHeatSystem::default();
        let mult = c.on_kill();
        assert_eq!(c.combo_count, 1);
        assert_eq!(c.combo_timer, COMBO_TIMEOUT);
        assert_eq!(mult, 1.0); // Below tier 1 threshold
    }

    #[test]
    fn combo_heat_combo_timer_decay() {
        let mut c = ComboHeatSystem::default();
        c.on_kill();
        assert_eq!(c.combo_count, 1);

        c.update(2.1); // Exceed COMBO_TIMEOUT
        assert_eq!(c.combo_count, 0);
    }

    #[test]
    fn combo_heat_max_combo_tracking() {
        let mut c = ComboHeatSystem::default();
        for _ in 0..10 {
            c.on_kill();
        }
        assert_eq!(c.combo_count, 10);
        assert_eq!(c.max_combo, 0); // Not updated until combo ends

        c.update(2.1); // Let combo expire
        assert_eq!(c.max_combo, 10);
        assert_eq!(c.combo_count, 0);
    }

    #[test]
    fn combo_heat_combo_multiplier_tiers() {
        let mut c = ComboHeatSystem::default();

        // Below tier 1
        c.combo_count = 4;
        assert_eq!(c.combo_multiplier(), 1.0);

        // Tier 1: 5+
        c.combo_count = 5;
        assert_eq!(c.combo_multiplier(), 1.2);

        // Tier 2: 10+
        c.combo_count = 10;
        assert_eq!(c.combo_multiplier(), 1.5);

        // Tier 3: 20+
        c.combo_count = 20;
        assert_eq!(c.combo_multiplier(), 2.0);

        // Tier 4: 50+
        c.combo_count = 50;
        assert_eq!(c.combo_multiplier(), 3.0);
    }

    #[test]
    fn combo_heat_tier_names() {
        let mut c = ComboHeatSystem::default();

        c.combo_count = 4;
        assert_eq!(c.combo_tier_name(), None);

        c.combo_count = 5;
        assert_eq!(c.combo_tier_name(), Some("COMBO!"));

        c.combo_count = 10;
        assert_eq!(c.combo_tier_name(), Some("KILLING SPREE!"));

        c.combo_count = 20;
        assert_eq!(c.combo_tier_name(), Some("DOMINATING!"));

        c.combo_count = 30;
        assert_eq!(c.combo_tier_name(), Some("RAMPAGE!"));

        c.combo_count = 50;
        assert_eq!(c.combo_tier_name(), Some("UNSTOPPABLE!"));

        c.combo_count = 100;
        assert_eq!(c.combo_tier_name(), Some("GODLIKE!"));
    }

    #[test]
    fn combo_heat_on_fire_adds_heat() {
        let mut c = ComboHeatSystem::default();
        c.on_fire();
        assert_eq!(c.heat, 2.0);

        c.on_fire();
        assert_eq!(c.heat, 4.0);
    }

    #[test]
    fn combo_heat_heat_caps_at_100() {
        let mut c = ComboHeatSystem::default();
        for _ in 0..100 {
            c.on_fire();
        }
        assert_eq!(c.heat, 100.0);
        assert_eq!(c.heat_level, HeatLevel::Overheated);
    }

    #[test]
    fn combo_heat_heat_decay() {
        let mut c = ComboHeatSystem::default();
        c.heat = 50.0;

        c.update(0.5); // Decay 36 units (72 * 0.5)
        assert!((c.heat - 14.0).abs() < 0.1);
    }

    #[test]
    fn combo_heat_heat_does_not_go_negative() {
        let mut c = ComboHeatSystem::default();
        c.heat = 10.0;

        c.update(1.0); // Would decay 72 units, but capped at 0
        assert_eq!(c.heat, 0.0);
    }

    #[test]
    fn combo_heat_hysteresis_in_update() {
        let mut c = ComboHeatSystem::default();

        // Heat up to overheated
        for _ in 0..50 {
            c.on_fire();
        }
        assert_eq!(c.heat_level, HeatLevel::Overheated);

        // Decay heat but stay above 50
        c.heat = 60.0;
        c.update(0.01); // Small update
        assert_eq!(c.heat_level, HeatLevel::Overheated); // Still overheated

        // Decay below 50
        c.heat = 45.0;
        let was_overheated = c.heat_level == HeatLevel::Overheated;
        c.heat_level = HeatLevel::from_heat(c.heat, was_overheated);
        assert_eq!(c.heat_level, HeatLevel::Cool); // Finally cooled
    }

    #[test]
    fn combo_heat_fire_rate_when_overheated() {
        let mut c = ComboHeatSystem::default();
        assert_eq!(c.fire_rate_mult(), 1.0);

        // Overheat
        for _ in 0..50 {
            c.on_fire();
        }
        assert_eq!(c.fire_rate_mult(), 0.7);
        assert!(c.is_overheated());
    }

    #[test]
    fn combo_heat_reduce_heat() {
        let mut c = ComboHeatSystem::default();
        c.heat = 80.0;
        c.heat_level = HeatLevel::Hot;

        c.reduce_heat(50.0);
        assert_eq!(c.heat, 30.0);
        assert_eq!(c.heat_level, HeatLevel::Cool);
    }

    #[test]
    fn combo_heat_reduce_heat_not_negative() {
        let mut c = ComboHeatSystem::default();
        c.heat = 20.0;

        c.reduce_heat(50.0);
        assert_eq!(c.heat, 0.0);
    }

    #[test]
    fn combo_heat_percentages() {
        let mut c = ComboHeatSystem::default();
        c.heat = 50.0;
        assert_eq!(c.heat_percent(), 0.5);

        c.combo_count = 5;
        c.combo_timer = 1.0;
        assert_eq!(c.combo_timer_percent(), 0.5); // 1.0 / 2.0
    }

    #[test]
    fn combo_heat_on_kill_returns_combined_multiplier() {
        let mut c = ComboHeatSystem::default();
        c.heat = 80.0; // Hot = 1.5x
        c.heat_level = HeatLevel::from_heat(c.heat, false);

        // Get to tier 1 combo (5 kills)
        for _ in 0..4 {
            c.on_kill();
        }
        let mult = c.on_kill(); // 5th kill

        // combo_multiplier (1.2) × heat_multiplier (1.5) = 1.8
        assert!((mult - 1.8).abs() < 0.01);
    }

    #[test]
    fn combo_heat_reset() {
        let mut c = ComboHeatSystem::default();
        c.combo_count = 10;
        c.heat = 80.0;
        c.total_kills = 100;
        c.souls_liberated = 50;

        c.reset();

        assert_eq!(c.combo_count, 0);
        assert_eq!(c.heat, 0.0);
        assert_eq!(c.total_kills, 0);
        assert_eq!(c.souls_liberated, 0);
        assert_eq!(c.heat_level, HeatLevel::Cool);
    }
}
