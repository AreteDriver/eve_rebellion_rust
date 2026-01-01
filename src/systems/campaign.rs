//! Campaign System
//!
//! Manages mission flow, wave spawning, and boss fights.

use crate::assets::{ShipModelCache, ShipSpriteCache};
use crate::core::events::BossDefeatedEvent;
use crate::core::*;
use crate::entities::{spawn_boss, spawn_enemy, Boss, BossData, BossState, Enemy, EnemyBehavior};
use crate::games::ActiveModule;
use bevy::ecs::schedule::common_conditions::not;
use bevy::prelude::*;

/// Campaign system plugin
pub struct CampaignPlugin;

impl Plugin for CampaignPlugin {
    fn build(&self, app: &mut App) {
        // These systems run only when NOT in Caldari/Gallente module
        // (CG module has its own campaign systems)
        app.add_systems(
            OnEnter(GameState::Playing),
            start_mission.run_if(not(is_cg_module)),
        )
        .add_systems(
            Update,
            (
                update_mission_timer,
                check_wave_complete,
                spawn_next_wave,
                update_boss_behavior,
                check_boss_defeated,
                check_mission_complete,
            )
                .run_if(in_state(GameState::Playing))
                .run_if(not(is_cg_module)),
        )
        .add_systems(
            OnEnter(GameState::BossIntro),
            spawn_mission_boss.run_if(not(is_cg_module)),
        )
        .add_systems(
            Update,
            boss_intro_sequence
                .run_if(in_state(GameState::BossIntro))
                .run_if(not(is_cg_module)),
        )
        .add_systems(
            OnEnter(GameState::BossFight),
            start_boss_fight.run_if(not(is_cg_module)),
        );
    }
}

/// Run condition: is Caldari/Gallente module active?
fn is_cg_module(active_module: Res<ActiveModule>) -> bool {
    active_module.is_caldari_gallente()
}

/// Start mission when entering Playing state
fn start_mission(
    mut campaign: ResMut<CampaignState>,
    mut mission_events: EventWriter<MissionStartEvent>,
) {
    campaign.start_mission();

    if let Some(mission) = campaign.current_mission() {
        info!(
            "Starting Mission {}: {} - {}",
            campaign.mission_number(),
            mission.name,
            mission.description
        );
        mission_events.send(MissionStartEvent { mission });
    }
}

/// Update mission timer
fn update_mission_timer(time: Res<Time>, mut campaign: ResMut<CampaignState>) {
    if campaign.in_mission {
        campaign.mission_timer += time.delta_secs();
    }
}

/// Check if current wave is complete
fn check_wave_complete(
    mut campaign: ResMut<CampaignState>,
    enemy_query: Query<Entity, With<Enemy>>,
    _boss_query: Query<Entity, With<Boss>>,
    mut wave_events: EventWriter<WaveCompleteEvent>,
) {
    // Don't check if we're in boss wave
    if campaign.is_boss_wave() {
        return;
    }

    // Count remaining enemies
    let enemy_count = enemy_query.iter().count();
    campaign.enemies_remaining = enemy_count as u32;

    // Wave complete when no enemies remain
    if enemy_count == 0 && campaign.current_wave > 0 {
        if let Some(mission) = campaign.current_mission() {
            if campaign.current_wave <= mission.enemy_waves {
                wave_events.send(WaveCompleteEvent {
                    wave_number: campaign.current_wave,
                });
                info!("Wave {} complete!", campaign.current_wave);
            }
        }
    }
}

/// Spawn next wave of enemies
fn spawn_next_wave(
    mut commands: Commands,
    mut campaign: ResMut<CampaignState>,
    session: Res<crate::core::GameSession>,
    enemy_query: Query<Entity, With<Enemy>>,
    boss_query: Query<Entity, With<Boss>>,
    sprite_cache: Res<ShipSpriteCache>,
    model_cache: Res<ShipModelCache>,
    difficulty: Res<Difficulty>,
    mut next_state: ResMut<NextState<GameState>>,
) {
    // Only spawn if no enemies remain
    if enemy_query.iter().count() > 0 || boss_query.iter().count() > 0 {
        return;
    }

    let Some(mission) = campaign.current_mission() else {
        return;
    };

    // Check if it's boss time
    if campaign.current_wave > mission.enemy_waves {
        if !campaign.boss_spawned {
            // Transition to boss intro
            next_state.set(GameState::BossIntro);
        }
        return;
    }

    // Spawn wave enemies
    let wave = campaign.current_wave;
    let base_count = 3 + wave as usize;
    let spawn_mult = difficulty.spawn_rate_mult();
    let count = (base_count as f32 * spawn_mult) as usize;

    info!("Spawning wave {} with {} enemies", wave, count);

    // Use faction-appropriate enemies from session
    for i in 0..count {
        let enemy_def = session.random_enemy();
        let type_id = enemy_def.type_id;
        let x = (i as f32 - count as f32 / 2.0) * 80.0;
        let y = SCREEN_HEIGHT / 2.0 + 50.0 + (i as f32 * 20.0);

        // Get sprite from cache if available
        let sprite_handle = sprite_cache.get(type_id);

        spawn_enemy(
            &mut commands,
            type_id,
            Vec2::new(x, y),
            EnemyBehavior::Linear,
            sprite_handle,
            Some(&model_cache),
        );
    }

    campaign.current_wave += 1;
}

