//! Collision Detection System
//!
//! Handles all collision between entities.

use crate::core::*;
use crate::entities::*;
use bevy::prelude::*;

/// Collision plugin
pub struct CollisionPlugin;

impl Plugin for CollisionPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(
            Update,
            (
                player_projectile_enemy_collision,
                enemy_projectile_player_collision,
            )
                .run_if(in_state(GameState::Playing)),
        );
    }
}

/// Player projectiles hitting enemies
fn player_projectile_enemy_collision(
    mut commands: Commands,
    projectile_query: Query<(Entity, &Transform, &ProjectileDamage), With<PlayerProjectile>>,
    mut enemy_query: Query<(Entity, &Transform, &mut EnemyStats), With<Enemy>>,
    player_query: Query<&Transform, With<Player>>,
    mut score: ResMut<ScoreSystem>,
    mut berserk: ResMut<BerserkSystem>,
    mut destroy_events: EventWriter<EnemyDestroyedEvent>,
    mut explosion_events: EventWriter<ExplosionEvent>,
    mut screen_shake: ResMut<super::effects::ScreenShake>,
    icon_cache: Res<crate::assets::PowerupIconCache>,
) {
    // Get player position for proximity check
    let player_pos = player_query
        .get_single()
        .map(|t| t.translation.truncate())
        .unwrap_or(Vec2::ZERO);

    for (proj_entity, proj_transform, proj_damage) in projectile_query.iter() {
        let proj_pos = proj_transform.translation.truncate();

        for (enemy_entity, enemy_transform, mut enemy_stats) in enemy_query.iter_mut() {
            let enemy_pos = enemy_transform.translation.truncate();
            let distance = (proj_pos - enemy_pos).length();

            // Simple circle collision (radius ~20 for enemies)
            if distance < 25.0 {
                // Apply damage
                enemy_stats.health -= proj_damage.damage;

                // Despawn projectile
                commands.entity(proj_entity).despawn_recursive();

                // Check if enemy destroyed
                if enemy_stats.health <= 0.0 {
                    // Calculate distance from player to enemy for berserk
                    let player_distance = (player_pos - enemy_pos).length();

                    // Update score (with berserk multiplier)
                    let base_score = enemy_stats.score_value;
                    let final_score = (base_score as f32 * berserk.score_mult()) as u64;
                    score.on_kill(final_score);

                    // Check for berserk activation
                    if berserk.on_kill_at_distance(player_distance) {
                        info!(
                            "BERSERK MODE ACTIVATED! {} proximity kills!",
                            berserk.kills_to_activate
                        );
                    }

                    // Send events
                    destroy_events.send(EnemyDestroyedEvent {
                        position: enemy_pos,
                        enemy_type: enemy_stats.name.clone(),
                        score_value: enemy_stats.score_value,
                        was_boss: enemy_stats.is_boss,
                    });

                    explosion_events.send(ExplosionEvent {
                        position: enemy_pos,
                        size: if enemy_stats.is_boss {
                            ExplosionSize::Massive
                        } else {
                            ExplosionSize::Small
                        },
                        color: Color::srgb(1.0, 0.5, 0.2),
                    });

                    // Screen shake on kill
                    if enemy_stats.is_boss {
                        screen_shake.massive();
                    } else {
                        screen_shake.trigger(3.0, 0.1); // Small shake for regular enemies
                    }

                    // Spawn liberation pods
                    spawn_liberation_pods(&mut commands, enemy_pos, enemy_stats.liberation_value);

                    // 30% chance to drop powerup (100% for bosses)
                    let drop_chance = if enemy_stats.is_boss { 1.0 } else { 0.30 };
                    if fastrand::f32() < drop_chance {
                        spawn_random_powerup(&mut commands, enemy_pos, Some(&icon_cache));
                    }

                    // Despawn enemy
                    commands.entity(enemy_entity).despawn_recursive();
                }

                break; // Projectile can only hit one enemy
            }
        }
    }
}

/// Enemy projectiles hitting player
fn enemy_projectile_player_collision(
    mut commands: Commands,
    projectile_query: Query<(Entity, &Transform, &ProjectileDamage), With<EnemyProjectile>>,
    mut player_query: Query<
        (
            &Transform,
            &mut ShipStats,
            &Hitbox,
            &PowerupEffects,
            &super::ManeuverState,
        ),
        With<Player>,
    >,
    mut score: ResMut<ScoreSystem>,
    mut damage_events: EventWriter<PlayerDamagedEvent>,
    mut screen_shake: ResMut<super::effects::ScreenShake>,
    mut next_state: ResMut<NextState<GameState>>,
) {
    let Ok((player_transform, mut player_stats, hitbox, powerups, maneuver)) =
        player_query.get_single_mut()
    else {
        return;
    };

    let player_pos = player_transform.translation.truncate();

    for (proj_entity, proj_transform, proj_damage) in projectile_query.iter() {
        let proj_pos = proj_transform.translation.truncate();
        let distance = (proj_pos - player_pos).length();

        if distance < hitbox.radius + 4.0 {
            // Despawn projectile regardless
            commands.entity(proj_entity).despawn_recursive();

            // Check invulnerability (powerups OR barrel roll i-frames)
            if powerups.is_invulnerable() || maneuver.invincible {
                continue;
            }

            // Apply damage
            let destroyed = player_stats.take_damage(proj_damage.damage, proj_damage.damage_type);

            // Lost no-damage bonus
            score.no_damage_bonus = false;

            // Send events
            damage_events.send(PlayerDamagedEvent {
                damage: proj_damage.damage,
                damage_type: proj_damage.damage_type,
                source_position: proj_pos,
            });

            // Screen shake on hit
            screen_shake.small();

            if destroyed {
                info!("Player destroyed!");
                next_state.set(GameState::GameOver);
            }
        }
    }
}
