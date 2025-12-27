//! Game Systems
//!
//! Core gameplay systems: collision, spawning, scoring, effects, input, dialogue.

pub mod collision;
pub mod spawning;
pub mod scoring;
pub mod scoring_v2;
pub mod effects;
pub mod joystick;
pub mod boss;
pub mod dialogue;

pub use collision::*;
pub use spawning::*;
pub use scoring::*;
pub use scoring_v2::*;
pub use effects::*;
pub use joystick::*;
pub use boss::*;
pub use dialogue::*;

use bevy::prelude::*;

/// Plugin that registers all gameplay systems
pub struct SystemsPlugin;

impl Plugin for SystemsPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins((
            CollisionPlugin,
            SpawningPlugin,
            ScoringPlugin,
            ScoringSystemPlugin,
            EffectsPlugin,
            JoystickPlugin,
            BossPlugin,
            DialoguePlugin,
        ));
    }
}
