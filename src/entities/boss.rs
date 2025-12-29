//! Boss Entities
//!
//! Stage bosses for the Elder Fleet campaign.

use super::player::Hitbox;
use crate::assets::{get_model_scale, ShipModelCache, ShipModelRotation};
use crate::core::*;
use bevy::prelude::*;

/// Marker component for bosses
#[derive(Component)]
pub struct Boss;

/// Boss data
#[derive(Component, Debug, Clone)]
pub struct BossData {
    /// Boss ID (1-13)
    pub id: u32,
    /// Stage number
    pub stage: u32,
    /// Display name
    pub name: String,
    /// Boss title
    pub title: String,
    /// Ship class name
    pub ship_class: String,
    /// EVE type ID (for sprite)
    pub type_id: u32,
    /// Maximum health
    pub max_health: f32,
    /// Current health
    pub health: f32,
    /// Current phase (1-5)
    pub current_phase: u32,
    /// Total phases
    pub total_phases: u32,
    /// Score value
    pub score_value: u64,
    /// Liberation value (souls freed on defeat)
    pub liberation_value: u32,
    /// Is stationary (stations/gates)
    pub stationary: bool,
    /// Intro dialogue
    pub dialogue_intro: String,
    /// Defeat dialogue
    pub dialogue_defeat: String,
    /// Is boss enraged (below 20% health)
    pub is_enraged: bool,
    /// Enrage threshold (default 0.2 = 20%)
    pub enrage_threshold: f32,
}

/// Boss health bar component
#[derive(Component)]
pub struct BossHealthBar;

/// Boss phase data
#[derive(Clone, Debug)]
pub struct BossPhase {
    pub phase_number: u32,
    pub health_threshold: f32,
    pub attack_pattern: String,
    pub spawns_escorts: bool,
    pub escort_count: u32,
}

/// Boss states for intro/battle/defeat sequences
#[derive(Component, Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum BossState {
    #[default]
    Intro, // Boss entering, name card showing
    Battle,          // Active combat
    PhaseTransition, // Changing phases
    Defeated,        // Death sequence playing
}

