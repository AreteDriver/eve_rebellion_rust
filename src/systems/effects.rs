//! Visual Effects System
//!
//! Starfield, explosions, particle effects, screen shake, engine trails.

#![allow(dead_code)]

use crate::core::*;
use bevy::prelude::*;
use bevy::text::{Text2d, TextColor, TextFont};

/// Maximum particles to prevent slowdown during intense combat
const MAX_EXPLOSION_PARTICLES: usize = 500;
const MAX_ENGINE_PARTICLES: usize = 200;

/// Effects plugin
pub struct EffectsPlugin;

impl Plugin for EffectsPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<ScreenShake>()
            .init_resource::<ScreenFlash>()
            .init_resource::<CameraZoom>()
            .add_systems(OnEnter(GameState::Playing), spawn_starfield)
            .add_systems(
                Update,
                (
                    update_starfield,
                    update_explosions,
                    update_screen_shake,
                    update_screen_flash,
                    update_berserk_tint,
                    update_camera_zoom,
                    handle_explosion_events,
                    spawn_engine_trails,
                    update_engine_particles,
                    spawn_bullet_trails,
                    update_bullet_trails,
                    update_hit_flash,
                    update_damage_numbers,
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
    mut rumble_events: EventWriter<super::RumbleRequest>,
) {
    let current_count = particle_query.iter().count();
    let mut spawned = 0;

    for event in events.read() {
        // Check particle cap before spawning
        if current_count + spawned < MAX_EXPLOSION_PARTICLES {
            let new_count =
                spawn_explosion_capped(&mut commands, event.position, &event.size, event.color);
            spawned += new_count;

            // Trigger rumble based on explosion size (only for large+ explosions to avoid spam)
            match event.size {
                ExplosionSize::Large => {
                    rumble_events.send(super::RumbleRequest::explosion());
                }
                ExplosionSize::Massive => {
                    rumble_events.send(super::RumbleRequest::big_explosion());
                }
                _ => {} // No rumble for tiny/small/medium (too spammy)
            }
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
// DAMAGE NUMBERS
// =============================================================================

/// Floating damage number that rises and fades
#[derive(Component)]
pub struct DamageNumber {
    /// Upward velocity
    pub velocity: Vec2,
    /// Time remaining
    pub lifetime: f32,
    /// Max lifetime for fade calculation
    pub max_lifetime: f32,
}

impl DamageNumber {
    pub fn new() -> Self {
        Self {
            velocity: Vec2::new(
                (fastrand::f32() - 0.5) * 30.0, // Random horizontal drift
                80.0,                            // Rise upward
            ),
            lifetime: 0.8,
            max_lifetime: 0.8,
        }
    }
}

impl Default for DamageNumber {
    fn default() -> Self {
        Self::new()
    }
}

/// Spawn a floating damage number at position
pub fn spawn_damage_number(commands: &mut Commands, position: Vec2, damage: f32, is_crit: bool) {
    let text = format!("{:.0}", damage);
    let (color, size) = if is_crit {
        (Color::srgb(1.0, 0.9, 0.2), 18.0) // Yellow, larger for crits
    } else if damage >= 20.0 {
        (Color::srgb(1.0, 0.5, 0.2), 16.0) // Orange for heavy hits
    } else {
        (Color::srgb(1.0, 1.0, 1.0), 14.0) // White for normal
    };

    commands.spawn((
        DamageNumber::new(),
        Text2d::new(text),
        TextFont {
            font_size: size,
            ..default()
        },
        TextColor(color),
        Transform::from_xyz(position.x, position.y + 20.0, LAYER_EFFECTS + 5.0),
    ));
}

/// Update damage number positions and fade
fn update_damage_numbers(
    mut commands: Commands,
    time: Res<Time>,
    mut query: Query<(Entity, &mut Transform, &mut DamageNumber, &mut TextColor)>,
) {
    let dt = time.delta_secs();

    for (entity, mut transform, mut dmg, mut color) in query.iter_mut() {
        // Move upward
        transform.translation.x += dmg.velocity.x * dt;
        transform.translation.y += dmg.velocity.y * dt;

        // Slow down horizontal drift
        dmg.velocity.x *= 1.0 - 3.0 * dt;

        // Update lifetime
        dmg.lifetime -= dt;
        let alpha = (dmg.lifetime / dmg.max_lifetime).max(0.0);

        // Fade out
        color.0 = color.0.with_alpha(alpha);

        // Scale up slightly as it rises
        let scale = 1.0 + (1.0 - alpha) * 0.3;
        transform.scale = Vec3::splat(scale);

        if dmg.lifetime <= 0.0 {
            commands.entity(entity).despawn();
        }
    }
}

// =============================================================================
// HIT FLASH
// =============================================================================

/// Component that makes a sprite flash white when damaged
#[derive(Component)]
pub struct HitFlash {
    /// Time remaining for flash effect
    pub timer: f32,
    /// Total duration of flash
    pub duration: f32,
    /// Original sprite color (to restore after flash)
    pub original_color: Color,
}

impl HitFlash {
    /// Create a new hit flash effect
    pub fn new(original_color: Color) -> Self {
        Self {
            timer: 0.1,
            duration: 0.1,
            original_color,
        }
    }

    /// Create a hit flash with custom duration
    pub fn with_duration(original_color: Color, duration: f32) -> Self {
        Self {
            timer: duration,
            duration,
            original_color,
        }
    }
}

/// Update hit flash effects on sprites
fn update_hit_flash(
    mut commands: Commands,
    time: Res<Time>,
    mut query: Query<(Entity, &mut Sprite, &mut HitFlash)>,
) {
    let dt = time.delta_secs();

    for (entity, mut sprite, mut flash) in query.iter_mut() {
        flash.timer -= dt;

        if flash.timer > 0.0 {
            // Lerp from white to original color
            let progress = 1.0 - (flash.timer / flash.duration);
            let white = Color::WHITE;
            let original = flash.original_color;

            // Simple lerp between white and original
            let r = white.to_srgba().red * (1.0 - progress)
                + original.to_srgba().red * progress;
            let g = white.to_srgba().green * (1.0 - progress)
                + original.to_srgba().green * progress;
            let b = white.to_srgba().blue * (1.0 - progress)
                + original.to_srgba().blue * progress;
            let a = original.to_srgba().alpha;

            sprite.color = Color::srgba(r, g, b, a);
        } else {
            // Flash complete, restore original and remove component
            sprite.color = flash.original_color;
            commands.entity(entity).remove::<HitFlash>();
        }
    }
}

// =============================================================================
// SCREEN FLASH
// =============================================================================

/// Screen-wide flash effect for big explosions
#[derive(Resource, Default)]
pub struct ScreenFlash {
    /// Current flash intensity (0.0 - 1.0)
    pub intensity: f32,
    /// Flash color
    pub color: Color,
    /// Fade speed
    pub fade_speed: f32,
}

impl ScreenFlash {
    /// Trigger a white screen flash
    pub fn white(&mut self, intensity: f32) {
        self.intensity = intensity.min(1.0);
        self.color = Color::WHITE;
        self.fade_speed = 4.0;
    }

    /// Trigger a colored screen flash
    pub fn colored(&mut self, color: Color, intensity: f32) {
        self.intensity = intensity.min(1.0);
        self.color = color;
        self.fade_speed = 4.0;
    }

    /// Trigger flash for massive explosion (boss kill)
    pub fn massive(&mut self) {
        self.white(0.8);
        self.fade_speed = 2.0; // Slower fade for dramatic effect
    }

    /// Trigger flash for large explosion
    pub fn large(&mut self) {
        self.white(0.5);
    }

    /// Trigger red flash for berserk activation
    pub fn berserk(&mut self) {
        self.colored(Color::srgb(1.0, 0.2, 0.2), 0.6);
        self.fade_speed = 3.0;
    }
}

/// Marker component for screen flash overlay sprite
#[derive(Component)]
pub struct ScreenFlashOverlay;

/// Update screen flash effect
fn update_screen_flash(
    mut commands: Commands,
    time: Res<Time>,
    mut flash: ResMut<ScreenFlash>,
    mut overlay_query: Query<(Entity, &mut Sprite), With<ScreenFlashOverlay>>,
) {
    let dt = time.delta_secs();

    if flash.intensity > 0.0 {
        // Fade out
        flash.intensity = (flash.intensity - flash.fade_speed * dt).max(0.0);

        // Update or create overlay
        if let Ok((_, mut sprite)) = overlay_query.get_single_mut() {
            sprite.color = flash.color.with_alpha(flash.intensity);
        } else if flash.intensity > 0.01 {
            // Spawn overlay sprite covering screen
            commands.spawn((
                ScreenFlashOverlay,
                Sprite {
                    color: flash.color.with_alpha(flash.intensity),
                    custom_size: Some(Vec2::new(SCREEN_WIDTH + 100.0, SCREEN_HEIGHT + 100.0)),
                    ..default()
                },
                Transform::from_xyz(0.0, 0.0, LAYER_HUD + 10.0), // Above everything
            ));
        }
    } else {
        // Remove overlay when done
        for (entity, _) in overlay_query.iter() {
            commands.entity(entity).despawn();
        }
    }
}

// =============================================================================
// BERSERK SCREEN TINT
// =============================================================================

/// Marker component for berserk tint overlay
#[derive(Component)]
pub struct BerserkTintOverlay;

/// Berserk screen tint effect - red tint while berserk is active
fn update_berserk_tint(
    mut commands: Commands,
    berserk: Res<BerserkSystem>,
    mut overlay_query: Query<(Entity, &mut Sprite), With<BerserkTintOverlay>>,
) {
    if berserk.is_active {
        // Pulse the tint based on remaining time
        let pulse = (berserk.timer * 8.0).sin().abs() * 0.1;
        let alpha = 0.15 + pulse;

        if let Ok((_, mut sprite)) = overlay_query.get_single_mut() {
            sprite.color = Color::srgba(1.0, 0.1, 0.1, alpha);
        } else {
            // Spawn tint overlay
            commands.spawn((
                BerserkTintOverlay,
                Sprite {
                    color: Color::srgba(1.0, 0.1, 0.1, alpha),
                    custom_size: Some(Vec2::new(SCREEN_WIDTH + 100.0, SCREEN_HEIGHT + 100.0)),
                    ..default()
                },
                Transform::from_xyz(0.0, 0.0, LAYER_HUD + 5.0), // Below flash, above game
            ));
        }
    } else {
        // Remove tint when berserk ends
        for (entity, _) in overlay_query.iter() {
            commands.entity(entity).despawn();
        }
    }
}

// =============================================================================
// CAMERA ZOOM PULSE
// =============================================================================

/// Camera zoom pulse for dramatic moments (boss kills)
#[derive(Resource)]
pub struct CameraZoom {
    /// Target scale (1.0 = normal, 1.1 = 10% zoom in)
    pub target_scale: f32,
    /// Current scale
    pub current_scale: f32,
    /// Return speed (how fast to return to normal)
    pub return_speed: f32,
}

impl Default for CameraZoom {
    fn default() -> Self {
        Self {
            target_scale: 1.0,
            current_scale: 1.0,
            return_speed: 3.0,
        }
    }
}

impl CameraZoom {
    /// Trigger a zoom pulse (zoom in then out)
    pub fn pulse(&mut self, intensity: f32) {
        self.target_scale = 1.0 + intensity;
        self.return_speed = 3.0;
    }

    /// Quick dramatic zoom for boss kills
    pub fn boss_kill(&mut self) {
        self.pulse(0.08); // 8% zoom in
        self.return_speed = 2.0; // Slower return for drama
    }

    /// Small zoom for regular kills
    pub fn small(&mut self) {
        self.pulse(0.02);
        self.return_speed = 5.0;
    }
}

/// Update camera zoom effect
fn update_camera_zoom(
    time: Res<Time>,
    mut zoom: ResMut<CameraZoom>,
    mut camera_query: Query<&mut OrthographicProjection, With<Camera2d>>,
) {
    let dt = time.delta_secs();

    // Move current scale toward target
    if zoom.current_scale != zoom.target_scale {
        let diff = zoom.target_scale - zoom.current_scale;
        zoom.current_scale += diff * 8.0 * dt; // Fast zoom in

        // Apply to camera
        if let Ok(mut projection) = camera_query.get_single_mut() {
            projection.scale = zoom.current_scale;
        }
    }

    // Return target to 1.0 over time
    if zoom.target_scale > 1.0 {
        zoom.target_scale = (zoom.target_scale - zoom.return_speed * dt).max(1.0);
    }

    // Snap to 1.0 when close
    if (zoom.current_scale - 1.0).abs() < 0.001 && zoom.target_scale == 1.0 {
        zoom.current_scale = 1.0;
        if let Ok(mut projection) = camera_query.get_single_mut() {
            projection.scale = 1.0;
        }
    }
}

// =============================================================================
// BULLET TRAILS
// =============================================================================

/// Component for projectiles that emit trails
#[derive(Component)]
pub struct BulletTrail {
    /// Trail color
    pub color: Color,
    /// Spawn rate (particles per second)
    pub spawn_rate: f32,
    /// Timer for spawning
    pub spawn_timer: f32,
}

impl BulletTrail {
    pub fn new(color: Color) -> Self {
        Self {
            color,
            spawn_rate: 40.0,
            spawn_timer: 0.0,
        }
    }
}

/// Bullet trail particle
#[derive(Component)]
pub struct BulletTrailParticle {
    pub lifetime: f32,
    pub max_lifetime: f32,
}

/// Spawn bullet trail particles from projectiles
fn spawn_bullet_trails(
    mut commands: Commands,
    time: Res<Time>,
    mut query: Query<(&Transform, &mut BulletTrail)>,
    particle_count: Query<&BulletTrailParticle>,
) {
    // Cap trail particles to avoid performance issues
    const MAX_TRAIL_PARTICLES: usize = 300;
    if particle_count.iter().count() >= MAX_TRAIL_PARTICLES {
        return;
    }

    let dt = time.delta_secs();

    for (transform, mut trail) in query.iter_mut() {
        trail.spawn_timer += dt;
        let spawn_interval = 1.0 / trail.spawn_rate;

        while trail.spawn_timer >= spawn_interval {
            trail.spawn_timer -= spawn_interval;

            let pos = transform.translation.truncate();
            let lifetime = 0.15;

            // Spawn fading particle
            commands.spawn((
                BulletTrailParticle {
                    lifetime,
                    max_lifetime: lifetime,
                },
                Sprite {
                    color: trail.color.with_alpha(0.6),
                    custom_size: Some(Vec2::new(3.0, 3.0)),
                    ..default()
                },
                Transform::from_xyz(pos.x, pos.y, LAYER_EFFECTS - 2.0),
            ));
        }
    }
}

/// Update bullet trail particles (fade and despawn)
fn update_bullet_trails(
    mut commands: Commands,
    time: Res<Time>,
    mut query: Query<(Entity, &mut BulletTrailParticle, &mut Sprite)>,
) {
    let dt = time.delta_secs();

    for (entity, mut particle, mut sprite) in query.iter_mut() {
        particle.lifetime -= dt;

        if particle.lifetime <= 0.0 {
            commands.entity(entity).despawn();
        } else {
            // Fade out and shrink
            let alpha = (particle.lifetime / particle.max_lifetime) * 0.6;
            sprite.color = sprite.color.with_alpha(alpha);

            if let Some(size) = sprite.custom_size {
                sprite.custom_size = Some(size * (1.0 - dt * 4.0));
            }
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
    flash_overlays: Query<Entity, With<ScreenFlashOverlay>>,
    damage_numbers: Query<Entity, With<DamageNumber>>,
    bullet_trail_particles: Query<Entity, With<BulletTrailParticle>>,
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
    for entity in flash_overlays.iter() {
        commands.entity(entity).despawn();
    }
    for entity in damage_numbers.iter() {
        commands.entity(entity).despawn();
    }
    for entity in bullet_trail_particles.iter() {
        commands.entity(entity).despawn();
    }
}
