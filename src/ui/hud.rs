//! Heads-Up Display
//!
//! In-game UI: health bars, score, combo, heat, berserk meter, powerup indicators.
//! EVE-style status panel with capacitor and health rings.

use bevy::prelude::*;
use crate::core::*;
use crate::entities::{Player, ShipStats, PowerupEffects, Boss, BossData, BossState, WingmanTracker, Wingman};
use crate::systems::{ComboHeatSystem, DialogueSystem};

/// HUD plugin
pub struct HudPlugin;

impl Plugin for HudPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(OnEnter(GameState::Playing), spawn_hud)
            .add_systems(
                Update,
                (
                    update_score_display,
                    update_health_bars,
                    update_berserk_meter,
                    update_combo_display,
                    update_heat_display,
                    update_combo_kills,
                    update_powerup_indicators,
                    update_wave_display,
                    update_mission_display,
                    update_boss_health_bar,
                    update_dialogue_display,
                    update_wingman_gauge,
                ).run_if(in_state(GameState::Playing)),
            )
            .add_systems(OnExit(GameState::Playing), despawn_hud);
    }
}

/// Marker for HUD root
#[derive(Component)]
pub struct HudRoot;

/// Score text
#[derive(Component)]
pub struct ScoreText;

/// Combo/multiplier text
#[derive(Component)]
pub struct ComboText;

/// Style grade text
#[derive(Component)]
pub struct GradeText;

/// Shield bar
#[derive(Component)]
pub struct ShieldBar;

/// Armor bar
#[derive(Component)]
pub struct ArmorBar;

/// Hull bar
#[derive(Component)]
pub struct HullBar;

/// Capacitor bar
#[derive(Component)]
pub struct CapacitorBar;

/// Berserk meter bar
#[derive(Component)]
pub struct BerserkBar;

/// Heat bar
#[derive(Component)]
pub struct HeatBar;

/// Combo kill count text
#[derive(Component)]
pub struct ComboKillsText;

/// Wave display text
#[derive(Component)]
pub struct WaveText;

/// Mission name text
#[derive(Component)]
pub struct MissionNameText;

/// Mission objective text
#[derive(Component)]
pub struct ObjectiveText;

/// Souls liberated text
#[derive(Component)]
pub struct SoulsText;

/// Powerup indicator container
#[derive(Component)]
pub struct PowerupIndicator;

/// Overdrive indicator
#[derive(Component)]
pub struct OverdriveIndicator;

/// Damage boost indicator
#[derive(Component)]
pub struct DamageBoostIndicator;

/// Invuln indicator
#[derive(Component)]
pub struct InvulnIndicator;

/// Boss health bar container
#[derive(Component)]
pub struct BossHealthContainer;

/// Boss health bar fill
#[derive(Component)]
pub struct BossHealthFill;

/// Boss name text
#[derive(Component)]
pub struct BossNameText;

/// Stage display text
#[derive(Component)]
pub struct StageText;

/// Dialogue box container
#[derive(Component)]
pub struct DialogueContainer;

/// Dialogue speaker name text
#[derive(Component)]
pub struct DialogueSpeakerText;

/// Dialogue content text
#[derive(Component)]
pub struct DialogueContentText;

/// Wingman gauge container
#[derive(Component)]
pub struct WingmanGauge;

/// Wingman gauge fill bar
#[derive(Component)]
pub struct WingmanGaugeFill;

/// Wingman count text
#[derive(Component)]
pub struct WingmanCountText;

