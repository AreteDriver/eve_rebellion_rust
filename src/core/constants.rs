//! Game Constants
//!
//! All magic numbers and configuration values in one place.

#![allow(dead_code)]

use bevy::prelude::*;

// =============================================================================
// SCREEN & WINDOW
// =============================================================================

/// Base game resolution width
pub const SCREEN_WIDTH: f32 = 800.0;

/// Base game resolution height
pub const SCREEN_HEIGHT: f32 = 700.0;

/// Window title
pub const WINDOW_TITLE: &str = "EVE Rebellion";

// =============================================================================
// PLAYER
// =============================================================================

/// Default player movement speed (pixels per second)
pub const PLAYER_SPEED: f32 = 300.0;

/// Player hitbox size (for collision)
pub const PLAYER_HITBOX_SIZE: f32 = 8.0;

/// Player sprite size
pub const PLAYER_SPRITE_SIZE: f32 = 48.0;

/// Player fire rate (shots per second)
pub const PLAYER_FIRE_RATE: f32 = 8.0;

/// Player bullet speed
pub const PLAYER_BULLET_SPEED: f32 = 600.0;

/// Player bullet damage
pub const PLAYER_BULLET_DAMAGE: f32 = 10.0;

/// Default player shield
pub const PLAYER_DEFAULT_SHIELD: f32 = 100.0;

/// Default player armor
pub const PLAYER_DEFAULT_ARMOR: f32 = 100.0;

/// Default player hull
pub const PLAYER_DEFAULT_HULL: f32 = 100.0;

/// Shield recharge rate (per second, after delay)
pub const PLAYER_SHIELD_RECHARGE_RATE: f32 = 5.0;

/// Delay before shields start recharging (seconds)
pub const PLAYER_SHIELD_RECHARGE_DELAY: f32 = 3.0;

// =============================================================================
// ENEMIES
// =============================================================================

/// Base enemy speed
pub const ENEMY_BASE_SPEED: f32 = 100.0;

/// Enemy spawn margin from screen edges
pub const ENEMY_SPAWN_MARGIN: f32 = 50.0;

/// Time between enemy waves (seconds)
pub const WAVE_DELAY: f32 = 3.0;

// =============================================================================
// PROJECTILES
// =============================================================================

/// Standard bullet size
pub const BULLET_SIZE: f32 = 8.0;

/// Enemy bullet speed
pub const ENEMY_BULLET_SPEED: f32 = 300.0;

// =============================================================================
// SCORING
// =============================================================================

/// Base points per kill
pub const POINTS_PER_KILL: u64 = 100;

/// Chain timeout (seconds to maintain combo)
pub const CHAIN_TIMEOUT: f32 = 2.0;

/// Multiplier increase per kill
pub const MULTIPLIER_PER_KILL: f32 = 0.1;

/// Maximum score multiplier
pub const MAX_MULTIPLIER: f32 = 99.9;

// =============================================================================
// BERSERK
// =============================================================================

/// Berserk meter fill per kill
pub const BERSERK_GAIN_PER_KILL: f32 = 5.0;

/// Berserk meter fill per graze
pub const BERSERK_GAIN_PER_GRAZE: f32 = 1.0;

/// Berserk duration (seconds)
pub const BERSERK_DURATION: f32 = 10.0;

/// Berserk damage multiplier
pub const BERSERK_DAMAGE_MULT: f32 = 2.0;

/// Berserk speed multiplier
pub const BERSERK_SPEED_MULT: f32 = 1.5;

// =============================================================================
// CAPACITOR (EVE-STYLE)
// =============================================================================

/// Base capacitor for frigates (GJ)
pub const CAP_FRIGATE: f32 = 250.0;

/// Base capacitor for destroyers (GJ)
pub const CAP_DESTROYER: f32 = 400.0;

/// Base capacitor for cruisers (GJ)
pub const CAP_CRUISER: f32 = 1200.0;

/// Base capacitor for battlecruisers (GJ)
pub const CAP_BATTLECRUISER: f32 = 2000.0;

