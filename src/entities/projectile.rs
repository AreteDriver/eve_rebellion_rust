//! Projectile Entities
//!
//! Player bullets, enemy bullets, missiles, drones.

#![allow(dead_code)]

use crate::core::*;
use bevy::prelude::*;

/// Marker for player projectiles
#[derive(Component, Debug)]
pub struct PlayerProjectile;

/// Marker for enemy projectiles
#[derive(Component, Debug)]
pub struct EnemyProjectile;

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
            (spawn_player_projectiles, projectile_update)
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
        let velocity = event.direction * PLAYER_BULLET_SPEED;

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

        // Simple single-entity projectile
        commands.spawn((
            PlayerProjectile,
            ProjectilePhysics {
                velocity,
                lifetime: 2.0,
            },
            ProjectileDamage {
                damage: event.damage * damage_mult,
                damage_type,
            },
            Sprite {
                color,
                custom_size: Some(Vec2::new(4.0, 12.0)),
                ..default()
            },
            Transform::from_xyz(event.position.x, event.position.y, LAYER_PLAYER_BULLETS),
        ));
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
