//! Drone Entity
//!
//! Combat drones spawned by ability activation.
//! - DeployDrone (Amarr): Single combat drone
//! - DroneBay (Gallente): 2 autonomous fighters

#![allow(dead_code)]

use super::{Enemy, PlayerProjectile, ProjectileDamage, ProjectilePhysics};
use crate::core::*;
use crate::systems::ability::{AbilityActivatedEvent, AbilityType};
use bevy::prelude::*;

/// Marker for ability-spawned drones
#[derive(Component, Debug)]
pub struct Drone;

/// Drone stats and state
#[derive(Component, Debug, Clone)]
pub struct DroneStats {
    /// Current health
    pub health: f32,
    /// Maximum health
    pub max_health: f32,
    /// Movement speed
    pub speed: f32,
    /// Orbit distance from player
    pub orbit_distance: f32,
    /// Current orbit angle
    pub orbit_angle: f32,
    /// Orbit angular speed (radians per second)
    pub orbit_speed: f32,
    /// Remaining lifetime
    pub lifetime: f32,
}

impl Default for DroneStats {
    fn default() -> Self {
        Self {
            health: 30.0,
            max_health: 30.0,
            speed: 400.0,
            orbit_distance: 60.0,
            orbit_angle: 0.0,
            orbit_speed: 2.0,
            lifetime: 15.0,
        }
    }
}

/// Drone weapon for auto-attacking
#[derive(Component, Debug)]
pub struct DroneWeapon {
    pub fire_rate: f32,
    pub cooldown: f32,
    pub damage: f32,
    pub range: f32,
}

impl Default for DroneWeapon {
    fn default() -> Self {
        Self {
            fire_rate: 3.0, // Shots per second
            cooldown: 0.0,
            damage: 12.0,
            range: 300.0,
        }
    }
}

/// Drone faction for visual styling
#[derive(Component, Debug, Clone, Copy, PartialEq, Eq)]
pub enum DroneFaction {
    Amarr,   // Gold/yellow drones
    Gallente, // Green drones
}

/// Drone plugin
pub struct DronePlugin;

impl Plugin for DronePlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(
            Update,
            (
                spawn_drones_on_ability,
                drone_orbit_player,
                drone_target_and_shoot,
                drone_take_damage,
                drone_lifetime_despawn,
            )
                .run_if(in_state(GameState::Playing)),
        );
    }
}

/// Spawn drones when DeployDrone or DroneBay ability activates
fn spawn_drones_on_ability(
    mut commands: Commands,
    mut ability_events: EventReader<AbilityActivatedEvent>,
    player_query: Query<&Transform, With<super::Player>>,
) {
    for event in ability_events.read() {
        let Ok(player_transform) = player_query.get_single() else {
            continue;
        };
        let player_pos = player_transform.translation.truncate();

        match event.ability_type {
            AbilityType::DeployDrone => {
                // Single Amarr combat drone
                spawn_drone(
                    &mut commands,
                    player_pos,
                    0.0, // Initial orbit angle
                    DroneFaction::Amarr,
                    15.0, // Duration from ability
                );
                info!("Deployed Amarr combat drone");
            }
            AbilityType::DroneBay => {
                // Two Gallente autonomous fighters
                spawn_drone(
                    &mut commands,
                    player_pos,
                    0.0,
                    DroneFaction::Gallente,
                    20.0,
                );
                spawn_drone(
                    &mut commands,
                    player_pos,
                    std::f32::consts::PI, // Opposite side
                    DroneFaction::Gallente,
                    20.0,
                );
                info!("Deployed 2 Gallente autonomous fighters");
            }
            _ => {}
        }
    }
}

/// Spawn a single drone
fn spawn_drone(
    commands: &mut Commands,
    player_pos: Vec2,
    orbit_angle: f32,
    faction: DroneFaction,
    lifetime: f32,
) -> Entity {
    let orbit_distance = 60.0;
    let spawn_pos = Vec2::new(
        player_pos.x + orbit_angle.cos() * orbit_distance,
        player_pos.y + orbit_angle.sin() * orbit_distance,
    );

    // Faction-specific stats and visuals
    let (color, health, damage, fire_rate) = match faction {
        DroneFaction::Amarr => (
            Color::srgb(1.0, 0.85, 0.3), // Gold
            40.0,                         // Tankier single drone
            15.0,                         // Higher damage
            2.5,                          // Slower fire rate
        ),
        DroneFaction::Gallente => (
            Color::srgb(0.4, 0.9, 0.4), // Green
            25.0,                        // Lighter fighters
            10.0,                        // Lower damage per drone
            4.0,                         // Faster fire rate
        ),
    };

    commands
        .spawn((
            Drone,
            DroneStats {
                health,
                max_health: health,
                orbit_angle,
                lifetime,
                ..default()
            },
            DroneWeapon {
                damage,
                fire_rate,
                ..default()
            },
            faction,
            Sprite {
                color,
                custom_size: Some(Vec2::new(16.0, 16.0)),
                ..default()
            },
            Transform::from_xyz(spawn_pos.x, spawn_pos.y, LAYER_PLAYER),
        ))
        .id()
}