/// Spawn boss for current mission
fn spawn_mission_boss(
    mut commands: Commands,
    mut campaign: ResMut<CampaignState>,
    session: Res<crate::core::GameSession>,
    sprite_cache: Res<ShipSpriteCache>,
    model_cache: Res<ShipModelCache>,
    mut boss_events: EventWriter<BossSpawnEvent>,
) {
    let Some(mission) = campaign.current_mission() else {
        return;
    };

    // Use stage number based on mission number
    let stage = campaign.mission_number() as u32;

    if spawn_boss(
        &mut commands,
        stage,
        session.enemy_faction,
        Some(&sprite_cache),
        Some(&model_cache),
    ) {
        campaign.boss_spawned = true;
        boss_events.send(BossSpawnEvent {
            boss_type: mission.boss,
        });
        info!("Boss spawned: {:?}", mission.boss);
    }
}

/// Boss intro sequence - descend and show name
fn boss_intro_sequence(
    time: Res<Time>,
    mut boss_query: Query<(&mut Transform, &mut BossState, &BossData), With<Boss>>,
    mut next_state: ResMut<NextState<GameState>>,
    mut timer: Local<f32>,
) {
    *timer += time.delta_secs();

    for (mut transform, mut state, data) in boss_query.iter_mut() {
        if *state == BossState::Intro {
            // Descend boss
            let target_y = SCREEN_HEIGHT / 2.0 - 100.0;
            if transform.translation.y > target_y {
                transform.translation.y -= 100.0 * time.delta_secs();
            }

            // After 2 seconds, start fight
            if *timer > 2.0 {
                *state = BossState::Battle;
                *timer = 0.0;
                next_state.set(GameState::BossFight);
                info!("Boss battle started: {}", data.title);
            }
        }
    }
}

/// Start boss fight phase
fn start_boss_fight(mut boss_query: Query<&mut BossState, With<Boss>>) {
    for mut state in boss_query.iter_mut() {
        *state = BossState::Battle;
    }
}

