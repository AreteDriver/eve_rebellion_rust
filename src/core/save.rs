//! Save/Load System
//!
//! Persists player progression, unlocks, and settings.

#![allow(dead_code)]

use bevy::prelude::*;
use serde::{Deserialize, Serialize};
use std::collections::HashSet;
use std::fs;
use std::path::PathBuf;

/// Save system plugin
pub struct SavePlugin;

impl Plugin for SavePlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<SaveData>()
            .add_systems(Startup, load_save_data)
            .add_systems(
                Update,
                auto_save.run_if(resource_changed::<SaveData>),
            );
    }
}

/// Persistent save data
#[derive(Resource, Serialize, Deserialize, Clone, Debug, Default)]
pub struct SaveData {
    /// Highest stage completed per faction pair
    pub stage_progress: Vec<FactionProgress>,
    /// Ships unlocked (by type_id)
    pub unlocked_ships: HashSet<u32>,
    /// Total credits earned (lifetime)
    pub lifetime_credits: u64,
    /// High scores per faction pair
    pub high_scores: Vec<HighScore>,
    /// Settings
    pub settings: GameSettings,
}

#[derive(Serialize, Deserialize, Clone, Debug, Default)]
pub struct FactionProgress {
    pub player_faction: String,
    pub enemy_faction: String,
    pub highest_stage: u32,
    pub highest_mission: u32,
}

#[derive(Serialize, Deserialize, Clone, Debug, Default)]
pub struct HighScore {
    pub player_faction: String,
    pub enemy_faction: String,
    pub score: u64,
    pub stage: u32,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct GameSettings {
    pub master_volume: f32,
    pub sfx_volume: f32,
    pub music_volume: f32,
    pub screen_shake: bool,
}

impl Default for GameSettings {
    fn default() -> Self {
        Self {
            master_volume: 0.7,
            sfx_volume: 0.8,
            music_volume: 0.5,
            screen_shake: true,
        }
    }
}

impl SaveData {
    /// Get save file path
    fn save_path() -> PathBuf {
        dirs::data_dir()
            .unwrap_or_else(|| PathBuf::from("."))
            .join("eve_rebellion")
            .join("save.json")
    }

    /// Load from disk
    pub fn load() -> Self {
        let path = Self::save_path();
        if path.exists() {
            match fs::read_to_string(&path) {
                Ok(data) => match serde_json::from_str(&data) {
                    Ok(save) => {
                        info!("Loaded save data from {:?}", path);
                        return save;
                    }
                    Err(e) => warn!("Failed to parse save data: {}", e),
                },
                Err(e) => warn!("Failed to read save file: {}", e),
            }
        }
        info!("No save data found, using defaults");
        Self::default()
    }

    /// Save to disk
    pub fn save(&self) {
        let path = Self::save_path();

        // Create directory if needed
        if let Some(parent) = path.parent() {
            if let Err(e) = fs::create_dir_all(parent) {
                warn!("Failed to create save directory: {}", e);
                return;
            }
        }

        match serde_json::to_string_pretty(self) {
            Ok(data) => {
                if let Err(e) = fs::write(&path, data) {
                    warn!("Failed to write save file: {}", e);
                } else {
                    info!("Saved progress to {:?}", path);
                }
            }
            Err(e) => warn!("Failed to serialize save data: {}", e),
        }
    }

    /// Check if a ship is unlocked
    pub fn is_ship_unlocked(&self, type_id: u32, unlock_stage: u32, faction: &str, enemy: &str) -> bool {
        // Stage 0 = always unlocked
        if unlock_stage == 0 {
            return true;
        }

        // Check if explicitly unlocked
        if self.unlocked_ships.contains(&type_id) {
            return true;
        }

        // Check faction progress
        for progress in &self.stage_progress {
            if progress.player_faction == faction && progress.enemy_faction == enemy {
                return progress.highest_stage >= unlock_stage;
            }
        }

        false
    }

    /// Unlock a ship
    pub fn unlock_ship(&mut self, type_id: u32) {
        self.unlocked_ships.insert(type_id);
    }

    /// Record stage completion
    pub fn complete_stage(&mut self, faction: &str, enemy: &str, stage: u32, mission: u32) {
        // Find or create progress entry
        let mut found = false;
        for progress in &mut self.stage_progress {
            if progress.player_faction == faction && progress.enemy_faction == enemy {
                if stage > progress.highest_stage {
                    progress.highest_stage = stage;
                }
                if mission > progress.highest_mission {
                    progress.highest_mission = mission;
                }
                found = true;
                break;
            }
        }

        if !found {
            self.stage_progress.push(FactionProgress {
                player_faction: faction.to_string(),
                enemy_faction: enemy.to_string(),
                highest_stage: stage,
                highest_mission: mission,
            });
        }
    }

    /// Get highest stage for faction pair
    pub fn get_highest_stage(&self, faction: &str, enemy: &str) -> u32 {
        for progress in &self.stage_progress {
            if progress.player_faction == faction && progress.enemy_faction == enemy {
                return progress.highest_stage;
            }
        }
        0
    }

    /// Get high score for faction pair
    pub fn get_high_score(&self, faction: &str, enemy: &str) -> u64 {
        for hs in &self.high_scores {
            if hs.player_faction == faction && hs.enemy_faction == enemy {
                return hs.score;
            }
        }
        0
    }

    /// Record high score
    pub fn record_score(&mut self, faction: &str, enemy: &str, score: u64, stage: u32) {
        // Find or create entry
        let mut found = false;
        for hs in &mut self.high_scores {
            if hs.player_faction == faction && hs.enemy_faction == enemy {
                if score > hs.score {
                    hs.score = score;
                    hs.stage = stage;
                }
                found = true;
                break;
            }
        }

        if !found {
            self.high_scores.push(HighScore {
                player_faction: faction.to_string(),
                enemy_faction: enemy.to_string(),
                score,
                stage,
            });
        }
    }

    /// Add credits
    pub fn add_credits(&mut self, amount: u64) {
        self.lifetime_credits += amount;
    }
}

/// Load save data on startup
fn load_save_data(mut commands: Commands) {
    let save = SaveData::load();
    commands.insert_resource(save);
}

/// Auto-save when data changes
fn auto_save(save: Res<SaveData>) {
    save.save();
}
