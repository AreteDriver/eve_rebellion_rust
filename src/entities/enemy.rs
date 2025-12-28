//! Enemy Entities
//!
//! All enemy ship types, AI behaviors, and wave spawning.

use bevy::prelude::*;
use crate::core::*;
use crate::assets::{ShipModelCache, ShipModelRotation, get_model_scale};

/// Marker component for enemy entities
#[derive(Component, Debug)]
pub struct Enemy;

/// Enemy AI behavior type
#[derive(Component, Debug, Clone, Copy, PartialEq, Eq)]
pub enum EnemyBehavior {
    /// Moves straight down
    Linear,
    /// Weaves side to side
    Zigzag,
    /// Moves toward player
    Homing,
    /// Circles around a point
    Orbital,
    /// Stays at distance, strafes
    Sniper,
    /// Rushes toward player at high speed
    Kamikaze,
}

/// Enemy stats
#[derive(Component, Debug, Clone)]
pub struct EnemyStats {
    /// EVE type ID
    pub type_id: u32,
    /// Display name
    pub name: String,
    /// Current HP
    pub health: f32,
    /// Maximum HP
    pub max_health: f32,
    /// Movement speed
    pub speed: f32,
    /// Score value when destroyed
    pub score_value: u64,
    /// Is this a boss?
    pub is_boss: bool,
}

impl Default for EnemyStats {
    fn default() -> Self {
        Self {
            type_id: 597, // Punisher
            name: "Punisher".into(),
            health: 30.0,
            max_health: 30.0,
            speed: ENEMY_BASE_SPEED,
            score_value: POINTS_PER_KILL,
            is_boss: false,
        }
    }
}

/// Enemy weapon
#[derive(Component, Debug, Clone)]
pub struct EnemyWeapon {
    /// Weapon type (determines projectile visuals and damage type)
    pub weapon_type: WeaponType,
    /// Fire rate
    pub fire_rate: f32,
    /// Cooldown timer
    pub cooldown: f32,
    /// Bullet speed
    pub bullet_speed: f32,
    /// Damage per hit
    pub damage: f32,
    /// Firing pattern
    pub pattern: FiringPattern,
}

/// Enemy firing patterns
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum FiringPattern {
    /// Single shot at player
    Single,
    /// 3-shot spread
    Spread3,
    /// 5-shot spread
    Spread5,
    /// Circular burst
    Circle,
    /// Aimed stream
    Stream,
}

impl Default for EnemyWeapon {
    fn default() -> Self {
        Self {
            weapon_type: WeaponType::Laser, // Default Amarr
            fire_rate: 1.0,
            cooldown: 1.0,
            bullet_speed: ENEMY_BULLET_SPEED,
            damage: 10.0,
            pattern: FiringPattern::Single,
        }
    }
}

/// AI state for behavior logic
#[derive(Component, Debug, Clone)]
pub struct EnemyAI {
    /// Current behavior
    pub behavior: EnemyBehavior,
    /// Timer for behavior patterns
    pub timer: f32,
    /// Phase for oscillating patterns
    pub phase: f32,
    /// Target position (for some behaviors)
    pub target: Vec2,
    /// Whether currently active (on screen)
    pub active: bool,
}

impl Default for EnemyAI {
    fn default() -> Self {
        Self {
            behavior: EnemyBehavior::Linear,
            timer: 0.0,
            phase: 0.0,
            target: Vec2::ZERO,
            active: true,
        }
    }
}

/// Bundle for spawning an enemy
#[derive(Bundle)]
pub struct EnemyBundle {
    pub enemy: Enemy,
    pub stats: EnemyStats,
    pub weapon: EnemyWeapon,
    pub ai: EnemyAI,
    pub sprite: Sprite,
    pub transform: Transform,
}

impl Default for EnemyBundle {
    fn default() -> Self {
        Self {
            enemy: Enemy,
            stats: EnemyStats::default(),
            weapon: EnemyWeapon::default(),
            ai: EnemyAI::default(),
            sprite: Sprite {
                color: COLOR_AMARR,
                custom_size: Some(Vec2::splat(40.0)),
                ..default()
            },
            transform: Transform::from_xyz(0.0, 300.0, LAYER_ENEMIES),
        }
    }
}

/// Enemy plugin
pub struct EnemyPlugin;

impl Plugin for EnemyPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(
            Update,
            (
                enemy_movement,
                update_enemy_ship_rotation,
                enemy_shooting,
                enemy_bounds_check,
            ).run_if(in_state(GameState::Playing)),
        );
    }
}

