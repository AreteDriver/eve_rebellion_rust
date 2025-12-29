//! Wingman Entity
//!
//! Allied Rifter frigates that assist the player during combat.
//! Rifter special ability: spawn wingman every 15 kills.

#![allow(dead_code)]

use super::{Player, PlayerProjectile, ProjectileDamage, ProjectilePhysics};
use crate::assets::{get_model_scale, ShipModelCache, ShipModelRotation};
use crate::core::*;
use bevy::prelude::*;

/// Marker for wingman entities
#[derive(Component, Debug)]
pub struct Wingman;

/// Wingman stats
#[derive(Component, Debug, Clone)]
pub struct WingmanStats {
    /// Current health
    pub health: f32,
    /// Maximum health
    pub max_health: f32,
    /// Offset from player position
    pub offset_x: f32,
    /// Movement speed
    pub speed: f32,
}

impl Default for WingmanStats {
    fn default() -> Self {
        Self {
            health: 50.0,
            max_health: 50.0,
            offset_x: 0.0,
            speed: 320.0,
        }
    }
}

/// Wingman weapon
#[derive(Component, Debug)]
pub struct WingmanWeapon {
    pub fire_rate: f32,
    pub cooldown: f32,
    pub damage: f32,
}

impl Default for WingmanWeapon {
    fn default() -> Self {
        Self {
            fire_rate: 2.5, // Shots per second
            cooldown: 0.0,
            damage: 8.0,
        }
    }
}

/// Resource to track kills for wingman spawning
#[derive(Resource, Default)]
pub struct WingmanTracker {
    /// Kill counter (resets when wingman spawns)
    pub kill_count: u32,
    /// Kills required per wingman
    pub kills_per_wingman: u32,
    /// Maximum wingmen active at once
    pub max_wingmen: u32,
}

impl WingmanTracker {
    pub fn new() -> Self {
        Self {
            kill_count: 0,
            kills_per_wingman: 15,
            max_wingmen: 4,
        }
    }

    /// Returns progress toward next wingman (0.0 to 1.0)
    pub fn progress(&self) -> f32 {
        self.kill_count as f32 / self.kills_per_wingman as f32
    }
}

/// Wingman plugin
pub struct WingmanPlugin;

impl Plugin for WingmanPlugin {
    fn build(&self, app: &mut App) {
        app.insert_resource(WingmanTracker::new()).add_systems(
            Update,
            (
                track_kills_for_wingman,
                wingman_follow_player,
                wingman_shooting,
                wingman_damage,
            )
                .run_if(in_state(GameState::Playing)),
        );
    }
}

/// Track enemy kills and spawn wingmen for Rifter
fn track_kills_for_wingman(
    mut commands: Commands,
    mut tracker: ResMut<WingmanTracker>,
    mut destroy_events: EventReader<EnemyDestroyedEvent>,
    selected_ship: Res<SelectedShip>,
    player_query: Query<&Transform, With<Player>>,
    wingmen_query: Query<&WingmanStats, With<Wingman>>,
    sprite_cache: Res<crate::assets::ShipSpriteCache>,
    model_cache: Res<ShipModelCache>,
) {
    // Only Rifter gets wingmen
    if selected_ship.ship != MinmatarShip::Rifter {
        return;
    }

    // Count kills
    for _event in destroy_events.read() {
        tracker.kill_count += 1;

        // Check if we should spawn a wingman
        if tracker.kill_count >= tracker.kills_per_wingman {
            tracker.kill_count = 0;

            // Check if we're at max wingmen
            let current_wingmen = wingmen_query.iter().count() as u32;
            if current_wingmen >= tracker.max_wingmen {
                continue;
            }

            // Find player position
            let Ok(player_transform) = player_query.get_single() else {
                continue;
            };
            let player_pos = player_transform.translation.truncate();

            // Calculate offset to avoid stacking
            let existing_offsets: Vec<f32> = wingmen_query.iter().map(|w| w.offset_x).collect();
            let possible_offsets = [-80.0, -50.0, 50.0, 80.0, -110.0, 110.0];

            let mut offset_x = 60.0;
            for &offset in &possible_offsets {
                if !existing_offsets.iter().any(|&x| (x - offset).abs() < 20.0) {
                    offset_x = offset;
                    break;
                }
            }

            spawn_wingman(
                &mut commands,
                player_pos,
                offset_x,
                Some(&sprite_cache),
                Some(&model_cache),
            );
            info!("Wingman spawned! (offset: {})", offset_x);
        }
    }
}

