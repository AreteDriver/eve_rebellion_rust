//! Campaign System
//!
//! Handles mission progression, objectives, and storyline.

#![allow(dead_code)]

use bevy::prelude::*;

/// Campaign acts - progression through the story
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum Act {
    #[default]
    Act1, // "The Call" - Rifter, proving yourself
    Act2, // "The Storm" - Wolf, full assault
    Act3, // "Liberation" - Jaguar, final push
}

impl Act {
    pub fn name(&self) -> &'static str {
        match self {
            Act::Act1 => "THE CALL",
            Act::Act2 => "THE STORM",
            Act::Act3 => "LIBERATION",
        }
    }

    pub fn catchphrase(&self) -> &'static str {
        match self {
            Act::Act1 => "In Rust We Trust!",
            Act::Act2 => "No more rust - just steel!",
            Act::Act3 => "For Freedom! For the Republic!",
        }
    }

    pub fn description(&self) -> &'static str {
        match self {
            Act::Act1 => {
                "Prove yourself worthy in a rust-bucket Rifter. Early raids on slave convoys."
            }
            Act::Act2 => "The invasion begins in earnest. You've earned an assault frigate.",
            Act::Act3 => "The final push. Strike at the heart of the Empire.",
        }
    }

    pub fn number(&self) -> u32 {
        match self {
            Act::Act1 => 1,
            Act::Act2 => 2,
            Act::Act3 => 3,
        }
    }

    pub fn next(&self) -> Option<Act> {
        match self {
            Act::Act1 => Some(Act::Act2),
            Act::Act2 => Some(Act::Act3),
            Act::Act3 => None,
        }
    }

    pub fn missions(&self) -> &'static [Mission] {
        match self {
            Act::Act1 => &ACT1_MISSIONS,
            Act::Act2 => &ACT2_MISSIONS,
            Act::Act3 => &ACT3_MISSIONS,
        }
    }
}

/// Mission definition
#[derive(Debug, Clone)]
pub struct Mission {
    pub id: &'static str,
    pub name: &'static str,
    pub description: &'static str,
    pub primary_objective: &'static str,
    pub bonus_objective: Option<&'static str>,
    pub boss: BossType,
    pub enemy_waves: u32,
    pub souls_to_liberate: u32,
}

/// Boss types for each mission
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum BossType {
    #[default]
    None,
    // Act 1 bosses
    TransportOverseer, // M1: Slave transport captain
    PatrolCommander,   // M2: Amarr patrol lead
    StationBattery,    // M3: Defense turret array
    HolderEscort,      // M4: Holder's escort captain
    // Act 2 bosses
    CustomsCommandant, // M5: Customs officer
    InquisitorVessel,  // M6: Inquisition ship
    HarbingerStrike,   // M7: Navy Harbinger
    StargateDefense,   // M8: Gate defense grid
    BattlestationCore, // M9: Battlestation
    // Act 3 bosses
    AbaddonBattleship, // M10: Abaddon-class
    TitanEscort,       // M11: Avatar escort fleet
    EmpressChampion,   // M12: Royal champion
    AvatarTitan,       // M13: Final boss - Avatar titan
}

