//! Shared Game Resources
//!
//! Global state that persists across systems.

use bevy::prelude::*;

/// Player score and combo system
#[derive(Debug, Clone, Resource)]
pub struct ScoreSystem {
    /// Current score
    pub score: u64,
    /// Current multiplier (1.0 - 99.9)
    pub multiplier: f32,
    /// Kill chain count
    pub chain: u32,
    /// Time remaining to maintain chain
    pub chain_timer: f32,
    /// Maximum chain time
    pub max_chain_time: f32,
    /// Style points earned
    pub style_points: u32,
    /// No damage bonus active
    pub no_damage_bonus: bool,
    /// Souls liberated count (Elder Fleet campaign)
    pub souls_liberated: u32,
}

impl Default for ScoreSystem {
    fn default() -> Self {
        Self {
            score: 0,
            multiplier: 1.0,
            chain: 0,
            chain_timer: 0.0,
            max_chain_time: 2.0,
            style_points: 0,
            no_damage_bonus: true,
            souls_liberated: 0,
        }
    }
}

impl ScoreSystem {
    /// Add points with current multiplier
    pub fn add_score(&mut self, base_points: u64) {
        let final_points = (base_points as f32 * self.multiplier) as u64;
        self.score += final_points;
    }

    /// Register a kill and extend chain
    pub fn on_kill(&mut self, base_points: u64) {
        self.chain += 1;
        self.chain_timer = self.max_chain_time;
        self.multiplier = (1.0 + self.chain as f32 * 0.1).min(99.9);
        self.add_score(base_points);
    }

    /// Update chain timer (call each frame)
    pub fn update(&mut self, dt: f32) {
        if self.chain > 0 {
            self.chain_timer -= dt;
            if self.chain_timer <= 0.0 {
                self.chain = 0;
                self.multiplier = 1.0;
            }
        }
    }

    /// Get style grade based on average multiplier
    pub fn get_grade(&self) -> StyleGrade {
        match self.multiplier {
            m if m >= 50.0 => StyleGrade::SSS,
            m if m >= 20.0 => StyleGrade::SS,
            m if m >= 10.0 => StyleGrade::S,
            m if m >= 5.0 => StyleGrade::A,
            m if m >= 3.0 => StyleGrade::B,
            m if m >= 1.5 => StyleGrade::C,
            _ => StyleGrade::D,
        }
    }

    /// Reset for new stage
    pub fn reset_stage(&mut self) {
        self.chain = 0;
        self.chain_timer = 0.0;
        self.multiplier = 1.0;
        self.no_damage_bonus = true;
    }

    /// Reset for new game
    pub fn reset_game(&mut self) {
        *self = Self::default();
    }
}

/// Style grades (like Devil May Cry)
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum StyleGrade {
    D,
    C,
    B,
    A,
    S,
    SS,
    SSS,
}

impl StyleGrade {
    pub fn as_str(&self) -> &'static str {
        match self {
            StyleGrade::D => "D",
            StyleGrade::C => "C",
            StyleGrade::B => "B",
            StyleGrade::A => "A",
            StyleGrade::S => "S",
            StyleGrade::SS => "SS",
            StyleGrade::SSS => "SSS",
        }
    }

    pub fn color(&self) -> Color {
        match self {
            StyleGrade::D => Color::srgb(0.5, 0.5, 0.5),
            StyleGrade::C => Color::srgb(0.6, 0.6, 0.4),
            StyleGrade::B => Color::srgb(0.3, 0.7, 0.3),
            StyleGrade::A => Color::srgb(0.3, 0.5, 0.9),
            StyleGrade::S => Color::srgb(0.9, 0.7, 0.2),
            StyleGrade::SS => Color::srgb(1.0, 0.5, 0.2),
            StyleGrade::SSS => Color::srgb(1.0, 0.2, 0.2),
        }
    }
}

/// Berserk mode meter and state
#[derive(Debug, Clone, Resource)]
pub struct BerserkSystem {
    /// Current meter (0.0 - 100.0)
    pub meter: f32,
    /// Meter gain per kill
    pub gain_per_kill: f32,
    /// Meter gain per graze (near miss)
    pub gain_per_graze: f32,
    /// Meter decay rate per second
    pub decay_rate: f32,
    /// Whether berserk mode is active
    pub is_active: bool,
    /// Remaining berserk duration
    pub timer: f32,
    /// Total berserk duration
    pub duration: f32,
}

impl Default for BerserkSystem {
    fn default() -> Self {
        Self {
            meter: 0.0,
            gain_per_kill: 5.0,
            gain_per_graze: 1.0,
            decay_rate: 2.0,
            is_active: false,
            timer: 0.0,
            duration: 10.0,
        }
    }
}

impl BerserkSystem {
    /// Add meter on kill
    pub fn on_kill(&mut self) {
        if !self.is_active {
            self.meter = (self.meter + self.gain_per_kill).min(100.0);
        }
    }

    /// Add meter on graze (bullet near miss)
    pub fn on_graze(&mut self) {
        if !self.is_active {
            self.meter = (self.meter + self.gain_per_graze).min(100.0);
        }
    }

    /// Attempt to activate berserk mode
    pub fn activate(&mut self) -> bool {
        if self.meter >= 100.0 && !self.is_active {
            self.is_active = true;
            self.timer = self.duration;
            self.meter = 0.0;
            return true;
        }
        false
    }

    /// Update berserk state (call each frame)
    pub fn update(&mut self, dt: f32) {
        if self.is_active {
            self.timer -= dt;
            if self.timer <= 0.0 {
                self.is_active = false;
            }
        } else {
            // Decay meter when not active and not at max
            if self.meter > 0.0 && self.meter < 100.0 {
                self.meter = (self.meter - self.decay_rate * dt).max(0.0);
            }
        }
    }

    /// Get damage multiplier
    pub fn damage_mult(&self) -> f32 {
        if self.is_active { 2.0 } else { 1.0 }
    }

    /// Get speed multiplier
    pub fn speed_mult(&self) -> f32 {
        if self.is_active { 1.5 } else { 1.0 }
    }
}

/// Game currency and progression
#[derive(Debug, Clone, Resource, Default)]
pub struct GameProgress {
    /// In-run currency for upgrades
    pub credits: u64,
    /// Lifetime currency for unlocks
    pub isk: u64,
    /// Highest stage reached
    pub highest_stage: u32,
    /// Campaigns completed
    pub campaigns_completed: Vec<String>,
    /// Ships unlocked
    pub ships_unlocked: Vec<u32>,
    /// Achievements unlocked
    pub achievements: Vec<String>,
}

/// Player input configuration
#[derive(Debug, Clone, Resource)]
pub struct InputConfig {
    pub controller_enabled: bool,
    pub controller_deadzone: f32,
    pub keyboard_enabled: bool,
    pub mouse_enabled: bool,
}

impl Default for InputConfig {
    fn default() -> Self {
        Self {
            controller_enabled: true,
            controller_deadzone: 0.15,
            keyboard_enabled: true,
            mouse_enabled: true,
        }
    }
}

/// Audio settings
#[derive(Debug, Clone, Resource)]
pub struct AudioSettings {
    pub master_volume: f32,
    pub music_volume: f32,
    pub sfx_volume: f32,
    pub music_enabled: bool,
    pub sfx_enabled: bool,
}

impl Default for AudioSettings {
    fn default() -> Self {
        Self {
            master_volume: 1.0,
            music_volume: 0.7,
            sfx_volume: 0.8,
            music_enabled: true,
            sfx_enabled: true,
        }
    }
}
