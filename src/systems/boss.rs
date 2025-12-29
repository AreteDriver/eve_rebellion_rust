//! Boss Battle Systems
//!
//! Handles boss movement, attacks, phases, and defeat sequences.

#![allow(dead_code)]

use super::dialogue::DialogueEvent;
use super::effects::ScreenShake;
use crate::assets::ShipModelCache;
use crate::core::*;
use crate::entities::projectile::{
    EnemyProjectile, ProjectileDamage, ProjectilePhysics,
};
use crate::entities::{
    get_phase_threshold, spawn_boss, Boss, BossAttack, BossData, BossMovement, BossState,
    MovementPattern,
};
use crate::systems::ComboHeatSystem;
use bevy::prelude::*;

/// Boss system plugin
pub struct BossPlugin;

impl Plugin for BossPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<BossEncounter>()
            .add_event::<BossSpawnEvent>()
            .add_event::<BossDefeatedEvent>()
            .add_systems(
                Update,
                (
                    handle_boss_spawn,
                    boss_intro_sequence,
                    boss_movement,
                    boss_attack,
                    boss_phase_check,
                    boss_drone_spawning,
                    boss_damage,
                )
                    .run_if(in_state(GameState::Playing)),
            );
    }
}

/// Tracks current boss encounter state
#[derive(Resource, Default)]
pub struct BossEncounter {
    /// Is a boss fight active?
    pub active: bool,
    /// Boss intro timer
    pub intro_timer: f32,
    /// Boss name card shown
    pub name_shown: bool,
    /// Phase transition timer
    pub phase_timer: f32,
    /// Defeat sequence timer
    pub defeat_timer: f32,
}

/// Event to spawn a boss
#[derive(Event)]
pub struct BossSpawnEvent {
    pub stage: u32,
}

/// Event when boss is defeated
#[derive(Event)]
pub struct BossDefeatedEvent {
    pub boss_name: String,
    pub score: u64,
    pub liberation_value: u32,
}

/// Component for bosses that spawn drones/fighters
#[derive(Component, Debug)]
pub struct BossDroneSpawner {
    /// Time between drone waves
    pub spawn_interval: f32,
    /// Cooldown timer
    pub spawn_timer: f32,
    /// Number of drones per wave
    pub drones_per_wave: u32,
    /// Type ID of spawned drones (589 = Executioner, 593 = Tristan)
    pub drone_type_id: u32,
    /// Maximum active drones
    pub max_drones: u32,
    /// Currently spawned drones count
    pub active_drones: u32,
    /// Drone spawn pattern
    pub pattern: DroneSpawnPattern,
}

impl Default for BossDroneSpawner {
    fn default() -> Self {
        Self {
            spawn_interval: 6.0,
            spawn_timer: 3.0, // First wave after 3 seconds
            drones_per_wave: 3,
            drone_type_id: 589, // Executioner (small, fast)
            max_drones: 6,
            active_drones: 0,
            pattern: DroneSpawnPattern::Flanking,
        }
    }
}

/// How drones are spawned around the boss
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum DroneSpawnPattern {
    /// Spawn in a line below boss
    #[default]
    Flanking,
    /// Spawn in a V formation
    VFormation,
    /// Spawn around boss in a circle
    Surround,
    /// Spawn from sides (for stations)
    FromSides,
}

/// Handle boss spawn events
fn handle_boss_spawn(
    mut commands: Commands,
    mut spawn_events: EventReader<BossSpawnEvent>,
    mut encounter: ResMut<BossEncounter>,
    sprite_cache: Res<crate::assets::ShipSpriteCache>,
    model_cache: Res<ShipModelCache>,
    boss_query: Query<Entity, With<Boss>>,
) {
    for event in spawn_events.read() {
        // Don't spawn if boss already exists
        if !boss_query.is_empty() {
            continue;
        }

        if spawn_boss(
            &mut commands,
            event.stage,
            Some(&sprite_cache),
            Some(&model_cache),
        ) {
            encounter.active = true;
            encounter.intro_timer = 3.0; // 3 second intro
            encounter.name_shown = false;
            encounter.phase_timer = 0.0;
            encounter.defeat_timer = 0.0;
            info!("Boss spawned for stage {}", event.stage);
        }
    }
}

