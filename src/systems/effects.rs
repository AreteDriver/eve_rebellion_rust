//! Visual Effects System
//!
//! Starfield, explosions, particle effects, screen shake, engine trails.

#![allow(dead_code)]

use crate::core::*;
use bevy::prelude::*;

/// Maximum particles to prevent slowdown during intense combat
const MAX_EXPLOSION_PARTICLES: usize = 500;
const MAX_ENGINE_PARTICLES: usize = 200;

/// Effects plugin
pub struct EffectsPlugin;

impl Plugin for EffectsPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<ScreenShake>()
            .add_systems(OnEnter(GameState::Playing), spawn_starfield)
            .add_systems(
                Update,
                (
                    update_starfield,
                    update_explosions,
                    update_screen_shake,
                    handle_explosion_events,
                    spawn_engine_trails,
                    update_engine_particles,
                )
                    .run_if(in_state(GameState::Playing)),
            )
            .add_systems(OnExit(GameState::Playing), cleanup_effects);
    }
}

// =============================================================================
// STARFIELD
// =============================================================================

/// Marker for star entities
#[derive(Component)]
pub struct Star {
    pub speed: f32,
    pub layer: u8,
}

/// Spawn scrolling starfield background
fn spawn_starfield(mut commands: Commands) {
    let mut rng = fastrand::Rng::new();

    // Spawn stars in 3 layers (parallax)
    for layer in 0..3 {
        let count = match layer {
            0 => 30, // Far stars (dim, slow)
            1 => 50, // Mid stars
            _ => 70, // Near stars (bright, fast)
        };

        let (speed, size, alpha) = match layer {
            0 => (20.0, 1.0, 0.3),
            1 => (40.0, 1.5, 0.5),
            _ => (80.0, 2.5, 0.8),
        };

        for _ in 0..count {
            let x = rng.f32() * SCREEN_WIDTH - SCREEN_WIDTH / 2.0;
            let y = rng.f32() * SCREEN_HEIGHT - SCREEN_HEIGHT / 2.0;

            commands.spawn((
                Star { speed, layer },
                Sprite {
                    color: Color::srgba(0.8, 0.85, 1.0, alpha),
                    custom_size: Some(Vec2::splat(size)),
                    ..default()
                },
                Transform::from_xyz(x, y, layer as f32),
            ));
        }
    }
}

/// Scroll stars downward
fn update_starfield(time: Res<Time>, mut query: Query<(&mut Transform, &Star)>) {
    let dt = time.delta_secs();

    for (mut transform, star) in query.iter_mut() {
        transform.translation.y -= star.speed * dt;

        // Wrap around
        if transform.translation.y < -SCREEN_HEIGHT / 2.0 - 10.0 {
            transform.translation.y = SCREEN_HEIGHT / 2.0 + 10.0;
            transform.translation.x = fastrand::f32() * SCREEN_WIDTH - SCREEN_WIDTH / 2.0;
        }
    }
}

// =============================================================================
// EXPLOSIONS
// =============================================================================

/// Explosion particle
#[derive(Component)]
pub struct ExplosionParticle {
    pub velocity: Vec2,
    pub lifetime: f32,
    pub max_lifetime: f32,
}

/// Handle explosion events with particle cap
fn handle_explosion_events(
    mut commands: Commands,
    mut events: EventReader<ExplosionEvent>,
    particle_query: Query<&ExplosionParticle>,
) {
    let current_count = particle_query.iter().count();
    let mut spawned = 0;

    for event in events.read() {
        // Check particle cap before spawning
        if current_count + spawned < MAX_EXPLOSION_PARTICLES {
            let new_count =
                spawn_explosion_capped(&mut commands, event.position, &event.size, event.color);
            spawned += new_count;
        }
    }
}

/// Spawn explosion particles (returns count spawned)
fn spawn_explosion_capped(
    commands: &mut Commands,
    position: Vec2,
    size: &ExplosionSize,
    color: Color,
) -> usize {
    let (count, speed, lifetime, particle_size) = match size {
        ExplosionSize::Tiny => (5, 50.0, 0.2, 3.0),
        ExplosionSize::Small => (12, 100.0, 0.4, 5.0),
        ExplosionSize::Medium => (20, 150.0, 0.5, 7.0),
        ExplosionSize::Large => (30, 200.0, 0.6, 10.0),
        ExplosionSize::Massive => (50, 300.0, 0.8, 15.0),
    };

    let mut rng = fastrand::Rng::new();

    for _ in 0..count {
        let angle = rng.f32() * std::f32::consts::TAU;
        let speed_var = speed * (0.5 + rng.f32() * 0.5);
        let velocity = Vec2::new(angle.cos(), angle.sin()) * speed_var;

        // Vary color slightly
        let color_var = Color::srgba(
            color.to_srgba().red * (0.8 + rng.f32() * 0.4),
            color.to_srgba().green * (0.7 + rng.f32() * 0.3),
            color.to_srgba().blue * (0.6 + rng.f32() * 0.2),
            1.0,
        );

        commands.spawn((
            ExplosionParticle {
                velocity,
                lifetime,
                max_lifetime: lifetime,
            },
            Sprite {
                color: color_var,
                custom_size: Some(Vec2::splat(particle_size * (0.5 + rng.f32() * 0.5))),
                ..default()
            },
            Transform::from_xyz(position.x, position.y, LAYER_EFFECTS),
        ));
    }

    count
}

