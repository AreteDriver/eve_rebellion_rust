//! Entity Components and Bundles
//!
//! All game entities: player, enemies, projectiles, collectibles, etc.

pub mod player;
pub mod enemy;
pub mod projectile;
pub mod collectible;
pub mod boss;
pub mod wingman;

pub use player::*;
pub use enemy::*;
pub use projectile::*;
pub use collectible::*;
pub use boss::*;
pub use wingman::*;

use bevy::prelude::*;

/// Plugin that registers all entity-related systems
pub struct EntitiesPlugin;

impl Plugin for EntitiesPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins((
            PlayerPlugin,
            EnemyPlugin,
            ProjectilePlugin,
            CollectiblePlugin,
            WingmanPlugin,
        ));
    }
}
