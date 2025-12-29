//! Menu Systems
//!
//! Complete menu flow: Title -> Difficulty -> Ship -> Playing
//! Supports keyboard, mouse, and joystick input.

#![allow(dead_code)]

use crate::core::*;
use crate::games::ActiveModule;
use crate::systems::JoystickState;
use crate::ui::TransitionEvent;
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
            // Faction Select (unified 4-faction)
            .add_systems(OnEnter(GameState::FactionSelect), spawn_faction_select)
            .add_systems(
                Update,
                faction_select_input.run_if(in_state(GameState::FactionSelect)),
            )
            .add_systems(
                OnExit(GameState::FactionSelect),
                despawn_menu::<FactionSelectRoot>,
            )
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
                (victory_input, update_victory_particles, update_victory_buttons)
                    .run_if(in_state(GameState::Victory)),
            )
            .add_systems(OnExit(GameState::Victory), despawn_victory_screen)
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
struct FactionSelectRoot;

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

/// Victory screen button
#[derive(Component)]
struct VictoryButton {
    action: VictoryAction,
}

#[derive(Clone, Copy, PartialEq)]
enum VictoryAction {
    PlayAgain,
    MainMenu,
}

/// Victory screen selection state
#[derive(Resource)]
struct VictorySelection {
    selected: VictoryAction,
}

