//! Boss Battle Systems
//!
//! Handles boss movement, attacks, phases, and defeat sequences.

use bevy::prelude::*;
use crate::core::*;
use crate::assets::ShipModelCache;
use crate::entities::{
    Boss, BossData, BossState, BossMovement, BossAttack, MovementPattern,
    spawn_boss, get_phase_threshold,
};
use crate::entities::projectile::{
    EnemyProjectile, EnemyProjectileBundle, ProjectilePhysics, ProjectileDamage,
};
use crate::systems::ComboHeatSystem;
use super::effects::ScreenShake;
use super::dialogue::DialogueEvent;

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
                    boss_damage,
                ).run_if(in_state(GameState::Playing)),
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

        if spawn_boss(&mut commands, event.stage, Some(&sprite_cache), Some(&model_cache)) {
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
    mut boss_query: Query<(&mut Transform, &mut BossState, &BossData, &mut BossMovement), With<Boss>>,
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
                let target_x = ((movement.timer * 0.5).sin() * 200.0) as f32;
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
) {
    let dt = time.delta_secs();
    let player_pos = player_query
        .get_single()
        .map(|t| t.translation.truncate())
        .unwrap_or(Vec2::ZERO);

    for (transform, state, _data, mut attack) in boss_query.iter_mut() {
        if *state != BossState::Battle {
            continue;
        }

        attack.fire_timer -= dt;

        if attack.fire_timer <= 0.0 {
            let boss_pos = transform.translation.truncate();

            // Fire pattern based on current phase
            match attack.pattern.as_str() {
                "steady_beam" | "focused_beams" => {
                    // Single aimed shot
                    let dir = (player_pos - boss_pos).normalize_or_zero();
                    spawn_boss_projectile(&mut commands, boss_pos + dir * 40.0, dir, 250.0, 20.0);
                    attack.fire_timer = 0.8;
                }
                "desperate_spray" | "turret_barrage" => {
                    // Multi-directional spray
                    for i in 0..5 {
                        let angle = -0.4 + (i as f32 * 0.2);
                        let dir = Vec2::new(angle.sin(), -angle.cos());
                        spawn_boss_projectile(&mut commands, boss_pos + dir * 40.0, dir, 200.0, 15.0);
                    }
                    attack.fire_timer = 0.5;
                }
                "beam_sweep" | "purifying_beams" => {
                    // Sweep pattern
                    let sweep_angle = (time.elapsed_secs() * 3.0).sin() * 0.6;
                    let dir = Vec2::new(sweep_angle, -1.0).normalize();
                    for offset in [-30.0, 0.0, 30.0] {
                        spawn_boss_projectile(&mut commands, boss_pos + Vec2::new(offset, -30.0), dir, 300.0, 15.0);
                    }
                    attack.fire_timer = 0.3;
                }
                "drone_swarm" | "missile_swarm" => {
                    // Homing-style missiles (aimed at player)
                    let dir = (player_pos - boss_pos).normalize_or_zero();
                    spawn_boss_projectile(&mut commands, boss_pos, dir, 180.0, 30.0);
                    attack.fire_timer = 1.2;
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
fn spawn_boss_projectile(
    commands: &mut Commands,
    pos: Vec2,
    dir: Vec2,
    speed: f32,
    damage: f32,
) {
    commands.spawn(EnemyProjectileBundle {
        marker: EnemyProjectile,
        physics: ProjectilePhysics {
            velocity: dir * speed,
            lifetime: 4.0,
        },
        damage: ProjectileDamage {
            damage,
            damage_type: DamageType::EM,
        },
        sprite: Sprite {
            color: Color::srgb(1.0, 0.4, 0.1), // Orange for boss projectiles
            custom_size: Some(Vec2::new(8.0, 8.0)),
            ..default()
        },
        transform: Transform::from_xyz(pos.x, pos.y, LAYER_ENEMY_BULLETS),
    });
}

/// Check for phase transitions
fn boss_phase_check(
    mut boss_query: Query<(&mut BossData, &mut BossAttack, &mut BossState, &mut BossMovement), With<Boss>>,
    mut encounter: ResMut<BossEncounter>,
    mut screen_shake: ResMut<ScreenShake>,
) {
    for (mut data, mut attack, mut state, mut movement) in boss_query.iter_mut() {
        if *state != BossState::Battle {
            continue;
        }

        let health_percent = data.health / data.max_health;
        let current_phase = data.current_phase;
        let next_phase = current_phase + 1;

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
                if next_phase >= 3 {
                    movement.pattern = MovementPattern::Aggressive;
                }

                // Screen shake on phase change
                screen_shake.large();

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

/// Get attack pattern for boss phase
fn get_phase_pattern(boss_id: u32, phase: u32) -> String {
    match (boss_id, phase) {
        (1, 1) => "steady_beam",
        (1, 2) => "desperate_spray",
        (2, 1) => "focused_beams",
        (2, 2) => "beam_sweep",
        (3, 1) => "turret_barrage",
        (3, 2) => "missile_swarm",
        (3, 3) => "overcharge_beam",
        (_, 1) => "steady_beam",
        (_, 2) => "beam_sweep",
        (_, 3) => "drone_swarm",
        (_, 4) => "desperate_spray",
        (_, 5) => "doomsday",
        _ => "steady_beam",
    }.to_string()
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
                    info!("+{} score, +{} souls liberated", final_score, data.liberation_value);

                    // Massive screen shake
                    screen_shake.massive();

                    // Chain explosions across the boss
                    for i in 0..8 {
                        let offset = Vec2::new(
                            (i as f32 * 0.7).sin() * 40.0,
                            (i as f32 * 1.3).cos() * 30.0,
                        );
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
