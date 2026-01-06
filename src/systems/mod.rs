//! Game Systems
//!
//! Core gameplay systems: collision, spawning, scoring, effects, input, dialogue, audio.

pub mod ability;
pub mod audio;
pub mod boss;
pub mod campaign;
pub mod collision;
pub mod dialogue;
pub mod effects;
pub mod joystick;
pub mod maneuvers;
pub mod music;
pub mod scoring;
pub mod scoring_v2;
pub mod spawning;

pub use ability::*;
pub use audio::*;
pub use boss::*;
pub use campaign::CampaignPlugin;
pub use collision::*;
pub use dialogue::*;
pub use effects::*;
pub use joystick::*;
pub use maneuvers::*;
pub use music::*;
pub use scoring::*;
pub use scoring_v2::*;
pub use spawning::*;

use bevy::prelude::*;

use crate::core::GameState;

/// Plugin that registers all gameplay systems
pub struct SystemsPlugin;

impl Plugin for SystemsPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins((
            AbilityPlugin,
            CollisionPlugin,
            SpawningPlugin,
            ScoringPlugin,
            ScoringSystemPlugin,
            EffectsPlugin,
            JoystickPlugin,
            BossPlugin,
            DialoguePlugin,
            AudioPlugin,
            MusicPlugin,
            ManeuverPlugin,
            CampaignPlugin,
        ))
        // Pause system - ESC during gameplay triggers pause
        .add_systems(
            Update,
            pause_trigger_system
                .run_if(in_state(GameState::Playing).or(in_state(GameState::BossFight))),
        );
    }
}

/// System that triggers pause when ESC or Start button is pressed during gameplay
fn pause_trigger_system(
    keyboard: Res<ButtonInput<KeyCode>>,
    joystick: Res<JoystickState>,
    mut next_state: ResMut<NextState<GameState>>,
) {
    if keyboard.just_pressed(KeyCode::Escape) || joystick.start() {
        next_state.set(GameState::Paused);
    }
}
