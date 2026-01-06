//! THE LAST STAND - CNS Kairiola's Final Hours
//!
//! A fixed-platform survival mode where you command the Leviathan-class titan
//! CNS Kairiola during its final evacuation. Based on the historical
//! CNS Shiigeru (YC115) sacrificial crash into Gallente Prime.
//!
//! MECHANICS:
//! - NO MOVEMENT - You are a fixed weapons platform
//! - HEAT IS UNIVERSAL - Every action generates heat
//! - EVACUATION PROGRESS - 0-100% replaces waves
//! - CONTROLS: Fighter Launch, ECM Burst, Shield Booster, Doomsday

#![allow(dead_code)]

use bevy::prelude::*;

/// Evacuation milestone messages from Admiral Tovil-Toba
pub const EVACUATION_MILESTONES: [(u32, &str); 10] = [
    (10, "First transports away. The State remembers you."),
    (20, "Twenty percent evacuated. Hold the line!"),
    (30, "Shields critical. Reroute auxiliary power."),
    (
        40,
        "Federation dreadnoughts inbound. All batteries, fire at will!",
    ),
    (50, "Halfway there. The Megas approaches."),
    (
        60,
        "Structural integrity failing. This ship will not surrender!",
    ),
    (70, "Most civilians are clear. We can do this."),
    (
        80,
        "Admiral: 'Set collision course. The Federation will remember this day.'",
    ),
    (90, "Final transports launching. Prepare for descent."),
    (
        100,
        "Evacuation complete. CNS Kairiola, you served the State well.",
    ),
];

/// Control bindings for Last Stand mode
#[derive(Debug, Clone, Copy)]
pub enum LastStandAction {
    /// RT - Launch fighter squadron
    FighterLaunch,
    /// LB - Area denial ECM burst
    EcmBurst,
    /// RB - Emergency shield booster
    ShieldBooster,
    /// Y - DOOMSDAY DEVICE (one use)
    Doomsday,
    /// A - Confirm descent (endgame only)
    ConfirmDescent,
}

impl LastStandAction {
    /// Heat cost for each action
    pub fn heat_cost(&self) -> f32 {
        match self {
            LastStandAction::FighterLaunch => 15.0,
            LastStandAction::EcmBurst => 25.0,
            LastStandAction::ShieldBooster => 30.0,
            LastStandAction::Doomsday => 100.0, // Maxes heat but worth it
            LastStandAction::ConfirmDescent => 0.0,
        }
    }

    /// Cooldown in seconds
    pub fn cooldown(&self) -> f32 {
        match self {
            LastStandAction::FighterLaunch => 3.0,
            LastStandAction::EcmBurst => 8.0,
            LastStandAction::ShieldBooster => 12.0,
            LastStandAction::Doomsday => 0.0, // One-time use
            LastStandAction::ConfirmDescent => 0.0,
        }
    }
}

/// The Last Stand game state resource
#[derive(Resource, Debug, Clone)]
pub struct LastStandState {
    /// Is Last Stand mode active
    pub active: bool,
    /// Current heat level (0-100)
    pub heat: f32,
    /// Heat dissipation rate per second
    pub heat_dissipation: f32,
    /// Evacuation progress (0-100)
    pub evacuation_progress: f32,
    /// Base evacuation rate per second
    pub evacuation_rate: f32,
    /// Shield HP (starts at 100)
    pub shield: f32,
    /// Armor HP (starts at 100)
    pub armor: f32,
    /// Hull HP (starts at 100)
    pub hull: f32,
    /// Fighter squadrons remaining
    pub fighters_remaining: u32,
    /// Doomsday device available
    pub doomsday_available: bool,
    /// Cooldowns for abilities
    pub fighter_cooldown: f32,
    pub ecm_cooldown: f32,
    pub shield_cooldown: f32,
    /// Time survived
    pub time_survived: f32,
    /// Enemies destroyed
    pub kills: u32,
    /// Next milestone index
    pub next_milestone: usize,
    /// In endgame descent phase
    pub in_descent: bool,
    /// Descent confirmed by player
    pub descent_confirmed: bool,
    /// Descent timer (for dramatic crash sequence)
    pub descent_timer: f32,
}

