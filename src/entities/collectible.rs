//! Collectible Entities
//!
//! Power-ups, refugees, credits, etc.

#![allow(dead_code)]

use crate::core::*;
use crate::systems::{check_liberation_milestone, ComboHeatSystem, DialogueEvent};
use bevy::prelude::*;

/// Marker component for collectibles
#[derive(Component, Debug)]
pub struct Collectible;

/// Collectible data
#[derive(Component, Debug, Clone)]
pub struct CollectibleData {
    /// Type of collectible
    pub collectible_type: CollectibleType,
    /// Value (amount of healing, credits, etc)
    pub value: u32,
}

/// Collectible movement pattern
#[derive(Component, Debug, Clone)]
pub struct CollectiblePhysics {
    /// Current velocity
    pub velocity: Vec2,
    /// Oscillation for floating effect
    pub oscillation: f32,
    /// Lifetime
    pub lifetime: f32,
}

impl Default for CollectiblePhysics {
    fn default() -> Self {
        Self {
            velocity: Vec2::new(0.0, -30.0),
            oscillation: 0.0,
            lifetime: 10.0,
        }
    }
}

/// Active powerup effects on the player
#[derive(Component, Debug, Default)]
pub struct PowerupEffects {
    /// Overdrive timer (speed boost)
    pub overdrive_timer: f32,
    /// Damage boost timer
    pub damage_boost_timer: f32,
    /// Invulnerability timer
    pub invuln_timer: f32,
}

impl PowerupEffects {
    pub fn is_overdrive(&self) -> bool {
        self.overdrive_timer > 0.0
    }

    pub fn is_damage_boosted(&self) -> bool {
        self.damage_boost_timer > 0.0
    }

    pub fn is_invulnerable(&self) -> bool {
        self.invuln_timer > 0.0
    }

    pub fn speed_mult(&self) -> f32 {
        if self.is_overdrive() {
            1.5
        } else {
            1.0
        }
    }

    pub fn damage_mult(&self) -> f32 {
        if self.is_damage_boosted() {
            2.0
        } else {
            1.0
        }
    }
}

/// Bundle for spawning collectibles
#[derive(Bundle)]
pub struct CollectibleBundle {
    pub collectible: Collectible,
    pub data: CollectibleData,
    pub physics: CollectiblePhysics,
    pub sprite: Sprite,
    pub transform: Transform,
}

/// Collectible plugin
pub struct CollectiblePlugin;

impl Plugin for CollectiblePlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(
            Update,
            (
                collectible_movement,
                collectible_lifetime,
                collectible_pickup,
                handle_pickup_effects,
                update_powerup_timers,
            )
                .run_if(in_state(GameState::Playing)),
        );
    }
}

/// Move collectibles with floating effect
fn collectible_movement(
    time: Res<Time>,
    mut query: Query<(&mut Transform, &mut CollectiblePhysics), With<Collectible>>,
) {
    let dt = time.delta_secs();

    for (mut transform, mut physics) in query.iter_mut() {
        physics.oscillation += dt * 3.0;

        // Float and drift
        let float_offset = physics.oscillation.sin() * 0.5;
        transform.translation.x += physics.velocity.x * dt + float_offset;
        transform.translation.y += physics.velocity.y * dt;
    }
}

/// Update collectible lifetime
fn collectible_lifetime(
    mut commands: Commands,
    time: Res<Time>,
    mut query: Query<(Entity, &mut CollectiblePhysics, &mut Sprite), With<Collectible>>,
) {
    let dt = time.delta_secs();

    for (entity, mut physics, mut sprite) in query.iter_mut() {
        physics.lifetime -= dt;

        // Blink when about to expire
        if physics.lifetime < 3.0 {
            let alpha = (physics.lifetime * 5.0).sin().abs() * 0.5 + 0.5;
            sprite.color = sprite.color.with_alpha(alpha);
        }

        if physics.lifetime <= 0.0 {
            commands.entity(entity).despawn_recursive();
        }
    }
}

