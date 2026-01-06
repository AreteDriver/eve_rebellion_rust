//! Audio System
//!
//! Procedural sound effects for EVE Rebellion.
//! Uses hound crate for proper WAV generation.

#![allow(dead_code)]

use bevy::audio::{PlaybackMode, Volume};
use bevy::prelude::*;
use std::f32::consts::PI;
#[cfg(not(target_arch = "wasm32"))]
use std::io::Cursor;

use crate::core::{BossSpawnEvent, WaveCompleteEvent, *};
use crate::systems::ability::{AbilityActivatedEvent, AbilityType};

/// Audio plugin
pub struct AudioPlugin;

impl Plugin for AudioPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<SoundSettings>()
            .init_resource::<SoundAssets>()
            .init_resource::<WarningState>()
            .add_systems(Startup, generate_sounds)
            .add_systems(
                Update,
                (
                    play_weapon_sounds,
                    play_explosion_sounds,
                    play_pickup_sounds,
                    play_damage_sounds,
                    play_health_warnings,
                    play_wave_complete_sound,
                    play_boss_spawn_sound,
                    play_ability_sounds,
                )
                    .run_if(in_state(GameState::Playing)),
            );
    }
}

/// Sound settings
#[derive(Resource)]
pub struct SoundSettings {
    pub master_volume: f32,
    pub sfx_volume: f32,
    pub music_volume: f32,
    pub enabled: bool,
}

impl Default for SoundSettings {
    fn default() -> Self {
        Self {
            master_volume: 0.7,
            sfx_volume: 0.8,
            music_volume: 0.5,
            enabled: true,
        }
    }
}

/// Pre-generated sound assets
#[derive(Resource, Default)]
pub struct SoundAssets {
    pub autocannon: Option<Handle<AudioSource>>,
    pub laser: Option<Handle<AudioSource>>,
    pub missile: Option<Handle<AudioSource>>,
    pub explosion_small: Option<Handle<AudioSource>>,
    pub explosion_medium: Option<Handle<AudioSource>>,
    pub explosion_large: Option<Handle<AudioSource>>,
    pub pickup: Option<Handle<AudioSource>>,
    pub shield_hit: Option<Handle<AudioSource>>,
    pub armor_hit: Option<Handle<AudioSource>>,
    pub hull_hit: Option<Handle<AudioSource>>,
    // EVE-style warning alarms
    pub shield_warning: Option<Handle<AudioSource>>,
    pub armor_warning: Option<Handle<AudioSource>>,
    pub hull_warning: Option<Handle<AudioSource>>,
    // Game events
    pub wave_complete: Option<Handle<AudioSource>>,
    pub boss_spawn: Option<Handle<AudioSource>>,
    // Powerup-specific sounds
    pub powerup_overdrive: Option<Handle<AudioSource>>,
    pub powerup_damage: Option<Handle<AudioSource>>,
    pub powerup_invuln: Option<Handle<AudioSource>>,
    pub powerup_health: Option<Handle<AudioSource>>,
    // Menu sounds
    pub menu_select: Option<Handle<AudioSource>>,
    pub menu_confirm: Option<Handle<AudioSource>>,
    // Ability sounds
    pub ability_speed: Option<Handle<AudioSource>>,      // Overdrive, Afterburner
    pub ability_shield: Option<Handle<AudioSource>>,     // Shield Boost
    pub ability_armor: Option<Handle<AudioSource>>,      // Armor Hardener, Armor Repair
    pub ability_weapon: Option<Handle<AudioSource>>,     // Salvo, Rocket Barrage, Scorch
    pub ability_drone: Option<Handle<AudioSource>>,      // Deploy Drone, Drone Bay
    pub ability_debuff: Option<Handle<AudioSource>>,     // Warp Disruptor
    pub ability_damage: Option<Handle<AudioSource>>,     // Close Range
}

/// Tracks when warnings should play (to avoid spamming)
#[derive(Resource)]
pub struct WarningState {
    pub shield_warned: bool,
    pub armor_warned: bool,
    pub hull_warned: bool,
    pub warning_cooldown: f32,
}

impl Default for WarningState {
    fn default() -> Self {
        Self {
            shield_warned: false,
            armor_warned: false,
            hull_warned: false,
            warning_cooldown: 0.0,
        }
    }
}