/// Enemy movement based on AI behavior
fn enemy_movement(
    time: Res<Time>,
    player_query: Query<&Transform, With<super::Player>>,
    mut query: Query<(&mut Transform, &EnemyStats, &mut EnemyAI), (With<Enemy>, Without<super::Player>)>,
) {
    let dt = time.delta_secs();
    let player_pos = player_query
        .get_single()
        .map(|t| t.translation.truncate())
        .unwrap_or(Vec2::ZERO);

    for (mut transform, stats, mut ai) in query.iter_mut() {
        ai.timer += dt;
        let pos = transform.translation.truncate();

        let velocity = match ai.behavior {
            EnemyBehavior::Linear => {
                Vec2::new(0.0, -1.0) * stats.speed
            }
            EnemyBehavior::Zigzag => {
                let x = (ai.timer * 3.0 + ai.phase).sin() * stats.speed;
                Vec2::new(x, -stats.speed * 0.5)
            }
            EnemyBehavior::Homing => {
                let dir = (player_pos - pos).normalize_or_zero();
                dir * stats.speed
            }
            EnemyBehavior::Orbital => {
                let angle = ai.timer * 2.0 + ai.phase;
                let orbit_center = Vec2::new(0.0, 100.0);
                let target = orbit_center + Vec2::new(angle.cos(), angle.sin()) * 150.0;
                (target - pos).normalize_or_zero() * stats.speed
            }
            EnemyBehavior::Sniper => {
                // Stay at top, strafe
                let target_y = SCREEN_HEIGHT / 2.0 - 100.0;
                let y_diff = target_y - pos.y;
                let x = (ai.timer * 1.5 + ai.phase).sin() * stats.speed;
                Vec2::new(x, y_diff.signum() * stats.speed.min(y_diff.abs()))
            }
            EnemyBehavior::Kamikaze => {
                let dir = (player_pos - pos).normalize_or_zero();
                dir * stats.speed * 2.0
            }
        };

        transform.translation.x += velocity.x * dt;
        transform.translation.y += velocity.y * dt;

        // Slight tilt based on horizontal movement (visual effect only)
        let tilt = (velocity.x / stats.speed).clamp(-1.0, 1.0) * 0.2;
        transform.rotation = Quat::from_rotation_z(tilt);
    }
}

/// Enemy shooting system
fn enemy_shooting(
    mut commands: Commands,
    time: Res<Time>,
    player_query: Query<&Transform, With<super::Player>>,
    mut query: Query<(&Transform, &mut EnemyWeapon, &EnemyAI), With<Enemy>>,
) {
    let dt = time.delta_secs();
    let player_pos = player_query
        .get_single()
        .map(|t| t.translation.truncate())
        .unwrap_or(Vec2::ZERO);

    for (transform, mut weapon, ai) in query.iter_mut() {
        if !ai.active {
            continue;
        }

        weapon.cooldown -= dt;
        if weapon.cooldown <= 0.0 {
            weapon.cooldown = 1.0 / weapon.fire_rate;

            let pos = transform.translation.truncate();
            let dir = (player_pos - pos).normalize_or_zero();

            // Spawn enemy projectile with correct weapon type
            super::projectile::spawn_enemy_projectile_typed(
                &mut commands,
                pos,
                dir,
                weapon.damage,
                weapon.bullet_speed,
                weapon.weapon_type,
            );
        }
    }
}

/// Remove enemies that go off screen
fn enemy_bounds_check(
    mut commands: Commands,
    query: Query<(Entity, &Transform), With<Enemy>>,
) {
    let margin = 100.0;
    for (entity, transform) in query.iter() {
        let pos = transform.translation;
        if pos.y < -SCREEN_HEIGHT / 2.0 - margin
            || pos.y > SCREEN_HEIGHT / 2.0 + margin
            || pos.x.abs() > SCREEN_WIDTH / 2.0 + margin
        {
            commands.entity(entity).despawn_recursive();
        }
    }
}

/// Update 3D enemy rotation based on movement (banking/tilting)
fn update_enemy_ship_rotation(
    time: Res<Time>,
    mut query: Query<(&EnemyStats, &EnemyAI, &mut Transform, &ShipModelRotation), With<Enemy>>,
) {
    let dt = time.delta_secs();

    for (stats, ai, mut transform, model_rot) in query.iter_mut() {
        // Estimate velocity from AI behavior
        let velocity = match ai.behavior {
            EnemyBehavior::Linear => Vec2::new(0.0, -stats.speed),
            EnemyBehavior::Zigzag => {
                let x = (ai.timer * 3.0 + ai.phase).sin() * stats.speed;
                Vec2::new(x, -stats.speed * 0.5)
            }
            EnemyBehavior::Homing | EnemyBehavior::Kamikaze => {
                // These move toward player, estimate based on target
                let dir = (ai.target - transform.translation.truncate()).normalize_or_zero();
                dir * stats.speed
            }
            EnemyBehavior::Orbital => {
                let angle = ai.timer * 2.0 + ai.phase;
                Vec2::new(-angle.sin(), angle.cos()) * stats.speed * 0.5
            }
            EnemyBehavior::Sniper => {
                let x = (ai.timer * 1.5 + ai.phase).sin() * stats.speed;
                Vec2::new(x, 0.0)
            }
        };

        let target_rotation = model_rot.calculate_rotation(velocity, stats.speed);
        transform.rotation = transform.rotation.slerp(
            target_rotation,
            (model_rot.smoothing * dt).min(1.0),
        );
    }
}