/// Boss intro sequence - descend and show name
fn boss_intro_sequence(
    time: Res<Time>,
    mut encounter: ResMut<BossEncounter>,
    mut boss_query: Query<
        (&mut Transform, &mut BossState, &BossData, &mut BossMovement),
        With<Boss>,
    >,
    mut dialogue_events: EventWriter<DialogueEvent>,
) {
    let dt = time.delta_secs();

    for (mut transform, mut state, data, mut movement) in boss_query.iter_mut() {
        if *state != BossState::Intro {
            continue;
        }

        encounter.intro_timer -= dt;

        // Descend to battle position
        let target_y = if data.stationary { 200.0 } else { 150.0 };
        if transform.translation.y > target_y {
            transform.translation.y -= 100.0 * dt;
            if transform.translation.y < target_y {
                transform.translation.y = target_y;
            }
        }

        // Show name card at 2 seconds
        if encounter.intro_timer < 2.0 && !encounter.name_shown {
            encounter.name_shown = true;
            info!("BOSS: {} - \"{}\"", data.name, data.title);

            // Trigger boss intro dialogue
            dialogue_events.send(DialogueEvent::boss_intro(
                data.name.clone(),
                data.dialogue_intro.clone(),
            ));
        }

        // Intro complete
        if encounter.intro_timer <= 0.0 {
            *state = BossState::Battle;
            movement.pattern = if data.stationary {
                MovementPattern::Stationary
            } else {
                MovementPattern::Sweep
            };
            info!("Boss battle begins!");
        }
    }
}

/// Boss movement patterns
fn boss_movement(
    time: Res<Time>,
    mut boss_query: Query<(&mut Transform, &mut BossMovement, &BossState, &BossData), With<Boss>>,
    player_query: Query<&Transform, (With<crate::entities::Player>, Without<Boss>)>,
) {
    let dt = time.delta_secs();
    let player_x = player_query
        .get_single()
        .map(|t| t.translation.x)
        .unwrap_or(0.0);

    for (mut transform, mut movement, state, _data) in boss_query.iter_mut() {
        if *state != BossState::Battle {
            continue;
        }

        movement.timer += dt;

        match movement.pattern {
            MovementPattern::Stationary => {
                // No movement
            }
            MovementPattern::Sweep => {
                // Sinusoidal side-to-side
                let amplitude = 150.0;
                let period = 4.0;
                let x = (movement.timer * std::f32::consts::TAU / period).sin() * amplitude;
                transform.translation.x = x;
            }
            MovementPattern::Strafe => {
                // Move to random positions
                let target_x = (movement.timer * 0.5).sin() * 200.0;
                let diff = target_x - transform.translation.x;
                transform.translation.x += diff.signum() * movement.speed.min(diff.abs()) * dt;
            }
            MovementPattern::Aggressive => {
                // Chase player (somewhat)
                let diff = player_x - transform.translation.x;
                transform.translation.x += diff.signum() * movement.speed * 0.5 * dt;
            }
            MovementPattern::Descend => {
                // Used during intro, shouldn't happen here
            }
        }

        // Clamp to screen bounds
        let half_screen = SCREEN_WIDTH / 2.0 - 100.0;
        transform.translation.x = transform.translation.x.clamp(-half_screen, half_screen);
    }
}

