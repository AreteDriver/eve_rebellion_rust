//! Battle of Caldari Prime Module
//!
//! Caldari vs Gallente faction warfare over Caldari Prime.

use super::{ActiveModule, FactionInfo, GameModuleInfo, ModuleRegistry};
use crate::core::{Faction, GameSession, GameState};
use crate::systems::JoystickState;
use bevy::prelude::*;
use bevy::ecs::schedule::common_conditions::not;

pub mod campaign;
pub mod ships;

pub use campaign::{CGBossType, CGCampaignState, NightmareBoss, NightmareEvent, ShiigeruNightmare};
pub use ships::*;

/// Caldari/Gallente module plugin
pub struct CaldariGallentePlugin;

impl Plugin for CaldariGallentePlugin {
    fn build(&self, app: &mut App) {
        // Register module
        app.add_systems(Startup, register_module);

        // Initialize state for mode select
        app.init_state::<CGModeSelect>();

        // Faction select screen - only when this module is active
        app.add_systems(
            OnEnter(GameState::FactionSelect),
            spawn_faction_select.run_if(is_caldari_gallente),
        )
        .add_systems(
            Update,
            faction_select_input
                .run_if(in_state(GameState::FactionSelect))
                .run_if(is_caldari_gallente),
        )
        .add_systems(
            OnExit(GameState::FactionSelect),
            despawn_faction_select.run_if(is_caldari_gallente),
        );

        // Mode select screen (Campaign vs Nightmare) - Caldari only
        app.add_systems(OnEnter(CGModeSelect::Active), spawn_mode_select)
            .add_systems(
                Update,
                mode_select_input.run_if(in_state(CGModeSelect::Active)),
            )
            .add_systems(OnExit(CGModeSelect::Active), despawn_mode_select);

        // Initialize resources
        app.init_resource::<CaldariGallenteShips>();
        app.init_resource::<ShiigeruNightmare>();
        app.init_resource::<CGCampaignState>();

        // CG Campaign systems - run instead of main campaign when CG module is active
        app.add_systems(
            OnEnter(GameState::Playing),
            start_cg_mission
                .run_if(is_caldari_gallente)
                .run_if(not(nightmare_active)),
        )
        .add_systems(
            Update,
            (
                update_cg_mission,
                check_cg_wave_complete,
                spawn_cg_wave,
            )
                .chain()
                .run_if(in_state(GameState::Playing))
                .run_if(is_caldari_gallente)
                .run_if(not(nightmare_active)),
        )
        .add_systems(
            OnEnter(GameState::BossIntro),
            (spawn_cg_boss, spawn_cg_boss_intro).run_if(is_caldari_gallente),
        )
        .add_systems(
            Update,
            (cg_boss_intro, cg_boss_intro_update)
                .run_if(in_state(GameState::BossIntro))
                .run_if(is_caldari_gallente),
        )
        .add_systems(
            OnExit(GameState::BossIntro),
            despawn_cg_boss_intro.run_if(is_caldari_gallente),
        )
        .add_systems(
            Update,
            (update_cg_boss, check_cg_boss_defeated)
                .run_if(in_state(GameState::BossFight))
                .run_if(is_caldari_gallente),
        );

        // Nightmare mode systems
        app.add_systems(
            Update,
            (
                update_nightmare_mode,
                spawn_nightmare_enemies,
                update_nightmare_hud,
                update_wave_announcements,
                update_miniboss_intros,
            )
                .chain()
                .run_if(in_state(GameState::Playing))
                .run_if(nightmare_active),
        );

        // Spawn nightmare HUD when entering Playing in nightmare mode
        app.add_systems(
            OnEnter(GameState::Playing),
            spawn_nightmare_hud.run_if(nightmare_active),
        );

        // CG Stage Complete screen
        app.add_systems(
            OnEnter(GameState::StageComplete),
            spawn_cg_stage_complete.run_if(is_caldari_gallente),
        )
        .add_systems(
            Update,
            cg_stage_complete_input
                .run_if(in_state(GameState::StageComplete))
                .run_if(is_caldari_gallente),
        )
        .add_systems(
            OnExit(GameState::StageComplete),
            despawn_cg_stage_complete.run_if(is_caldari_gallente),
        );

        // CG Victory screen
        app.add_systems(
            OnEnter(GameState::Victory),
            spawn_cg_victory_screen.run_if(is_caldari_gallente),
        )
        .add_systems(
            Update,
            (update_cg_victory_particles, cg_victory_input)
                .run_if(in_state(GameState::Victory))
                .run_if(is_caldari_gallente),
        )
        .add_systems(
            OnExit(GameState::Victory),
            despawn_cg_victory.run_if(is_caldari_gallente),
        );
    }
}

/// Run condition: is the active module Caldari vs Gallente?
fn is_caldari_gallente(active_module: Res<ActiveModule>) -> bool {
    active_module.is_caldari_gallente()
}

/// Run condition: is nightmare mode active?
fn nightmare_active(nightmare: Res<ShiigeruNightmare>) -> bool {
    nightmare.active
}

// ============================================================================
// Mode Select Screen (Caldari only - Campaign vs Nightmare)
// ============================================================================

/// State for mode selection
#[derive(States, Default, Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum CGModeSelect {
    #[default]
    Inactive,
    Active,
}

#[derive(Component)]
struct ModeSelectRoot;

#[derive(Component)]
struct ModeOption {
    is_nightmare: bool,
}

#[derive(Resource, Default)]
struct ModeSelectState {
    selected: usize, // 0 = Campaign, 1 = Nightmare
    cooldown: f32,
}

fn spawn_mode_select(mut commands: Commands) {
    info!("Spawning mode select screen (Campaign vs Nightmare)");
    commands.init_resource::<ModeSelectState>();

    // Root container
    commands
        .spawn((
            ModeSelectRoot,
            Node {
                width: Val::Percent(100.0),
                height: Val::Percent(100.0),
                flex_direction: FlexDirection::Column,
                align_items: AlignItems::Center,
                justify_content: JustifyContent::Center,
                row_gap: Val::Px(30.0),
                ..default()
            },
            BackgroundColor(Color::srgb(0.02, 0.02, 0.05)),
        ))
        .with_children(|parent| {
            // Title
            parent.spawn((
                Text::new("CALDARI STATE"),
                TextFont {
                    font_size: 24.0,
                    ..default()
                },
                TextColor(COLOR_CALDARI_ACCENT),
            ));
            parent.spawn((
                Text::new("SELECT MODE"),
                TextFont {
                    font_size: 48.0,
                    ..default()
                },
                TextColor(Color::WHITE),
                Node {
                    margin: UiRect::bottom(Val::Px(40.0)),
                    ..default()
                },
            ));

            // Campaign option
            spawn_mode_option(parent, false, "CAMPAIGN", "5 Mission Story Arc", true);

            // Nightmare option
            spawn_mode_option(
                parent,
                true,
                "SHIIGERU NIGHTMARE",
                "Endless Survival • High Scores",
                false,
            );

            // Instructions
            parent.spawn((
                Text::new("[↑/↓] Select   [SPACE/ENTER] Confirm   [ESC] Back"),
                TextFont {
                    font_size: 18.0,
                    ..default()
                },
                TextColor(Color::srgb(0.5, 0.5, 0.5)),
                Node {
                    margin: UiRect::top(Val::Px(40.0)),
                    ..default()
                },
            ));
        });
}

fn spawn_mode_option(
    parent: &mut ChildBuilder,
    is_nightmare: bool,
    title: &str,
    subtitle: &str,
    selected: bool,
) {
    let border_color = if selected {
        if is_nightmare {
            Color::srgb(0.9, 0.2, 0.2) // Red for nightmare
        } else {
            COLOR_CALDARI_ACCENT
        }
    } else {
        Color::srgb(0.2, 0.2, 0.3)
    };

    let bg_color = if is_nightmare {
        Color::srgb(0.15, 0.05, 0.05) // Dark red tint
    } else {
        COLOR_CALDARI_PRIMARY.with_alpha(0.3)
    };

    parent
        .spawn((
            ModeOption { is_nightmare },
            Node {
                width: Val::Px(400.0),
                height: Val::Px(100.0),
                flex_direction: FlexDirection::Column,
                align_items: AlignItems::Center,
                justify_content: JustifyContent::Center,
                border: UiRect::all(Val::Px(3.0)),
                ..default()
            },
            BackgroundColor(bg_color),
            BorderColor(border_color),
        ))
        .with_children(|card| {
            // Title
            let title_color = if is_nightmare {
                Color::srgb(1.0, 0.4, 0.4)
            } else {
                Color::WHITE
            };
            card.spawn((
                Text::new(title),
                TextFont {
                    font_size: 28.0,
                    ..default()
                },
                TextColor(title_color),
            ));
            // Subtitle
            card.spawn((
                Text::new(subtitle),
                TextFont {
                    font_size: 16.0,
                    ..default()
                },
                TextColor(Color::srgb(0.6, 0.6, 0.6)),
            ));
        });
}