/// Generate procedural sound effects at startup
fn generate_sounds(
    mut sounds: ResMut<SoundAssets>,
    mut audio_sources: ResMut<Assets<AudioSource>>,
) {
    info!("Generating procedural sound effects...");

    // Autocannon - chunky industrial sound
    if let Some(source) = generate_autocannon() {
        sounds.autocannon = Some(audio_sources.add(source));
    }

    // Laser - high-pitched beam
    if let Some(source) = generate_laser() {
        sounds.laser = Some(audio_sources.add(source));
    }

    // Missile launch - whoosh
    if let Some(source) = generate_missile() {
        sounds.missile = Some(audio_sources.add(source));
    }

    // Explosions - various sizes
    if let Some(source) = generate_explosion(0.15, 300.0) {
        sounds.explosion_small = Some(audio_sources.add(source));
    }
    if let Some(source) = generate_explosion(0.25, 200.0) {
        sounds.explosion_medium = Some(audio_sources.add(source));
    }
    if let Some(source) = generate_explosion(0.4, 120.0) {
        sounds.explosion_large = Some(audio_sources.add(source));
    }

    // Pickup - cheerful blip
    if let Some(source) = generate_pickup() {
        sounds.pickup = Some(audio_sources.add(source));
    }

    // Damage sounds
    if let Some(source) = generate_shield_hit() {
        sounds.shield_hit = Some(audio_sources.add(source));
    }
    if let Some(source) = generate_armor_hit() {
        sounds.armor_hit = Some(audio_sources.add(source));
    }
    if let Some(source) = generate_hull_hit() {
        sounds.hull_hit = Some(audio_sources.add(source));
    }

    // EVE-style warning alarms (when health drops below 20%)
    if let Some(source) = generate_shield_warning() {
        sounds.shield_warning = Some(audio_sources.add(source));
    }
    if let Some(source) = generate_armor_warning() {
        sounds.armor_warning = Some(audio_sources.add(source));
    }
    if let Some(source) = generate_hull_warning() {
        sounds.hull_warning = Some(audio_sources.add(source));
    }

    // Game event sounds
    if let Some(source) = generate_wave_complete() {
        sounds.wave_complete = Some(audio_sources.add(source));
    }
    if let Some(source) = generate_boss_spawn() {
        sounds.boss_spawn = Some(audio_sources.add(source));
    }

    // Powerup-specific sounds
    if let Some(source) = generate_powerup_overdrive() {
        sounds.powerup_overdrive = Some(audio_sources.add(source));
    }
    if let Some(source) = generate_powerup_damage() {
        sounds.powerup_damage = Some(audio_sources.add(source));
    }
    if let Some(source) = generate_powerup_invuln() {
        sounds.powerup_invuln = Some(audio_sources.add(source));
    }
    if let Some(source) = generate_powerup_health() {
        sounds.powerup_health = Some(audio_sources.add(source));
    }

    // Menu sounds
    if let Some(source) = generate_menu_select() {
        sounds.menu_select = Some(audio_sources.add(source));
    }
    if let Some(source) = generate_menu_confirm() {
        sounds.menu_confirm = Some(audio_sources.add(source));
    }

    // Ability sounds
    if let Some(source) = generate_ability_speed() {
        sounds.ability_speed = Some(audio_sources.add(source));
    }
    if let Some(source) = generate_ability_shield() {
        sounds.ability_shield = Some(audio_sources.add(source));
    }
    if let Some(source) = generate_ability_armor() {
        sounds.ability_armor = Some(audio_sources.add(source));
    }
    if let Some(source) = generate_ability_weapon() {
        sounds.ability_weapon = Some(audio_sources.add(source));
    }
    if let Some(source) = generate_ability_drone() {
        sounds.ability_drone = Some(audio_sources.add(source));
    }
    if let Some(source) = generate_ability_debuff() {
        sounds.ability_debuff = Some(audio_sources.add(source));
    }
    if let Some(source) = generate_ability_damage() {
        sounds.ability_damage = Some(audio_sources.add(source));
    }

    info!("Sound effects generated!");
}

/// Generate autocannon sound - deep industrial thump
fn generate_autocannon() -> Option<AudioSource> {
    let sample_rate = 44100u32;
    let duration = 0.12;
    let num_samples = (sample_rate as f32 * duration) as usize;
    let mut samples = Vec::with_capacity(num_samples);

    for i in 0..num_samples {
        let t = i as f32 / sample_rate as f32;

        // Deep bass thump
        let bass = (2.0 * PI * 80.0 * t).sin() * 0.5;

        // Mid punch
        let mid = (2.0 * PI * 200.0 * t).sin() * (-t * 50.0).exp() * 0.4;

        // High crack
        let crack = (2.0 * PI * 600.0 * t).sin() * (-t * 80.0).exp() * 0.3;

        // Noise burst
        let noise = (fastrand::f32() * 2.0 - 1.0) * (-t * 40.0).exp() * 0.2;

        // Envelope
        let env = (-t * 15.0).exp();

        let sample = ((bass + mid + crack + noise) * env * 0.8).clamp(-1.0, 1.0);
        samples.push(sample);
    }

    create_audio_source(&samples, sample_rate)
}

/// Generate laser sound - high-pitched zap
fn generate_laser() -> Option<AudioSource> {
    let sample_rate = 44100u32;
    let duration = 0.15;
    let num_samples = (sample_rate as f32 * duration) as usize;
    let mut samples = Vec::with_capacity(num_samples);

    for i in 0..num_samples {
        let t = i as f32 / sample_rate as f32;

        // Descending frequency
        let freq = 1200.0 - t * 3000.0;
        let wave = (2.0 * PI * freq * t).sin();

        // Add harmonics
        let harm = (2.0 * PI * freq * 2.0 * t).sin() * 0.3;

        // Envelope
        let env = (-t * 20.0).exp();

        let sample = ((wave + harm) * env * 0.6).clamp(-1.0, 1.0);
        samples.push(sample);
    }

    create_audio_source(&samples, sample_rate)
}

/// Generate explosion sound
fn generate_explosion(duration: f32, base_freq: f32) -> Option<AudioSource> {
    let sample_rate = 44100u32;
    let num_samples = (sample_rate as f32 * duration) as usize;
    let mut samples = Vec::with_capacity(num_samples);

    for i in 0..num_samples {
        let t = i as f32 / sample_rate as f32;

        // Low rumble
        let rumble = (2.0 * PI * base_freq * t).sin() * 0.5;

        // Noise
        let noise = (fastrand::f32() * 2.0 - 1.0) * 0.6;

        // Crackle (filtered noise bursts)
        let crackle = if fastrand::f32() < 0.1 {
            fastrand::f32() * 2.0 - 1.0
        } else {
            0.0
        } * (-t * 5.0).exp()
            * 0.3;

        // Envelope - quick attack, slow decay
        let env = (1.0 - (-t * 30.0).exp()) * (-t * 4.0).exp();

        let sample = ((rumble + noise + crackle) * env * 0.7).clamp(-1.0, 1.0);
        samples.push(sample);
    }

    create_audio_source(&samples, sample_rate)
}

