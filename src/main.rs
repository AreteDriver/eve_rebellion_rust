//! EVE Rebellion - Arcade Space Shooter
//!
//! A Rust/Bevy rewrite of the Python arcade game inspired by EVE Online.
//! Features 5 campaigns, EVE-style mechanics, and ship sprites from CCP's Image Server.

use bevy::prelude::*;

mod core;
mod entities;
mod systems;
mod ui;
mod assets;

use core::{
    GameState, GameEventsPlugin, ScoreSystem, BerserkSystem, GameProgress,
    InputConfig, AudioSettings, Difficulty, SelectedShip, CurrentStage,
};
use entities::EntitiesPlugin;
use systems::SystemsPlugin;
use ui::UiPlugin;
use assets::AssetsPlugin;

fn main() {
    App::new()
        // Bevy plugins
        .add_plugins(DefaultPlugins.set(WindowPlugin {
            primary_window: Some(Window {
                title: core::WINDOW_TITLE.into(),
                resolution: (core::SCREEN_WIDTH, core::SCREEN_HEIGHT).into(),
                resizable: true,
                ..default()
            }),
            ..default()
        }))

        // Game state
        .init_state::<GameState>()

        // Resources
        .init_resource::<ScoreSystem>()
        .init_resource::<BerserkSystem>()
        .init_resource::<GameProgress>()
        .init_resource::<InputConfig>()
        .init_resource::<AudioSettings>()
        .init_resource::<Difficulty>()
        .init_resource::<SelectedShip>()
        .init_resource::<CurrentStage>()

        // Game plugins
        .add_plugins((
            AssetsPlugin,
            GameEventsPlugin,
            EntitiesPlugin,
            SystemsPlugin,
            UiPlugin,
        ))

        // Setup
        .add_systems(Startup, setup)

        .run();
}

/// Initial game setup
fn setup(mut commands: Commands) {
    // Spawn 2D camera
    commands.spawn((
        Camera2d,
        Transform::default(),
    ));

    info!("EVE Rebellion initialized!");
}