/// Spawn explosion particles (legacy public API)
pub fn spawn_explosion(
    commands: &mut Commands,
    position: Vec2,
    size: &ExplosionSize,
    color: Color,
) {
    spawn_explosion_capped(commands, position, size, color);
}

/// Update explosion particles
fn update_explosions(
    mut commands: Commands,
    time: Res<Time>,
    mut query: Query<(Entity, &mut Transform, &mut ExplosionParticle, &mut Sprite)>,
) {
    let dt = time.delta_secs();

    for (entity, mut transform, mut particle, mut sprite) in query.iter_mut() {
        // Move
        transform.translation.x += particle.velocity.x * dt;
        transform.translation.y += particle.velocity.y * dt;

        // Slow down
        particle.velocity *= 1.0 - 3.0 * dt;

        // Fade out
        particle.lifetime -= dt;
        let alpha = (particle.lifetime / particle.max_lifetime).max(0.0);
        sprite.color = sprite.color.with_alpha(alpha);

        // Shrink
        if let Some(size) = sprite.custom_size {
            sprite.custom_size = Some(size * (1.0 - 0.5 * dt));
        }

        if particle.lifetime <= 0.0 {
            commands.entity(entity).despawn();
        }
    }
}

// =============================================================================
// SCREEN SHAKE
// =============================================================================

/// Screen shake state
#[derive(Resource, Default)]
pub struct ScreenShake {
    pub intensity: f32,
    pub duration: f32,
    pub timer: f32,
}

impl ScreenShake {
    /// Trigger a screen shake
    pub fn trigger(&mut self, intensity: f32, duration: f32) {
        if intensity > self.intensity || self.timer <= 0.0 {
            self.intensity = intensity;
            self.duration = duration;
            self.timer = duration;
        }
    }

    /// Small shake (player hit)
    pub fn small(&mut self) {
        self.trigger(5.0, 0.15);
    }

    /// Medium shake (enemy explosion)
    pub fn medium(&mut self) {
        self.trigger(8.0, 0.2);
    }

    /// Large shake (boss phase change)
    pub fn large(&mut self) {
        self.trigger(15.0, 0.3);
    }

    /// Massive shake (boss defeat)
    pub fn massive(&mut self) {
        self.trigger(25.0, 0.5);
    }
}

/// Handle screen shake events
fn update_screen_shake(
    time: Res<Time>,
    mut shake: ResMut<ScreenShake>,
    mut camera_query: Query<&mut Transform, With<Camera2d>>,
    mut shake_events: EventReader<ScreenShakeEvent>,
) {
    // Process new shake events
    for event in shake_events.read() {
        if event.intensity > shake.intensity {
            shake.intensity = event.intensity;
            shake.duration = event.duration;
            shake.timer = event.duration;
        }
    }

    let dt = time.delta_secs();

    if shake.timer > 0.0 {
        shake.timer -= dt;

        let progress = shake.timer / shake.duration;
        let current_intensity = shake.intensity * progress;

        if let Ok(mut transform) = camera_query.get_single_mut() {
            let offset_x = (fastrand::f32() - 0.5) * 2.0 * current_intensity;
            let offset_y = (fastrand::f32() - 0.5) * 2.0 * current_intensity;
            transform.translation.x = offset_x;
            transform.translation.y = offset_y;
        }
    } else {
        // Reset camera
        if let Ok(mut transform) = camera_query.get_single_mut() {
            transform.translation.x = 0.0;
            transform.translation.y = 0.0;
        }
    }
}

// =============================================================================
// ENGINE TRAILS
// =============================================================================

/// Component for entities that emit engine trails
#[derive(Component)]
pub struct EngineTrail {
    /// Trail color (faction-based)
    pub color: Color,
    /// Spawn rate (particles per second)
    pub spawn_rate: f32,
    /// Timer for spawning
    pub spawn_timer: f32,
    /// Offset from entity center (engine position)
    pub offset: Vec2,
    /// Whether trail is active (moving)
    pub active: bool,
}

impl Default for EngineTrail {
    fn default() -> Self {
        Self {
            color: Color::srgba(0.4, 0.7, 1.0, 0.9), // Blue engine glow
            spawn_rate: 60.0,
            spawn_timer: 0.0,
            offset: Vec2::new(0.0, -25.0), // Behind ship
            active: true,
        }
    }
}

