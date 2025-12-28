//! Enemy Spawning System
//!
//! Handles wave-based enemy spawning.

use bevy::prelude::*;
use crate::core::*;
use crate::entities::{spawn_enemy, EnemyBehavior};
use crate::assets::ShipModelCache;
use super::dialogue::{DialogueEvent, DialogueSystem};

/// Spawning plugin
pub struct SpawningPlugin;

impl Plugin for SpawningPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<WaveManager>()
            .add_systems(OnEnter(GameState::Playing), reset_wave_manager)
            .add_systems(
                Update,
                (
                    wave_spawning,
                    handle_spawn_events,
                ).run_if(in_state(GameState::Playing)),
            );
    }
}

/// Manages wave spawning
#[derive(Resource, Debug)]
pub struct WaveManager {
    /// Current wave number (within stage)
    pub wave: u32,
    /// Waves per stage before boss
    pub waves_per_stage: u32,
    /// Current stage (1-13)
    pub current_stage: u32,
    /// Enemies remaining in current wave
    pub enemies_remaining: u32,
    /// Time until next spawn
    pub spawn_timer: f32,
    /// Time between spawns
    pub spawn_interval: f32,
    /// Wave delay timer
    pub wave_delay: f32,
    /// Is currently in wave delay?
    pub in_delay: bool,
    /// Boss fight active (don't spawn waves)
    pub boss_active: bool,
    /// Stage complete, waiting for next
    pub stage_complete: bool,
}

impl Default for WaveManager {
    fn default() -> Self {
        Self {
            wave: 0,
            waves_per_stage: 5, // 5 waves then boss
            current_stage: 1,
            enemies_remaining: 0,
            spawn_timer: 0.0,
            spawn_interval: 0.8,
            wave_delay: 0.0,
            in_delay: true,
            boss_active: false,
            stage_complete: false,
        }
    }
}

/// Wave definition
#[derive(Debug, Clone)]
pub struct WaveDefinition {
    pub enemy_count: u32,
    pub enemy_types: Vec<u32>,
    pub behaviors: Vec<EnemyBehavior>,
    pub spawn_pattern: SpawnPattern,
}

fn reset_wave_manager(
    mut manager: ResMut<WaveManager>,
    mut dialogue_system: ResMut<DialogueSystem>,
    mut dialogue_events: EventWriter<DialogueEvent>,
) {
    *manager = WaveManager {
        wave: 0,
        waves_per_stage: 5,
        current_stage: 1,
        in_delay: true,
        wave_delay: 3.0, // Give time to read briefing
        boss_active: false,
        stage_complete: false,
        ..default()
    };

    // Reset dialogue system and trigger stage 1 briefing
    dialogue_system.reset();
    dialogue_events.send(DialogueEvent::stage_briefing(1));

    info!("Stage 1 - The Call begins!");
}