impl Default for VictorySelection {
    fn default() -> Self {
        Self {
            selected: VictoryAction::PlayAgain,
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
            BackgroundColor(Color::srgba(0.0, 0.0, 0.0, 0.4)),
        ))
        .with_children(|parent| {
            // Title
            parent.spawn((
                Text::new("EVE REBELLION"),
                TextFont {
                    font_size: 72.0,
                    ..default()
                },
                TextColor(Color::srgb(0.8, 0.5, 0.2)), // Orange/gold
            ));

            parent.spawn((
                Text::new("THE ELDER FLEET RISES"),
                TextFont {
                    font_size: 24.0,
                    ..default()
                },
                TextColor(Color::srgb(0.6, 0.4, 0.2)), // Bronze/copper
            ));

            // Spacer
            parent.spawn(Node {
                height: Val::Px(60.0),
                ..default()
            });

            // Menu buttons
            spawn_menu_item(parent, "PLAY", 0);
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
                Text::new("v0.5.0 - Liberation"),
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
    _active_module: ResMut<ActiveModule>,
    mut exit: EventWriter<AppExit>,
    mut transitions: EventWriter<TransitionEvent>,
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
                // PLAY - go to unified faction select
                transitions.send(TransitionEvent::to(GameState::FactionSelect));
            }
            1 => {} // Options - TODO
            2 => {
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
// Faction Select (Unified 4-Faction)
// ============================================================================

fn spawn_faction_select(
    mut commands: Commands,
    mut selection: ResMut<MenuSelection>,
    mut session: ResMut<GameSession>,
) {
    selection.index = 0;
    selection.total = 4; // 4 factions

    // Default to Minmatar vs Amarr
    *session = GameSession::new(Faction::Minmatar, Faction::Amarr);

    commands
        .spawn((
            FactionSelectRoot,
            Node {
                width: Val::Percent(100.0),
                height: Val::Percent(100.0),
                flex_direction: FlexDirection::Column,
                align_items: AlignItems::Center,
                justify_content: JustifyContent::Center,
                row_gap: Val::Px(15.0),
                ..default()
            },
            BackgroundColor(Color::srgba(0.0, 0.0, 0.0, 0.5)),
        ))
        .with_children(|parent| {
            // Title
            parent.spawn((
                Text::new("CHOOSE YOUR FACTION"),
                TextFont {
                    font_size: 48.0,
                    ..default()
                },
                TextColor(Color::srgb(0.8, 0.8, 0.8)),
            ));

            parent.spawn(Node {
                height: Val::Px(30.0),
                ..default()
            });

            // Faction grid - 2x2
            parent
                .spawn((Node {
                    flex_direction: FlexDirection::Row,
                    column_gap: Val::Px(20.0),
                    ..default()
                },))
                .with_children(|row| {
                    // Left column: Minmatar, Caldari
                    row.spawn((Node {
                        flex_direction: FlexDirection::Column,
                        row_gap: Val::Px(15.0),
                        ..default()
                    },))
                    .with_children(|col| {
                        spawn_faction_card(col, Faction::Minmatar, 0);
                        spawn_faction_card(col, Faction::Caldari, 2);
                    });

                    // Right column: Amarr, Gallente
                    row.spawn((Node {
                        flex_direction: FlexDirection::Column,
                        row_gap: Val::Px(15.0),
                        ..default()
                    },))
                    .with_children(|col| {
                        spawn_faction_card(col, Faction::Amarr, 1);
                        spawn_faction_card(col, Faction::Gallente, 3);
                    });
                });

            parent.spawn(Node {
                height: Val::Px(30.0),
                ..default()
            });

            // Instructions
            parent.spawn((
                Text::new("← → ↑ ↓ to select • SPACE/A to confirm • ESC/B to back"),
                TextFont {
                    font_size: 14.0,
                    ..default()
                },
                TextColor(Color::srgb(0.4, 0.4, 0.4)),
            ));
        });
}

fn spawn_faction_card(parent: &mut ChildBuilder, faction: Faction, index: usize) {
    let primary = faction.primary_color();
    let secondary = faction.secondary_color();

    parent
        .spawn((
            FactionSelectRoot,
            MenuItem { index },
            Node {
                width: Val::Px(280.0),
                padding: UiRect::all(Val::Px(15.0)),
                flex_direction: FlexDirection::Column,
                align_items: AlignItems::Center,
                row_gap: Val::Px(8.0),
                border: UiRect::all(Val::Px(2.0)),
                ..default()
            },
            BackgroundColor(secondary.with_alpha(0.8)),
            BorderColor(primary.with_alpha(0.5)),
        ))
        .with_children(|card| {
            // Faction name
            card.spawn((
                Text::new(faction.short_name()),
                TextFont {
                    font_size: 28.0,
                    ..default()
                },
                TextColor(primary),
            ));

            // Full name
            card.spawn((
                Text::new(faction.name()),
                TextFont {
                    font_size: 14.0,
                    ..default()
                },
                TextColor(Color::srgb(0.7, 0.7, 0.7)),
            ));

            // Tagline
            card.spawn((
                Text::new(format!("\"{}\"", faction.tagline())),
                TextFont {
                    font_size: 12.0,
                    ..default()
                },
                TextColor(Color::srgb(0.5, 0.5, 0.5)),
            ));

            // Doctrine
            card.spawn((
                Text::new(format!(
                    "{} • {}",
                    faction.weapon_type().name(),
                    match faction.tank_type() {
                        TankDoctrine::Shield => "Shield Tank",
                        TankDoctrine::Armor => "Armor Tank",
                        TankDoctrine::Speed => "Speed Tank",
                    }
                )),
                TextFont {
                    font_size: 11.0,
                    ..default()
                },
                TextColor(primary.with_alpha(0.8)),
            ));

            // Enemy faction
            card.spawn((
                Text::new(format!("vs {}", faction.rival().short_name())),
                TextFont {
                    font_size: 12.0,
                    ..default()
                },
                TextColor(faction.rival().primary_color().with_alpha(0.7)),
            ));
        });
}

fn faction_select_input(
    keyboard: Res<ButtonInput<KeyCode>>,
    joystick: Res<JoystickState>,
    mut selection: ResMut<MenuSelection>,
    mut session: ResMut<GameSession>,
    time: Res<Time>,
    mut next_state: ResMut<NextState<GameState>>,
    mut cards: Query<(&MenuItem, &mut BackgroundColor, &mut BorderColor), With<FactionSelectRoot>>,
) {
    selection.cooldown -= time.delta_secs();

    // 2D navigation for 2x2 grid
    // Layout: 0=Minmatar(top-left), 1=Amarr(top-right), 2=Caldari(bottom-left), 3=Gallente(bottom-right)
    if selection.cooldown <= 0.0 {
        let left = keyboard.pressed(KeyCode::ArrowLeft)
            || keyboard.pressed(KeyCode::KeyA)
            || joystick.dpad_x < 0;
        let right = keyboard.pressed(KeyCode::ArrowRight)
            || keyboard.pressed(KeyCode::KeyD)
            || joystick.dpad_x > 0;
        let up = keyboard.pressed(KeyCode::ArrowUp)
            || keyboard.pressed(KeyCode::KeyW)
            || joystick.dpad_y < 0;
        let down = keyboard.pressed(KeyCode::ArrowDown)
            || keyboard.pressed(KeyCode::KeyS)
            || joystick.dpad_y > 0;

        let mut new_index = selection.index;

        if left && (selection.index == 1 || selection.index == 3) {
            new_index = selection.index - 1;
        } else if right && (selection.index == 0 || selection.index == 2) {
            new_index = selection.index + 1;
        } else if up && selection.index >= 2 {
            new_index = selection.index - 2;
        } else if down && selection.index <= 1 {
            new_index = selection.index + 2;
        }

        if new_index != selection.index {
            selection.index = new_index;
            selection.cooldown = MENU_NAV_COOLDOWN;
        }
    }

    // Update card highlights
    let factions = [
        Faction::Minmatar,
        Faction::Amarr,
        Faction::Caldari,
        Faction::Gallente,
    ];

    for (item, mut bg, mut border) in cards.iter_mut() {
        let faction = factions[item.index];
        let is_selected = item.index == selection.index;

        if is_selected {
            *bg = BackgroundColor(faction.primary_color().with_alpha(0.4));
            *border = BorderColor(faction.primary_color());
        } else {
            *bg = BackgroundColor(faction.secondary_color().with_alpha(0.6));
            *border = BorderColor(faction.primary_color().with_alpha(0.3));
        }
    }

    // Confirm selection
    if is_confirm(&keyboard, &joystick) {
        let player_faction = factions[selection.index];
        let enemy_faction = player_faction.rival();

        *session = GameSession::new(player_faction, enemy_faction);
        info!(
            "Selected {} vs {}",
            player_faction.name(),
            enemy_faction.name()
        );
        next_state.set(GameState::DifficultySelect);
    }

    // Back to main menu
    if keyboard.just_pressed(KeyCode::Escape) || joystick.back() {
        next_state.set(GameState::MainMenu);
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
            DifficultyMenuRoot, // Marker for update_menu_selection query
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
    session: Res<GameSession>,
    save_data: Res<crate::core::SaveData>,
) {
    let ships = session.player_ships();
    let faction = session.player_faction;
    let enemy = session.enemy_faction;
    let faction_color = faction.primary_color();

    selection.index = 0;
    selection.total = ships.len();

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
            // Title with faction name
            parent.spawn((
                Text::new(format!("{} - SELECT SHIP", faction.short_name())),
                TextFont {
                    font_size: 42.0,
                    ..default()
                },
                TextColor(faction_color),
            ));

            parent.spawn((
                Text::new(format!(
                    "{} • {} - \"{}\"",
                    faction.weapon_type().name(),
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

            // Ship options from selected faction
            for (i, ship) in ships.iter().enumerate() {
                let is_unlocked = save_data.is_ship_unlocked(
                    ship.type_id,
                    ship.unlock_stage,
                    faction.short_name(),
                    enemy.short_name(),
                );
                spawn_ship_item_new(parent, ship, i, is_unlocked, faction_color);
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

fn spawn_ship_item_new(
    parent: &mut ChildBuilder,
    ship: &ShipDef,
    index: usize,
    is_unlocked: bool,
    faction_color: Color,
) {
    let name_color = if is_unlocked {
        faction_color
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
            ShipMenuRoot,
            MenuItem { index },
            Node {
                width: Val::Px(500.0),
                height: Val::Px(85.0),
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
                let name_text = format!("{} [{}]", ship.name, ship.class.name());
                left.spawn((
                    Text::new(name_text),
                    TextFont {
                        font_size: 22.0,
                        ..default()
                    },
                    TextColor(name_color),
                ));

                // Role
                left.spawn((
                    Text::new(ship.role),
                    TextFont {
                        font_size: 11.0,
                        ..default()
                    },
                    TextColor(desc_color),
                ));

                // Special ability or lock message
                if is_unlocked {
                    left.spawn((
                        Text::new(ship.special),
                        TextFont {
                            font_size: 10.0,
                            ..default()
                        },
                        TextColor(special_color),
                    ));
                } else {
                    left.spawn((
                        Text::new(format!("LOCKED - Complete Stage {} to unlock", ship.unlock_stage)),
                        TextFont {
                            font_size: 10.0,
                            ..default()
                        },
                        TextColor(Color::srgb(0.6, 0.3, 0.3)),
                    ));
                }
            });

            // Right side - stats
            btn.spawn(Node {
                flex_direction: FlexDirection::Column,
                align_items: AlignItems::FlexEnd,
                ..default()
            })
            .with_children(|right| {
                let stat_alpha = if is_unlocked { 1.0 } else { 0.4 };
                right.spawn((
                    Text::new(format!("SPD: {:.0}", ship.speed)),
                    TextFont {
                        font_size: 11.0,
                        ..default()
                    },
                    TextColor(Color::srgba(0.3, 0.8, 0.3, stat_alpha)),
                ));
                right.spawn((
                    Text::new(format!("DMG: {:.0}", ship.damage)),
                    TextFont {
                        font_size: 11.0,
                        ..default()
                    },
                    TextColor(Color::srgba(0.9, 0.3, 0.3, stat_alpha)),
                ));
                right.spawn((
                    Text::new(format!("HP: {:.0}", ship.health)),
                    TextFont {
                        font_size: 11.0,
                        ..default()
                    },
                    TextColor(Color::srgba(0.3, 0.6, 0.9, stat_alpha)),
                ));
                right.spawn((
                    Text::new(format!("ROF: {:.1}/s", ship.fire_rate)),
                    TextFont {
                        font_size: 11.0,
                        ..default()
                    },
                    TextColor(Color::srgba(0.9, 0.7, 0.3, stat_alpha)),
                ));
            });
        });
}

fn ship_menu_input(
    keyboard: Res<ButtonInput<KeyCode>>,
    joystick: Res<JoystickState>,
    mut selection: ResMut<MenuSelection>,
    mut session: ResMut<GameSession>,
    time: Res<Time>,
    mut transitions: EventWriter<TransitionEvent>,
    save_data: Res<crate::core::SaveData>,
) {
    selection.cooldown -= time.delta_secs();

    let nav = get_nav_input(&keyboard, &joystick);
    if nav != 0 && selection.cooldown <= 0.0 {
        selection.index =
            (selection.index as i32 + nav).rem_euclid(selection.total as i32) as usize;
        selection.cooldown = MENU_NAV_COOLDOWN;
    }

    let ships = session.player_ships();
    let faction = session.player_faction;
    let enemy = session.enemy_faction;

    if is_confirm(&keyboard, &joystick)
        && selection.index < ships.len() {
            let ship = &ships[selection.index];
            let is_unlocked = save_data.is_ship_unlocked(
                ship.type_id,
                ship.unlock_stage,
                faction.short_name(),
                enemy.short_name(),
            );

            if is_unlocked {
                session.selected_ship_index = selection.index;
                info!("Selected ship: {} ({})", ship.name, ship.class.name());
                // Slow transition into gameplay
                transitions.send(TransitionEvent::slow(GameState::Playing));
            } else {
                info!(
                    "Ship {} is locked - complete Stage {} to unlock",
                    ship.name, ship.unlock_stage
                );
            }
        }

    if keyboard.just_pressed(KeyCode::Escape) || joystick.back() {
        transitions.send(TransitionEvent::quick(GameState::DifficultySelect));
    }
}

// ============================================================================
// Pause Menu
// ============================================================================

/// Pause menu selection state
#[derive(Resource, Default)]
struct PauseSelection {
    index: usize,
}

const PAUSE_OPTIONS: &[&str] = &["RESUME", "RESTART MISSION", "QUIT TO MENU"];

fn spawn_pause_menu(
    mut commands: Commands,
    campaign: Res<CampaignState>,
    score: Res<ScoreSystem>,
    session: Res<GameSession>,
) {
    commands.insert_resource(PauseSelection::default());

    let mission_name = campaign
        .current_mission()
        .map(|m| m.name)
        .unwrap_or("MISSION");

    let faction_color = session.player_faction.primary_color();

    commands
        .spawn((
            PauseMenuRoot,
            Node {
                width: Val::Percent(100.0),
                height: Val::Percent(100.0),
                justify_content: JustifyContent::Center,
                align_items: AlignItems::Center,
                flex_direction: FlexDirection::Column,
                row_gap: Val::Px(15.0),
                ..default()
            },
            BackgroundColor(Color::srgba(0.0, 0.0, 0.05, 0.85)),
        ))
        .with_children(|parent| {
            // Title
            parent.spawn((
                Text::new("PAUSED"),
                TextFont {
                    font_size: 56.0,
                    ..default()
                },
                TextColor(faction_color),
            ));

            // Mission info
            parent.spawn((
                Text::new(mission_name),
                TextFont {
                    font_size: 18.0,
                    ..default()
                },
                TextColor(Color::srgb(0.5, 0.5, 0.5)),
            ));

            // Current stats
            parent.spawn((
                Text::new(format!(
                    "Score: {} • Souls: {}",
                    score.score, campaign.mission_souls
                )),
                TextFont {
                    font_size: 14.0,
                    ..default()
                },
                TextColor(Color::srgb(0.4, 0.6, 0.8)),
            ));

            // Spacer
            parent.spawn(Node {
                height: Val::Px(25.0),
                ..default()
            });

            // Menu options
            for (i, option) in PAUSE_OPTIONS.iter().enumerate() {
                parent
                    .spawn((
                        PauseMenuItem(i),
                        Node {
                            padding: UiRect::axes(Val::Px(30.0), Val::Px(12.0)),
                            ..default()
                        },
                        BackgroundColor(Color::srgba(0.1, 0.1, 0.1, 0.8)),
                    ))
                    .with_children(|btn| {
                        btn.spawn((
                            PauseMenuItemText(i),
                            Text::new(*option),
                            TextFont {
                                font_size: 22.0,
                                ..default()
                            },
                            TextColor(Color::srgb(0.7, 0.7, 0.7)),
                        ));
                    });
            }

            // Spacer
            parent.spawn(Node {
                height: Val::Px(20.0),
                ..default()
            });

            // Controls hint
            parent.spawn((
                Text::new("↑↓ Navigate • SPACE/A Select • ESC Resume"),
                TextFont {
                    font_size: 12.0,
                    ..default()
                },
                TextColor(Color::srgb(0.35, 0.35, 0.35)),
            ));
        });
}

#[derive(Component)]
struct PauseMenuItem(usize);

#[derive(Component)]
struct PauseMenuItemText(usize);

fn pause_menu_input(
    keyboard: Res<ButtonInput<KeyCode>>,
    joystick: Res<JoystickState>,
    mut selection: ResMut<PauseSelection>,
    mut next_state: ResMut<NextState<GameState>>,
    mut transitions: EventWriter<TransitionEvent>,
    mut item_query: Query<(&PauseMenuItem, &mut BackgroundColor)>,
    mut text_query: Query<(&PauseMenuItemText, &mut TextColor)>,
    time: Res<Time>,
    mut cooldown: Local<f32>,
) {
    *cooldown -= time.delta_secs();

    // Navigation
    let nav = get_nav_input(&keyboard, &joystick);
    if nav != 0 && *cooldown <= 0.0 {
        selection.index = (selection.index as i32 + nav).rem_euclid(PAUSE_OPTIONS.len() as i32) as usize;
        *cooldown = MENU_NAV_COOLDOWN;
    }

    // Update visual selection
    let session_faction_color = Color::srgb(0.8, 0.5, 0.2); // Default orange
    for (item, mut bg) in item_query.iter_mut() {
        if item.0 == selection.index {
            bg.0 = Color::srgba(0.3, 0.25, 0.15, 0.9);
        } else {
            bg.0 = Color::srgba(0.1, 0.1, 0.1, 0.8);
        }
    }
    for (item, mut color) in text_query.iter_mut() {
        if item.0 == selection.index {
            color.0 = session_faction_color;
        } else {
            color.0 = Color::srgb(0.6, 0.6, 0.6);
        }
    }

    // Selection
    if is_confirm(&keyboard, &joystick) {
        match selection.index {
            0 => {
                // Resume
                next_state.set(GameState::Playing);
            }
            1 => {
                // Restart Mission
                transitions.send(TransitionEvent::quick(GameState::Playing));
            }
            2 => {
                // Quit to Menu
                transitions.send(TransitionEvent::to(GameState::MainMenu));
            }
            _ => {}
        }
    }

    // Quick resume with ESC or Start
    if keyboard.just_pressed(KeyCode::Escape) || joystick.start() {
        next_state.set(GameState::Playing);
    }
}

// ============================================================================
// Death Screen (EVE-style frozen corpse in wreckage)
// ============================================================================

/// EVE UI amber color
const COLOR_EVE_AMBER: Color = Color::srgb(0.83, 0.66, 0.29);
const COLOR_EVE_AMBER_BRIGHT: Color = Color::srgb(1.0, 0.8, 0.0);

fn spawn_death_screen(
    mut commands: Commands,
    score: Res<ScoreSystem>,
    campaign: Res<CampaignState>,
    session: Res<GameSession>,
    save_data: Res<SaveData>,
) {
    // Initialize selection resource
    commands.insert_resource(DeathSelection::default());

    // Get high score for comparison
    let high_score = save_data.get_high_score(session.player_faction.name(), session.enemy_faction.name());
    let is_new_high = score.score > high_score && score.score > 0;

    // Get mission info
    let mission_name = campaign.current_mission_name();

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
                row_gap: Val::Px(12.0),
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

            // Mission failed info
            parent.spawn((
                Text::new(format!("Mission Failed: {}", mission_name)),
                TextFont {
                    font_size: 20.0,
                    ..default()
                },
                TextColor(Color::srgb(0.6, 0.4, 0.4)),
            ));

            // Spacer
            parent.spawn(Node {
                height: Val::Px(30.0),
                ..default()
            });

            // New high score banner (if achieved)
            if is_new_high {
                parent.spawn((
                    Text::new("★ NEW HIGH SCORE ★"),
                    TextFont {
                        font_size: 26.0,
                        ..default()
                    },
                    TextColor(Color::srgb(1.0, 0.9, 0.0)),
                ));
            }

            // Final score
            parent.spawn((
                Text::new(format!("FINAL SCORE: {}", format_score(score.score))),
                TextFont {
                    font_size: 36.0,
                    ..default()
                },
                TextColor(COLOR_EVE_AMBER),
            ));

            // Previous high score (if not beaten)
            if !is_new_high && high_score > 0 {
                parent.spawn((
                    Text::new(format!("High Score: {}", format_score(high_score))),
                    TextFont {
                        font_size: 18.0,
                        ..default()
                    },
                    TextColor(Color::srgb(0.5, 0.5, 0.5)),
                ));
            }

            // Stats row
            parent
                .spawn(Node {
                    flex_direction: FlexDirection::Row,
                    column_gap: Val::Px(30.0),
                    ..default()
                })
                .with_children(|row| {
                    if score.souls_liberated > 0 {
                        row.spawn((
                            Text::new(format!("Souls: {}", score.souls_liberated)),
                            TextFont {
                                font_size: 20.0,
                                ..default()
                            },
                            TextColor(Color::srgb(0.59, 0.51, 0.35)),
                        ));
                    }

                    if score.chain > 1 {
                        row.spawn((
                            Text::new(format!("Chain: {}x", score.chain)),
                            TextFont {
                                font_size: 20.0,
                                ..default()
                            },
                            TextColor(Color::srgb(0.59, 0.51, 0.35)),
                        ));
                    }

                    row.spawn((
                        Text::new(format!("Stage {}-{}", campaign.stage_number(), campaign.mission_in_stage())),
                        TextFont {
                            font_size: 20.0,
                            ..default()
                        },
                        TextColor(Color::srgb(0.59, 0.51, 0.35)),
                    ));
                });

            // Spacer
            parent.spawn(Node {
                height: Val::Px(30.0),
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
                height: Val::Px(20.0),
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
    mut score: ResMut<ScoreSystem>,
    mut campaign: ResMut<CampaignState>,
    mut transitions: EventWriter<TransitionEvent>,
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
                transitions.send(TransitionEvent::to(GameState::ShipSelect));
            }
            DeathAction::Exit => {
                transitions.send(TransitionEvent::to(GameState::MainMenu));
            }
        }
    }

    // Quick exit
    if keyboard.just_pressed(KeyCode::Escape) || joystick.back() {
        transitions.send(TransitionEvent::to(GameState::MainMenu));
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
    mut transitions: EventWriter<TransitionEvent>,
) {
    if keyboard.just_pressed(KeyCode::Space)
        || keyboard.just_pressed(KeyCode::Enter)
        || joystick.confirm()
    {
        // Advance to next mission
        if campaign.complete_mission() {
            // More missions available
            transitions.send(TransitionEvent::to(GameState::Playing));
        } else {
            // Campaign complete!
            transitions.send(TransitionEvent::slow(GameState::Victory));
        }
    }

    if keyboard.just_pressed(KeyCode::Escape) || joystick.back() {
        transitions.send(TransitionEvent::to(GameState::MainMenu));
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
    session: Res<GameSession>,
    campaign: Res<CampaignState>,
    mut save_data: ResMut<SaveData>,
) {
    // Initialize selection
    commands.insert_resource(VictorySelection::default());

    // Check for new high score
    let previous_high = save_data.get_high_score(session.player_faction.name(), session.enemy_faction.name());
    let is_new_high_score = score.score > previous_high;

    // Record the score if it's a new high
    if is_new_high_score {
        save_data.record_score(
            session.player_faction.name(),
            session.enemy_faction.name(),
            score.score,
            campaign.stage_number(),
        );
    }

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
                row_gap: Val::Px(12.0),
                ..default()
            },
            BackgroundColor(Color::srgba(0.0, 0.02, 0.05, 0.9)),
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
                    font_size: 26.0,
                    ..default()
                },
                TextColor(COLOR_MINMATAR),
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
                    BorderColor(Color::srgb(0.8, 0.6, 0.2)),
                    BackgroundColor(Color::srgba(0.1, 0.08, 0.02, 0.8)),
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
                        Text::new(format!("FINAL SCORE: {}", format_score(score.score))),
                        TextFont {
                            font_size: 32.0,
                            ..default()
                        },
                        TextColor(Color::srgb(1.0, 0.9, 0.3)),
                    ));

                    // Show previous high if not beaten
                    if !is_new_high_score && previous_high > 0 {
                        stats.spawn((
                            Text::new(format!("High Score: {}", format_score(previous_high))),
                            TextFont {
                                font_size: 18.0,
                                ..default()
                            },
                            TextColor(Color::srgb(0.6, 0.6, 0.6)),
                        ));
                    }

                    stats.spawn((
                        Text::new(format!("Souls Liberated: {}", score.souls_liberated)),
                        TextFont {
                            font_size: 24.0,
                            ..default()
                        },
                        TextColor(Color::srgb(0.4, 0.85, 1.0)),
                    ));

                    stats.spawn((
                        Text::new(format!("Kill Multiplier: {:.1}x", score.multiplier)),
                        TextFont {
                            font_size: 20.0,
                            ..default()
                        },
                        TextColor(Color::srgb(1.0, 0.6, 0.3)),
                    ));
                });

            parent.spawn(Node {
                height: Val::Px(15.0),
                ..default()
            });

            // Elder's final words
            parent.spawn((
                Text::new("\"Our ancestors smile upon us this day.\""),
                TextFont {
                    font_size: 18.0,
                    ..default()
                },
                TextColor(Color::srgb(0.7, 0.7, 0.8)),
            ));

            parent.spawn((
                Text::new("— Elder Drupar Maak"),
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
                        VictoryButton {
                            action: VictoryAction::PlayAgain,
                        },
                        Node {
                            width: Val::Px(160.0),
                            height: Val::Px(50.0),
                            justify_content: JustifyContent::Center,
                            align_items: AlignItems::Center,
                            border: UiRect::all(Val::Px(2.0)),
                            ..default()
                        },
                        BorderColor(Color::srgb(1.0, 0.85, 0.2)),
                        BackgroundColor(Color::srgba(1.0, 0.85, 0.2, 0.15)),
                    ))
                    .with_children(|btn| {
                        btn.spawn((
                            Text::new("PLAY AGAIN"),
                            TextFont {
                                font_size: 22.0,
                                ..default()
                            },
                            TextColor(Color::srgb(1.0, 0.85, 0.2)),
                        ));
                    });

                    // MAIN MENU button
                    row.spawn((
                        VictoryButton {
                            action: VictoryAction::MainMenu,
                        },
                        Node {
                            width: Val::Px(160.0),
                            height: Val::Px(50.0),
                            justify_content: JustifyContent::Center,
                            align_items: AlignItems::Center,
                            border: UiRect::all(Val::Px(2.0)),
                            ..default()
                        },
                        BorderColor(Color::srgb(1.0, 0.85, 0.2)),
                        BackgroundColor(Color::NONE),
                    ))
                    .with_children(|btn| {
                        btn.spawn((
                            Text::new("MAIN MENU"),
                            TextFont {
                                font_size: 22.0,
                                ..default()
                            },
                            TextColor(Color::srgb(1.0, 0.85, 0.2)),
                        ));
                    });
                });

            parent.spawn(Node {
                height: Val::Px(20.0),
                ..default()
            });

            // Minmatar motto
            parent.spawn((
                Text::new("IN RUST WE TRUST"),
                TextFont {
                    font_size: 20.0,
                    ..default()
                },
                TextColor(COLOR_MINMATAR),
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
    mut selection: ResMut<VictorySelection>,
    mut score: ResMut<ScoreSystem>,
    mut campaign: ResMut<CampaignState>,
    mut transitions: EventWriter<TransitionEvent>,
) {
    // Navigation (left/right for button selection)
    if keyboard.just_pressed(KeyCode::ArrowLeft)
        || keyboard.just_pressed(KeyCode::KeyA)
        || joystick.dpad_x < 0
    {
        selection.selected = VictoryAction::PlayAgain;
    }
    if keyboard.just_pressed(KeyCode::ArrowRight)
        || keyboard.just_pressed(KeyCode::KeyD)
        || joystick.dpad_x > 0
    {
        selection.selected = VictoryAction::MainMenu;
    }

    // Confirm selection
    if keyboard.just_pressed(KeyCode::Space)
        || keyboard.just_pressed(KeyCode::Enter)
        || joystick.confirm()
    {
        match selection.selected {
            VictoryAction::PlayAgain => {
                score.reset_game();
                *campaign = CampaignState::default();
                transitions.send(TransitionEvent::to(GameState::ShipSelect));
            }
            VictoryAction::MainMenu => {
                score.reset_game();
                *campaign = CampaignState::default();
                transitions.send(TransitionEvent::slow(GameState::MainMenu));
            }
        }
    }

    // Quick exit to menu
    if keyboard.just_pressed(KeyCode::Escape) || joystick.back() {
        score.reset_game();
        *campaign = CampaignState::default();
        transitions.send(TransitionEvent::slow(GameState::MainMenu));
    }
}

fn update_victory_buttons(
    selection: Res<VictorySelection>,
    mut button_query: Query<(&VictoryButton, &mut BorderColor, &mut BackgroundColor)>,
) {
    let gold = Color::srgb(1.0, 0.85, 0.2);
    let gold_bright = Color::srgb(1.0, 0.95, 0.4);

    for (button, mut border, mut bg) in button_query.iter_mut() {
        if button.action == selection.selected {
            border.0 = gold_bright;
            bg.0 = Color::srgba(1.0, 0.85, 0.2, 0.2);
        } else {
            border.0 = gold;
            bg.0 = Color::NONE;
        }
    }
}

fn despawn_victory_screen(mut commands: Commands, query: Query<Entity, With<VictoryRoot>>) {
    for entity in query.iter() {
        commands.entity(entity).despawn_recursive();
    }
    commands.remove_resource::<VictorySelection>();
}

// ============================================================================
// Helper Functions
// ============================================================================

fn spawn_menu_item(parent: &mut ChildBuilder, text: &str, index: usize) {
    parent
        .spawn((
            MainMenuRoot, // Marker for update_menu_selection query
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

    // Joystick dpad (edge triggered)
    if joystick.dpad_just_up() {
        nav = -1;
    }
    if joystick.dpad_just_down() {
        nav = 1;
    }

    // Analog stick (held state - menu cooldown prevents rapid repeat)
    // This is more reliable than edge detection for gradual analog input
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
