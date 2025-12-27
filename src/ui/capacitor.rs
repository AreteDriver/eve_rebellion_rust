//! EVE-Style Capacitor Wheel
//!
//! Circular status display with:
//! - Central capacitor "star" (radial yellow dashes)
//! - Concentric health rings: Shield (outer), Armor (middle), Hull (inner)
//! - Segmented arc display like EVE Online

use bevy::prelude::*;
use bevy_egui::{egui, EguiContexts};
use std::f32::consts::{PI, TAU};

use crate::core::*;
use crate::entities::{Player, ShipStats};
use crate::systems::ComboHeatSystem;

/// Capacitor wheel plugin
pub struct CapacitorWheelPlugin;

impl Plugin for CapacitorWheelPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(
            Update,
            draw_capacitor_wheel
                .run_if(in_state(GameState::Playing))
                .after(bevy_egui::EguiSet::ProcessInput),
        );
    }
}

/// Draw EVE-style capacitor wheel using egui
fn draw_capacitor_wheel(
    mut egui_ctx: EguiContexts,
    player_query: Query<&ShipStats, With<Player>>,
    heat_system: Res<ComboHeatSystem>,
    windows: Query<&Window>,
) {
    let Ok(stats) = player_query.get_single() else {
        return;
    };

    let Ok(window) = windows.get_single() else {
        return;
    };

    let Some(ctx) = egui_ctx.try_ctx_mut() else {
        return;
    };

    // Position at bottom center of screen
    let wheel_radius = 70.0;
    let center_x = window.width() / 2.0;
    let center_y = window.height() - 90.0;

    // Calculate percentages
    let shield_pct = (stats.shield / stats.max_shield).clamp(0.0, 1.0);
    let armor_pct = (stats.armor / stats.max_armor).clamp(0.0, 1.0);
    let hull_pct = (stats.hull / stats.max_hull).clamp(0.0, 1.0);
    let cap_pct = (stats.capacitor / stats.max_capacitor).clamp(0.0, 1.0);
    let heat_pct = heat_system.heat / 100.0;

    // Draw using egui Area (positioned overlay)
    egui::Area::new(egui::Id::new("capacitor_wheel"))
        .fixed_pos(egui::pos2(center_x - wheel_radius - 20.0, center_y - wheel_radius - 20.0))
        .show(ctx, |ui| {
            let size = egui::vec2((wheel_radius + 20.0) * 2.0, (wheel_radius + 20.0) * 2.0);
            let (response, painter) = ui.allocate_painter(size, egui::Sense::hover());
            let center = response.rect.center();

            // === BACKGROUND CIRCLE ===
            painter.circle_filled(center, wheel_radius + 5.0, egui::Color32::from_rgba_unmultiplied(15, 18, 25, 220));
            painter.circle_stroke(center, wheel_radius + 5.0, egui::Stroke::new(2.0, egui::Color32::from_rgb(40, 45, 55)));

            // === HEALTH RINGS (outer to inner: Shield -> Armor -> Hull) ===
            let ring_width = 8.0;
            let ring_gap = 3.0;

            // Shield ring (outermost) - Blue
            let shield_radius = wheel_radius;
            draw_segmented_ring(
                &painter, center, shield_radius, ring_width,
                shield_pct,
                egui::Color32::from_rgb(64, 156, 255),  // EVE shield blue
                egui::Color32::from_rgb(25, 35, 50),    // Dark background
                24, // segments
            );

            // Armor ring (middle) - Orange/Gold
            let armor_radius = wheel_radius - ring_width - ring_gap;
            draw_segmented_ring(
                &painter, center, armor_radius, ring_width,
                armor_pct,
                egui::Color32::from_rgb(255, 176, 48),  // EVE armor orange
                egui::Color32::from_rgb(50, 35, 20),    // Dark background
                20, // segments
            );

            // Hull/Structure ring (innermost) - Gray/White
            let hull_radius = armor_radius - ring_width - ring_gap;
            draw_segmented_ring(
                &painter, center, hull_radius, ring_width,
                hull_pct,
                egui::Color32::from_rgb(180, 185, 195), // EVE structure gray
                egui::Color32::from_rgb(35, 35, 40),    // Dark background
                16, // segments
            );

            // === CENTRAL CAPACITOR "STAR" ===
            let cap_radius = hull_radius - ring_width - ring_gap - 5.0;
            draw_capacitor_star(&painter, center, cap_radius, cap_pct, heat_pct);

            // === PERCENTAGE TEXT (left side) ===
            let text_x = center.x - wheel_radius - 45.0;
            let shield_color = if shield_pct < 0.25 { egui::Color32::RED } else { egui::Color32::from_rgb(64, 156, 255) };
            let armor_color = if armor_pct < 0.25 { egui::Color32::RED } else { egui::Color32::from_rgb(255, 176, 48) };
            let hull_color = if hull_pct < 0.25 { egui::Color32::RED } else { egui::Color32::from_rgb(180, 185, 195) };

            painter.text(
                egui::pos2(text_x, center.y - 20.0),
                egui::Align2::RIGHT_CENTER,
                format!("{:.0}%", shield_pct * 100.0),
                egui::FontId::proportional(11.0),
                shield_color,
            );
            painter.text(
                egui::pos2(text_x, center.y),
                egui::Align2::RIGHT_CENTER,
                format!("{:.0}%", armor_pct * 100.0),
                egui::FontId::proportional(11.0),
                armor_color,
            );
            painter.text(
                egui::pos2(text_x, center.y + 20.0),
                egui::Align2::RIGHT_CENTER,
                format!("{:.0}%", hull_pct * 100.0),
                egui::FontId::proportional(11.0),
                hull_color,
            );

            // === HEAT INDICATOR (top, if overheating) ===
            if heat_pct > 0.5 {
                let heat_color = if heat_pct > 0.75 {
                    egui::Color32::from_rgb(255, 80, 80)
                } else {
                    egui::Color32::from_rgb(255, 150, 50)
                };
                painter.text(
                    egui::pos2(center.x, center.y - wheel_radius - 15.0),
                    egui::Align2::CENTER_CENTER,
                    "⚠ HEAT",
                    egui::FontId::proportional(10.0),
                    heat_color,
                );
            }
        });
}