/// Main wave spawning logic
fn wave_spawning(
    mut commands: Commands,
    time: Res<Time>,
    mut manager: ResMut<WaveManager>,
    _stage: Res<CurrentStage>,
    enemy_query: Query<Entity, With<crate::entities::Enemy>>,
    boss_query: Query<Entity, With<crate::entities::Boss>>,
    mut wave_events: EventWriter<SpawnWaveEvent>,
    mut boss_spawn_events: EventWriter<super::boss::BossSpawnEvent>,
    mut boss_defeated_events: EventReader<super::boss::BossDefeatedEvent>,
    mut dialogue_events: EventWriter<DialogueEvent>,
    sprite_cache: Res<crate::assets::ShipSpriteCache>,
    model_cache: Res<ShipModelCache>,
) {
    let dt = time.delta_secs();

    // Handle boss defeated - progress to next stage
    for event in boss_defeated_events.read() {
        manager.boss_active = false;
        manager.stage_complete = true;
        manager.wave_delay = 4.0; // Pause before next stage
        manager.in_delay = true;

        // Check for act completion
        let completed_stage = manager.current_stage;
        if completed_stage == 4 {
            dialogue_events.send(DialogueEvent::act_complete(1));
        } else if completed_stage == 9 {
            dialogue_events.send(DialogueEvent::act_complete(2));
        } else if completed_stage == 13 {
            dialogue_events.send(DialogueEvent::act_complete(3));
        }

        info!("Stage {} complete! {} defeated!", manager.current_stage, event.boss_name);
    }

    // If boss is active, don't spawn waves
    if manager.boss_active || !boss_query.is_empty() {
        return;
    }

    // Stage complete - wait then advance
    if manager.stage_complete {
        manager.wave_delay -= dt;
        if manager.wave_delay <= 0.0 {
            manager.current_stage += 1;
            if manager.current_stage > 13 {
                info!("CAMPAIGN COMPLETE! The Elder Fleet has liberated the Minmatar people!");
                // TODO: Transition to victory screen
                return;
            }
            manager.wave = 0;
            manager.stage_complete = false;
            manager.wave_delay = 3.0;

            // Trigger stage briefing
            dialogue_events.send(DialogueEvent::stage_briefing(manager.current_stage));

            info!("Stage {} begins!", manager.current_stage);
        }
        return;
    }

    // Handle wave delay
    if manager.in_delay {
        manager.wave_delay -= dt;
        if manager.wave_delay <= 0.0 {
            manager.in_delay = false;
            manager.wave += 1;

            // Check if time for boss
            if manager.wave > manager.waves_per_stage {
                manager.boss_active = true;
                boss_spawn_events.send(super::boss::BossSpawnEvent {
                    stage: manager.current_stage,
                });
                info!("WARNING: Boss incoming!");
                return;
            }

            // Setup new wave
            let wave_def = get_wave_definition(manager.current_stage, manager.wave);
            manager.enemies_remaining = wave_def.enemy_count;
            manager.spawn_interval = 0.5 + 0.3 / (manager.wave as f32).sqrt();

            wave_events.send(SpawnWaveEvent {
                wave_number: manager.wave,
                enemy_count: wave_def.enemy_count,
                enemy_types: wave_def.enemy_types.iter().map(|id| format!("{}", id)).collect(),
            });

            info!("Stage {} Wave {}/{}: {} enemies",
                manager.current_stage, manager.wave, manager.waves_per_stage, wave_def.enemy_count);
        }
        return;
    }

    // Spawn enemies
    if manager.enemies_remaining > 0 {
        manager.spawn_timer -= dt;
        if manager.spawn_timer <= 0.0 {
            manager.spawn_timer = manager.spawn_interval;

            // Get wave definition
            let wave_def = get_wave_definition(manager.current_stage, manager.wave);

            // Pick random enemy type
            let type_idx = fastrand::usize(..wave_def.enemy_types.len());
            let type_id = wave_def.enemy_types[type_idx];

            // Pick behavior
            let behavior_idx = fastrand::usize(..wave_def.behaviors.len());
            let behavior = wave_def.behaviors[behavior_idx];

            // Spawn position based on pattern
            let pos = match wave_def.spawn_pattern {
                SpawnPattern::Single | SpawnPattern::Random => {
                    let x = fastrand::f32() * (SCREEN_WIDTH - 100.0) - (SCREEN_WIDTH / 2.0 - 50.0);
                    Vec2::new(x, SCREEN_HEIGHT / 2.0 + 50.0)
                }
                SpawnPattern::Line => {
                    let spacing = SCREEN_WIDTH / (wave_def.enemy_count as f32 + 1.0);
                    let idx = wave_def.enemy_count - manager.enemies_remaining;
                    let x = spacing * (idx as f32 + 1.0) - SCREEN_WIDTH / 2.0;
                    Vec2::new(x, SCREEN_HEIGHT / 2.0 + 50.0)
                }
                SpawnPattern::VFormation => {
                    let idx = wave_def.enemy_count - manager.enemies_remaining;
                    let center_idx = wave_def.enemy_count / 2;
                    let offset = (idx as i32 - center_idx as i32) as f32;
                    let x = offset * 60.0;
                    let y = SCREEN_HEIGHT / 2.0 + 50.0 - offset.abs() * 30.0;
                    Vec2::new(x, y)
                }
                SpawnPattern::Circle => {
                    let angle = (manager.enemies_remaining as f32) / (wave_def.enemy_count as f32) * std::f32::consts::TAU;
                    let x = angle.cos() * 200.0;
                    let y = angle.sin() * 100.0 + 200.0;
                    Vec2::new(x, y)
                }
                SpawnPattern::Swarm => {
                    let x = fastrand::f32() * 400.0 - 200.0;
                    let y = SCREEN_HEIGHT / 2.0 + 30.0 + fastrand::f32() * 50.0;
                    Vec2::new(x, y)
                }
            };

            let sprite = sprite_cache.get(type_id);
            spawn_enemy(&mut commands, type_id, pos, behavior, sprite, Some(&model_cache));
            manager.enemies_remaining -= 1;
        }
    }

    // Check if wave complete
    if manager.enemies_remaining == 0 && enemy_query.is_empty() && !manager.in_delay {
        manager.in_delay = true;
        manager.wave_delay = WAVE_DELAY;
        info!("Wave {} complete!", manager.wave);
    }
}

