//! Collision Detection System
//!
//! Handles all collision between entities.

use bevy::prelude::*;
use crate::core::*;
use crate::entities::*;

/// Collision plugin
pub struct CollisionPlugin;

impl Plugin for CollisionPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(
            Update,
            (
                player_projectile_enemy_collision,
                enemy_projectile_player_collision,
            ).run_if(in_state(GameState::Playing)),
        );
    }
}

/// Player projectiles hitting enemies
fn player_projectile_enemy_collision(
    mut commands: Commands,
    projectile_query: Query<(Entity, &Transform, &ProjectileDamage), With<PlayerProjectile>>,
    mut enemy_query: Query<(Entity, &Transform, &mut EnemyStats), With<Enemy>>,
    mut score: ResMut<ScoreSystem>,
    mut berserk: ResMut<BerserkSystem>,
    mut destroy_events: EventWriter<EnemyDestroyedEvent>,
    mut explosion_events: EventWriter<ExplosionEvent>,
    mut screen_shake: ResMut<super::effects::ScreenShake>,
    icon_cache: Res<crate::assets::PowerupIconCache>,
) {
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
                    // Update score
                    score.on_kill(enemy_stats.score_value);
                    berserk.on_kill();

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
    mut player_query: Query<(&Transform, &mut ShipStats, &Hitbox, &PowerupEffects), With<Player>>,
    mut score: ResMut<ScoreSystem>,
    mut damage_events: EventWriter<PlayerDamagedEvent>,
    mut screen_shake: ResMut<super::effects::ScreenShake>,
    mut next_state: ResMut<NextState<GameState>>,
) {
    let Ok((player_transform, mut player_stats, hitbox, powerups)) = player_query.get_single_mut() else {
        return;
    };

    let player_pos = player_transform.translation.truncate();

    for (proj_entity, proj_transform, proj_damage) in projectile_query.iter() {
        let proj_pos = proj_transform.translation.truncate();
        let distance = (proj_pos - player_pos).length();

        if distance < hitbox.radius + 4.0 {
            // Despawn projectile regardless
            commands.entity(proj_entity).despawn_recursive();

            // Check invulnerability
            if powerups.is_invulnerable() {
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