fn spawn_hud(mut commands: Commands) {
    commands
        .spawn((
            HudRoot,
            Node {
                width: Val::Percent(100.0),
                height: Val::Percent(100.0),
                flex_direction: FlexDirection::Column,
                justify_content: JustifyContent::SpaceBetween,
                ..default()
            },
        ))
        .with_children(|parent| {
            // === TOP BAR ===
            parent
                .spawn(Node {
                    width: Val::Percent(100.0),
                    height: Val::Px(80.0),
                    flex_direction: FlexDirection::Row,
                    justify_content: JustifyContent::SpaceBetween,
                    padding: UiRect::all(Val::Px(10.0)),
                    ..default()
                })
                .with_children(|top| {
                    // Left: Score, mission, and wave
                    top.spawn(Node {
                        flex_direction: FlexDirection::Column,
                        align_items: AlignItems::FlexStart,
                        ..default()
                    }).with_children(|left| {
                        left.spawn((
                            ScoreText,
                            Text::new("SCORE: 0"),
                            TextFont {
                                font_size: 28.0,
                                ..default()
                            },
                            TextColor(Color::WHITE),
                        ));
                        left.spawn((
                            MissionNameText,
                            Text::new(""),
                            TextFont {
                                font_size: 14.0,
                                ..default()
                            },
                            TextColor(Color::srgb(0.8, 0.6, 0.3)), // Rust/amber
                        ));
                        left.spawn((
                            WaveText,
                            Text::new("WAVE 1"),
                            TextFont {
                                font_size: 16.0,
                                ..default()
                            },
                            TextColor(Color::srgb(0.6, 0.6, 0.6)),
                        ));
                        left.spawn((
                            ObjectiveText,
                            Text::new(""),
                            TextFont {
                                font_size: 12.0,
                                ..default()
                            },
                            TextColor(Color::srgb(0.5, 0.8, 0.5)), // Green for objectives
                        ));
                        left.spawn((
                            SoulsText,
                            Text::new(""),
                            TextFont {
                                font_size: 12.0,
                                ..default()
                            },
                            TextColor(Color::srgb(0.4, 0.7, 1.0)), // Blue for souls
                        ));
                    });

                    // Center: Combo kills and tier
                    top.spawn(Node {
                        flex_direction: FlexDirection::Column,
                        align_items: AlignItems::Center,
                        ..default()
                    }).with_children(|center| {
                        center.spawn((
                            ComboKillsText,
                            Text::new(""),
                            TextFont {
                                font_size: 36.0,
                                ..default()
                            },
                            TextColor(Color::srgb(1.0, 0.8, 0.2)),
                        ));
                    });

                    // Right: Multiplier and Grade
                    top.spawn(Node {
                        flex_direction: FlexDirection::Column,
                        align_items: AlignItems::End,
                        ..default()
                    }).with_children(|right| {
                        right.spawn((
                            ComboText,
                            Text::new("x1.0"),
                            TextFont {
                                font_size: 20.0,
                                ..default()
                            },
                            TextColor(Color::srgb(1.0, 0.9, 0.3)),
                        ));
                        right.spawn((
                            GradeText,
                            Text::new("D"),
                            TextFont {
                                font_size: 32.0,
                                ..default()
                            },
                            TextColor(Color::srgb(0.5, 0.5, 0.5)),
                        ));
                    });
                });

            // === BOSS HEALTH BAR (hidden by default) ===
            parent
                .spawn((
                    BossHealthContainer,
                    Node {
                        width: Val::Percent(100.0),
                        height: Val::Px(50.0),
                        flex_direction: FlexDirection::Column,
                        align_items: AlignItems::Center,
                        margin: UiRect::top(Val::Px(10.0)),
                        display: Display::None, // Hidden until boss spawns
                        ..default()
                    },
                ))
                .with_children(|boss_ui| {
                    // Boss name
                    boss_ui.spawn((
                        BossNameText,
                        Text::new("BOSS NAME"),
                        TextFont {
                            font_size: 18.0,
                            ..default()
                        },
                        TextColor(Color::srgb(1.0, 0.3, 0.3)),
                    ));
                    // Health bar background
                    boss_ui.spawn((
                        Node {
                            width: Val::Percent(60.0),
                            height: Val::Px(16.0),
                            margin: UiRect::top(Val::Px(5.0)),
                            ..default()
                        },
                        BackgroundColor(Color::srgba(0.2, 0.0, 0.0, 0.8)),
                    )).with_children(|bar| {
                        bar.spawn((
                            BossHealthFill,
                            Node {
                                width: Val::Percent(100.0),
                                height: Val::Percent(100.0),
                                ..default()
                            },
                            BackgroundColor(Color::srgb(0.9, 0.2, 0.2)),
                        ));
                    });
                });

            // === MIDDLE: Powerup indicators ===
            parent
                .spawn((
                    PowerupIndicator,
                    Node {
                        width: Val::Percent(100.0),
                        height: Val::Px(30.0),
                        flex_direction: FlexDirection::Row,
                        justify_content: JustifyContent::Center,
                        column_gap: Val::Px(10.0),
                        ..default()
                    },
                ))
                .with_children(|indicators| {
                    // Overdrive indicator
                    indicators.spawn((
                        OverdriveIndicator,
                        Text::new(""),
                        TextFont {
                            font_size: 18.0,
                            ..default()
                        },
                        TextColor(Color::srgb(0.3, 0.9, 1.0)),
                    ));
                    // Damage boost indicator
                    indicators.spawn((
                        DamageBoostIndicator,
                        Text::new(""),
                        TextFont {
                            font_size: 18.0,
                            ..default()
                        },
                        TextColor(Color::srgb(1.0, 0.3, 0.3)),
                    ));
                    // Invuln indicator
                    indicators.spawn((
                        InvulnIndicator,
                        Text::new(""),
                        TextFont {
                            font_size: 18.0,
                            ..default()
                        },
                        TextColor(Color::srgb(1.0, 1.0, 1.0)),
                    ));
                });

            // === BOTTOM BAR: Health and meters ===
            parent
                .spawn(Node {
                    width: Val::Percent(100.0),
                    height: Val::Px(130.0),
                    flex_direction: FlexDirection::Row,
                    justify_content: JustifyContent::SpaceBetween,
                    padding: UiRect::all(Val::Px(10.0)),
                    ..default()
                })
                .with_children(|bottom| {
                    // Left side: Health bars (EVE-style vertical arrangement)
                    bottom
                        .spawn(Node {
                            flex_direction: FlexDirection::Column,
                            row_gap: Val::Px(3.0),
                            ..default()
                        })
                        .with_children(|left| {
                            // Shield bar (blue)
                            spawn_health_bar(left, ShieldBar, COLOR_SHIELD, "SHIELD");
                            // Armor bar (orange/gold)
                            spawn_health_bar(left, ArmorBar, COLOR_ARMOR, "ARMOR");
                            // Hull bar (gray)
                            spawn_health_bar(left, HullBar, COLOR_HULL, "HULL");
                            // Capacitor bar (yellow)
                            spawn_health_bar(left, CapacitorBar, COLOR_CAPACITOR, "CAP");
                        });

                    // Center: Status meters (Heat, Berserk)
                    bottom
                        .spawn(Node {
                            flex_direction: FlexDirection::Column,
                            row_gap: Val::Px(3.0),
                            align_items: AlignItems::Center,
                            ..default()
                        })
                        .with_children(|center| {
                            // Heat meter (orange/red)
                            spawn_health_bar(center, HeatBar, Color::srgb(1.0, 0.5, 0.0), "HEAT");
                            // Berserk meter (purple)
                            spawn_health_bar(center, BerserkBar, Color::srgb(0.8, 0.2, 0.8), "BERSERK");
                        });

                    // Right side: Wingman gauge (Rifter only)
                    bottom
                        .spawn((
                            WingmanGauge,
                            Node {
                                flex_direction: FlexDirection::Column,
                                row_gap: Val::Px(4.0),
                                align_items: AlignItems::FlexEnd,
                                ..default()
                            },
                        ))
                        .with_children(|right| {
                            // Label
                            right.spawn((
                                Text::new("WINGMAN"),
                                TextFont {
                                    font_size: 12.0,
                                    ..default()
                                },
                                TextColor(Color::srgb(0.8, 0.6, 0.3)),
                            ));

                            // Progress bar container
                            right
                                .spawn((
                                    Node {
                                        width: Val::Px(100.0),
                                        height: Val::Px(10.0),
                                        border: UiRect::all(Val::Px(1.0)),
                                        ..default()
                                    },
                                    BackgroundColor(Color::srgba(0.15, 0.1, 0.05, 0.9)),
                                    BorderColor(Color::srgb(0.5, 0.35, 0.2)),
                                    BorderRadius::all(Val::Px(2.0)),
                                ))
                                .with_children(|bar| {
                                    bar.spawn((
                                        WingmanGaugeFill,
                                        Node {
                                            width: Val::Percent(0.0),
                                            height: Val::Percent(100.0),
                                            ..default()
                                        },
                                        BackgroundColor(Color::srgb(0.8, 0.5, 0.2)),
                                        BorderRadius::all(Val::Px(2.0)),
                                    ));
                                });

                            // Kill count
                            right.spawn((
                                WingmanCountText,
                                Text::new("0/15"),
                                TextFont {
                                    font_size: 11.0,
                                    ..default()
                                },
                                TextColor(Color::srgb(0.6, 0.5, 0.35)),
                            ));

                            // Active wingman icons placeholder
                            right.spawn((
                                Text::new(""),
                                TextFont {
                                    font_size: 14.0,
                                    ..default()
                                },
                                TextColor(Color::srgb(0.9, 0.7, 0.4)),
                            ));
                        });
                });
        });

    // === DIALOGUE BOX (separate from HUD root for positioning) ===
    commands
        .spawn((
            DialogueContainer,
            Node {
                position_type: PositionType::Absolute,
                bottom: Val::Px(120.0),
                left: Val::Percent(15.0),
                width: Val::Percent(70.0),
                height: Val::Auto,
                flex_direction: FlexDirection::Row,
                align_items: AlignItems::FlexStart,
                padding: UiRect::all(Val::Px(15.0)),
                column_gap: Val::Px(15.0),
                display: Display::None, // Hidden by default
                ..default()
            },
            BackgroundColor(Color::srgba(0.05, 0.05, 0.1, 0.9)),
            BorderRadius::all(Val::Px(8.0)),
        ))
        .with_children(|dialogue| {
            // Elder portrait placeholder (rust-colored square)
            dialogue.spawn((
                Node {
                    width: Val::Px(64.0),
                    height: Val::Px(64.0),
                    ..default()
                },
                BackgroundColor(Color::srgb(0.6, 0.35, 0.2)), // Rust/bronze color for Minmatar
                BorderRadius::all(Val::Px(4.0)),
            ));

            // Text container
            dialogue
                .spawn(Node {
                    flex_direction: FlexDirection::Column,
                    flex_grow: 1.0,
                    row_gap: Val::Px(5.0),
                    ..default()
                })
                .with_children(|text_area| {
                    // Speaker name
                    text_area.spawn((
                        DialogueSpeakerText,
                        Text::new("Tribal Elder"),
                        TextFont {
                            font_size: 14.0,
                            ..default()
                        },
                        TextColor(Color::srgb(0.8, 0.6, 0.4)), // Rust/amber color
                    ));

                    // Dialogue text
                    text_area.spawn((
                        DialogueContentText,
                        Text::new(""),
                        TextFont {
                            font_size: 16.0,
                            ..default()
                        },
                        TextColor(Color::srgb(0.9, 0.9, 0.85)),
                    ));
                });
        });

    info!("HUD spawned");
}