impl Default for LastStandState {
    fn default() -> Self {
        Self {
            active: false,
            heat: 0.0,
            heat_dissipation: 5.0, // 5% per second base
            evacuation_progress: 0.0,
            evacuation_rate: 1.5, // ~67 seconds to complete at base
            shield: 100.0,
            armor: 100.0,
            hull: 100.0,
            fighters_remaining: 6, // 6 fighter squadrons
            doomsday_available: true,
            fighter_cooldown: 0.0,
            ecm_cooldown: 0.0,
            shield_cooldown: 0.0,
            time_survived: 0.0,
            kills: 0,
            next_milestone: 0,
            in_descent: false,
            descent_confirmed: false,
            descent_timer: 0.0,
        }
    }
}

impl LastStandState {
    /// Start a new Last Stand mission
    pub fn start(&mut self) {
        *self = Self {
            active: true,
            ..Default::default()
        };
    }

    /// End the mission (player died or completed)
    pub fn end(&mut self) {
        self.active = false;
    }

    /// Check if an action can be performed (heat + cooldown)
    pub fn can_perform(&self, action: LastStandAction) -> bool {
        if !self.active {
            return false;
        }

        // During descent phase, only ConfirmDescent is allowed
        if self.in_descent && !matches!(action, LastStandAction::ConfirmDescent) {
            return false;
        }

        let heat_ok = self.heat + action.heat_cost() <= 100.0 || action.heat_cost() == 0.0;
        let cooldown_ok = match action {
            LastStandAction::FighterLaunch => {
                self.fighter_cooldown <= 0.0 && self.fighters_remaining > 0
            }
            LastStandAction::EcmBurst => self.ecm_cooldown <= 0.0,
            LastStandAction::ShieldBooster => self.shield_cooldown <= 0.0,
            LastStandAction::Doomsday => self.doomsday_available,
            LastStandAction::ConfirmDescent => self.in_descent && !self.descent_confirmed,
        };

        heat_ok && cooldown_ok
    }

    /// Perform an action (consume resources, apply effects)
    pub fn perform(&mut self, action: LastStandAction) -> bool {
        if !self.can_perform(action) {
            return false;
        }

        // Apply heat cost
        self.heat = (self.heat + action.heat_cost()).min(100.0);

        // Apply cooldown and specific effects
        match action {
            LastStandAction::FighterLaunch => {
                self.fighter_cooldown = action.cooldown();
                self.fighters_remaining -= 1;
            }
            LastStandAction::EcmBurst => {
                self.ecm_cooldown = action.cooldown();
            }
            LastStandAction::ShieldBooster => {
                self.shield_cooldown = action.cooldown();
                self.shield = (self.shield + 30.0).min(100.0);
            }
            LastStandAction::Doomsday => {
                self.doomsday_available = false;
                // Damage handled by calling system
            }
            LastStandAction::ConfirmDescent => {
                self.descent_confirmed = true;
            }
        }

        true
    }