fn mode_select_input(
    keyboard: Res<ButtonInput<KeyCode>>,
    joystick: Res<JoystickState>,
    time: Res<Time>,
    mut state: ResMut<ModeSelectState>,
    mut nightmare: ResMut<ShiigeruNightmare>,
    mut next_state: ResMut<NextState<GameState>>,
    mut mode_state: ResMut<NextState<CGModeSelect>>,
    mut options: Query<(&ModeOption, &mut BorderColor)>,
) {
    let dt = time.delta_secs();
    state.cooldown = (state.cooldown - dt).max(0.0);

    // Navigation
    if state.cooldown <= 0.0 {
        let move_up = keyboard.pressed(KeyCode::ArrowUp)
            || keyboard.pressed(KeyCode::KeyW)
            || joystick.dpad_y > 0;
        let move_down = keyboard.pressed(KeyCode::ArrowDown)
            || keyboard.pressed(KeyCode::KeyS)
            || joystick.dpad_y < 0;

        if move_up && state.selected > 0 {
            state.selected = 0;
            state.cooldown = 0.2;
        } else if move_down && state.selected < 1 {
            state.selected = 1;
            state.cooldown = 0.2;
        }
    }

    // Update option borders
    for (option, mut border) in options.iter_mut() {
        let is_selected = (!option.is_nightmare && state.selected == 0)
            || (option.is_nightmare && state.selected == 1);

        let color = if is_selected {
            if option.is_nightmare {
                Color::srgb(1.0, 0.3, 0.3) // Bright red
            } else {
                COLOR_CALDARI_ACCENT
            }
        } else {
            Color::srgb(0.2, 0.2, 0.3)
        };
        *border = BorderColor(color);
    }

    // Confirm selection
    if keyboard.just_pressed(KeyCode::Space)
        || keyboard.just_pressed(KeyCode::Enter)
        || joystick.confirm()
    {
        if state.selected == 1 {
            // Nightmare mode selected
            nightmare.start();
            info!("Starting SHIIGERU NIGHTMARE mode!");
        } else {
            info!("Starting Campaign mode");
        }
        mode_state.set(CGModeSelect::Inactive);
        next_state.set(GameState::DifficultySelect);
    }

    // Back to faction select
    if keyboard.just_pressed(KeyCode::Escape) || joystick.back() {
        mode_state.set(CGModeSelect::Inactive);
        next_state.set(GameState::FactionSelect);
    }
}

fn despawn_mode_select(mut commands: Commands, query: Query<Entity, With<ModeSelectRoot>>) {
    for entity in query.iter() {
        commands.entity(entity).despawn_recursive();
    }
    commands.remove_resource::<ModeSelectState>();
}

// ============================================================================
// Nightmare Mode Systems
// ============================================================================

/// Component to mark nightmare HUD elements
#[derive(Component)]
struct NightmareHud;

/// Component to mark nightmare mini-boss
#[derive(Component)]
struct NightmareMiniBoss {
    boss_type: NightmareBoss,
}

// ============================================================================
// Nightmare Wave/Boss Announcements
// ============================================================================

/// Root marker for wave announcement overlay
#[derive(Component)]
struct NightmareWaveAnnouncement {
    timer: f32,
    max_time: f32,
}

/// Root marker for mini-boss intro overlay
#[derive(Component)]
struct NightmareMiniBossIntro {
    timer: f32,
    max_time: f32,
    boss_type: NightmareBoss,
    spawned: bool,
}

/// Pulse animation for warning text
#[derive(Component)]
struct NightmareWarningPulse {
    timer: f32,
}

/// Typewriter effect for dialogue
#[derive(Component)]
struct NightmareDialogue {
    full_text: String,
    timer: f32,
}

/// Spawn wave announcement overlay
fn spawn_wave_announcement(commands: &mut Commands, wave: u32) {
    // Only show announcement every 5th wave or wave 1
    if wave != 1 && wave % 5 != 0 {
        return;
    }

    commands
        .spawn((
            NightmareWaveAnnouncement {
                timer: 0.0,
                max_time: 1.5,
            },
            Node {
                width: Val::Percent(100.0),
                height: Val::Percent(100.0),
                justify_content: JustifyContent::Center,
                align_items: AlignItems::Center,
                flex_direction: FlexDirection::Column,
                ..default()
            },
            BackgroundColor(Color::NONE),
        ))
        .with_children(|parent| {
            parent.spawn((
                Text::new(format!("WAVE {}", wave)),
                TextFont {
                    font_size: 72.0,
                    ..default()
                },
                TextColor(Color::srgb(1.0, 0.2, 0.2)),
                NightmareWarningPulse { timer: 0.0 },
            ));

            if wave >= 20 {
                parent.spawn((
                    Text::new("EXTREME DANGER"),
                    TextFont {
                        font_size: 24.0,
                        ..default()
                    },
                    TextColor(Color::srgb(1.0, 0.4, 0.4)),
                ));
            } else if wave >= 10 {
                parent.spawn((
                    Text::new("DANGER INCREASING"),
                    TextFont {
                        font_size: 24.0,
                        ..default()
                    },
                    TextColor(Color::srgb(1.0, 0.6, 0.3)),
                ));
            }
        });
}

/// Spawn mini-boss intro overlay
fn spawn_miniboss_intro(commands: &mut Commands, boss: NightmareBoss) {
    commands
        .spawn((
            NightmareMiniBossIntro {
                timer: 0.0,
                max_time: 2.5,
                boss_type: boss,
                spawned: false,
            },
            Node {
                width: Val::Percent(100.0),
                height: Val::Percent(100.0),
                justify_content: JustifyContent::Center,
                align_items: AlignItems::Center,
                flex_direction: FlexDirection::Column,
                row_gap: Val::Px(10.0),
                ..default()
            },
            BackgroundColor(Color::srgba(0.0, 0.0, 0.0, 0.7)),
        ))
        .with_children(|parent| {
            // Warning
            parent.spawn((
                Text::new("⚠ MINI-BOSS ⚠"),
                TextFont {
                    font_size: 28.0,
                    ..default()
                },
                TextColor(Color::srgb(1.0, 0.2, 0.2)),
                NightmareWarningPulse { timer: 0.0 },
            ));

            parent.spawn(Node {
                height: Val::Px(10.0),
                ..default()
            });

            // Boss name
            parent.spawn((
                Text::new(boss.name()),
                TextFont {
                    font_size: 56.0,
                    ..default()
                },
                TextColor(Color::srgb(1.0, 0.4, 0.2)), // Orange-red
            ));

            parent.spawn(Node {
                height: Val::Px(15.0),
                ..default()
            });

            // Dialogue (typewriter)
            parent.spawn((
                Text::new(""),
                TextFont {
                    font_size: 18.0,
                    ..default()
                },
                TextColor(Color::srgb(0.8, 0.8, 0.8)),
                NightmareDialogue {
                    full_text: format!("\"{}\"", boss.dialogue()),
                    timer: 0.0,
                },
            ));
        });
}

/// Update wave announcements (fade out and despawn)
fn update_wave_announcements(
    time: Res<Time>,
    mut commands: Commands,
    mut query: Query<(Entity, &mut NightmareWaveAnnouncement, &mut BackgroundColor)>,
    mut text_query: Query<(&mut TextColor, &mut NightmareWarningPulse)>,
) {
    let dt = time.delta_secs();

    for (entity, mut announcement, mut bg) in query.iter_mut() {
        announcement.timer += dt;

        // Fade in/out background
        let progress = announcement.timer / announcement.max_time;
        let alpha = if progress < 0.2 {
            progress / 0.2 * 0.3
        } else if progress > 0.7 {
            (1.0 - progress) / 0.3 * 0.3
        } else {
            0.3
        };
        *bg = BackgroundColor(Color::srgba(0.0, 0.0, 0.0, alpha));

        if announcement.timer >= announcement.max_time {
            commands.entity(entity).despawn_recursive();
        }
    }

    // Pulse warning text
    for (mut color, mut pulse) in text_query.iter_mut() {
        pulse.timer += dt * 6.0;
        let intensity = (pulse.timer.sin() * 0.3 + 0.7).clamp(0.4, 1.0);
        *color = TextColor(Color::srgb(1.0, 0.2 * intensity, 0.2 * intensity));
    }
}