/// Drones orbit around the player
fn drone_orbit_player(
    time: Res<Time>,
    player_query: Query<&Transform, With<super::Player>>,
    mut drone_query: Query<(&mut Transform, &mut DroneStats), (With<Drone>, Without<super::Player>)>,
) {
    let Ok(player_transform) = player_query.get_single() else {
        return;
    };
    let player_pos = player_transform.translation.truncate();
    let dt = time.delta_secs();

    for (mut transform, mut stats) in drone_query.iter_mut() {
        // Update orbit angle
        stats.orbit_angle += stats.orbit_speed * dt;
        if stats.orbit_angle > std::f32::consts::TAU {
            stats.orbit_angle -= std::f32::consts::TAU;
        }

        // Calculate target position on orbit
        let target_pos = Vec2::new(
            player_pos.x + stats.orbit_angle.cos() * stats.orbit_distance,
            player_pos.y + stats.orbit_angle.sin() * stats.orbit_distance,
        );

        // Smooth movement toward orbit position
        let current_pos = transform.translation.truncate();
        let delta = target_pos - current_pos;

        if delta.length() > 1.0 {
            let move_speed = stats.speed.min(delta.length() * 5.0);
            let move_dir = delta.normalize();
            transform.translation.x += move_dir.x * move_speed * dt;
            transform.translation.y += move_dir.y * move_speed * dt;
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

/// Drones target nearest enemy and shoot
fn drone_target_and_shoot(
    mut commands: Commands,
    time: Res<Time>,
    enemy_query: Query<&Transform, With<Enemy>>,
    mut drone_query: Query<(&Transform, &mut DroneWeapon, &DroneFaction), With<Drone>>,
) {
    let dt = time.delta_secs();

    for (drone_transform, mut weapon, faction) in drone_query.iter_mut() {
        weapon.cooldown -= dt;

        if weapon.cooldown > 0.0 {
            continue;
        }

        let drone_pos = drone_transform.translation.truncate();

        // Find nearest enemy in range
        let mut nearest_enemy: Option<Vec2> = None;
        let mut nearest_dist = weapon.range;

        for enemy_transform in enemy_query.iter() {
            let enemy_pos = enemy_transform.translation.truncate();
            let dist = (enemy_pos - drone_pos).length();

            if dist < nearest_dist {
                nearest_dist = dist;
                nearest_enemy = Some(enemy_pos);
            }
        }

        // Fire at target if found
        if let Some(target_pos) = nearest_enemy {
            weapon.cooldown = 1.0 / weapon.fire_rate;

            let direction = (target_pos - drone_pos).normalize_or_zero();
            let velocity = direction * PLAYER_BULLET_SPEED * 0.85;
            let angle = direction.y.atan2(direction.x) - std::f32::consts::FRAC_PI_2;

            // Projectile color based on faction
            let bullet_color = match faction {
                DroneFaction::Amarr => Color::srgb(1.0, 0.9, 0.4),   // Gold laser
                DroneFaction::Gallente => Color::srgb(0.5, 1.0, 0.5), // Green plasma
            };

            let damage_type = match faction {
                DroneFaction::Amarr => DamageType::EM,
                DroneFaction::Gallente => DamageType::Thermal,
            };

            commands.spawn((
                PlayerProjectile,
                ProjectilePhysics {
                    velocity,
                    lifetime: 1.5,
                },
                ProjectileDamage {
                    damage: weapon.damage,
                    damage_type,
                    crit_chance: 0.08,
                    crit_multiplier: 1.4,
                },
                Sprite {
                    color: bullet_color,
                    custom_size: Some(Vec2::new(4.0, 8.0)),
                    ..default()
                },
                Transform::from_xyz(drone_pos.x, drone_pos.y, LAYER_PLAYER_BULLETS)
                    .with_rotation(Quat::from_rotation_z(angle)),
            ));
        }
    }
}

/// Drones take damage from enemy projectiles
fn drone_take_damage(
    mut commands: Commands,
    projectile_query: Query<(Entity, &Transform, &ProjectileDamage), With<super::EnemyProjectile>>,
    mut drone_query: Query<(Entity, &Transform, &mut DroneStats), With<Drone>>,
) {
    for (proj_entity, proj_transform, proj_damage) in projectile_query.iter() {
        let proj_pos = proj_transform.translation.truncate();

        for (drone_entity, drone_transform, mut stats) in drone_query.iter_mut() {
            let drone_pos = drone_transform.translation.truncate();
            let distance = (proj_pos - drone_pos).length();

            if distance < 12.0 {
                // Despawn projectile
                commands.entity(proj_entity).despawn_recursive();

                // Apply damage
                stats.health -= proj_damage.damage;

                if stats.health <= 0.0 {
                    commands.entity(drone_entity).despawn_recursive();
                    info!("Drone destroyed!");
                }

                break;
            }
        }
    }
}

/// Despawn drones when lifetime expires
fn drone_lifetime_despawn(
    mut commands: Commands,
    time: Res<Time>,
    mut drone_query: Query<(Entity, &mut DroneStats), With<Drone>>,
) {
    let dt = time.delta_secs();

    for (entity, mut stats) in drone_query.iter_mut() {
        stats.lifetime -= dt;

        if stats.lifetime <= 0.0 {
            commands.entity(entity).despawn_recursive();
            info!("Drone lifetime expired");
        }
    }
}
