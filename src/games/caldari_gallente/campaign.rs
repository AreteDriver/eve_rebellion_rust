//! Caldari/Gallente Campaign Missions
//!
//! Battle of Caldari Prime mission chain.

#![allow(dead_code)]

use bevy::prelude::*;

/// Mission definition for Caldari/Gallente campaign
#[derive(Debug, Clone)]
pub struct CGMission {
    pub id: &'static str,
    pub name: &'static str,
    pub description: &'static str,
    pub primary_objective: &'static str,
    pub bonus_objective: Option<&'static str>,
    pub waves: u32,
    pub boss: Option<CGBossType>,
    pub is_tutorial: bool,
    pub unlocks_t3: bool,
}

/// Boss types for Caldari/Gallente campaign
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum CGBossType {
    PatrolCommander,
    FleetCommander,
    EliteSquadron,
    FleetAdmiral,
}

impl CGBossType {
    pub fn name(&self) -> &'static str {
        match self {
            CGBossType::PatrolCommander => "PATROL COMMANDER",
            CGBossType::FleetCommander => "FLEET COMMANDER",
            CGBossType::EliteSquadron => "ELITE SQUADRON",
            CGBossType::FleetAdmiral => "FLEET ADMIRAL",
        }
    }

    pub fn health(&self) -> f32 {
        match self {
            CGBossType::PatrolCommander => 400.0,
            CGBossType::FleetCommander => 700.0,
            CGBossType::EliteSquadron => 1000.0,
            CGBossType::FleetAdmiral => 1500.0,
        }
    }

    pub fn phases(&self) -> u32 {
        match self {
            CGBossType::PatrolCommander => 2,
            CGBossType::FleetCommander => 3,
            CGBossType::EliteSquadron => 3,
            CGBossType::FleetAdmiral => 4,
        }
    }
}

/// All missions in the Caldari/Gallente campaign
pub const CG_MISSIONS: [CGMission; 5] = [
    CGMission {
        id: "cg_m1_orbital_skirmish",
        name: "ORBITAL SKIRMISH",
        description: "Federation forces probe Caldari orbital defenses. First contact.",
        primary_objective: "Destroy enemy patrol ships",
        bonus_objective: Some("No damage taken"),
        waves: 3,
        boss: None,
        is_tutorial: true,
        unlocks_t3: false,
    },
    CGMission {
        id: "cg_m2_urban_firefight",
        name: "URBAN FIREFIGHT",
        description: "Combat above Caldari Prime's cities. The skyline burns.",
        primary_objective: "Clear the airspace",
        bonus_objective: Some("Protect civilian transports"),
        waves: 4,
        boss: Some(CGBossType::PatrolCommander),
        is_tutorial: false,
        unlocks_t3: false,
    },
    CGMission {
        id: "cg_m3_fleet_interdiction",
        name: "FLEET INTERDICTION",
        description: "Enemy reinforcements inbound. Intercept before they reach the front.",
        primary_objective: "Destroy the convoy",
        bonus_objective: Some("Destroy all escorts first"),
        waves: 5,
        boss: Some(CGBossType::FleetCommander),
        is_tutorial: false,
        unlocks_t3: false,
    },
    CGMission {
        id: "cg_m4_escalation",
        name: "ESCALATION POINT",
        description: "Both sides commit heavier assets. T3 destroyers enter the fray.",
        primary_objective: "Hold the line",
        bonus_objective: Some("Destroy elite squadron"),
        waves: 6,
        boss: Some(CGBossType::EliteSquadron),
        is_tutorial: false,
        unlocks_t3: true,
    },
    CGMission {
        id: "cg_m5_decisive_push",
        name: "DECISIVE PUSH",
        description: "The final battle for orbital superiority. No retreat.",
        primary_objective: "Achieve air dominance",
        bonus_objective: Some("Perfect victory"),
        waves: 8,
        boss: Some(CGBossType::FleetAdmiral),
        is_tutorial: false,
        unlocks_t3: false,
    },
];