impl EngineTrail {
    /// Minmatar rust-orange engine
    pub fn minmatar() -> Self {
        Self {
            color: Color::srgba(1.0, 0.5, 0.2, 0.9),
            ..default()
        }
    }

    /// Amarr golden engine
    pub fn amarr() -> Self {
        Self {
            color: Color::srgba(1.0, 0.85, 0.3, 0.9),
            ..default()
        }
    }

    /// Caldari blue engine
    pub fn caldari() -> Self {
        Self {
            color: Color::srgba(0.3, 0.6, 1.0, 0.9),
            ..default()
        }
    }

    /// Gallente green engine
    pub fn gallente() -> Self {
        Self {
            color: Color::srgba(0.3, 0.9, 0.5, 0.9),
            ..default()
        }
    }

    /// Create engine trail from faction
    pub fn from_faction(faction: crate::core::Faction) -> Self {
        Self {
            color: faction.engine_color(),
            ..default()
        }
    }
}

/// Engine trail particle
#[derive(Component)]
pub struct EngineParticle {
    pub velocity: Vec2,
    pub lifetime: f32,
    pub max_lifetime: f32,
}

/// Spawn engine trail particles from entities with EngineTrail
fn spawn_engine_trails(
    mut commands: Commands,
    time: Res<Time>,
    mut query: Query<(&Transform, &mut EngineTrail)>,
) {
    let dt = time.delta_secs();

    for (transform, mut trail) in query.iter_mut() {
        if !trail.active {
            continue;
        }

        trail.spawn_timer += dt;
        let spawn_interval = 1.0 / trail.spawn_rate;

        while trail.spawn_timer >= spawn_interval {
            trail.spawn_timer -= spawn_interval;

            // Calculate spawn position with offset
            let rotation = transform.rotation.to_euler(EulerRot::ZYX).0;
            let rotated_offset = Vec2::new(
                trail.offset.x * rotation.cos() - trail.offset.y * rotation.sin(),
                trail.offset.x * rotation.sin() + trail.offset.y * rotation.cos(),
            );
            let spawn_pos = transform.translation.truncate() + rotated_offset;

            // Random variation
            let spread = 8.0;
            let offset_x = (fastrand::f32() - 0.5) * spread;
            let offset_y = (fastrand::f32() - 0.5) * spread;

            // Velocity pointing backward/down with some spread
            let base_vel = Vec2::new(0.0, -80.0);
            let vel_spread = Vec2::new(
                (fastrand::f32() - 0.5) * 40.0,
                (fastrand::f32() - 0.5) * 30.0,
            );

            let lifetime = 0.15 + fastrand::f32() * 0.15;
            let size = 3.0 + fastrand::f32() * 4.0;

            // Spawn particle
            commands.spawn((
                EngineParticle {
                    velocity: base_vel + vel_spread,
                    lifetime,
                    max_lifetime: lifetime,
                },
                Sprite {
                    color: trail.color,
                    custom_size: Some(Vec2::splat(size)),
                    ..default()
                },
                Transform::from_xyz(
                    spawn_pos.x + offset_x,
                    spawn_pos.y + offset_y,
                    LAYER_EFFECTS - 1.0, // Behind ships
                ),
            ));
        }
    }
}

/// Update engine trail particles
fn update_engine_particles(
    mut commands: Commands,
    time: Res<Time>,
    mut query: Query<(Entity, &mut Transform, &mut EngineParticle, &mut Sprite)>,
) {
    let dt = time.delta_secs();

    for (entity, mut transform, mut particle, mut sprite) in query.iter_mut() {
        // Move
        transform.translation.x += particle.velocity.x * dt;
        transform.translation.y += particle.velocity.y * dt;

        // Slow down
        particle.velocity *= 1.0 - 5.0 * dt;

        // Update lifetime
        particle.lifetime -= dt;
        let progress = particle.lifetime / particle.max_lifetime;

        // Fade out
        let alpha = progress * 0.9;
        sprite.color = sprite.color.with_alpha(alpha);

        // Shrink
        if let Some(size) = sprite.custom_size {
            sprite.custom_size = Some(size * (1.0 - 2.0 * dt));
        }

        if particle.lifetime <= 0.0 {
            commands.entity(entity).despawn();
        }
    }
}

// =============================================================================
// CLEANUP
// =============================================================================

fn cleanup_effects(
    mut commands: Commands,
    stars: Query<Entity, With<Star>>,
    explosion_particles: Query<Entity, With<ExplosionParticle>>,
    engine_particles: Query<Entity, With<EngineParticle>>,
) {
    for entity in stars.iter() {
        commands.entity(entity).despawn();
    }
    for entity in explosion_particles.iter() {
        commands.entity(entity).despawn();
    }
    for entity in engine_particles.iter() {
        commands.entity(entity).despawn();
    }
}