fn spawn_health_bar<M: Component>(parent: &mut ChildBuilder, marker: M, color: Color, label: &str) {
    parent
        .spawn(Node {
            width: Val::Px(200.0),
            height: Val::Px(12.0),
            flex_direction: FlexDirection::Row,
            align_items: AlignItems::Center,
            column_gap: Val::Px(5.0),
            ..default()
        })
        .with_children(|parent| {
            // Label
            parent.spawn((
                Text::new(label),
                TextFont {
                    font_size: 10.0,
                    ..default()
                },
                TextColor(color),
            ));

            // Bar background
            parent
                .spawn((
                    Node {
                        width: Val::Px(150.0),
                        height: Val::Px(8.0),
                        ..default()
                    },
                    BackgroundColor(Color::srgba(0.2, 0.2, 0.2, 0.8)),
                ))
                .with_children(|parent| {
                    // Bar fill
                    parent.spawn((
                        marker,
                        Node {
                            width: Val::Percent(100.0),
                            height: Val::Percent(100.0),
                            ..default()
                        },
                        BackgroundColor(color),
                    ));
                });
        });
}

fn update_score_display(
    score: Res<ScoreSystem>,
    mut query: Query<&mut Text, With<ScoreText>>,
) {
    for mut text in query.iter_mut() {
        **text = format!("SCORE: {}", score.score);
    }
}