/// Check for player pickup
fn collectible_pickup(
    mut commands: Commands,
    player_query: Query<&Transform, With<super::Player>>,
    collectible_query: Query<(Entity, &Transform, &CollectibleData), With<Collectible>>,
    mut pickup_events: EventWriter<CollectiblePickedUpEvent>,
) {
    let Ok(player_transform) = player_query.get_single() else {
        return;
    };

    let player_pos = player_transform.translation.truncate();
    let pickup_radius = 30.0;

    for (entity, transform, data) in collectible_query.iter() {
        let collectible_pos = transform.translation.truncate();
        let distance = (player_pos - collectible_pos).length();

        if distance < pickup_radius {
            // Send pickup event
            pickup_events.send(CollectiblePickedUpEvent {
                collectible_type: data.collectible_type,
                position: collectible_pos,
                value: data.value,
            });

            // Despawn collectible
            commands.entity(entity).despawn_recursive();
        }
    }
}

/// Handle pickup effects - apply powerup to player
fn handle_pickup_effects(
    mut pickup_events: EventReader<CollectiblePickedUpEvent>,
    mut player_query: Query<
        (&mut super::player::ShipStats, &mut PowerupEffects),
        With<super::Player>,
    >,
    mut score: ResMut<ScoreSystem>,
    mut progress: ResMut<GameProgress>,
    mut heat_system: ResMut<ComboHeatSystem>,
    mut dialogue_events: EventWriter<DialogueEvent>,
    mut rumble_events: EventWriter<crate::systems::RumbleRequest>,
) {
    let Ok((mut stats, mut effects)) = player_query.get_single_mut() else {
        return;
    };

    for event in pickup_events.read() {
        match event.collectible_type {
            CollectibleType::LiberationPod => {
                let old_count = score.souls_liberated;
                score.souls_liberated += 1;
                score.add_score(500);

                // Check for liberation milestone
                if let Some(milestone) =
                    check_liberation_milestone(old_count, score.souls_liberated)
                {
                    dialogue_events.send(DialogueEvent::liberation_milestone(milestone));
                    info!("Liberation milestone reached: {} souls!", milestone);
                }
            }
            CollectibleType::Credits => {
                progress.credits += event.value as u64;
            }
            CollectibleType::ShieldBoost => {
                stats.shield = (stats.shield + event.value as f32).min(stats.max_shield);
                info!("Shield +{}", event.value);
            }
            CollectibleType::ArmorRepair => {
                stats.armor = (stats.armor + event.value as f32).min(stats.max_armor);
                info!("Armor +{}", event.value);
            }
            CollectibleType::HullRepair => {
                stats.hull = (stats.hull + event.value as f32).min(stats.max_hull);
                info!("Hull +{}", event.value);
            }
            CollectibleType::CapacitorCharge => {
                stats.capacitor = (stats.capacitor + event.value as f32).min(stats.max_capacitor);
            }
            CollectibleType::Overdrive => {
                effects.overdrive_timer = 5.0; // 5 second speed boost
                rumble_events.send(crate::systems::RumbleRequest::powerup());
                info!("OVERDRIVE ACTIVATED!");
            }
            CollectibleType::DamageBoost => {
                effects.damage_boost_timer = 10.0; // 10 second damage boost
                rumble_events.send(crate::systems::RumbleRequest::powerup());
                info!("DAMAGE BOOST!");
            }
            CollectibleType::Invulnerability => {
                effects.invuln_timer = 3.0; // 3 seconds of invuln
                rumble_events.send(crate::systems::RumbleRequest::powerup());
                info!("INVULNERABLE!");
            }
            CollectibleType::Nanite => {
                heat_system.reduce_heat(50.0);
                info!("Heat reduced by nanites");
            }
            CollectibleType::ExtraLife => {
                // Restore all HP
                stats.shield = stats.max_shield;
                stats.armor = stats.max_armor;
                stats.hull = stats.max_hull;
                info!("EXTRA LIFE! Full HP restored!");
            }
        }
    }
}

