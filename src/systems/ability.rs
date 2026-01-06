//! Ship Ability System
//!
//! Active abilities with cooldowns, activated via RT/Shift.
//! Each ship has a unique ability based on ShipDef.special.

use bevy::prelude::*;

use crate::core::game_state::GameState;
use crate::entities::player::{Movement, Player, ShipStats};
use crate::systems::joystick::JoystickState;

/// Ability types matching ShipDef.special descriptions
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum AbilityType {
    /// +50% speed burst (Minmatar frigates)
    Overdrive,
    /// Invulnerable dash (Minmatar assault)
    Afterburner,
    /// Triple spread shot (Minmatar destroyers)
    RocketBarrage,
    /// Extended laser range (Amarr frigates)
    Scorch,
    /// -50% damage taken (Amarr assault)
    ArmorHardener,
    /// Summon combat drone (Amarr destroyers)
    DeployDrone,
    /// 4 missiles at once (Caldari frigates)
    Salvo,
    /// Instant shield regen (Caldari assault)
    ShieldBoost,
    /// Slow nearby enemies (Caldari destroyers)
    WarpDisruptor,
    /// 2 autonomous fighters (Gallente frigates)
    DroneBay,
    /// Heal over time (Gallente assault)
    ArmorRepair,
    /// +100% damage when close (Gallente destroyers)
    CloseRange,
    /// No ability
    None,
}

impl AbilityType {
    /// Parse ability type from ShipDef.special string
    pub fn from_special(special: &str) -> Self {
        let lower = special.to_lowercase();
        if lower.contains("overdrive") {
            AbilityType::Overdrive
        } else if lower.contains("afterburner") {
            AbilityType::Afterburner
        } else if lower.contains("rocket barrage") || lower.contains("triple spread") {
            AbilityType::RocketBarrage
        } else if lower.contains("scorch") {
            AbilityType::Scorch
        } else if lower.contains("armor hardener") || lower.contains("hardener") {
            AbilityType::ArmorHardener
        } else if lower.contains("deploy drone") {
            AbilityType::DeployDrone
        } else if lower.contains("salvo") {
            AbilityType::Salvo
        } else if lower.contains("shield boost") {
            AbilityType::ShieldBoost
        } else if lower.contains("warp disruptor") || lower.contains("slow") {
            AbilityType::WarpDisruptor
        } else if lower.contains("drones:") || lower.contains("autonomous") {
            AbilityType::DroneBay
        } else if lower.contains("armor repair") || lower.contains("heal over time") {
            AbilityType::ArmorRepair
        } else if lower.contains("close range") {
            AbilityType::CloseRange
        } else {
            AbilityType::None
        }
    }

    /// Get cooldown duration in seconds
    pub fn cooldown(&self) -> f32 {
        match self {
            AbilityType::Overdrive => 8.0,
            AbilityType::Afterburner => 10.0,
            AbilityType::RocketBarrage => 6.0,
            AbilityType::Scorch => 12.0,
            AbilityType::ArmorHardener => 15.0,
            AbilityType::DeployDrone => 20.0,
            AbilityType::Salvo => 5.0,
            AbilityType::ShieldBoost => 10.0,
            AbilityType::WarpDisruptor => 12.0,
            AbilityType::DroneBay => 25.0,
            AbilityType::ArmorRepair => 8.0,
            AbilityType::CloseRange => 6.0,
            AbilityType::None => 0.0,
        }
    }

    /// Get effect duration in seconds (0 = instant)
    pub fn duration(&self) -> f32 {
        match self {
            AbilityType::Overdrive => 3.0,
            AbilityType::Afterburner => 1.5,   // Short dash
            AbilityType::RocketBarrage => 0.0, // Instant burst
            AbilityType::Scorch => 5.0,
            AbilityType::ArmorHardener => 4.0,
            AbilityType::DeployDrone => 15.0, // Drone lifetime
            AbilityType::Salvo => 0.0,        // Instant burst
            AbilityType::ShieldBoost => 0.0,  // Instant heal
            AbilityType::WarpDisruptor => 3.0,
            AbilityType::DroneBay => 20.0,   // Drone lifetime
            AbilityType::ArmorRepair => 5.0, // HoT duration
            AbilityType::CloseRange => 4.0,
            AbilityType::None => 0.0,
        }
    }