fn update_combo_display(
    score: Res<ScoreSystem>,
    mut combo_query: Query<(&mut Text, &mut TextColor), (With<ComboText>, Without<GradeText>)>,
    mut grade_query: Query<(&mut Text, &mut TextColor), (With<GradeText>, Without<ComboText>)>,
) {
    for (mut text, mut color) in combo_query.iter_mut() {
        **text = format!("x{:.1}", score.multiplier);
        // Color based on multiplier
        color.0 = if score.multiplier >= 10.0 {
            Color::srgb(1.0, 0.3, 0.3)
        } else if score.multiplier >= 5.0 {
            Color::srgb(1.0, 0.6, 0.2)
        } else if score.multiplier >= 2.0 {
            Color::srgb(1.0, 0.9, 0.3)
        } else {
            Color::WHITE
        };
    }

    for (mut text, mut text_color) in grade_query.iter_mut() {
        let grade = score.get_grade();
        **text = grade.as_str().to_string();
        text_color.0 = grade.color();
    }
}

fn update_health_bars(
    player_query: Query<&ShipStats, With<Player>>,
    mut shield_query: Query<&mut Node, (With<ShieldBar>, Without<ArmorBar>, Without<HullBar>, Without<CapacitorBar>)>,
    mut armor_query: Query<&mut Node, (With<ArmorBar>, Without<ShieldBar>, Without<HullBar>, Without<CapacitorBar>)>,
    mut hull_query: Query<&mut Node, (With<HullBar>, Without<ShieldBar>, Without<ArmorBar>, Without<CapacitorBar>)>,
    mut cap_query: Query<&mut Node, (With<CapacitorBar>, Without<ShieldBar>, Without<ArmorBar>, Without<HullBar>)>,
) {
    let Ok(stats) = player_query.get_single() else {
        return;
    };

    for mut node in shield_query.iter_mut() {
        node.width = Val::Percent((stats.shield / stats.max_shield * 100.0).max(0.0));
    }
    for mut node in armor_query.iter_mut() {
        node.width = Val::Percent((stats.armor / stats.max_armor * 100.0).max(0.0));
    }
    for mut node in hull_query.iter_mut() {
        node.width = Val::Percent((stats.hull / stats.max_hull * 100.0).max(0.0));
    }
    for mut node in cap_query.iter_mut() {
        node.width = Val::Percent((stats.capacitor / stats.max_capacitor * 100.0).max(0.0));
    }
}

