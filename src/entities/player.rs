//! Player Ship Entity
//!
//! The player-controlled ship with EVE-style shield/armor/hull.

use bevy::prelude::*;
use crate::core::*;

/// Marker component for the player entity
#[derive(Component, Debug)]
pub struct Player;

/// Player ship stats
#[derive(Component, Debug, Clone)]
pub struct ShipStats {
    /// EVE Online type ID
    pub type_id: u32,
    /// Ship display name
    pub name: String,
    /// Maximum shield HP
    pub max_shield: f32,
    /// Current shield HP
    pub shield: f32,
    /// Shield recharge rate per second
    pub shield_recharge: f32,
    /// Time until shields start recharging
    pub shield_recharge_delay: f32,
    /// Timer for shield recharge delay
    pub shield_timer: f32,
    /// Maximum armor HP
    pub max_armor: f32,
    /// Current armor HP
    pub armor: f32,
    /// Maximum hull HP
    pub max_hull: f32,
    /// Current hull HP
    pub hull: f32,
    /// Maximum capacitor (GJ)
    pub max_capacitor: f32,
    /// Current capacitor
    pub capacitor: f32,
    /// Capacitor recharge rate per second
    pub capacitor_recharge: f32,
}

impl Default for ShipStats {
    fn default() -> Self {
        Self {
            type_id: 587, // Rifter
            name: "Rifter".into(),
            max_shield: PLAYER_DEFAULT_SHIELD,
            shield: PLAYER_DEFAULT_SHIELD,
            shield_recharge: PLAYER_SHIELD_RECHARGE_RATE,
            shield_recharge_delay: PLAYER_SHIELD_RECHARGE_DELAY,
            shield_timer: 0.0,
            max_armor: PLAYER_DEFAULT_ARMOR,
            armor: PLAYER_DEFAULT_ARMOR,
            max_hull: PLAYER_DEFAULT_HULL,
            hull: PLAYER_DEFAULT_HULL,
            max_capacitor: CAP_FRIGATE,
            capacitor: CAP_FRIGATE,
            capacitor_recharge: 10.0,
        }
    }
}

impl ShipStats {
    /// Create stats for a ship by type ID
    pub fn from_type_id(type_id: u32) -> Self {
        let (name, cap, shield_mult, armor_mult) = match type_id {
            // Minmatar Frigates
            587 => ("Rifter", CAP_FRIGATE, 0.8, 1.2),
            585 => ("Slasher", CAP_FRIGATE * 0.9, 0.7, 1.0),
            586 => ("Probe", CAP_FRIGATE * 0.8, 0.6, 0.8),
            598 => ("Breacher", CAP_FRIGATE, 1.0, 0.9),
            // Amarr Frigates
            597 => ("Punisher", CAP_FRIGATE * 1.2, 0.6, 1.5),
            589 => ("Executioner", CAP_FRIGATE, 0.7, 1.1),
            // Caldari Frigates
            603 => ("Merlin", CAP_FRIGATE, 1.3, 0.7),
            602 => ("Kestrel", CAP_FRIGATE * 1.1, 1.2, 0.6),
            // Gallente Frigates
            593 => ("Tristan", CAP_FRIGATE * 1.1, 0.9, 1.1),
            594 => ("Incursus", CAP_FRIGATE, 0.8, 1.3),
            _ => ("Unknown", CAP_FRIGATE, 1.0, 1.0),
        };

        Self {
            type_id,
            name: name.into(),
            max_shield: PLAYER_DEFAULT_SHIELD * shield_mult,
            shield: PLAYER_DEFAULT_SHIELD * shield_mult,
            max_armor: PLAYER_DEFAULT_ARMOR * armor_mult,
            armor: PLAYER_DEFAULT_ARMOR * armor_mult,
            max_capacitor: cap,
            capacitor: cap,
            ..default()
        }
    }