/// Update powerup effect timers
fn update_powerup_timers(time: Res<Time>, mut query: Query<&mut PowerupEffects>) {
    let dt = time.delta_secs();
    for mut effects in query.iter_mut() {
        if effects.overdrive_timer > 0.0 {
            effects.overdrive_timer -= dt;
        }
        if effects.damage_boost_timer > 0.0 {
            effects.damage_boost_timer -= dt;
        }
        if effects.invuln_timer > 0.0 {
            effects.invuln_timer -= dt;
        }
    }
}

/// Spawn a collectible at position
pub fn spawn_collectible(
    commands: &mut Commands,
    position: Vec2,
    collectible_type: CollectibleType,
    icon_cache: Option<&crate::assets::PowerupIconCache>,
) {
    let (color, size, value) = match collectible_type {
        CollectibleType::LiberationPod => (Color::srgb(0.2, 0.9, 0.5), 20.0, 1), // Green glow
        CollectibleType::Credits => (Color::srgb(1.0, 0.84, 0.0), 12.0, 100),
        CollectibleType::ShieldBoost => (COLOR_SHIELD, 28.0, 25),
        CollectibleType::ArmorRepair => (COLOR_ARMOR, 28.0, 25),
        CollectibleType::HullRepair => (COLOR_HULL, 28.0, 25),
        CollectibleType::CapacitorCharge => (COLOR_CAPACITOR, 14.0, 50),
        CollectibleType::Overdrive => (Color::srgb(0.3, 0.9, 1.0), 28.0, 1),
        CollectibleType::DamageBoost => (Color::srgb(1.0, 0.3, 0.3), 28.0, 1),
        CollectibleType::Invulnerability => (Color::srgb(1.0, 1.0, 1.0), 28.0, 1),
        CollectibleType::Nanite => (Color::srgb(0.0, 0.8, 0.6), 28.0, 1),
        CollectibleType::ExtraLife => (Color::srgb(0.0, 1.0, 0.5), 28.0, 1),
    };

    // Try to use icon from cache, fallback to colored sprite
    let sprite = if let Some(cache) = icon_cache {
        if let Some(texture) = cache.get(&collectible_type) {
            Sprite {
                image: texture,
                custom_size: Some(Vec2::splat(size)),
                ..default()
            }
        } else {
            Sprite {
                color,
                custom_size: Some(Vec2::splat(size)),
                ..default()
            }
        }
    } else {
        Sprite {
            color,
            custom_size: Some(Vec2::splat(size)),
            ..default()
        }
    };

    commands.spawn(CollectibleBundle {
        collectible: Collectible,
        data: CollectibleData {
            collectible_type,
            value,
        },
        physics: CollectiblePhysics {
            velocity: Vec2::new(0.0, -20.0),
            oscillation: fastrand::f32() * std::f32::consts::TAU,
            lifetime: 10.0,
        },
        sprite,
        transform: Transform::from_xyz(position.x, position.y, LAYER_EFFECTS),
    });
}

/// Spawn liberation pods in a burst pattern
pub fn spawn_liberation_pods(commands: &mut Commands, position: Vec2, count: u32) {
    use std::f32::consts::TAU;

    // Cap at reasonable maximum to avoid lag
    let pod_count = count.min(20);

    for i in 0..pod_count {
        // Spread pods in a circle burst
        let angle = (i as f32 / pod_count as f32) * TAU + fastrand::f32() * 0.3;
        let speed = 40.0 + fastrand::f32() * 30.0;
        let velocity = Vec2::new(angle.cos() * speed, angle.sin() * speed - 20.0);

        // Offset spawn position slightly
        let offset = Vec2::new(
            (fastrand::f32() - 0.5) * 20.0,
            (fastrand::f32() - 0.5) * 20.0,
        );

        commands.spawn(CollectibleBundle {
            collectible: Collectible,
            data: CollectibleData {
                collectible_type: CollectibleType::LiberationPod,
                value: 1,
            },
            physics: CollectiblePhysics {
                velocity,
                oscillation: fastrand::f32() * TAU,
                lifetime: 12.0, // Pods last longer than powerups
            },
            sprite: Sprite {
                color: Color::srgb(0.2, 0.9, 0.5), // Green glow
                custom_size: Some(Vec2::splat(16.0)),
                ..default()
            },
            transform: Transform::from_xyz(
                position.x + offset.x,
                position.y + offset.y,
                LAYER_EFFECTS,
            ),
        });
    }
}