/// Handle manual spawn events
fn handle_spawn_events(
    mut commands: Commands,
    mut spawn_events: EventReader<SpawnEnemyEvent>,
    sprite_cache: Res<crate::assets::ShipSpriteCache>,
    model_cache: Res<ShipModelCache>,
) {
    for event in spawn_events.read() {
        let type_id: u32 = event.enemy_type.parse().unwrap_or(597);
        let behavior = match event.spawn_pattern {
            SpawnPattern::Single => EnemyBehavior::Linear,
            SpawnPattern::Line => EnemyBehavior::Linear,
            SpawnPattern::VFormation => EnemyBehavior::Zigzag,
            SpawnPattern::Circle => EnemyBehavior::Orbital,
            SpawnPattern::Random => EnemyBehavior::Homing,
            SpawnPattern::Swarm => EnemyBehavior::Kamikaze,
        };

        let sprite = sprite_cache.get(type_id);
        spawn_enemy(&mut commands, type_id, event.position, behavior, sprite, Some(&model_cache));
    }
}

/// Get wave definition based on stage and wave number
fn get_wave_definition(stage: u32, wave: u32) -> WaveDefinition {
    // Amarr enemy type IDs
    const PUNISHER: u32 = 597;
    const EXECUTIONER: u32 = 589;
    const TORMENTOR: u32 = 591;
    const COERCER: u32 = 16236;
    const MALLER: u32 = 624;
    const OMEN: u32 = 625;

    // Base enemy count scales with stage and wave
    let base_count = 3 + wave + (stage / 2);

    // Enemy types based on stage (Acts 1, 2, 3)
    let enemy_types = match stage {
        // Act 1: Stages 1-4 - Frigates
        1 => vec![PUNISHER],
        2 => vec![PUNISHER, EXECUTIONER],
        3 => vec![PUNISHER, EXECUTIONER, TORMENTOR],
        4 => vec![PUNISHER, EXECUTIONER, TORMENTOR],

        // Act 2: Stages 5-9 - Destroyers and Cruisers
        5..=6 => vec![PUNISHER, EXECUTIONER, COERCER],
        7..=8 => vec![EXECUTIONER, COERCER, MALLER],
        9 => vec![COERCER, MALLER, OMEN],

        // Act 3: Stages 10-13 - Heavy enemies
        10..=11 => vec![MALLER, OMEN, COERCER],
        12..=13 => vec![MALLER, OMEN],
        _ => vec![PUNISHER],
    };

    // Behaviors get more aggressive with stage
    let behaviors = match stage {
        1..=2 => vec![EnemyBehavior::Linear],
        3..=4 => vec![EnemyBehavior::Linear, EnemyBehavior::Zigzag],
        5..=7 => vec![EnemyBehavior::Linear, EnemyBehavior::Zigzag, EnemyBehavior::Homing],
        8..=10 => vec![EnemyBehavior::Zigzag, EnemyBehavior::Homing, EnemyBehavior::Orbital],
        _ => vec![EnemyBehavior::Homing, EnemyBehavior::Orbital, EnemyBehavior::Kamikaze],
    };

    // Spawn patterns cycle with wave
    let spawn_pattern = match wave % 5 {
        1 => SpawnPattern::Single,
        2 => SpawnPattern::Line,
        3 => SpawnPattern::VFormation,
        4 => SpawnPattern::Random,
        0 => SpawnPattern::Swarm,
        _ => SpawnPattern::Single,
    };

    WaveDefinition {
        enemy_count: base_count.min(12 + stage / 2), // Max scales with stage
        enemy_types,
        behaviors,
        spawn_pattern,
    }
}
