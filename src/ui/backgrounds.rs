//! Background System
//!
//! Loads and displays background images for menus and gameplay.

use bevy::prelude::*;
use crate::core::*;

/// Background plugin
pub struct BackgroundPlugin;

impl Plugin for BackgroundPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<BackgroundAssets>()
            .add_systems(Startup, load_backgrounds)
            .add_systems(OnEnter(GameState::Loading), spawn_title_background)
            .add_systems(OnEnter(GameState::MainMenu), spawn_title_background)
            .add_systems(OnEnter(GameState::DifficultySelect), spawn_title_background)
            .add_systems(OnEnter(GameState::ShipSelect), spawn_title_background)
            .add_systems(OnExit(GameState::MainMenu), despawn_menu_background)
            .add_systems(OnExit(GameState::DifficultySelect), despawn_menu_background)
            .add_systems(OnExit(GameState::ShipSelect), despawn_menu_background)
            .add_systems(OnExit(GameState::Loading), despawn_menu_background);
    }
}

/// Background image assets
#[derive(Resource, Default)]
pub struct BackgroundAssets {
    pub title: Option<Handle<Image>>,
}

/// Marker for menu background sprite
#[derive(Component)]
pub struct MenuBackground;

/// Load background images
fn load_backgrounds(
    mut backgrounds: ResMut<BackgroundAssets>,
    asset_server: Res<AssetServer>,
) {
    backgrounds.title = Some(asset_server.load("backgrounds/title_background.png"));
    info!("Loading background images...");
}

/// Spawn title background for menus
fn spawn_title_background(
    mut commands: Commands,
    backgrounds: Res<BackgroundAssets>,
    existing: Query<Entity, With<MenuBackground>>,
    windows: Query<&Window>,
) {
    // Don't spawn if already exists
    if !existing.is_empty() {
        return;
    }

    let Some(handle) = backgrounds.title.clone() else {
        return;
    };

    let Ok(window) = windows.get_single() else {
        return;
    };

    // Spawn background sprite that covers the screen
    commands.spawn((
        MenuBackground,
        Sprite {
            image: handle,
            custom_size: Some(Vec2::new(window.width(), window.height())),
            ..default()
        },
        Transform::from_xyz(0.0, 0.0, -100.0), // Behind everything
    ));
}

/// Despawn menu background when leaving menu states
fn despawn_menu_background(
    mut commands: Commands,
    query: Query<Entity, With<MenuBackground>>,
) {
    for entity in query.iter() {
        commands.entity(entity).despawn();
    }
}