impl BossType {
    pub fn name(&self) -> &'static str {
        match self {
            BossType::None => "None",
            BossType::TransportOverseer => "TRANSPORT OVERSEER",
            BossType::PatrolCommander => "PATROL COMMANDER",
            BossType::StationBattery => "STATION DEFENSE GRID",
            BossType::HolderEscort => "HOLDER'S CHAMPION",
            BossType::CustomsCommandant => "CUSTOMS COMMANDANT",
            BossType::InquisitorVessel => "INQUISITOR VESSEL",
            BossType::HarbingerStrike => "HARBINGER STRIKE LEAD",
            BossType::StargateDefense => "STARGATE DEFENSE GRID",
            BossType::BattlestationCore => "PURITY'S LIGHT",
            BossType::AbaddonBattleship => "ABADDON BATTLESHIP",
            BossType::TitanEscort => "TITAN ESCORT FLEET",
            BossType::EmpressChampion => "EMPRESS'S CHAMPION",
            BossType::AvatarTitan => "AVATAR TITAN",
        }
    }

    pub fn health(&self) -> f32 {
        match self {
            BossType::None => 0.0,
            // Act 1 - easier bosses
            BossType::TransportOverseer => 500.0,
            BossType::PatrolCommander => 600.0,
            BossType::StationBattery => 800.0,
            BossType::HolderEscort => 700.0,
            // Act 2 - medium bosses
            BossType::CustomsCommandant => 900.0,
            BossType::InquisitorVessel => 1000.0,
            BossType::HarbingerStrike => 1200.0,
            BossType::StargateDefense => 1500.0,
            BossType::BattlestationCore => 2000.0,
            // Act 3 - hard bosses
            BossType::AbaddonBattleship => 2500.0,
            BossType::TitanEscort => 3000.0,
            BossType::EmpressChampion => 3500.0,
            BossType::AvatarTitan => 5000.0,
        }
    }

    pub fn phases(&self) -> u32 {
        match self {
            BossType::None => 0,
            BossType::TransportOverseer | BossType::PatrolCommander => 2,
            BossType::StationBattery | BossType::HolderEscort => 2,
            BossType::CustomsCommandant | BossType::InquisitorVessel => 3,
            BossType::HarbingerStrike | BossType::StargateDefense => 3,
            BossType::BattlestationCore => 3,
            BossType::AbaddonBattleship | BossType::TitanEscort => 4,
            BossType::EmpressChampion => 4,
            BossType::AvatarTitan => 5,
        }
    }

    pub fn eve_type_id(&self) -> u32 {
        match self {
            BossType::None => 0,
            BossType::TransportOverseer => 20185, // Bestower
            BossType::PatrolCommander => 597,     // Punisher
            BossType::StationBattery => 0,        // Custom structure
            BossType::HolderEscort => 589,        // Executioner
            BossType::CustomsCommandant => 2006,  // Apocalypse
            BossType::InquisitorVessel => 24690,  // Absolution
            BossType::HarbingerStrike => 24690,   // Harbinger
            BossType::StargateDefense => 0,       // Custom structure
            BossType::BattlestationCore => 0,     // Custom structure
            BossType::AbaddonBattleship => 24692, // Abaddon
            BossType::TitanEscort => 24690,       // Fleet mix
            BossType::EmpressChampion => 17726,   // Apocalypse Navy
            BossType::AvatarTitan => 11567,       // Avatar
        }
    }
}

// Act 1 Missions - "The Call"
const ACT1_MISSIONS: [Mission; 4] = [
    Mission {
        id: "m1_convoy_raid",
        name: "FIRST BLOOD",
        description: "Intercept a slave transport in the Arzad corridor.",
        primary_objective: "Destroy the slave transport",
        bonus_objective: Some("Liberate 10+ slaves"),
        boss: BossType::TransportOverseer,
        enemy_waves: 3,
        souls_to_liberate: 10,
    },
    Mission {
        id: "m2_patrol_ambush",
        name: "HUNTER HUNTED",
        description: "Amarr patrols hunt our scouts. Turn the tables.",
        primary_objective: "Destroy all patrol ships",
        bonus_objective: Some("No damage taken"),
        boss: BossType::PatrolCommander,
        enemy_waves: 4,
        souls_to_liberate: 5,
    },
    Mission {
        id: "m3_station_raid",
        name: "STATION RAID",
        description: "Disable orbital station defenses for extraction teams.",
        primary_objective: "Destroy defense turrets",
        bonus_objective: Some("Liberate 30+ slaves"),
        boss: BossType::StationBattery,
        enemy_waves: 5,
        souls_to_liberate: 30,
    },
    Mission {
        id: "m4_holder_escape",
        name: "THE HOLDER'S FLIGHT",
        description: "A slave lord flees with his 'property.' End his escape.",
        primary_objective: "Destroy the Holder's escort",
        bonus_objective: Some("Complete in under 3 minutes"),
        boss: BossType::HolderEscort,
        enemy_waves: 4,
        souls_to_liberate: 20,
    },
];

