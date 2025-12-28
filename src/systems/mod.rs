//! Game Systems
//!
//! Core gameplay systems: collision, spawning, scoring, effects, input, dialogue, audio.

pub mod audio;
pub mod boss;
pub mod campaign;
pub mod collision;
pub mod dialogue;
pub mod effects;
pub mod joystick;
pub mod maneuvers;
pub mod scoring;
pub mod scoring_v2;
pub mod spawning;

pub use audio::*;
pub use boss::*;
pub use campaign::CampaignPlugin;
pub use collision::*;
pub use dialogue::*;
pub use effects::*;
pub use joystick::*;
pub use maneuvers::*;
pub use scoring::*;
pub use scoring_v2::*;
pub use spawning::*;

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
            AudioPlugin,
            ManeuverPlugin,
            CampaignPlugin,
        ));
    }
}