/// Epilogue mission - Shiigeru Endless Nightmare
pub const CG_EPILOGUE_SHIIGERU: CGMission = CGMission {
    id: "cg_epilogue_shiigeru",
    name: "FINAL DIRECTIVE: SHIIGERU",
    description: "The Caldari titan Shiigeru falls. An endless nightmare aboard the dying vessel.",
    primary_objective: "Survive as long as possible",
    bonus_objective: None,
    waves: 0,   // Endless
    boss: None, // Multiple mini-bosses spawn over time
    is_tutorial: false,
    unlocks_t3: false,
};

// ============================================================================
// Shiigeru Nightmare Mode - Endless Survival
// ============================================================================

/// Mini-boss types that spawn during the Shiigeru nightmare
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum NightmareBoss {
    /// Security chief - fast, aggressive
    SecurityChief,
    /// Weapons officer - heavy firepower
    WeaponsOfficer,
    /// Drone swarm from the hangar bay
    DroneSwarm,
    /// Bridge commander - final spawns only
    BridgeCommander,
}

impl NightmareBoss {
    pub fn name(&self) -> &'static str {
        match self {
            NightmareBoss::SecurityChief => "SECURITY CHIEF",
            NightmareBoss::WeaponsOfficer => "WEAPONS OFFICER",
            NightmareBoss::DroneSwarm => "DRONE SWARM",
            NightmareBoss::BridgeCommander => "BRIDGE COMMANDER",
        }
    }

    pub fn health(&self) -> f32 {
        match self {
            NightmareBoss::SecurityChief => 300.0,
            NightmareBoss::WeaponsOfficer => 400.0,
            NightmareBoss::DroneSwarm => 200.0, // Multiple targets
            NightmareBoss::BridgeCommander => 600.0,
        }
    }

    pub fn dialogue(&self) -> &'static str {
        match self {
            NightmareBoss::SecurityChief => "All hands, repel boarders! This is not a drill!",
            NightmareBoss::WeaponsOfficer => "Weapons hot! Target that intruder!",
            NightmareBoss::DroneSwarm => "Launching automated defense drones!",
            NightmareBoss::BridgeCommander => "You won't take this ship while I breathe!",
        }
    }

    /// Get a boss type appropriate for the current wave
    pub fn for_wave(wave: u32) -> Self {
        match wave % 4 {
            0 => NightmareBoss::DroneSwarm,
            1 => NightmareBoss::SecurityChief,
            2 => NightmareBoss::WeaponsOfficer,
            _ => {
                if wave >= 20 {
                    NightmareBoss::BridgeCommander
                } else {
                    NightmareBoss::SecurityChief
                }
            }
        }
    }
}

/// Shiigeru Nightmare endless mode state
#[derive(Resource, Debug, Clone)]
pub struct ShiigeruNightmare {
    /// Is nightmare mode active
    pub active: bool,
    /// Current wave number
    pub wave: u32,
    /// Time survived in seconds
    pub time_survived: f32,
    /// Best time (for high score)
    pub best_time: f32,
    /// Best wave reached
    pub best_wave: u32,
    /// Total enemies killed this run
    pub kills: u32,
    /// Mini-bosses defeated this run
    pub mini_bosses_defeated: u32,
    /// Time until next wave spawns
    pub wave_timer: f32,
    /// Time between waves (decreases with escalation)
    pub wave_interval: f32,
    /// Escalation multiplier (enemy health/damage scale)
    pub escalation: f32,
    /// Time until next mini-boss
    pub boss_timer: f32,
    /// Current mini-boss (if spawned)
    pub current_boss: Option<NightmareBoss>,
    /// Hull integrity (0-100, visual effect only)
    pub hull_integrity: f32,
}

impl Default for ShiigeruNightmare {
    fn default() -> Self {
        Self {
            active: false,
            wave: 0,
            time_survived: 0.0,
            best_time: 0.0,
            best_wave: 0,
            kills: 0,
            mini_bosses_defeated: 0,
            wave_timer: 0.0,
            wave_interval: 8.0, // 8 seconds between waves initially
            escalation: 1.0,
            boss_timer: 30.0, // First mini-boss after 30 seconds
            current_boss: None,
            hull_integrity: 100.0,
        }
    }
}