/// Boss movement pattern
#[derive(Component, Debug, Clone)]
pub struct BossMovement {
    pub pattern: MovementPattern,
    pub timer: f32,
    pub speed: f32,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum MovementPattern {
    Stationary,
    Sweep,      // Side to side
    Strafe,     // Move and stop
    Aggressive, // Chase player
    Descend,    // For intro
}

impl Default for BossMovement {
    fn default() -> Self {
        Self {
            pattern: MovementPattern::Sweep,
            timer: 0.0,
            speed: 80.0,
        }
    }
}

/// Boss attack timer
#[derive(Component, Debug)]
pub struct BossAttack {
    pub pattern: String,
    pub fire_timer: f32,
    pub fire_rate: f32,
    pub burst_count: u32,
    pub burst_remaining: u32,
}

impl Default for BossAttack {
    fn default() -> Self {
        Self {
            pattern: "steady_beam".to_string(),
            fire_timer: 0.0,
            fire_rate: 0.8,
            burst_count: 3,
            burst_remaining: 0,
        }
    }
}

/// Boss bundle
#[derive(Bundle)]
pub struct BossBundle {
    pub boss: Boss,
    pub data: BossData,
    pub state: BossState,
    pub movement: BossMovement,
    pub attack: BossAttack,
    pub hitbox: Hitbox,
    pub sprite: Sprite,
    pub transform: Transform,
}

/// Load boss data from stage number
pub fn get_boss_for_stage(stage: u32) -> Option<BossData> {
    // Boss definitions based on config/bosses_campaign.json
    match stage {
        1 => Some(BossData {
            id: 1,
            stage: 1,
            name: "Slave Transport Overseer".to_string(),
            title: "Convoy Master Krador".to_string(),
            ship_class: "Bestower".to_string(),
            type_id: 1944,
            max_health: 500.0,
            health: 500.0,
            current_phase: 1,
            total_phases: 2,
            score_value: 5000,
            liberation_value: 50,
            stationary: false,
            dialogue_intro: "You dare attack an Imperial transport? Foolish rebel!".to_string(),
            dialogue_defeat: "The slaves... they're escaping...".to_string(),
            is_enraged: false,
            enrage_threshold: 0.2,
        }),
        2 => Some(BossData {
            id: 2,
            stage: 2,
            name: "Patrol Commander".to_string(),
            title: "Commander Sarevok".to_string(),
            ship_class: "Navy Omen".to_string(),
            type_id: 2006,
            max_health: 800.0,
            health: 800.0,
            current_phase: 1,
            total_phases: 2,
            score_value: 7500,
            liberation_value: 0,
            stationary: false,
            dialogue_intro: "Another rebel scum. I've crushed dozens like you.".to_string(),
            dialogue_defeat: "Impossible... a frigate...".to_string(),
            is_enraged: false,
            enrage_threshold: 0.2,
        }),
        3 => Some(BossData {
            id: 3,
            stage: 3,
            name: "Station Defense Battery".to_string(),
            title: "Holding Facility Defense Grid".to_string(),
            ship_class: "Orbital Platform".to_string(),
            type_id: 0,
            max_health: 1200.0,
            health: 1200.0,
            current_phase: 1,
            total_phases: 3,
            score_value: 10000,
            liberation_value: 100,
            stationary: true,
            dialogue_intro: "Defense grid online. Eliminate hostile.".to_string(),
            dialogue_defeat: "Core breach... structural failure...".to_string(),
            is_enraged: false,
            enrage_threshold: 0.2,
        }),
        4 => Some(BossData {
            id: 4,
            stage: 4,
            name: "Holder's Escort Fleet".to_string(),
            title: "Lord Holder Arzad's Guard".to_string(),
            ship_class: "Mixed".to_string(),
            type_id: 624, // Maller as primary
            max_health: 1500.0,
            health: 1500.0,
            current_phase: 1,
            total_phases: 3,
            score_value: 15000,
            liberation_value: 75,
            stationary: false,
            dialogue_intro: "You attack a Holder's estate? You will burn for this heresy!".to_string(),
            dialogue_defeat: "My slaves... my property... all lost...".to_string(),
            is_enraged: false,
            enrage_threshold: 0.2,
        }),
        5 => Some(BossData {
            id: 5,
            stage: 5,
            name: "Imperial Customs Commandant".to_string(),
            title: "Commandant Torash".to_string(),
            ship_class: "Prophecy".to_string(),
            type_id: 630,
            max_health: 2000.0,
            health: 2000.0,
            current_phase: 1,
            total_phases: 3,
            score_value: 20000,
            liberation_value: 150,
            stationary: false,
            dialogue_intro: "No cargo passes without Imperial inspection. Surrender.".to_string(),
            dialogue_defeat: "The customs... will... be avenged...".to_string(),
            is_enraged: false,
            enrage_threshold: 0.2,
        }),
        6 => Some(BossData {
            id: 6,
            stage: 6,
            name: "Inquisitor Vessel".to_string(),
            title: "Inquisitor Malkov".to_string(),
            ship_class: "Prophecy Variant".to_string(),
            type_id: 630,
            max_health: 2500.0,
            health: 2500.0,
            current_phase: 1,
            total_phases: 3,
            score_value: 25000,
            liberation_value: 0,
            stationary: false,
            dialogue_intro: "Heretics! The Scriptures demand your purification!".to_string(),
            dialogue_defeat: "God... will not... forget this blasphemy...".to_string(),
            is_enraged: false,
            enrage_threshold: 0.2,
        }),
        7 => Some(BossData {
            id: 7,
            stage: 7,
            name: "Navy Harbinger Strike Group".to_string(),
            title: "Strike Commander Venak".to_string(),
            ship_class: "Harbinger".to_string(),
            type_id: 24696,
            max_health: 3000.0,
            health: 3000.0,
            current_phase: 1,
            total_phases: 3,
            score_value: 30000,
            liberation_value: 0,
            stationary: false,
            dialogue_intro: "Strike group, weapons free. Eliminate rebel contact.".to_string(),
            dialogue_defeat: "All ships... lost... how...".to_string(),
            is_enraged: false,
            enrage_threshold: 0.2,
        }),
        8 => Some(BossData {
            id: 8,
            stage: 8,
            name: "Stargate Defense Grid".to_string(),
            title: "Gate Control: Sahtogas".to_string(),
            ship_class: "Infrastructure".to_string(),
            type_id: 0,
            max_health: 4000.0,
            health: 4000.0,
            current_phase: 1,
            total_phases: 4,
            score_value: 35000,
            liberation_value: 0,
            stationary: true,
            dialogue_intro: "Unauthorized vessel. Activating defense protocols.".to_string(),
            dialogue_defeat: "Gate control... offline... rebels... have breached...".to_string(),
            is_enraged: false,
            enrage_threshold: 0.2,
        }),
        9 => Some(BossData {
            id: 9,
            stage: 9,
            name: "Amarr Battlestation Core".to_string(),
            title: "Holding Facility Theta-7".to_string(),
            ship_class: "Station".to_string(),
            type_id: 0,
            max_health: 6000.0,
            health: 6000.0,
            current_phase: 1,
            total_phases: 5,
            score_value: 50000,
            liberation_value: 500,
            stationary: true,
            dialogue_intro: "Station defense grid activated. All personnel to combat stations.".to_string(),
            dialogue_defeat: "Reactor critical... containment failing... the slaves are free...".to_string(),
            is_enraged: false,
            enrage_threshold: 0.2,
        }),
        10 => Some(BossData {
            id: 10,
            stage: 10,
            name: "Imperial Navy Armageddon".to_string(),
            title: "Admiral Karsoth's Hammer".to_string(),
            ship_class: "Armageddon".to_string(),
            type_id: 643,
            max_health: 8000.0,
            health: 8000.0,
            current_phase: 1,
            total_phases: 3,
            score_value: 75000,
            liberation_value: 200,
            stationary: false,
            dialogue_intro: "You face the might of the Imperial Navy. Prepare for oblivion.".to_string(),
            dialogue_defeat: "The Armageddon... falls... this cannot be...".to_string(),
            is_enraged: false,
            enrage_threshold: 0.2,
        }),
        11 => Some(BossData {
            id: 11,
            stage: 11,
            name: "Amarr Carrier".to_string(),
            title: "Carrier Divine Providence".to_string(),
            ship_class: "Archon".to_string(),
            type_id: 23757,
            max_health: 10000.0,
            health: 10000.0,
            current_phase: 1,
            total_phases: 4,
            score_value: 100000,
            liberation_value: 300,
            stationary: false,
            dialogue_intro: "Launch all fighters. Annihilate the rebel frigate.".to_string(),
            dialogue_defeat: "Flight deck... compromised... she's going down...".to_string(),
            is_enraged: false,
            enrage_threshold: 0.2,
        }),
        12 => Some(BossData {
            id: 12,
            stage: 12,
            name: "Lord Admiral's Apocalypse".to_string(),
            title: "Lord Admiral Vanir".to_string(),
            ship_class: "Apocalypse Navy Issue".to_string(),
            type_id: 24690,
            max_health: 12000.0,
            health: 12000.0,
            current_phase: 1,
            total_phases: 4,
            score_value: 150000,
            liberation_value: 0,
            stationary: false,
            dialogue_intro: "I have served the Empire for two hundred years. You will not take this day.".to_string(),
            dialogue_defeat: "My Emperor... I have... failed...".to_string(),
            is_enraged: false,
            enrage_threshold: 0.2,
        }),
        13 => Some(BossData {
            id: 13,
            stage: 13,
            name: "Avatar-class Titan".to_string(),
            title: "Imperial Titan Empress's Wrath".to_string(),
            ship_class: "Avatar".to_string(),
            type_id: 11567,
            max_health: 25000.0,
            health: 25000.0,
            current_phase: 1,
            total_phases: 5,
            score_value: 500000,
            liberation_value: 1000,
            stationary: false,
            dialogue_intro: "A frigate? Against a Titan? Your courage is matched only by your foolishness.".to_string(),
            dialogue_defeat: "The Empress's Wrath... destroyed by... a single pilot... The Empire... will remember this day...".to_string(),
            is_enraged: false,
            enrage_threshold: 0.15, // Titan enrages at 15%
        }),
        _ => None,
    }
}

/// Spawn a stage boss
pub fn spawn_boss(
    commands: &mut Commands,
    stage: u32,
    sprite_cache: Option<&crate::assets::ShipSpriteCache>,
    model_cache: Option<&ShipModelCache>,
) -> bool {
    let Some(boss_data) = get_boss_for_stage(stage) else {
        return false;
    };

    // Get boss scale based on ship class
    let scale_mult = match boss_data.ship_class.as_str() {
        "Bestower" | "Orbital Platform" => 2.0,
        "Navy Omen" | "Prophecy" | "Prophecy Variant" | "Maller" => 2.5,
        "Harbinger" | "Infrastructure" => 3.0,
        "Station" | "Armageddon" => 4.0,
        "Archon" | "Apocalypse Navy Issue" => 5.0,
        "Avatar" => 7.0,
        _ => 2.0,
    };

    let size = 64.0 * scale_mult;
    let stationary = boss_data.stationary;

    let movement = if stationary {
        BossMovement {
            pattern: MovementPattern::Stationary,
            ..default()
        }
    } else {
        BossMovement {
            pattern: MovementPattern::Descend,
            timer: 0.0,
            speed: 80.0,
        }
    };

    // Spawn at top of screen
    let start_y = SCREEN_HEIGHT / 2.0 + size;

    // Try 3D model first
    if boss_data.type_id > 0 {
        if let Some(cache) = model_cache {
            if let Some(scene_handle) = cache.get(boss_data.type_id) {
                let model_rot = ShipModelRotation::new_boss();
                let model_scale = get_model_scale(boss_data.type_id) * size;

                commands.spawn((
                    Boss,
                    boss_data,
                    BossState::Intro,
                    movement,
                    BossAttack::default(),
                    Hitbox {
                        radius: size / 2.0 * 0.8,
                    },
                    model_rot.clone(),
                    SceneRoot(scene_handle),
                    Transform::from_xyz(0.0, start_y, 0.0)
                        .with_scale(Vec3::splat(model_scale))
                        .with_rotation(model_rot.base_rotation),
                ));
                return true;
            }
        }
    }

    // Fallback to sprite
    let sprite = if boss_data.type_id > 0 {
        if let Some(cache) = sprite_cache {
            if let Some(texture) = cache.get(boss_data.type_id) {
                Sprite {
                    image: texture,
                    custom_size: Some(Vec2::splat(size)),
                    flip_y: true, // Face downward
                    ..default()
                }
            } else {
                Sprite {
                    color: Color::srgb(0.9, 0.7, 0.3), // Amarr gold
                    custom_size: Some(Vec2::splat(size)),
                    ..default()
                }
            }
        } else {
            Sprite {
                color: Color::srgb(0.9, 0.7, 0.3),
                custom_size: Some(Vec2::splat(size)),
                ..default()
            }
        }
    } else {
        // Station/infrastructure - red square
        Sprite {
            color: Color::srgb(0.8, 0.2, 0.2),
            custom_size: Some(Vec2::splat(size)),
            ..default()
        }
    };

    commands.spawn(BossBundle {
        boss: Boss,
        data: boss_data,
        state: BossState::Intro,
        movement,
        attack: BossAttack::default(),
        hitbox: Hitbox {
            radius: size / 2.0 * 0.8,
        },
        sprite,
        transform: Transform::from_xyz(0.0, start_y, LAYER_ENEMIES),
    });

    true
}

/// Get phase health threshold
pub fn get_phase_threshold(phase: u32, total_phases: u32) -> f32 {
    match (phase, total_phases) {
        (1, _) => 1.0,
        (2, 2) => 0.4,
        (2, 3) => 0.6,
        (2, 4) => 0.7,
        (2, 5) => 0.75,
        (3, 3) => 0.3,
        (3, 4) => 0.4,
        (3, 5) => 0.5,
        (4, 4) => 0.15,
        (4, 5) => 0.25,
        (5, 5) => 0.05,
        _ => 0.0,
    }
}
