//! Scoring System
//!
//! Handles score, multipliers, chain combos, and berserk meter.

use crate::core::*;
use bevy::prelude::*;

/// Scoring plugin
pub struct ScoringPlugin;

impl Plugin for ScoringPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(
            Update,
            (update_score_system, update_berserk_system).run_if(in_state(GameState::Playing)),
        );
    }
}

/// Update score chain timer
fn update_score_system(time: Res<Time>, mut score: ResMut<ScoreSystem>) {
    score.update(time.delta_secs());
}

/// Update berserk meter
fn update_berserk_system(
    time: Res<Time>,
    mut berserk: ResMut<BerserkSystem>,
    mut end_events: EventWriter<BerserkEndedEvent>,
) {
    let was_active = berserk.is_active;
    berserk.update(time.delta_secs());

    // Check if berserk just ended
    if was_active && !berserk.is_active {
        end_events.send(BerserkEndedEvent);
        info!("Berserk mode ended!");
    }
}

// Berserk now auto-activates on 5 proximity kills (within 80 units)
// See collision.rs: player_projectile_enemy_collision
