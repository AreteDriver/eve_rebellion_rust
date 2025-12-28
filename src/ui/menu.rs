//! Menu Systems
//!
//! Complete menu flow: Title -> Difficulty -> Ship -> Playing
//! Supports keyboard, mouse, and joystick input.

use crate::core::*;
use crate::games::ActiveModule;
use crate::systems::JoystickState;
use bevy::prelude::*;

/// Menu plugin
pub struct MenuPlugin;

impl Plugin for MenuPlugin {
    fn build(&self, app: &mut App) {
        app
            // Loading
            .add_systems(OnEnter(GameState::Loading), spawn_loading_screen)
            .add_systems(
                Update,
                loading_progress.run_if(in_state(GameState::Loading)),
            )
            .add_systems(OnExit(GameState::Loading), despawn_menu::<LoadingRoot>)
            // Main Menu
            .add_systems(OnEnter(GameState::MainMenu), spawn_main_menu)
            .add_systems(
                Update,
                (main_menu_input, update_menu_selection::<MainMenuRoot>)
                    .run_if(in_state(GameState::MainMenu)),
            )
            .add_systems(OnExit(GameState::MainMenu), despawn_menu::<MainMenuRoot>)
            // Difficulty Select
            .add_systems(OnEnter(GameState::DifficultySelect), spawn_difficulty_menu)
            .add_systems(
                Update,
                (
                    difficulty_menu_input,
                    update_menu_selection::<DifficultyMenuRoot>,
                )
                    .run_if(in_state(GameState::DifficultySelect)),
            )
            .add_systems(
                OnExit(GameState::DifficultySelect),
                despawn_menu::<DifficultyMenuRoot>,
            )
            // Ship Select
            .add_systems(OnEnter(GameState::ShipSelect), spawn_ship_menu)
            .add_systems(
                Update,
                (ship_menu_input, update_menu_selection::<ShipMenuRoot>)
                    .run_if(in_state(GameState::ShipSelect)),
            )
            .add_systems(OnExit(GameState::ShipSelect), despawn_menu::<ShipMenuRoot>)
            // Pause Menu
            .add_systems(OnEnter(GameState::Paused), spawn_pause_menu)
            .add_systems(Update, pause_menu_input.run_if(in_state(GameState::Paused)))
            .add_systems(OnExit(GameState::Paused), despawn_menu::<PauseMenuRoot>)
            // Game Over (Death Screen with corpse and debris)
            .add_systems(OnEnter(GameState::GameOver), spawn_death_screen)
            .add_systems(
                Update,
                (update_death_screen_animation, death_screen_input)
                    .run_if(in_state(GameState::GameOver)),
            )
            .add_systems(OnExit(GameState::GameOver), despawn_death_screen)
            // Boss Intro
            .add_systems(OnEnter(GameState::BossIntro), spawn_boss_intro)
            .add_systems(
                Update,
                boss_intro_update.run_if(in_state(GameState::BossIntro)),
            )
            .add_systems(OnExit(GameState::BossIntro), despawn_menu::<BossIntroRoot>)
            // Stage Complete
            .add_systems(OnEnter(GameState::StageComplete), spawn_stage_complete)
            .add_systems(
                Update,
                stage_complete_input.run_if(in_state(GameState::StageComplete)),
            )
            .add_systems(
                OnExit(GameState::StageComplete),
                despawn_menu::<StageCompleteRoot>,
            )
            // Victory (Campaign Complete)
            .add_systems(OnEnter(GameState::Victory), spawn_victory_screen)
            .add_systems(
                Update,
                (victory_input, update_victory_particles).run_if(in_state(GameState::Victory)),
            )
            .add_systems(OnExit(GameState::Victory), despawn_menu::<VictoryRoot>)
            // Init menu selection resource
            .init_resource::<MenuSelection>();
    }
}

// ============================================================================
// Menu Selection System (keyboard/joystick navigation)
// ============================================================================

#[derive(Resource, Default)]
struct MenuSelection {
    index: usize,
    total: usize,
    cooldown: f32,
}

const MENU_NAV_COOLDOWN: f32 = 0.15;

// ============================================================================
// Marker Components
// ============================================================================

#[derive(Component)]
struct LoadingRoot;

#[derive(Component)]
struct MainMenuRoot;

#[derive(Component)]
struct DifficultyMenuRoot;

#[derive(Component)]
struct ShipMenuRoot;

#[derive(Component)]
struct PauseMenuRoot;

#[derive(Component)]
struct GameOverRoot;

#[derive(Component)]
struct BossIntroRoot;

#[derive(Component)]
struct StageCompleteRoot;

#[derive(Component)]
struct VictoryRoot;

/// Death screen floating debris
#[derive(Component)]
struct DeathDebris {
    velocity: Vec2,
    spin: f32,
}

/// Death screen corpse
#[derive(Component)]
struct DeathCorpse {
    velocity: Vec2,
    spin: f32,
}

/// Death screen button
#[derive(Component)]
struct DeathButton {
    action: DeathAction,
}

#[derive(Clone, Copy, PartialEq)]
enum DeathAction {
    Retry,
    Exit,
}

/// Death screen selection state
#[derive(Resource)]
struct DeathSelection {
    selected: DeathAction,
}

impl Default for DeathSelection {
    fn default() -> Self {
        Self {
            selected: DeathAction::Retry,
        }
    }
}

/// Menu item that can be selected
#[derive(Component)]
struct MenuItem {
    index: usize,
}

/// Marker for selected menu item highlight
#[derive(Component)]
struct SelectionIndicator;

// ============================================================================
// Loading Screen
// ============================================================================

fn spawn_loading_screen(mut commands: Commands) {
    commands
        .spawn((
            LoadingRoot,
            Node {
                width: Val::Percent(100.0),
                height: Val::Percent(100.0),
                justify_content: JustifyContent::Center,
                align_items: AlignItems::Center,
                flex_direction: FlexDirection::Column,
                ..default()
            },
            BackgroundColor(Color::BLACK),
        ))
        .with_children(|parent| {
            parent.spawn((
                Text::new("EVE REBELLION"),
                TextFont {
                    font_size: 48.0,
                    ..default()
                },
                TextColor(COLOR_MINMATAR),
            ));

            parent.spawn((
                Text::new("Loading..."),
                TextFont {
                    font_size: 24.0,
                    ..default()
                },
                TextColor(Color::WHITE),
            ));
        });
}

