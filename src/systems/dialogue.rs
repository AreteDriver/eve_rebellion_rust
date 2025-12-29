//! Dialogue System
//!
//! Handles Elder dialogue display during gameplay.

#![allow(dead_code)]

use crate::core::*;
use bevy::prelude::*;

/// Dialogue plugin
pub struct DialoguePlugin;

impl Plugin for DialoguePlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<DialogueSystem>()
            .add_event::<DialogueEvent>()
            .add_systems(
                Update,
                (handle_dialogue_events, update_dialogue_timer)
                    .run_if(in_state(GameState::Playing)),
            );
    }
}

/// Dialogue trigger types
#[derive(Clone, Debug, PartialEq)]
pub enum DialogueTrigger {
    /// Mission briefing at stage start
    StageBriefing(u32),
    /// Boss intro dialogue
    BossIntro(String),
    /// Boss defeated
    BossDefeated(String),
    /// Act transition
    ActComplete(u32),
    /// Liberation milestone reached
    LiberationMilestone(u32),
    /// Combat callout
    CombatCallout(CombatCalloutType),
    /// Post-mission success
    MissionSuccess,
    /// Perfect run (no damage)
    PerfectRun,
    /// Player death
    PlayerDeath,
}

/// Combat callout types
#[derive(Clone, Debug, PartialEq)]
pub enum CombatCalloutType {
    LowHealth,
    NearDeath,
    BerserkActive,
    Overheated,
    BossLowHealth,
    WaveIncoming,
    LiberationPod,
}

/// Event to trigger dialogue
#[derive(Event)]
pub struct DialogueEvent {
    pub trigger: DialogueTrigger,
    /// Optional custom text (overrides trigger-based lookup)
    pub custom_text: Option<String>,
    /// Duration to show dialogue (default 4.0)
    pub duration: f32,
    /// Priority (higher = more important, can interrupt lower)
    pub priority: u8,
}

impl Default for DialogueEvent {
    fn default() -> Self {
        Self {
            trigger: DialogueTrigger::MissionSuccess,
            custom_text: None,
            duration: 4.0,
            priority: 1,
        }
    }
}

impl DialogueEvent {
    pub fn stage_briefing(stage: u32) -> Self {
        Self {
            trigger: DialogueTrigger::StageBriefing(stage),
            duration: 5.0,
            priority: 10,
            ..default()
        }
    }

    pub fn boss_intro(name: String, dialogue: String) -> Self {
        Self {
            trigger: DialogueTrigger::BossIntro(name),
            custom_text: Some(dialogue),
            duration: 3.0,
            priority: 8,
        }
    }

    pub fn boss_defeated(name: String, dialogue: String) -> Self {
        Self {
            trigger: DialogueTrigger::BossDefeated(name),
            custom_text: Some(dialogue),
            duration: 4.0,
            priority: 9,
        }
    }

    pub fn combat_callout(callout_type: CombatCalloutType) -> Self {
        Self {
            trigger: DialogueTrigger::CombatCallout(callout_type),
            duration: 2.5,
            priority: 3,
            ..default()
        }
    }

    pub fn liberation_milestone(count: u32) -> Self {
        Self {
            trigger: DialogueTrigger::LiberationMilestone(count),
            duration: 4.0,
            priority: 5,
            ..default()
        }
    }

    pub fn act_complete(act: u32) -> Self {
        Self {
            trigger: DialogueTrigger::ActComplete(act),
            duration: 6.0,
            priority: 10,
            ..default()
        }
    }
}

/// Dialogue system state
#[derive(Resource, Default)]
pub struct DialogueSystem {
    /// Currently displayed text
    pub active_text: Option<String>,
    /// Speaker name (usually "Tribal Elder")
    pub speaker: String,
    /// Time remaining to show current dialogue
    pub timer: f32,
    /// Current dialogue priority
    pub priority: u8,
    /// Queue of pending dialogues
    pub queue: Vec<(String, f32, u8)>, // (text, duration, priority)
    /// Last liberation milestone shown
    pub last_liberation_milestone: u32,
    /// Has shown stage briefing for current stage
    pub shown_stage_briefing: bool,
}