// Act 2 Missions - "The Storm"
const ACT2_MISSIONS: [Mission; 5] = [
    Mission {
        id: "m5_customs_strike",
        name: "CUSTOMS CLEARANCE",
        description: "Imperial Customs bleeds our supply lines. Remove them.",
        primary_objective: "Destroy the Customs station",
        bonus_objective: Some("Destroy all cargo pods"),
        boss: BossType::CustomsCommandant,
        enemy_waves: 5,
        souls_to_liberate: 15,
    },
    Mission {
        id: "m6_inquisition",
        name: "DIVINE JUDGMENT",
        description: "The Inquisition sends a vessel to 'cleanse' liberated systems.",
        primary_objective: "Destroy the Inquisitor vessel",
        bonus_objective: Some("No allied losses"),
        boss: BossType::InquisitorVessel,
        enemy_waves: 6,
        souls_to_liberate: 25,
    },
    Mission {
        id: "m7_navy_battle",
        name: "BREAKING THE LINE",
        description: "A Navy Harbinger strike group threatens our liberation fleet.",
        primary_objective: "Destroy the strike lead",
        bonus_objective: Some("Destroy all escorts first"),
        boss: BossType::HarbingerStrike,
        enemy_waves: 6,
        souls_to_liberate: 20,
    },
    Mission {
        id: "m8_stargate",
        name: "GATE CRASHERS",
        description: "The stargate to Arzad Prime is heavily fortified.",
        primary_objective: "Disable the gate defenses",
        bonus_objective: Some("Under 4 minutes"),
        boss: BossType::StargateDefense,
        enemy_waves: 7,
        souls_to_liberate: 30,
    },
    Mission {
        id: "m9_battlestation",
        name: "PURITY'S LIGHT",
        description: "An Amarr battlestation guards the slave processing hub.",
        primary_objective: "Destroy the battlestation core",
        bonus_objective: Some("Liberate 50+ slaves"),
        boss: BossType::BattlestationCore,
        enemy_waves: 8,
        souls_to_liberate: 50,
    },
];

// Act 3 Missions - "Liberation"
const ACT3_MISSIONS: [Mission; 4] = [
    Mission {
        id: "m10_abaddon",
        name: "GOLDEN FLEET",
        description: "The Amarr Navy deploys Abaddon battleships to stop our advance.",
        primary_objective: "Destroy the Abaddon flagship",
        bonus_objective: Some("No damage taken in phase 1"),
        boss: BossType::AbaddonBattleship,
        enemy_waves: 8,
        souls_to_liberate: 40,
    },
    Mission {
        id: "m11_titan_escort",
        name: "TITAN'S SHADOW",
        description: "The Avatar titan's escort fleet blocks the approach.",
        primary_objective: "Clear the escort fleet",
        bonus_objective: Some("Destroy all in one chain"),
        boss: BossType::TitanEscort,
        enemy_waves: 9,
        souls_to_liberate: 50,
    },
    Mission {
        id: "m12_champion",
        name: "IMPERIAL CHAMPION",
        description: "The Empress's personal champion challenges you.",
        primary_objective: "Defeat the champion",
        bonus_objective: Some("Perfect no-damage victory"),
        boss: BossType::EmpressChampion,
        enemy_waves: 7,
        souls_to_liberate: 30,
    },
    Mission {
        id: "m13_avatar",
        name: "AVATAR",
        description: "The Avatar titan. The symbol of Amarr oppression. End it.",
        primary_objective: "Destroy the Avatar",
        bonus_objective: Some("Complete the liberation"),
        boss: BossType::AvatarTitan,
        enemy_waves: 10,
        souls_to_liberate: 100,
    },
];