/// Update mini-boss intros (typewriter, spawn boss, despawn)
fn update_miniboss_intros(
    time: Res<Time>,
    mut commands: Commands,
    mut query: Query<(Entity, &mut NightmareMiniBossIntro)>,
    mut dialogue_query: Query<(&mut Text, &mut NightmareDialogue)>,
) {
    let dt = time.delta_secs();

    for (entity, mut intro) in query.iter_mut() {
        intro.timer += dt;

        // Spawn boss after 1.5 seconds
        if intro.timer >= 1.5 && !intro.spawned {
            intro.spawned = true;
            commands.spawn(NightmareSpawnRequest::Boss(intro.boss_type));
        }

        // Despawn overlay after max time
        if intro.timer >= intro.max_time {
            commands.entity(entity).despawn_recursive();
        }
    }

    // Typewriter effect for dialogue
    for (mut text, mut dialogue) in dialogue_query.iter_mut() {
        dialogue.timer += dt;
        let chars_to_show = ((dialogue.timer - 0.3) * 35.0) as usize; // 35 chars/sec
        let chars_to_show = chars_to_show.min(dialogue.full_text.len());
        if chars_to_show > 0 {
            **text = dialogue.full_text[..chars_to_show].to_string();
        }
    }
}

/// Update nightmare state timers and spawn events
fn update_nightmare_mode(
    time: Res<Time>,
    mut nightmare: ResMut<ShiigeruNightmare>,
    mut commands: Commands,
) {
    let event = nightmare.update(time.delta_secs());

    match event {
        NightmareEvent::SpawnWave(wave) => {
            info!("NIGHTMARE Wave {} - {} enemies incoming!", wave, nightmare.enemies_per_wave());
            // Spawn wave announcement overlay (shows every 5th wave and wave 1)
            spawn_wave_announcement(&mut commands, wave);
            // Spawn the wave immediately
            commands.spawn(NightmareSpawnRequest::Wave);
        }
        NightmareEvent::SpawnBoss(boss) => {
            info!("NIGHTMARE BOSS: {} - \"{}\"", boss.name(), boss.dialogue());
            // Spawn mini-boss intro overlay (will spawn boss after delay)
            spawn_miniboss_intro(&mut commands, boss);
            // Note: Boss spawn request is now created by update_miniboss_intros after delay
        }
        NightmareEvent::None => {}
    }
}

/// Marker for spawn requests
#[derive(Component)]
enum NightmareSpawnRequest {
    Wave,
    Boss(NightmareBoss),
}

/// Spawn enemies based on nightmare mode state
fn spawn_nightmare_enemies(
    mut commands: Commands,
    nightmare: Res<ShiigeruNightmare>,
    session: Res<GameSession>,
    spawn_requests: Query<(Entity, &NightmareSpawnRequest)>,
) {
    use crate::entities::enemy::{spawn_enemy, EnemyBehavior};

    // Get enemy type IDs based on faction
    let enemy_types: Vec<u32> = match session.enemy_faction {
        Faction::Caldari => vec![601, 602, 603], // Condor, Merlin, Kestrel
        Faction::Gallente => vec![607, 608, 609], // Atron, Incursus, Tristan
        Faction::Amarr => vec![597, 589, 590], // Punisher, Executioner, Tormentor
        Faction::Minmatar => vec![584, 585, 587], // Rifter, Slasher, Breacher
    };

    for (entity, request) in spawn_requests.iter() {
        // Despawn the request marker
        commands.entity(entity).despawn();

        match request {
            NightmareSpawnRequest::Wave => {
                // Spawn wave enemies
                let count = nightmare.enemies_per_wave();

                for i in 0..count {
                    // Spread spawn positions across top of screen
                    let x = -300.0 + (i as f32 * 600.0 / count.max(1) as f32);
                    let y = 300.0 + fastrand::f32() * 50.0;

                    // Random enemy type and behavior
                    let type_id = enemy_types[fastrand::usize(..enemy_types.len())];
                    let behavior = match fastrand::u32(0..4) {
                        0 => EnemyBehavior::Linear,
                        1 => EnemyBehavior::Zigzag,
                        2 => EnemyBehavior::Homing,
                        _ => EnemyBehavior::Weaver,
                    };

                    spawn_enemy(
                        &mut commands,
                        type_id,
                        Vec2::new(x, y),
                        behavior,
                        None,
                        None,
                    );
                }
            }
            NightmareSpawnRequest::Boss(boss_type) => {
                // Spawn mini-boss at top center
                let type_id = enemy_types[0]; // Use first type as "elite"

                spawn_enemy(
                    &mut commands,
                    type_id,
                    Vec2::new(0.0, 320.0),
                    EnemyBehavior::Homing, // Bosses track player
                    None,
                    None,
                );

                info!("Mini-boss {} spawned!", boss_type.name());
            }
        }
    }
}

/// Update nightmare HUD elements
fn update_nightmare_hud(
    nightmare: Res<ShiigeruNightmare>,
    mut hud_query: Query<(&mut Text, &NightmareHudElement)>,
) {
    for (mut text, element) in hud_query.iter_mut() {
        match element {
            NightmareHudElement::Wave => {
                **text = format!("WAVE {}", nightmare.wave);
            }
            NightmareHudElement::Time => {
                let mins = (nightmare.time_survived / 60.0) as u32;
                let secs = (nightmare.time_survived % 60.0) as u32;
                **text = format!("{:02}:{:02}", mins, secs);
            }
            NightmareHudElement::Kills => {
                **text = format!("KILLS: {}", nightmare.kills);
            }
            NightmareHudElement::Hull => {
                **text = format!("HULL: {:.0}%", nightmare.hull_integrity);
            }
        }
    }
}

/// HUD element types for nightmare mode
#[derive(Component)]
enum NightmareHudElement {
    Wave,
    Time,
    Kills,
    Hull,
}

/// Spawn the nightmare mode HUD
fn spawn_nightmare_hud(mut commands: Commands) {
    info!("Spawning nightmare mode HUD");

    // HUD container at top-left
    commands
        .spawn((
            NightmareHud,
            Node {
                position_type: PositionType::Absolute,
                top: Val::Px(10.0),
                left: Val::Px(10.0),
                flex_direction: FlexDirection::Column,
                row_gap: Val::Px(8.0),
                padding: UiRect::all(Val::Px(12.0)),
                ..default()
            },
            BackgroundColor(Color::srgba(0.1, 0.0, 0.0, 0.7)),
        ))
        .with_children(|parent| {
            // Title
            parent.spawn((
                Text::new("⚠ SHIIGERU NIGHTMARE ⚠"),
                TextFont {
                    font_size: 18.0,
                    ..default()
                },
                TextColor(Color::srgb(1.0, 0.3, 0.3)),
            ));

            // Wave counter
            parent.spawn((
                NightmareHudElement::Wave,
                Text::new("WAVE 0"),
                TextFont {
                    font_size: 24.0,
                    ..default()
                },
                TextColor(Color::WHITE),
            ));

            // Time survived
            parent.spawn((
                NightmareHudElement::Time,
                Text::new("00:00"),
                TextFont {
                    font_size: 20.0,
                    ..default()
                },
                TextColor(COLOR_CALDARI_ACCENT),
            ));

            // Kills
            parent.spawn((
                NightmareHudElement::Kills,
                Text::new("KILLS: 0"),
                TextFont {
                    font_size: 16.0,
                    ..default()
                },
                TextColor(Color::srgb(0.7, 0.7, 0.7)),
            ));

            // Hull integrity
            parent.spawn((
                NightmareHudElement::Hull,
                Text::new("HULL: 100%"),
                TextFont {
                    font_size: 16.0,
                    ..default()
                },
                TextColor(Color::srgb(0.8, 0.4, 0.4)),
            ));
        });
}

// ============================================================================
// CG Campaign Systems
// ============================================================================

/// Component for CG boss entities
#[derive(Component)]
struct CGBoss {
    boss_type: CGBossType,
    health: f32,
    max_health: f32,
    current_phase: u32,
    total_phases: u32,
}

/// Component for CG boss movement
#[derive(Component)]
struct CGBossMovement {
    timer: f32,
    speed: f32,
}

/// Component for CG boss attacks
#[derive(Component)]
struct CGBossAttack {
    fire_timer: f32,
    fire_rate: f32,
}

// ============================================================================
// CG Boss Intro UI Components
// ============================================================================

/// Root marker for CG boss intro overlay
#[derive(Component)]
struct CGBossIntroRoot;

/// Warning text that pulses
#[derive(Component)]
struct CGBossIntroWarning {
    timer: f32,
}

/// Boss name that fades in
#[derive(Component)]
struct CGBossIntroName {
    timer: f32,
}

/// Boss dialogue that types in
#[derive(Component)]
struct CGBossIntroDialogue {
    full_text: String,
    timer: f32,
}

