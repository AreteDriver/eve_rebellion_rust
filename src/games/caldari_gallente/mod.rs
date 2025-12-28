//! Battle of Caldari Prime Module
//!
//! Caldari vs Gallente faction warfare over Caldari Prime.

use super::{ActiveModule, FactionInfo, GameModuleInfo, ModuleRegistry};
use crate::core::GameState;
use crate::systems::JoystickState;
use bevy::prelude::*;

pub mod campaign;
pub mod ships;

pub use ships::*;

/// Caldari/Gallente module plugin
pub struct CaldariGallentePlugin;

impl Plugin for CaldariGallentePlugin {
    fn build(&self, app: &mut App) {
        // Register module
        app.add_systems(Startup, register_module);

        // Faction select screen
        app.add_systems(OnEnter(GameState::FactionSelect), spawn_faction_select)
            .add_systems(
                Update,
                faction_select_input.run_if(in_state(GameState::FactionSelect)),
            )
            .add_systems(OnExit(GameState::FactionSelect), despawn_faction_select);

        // Initialize ship pools resource
        app.init_resource::<CaldariGallenteShips>();
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

            // Divider
            parent.spawn((
                Node {
                    width: Val::Px(4.0),
                    height: Val::Percent(100.0),
                    ..default()
                },
                BackgroundColor(Color::srgb(0.3, 0.3, 0.3)),
            ));

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
                justify_content: JustifyContent::Center,
                ..default()
            },
        ))
        .with_children(|parent| {
            parent.spawn((
                Text::new("CHOOSE YOUR SIDE"),
                TextFont {
                    font_size: 48.0,
                    ..default()
                },
                TextColor(Color::srgb(0.8, 0.8, 0.8)),
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
                Text::new("← → to select   SPACE/A to confirm   ESC/B to back"),
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

    let description = if is_caldari {
        "Corporate efficiency meets military precision.\nShield-tanked missile platforms dominate the battlefield."
    } else {
        "Freedom through firepower.\nArmor-tanked drone and blaster platforms."
    };

    parent
        .spawn((
            FactionPanel { faction },
            Node {
                width: Val::Percent(50.0),
                height: Val::Percent(100.0),
                flex_direction: FlexDirection::Column,
                align_items: AlignItems::Center,
                justify_content: JustifyContent::Center,
                padding: UiRect::all(Val::Px(40.0)),
                row_gap: Val::Px(20.0),
                ..default()
            },
            BackgroundColor(primary.with_alpha(0.3)),
        ))
        .with_children(|panel| {
            // Faction emblem placeholder (colored square)
            panel
                .spawn((
                    Node {
                        width: Val::Px(120.0),
                        height: Val::Px(120.0),
                        border: UiRect::all(Val::Px(3.0)),
                        justify_content: JustifyContent::Center,
                        align_items: AlignItems::Center,
                        ..default()
                    },
                    BackgroundColor(primary),
                    BorderColor(accent),
                ))
                .with_children(|emblem| {
                    // Faction initial
                    let initial = if is_caldari { "C" } else { "G" };
                    emblem.spawn((
                        Text::new(initial),
                        TextFont {
                            font_size: 72.0,
                            ..default()
                        },
                        TextColor(accent),
                    ));
                });

            // Faction name
            panel.spawn((
                Text::new(name),
                TextFont {
                    font_size: 36.0,
                    ..default()
                },
                TextColor(secondary),
            ));

            // Doctrine tags
            panel
                .spawn((Node {
                    flex_direction: FlexDirection::Row,
                    column_gap: Val::Px(15.0),
                    ..default()
                },))
                .with_children(|tags| {
                    for tag in doctrine {
                        tags.spawn((
                            Node {
                                padding: UiRect::axes(Val::Px(12.0), Val::Px(6.0)),
                                border: UiRect::all(Val::Px(1.0)),
                                ..default()
                            },
                            BackgroundColor(primary),
                            BorderColor(accent),
                        ))
                        .with_children(|tag_node| {
                            tag_node.spawn((
                                Text::new(tag),
                                TextFont {
                                    font_size: 14.0,
                                    ..default()
                                },
                                TextColor(accent),
                            ));
                        });
                    }
                });

            // Description
            panel.spawn((
                Text::new(description),
                TextFont {
                    font_size: 16.0,
                    ..default()
                },
                TextColor(Color::srgb(0.7, 0.7, 0.7)),
                Node {
                    max_width: Val::Px(300.0),
                    ..default()
                },
            ));
        });
}

fn faction_select_input(
    keyboard: Res<ButtonInput<KeyCode>>,
    joystick: Res<JoystickState>,
    time: Res<Time>,
    mut state: ResMut<FactionSelectState>,
    mut next_state: ResMut<NextState<GameState>>,
    mut active_module: ResMut<ActiveModule>,
    mut panels: Query<(&FactionPanel, &mut BackgroundColor)>,
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

    // Update panel highlights
    for (panel, mut bg) in panels.iter_mut() {
        let is_selected = (panel.faction == "caldari" && state.selected == 0)
            || (panel.faction == "gallente" && state.selected == 1);

        let primary = if panel.faction == "caldari" {
            COLOR_CALDARI_PRIMARY
        } else {
            COLOR_GALLENTE_PRIMARY
        };

        *bg = if is_selected {
            BackgroundColor(primary.with_alpha(0.6))
        } else {
            BackgroundColor(primary.with_alpha(0.2))
        };
    }

    // Confirm selection
    if keyboard.just_pressed(KeyCode::Space)
        || keyboard.just_pressed(KeyCode::Enter)
        || joystick.confirm()
    {
        let (player, enemy) = if state.selected == 0 {
            ("caldari", "gallente")
        } else {
            ("gallente", "caldari")
        };

        active_module.set_faction(player, enemy);
        next_state.set(GameState::DifficultySelect);
    }

    // Back to module select / main menu
    if keyboard.just_pressed(KeyCode::Escape) || joystick.back() {
        active_module.module_id = None;
        next_state.set(GameState::MainMenu);
    }
}

fn despawn_faction_select(mut commands: Commands, query: Query<Entity, With<FactionSelectRoot>>) {
    for entity in query.iter() {
        commands.entity(entity).despawn_recursive();
    }
    commands.remove_resource::<FactionSelectState>();
}