fn loading_progress(
    time: Res<Time>,
    mut timer: Local<f32>,
    mut next_state: ResMut<NextState<GameState>>,
) {
    *timer += time.delta_secs();
    if *timer > 1.0 {
        next_state.set(GameState::MainMenu);
    }
}

// ============================================================================
// Main Menu
// ============================================================================

fn spawn_main_menu(mut commands: Commands, mut selection: ResMut<MenuSelection>) {
    selection.index = 0;
    selection.total = 4;

    commands
        .spawn((
            MainMenuRoot,
            Node {
                width: Val::Percent(100.0),
                height: Val::Percent(100.0),
                justify_content: JustifyContent::Center,
                align_items: AlignItems::Center,
                flex_direction: FlexDirection::Column,
                row_gap: Val::Px(20.0),
                ..default()
            },
            BackgroundColor(Color::srgba(0.0, 0.0, 0.0, 0.4)), // Semi-transparent to show background
        ))
        .with_children(|parent| {
            // Title
            parent.spawn((
                Text::new("EVE REBELLION"),
                TextFont {
                    font_size: 72.0,
                    ..default()
                },
                TextColor(COLOR_MINMATAR),
            ));

            parent.spawn((
                Text::new("MINMATAR RISING"),
                TextFont {
                    font_size: 24.0,
                    ..default()
                },
                TextColor(Color::srgb(0.6, 0.4, 0.3)),
            ));

            // Spacer
            parent.spawn(Node {
                height: Val::Px(60.0),
                ..default()
            });

            // Menu buttons
            spawn_menu_item(parent, "ELDER FLEET CAMPAIGN", 0);
            spawn_menu_item(parent, "CALDARI VS GALLENTE", 1);
            spawn_menu_item(parent, "OPTIONS", 2);
            spawn_menu_item(parent, "QUIT", 3);

            // Footer
            parent.spawn(Node {
                height: Val::Px(60.0),
                ..default()
            });

            parent.spawn((
                Text::new("Press SPACE/ENTER or A to select"),
                TextFont {
                    font_size: 14.0,
                    ..default()
                },
                TextColor(Color::srgb(0.4, 0.4, 0.4)),
            ));

            parent.spawn((
                Text::new("v0.3.0 - Multi-Campaign"),
                TextFont {
                    font_size: 12.0,
                    ..default()
                },
                TextColor(Color::srgb(0.3, 0.3, 0.3)),
            ));
        });
}

fn main_menu_input(
    keyboard: Res<ButtonInput<KeyCode>>,
    joystick: Res<JoystickState>,
    mut selection: ResMut<MenuSelection>,
    time: Res<Time>,
    mut next_state: ResMut<NextState<GameState>>,
    mut active_module: ResMut<ActiveModule>,
    mut exit: EventWriter<AppExit>,
) {
    selection.cooldown -= time.delta_secs();

    // Navigation
    let nav = get_nav_input(&keyboard, &joystick);
    if nav != 0 && selection.cooldown <= 0.0 {
        selection.index =
            (selection.index as i32 + nav).rem_euclid(selection.total as i32) as usize;
        selection.cooldown = MENU_NAV_COOLDOWN;
    }

    // Selection
    if is_confirm(&keyboard, &joystick) {
        match selection.index {
            0 => {
                // Elder Fleet Campaign (original game)
                active_module.set_module("elder_fleet");
                next_state.set(GameState::DifficultySelect);
            }
            1 => {
                // Caldari vs Gallente - go to faction select
                info!("Selected CALDARI VS GALLENTE - transitioning to FactionSelect");
                active_module.set_module("caldari_gallente");
                next_state.set(GameState::FactionSelect);
            }
            2 => {} // Options - TODO
            3 => {
                exit.send(AppExit::Success);
            }
            _ => {}
        }
    }

    // Quick quit
    if keyboard.just_pressed(KeyCode::Escape) || joystick.back() {
        exit.send(AppExit::Success);
    }
}

// ============================================================================
// Difficulty Select
// ============================================================================

fn spawn_difficulty_menu(mut commands: Commands, mut selection: ResMut<MenuSelection>) {
    selection.index = 1; // Default to Normal
    selection.total = 4;

    commands
        .spawn((
            DifficultyMenuRoot,
            Node {
                width: Val::Percent(100.0),
                height: Val::Percent(100.0),
                justify_content: JustifyContent::Center,
                align_items: AlignItems::Center,
                flex_direction: FlexDirection::Column,
                row_gap: Val::Px(15.0),
                ..default()
            },
            BackgroundColor(Color::srgba(0.0, 0.0, 0.0, 0.4)), // Semi-transparent to show background
        ))
        .with_children(|parent| {
            parent.spawn((
                Text::new("SELECT DIFFICULTY"),
                TextFont {
                    font_size: 48.0,
                    ..default()
                },
                TextColor(COLOR_MINMATAR),
            ));

            parent.spawn(Node {
                height: Val::Px(30.0),
                ..default()
            });

            // Difficulty options - EVE-themed
            for (i, diff) in Difficulty::all().iter().enumerate() {
                spawn_difficulty_item(parent, *diff, i);
            }

            parent.spawn(Node {
                height: Val::Px(30.0),
                ..default()
            });

            parent.spawn((
                Text::new("Press B/ESC to go back"),
                TextFont {
                    font_size: 14.0,
                    ..default()
                },
                TextColor(Color::srgb(0.4, 0.4, 0.4)),
            ));
        });
}