/// Current campaign state
#[derive(Debug, Clone, Resource)]
pub struct CampaignState {
    /// Current act
    pub act: Act,
    /// Current mission index within the act (0-indexed)
    pub mission_index: usize,
    /// Whether we're in a mission
    pub in_mission: bool,
    /// Current wave within the mission
    pub current_wave: u32,
    /// Enemies remaining in current wave
    pub enemies_remaining: u32,
    /// Boss spawned flag
    pub boss_spawned: bool,
    /// Boss defeated flag
    pub boss_defeated: bool,
    /// Mission timer
    pub mission_timer: f32,
    /// Souls liberated this mission
    pub mission_souls: u32,
    /// No damage taken this mission
    pub no_damage_taken: bool,
    /// Primary objective complete
    pub primary_complete: bool,
    /// Bonus objective complete
    pub bonus_complete: bool,
}

impl Default for CampaignState {
    fn default() -> Self {
        Self {
            act: Act::Act1,
            mission_index: 0,
            in_mission: false,
            current_wave: 0,
            enemies_remaining: 0,
            boss_spawned: false,
            boss_defeated: false,
            mission_timer: 0.0,
            mission_souls: 0,
            no_damage_taken: true,
            primary_complete: false,
            bonus_complete: false,
        }
    }
}

impl CampaignState {
    /// Get current mission
    pub fn current_mission(&self) -> Option<&'static Mission> {
        let missions = self.act.missions();
        missions.get(self.mission_index)
    }

    /// Start the current mission
    pub fn start_mission(&mut self) {
        self.in_mission = true;
        self.current_wave = 1;
        self.enemies_remaining = 0;
        self.boss_spawned = false;
        self.boss_defeated = false;
        self.mission_timer = 0.0;
        self.mission_souls = 0;
        self.no_damage_taken = true;
        self.primary_complete = false;
        self.bonus_complete = false;
    }

    /// Complete current mission and advance
    pub fn complete_mission(&mut self) -> bool {
        self.in_mission = false;
        self.primary_complete = true;

        let missions = self.act.missions();
        if self.mission_index + 1 < missions.len() {
            self.mission_index += 1;
            true
        } else {
            // Act complete, try to advance to next act
            if let Some(next_act) = self.act.next() {
                self.act = next_act;
                self.mission_index = 0;
                true
            } else {
                // Campaign complete!
                false
            }
        }
    }

    /// Check if current wave is the boss wave
    pub fn is_boss_wave(&self) -> bool {
        if let Some(mission) = self.current_mission() {
            self.current_wave > mission.enemy_waves
        } else {
            false
        }
    }

    /// Advance to next wave
    pub fn next_wave(&mut self) -> bool {
        if let Some(mission) = self.current_mission() {
            if self.current_wave <= mission.enemy_waves {
                self.current_wave += 1;
                true
            } else {
                false // Already at boss
            }
        } else {
            false
        }
    }

    /// Get total missions in campaign
    pub fn total_missions() -> usize {
        ACT1_MISSIONS.len() + ACT2_MISSIONS.len() + ACT3_MISSIONS.len()
    }

    /// Get current mission number (1-indexed, across all acts)
    pub fn mission_number(&self) -> usize {
        match self.act {
            Act::Act1 => self.mission_index + 1,
            Act::Act2 => ACT1_MISSIONS.len() + self.mission_index + 1,
            Act::Act3 => ACT1_MISSIONS.len() + ACT2_MISSIONS.len() + self.mission_index + 1,
        }
    }

    /// Get current mission name
    pub fn current_mission_name(&self) -> &'static str {
        self.current_mission()
            .map(|m| m.name)
            .unwrap_or("Unknown Mission")
    }

    /// Get current stage number (act number)
    pub fn stage_number(&self) -> u32 {
        match self.act {
            Act::Act1 => 1,
            Act::Act2 => 2,
            Act::Act3 => 3,
        }
    }

    /// Get current mission number within current stage (1-indexed)
    pub fn mission_in_stage(&self) -> u32 {
        (self.mission_index + 1) as u32
    }
}