/// Generate pickup sound - happy blip
fn generate_pickup() -> Option<AudioSource> {
    let sample_rate = 44100u32;
    let duration = 0.1;
    let num_samples = (sample_rate as f32 * duration) as usize;
    let mut samples = Vec::with_capacity(num_samples);

    for i in 0..num_samples {
        let t = i as f32 / sample_rate as f32;

        // Rising frequency
        let freq = 400.0 + t * 2000.0;
        let wave = (2.0 * PI * freq * t).sin();

        // Envelope
        let env = (1.0 - t / duration) * (1.0 - (-t * 50.0).exp());

        let sample = (wave * env * 0.5).clamp(-1.0, 1.0);
        samples.push(sample);
    }

    create_audio_source(&samples, sample_rate)
}

/// Generate shield hit sound - electric crackle
fn generate_shield_hit() -> Option<AudioSource> {
    let sample_rate = 44100u32;
    let duration = 0.08;
    let num_samples = (sample_rate as f32 * duration) as usize;
    let mut samples = Vec::with_capacity(num_samples);

    for i in 0..num_samples {
        let t = i as f32 / sample_rate as f32;

        // High frequency buzz
        let buzz = (2.0 * PI * 800.0 * t).sin() * 0.4;

        // Electric crackle
        let crackle = (fastrand::f32() * 2.0 - 1.0) * 0.5;

        let env = (-t * 30.0).exp();

        let sample = ((buzz + crackle) * env * 0.5).clamp(-1.0, 1.0);
        samples.push(sample);
    }

    create_audio_source(&samples, sample_rate)
}

/// Generate armor hit sound - metallic clang
fn generate_armor_hit() -> Option<AudioSource> {
    let sample_rate = 44100u32;
    let duration = 0.1;
    let num_samples = (sample_rate as f32 * duration) as usize;
    let mut samples = Vec::with_capacity(num_samples);

    for i in 0..num_samples {
        let t = i as f32 / sample_rate as f32;

        // Metallic frequencies
        let f1 = (2.0 * PI * 300.0 * t).sin() * 0.5;
        let f2 = (2.0 * PI * 450.0 * t).sin() * 0.3;
        let f3 = (2.0 * PI * 180.0 * t).sin() * 0.4;

        let env = (-t * 25.0).exp();

        let sample = ((f1 + f2 + f3) * env * 0.6).clamp(-1.0, 1.0);
        samples.push(sample);
    }

    create_audio_source(&samples, sample_rate)
}

/// Generate hull hit sound - deep crunch
fn generate_hull_hit() -> Option<AudioSource> {
    let sample_rate = 44100u32;
    let duration = 0.12;
    let num_samples = (sample_rate as f32 * duration) as usize;
    let mut samples = Vec::with_capacity(num_samples);

    for i in 0..num_samples {
        let t = i as f32 / sample_rate as f32;

        // Low crunch
        let crunch = (2.0 * PI * 100.0 * t).sin() * 0.6;

        // Noise
        let noise = (fastrand::f32() * 2.0 - 1.0) * 0.4;

        let env = (-t * 20.0).exp();

        let sample = ((crunch + noise) * env * 0.6).clamp(-1.0, 1.0);
        samples.push(sample);
    }

    create_audio_source(&samples, sample_rate)
}

/// Create AudioSource from f32 samples using hound for proper WAV encoding
#[cfg(not(target_arch = "wasm32"))]
fn create_audio_source(samples: &[f32], sample_rate: u32) -> Option<AudioSource> {
    use std::sync::Arc;

    let spec = hound::WavSpec {
        channels: 1,
        sample_rate,
        bits_per_sample: 16,
        sample_format: hound::SampleFormat::Int,
    };

    // Write to in-memory buffer
    let mut buffer = Cursor::new(Vec::new());

    {
        let mut writer = match hound::WavWriter::new(&mut buffer, spec) {
            Ok(w) => w,
            Err(e) => {
                warn!("Failed to create WAV writer: {}", e);
                return None;
            }
        };

        // Convert f32 samples to i16
        for &sample in samples {
            let s = (sample * 32767.0) as i16;
            if writer.write_sample(s).is_err() {
                warn!("Failed to write audio sample");
                return None;
            }
        }

        if writer.finalize().is_err() {
            warn!("Failed to finalize WAV");
            return None;
        }
    }

    let wav_data = buffer.into_inner();
    Some(AudioSource {
        bytes: Arc::from(wav_data.into_boxed_slice()),
    })
}

/// WASM stub - no procedural audio generation
#[cfg(target_arch = "wasm32")]
fn create_audio_source(_samples: &[f32], _sample_rate: u32) -> Option<AudioSource> {
    None
}

/// Play weapon firing sounds
fn play_weapon_sounds(
    mut commands: Commands,
    mut fire_events: EventReader<PlayerFireEvent>,
    sounds: Res<SoundAssets>,
    settings: Res<SoundSettings>,
) {
    if !settings.enabled {
        fire_events.clear();
        return;
    }

    for event in fire_events.read() {
        let sound = match event.weapon_type {
            WeaponType::Autocannon | WeaponType::Artillery => sounds.autocannon.clone(),
            WeaponType::Laser | WeaponType::Railgun => sounds.laser.clone(),
            WeaponType::MissileLauncher => sounds.missile.clone(),
            WeaponType::Drone => sounds.laser.clone(), // Drones use laser-like sound
        };

        if let Some(source) = sound {
            commands.spawn((
                AudioPlayer(source),
                PlaybackSettings {
                    mode: PlaybackMode::Despawn,
                    volume: Volume::new(settings.sfx_volume * settings.master_volume * 0.5),
                    ..default()
                },
            ));
        }
    }
}

