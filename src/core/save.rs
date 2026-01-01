//! Save/Load System
//!
//! Persists player progression, unlocks, and settings.

#![allow(dead_code)]

use crate::systems::{RumbleSettings, ScreenShake, SoundSettings};
use bevy::prelude::*;
use serde::{Deserialize, Serialize};
use std::collections::HashSet;
#[cfg(not(target_arch = "wasm32"))]
use std::fs;
#[cfg(not(target_arch = "wasm32"))]
use std::path::PathBuf;

/// Save system plugin
pub struct SavePlugin;

impl Plugin for SavePlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<SaveData>()
            .add_systems(Startup, load_save_data)
            .add_systems(PostStartup, apply_saved_settings)
            .add_systems(Update, auto_save.run_if(resource_changed::<SaveData>))
            .add_systems(Update, sync_settings_to_save);
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
    /// Screen shake intensity (0.0 = off, 1.0 = full)
    #[serde(default = "default_shake_intensity")]
    pub screen_shake_intensity: f32,
    /// Controller rumble intensity (0.0 = off, 1.0 = full)
    #[serde(default = "default_rumble_intensity")]
    pub rumble_intensity: f32,
}

fn default_shake_intensity() -> f32 {
    1.0
}

fn default_rumble_intensity() -> f32 {
    1.0
}

impl Default for GameSettings {
    fn default() -> Self {
        Self {
            master_volume: 0.7,
            sfx_volume: 0.8,
            music_volume: 0.5,
            screen_shake_intensity: 1.0,
            rumble_intensity: 1.0,
        }
    }
}

impl SaveData {
    /// Get save file path (native only)
    #[cfg(not(target_arch = "wasm32"))]
    fn save_path() -> PathBuf {
        dirs::data_dir()
            .unwrap_or_else(|| PathBuf::from("."))
            .join("eve_rebellion")
            .join("save.json")
    }

    /// Load from disk (native)
    #[cfg(not(target_arch = "wasm32"))]
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

    /// Load from localStorage (WASM)
    #[cfg(target_arch = "wasm32")]
    pub fn load() -> Self {
        use web_sys::window;

        if let Some(window) = window() {
            if let Ok(Some(storage)) = window.local_storage() {
                if let Ok(Some(data)) = storage.get_item("eve_rebellion_save") {
                    if let Ok(save) = serde_json::from_str(&data) {
                        info!("Loaded save data from localStorage");
                        return save;
                    }
                }
            }
        }
        info!("No save data found, using defaults");
        Self::default()
    }

    /// Save to disk (native)
    #[cfg(not(target_arch = "wasm32"))]
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

    /// Save to localStorage (WASM)
    #[cfg(target_arch = "wasm32")]
    pub fn save(&self) {
        use web_sys::window;

        if let Some(window) = window() {
            if let Ok(Some(storage)) = window.local_storage() {
                if let Ok(data) = serde_json::to_string(self) {
                    if storage.set_item("eve_rebellion_save", &data).is_ok() {
                        info!("Saved progress to localStorage");
                    }
                }
            }
        }
    }