/// Start a CG mission when entering Playing state
fn start_cg_mission(mut cg_campaign: ResMut<CGCampaignState>) {
    cg_campaign.start_mission();

    if let Some(mission) = cg_campaign.current_mission() {
        info!(
            "Starting CG Mission {}: {} - {}",
            cg_campaign.mission_number(),
            mission.name,
            mission.description
        );
    }
}

/// Update CG mission timer
fn update_cg_mission(
    _time: Res<Time>,
    cg_campaign: Res<CGCampaignState>,
    nightmare: Res<ShiigeruNightmare>,
) {
    // Don't update if nightmare mode is active
    if nightmare.active {
        return;
    }

    if cg_campaign.in_mission {
        // Timer tracking could be added here if needed
    }
}

/// Check if current wave is complete in CG campaign
fn check_cg_wave_complete(
    cg_campaign: Res<CGCampaignState>,
    enemy_query: Query<Entity, With<crate::entities::Enemy>>,
    boss_query: Query<Entity, With<CGBoss>>,
) {
    // Don't check if we're in boss wave
    if cg_campaign.is_boss_wave() {
        return;
    }

    // Don't check if boss exists
    if boss_query.iter().count() > 0 {
        return;
    }

    // Wave complete when no enemies remain
    let enemy_count = enemy_query.iter().count();
    if enemy_count == 0 && cg_campaign.current_wave > 0 && cg_campaign.in_mission {
        if let Some(mission) = cg_campaign.current_mission() {
            if cg_campaign.current_wave <= mission.waves {
                info!("CG Wave {} complete!", cg_campaign.current_wave);
            }
        }
    }
}

/// Spawn next wave of enemies for CG campaign
fn spawn_cg_wave(
    mut commands: Commands,
    mut cg_campaign: ResMut<CGCampaignState>,
    session: Res<GameSession>,
    difficulty: Res<crate::core::Difficulty>,
    enemy_query: Query<Entity, With<crate::entities::Enemy>>,
    boss_query: Query<Entity, With<CGBoss>>,
    mut next_state: ResMut<NextState<GameState>>,
) {
    use crate::entities::enemy::{spawn_enemy, EnemyBehavior};

    // Only spawn if no enemies remain
    if enemy_query.iter().count() > 0 || boss_query.iter().count() > 0 {
        return;
    }

    let Some(mission) = cg_campaign.current_mission() else {
        return;
    };

    // Check if it's boss time
    if cg_campaign.current_wave > mission.waves {
        if !cg_campaign.boss_spawned && mission.boss.is_some() {
            // Transition to boss intro
            next_state.set(GameState::BossIntro);
        } else if mission.boss.is_none() {
            // No boss mission - complete immediately
            next_state.set(GameState::StageComplete);
        }
        return;
    }

    // Spawn wave enemies
    let wave = cg_campaign.current_wave;
    let base_count = 3 + wave as usize;
    let spawn_mult = difficulty.spawn_rate_mult();
    let count = (base_count as f32 * spawn_mult) as usize;

    info!("CG: Spawning wave {} with {} enemies", wave, count);

    // Get enemy type IDs based on enemy faction
    let enemy_types: Vec<u32> = match session.enemy_faction {
        Faction::Caldari => vec![601, 602, 603], // Condor, Merlin, Kestrel
        Faction::Gallente => vec![607, 608, 609], // Atron, Incursus, Tristan
        Faction::Amarr => vec![597, 589, 590],
        Faction::Minmatar => vec![584, 585, 587],
    };

    for i in 0..count {
        let type_id = enemy_types[fastrand::usize(..enemy_types.len())];
        let x = (i as f32 - count as f32 / 2.0) * 80.0;
        let y = 300.0 + 50.0 + (i as f32 * 20.0);

        let behavior = match fastrand::u32(0..4) {
            0 => EnemyBehavior::Linear,
            1 => EnemyBehavior::Zigzag,
            2 => EnemyBehavior::Homing,
            _ => EnemyBehavior::Weaver,
        };

        spawn_enemy(&mut commands, type_id, Vec2::new(x, y), behavior, None, None);
    }

    cg_campaign.current_wave += 1;
}

/// Spawn CG boss for current mission
fn spawn_cg_boss(
    mut commands: Commands,
    mut cg_campaign: ResMut<CGCampaignState>,
    session: Res<GameSession>,
) {
    let Some(mission) = cg_campaign.current_mission() else {
        return;
    };

    let Some(boss_type) = mission.boss else {
        return;
    };

    info!("Spawning CG Boss: {}", boss_type.name());

    let health = boss_type.health();
    let phases = boss_type.phases();

    // Boss color based on enemy faction
    let boss_color = match session.enemy_faction {
        Faction::Caldari => Color::srgb(0.4, 0.6, 0.9),   // Blue-ish for Caldari
        Faction::Gallente => Color::srgb(0.4, 0.9, 0.5), // Green-ish for Gallente
        _ => Color::srgb(1.0, 0.5, 0.5),
    };

    // Spawn the boss entity with Enemy + EnemyStats for collision system compatibility
    commands.spawn((
        crate::entities::Enemy,
        crate::entities::EnemyStats {
            type_id: 0, // CG boss uses custom type
            name: boss_type.name().to_string(),
            health,
            max_health: health,
            speed: 80.0,
            score_value: (health as u64) * 10,
            is_boss: true,
            liberation_value: 50,
        },
        CGBoss {
            boss_type,
            health,
            max_health: health,
            current_phase: 1,
            total_phases: phases,
        },
        CGBossMovement {
            timer: 0.0,
            speed: 80.0,
        },
        CGBossAttack {
            fire_timer: 0.0,
            fire_rate: 1.2,
        },
        Sprite {
            color: boss_color,
            custom_size: Some(Vec2::new(80.0, 80.0)), // Larger for boss
            ..default()
        },
        Transform::from_xyz(0.0, 400.0, 10.0),
    ));

    cg_campaign.boss_spawned = true;
}

/// CG Boss intro sequence
fn cg_boss_intro(
    time: Res<Time>,
    mut boss_query: Query<(&mut Transform, &CGBoss)>,
    mut next_state: ResMut<NextState<GameState>>,
    mut timer: Local<f32>,
) {
    *timer += time.delta_secs();

    for (mut transform, boss) in boss_query.iter_mut() {
        // Descend boss to battle position
        let target_y = 200.0;
        if transform.translation.y > target_y {
            transform.translation.y -= 100.0 * time.delta_secs();
        }

        // After 2 seconds, start fight
        if *timer > 2.0 {
            *timer = 0.0;
            next_state.set(GameState::BossFight);
            info!("CG Boss battle started: {}", boss.boss_type.name());
        }
    }
}

/// Spawn CG boss intro UI overlay
fn spawn_cg_boss_intro(mut commands: Commands, cg_campaign: Res<CGCampaignState>) {
    let Some(mission) = cg_campaign.current_mission() else {
        return;
    };

    let Some(boss_type) = mission.boss else {
        return;
    };

    let boss_name = boss_type.name();
    let boss_title = boss_type.title();
    let dialogue = boss_type.dialogue_intro();
    let phases = boss_type.phases();

    // Phase difficulty indicator
    let phase_text = match phases {
        1 => "Single Phase",
        2 => "Two Phases",
        3 => "Three Phases • Challenging",
        4 => "Four Phases • Dangerous",
        _ => "Multi-Phase",
    };

    commands
        .spawn((
            CGBossIntroRoot,
            Node {
                width: Val::Percent(100.0),
                height: Val::Percent(100.0),
                justify_content: JustifyContent::Center,
                align_items: AlignItems::Center,
                flex_direction: FlexDirection::Column,
                row_gap: Val::Px(12.0),
                ..default()
            },
            BackgroundColor(Color::srgba(0.0, 0.0, 0.0, 0.9)),
        ))
        .with_children(|parent| {
            // Warning text (pulses)
            parent.spawn((
                Text::new("⚠ WARNING ⚠"),
                TextFont {
                    font_size: 28.0,
                    ..default()
                },
                TextColor(Color::srgb(1.0, 0.2, 0.2)),
                CGBossIntroWarning { timer: 0.0 },
            ));

            parent.spawn(Node {
                height: Val::Px(15.0),
                ..default()
            });

            // Boss name (fades in) - Caldari blue instead of Amarr gold
            parent.spawn((
                Text::new(boss_name),
                TextFont {
                    font_size: 72.0,
                    ..default()
                },
                TextColor(Color::srgba(0.2, 0.6, 1.0, 0.0)), // Start transparent, Caldari blue
                CGBossIntroName { timer: 0.0 },
            ));

            // Boss title
            parent.spawn((
                Text::new(boss_title),
                TextFont {
                    font_size: 22.0,
                    ..default()
                },
                TextColor(Color::srgb(0.4, 0.7, 0.9)), // Lighter blue
            ));

            // Phase indicator
            parent.spawn((
                Text::new(phase_text),
                TextFont {
                    font_size: 14.0,
                    ..default()
                },
                TextColor(if phases >= 4 {
                    Color::srgb(1.0, 0.4, 0.4) // Red for dangerous
                } else if phases >= 3 {
                    Color::srgb(1.0, 0.7, 0.3) // Orange for challenging
                } else {
                    Color::srgb(0.6, 0.6, 0.6) // Gray for normal
                }),
            ));

            parent.spawn(Node {
                height: Val::Px(30.0),
                ..default()
            });

            // Boss dialogue (types in)
            parent.spawn((
                Text::new(""),
                TextFont {
                    font_size: 18.0,
                    ..default()
                },
                TextColor(Color::srgb(0.8, 0.8, 0.8)),
                CGBossIntroDialogue {
                    full_text: format!("\"{}\"", dialogue),
                    timer: 0.0,
                },
            ));
        });
}