/// Spawn a wingman
pub fn spawn_wingman(
    commands: &mut Commands,
    player_pos: Vec2,
    offset_x: f32,
    sprite_cache: Option<&crate::assets::ShipSpriteCache>,
    model_cache: Option<&ShipModelCache>,
) -> Entity {
    let spawn_pos = Vec2::new(player_pos.x + offset_x, player_pos.y + 40.0);
    let rifter_type_id: u32 = 587;

    // Try 3D model first
    if let Some(cache) = model_cache {
        if let Some(scene_handle) = cache.get(rifter_type_id) {
            let model_rot = ShipModelRotation::new_player();
            let scale = get_model_scale(rifter_type_id) * 40.0; // Slightly smaller than player

            return commands
                .spawn((
                    Wingman,
                    WingmanStats {
                        offset_x,
                        ..default()
                    },
                    WingmanWeapon::default(),
                    model_rot.clone(),
                    SceneRoot(scene_handle),
                    Transform::from_xyz(spawn_pos.x, spawn_pos.y, 0.0)
                        .with_scale(Vec3::splat(scale))
                        .with_rotation(model_rot.base_rotation),
                ))
                .id();
        }
    }

    // Fallback to sprite
    let sprite = if let Some(cache) = sprite_cache {
        if let Some(texture) = cache.get(rifter_type_id) {
            Sprite {
                image: texture,
                custom_size: Some(Vec2::new(35.0, 44.0)),
                ..default()
            }
        } else {
            Sprite {
                color: COLOR_MINMATAR,
                custom_size: Some(Vec2::new(30.0, 38.0)),
                ..default()
            }
        }
    } else {
        Sprite {
            color: COLOR_MINMATAR,
            custom_size: Some(Vec2::new(30.0, 38.0)),
            ..default()
        }
    };

    commands
        .spawn((
            Wingman,
            WingmanStats {
                offset_x,
                ..default()
            },
            WingmanWeapon::default(),
            sprite,
            Transform::from_xyz(spawn_pos.x, spawn_pos.y, LAYER_PLAYER),
        ))
        .id()
}

/// Wingmen follow the player
fn wingman_follow_player(
    time: Res<Time>,
    player_query: Query<&Transform, With<Player>>,
    mut wingmen_query: Query<(&mut Transform, &WingmanStats), (With<Wingman>, Without<Player>)>,
) {
    let Ok(player_transform) = player_query.get_single() else {
        return;
    };

    let player_pos = player_transform.translation.truncate();
    let dt = time.delta_secs();

    for (mut transform, stats) in wingmen_query.iter_mut() {
        let target_x = player_pos.x + stats.offset_x;
        let target_y = player_pos.y + 40.0; // Slightly behind

        let current_pos = transform.translation.truncate();
        let delta = Vec2::new(target_x, target_y) - current_pos;

        // Smooth movement toward target
        if delta.length() > 2.0 {
            let move_dir = delta.normalize();
            transform.translation.x += move_dir.x * stats.speed * dt;
            transform.translation.y += move_dir.y * stats.speed * dt;
        }

        // Clamp to screen
        transform.translation.x = transform
            .translation
            .x
            .clamp(-SCREEN_WIDTH / 2.0 + 20.0, SCREEN_WIDTH / 2.0 - 20.0);
        transform.translation.y = transform
            .translation
            .y
            .clamp(-SCREEN_HEIGHT / 2.0 + 20.0, SCREEN_HEIGHT / 2.0 - 20.0);
    }
}

/// Wingmen shoot at enemies
fn wingman_shooting(
    mut commands: Commands,
    time: Res<Time>,
    mut wingmen_query: Query<(&Transform, &mut WingmanWeapon), With<Wingman>>,
) {
    let dt = time.delta_secs();

    for (transform, mut weapon) in wingmen_query.iter_mut() {
        weapon.cooldown -= dt;

        if weapon.cooldown <= 0.0 {
            weapon.cooldown = 1.0 / weapon.fire_rate;

            let pos = transform.translation.truncate();

            // Fire straight up
            let velocity = Vec2::Y * PLAYER_BULLET_SPEED * 0.9;

            commands.spawn((
                PlayerProjectile,
                ProjectilePhysics {
                    velocity,
                    lifetime: 1.5,
                },
                ProjectileDamage {
                    damage: weapon.damage,
                    damage_type: DamageType::Kinetic,
                },
                Sprite {
                    color: Color::srgb(0.8, 0.5, 0.3), // Rust-colored bullets
                    custom_size: Some(Vec2::new(3.0, 10.0)),
                    ..default()
                },
                Transform::from_xyz(pos.x, pos.y + 20.0, LAYER_PLAYER_BULLETS),
            ));
        }
    }
}

/// Wingmen take damage from enemy projectiles
fn wingman_damage(
    mut commands: Commands,
    projectile_query: Query<(Entity, &Transform, &ProjectileDamage), With<super::EnemyProjectile>>,
    mut wingmen_query: Query<(Entity, &Transform, &mut WingmanStats), With<Wingman>>,
) {
    for (proj_entity, proj_transform, proj_damage) in projectile_query.iter() {
        let proj_pos = proj_transform.translation.truncate();

        for (wingman_entity, wingman_transform, mut stats) in wingmen_query.iter_mut() {
            let wingman_pos = wingman_transform.translation.truncate();
            let distance = (proj_pos - wingman_pos).length();

            if distance < 20.0 {
                // Despawn projectile
                commands.entity(proj_entity).despawn_recursive();

                // Apply damage
                stats.health -= proj_damage.damage;

                if stats.health <= 0.0 {
                    commands.entity(wingman_entity).despawn_recursive();
                    info!("Wingman destroyed!");
                }

                break;
            }
        }
    }
}
