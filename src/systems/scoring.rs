//! Scoring System
//!
//! Handles score, multipliers, chain combos, and berserk meter.

use bevy::prelude::*;
use crate::core::*;
use crate::systems::JoystickState;

/// Scoring plugin
pub struct ScoringPlugin;

impl Plugin for ScoringPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(
            Update,
            (
                update_score_system,
                update_berserk_system,
                check_berserk_activation,
            ).run_if(in_state(GameState::Playing)),
        );
    }
}

/// Update score chain timer
fn update_score_system(
    time: Res<Time>,
    mut score: ResMut<ScoreSystem>,
) {
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

/// Track previous button state for edge detection
#[derive(Resource, Default)]
struct BerserkButtonState {
    was_pressed: bool,
}

/// Check for berserk activation (player presses activate button)
fn check_berserk_activation(
    keyboard: Res<ButtonInput<KeyCode>>,
    joystick: Res<JoystickState>,
    mut berserk: ResMut<BerserkSystem>,
    mut activate_events: EventWriter<BerserkActivatedEvent>,
    mut button_state: Local<BerserkButtonState>,
) {
    // Activate with E key or controller Y button (formation switch in Python game)
    let y_pressed = joystick.formation_switch();
    let y_just_pressed = y_pressed && !button_state.was_pressed;
    button_state.was_pressed = y_pressed;

    if keyboard.just_pressed(KeyCode::KeyE) || y_just_pressed {
        if berserk.activate() {
            activate_events.send(BerserkActivatedEvent);
            info!("BERSERK MODE ACTIVATED!");
        }
    }
}
