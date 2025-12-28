//! Caldari/Gallente Campaign Missions
//!
//! Battle of Caldari Prime mission chain.

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

/// Epilogue mission stub (Caldari arc only)
/// TODO: Implement endless nightmare endurance mission
pub const CG_EPILOGUE_SHIIGERU: CGMission = CGMission {
    id: "cg_epilogue_shiigeru",
    name: "FINAL DIRECTIVE: SHIIGERU",
    description: "The Caldari titan Shiigeru falls. An endless nightmare aboard the dying vessel.",
    primary_objective: "Survive as long as possible",
    bonus_objective: None,
    waves: 0, // Endless
    boss: None, // Multiple mini-bosses spawn over time
    is_tutorial: false,
    unlocks_t3: false,
};

/// Campaign state for Caldari/Gallente module
#[derive(Debug, Clone, Resource)]
pub struct CGCampaignState {
    pub mission_index: usize,
    pub current_wave: u32,
    pub in_mission: bool,
    pub boss_spawned: bool,
    pub boss_defeated: bool,
    pub t3_unlocked: bool,
}

impl Default for CGCampaignState {
    fn default() -> Self {
        Self {
            mission_index: 0,
            current_wave: 0,
            in_mission: false,
            boss_spawned: false,
            boss_defeated: false,
            t3_unlocked: false,
        }
    }
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