/// Play explosion sounds on enemy destruction
fn play_explosion_sounds(
    mut commands: Commands,
    mut destroy_events: EventReader<EnemyDestroyedEvent>,
    sounds: Res<SoundAssets>,
    settings: Res<SoundSettings>,
) {
    if !settings.enabled {
        destroy_events.clear();
        return;
    }

    for event in destroy_events.read() {
        let sound = if event.was_boss {
            sounds.explosion_large.clone()
        } else {
            sounds.explosion_small.clone()
        };

        if let Some(source) = sound {
            commands.spawn((
                AudioPlayer(source),
                PlaybackSettings {
                    mode: PlaybackMode::Despawn,
                    volume: Volume::new(settings.sfx_volume * settings.master_volume * 0.6),
                    ..default()
                },
            ));
        }
    }
}

/// Play pickup sounds with different sounds for different powerup types
fn play_pickup_sounds(
    mut commands: Commands,
    mut pickup_events: EventReader<CollectiblePickedUpEvent>,
    sounds: Res<SoundAssets>,
    settings: Res<SoundSettings>,
) {
    if !settings.enabled {
        pickup_events.clear();
        return;
    }

    for event in pickup_events.read() {
        // Choose sound based on collectible type
        let sound = match event.collectible_type {
            CollectibleType::Overdrive => sounds.powerup_overdrive.clone(),
            CollectibleType::DamageBoost => sounds.powerup_damage.clone(),
            CollectibleType::Invulnerability => sounds.powerup_invuln.clone(),
            CollectibleType::ShieldBoost
            | CollectibleType::ArmorRepair
            | CollectibleType::HullRepair => sounds.powerup_health.clone(),
            _ => sounds.pickup.clone(), // Credits, souls, etc use generic pickup
        };

        if let Some(source) = sound.or(sounds.pickup.clone()) {
            commands.spawn((
                AudioPlayer(source),
                PlaybackSettings {
                    mode: PlaybackMode::Despawn,
                    volume: Volume::new(settings.sfx_volume * settings.master_volume * 0.7),
                    ..default()
                },
            ));
        }
    }
}

/// Play damage sounds when player is hit
fn play_damage_sounds(
    mut commands: Commands,
    mut damage_events: EventReader<PlayerDamagedEvent>,
    sounds: Res<SoundAssets>,
    settings: Res<SoundSettings>,
) {
    if !settings.enabled {
        damage_events.clear();
        return;
    }

    for event in damage_events.read() {
        let sound = match event.damage_type {
            DamageType::EM => sounds.shield_hit.clone(),
            DamageType::Thermal | DamageType::Kinetic => sounds.armor_hit.clone(),
            DamageType::Explosive => sounds.hull_hit.clone(),
        };

        if let Some(source) = sound {
            commands.spawn((
                AudioPlayer(source),
                PlaybackSettings {
                    mode: PlaybackMode::Despawn,
                    volume: Volume::new(settings.sfx_volume * settings.master_volume * 0.8),
                    ..default()
                },
            ));
        }
    }
}

/// Play EVE-style warning sounds when health drops below 20%
fn play_health_warnings(
    mut commands: Commands,
    player_query: Query<&crate::entities::ShipStats, With<crate::entities::Player>>,
    sounds: Res<SoundAssets>,
    settings: Res<SoundSettings>,
    mut warning_state: ResMut<WarningState>,
    time: Res<Time>,
) {
    if !settings.enabled {
        return;
    }

    // Cooldown between warnings
    warning_state.warning_cooldown -= time.delta_secs();

    let Ok(stats) = player_query.get_single() else {
        return;
    };

    let shield_pct = stats.shield / stats.max_shield;
    let armor_pct = stats.armor / stats.max_armor;
    let hull_pct = stats.hull / stats.max_hull;

    const WARNING_THRESHOLD: f32 = 0.20;

    // Shield warning
    if shield_pct <= WARNING_THRESHOLD && shield_pct > 0.0 {
        if !warning_state.shield_warned && warning_state.warning_cooldown <= 0.0 {
            if let Some(source) = sounds.shield_warning.clone() {
                commands.spawn((
                    AudioPlayer(source),
                    PlaybackSettings {
                        mode: PlaybackMode::Despawn,
                        volume: Volume::new(settings.sfx_volume * settings.master_volume * 0.9),
                        ..default()
                    },
                ));
                warning_state.shield_warned = true;
                warning_state.warning_cooldown = 3.0; // 3 second cooldown between warnings
            }
        }
    } else if shield_pct > WARNING_THRESHOLD {
        warning_state.shield_warned = false;
    }

    // Armor warning (more urgent)
    if armor_pct <= WARNING_THRESHOLD && armor_pct > 0.0 {
        if !warning_state.armor_warned && warning_state.warning_cooldown <= 0.0 {
            if let Some(source) = sounds.armor_warning.clone() {
                commands.spawn((
                    AudioPlayer(source),
                    PlaybackSettings {
                        mode: PlaybackMode::Despawn,
                        volume: Volume::new(settings.sfx_volume * settings.master_volume * 0.95),
                        ..default()
                    },
                ));
                warning_state.armor_warned = true;
                warning_state.warning_cooldown = 2.5;
            }
        }
    } else if armor_pct > WARNING_THRESHOLD {
        warning_state.armor_warned = false;
    }

    // Hull warning (critical - most urgent)
    if hull_pct <= WARNING_THRESHOLD && hull_pct > 0.0 {
        if !warning_state.hull_warned && warning_state.warning_cooldown <= 0.0 {
            if let Some(source) = sounds.hull_warning.clone() {
                commands.spawn((
                    AudioPlayer(source),
                    PlaybackSettings {
                        mode: PlaybackMode::Despawn,
                        volume: Volume::new(settings.sfx_volume * settings.master_volume),
                        ..default()
                    },
                ));
                warning_state.hull_warned = true;
                warning_state.warning_cooldown = 2.0;
            }
        }
    } else if hull_pct > WARNING_THRESHOLD {
        warning_state.hull_warned = false;
    }
}

// =============================================================================
// EVE-STYLE WARNING SOUND GENERATORS
// =============================================================================