fn spawn_difficulty_item(parent: &mut ChildBuilder, diff: Difficulty, index: usize) {
    parent
        .spawn((
            MenuItem { index },
            Node {
                width: Val::Px(450.0),
                height: Val::Px(85.0),
                justify_content: JustifyContent::Center,
                align_items: AlignItems::Center,
                flex_direction: FlexDirection::Column,
                border: UiRect::all(Val::Px(2.0)),
                row_gap: Val::Px(3.0),
                ..default()
            },
            BackgroundColor(Color::srgba(0.1, 0.1, 0.1, 0.9)),
            BorderColor(Color::srgb(0.3, 0.3, 0.3)),
        ))
        .with_children(|btn| {
            // Difficulty name
            btn.spawn((
                Text::new(diff.name()),
                TextFont {
                    font_size: 28.0,
                    ..default()
                },
                TextColor(diff.color()),
            ));
            // Tagline
            btn.spawn((
                Text::new(format!("\"{}\"", diff.tagline())),
                TextFont {
                    font_size: 14.0,
                    ..default()
                },
                TextColor(Color::srgb(0.7, 0.7, 0.7)),
            ));
            // Description
            btn.spawn((
                Text::new(diff.description()),
                TextFont {
                    font_size: 12.0,
                    ..default()
                },
                TextColor(Color::srgb(0.5, 0.5, 0.5)),
            ));
        });
}

fn difficulty_menu_input(
    keyboard: Res<ButtonInput<KeyCode>>,
    joystick: Res<JoystickState>,
    mut selection: ResMut<MenuSelection>,
    mut difficulty: ResMut<Difficulty>,
    time: Res<Time>,
    mut next_state: ResMut<NextState<GameState>>,
) {
    selection.cooldown -= time.delta_secs();

    let nav = get_nav_input(&keyboard, &joystick);
    if nav != 0 && selection.cooldown <= 0.0 {
        selection.index =
            (selection.index as i32 + nav).rem_euclid(selection.total as i32) as usize;
        selection.cooldown = MENU_NAV_COOLDOWN;
    }

    if is_confirm(&keyboard, &joystick) {
        *difficulty = Difficulty::all()[selection.index.min(3)];
        info!(
            "Selected difficulty: {} - {}",
            difficulty.name(),
            difficulty.tagline()
        );
        next_state.set(GameState::ShipSelect);
    }

    if keyboard.just_pressed(KeyCode::Escape) || joystick.back() {
        next_state.set(GameState::MainMenu);
    }
}

// ============================================================================
// Ship Select
// ============================================================================

fn spawn_ship_menu(
    mut commands: Commands,
    mut selection: ResMut<MenuSelection>,
    difficulty: Res<Difficulty>,
    unlocks: Res<ShipUnlocks>,
) {
    selection.index = 0;
    selection.total = MinmatarShip::all_including_unlocks().len();

    commands
        .spawn((
            ShipMenuRoot,
            Node {
                width: Val::Percent(100.0),
                height: Val::Percent(100.0),
                justify_content: JustifyContent::Center,
                align_items: AlignItems::Center,
                flex_direction: FlexDirection::Column,
                row_gap: Val::Px(12.0),
                ..default()
            },
            BackgroundColor(Color::srgba(0.0, 0.0, 0.0, 0.4)),
        ))
        .with_children(|parent| {
            parent.spawn((
                Text::new("SELECT SHIP"),
                TextFont {
                    font_size: 48.0,
                    ..default()
                },
                TextColor(COLOR_MINMATAR),
            ));

            parent.spawn((
                Text::new(format!(
                    "Difficulty: {} - \"{}\"",
                    difficulty.name(),
                    difficulty.tagline()
                )),
                TextFont {
                    font_size: 16.0,
                    ..default()
                },
                TextColor(difficulty.color()),
            ));

            parent.spawn(Node {
                height: Val::Px(15.0),
                ..default()
            });

            // Ship options - show all including unlockables
            for (i, ship) in MinmatarShip::all_including_unlocks().iter().enumerate() {
                let is_unlocked = unlocks.is_unlocked(*ship);
                spawn_ship_item(parent, *ship, i, is_unlocked);
            }

            parent.spawn(Node {
                height: Val::Px(15.0),
                ..default()
            });

            parent.spawn((
                Text::new("Press B/ESC to go back"),
                TextFont {
                    font_size: 14.0,
                    ..default()
                },
                TextColor(Color::srgb(0.4, 0.4, 0.4)),
            ));
        });
}

fn spawn_ship_item(parent: &mut ChildBuilder, ship: MinmatarShip, index: usize, is_unlocked: bool) {
    // Colors depend on unlock state
    let name_color = if is_unlocked {
        COLOR_MINMATAR
    } else {
        Color::srgb(0.4, 0.4, 0.4)
    };
    let desc_color = if is_unlocked {
        Color::srgb(0.5, 0.5, 0.5)
    } else {
        Color::srgb(0.3, 0.3, 0.3)
    };
    let special_color = if is_unlocked {
        Color::srgb(0.4, 0.7, 0.9)
    } else {
        Color::srgb(0.3, 0.3, 0.3)
    };
    let bg_color = if is_unlocked {
        Color::srgba(0.1, 0.1, 0.1, 0.9)
    } else {
        Color::srgba(0.05, 0.05, 0.05, 0.9)
    };

    parent
        .spawn((
            MenuItem { index },
            Node {
                width: Val::Px(450.0),
                height: Val::Px(80.0),
                padding: UiRect::all(Val::Px(10.0)),
                justify_content: JustifyContent::SpaceBetween,
                align_items: AlignItems::Center,
                flex_direction: FlexDirection::Row,
                border: UiRect::all(Val::Px(2.0)),
                ..default()
            },
            BackgroundColor(bg_color),
            BorderColor(Color::srgb(0.3, 0.3, 0.3)),
        ))
        .with_children(|btn| {
            // Left side - name and description
            btn.spawn(Node {
                flex_direction: FlexDirection::Column,
                align_items: AlignItems::FlexStart,
                ..default()
            })
            .with_children(|left| {
                // Ship name with class
                let name_text = if ship.requires_unlock() {
                    format!("{} [{}]", ship.name(), ship.ship_class())
                } else {
                    ship.name().to_string()
                };
                left.spawn((
                    Text::new(name_text),
                    TextFont {
                        font_size: 22.0,
                        ..default()
                    },
                    TextColor(name_color),
                ));

                // Description or lock message
                if is_unlocked {
                    left.spawn((
                        Text::new(ship.description()),
                        TextFont {
                            font_size: 11.0,
                            ..default()
                        },
                        TextColor(desc_color),
                    ));
                    left.spawn((
                        Text::new(ship.special()),
                        TextFont {
                            font_size: 10.0,
                            ..default()
                        },
                        TextColor(special_color),
                    ));
                } else {
                    left.spawn((
                        Text::new(format!(
                            "LOCKED - Complete Act {} to unlock",
                            ship.unlock_act()
                        )),
                        TextFont {
                            font_size: 11.0,
                            ..default()
                        },
                        TextColor(Color::srgb(0.6, 0.3, 0.3)),
                    ));
                    left.spawn((
                        Text::new(ship.description()),
                        TextFont {
                            font_size: 10.0,
                            ..default()
                        },
                        TextColor(desc_color),
                    ));
                }
            });

            // Right side - stats (dimmed if locked)
            btn.spawn(Node {
                flex_direction: FlexDirection::Column,
                align_items: AlignItems::FlexEnd,
                ..default()
            })
            .with_children(|right| {
                let stat_alpha = if is_unlocked { 1.0 } else { 0.4 };
                right.spawn((
                    Text::new(format!("SPD: {:.0}%", ship.speed_mult() * 100.0)),
                    TextFont {
                        font_size: 11.0,
                        ..default()
                    },
                    TextColor(Color::srgba(0.3, 0.8, 0.3, stat_alpha)),
                ));
                right.spawn((
                    Text::new(format!("DMG: {:.0}%", ship.damage_mult() * 100.0)),
                    TextFont {
                        font_size: 11.0,
                        ..default()
                    },
                    TextColor(Color::srgba(0.9, 0.3, 0.3, stat_alpha)),
                ));
                right.spawn((
                    Text::new(format!("HP: {:.0}%", ship.health_mult() * 100.0)),
                    TextFont {
                        font_size: 11.0,
                        ..default()
                    },
                    TextColor(Color::srgba(0.3, 0.6, 0.9, stat_alpha)),
                ));
            });
        });
}