/// Boss attack patterns
fn boss_attack(
    mut commands: Commands,
    time: Res<Time>,
    mut boss_query: Query<(&Transform, &BossState, &BossData, &mut BossAttack), With<Boss>>,
    player_query: Query<&Transform, (With<crate::entities::Player>, Without<Boss>)>,
    mut explosion_events: EventWriter<ExplosionEvent>,
) {
    let dt = time.delta_secs();
    let elapsed = time.elapsed_secs();
    let player_pos = player_query
        .get_single()
        .map(|t| t.translation.truncate())
        .unwrap_or(Vec2::ZERO);

    for (transform, state, data, mut attack) in boss_query.iter_mut() {
        if *state != BossState::Battle {
            continue;
        }

        attack.fire_timer -= dt;

        if attack.fire_timer <= 0.0 {
            let boss_pos = transform.translation.truncate();
            let phase = data.current_phase;
            let is_enraged = data.health / data.max_health <= 0.2;

            // Fire pattern based on current phase
            match attack.pattern.as_str() {
                "steady_beam" | "focused_beams" => {
                    // Single aimed shot - basic attack
                    let dir = (player_pos - boss_pos).normalize_or_zero();
                    spawn_boss_projectile_styled(
                        &mut commands,
                        boss_pos + dir * 40.0,
                        dir,
                        250.0,
                        20.0,
                        BossProjectileStyle::Laser,
                    );
                    attack.fire_timer = if is_enraged { 0.4 } else { 0.8 };
                }

                "spread" => {
                    // Wide spread shot - fan of bullets toward player
                    let base_dir = (player_pos - boss_pos).normalize_or_zero();
                    let base_angle = base_dir.y.atan2(base_dir.x);
                    let bullet_count = if is_enraged { 11 } else { 7 };

                    for i in 0..bullet_count {
                        let angle_offset =
                            (i as f32 - (bullet_count - 1) as f32 / 2.0) * 0.18;
                        let angle = base_angle + angle_offset;
                        let dir = Vec2::new(angle.cos(), angle.sin());
                        spawn_boss_projectile_styled(
                            &mut commands,
                            boss_pos + dir * 40.0,
                            dir,
                            200.0,
                            12.0,
                            BossProjectileStyle::Default,
                        );
                    }
                    attack.fire_timer = if is_enraged { 0.6 } else { 1.0 };
                }

                "spiral" => {
                    // Rotating spiral pattern - 8 bullets in circle
                    let base_angle = elapsed * 2.5;
                    for i in 0..8 {
                        let angle = base_angle + (i as f32 * std::f32::consts::TAU / 8.0);
                        let dir = Vec2::new(angle.cos(), angle.sin());
                        spawn_boss_projectile_styled(
                            &mut commands,
                            boss_pos,
                            dir,
                            150.0,
                            10.0,
                            BossProjectileStyle::Default,
                        );
                    }
                    attack.fire_timer = if is_enraged { 0.15 } else { 0.25 };
                }

                "ring" => {
                    // 360° ring of bullets expanding outward
                    let count = 16 + (phase * 4) as usize;
                    for i in 0..count {
                        let angle = (i as f32 / count as f32) * std::f32::consts::TAU;
                        let dir = Vec2::new(angle.cos(), angle.sin());
                        spawn_boss_projectile_styled(
                            &mut commands,
                            boss_pos,
                            dir,
                            120.0,
                            8.0,
                            BossProjectileStyle::Heavy,
                        );
                    }
                    // Screen flash for ring attack
                    explosion_events.send(ExplosionEvent {
                        position: boss_pos,
                        size: ExplosionSize::Tiny,
                        color: Color::srgb(1.0, 0.8, 0.3),
                    });
                    attack.fire_timer = if is_enraged { 1.2 } else { 2.0 };
                }

                "barrage" => {
                    // Rapid fire barrage - 5 bullets in tight cluster
                    let dir = (player_pos - boss_pos).normalize_or_zero();
                    for i in 0..5 {
                        let offset = (i as f32 - 2.0) * 15.0;
                        let spread = (i as f32 - 2.0) * 0.08;
                        let bullet_dir = Vec2::new(
                            dir.x + spread,
                            dir.y,
                        ).normalize_or_zero();
                        spawn_boss_projectile_styled(
                            &mut commands,
                            boss_pos + Vec2::new(offset, -30.0),
                            bullet_dir,
                            280.0,
                            15.0,
                            BossProjectileStyle::Laser,
                        );
                    }
                    attack.fire_timer = if is_enraged { 0.3 } else { 0.5 };
                }

                "laser_sweep" => {
                    // Sweeping laser beams - oscillates left/right
                    let sweep_angle = (elapsed * 2.0).sin() * 0.8;
                    for i in -2..=2 {
                        let angle = sweep_angle + (i as f32 * 0.15);
                        let dir = Vec2::new(angle.sin(), -angle.cos());
                        spawn_boss_projectile_styled(
                            &mut commands,
                            boss_pos + Vec2::new(i as f32 * 30.0, -30.0),
                            dir,
                            320.0,
                            18.0,
                            BossProjectileStyle::Laser,
                        );
                    }
                    attack.fire_timer = if is_enraged { 0.2 } else { 0.35 };
                }

                "mega_beam" => {
                    // Wall of heavy projectiles
                    for i in 0..5 {
                        let x_offset = (i as f32 - 2.0) * 50.0;
                        spawn_boss_projectile_styled(
                            &mut commands,
                            boss_pos + Vec2::new(x_offset, -40.0),
                            Vec2::NEG_Y,
                            100.0,
                            25.0,
                            BossProjectileStyle::Heavy,
                        );
                    }
                    attack.fire_timer = if is_enraged { 0.8 } else { 1.5 };
                }

                "desperate_spray" | "turret_barrage" => {
                    // Multi-directional spray - chaotic in enrage
                    let count = if is_enraged { 9 } else { 5 };
                    for i in 0..count {
                        let angle = -0.6 + (i as f32 * 1.2 / count as f32);
                        let dir = Vec2::new(angle.sin(), -angle.cos());
                        spawn_boss_projectile(&mut commands, boss_pos + dir * 40.0, dir, 200.0, 15.0);
                    }
                    attack.fire_timer = if is_enraged { 0.3 } else { 0.5 };
                }

                "beam_sweep" | "purifying_beams" => {
                    // Sweep pattern with 3 parallel beams
                    let sweep_angle = (elapsed * 3.0).sin() * 0.6;
                    let dir = Vec2::new(sweep_angle, -1.0).normalize();
                    for offset in [-30.0, 0.0, 30.0] {
                        spawn_boss_projectile_styled(
                            &mut commands,
                            boss_pos + Vec2::new(offset, -30.0),
                            dir,
                            300.0,
                            15.0,
                            BossProjectileStyle::Laser,
                        );
                    }
                    attack.fire_timer = if is_enraged { 0.15 } else { 0.3 };
                }

                "drone_swarm" | "missile_swarm" => {
                    // Multiple missiles aimed at player
                    let dir = (player_pos - boss_pos).normalize_or_zero();
                    let count = if is_enraged { 5 } else { 3 };
                    for i in 0..count {
                        let offset = (i as f32 - (count - 1) as f32 / 2.0) * 20.0;
                        spawn_boss_projectile_styled(
                            &mut commands,
                            boss_pos + Vec2::new(offset, -20.0),
                            dir,
                            180.0,
                            20.0,
                            BossProjectileStyle::Missile,
                        );
                    }
                    attack.fire_timer = if is_enraged { 0.8 } else { 1.2 };
                }

                "doomsday" => {
                    // Titan's doomsday - massive ring + targeted beam
                    // Ring component
                    for i in 0..24 {
                        let angle = (i as f32 / 24.0) * std::f32::consts::TAU;
                        let dir = Vec2::new(angle.cos(), angle.sin());
                        spawn_boss_projectile_styled(
                            &mut commands,
                            boss_pos,
                            dir,
                            80.0,
                            15.0,
                            BossProjectileStyle::Heavy,
                        );
                    }
                    // Targeted beam component
                    let dir = (player_pos - boss_pos).normalize_or_zero();
                    for i in 0..7 {
                        spawn_boss_projectile_styled(
                            &mut commands,
                            boss_pos + dir * (30.0 + i as f32 * 10.0),
                            dir,
                            400.0,
                            30.0,
                            BossProjectileStyle::Heavy,
                        );
                    }
                    // Big visual effect
                    explosion_events.send(ExplosionEvent {
                        position: boss_pos,
                        size: ExplosionSize::Large,
                        color: Color::srgb(1.0, 0.5, 0.1),
                    });
                    attack.fire_timer = 3.0;
                }

                _ => {
                    // Default pattern
                    let dir = (player_pos - boss_pos).normalize_or_zero();
                    spawn_boss_projectile(&mut commands, boss_pos + dir * 40.0, dir, 220.0, 18.0);
                    attack.fire_timer = 0.6;
                }
            }
        }
    }
}