    /// Take damage with EVE-style damage application order
    pub fn take_damage(&mut self, damage: f32, damage_type: DamageType) -> bool {
        // Apply damage type resistances (simplified)
        let resistance = match damage_type {
            DamageType::EM => 0.0,        // Shield weak to EM
            DamageType::Thermal => 0.2,
            DamageType::Kinetic => 0.4,
            DamageType::Explosive => 0.5, // Shield strong vs Explosive
        };

        let mut remaining = damage * (1.0 - resistance);

        // Damage order: Shield -> Armor -> Hull
        if self.shield > 0.0 {
            let shield_damage = remaining.min(self.shield);
            self.shield -= shield_damage;
            remaining -= shield_damage;
            self.shield_timer = self.shield_recharge_delay;
        }

        if remaining > 0.0 && self.armor > 0.0 {
            let armor_damage = remaining.min(self.armor);
            self.armor -= armor_damage;
            remaining -= armor_damage;
        }

        if remaining > 0.0 {
            self.hull -= remaining;
        }

        // Return true if ship is destroyed
        self.hull <= 0.0
    }

    /// Update shield recharge
    pub fn update(&mut self, dt: f32) {
        // Shield recharge after delay
        if self.shield_timer > 0.0 {
            self.shield_timer -= dt;
        } else if self.shield < self.max_shield {
            self.shield = (self.shield + self.shield_recharge * dt).min(self.max_shield);
        }

        // Capacitor recharge
        if self.capacitor < self.max_capacitor {
            self.capacitor = (self.capacitor + self.capacitor_recharge * dt).min(self.max_capacitor);
        }
    }

    /// Get health percentage (combined)
    pub fn health_percent(&self) -> f32 {
        let total_max = self.max_shield + self.max_armor + self.max_hull;
        let total_current = self.shield + self.armor + self.hull;
        total_current / total_max
    }
}

/// Player movement component
#[derive(Component, Debug, Clone)]
pub struct Movement {
    /// Current velocity
    pub velocity: Vec2,
    /// Maximum speed
    pub max_speed: f32,
    /// Acceleration
    pub acceleration: f32,
    /// Deceleration (friction)
    pub friction: f32,
}

impl Default for Movement {
    fn default() -> Self {
        Self {
            velocity: Vec2::ZERO,
            max_speed: PLAYER_SPEED,
            acceleration: 1500.0,
            friction: 8.0,
        }
    }
}

/// Player weapon component
#[derive(Component, Debug, Clone)]
pub struct Weapon {
    /// Weapon type
    pub weapon_type: WeaponType,
    /// Shots per second
    pub fire_rate: f32,
    /// Time until next shot
    pub cooldown: f32,
    /// Bullet speed
    pub bullet_speed: f32,
    /// Damage per hit
    pub damage: f32,
    /// Capacitor cost per shot
    pub cap_usage: f32,
    /// Current aim direction
    pub aim_direction: Vec2,
}

impl Default for Weapon {
    fn default() -> Self {
        Self {
            weapon_type: WeaponType::Autocannon,
            fire_rate: PLAYER_FIRE_RATE,
            cooldown: 0.0,
            bullet_speed: PLAYER_BULLET_SPEED,
            damage: PLAYER_BULLET_DAMAGE,
            cap_usage: 5.0,
            aim_direction: Vec2::Y, // Up by default
        }
    }
}

/// Player hitbox for collision detection
#[derive(Component, Debug)]
pub struct Hitbox {
    pub radius: f32,
}

impl Default for Hitbox {
    fn default() -> Self {
        Self {
            radius: PLAYER_HITBOX_SIZE,
        }
    }
}

/// Bundle for spawning a player ship
#[derive(Bundle)]
pub struct PlayerBundle {
    pub player: Player,
    pub stats: ShipStats,
    pub movement: Movement,
    pub weapon: Weapon,
    pub hitbox: Hitbox,
    pub sprite: Sprite,
    pub transform: Transform,
}

impl Default for PlayerBundle {
    fn default() -> Self {
        Self {
            player: Player,
            stats: ShipStats::default(),
            movement: Movement::default(),
            weapon: Weapon::default(),
            hitbox: Hitbox::default(),
            sprite: Sprite {
                color: COLOR_MINMATAR,
                custom_size: Some(Vec2::splat(PLAYER_SPRITE_SIZE)),
                ..default()
            },
            transform: Transform::from_xyz(0.0, -250.0, LAYER_PLAYER),
        }
    }
}

/// Player plugin
pub struct PlayerPlugin;

impl Plugin for PlayerPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(OnEnter(GameState::Playing), spawn_player)
            .add_systems(
                Update,
                (
                    player_movement,
                    player_shooting,
                    update_player_stats,
                ).run_if(in_state(GameState::Playing)),
            )
            .add_systems(OnExit(GameState::Playing), despawn_player);
    }
}