/// Get faction color for enemy type
fn get_enemy_color(type_id: u32) -> Color {
    match type_id {
        // Amarr - Gold
        597 | 589 | 591 => COLOR_AMARR,
        // Caldari - Steel Blue
        603 | 602 => COLOR_CALDARI,
        // Gallente - Green
        593 | 594 => COLOR_GALLENTE,
        _ => Color::srgb(0.5, 0.5, 0.5),
    }
}

/// Get weapon type for faction based on type_id
fn get_faction_weapon(type_id: u32) -> WeaponType {
    match type_id {
        // Amarr - Lasers (EM damage)
        597 | 589 | 591 => WeaponType::Laser,
        // Caldari - Railguns/Missiles (Kinetic/Explosive)
        603 => WeaponType::Railgun,
        602 => WeaponType::MissileLauncher,
        // Gallente - Drones/Blasters (Thermal)
        593 | 594 => WeaponType::Drone,
        // Minmatar (if any enemy) - Autocannons
        587 | 585 | 586 => WeaponType::Autocannon,
        _ => WeaponType::Laser,
    }
}

/// Spawn a single enemy with 3D model, EVE sprite, or fallback color
pub fn spawn_enemy(
    commands: &mut Commands,
    type_id: u32,
    position: Vec2,
    behavior: EnemyBehavior,
    sprite: Option<Handle<Image>>,
    model_cache: Option<&ShipModelCache>,
) -> Entity {
    let (name, health, speed, score) = match type_id {
        // Amarr
        597 => ("Punisher", 40.0, 80.0, 100),
        589 => ("Executioner", 25.0, 120.0, 80),
        591 => ("Tormentor", 35.0, 90.0, 90),
        // Caldari
        603 => ("Merlin", 45.0, 70.0, 100),
        602 => ("Kestrel", 30.0, 100.0, 90),
        // Gallente
        593 => ("Tristan", 35.0, 90.0, 100),
        594 => ("Incursus", 40.0, 85.0, 95),
        _ => ("Unknown", 30.0, 100.0, 50),
    };

    let base_color = get_enemy_color(type_id);
    let weapon_type = get_faction_weapon(type_id);

    // Configure weapon based on faction
    let weapon = EnemyWeapon {
        weapon_type,
        fire_rate: match weapon_type {
            WeaponType::Laser => 0.8,           // Amarr: Slower, harder hitting
            WeaponType::Railgun => 0.6,         // Caldari: Slow but powerful
            WeaponType::MissileLauncher => 0.5, // Caldari missiles: Slowest
            WeaponType::Drone => 1.2,           // Gallente: Fast drones
            WeaponType::Autocannon => 1.5,      // Minmatar: Fastest
            _ => 1.0,
        },
        damage: match weapon_type {
            WeaponType::Laser => 12.0,
            WeaponType::Railgun => 18.0,
            WeaponType::MissileLauncher => 20.0,
            WeaponType::Drone => 8.0,
            WeaponType::Autocannon => 10.0,
            _ => 10.0,
        },
        bullet_speed: match weapon_type {
            WeaponType::Laser => 280.0,         // Fast beams
            WeaponType::Railgun => 350.0,       // Fastest projectiles
            WeaponType::MissileLauncher => 180.0, // Slow missiles
            WeaponType::Drone => 200.0,         // Medium
            WeaponType::Autocannon => 250.0,    // Fast bullets
            _ => 200.0,
        },
        cooldown: 0.5 + fastrand::f32() * 1.0, // Random initial delay
        pattern: FiringPattern::Single,
    };

    let stats = EnemyStats {
        type_id,
        name: name.into(),
        health,
        max_health: health,
        speed,
        score_value: score,
        is_boss: false,
    };

    let ai = EnemyAI {
        behavior,
        phase: fastrand::f32() * std::f32::consts::TAU,
        ..default()
    };

    // Try 3D model first, then sprite, then color fallback
    if let Some(cache) = model_cache {
        if let Some(scene_handle) = cache.get(type_id) {
            let model_rot = ShipModelRotation::new_enemy();
            let scale = get_model_scale(type_id) * 48.0; // Scale to match sprite size

            return commands.spawn((
                Enemy,
                stats,
                weapon,
                ai,
                model_rot.clone(),
                SceneRoot(scene_handle),
                Transform::from_xyz(position.x, position.y, 0.0)
                    .with_scale(Vec3::splat(scale))
                    .with_rotation(model_rot.base_rotation),
            )).id();
        }
    }

    // Fallback to sprite
    if let Some(texture) = sprite {
        commands.spawn((
            Enemy,
            stats,
            weapon,
            ai,
            Sprite {
                image: texture,
                custom_size: Some(Vec2::new(48.0, 48.0)),
                flip_y: true, // Flip to face downward
                ..default()
            },
            Transform::from_xyz(position.x, position.y, LAYER_ENEMIES),
        )).id()
    } else {
        // Color fallback
        commands.spawn((
            Enemy,
            stats,
            weapon,
            ai,
            Sprite {
                color: base_color,
                custom_size: Some(Vec2::new(40.0, 48.0)),
                ..default()
            },
            Transform::from_xyz(position.x, position.y, LAYER_ENEMIES),
        )).id()
    }
}
