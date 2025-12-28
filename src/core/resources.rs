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

/// Berserk mode - activates after proximity kills
/// Based on design doc: 5 proximity kills (within 80 units), 8 second duration, 2x score
#[derive(Debug, Clone, Resource)]
pub struct BerserkSystem {
    /// Proximity kill counter (resets on activation or timeout)
    pub proximity_kills: u32,
    /// Kills required to activate
    pub kills_to_activate: u32,
    /// Proximity range for kills to count
    pub proximity_range: f32,
    /// Timer for kill chain (resets if no kill within window)
    pub chain_timer: f32,
    /// Chain window (seconds)
    pub chain_window: f32,
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
            proximity_kills: 0,
            kills_to_activate: 5,
            proximity_range: 80.0,
            chain_timer: 0.0,
            chain_window: 3.0, // 3 seconds to chain kills
            is_active: false,
            timer: 0.0,
            duration: 8.0, // Design doc: 8 seconds
        }
    }
}

impl BerserkSystem {
    /// Register a kill. Returns true if berserk activated.
    /// `distance` is distance from player to killed enemy.
    pub fn on_kill_at_distance(&mut self, distance: f32) -> bool {
        if self.is_active {
            return false; // Already active
        }

        // Check if kill is within proximity range
        if distance <= self.proximity_range {
            self.proximity_kills += 1;
            self.chain_timer = self.chain_window;

            // Check if we hit the threshold
            if self.proximity_kills >= self.kills_to_activate {
                self.is_active = true;
                self.timer = self.duration;
                self.proximity_kills = 0;
                return true;
            }
        }
        false
    }

    /// Legacy on_kill for compatibility (assumes max distance)
    pub fn on_kill(&mut self) {
        // Count as proximity kill if called directly
        self.on_kill_at_distance(0.0);
    }

    /// Update berserk state (call each frame)
    pub fn update(&mut self, dt: f32) {
        if self.is_active {
            self.timer -= dt;
            if self.timer <= 0.0 {
                self.is_active = false;
            }
        } else {
            // Decay chain timer
            if self.chain_timer > 0.0 {
                self.chain_timer -= dt;
                if self.chain_timer <= 0.0 {
                    // Chain broken, reset proximity kills
                    self.proximity_kills = 0;
                }
            }
        }
    }

    /// Get score multiplier
    pub fn score_mult(&self) -> f32 {
        if self.is_active {
            2.0
        } else {
            1.0
        }
    }

    /// Get damage multiplier
    pub fn damage_mult(&self) -> f32 {
        if self.is_active {
            2.0
        } else {
            1.0
        }
    }

    /// Get speed multiplier
    pub fn speed_mult(&self) -> f32 {
        if self.is_active {
            1.5
        } else {
            1.0
        }
    }

    /// Get progress toward berserk (0.0 - 1.0)
    pub fn progress(&self) -> f32 {
        if self.is_active {
            self.timer / self.duration
        } else {
            self.proximity_kills as f32 / self.kills_to_activate as f32
        }
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

/// Difficulty levels - EVE-themed
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Hash)]
pub enum DifficultyLevel {
    /// Easy - Relaxed gameplay, generous shields, forgiving combat
    Carebear,
    /// Normal - Balanced experience for new pilots
    #[default]
    Newbro,
    /// Hard - Punishing difficulty for experienced pilots
    BitterVet,
    /// Nightmare - One-shot kills, relentless enemies, no mercy
    Triglavian,
}