/// Generate shield warning - high-pitched triple beep (EVE style)
fn generate_shield_warning() -> Option<AudioSource> {
    let sample_rate = 44100u32;
    let duration = 0.6;
    let num_samples = (sample_rate as f32 * duration) as usize;
    let mut samples = Vec::with_capacity(num_samples);

    for i in 0..num_samples {
        let t = i as f32 / sample_rate as f32;

        // Three beeps
        let beep_duration = 0.12;
        let gap = 0.08;
        let cycle = beep_duration + gap;

        let beep_num = (t / cycle).floor() as i32;
        let beep_t = t - (beep_num as f32 * cycle);

        let sample = if beep_num < 3 && beep_t < beep_duration {
            let freq = 1200.0; // High pitched
            let wave = (2.0 * PI * freq * beep_t).sin();
            let env = (1.0 - (beep_t / beep_duration)).powf(0.5);
            wave * env * 0.6
        } else {
            0.0
        };

        samples.push(sample.clamp(-1.0, 1.0));
    }

    create_audio_source(&samples, sample_rate)
}

/// Generate armor warning - mid-tone double beep with urgency (EVE style)
fn generate_armor_warning() -> Option<AudioSource> {
    let sample_rate = 44100u32;
    let duration = 0.5;
    let num_samples = (sample_rate as f32 * duration) as usize;
    let mut samples = Vec::with_capacity(num_samples);

    for i in 0..num_samples {
        let t = i as f32 / sample_rate as f32;

        // Two longer beeps
        let beep_duration = 0.15;
        let gap = 0.1;
        let cycle = beep_duration + gap;

        let beep_num = (t / cycle).floor() as i32;
        let beep_t = t - (beep_num as f32 * cycle);

        let sample = if beep_num < 2 && beep_t < beep_duration {
            let freq = 800.0; // Mid tone
            let wave = (2.0 * PI * freq * beep_t).sin();
            // Add slight harmonic for urgency
            let harm = (2.0 * PI * freq * 1.5 * beep_t).sin() * 0.3;
            let env = (1.0 - (beep_t / beep_duration)).powf(0.3);
            (wave + harm) * env * 0.7
        } else {
            0.0
        };

        samples.push(sample.clamp(-1.0, 1.0));
    }

    create_audio_source(&samples, sample_rate)
}

/// Generate hull warning - low urgent alarm (EVE style critical warning)
fn generate_hull_warning() -> Option<AudioSource> {
    let sample_rate = 44100u32;
    let duration = 0.8;
    let num_samples = (sample_rate as f32 * duration) as usize;
    let mut samples = Vec::with_capacity(num_samples);

    for i in 0..num_samples {
        let t = i as f32 / sample_rate as f32;

        // Continuous warbling alarm
        let base_freq = 400.0;
        // Frequency modulation for urgency
        let mod_freq = 8.0; // 8 Hz wobble
        let freq = base_freq + 100.0 * (2.0 * PI * mod_freq * t).sin();

        let wave = (2.0 * PI * freq * t).sin();
        // Add harmonics for harshness
        let harm1 = (2.0 * PI * freq * 2.0 * t).sin() * 0.4;
        let harm2 = (2.0 * PI * freq * 3.0 * t).sin() * 0.2;

        // Envelope with attack
        let env = (1.0 - (-t * 20.0).exp()) * (1.0 - (t / duration).powf(2.0));

        let sample = (wave + harm1 + harm2) * env * 0.65;
        samples.push(sample.clamp(-1.0, 1.0));
    }

    create_audio_source(&samples, sample_rate)
}

// =============================================================================
// NEW SOUND GENERATORS
// =============================================================================

/// Generate missile launch sound - whooshing rocket
fn generate_missile() -> Option<AudioSource> {
    let sample_rate = 44100u32;
    let duration = 0.2;
    let num_samples = (sample_rate as f32 * duration) as usize;
    let mut samples = Vec::with_capacity(num_samples);

    for i in 0..num_samples {
        let t = i as f32 / sample_rate as f32;

        // Whoosh noise
        let noise = (fastrand::f32() * 2.0 - 1.0) * 0.5;

        // Rising frequency for ignition
        let freq = 150.0 + t * 400.0;
        let rumble = (2.0 * PI * freq * t).sin() * 0.4;

        // High hiss
        let hiss = (2.0 * PI * 2000.0 * t).sin() * 0.15 * (-t * 20.0).exp();

        let env = (1.0 - (-t * 30.0).exp()) * (-t * 8.0).exp();

        let sample = ((noise + rumble + hiss) * env * 0.7).clamp(-1.0, 1.0);
        samples.push(sample);
    }

    create_audio_source(&samples, sample_rate)
}

/// Generate wave complete sound - triumphant ascending chime
fn generate_wave_complete() -> Option<AudioSource> {
    let sample_rate = 44100u32;
    let duration = 0.5;
    let num_samples = (sample_rate as f32 * duration) as usize;
    let mut samples = Vec::with_capacity(num_samples);

    for i in 0..num_samples {
        let t = i as f32 / sample_rate as f32;

        // Three ascending notes
        let note1 = if t < 0.15 {
            (2.0 * PI * 523.25 * t).sin() * (1.0 - t / 0.15).powf(0.5) // C5
        } else {
            0.0
        };

        let note2 = if (0.12..0.3).contains(&t) {
            let nt = t - 0.12;
            (2.0 * PI * 659.25 * t).sin() * (1.0 - nt / 0.18).powf(0.5) // E5
        } else {
            0.0
        };

        let note3 = if t >= 0.25 {
            let nt = t - 0.25;
            (2.0 * PI * 783.99 * t).sin() * (-nt * 6.0).exp() // G5
        } else {
            0.0
        };

        let sample = ((note1 + note2 + note3) * 0.5).clamp(-1.0, 1.0);
        samples.push(sample);
    }

    create_audio_source(&samples, sample_rate)
}