    /// Update timers and state
    pub fn update(&mut self, dt: f32) -> LastStandEvent {
        if !self.active {
            return LastStandEvent::None;
        }

        self.time_survived += dt;

        // Heat dissipation
        self.heat = (self.heat - self.heat_dissipation * dt).max(0.0);

        // Cooldowns
        self.fighter_cooldown = (self.fighter_cooldown - dt).max(0.0);
        self.ecm_cooldown = (self.ecm_cooldown - dt).max(0.0);
        self.shield_cooldown = (self.shield_cooldown - dt).max(0.0);

        // In descent phase?
        if self.in_descent {
            if self.descent_confirmed {
                self.descent_timer += dt;
                if self.descent_timer >= 5.0 {
                    // Crash complete
                    return LastStandEvent::DescentComplete;
                }
            }
            return LastStandEvent::None;
        }

        // Evacuation progress (slowed by damage)
        let damage_penalty = 1.0 - (100.0 - self.hull) / 200.0; // Half speed at 0 hull
        self.evacuation_progress += self.evacuation_rate * damage_penalty * dt;

        // Check for milestone
        if self.next_milestone < EVACUATION_MILESTONES.len() {
            let (threshold, _message) = EVACUATION_MILESTONES[self.next_milestone];
            if self.evacuation_progress >= threshold as f32 {
                let milestone = self.next_milestone;
                self.next_milestone += 1;
                return LastStandEvent::Milestone(milestone);
            }
        }

        // Check for evacuation complete
        if self.evacuation_progress >= 100.0 {
            self.evacuation_progress = 100.0;
            self.in_descent = true;
            return LastStandEvent::EvacuationComplete;
        }

        // Check for death
        if self.hull <= 0.0 {
            return LastStandEvent::Destroyed;
        }

        LastStandEvent::None
    }

    /// Apply damage to the titan (shields → armor → hull)
    pub fn take_damage(&mut self, amount: f32) {
        let mut remaining = amount;

        // Shields first
        if self.shield > 0.0 {
            let absorbed = remaining.min(self.shield);
            self.shield -= absorbed;
            remaining -= absorbed;
        }

        // Then armor
        if remaining > 0.0 && self.armor > 0.0 {
            let absorbed = remaining.min(self.armor);
            self.armor -= absorbed;
            remaining -= absorbed;
        }

        // Finally hull
        if remaining > 0.0 {
            self.hull -= remaining;
        }
    }

    /// Get current health percentage (for HUD)
    pub fn total_health_pct(&self) -> f32 {
        (self.shield + self.armor + self.hull) / 300.0 * 100.0
    }

    /// Is the titan overheating?
    pub fn is_overheating(&self) -> bool {
        self.heat > 80.0
    }

    /// Get message for current milestone
    pub fn current_milestone_message(&self) -> Option<&'static str> {
        if self.next_milestone > 0 && self.next_milestone <= EVACUATION_MILESTONES.len() {
            Some(EVACUATION_MILESTONES[self.next_milestone - 1].1)
        } else {
            None
        }
    }
}

/// Events triggered by Last Stand state changes
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum LastStandEvent {
    None,
    /// Evacuation milestone reached (index into EVACUATION_MILESTONES)
    Milestone(usize),
    /// 100% evacuation - begin descent sequence
    EvacuationComplete,
    /// Player confirmed descent - crash into Gallente Prime
    DescentComplete,
    /// Titan destroyed before evacuation complete
    Destroyed,
}

// ============================================================================
// Components for Last Stand entities
// ============================================================================

/// Marker for the CNS Kairiola titan entity
#[derive(Component, Debug)]
pub struct KairiolaTitan;

/// Fighter squadron spawned by titan
#[derive(Component, Debug)]
pub struct TitanFighter {
    /// Time remaining before despawn
    pub lifetime: f32,
    /// Current target (if any)
    pub target: Option<Entity>,
}

/// ECM burst area effect
#[derive(Component, Debug)]
pub struct EcmBurst {
    /// Expansion radius
    pub radius: f32,
    /// Expansion speed
    pub speed: f32,
    /// Time remaining
    pub lifetime: f32,
}

/// Doomsday beam (one-time use)
#[derive(Component, Debug)]
pub struct DoomsdayBeam {
    /// Beam width
    pub width: f32,
    /// Damage per second
    pub damage_per_sec: f32,
    /// Duration remaining
    pub duration: f32,
}