/// Player health state for smart powerup drops
#[derive(Debug, Clone, Copy)]
pub struct PlayerHealthState {
    pub shield_percent: f32,
    pub armor_percent: f32,
    pub hull_percent: f32,
}

impl PlayerHealthState {
    pub fn from_stats(stats: &super::player::ShipStats) -> Self {
        Self {
            shield_percent: if stats.max_shield > 0.0 {
                stats.shield / stats.max_shield
            } else {
                1.0
            },
            armor_percent: if stats.max_armor > 0.0 {
                stats.armor / stats.max_armor
            } else {
                1.0
            },
            hull_percent: if stats.max_hull > 0.0 {
                stats.hull / stats.max_hull
            } else {
                1.0
            },
        }
    }

    /// Determine what health type is most needed
    pub fn most_needed_health(&self) -> CollectibleType {
        // Priority: Hull (critical) > Armor > Shield
        if self.hull_percent < 0.5 {
            // Hull is low - could give any health type, weighted toward hull/armor
            let roll = fastrand::f32();
            if roll < 0.4 {
                CollectibleType::HullRepair
            } else if roll < 0.75 {
                CollectibleType::ArmorRepair
            } else {
                CollectibleType::ShieldBoost
            }
        } else if self.armor_percent < 0.5 {
            // Armor is low - give armor or shield
            let roll = fastrand::f32();
            if roll < 0.6 {
                CollectibleType::ArmorRepair
            } else {
                CollectibleType::ShieldBoost
            }
        } else if self.shield_percent < 0.7 {
            // Shield is down - primarily shield
            CollectibleType::ShieldBoost
        } else {
            // Player is healthy - random health type
            let roll = fastrand::f32();
            if roll < 0.5 {
                CollectibleType::ShieldBoost
            } else if roll < 0.8 {
                CollectibleType::ArmorRepair
            } else {
                CollectibleType::HullRepair
            }
        }
    }
}

/// Spawn random powerup with weighted chances (legacy - no health awareness)
pub fn spawn_random_powerup(
    commands: &mut Commands,
    position: Vec2,
    icon_cache: Option<&crate::assets::PowerupIconCache>,
) {
    spawn_smart_powerup(commands, position, icon_cache, None);
}

/// Spawn powerup that's smart about what the player needs
pub fn spawn_smart_powerup(
    commands: &mut Commands,
    position: Vec2,
    icon_cache: Option<&crate::assets::PowerupIconCache>,
    player_health: Option<PlayerHealthState>,
) {
    let roll = fastrand::f32();

    // 30% credits, 40% health (smart), 30% special powerups
    let powerup = if roll < 0.25 {
        CollectibleType::Credits
    } else if roll < 0.65 {
        // Health drop - be smart about what type
        if let Some(health) = player_health {
            health.most_needed_health()
        } else {
            // Fallback to random health type
            let health_roll = fastrand::f32();
            if health_roll < 0.4 {
                CollectibleType::ShieldBoost
            } else if health_roll < 0.75 {
                CollectibleType::ArmorRepair
            } else {
                CollectibleType::HullRepair
            }
        }
    } else if roll < 0.75 {
        CollectibleType::Overdrive
    } else if roll < 0.85 {
        CollectibleType::DamageBoost
    } else if roll < 0.92 {
        CollectibleType::Nanite
    } else if roll < 0.97 {
        CollectibleType::Invulnerability
    } else {
        CollectibleType::ExtraLife
    };

    spawn_collectible(commands, position, powerup, icon_cache);
}