impl DifficultyLevel {
    pub fn name(&self) -> &'static str {
        match self {
            DifficultyLevel::Carebear => "CAREBEAR",
            DifficultyLevel::Newbro => "NEWBRO",
            DifficultyLevel::BitterVet => "BITTER VET",
            DifficultyLevel::Triglavian => "TRIGLAVIAN",
        }
    }

    pub fn tagline(&self) -> &'static str {
        match self {
            DifficultyLevel::Carebear => "High-sec living",
            DifficultyLevel::Newbro => "Welcome to New Eden",
            DifficultyLevel::BitterVet => "I remember when...",
            DifficultyLevel::Triglavian => "Clade proving grounds",
        }
    }

    pub fn description(&self) -> &'static str {
        match self {
            DifficultyLevel::Carebear => {
                "Relaxed gameplay with generous shields and forgiving combat."
            }
            DifficultyLevel::Newbro => {
                "Balanced experience for new pilots. Fair challenge with room to learn."
            }
            DifficultyLevel::BitterVet => {
                "Punishing difficulty for experienced pilots. Enemies hit hard."
            }
            DifficultyLevel::Triglavian => {
                "Nightmare mode. One-shot kills, relentless enemies, no mercy."
            }
        }
    }

    pub fn color(&self) -> Color {
        match self {
            DifficultyLevel::Carebear => Color::srgb(0.4, 0.8, 0.4), // Green
            DifficultyLevel::Newbro => Color::srgb(0.4, 0.6, 1.0),   // Blue
            DifficultyLevel::BitterVet => Color::srgb(1.0, 0.6, 0.2), // Orange
            DifficultyLevel::Triglavian => Color::srgb(0.8, 0.2, 0.2), // Red
        }
    }

    /// Get all difficulty levels in order
    pub fn all() -> [DifficultyLevel; 4] {
        [
            DifficultyLevel::Carebear,
            DifficultyLevel::Newbro,
            DifficultyLevel::BitterVet,
            DifficultyLevel::Triglavian,
        ]
    }

    /// Get the next difficulty (wraps around)
    pub fn next(&self) -> DifficultyLevel {
        match self {
            DifficultyLevel::Carebear => DifficultyLevel::Newbro,
            DifficultyLevel::Newbro => DifficultyLevel::BitterVet,
            DifficultyLevel::BitterVet => DifficultyLevel::Triglavian,
            DifficultyLevel::Triglavian => DifficultyLevel::Carebear,
        }
    }

    /// Get the previous difficulty (wraps around)
    pub fn prev(&self) -> DifficultyLevel {
        match self {
            DifficultyLevel::Carebear => DifficultyLevel::Triglavian,
            DifficultyLevel::Newbro => DifficultyLevel::Carebear,
            DifficultyLevel::BitterVet => DifficultyLevel::Newbro,
            DifficultyLevel::Triglavian => DifficultyLevel::BitterVet,
        }
    }
}

/// Player stat modifiers based on difficulty
#[derive(Debug, Clone, Copy)]
pub struct PlayerModifiers {
    pub hull_multiplier: f32,
    pub shield_multiplier: f32,
    pub armor_multiplier: f32,
    pub damage_multiplier: f32,
    pub capacitor_recharge_multiplier: f32,
    pub capacitor_drain_multiplier: f32,
    pub maneuver_cooldown_multiplier: f32,
    pub invincibility_duration_multiplier: f32,
}

impl Default for PlayerModifiers {
    fn default() -> Self {
        Self {
            hull_multiplier: 1.0,
            shield_multiplier: 1.0,
            armor_multiplier: 1.0,
            damage_multiplier: 1.0,
            capacitor_recharge_multiplier: 1.0,
            capacitor_drain_multiplier: 1.0,
            maneuver_cooldown_multiplier: 1.0,
            invincibility_duration_multiplier: 1.0,
        }
    }
}

/// Enemy stat modifiers based on difficulty
#[derive(Debug, Clone, Copy)]
pub struct EnemyModifiers {
    pub health_multiplier: f32,
    pub damage_multiplier: f32,
    pub fire_rate_multiplier: f32,
    pub speed_multiplier: f32,
    pub accuracy_multiplier: f32,
    pub spawn_rate_multiplier: f32,
}

impl Default for EnemyModifiers {
    fn default() -> Self {
        Self {
            health_multiplier: 1.0,
            damage_multiplier: 1.0,
            fire_rate_multiplier: 1.0,
            speed_multiplier: 1.0,
            accuracy_multiplier: 1.0,
            spawn_rate_multiplier: 1.0,
        }
    }
}

/// Boss modifiers based on difficulty
#[derive(Debug, Clone, Copy)]
pub struct BossModifiers {
    pub health_multiplier: f32,
    pub damage_multiplier: f32,
    pub attack_cooldown_multiplier: f32,
}

impl Default for BossModifiers {
    fn default() -> Self {
        Self {
            health_multiplier: 1.0,
            damage_multiplier: 1.0,
            attack_cooldown_multiplier: 1.0,
        }
    }
}

/// Scoring modifiers based on difficulty
#[derive(Debug, Clone, Copy)]
pub struct ScoringModifiers {
    pub base_score_multiplier: f32,
    pub combo_decay_multiplier: f32,
}