fn update_berserk_meter(
    berserk: Res<BerserkSystem>,
    mut query: Query<(&mut Node, &mut BackgroundColor), With<BerserkBar>>,
) {
    for (mut node, mut bg) in query.iter_mut() {
        if berserk.is_active {
            // Pulsing effect when active - show remaining time
            let pulse = (berserk.timer * 10.0).sin().abs();
            node.width = Val::Percent(berserk.progress() * 100.0);
            bg.0 = Color::srgb(0.8 + pulse * 0.2, 0.2, 0.8 + pulse * 0.2);
        } else {
            // Show proximity kills progress toward berserk
            node.width = Val::Percent(berserk.progress() * 100.0);
            bg.0 = Color::srgb(0.8, 0.2, 0.8);
        }
    }
}

/// Update heat display bar
fn update_heat_display(
    heat_system: Res<ComboHeatSystem>,
    mut query: Query<(&mut Node, &mut BackgroundColor), With<HeatBar>>,
) {
    for (mut node, mut bg) in query.iter_mut() {
        node.width = Val::Percent(heat_system.heat);
        // Color changes with heat level
        bg.0 = heat_system.heat_level.color();
    }
}

/// Update combo kills display
fn update_combo_kills(
    heat_system: Res<ComboHeatSystem>,
    mut query: Query<&mut Text, With<ComboKillsText>>,
) {
    for mut text in query.iter_mut() {
        if let Some(tier_name) = heat_system.combo_tier_name() {
            **text = format!("{} x{}", tier_name, heat_system.combo_count);
        } else if heat_system.combo_count > 0 {
            **text = format!("{}x", heat_system.combo_count);
        } else {
            **text = String::new();
        }
    }
}