/// Mission events
#[derive(Event)]
pub struct MissionStartEvent {
    pub mission: &'static Mission,
}

#[derive(Event)]
pub struct MissionCompleteEvent {
    pub mission_id: &'static str,
    pub bonus_achieved: bool,
    pub souls_liberated: u32,
    pub time_taken: f32,
}

#[derive(Event)]
pub struct WaveCompleteEvent {
    pub wave_number: u32,
}

#[derive(Event)]
pub struct BossSpawnEvent {
    pub boss_type: BossType,
}

// Note: Use core::events::BossDefeatedEvent for boss defeat events

#[derive(Event)]
pub struct ActCompleteEvent {
    pub act: Act,
}

#[cfg(test)]
mod tests {
    use super::*;

    // ==================== Act Tests ====================

    #[test]
    fn act_default_is_act1() {
        assert_eq!(Act::default(), Act::Act1);
    }

    #[test]
    fn act_names() {
        assert_eq!(Act::Act1.name(), "THE CALL");
        assert_eq!(Act::Act2.name(), "THE STORM");
        assert_eq!(Act::Act3.name(), "LIBERATION");
    }

    #[test]
    fn act_catchphrases() {
        assert_eq!(Act::Act1.catchphrase(), "In Rust We Trust!");
        assert_eq!(Act::Act2.catchphrase(), "No more rust - just steel!");
        assert_eq!(Act::Act3.catchphrase(), "For Freedom! For the Republic!");
    }

    #[test]
    fn act_numbers() {
        assert_eq!(Act::Act1.number(), 1);
        assert_eq!(Act::Act2.number(), 2);
        assert_eq!(Act::Act3.number(), 3);
    }

    #[test]
    fn act_progression() {
        assert_eq!(Act::Act1.next(), Some(Act::Act2));
        assert_eq!(Act::Act2.next(), Some(Act::Act3));
        assert_eq!(Act::Act3.next(), None); // Final act
    }

    #[test]
    fn act_missions_not_empty() {
        assert!(!Act::Act1.missions().is_empty());
        assert!(!Act::Act2.missions().is_empty());
        assert!(!Act::Act3.missions().is_empty());
    }

    #[test]
    fn act_mission_counts() {
        assert_eq!(Act::Act1.missions().len(), 4);
        assert_eq!(Act::Act2.missions().len(), 5);
        assert_eq!(Act::Act3.missions().len(), 4);
    }

    // ==================== BossType Tests ====================

    #[test]
    fn boss_type_default_is_none() {
        assert_eq!(BossType::default(), BossType::None);
    }

    #[test]
    fn boss_type_none_has_zero_health() {
        assert_eq!(BossType::None.health(), 0.0);
        assert_eq!(BossType::None.phases(), 0);
    }

    #[test]
    fn boss_health_increases_per_act() {
        // Act 1 bosses
        let act1_health = BossType::TransportOverseer.health();
        // Act 2 bosses
        let act2_health = BossType::BattlestationCore.health();
        // Act 3 bosses
        let act3_health = BossType::AvatarTitan.health();

        assert!(act1_health < act2_health);
        assert!(act2_health < act3_health);
    }

    #[test]
    fn boss_phases_increase_per_act() {
        // Act 1 bosses have 2 phases
        assert_eq!(BossType::TransportOverseer.phases(), 2);
        assert_eq!(BossType::PatrolCommander.phases(), 2);

        // Act 2 bosses have 3 phases
        assert_eq!(BossType::CustomsCommandant.phases(), 3);
        assert_eq!(BossType::BattlestationCore.phases(), 3);

        // Act 3 bosses have 4-5 phases
        assert_eq!(BossType::AbaddonBattleship.phases(), 4);
        assert_eq!(BossType::AvatarTitan.phases(), 5);
    }

