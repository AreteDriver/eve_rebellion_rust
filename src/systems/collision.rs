//! Collision Detection System
//!
//! Handles all collision between entities using spatial partitioning.
//! Uses a grid-based approach to reduce O(n²) to O(n).

use crate::core::*;
use crate::entities::collectible::{spawn_smart_powerup, PlayerHealthState};
use crate::entities::*;
use bevy::prelude::*;

// Spatial grid configuration
const CELL_SIZE: f32 = 50.0;
const GRID_WIDTH: usize = 18; // 800 / 50 + padding
const GRID_HEIGHT: usize = 16; // 700 / 50 + padding

/// Spatial grid for fast collision lookups
#[derive(Resource, Default)]
pub struct SpatialGrid {
    /// Grid cells containing enemy entity indices
    enemy_cells: Vec<Vec<(Entity, Vec2)>>,
}

impl SpatialGrid {
    fn new() -> Self {
        Self {
            enemy_cells: (0..GRID_WIDTH * GRID_HEIGHT)
                .map(|_| Vec::with_capacity(8))
                .collect(),
        }
    }

    fn clear(&mut self) {
        for cell in &mut self.enemy_cells {
            cell.clear();
        }
    }

    #[inline]
    fn pos_to_cell(pos: Vec2) -> Option<usize> {
        // Convert from centered coords (-400..400, -350..350) to grid coords
        let gx = ((pos.x + SCREEN_WIDTH / 2.0) / CELL_SIZE) as usize;
        let gy = ((pos.y + SCREEN_HEIGHT / 2.0) / CELL_SIZE) as usize;

        if gx < GRID_WIDTH && gy < GRID_HEIGHT {
            Some(gy * GRID_WIDTH + gx)
        } else {
            None
        }
    }

    fn insert_enemy(&mut self, entity: Entity, pos: Vec2) {
        if let Some(idx) = Self::pos_to_cell(pos) {
            self.enemy_cells[idx].push((entity, pos));
        }
    }

    /// Get enemies in the same cell and adjacent cells (for border cases)
    fn get_nearby_enemies(&self, pos: Vec2) -> impl Iterator<Item = &(Entity, Vec2)> {
        let gx = ((pos.x + SCREEN_WIDTH / 2.0) / CELL_SIZE) as i32;
        let gy = ((pos.y + SCREEN_HEIGHT / 2.0) / CELL_SIZE) as i32;

        // Check 3x3 neighborhood for robustness
        let mut indices = Vec::with_capacity(9);
        for dy in -1..=1 {
            for dx in -1..=1 {
                let nx = gx + dx;
                let ny = gy + dy;
                if nx >= 0 && nx < GRID_WIDTH as i32 && ny >= 0 && ny < GRID_HEIGHT as i32 {
                    indices.push((ny * GRID_WIDTH as i32 + nx) as usize);
                }
            }
        }

        indices
            .into_iter()
            .flat_map(move |idx| self.enemy_cells[idx].iter())
    }
}

/// Collision plugin
pub struct CollisionPlugin;

impl Plugin for CollisionPlugin {
    fn build(&self, app: &mut App) {
        app.insert_resource(SpatialGrid::new())
            .add_systems(
                Update,
                (
                    update_spatial_grid,
                    player_projectile_enemy_collision,
                    enemy_projectile_player_collision,
                )
                    .chain()
                    .run_if(in_state(GameState::Playing)),
            );
    }
}

/// Update spatial grid with current enemy positions
fn update_spatial_grid(
    mut grid: ResMut<SpatialGrid>,
    enemy_query: Query<(Entity, &Transform), With<Enemy>>,
) {
    grid.clear();
    for (entity, transform) in enemy_query.iter() {
        grid.insert_enemy(entity, transform.translation.truncate());
    }
}

/// Player projectiles hitting enemies (optimized with spatial grid)
fn player_projectile_enemy_collision(
    mut commands: Commands,
    grid: Res<SpatialGrid>,
    projectile_query: Query<(Entity, &Transform, &ProjectileDamage), With<PlayerProjectile>>,
    mut enemy_query: Query<&mut EnemyStats, With<Enemy>>,
    player_query: Query<(&Transform, &ShipStats), With<Player>>,
    mut score: ResMut<ScoreSystem>,
    mut berserk: ResMut<BerserkSystem>,
    mut destroy_events: EventWriter<EnemyDestroyedEvent>,
    mut explosion_events: EventWriter<ExplosionEvent>,
    mut screen_shake: ResMut<super::effects::ScreenShake>,
    icon_cache: Res<crate::assets::PowerupIconCache>,
) {
    // Get player position and health for proximity check and smart powerups
    let (player_pos, player_health) = player_query
        .get_single()
        .map(|(t, stats)| {
            (
                t.translation.truncate(),
                Some(PlayerHealthState::from_stats(stats)),
            )
        })
        .unwrap_or((Vec2::ZERO, None));

    // Collision radius squared for faster distance checks
    const COLLISION_RADIUS_SQ: f32 = 25.0 * 25.0;

    for (proj_entity, proj_transform, proj_damage) in projectile_query.iter() {
        let proj_pos = proj_transform.translation.truncate();

        // Only check enemies in nearby grid cells (O(1) average instead of O(n))
        for &(enemy_entity, enemy_pos) in grid.get_nearby_enemies(proj_pos) {
            let dist_sq = (proj_pos - enemy_pos).length_squared();

            // Use squared distance to avoid sqrt
            if dist_sq < COLLISION_RADIUS_SQ {
                // Get mutable enemy stats
                let Ok(mut enemy_stats) = enemy_query.get_mut(enemy_entity) else {
                    continue;
                };

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
                        spawn_smart_powerup(&mut commands, enemy_pos, Some(&icon_cache), player_health);
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
    let hit_radius_sq = (hitbox.radius + 4.0) * (hitbox.radius + 4.0);

    for (proj_entity, proj_transform, proj_damage) in projectile_query.iter() {
        let proj_pos = proj_transform.translation.truncate();
        let dist_sq = (proj_pos - player_pos).length_squared();

        if dist_sq < hit_radius_sq {
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
