//! Projectile Entities
//!
//! Player bullets, enemy bullets, missiles, drones.

#![allow(dead_code)]

use crate::core::*;
use crate::systems::effects::BulletTrail;
use bevy::prelude::*;

/// Marker for player projectiles
#[derive(Component, Debug)]
pub struct PlayerProjectile;

/// Marker for enemy projectiles
#[derive(Component, Debug)]
pub struct EnemyProjectile;

/// Seeking/homing projectile - tracks nearest enemy
#[derive(Component, Debug)]
pub struct SeekingProjectile {
    /// Turn rate in radians per second
    pub turn_rate: f32,
    /// Maximum range to acquire target
    pub acquire_range: f32,
}

/// Projectile physics
#[derive(Component, Debug, Clone)]
pub struct ProjectilePhysics {
    /// Current velocity
    pub velocity: Vec2,
    /// Lifetime remaining
    pub lifetime: f32,
}

/// Projectile damage info
#[derive(Component, Debug, Clone)]
pub struct ProjectileDamage {
    /// Damage amount
    pub damage: f32,
    /// Damage type
    pub damage_type: DamageType,
    /// Critical hit chance (0.0 - 1.0)
    pub crit_chance: f32,
    /// Critical hit damage multiplier
    pub crit_multiplier: f32,
}

impl Default for ProjectileDamage {
    fn default() -> Self {
        Self {
            damage: 10.0,
            damage_type: DamageType::Kinetic,
            crit_chance: 0.1,     // 10% base crit chance
            crit_multiplier: 1.5, // 1.5x crit damage
        }
    }
}

/// Bundle for player projectile
#[derive(Bundle)]
pub struct PlayerProjectileBundle {
    pub marker: PlayerProjectile,
    pub physics: ProjectilePhysics,
    pub damage: ProjectileDamage,
    pub sprite: Sprite,
    pub transform: Transform,
}

impl Default for PlayerProjectileBundle {
    fn default() -> Self {
        Self {
            marker: PlayerProjectile,
            physics: ProjectilePhysics {
                velocity: Vec2::Y * PLAYER_BULLET_SPEED,
                lifetime: 2.0,
            },
            damage: ProjectileDamage {
                damage: PLAYER_BULLET_DAMAGE,
                damage_type: DamageType::Kinetic,
                crit_chance: 0.1, // 10% crit for autocannons
                crit_multiplier: 1.5,
            },
            sprite: Sprite {
                color: Color::srgb(1.0, 0.9, 0.3),
                custom_size: Some(Vec2::new(4.0, 12.0)),
                ..default()
            },
            transform: Transform::from_xyz(0.0, 0.0, LAYER_PLAYER_BULLETS),
        }
    }
}

/// Bundle for enemy projectile
#[derive(Bundle)]
pub struct EnemyProjectileBundle {
    pub marker: EnemyProjectile,
    pub physics: ProjectilePhysics,
    pub damage: ProjectileDamage,
    pub sprite: Sprite,
    pub transform: Transform,
}

impl Default for EnemyProjectileBundle {
    fn default() -> Self {
        Self {
            marker: EnemyProjectile,
            physics: ProjectilePhysics {
                velocity: Vec2::NEG_Y * ENEMY_BULLET_SPEED,
                lifetime: 3.0,
            },
            damage: ProjectileDamage {
                damage: 10.0,
                damage_type: DamageType::EM,
                crit_chance: 0.05,     // 5% crit for enemies (lower)
                crit_multiplier: 1.25, // 1.25x crit for enemies
            },
            sprite: Sprite {
                color: Color::srgb(1.0, 0.3, 0.3),
                custom_size: Some(Vec2::new(6.0, 6.0)),
                ..default()
            },
            transform: Transform::from_xyz(0.0, 0.0, LAYER_ENEMY_BULLETS),
        }
    }
}

/// Projectile plugin
pub struct ProjectilePlugin;

impl Plugin for ProjectilePlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(
            Update,
            (
                spawn_player_projectiles,
                seeking_projectile_update,
                projectile_update,
            )
                .chain()
                .run_if(in_state(GameState::Playing)),
        );
    }
}