/// Generate boss spawn sound - dramatic low impact
fn generate_boss_spawn() -> Option<AudioSource> {
    let sample_rate = 44100u32;
    let duration = 0.8;
    let num_samples = (sample_rate as f32 * duration) as usize;
    let mut samples = Vec::with_capacity(num_samples);

    for i in 0..num_samples {
        let t = i as f32 / sample_rate as f32;

        // Deep impact
        let bass = (2.0 * PI * 60.0 * t).sin() * 0.6;

        // Ominous drone
        let drone = (2.0 * PI * 100.0 * t).sin() * 0.3;
        let drone2 = (2.0 * PI * 150.0 * t).sin() * 0.2;

        // Metallic ring
        let ring = (2.0 * PI * 300.0 * t).sin() * (-t * 4.0).exp() * 0.3;

        // Rumble
        let rumble = (fastrand::f32() * 2.0 - 1.0) * 0.2 * (-t * 3.0).exp();

        let env = (1.0 - (-t * 10.0).exp()) * (-t * 2.5).exp();

        let sample = ((bass + drone + drone2 + ring + rumble) * env * 0.7).clamp(-1.0, 1.0);
        samples.push(sample);
    }

    create_audio_source(&samples, sample_rate)
}

/// Generate overdrive powerup sound - engine rev
fn generate_powerup_overdrive() -> Option<AudioSource> {
    let sample_rate = 44100u32;
    let duration = 0.3;
    let num_samples = (sample_rate as f32 * duration) as usize;
    let mut samples = Vec::with_capacity(num_samples);

    for i in 0..num_samples {
        let t = i as f32 / sample_rate as f32;

        // Rising engine freq
        let freq = 200.0 + t * 600.0;
        let engine = (2.0 * PI * freq * t).sin() * 0.5;

        // Turbo whoosh
        let whoosh = (fastrand::f32() * 2.0 - 1.0) * 0.3 * (t * 4.0).min(1.0);

        let env = (1.0 - (-t * 20.0).exp()) * (1.0 - (t / duration).powf(2.0));

        let sample = ((engine + whoosh) * env * 0.6).clamp(-1.0, 1.0);
        samples.push(sample);
    }

    create_audio_source(&samples, sample_rate)
}

/// Generate damage boost powerup sound - power surge
fn generate_powerup_damage() -> Option<AudioSource> {
    let sample_rate = 44100u32;
    let duration = 0.25;
    let num_samples = (sample_rate as f32 * duration) as usize;
    let mut samples = Vec::with_capacity(num_samples);

    for i in 0..num_samples {
        let t = i as f32 / sample_rate as f32;

        // Power charge
        let charge = (2.0 * PI * (400.0 + t * 800.0) * t).sin() * 0.5;

        // Electric crackle
        let crackle = if fastrand::f32() < 0.15 {
            (fastrand::f32() * 2.0 - 1.0) * 0.4
        } else {
            0.0
        };

        let env = (1.0 - (-t * 30.0).exp()) * (-t * 6.0).exp();

        let sample = ((charge + crackle) * env * 0.6).clamp(-1.0, 1.0);
        samples.push(sample);
    }

    create_audio_source(&samples, sample_rate)
}

/// Generate invulnerability powerup sound - shield activation
fn generate_powerup_invuln() -> Option<AudioSource> {
    let sample_rate = 44100u32;
    let duration = 0.35;
    let num_samples = (sample_rate as f32 * duration) as usize;
    let mut samples = Vec::with_capacity(num_samples);

    for i in 0..num_samples {
        let t = i as f32 / sample_rate as f32;

        // Shield hum
        let hum = (2.0 * PI * 300.0 * t).sin() * 0.3;

        // Shimmer
        let shimmer = (2.0 * PI * 1200.0 * t).sin() * 0.2 * (t * 8.0).sin().abs();

        // Bass impact
        let bass = (2.0 * PI * 80.0 * t).sin() * 0.4 * (-t * 15.0).exp();

        let env = (1.0 - (-t * 20.0).exp()) * (1.0 - (t / duration).powf(3.0));

        let sample = ((hum + shimmer + bass) * env * 0.6).clamp(-1.0, 1.0);
        samples.push(sample);
    }

    create_audio_source(&samples, sample_rate)
}

/// Generate health restore powerup sound - healing chime
fn generate_powerup_health() -> Option<AudioSource> {
    let sample_rate = 44100u32;
    let duration = 0.2;
    let num_samples = (sample_rate as f32 * duration) as usize;
    let mut samples = Vec::with_capacity(num_samples);

    for i in 0..num_samples {
        let t = i as f32 / sample_rate as f32;

        // Gentle ascending tone
        let freq = 600.0 + t * 400.0;
        let tone = (2.0 * PI * freq * t).sin() * 0.4;

        // Soft shimmer
        let shimmer = (2.0 * PI * freq * 2.0 * t).sin() * 0.15;

        let env = (1.0 - (-t * 30.0).exp()) * (-t * 8.0).exp();

        let sample = ((tone + shimmer) * env * 0.5).clamp(-1.0, 1.0);
        samples.push(sample);
    }

    create_audio_source(&samples, sample_rate)
}

/// Generate menu navigation sound - soft blip
fn generate_menu_select() -> Option<AudioSource> {
    let sample_rate = 44100u32;
    let duration = 0.05;
    let num_samples = (sample_rate as f32 * duration) as usize;
    let mut samples = Vec::with_capacity(num_samples);

    for i in 0..num_samples {
        let t = i as f32 / sample_rate as f32;

        let wave = (2.0 * PI * 800.0 * t).sin();
        let env = (-t * 60.0).exp();

        let sample = (wave * env * 0.4).clamp(-1.0, 1.0);
        samples.push(sample);
    }

    create_audio_source(&samples, sample_rate)
}