/// Spawn player at start of gameplay
fn spawn_player(
    mut commands: Commands,
    selected_ship: Res<SelectedShip>,
    sprite_cache: Res<crate::assets::ShipSpriteCache>,
) {
    let ship = selected_ship.ship;
    let type_id = ship.type_id();

    let mut stats = ShipStats::from_type_id(type_id);
    // Apply ship health multipliers
    stats.max_shield *= ship.health_mult();
    stats.shield *= ship.health_mult();
    stats.max_armor *= ship.health_mult();
    stats.armor *= ship.health_mult();
    stats.max_hull *= ship.health_mult();
    stats.hull *= ship.health_mult();

    // Create movement with ship speed multiplier
    let mut movement = Movement::default();
    movement.max_speed *= ship.speed_mult();
    movement.acceleration *= ship.speed_mult();

    let base_color = COLOR_MINMATAR;

    // Check if we have a cached sprite for this ship
    let ship_sprite = sprite_cache.get(type_id);

    if ship_sprite.is_some() {
        info!("Using EVE sprite for ship type {}", type_id);
    } else {
        warn!("No sprite found for ship type {}, using fallback", type_id);
    }

    // Create ship entity with just the sprite (no parent container needed)
    // This is simpler and avoids potential hierarchy issues
    if let Some(texture) = ship_sprite {
        // EVE ship renders face RIGHT by default
        // Rotate 90° counterclockwise to face UP for player
        commands.spawn((
            Player,
            stats,
            movement,
            Weapon::default(),
            Hitbox::default(),
            super::collectible::PowerupEffects::default(),
            Sprite {
                image: texture,
                custom_size: Some(Vec2::new(96.0, 96.0)),
                ..default()
            },
            Transform::from_xyz(0.0, -250.0, LAYER_PLAYER)
                .with_rotation(Quat::from_rotation_z(std::f32::consts::FRAC_PI_2)),
        ));
    } else {
        // Fallback: simple colored sprite
        commands.spawn((
            Player,
            stats,
            movement,
            Weapon::default(),
            Hitbox::default(),
            super::collectible::PowerupEffects::default(),
            Sprite {
                color: base_color,
                custom_size: Some(Vec2::new(48.0, 64.0)),
                ..default()
            },
            Transform::from_xyz(0.0, -250.0, LAYER_PLAYER),
        ));
    }

    info!("Player spawned!");
}

/// Lighten a color
fn lighten_color(color: Color, amount: f32) -> Color {
    let srgba = color.to_srgba();
    Color::srgba(
        (srgba.red + amount).min(1.0),
        (srgba.green + amount).min(1.0),
        (srgba.blue + amount).min(1.0),
        srgba.alpha,
    )
}

/// Darken a color
fn darken_color(color: Color, amount: f32) -> Color {
    let srgba = color.to_srgba();
    Color::srgba(
        (srgba.red - amount).max(0.0),
        (srgba.green - amount).max(0.0),
        (srgba.blue - amount).max(0.0),
        srgba.alpha,
    )
}

/// Player movement system
fn player_movement(
    time: Res<Time>,
    keyboard: Res<ButtonInput<KeyCode>>,
    joystick: Res<crate::systems::JoystickState>,
    mut query: Query<(&mut Transform, &mut Movement), With<Player>>,
    berserk: Res<BerserkSystem>,
) {
    let Ok((mut transform, mut movement)) = query.get_single_mut() else {
        return;
    };

    // Get keyboard input direction
    let mut input = Vec2::ZERO;
    if keyboard.pressed(KeyCode::KeyW) || keyboard.pressed(KeyCode::ArrowUp) {
        input.y += 1.0;
    }
    if keyboard.pressed(KeyCode::KeyS) || keyboard.pressed(KeyCode::ArrowDown) {
        input.y -= 1.0;
    }
    if keyboard.pressed(KeyCode::KeyA) || keyboard.pressed(KeyCode::ArrowLeft) {
        input.x -= 1.0;
    }
    if keyboard.pressed(KeyCode::KeyD) || keyboard.pressed(KeyCode::ArrowRight) {
        input.x += 1.0;
    }

    // Combine with joystick input
    let joy_input = joystick.movement();
    if joy_input.length_squared() > input.length_squared() {
        input = joy_input;
    }

    let dt = time.delta_secs();
    let speed_mult = berserk.speed_mult();

    // Apply acceleration
    if input != Vec2::ZERO {
        let input_normalized = input.normalize();
        let accel = movement.acceleration;
        movement.velocity += input_normalized * accel * dt;
    }

    // Apply friction
    let friction = movement.friction;
    movement.velocity *= 1.0 - friction * dt;

    // Clamp speed
    let max_speed = movement.max_speed * speed_mult;
    if movement.velocity.length() > max_speed {
        movement.velocity = movement.velocity.normalize() * max_speed;
    }

    // Update position
    transform.translation.x += movement.velocity.x * dt;
    transform.translation.y += movement.velocity.y * dt;

    // Clamp to screen bounds
    let half_width = SCREEN_WIDTH / 2.0 - PLAYER_SPRITE_SIZE / 2.0;
    let half_height = SCREEN_HEIGHT / 2.0 - PLAYER_SPRITE_SIZE / 2.0;
    transform.translation.x = transform.translation.x.clamp(-half_width, half_width);
    transform.translation.y = transform.translation.y.clamp(-half_height, half_height);
}