/// Update CG boss intro animations
fn cg_boss_intro_update(
    time: Res<Time>,
    mut warning_query: Query<(&mut TextColor, &mut CGBossIntroWarning)>,
    mut name_query: Query<(&mut TextColor, &mut CGBossIntroName), Without<CGBossIntroWarning>>,
    mut dialogue_query: Query<(&mut Text, &mut CGBossIntroDialogue)>,
) {
    let dt = time.delta_secs();

    // Pulse warning text
    for (mut color, mut warning) in warning_query.iter_mut() {
        warning.timer += dt * 4.0;
        let pulse = (warning.timer.sin() * 0.3 + 0.7).clamp(0.4, 1.0);
        *color = TextColor(Color::srgb(1.0, 0.2 * pulse, 0.2 * pulse));
    }

    // Fade in boss name (Caldari blue)
    for (mut color, mut name) in name_query.iter_mut() {
        name.timer += dt * 2.0;
        let alpha = (name.timer - 0.3).clamp(0.0, 1.0); // Delay 0.3s then fade in
        *color = TextColor(Color::srgba(0.2, 0.6, 1.0, alpha));
    }

    // Type in dialogue
    for (mut text, mut dialogue) in dialogue_query.iter_mut() {
        dialogue.timer += dt;
        let chars_to_show = ((dialogue.timer - 0.5) * 30.0) as usize; // 30 chars/sec, 0.5s delay
        let chars_to_show = chars_to_show.min(dialogue.full_text.len());
        if chars_to_show > 0 {
            **text = dialogue.full_text[..chars_to_show].to_string();
        }
    }
}

/// Despawn CG boss intro UI
fn despawn_cg_boss_intro(mut commands: Commands, query: Query<Entity, With<CGBossIntroRoot>>) {
    for entity in query.iter() {
        commands.entity(entity).despawn_recursive();
    }
}