/// Generate menu confirm sound - satisfying click
fn generate_menu_confirm() -> Option<AudioSource> {
    let sample_rate = 44100u32;
    let duration = 0.1;
    let num_samples = (sample_rate as f32 * duration) as usize;
    let mut samples = Vec::with_capacity(num_samples);

    for i in 0..num_samples {
        let t = i as f32 / sample_rate as f32;

        let wave1 = (2.0 * PI * 600.0 * t).sin() * 0.4;
        let wave2 = (2.0 * PI * 900.0 * t).sin() * 0.3;

        let env = (-t * 30.0).exp();

        let sample = ((wave1 + wave2) * env * 0.5).clamp(-1.0, 1.0);
        samples.push(sample);
    }

    create_audio_source(&samples, sample_rate)
}

// =============================================================================
// ABILITY SOUNDS
// =============================================================================

/// Generate speed ability sound - engine boost whoosh
fn generate_ability_speed() -> Option<AudioSource> {
    let sample_rate = 44100u32;
    let duration = 0.4;
    let num_samples = (sample_rate as f32 * duration) as usize;
    let mut samples = Vec::with_capacity(num_samples);

    for i in 0..num_samples {
        let t = i as f32 / sample_rate as f32;

        // Rising engine frequency
        let freq = 150.0 + t * 800.0;
        let engine = (2.0 * PI * freq * t).sin() * 0.4;

        // Turbo whoosh (filtered noise)
        let whoosh = (fastrand::f32() * 2.0 - 1.0) * 0.35 * (t * 5.0).min(1.0);

        // High overtone
        let high = (2.0 * PI * (freq * 2.5) * t).sin() * 0.15 * (t * 8.0).min(1.0);

        let env = (1.0 - (-t * 15.0).exp()) * (1.0 - (t / duration).powf(1.5));

        let sample = ((engine + whoosh + high) * env * 0.7).clamp(-1.0, 1.0);
        samples.push(sample);
    }

    create_audio_source(&samples, sample_rate)
}

/// Generate shield ability sound - energy bubble activation
fn generate_ability_shield() -> Option<AudioSource> {
    let sample_rate = 44100u32;
    let duration = 0.35;
    let num_samples = (sample_rate as f32 * duration) as usize;
    let mut samples = Vec::with_capacity(num_samples);

    for i in 0..num_samples {
        let t = i as f32 / sample_rate as f32;

        // Shield activation sweep
        let freq = 800.0 - t * 400.0;
        let sweep = (2.0 * PI * freq * t).sin() * 0.4;

        // Shimmer
        let shimmer = (2.0 * PI * 2400.0 * t).sin() * 0.2 * (1.0 + (PI * 20.0 * t).sin() * 0.5);

        // Bubble pop at start
        let pop = (2.0 * PI * 300.0 * t).sin() * (-t * 60.0).exp() * 0.3;

        let env = (1.0 - (-t * 25.0).exp()) * (-t * 4.0).exp();

        let sample = ((sweep + shimmer + pop) * env * 0.6).clamp(-1.0, 1.0);
        samples.push(sample);
    }

    create_audio_source(&samples, sample_rate)
}

/// Generate armor ability sound - metallic clang/hardening
fn generate_ability_armor() -> Option<AudioSource> {
    let sample_rate = 44100u32;
    let duration = 0.3;
    let num_samples = (sample_rate as f32 * duration) as usize;
    let mut samples = Vec::with_capacity(num_samples);

    for i in 0..num_samples {
        let t = i as f32 / sample_rate as f32;

        // Metallic clang
        let clang = (2.0 * PI * 400.0 * t).sin() * 0.3 * (-t * 20.0).exp();

        // Harmonic overtones (metallic)
        let harm1 = (2.0 * PI * 800.0 * t).sin() * 0.2 * (-t * 25.0).exp();
        let harm2 = (2.0 * PI * 1200.0 * t).sin() * 0.15 * (-t * 30.0).exp();

        // Low rumble for weight
        let rumble = (2.0 * PI * 80.0 * t).sin() * 0.25 * (-t * 10.0).exp();

        let sample = ((clang + harm1 + harm2 + rumble) * 0.7).clamp(-1.0, 1.0);
        samples.push(sample);
    }

    create_audio_source(&samples, sample_rate)
}

/// Generate weapon ability sound - charging burst
fn generate_ability_weapon() -> Option<AudioSource> {
    let sample_rate = 44100u32;
    let duration = 0.25;
    let num_samples = (sample_rate as f32 * duration) as usize;
    let mut samples = Vec::with_capacity(num_samples);

    for i in 0..num_samples {
        let t = i as f32 / sample_rate as f32;

        // Rapid charge up
        let freq = 200.0 + t * 1200.0;
        let charge = (2.0 * PI * freq * t).sin() * 0.4;

        // Burst
        let burst = if t > 0.15 {
            (2.0 * PI * 500.0 * t).sin() * 0.5 * (-(t - 0.15) * 40.0).exp()
        } else {
            0.0
        };

        // Crackle
        let crackle = if fastrand::f32() < 0.1 {
            (fastrand::f32() * 2.0 - 1.0) * 0.3
        } else {
            0.0
        };

        let env = (1.0 - (-t * 40.0).exp());

        let sample = ((charge + burst + crackle) * env * 0.6).clamp(-1.0, 1.0);
        samples.push(sample);
    }

    create_audio_source(&samples, sample_rate)
}

