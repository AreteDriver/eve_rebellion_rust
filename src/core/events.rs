//! Custom Game Events
//!
//! Events for decoupled communication between systems.

use bevy::prelude::*;

/// Player took damage
#[derive(Event)]
pub struct PlayerDamagedEvent {
    pub damage: f32,
    pub damage_type: DamageType,
    pub source_position: Vec2,
}

/// Enemy was destroyed
#[derive(Event)]
pub struct EnemyDestroyedEvent {
    pub position: Vec2,
    pub enemy_type: String,
    pub score_value: u64,
    pub was_boss: bool,
}

/// Player fired weapon
#[derive(Event)]
pub struct PlayerFireEvent {
    pub position: Vec2,
    pub direction: Vec2,
    pub weapon_type: WeaponType,
}

/// Spawn enemy event
#[derive(Event)]
pub struct SpawnEnemyEvent {
    pub enemy_type: String,
    pub position: Vec2,
    pub spawn_pattern: SpawnPattern,
}

/// Spawn wave event
#[derive(Event)]
pub struct SpawnWaveEvent {
    pub wave_number: u32,
    pub enemy_count: u32,
    pub enemy_types: Vec<String>,
}

/// Stage completed
#[derive(Event)]
pub struct StageCompleteEvent {
    pub stage_number: u32,
    pub score: u64,
    pub time_taken: f32,
    pub refugees_rescued: u32,
}

/// Boss defeated
#[derive(Event)]
pub struct BossDefeatedEvent {
    pub boss_type: String,
    pub position: Vec2,
    pub score_value: u64,
}

/// Collectible picked up
#[derive(Event)]
pub struct CollectiblePickedUpEvent {
    pub collectible_type: CollectibleType,
    pub position: Vec2,
    pub value: u32,
}

/// Berserk mode activated
#[derive(Event)]
pub struct BerserkActivatedEvent;

/// Berserk mode ended
#[derive(Event)]
pub struct BerserkEndedEvent;

/// Screen shake request
#[derive(Event)]
pub struct ScreenShakeEvent {
    pub intensity: f32,
    pub duration: f32,
}

/// Explosion effect
#[derive(Event)]
pub struct ExplosionEvent {
    pub position: Vec2,
    pub size: ExplosionSize,
    pub color: Color,
}

/// Play sound effect
#[derive(Event)]
pub struct PlaySoundEvent {
    pub sound: SoundType,
    pub volume: f32,
}

// =============================================================================
// SUPPORTING TYPES
// =============================================================================

/// Damage types (EVE Online style)
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DamageType {
    EM,        // Lasers, smartbombs
    Thermal,   // Lasers, some missiles
    Kinetic,   // Projectiles, railguns
    Explosive, // Missiles, artillery
}

/// Weapon types
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum WeaponType {
    Autocannon,
    Artillery,
    Laser,
    Railgun,
    MissileLauncher,
    Drone,
}

/// Enemy spawn patterns
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SpawnPattern {
    Single,
    Line,
    VFormation,
    Circle,
    Random,
    Swarm,
}

/// Collectible types
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum CollectibleType {
    LiberationPod, // Freed slave - collect to liberate
    Credits,
    ShieldBoost,
    ArmorRepair,
    HullRepair,
    CapacitorCharge,
    Overdrive,       // Temporary speed boost
    DamageBoost,     // Temporary damage boost
    Invulnerability, // Temporary invincibility
    Nanite,          // Reduces weapon heat
    ExtraLife,
}

/// Explosion sizes for visual effects
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ExplosionSize {
    Tiny,    // Bullet impact
    Small,   // Frigate explosion
    Medium,  // Cruiser explosion
    Large,   // Battleship explosion
    Massive, // Boss explosion
}

/// Sound effect types
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SoundType {
    // Weapons
    Autocannon,
    Artillery,
    Laser,
    Missile,

    // Impacts
    ShieldHit,
    ArmorHit,
    HullHit,

    // Explosions
    SmallExplosion,
    MediumExplosion,
    LargeExplosion,

    // UI
    MenuSelect,
    MenuConfirm,
    MenuBack,

    // Gameplay
    PowerUp,
    Liberation, // Soul liberated
    BerserkActivate,
    Warning,
    Victory,
    GameOver,
}

/// Plugin to register all events
pub struct GameEventsPlugin;

impl Plugin for GameEventsPlugin {
    fn build(&self, app: &mut App) {
        app.add_event::<PlayerDamagedEvent>()
            .add_event::<EnemyDestroyedEvent>()
            .add_event::<PlayerFireEvent>()
            .add_event::<SpawnEnemyEvent>()
            .add_event::<SpawnWaveEvent>()
            .add_event::<StageCompleteEvent>()
            .add_event::<BossDefeatedEvent>()
            .add_event::<CollectiblePickedUpEvent>()
            .add_event::<BerserkActivatedEvent>()
            .add_event::<BerserkEndedEvent>()
            .add_event::<ScreenShakeEvent>()
            .add_event::<ExplosionEvent>()
            .add_event::<PlaySoundEvent>();
    }
}