fn ship_menu_input(
    keyboard: Res<ButtonInput<KeyCode>>,
    joystick: Res<JoystickState>,
    mut selection: ResMut<MenuSelection>,
    mut selected_ship: ResMut<SelectedShip>,
    unlocks: Res<ShipUnlocks>,
    time: Res<Time>,
    mut next_state: ResMut<NextState<GameState>>,
) {
    selection.cooldown -= time.delta_secs();

    let nav = get_nav_input(&keyboard, &joystick);
    if nav != 0 && selection.cooldown <= 0.0 {
        selection.index =
            (selection.index as i32 + nav).rem_euclid(selection.total as i32) as usize;
        selection.cooldown = MENU_NAV_COOLDOWN;
    }

    if is_confirm(&keyboard, &joystick) {
        let all_ships = MinmatarShip::all_including_unlocks();
        if selection.index < all_ships.len() {
            let ship = all_ships[selection.index];
            if unlocks.is_unlocked(ship) {
                selected_ship.ship = ship;
                info!("Selected ship: {} ({})", ship.name(), ship.ship_class());
                next_state.set(GameState::Playing);
            } else {
                info!(
                    "Ship {} is locked - complete Act {} to unlock",
                    ship.name(),
                    ship.unlock_act()
                );
            }
        }
    }

    if keyboard.just_pressed(KeyCode::Escape) || joystick.back() {
        next_state.set(GameState::DifficultySelect);
    }
}

// ============================================================================
// Pause Menu
// ============================================================================

fn spawn_pause_menu(mut commands: Commands) {
    commands
        .spawn((
            PauseMenuRoot,
            Node {
                width: Val::Percent(100.0),
                height: Val::Percent(100.0),
                justify_content: JustifyContent::Center,
                align_items: AlignItems::Center,
                flex_direction: FlexDirection::Column,
                row_gap: Val::Px(20.0),
                ..default()
            },
            BackgroundColor(Color::srgba(0.0, 0.0, 0.0, 0.7)),
        ))
        .with_children(|parent| {
            parent.spawn((
                Text::new("PAUSED"),
                TextFont {
                    font_size: 64.0,
                    ..default()
                },
                TextColor(COLOR_MINMATAR),
            ));

            parent.spawn((
                Text::new("Press SPACE to resume"),
                TextFont {
                    font_size: 20.0,
                    ..default()
                },
                TextColor(Color::WHITE),
            ));

            parent.spawn((
                Text::new("Press ESC to quit to menu"),
                TextFont {
                    font_size: 16.0,
                    ..default()
                },
                TextColor(Color::srgb(0.6, 0.6, 0.6)),
            ));
        });
}

fn pause_menu_input(
    keyboard: Res<ButtonInput<KeyCode>>,
    joystick: Res<JoystickState>,
    mut next_state: ResMut<NextState<GameState>>,
) {
    if keyboard.just_pressed(KeyCode::Space)
        || keyboard.just_pressed(KeyCode::Enter)
        || joystick.confirm()
        || joystick.start()
    {
        next_state.set(GameState::Playing);
    }

    if keyboard.just_pressed(KeyCode::Escape) || joystick.back() {
        next_state.set(GameState::MainMenu);
    }
}

// ============================================================================
// Death Screen (EVE-style frozen corpse in wreckage)
// ============================================================================

/// EVE UI amber color
const COLOR_EVE_AMBER: Color = Color::srgb(0.83, 0.66, 0.29);
const COLOR_EVE_AMBER_BRIGHT: Color = Color::srgb(1.0, 0.8, 0.0);

