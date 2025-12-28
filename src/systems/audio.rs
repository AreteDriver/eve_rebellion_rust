//! Audio System
//!
//! Procedural sound effects for EVE Rebellion.
//! Uses hound crate for proper WAV generation.

use bevy::prelude::*;
use bevy::audio::{PlaybackMode, Volume};
use std::f32::consts::PI;
use std::io::Cursor;

use crate::core::*;

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
                ).run_if(in_state(GameState::Playing)),
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
        } * (-t * 5.0).exp() * 0.3;

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
    Some(AudioSource { bytes: Arc::from(wav_data.into_boxed_slice()) })
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
            _ => sounds.autocannon.clone(),
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

/// Play pickup sounds
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

    for _event in pickup_events.read() {
        if let Some(source) = sounds.pickup.clone() {
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