    #[test]
    fn avatar_titan_is_final_boss() {
        let avatar = BossType::AvatarTitan;
        assert_eq!(avatar.health(), 5000.0);
        assert_eq!(avatar.phases(), 5);
        assert_eq!(avatar.eve_type_id(), 11567); // Avatar titan type_id
    }

    #[test]
    fn boss_names_not_empty() {
        let bosses = [
            BossType::TransportOverseer,
            BossType::PatrolCommander,
            BossType::CustomsCommandant,
            BossType::AvatarTitan,
        ];

        for boss in bosses {
            assert!(!boss.name().is_empty());
        }
    }

    // ==================== Mission Tests ====================

    #[test]
    fn all_missions_have_ids() {
        for act in [Act::Act1, Act::Act2, Act::Act3] {
            for mission in act.missions() {
                assert!(!mission.id.is_empty(), "Mission should have an ID");
            }
        }
    }

    #[test]
    fn all_missions_have_bosses() {
        for act in [Act::Act1, Act::Act2, Act::Act3] {
            for mission in act.missions() {
                assert_ne!(
                    mission.boss,
                    BossType::None,
                    "Mission {} should have a boss",
                    mission.name
                );
            }
        }
    }

    #[test]
    fn all_missions_have_waves() {
        for act in [Act::Act1, Act::Act2, Act::Act3] {
            for mission in act.missions() {
                assert!(
                    mission.enemy_waves > 0,
                    "Mission {} should have waves",
                    mission.name
                );
            }
        }
    }

    #[test]
    fn first_mission_is_first_blood() {
        let mission = &Act::Act1.missions()[0];
        assert_eq!(mission.name, "FIRST BLOOD");
        assert_eq!(mission.boss, BossType::TransportOverseer);
    }

    #[test]
    fn final_mission_is_avatar() {
        let missions = Act::Act3.missions();
        let final_mission = &missions[missions.len() - 1];
        assert_eq!(final_mission.name, "AVATAR");
        assert_eq!(final_mission.boss, BossType::AvatarTitan);
        assert_eq!(final_mission.souls_to_liberate, 100);
    }

    #[test]
    fn total_missions_is_13() {
        assert_eq!(CampaignState::total_missions(), 13);
    }

    // ==================== CampaignState Tests ====================

    #[test]
    fn campaign_state_default() {
        let state = CampaignState::default();
        assert_eq!(state.act, Act::Act1);
        assert_eq!(state.mission_index, 0);
        assert!(!state.in_mission);
        assert!(state.no_damage_taken);
    }

    #[test]
    fn campaign_state_current_mission() {
        let state = CampaignState::default();
        let mission = state.current_mission().unwrap();
        assert_eq!(mission.name, "FIRST BLOOD");
    }

    #[test]
    fn campaign_state_start_mission() {
        let mut state = CampaignState::default();
        state.start_mission();

        assert!(state.in_mission);
        assert_eq!(state.current_wave, 1);
        assert!(!state.boss_spawned);
        assert!(!state.boss_defeated);
        assert!(state.no_damage_taken);
    }

    #[test]
    fn campaign_state_next_wave() {
        let mut state = CampaignState::default();
        state.start_mission();
        assert_eq!(state.current_wave, 1);

        assert!(state.next_wave());
        assert_eq!(state.current_wave, 2);
    }

    #[test]
    fn campaign_state_is_boss_wave() {
        let mut state = CampaignState::default();
        state.start_mission();

        // First mission has 3 waves, so wave 4 is boss
        assert!(!state.is_boss_wave());

        state.current_wave = 3;
        assert!(!state.is_boss_wave());

        state.current_wave = 4;
        assert!(state.is_boss_wave());
    }

    #[test]
    fn campaign_state_complete_mission_advances() {
        let mut state = CampaignState::default();
        state.start_mission();

        assert!(state.complete_mission());
        assert_eq!(state.mission_index, 1);
        assert!(!state.in_mission);
    }