/// Spawn player projectiles on fire event
fn spawn_player_projectiles(
    mut commands: Commands,
    mut fire_events: EventReader<PlayerFireEvent>,
    berserk: Res<BerserkSystem>,
) {
    for event in fire_events.read() {
        let damage_mult = berserk.damage_mult();

        // Determine damage type from weapon
        let damage_type = match event.weapon_type {
            WeaponType::Autocannon | WeaponType::Artillery => DamageType::Kinetic,
            WeaponType::Laser => DamageType::EM,
            WeaponType::Railgun => DamageType::Kinetic,
            WeaponType::MissileLauncher => DamageType::Explosive,
            WeaponType::Drone => DamageType::Thermal,
        };

        // Use event's bullet color, or purple if berserk
        let color = if berserk.is_active {
            Color::srgb(1.0, 0.2, 0.8)
        } else {
            event.bullet_color
        };

        // Check if this is a seeking missile (Kestrel/Caldari missile launcher)
        let is_missile = event.weapon_type == WeaponType::MissileLauncher;

        // Calculate projectile spread for burst fire
        let burst_count = event.burst_count.max(1);
        let spread_angle = event.spread_angle;

        for i in 0..burst_count {
            // Calculate direction offset for this projectile
            let angle_offset = if burst_count > 1 {
                // Distribute evenly across spread angle, centered
                let spread_step = spread_angle / (burst_count - 1) as f32;
                -spread_angle / 2.0 + spread_step * i as f32
            } else {
                0.0
            };

            // Rotate direction by angle offset
            let base_angle = event.direction.y.atan2(event.direction.x);
            let proj_angle = base_angle + angle_offset;
            let direction = Vec2::new(proj_angle.cos(), proj_angle.sin());

            // Small position offset for visual spread
            let pos_offset = Vec2::new(
                (i as f32 - (burst_count - 1) as f32 / 2.0) * 5.0,
                0.0,
            );
            let spawn_pos = event.position + pos_offset;

            if is_missile {
                // Seeking missile - larger, slower, homes on enemies, more damage
                let missile_velocity = direction * (PLAYER_BULLET_SPEED * 0.7);
                let missile_damage = event.damage * damage_mult * 1.25;

                commands.spawn((
                    PlayerProjectile,
                    SeekingProjectile {
                        turn_rate: 4.0,
                        acquire_range: 400.0,
                    },
                    ProjectilePhysics {
                        velocity: missile_velocity,
                        lifetime: 3.0,
                    },
                    ProjectileDamage {
                        damage: missile_damage,
                        damage_type,
                        crit_chance: 0.15,
                        crit_multiplier: 1.75,
                    },
                    BulletTrail::new(Color::srgb(1.0, 0.6, 0.2)),
                    Sprite {
                        color,
                        custom_size: Some(Vec2::new(6.0, 14.0)),
                        ..default()
                    },
                    Transform::from_xyz(spawn_pos.x, spawn_pos.y, LAYER_PLAYER_BULLETS),
                ));
            } else {
                // Standard projectile with bullet trail
                let velocity = direction * PLAYER_BULLET_SPEED;

                commands.spawn((
                    PlayerProjectile,
                    ProjectilePhysics {
                        velocity,
                        lifetime: 2.0,
                    },
                    ProjectileDamage {
                        damage: event.damage * damage_mult,
                        damage_type,
                        crit_chance: 0.1,
                        crit_multiplier: 1.5,
                    },
                    BulletTrail::new(color.with_alpha(0.5)),
                    Sprite {
                        color,
                        custom_size: Some(Vec2::new(4.0, 12.0)),
                        ..default()
                    },
                    Transform::from_xyz(spawn_pos.x, spawn_pos.y, LAYER_PLAYER_BULLETS),
                ));
            }
        }
    }
}

/// Seeking projectile homing behavior - finds nearest enemy and turns toward it
fn seeking_projectile_update(
    time: Res<Time>,
    enemy_query: Query<&Transform, With<super::Enemy>>,
    mut seeking_query: Query<
        (&Transform, &mut ProjectilePhysics, &SeekingProjectile),
        With<PlayerProjectile>,
    >,
) {
    let dt = time.delta_secs();

    for (transform, mut physics, seeking) in seeking_query.iter_mut() {
        let missile_pos = transform.translation.truncate();

        // Find nearest enemy within range
        let mut nearest_enemy: Option<Vec2> = None;
        let mut nearest_dist = seeking.acquire_range;

        for enemy_transform in enemy_query.iter() {
            let enemy_pos = enemy_transform.translation.truncate();
            let dist = (enemy_pos - missile_pos).length();

            if dist < nearest_dist {
                nearest_dist = dist;
                nearest_enemy = Some(enemy_pos);
            }
        }

        // If we found a target, turn toward it
        if let Some(target_pos) = nearest_enemy {
            let current_dir = physics.velocity.normalize_or_zero();
            let target_dir = (target_pos - missile_pos).normalize_or_zero();

            // Calculate angle difference
            let current_angle = current_dir.y.atan2(current_dir.x);
            let target_angle = target_dir.y.atan2(target_dir.x);
            let mut angle_diff = target_angle - current_angle;

            // Normalize to -PI..PI
            while angle_diff > std::f32::consts::PI {
                angle_diff -= std::f32::consts::TAU;
            }
            while angle_diff < -std::f32::consts::PI {
                angle_diff += std::f32::consts::TAU;
            }

            // Limit turn rate
            let max_turn = seeking.turn_rate * dt;
            let turn = angle_diff.clamp(-max_turn, max_turn);

            // Apply turn
            let new_angle = current_angle + turn;
            let speed = physics.velocity.length();
            physics.velocity = Vec2::new(new_angle.cos(), new_angle.sin()) * speed;
        }
    }
}