    /// Get capacitor cost
    pub fn capacitor_cost(&self) -> f32 {
        match self {
            AbilityType::Overdrive => 25.0,
            AbilityType::Afterburner => 35.0,
            AbilityType::RocketBarrage => 20.0,
            AbilityType::Scorch => 15.0,
            AbilityType::ArmorHardener => 30.0,
            AbilityType::DeployDrone => 40.0,
            AbilityType::Salvo => 20.0,
            AbilityType::ShieldBoost => 50.0,
            AbilityType::WarpDisruptor => 25.0,
            AbilityType::DroneBay => 50.0,
            AbilityType::ArmorRepair => 35.0,
            AbilityType::CloseRange => 20.0,
            AbilityType::None => 0.0,
        }
    }

    /// Display name for HUD
    pub fn name(&self) -> &'static str {
        match self {
            AbilityType::Overdrive => "OVERDRIVE",
            AbilityType::Afterburner => "AFTERBURNER",
            AbilityType::RocketBarrage => "ROCKET BARRAGE",
            AbilityType::Scorch => "SCORCH",
            AbilityType::ArmorHardener => "ARMOR HARDENER",
            AbilityType::DeployDrone => "DEPLOY DRONE",
            AbilityType::Salvo => "SALVO",
            AbilityType::ShieldBoost => "SHIELD BOOST",
            AbilityType::WarpDisruptor => "WARP DISRUPTOR",
            AbilityType::DroneBay => "DRONE BAY",
            AbilityType::ArmorRepair => "ARMOR REPAIR",
            AbilityType::CloseRange => "CLOSE RANGE",
            AbilityType::None => "",
        }
    }
}

/// Ability component attached to player
#[derive(Component, Debug, Clone)]
pub struct Ability {
    pub ability_type: AbilityType,
    /// Time remaining until ability can be used again
    pub cooldown_remaining: f32,
    /// Time remaining for active effect (0 = inactive)
    pub effect_remaining: f32,
    /// Whether ability is currently active
    pub is_active: bool,
}

impl Ability {
    pub fn new(ability_type: AbilityType) -> Self {
        Self {
            ability_type,
            cooldown_remaining: 0.0,
            effect_remaining: 0.0,
            is_active: false,
        }
    }

    /// Check if ability can be activated
    pub fn can_activate(&self, capacitor: f32) -> bool {
        self.cooldown_remaining <= 0.0
            && !self.is_active
            && capacitor >= self.ability_type.capacitor_cost()
            && self.ability_type != AbilityType::None
    }

    /// Activate the ability
    pub fn activate(&mut self) {
        self.is_active = true;
        self.effect_remaining = self.ability_type.duration();
        self.cooldown_remaining = self.ability_type.cooldown();
    }

    /// Get cooldown progress (0.0 = on cooldown, 1.0 = ready)
    pub fn cooldown_progress(&self) -> f32 {
        if self.ability_type.cooldown() <= 0.0 {
            return 1.0;
        }
        1.0 - (self.cooldown_remaining / self.ability_type.cooldown()).clamp(0.0, 1.0)
    }
}

/// Event: Ability was activated
#[derive(Event)]
pub struct AbilityActivatedEvent {
    pub ability_type: AbilityType,
    pub player_entity: Entity,
}

/// Event: Ability effect ended
#[derive(Event)]
#[allow(dead_code)]
pub struct AbilityEndedEvent {
    pub ability_type: AbilityType,
    pub player_entity: Entity,
}

/// Temporary effect modifiers while ability is active
#[derive(Component, Debug, Clone, Default)]
pub struct AbilityEffects {
    /// Speed multiplier (1.0 = normal)
    pub speed_multiplier: f32,
    /// Damage taken multiplier (1.0 = normal, 0.5 = half damage)
    pub damage_taken_multiplier: f32,
    /// Damage dealt multiplier (1.0 = normal)
    pub damage_dealt_multiplier: f32,
    /// Weapon range multiplier
    pub range_multiplier: f32,
    /// Invulnerable (afterburner dash)
    pub invulnerable: bool,
    /// Extra projectiles per shot
    pub extra_projectiles: u32,
}