/// Spawn a boss projectile
fn spawn_boss_projectile(commands: &mut Commands, pos: Vec2, dir: Vec2, speed: f32, damage: f32) {
    spawn_boss_projectile_styled(commands, pos, dir, speed, damage, BossProjectileStyle::Default);
}

/// Boss projectile visual styles
#[derive(Clone, Copy)]
enum BossProjectileStyle {
    Default,    // Orange boss bullets
    Laser,      // Red Amarr laser beam
    Heavy,      // Large slow projectile
    Missile,    // Caldari missile style
    Drone,      // Gallente drone shot
}

/// Spawn a styled boss projectile
fn spawn_boss_projectile_styled(
    commands: &mut Commands,
    pos: Vec2,
    dir: Vec2,
    speed: f32,
    damage: f32,
    style: BossProjectileStyle,
) {
    let (color, size, damage_type) = match style {
        BossProjectileStyle::Default => (
            Color::srgb(1.0, 0.4, 0.1),
            Vec2::new(8.0, 8.0),
            DamageType::EM,
        ),
        BossProjectileStyle::Laser => (
            Color::srgb(1.0, 0.2, 0.2),
            Vec2::new(4.0, 16.0),
            DamageType::EM,
        ),
        BossProjectileStyle::Heavy => (
            Color::srgb(1.0, 0.7, 0.2),
            Vec2::new(12.0, 12.0),
            DamageType::Thermal,
        ),
        BossProjectileStyle::Missile => (
            Color::srgb(0.8, 0.5, 0.2),
            Vec2::new(6.0, 10.0),
            DamageType::Explosive,
        ),
        BossProjectileStyle::Drone => (
            Color::srgb(0.4, 0.9, 0.4),
            Vec2::new(6.0, 6.0),
            DamageType::Thermal,
        ),
    };

    let angle = dir.y.atan2(dir.x) - std::f32::consts::FRAC_PI_2;

    commands.spawn((
        EnemyProjectile,
        ProjectilePhysics {
            velocity: dir * speed,
            lifetime: 4.0,
        },
        ProjectileDamage { damage, damage_type },
        Sprite {
            color,
            custom_size: Some(size),
            ..default()
        },
        Transform::from_xyz(pos.x, pos.y, LAYER_ENEMY_BULLETS)
            .with_rotation(Quat::from_rotation_z(angle)),
    ));
}

