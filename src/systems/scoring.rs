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

/// Update berserk meter and handle activation input
fn update_berserk_system(
    time: Res<Time>,
    keyboard: Res<ButtonInput<KeyCode>>,
    joystick: Res<crate::systems::JoystickState>,
    mut berserk: ResMut<BerserkSystem>,
    mut end_events: EventWriter<BerserkEndedEvent>,
    mut screen_flash: ResMut<crate::systems::ScreenFlash>,
    mut dialogue_events: EventWriter<super::DialogueEvent>,
) {
    let was_active = berserk.is_active;
    berserk.update(time.delta_secs());

    // Check if berserk just ended
    if was_active && !berserk.is_active {
        end_events.send(BerserkEndedEvent);
        info!("Berserk mode ended!");
    }

    // B key or gamepad Y button to activate berserk when meter is full
    let activate_pressed = keyboard.just_pressed(KeyCode::KeyB) || joystick.berserk();

    if activate_pressed && berserk.can_activate() {
        if berserk.try_activate() {
            info!("BERSERK MODE ACTIVATED! 5x score for 8 seconds!");
            screen_flash.berserk(); // Red flash on activation
            dialogue_events.send(super::DialogueEvent::combat_callout(
                super::CombatCalloutType::BerserkActive,
            ));
        }
    }
}

// Berserk meter fills from proximity kills
// See collision.rs: player_projectile_enemy_collision
