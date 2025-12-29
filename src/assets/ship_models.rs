//! EVE Ship 3D Model Loading
//!
//! Loads and caches 3D ship models from GLTF/GLB files.
//! Falls back to 2D sprites when models are unavailable.

#![allow(dead_code)]

use bevy::prelude::*;
use std::collections::HashMap;

use crate::core::*;

/// Ship model loading plugin
pub struct ShipModelsPlugin;

impl Plugin for ShipModelsPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<ShipModelCache>()
            .add_systems(Startup, setup_model_cache)
            .add_systems(
                Update,
                load_available_models.run_if(in_state(GameState::Loading)),
            );
    }
}

/// Cache of loaded ship 3D model handles
#[derive(Resource, Default)]
pub struct ShipModelCache {
    /// Map of type_id -> scene handle (GLTF scene)
    pub models: HashMap<u32, Handle<Scene>>,
    /// Whether initial load attempt is complete
    pub ready: bool,
}

impl ShipModelCache {
    /// Get 3D model scene for a ship type
    pub fn get(&self, type_id: u32) -> Option<Handle<Scene>> {
        self.models.get(&type_id).cloned()
    }

    /// Check if a model exists for this type
    pub fn has_model(&self, type_id: u32) -> bool {
        self.models.contains_key(&type_id)
    }
}

/// Ship model metadata: (type_id, filename, scale)
/// Scale normalizes model to roughly 1 unit = 1 pixel at base size
const SHIP_MODELS: &[(u32, &str, f32)] = &[
    // Minmatar
    (587, "rifter.glb", 0.015),
    (585, "slasher.glb", 0.015),
    (586, "probe.glb", 0.015),
    (598, "breacher.glb", 0.015),
    // Amarr
    (597, "punisher.glb", 0.015),
    (589, "executioner.glb", 0.015),
    (591, "tormentor.glb", 0.015),
    // Caldari
    (603, "merlin.glb", 0.015),
    (602, "kestrel.glb", 0.015),
    // Gallente
    (593, "tristan.glb", 0.015),
    (594, "incursus.glb", 0.015),
];

fn setup_model_cache(mut cache: ResMut<ShipModelCache>) {
    cache.ready = false;
    info!("Ship model cache initialized");
}

fn load_available_models(mut cache: ResMut<ShipModelCache>, asset_server: Res<AssetServer>) {
    if cache.ready {
        return;
    }

    // Try to load each model
    for &(type_id, filename, _scale) in SHIP_MODELS {
        if let std::collections::hash_map::Entry::Vacant(e) = cache.models.entry(type_id) {
            // Bevy 0.15: Load GLTF scene using path#Scene0 syntax
            let path = format!("models/{}#Scene0", filename);
            let handle: Handle<Scene> = asset_server.load(&path);
            e.insert(handle);
            info!("Queued model load: {} for type_id {}", filename, type_id);
        }
    }

    cache.ready = true;
    info!(
        "Ship model loading initiated for {} potential models",
        SHIP_MODELS.len()
    );
}

/// Get the scale factor for a ship model
pub fn get_model_scale(type_id: u32) -> f32 {
    SHIP_MODELS
        .iter()
        .find(|(id, _, _)| *id == type_id)
        .map(|(_, _, scale)| *scale)
        .unwrap_or(0.015)
}

/// Component to track velocity-based rotation for banking/tilting
#[derive(Component, Debug, Clone)]
pub struct ShipModelRotation {
    /// Base orientation (facing direction when stationary)
    pub base_rotation: Quat,
    /// Maximum bank angle in radians
    pub max_bank: f32,
    /// Maximum pitch angle in radians
    pub max_pitch: f32,
    /// Smoothing factor for rotation (higher = faster response)
    pub smoothing: f32,
}

impl Default for ShipModelRotation {
    fn default() -> Self {
        Self::new_player()
    }
}

impl ShipModelRotation {
    /// Create rotation config for player (faces +Y / up)
    pub fn new_player() -> Self {
        Self {
            // GLTF models typically face +Z, rotate to face +Y (up in 2D space)
            // Then rotate around X to lay flat in XY plane when viewed from above
            base_rotation: Quat::from_rotation_x(-std::f32::consts::FRAC_PI_2),
            max_bank: 0.4,   // ~23 degrees roll
            max_pitch: 0.25, // ~14 degrees pitch
            smoothing: 8.0,
        }
    }

    /// Create rotation config for enemy (faces -Y / down toward player)
    pub fn new_enemy() -> Self {
        Self {
            // Same as player but rotated 180 degrees to face down
            base_rotation: Quat::from_rotation_x(-std::f32::consts::FRAC_PI_2)
                * Quat::from_rotation_z(std::f32::consts::PI),
            max_bank: 0.35,
            max_pitch: 0.2,
            smoothing: 6.0,
        }
    }

    /// Create rotation config for boss (less agile)
    pub fn new_boss() -> Self {
        Self {
            base_rotation: Quat::from_rotation_x(-std::f32::consts::FRAC_PI_2)
                * Quat::from_rotation_z(std::f32::consts::PI),
            max_bank: 0.15, // Bosses bank less
            max_pitch: 0.1,
            smoothing: 3.0, // Slower response
        }
    }

    /// Calculate target rotation based on velocity
    pub fn calculate_rotation(&self, velocity: Vec2, max_speed: f32) -> Quat {
        if max_speed <= 0.0 {
            return self.base_rotation;
        }

        // Normalize velocity to get bank/pitch factor (-1 to 1)
        let vx = (velocity.x / max_speed).clamp(-1.0, 1.0);
        let vy = (velocity.y / max_speed).clamp(-1.0, 1.0);

        // Bank (roll around forward axis) based on horizontal velocity
        // Moving right = bank right (negative rotation for visual effect)
        let bank_angle = -vx * self.max_bank;

        // Pitch (nose up/down) based on vertical velocity
        let pitch_angle = vy * self.max_pitch;

        // Combine: base rotation * local pitch * local bank
        self.base_rotation * Quat::from_rotation_x(pitch_angle) * Quat::from_rotation_y(bank_angle)
    }
}