/// Check for phase transitions and enrage
fn boss_phase_check(
    mut boss_query: Query<
        (
            &Transform,
            &mut BossData,
            &mut BossAttack,
            &mut BossState,
            &mut BossMovement,
        ),
        With<Boss>,
    >,
    mut encounter: ResMut<BossEncounter>,
    mut screen_shake: ResMut<ScreenShake>,
    mut explosion_events: EventWriter<ExplosionEvent>,
) {
    for (transform, mut data, mut attack, mut state, mut movement) in boss_query.iter_mut() {
        if *state != BossState::Battle {
            continue;
        }

        let health_percent = data.health / data.max_health;
        let current_phase = data.current_phase;
        let next_phase = current_phase + 1;
        let boss_pos = transform.translation.truncate();

        // Check for enrage trigger
        if !data.is_enraged && health_percent <= data.enrage_threshold {
            data.is_enraged = true;

            // Massive screen shake for enrage
            screen_shake.massive();

            // Big explosion effect
            explosion_events.send(ExplosionEvent {
                position: boss_pos,
                size: ExplosionSize::Large,
                color: Color::srgb(1.0, 0.2, 0.2),
            });

            // Speed up attacks significantly
            attack.fire_rate *= 0.6;

            // More aggressive movement
            if !data.stationary {
                movement.pattern = MovementPattern::Aggressive;
                movement.speed *= 1.5;
            }

            info!(
                "BOSS ENRAGED! {} at {:.0}% HP - attacks intensified!",
                data.name,
                health_percent * 100.0
            );
        }

        // Visual enrage effect - periodic sparks
        if data.is_enraged && fastrand::f32() < 0.1 {
            let offset = Vec2::new(
                (fastrand::f32() - 0.5) * 60.0,
                (fastrand::f32() - 0.5) * 40.0,
            );
            explosion_events.send(ExplosionEvent {
                position: boss_pos + offset,
                size: ExplosionSize::Tiny,
                color: Color::srgb(1.0, 0.3, 0.1),
            });
        }

        // Check if should transition to next phase
        if next_phase <= data.total_phases {
            let threshold = get_phase_threshold(next_phase, data.total_phases);
            if health_percent <= threshold {
                data.current_phase = next_phase;
                *state = BossState::PhaseTransition;
                encounter.phase_timer = 1.0;

                // Update attack pattern based on phase
                attack.pattern = get_phase_pattern(data.id, next_phase);
                attack.fire_rate *= 0.85; // Speed up attacks

                // Some bosses change movement in later phases
                if next_phase >= 3 && !data.stationary {
                    movement.pattern = MovementPattern::Aggressive;
                }

                // Screen shake on phase change
                screen_shake.large();

                // Phase transition explosion
                explosion_events.send(ExplosionEvent {
                    position: boss_pos,
                    size: ExplosionSize::Medium,
                    color: Color::srgb(1.0, 0.8, 0.3),
                });

                info!(
                    "Boss phase {}/{}: {} at {:.0}% HP",
                    next_phase,
                    data.total_phases,
                    attack.pattern,
                    health_percent * 100.0
                );
            }
        }
    }
}