impl ShiigeruNightmare {
    /// Start a new nightmare run
    pub fn start(&mut self) {
        self.active = true;
        self.wave = 0;
        self.time_survived = 0.0;
        self.kills = 0;
        self.mini_bosses_defeated = 0;
        self.wave_timer = 3.0; // First wave after 3 seconds
        self.wave_interval = 8.0;
        self.escalation = 1.0;
        self.boss_timer = 30.0;
        self.current_boss = None;
        self.hull_integrity = 100.0;
    }

    /// End the nightmare run (player died)
    pub fn end(&mut self) {
        self.active = false;

        // Update high scores
        if self.time_survived > self.best_time {
            self.best_time = self.time_survived;
        }
        if self.wave > self.best_wave {
            self.best_wave = self.wave;
        }
    }

    /// Update timers and escalation
    pub fn update(&mut self, dt: f32) -> NightmareEvent {
        if !self.active {
            return NightmareEvent::None;
        }

        self.time_survived += dt;
        self.wave_timer -= dt;
        self.boss_timer -= dt;

        // Hull integrity slowly decreases (visual tension)
        self.hull_integrity = (self.hull_integrity - dt * 0.5).max(10.0);

        // Escalation increases over time
        self.escalation = 1.0 + (self.time_survived / 60.0) * 0.25; // +25% per minute

        // Wave interval decreases with escalation (faster spawns)
        self.wave_interval = (8.0 - self.escalation * 0.5).max(3.0);

        // Check for mini-boss spawn
        if self.boss_timer <= 0.0 && self.current_boss.is_none() {
            self.boss_timer = 45.0 - (self.escalation * 5.0).min(20.0); // Faster boss spawns later
            let boss = NightmareBoss::for_wave(self.wave);
            self.current_boss = Some(boss);
            return NightmareEvent::SpawnBoss(boss);
        }

        // Check for wave spawn
        if self.wave_timer <= 0.0 {
            self.wave += 1;
            self.wave_timer = self.wave_interval;
            return NightmareEvent::SpawnWave(self.wave);
        }

        NightmareEvent::None
    }

    /// Called when an enemy is killed
    pub fn on_kill(&mut self) {
        self.kills += 1;
    }

    /// Called when a mini-boss is defeated
    pub fn on_boss_defeated(&mut self) {
        self.mini_bosses_defeated += 1;
        self.current_boss = None;
    }

    /// Get enemies per wave based on current escalation
    pub fn enemies_per_wave(&self) -> u32 {
        let base = 4 + (self.wave / 3);
        ((base as f32) * self.escalation.sqrt()) as u32
    }

    /// Get enemy health multiplier
    pub fn enemy_health_mult(&self) -> f32 {
        self.escalation
    }

    /// Get enemy damage multiplier
    pub fn enemy_damage_mult(&self) -> f32 {
        1.0 + (self.escalation - 1.0) * 0.5 // Damage scales slower than health
    }
}

/// Events that the nightmare mode can trigger
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum NightmareEvent {
    None,
    SpawnWave(u32),
    SpawnBoss(NightmareBoss),
}

/// Campaign state for Caldari/Gallente module
#[derive(Debug, Clone, Resource, Default)]
pub struct CGCampaignState {
    pub mission_index: usize,
    pub current_wave: u32,
    pub in_mission: bool,
    pub boss_spawned: bool,
    pub boss_defeated: bool,
    pub t3_unlocked: bool,
}

impl CGCampaignState {
    pub fn current_mission(&self) -> Option<&'static CGMission> {
        if self.mission_index < CG_MISSIONS.len() {
            Some(&CG_MISSIONS[self.mission_index])
        } else {
            None
        }
    }

    pub fn mission_number(&self) -> usize {
        self.mission_index + 1
    }

    pub fn total_missions() -> usize {
        CG_MISSIONS.len()
    }

    pub fn start_mission(&mut self) {
        self.in_mission = true;
        self.current_wave = 1;
        self.boss_spawned = false;
        self.boss_defeated = false;
    }

    pub fn complete_mission(&mut self) -> bool {
        self.in_mission = false;

        // Check for T3 unlock
        if let Some(mission) = self.current_mission() {
            if mission.unlocks_t3 {
                self.t3_unlocked = true;
            }
        }

        if self.mission_index + 1 < CG_MISSIONS.len() {
            self.mission_index += 1;
            true
        } else {
            false // Campaign complete
        }
    }

    pub fn is_boss_wave(&self) -> bool {
        if let Some(mission) = self.current_mission() {
            mission.boss.is_some() && self.current_wave > mission.waves
        } else {
            false
        }
    }
}