    #[test]
    fn campaign_state_complete_act_advances() {
        let mut state = CampaignState {
            act: Act::Act1,
            mission_index: 3, // Last mission in Act 1
            ..Default::default()
        };

        assert!(state.complete_mission());
        assert_eq!(state.act, Act::Act2);
        assert_eq!(state.mission_index, 0);
    }

    #[test]
    fn campaign_state_complete_final_returns_false() {
        let mut state = CampaignState {
            act: Act::Act3,
            mission_index: 3, // Last mission in Act 3
            ..Default::default()
        };

        assert!(!state.complete_mission()); // Campaign complete
    }

    #[test]
    fn campaign_state_mission_number() {
        let mut state = CampaignState::default();

        // Act 1, Mission 1
        assert_eq!(state.mission_number(), 1);

        // Act 1, Mission 4
        state.mission_index = 3;
        assert_eq!(state.mission_number(), 4);

        // Act 2, Mission 1
        state.act = Act::Act2;
        state.mission_index = 0;
        assert_eq!(state.mission_number(), 5);

        // Act 3, Mission 4 (final)
        state.act = Act::Act3;
        state.mission_index = 3;
        assert_eq!(state.mission_number(), 13);
    }

    #[test]
    fn campaign_state_stage_number() {
        let mut state = CampaignState::default();
        assert_eq!(state.stage_number(), 1);

        state.act = Act::Act2;
        assert_eq!(state.stage_number(), 2);

        state.act = Act::Act3;
        assert_eq!(state.stage_number(), 3);
    }

    #[test]
    fn campaign_state_mission_in_stage() {
        let mut state = CampaignState::default();
        assert_eq!(state.mission_in_stage(), 1);

        state.mission_index = 2;
        assert_eq!(state.mission_in_stage(), 3);
    }

    #[test]
    fn campaign_state_current_mission_name() {
        let state = CampaignState::default();
        assert_eq!(state.current_mission_name(), "FIRST BLOOD");
    }

    #[test]
    fn campaign_state_invalid_mission_index() {
        let state = CampaignState {
            mission_index: 999,
            ..Default::default()
        };

        // Should return None for invalid index
        assert!(state.current_mission().is_none());
        assert_eq!(state.current_mission_name(), "Unknown Mission");
    }

    // ==================== Wave Progression Tests ====================

    #[test]
    fn wave_progression_stops_at_boss() {
        let mut state = CampaignState::default();
        state.start_mission();

        // First mission has 3 waves
        assert!(state.next_wave()); // Wave 2
        assert!(state.next_wave()); // Wave 3
        assert!(state.next_wave()); // Wave 4 (boss)
        assert!(!state.next_wave()); // Can't go past boss
    }

    // ==================== Mission Data Integrity ====================

    #[test]
    fn mission_ids_are_unique() {
        let mut ids = std::collections::HashSet::new();
        for act in [Act::Act1, Act::Act2, Act::Act3] {
            for mission in act.missions() {
                assert!(
                    ids.insert(mission.id),
                    "Duplicate mission ID: {}",
                    mission.id
                );
            }
        }
    }

    #[test]
    fn mission_waves_increase_through_campaign() {
        let m1_waves = Act::Act1.missions()[0].enemy_waves;
        let m9_waves = Act::Act2.missions()[4].enemy_waves; // Battlestation
        let m13_waves = Act::Act3.missions()[3].enemy_waves; // Avatar

        assert!(m1_waves < m9_waves);
        assert!(m9_waves <= m13_waves);
    }

    #[test]
    fn all_missions_have_primary_objectives() {
        for act in [Act::Act1, Act::Act2, Act::Act3] {
            for mission in act.missions() {
                assert!(
                    !mission.primary_objective.is_empty(),
                    "Mission {} needs primary objective",
                    mission.name
                );
            }
        }
    }
}
