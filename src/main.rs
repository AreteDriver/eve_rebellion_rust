//! EVE Rebellion - Arcade Space Shooter
//!
//! A Rust/Bevy rewrite of the Python arcade game inspired by EVE Online.
//! Features 5 campaigns, EVE-style mechanics, and ship sprites from CCP's Image Server.

// Bevy systems naturally have complex query types and many parameters
#![allow(clippy::type_complexity)]
#![allow(clippy::too_many_arguments)]

use bevy::prelude::*;
use bevy_egui::EguiPlugin;

mod assets;
mod core;
mod entities;
mod games;
mod systems;
mod ui;

use assets::AssetsPlugin;
use core::{
    ActCompleteEvent, AudioSettings, BerserkSystem, BossSpawnEvent, CampaignState, CurrentStage,
    Difficulty, EndlessMode, GameEventsPlugin, GameProgress, GameSession, GameState, InputConfig,
    MissionCompleteEvent, MissionStartEvent, SavePlugin, ScoreSystem, SelectedShip, ShipUnlocks,
    WaveCompleteEvent,
};
use entities::EntitiesPlugin;
use games::GameModulesPlugin;
use systems::SystemsPlugin;
use ui::UiPlugin;

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
        .add_plugins(EguiPlugin)
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
        .init_resource::<ShipUnlocks>()
        .init_resource::<CampaignState>()
        .init_resource::<GameSession>()
        .init_resource::<EndlessMode>()
        // Campaign events
        .add_event::<MissionStartEvent>()
        .add_event::<MissionCompleteEvent>()
        .add_event::<WaveCompleteEvent>()
        .add_event::<BossSpawnEvent>()
        .add_event::<ActCompleteEvent>()
        // Game plugins
        .add_plugins((
            SavePlugin,
            AssetsPlugin,
            GameEventsPlugin,
            EntitiesPlugin,
            SystemsPlugin,
            UiPlugin,
            GameModulesPlugin,
        ))
        // Setup
        .add_systems(Startup, setup)
        .run();
}

/// Initial game setup
fn setup(mut commands: Commands) {
    // Use 2D camera - sprites work reliably with this
    commands.spawn(Camera2d);

    info!("EVE Rebellion initialized!");
}