/// Update wave display (with stage info)
fn update_wave_display(
    campaign: Res<CampaignState>,
    mut query: Query<&mut Text, With<WaveText>>,
) {
    for mut text in query.iter_mut() {
        if let Some(mission) = campaign.current_mission() {
            if campaign.is_boss_wave() {
                **text = format!("WAVE {}/{} - BOSS", campaign.current_wave, mission.enemy_waves + 1);
            } else {
                **text = format!("WAVE {}/{}", campaign.current_wave, mission.enemy_waves + 1);
            }
        } else {
            **text = format!("WAVE {}", campaign.current_wave);
        }
    }
}

/// Update mission info display
fn update_mission_display(
    campaign: Res<CampaignState>,
    score: Res<ScoreSystem>,
    mut mission_query: Query<&mut Text, (With<MissionNameText>, Without<ObjectiveText>, Without<SoulsText>)>,
    mut objective_query: Query<(&mut Text, &mut TextColor), (With<ObjectiveText>, Without<MissionNameText>, Without<SoulsText>)>,
    mut souls_query: Query<&mut Text, (With<SoulsText>, Without<MissionNameText>, Without<ObjectiveText>)>,
) {
    // Update mission name
    for mut text in mission_query.iter_mut() {
        if let Some(mission) = campaign.current_mission() {
            **text = format!("M{}: {} - {}", campaign.mission_number(), mission.name, campaign.act.name());
        } else {
            **text = String::new();
        }
    }

    // Update objective
    for (mut text, mut color) in objective_query.iter_mut() {
        if let Some(mission) = campaign.current_mission() {
            if campaign.primary_complete {
                **text = format!("✓ {}", mission.primary_objective);
                color.0 = Color::srgb(0.3, 1.0, 0.3); // Bright green when complete
            } else {
                **text = format!("◯ {}", mission.primary_objective);
                color.0 = Color::srgb(0.5, 0.8, 0.5); // Dim green when incomplete
            }
        } else {
            **text = String::new();
        }
    }

    // Update souls liberated
    for mut text in souls_query.iter_mut() {
        if campaign.in_mission {
            let bonus = if let Some(mission) = campaign.current_mission() {
                if campaign.mission_souls >= mission.souls_to_liberate {
                    " ✓"
                } else {
                    ""
                }
            } else {
                ""
            };
            **text = format!("SOULS LIBERATED: {}{}", score.souls_liberated, bonus);
        } else {
            **text = String::new();
        }
    }
}

/// Update powerup effect indicators
fn update_powerup_indicators(
    player_query: Query<&PowerupEffects, With<Player>>,
    mut overdrive_query: Query<&mut Text, (With<OverdriveIndicator>, Without<DamageBoostIndicator>, Without<InvulnIndicator>)>,
    mut damage_query: Query<&mut Text, (With<DamageBoostIndicator>, Without<OverdriveIndicator>, Without<InvulnIndicator>)>,
    mut invuln_query: Query<&mut Text, (With<InvulnIndicator>, Without<OverdriveIndicator>, Without<DamageBoostIndicator>)>,
) {
    let Ok(effects) = player_query.get_single() else {
        return;
    };

    for mut text in overdrive_query.iter_mut() {
        if effects.overdrive_timer > 0.0 {
            **text = format!("OVERDRIVE {:.1}s", effects.overdrive_timer);
        } else {
            **text = String::new();
        }
    }

    for mut text in damage_query.iter_mut() {
        if effects.damage_boost_timer > 0.0 {
            **text = format!("DAMAGE x2 {:.1}s", effects.damage_boost_timer);
        } else {
            **text = String::new();
        }
    }

    for mut text in invuln_query.iter_mut() {
        if effects.invuln_timer > 0.0 {
            **text = format!("INVULNERABLE {:.1}s", effects.invuln_timer);
        } else {
            **text = String::new();
        }
    }
}