impl AbilityEffects {
    pub fn reset(&mut self) {
        self.speed_multiplier = 1.0;
        self.damage_taken_multiplier = 1.0;
        self.damage_dealt_multiplier = 1.0;
        self.range_multiplier = 1.0;
        self.invulnerable = false;
        self.extra_projectiles = 0;
    }
}

/// Plugin for ability system
pub struct AbilityPlugin;

impl Plugin for AbilityPlugin {
    fn build(&self, app: &mut App) {
        app.add_event::<AbilityActivatedEvent>()
            .add_event::<AbilityEndedEvent>()
            .add_systems(
                Update,
                (
                    ability_input,
                    ability_update_cooldowns,
                    ability_apply_effects,
                    ability_handle_instant_effects,
                    ability_end_effects,
                )
                    .chain()
                    .run_if(in_state(GameState::Playing)),
            );
    }
}

/// Handle ability input (RT or Shift)
fn ability_input(
    keyboard: Res<ButtonInput<KeyCode>>,
    joystick: Res<JoystickState>,
    mut query: Query<(Entity, &mut Ability, &mut ShipStats), With<Player>>,
    mut events: EventWriter<AbilityActivatedEvent>,
) {
    let ability_pressed =
        keyboard.just_pressed(KeyCode::ShiftLeft) || joystick.right_trigger_pressed();

    if !ability_pressed {
        return;
    }

    for (entity, mut ability, mut stats) in query.iter_mut() {
        if ability.can_activate(stats.capacitor) {
            // Consume capacitor
            stats.capacitor -= ability.ability_type.capacitor_cost();

            // Activate ability
            ability.activate();

            // Send event
            events.send(AbilityActivatedEvent {
                ability_type: ability.ability_type,
                player_entity: entity,
            });

            info!(
                "Ability activated: {:?} (cap: {:.0})",
                ability.ability_type, stats.capacitor
            );
        }
    }
}

/// Update cooldown and effect timers
fn ability_update_cooldowns(time: Res<Time>, mut query: Query<&mut Ability, With<Player>>) {
    let dt = time.delta_secs();

    for mut ability in query.iter_mut() {
        // Tick cooldown
        if ability.cooldown_remaining > 0.0 {
            ability.cooldown_remaining = (ability.cooldown_remaining - dt).max(0.0);
        }

        // Tick effect duration
        if ability.is_active && ability.effect_remaining > 0.0 {
            ability.effect_remaining = (ability.effect_remaining - dt).max(0.0);
        }
    }
}

/// Apply ability effects when active
fn ability_apply_effects(
    mut query: Query<(&Ability, &mut AbilityEffects, &mut ShipStats, &mut Movement), With<Player>>,
    time: Res<Time>,
) {
    for (ability, mut effects, mut stats, mut movement) in query.iter_mut() {
        // Reset effects first
        effects.reset();

        if !ability.is_active {
            continue;
        }

        // Apply effects based on ability type
        match ability.ability_type {
            AbilityType::Overdrive => {
                effects.speed_multiplier = 1.5;
            }
            AbilityType::Afterburner => {
                effects.speed_multiplier = 2.0;
                effects.invulnerable = true;
            }
            AbilityType::RocketBarrage => {
                effects.extra_projectiles = 2; // Triple shot
            }
            AbilityType::Scorch => {
                effects.range_multiplier = 1.5;
            }
            AbilityType::ArmorHardener => {
                effects.damage_taken_multiplier = 0.5;
            }
            AbilityType::Salvo => {
                effects.extra_projectiles = 3; // 4 missiles total
            }
            AbilityType::ShieldBoost => {
                // Instant effect - handled in ability_activated
            }
            AbilityType::ArmorRepair => {
                // Heal over time
                let heal_per_sec = stats.max_armor * 0.15; // 15% armor per second
                stats.armor = (stats.armor + heal_per_sec * time.delta_secs()).min(stats.max_armor);
            }
            AbilityType::CloseRange => {
                effects.damage_dealt_multiplier = 2.0;
            }
            AbilityType::WarpDisruptor | AbilityType::DeployDrone | AbilityType::DroneBay => {
                // These spawn entities - handled elsewhere
            }
            AbilityType::None => {}
        }

        // Apply speed effect to movement
        if effects.speed_multiplier != 1.0 {
            movement.max_speed *= effects.speed_multiplier;
        }
    }
}

