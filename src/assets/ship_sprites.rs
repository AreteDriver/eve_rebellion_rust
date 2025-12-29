//! EVE Ship Sprite Loading
//!
//! Loads ship sprites from bundled assets, with fallback to CCP's Image Server.
//! Priority: assets/ships/{type_id}.png -> cache -> download

#![allow(dead_code)]

use bevy::prelude::*;
use bevy::render::render_asset::RenderAssetUsages;
use std::collections::HashMap;
use std::fs;
use std::path::PathBuf;

use crate::core::*;

/// EVE Image Server base URL (fallback only)
const IMAGE_SERVER: &str = "https://images.evetech.net";

/// Default render size for downloads
const RENDER_SIZE: u32 = 128;

/// Bundled assets directory
const BUNDLED_SHIPS_DIR: &str = "assets/ships";

/// Ship sprites plugin
pub struct ShipSpritesPlugin;

impl Plugin for ShipSpritesPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<ShipSpriteCache>()
            .add_systems(Startup, setup_sprite_cache)
            .add_systems(OnEnter(GameState::Loading), start_loading_sprites)
            .add_systems(
                Update,
                check_sprite_loading.run_if(in_state(GameState::Loading)),
            );
    }
}

/// Cache of loaded ship sprite handles
#[derive(Resource, Default)]
pub struct ShipSpriteCache {
    /// Map of type_id -> texture handle
    pub sprites: HashMap<u32, Handle<Image>>,
    /// Ships currently being loaded
    pub loading: Vec<u32>,
    /// Whether initial load is complete
    pub ready: bool,
    /// Cache directory path
    pub cache_dir: PathBuf,
}

impl ShipSpriteCache {
    /// Get sprite for a ship type, returns None if not loaded
    pub fn get(&self, type_id: u32) -> Option<Handle<Image>> {
        self.sprites.get(&type_id).cloned()
    }
}

/// Ships to preload - all player and enemy ships used in game
const SHIPS_TO_LOAD: &[u32] = &[
    // === MINMATAR (player + enemy ships) ===
    587,   // Rifter
    585,   // Slasher
    598,   // Breacher
    11371, // Wolf
    11400, // Jaguar
    // === AMARR (player + enemy ships) ===
    589,   // Executioner
    597,   // Punisher
    591,   // Tormentor
    11186, // Crusader (interceptor)
    11184, // Malediction (interceptor)
    16236, // Coercer (destroyer)
    24690, // Harbinger (battlecruiser)
    // === CALDARI (player + enemy ships) ===
    602,   // Kestrel
    603,   // Merlin
    583,   // Condor
    11381, // Hawk
    11387, // Harpy
    35683, // Jackdaw
    16238, // Cormorant (destroyer)
    24688, // Drake (battlecruiser)
    // === GALLENTE (player + enemy ships) ===
    593,   // Tristan
    594,   // Incursus
    608,   // Atron
    11373, // Enyo
    11377, // Ishkur
    35685, // Hecate
    16242, // Catalyst (destroyer)
    24700, // Myrmidon (battlecruiser)
    // === CARRIERS (wave spawners) ===
    23757, // Archon (Amarr carrier)
    23911, // Thanatos (Gallente carrier)
    23915, // Chimera (Caldari carrier)
    24483, // Nidhoggur (Minmatar carrier)
];

/// Setup the sprite cache directory
fn setup_sprite_cache(mut cache: ResMut<ShipSpriteCache>) {
    // Use home directory cache
    let cache_dir = dirs::cache_dir()
        .unwrap_or_else(|| PathBuf::from("."))
        .join("eve_rebellion")
        .join("sprites");

    // Create directory if it doesn't exist
    if let Err(e) = fs::create_dir_all(&cache_dir) {
        warn!("Failed to create sprite cache dir: {}", e);
    }

    cache.cache_dir = cache_dir;
    info!("Sprite cache directory: {:?}", cache.cache_dir);
}

/// Start loading ship sprites
fn start_loading_sprites(mut cache: ResMut<ShipSpriteCache>, mut images: ResMut<Assets<Image>>) {
    // Ensure cache_dir is set (in case setup hasn't run yet)
    if cache.cache_dir.as_os_str().is_empty() {
        cache.cache_dir = dirs::cache_dir()
            .unwrap_or_else(|| PathBuf::from("."))
            .join("eve_rebellion")
            .join("sprites");
        if let Err(e) = fs::create_dir_all(&cache.cache_dir) {
            warn!("Failed to create sprite cache dir: {}", e);
        }
    }

    info!("Loading {} ship sprites...", SHIPS_TO_LOAD.len());

    let bundled_dir = PathBuf::from(BUNDLED_SHIPS_DIR);
    let mut loaded_bundled = 0;
    let mut loaded_cached = 0;

    for &type_id in SHIPS_TO_LOAD {
        // Priority 1: Check bundled assets (fastest, works offline)
        let bundled_path = bundled_dir.join(format!("{}.png", type_id));
        if bundled_path.exists() {
            match load_image_file(&bundled_path) {
                Ok(image) => {
                    let handle = images.add(image);
                    cache.sprites.insert(type_id, handle);
                    loaded_bundled += 1;
                    continue;
                }
                Err(e) => {
                    warn!("Failed to load bundled sprite {}: {}", type_id, e);
                }
            }
        }

        // Priority 2: Check cache directory
        let cache_path = cache.cache_dir.join(format!("{}.png", type_id));
        if cache_path.exists() {
            match load_image_file(&cache_path) {
                Ok(image) => {
                    let handle = images.add(image);
                    cache.sprites.insert(type_id, handle);
                    loaded_cached += 1;
                    continue;
                }
                Err(e) => {
                    warn!("Failed to load cached sprite {}: {}", type_id, e);
                }
            }
        }

        // Priority 3: Need to download
        cache.loading.push(type_id);
    }

    info!(
        "Loaded {} bundled, {} cached sprites",
        loaded_bundled, loaded_cached
    );

    // If nothing to download, we're ready
    if cache.loading.is_empty() {
        cache.ready = true;
        info!("All sprites loaded!");
    } else {
        info!("Need to download {} sprites", cache.loading.len());
        // Spawn download task
        download_sprites(cache.loading.clone(), cache.cache_dir.clone());
    }
}

