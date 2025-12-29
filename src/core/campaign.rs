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