/// Player shooting system
/// Note: Python game removed capacitor - unlimited ammo, only heat matters
fn player_shooting(
    time: Res<Time>,
    keyboard: Res<ButtonInput<KeyCode>>,
    joystick: Res<crate::systems::JoystickState>,
    mut query: Query<(&Transform, &mut Weapon), With<Player>>,
    mut fire_events: EventWriter<PlayerFireEvent>,
    berserk: Res<BerserkSystem>,
    mut heat_system: ResMut<crate::systems::ComboHeatSystem>,
) {
    let Ok((transform, mut weapon)) = query.get_single_mut() else {
        return;
    };

    let dt = time.delta_secs();

    // Update cooldown
    if weapon.cooldown > 0.0 {
        weapon.cooldown -= dt;
    }

    // Update aim direction from keyboard
    let mut aim = Vec2::ZERO;
    if keyboard.pressed(KeyCode::ArrowUp) || keyboard.pressed(KeyCode::KeyI) {
        aim.y += 1.0;
    }
    if keyboard.pressed(KeyCode::ArrowDown) || keyboard.pressed(KeyCode::KeyK) {
        aim.y -= 1.0;
    }
    if keyboard.pressed(KeyCode::ArrowLeft) || keyboard.pressed(KeyCode::KeyJ) {
        aim.x -= 1.0;
    }
    if keyboard.pressed(KeyCode::ArrowRight) || keyboard.pressed(KeyCode::KeyL) {
        aim.x += 1.0;
    }

    // Use right stick for aiming if available
    let joy_aim = Vec2::new(joystick.right_x, -joystick.right_y);
    if joy_aim.length_squared() > 0.3 {
        aim = joy_aim;
    }

    if aim != Vec2::ZERO {
        weapon.aim_direction = aim.normalize();
    }

    // Fire if holding space/trigger (no capacitor requirement - matches Python)
    let fire_pressed = keyboard.pressed(KeyCode::Space) || joystick.fire();

    if fire_pressed && weapon.cooldown <= 0.0 {
        // Track heat (doesn't block firing, just affects fire rate)
        heat_system.on_fire();

        // Calculate fire rate with modifiers:
        // - Base fire rate
        // - Berserk bonus (1.5x when active)
        // - Heat penalty (0.7x when overheated)
        let berserk_mult = if berserk.is_active { 1.5 } else { 1.0 };
        let heat_mult = heat_system.fire_rate_mult();
        let fire_rate = weapon.fire_rate * berserk_mult * heat_mult;
        weapon.cooldown = 1.0 / fire_rate;

        // Send fire event
        fire_events.send(PlayerFireEvent {
            position: transform.translation.truncate(),
            direction: weapon.aim_direction,
            weapon_type: weapon.weapon_type,
        });
    }
}

/// Update player stats (shield recharge, etc)
fn update_player_stats(
    time: Res<Time>,
    mut query: Query<&mut ShipStats, With<Player>>,
) {
    let Ok(mut stats) = query.get_single_mut() else {
        return;
    };

    stats.update(time.delta_secs());
}

/// Despawn player when leaving gameplay
fn despawn_player(
    mut commands: Commands,
    query: Query<Entity, With<Player>>,
) {
    for entity in query.iter() {
        commands.entity(entity).despawn_recursive();
    }
}