// ============================================================================
// Tests
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn nightmare_starts_correctly() {
        let mut nightmare = ShiigeruNightmare::default();
        assert!(!nightmare.active);

        nightmare.start();
        assert!(nightmare.active);
        assert_eq!(nightmare.wave, 0);
        assert_eq!(nightmare.kills, 0);
        assert_eq!(nightmare.escalation, 1.0);
    }

    #[test]
    fn nightmare_escalation_increases() {
        let mut nightmare = ShiigeruNightmare::default();
        nightmare.start();

        // Simulate 60 seconds
        for _ in 0..60 {
            nightmare.update(1.0);
        }

        // Should have escalated by ~25%
        assert!(nightmare.escalation > 1.2);
        assert!(nightmare.escalation < 1.3);
    }

    #[test]
    fn nightmare_spawns_waves() {
        let mut nightmare = ShiigeruNightmare::default();
        nightmare.start();

        // First wave spawns after 3 seconds
        let event = nightmare.update(3.5);
        assert_eq!(event, NightmareEvent::SpawnWave(1));
        assert_eq!(nightmare.wave, 1);
    }

    #[test]
    fn nightmare_spawns_boss() {
        let mut nightmare = ShiigeruNightmare::default();
        nightmare.start();

        // Boss spawns after 30 seconds
        nightmare.update(31.0);

        // Check that boss was spawned
        assert!(nightmare.current_boss.is_some());
    }

    #[test]
    fn nightmare_tracks_kills() {
        let mut nightmare = ShiigeruNightmare::default();
        nightmare.start();

        nightmare.on_kill();
        nightmare.on_kill();
        nightmare.on_kill();

        assert_eq!(nightmare.kills, 3);
    }

    #[test]
    fn nightmare_tracks_boss_defeats() {
        let mut nightmare = ShiigeruNightmare::default();
        nightmare.start();
        nightmare.current_boss = Some(NightmareBoss::SecurityChief);

        nightmare.on_boss_defeated();

        assert_eq!(nightmare.mini_bosses_defeated, 1);
        assert!(nightmare.current_boss.is_none());
    }

    #[test]
    fn nightmare_updates_high_scores() {
        let mut nightmare = ShiigeruNightmare::default();
        nightmare.start();

        // Simulate a run
        nightmare.wave = 15;
        nightmare.time_survived = 120.0;

        nightmare.end();

        assert_eq!(nightmare.best_wave, 15);
        assert_eq!(nightmare.best_time, 120.0);

        // Start another run
        nightmare.start();
        nightmare.wave = 10;
        nightmare.time_survived = 60.0;
        nightmare.end();

        // Best scores should remain
        assert_eq!(nightmare.best_wave, 15);
        assert_eq!(nightmare.best_time, 120.0);
    }

    #[test]
    fn nightmare_boss_for_wave() {
        assert_eq!(NightmareBoss::for_wave(1), NightmareBoss::SecurityChief);
        assert_eq!(NightmareBoss::for_wave(2), NightmareBoss::WeaponsOfficer);
        assert_eq!(NightmareBoss::for_wave(4), NightmareBoss::DroneSwarm);
        assert_eq!(NightmareBoss::for_wave(23), NightmareBoss::BridgeCommander);
    }

    #[test]
    fn nightmare_enemies_per_wave_scales() {
        let mut nightmare = ShiigeruNightmare::default();
        nightmare.start();

        let wave1_enemies = nightmare.enemies_per_wave();
        nightmare.wave = 10;
        nightmare.escalation = 1.5;
        let wave10_enemies = nightmare.enemies_per_wave();

        assert!(wave10_enemies > wave1_enemies);
    }
}