/// Update CG boss behavior during fight
fn update_cg_boss(
    time: Res<Time>,
    mut boss_query: Query<(
        &mut Transform,
        &mut CGBoss,
        &mut CGBossMovement,
        &mut CGBossAttack,
        &crate::entities::EnemyStats,
    )>,
    player_query: Query<&Transform, (With<crate::entities::Player>, Without<CGBoss>)>,
    mut commands: Commands,
) {
    let player_pos = player_query
        .get_single()
        .map(|t| t.translation.truncate())
        .unwrap_or(Vec2::ZERO);

    for (mut transform, mut boss, mut movement, mut attack, enemy_stats) in boss_query.iter_mut() {
        let pos = transform.translation.truncate();
        let dt = time.delta_secs();

        // Sync health from EnemyStats (collision system updates this)
        boss.health = enemy_stats.health;

        // Movement - sweep pattern
        movement.timer += dt;
        let offset = (movement.timer * 0.5).sin() * 200.0;
        transform.translation.x = offset;

        // Phase transitions
        let health_percent = boss.health / boss.max_health;
        let phase_threshold = 1.0 - (boss.current_phase as f32 / boss.total_phases as f32);

        if health_percent <= phase_threshold && boss.current_phase < boss.total_phases {
            boss.current_phase += 1;
            movement.speed *= 1.2;
            attack.fire_rate *= 0.8;
            info!("CG Boss entering phase {}!", boss.current_phase);
        }

        // Attack
        attack.fire_timer += dt;
        if attack.fire_timer >= attack.fire_rate {
            attack.fire_timer = 0.0;

            let dir = (player_pos - pos).normalize_or_zero();
            let projectile_speed = 250.0 + (boss.current_phase as f32 * 50.0);

            commands.spawn((
                crate::entities::EnemyProjectile,
                crate::entities::ProjectileDamage {
                    damage: 20.0 + (boss.current_phase as f32 * 5.0),
                    damage_type: crate::core::DamageType::EM,
                    crit_chance: 0.08,
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
                Transform::from_xyz(pos.x, pos.y - 30.0, 9.0),
            ));
        }
    }
}

/// Check if CG boss is defeated
fn check_cg_boss_defeated(
    mut commands: Commands,
    mut cg_campaign: ResMut<CGCampaignState>,
    boss_query: Query<(Entity, &CGBoss, &crate::entities::EnemyStats)>,
    mut next_state: ResMut<NextState<GameState>>,
) {
    for (entity, boss, enemy_stats) in boss_query.iter() {
        // Check EnemyStats health (collision system updates this)
        if enemy_stats.health <= 0.0 {
            info!("CG Boss defeated: {}", boss.boss_type.name());

            // Mark boss defeated
            cg_campaign.boss_defeated = true;

            // Despawn boss
            commands.entity(entity).despawn_recursive();

            // Go to stage complete (mission advancement happens when player confirms)
            next_state.set(GameState::StageComplete);
        }
    }
}

// ============================================================================
// CG Stage Complete Screen
// ============================================================================

/// Marker for CG stage complete screen
#[derive(Component)]
struct CGStageCompleteRoot;

fn spawn_cg_stage_complete(
    mut commands: Commands,
    cg_campaign: Res<CGCampaignState>,
    score: Res<crate::core::ScoreSystem>,
    session: Res<GameSession>,
) {
    let mission_name = cg_campaign
        .current_mission()
        .map(|m| m.name)
        .unwrap_or("MISSION");

    // Determine faction color based on player faction
    let faction_color = match session.player_faction {
        Faction::Caldari => Color::srgb(0.2, 0.6, 1.0), // Caldari blue
        Faction::Gallente => Color::srgb(0.3, 0.9, 0.4), // Gallente green
        _ => Color::WHITE,
    };

    // Check if T3 was just unlocked
    let t3_just_unlocked = cg_campaign
        .current_mission()
        .map(|m| m.unlocks_t3)
        .unwrap_or(false);

    commands
        .spawn((
            CGStageCompleteRoot,
            Node {
                width: Val::Percent(100.0),
                height: Val::Percent(100.0),
                justify_content: JustifyContent::Center,
                align_items: AlignItems::Center,
                flex_direction: FlexDirection::Column,
                row_gap: Val::Px(15.0),
                ..default()
            },
            BackgroundColor(Color::srgba(0.0, 0.02, 0.08, 0.95)),
        ))
        .with_children(|parent| {
            // Victory header
            parent.spawn((
                Text::new("MISSION COMPLETE"),
                TextFont {
                    font_size: 56.0,
                    ..default()
                },
                TextColor(Color::srgb(0.3, 1.0, 0.3)),
            ));

            parent.spawn((
                Text::new(mission_name),
                TextFont {
                    font_size: 28.0,
                    ..default()
                },
                TextColor(faction_color),
            ));

            parent.spawn(Node {
                height: Val::Px(20.0),
                ..default()
            });

            // Stats
            parent.spawn((
                Text::new(format!("Score: {}", score.score)),
                TextFont {
                    font_size: 24.0,
                    ..default()
                },
                TextColor(Color::WHITE),
            ));

            parent.spawn((
                Text::new(format!("Best Chain: {}x", score.chain)),
                TextFont {
                    font_size: 20.0,
                    ..default()
                },
                TextColor(Color::srgb(0.8, 0.8, 0.8)),
            ));

            // T3 unlock notification
            if t3_just_unlocked {
                parent.spawn(Node {
                    height: Val::Px(10.0),
                    ..default()
                });

                parent.spawn((
                    Text::new("★ TACTICAL DESTROYERS UNLOCKED ★"),
                    TextFont {
                        font_size: 24.0,
                        ..default()
                    },
                    TextColor(Color::srgb(1.0, 0.9, 0.2)),
                ));
            }

            parent.spawn(Node {
                height: Val::Px(30.0),
                ..default()
            });

            // Continue prompt
            parent.spawn((
                Text::new("Press SPACE to continue"),
                TextFont {
                    font_size: 18.0,
                    ..default()
                },
                TextColor(Color::srgb(0.6, 0.6, 0.6)),
            ));

            // Controller hint
            parent.spawn((
                Text::new("A/ENTER Continue • B/ESC Main Menu"),
                TextFont {
                    font_size: 14.0,
                    ..default()
                },
                TextColor(Color::srgb(0.4, 0.4, 0.4)),
            ));
        });
}

fn cg_stage_complete_input(
    keyboard: Res<ButtonInput<KeyCode>>,
    joystick: Res<JoystickState>,
    mut cg_campaign: ResMut<CGCampaignState>,
    mut transitions: EventWriter<crate::ui::TransitionEvent>,
) {
    if keyboard.just_pressed(KeyCode::Space)
        || keyboard.just_pressed(KeyCode::Enter)
        || joystick.confirm()
    {
        // Advance to next mission (returns false if campaign complete)
        if cg_campaign.complete_mission() {
            // More missions available
            transitions.send(crate::ui::TransitionEvent::to(GameState::Playing));
        } else {
            // Campaign complete!
            transitions.send(crate::ui::TransitionEvent::slow(GameState::Victory));
        }
    }

    if keyboard.just_pressed(KeyCode::Escape) || joystick.back() {
        transitions.send(crate::ui::TransitionEvent::to(GameState::MainMenu));
    }
}

fn despawn_cg_stage_complete(
    mut commands: Commands,
    query: Query<Entity, With<CGStageCompleteRoot>>,
) {
    for entity in query.iter() {
        commands.entity(entity).despawn_recursive();
    }
}

// ============================================================================
// CG Victory Screen (Campaign Complete)
// ============================================================================

/// Marker for CG victory screen
#[derive(Component)]
struct CGVictoryRoot;

/// Marker for CG victory particles
#[derive(Component)]
struct CGVictoryParticle {
    velocity: Vec2,
    lifetime: f32,
    max_lifetime: f32,
}

fn spawn_cg_victory_screen(
    mut commands: Commands,
    score: Res<crate::core::ScoreSystem>,
    session: Res<GameSession>,
    cg_campaign: Res<CGCampaignState>,
    mut save_data: ResMut<crate::core::SaveData>,
) {
    // Determine faction-specific content
    let (header, subtitle, quote, author, motto, particle_color1, particle_color2) =
        match session.player_faction {
            Faction::Caldari => (
                "CALDARI PRIME SECURED",
                "The State Stands Victorious",
                "\"The Caldari way is the only way.\"",
                "— Caldari Navy Command",
                "FOR THE STATE",
                Color::srgb(0.2, 0.6, 1.0),  // Caldari blue
                Color::srgb(0.4, 0.8, 0.9),  // Light cyan
            ),
            Faction::Gallente => (
                "CALDARI PRIME LIBERATED",
                "Freedom Prevails",
                "\"Liberty must be defended, at any cost.\"",
                "— Federation High Command",
                "LIBERTÉ POUR TOUS",
                Color::srgb(0.3, 0.9, 0.4),  // Gallente green
                Color::srgb(0.5, 0.8, 0.3),  // Olive
            ),
            _ => (
                "CAMPAIGN COMPLETE",
                "Victory Achieved",
                "\"Well fought.\"",
                "— Command",
                "VICTORY",
                Color::WHITE,
                Color::srgb(0.8, 0.8, 0.8),
            ),
        };

    // Check for new high score
    let faction_key = format!("cg_{}", session.player_faction.short_name());
    let enemy_key = format!("cg_{}", session.enemy_faction.short_name());
    let previous_high = save_data.get_high_score(&faction_key, &enemy_key);
    let is_new_high_score = score.score > previous_high;

    if is_new_high_score {
        save_data.record_score(&faction_key, &enemy_key, score.score, 5);
    }

    // Spawn celebration particles
    for _ in 0..60 {
        let x = (fastrand::f32() - 0.5) * crate::core::SCREEN_WIDTH;
        let y = -crate::core::SCREEN_HEIGHT / 2.0 - fastrand::f32() * 100.0;
        let vx = (fastrand::f32() - 0.5) * 100.0;
        let vy = 80.0 + fastrand::f32() * 120.0;
        let size = 4.0 + fastrand::f32() * 8.0;
        let lifetime = 3.0 + fastrand::f32() * 4.0;

        let color = if fastrand::bool() {
            particle_color1
        } else {
            particle_color2
        };

        commands.spawn((
            CGVictoryRoot,
            CGVictoryParticle {
                velocity: Vec2::new(vx, vy),
                lifetime,
                max_lifetime: lifetime,
            },
            Sprite {
                color,
                custom_size: Some(Vec2::splat(size)),
                ..default()
            },
            Transform::from_xyz(x, y, 50.0),
        ));
    }

    // Main UI container
    commands
        .spawn((
            CGVictoryRoot,
            Node {
                width: Val::Percent(100.0),
                height: Val::Percent(100.0),
                justify_content: JustifyContent::Center,
                align_items: AlignItems::Center,
                flex_direction: FlexDirection::Column,
                row_gap: Val::Px(12.0),
                ..default()
            },
            BackgroundColor(Color::srgba(0.0, 0.02, 0.05, 0.9)),
        ))
        .with_children(|parent| {
            // Victory header
            parent.spawn((
                Text::new(header),
                TextFont {
                    font_size: 64.0,
                    ..default()
                },
                TextColor(particle_color1),
            ));

            parent.spawn((
                Text::new(subtitle),
                TextFont {
                    font_size: 26.0,
                    ..default()
                },
                TextColor(particle_color2),
            ));

            parent.spawn(Node {
                height: Val::Px(20.0),
                ..default()
            });

            // Campaign stats box
            parent
                .spawn((
                    Node {
                        padding: UiRect::all(Val::Px(20.0)),
                        flex_direction: FlexDirection::Column,
                        align_items: AlignItems::Center,
                        row_gap: Val::Px(8.0),
                        border: UiRect::all(Val::Px(2.0)),
                        ..default()
                    },
                    BorderColor(particle_color1),
                    BackgroundColor(Color::srgba(0.05, 0.05, 0.1, 0.8)),
                ))
                .with_children(|stats| {
                    // New high score banner
                    if is_new_high_score {
                        stats.spawn((
                            Text::new("★ NEW HIGH SCORE ★"),
                            TextFont {
                                font_size: 28.0,
                                ..default()
                            },
                            TextColor(Color::srgb(1.0, 0.9, 0.0)),
                        ));
                    }

                    stats.spawn((
                        Text::new(format!("FINAL SCORE: {}", score.score)),
                        TextFont {
                            font_size: 32.0,
                            ..default()
                        },
                        TextColor(Color::srgb(1.0, 0.9, 0.3)),
                    ));

                    if !is_new_high_score && previous_high > 0 {
                        stats.spawn((
                            Text::new(format!("High Score: {}", previous_high)),
                            TextFont {
                                font_size: 18.0,
                                ..default()
                            },
                            TextColor(Color::srgb(0.6, 0.6, 0.6)),
                        ));
                    }

                    stats.spawn((
                        Text::new(format!("Max Multiplier: {:.1}x", score.multiplier)),
                        TextFont {
                            font_size: 24.0,
                            ..default()
                        },
                        TextColor(Color::srgb(0.8, 0.8, 0.8)),
                    ));

                    stats.spawn((
                        Text::new(format!("Missions Completed: {}/5", cg_campaign.mission_index + 1)),
                        TextFont {
                            font_size: 20.0,
                            ..default()
                        },
                        TextColor(particle_color2),
                    ));
                });

            parent.spawn(Node {
                height: Val::Px(15.0),
                ..default()
            });

            // Quote
            parent.spawn((
                Text::new(quote),
                TextFont {
                    font_size: 18.0,
                    ..default()
                },
                TextColor(Color::srgb(0.7, 0.7, 0.8)),
            ));

            parent.spawn((
                Text::new(author),
                TextFont {
                    font_size: 14.0,
                    ..default()
                },
                TextColor(Color::srgb(0.5, 0.5, 0.6)),
            ));

            parent.spawn(Node {
                height: Val::Px(25.0),
                ..default()
            });

            // Button row
            parent
                .spawn(Node {
                    flex_direction: FlexDirection::Row,
                    column_gap: Val::Px(40.0),
                    ..default()
                })
                .with_children(|row| {
                    // PLAY AGAIN button
                    row.spawn((
                        Node {
                            width: Val::Px(160.0),
                            height: Val::Px(50.0),
                            justify_content: JustifyContent::Center,
                            align_items: AlignItems::Center,
                            border: UiRect::all(Val::Px(2.0)),
                            ..default()
                        },
                        BorderColor(particle_color1),
                        BackgroundColor(Color::srgba(0.2, 0.6, 1.0, 0.15)),
                    ))
                    .with_children(|btn| {
                        btn.spawn((
                            Text::new("PLAY AGAIN"),
                            TextFont {
                                font_size: 22.0,
                                ..default()
                            },
                            TextColor(particle_color1),
                        ));
                    });

                    // MAIN MENU button
                    row.spawn((
                        Node {
                            width: Val::Px(160.0),
                            height: Val::Px(50.0),
                            justify_content: JustifyContent::Center,
                            align_items: AlignItems::Center,
                            border: UiRect::all(Val::Px(2.0)),
                            ..default()
                        },
                        BorderColor(particle_color1),
                        BackgroundColor(Color::NONE),
                    ))
                    .with_children(|btn| {
                        btn.spawn((
                            Text::new("MAIN MENU"),
                            TextFont {
                                font_size: 22.0,
                                ..default()
                            },
                            TextColor(particle_color1),
                        ));
                    });
                });

            parent.spawn(Node {
                height: Val::Px(20.0),
                ..default()
            });

            // Faction motto
            parent.spawn((
                Text::new(motto),
                TextFont {
                    font_size: 20.0,
                    ..default()
                },
                TextColor(particle_color2.with_alpha(0.7)),
            ));
        });
}

fn update_cg_victory_particles(
    time: Res<Time>,
    mut query: Query<(&mut Transform, &mut CGVictoryParticle, &mut Sprite)>,
) {
    let dt = time.delta_secs();
    for (mut transform, mut particle, mut sprite) in query.iter_mut() {
        particle.lifetime -= dt;
        transform.translation.x += particle.velocity.x * dt;
        transform.translation.y += particle.velocity.y * dt;

        // Fade out
        let alpha = (particle.lifetime / particle.max_lifetime).clamp(0.0, 1.0);
        sprite.color = sprite.color.with_alpha(alpha);
    }
}

fn cg_victory_input(
    keyboard: Res<ButtonInput<KeyCode>>,
    joystick: Res<JoystickState>,
    mut cg_campaign: ResMut<CGCampaignState>,
    mut score: ResMut<crate::core::ScoreSystem>,
    mut transitions: EventWriter<crate::ui::TransitionEvent>,
) {
    // Left/Right to select button (simplified - just accept any input)
    if keyboard.just_pressed(KeyCode::Space)
        || keyboard.just_pressed(KeyCode::Enter)
        || joystick.confirm()
    {
        // Reset and play again
        *cg_campaign = CGCampaignState::default();
        score.reset_game();
        transitions.send(crate::ui::TransitionEvent::to(GameState::FactionSelect));
    }

    if keyboard.just_pressed(KeyCode::Escape) || joystick.back() {
        *cg_campaign = CGCampaignState::default();
        transitions.send(crate::ui::TransitionEvent::to(GameState::MainMenu));
    }
}

fn despawn_cg_victory(mut commands: Commands, query: Query<Entity, With<CGVictoryRoot>>) {
    for entity in query.iter() {
        commands.entity(entity).despawn_recursive();
    }
}

fn register_module(mut registry: ResMut<ModuleRegistry>) {
    registry.register(GameModuleInfo {
        id: "caldari_gallente",
        display_name: "Battle of Caldari Prime",
        subtitle: "The War for Caldari Prime",
        description: "Experience the brutal conflict between Caldari and Gallente forces.",
        factions: vec![
            FactionInfo {
                id: "caldari",
                name: "Caldari State",
                primary_color: Color::srgb(0.1, 0.29, 0.48),
                secondary_color: Color::srgb(0.29, 0.6, 0.79),
                accent_color: Color::srgb(0.48, 0.79, 0.79),
                doctrine: vec!["Missiles", "Shields", "ECM"],
                description: "Corporate efficiency meets military precision.",
            },
            FactionInfo {
                id: "gallente",
                name: "Gallente Federation",
                primary_color: Color::srgb(0.16, 0.35, 0.16),
                secondary_color: Color::srgb(0.35, 0.79, 0.35),
                accent_color: Color::srgb(0.54, 0.92, 0.54),
                doctrine: vec!["Drones", "Armor", "Blasters"],
                description: "Freedom through firepower.",
            },
        ],
    });
}

// ============================================================================
// Faction Select Screen
// ============================================================================

#[derive(Component)]
struct FactionSelectRoot;

#[derive(Component)]
struct FactionPanel {
    faction: &'static str,
}

#[derive(Resource, Default)]
struct FactionSelectState {
    selected: usize, // 0 = Caldari, 1 = Gallente
    cooldown: f32,
}

// Caldari colors
const COLOR_CALDARI_PRIMARY: Color = Color::srgb(0.1, 0.29, 0.48);
const COLOR_CALDARI_SECONDARY: Color = Color::srgb(0.29, 0.6, 0.79);
const COLOR_CALDARI_ACCENT: Color = Color::srgb(0.48, 0.79, 0.79);

// Gallente colors
const COLOR_GALLENTE_PRIMARY: Color = Color::srgb(0.16, 0.35, 0.16);
const COLOR_GALLENTE_SECONDARY: Color = Color::srgb(0.35, 0.79, 0.35);
const COLOR_GALLENTE_ACCENT: Color = Color::srgb(0.54, 0.92, 0.54);

fn spawn_faction_select(mut commands: Commands) {
    info!("Spawning faction select screen!");
    commands.init_resource::<FactionSelectState>();

    // Root container - split screen
    commands
        .spawn((
            FactionSelectRoot,
            Node {
                width: Val::Percent(100.0),
                height: Val::Percent(100.0),
                flex_direction: FlexDirection::Row,
                ..default()
            },
            BackgroundColor(Color::srgb(0.02, 0.02, 0.05)),
        ))
        .with_children(|parent| {
            // Left panel - Caldari
            spawn_faction_panel(parent, "caldari", "CALDARI STATE", true);

            // Center divider with VS
            parent
                .spawn((
                    Node {
                        width: Val::Px(80.0),
                        height: Val::Percent(100.0),
                        justify_content: JustifyContent::Center,
                        align_items: AlignItems::Center,
                        flex_direction: FlexDirection::Column,
                        ..default()
                    },
                    BackgroundColor(Color::srgb(0.05, 0.05, 0.08)),
                ))
                .with_children(|divider| {
                    // Top line
                    divider.spawn((
                        Node {
                            width: Val::Px(2.0),
                            height: Val::Percent(35.0),
                            ..default()
                        },
                        BackgroundColor(Color::srgb(0.3, 0.3, 0.4)),
                    ));
                    // VS text
                    divider.spawn((
                        Text::new("VS"),
                        TextFont {
                            font_size: 36.0,
                            ..default()
                        },
                        TextColor(Color::srgb(0.9, 0.3, 0.3)),
                        Node {
                            margin: UiRect::axes(Val::ZERO, Val::Px(20.0)),
                            ..default()
                        },
                    ));
                    // Bottom line
                    divider.spawn((
                        Node {
                            width: Val::Px(2.0),
                            height: Val::Percent(35.0),
                            ..default()
                        },
                        BackgroundColor(Color::srgb(0.3, 0.3, 0.4)),
                    ));
                });

            // Right panel - Gallente
            spawn_faction_panel(parent, "gallente", "GALLENTE FEDERATION", false);
        });

    // Title overlay
    commands
        .spawn((
            FactionSelectRoot,
            Node {
                width: Val::Percent(100.0),
                position_type: PositionType::Absolute,
                top: Val::Px(30.0),
                flex_direction: FlexDirection::Column,
                align_items: AlignItems::Center,
                row_gap: Val::Px(8.0),
                ..default()
            },
        ))
        .with_children(|parent| {
            parent.spawn((
                Text::new("BATTLE OF CALDARI PRIME"),
                TextFont {
                    font_size: 24.0,
                    ..default()
                },
                TextColor(Color::srgb(0.5, 0.5, 0.6)),
            ));
            parent.spawn((
                Text::new("CHOOSE YOUR SIDE"),
                TextFont {
                    font_size: 48.0,
                    ..default()
                },
                TextColor(Color::srgb(0.9, 0.9, 0.9)),
            ));
        });

    // Bottom instruction
    commands
        .spawn((
            FactionSelectRoot,
            Node {
                width: Val::Percent(100.0),
                position_type: PositionType::Absolute,
                bottom: Val::Px(40.0),
                justify_content: JustifyContent::Center,
                ..default()
            },
        ))
        .with_children(|parent| {
            parent.spawn((
                Text::new("[←/→] Select   [SPACE/ENTER] Confirm   [ESC] Back"),
                TextFont {
                    font_size: 18.0,
                    ..default()
                },
                TextColor(Color::srgb(0.5, 0.5, 0.5)),
            ));
        });
}

fn spawn_faction_panel(
    parent: &mut ChildBuilder,
    faction: &'static str,
    name: &str,
    is_caldari: bool,
) {
    let (primary, secondary, accent) = if is_caldari {
        (
            COLOR_CALDARI_PRIMARY,
            COLOR_CALDARI_SECONDARY,
            COLOR_CALDARI_ACCENT,
        )
    } else {
        (
            COLOR_GALLENTE_PRIMARY,
            COLOR_GALLENTE_SECONDARY,
            COLOR_GALLENTE_ACCENT,
        )
    };

    let doctrine = if is_caldari {
        vec!["MISSILES", "SHIELDS", "ECM"]
    } else {
        vec!["DRONES", "ARMOR", "BLASTERS"]
    };

    let tagline = if is_caldari {
        "\"The State Provides\""
    } else {
        "\"Liberty or Death\""
    };

    let description = if is_caldari {
        "Corporate efficiency meets military precision.\nShield-tanked missile platforms\ndominate the battlefield."
    } else {
        "Freedom through firepower.\nArmor-tanked drone and blaster\nplatforms break all opposition."
    };

    // Outer container with border for selection
    parent
        .spawn((
            FactionPanel { faction },
            Node {
                width: Val::Percent(50.0),
                height: Val::Percent(100.0),
                border: UiRect::all(Val::Px(4.0)),
                ..default()
            },
            BackgroundColor(Color::NONE),
            BorderColor(Color::NONE), // Will be set by selection logic
        ))
        .with_children(|outer| {
            // Inner panel
            outer
                .spawn((
                    Node {
                        width: Val::Percent(100.0),
                        height: Val::Percent(100.0),
                        flex_direction: FlexDirection::Column,
                        align_items: AlignItems::Center,
                        justify_content: JustifyContent::Center,
                        padding: UiRect::all(Val::Px(40.0)),
                        row_gap: Val::Px(16.0),
                        ..default()
                    },
                    BackgroundColor(primary.with_alpha(0.25)),
                ))
                .with_children(|panel| {
                    // Faction emblem (hexagon-ish shape)
                    panel
                        .spawn((
                            Node {
                                width: Val::Px(140.0),
                                height: Val::Px(140.0),
                                border: UiRect::all(Val::Px(4.0)),
                                justify_content: JustifyContent::Center,
                                align_items: AlignItems::Center,
                                margin: UiRect::bottom(Val::Px(10.0)),
                                ..default()
                            },
                            BackgroundColor(primary.with_alpha(0.8)),
                            BorderColor(accent),
                        ))
                        .with_children(|emblem| {
                            // Faction symbol
                            let symbol = if is_caldari { "◆" } else { "✦" };
                            emblem.spawn((
                                Text::new(symbol),
                                TextFont {
                                    font_size: 80.0,
                                    ..default()
                                },
                                TextColor(accent),
                            ));
                        });

                    // Faction name
                    panel.spawn((
                        Text::new(name),
                        TextFont {
                            font_size: 32.0,
                            ..default()
                        },
                        TextColor(Color::WHITE),
                    ));

                    // Tagline
                    panel.spawn((
                        Text::new(tagline),
                        TextFont {
                            font_size: 18.0,
                            ..default()
                        },
                        TextColor(accent),
                        Node {
                            margin: UiRect::bottom(Val::Px(10.0)),
                            ..default()
                        },
                    ));

                    // Doctrine tags
                    panel
                        .spawn((Node {
                            flex_direction: FlexDirection::Row,
                            column_gap: Val::Px(12.0),
                            margin: UiRect::bottom(Val::Px(15.0)),
                            ..default()
                        },))
                        .with_children(|tags| {
                            for tag in doctrine {
                                tags.spawn((
                                    Node {
                                        padding: UiRect::axes(Val::Px(14.0), Val::Px(8.0)),
                                        border: UiRect::all(Val::Px(2.0)),
                                        ..default()
                                    },
                                    BackgroundColor(primary.with_alpha(0.6)),
                                    BorderColor(secondary),
                                ))
                                .with_children(|tag_node| {
                                    tag_node.spawn((
                                        Text::new(tag),
                                        TextFont {
                                            font_size: 13.0,
                                            ..default()
                                        },
                                        TextColor(Color::WHITE),
                                    ));
                                });
                            }
                        });

                    // Description
                    panel.spawn((
                        Text::new(description),
                        TextFont {
                            font_size: 15.0,
                            ..default()
                        },
                        TextColor(Color::srgb(0.7, 0.7, 0.7)),
                        Node {
                            max_width: Val::Px(320.0),
                            ..default()
                        },
                    ));

                    // Selection indicator arrow
                    panel.spawn((
                        SelectionArrow { faction },
                        Text::new("▼ SELECT ▼"),
                        TextFont {
                            font_size: 20.0,
                            ..default()
                        },
                        TextColor(Color::NONE), // Hidden by default
                        Node {
                            margin: UiRect::top(Val::Px(20.0)),
                            ..default()
                        },
                    ));
                });
        });
}

#[derive(Component)]
struct SelectionArrow {
    faction: &'static str,
}

fn faction_select_input(
    keyboard: Res<ButtonInput<KeyCode>>,
    joystick: Res<JoystickState>,
    time: Res<Time>,
    mut state: ResMut<FactionSelectState>,
    mut next_state: ResMut<NextState<GameState>>,
    mut mode_state: ResMut<NextState<CGModeSelect>>,
    mut active_module: ResMut<ActiveModule>,
    mut session: ResMut<GameSession>,
    mut panels: Query<(&FactionPanel, &mut BorderColor)>,
    mut arrows: Query<(&SelectionArrow, &mut TextColor)>,
) {
    let dt = time.delta_secs();
    state.cooldown = (state.cooldown - dt).max(0.0);

    // Navigation
    if state.cooldown <= 0.0 {
        let move_left = keyboard.pressed(KeyCode::ArrowLeft)
            || keyboard.pressed(KeyCode::KeyA)
            || joystick.dpad_x < 0;
        let move_right = keyboard.pressed(KeyCode::ArrowRight)
            || keyboard.pressed(KeyCode::KeyD)
            || joystick.dpad_x > 0;

        if move_left && state.selected > 0 {
            state.selected = 0;
            state.cooldown = 0.2;
        } else if move_right && state.selected < 1 {
            state.selected = 1;
            state.cooldown = 0.2;
        }
    }

    // Update panel borders for selection
    for (panel, mut border) in panels.iter_mut() {
        let is_selected = (panel.faction == "caldari" && state.selected == 0)
            || (panel.faction == "gallente" && state.selected == 1);

        let accent = if panel.faction == "caldari" {
            COLOR_CALDARI_ACCENT
        } else {
            COLOR_GALLENTE_ACCENT
        };

        *border = if is_selected {
            BorderColor(accent)
        } else {
            BorderColor(Color::NONE)
        };
    }

    // Update selection arrows
    for (arrow, mut color) in arrows.iter_mut() {
        let is_selected = (arrow.faction == "caldari" && state.selected == 0)
            || (arrow.faction == "gallente" && state.selected == 1);

        let accent = if arrow.faction == "caldari" {
            COLOR_CALDARI_ACCENT
        } else {
            COLOR_GALLENTE_ACCENT
        };

        *color = if is_selected {
            TextColor(accent)
        } else {
            TextColor(Color::NONE)
        };
    }

    // Confirm selection
    if keyboard.just_pressed(KeyCode::Space)
        || keyboard.just_pressed(KeyCode::Enter)
        || joystick.confirm()
    {
        let (player_faction, enemy_faction) = if state.selected == 0 {
            (Faction::Caldari, Faction::Gallente)
        } else {
            (Faction::Gallente, Faction::Caldari)
        };

        // Set both ActiveModule and GameSession for compatibility
        active_module.set_faction(player_faction.short_name(), enemy_faction.short_name());
        *session = GameSession::new(player_faction, enemy_faction);

        info!(
            "Selected {} vs {}",
            player_faction.name(),
            enemy_faction.name()
        );

        // Caldari gets mode select (Campaign vs Nightmare)
        // Gallente goes directly to difficulty (no nightmare mode)
        if player_faction == Faction::Caldari {
            mode_state.set(CGModeSelect::Active);
        } else {
            next_state.set(GameState::DifficultySelect);
        }
    }

    // Back to module select
    if keyboard.just_pressed(KeyCode::Escape) || joystick.back() {
        active_module.module_id = None;
        next_state.set(GameState::ModuleSelect);
    }
}

fn despawn_faction_select(mut commands: Commands, query: Query<Entity, With<FactionSelectRoot>>) {
    for entity in query.iter() {
        commands.entity(entity).despawn_recursive();
    }
    commands.remove_resource::<FactionSelectState>();
}
