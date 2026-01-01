//! Battle of Caldari Prime Module
//!
//! Caldari vs Gallente faction warfare over Caldari Prime.

use super::{ActiveModule, FactionInfo, GameModuleInfo, ModuleRegistry};
use crate::core::{Faction, GameSession, GameState};
use crate::systems::JoystickState;
use bevy::prelude::*;

pub mod campaign;
pub mod ships;

pub use campaign::{NightmareBoss, NightmareEvent, ShiigeruNightmare};
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

        // Nightmare mode systems
        app.add_systems(
            Update,
            (
                update_nightmare_mode,
                spawn_nightmare_enemies,
                update_nightmare_hud,
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
        }
        NightmareEvent::SpawnBoss(boss) => {
            info!("NIGHTMARE BOSS: {} - \"{}\"", boss.name(), boss.dialogue());
            // Boss spawning handled in spawn_nightmare_enemies
        }
        NightmareEvent::None => {}
    }

    // Spawn the wave or boss marker for the spawning system
    if let NightmareEvent::SpawnWave(_) = event {
        commands.spawn(NightmareSpawnRequest::Wave);
    } else if let NightmareEvent::SpawnBoss(boss) = event {
        commands.spawn(NightmareSpawnRequest::Boss(boss));
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