/// Boss drone spawning system
fn boss_drone_spawning(
    mut commands: Commands,
    time: Res<Time>,
    mut boss_query: Query<
        (&Transform, &BossState, &BossData, &mut BossDroneSpawner),
        With<Boss>,
    >,
    enemy_query: Query<Entity, With<crate::entities::Enemy>>,
    sprite_cache: Res<crate::assets::ShipSpriteCache>,
    model_cache: Res<ShipModelCache>,
    mut explosion_events: EventWriter<ExplosionEvent>,
) {
    let dt = time.delta_secs();

    for (transform, state, data, mut spawner) in boss_query.iter_mut() {
        // Only spawn during battle phase
        if *state != BossState::Battle {
            continue;
        }

        spawner.spawn_timer -= dt;

        // Count current active drones (rough estimate - all enemies)
        let current_enemies = enemy_query.iter().count() as u32;

        // Spawn more frequently when enraged
        let interval = if data.is_enraged {
            spawner.spawn_interval * 0.6
        } else {
            spawner.spawn_interval
        };

        if spawner.spawn_timer <= 0.0 && current_enemies < spawner.max_drones + 10 {
            spawner.spawn_timer = interval;

            let boss_pos = transform.translation.truncate();
            let count = spawner.drones_per_wave;

            // Spawn drones based on pattern
            for i in 0..count {
                let spawn_pos = match spawner.pattern {
                    DroneSpawnPattern::Flanking => {
                        let offset = (i as f32 - (count - 1) as f32 / 2.0) * 50.0;
                        Vec2::new(boss_pos.x + offset, boss_pos.y - 60.0)
                    }
                    DroneSpawnPattern::VFormation => {
                        let offset = (i as f32 - (count - 1) as f32 / 2.0) * 40.0;
                        let y_offset = offset.abs() * 0.5;
                        Vec2::new(boss_pos.x + offset, boss_pos.y - 50.0 - y_offset)
                    }
                    DroneSpawnPattern::Surround => {
                        let angle = (i as f32 / count as f32) * std::f32::consts::TAU;
                        let radius = 80.0;
                        Vec2::new(
                            boss_pos.x + angle.cos() * radius,
                            boss_pos.y + angle.sin() * radius,
                        )
                    }
                    DroneSpawnPattern::FromSides => {
                        let side = if i % 2 == 0 { -1.0 } else { 1.0 };
                        let offset = (i as f32 / 2.0).floor() * 30.0;
                        Vec2::new(boss_pos.x + side * 100.0, boss_pos.y - offset)
                    }
                };

                // Spawn effect
                explosion_events.send(ExplosionEvent {
                    position: spawn_pos,
                    size: ExplosionSize::Tiny,
                    color: Color::srgb(0.5, 0.8, 1.0),
                });

                // Get sprite for drone type
                let sprite = sprite_cache.get(spawner.drone_type_id);

                // Determine behavior based on drone type
                let behavior = match spawner.drone_type_id {
                    589 => crate::entities::EnemyBehavior::Homing,  // Executioner - chase player
                    591 => crate::entities::EnemyBehavior::Sniper,  // Tormentor - stay at range
                    593 => crate::entities::EnemyBehavior::Weaver,  // Tristan - erratic
                    _ => crate::entities::EnemyBehavior::Linear,
                };

                crate::entities::spawn_enemy(
                    &mut commands,
                    spawner.drone_type_id,
                    spawn_pos,
                    behavior,
                    sprite,
                    Some(&model_cache),
                );
            }

            info!(
                "{} launched {} drones!",
                data.name, count
            );
        }
    }
}