fn spawn_death_screen(mut commands: Commands, score: Res<ScoreSystem>) {
    // Initialize selection resource
    commands.insert_resource(DeathSelection::default());

    // Spawn debris field (background sprites)
    let debris_colors = [
        Color::srgb(0.31, 0.24, 0.20), // Rusty brown
        Color::srgb(0.24, 0.24, 0.25), // Dark gray
        Color::srgb(0.35, 0.27, 0.22), // Warm rust
        Color::srgb(0.20, 0.20, 0.22), // Cold gray
    ];

    for i in 0..25 {
        let x = (fastrand::f32() - 0.5) * SCREEN_WIDTH;
        let y = (fastrand::f32() - 0.5) * SCREEN_HEIGHT;
        let size = 4.0 + fastrand::f32() * 12.0;
        let color = debris_colors[i % debris_colors.len()];

        commands.spawn((
            GameOverRoot,
            DeathDebris {
                velocity: Vec2::new((fastrand::f32() - 0.5) * 8.0, (fastrand::f32() - 0.5) * 5.0),
                spin: (fastrand::f32() - 0.5) * 0.5,
            },
            Sprite {
                color,
                custom_size: Some(Vec2::new(size * 2.0, size)),
                ..default()
            },
            Transform::from_xyz(x, y, 1.0).with_rotation(Quat::from_rotation_z(
                fastrand::f32() * std::f32::consts::TAU,
            )),
        ));
    }

    // Spawn frozen corpse (center of screen)
    commands.spawn((
        GameOverRoot,
        DeathCorpse {
            velocity: Vec2::new((fastrand::f32() - 0.5) * 3.0, (fastrand::f32() - 0.5) * 2.0),
            spin: (fastrand::f32() - 0.5) * 0.2,
        },
        Sprite {
            color: Color::srgb(0.27, 0.25, 0.24), // Frozen body color
            custom_size: Some(Vec2::new(40.0, 20.0)),
            ..default()
        },
        Transform::from_xyz(0.0, 50.0, 5.0)
            .with_rotation(Quat::from_rotation_z(fastrand::f32() * 0.5)),
    ));

    // Spawn UI overlay
    commands
        .spawn((
            GameOverRoot,
            Node {
                width: Val::Percent(100.0),
                height: Val::Percent(100.0),
                justify_content: JustifyContent::Center,
                align_items: AlignItems::Center,
                flex_direction: FlexDirection::Column,
                row_gap: Val::Px(15.0),
                ..default()
            },
            BackgroundColor(Color::srgba(0.04, 0.04, 0.07, 0.85)),
        ))
        .with_children(|parent| {
            // Title - "CLONE LOST"
            parent.spawn((
                Text::new("CLONE LOST"),
                TextFont {
                    font_size: 64.0,
                    ..default()
                },
                TextColor(COLOR_EVE_AMBER),
            ));

            // Spacer
            parent.spawn(Node {
                height: Val::Px(80.0),
                ..default()
            });

            // Final score
            parent.spawn((
                Text::new(format!("FINAL SCORE: {}", score.score)),
                TextFont {
                    font_size: 36.0,
                    ..default()
                },
                TextColor(COLOR_EVE_AMBER),
            ));

            // Souls liberated
            if score.souls_liberated > 0 {
                parent.spawn((
                    Text::new(format!("Souls Liberated: {}", score.souls_liberated)),
                    TextFont {
                        font_size: 22.0,
                        ..default()
                    },
                    TextColor(Color::srgb(0.59, 0.51, 0.35)),
                ));
            }

            // Max chain if achieved
            if score.chain > 1 {
                parent.spawn((
                    Text::new(format!("Max Chain: {}x", score.chain)),
                    TextFont {
                        font_size: 18.0,
                        ..default()
                    },
                    TextColor(Color::srgb(0.59, 0.51, 0.35)),
                ));
            }

            // Spacer
            parent.spawn(Node {
                height: Val::Px(50.0),
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
                    // RETRY button
                    row.spawn((
                        DeathButton {
                            action: DeathAction::Retry,
                        },
                        Node {
                            width: Val::Px(150.0),
                            height: Val::Px(50.0),
                            justify_content: JustifyContent::Center,
                            align_items: AlignItems::Center,
                            border: UiRect::all(Val::Px(2.0)),
                            ..default()
                        },
                        BorderColor(COLOR_EVE_AMBER),
                        BackgroundColor(Color::srgba(0.83, 0.66, 0.29, 0.1)),
                    ))
                    .with_children(|btn| {
                        btn.spawn((
                            Text::new("RETRY"),
                            TextFont {
                                font_size: 24.0,
                                ..default()
                            },
                            TextColor(COLOR_EVE_AMBER),
                        ));
                    });

                    // EXIT button
                    row.spawn((
                        DeathButton {
                            action: DeathAction::Exit,
                        },
                        Node {
                            width: Val::Px(150.0),
                            height: Val::Px(50.0),
                            justify_content: JustifyContent::Center,
                            align_items: AlignItems::Center,
                            border: UiRect::all(Val::Px(2.0)),
                            ..default()
                        },
                        BorderColor(COLOR_EVE_AMBER),
                        BackgroundColor(Color::NONE),
                    ))
                    .with_children(|btn| {
                        btn.spawn((
                            Text::new("EXIT"),
                            TextFont {
                                font_size: 24.0,
                                ..default()
                            },
                            TextColor(COLOR_EVE_AMBER),
                        ));
                    });
                });

            // Spacer
            parent.spawn(Node {
                height: Val::Px(30.0),
                ..default()
            });

            // Flavor text
            parent.spawn((
                Text::new("\"You fall... but the Fleet continues.\""),
                TextFont {
                    font_size: 14.0,
                    ..default()
                },
                TextColor(Color::srgb(0.4, 0.4, 0.4)),
            ));
        });
}