/// Update boss behavior during fight
fn update_boss_behavior(
    time: Res<Time>,
    mut boss_query: Query<
        (
            &mut Transform,
            &mut BossData,
            &mut crate::entities::boss::BossMovement,
            &mut crate::entities::boss::BossAttack,
            &BossState,
        ),
        With<Boss>,
    >,
    player_query: Query<&Transform, (With<crate::entities::Player>, Without<Boss>)>,
    mut commands: Commands,
) {
    let player_pos = player_query
        .get_single()
        .map(|t| t.translation.truncate())
        .unwrap_or(Vec2::ZERO);

    for (mut transform, mut data, mut movement, mut attack, state) in boss_query.iter_mut() {
        if *state != BossState::Battle {
            continue;
        }

        let pos = transform.translation.truncate();
        let dt = time.delta_secs();

        // Movement patterns
        match movement.pattern {
            crate::entities::boss::MovementPattern::Sweep => {
                movement.timer += dt;
                let offset = (movement.timer * 0.5).sin() * 200.0;
                transform.translation.x = offset;
            }
            crate::entities::boss::MovementPattern::Strafe => {
                movement.timer += dt;
                if movement.timer > 2.0 {
                    movement.timer = 0.0;
                    // Move toward player X
                    let dir = (player_pos.x - pos.x).signum();
                    transform.translation.x += dir * 100.0;
                }
            }
            crate::entities::boss::MovementPattern::Aggressive => {
                let dir = (player_pos - pos).normalize_or_zero();
                transform.translation.x += dir.x * movement.speed * dt * 0.5;
                // Don't move too far down
                if transform.translation.y > 100.0 {
                    transform.translation.y += dir.y * movement.speed * dt * 0.3;
                }
            }
            _ => {}
        }

        // Check for phase transitions
        let health_percent = data.health / data.max_health;
        let phase_threshold =
            crate::entities::boss::get_phase_threshold(data.current_phase + 1, data.total_phases);

        if health_percent <= phase_threshold && data.current_phase < data.total_phases {
            data.current_phase += 1;
            info!("Boss entering phase {}!", data.current_phase);

            // Change pattern on phase change
            movement.pattern = match data.current_phase {
                2 => crate::entities::boss::MovementPattern::Strafe,
                3 => crate::entities::boss::MovementPattern::Aggressive,
                _ => crate::entities::boss::MovementPattern::Sweep,
            };
            movement.speed *= 1.2;
            attack.fire_rate *= 0.8; // Fire faster
        }

        // Attack pattern
        attack.fire_timer += dt;
        if attack.fire_timer >= attack.fire_rate {
            attack.fire_timer = 0.0;

            // Spawn projectile toward player
            let dir = (player_pos - pos).normalize_or_zero();
            let projectile_speed = 250.0 + (data.current_phase as f32 * 50.0);

            commands.spawn((
                crate::entities::EnemyProjectile,
                crate::entities::ProjectileDamage {
                    damage: 20.0 + (data.current_phase as f32 * 5.0),
                    damage_type: DamageType::EM,
                    crit_chance: 0.08,     // 8% crit for boss
                    crit_multiplier: 1.5,
                },
                crate::entities::Movement {
                    velocity: dir * projectile_speed,
                    max_speed: projectile_speed,
                    ..default()
                },
                Sprite {
                    color: Color::srgb(1.0, 0.8, 0.2),
                    custom_size: Some(Vec2::new(8.0, 16.0)),
                    ..default()
                },
                Transform::from_xyz(pos.x, pos.y - 30.0, LAYER_ENEMY_BULLETS),
            ));
        }
    }
}

/// Check if boss is defeated
fn check_boss_defeated(
    mut commands: Commands,
    mut campaign: ResMut<CampaignState>,
    mut score: ResMut<ScoreSystem>,
    mut ship_unlocks: ResMut<ShipUnlocks>,
    mut save_data: ResMut<crate::core::SaveData>,
    session: Res<crate::core::GameSession>,
    boss_query: Query<(Entity, &Transform, &BossData), With<Boss>>,
    mut boss_events: EventWriter<BossDefeatedEvent>,
    mut act_events: EventWriter<ActCompleteEvent>,
    mut next_state: ResMut<NextState<GameState>>,
) {
    for (entity, transform, data) in boss_query.iter() {
        if data.health <= 0.0 {
            info!("Boss defeated: {}", data.title);

            // Add score
            score.add_score(data.score_value);

            // Spawn massive liberation pod burst for boss defeat
            let pos = transform.translation.truncate();
            crate::entities::spawn_liberation_pods(&mut commands, pos, data.liberation_value);
            campaign.mission_souls += data.liberation_value;

            // Mark boss defeated
            campaign.boss_defeated = true;
            campaign.primary_complete = true;

            // Send event
            boss_events.send(BossDefeatedEvent {
                boss_type: data.name.clone(),
                position: transform.translation.truncate(),
                score_value: data.score_value,
            });

            // Save progress
            let stage = campaign.mission_index as u32 + 1;
            save_data.complete_stage(
                session.player_faction.short_name(),
                session.enemy_faction.short_name(),
                stage,
                campaign.mission_index as u32,
            );
            save_data.record_score(
                session.player_faction.short_name(),
                session.enemy_faction.short_name(),
                score.score,
                stage,
            );

            // Check for act completion and ship unlocks
            let missions = campaign.act.missions();
            if campaign.mission_index + 1 >= missions.len() {
                let completed_act = campaign.act;
                ship_unlocks.complete_act(completed_act.number());
                act_events.send(ActCompleteEvent { act: completed_act });
                info!("Act {} complete!", completed_act.number());
            }

            // Despawn boss
            commands.entity(entity).despawn_recursive();

            // Go to stage complete
            next_state.set(GameState::StageComplete);
        }
    }
}

/// Check if mission is complete
fn check_mission_complete(
    campaign: Res<CampaignState>,
    boss_query: Query<Entity, With<Boss>>,
    _next_state: ResMut<NextState<GameState>>,
) {
    // Mission complete when boss is defeated and no boss entities remain
    if campaign.boss_defeated && boss_query.iter().count() == 0 {
        // Already handled in check_boss_defeated
    }
}