impl DialogueSystem {
    /// Check if dialogue is currently active
    pub fn is_active(&self) -> bool {
        self.active_text.is_some()
    }

    /// Show dialogue immediately (respects priority)
    pub fn show(&mut self, text: String, duration: f32, priority: u8) {
        if priority >= self.priority || self.active_text.is_none() {
            self.active_text = Some(text);
            self.timer = duration;
            self.priority = priority;
            self.speaker = "Tribal Elder".to_string();
        } else {
            // Queue lower priority dialogue
            self.queue.push((text, duration, priority));
        }
    }

    /// Clear current dialogue
    pub fn clear(&mut self) {
        self.active_text = None;
        self.timer = 0.0;
        self.priority = 0;
    }

    /// Reset for new game
    pub fn reset(&mut self) {
        self.clear();
        self.queue.clear();
        self.last_liberation_milestone = 0;
        self.shown_stage_briefing = false;
    }
}

/// Handle incoming dialogue events
fn handle_dialogue_events(
    mut events: EventReader<DialogueEvent>,
    mut dialogue: ResMut<DialogueSystem>,
) {
    for event in events.read() {
        let text = if let Some(custom) = &event.custom_text {
            custom.clone()
        } else {
            get_dialogue_text(&event.trigger)
        };

        dialogue.show(text, event.duration, event.priority);
    }
}

/// Update dialogue timer and process queue
fn update_dialogue_timer(time: Res<Time>, mut dialogue: ResMut<DialogueSystem>) {
    if dialogue.active_text.is_some() {
        dialogue.timer -= time.delta_secs();

        if dialogue.timer <= 0.0 {
            dialogue.clear();

            // Process queue (highest priority first)
            if !dialogue.queue.is_empty() {
                dialogue.queue.sort_by(|a, b| b.2.cmp(&a.2));
                if let Some((text, duration, priority)) = dialogue.queue.pop() {
                    dialogue.show(text, duration, priority);
                }
            }
        }
    }
}

/// Get dialogue text for a trigger
fn get_dialogue_text(trigger: &DialogueTrigger) -> String {
    match trigger {
        DialogueTrigger::StageBriefing(stage) => get_stage_briefing(*stage),
        DialogueTrigger::BossIntro(name) => format!("{} approaches...", name),
        DialogueTrigger::BossDefeated(name) => format!("{} destroyed!", name),
        DialogueTrigger::ActComplete(act) => get_act_complete(*act),
        DialogueTrigger::LiberationMilestone(count) => get_liberation_dialogue(*count),
        DialogueTrigger::CombatCallout(callout) => get_combat_callout(callout),
        DialogueTrigger::MissionSuccess => get_success_dialogue(),
        DialogueTrigger::PerfectRun => get_perfect_dialogue(),
        DialogueTrigger::PlayerDeath => get_death_dialogue(),
    }
}

/// Get stage briefing dialogue
fn get_stage_briefing(stage: u32) -> String {
    match stage {
        1 => "The Elders have watched from the shadows for centuries. Now we strike. A slave convoy approaches - lightly defended. Prove you are worthy to fly with the Fleet.",
        2 => "Imperial patrol routes grow predictable. Intercept and destroy. No survivors to report our movements.",
        3 => "An orbital depot supplies their patrols. Cripple it. Let them know nowhere is safe.",
        4 => "A minor Holder's estate. His slaves await liberation. His guards await death. Do not disappoint either.",
        5 => "The Empire reels. Press the attack. Their customs checkpoint controls this sector - destroy it.",
        6 => "An Inquisitor vessel enforces their religious tyranny. Show them our faith is stronger.",
        7 => "Imperial Navy strike group inbound. Elite pilots. But they have never faced the fury of the liberated.",
        8 => "The stargate defense grid controls access to core Amarr space. Disable it. Open the path.",
        9 => "Every slave freed is a soul returned to our people. This battlestation holds thousands. Crack it open.",
        10 => "This is the hour our ancestors dreamed of. An Armageddon-class battleship guards their retreat. End it.",
        11 => "Their carrier launches endless fighters. Endless. But you are more endless still.",
        12 => "The Lord Admiral himself. Two centuries of Imperial service. He ends today.",
        13 => "The chains break today. An Avatar-class Titan - the Empire's last desperate response. Destroy it, and the liberation is complete. The Elders watch. History watches. Fly well, pilot.",
        _ => "The Fleet awaits your command.",
    }.to_string()
}

