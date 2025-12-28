//! EVE Powerup Icon Loading
//!
//! Loads powerup icons from the assets/powerups directory.

use bevy::prelude::*;
use bevy::render::render_asset::RenderAssetUsages;
use std::collections::HashMap;
use std::fs;
use std::path::PathBuf;

use crate::core::CollectibleType;

/// Powerup icons plugin
pub struct PowerupIconsPlugin;

impl Plugin for PowerupIconsPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<PowerupIconCache>()
            .add_systems(Startup, load_powerup_icons);
    }
}

/// Cache of loaded powerup icon handles
#[derive(Resource, Default)]
pub struct PowerupIconCache {
    /// Map of collectible type -> texture handle
    pub icons: HashMap<CollectibleType, Handle<Image>>,
}

impl PowerupIconCache {
    /// Get icon for a collectible type
    pub fn get(&self, collectible_type: &CollectibleType) -> Option<Handle<Image>> {
        self.icons.get(collectible_type).cloned()
    }
}

/// Map collectible types to icon filenames
fn get_icon_filename(collectible_type: &CollectibleType) -> Option<&'static str> {
    match collectible_type {
        CollectibleType::ShieldBoost => Some("shield_hardener.png"),
        CollectibleType::ArmorRepair => Some("armor_hardener.png"),
        CollectibleType::HullRepair => Some("reinforced_bulkheads.png"),
        CollectibleType::Overdrive => Some("microwarpdrive.png"),
        CollectibleType::DamageBoost => Some("combat_booster.png"),
        CollectibleType::Invulnerability => Some("assault_damage_control.png"),
        CollectibleType::Nanite => Some("nanite_paste.png"),
        CollectibleType::ExtraLife => Some("speed_booster.png"),
        _ => None, // Credits, Refugee, Capacitor use simple shapes
    }
}

/// Load powerup icons from assets directory
fn load_powerup_icons(mut cache: ResMut<PowerupIconCache>, mut images: ResMut<Assets<Image>>) {
    // Construct path to powerups assets directory
    let assets_dir = std::env::current_dir()
        .unwrap_or_else(|_| PathBuf::from("."))
        .join("assets")
        .join("powerups");

    if !assets_dir.exists() {
        warn!("Powerup icons directory not found: {:?}", assets_dir);
        return;
    }

    info!("Loading powerup icons from {:?}", assets_dir);

    // Load each icon type
    let types = [
        CollectibleType::ShieldBoost,
        CollectibleType::ArmorRepair,
        CollectibleType::HullRepair,
        CollectibleType::Overdrive,
        CollectibleType::DamageBoost,
        CollectibleType::Invulnerability,
        CollectibleType::Nanite,
        CollectibleType::ExtraLife,
    ];

    for collectible_type in types {
        if let Some(filename) = get_icon_filename(&collectible_type) {
            let path = assets_dir.join(filename);
            if path.exists() {
                match load_image_file(&path) {
                    Ok(image) => {
                        let handle = images.add(image);
                        cache.icons.insert(collectible_type, handle);
                        info!("Loaded powerup icon: {}", filename);
                    }
                    Err(e) => {
                        warn!("Failed to load powerup icon {}: {}", filename, e);
                    }
                }
            } else {
                warn!("Powerup icon not found: {:?}", path);
            }
        }
    }

    info!("Loaded {} powerup icons", cache.icons.len());
}

/// Load an image file and convert to Bevy Image
fn load_image_file(path: &PathBuf) -> Result<Image, String> {
    let bytes = fs::read(path).map_err(|e| e.to_string())?;

    let img = image::load_from_memory(&bytes)
        .map_err(|e| e.to_string())?
        .into_rgba8();

    let (width, height) = img.dimensions();
    let data = img.into_raw();

    Ok(Image::new(
        bevy::render::render_resource::Extent3d {
            width,
            height,
            depth_or_array_layers: 1,
        },
        bevy::render::render_resource::TextureDimension::D2,
        data,
        bevy::render::render_resource::TextureFormat::Rgba8UnormSrgb,
        RenderAssetUsages::RENDER_WORLD,
    ))
}