/// Get attack pattern for boss phase
fn get_phase_pattern(boss_id: u32, phase: u32) -> String {
    match (boss_id, phase) {
        // Stage 1 - Bestower (Transport)
        (1, 1) => "steady_beam",
        (1, 2) => "spread",

        // Stage 2 - Navy Omen (Patrol)
        (2, 1) => "focused_beams",
        (2, 2) => "barrage",

        // Stage 3 - Orbital Platform
        (3, 1) => "turret_barrage",
        (3, 2) => "ring",
        (3, 3) => "spiral",

        // Stage 4 - Maller Fleet
        (4, 1) => "spread",
        (4, 2) => "laser_sweep",
        (4, 3) => "barrage",

        // Stage 5 - Prophecy (Customs)
        (5, 1) => "beam_sweep",
        (5, 2) => "spiral",
        (5, 3) => "ring",

        // Stage 6 - Inquisitor
        (6, 1) => "laser_sweep",
        (6, 2) => "barrage",
        (6, 3) => "spread",

        // Stage 7 - Harbinger Strike Group
        (7, 1) => "spread",
        (7, 2) => "mega_beam",
        (7, 3) => "laser_sweep",

        // Stage 8 - Stargate Defense
        (8, 1) => "turret_barrage",
        (8, 2) => "ring",
        (8, 3) => "spiral",
        (8, 4) => "mega_beam",

        // Stage 9 - Battlestation
        (9, 1) => "turret_barrage",
        (9, 2) => "ring",
        (9, 3) => "spiral",
        (9, 4) => "missile_swarm",
        (9, 5) => "mega_beam",

        // Stage 10 - Armageddon
        (10, 1) => "laser_sweep",
        (10, 2) => "mega_beam",
        (10, 3) => "desperate_spray",

        // Stage 11 - Archon (Carrier)
        (11, 1) => "drone_swarm",
        (11, 2) => "spread",
        (11, 3) => "ring",
        (11, 4) => "missile_swarm",

        // Stage 12 - Apocalypse Navy
        (12, 1) => "laser_sweep",
        (12, 2) => "mega_beam",
        (12, 3) => "barrage",
        (12, 4) => "desperate_spray",

        // Stage 13 - Avatar Titan
        (13, 1) => "spread",
        (13, 2) => "ring",
        (13, 3) => "spiral",
        (13, 4) => "mega_beam",
        (13, 5) => "doomsday",

        // Default fallbacks
        (_, 1) => "steady_beam",
        (_, 2) => "spread",
        (_, 3) => "laser_sweep",
        (_, 4) => "ring",
        (_, 5) => "doomsday",
        _ => "steady_beam",
    }
    .to_string()
}