/// Get act complete dialogue
fn get_act_complete(act: u32) -> String {
    match act {
        1 => "The first victories are won. But this was merely the beginning. The true storm approaches.",
        2 => "The Empire's outer defenses crumble. Now we strike at their heart. Prepare yourself.",
        3 => "It is done. The Titan falls. The Elder Fleet's mission is complete. Millions are free. And you... you are legend.",
        _ => "The Fleet salutes you.",
    }.to_string()
}

/// Get liberation milestone dialogue
fn get_liberation_dialogue(count: u32) -> String {
    match count {
        100 => "A hundred souls returned to freedom. The Elders see your worth.",
        250 => "Two hundred and fifty. You begin to understand what this means.",
        500 => "Five hundred chains broken. You carry the spirit of the Rebellion.",
        1000 => "A thousand lives reclaimed. Songs will be sung of this day.",
        2500 => "Twenty-five hundred souls freed by your hand. A village reborn.",
        5000 => "Five thousand. You have liberated a city. History will remember.",
        10000 => "Ten thousand souls freed by your hand. You are legend now.",
        25000 => "The Elders bow to you. Twenty-five thousand. A nation freed.",
        _ => "Souls freed. This is why we fight.",
    }
    .to_string()
}

/// Get combat callout dialogue
fn get_combat_callout(callout: &CombatCalloutType) -> String {
    match callout {
        CombatCalloutType::LowHealth => "Your ship struggles. Fight smarter.",
        CombatCalloutType::NearDeath => "Do not fall here. Not now. Not when we are so close.",
        CombatCalloutType::BerserkActive => "The ancestors fill you with rage. Use it!",
        CombatCalloutType::Overheated => "Your weapons strain. But do not stop. Never stop.",
        CombatCalloutType::BossLowHealth => "It weakens! Strike true!",
        CombatCalloutType::WaveIncoming => "More enemies approach. Steel yourself.",
        CombatCalloutType::LiberationPod => "A liberation pod! Collect it - a soul awaits freedom.",
    }
    .to_string()
}

/// Get random success dialogue
fn get_success_dialogue() -> String {
    let options = [
        "Well flown, pilot. The Elders take note.",
        "Another blow struck for freedom. Continue.",
        "Your ancestors would be proud this day.",
        "Steel and fire. That is what you have become.",
        "The Empire bleeds. Do not let them recover.",
        "Every ship you destroy is a message: we are coming.",
        "The rust falls away. Only strength remains.",
        "You carry the fury of generations. Let it burn.",
    ];
    options[fastrand::usize(..options.len())].to_string()
}

/// Get random perfect run dialogue
fn get_perfect_dialogue() -> String {
    let options = [
        "Flawless. The Elders are impressed.",
        "Not a scratch. You fly as our ancestors did - untouchable.",
        "Perfect execution. You honor the Fleet.",
    ];
    options[fastrand::usize(..options.len())].to_string()
}

/// Get death dialogue
fn get_death_dialogue() -> String {
    "You fall... but the Fleet continues. Rise again, pilot.".to_string()
}

/// Get liberation milestone thresholds
pub fn get_liberation_milestones() -> &'static [u32] {
    &[100, 250, 500, 1000, 2500, 5000, 10000, 25000]
}

/// Check if a count has crossed a milestone
pub fn check_liberation_milestone(old_count: u32, new_count: u32) -> Option<u32> {
    get_liberation_milestones().iter().find(|&&milestone| old_count < milestone && new_count >= milestone).copied()
}