/// Update boss health bar
fn update_boss_health_bar(
    boss_query: Query<(&BossData, &BossState), With<Boss>>,
    mut container_query: Query<&mut Node, With<BossHealthContainer>>,
    mut fill_query: Query<&mut Node, (With<BossHealthFill>, Without<BossHealthContainer>)>,
    mut name_query: Query<&mut Text, With<BossNameText>>,
) {
    let has_boss = boss_query.get_single().is_ok();

    // Show/hide boss health bar
    for mut node in container_query.iter_mut() {
        node.display = if has_boss { Display::Flex } else { Display::None };
    }

    if let Ok((data, state)) = boss_query.get_single() {
        // Update health bar fill
        for mut node in fill_query.iter_mut() {
            let health_percent = (data.health / data.max_health * 100.0).max(0.0);
            node.width = Val::Percent(health_percent);
        }

        // Update boss name
        for mut text in name_query.iter_mut() {
            let phase_info = if data.total_phases > 1 {
                format!(" (Phase {}/{})", data.current_phase, data.total_phases)
            } else {
                String::new()
            };

            match *state {
                BossState::Intro => {
                    **text = format!("{} - {}", data.name, data.title);
                }
                BossState::Battle | BossState::PhaseTransition => {
                    **text = format!("{}{}", data.name, phase_info);
                }
                BossState::Defeated => {
                    **text = format!("{} DEFEATED!", data.name);
                }
            }
        }
    }
}

/// Update dialogue display based on DialogueSystem state
fn update_dialogue_display(
    dialogue_system: Res<DialogueSystem>,
    mut container_query: Query<&mut Node, With<DialogueContainer>>,
    mut speaker_query: Query<&mut Text, (With<DialogueSpeakerText>, Without<DialogueContentText>)>,
    mut content_query: Query<&mut Text, (With<DialogueContentText>, Without<DialogueSpeakerText>)>,
) {
    let is_active = dialogue_system.is_active();

    // Show/hide dialogue container
    for mut node in container_query.iter_mut() {
        node.display = if is_active { Display::Flex } else { Display::None };
    }

    if let Some(text) = &dialogue_system.active_text {
        // Update speaker name
        for mut speaker in speaker_query.iter_mut() {
            **speaker = dialogue_system.speaker.clone();
        }

        // Update dialogue content
        for mut content in content_query.iter_mut() {
            **content = text.clone();
        }
    }
}

/// Update wingman gauge (Rifter only)
fn update_wingman_gauge(
    tracker: Res<WingmanTracker>,
    selected_ship: Res<SelectedShip>,
    wingmen_query: Query<Entity, With<Wingman>>,
    mut gauge_query: Query<&mut Node, With<WingmanGauge>>,
    mut fill_query: Query<&mut Node, (With<WingmanGaugeFill>, Without<WingmanGauge>)>,
    mut count_query: Query<&mut Text, With<WingmanCountText>>,
) {
    let is_rifter = selected_ship.ship == MinmatarShip::Rifter;

    // Show/hide wingman gauge
    for mut node in gauge_query.iter_mut() {
        node.display = if is_rifter { Display::Flex } else { Display::None };
    }

    if !is_rifter {
        return;
    }

    // Update fill bar
    let progress = tracker.progress() * 100.0;
    for mut node in fill_query.iter_mut() {
        node.width = Val::Percent(progress);
    }

    // Update count text
    let wingman_count = wingmen_query.iter().count();
    for mut text in count_query.iter_mut() {
        **text = format!("{}/{} | Active: {}", tracker.kill_count, tracker.kills_per_wingman, wingman_count);
    }
}

fn despawn_hud(
    mut commands: Commands,
    hud_query: Query<Entity, With<HudRoot>>,
    dialogue_query: Query<Entity, With<DialogueContainer>>,
) {
    for entity in hud_query.iter() {
        commands.entity(entity).despawn_recursive();
    }
    for entity in dialogue_query.iter() {
        commands.entity(entity).despawn_recursive();
    }
}