fn update_death_screen_animation(
    time: Res<Time>,
    mut debris_query: Query<(&mut Transform, &DeathDebris), Without<DeathCorpse>>,
    mut corpse_query: Query<(&mut Transform, &DeathCorpse), Without<DeathDebris>>,
    selection: Res<DeathSelection>,
    mut button_query: Query<(&DeathButton, &mut BorderColor, &mut BackgroundColor)>,
) {
    let dt = time.delta_secs();

    // Animate debris
    for (mut transform, debris) in debris_query.iter_mut() {
        transform.translation.x += debris.velocity.x * dt;
        transform.translation.y += debris.velocity.y * dt;
        transform.rotate_z(debris.spin * dt);

        // Wrap around screen
        if transform.translation.x < -SCREEN_WIDTH / 2.0 - 20.0 {
            transform.translation.x = SCREEN_WIDTH / 2.0 + 20.0;
        }
        if transform.translation.x > SCREEN_WIDTH / 2.0 + 20.0 {
            transform.translation.x = -SCREEN_WIDTH / 2.0 - 20.0;
        }
        if transform.translation.y < -SCREEN_HEIGHT / 2.0 - 20.0 {
            transform.translation.y = SCREEN_HEIGHT / 2.0 + 20.0;
        }
        if transform.translation.y > SCREEN_HEIGHT / 2.0 + 20.0 {
            transform.translation.y = -SCREEN_HEIGHT / 2.0 - 20.0;
        }
    }

    // Animate corpse (slower, more constrained)
    for (mut transform, corpse) in corpse_query.iter_mut() {
        transform.translation.x += corpse.velocity.x * dt;
        transform.translation.y += corpse.velocity.y * dt;
        transform.rotate_z(corpse.spin * dt);

        // Keep corpse near center
        if transform.translation.x.abs() > 100.0 {
            transform.translation.x *= 0.99;
        }
        if transform.translation.y.abs() > 80.0 {
            transform.translation.y *= 0.99;
        }
    }

    // Update button highlights
    for (button, mut border, mut bg) in button_query.iter_mut() {
        if button.action == selection.selected {
            border.0 = COLOR_EVE_AMBER_BRIGHT;
            bg.0 = Color::srgba(1.0, 0.8, 0.0, 0.15);
        } else {
            border.0 = COLOR_EVE_AMBER;
            bg.0 = Color::NONE;
        }
    }
}

fn death_screen_input(
    keyboard: Res<ButtonInput<KeyCode>>,
    joystick: Res<JoystickState>,
    mut selection: ResMut<DeathSelection>,
    mut next_state: ResMut<NextState<GameState>>,
    mut score: ResMut<ScoreSystem>,
    mut campaign: ResMut<CampaignState>,
) {
    // Navigation
    if keyboard.just_pressed(KeyCode::ArrowLeft)
        || keyboard.just_pressed(KeyCode::KeyA)
        || joystick.dpad_x < 0
    {
        selection.selected = DeathAction::Retry;
    }
    if keyboard.just_pressed(KeyCode::ArrowRight)
        || keyboard.just_pressed(KeyCode::KeyD)
        || joystick.dpad_x > 0
    {
        selection.selected = DeathAction::Exit;
    }

    // Confirm selection
    if keyboard.just_pressed(KeyCode::Space)
        || keyboard.just_pressed(KeyCode::Enter)
        || joystick.confirm()
    {
        match selection.selected {
            DeathAction::Retry => {
                score.reset_game();
                *campaign = CampaignState::default();
                next_state.set(GameState::ShipSelect);
            }
            DeathAction::Exit => {
                next_state.set(GameState::MainMenu);
            }
        }
    }

    // Quick exit
    if keyboard.just_pressed(KeyCode::Escape) || joystick.back() {
        next_state.set(GameState::MainMenu);
    }
}

fn despawn_death_screen(mut commands: Commands, query: Query<Entity, With<GameOverRoot>>) {
    for entity in query.iter() {
        commands.entity(entity).despawn_recursive();
    }
    commands.remove_resource::<DeathSelection>();
}

// ============================================================================
// Boss Intro Screen
// ============================================================================

fn spawn_boss_intro(mut commands: Commands, campaign: Res<CampaignState>) {
    let (boss_name, boss_title) = if let Some(mission) = campaign.current_mission() {
        (mission.boss.name(), mission.name)
    } else {
        ("UNKNOWN", "???")
    };

    commands
        .spawn((
            BossIntroRoot,
            Node {
                width: Val::Percent(100.0),
                height: Val::Percent(100.0),
                justify_content: JustifyContent::Center,
                align_items: AlignItems::Center,
                flex_direction: FlexDirection::Column,
                row_gap: Val::Px(15.0),
                ..default()
            },
            BackgroundColor(Color::srgba(0.0, 0.0, 0.0, 0.85)),
        ))
        .with_children(|parent| {
            // Warning text
            parent.spawn((
                Text::new("WARNING"),
                TextFont {
                    font_size: 24.0,
                    ..default()
                },
                TextColor(Color::srgb(1.0, 0.3, 0.3)),
            ));

            parent.spawn(Node {
                height: Val::Px(20.0),
                ..default()
            });

            // Boss name with dramatic reveal
            parent.spawn((
                Text::new(boss_name),
                TextFont {
                    font_size: 64.0,
                    ..default()
                },
                TextColor(Color::srgb(1.0, 0.85, 0.2)), // Gold for Amarr
            ));

            // Boss mission title
            parent.spawn((
                Text::new(boss_title),
                TextFont {
                    font_size: 20.0,
                    ..default()
                },
                TextColor(Color::srgb(0.8, 0.6, 0.2)),
            ));

            parent.spawn(Node {
                height: Val::Px(40.0),
                ..default()
            });

            // Flavor text
            parent.spawn((
                Text::new("Prepare for battle..."),
                TextFont {
                    font_size: 16.0,
                    ..default()
                },
                TextColor(Color::srgb(0.6, 0.6, 0.6)),
            ));
        });
}

fn boss_intro_update(time: Res<Time>, mut timer: Local<f32>) {
    // The actual state transition happens in systems/campaign.rs boss_intro_sequence
    // This just tracks time for potential animation effects
    *timer += time.delta_secs();
}

// ============================================================================
// Stage Complete Screen
// ============================================================================