// ============================================================================
// Tests
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn last_stand_starts_correctly() {
        let mut state = LastStandState::default();
        assert!(!state.active);

        state.start();
        assert!(state.active);
        assert_eq!(state.heat, 0.0);
        assert_eq!(state.evacuation_progress, 0.0);
        assert_eq!(state.fighters_remaining, 6);
        assert!(state.doomsday_available);
    }

    #[test]
    fn heat_dissipates() {
        let mut state = LastStandState::default();
        state.start();
        state.heat = 50.0;

        state.update(2.0); // 2 seconds

        // Should have lost 10% heat (5% per second)
        assert!(state.heat < 50.0);
        assert!(state.heat > 38.0); // Allow some tolerance
    }

    #[test]
    fn can_perform_checks_heat() {
        let mut state = LastStandState::default();
        state.start();
        state.heat = 90.0;

        // Fighter launch costs 15, would exceed 100
        assert!(!state.can_perform(LastStandAction::FighterLaunch));

        state.heat = 80.0;
        assert!(state.can_perform(LastStandAction::FighterLaunch));
    }

    #[test]
    fn can_perform_checks_cooldown() {
        let mut state = LastStandState::default();
        state.start();
        state.ecm_cooldown = 5.0;

        assert!(!state.can_perform(LastStandAction::EcmBurst));

        state.ecm_cooldown = 0.0;
        assert!(state.can_perform(LastStandAction::EcmBurst));
    }

    #[test]
    fn perform_action_applies_effects() {
        let mut state = LastStandState::default();
        state.start();

        assert!(state.perform(LastStandAction::FighterLaunch));
        assert_eq!(state.fighters_remaining, 5);
        assert_eq!(state.heat, 15.0);
        assert!(state.fighter_cooldown > 0.0);
    }

    #[test]
    fn doomsday_is_one_use() {
        let mut state = LastStandState::default();
        state.start();

        assert!(state.can_perform(LastStandAction::Doomsday));
        assert!(state.perform(LastStandAction::Doomsday));

        assert!(!state.can_perform(LastStandAction::Doomsday));
        assert!(!state.doomsday_available);
    }

    #[test]
    fn evacuation_progresses() {
        let mut state = LastStandState::default();
        state.start();

        state.update(10.0); // 10 seconds

        assert!(state.evacuation_progress > 10.0);
    }

    #[test]
    fn milestones_trigger() {
        let mut state = LastStandState::default();
        state.start();
        state.evacuation_progress = 9.0;

        let event = state.update(1.0);

        // Should trigger 10% milestone
        assert_eq!(event, LastStandEvent::Milestone(0));
        assert_eq!(state.next_milestone, 1);
    }

    #[test]
    fn damage_model_works() {
        let mut state = LastStandState::default();
        state.start();

        // Take 150 damage (should deplete shield and half armor)
        state.take_damage(150.0);

        assert_eq!(state.shield, 0.0);
        assert_eq!(state.armor, 50.0);
        assert_eq!(state.hull, 100.0);
    }

    #[test]
    fn death_triggers_event() {
        let mut state = LastStandState::default();
        state.start();
        state.hull = 5.0;
        state.shield = 0.0;
        state.armor = 0.0;

        state.take_damage(10.0);
        let event = state.update(0.1);

        assert_eq!(event, LastStandEvent::Destroyed);
    }

    #[test]
    fn evacuation_complete_triggers_descent() {
        let mut state = LastStandState::default();
        state.start();
        state.evacuation_progress = 99.0;
        state.next_milestone = 10; // Skip milestones

        let event = state.update(1.0);

        assert_eq!(event, LastStandEvent::EvacuationComplete);
        assert!(state.in_descent);
    }

    #[test]
    fn descent_confirmation() {
        let mut state = LastStandState::default();
        state.start();
        state.in_descent = true;

        assert!(state.can_perform(LastStandAction::ConfirmDescent));
        assert!(state.perform(LastStandAction::ConfirmDescent));
        assert!(state.descent_confirmed);
    }
}