/// Handle boss taking damage
fn boss_damage(
    mut commands: Commands,
    mut boss_query: Query<(Entity, &Transform, &mut BossData, &mut BossState), With<Boss>>,
    projectile_query: Query<
        (Entity, &Transform, &ProjectileDamage),
        With<crate::entities::PlayerProjectile>,
    >,
    mut score: ResMut<ScoreSystem>,
    mut heat_system: ResMut<ComboHeatSystem>,
    mut encounter: ResMut<BossEncounter>,
    mut defeated_events: EventWriter<BossDefeatedEvent>,
    mut explosion_events: EventWriter<ExplosionEvent>,
    mut dialogue_events: EventWriter<DialogueEvent>,
    mut screen_shake: ResMut<ScreenShake>,
) {
    for (boss_entity, boss_transform, mut data, mut state) in boss_query.iter_mut() {
        if *state == BossState::Defeated {
            continue;
        }

        let boss_pos = boss_transform.translation.truncate();
        let boss_radius = 60.0; // Approximate hitbox

        // Check projectile collisions (only player projectiles in this query)
        for (proj_entity, proj_transform, damage) in projectile_query.iter() {
            let proj_pos = proj_transform.translation.truncate();
            let distance = (boss_pos - proj_pos).length();

            if distance < boss_radius + 10.0 {
                // Hit!
                data.health -= damage.damage;
                commands.entity(proj_entity).despawn();

                // Check for defeat
                if data.health <= 0.0 {
                    *state = BossState::Defeated;
                    encounter.defeat_timer = 3.0;

                    // Add score
                    let mult = heat_system.on_kill();
                    let final_score = (data.score_value as f32 * mult) as u64;
                    score.score += final_score;
                    heat_system.souls_liberated += data.liberation_value;

                    defeated_events.send(BossDefeatedEvent {
                        boss_name: data.name.clone(),
                        score: final_score,
                        liberation_value: data.liberation_value,
                    });

                    // Trigger boss defeat dialogue
                    dialogue_events.send(DialogueEvent::boss_defeated(
                        data.name.clone(),
                        data.dialogue_defeat.clone(),
                    ));

                    info!("BOSS DEFEATED: {}", data.name);
                    info!(
                        "+{} score, +{} souls liberated",
                        final_score, data.liberation_value
                    );

                    // Massive screen shake
                    screen_shake.massive();

                    // Chain explosions across the boss
                    for i in 0..8 {
                        let offset =
                            Vec2::new((i as f32 * 0.7).sin() * 40.0, (i as f32 * 1.3).cos() * 30.0);
                        explosion_events.send(ExplosionEvent {
                            position: boss_pos + offset,
                            size: ExplosionSize::Massive,
                            color: Color::srgb(1.0, 0.6, 0.2),
                        });
                    }

                    // Despawn boss
                    commands.entity(boss_entity).despawn_recursive();
                    encounter.active = false;
                }
                break;
            }
        }
    }
}