/// Draw a segmented ring like EVE Online
fn draw_segmented_ring(
    painter: &egui::Painter,
    center: egui::Pos2,
    radius: f32,
    width: f32,
    fill_pct: f32,
    fill_color: egui::Color32,
    empty_color: egui::Color32,
    num_segments: u32,
) {
    let segment_gap = 0.03; // radians
    let segment_arc = (TAU / num_segments as f32) - segment_gap;
    let start_angle = -PI / 2.0; // Start from top

    let filled_segments = (fill_pct * num_segments as f32).ceil() as u32;

    for i in 0..num_segments {
        let angle_start = start_angle + (i as f32) * (TAU / num_segments as f32);
        let is_filled = i < filled_segments;
        let color = if is_filled { fill_color } else { empty_color };

        draw_arc_segment(painter, center, radius, width, angle_start, segment_arc, color);
    }
}

/// Draw a single arc segment
fn draw_arc_segment(
    painter: &egui::Painter,
    center: egui::Pos2,
    radius: f32,
    width: f32,
    start_angle: f32,
    arc_span: f32,
    color: egui::Color32,
) {
    let steps = 8;
    let inner_r = radius - width / 2.0;
    let outer_r = radius + width / 2.0;

    let mut points = Vec::new();

    // Outer arc
    for i in 0..=steps {
        let t = i as f32 / steps as f32;
        let angle = start_angle + arc_span * t;
        points.push(egui::pos2(
            center.x + outer_r * angle.cos(),
            center.y + outer_r * angle.sin(),
        ));
    }

    // Inner arc (reversed)
    for i in (0..=steps).rev() {
        let t = i as f32 / steps as f32;
        let angle = start_angle + arc_span * t;
        points.push(egui::pos2(
            center.x + inner_r * angle.cos(),
            center.y + inner_r * angle.sin(),
        ));
    }

    if points.len() >= 3 {
        painter.add(egui::Shape::convex_polygon(points, color, egui::Stroke::NONE));
    }
}

/// Draw the central capacitor "star" pattern
fn draw_capacitor_star(
    painter: &egui::Painter,
    center: egui::Pos2,
    radius: f32,
    cap_pct: f32,
    heat_pct: f32,
) {
    let num_rays = 12;
    let ray_length = radius * 0.7;
    let ray_width = 3.0;

    // Capacitor color - yellow/orange when full, gray when depleted
    // Redder when overheating
    let base_color = if heat_pct > 0.75 {
        egui::Color32::from_rgb(255, 100, 50) // Red-orange when hot
    } else {
        egui::Color32::from_rgb(255, 200, 80) // Yellow-gold normal
    };
    let empty_color = egui::Color32::from_rgb(60, 55, 50);

    let filled_rays = (cap_pct * num_rays as f32).ceil() as u32;

    for i in 0..num_rays {
        let angle = (i as f32 / num_rays as f32) * TAU - PI / 2.0;
        let is_filled = i < filled_rays;
        let color = if is_filled { base_color } else { empty_color };

        // Each ray is a small rectangle pointing outward
        let inner_dist = radius * 0.25;
        let outer_dist = inner_dist + ray_length * cap_pct.max(0.2);

        let cos_a = angle.cos();
        let sin_a = angle.sin();
        let perp_cos = (angle + PI / 2.0).cos();
        let perp_sin = (angle + PI / 2.0).sin();

        let half_width = ray_width / 2.0;

        let points = vec![
            egui::pos2(
                center.x + cos_a * inner_dist - perp_cos * half_width,
                center.y + sin_a * inner_dist - perp_sin * half_width,
            ),
            egui::pos2(
                center.x + cos_a * inner_dist + perp_cos * half_width,
                center.y + sin_a * inner_dist + perp_sin * half_width,
            ),
            egui::pos2(
                center.x + cos_a * outer_dist + perp_cos * half_width * 0.5,
                center.y + sin_a * outer_dist + perp_sin * half_width * 0.5,
            ),
            egui::pos2(
                center.x + cos_a * outer_dist - perp_cos * half_width * 0.5,
                center.y + sin_a * outer_dist - perp_sin * half_width * 0.5,
            ),
        ];

        painter.add(egui::Shape::convex_polygon(points, color, egui::Stroke::NONE));
    }

    // Center dot
    painter.circle_filled(center, 4.0, egui::Color32::from_rgb(40, 45, 55));
}
