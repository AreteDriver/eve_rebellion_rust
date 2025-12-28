//! EVE Rebellion - Arcade Space Shooter
//!
//! A Rust/Bevy rewrite of the Python arcade game inspired by EVE Online.
//! Features 5 campaigns, EVE-style mechanics, and ship sprites from CCP's Image Server.

use bevy::prelude::*;
use bevy_egui::EguiPlugin;

mod core;
mod entities;
mod systems;
mod ui;
mod assets;
mod games;

use core::{
    GameState, GameEventsPlugin, ScoreSystem, BerserkSystem, GameProgress,
    InputConfig, AudioSettings, Difficulty, SelectedShip, CurrentStage, ShipUnlocks,
    CampaignState, MissionStartEvent, MissionCompleteEvent, WaveCompleteEvent,
    BossSpawnEvent, ActCompleteEvent,
};
use entities::EntitiesPlugin;
use systems::SystemsPlugin;
use ui::UiPlugin;
use assets::AssetsPlugin;
use games::GameModulesPlugin;

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

        // Campaign events
        .add_event::<MissionStartEvent>()
        .add_event::<MissionCompleteEvent>()
        .add_event::<WaveCompleteEvent>()
        .add_event::<BossSpawnEvent>()
        .add_event::<ActCompleteEvent>()

        // Game plugins
        .add_plugins((
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
    // Spawn 3D camera with orthographic projection for top-down view
    // Camera looks down at XY plane from above (positive Z)
    commands.spawn((
        Camera3d::default(),
        Projection::Orthographic(OrthographicProjection {
            // Match the 2D game's coordinate scale
            scaling_mode: bevy::render::camera::ScalingMode::FixedVertical {
                viewport_height: core::SCREEN_HEIGHT,
            },
            near: 0.1,
            far: 1000.0,
            ..OrthographicProjection::default_3d()
        }),
        Transform::from_xyz(0.0, 0.0, 500.0)
            .looking_at(Vec3::ZERO, Vec3::Y),
    ));

    // Directional light (main illumination from above-right)
    commands.spawn((
        DirectionalLight {
            illuminance: 15000.0,
            shadows_enabled: false,
            ..default()
        },
        Transform::from_xyz(200.0, 400.0, 500.0)
            .looking_at(Vec3::ZERO, Vec3::Y),
    ));

    // Ambient light for base visibility (prevents pure black shadows)
    commands.insert_resource(AmbientLight {
        color: Color::srgb(0.3, 0.35, 0.4),
        brightness: 300.0,
    });

    info!("EVE Rebellion initialized with 3D rendering!");
}