/// Load an image file (JPEG or PNG) and convert to Bevy Image
fn load_image_file(path: &PathBuf) -> Result<Image, String> {
    let bytes = fs::read(path).map_err(|e| e.to_string())?;

    // Use image crate to auto-detect format and decode
    let img = image::load_from_memory(&bytes)
        .map_err(|e| e.to_string())?
        .into_rgba8();

    // Note: Bundled sprites already have transparent backgrounds
    // Downloaded sprites from CCP's server need background removal (handled separately)

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

/// Load an image from downloaded bytes (needs background removal)
fn load_downloaded_image(path: &PathBuf) -> Result<Image, String> {
    let bytes = fs::read(path).map_err(|e| e.to_string())?;

    let mut img = image::load_from_memory(&bytes)
        .map_err(|e| e.to_string())?
        .into_rgba8();

    // Remove black background from CCP server images
    remove_black_background(&mut img);

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

/// Remove black background from ship sprites and smooth edges
fn remove_black_background(img: &mut image::RgbaImage) {
    let (width, height) = img.dimensions();

    // First pass: identify background pixels and make them transparent
    // EVE ship renders have a dark/black background
    for y in 0..height {
        for x in 0..width {
            let pixel = img.get_pixel(x, y);
            let r = pixel[0] as f32;
            let g = pixel[1] as f32;
            let b = pixel[2] as f32;

            // Calculate brightness (luminance)
            let brightness = 0.299 * r + 0.587 * g + 0.114 * b;

            // If pixel is very dark, make it transparent
            // Use a threshold that catches the black background but keeps ship details
            if brightness < 15.0 {
                img.put_pixel(x, y, image::Rgba([0, 0, 0, 0]));
            } else if brightness < 40.0 {
                // Semi-transparent for edge smoothing
                let alpha = ((brightness - 15.0) / 25.0 * 255.0) as u8;
                img.put_pixel(x, y, image::Rgba([pixel[0], pixel[1], pixel[2], alpha]));
            }
        }
    }
}

/// Download sprites in background (blocking for now, should be async)
fn download_sprites(type_ids: Vec<u32>, cache_dir: PathBuf) {
    std::thread::spawn(move || {
        for type_id in type_ids {
            let url = format!(
                "{}/types/{}/render?size={}",
                IMAGE_SERVER, type_id, RENDER_SIZE
            );
            let cache_path = cache_dir.join(format!("{}.png", type_id));

            info!("Downloading sprite for type {} from {}", type_id, url);

            match download_image(&url, &cache_path) {
                Ok(_) => info!("Downloaded sprite for type {}", type_id),
                Err(e) => warn!("Failed to download sprite for type {}: {}", type_id, e),
            }
        }
    });
}

/// Download a single image (blocking)
fn download_image(
    url: &str,
    path: &PathBuf,
) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    let response = reqwest::blocking::get(url)?;

    if !response.status().is_success() {
        return Err(format!("HTTP {}", response.status()).into());
    }

    let bytes = response.bytes()?;
    fs::write(path, &bytes)?;

    Ok(())
}

/// Check if sprites are loaded and transition state
fn check_sprite_loading(
    mut cache: ResMut<ShipSpriteCache>,
    mut images: ResMut<Assets<Image>>,
    time: Res<Time>,
    mut timer: Local<f32>,
    mut next_state: ResMut<NextState<GameState>>,
) {
    *timer += time.delta_secs();

    // Check every 0.5 seconds
    if *timer < 0.5 {
        return;
    }
    *timer = 0.0;

    if cache.ready {
        next_state.set(GameState::MainMenu);
        return;
    }

    // Check if downloads completed
    let mut all_loaded = true;
    for &type_id in &cache.loading.clone() {
        let cache_path = cache.cache_dir.join(format!("{}.png", type_id));
        if cache_path.exists() && !cache.sprites.contains_key(&type_id) {
            // Load downloaded image (needs background removal)
            match load_downloaded_image(&cache_path) {
                Ok(image) => {
                    let handle = images.add(image);
                    cache.sprites.insert(type_id, handle);
                    info!("Loaded downloaded sprite for type {}", type_id);
                }
                Err(e) => {
                    warn!("Failed to load downloaded sprite {}: {}", type_id, e);
                    all_loaded = false;
                }
            }
        } else if !cache_path.exists() {
            all_loaded = false;
        }
    }

    if all_loaded && !cache.loading.is_empty() {
        cache.loading.clear();
        cache.ready = true;
        info!("All sprites loaded!");
    }

    // Timeout after 10 seconds - proceed anyway
    if *timer > 10.0 && !cache.ready {
        warn!("Sprite loading timeout, proceeding without all sprites");
        cache.ready = true;
    }
}

/// Helper to get cache dir (for external use)
pub fn get_sprite_cache_dir() -> PathBuf {
    dirs::cache_dir()
        .unwrap_or_else(|| PathBuf::from("."))
        .join("eve_rebellion")
        .join("sprites")
}