fn spawn_stage_complete(
    mut commands: Commands,
    campaign: Res<CampaignState>,
    score: Res<ScoreSystem>,
) {
    let mission_name = campaign
        .current_mission()
        .map(|m| m.name)
        .unwrap_or("MISSION");

    let bonus_text = if campaign.bonus_complete {
        "BONUS OBJECTIVE COMPLETE!"
    } else if let Some(m) = campaign.current_mission() {
        m.bonus_objective.unwrap_or("")
    } else {
        ""
    };

    commands
        .spawn((
            StageCompleteRoot,
            Node {
                width: Val::Percent(100.0),
                height: Val::Percent(100.0),
                justify_content: JustifyContent::Center,
                align_items: AlignItems::Center,
                flex_direction: FlexDirection::Column,
                row_gap: Val::Px(15.0),
                ..default()
            },
            BackgroundColor(Color::srgba(0.0, 0.05, 0.0, 0.9)),
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
                TextColor(COLOR_MINMATAR),
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
                Text::new(format!("Souls Liberated: {}", campaign.mission_souls)),
                TextFont {
                    font_size: 20.0,
                    ..default()
                },
                TextColor(Color::srgb(0.4, 0.8, 1.0)),
            ));

            parent.spawn((
                Text::new(format!("Time: {:.1}s", campaign.mission_timer)),
                TextFont {
                    font_size: 18.0,
                    ..default()
                },
                TextColor(Color::srgb(0.6, 0.6, 0.6)),
            ));

            // Bonus objective
            if !bonus_text.is_empty() {
                parent.spawn(Node {
                    height: Val::Px(10.0),
                    ..default()
                });

                let bonus_color = if campaign.bonus_complete {
                    Color::srgb(1.0, 0.85, 0.2) // Gold
                } else {
                    Color::srgb(0.5, 0.5, 0.5) // Gray
                };

                parent.spawn((
                    Text::new(bonus_text),
                    TextFont {
                        font_size: 16.0,
                        ..default()
                    },
                    TextColor(bonus_color),
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
                TextColor(Color::srgb(0.7, 0.7, 0.7)),
            ));
        });
}

fn stage_complete_input(
    keyboard: Res<ButtonInput<KeyCode>>,
    joystick: Res<JoystickState>,
    mut campaign: ResMut<CampaignState>,
    mut next_state: ResMut<NextState<GameState>>,
) {
    if keyboard.just_pressed(KeyCode::Space)
        || keyboard.just_pressed(KeyCode::Enter)
        || joystick.confirm()
    {
        // Advance to next mission
        if campaign.complete_mission() {
            // More missions available
            next_state.set(GameState::Playing);
        } else {
            // Campaign complete!
            next_state.set(GameState::Victory);
        }
    }

    if keyboard.just_pressed(KeyCode::Escape) || joystick.back() {
        next_state.set(GameState::MainMenu);
    }
}

// ============================================================================
// Victory Screen (Campaign Complete)
// ============================================================================

/// Marker for victory celebration particles
#[derive(Component)]
struct VictoryParticle {
    velocity: Vec2,
    lifetime: f32,
    max_lifetime: f32,
}