/// Generate drone ability sound - mechanical launch
fn generate_ability_drone() -> Option<AudioSource> {
    let sample_rate = 44100u32;
    let duration = 0.4;
    let num_samples = (sample_rate as f32 * duration) as usize;
    let mut samples = Vec::with_capacity(num_samples);

    for i in 0..num_samples {
        let t = i as f32 / sample_rate as f32;

        // Mechanical hum
        let hum = (2.0 * PI * 120.0 * t).sin() * 0.3;

        // Drone whine (rising)
        let freq = 400.0 + t * 300.0;
        let whine = (2.0 * PI * freq * t).sin() * 0.25;

        // Launch click
        let click = (2.0 * PI * 1000.0 * t).sin() * (-t * 100.0).exp() * 0.4;

        // Propeller flutter
        let flutter = (2.0 * PI * 60.0 * t).sin() * 0.15 * (t * 4.0).min(1.0);

        let env = (1.0 - (-t * 20.0).exp()) * (1.0 - (t / duration).powf(2.0));

        let sample = ((hum + whine + click + flutter) * env * 0.6).clamp(-1.0, 1.0);
        samples.push(sample);
    }

    create_audio_source(&samples, sample_rate)
}

/// Generate debuff ability sound - disrupting pulse
fn generate_ability_debuff() -> Option<AudioSource> {
    let sample_rate = 44100u32;
    let duration = 0.35;
    let num_samples = (sample_rate as f32 * duration) as usize;
    let mut samples = Vec::with_capacity(num_samples);

    for i in 0..num_samples {
        let t = i as f32 / sample_rate as f32;

        // Warping frequency
        let warp = (2.0 * PI * (300.0 + 200.0 * (PI * 15.0 * t).sin()) * t).sin() * 0.4;

        // Disruptor pulse
        let pulse = (2.0 * PI * 100.0 * t).sin() * 0.3 * (1.0 + (PI * 8.0 * t).sin() * 0.5);

        // Static
        let static_noise = (fastrand::f32() * 2.0 - 1.0) * 0.15;

        let env = (1.0 - (-t * 20.0).exp()) * (-t * 5.0).exp();

        let sample = ((warp + pulse + static_noise) * env * 0.6).clamp(-1.0, 1.0);
        samples.push(sample);
    }

    create_audio_source(&samples, sample_rate)
}

/// Generate damage ability sound - power surge
fn generate_ability_damage() -> Option<AudioSource> {
    let sample_rate = 44100u32;
    let duration = 0.3;
    let num_samples = (sample_rate as f32 * duration) as usize;
    let mut samples = Vec::with_capacity(num_samples);

    for i in 0..num_samples {
        let t = i as f32 / sample_rate as f32;

        // Power charge
        let charge = (2.0 * PI * (500.0 + t * 600.0) * t).sin() * 0.4;

        // Impact hit
        let impact = (2.0 * PI * 150.0 * t).sin() * (-t * 30.0).exp() * 0.5;

        // Crackle
        let crackle = if fastrand::f32() < 0.12 {
            (fastrand::f32() * 2.0 - 1.0) * 0.35
        } else {
            0.0
        };

        let env = (1.0 - (-t * 35.0).exp()) * (-t * 6.0).exp();

        let sample = ((charge + impact + crackle) * env * 0.7).clamp(-1.0, 1.0);
        samples.push(sample);
    }

    create_audio_source(&samples, sample_rate)
}

// =============================================================================
// NEW PLAYBACK SYSTEMS
// =============================================================================

/// Play ability activation sounds
fn play_ability_sounds(
    mut commands: Commands,
    mut ability_events: EventReader<AbilityActivatedEvent>,
    sounds: Res<SoundAssets>,
    settings: Res<SoundSettings>,
) {
    if !settings.enabled {
        ability_events.clear();
        return;
    }

    for event in ability_events.read() {
        let sound = match event.ability_type {
            AbilityType::Overdrive | AbilityType::Afterburner => sounds.ability_speed.clone(),
            AbilityType::ShieldBoost => sounds.ability_shield.clone(),
            AbilityType::ArmorHardener | AbilityType::ArmorRepair => sounds.ability_armor.clone(),
            AbilityType::RocketBarrage | AbilityType::Salvo | AbilityType::Scorch => {
                sounds.ability_weapon.clone()
            }
            AbilityType::DeployDrone | AbilityType::DroneBay => sounds.ability_drone.clone(),
            AbilityType::WarpDisruptor => sounds.ability_debuff.clone(),
            AbilityType::CloseRange => sounds.ability_damage.clone(),
            AbilityType::None => None,
        };

        if let Some(source) = sound {
            commands.spawn((
                AudioPlayer(source),
                PlaybackSettings {
                    mode: PlaybackMode::Despawn,
                    volume: Volume::new(settings.sfx_volume * settings.master_volume * 0.85),
                    ..default()
                },
            ));
        }
    }
}

/// Play wave complete sound
fn play_wave_complete_sound(
    mut commands: Commands,
    mut wave_events: EventReader<WaveCompleteEvent>,
    sounds: Res<SoundAssets>,
    settings: Res<SoundSettings>,
) {
    if !settings.enabled {
        wave_events.clear();
        return;
    }

    for _event in wave_events.read() {
        if let Some(source) = sounds.wave_complete.clone() {
            commands.spawn((
                AudioPlayer(source),
                PlaybackSettings {
                    mode: PlaybackMode::Despawn,
                    volume: Volume::new(settings.sfx_volume * settings.master_volume * 0.8),
                    ..default()
                },
            ));
        }
    }
}

/// Play boss spawn sound
fn play_boss_spawn_sound(
    mut commands: Commands,
    mut boss_events: EventReader<BossSpawnEvent>,
    sounds: Res<SoundAssets>,
    settings: Res<SoundSettings>,
) {
    if !settings.enabled {
        boss_events.clear();
        return;
    }

    for _event in boss_events.read() {
        if let Some(source) = sounds.boss_spawn.clone() {
            commands.spawn((
                AudioPlayer(source),
                PlaybackSettings {
                    mode: PlaybackMode::Despawn,
                    volume: Volume::new(settings.sfx_volume * settings.master_volume * 0.9),
                    ..default()
                },
            ));
        }
    }
}
