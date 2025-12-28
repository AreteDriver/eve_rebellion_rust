//! Menu Systems
//!
//! Complete menu flow: Title -> Difficulty -> Ship -> Playing
//! Supports keyboard, mouse, and joystick input.

use bevy::prelude::*;
use crate::core::*;
use crate::systems::JoystickState;

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
                (difficulty_menu_input, update_menu_selection::<DifficultyMenuRoot>)
                    .run_if(in_state(GameState::DifficultySelect)),
            )
            .add_systems(OnExit(GameState::DifficultySelect), despawn_menu::<DifficultyMenuRoot>)

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
            .add_systems(
                Update,
                pause_menu_input.run_if(in_state(GameState::Paused)),
            )
            .add_systems(OnExit(GameState::Paused), despawn_menu::<PauseMenuRoot>)

            // Game Over
            .add_systems(OnEnter(GameState::GameOver), spawn_game_over)
            .add_systems(
                Update,
                game_over_input.run_if(in_state(GameState::GameOver)),
            )
            .add_systems(OnExit(GameState::GameOver), despawn_menu::<GameOverRoot>)

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
            .add_systems(OnExit(GameState::StageComplete), despawn_menu::<StageCompleteRoot>)

            // Victory (Campaign Complete)
            .add_systems(OnEnter(GameState::Victory), spawn_victory_screen)
            .add_systems(
                Update,
                victory_input.run_if(in_state(GameState::Victory)),
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
    selection.total = 3;

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
            spawn_menu_item(parent, "START GAME", 0);
            spawn_menu_item(parent, "OPTIONS", 1);
            spawn_menu_item(parent, "QUIT", 2);

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
                Text::new("v0.2.0 - Rust/Bevy"),
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
    mut exit: EventWriter<AppExit>,
) {
    selection.cooldown -= time.delta_secs();

    // Navigation
    let nav = get_nav_input(&keyboard, &joystick);
    if nav != 0 && selection.cooldown <= 0.0 {
        selection.index = (selection.index as i32 + nav)
            .rem_euclid(selection.total as i32) as usize;
        selection.cooldown = MENU_NAV_COOLDOWN;
    }

    // Selection
    if is_confirm(&keyboard, &joystick) {
        match selection.index {
            0 => next_state.set(GameState::DifficultySelect),
            1 => {} // Options - TODO
            2 => { exit.send(AppExit::Success); }
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
        selection.index = (selection.index as i32 + nav)
            .rem_euclid(selection.total as i32) as usize;
        selection.cooldown = MENU_NAV_COOLDOWN;
    }

    if is_confirm(&keyboard, &joystick) {
        *difficulty = Difficulty::all()[selection.index.min(3)];
        info!("Selected difficulty: {} - {}", difficulty.name(), difficulty.tagline());
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
                Text::new(format!("Difficulty: {} - \"{}\"", difficulty.name(), difficulty.tagline())),
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
    let name_color = if is_unlocked { COLOR_MINMATAR } else { Color::srgb(0.4, 0.4, 0.4) };
    let desc_color = if is_unlocked { Color::srgb(0.5, 0.5, 0.5) } else { Color::srgb(0.3, 0.3, 0.3) };
    let special_color = if is_unlocked { Color::srgb(0.4, 0.7, 0.9) } else { Color::srgb(0.3, 0.3, 0.3) };
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
            }).with_children(|left| {
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
                        Text::new(format!("LOCKED - Complete Act {} to unlock", ship.unlock_act())),
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
            }).with_children(|right| {
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
        selection.index = (selection.index as i32 + nav)
            .rem_euclid(selection.total as i32) as usize;
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
                info!("Ship {} is locked - complete Act {} to unlock", ship.name(), ship.unlock_act());
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
// Game Over
// ============================================================================

fn spawn_game_over(mut commands: Commands, score: Res<ScoreSystem>) {
    commands
        .spawn((
            GameOverRoot,
            Node {
                width: Val::Percent(100.0),
                height: Val::Percent(100.0),
                justify_content: JustifyContent::Center,
                align_items: AlignItems::Center,
                flex_direction: FlexDirection::Column,
                row_gap: Val::Px(20.0),
                ..default()
            },
            BackgroundColor(Color::srgba(0.1, 0.0, 0.0, 0.9)),
        ))
        .with_children(|parent| {
            parent.spawn((
                Text::new("GAME OVER"),
                TextFont {
                    font_size: 72.0,
                    ..default()
                },
                TextColor(Color::srgb(1.0, 0.2, 0.2)),
            ));

            parent.spawn((
                Text::new(format!("Final Score: {}", score.score)),
                TextFont {
                    font_size: 32.0,
                    ..default()
                },
                TextColor(Color::WHITE),
            ));

            parent.spawn((
                Text::new(format!("Souls Liberated: {}", score.souls_liberated)),
                TextFont {
                    font_size: 20.0,
                    ..default()
                },
                TextColor(Color::srgb(0.3, 0.9, 0.5)),
            ));

            parent.spawn(Node {
                height: Val::Px(30.0),
                ..default()
            });

            parent.spawn((
                Text::new("Press SPACE to try again"),
                TextFont {
                    font_size: 20.0,
                    ..default()
                },
                TextColor(Color::srgb(0.6, 0.6, 0.6)),
            ));

            parent.spawn((
                Text::new("Press ESC to return to menu"),
                TextFont {
                    font_size: 16.0,
                    ..default()
                },
                TextColor(Color::srgb(0.4, 0.4, 0.4)),
            ));
        });
}

fn game_over_input(
    keyboard: Res<ButtonInput<KeyCode>>,
    joystick: Res<JoystickState>,
    mut next_state: ResMut<NextState<GameState>>,
    mut score: ResMut<ScoreSystem>,
) {
    if keyboard.just_pressed(KeyCode::Space)
        || keyboard.just_pressed(KeyCode::Enter)
        || joystick.confirm()
    {
        score.reset_game();
        next_state.set(GameState::ShipSelect);
    }

    if keyboard.just_pressed(KeyCode::Escape) || joystick.back() {
        score.reset_game();
        next_state.set(GameState::MainMenu);
    }
}

// ============================================================================
// Boss Intro Screen
// ============================================================================

fn spawn_boss_intro(
    mut commands: Commands,
    campaign: Res<CampaignState>,
) {
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

fn boss_intro_update(
    time: Res<Time>,
    mut timer: Local<f32>,
) {
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
    let mission_name = campaign.current_mission()
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

fn spawn_victory_screen(
    mut commands: Commands,
    score: Res<ScoreSystem>,
    campaign: Res<CampaignState>,
) {
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
            BackgroundColor(Color::srgba(0.0, 0.0, 0.0, 0.95)),
        ))
        .with_children(|parent| {
            // Victory header
            parent.spawn((
                Text::new("LIBERATION COMPLETE"),
                TextFont {
                    font_size: 64.0,
                    ..default()
                },
                TextColor(Color::srgb(1.0, 0.85, 0.2)), // Gold
            ));

            parent.spawn((
                Text::new("The Amarr Empire Has Fallen"),
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

            // Campaign stats
            parent.spawn((
                Text::new(format!("Final Score: {}", score.score)),
                TextFont {
                    font_size: 32.0,
                    ..default()
                },
                TextColor(Color::WHITE),
            ));

            parent.spawn((
                Text::new(format!("Total Souls Liberated: {}", score.souls_liberated)),
                TextFont {
                    font_size: 24.0,
                    ..default()
                },
                TextColor(Color::srgb(0.4, 0.8, 1.0)),
            ));

            parent.spawn((
                Text::new(format!("Missions Completed: {}", CampaignState::total_missions())),
                TextFont {
                    font_size: 20.0,
                    ..default()
                },
                TextColor(Color::srgb(0.3, 1.0, 0.3)),
            ));

            parent.spawn(Node {
                height: Val::Px(30.0),
                ..default()
            });

            // Minmatar catchphrase
            parent.spawn((
                Text::new("In Rust We Trust - For Freedom! For the Republic!"),
                TextFont {
                    font_size: 18.0,
                    ..default()
                },
                TextColor(COLOR_MINMATAR),
            ));

            parent.spawn(Node {
                height: Val::Px(40.0),
                ..default()
            });

            parent.spawn((
                Text::new("Press SPACE to return to menu"),
                TextFont {
                    font_size: 16.0,
                    ..default()
                },
                TextColor(Color::srgb(0.6, 0.6, 0.6)),
            ));
        });
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

    // Keyboard
    if keyboard.just_pressed(KeyCode::ArrowUp) || keyboard.just_pressed(KeyCode::KeyW) {
        nav = -1;
    }
    if keyboard.just_pressed(KeyCode::ArrowDown) || keyboard.just_pressed(KeyCode::KeyS) {
        nav = 1;
    }

    // Joystick (with threshold)
    if joystick.left_y < -0.5 || joystick.dpad_y < 0 {
        nav = -1;
    }
    if joystick.left_y > 0.5 || joystick.dpad_y > 0 {
        nav = 1;
    }

    nav
}

fn is_confirm(keyboard: &ButtonInput<KeyCode>, joystick: &JoystickState) -> bool {
    keyboard.just_pressed(KeyCode::Space)
        || keyboard.just_pressed(KeyCode::Enter)
        || joystick.confirm()
}

fn despawn_menu<T: Component>(
    mut commands: Commands,
    query: Query<Entity, With<T>>,
) {
    for entity in query.iter() {
        commands.entity(entity).despawn_recursive();
    }
}