fn spawn_victory_screen(
    mut commands: Commands,
    score: Res<ScoreSystem>,
    _campaign: Res<CampaignState>,
) {
    // Spawn celebration particles
    for _ in 0..60 {
        let x = (fastrand::f32() - 0.5) * SCREEN_WIDTH;
        let y = -SCREEN_HEIGHT / 2.0 - fastrand::f32() * 100.0;
        let vx = (fastrand::f32() - 0.5) * 100.0;
        let vy = 80.0 + fastrand::f32() * 120.0;
        let size = 4.0 + fastrand::f32() * 8.0;
        let lifetime = 3.0 + fastrand::f32() * 4.0;

        // Gold/amber particles
        let color = if fastrand::bool() {
            Color::srgb(1.0, 0.85, 0.2) // Gold
        } else {
            Color::srgb(0.85, 0.4, 0.15) // Minmatar rust
        };

        commands.spawn((
            VictoryRoot,
            VictoryParticle {
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
            VictoryRoot,
            Node {
                width: Val::Percent(100.0),
                height: Val::Percent(100.0),
                justify_content: JustifyContent::Center,
                align_items: AlignItems::Center,
                flex_direction: FlexDirection::Column,
                row_gap: Val::Px(15.0),
                ..default()
            },
            BackgroundColor(Color::srgba(0.0, 0.02, 0.05, 0.9)),
        ))
        .with_children(|parent| {
            // Victory header with glow effect (simulated with multiple layers)
            parent.spawn((
                Text::new("LIBERATION COMPLETE"),
                TextFont {
                    font_size: 72.0,
                    ..default()
                },
                TextColor(Color::srgb(1.0, 0.85, 0.2)), // Gold
            ));

            parent.spawn((
                Text::new("The Amarr Empire Has Fallen"),
                TextFont {
                    font_size: 28.0,
                    ..default()
                },
                TextColor(COLOR_MINMATAR),
            ));

            parent.spawn((
                Text::new("The Minmatar are FREE"),
                TextFont {
                    font_size: 22.0,
                    ..default()
                },
                TextColor(Color::srgb(0.6, 0.9, 1.0)),
            ));

            parent.spawn(Node {
                height: Val::Px(40.0),
                ..default()
            });

            // Campaign stats in a styled box
            parent
                .spawn((
                    Node {
                        padding: UiRect::all(Val::Px(20.0)),
                        flex_direction: FlexDirection::Column,
                        align_items: AlignItems::Center,
                        row_gap: Val::Px(10.0),
                        border: UiRect::all(Val::Px(2.0)),
                        ..default()
                    },
                    BorderColor(Color::srgb(0.8, 0.6, 0.2)),
                    BackgroundColor(Color::srgba(0.1, 0.08, 0.02, 0.8)),
                ))
                .with_children(|stats| {
                    stats.spawn((
                        Text::new(format!("FINAL SCORE: {:>12}", format_score(score.score))),
                        TextFont {
                            font_size: 36.0,
                            ..default()
                        },
                        TextColor(Color::srgb(1.0, 0.9, 0.3)),
                    ));

                    stats.spawn((
                        Text::new(format!("Souls Liberated: {}", score.souls_liberated)),
                        TextFont {
                            font_size: 26.0,
                            ..default()
                        },
                        TextColor(Color::srgb(0.4, 0.85, 1.0)),
                    ));

                    stats.spawn((
                        Text::new(format!("Kill Multiplier: {:.1}x", score.multiplier)),
                        TextFont {
                            font_size: 22.0,
                            ..default()
                        },
                        TextColor(Color::srgb(1.0, 0.6, 0.3)),
                    ));

                    stats.spawn((
                        Text::new(format!(
                            "Missions Completed: {}/13",
                            CampaignState::total_missions()
                        )),
                        TextFont {
                            font_size: 20.0,
                            ..default()
                        },
                        TextColor(Color::srgb(0.5, 1.0, 0.5)),
                    ));
                });

            parent.spawn(Node {
                height: Val::Px(30.0),
                ..default()
            });

            // Elder's final words
            parent.spawn((
                Text::new("\"Our ancestors smile upon us this day.\""),
                TextFont {
                    font_size: 20.0,
                    ..default()
                },
                TextColor(Color::srgb(0.7, 0.7, 0.8)),
            ));

            parent.spawn((
                Text::new("— Elder Drupar Maak"),
                TextFont {
                    font_size: 16.0,
                    ..default()
                },
                TextColor(Color::srgb(0.5, 0.5, 0.6)),
            ));

            parent.spawn(Node {
                height: Val::Px(20.0),
                ..default()
            });

            // Minmatar motto
            parent.spawn((
                Text::new("IN RUST WE TRUST"),
                TextFont {
                    font_size: 24.0,
                    ..default()
                },
                TextColor(COLOR_MINMATAR),
            ));

            parent.spawn(Node {
                height: Val::Px(30.0),
                ..default()
            });

            parent.spawn((
                Text::new("Press SPACE to return to menu"),
                TextFont {
                    font_size: 18.0,
                    ..default()
                },
                TextColor(Color::srgb(0.5, 0.5, 0.5)),
            ));
        });
}

/// Format score with commas
fn format_score(score: u64) -> String {
    let s = score.to_string();
    let mut result = String::new();
    for (i, c) in s.chars().rev().enumerate() {
        if i > 0 && i % 3 == 0 {
            result.insert(0, ',');
        }
        result.insert(0, c);
    }
    result
}

/// Update victory celebration particles
fn update_victory_particles(
    time: Res<Time>,
    mut query: Query<(&mut Transform, &mut VictoryParticle, &mut Sprite)>,
) {
    let dt = time.delta_secs();

    for (mut transform, mut particle, mut sprite) in query.iter_mut() {
        // Move particle upward
        transform.translation.x += particle.velocity.x * dt;
        transform.translation.y += particle.velocity.y * dt;

        // Add slight wave motion
        transform.translation.x += (particle.lifetime * 3.0).sin() * 20.0 * dt;

        // Slow down over time
        particle.velocity.y *= 0.995;
        particle.lifetime -= dt;

        // Fade out
        let alpha = (particle.lifetime / particle.max_lifetime).clamp(0.0, 1.0);
        sprite.color = sprite.color.with_alpha(alpha);

        // Reset if off screen or dead
        if particle.lifetime <= 0.0 || transform.translation.y > SCREEN_HEIGHT / 2.0 + 50.0 {
            transform.translation.x = (fastrand::f32() - 0.5) * SCREEN_WIDTH;
            transform.translation.y = -SCREEN_HEIGHT / 2.0 - fastrand::f32() * 50.0;
            particle.lifetime = particle.max_lifetime;
            particle.velocity.y = 80.0 + fastrand::f32() * 120.0;
        }
    }
}

fn victory_input(
    keyboard: Res<ButtonInput<KeyCode>>,
    joystick: Res<JoystickState>,
    mut next_state: ResMut<NextState<GameState>>,
    mut score: ResMut<ScoreSystem>,
    mut campaign: ResMut<CampaignState>,
) {
    if keyboard.just_pressed(KeyCode::Space)
        || keyboard.just_pressed(KeyCode::Enter)
        || joystick.confirm()
        || keyboard.just_pressed(KeyCode::Escape)
        || joystick.back()
    {
        // Reset for new game
        score.reset_game();
        *campaign = CampaignState::default();
        next_state.set(GameState::MainMenu);
    }
}

// ============================================================================
// Helper Functions
// ============================================================================

fn spawn_menu_item(parent: &mut ChildBuilder, text: &str, index: usize) {
    parent
        .spawn((
            MenuItem { index },
            Node {
                width: Val::Px(280.0),
                height: Val::Px(55.0),
                justify_content: JustifyContent::Center,
                align_items: AlignItems::Center,
                border: UiRect::all(Val::Px(2.0)),
                ..default()
            },
            BackgroundColor(Color::srgba(0.15, 0.15, 0.15, 0.9)),
            BorderColor(Color::srgb(0.3, 0.3, 0.3)),
        ))
        .with_children(|btn| {
            btn.spawn((
                Text::new(text),
                TextFont {
                    font_size: 22.0,
                    ..default()
                },
                TextColor(Color::WHITE),
            ));
        });
}

fn update_menu_selection<T: Component>(
    selection: Res<MenuSelection>,
    mut query: Query<(&MenuItem, &mut BorderColor, &mut BackgroundColor), With<T>>,
) {
    for (item, mut border, mut bg) in query.iter_mut() {
        if item.index == selection.index {
            border.0 = COLOR_MINMATAR;
            bg.0 = Color::srgba(0.25, 0.15, 0.1, 0.95);
        } else {
            border.0 = Color::srgb(0.3, 0.3, 0.3);
            bg.0 = Color::srgba(0.1, 0.1, 0.1, 0.9);
        }
    }
}

fn get_nav_input(keyboard: &ButtonInput<KeyCode>, joystick: &JoystickState) -> i32 {
    let mut nav = 0;

    // Keyboard (edge triggered)
    if keyboard.just_pressed(KeyCode::ArrowUp) || keyboard.just_pressed(KeyCode::KeyW) {
        nav = -1;
    }
    if keyboard.just_pressed(KeyCode::ArrowDown) || keyboard.just_pressed(KeyCode::KeyS) {
        nav = 1;
    }

    // Joystick dpad (edge triggered for snappy menu feel)
    if joystick.dpad_just_up() {
        nav = -1;
    }
    if joystick.dpad_just_down() {
        nav = 1;
    }

    // Analog stick (with threshold) - held state is fine since there's cooldown
    if joystick.left_y < -0.5 {
        nav = -1;
    }
    if joystick.left_y > 0.5 {
        nav = 1;
    }

    nav
}

fn is_confirm(keyboard: &ButtonInput<KeyCode>, joystick: &JoystickState) -> bool {
    keyboard.just_pressed(KeyCode::Space)
        || keyboard.just_pressed(KeyCode::Enter)
        || joystick.confirm()
}

fn despawn_menu<T: Component>(mut commands: Commands, query: Query<Entity, With<T>>) {
    for entity in query.iter() {
        commands.entity(entity).despawn_recursive();
    }
}
