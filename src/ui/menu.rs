//! Menu Systems
//!
//! Complete menu flow: Title -> Difficulty -> Ship -> Playing
//! Supports keyboard, mouse, and joystick input.

#![allow(dead_code)]

use crate::core::*;
use crate::entities::boss::get_boss_for_stage;
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
            // Module Select
            .add_systems(OnEnter(GameState::ModuleSelect), spawn_module_select)
            .add_systems(
                Update,
                (
                    module_select_input,
                    update_menu_selection::<ModuleSelectRoot>,
                )
                    .run_if(in_state(GameState::ModuleSelect)),
            )
            .add_systems(
                OnExit(GameState::ModuleSelect),
                despawn_menu::<ModuleSelectRoot>,
            )
            // Options Menu
            .add_systems(OnEnter(GameState::Options), spawn_options_menu)
            .add_systems(
                Update,
                options_menu_input.run_if(in_state(GameState::Options)),
            )
            .add_systems(OnExit(GameState::Options), despawn_menu::<OptionsMenuRoot>)
            // Faction Select (unified 4-faction) - only for Elder Fleet module
            .add_systems(
                OnEnter(GameState::FactionSelect),
                spawn_faction_select.run_if(is_elder_fleet),
            )
            .add_systems(
                Update,
                faction_select_input
                    .run_if(in_state(GameState::FactionSelect))
                    .run_if(is_elder_fleet),
            )
            .add_systems(
                OnExit(GameState::FactionSelect),
                despawn_menu::<FactionSelectRoot>.run_if(is_elder_fleet),
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
            // Stage Select
            .add_systems(OnEnter(GameState::StageSelect), spawn_stage_select)
            .add_systems(
                Update,
                (stage_select_input, update_menu_selection::<StageSelectRoot>)
                    .run_if(in_state(GameState::StageSelect)),
            )
            .add_systems(
                OnExit(GameState::StageSelect),
                despawn_menu::<StageSelectRoot>,
            )
            // Ship Select
            .add_systems(OnEnter(GameState::ShipSelect), spawn_ship_menu)
            .add_systems(
                Update,
                (ship_menu_input, update_menu_selection::<ShipMenuRoot>, update_ship_detail_panel)
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
            // Boss Intro (Elder Fleet only - CG has its own)
            .add_systems(
                OnEnter(GameState::BossIntro),
                spawn_boss_intro.run_if(is_elder_fleet),
            )
            .add_systems(
                Update,
                boss_intro_update
                    .run_if(in_state(GameState::BossIntro))
                    .run_if(is_elder_fleet),
            )
            .add_systems(OnExit(GameState::BossIntro), despawn_menu::<BossIntroRoot>)
            // Stage Complete (Elder Fleet only - CG has its own)
            .add_systems(
                OnEnter(GameState::StageComplete),
                spawn_stage_complete.run_if(is_elder_fleet),
            )
            .add_systems(
                Update,
                stage_complete_input
                    .run_if(in_state(GameState::StageComplete))
                    .run_if(is_elder_fleet),
            )
            .add_systems(
                OnExit(GameState::StageComplete),
                despawn_menu::<StageCompleteRoot>.run_if(is_elder_fleet),
            )
            // Victory (Elder Fleet only - CG has its own)
            .add_systems(
                OnEnter(GameState::Victory),
                spawn_victory_screen.run_if(is_elder_fleet),
            )
            .add_systems(
                Update,
                (
                    victory_input,
                    update_victory_particles,
                    update_victory_buttons,
                )
                    .run_if(in_state(GameState::Victory))
                    .run_if(is_elder_fleet),
            )
            .add_systems(
                OnExit(GameState::Victory),
                despawn_victory_screen.run_if(is_elder_fleet),
            )
            // Endless Mode Announcements (Elder Fleet only)
            .add_systems(
                Update,
                (
                    spawn_endless_wave_announcements,
                    spawn_endless_miniboss_announcement,
                    update_endless_announcements,
                )
                    .run_if(in_state(GameState::Playing))
                    .run_if(is_elder_fleet),
            )
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
struct ModuleSelectRoot;

#[derive(Component)]
struct FactionSelectRoot;

#[derive(Component)]
struct DifficultyMenuRoot;

#[derive(Component)]
struct StageSelectRoot;

#[derive(Component)]
struct StageCard {
    stage: u32,
    locked: bool,
}

#[derive(Component)]
struct ShipMenuRoot;

#[derive(Component)]
struct PauseMenuRoot;

#[derive(Component)]
struct GameOverRoot;

#[derive(Component)]
struct BossIntroRoot;

/// Boss intro text that pulses
#[derive(Component)]
struct BossIntroWarning {
    timer: f32,
}

/// Boss intro name that fades in
#[derive(Component)]
struct BossIntroName {
    timer: f32,
}

/// Boss intro dialogue that types in
#[derive(Component)]
struct BossIntroDialogue {
    full_text: String,
    timer: f32,
}

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

fn spawn_main_menu(
    mut commands: Commands,
    mut selection: ResMut<MenuSelection>,
    save_data: Res<SaveData>,
) {
    selection.index = 0;
    selection.total = 3;

    // Get best high score across all faction pairs
    let best_score = save_data
        .high_scores
        .iter()
        .map(|hs| hs.score)
        .max()
        .unwrap_or(0);

    commands
        .spawn((
            MainMenuRoot,
            Node {
                width: Val::Percent(100.0),
                height: Val::Percent(100.0),
                justify_content: JustifyContent::Center,
                align_items: AlignItems::Center,
                flex_direction: FlexDirection::Column,
                row_gap: Val::Px(15.0),
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
                height: Val::Px(40.0),
                ..default()
            });

            // Menu buttons
            spawn_menu_item(parent, "PLAY", 0);
            spawn_menu_item(parent, "OPTIONS", 1);
            spawn_menu_item(parent, "QUIT", 2);

            // High score display
            if best_score > 0 {
                parent.spawn(Node {
                    height: Val::Px(20.0),
                    ..default()
                });

                parent
                    .spawn((
                        Node {
                            padding: UiRect::new(
                                Val::Px(20.0),
                                Val::Px(20.0),
                                Val::Px(8.0),
                                Val::Px(8.0),
                            ),
                            border: UiRect::all(Val::Px(1.0)),
                            flex_direction: FlexDirection::Column,
                            align_items: AlignItems::Center,
                            ..default()
                        },
                        BorderColor(Color::srgb(0.3, 0.25, 0.15)),
                        BackgroundColor(Color::srgba(0.1, 0.08, 0.05, 0.8)),
                    ))
                    .with_children(|score_box| {
                        score_box.spawn((
                            Text::new("HIGH SCORE"),
                            TextFont {
                                font_size: 12.0,
                                ..default()
                            },
                            TextColor(Color::srgb(0.5, 0.4, 0.3)),
                        ));
                        score_box.spawn((
                            Text::new(format_score(best_score)),
                            TextFont {
                                font_size: 24.0,
                                ..default()
                            },
                            TextColor(Color::srgb(1.0, 0.85, 0.3)),
                        ));
                    });
            }

            // Footer
            parent.spawn(Node {
                height: Val::Px(30.0),
                ..default()
            });

            parent.spawn((
                Text::new("↑↓ Navigate • A/ENTER Select • ESC Quit"),
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
                // PLAY - go to module select
                transitions.send(TransitionEvent::to(GameState::ModuleSelect));
            }
            1 => {
                // OPTIONS - go to options menu
                transitions.send(TransitionEvent::to(GameState::Options));
            }
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
// Module Select
// ============================================================================

/// Run condition: is the active module Elder Fleet (default)?
fn is_elder_fleet(active_module: Res<ActiveModule>) -> bool {
    active_module.is_elder_fleet()
}

fn spawn_module_select(mut commands: Commands, mut selection: ResMut<MenuSelection>) {
    selection.index = 0;
    selection.total = 3; // Elder Fleet, Caldari vs Gallente, Endless

    commands
        .spawn((
            ModuleSelectRoot,
            Node {
                width: Val::Percent(100.0),
                height: Val::Percent(100.0),
                flex_direction: FlexDirection::Column,
                align_items: AlignItems::Center,
                justify_content: JustifyContent::Center,
                row_gap: Val::Px(20.0),
                ..default()
            },
            BackgroundColor(Color::srgba(0.0, 0.0, 0.0, 0.5)),
        ))
        .with_children(|parent| {
            // Title
            parent.spawn((
                Text::new("SELECT CAMPAIGN"),
                TextFont {
                    font_size: 48.0,
                    ..default()
                },
                TextColor(Color::srgb(0.8, 0.8, 0.8)),
            ));

            parent.spawn(Node {
                height: Val::Px(20.0),
                ..default()
            });

            // Module cards container
            parent
                .spawn(Node {
                    flex_direction: FlexDirection::Row,
                    column_gap: Val::Px(30.0),
                    ..default()
                })
                .with_children(|row| {
                    // Elder Fleet card
                    spawn_module_card(
                        row,
                        0,
                        "THE ELDER FLEET",
                        "Minmatar Liberation",
                        "Play as any faction against their rival.\n13 missions across 3 acts.",
                        Color::srgb(0.8, 0.5, 0.2), // Minmatar orange
                        "⚔",
                    );

                    // Caldari vs Gallente card
                    spawn_module_card(
                        row,
                        1,
                        "CALDARI PRIME",
                        "Faction Warfare",
                        "Caldari vs Gallente conflict.\n5 missions of brutal combat.",
                        Color::srgb(0.2, 0.4, 0.7), // Caldari blue
                        "◆",
                    );

                    // Endless Mode card
                    spawn_module_card(
                        row,
                        2,
                        "ENDLESS",
                        "Survival Mode",
                        "Infinite waves of enemies.\nSurvive as long as you can!",
                        Color::srgb(0.7, 0.2, 0.2), // Red for danger
                        "∞",
                    );
                });

            parent.spawn(Node {
                height: Val::Px(30.0),
                ..default()
            });

            // Instructions
            parent.spawn((
                Text::new("← → Navigate • A/ENTER Select • B/ESC Back"),
                TextFont {
                    font_size: 14.0,
                    ..default()
                },
                TextColor(Color::srgb(0.4, 0.4, 0.4)),
            ));
        });
}

fn spawn_module_card(
    parent: &mut ChildBuilder,
    index: usize,
    title: &str,
    subtitle: &str,
    description: &str,
    color: Color,
    symbol: &str,
) {
    parent
        .spawn((
            MenuItem { index },
            Node {
                width: Val::Px(280.0),
                height: Val::Px(320.0),
                flex_direction: FlexDirection::Column,
                align_items: AlignItems::Center,
                padding: UiRect::all(Val::Px(20.0)),
                border: UiRect::all(Val::Px(3.0)),
                row_gap: Val::Px(12.0),
                ..default()
            },
            BackgroundColor(color.with_alpha(0.2)),
            BorderColor(color.with_alpha(0.5)),
        ))
        .with_children(|card| {
            // Symbol
            card.spawn((
                Node {
                    width: Val::Px(80.0),
                    height: Val::Px(80.0),
                    justify_content: JustifyContent::Center,
                    align_items: AlignItems::Center,
                    border: UiRect::all(Val::Px(2.0)),
                    margin: UiRect::bottom(Val::Px(10.0)),
                    ..default()
                },
                BackgroundColor(color.with_alpha(0.4)),
                BorderColor(color),
            ))
            .with_children(|emblem| {
                emblem.spawn((
                    Text::new(symbol),
                    TextFont {
                        font_size: 48.0,
                        ..default()
                    },
                    TextColor(color),
                ));
            });

            // Title
            card.spawn((
                Text::new(title),
                TextFont {
                    font_size: 24.0,
                    ..default()
                },
                TextColor(Color::WHITE),
            ));

            // Subtitle
            card.spawn((
                Text::new(subtitle),
                TextFont {
                    font_size: 16.0,
                    ..default()
                },
                TextColor(color),
            ));

            // Description
            card.spawn((
                Text::new(description),
                TextFont {
                    font_size: 14.0,
                    ..default()
                },
                TextColor(Color::srgb(0.6, 0.6, 0.6)),
                Node {
                    max_width: Val::Px(240.0),
                    ..default()
                },
            ));
        });
}

fn module_select_input(
    keyboard: Res<ButtonInput<KeyCode>>,
    joystick: Res<JoystickState>,
    mut selection: ResMut<MenuSelection>,
    mut active_module: ResMut<ActiveModule>,
    mut endless: ResMut<crate::core::EndlessMode>,
    time: Res<Time>,
    mut transitions: EventWriter<TransitionEvent>,
    mut cards: Query<(&MenuItem, &mut BackgroundColor, &mut BorderColor), With<ModuleSelectRoot>>,
) {
    selection.cooldown -= time.delta_secs();

    // Navigation
    let nav = get_nav_input(&keyboard, &joystick);
    if nav != 0 && selection.cooldown <= 0.0 {
        selection.index =
            (selection.index as i32 + nav).rem_euclid(selection.total as i32) as usize;
        selection.cooldown = MENU_NAV_COOLDOWN;
    }

    // Update card highlights
    let colors = [
        Color::srgb(0.8, 0.5, 0.2), // Elder Fleet orange
        Color::srgb(0.2, 0.4, 0.7), // Caldari blue
        Color::srgb(0.7, 0.2, 0.2), // Endless red
    ];

    for (item, mut bg, mut border) in cards.iter_mut() {
        let color = colors.get(item.index).copied().unwrap_or(colors[0]);
        let is_selected = item.index == selection.index;

        if is_selected {
            *bg = BackgroundColor(color.with_alpha(0.4));
            *border = BorderColor(color);
        } else {
            *bg = BackgroundColor(color.with_alpha(0.2));
            *border = BorderColor(color.with_alpha(0.5));
        }
    }

    // Confirm selection
    if is_confirm(&keyboard, &joystick) {
        match selection.index {
            0 => {
                // Elder Fleet
                active_module.set_module("elder_fleet");
                endless.active = false;
                info!("Selected Elder Fleet campaign");
                transitions.send(TransitionEvent::to(GameState::FactionSelect));
            }
            1 => {
                // Caldari vs Gallente
                active_module.set_module("caldari_gallente");
                endless.active = false;
                info!("Selected Caldari vs Gallente campaign");
                transitions.send(TransitionEvent::to(GameState::FactionSelect));
            }
            2 => {
                // Endless Mode
                active_module.set_module("elder_fleet"); // Use Elder Fleet enemies
                endless.active = true;
                info!("Selected ENDLESS MODE!");
                transitions.send(TransitionEvent::to(GameState::FactionSelect));
            }
            _ => {}
        }
    }

    // Back to main menu
    if keyboard.just_pressed(KeyCode::Escape) || joystick.back() {
        transitions.send(TransitionEvent::to(GameState::MainMenu));
    }
}

// ============================================================================
// Options Menu
// ============================================================================

#[derive(Component)]
struct OptionsMenuRoot;

#[derive(Component)]
struct VolumeSlider {
    setting: VolumeSetting,
}

#[derive(Component)]
struct VolumeLabel {
    setting: VolumeSetting,
}

#[derive(Clone, Copy, PartialEq, Eq)]
enum VolumeSetting {
    Master,
    Music,
    Sfx,
}

#[derive(Resource)]
struct OptionsMenuState {
    selected: usize,
    cooldown: f32,
}

impl Default for OptionsMenuState {
    fn default() -> Self {
        Self {
            selected: 0,
            cooldown: 0.0,
        }
    }
}

fn spawn_options_menu(
    mut commands: Commands,
    sound_settings: Res<crate::systems::audio::SoundSettings>,
) {
    commands.init_resource::<OptionsMenuState>();

    // Root container
    commands
        .spawn((
            OptionsMenuRoot,
            Node {
                width: Val::Percent(100.0),
                height: Val::Percent(100.0),
                flex_direction: FlexDirection::Column,
                align_items: AlignItems::Center,
                justify_content: JustifyContent::Center,
                row_gap: Val::Px(20.0),
                ..default()
            },
            BackgroundColor(Color::srgba(0.02, 0.02, 0.05, 0.95)),
        ))
        .with_children(|parent| {
            // Title
            parent.spawn((
                Text::new("OPTIONS"),
                TextFont {
                    font_size: 48.0,
                    ..default()
                },
                TextColor(Color::srgb(0.9, 0.9, 0.9)),
                Node {
                    margin: UiRect::bottom(Val::Px(40.0)),
                    ..default()
                },
            ));

            // Audio section header
            parent.spawn((
                Text::new("AUDIO"),
                TextFont {
                    font_size: 24.0,
                    ..default()
                },
                TextColor(Color::srgb(0.6, 0.6, 0.7)),
                Node {
                    margin: UiRect::bottom(Val::Px(20.0)),
                    ..default()
                },
            ));

            // Volume sliders
            spawn_volume_row(
                parent,
                "Master Volume",
                VolumeSetting::Master,
                sound_settings.master_volume,
                0,
            );
            spawn_volume_row(
                parent,
                "Music Volume",
                VolumeSetting::Music,
                sound_settings.music_volume,
                1,
            );
            spawn_volume_row(
                parent,
                "SFX Volume",
                VolumeSetting::Sfx,
                sound_settings.sfx_volume,
                2,
            );

            // Back instruction
            parent.spawn((
                Text::new("[ESC] Back   [←/→] Adjust   [↑/↓] Select"),
                TextFont {
                    font_size: 16.0,
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

fn spawn_volume_row(
    parent: &mut ChildBuilder,
    label: &str,
    setting: VolumeSetting,
    value: f32,
    index: usize,
) {
    parent
        .spawn((
            Node {
                width: Val::Px(400.0),
                flex_direction: FlexDirection::Row,
                align_items: AlignItems::Center,
                justify_content: JustifyContent::SpaceBetween,
                padding: UiRect::all(Val::Px(10.0)),
                margin: UiRect::bottom(Val::Px(10.0)),
                border: UiRect::all(Val::Px(2.0)),
                ..default()
            },
            BackgroundColor(Color::srgba(0.1, 0.1, 0.15, 0.8)),
            BorderColor(if index == 0 {
                Color::srgb(0.4, 0.6, 0.8)
            } else {
                Color::srgba(0.3, 0.3, 0.4, 0.5)
            }),
            VolumeSlider { setting },
        ))
        .with_children(|row| {
            // Label
            row.spawn((
                Text::new(label),
                TextFont {
                    font_size: 18.0,
                    ..default()
                },
                TextColor(Color::srgb(0.8, 0.8, 0.8)),
            ));

            // Value + bar container
            row.spawn((Node {
                flex_direction: FlexDirection::Row,
                align_items: AlignItems::Center,
                column_gap: Val::Px(10.0),
                ..default()
            },))
                .with_children(|value_row| {
                    // Visual bar background
                    value_row
                        .spawn((
                            Node {
                                width: Val::Px(100.0),
                                height: Val::Px(12.0),
                                ..default()
                            },
                            BackgroundColor(Color::srgb(0.15, 0.15, 0.2)),
                        ))
                        .with_children(|bar_bg| {
                            // Filled portion
                            bar_bg.spawn((
                                VolumeSlider { setting },
                                Node {
                                    width: Val::Percent(value * 100.0),
                                    height: Val::Percent(100.0),
                                    ..default()
                                },
                                BackgroundColor(Color::srgb(0.3, 0.6, 0.9)),
                            ));
                        });

                    // Percentage text
                    value_row.spawn((
                        VolumeLabel { setting },
                        Text::new(format!("{}%", (value * 100.0) as i32)),
                        TextFont {
                            font_size: 16.0,
                            ..default()
                        },
                        TextColor(Color::srgb(0.7, 0.7, 0.7)),
                    ));
                });
        });
}

fn options_menu_input(
    keyboard: Res<ButtonInput<KeyCode>>,
    joystick: Res<JoystickState>,
    time: Res<Time>,
    mut state: ResMut<OptionsMenuState>,
    mut sound_settings: ResMut<crate::systems::audio::SoundSettings>,
    mut next_state: ResMut<NextState<GameState>>,
    mut sliders: Query<(&VolumeSlider, &mut BorderColor), Without<VolumeLabel>>,
    mut bars: Query<(&VolumeSlider, &mut Node), (Without<VolumeLabel>, Without<BorderColor>)>,
    mut labels: Query<(&VolumeLabel, &mut Text)>,
) {
    let dt = time.delta_secs();
    state.cooldown = (state.cooldown - dt).max(0.0);

    // Navigation (up/down)
    if state.cooldown <= 0.0 {
        let nav = get_nav_input(&keyboard, &joystick);
        if nav != 0 {
            state.selected = (state.selected as i32 + nav).rem_euclid(3) as usize;
            state.cooldown = 0.15;
        }

        // Adjust volume (left/right)
        let adjust = if keyboard.pressed(KeyCode::ArrowLeft) || joystick.dpad_x < 0 {
            -0.05
        } else if keyboard.pressed(KeyCode::ArrowRight) || joystick.dpad_x > 0 {
            0.05
        } else {
            0.0
        };

        if adjust != 0.0 {
            let current_setting = match state.selected {
                0 => VolumeSetting::Master,
                1 => VolumeSetting::Music,
                2 => VolumeSetting::Sfx,
                _ => VolumeSetting::Master,
            };

            // Update the setting
            let new_value = match current_setting {
                VolumeSetting::Master => {
                    sound_settings.master_volume =
                        (sound_settings.master_volume + adjust).clamp(0.0, 1.0);
                    sound_settings.master_volume
                }
                VolumeSetting::Music => {
                    sound_settings.music_volume =
                        (sound_settings.music_volume + adjust).clamp(0.0, 1.0);
                    sound_settings.music_volume
                }
                VolumeSetting::Sfx => {
                    sound_settings.sfx_volume =
                        (sound_settings.sfx_volume + adjust).clamp(0.0, 1.0);
                    sound_settings.sfx_volume
                }
            };

            // Update bar width
            for (slider, mut node) in bars.iter_mut() {
                if slider.setting == current_setting {
                    node.width = Val::Percent(new_value * 100.0);
                }
            }

            // Update label
            for (label, mut text) in labels.iter_mut() {
                if label.setting == current_setting {
                    **text = format!("{}%", (new_value * 100.0) as i32);
                }
            }

            state.cooldown = 0.08;
        }
    }

    // Update selection highlighting
    for (slider, mut border) in sliders.iter_mut() {
        let is_selected = match slider.setting {
            VolumeSetting::Master => state.selected == 0,
            VolumeSetting::Music => state.selected == 1,
            VolumeSetting::Sfx => state.selected == 2,
        };
        *border = if is_selected {
            BorderColor(Color::srgb(0.4, 0.6, 0.8))
        } else {
            BorderColor(Color::srgba(0.3, 0.3, 0.4, 0.5))
        };
    }

    // Back to main menu
    if keyboard.just_pressed(KeyCode::Escape) || joystick.back() {
        next_state.set(GameState::MainMenu);
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
                Text::new("← → ↑ ↓ Navigate • A/ENTER Select • B/ESC Back"),
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
    let rival = faction.rival();
    let ship_count = faction.player_ships().len();

    // Get first line of lore for preview
    let lore_preview = faction.story_intro().lines().next().unwrap_or("");
    let lore_short = if lore_preview.len() > 60 {
        format!("{}...", &lore_preview[..57])
    } else {
        lore_preview.to_string()
    };

    parent
        .spawn((
            FactionSelectRoot,
            MenuItem { index },
            Node {
                width: Val::Px(320.0),
                padding: UiRect::all(Val::Px(15.0)),
                flex_direction: FlexDirection::Column,
                align_items: AlignItems::FlexStart,
                row_gap: Val::Px(6.0),
                border: UiRect::all(Val::Px(3.0)),
                ..default()
            },
            BackgroundColor(Color::srgba(0.05, 0.05, 0.08, 0.95)),
            BorderColor(primary.with_alpha(0.4)),
        ))
        .with_children(|card| {
            // Header row: Faction name + emblem placeholder
            card.spawn(Node {
                width: Val::Percent(100.0),
                justify_content: JustifyContent::SpaceBetween,
                align_items: AlignItems::Center,
                ..default()
            })
            .with_children(|header| {
                header.spawn((
                    Text::new(faction.short_name()),
                    TextFont {
                        font_size: 32.0,
                        ..default()
                    },
                    TextColor(primary),
                ));

                // Faction emblem placeholder (colored square)
                header.spawn((
                    Node {
                        width: Val::Px(40.0),
                        height: Val::Px(40.0),
                        border: UiRect::all(Val::Px(2.0)),
                        ..default()
                    },
                    BackgroundColor(secondary.with_alpha(0.6)),
                    BorderColor(primary.with_alpha(0.8)),
                ));
            });

            // Full name
            card.spawn((
                Text::new(faction.name()),
                TextFont {
                    font_size: 13.0,
                    ..default()
                },
                TextColor(Color::srgb(0.6, 0.6, 0.6)),
            ));

            // Divider
            card.spawn((
                Node {
                    width: Val::Percent(100.0),
                    height: Val::Px(1.0),
                    margin: UiRect::vertical(Val::Px(4.0)),
                    ..default()
                },
                BackgroundColor(primary.with_alpha(0.3)),
            ));

            // Tagline
            card.spawn((
                Text::new(format!("\"{}\"", faction.tagline())),
                TextFont {
                    font_size: 14.0,
                    ..default()
                },
                TextColor(Color::srgb(0.7, 0.7, 0.7)),
            ));

            // Lore preview
            card.spawn((
                Text::new(lore_short),
                TextFont {
                    font_size: 11.0,
                    ..default()
                },
                TextColor(Color::srgb(0.45, 0.45, 0.5)),
            ));

            // Spacer
            card.spawn(Node {
                height: Val::Px(6.0),
                ..default()
            });

            // Combat stats row
            card.spawn(Node {
                width: Val::Percent(100.0),
                flex_direction: FlexDirection::Row,
                justify_content: JustifyContent::SpaceBetween,
                ..default()
            })
            .with_children(|stats| {
                // Weapon doctrine
                stats.spawn((
                    Text::new(faction.weapon_type().name()),
                    TextFont {
                        font_size: 12.0,
                        ..default()
                    },
                    TextColor(faction.weapon_type().bullet_color()),
                ));

                // Tank doctrine
                let tank_text = match faction.tank_type() {
                    TankDoctrine::Shield => "Shield",
                    TankDoctrine::Armor => "Armor",
                    TankDoctrine::Speed => "Speed",
                };
                stats.spawn((
                    Text::new(tank_text),
                    TextFont {
                        font_size: 12.0,
                        ..default()
                    },
                    TextColor(Color::srgb(0.5, 0.7, 0.9)),
                ));

                // Ship count
                stats.spawn((
                    Text::new(format!("{} Ships", ship_count)),
                    TextFont {
                        font_size: 12.0,
                        ..default()
                    },
                    TextColor(Color::srgb(0.5, 0.5, 0.5)),
                ));
            });

            // Divider
            card.spawn((
                Node {
                    width: Val::Percent(100.0),
                    height: Val::Px(1.0),
                    margin: UiRect::vertical(Val::Px(4.0)),
                    ..default()
                },
                BackgroundColor(rival.primary_color().with_alpha(0.3)),
            ));

            // Enemy faction row
            card.spawn(Node {
                width: Val::Percent(100.0),
                flex_direction: FlexDirection::Row,
                align_items: AlignItems::Center,
                column_gap: Val::Px(8.0),
                ..default()
            })
            .with_children(|enemy_row| {
                enemy_row.spawn((
                    Text::new("ENEMY:"),
                    TextFont {
                        font_size: 10.0,
                        ..default()
                    },
                    TextColor(Color::srgb(0.4, 0.4, 0.4)),
                ));

                enemy_row.spawn((
                    Text::new(rival.short_name()),
                    TextFont {
                        font_size: 14.0,
                        ..default()
                    },
                    TextColor(rival.primary_color()),
                ));

                enemy_row.spawn((
                    Text::new(format!("({})", rival.weapon_type().name())),
                    TextFont {
                        font_size: 11.0,
                        ..default()
                    },
                    TextColor(rival.primary_color().with_alpha(0.6)),
                ));
            });
        });
}

fn faction_select_input(
    keyboard: Res<ButtonInput<KeyCode>>,
    joystick: Res<JoystickState>,
    mut selection: ResMut<MenuSelection>,
    mut session: ResMut<GameSession>,
    endless: Res<crate::core::EndlessMode>,
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

        // Endless mode skips stage select, goes to difficulty
        // Campaign mode goes to stage select
        if endless.active {
            next_state.set(GameState::DifficultySelect);
        } else {
            next_state.set(GameState::StageSelect);
        }
    }

    // Back to module select
    if keyboard.just_pressed(KeyCode::Escape) || joystick.back() {
        next_state.set(GameState::ModuleSelect);
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
                Text::new("↑↓ Navigate • A/ENTER Select • B/ESC Back"),
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
// Stage Select - 13 Stages across 3 Acts
// ============================================================================

/// Stage definition for UI display
struct StageInfo {
    stage: u32,
    name: &'static str,
    act: u32,
    boss_name: &'static str,
    waves: u32,
}

const STAGES: [StageInfo; 13] = [
    // Act 1: The Call
    StageInfo { stage: 1, name: "Border Patrol", act: 1, boss_name: "Bestower", waves: 5 },
    StageInfo { stage: 2, name: "Slave Convoy", act: 1, boss_name: "Navy Omen", waves: 6 },
    StageInfo { stage: 3, name: "Outpost Assault", act: 1, boss_name: "Platform", waves: 7 },
    StageInfo { stage: 4, name: "Holder's Guard", act: 1, boss_name: "Mixed Fleet", waves: 8 },
    // Act 2: The Storm
    StageInfo { stage: 5, name: "Deep Patrol", act: 2, boss_name: "Prophecy", waves: 8 },
    StageInfo { stage: 6, name: "Inquisitor", act: 2, boss_name: "Prophecy", waves: 9 },
    StageInfo { stage: 7, name: "Strike Group", act: 2, boss_name: "Harbinger", waves: 10 },
    StageInfo { stage: 8, name: "Gate Defense", act: 2, boss_name: "Stargate", waves: 10 },
    StageInfo { stage: 9, name: "Station Core", act: 2, boss_name: "Station", waves: 12 },
    // Act 3: Liberation
    StageInfo { stage: 10, name: "Admiral's Ship", act: 3, boss_name: "Armageddon", waves: 12 },
    StageInfo { stage: 11, name: "Divine Carrier", act: 3, boss_name: "Archon", waves: 14 },
    StageInfo { stage: 12, name: "Lord Admiral", act: 3, boss_name: "Apocalypse", waves: 15 },
    StageInfo { stage: 13, name: "Avatar Titan", act: 3, boss_name: "AVATAR", waves: 20 },
];

fn spawn_stage_select(
    mut commands: Commands,
    mut selection: ResMut<MenuSelection>,
    session: Res<GameSession>,
    save_data: Res<crate::core::SaveData>,
) {
    let faction = session.player_faction;
    let enemy = session.enemy_faction;
    let highest = save_data.get_highest_stage(faction.short_name(), enemy.short_name());

    selection.index = 0;
    selection.total = 13;

    commands
        .spawn((
            StageSelectRoot,
            Node {
                width: Val::Percent(100.0),
                height: Val::Percent(100.0),
                flex_direction: FlexDirection::Column,
                align_items: AlignItems::Center,
                justify_content: JustifyContent::Center,
                row_gap: Val::Px(20.0),
                padding: UiRect::all(Val::Px(30.0)),
                ..default()
            },
            BackgroundColor(Color::srgba(0.0, 0.0, 0.0, 0.6)),
        ))
        .with_children(|parent| {
            // Title
            parent.spawn((
                Text::new("SELECT STAGE"),
                TextFont {
                    font_size: 42.0,
                    ..default()
                },
                TextColor(Color::srgb(0.9, 0.7, 0.3)),
            ));

            // Progress indicator
            parent.spawn((
                Text::new(format!("Progress: Stage {} / 13", highest.min(13))),
                TextFont {
                    font_size: 16.0,
                    ..default()
                },
                TextColor(Color::srgb(0.5, 0.5, 0.5)),
            ));

            parent.spawn(Node {
                height: Val::Px(10.0),
                ..default()
            });

            // Act 1 row
            spawn_act_row(parent, 1, "THE CALL", &STAGES[0..4], highest);

            // Act 2 row
            spawn_act_row(parent, 2, "THE STORM", &STAGES[4..9], highest);

            // Act 3 row
            spawn_act_row(parent, 3, "LIBERATION", &STAGES[9..13], highest);

            parent.spawn(Node {
                height: Val::Px(15.0),
                ..default()
            });

            // Instructions
            parent.spawn((
                Text::new("← → ↑ ↓ Navigate • A/ENTER Select • B/ESC Back"),
                TextFont {
                    font_size: 14.0,
                    ..default()
                },
                TextColor(Color::srgb(0.4, 0.4, 0.4)),
            ));
        });
}

fn spawn_act_row(
    parent: &mut ChildBuilder,
    act: u32,
    act_name: &str,
    stages: &[StageInfo],
    highest_cleared: u32,
) {
    let act_color = match act {
        1 => Color::srgb(0.8, 0.5, 0.2), // Orange - Rifter
        2 => Color::srgb(0.6, 0.3, 0.1), // Brown - Wolf
        3 => Color::srgb(0.9, 0.7, 0.3), // Gold - Jaguar
        _ => Color::WHITE,
    };

    parent
        .spawn(Node {
            flex_direction: FlexDirection::Column,
            align_items: AlignItems::Center,
            row_gap: Val::Px(8.0),
            ..default()
        })
        .with_children(|act_col| {
            // Act header
            act_col.spawn((
                Text::new(format!("ACT {}: {}", act, act_name)),
                TextFont {
                    font_size: 18.0,
                    ..default()
                },
                TextColor(act_color),
            ));

            // Stage cards row
            act_col
                .spawn(Node {
                    flex_direction: FlexDirection::Row,
                    column_gap: Val::Px(12.0),
                    ..default()
                })
                .with_children(|row| {
                    for stage in stages {
                        let locked = stage.stage > highest_cleared + 1;
                        spawn_stage_card(row, stage, locked, act_color);
                    }
                });
        });
}

fn spawn_stage_card(
    parent: &mut ChildBuilder,
    stage: &StageInfo,
    locked: bool,
    act_color: Color,
) {
    let bg_color = if locked {
        Color::srgba(0.2, 0.2, 0.2, 0.8)
    } else {
        act_color.with_alpha(0.3)
    };

    let border_color = if locked {
        Color::srgb(0.3, 0.3, 0.3)
    } else {
        act_color
    };

    let text_color = if locked {
        Color::srgb(0.4, 0.4, 0.4)
    } else {
        Color::WHITE
    };

    parent
        .spawn((
            MenuItem { index: (stage.stage - 1) as usize },
            StageCard { stage: stage.stage, locked },
            Node {
                width: Val::Px(130.0),
                height: Val::Px(100.0),
                flex_direction: FlexDirection::Column,
                align_items: AlignItems::Center,
                justify_content: JustifyContent::Center,
                padding: UiRect::all(Val::Px(8.0)),
                border: UiRect::all(Val::Px(2.0)),
                row_gap: Val::Px(4.0),
                ..default()
            },
            BackgroundColor(bg_color),
            BorderColor(border_color),
        ))
        .with_children(|card| {
            // Stage number
            card.spawn((
                Text::new(format!("STAGE {}", stage.stage)),
                TextFont {
                    font_size: 14.0,
                    ..default()
                },
                TextColor(if locked { Color::srgb(0.5, 0.5, 0.5) } else { act_color }),
            ));

            // Stage name
            card.spawn((
                Text::new(stage.name),
                TextFont {
                    font_size: 12.0,
                    ..default()
                },
                TextColor(text_color),
            ));

            // Boss or lock icon
            if locked {
                card.spawn((
                    Text::new("🔒"),
                    TextFont {
                        font_size: 18.0,
                        ..default()
                    },
                    TextColor(Color::srgb(0.5, 0.5, 0.5)),
                ));
            } else {
                card.spawn((
                    Text::new(format!("vs {}", stage.boss_name)),
                    TextFont {
                        font_size: 10.0,
                        ..default()
                    },
                    TextColor(Color::srgb(0.6, 0.6, 0.6)),
                ));
            }
        });
}

fn stage_select_input(
    keyboard: Res<ButtonInput<KeyCode>>,
    joystick: Res<JoystickState>,
    mut selection: ResMut<MenuSelection>,
    mut campaign: ResMut<CampaignState>,
    session: Res<GameSession>,
    save_data: Res<crate::core::SaveData>,
    time: Res<Time>,
    mut next_state: ResMut<NextState<GameState>>,
    mut cards: Query<(&MenuItem, &StageCard, &mut BackgroundColor, &mut BorderColor)>,
) {
    selection.cooldown -= time.delta_secs();

    let faction = session.player_faction;
    let enemy = session.enemy_faction;
    let highest = save_data.get_highest_stage(faction.short_name(), enemy.short_name());

    // Navigation - horizontal within acts
    let nav_h = get_nav_input(&keyboard, &joystick);
    // Vertical navigation between acts
    let nav_v = if keyboard.just_pressed(KeyCode::ArrowUp) || joystick.dpad_just_up() {
        -1
    } else if keyboard.just_pressed(KeyCode::ArrowDown) || joystick.dpad_just_down() {
        1
    } else {
        0
    };

    if selection.cooldown <= 0.0 {
        if nav_h != 0 {
            // Move within the same row where possible
            let new_idx = (selection.index as i32 + nav_h).clamp(0, 12) as usize;
            selection.index = new_idx;
            selection.cooldown = MENU_NAV_COOLDOWN;
        }
        if nav_v != 0 {
            // Jump between acts (4 stages in act 1, 5 in act 2, 4 in act 3)
            let current = selection.index;
            let new_idx = if nav_v < 0 {
                // Up
                match current {
                    0..=3 => current,           // Act 1, stay
                    4..=8 => current - 4,       // Act 2 -> Act 1
                    9..=12 => current - 5,      // Act 3 -> Act 2
                    _ => current,
                }
            } else {
                // Down
                match current {
                    0..=3 => (current + 4).min(8),  // Act 1 -> Act 2
                    4..=8 => (current + 5).min(12), // Act 2 -> Act 3
                    9..=12 => current,              // Act 3, stay
                    _ => current,
                }
            };
            selection.index = new_idx;
            selection.cooldown = MENU_NAV_COOLDOWN;
        }
    }

    // Update card highlights
    let act_colors = [
        Color::srgb(0.8, 0.5, 0.2), // Act 1
        Color::srgb(0.6, 0.3, 0.1), // Act 2
        Color::srgb(0.9, 0.7, 0.3), // Act 3
    ];

    for (item, card, mut bg, mut border) in cards.iter_mut() {
        let act_idx = if card.stage <= 4 { 0 } else if card.stage <= 9 { 1 } else { 2 };
        let act_color = act_colors[act_idx];
        let is_selected = item.index == selection.index;

        if card.locked {
            if is_selected {
                *bg = BackgroundColor(Color::srgba(0.3, 0.2, 0.2, 0.9));
                *border = BorderColor(Color::srgb(0.5, 0.3, 0.3));
            } else {
                *bg = BackgroundColor(Color::srgba(0.2, 0.2, 0.2, 0.8));
                *border = BorderColor(Color::srgb(0.3, 0.3, 0.3));
            }
        } else if is_selected {
            *bg = BackgroundColor(act_color.with_alpha(0.6));
            *border = BorderColor(Color::WHITE);
        } else {
            *bg = BackgroundColor(act_color.with_alpha(0.3));
            *border = BorderColor(act_color);
        }
    }

    // Confirm selection
    if is_confirm(&keyboard, &joystick) {
        let stage = (selection.index + 1) as u32;
        let locked = stage > highest + 1;

        if !locked {
            // Set campaign state to selected stage
            let act = if stage <= 4 {
                crate::core::Act::Act1
            } else if stage <= 9 {
                crate::core::Act::Act2
            } else {
                crate::core::Act::Act3
            };

            let mission_idx = match act {
                crate::core::Act::Act1 => (stage - 1) as usize,
                crate::core::Act::Act2 => (stage - 5) as usize,
                crate::core::Act::Act3 => (stage - 10) as usize,
            };

            campaign.act = act;
            campaign.mission_index = mission_idx;

            info!("Selected Stage {} (Act {:?}, Mission {})", stage, act, mission_idx + 1);
            next_state.set(GameState::ShipSelect);
        }
    }

    // Back
    if keyboard.just_pressed(KeyCode::Escape) || joystick.back() {
        next_state.set(GameState::FactionSelect);
    }
}

// ============================================================================
// Ship Select
// ============================================================================

/// Marker for the selected ship detail panel
#[derive(Component)]
struct ShipDetailPanel;

/// Marker for ship detail text elements
#[derive(Component)]
struct ShipDetailName;
#[derive(Component)]
struct ShipDetailClass;
#[derive(Component)]
struct ShipDetailRole;
#[derive(Component)]
struct ShipDetailSpecial;
#[derive(Component)]
struct ShipDetailWeapon;

/// Stat bar markers
#[derive(Component)]
struct StatBarFill(StatType);

#[derive(Clone, Copy)]
enum StatType {
    Speed,
    Damage,
    Health,
    FireRate,
}

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

    // Calculate stat ranges for normalization
    let max_speed = ships.iter().map(|s| s.speed).fold(0.0_f32, f32::max);
    let max_damage = ships.iter().map(|s| s.damage).fold(0.0_f32, f32::max);
    let max_health = ships.iter().map(|s| s.health).fold(0.0_f32, f32::max);
    let max_fire_rate = ships.iter().map(|s| s.fire_rate).fold(0.0_f32, f32::max);

    commands
        .spawn((
            ShipMenuRoot,
            Node {
                width: Val::Percent(100.0),
                height: Val::Percent(100.0),
                justify_content: JustifyContent::Center,
                align_items: AlignItems::Center,
                flex_direction: FlexDirection::Column,
                row_gap: Val::Px(20.0),
                padding: UiRect::all(Val::Px(40.0)),
                ..default()
            },
            BackgroundColor(Color::srgba(0.0, 0.0, 0.0, 0.5)),
        ))
        .with_children(|parent| {
            // Title with faction name
            parent.spawn((
                Text::new(format!("{} FLEET - SELECT SHIP", faction.short_name())),
                TextFont {
                    font_size: 36.0,
                    ..default()
                },
                TextColor(faction_color),
            ));

            // Subtitle with weapon doctrine and difficulty
            parent.spawn((
                Text::new(format!(
                    "{} Doctrine • {} Mode",
                    faction.weapon_type().name(),
                    difficulty.name()
                )),
                TextFont {
                    font_size: 14.0,
                    ..default()
                },
                TextColor(Color::srgb(0.5, 0.5, 0.5)),
            ));

            // Main content: Detail panel (left) + Ship list (right)
            parent
                .spawn(Node {
                    width: Val::Percent(100.0),
                    max_width: Val::Px(900.0),
                    flex_direction: FlexDirection::Row,
                    justify_content: JustifyContent::Center,
                    column_gap: Val::Px(30.0),
                    ..default()
                })
                .with_children(|content| {
                    // Left: Selected ship detail panel
                    spawn_ship_detail_panel(content, &ships[0], faction_color, max_speed, max_damage, max_health, max_fire_rate);

                    // Right: Ship list
                    content
                        .spawn(Node {
                            flex_direction: FlexDirection::Column,
                            row_gap: Val::Px(8.0),
                            ..default()
                        })
                        .with_children(|list| {
                            for (i, ship) in ships.iter().enumerate() {
                                let is_unlocked = save_data.is_ship_unlocked(
                                    ship.type_id,
                                    ship.unlock_stage,
                                    faction.short_name(),
                                    enemy.short_name(),
                                );
                                spawn_ship_list_item(list, ship, i, is_unlocked, faction_color);
                            }
                        });
                });

            // Navigation hint
            parent.spawn((
                Text::new("↑↓ Navigate • A/ENTER Select • B/ESC Back"),
                TextFont {
                    font_size: 12.0,
                    ..default()
                },
                TextColor(Color::srgb(0.4, 0.4, 0.4)),
            ));
        });
}

/// Spawn the detailed ship info panel (left side)
fn spawn_ship_detail_panel(
    parent: &mut ChildBuilder,
    ship: &ShipDef,
    faction_color: Color,
    max_speed: f32,
    max_damage: f32,
    max_health: f32,
    max_fire_rate: f32,
) {
    parent
        .spawn((
            ShipDetailPanel,
            Node {
                width: Val::Px(380.0),
                flex_direction: FlexDirection::Column,
                padding: UiRect::all(Val::Px(20.0)),
                row_gap: Val::Px(12.0),
                ..default()
            },
            BackgroundColor(Color::srgba(0.08, 0.08, 0.12, 0.95)),
            BorderRadius::all(Val::Px(8.0)),
        ))
        .with_children(|panel| {
            // Ship name (large)
            panel.spawn((
                ShipDetailName,
                Text::new(ship.name),
                TextFont {
                    font_size: 32.0,
                    ..default()
                },
                TextColor(faction_color),
            ));

            // Class and role
            panel
                .spawn(Node {
                    flex_direction: FlexDirection::Row,
                    column_gap: Val::Px(10.0),
                    ..default()
                })
                .with_children(|row| {
                    row.spawn((
                        ShipDetailClass,
                        Text::new(ship.class.name()),
                        TextFont {
                            font_size: 14.0,
                            ..default()
                        },
                        TextColor(Color::srgb(0.6, 0.6, 0.6)),
                    ));
                    row.spawn((
                        Text::new("•"),
                        TextFont {
                            font_size: 14.0,
                            ..default()
                        },
                        TextColor(Color::srgb(0.4, 0.4, 0.4)),
                    ));
                    row.spawn((
                        ShipDetailRole,
                        Text::new(ship.role),
                        TextFont {
                            font_size: 14.0,
                            ..default()
                        },
                        TextColor(Color::srgb(0.7, 0.7, 0.7)),
                    ));
                });

            // Divider
            panel.spawn((
                Node {
                    width: Val::Percent(100.0),
                    height: Val::Px(1.0),
                    margin: UiRect::vertical(Val::Px(5.0)),
                    ..default()
                },
                BackgroundColor(Color::srgb(0.25, 0.25, 0.3)),
            ));

            // Stat bars section
            panel
                .spawn(Node {
                    flex_direction: FlexDirection::Column,
                    row_gap: Val::Px(10.0),
                    ..default()
                })
                .with_children(|stats| {
                    spawn_stat_bar(stats, "SPEED", ship.speed, max_speed, Color::srgb(0.3, 0.8, 0.3), StatType::Speed);
                    spawn_stat_bar(stats, "DAMAGE", ship.damage, max_damage, Color::srgb(0.9, 0.3, 0.3), StatType::Damage);
                    spawn_stat_bar(stats, "HEALTH", ship.health, max_health, Color::srgb(0.3, 0.6, 0.9), StatType::Health);
                    spawn_stat_bar(stats, "FIRE RATE", ship.fire_rate, max_fire_rate, Color::srgb(0.9, 0.7, 0.3), StatType::FireRate);
                });

            // Divider
            panel.spawn((
                Node {
                    width: Val::Percent(100.0),
                    height: Val::Px(1.0),
                    margin: UiRect::vertical(Val::Px(5.0)),
                    ..default()
                },
                BackgroundColor(Color::srgb(0.25, 0.25, 0.3)),
            ));

            // Special ability
            panel
                .spawn(Node {
                    flex_direction: FlexDirection::Column,
                    row_gap: Val::Px(4.0),
                    ..default()
                })
                .with_children(|special| {
                    special.spawn((
                        Text::new("SPECIAL ABILITY"),
                        TextFont {
                            font_size: 10.0,
                            ..default()
                        },
                        TextColor(Color::srgb(0.5, 0.5, 0.5)),
                    ));
                    special.spawn((
                        ShipDetailSpecial,
                        Text::new(ship.special),
                        TextFont {
                            font_size: 13.0,
                            ..default()
                        },
                        TextColor(Color::srgb(0.4, 0.8, 1.0)),
                    ));
                });
        });
}

/// Spawn a stat bar with label and fill
fn spawn_stat_bar(
    parent: &mut ChildBuilder,
    label: &str,
    value: f32,
    max_value: f32,
    color: Color,
    stat_type: StatType,
) {
    let percent = (value / max_value * 100.0).clamp(0.0, 100.0);

    parent
        .spawn(Node {
            flex_direction: FlexDirection::Column,
            row_gap: Val::Px(3.0),
            ..default()
        })
        .with_children(|stat| {
            // Label row with value
            stat.spawn(Node {
                flex_direction: FlexDirection::Row,
                justify_content: JustifyContent::SpaceBetween,
                width: Val::Percent(100.0),
                ..default()
            })
            .with_children(|row| {
                row.spawn((
                    Text::new(label),
                    TextFont {
                        font_size: 11.0,
                        ..default()
                    },
                    TextColor(color),
                ));
                row.spawn((
                    Text::new(format!("{:.0}", value)),
                    TextFont {
                        font_size: 11.0,
                        ..default()
                    },
                    TextColor(Color::srgb(0.7, 0.7, 0.7)),
                ));
            });

            // Bar background
            stat.spawn((
                Node {
                    width: Val::Percent(100.0),
                    height: Val::Px(8.0),
                    ..default()
                },
                BackgroundColor(Color::srgba(0.15, 0.15, 0.2, 0.9)),
                BorderRadius::all(Val::Px(2.0)),
            ))
            .with_children(|bar| {
                // Bar fill
                bar.spawn((
                    StatBarFill(stat_type),
                    Node {
                        width: Val::Percent(percent),
                        height: Val::Percent(100.0),
                        ..default()
                    },
                    BackgroundColor(color),
                    BorderRadius::all(Val::Px(2.0)),
                ));
            });
        });
}

/// Spawn a compact ship list item (right side)
fn spawn_ship_list_item(
    parent: &mut ChildBuilder,
    ship: &ShipDef,
    index: usize,
    is_unlocked: bool,
    faction_color: Color,
) {
    let name_color = if is_unlocked {
        faction_color
    } else {
        Color::srgb(0.35, 0.35, 0.35)
    };
    let bg_color = if is_unlocked {
        Color::srgba(0.1, 0.1, 0.12, 0.9)
    } else {
        Color::srgba(0.06, 0.06, 0.08, 0.9)
    };
    let border_color = if is_unlocked {
        Color::srgb(0.25, 0.25, 0.3)
    } else {
        Color::srgb(0.15, 0.15, 0.18)
    };

    parent
        .spawn((
            ShipMenuRoot,
            MenuItem { index },
            Node {
                width: Val::Px(280.0),
                padding: UiRect::all(Val::Px(12.0)),
                justify_content: JustifyContent::SpaceBetween,
                align_items: AlignItems::Center,
                flex_direction: FlexDirection::Row,
                border: UiRect::all(Val::Px(2.0)),
                ..default()
            },
            BackgroundColor(bg_color),
            BorderColor(border_color),
            BorderRadius::all(Val::Px(4.0)),
        ))
        .with_children(|btn| {
            // Left: Name and class
            btn.spawn(Node {
                flex_direction: FlexDirection::Column,
                ..default()
            })
            .with_children(|left| {
                let name_text = if is_unlocked {
                    ship.name.to_string()
                } else {
                    format!("🔒 {}", ship.name)
                };
                left.spawn((
                    Text::new(name_text),
                    TextFont {
                        font_size: 16.0,
                        ..default()
                    },
                    TextColor(name_color),
                ));
                left.spawn((
                    Text::new(ship.class.name()),
                    TextFont {
                        font_size: 10.0,
                        ..default()
                    },
                    TextColor(Color::srgb(0.45, 0.45, 0.45)),
                ));
            });

            // Right: Quick stats
            if is_unlocked {
                btn.spawn(Node {
                    flex_direction: FlexDirection::Column,
                    align_items: AlignItems::FlexEnd,
                    ..default()
                })
                .with_children(|right| {
                    right.spawn((
                        Text::new(format!("DMG {:.0}", ship.damage)),
                        TextFont {
                            font_size: 10.0,
                            ..default()
                        },
                        TextColor(Color::srgb(0.7, 0.4, 0.4)),
                    ));
                    right.spawn((
                        Text::new(format!("SPD {:.0}", ship.speed)),
                        TextFont {
                            font_size: 10.0,
                            ..default()
                        },
                        TextColor(Color::srgb(0.4, 0.6, 0.4)),
                    ));
                });
            } else {
                btn.spawn((
                    Text::new(format!("Stage {}", ship.unlock_stage)),
                    TextFont {
                        font_size: 10.0,
                        ..default()
                    },
                    TextColor(Color::srgb(0.5, 0.3, 0.3)),
                ));
            }
        });
}

/// Update detail panel when selection changes
fn update_ship_detail_panel(
    selection: Res<MenuSelection>,
    session: Res<GameSession>,
    mut name_query: Query<&mut Text, (With<ShipDetailName>, Without<ShipDetailClass>, Without<ShipDetailRole>, Without<ShipDetailSpecial>)>,
    mut class_query: Query<&mut Text, (With<ShipDetailClass>, Without<ShipDetailName>, Without<ShipDetailRole>, Without<ShipDetailSpecial>)>,
    mut role_query: Query<&mut Text, (With<ShipDetailRole>, Without<ShipDetailName>, Without<ShipDetailClass>, Without<ShipDetailSpecial>)>,
    mut special_query: Query<&mut Text, (With<ShipDetailSpecial>, Without<ShipDetailName>, Without<ShipDetailClass>, Without<ShipDetailRole>)>,
    mut stat_bars: Query<(&StatBarFill, &mut Node)>,
) {
    if !selection.is_changed() {
        return;
    }

    let ships = session.player_ships();
    if selection.index >= ships.len() {
        return;
    }

    let ship = &ships[selection.index];

    // Calculate stat ranges for normalization
    let max_speed = ships.iter().map(|s| s.speed).fold(0.0_f32, f32::max);
    let max_damage = ships.iter().map(|s| s.damage).fold(0.0_f32, f32::max);
    let max_health = ships.iter().map(|s| s.health).fold(0.0_f32, f32::max);
    let max_fire_rate = ships.iter().map(|s| s.fire_rate).fold(0.0_f32, f32::max);

    // Update text fields
    for mut text in name_query.iter_mut() {
        **text = ship.name.to_string();
    }
    for mut text in class_query.iter_mut() {
        **text = ship.class.name().to_string();
    }
    for mut text in role_query.iter_mut() {
        **text = ship.role.to_string();
    }
    for mut text in special_query.iter_mut() {
        **text = ship.special.to_string();
    }

    // Update stat bars
    for (stat_fill, mut node) in stat_bars.iter_mut() {
        let (value, max) = match stat_fill.0 {
            StatType::Speed => (ship.speed, max_speed),
            StatType::Damage => (ship.damage, max_damage),
            StatType::Health => (ship.health, max_health),
            StatType::FireRate => (ship.fire_rate, max_fire_rate),
        };
        let percent = (value / max * 100.0).clamp(0.0, 100.0);
        node.width = Val::Percent(percent);
    }
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

    if is_confirm(&keyboard, &joystick) && selection.index < ships.len() {
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

/// Pause menu items
const PAUSE_ITEM_COUNT: usize = 8;
const PAUSE_IDX_RESUME: usize = 0;
const PAUSE_IDX_MASTER: usize = 1;
const PAUSE_IDX_MUSIC: usize = 2;
const PAUSE_IDX_SFX: usize = 3;
const PAUSE_IDX_SHAKE: usize = 4;
const PAUSE_IDX_RUMBLE: usize = 5;
const PAUSE_IDX_RESTART: usize = 6;
const PAUSE_IDX_QUIT: usize = 7;

/// Slider type for identifying which setting to adjust
#[derive(Clone, Copy, PartialEq)]
enum SliderType {
    MasterVolume,
    MusicVolume,
    SfxVolume,
    ScreenShake,
    Rumble,
}

/// Marker for slider bar fill
#[derive(Component)]
struct SliderFill {
    slider_type: SliderType,
}

/// Marker for slider value text
#[derive(Component)]
struct SliderValueText {
    slider_type: SliderType,
}

fn spawn_pause_menu(
    mut commands: Commands,
    campaign: Res<CampaignState>,
    score: Res<ScoreSystem>,
    session: Res<GameSession>,
    sound_settings: Res<crate::systems::SoundSettings>,
    screen_shake: Res<crate::systems::ScreenShake>,
    rumble_settings: Res<crate::systems::RumbleSettings>,
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
                row_gap: Val::Px(8.0),
                ..default()
            },
            BackgroundColor(Color::srgba(0.0, 0.0, 0.05, 0.85)),
        ))
        .with_children(|parent| {
            // Title
            parent.spawn((
                Text::new("PAUSED"),
                TextFont {
                    font_size: 48.0,
                    ..default()
                },
                TextColor(faction_color),
            ));

            // Mission info
            parent.spawn((
                Text::new(mission_name),
                TextFont {
                    font_size: 16.0,
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
                    font_size: 12.0,
                    ..default()
                },
                TextColor(Color::srgb(0.4, 0.6, 0.8)),
            ));

            // Spacer
            parent.spawn(Node {
                height: Val::Px(12.0),
                ..default()
            });

            // Resume button
            spawn_pause_menu_item(parent, PAUSE_IDX_RESUME, "RESUME");

            // Audio sliders section
            parent.spawn(Node {
                height: Val::Px(4.0),
                ..default()
            });

            // Master volume slider
            spawn_settings_slider(parent, PAUSE_IDX_MASTER, "MASTER", sound_settings.master_volume, SliderType::MasterVolume);

            // Music volume slider
            spawn_settings_slider(parent, PAUSE_IDX_MUSIC, "MUSIC", sound_settings.music_volume, SliderType::MusicVolume);

            // SFX volume slider
            spawn_settings_slider(parent, PAUSE_IDX_SFX, "SFX", sound_settings.sfx_volume, SliderType::SfxVolume);

            // Screen shake slider
            spawn_settings_slider(parent, PAUSE_IDX_SHAKE, "SHAKE", screen_shake.multiplier, SliderType::ScreenShake);

            // Rumble slider
            spawn_settings_slider(parent, PAUSE_IDX_RUMBLE, "RUMBLE", rumble_settings.intensity, SliderType::Rumble);

            parent.spawn(Node {
                height: Val::Px(4.0),
                ..default()
            });

            // Restart button
            spawn_pause_menu_item(parent, PAUSE_IDX_RESTART, "RESTART MISSION");

            // Quit button
            spawn_pause_menu_item(parent, PAUSE_IDX_QUIT, "QUIT TO MENU");

            // Spacer
            parent.spawn(Node {
                height: Val::Px(15.0),
                ..default()
            });

            // Controls hint
            parent.spawn((
                Text::new("↑↓ Navigate • ←→ Adjust • A/ENTER Select"),
                TextFont {
                    font_size: 11.0,
                    ..default()
                },
                TextColor(Color::srgb(0.35, 0.35, 0.35)),
            ));
        });
}

/// Spawn a simple pause menu button item
fn spawn_pause_menu_item(parent: &mut ChildBuilder, index: usize, label: &str) {
    parent
        .spawn((
            PauseMenuItem(index),
            Node {
                padding: UiRect::axes(Val::Px(25.0), Val::Px(8.0)),
                min_width: Val::Px(260.0),
                justify_content: JustifyContent::Center,
                ..default()
            },
            BackgroundColor(Color::srgba(0.1, 0.1, 0.1, 0.8)),
        ))
        .with_children(|btn| {
            btn.spawn((
                PauseMenuItemText(index),
                Text::new(label),
                TextFont {
                    font_size: 18.0,
                    ..default()
                },
                TextColor(Color::srgb(0.7, 0.7, 0.7)),
            ));
        });
}

/// Spawn a settings slider row
fn spawn_settings_slider(
    parent: &mut ChildBuilder,
    index: usize,
    label: &str,
    value: f32,
    slider_type: SliderType,
) {
    parent
        .spawn((
            PauseMenuItem(index),
            Node {
                padding: UiRect::axes(Val::Px(15.0), Val::Px(6.0)),
                min_width: Val::Px(260.0),
                flex_direction: FlexDirection::Row,
                justify_content: JustifyContent::SpaceBetween,
                align_items: AlignItems::Center,
                column_gap: Val::Px(12.0),
                ..default()
            },
            BackgroundColor(Color::srgba(0.1, 0.1, 0.1, 0.8)),
        ))
        .with_children(|row| {
            // Label
            row.spawn((
                PauseMenuItemText(index),
                Text::new(label),
                TextFont {
                    font_size: 16.0,
                    ..default()
                },
                TextColor(Color::srgb(0.6, 0.6, 0.6)),
            ));

            // Slider container
            row.spawn(Node {
                flex_direction: FlexDirection::Row,
                align_items: AlignItems::Center,
                column_gap: Val::Px(8.0),
                ..default()
            })
            .with_children(|slider_row| {
                // Slider background bar
                slider_row
                    .spawn(Node {
                        width: Val::Px(100.0),
                        height: Val::Px(10.0),
                        ..default()
                    })
                    .insert(BackgroundColor(Color::srgb(0.15, 0.15, 0.15)))
                    .with_children(|bar| {
                        // Slider fill
                        bar.spawn((
                            SliderFill { slider_type },
                            Node {
                                width: Val::Percent(value * 100.0),
                                height: Val::Percent(100.0),
                                ..default()
                            },
                            BackgroundColor(Color::srgb(0.4, 0.6, 0.8)),
                        ));
                    });

                // Value text
                slider_row.spawn((
                    SliderValueText { slider_type },
                    Text::new(format!("{}%", (value * 100.0) as i32)),
                    TextFont {
                        font_size: 12.0,
                        ..default()
                    },
                    TextColor(Color::srgb(0.5, 0.5, 0.5)),
                ));
            });
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
    mut sound_settings: ResMut<crate::systems::SoundSettings>,
    mut screen_shake: ResMut<crate::systems::ScreenShake>,
    mut rumble_settings: ResMut<crate::systems::RumbleSettings>,
    mut item_query: Query<(&PauseMenuItem, &mut BackgroundColor)>,
    mut text_query: Query<(&PauseMenuItemText, &mut TextColor)>,
    mut slider_fill_query: Query<(&SliderFill, &mut Node)>,
    mut slider_text_query: Query<(&SliderValueText, &mut Text)>,
    time: Res<Time>,
    mut cooldown: Local<f32>,
) {
    *cooldown -= time.delta_secs();

    // Navigation (up/down)
    let nav = get_nav_input(&keyboard, &joystick);
    if nav != 0 && *cooldown <= 0.0 {
        selection.index =
            (selection.index as i32 + nav).rem_euclid(PAUSE_ITEM_COUNT as i32) as usize;
        *cooldown = MENU_NAV_COOLDOWN;
    }

    // Horizontal input for sliders (left/right)
    let h_input = get_horizontal_input(&keyboard, &joystick);
    if h_input != 0 && *cooldown <= 0.0 {
        let delta = h_input as f32 * 0.05; // 5% per press

        match selection.index {
            PAUSE_IDX_MASTER => {
                sound_settings.master_volume = (sound_settings.master_volume + delta).clamp(0.0, 1.0);
                *cooldown = 0.08;
            }
            PAUSE_IDX_MUSIC => {
                sound_settings.music_volume = (sound_settings.music_volume + delta).clamp(0.0, 1.0);
                *cooldown = 0.08;
            }
            PAUSE_IDX_SFX => {
                sound_settings.sfx_volume = (sound_settings.sfx_volume + delta).clamp(0.0, 1.0);
                *cooldown = 0.08;
            }
            PAUSE_IDX_SHAKE => {
                screen_shake.multiplier = (screen_shake.multiplier + delta).clamp(0.0, 1.0);
                *cooldown = 0.08;
            }
            PAUSE_IDX_RUMBLE => {
                rumble_settings.intensity = (rumble_settings.intensity + delta).clamp(0.0, 1.0);
                *cooldown = 0.08;
            }
            _ => {}
        }
    }

    // Update slider visuals
    for (fill, mut node) in slider_fill_query.iter_mut() {
        let value = match fill.slider_type {
            SliderType::MasterVolume => sound_settings.master_volume,
            SliderType::MusicVolume => sound_settings.music_volume,
            SliderType::SfxVolume => sound_settings.sfx_volume,
            SliderType::ScreenShake => screen_shake.multiplier,
            SliderType::Rumble => rumble_settings.intensity,
        };
        node.width = Val::Percent(value * 100.0);
    }
    for (text_marker, mut text) in slider_text_query.iter_mut() {
        let value = match text_marker.slider_type {
            SliderType::MasterVolume => sound_settings.master_volume,
            SliderType::MusicVolume => sound_settings.music_volume,
            SliderType::SfxVolume => sound_settings.sfx_volume,
            SliderType::ScreenShake => screen_shake.multiplier,
            SliderType::Rumble => rumble_settings.intensity,
        };
        **text = format!("{}%", (value * 100.0) as i32);
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

    // Selection (confirm button)
    if is_confirm(&keyboard, &joystick) {
        match selection.index {
            PAUSE_IDX_RESUME => {
                next_state.set(GameState::Playing);
            }
            PAUSE_IDX_RESTART => {
                transitions.send(TransitionEvent::quick(GameState::Playing));
            }
            PAUSE_IDX_QUIT => {
                transitions.send(TransitionEvent::to(GameState::MainMenu));
            }
            PAUSE_IDX_MASTER | PAUSE_IDX_MUSIC | PAUSE_IDX_SFX | PAUSE_IDX_SHAKE | PAUSE_IDX_RUMBLE => {
                // Pressing confirm on sliders does nothing (use left/right)
            }
            _ => {}
        }
    }

    // Quick resume with ESC or Start
    if keyboard.just_pressed(KeyCode::Escape) || joystick.start() {
        next_state.set(GameState::Playing);
    }
}

/// Get horizontal input (-1 left, 0 none, 1 right)
fn get_horizontal_input(keyboard: &ButtonInput<KeyCode>, joystick: &JoystickState) -> i32 {
    let mut h = 0;

    // Keyboard
    if keyboard.pressed(KeyCode::ArrowLeft) || keyboard.pressed(KeyCode::KeyA) {
        h -= 1;
    }
    if keyboard.pressed(KeyCode::ArrowRight) || keyboard.pressed(KeyCode::KeyD) {
        h += 1;
    }

    // Joystick d-pad
    if joystick.dpad_x < 0 {
        h -= 1;
    } else if joystick.dpad_x > 0 {
        h += 1;
    }

    // Joystick left stick
    if joystick.left_x < -0.5 {
        h -= 1;
    } else if joystick.left_x > 0.5 {
        h += 1;
    }

    h.clamp(-1, 1)
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
    mut endless: ResMut<crate::core::EndlessMode>,
    mut nightmare: ResMut<crate::games::caldari_gallente::ShiigeruNightmare>,
    session: Res<GameSession>,
    save_data: Res<SaveData>,
) {
    // Initialize selection resource
    commands.insert_resource(DeathSelection::default());

    // End endless run if active
    let was_endless = endless.active;
    if was_endless {
        endless.end_run();
    }

    // End nightmare run if active
    let was_nightmare = nightmare.active;
    let nightmare_stats = if was_nightmare {
        Some((
            nightmare.wave,
            nightmare.time_survived,
            nightmare.kills,
            nightmare.mini_bosses_defeated,
            nightmare.wave > nightmare.best_wave,
            nightmare.time_survived > nightmare.best_time,
        ))
    } else {
        None
    };
    if was_nightmare {
        nightmare.end();
    }

    // Get high score for comparison
    let high_score =
        save_data.get_high_score(session.player_faction.name(), session.enemy_faction.name());
    let is_new_high = score.score > high_score && score.score > 0;

    // Get mission info - different for endless/nightmare mode
    let mission_name = if was_nightmare {
        "SHIIGERU NIGHTMARE".to_string()
    } else if was_endless {
        format!("Endless Wave {}", endless.wave)
    } else {
        campaign.current_mission_name().to_string()
    };

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

            // Stats row - different for nightmare mode
            if let Some((wave, time, kills, bosses, new_wave_record, new_time_record)) =
                nightmare_stats
            {
                // Nightmare-specific stats
                parent
                    .spawn(Node {
                        flex_direction: FlexDirection::Column,
                        align_items: AlignItems::Center,
                        row_gap: Val::Px(8.0),
                        ..default()
                    })
                    .with_children(|col| {
                        // Wave reached
                        let wave_text = if new_wave_record {
                            format!("★ WAVE {} (NEW RECORD!) ★", wave)
                        } else {
                            format!("Wave Reached: {}", wave)
                        };
                        col.spawn((
                            Text::new(wave_text),
                            TextFont {
                                font_size: 24.0,
                                ..default()
                            },
                            TextColor(if new_wave_record {
                                Color::srgb(1.0, 0.9, 0.0)
                            } else {
                                Color::srgb(0.9, 0.5, 0.5)
                            }),
                        ));

                        // Time survived
                        let mins = (time / 60.0) as u32;
                        let secs = (time % 60.0) as u32;
                        let time_text = if new_time_record {
                            format!("★ {:02}:{:02} (NEW RECORD!) ★", mins, secs)
                        } else {
                            format!("Time Survived: {:02}:{:02}", mins, secs)
                        };
                        col.spawn((
                            Text::new(time_text),
                            TextFont {
                                font_size: 20.0,
                                ..default()
                            },
                            TextColor(if new_time_record {
                                Color::srgb(1.0, 0.9, 0.0)
                            } else {
                                Color::srgb(0.7, 0.7, 0.7)
                            }),
                        ));

                        // Kills and bosses row
                        col.spawn(Node {
                            flex_direction: FlexDirection::Row,
                            column_gap: Val::Px(30.0),
                            ..default()
                        })
                        .with_children(|row| {
                            row.spawn((
                                Text::new(format!("Kills: {}", kills)),
                                TextFont {
                                    font_size: 18.0,
                                    ..default()
                                },
                                TextColor(Color::srgb(0.59, 0.51, 0.35)),
                            ));

                            if bosses > 0 {
                                row.spawn((
                                    Text::new(format!("Mini-Bosses: {}", bosses)),
                                    TextFont {
                                        font_size: 18.0,
                                        ..default()
                                    },
                                    TextColor(Color::srgb(0.59, 0.51, 0.35)),
                                ));
                            }
                        });
                    });
            } else {
                // Regular campaign stats
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
                            Text::new(format!(
                                "Stage {}-{}",
                                campaign.stage_number(),
                                campaign.mission_in_stage()
                            )),
                            TextFont {
                                font_size: 20.0,
                                ..default()
                            },
                            TextColor(Color::srgb(0.59, 0.51, 0.35)),
                        ));
                    });
            }

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

            // Controller hint
            parent.spawn((
                Text::new("← → Navigate • A/ENTER Select • B/ESC Quit"),
                TextFont {
                    font_size: 12.0,
                    ..default()
                },
                TextColor(Color::srgb(0.3, 0.3, 0.3)),
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
    // Get boss data for dialogue and phase info
    let stage = (campaign.mission_index + 1) as u32;
    let boss_data = get_boss_for_stage(stage);

    let (boss_name, boss_title, dialogue, phases) = if let Some(data) = &boss_data {
        (
            data.name.as_str(),
            data.title.as_str(),
            data.dialogue_intro.clone(),
            data.total_phases,
        )
    } else if let Some(mission) = campaign.current_mission() {
        (
            mission.boss.name(),
            mission.name,
            "Prepare for battle...".to_string(),
            2,
        )
    } else {
        ("UNKNOWN", "???", "Prepare for battle...".to_string(), 1)
    };

    // Phase difficulty indicator
    let phase_text = match phases {
        1 => "Single Phase",
        2 => "Two Phases",
        3 => "Three Phases • Challenging",
        4 => "Four Phases • Dangerous",
        5 => "Five Phases • EXTREME",
        _ => "Multi-Phase",
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
                BossIntroWarning { timer: 0.0 },
            ));

            parent.spawn(Node {
                height: Val::Px(15.0),
                ..default()
            });

            // Boss name (fades in)
            parent.spawn((
                Text::new(boss_name),
                TextFont {
                    font_size: 72.0,
                    ..default()
                },
                TextColor(Color::srgba(1.0, 0.85, 0.2, 0.0)), // Start transparent
                BossIntroName { timer: 0.0 },
            ));

            // Boss title
            parent.spawn((
                Text::new(boss_title),
                TextFont {
                    font_size: 22.0,
                    ..default()
                },
                TextColor(Color::srgb(0.9, 0.7, 0.3)),
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
                BossIntroDialogue {
                    full_text: format!("\"{}\"", dialogue),
                    timer: 0.0,
                },
            ));
        });
}

fn boss_intro_update(
    time: Res<Time>,
    mut warning_query: Query<(&mut TextColor, &mut BossIntroWarning)>,
    mut name_query: Query<(&mut TextColor, &mut BossIntroName), Without<BossIntroWarning>>,
    mut dialogue_query: Query<(&mut Text, &mut BossIntroDialogue)>,
) {
    let dt = time.delta_secs();

    // Pulse warning text
    for (mut color, mut warning) in warning_query.iter_mut() {
        warning.timer += dt * 4.0;
        let pulse = (warning.timer.sin() * 0.3 + 0.7).clamp(0.4, 1.0);
        *color = TextColor(Color::srgb(1.0, 0.2 * pulse, 0.2 * pulse));
    }

    // Fade in boss name
    for (mut color, mut name) in name_query.iter_mut() {
        name.timer += dt * 2.0;
        let alpha = (name.timer - 0.3).clamp(0.0, 1.0); // Delay 0.3s then fade in
        *color = TextColor(Color::srgba(1.0, 0.85, 0.2, alpha));
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

// ============================================================================
// Stage Complete Screen
// ============================================================================

fn spawn_stage_complete(
    mut commands: Commands,
    campaign: Res<CampaignState>,
    score: Res<ScoreSystem>,
    session: Res<GameSession>,
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

    // Check if any ships were unlocked by completing this stage
    let completed_stage = campaign.stage_number();
    let ships = session.player_ships();
    let unlocked_ships: Vec<&str> = ships
        .iter()
        .filter(|s| s.unlock_stage == completed_stage)
        .map(|s| s.name)
        .collect();

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

            // Ship unlock notification
            if !unlocked_ships.is_empty() {
                parent.spawn(Node {
                    height: Val::Px(15.0),
                    ..default()
                });

                parent
                    .spawn((
                        Node {
                            padding: UiRect::all(Val::Px(12.0)),
                            border: UiRect::all(Val::Px(2.0)),
                            flex_direction: FlexDirection::Column,
                            align_items: AlignItems::Center,
                            row_gap: Val::Px(5.0),
                            ..default()
                        },
                        BorderColor(Color::srgb(0.2, 0.8, 0.4)),
                        BackgroundColor(Color::srgba(0.1, 0.3, 0.15, 0.9)),
                    ))
                    .with_children(|unlock_box| {
                        unlock_box.spawn((
                            Text::new("NEW SHIP UNLOCKED!"),
                            TextFont {
                                font_size: 22.0,
                                ..default()
                            },
                            TextColor(Color::srgb(0.3, 1.0, 0.4)),
                        ));

                        for ship_name in &unlocked_ships {
                            unlock_box.spawn((
                                Text::new(*ship_name),
                                TextFont {
                                    font_size: 18.0,
                                    ..default()
                                },
                                TextColor(Color::srgb(0.9, 0.9, 0.3)),
                            ));
                        }
                    });
            }

            parent.spawn(Node {
                height: Val::Px(20.0),
                ..default()
            });

            // Continue prompt
            parent.spawn((
                Text::new("A/ENTER Continue • B/ESC Quit"),
                TextFont {
                    font_size: 14.0,
                    ..default()
                },
                TextColor(Color::srgb(0.5, 0.5, 0.5)),
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
    let previous_high =
        save_data.get_high_score(session.player_faction.name(), session.enemy_faction.name());
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

            // Controller hint
            parent.spawn((
                Text::new("← → Navigate • A/ENTER Select"),
                TextFont {
                    font_size: 12.0,
                    ..default()
                },
                TextColor(Color::srgb(0.3, 0.3, 0.3)),
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
// Endless Mode Wave Announcements
// ============================================================================

/// Marker for endless wave announcement
#[derive(Component)]
struct EndlessWaveAnnouncement {
    timer: f32,
    max_time: f32,
}

/// Marker for endless mini-boss intro
#[derive(Component)]
struct EndlessMiniBossIntro {
    timer: f32,
    max_time: f32,
}

/// Pulse text for announcements
#[derive(Component)]
struct EndlessAnnouncementPulse {
    timer: f32,
}

/// Listen for wave events and spawn announcements (endless mode only)
fn spawn_endless_wave_announcements(
    mut commands: Commands,
    endless: Res<crate::core::EndlessMode>,
    mut wave_events: EventReader<crate::core::events::SpawnWaveEvent>,
) {
    // Only in endless mode
    if !endless.active {
        return;
    }

    for event in wave_events.read() {
        // Show announcement on wave 1, every 5th wave, and every 10th wave
        let wave = event.wave_number;
        if wave == 1 || wave % 5 == 0 {
            commands
                .spawn((
                    EndlessWaveAnnouncement {
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
                        TextColor(Color::srgb(1.0, 0.85, 0.2)), // Gold for Elder Fleet
                        EndlessAnnouncementPulse { timer: 0.0 },
                    ));

                    // Escalation warning
                    if wave >= 30 {
                        parent.spawn((
                            Text::new("EXTREME ESCALATION"),
                            TextFont {
                                font_size: 24.0,
                                ..default()
                            },
                            TextColor(Color::srgb(1.0, 0.3, 0.3)),
                        ));
                    } else if wave >= 20 {
                        parent.spawn((
                            Text::new("HIGH ESCALATION"),
                            TextFont {
                                font_size: 24.0,
                                ..default()
                            },
                            TextColor(Color::srgb(1.0, 0.5, 0.2)),
                        ));
                    } else if wave >= 10 {
                        parent.spawn((
                            Text::new("ESCALATING"),
                            TextFont {
                                font_size: 24.0,
                                ..default()
                            },
                            TextColor(Color::srgb(0.9, 0.7, 0.3)),
                        ));
                    }
                });
        }
    }
}

/// Listen for boss spawn events in endless mode
fn spawn_endless_miniboss_announcement(
    mut commands: Commands,
    endless: Res<crate::core::EndlessMode>,
    mut boss_events: EventReader<crate::core::BossSpawnEvent>,
) {
    // Only in endless mode for mini-bosses
    if !endless.active {
        return;
    }

    for _event in boss_events.read() {
        commands
            .spawn((
                EndlessMiniBossIntro {
                    timer: 0.0,
                    max_time: 2.0,
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
                BackgroundColor(Color::srgba(0.0, 0.0, 0.0, 0.6)),
            ))
            .with_children(|parent| {
                parent.spawn((
                    Text::new("⚠ MINI-BOSS ⚠"),
                    TextFont {
                        font_size: 32.0,
                        ..default()
                    },
                    TextColor(Color::srgb(1.0, 0.3, 0.3)),
                    EndlessAnnouncementPulse { timer: 0.0 },
                ));

                parent.spawn(Node {
                    height: Val::Px(10.0),
                    ..default()
                });

                parent.spawn((
                    Text::new(format!("Wave {} Champion", endless.wave)),
                    TextFont {
                        font_size: 48.0,
                        ..default()
                    },
                    TextColor(Color::srgb(1.0, 0.85, 0.2)),
                ));

                parent.spawn(Node {
                    height: Val::Px(10.0),
                    ..default()
                });

                parent.spawn((
                    Text::new(format!("Escalation: {:.1}x", endless.escalation)),
                    TextFont {
                        font_size: 20.0,
                        ..default()
                    },
                    TextColor(Color::srgb(0.8, 0.6, 0.2)),
                ));
            });
    }
}

/// Update endless wave announcements
fn update_endless_announcements(
    time: Res<Time>,
    mut commands: Commands,
    mut wave_query: Query<(Entity, &mut EndlessWaveAnnouncement, &mut BackgroundColor)>,
    mut boss_query: Query<
        (Entity, &mut EndlessMiniBossIntro, &mut BackgroundColor),
        Without<EndlessWaveAnnouncement>,
    >,
    mut pulse_query: Query<(&mut TextColor, &mut EndlessAnnouncementPulse)>,
) {
    let dt = time.delta_secs();

    // Update wave announcements
    for (entity, mut announcement, mut bg) in wave_query.iter_mut() {
        announcement.timer += dt;

        // Fade in/out
        let progress = announcement.timer / announcement.max_time;
        let alpha = if progress < 0.15 {
            progress / 0.15 * 0.25
        } else if progress > 0.7 {
            (1.0 - progress) / 0.3 * 0.25
        } else {
            0.25
        };
        *bg = BackgroundColor(Color::srgba(0.0, 0.0, 0.0, alpha));

        if announcement.timer >= announcement.max_time {
            commands.entity(entity).despawn_recursive();
        }
    }

    // Update mini-boss intros
    for (entity, mut intro, mut bg) in boss_query.iter_mut() {
        intro.timer += dt;

        // Fade out near end
        let progress = intro.timer / intro.max_time;
        let alpha = if progress > 0.7 {
            (1.0 - progress) / 0.3 * 0.6
        } else {
            0.6
        };
        *bg = BackgroundColor(Color::srgba(0.0, 0.0, 0.0, alpha));

        if intro.timer >= intro.max_time {
            commands.entity(entity).despawn_recursive();
        }
    }

    // Pulse text
    for (mut color, mut pulse) in pulse_query.iter_mut() {
        pulse.timer += dt * 5.0;
        let intensity = (pulse.timer.sin() * 0.2 + 0.8).clamp(0.6, 1.0);
        // Preserve the base color but vary intensity
        let base = color.0;
        *color = TextColor(Color::srgb(
            base.to_srgba().red * intensity,
            base.to_srgba().green * intensity,
            base.to_srgba().blue * intensity,
        ));
    }
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