/// Handle instant ability effects (Shield Boost, etc.)
fn ability_handle_instant_effects(
    mut ability_events: EventReader<AbilityActivatedEvent>,
    mut query: Query<&mut ShipStats, With<Player>>,
) {
    for event in ability_events.read() {
        let Ok(mut stats) = query.get_mut(event.player_entity) else {
            continue;
        };

        match event.ability_type {
            AbilityType::ShieldBoost => {
                // Instant shield restore - 50% of max shield
                let heal_amount = stats.max_shield * 0.5;
                stats.shield = (stats.shield + heal_amount).min(stats.max_shield);
                info!(
                    "Shield Boost: +{:.0} shield ({:.0}/{:.0})",
                    heal_amount, stats.shield, stats.max_shield
                );
            }
            AbilityType::Salvo | AbilityType::RocketBarrage => {
                // Instant burst abilities - already handled by weapon system via extra_projectiles
                // But we mark the ability as done since duration is 0
            }
            _ => {
                // Other abilities have duration-based effects or are entity spawners
            }
        }
    }
}

/// End ability effects when duration expires
fn ability_end_effects(
    mut query: Query<(Entity, &mut Ability), With<Player>>,
    mut events: EventWriter<AbilityEndedEvent>,
) {
    for (entity, mut ability) in query.iter_mut() {
        if ability.is_active && ability.effect_remaining <= 0.0 {
            ability.is_active = false;

            events.send(AbilityEndedEvent {
                ability_type: ability.ability_type,
                player_entity: entity,
            });

            info!("Ability ended: {:?}", ability.ability_type);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_ability_type_from_special() {
        assert_eq!(
            AbilityType::from_special("Overdrive: +50% speed burst"),
            AbilityType::Overdrive
        );
        assert_eq!(
            AbilityType::from_special("Shield Boost: Instant regen"),
            AbilityType::ShieldBoost
        );
        assert_eq!(
            AbilityType::from_special("Drones: 2 autonomous fighters"),
            AbilityType::DroneBay
        );
        assert_eq!(
            AbilityType::from_special("Unknown ability"),
            AbilityType::None
        );
    }

    #[test]
    fn test_ability_can_activate() {
        let ability = Ability::new(AbilityType::Overdrive);
        assert!(ability.can_activate(100.0)); // Has enough cap
        assert!(!ability.can_activate(10.0)); // Not enough cap

        let mut ability_on_cd = Ability::new(AbilityType::Overdrive);
        ability_on_cd.cooldown_remaining = 5.0;
        assert!(!ability_on_cd.can_activate(100.0)); // On cooldown
    }

    #[test]
    fn test_ability_activate() {
        let mut ability = Ability::new(AbilityType::Overdrive);
        ability.activate();

        assert!(ability.is_active);
        assert_eq!(ability.effect_remaining, 3.0); // Overdrive duration
        assert_eq!(ability.cooldown_remaining, 8.0); // Overdrive cooldown
    }

    #[test]
    fn test_ability_cooldown_progress() {
        let mut ability = Ability::new(AbilityType::Overdrive);
        assert_eq!(ability.cooldown_progress(), 1.0); // Ready

        ability.cooldown_remaining = 4.0; // Half cooldown
        assert!((ability.cooldown_progress() - 0.5).abs() < 0.01);

        ability.cooldown_remaining = 8.0; // Full cooldown
        assert_eq!(ability.cooldown_progress(), 0.0);
    }
}