impl Default for ScoringModifiers {
    fn default() -> Self {
        Self {
            base_score_multiplier: 1.0,
            combo_decay_multiplier: 1.0,
        }
    }
}

/// Complete difficulty settings resource
#[derive(Debug, Clone, Resource)]
pub struct DifficultySettings {
    pub level: DifficultyLevel,
    pub player: PlayerModifiers,
    pub enemy: EnemyModifiers,
    pub boss: BossModifiers,
    pub scoring: ScoringModifiers,
}

impl Default for DifficultySettings {
    fn default() -> Self {
        Self::from_level(DifficultyLevel::default())
    }
}

impl DifficultySettings {
    /// Create settings for a specific difficulty level
    pub fn from_level(level: DifficultyLevel) -> Self {
        match level {
            DifficultyLevel::Carebear => Self {
                level,
                player: PlayerModifiers {
                    hull_multiplier: 1.5,
                    shield_multiplier: 2.0,
                    armor_multiplier: 1.5,
                    damage_multiplier: 1.2,
                    capacitor_recharge_multiplier: 1.5,
                    capacitor_drain_multiplier: 0.7,
                    maneuver_cooldown_multiplier: 0.7,
                    invincibility_duration_multiplier: 1.5,
                },
                enemy: EnemyModifiers {
                    health_multiplier: 0.7,
                    damage_multiplier: 0.5,
                    fire_rate_multiplier: 0.7,
                    speed_multiplier: 0.85,
                    accuracy_multiplier: 0.6,
                    spawn_rate_multiplier: 0.8,
                },
                boss: BossModifiers {
                    health_multiplier: 0.6,
                    damage_multiplier: 0.5,
                    attack_cooldown_multiplier: 1.3,
                },
                scoring: ScoringModifiers {
                    base_score_multiplier: 0.5,
                    combo_decay_multiplier: 0.7,
                },
            },
            DifficultyLevel::Newbro => Self {
                level,
                player: PlayerModifiers::default(),
                enemy: EnemyModifiers::default(),
                boss: BossModifiers::default(),
                scoring: ScoringModifiers::default(),
            },
            DifficultyLevel::BitterVet => Self {
                level,
                player: PlayerModifiers {
                    hull_multiplier: 0.8,
                    shield_multiplier: 0.8,
                    armor_multiplier: 0.8,
                    damage_multiplier: 0.9,
                    capacitor_recharge_multiplier: 0.8,
                    capacitor_drain_multiplier: 1.2,
                    maneuver_cooldown_multiplier: 1.2,
                    invincibility_duration_multiplier: 0.8,
                },
                enemy: EnemyModifiers {
                    health_multiplier: 1.3,
                    damage_multiplier: 1.5,
                    fire_rate_multiplier: 1.3,
                    speed_multiplier: 1.15,
                    accuracy_multiplier: 1.3,
                    spawn_rate_multiplier: 1.2,
                },
                boss: BossModifiers {
                    health_multiplier: 1.4,
                    damage_multiplier: 1.5,
                    attack_cooldown_multiplier: 0.8,
                },
                scoring: ScoringModifiers {
                    base_score_multiplier: 1.5,
                    combo_decay_multiplier: 1.3,
                },
            },
            DifficultyLevel::Triglavian => Self {
                level,
                player: PlayerModifiers {
                    hull_multiplier: 0.5,
                    shield_multiplier: 0.5,
                    armor_multiplier: 0.5,
                    damage_multiplier: 0.8,
                    capacitor_recharge_multiplier: 0.6,
                    capacitor_drain_multiplier: 1.5,
                    maneuver_cooldown_multiplier: 1.4,
                    invincibility_duration_multiplier: 0.5,
                },
                enemy: EnemyModifiers {
                    health_multiplier: 1.5,
                    damage_multiplier: 3.0,
                    fire_rate_multiplier: 1.5,
                    speed_multiplier: 1.3,
                    accuracy_multiplier: 1.5,
                    spawn_rate_multiplier: 1.5,
                },
                boss: BossModifiers {
                    health_multiplier: 2.0,
                    damage_multiplier: 2.5,
                    attack_cooldown_multiplier: 0.6,
                },
                scoring: ScoringModifiers {
                    base_score_multiplier: 3.0,
                    combo_decay_multiplier: 2.0,
                },
            },
        }
    }

    /// Set difficulty level and update all modifiers
    pub fn set_level(&mut self, level: DifficultyLevel) {
        *self = Self::from_level(level);
    }
}