    /// Check if a ship is unlocked
    pub fn is_ship_unlocked(
        &self,
        type_id: u32,
        unlock_stage: u32,
        faction: &str,
        enemy: &str,
    ) -> bool {
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

/// Apply saved settings to runtime resources (runs after all plugins init)
fn apply_saved_settings(
    save: Res<SaveData>,
    mut sound: ResMut<SoundSettings>,
    mut shake: ResMut<ScreenShake>,
    mut rumble: ResMut<RumbleSettings>,
) {
    let settings = &save.settings;

    // Apply audio settings
    sound.master_volume = settings.master_volume;
    sound.sfx_volume = settings.sfx_volume;
    sound.music_volume = settings.music_volume;

    // Apply screen shake intensity
    shake.multiplier = settings.screen_shake_intensity;

    // Apply rumble intensity
    rumble.intensity = settings.rumble_intensity;

    info!(
        "Applied saved settings: master={:.0}%, sfx={:.0}%, music={:.0}%, shake={:.0}%, rumble={:.0}%",
        settings.master_volume * 100.0,
        settings.sfx_volume * 100.0,
        settings.music_volume * 100.0,
        settings.screen_shake_intensity * 100.0,
        settings.rumble_intensity * 100.0
    );
}

/// Sync runtime settings changes back to SaveData
/// Only runs when SoundSettings, ScreenShake, or RumbleSettings resources change
fn sync_settings_to_save(
    sound: Res<SoundSettings>,
    shake: Res<ScreenShake>,
    rumble: Res<RumbleSettings>,
    mut save: ResMut<SaveData>,
) {
    // Only process if any resource changed this frame
    if !sound.is_changed() && !shake.is_changed() && !rumble.is_changed() {
        return;
    }

    // Check if settings actually differ from saved values
    let settings = &save.settings;
    let sound_changed = (settings.master_volume - sound.master_volume).abs() > 0.001
        || (settings.sfx_volume - sound.sfx_volume).abs() > 0.001
        || (settings.music_volume - sound.music_volume).abs() > 0.001;
    let shake_changed = (settings.screen_shake_intensity - shake.multiplier).abs() > 0.001;
    let rumble_changed = (settings.rumble_intensity - rumble.intensity).abs() > 0.001;

    if !sound_changed && !shake_changed && !rumble_changed {
        return;
    }

    // Update settings
    let settings = &mut save.settings;
    if sound_changed {
        settings.master_volume = sound.master_volume;
        settings.sfx_volume = sound.sfx_volume;
        settings.music_volume = sound.music_volume;
    }
    if shake_changed {
        settings.screen_shake_intensity = shake.multiplier;
    }
    if rumble_changed {
        settings.rumble_intensity = rumble.intensity;
    }

    info!(
        "Settings synced to save: master={:.0}%, sfx={:.0}%, music={:.0}%, shake={:.0}%, rumble={:.0}%",
        settings.master_volume * 100.0,
        settings.sfx_volume * 100.0,
        settings.music_volume * 100.0,
        settings.screen_shake_intensity * 100.0,
        settings.rumble_intensity * 100.0
    );
}

#[cfg(test)]
mod tests {
    use super::*;

    // ==================== Ship Unlock Tests ====================

    #[test]
    fn ship_unlock_stage_zero_always_unlocked() {
        let save = SaveData::default();
        // unlock_stage 0 = always available
        assert!(save.is_ship_unlocked(12345, 0, "Minmatar", "Amarr"));
    }

    #[test]
    fn ship_unlock_explicit_unlock() {
        let mut save = SaveData::default();
        save.unlock_ship(587); // Wolf type_id

        // Should be unlocked regardless of stage requirement
        assert!(save.is_ship_unlocked(587, 5, "Minmatar", "Amarr"));
    }

    #[test]
    fn ship_unlock_via_stage_progress() {
        let mut save = SaveData::default();
        save.complete_stage("Minmatar", "Amarr", 5, 1);

        // Ship requiring stage 5 should be unlocked
        assert!(save.is_ship_unlocked(587, 5, "Minmatar", "Amarr"));

        // Ship requiring stage 6 should NOT be unlocked
        assert!(!save.is_ship_unlocked(588, 6, "Minmatar", "Amarr"));
    }

    #[test]
    fn ship_unlock_wrong_faction_not_counted() {
        let mut save = SaveData::default();
        save.complete_stage("Caldari", "Gallente", 10, 1);

        // Progress in Caldari vs Gallente doesn't unlock Minmatar ships
        assert!(!save.is_ship_unlocked(587, 5, "Minmatar", "Amarr"));
    }

    // ==================== Stage Progress Tests ====================

    #[test]
    fn complete_stage_creates_entry() {
        let mut save = SaveData::default();
        assert_eq!(save.get_highest_stage("Minmatar", "Amarr"), 0);

        save.complete_stage("Minmatar", "Amarr", 3, 2);
        assert_eq!(save.get_highest_stage("Minmatar", "Amarr"), 3);
    }

    #[test]
    fn complete_stage_updates_highest() {
        let mut save = SaveData::default();
        save.complete_stage("Minmatar", "Amarr", 3, 1);
        save.complete_stage("Minmatar", "Amarr", 5, 2);

        assert_eq!(save.get_highest_stage("Minmatar", "Amarr"), 5);
    }

    #[test]
    fn complete_stage_does_not_decrease() {
        let mut save = SaveData::default();
        save.complete_stage("Minmatar", "Amarr", 5, 1);
        save.complete_stage("Minmatar", "Amarr", 3, 1); // Lower stage

        assert_eq!(save.get_highest_stage("Minmatar", "Amarr"), 5);
    }

    #[test]
    fn complete_stage_multiple_factions_independent() {
        let mut save = SaveData::default();
        save.complete_stage("Minmatar", "Amarr", 5, 1);
        save.complete_stage("Caldari", "Gallente", 3, 1);

        assert_eq!(save.get_highest_stage("Minmatar", "Amarr"), 5);
        assert_eq!(save.get_highest_stage("Caldari", "Gallente"), 3);
        assert_eq!(save.get_highest_stage("Gallente", "Caldari"), 0);
    }

    // ==================== High Score Tests ====================

    #[test]
    fn record_score_creates_entry() {
        let mut save = SaveData::default();
        assert_eq!(save.get_high_score("Minmatar", "Amarr"), 0);

        save.record_score("Minmatar", "Amarr", 50000, 5);
        assert_eq!(save.get_high_score("Minmatar", "Amarr"), 50000);
    }

    #[test]
    fn record_score_only_updates_on_beat() {
        let mut save = SaveData::default();
        save.record_score("Minmatar", "Amarr", 50000, 5);
        save.record_score("Minmatar", "Amarr", 30000, 3); // Lower score

        assert_eq!(save.get_high_score("Minmatar", "Amarr"), 50000);
    }

    #[test]
    fn record_score_updates_on_new_high() {
        let mut save = SaveData::default();
        save.record_score("Minmatar", "Amarr", 50000, 5);
        save.record_score("Minmatar", "Amarr", 75000, 7); // Higher score

        assert_eq!(save.get_high_score("Minmatar", "Amarr"), 75000);
    }

    #[test]
    fn record_score_multiple_factions_independent() {
        let mut save = SaveData::default();
        save.record_score("Minmatar", "Amarr", 50000, 5);
        save.record_score("Caldari", "Gallente", 100000, 10);

        assert_eq!(save.get_high_score("Minmatar", "Amarr"), 50000);
        assert_eq!(save.get_high_score("Caldari", "Gallente"), 100000);
    }

    // ==================== Credits Tests ====================

    #[test]
    fn add_credits_accumulates() {
        let mut save = SaveData::default();
        assert_eq!(save.lifetime_credits, 0);

        save.add_credits(1000);
        assert_eq!(save.lifetime_credits, 1000);

        save.add_credits(500);
        assert_eq!(save.lifetime_credits, 1500);
    }

    // ==================== Settings Tests ====================

    #[test]
    fn default_settings() {
        let settings = GameSettings::default();
        assert_eq!(settings.master_volume, 0.7);
        assert_eq!(settings.sfx_volume, 0.8);
        assert_eq!(settings.music_volume, 0.5);
        assert_eq!(settings.screen_shake_intensity, 1.0);
        assert_eq!(settings.rumble_intensity, 1.0);
    }

    // ==================== Serialization Tests ====================

    #[test]
    fn save_data_serialization_roundtrip() {
        let mut save = SaveData::default();
        save.complete_stage("Minmatar", "Amarr", 5, 3);
        save.record_score("Minmatar", "Amarr", 50000, 5);
        save.unlock_ship(587);
        save.add_credits(10000);

        // Serialize
        let json = serde_json::to_string(&save).expect("serialize");

        // Deserialize
        let loaded: SaveData = serde_json::from_str(&json).expect("deserialize");

        assert_eq!(loaded.get_highest_stage("Minmatar", "Amarr"), 5);
        assert_eq!(loaded.get_high_score("Minmatar", "Amarr"), 50000);
        assert!(loaded.unlocked_ships.contains(&587));
        assert_eq!(loaded.lifetime_credits, 10000);
    }
}