/// Combined projectile update: movement, lifetime, and bounds in one pass
/// This reduces from 3 iterations over all projectiles to just 1.
fn projectile_update(
    mut commands: Commands,
    time: Res<Time>,
    mut query: Query<(Entity, &mut Transform, &mut ProjectilePhysics)>,
) {
    let dt = time.delta_secs();

    // Precompute bounds (with margin for off-screen cleanup)
    const MARGIN: f32 = 50.0;
    let half_w = SCREEN_WIDTH / 2.0 + MARGIN;
    let half_h = SCREEN_HEIGHT / 2.0 + MARGIN;

    for (entity, mut transform, mut physics) in query.iter_mut() {
        // Update lifetime
        physics.lifetime -= dt;

        // Move projectile
        transform.translation.x += physics.velocity.x * dt;
        transform.translation.y += physics.velocity.y * dt;

        // Check lifetime and bounds in one go
        let pos = transform.translation;
        if physics.lifetime <= 0.0 || pos.x.abs() > half_w || pos.y.abs() > half_h {
            commands.entity(entity).despawn_recursive();
        }
    }
}

/// Spawn enemy projectile helper
pub fn spawn_enemy_projectile(
    commands: &mut Commands,
    position: Vec2,
    direction: Vec2,
    damage: f32,
    speed: f32,
) {
    let velocity = direction.normalize_or_zero() * speed;
    let angle = direction.y.atan2(direction.x) - std::f32::consts::FRAC_PI_2;

    commands.spawn(EnemyProjectileBundle {
        physics: ProjectilePhysics {
            velocity,
            lifetime: 5.0,
        },
        damage: ProjectileDamage {
            damage,
            damage_type: DamageType::EM,
            crit_chance: 0.05, // 5% crit for enemies
            crit_multiplier: 1.25,
        },
        transform: Transform::from_xyz(position.x, position.y, LAYER_ENEMY_BULLETS)
            .with_rotation(Quat::from_rotation_z(angle)),
        ..default()
    });
}

/// Spawn enemy projectile with faction-appropriate weapon visuals
pub fn spawn_enemy_projectile_typed(
    commands: &mut Commands,
    position: Vec2,
    direction: Vec2,
    damage: f32,
    speed: f32,
    weapon_type: WeaponType,
) {
    let velocity = direction.normalize_or_zero() * speed;
    let angle = direction.y.atan2(direction.x) - std::f32::consts::FRAC_PI_2;

    // Get damage type and color based on weapon type
    let (damage_type, color, size) = match weapon_type {
        WeaponType::Laser => (
            DamageType::EM,
            Color::srgb(1.0, 0.2, 0.2), // Amarr red laser
            Vec2::new(3.0, 16.0),       // Beam shape
        ),
        WeaponType::Railgun => (
            DamageType::Kinetic,
            Color::srgb(0.4, 0.8, 1.0), // Caldari cyan
            Vec2::new(4.0, 10.0),       // Fast bolt
        ),
        WeaponType::MissileLauncher => (
            DamageType::Explosive,
            Color::srgb(1.0, 0.5, 0.15), // Orange missile
            Vec2::new(6.0, 8.0),         // Larger missile
        ),
        WeaponType::Drone => (
            DamageType::Thermal,
            Color::srgb(0.5, 1.0, 0.4), // Gallente green
            Vec2::new(5.0, 5.0),        // Round drone shot
        ),
        WeaponType::Autocannon | WeaponType::Artillery => (
            DamageType::Kinetic,
            Color::srgb(1.0, 0.8, 0.3), // Minmatar yellow/orange
            Vec2::new(4.0, 8.0),        // Bullet shape
        ),
    };

    commands.spawn((
        EnemyProjectile,
        ProjectilePhysics {
            velocity,
            lifetime: 5.0,
        },
        ProjectileDamage {
            damage,
            damage_type,
            crit_chance: 0.05, // 5% crit for enemies
            crit_multiplier: 1.25,
        },
        Sprite {
            color,
            custom_size: Some(size),
            ..default()
        },
        Transform::from_xyz(position.x, position.y, LAYER_ENEMY_BULLETS)
            .with_rotation(Quat::from_rotation_z(angle)),
    ));
}