/// Base capacitor for battleships (GJ)
pub const CAP_BATTLESHIP: f32 = 5000.0;

// =============================================================================
// SHIP SIZES (relative to each other, in pixels)
// =============================================================================

/// Frigate sprite size (base unit)
pub const SIZE_FRIGATE: f32 = 36.0;

/// Assault Frigate sprite size (slightly larger than frigate)
pub const SIZE_ASSAULT_FRIGATE: f32 = 40.0;

/// Interceptor sprite size (smaller, faster ships)
pub const SIZE_INTERCEPTOR: f32 = 32.0;

/// Destroyer sprite size (~1.5x frigate)
pub const SIZE_DESTROYER: f32 = 52.0;

/// Tactical Destroyer sprite size
pub const SIZE_TACTICAL_DESTROYER: f32 = 48.0;

/// Cruiser sprite size (~2x frigate)
pub const SIZE_CRUISER: f32 = 72.0;

/// Battlecruiser sprite size (~2.5x frigate)
pub const SIZE_BATTLECRUISER: f32 = 88.0;

/// Battleship sprite size (~3x frigate)
pub const SIZE_BATTLESHIP: f32 = 110.0;

/// Carrier sprite size (~4x frigate, background element)
pub const SIZE_CARRIER: f32 = 160.0;

/// Player ship size bonus (player ships slightly larger for visibility)
pub const PLAYER_SIZE_BONUS: f32 = 1.15;

// =============================================================================
// EVE IMAGE SERVER
// =============================================================================

/// EVE Image Server base URL
pub const EVE_IMAGE_SERVER: &str = "https://images.evetech.net";

/// Default ship render size
pub const SHIP_RENDER_SIZE: u32 = 256;

// =============================================================================
// COLORS
// =============================================================================

/// Minmatar faction color (rust/copper)
pub const COLOR_MINMATAR: Color = Color::srgb(0.8, 0.35, 0.2);

/// Amarr faction color (gold)
pub const COLOR_AMARR: Color = Color::srgb(0.9, 0.75, 0.2);

/// Caldari faction color (steel blue)
pub const COLOR_CALDARI: Color = Color::srgb(0.3, 0.5, 0.7);

/// Gallente faction color (green)
pub const COLOR_GALLENTE: Color = Color::srgb(0.35, 0.6, 0.35);

/// Triglavian faction color (red/crimson)
pub const COLOR_TRIGLAVIAN: Color = Color::srgb(0.7, 0.15, 0.2);

/// Shield color (blue)
pub const COLOR_SHIELD: Color = Color::srgb(0.2, 0.6, 1.0);

/// Armor color (red)
pub const COLOR_ARMOR: Color = Color::srgb(0.8, 0.2, 0.2);

/// Hull color (gray)
pub const COLOR_HULL: Color = Color::srgb(0.5, 0.5, 0.5);

/// Capacitor color (orange)
pub const COLOR_CAPACITOR: Color = Color::srgb(1.0, 0.55, 0.0);

// =============================================================================
// LAYERS (Z-ordering)
// =============================================================================

/// Background layer
pub const LAYER_BACKGROUND: f32 = 0.0;

/// Stars/parallax layer
pub const LAYER_STARS: f32 = 1.0;

/// Hazards layer
pub const LAYER_HAZARDS: f32 = 5.0;

/// Collectibles layer
pub const LAYER_COLLECTIBLES: f32 = 8.0;

/// Enemy bullets layer
pub const LAYER_ENEMY_BULLETS: f32 = 9.0;

/// Enemies layer
pub const LAYER_ENEMIES: f32 = 10.0;

/// Player bullets layer
pub const LAYER_PLAYER_BULLETS: f32 = 11.0;

/// Player layer
pub const LAYER_PLAYER: f32 = 12.0;

/// Effects layer
pub const LAYER_EFFECTS: f32 = 15.0;

/// HUD layer
pub const LAYER_HUD: f32 = 100.0;
