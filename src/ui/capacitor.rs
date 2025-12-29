//! EVE-Style Capacitor Wheel
//!
//! Exact replica of EVE Online's HUD capacitor display:
//! - Three concentric semicircular health arcs (Shield/Armor/Structure)
//! - Central animated capacitor "sun" with radial dashes
//! - Speed display at bottom
//! - Percentage readouts on left

use bevy::prelude::*;
use bevy_egui::{egui, EguiContexts};
use std::f32::consts::PI;

use crate::core::*;
use crate::entities::{Movement, Player, ShipStats};
use crate::systems::ComboHeatSystem;

/// Capacitor wheel plugin
pub struct CapacitorWheelPlugin;

impl Plugin for CapacitorWheelPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<CapacitorAnimation>().add_systems(
            Update,
            (update_capacitor_animation, draw_capacitor_wheel)
                .chain()
                .run_if(in_state(GameState::Playing))
                .after(bevy_egui::EguiSet::ProcessInput),
        );
    }
}

/// Animation state for capacitor effects
#[derive(Resource)]
pub struct CapacitorAnimation {
    pub rotation: f32,
    pub pulse: f32,
    pub pulse_direction: f32,
}

impl Default for CapacitorAnimation {
    fn default() -> Self {
        Self {
            rotation: 0.0,
            pulse: 1.0,
            pulse_direction: 1.0,
        }
    }
}

/// Update capacitor animation
fn update_capacitor_animation(time: Res<Time>, mut anim: ResMut<CapacitorAnimation>) {
    let dt = time.delta_secs();

    // Slow rotation for the capacitor star
    anim.rotation += dt * 0.3;
    if anim.rotation > PI * 2.0 {
        anim.rotation -= PI * 2.0;
    }

    // Pulsing effect
    anim.pulse += anim.pulse_direction * dt * 0.8;
    if anim.pulse > 1.2 {
        anim.pulse = 1.2;
        anim.pulse_direction = -1.0;
    } else if anim.pulse < 0.8 {
        anim.pulse = 0.8;
        anim.pulse_direction = 1.0;
    }
}

/// Draw EVE-style capacitor wheel using egui
fn draw_capacitor_wheel(
    mut egui_ctx: EguiContexts,
    player_query: Query<(&ShipStats, Option<&Movement>), With<Player>>,
    heat_system: Res<ComboHeatSystem>,
    anim: Res<CapacitorAnimation>,
    windows: Query<&Window>,
) {
    let Ok((stats, movement)) = player_query.get_single() else {
        return;
    };

    let Ok(window) = windows.get_single() else {
        return;
    };

    let Some(ctx) = egui_ctx.try_ctx_mut() else {
        return;
    };

    // Position at bottom center of screen (60% size of original)
    let wheel_radius = 50.0; // Reduced from 85.0
    let center_x = window.width() / 2.0;
    let center_y = window.height() - 65.0;

    // Calculate percentages
    let shield_pct = (stats.shield / stats.max_shield).clamp(0.0, 1.0);
    let armor_pct = (stats.armor / stats.max_armor).clamp(0.0, 1.0);
    let hull_pct = (stats.hull / stats.max_hull).clamp(0.0, 1.0);
    let cap_pct = (stats.capacitor / stats.max_capacitor).clamp(0.0, 1.0);
    let heat_pct = heat_system.heat / 100.0;

    // Get speed
    let speed = movement.map(|m| m.velocity.length()).unwrap_or(0.0);

    // Draw using egui Area (positioned overlay)
    egui::Area::new(egui::Id::new("capacitor_wheel"))
        .fixed_pos(egui::pos2(
            center_x - wheel_radius - 40.0,
            center_y - wheel_radius - 20.0,
        ))
        .show(ctx, |ui| {
            let size = egui::vec2((wheel_radius + 40.0) * 2.0, (wheel_radius + 35.0) * 2.0);
            let (response, painter) = ui.allocate_painter(size, egui::Sense::hover());
            let center = egui::pos2(response.rect.center().x, response.rect.center().y - 5.0);

            // === DARK BACKGROUND CIRCLE ===
            painter.circle_filled(
                center,
                wheel_radius + 5.0,
                egui::Color32::from_rgba_unmultiplied(15, 18, 25, 245),
            );
            painter.circle_stroke(
                center,
                wheel_radius + 5.0,
                egui::Stroke::new(1.0, egui::Color32::from_rgb(45, 50, 60)),
            );

            // === INNER DARK CIRCLE (for speed display) ===
            let inner_radius = 20.0;
            painter.circle_filled(
                center,
                inner_radius,
                egui::Color32::from_rgba_unmultiplied(25, 32, 45, 230),
            );
            painter.circle_stroke(
                center,
                inner_radius,
                egui::Stroke::new(0.8, egui::Color32::from_rgb(55, 65, 80)),
            );

            // === HEALTH ARCS (semicircular at top, EVE style) ===
            // The arcs span from left to right over the top (like EVE)
            // Shield = outermost, Armor = middle, Structure = innermost

            let arc_width = 6.0;
            let arc_gap = 2.5;
            let arc_start = -PI; // Start from left
            let arc_end = 0.0; // End at right (top semicircle)

            // Shield arc (outermost) - grayish-white
            let shield_radius = wheel_radius - 3.0;
            draw_eve_health_arc(
                &painter,
                center,
                shield_radius,
                arc_width,
                shield_pct,
                arc_start,
                arc_end,
                egui::Color32::from_rgb(200, 210, 220), // Filled
                egui::Color32::from_rgb(50, 55, 65),    // Empty
                22,                                     // segments
            );

            // Armor arc (middle) - grayish-white
            let armor_radius = shield_radius - arc_width - arc_gap;
            draw_eve_health_arc(
                &painter,
                center,
                armor_radius,
                arc_width,
                armor_pct,
                arc_start,
                arc_end,
                egui::Color32::from_rgb(190, 195, 205),
                egui::Color32::from_rgb(45, 50, 60),
                18,
            );

            // Structure arc (innermost) - grayish-white
            let structure_radius = armor_radius - arc_width - arc_gap;
            draw_eve_health_arc(
                &painter,
                center,
                structure_radius,
                arc_width,
                hull_pct,
                arc_start,
                arc_end,
                egui::Color32::from_rgb(180, 185, 195),
                egui::Color32::from_rgb(40, 45, 55),
                14,
            );

            // === CENTRAL CAPACITOR "SUN" ===
            draw_capacitor_sun(
                &painter,
                center,
                inner_radius - 3.0,
                cap_pct,
                heat_pct,
                anim.rotation,
                anim.pulse,
            );

            // === PERCENTAGE TEXT (left side, stacked vertically) ===
            let text_x = center.x - wheel_radius - 22.0;
            let text_y_start = center.y - 22.0;
            let text_spacing = 12.0;

            // Shield %
            let shield_color = if shield_pct < 0.25 {
                egui::Color32::from_rgb(255, 100, 100)
            } else {
                egui::Color32::from_rgb(160, 170, 180)
            };
            painter.text(
                egui::pos2(text_x, text_y_start),
                egui::Align2::RIGHT_CENTER,
                format!("{:.0}%", shield_pct * 100.0),
                egui::FontId::monospace(9.0),
                shield_color,
            );

            // Armor %
            let armor_color = if armor_pct < 0.25 {
                egui::Color32::from_rgb(255, 100, 100)
            } else {
                egui::Color32::from_rgb(160, 170, 180)
            };
            painter.text(
                egui::pos2(text_x, text_y_start + text_spacing),
                egui::Align2::RIGHT_CENTER,
                format!("{:.0}%", armor_pct * 100.0),
                egui::FontId::monospace(9.0),
                armor_color,
            );

            // Hull %
            let hull_color = if hull_pct < 0.25 {
                egui::Color32::from_rgb(255, 100, 100)
            } else {
                egui::Color32::from_rgb(160, 170, 180)
            };
            painter.text(
                egui::pos2(text_x, text_y_start + text_spacing * 2.0),
                egui::Align2::RIGHT_CENTER,
                format!("{:.0}%", hull_pct * 100.0),
                egui::FontId::monospace(9.0),
                hull_color,
            );

            // === SPEED DISPLAY (bottom center) ===
            painter.text(
                egui::pos2(center.x, center.y + wheel_radius + 10.0),
                egui::Align2::CENTER_CENTER,
                format!("{:.0} m/s", speed),
                egui::FontId::monospace(9.0),
                egui::Color32::from_rgb(90, 160, 200),
            );

            // === HEAT WARNING (if overheating) ===
            if heat_pct > 0.6 {
                let heat_alpha = ((heat_pct - 0.6) * 2.5 * 255.0) as u8;
                let heat_color = egui::Color32::from_rgba_unmultiplied(255, 80, 50, heat_alpha);
                painter.text(
                    egui::pos2(center.x, center.y - wheel_radius - 10.0),
                    egui::Align2::CENTER_CENTER,
                    "HEAT",
                    egui::FontId::proportional(8.0),
                    heat_color,
                );
            }

            // === -/+ SPEED INDICATORS (bottom left/right) ===
            let indicator_y = center.y + 5.0;
            painter.text(
                egui::pos2(center.x - inner_radius - 10.0, indicator_y),
                egui::Align2::CENTER_CENTER,
                "−",
                egui::FontId::proportional(12.0),
                egui::Color32::from_rgb(120, 130, 140),
            );
            painter.text(
                egui::pos2(center.x + inner_radius + 10.0, indicator_y),
                egui::Align2::CENTER_CENTER,
                "+",
                egui::FontId::proportional(12.0),
                egui::Color32::from_rgb(120, 130, 140),
            );
        });
}

/// Draw EVE-style health arc (semicircular, segmented)
fn draw_eve_health_arc(
    painter: &egui::Painter,
    center: egui::Pos2,
    radius: f32,
    width: f32,
    fill_pct: f32,
    arc_start: f32,
    arc_end: f32,
    fill_color: egui::Color32,
    empty_color: egui::Color32,
    num_segments: u32,
) {
    let segment_gap = 0.025; // Gap between segments in radians
    let total_arc = arc_end - arc_start;
    let segment_arc = (total_arc / num_segments as f32) - segment_gap;

    // Calculate how many segments should be filled
    // In EVE, segments fill from the edges toward center (both sides)
    let filled_segments = (fill_pct * num_segments as f32).ceil() as u32;

    for i in 0..num_segments {
        let angle_start = arc_start + (i as f32) * (total_arc / num_segments as f32);

        // Determine if this segment should be filled
        // EVE fills from both edges toward the middle
        let half = num_segments / 2;
        let is_filled = if i < half {
            i < filled_segments / 2
        } else {
            (num_segments - i - 1) < filled_segments.div_ceil(2)
        };

        let color = if is_filled || fill_pct >= 1.0 {
            fill_color
        } else {
            empty_color
        };

        draw_arc_segment(
            painter,
            center,
            radius,
            width,
            angle_start,
            segment_arc,
            color,
        );
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
    let steps = 6;
    let inner_r = radius - width / 2.0;
    let outer_r = radius + width / 2.0;

    let mut points = Vec::with_capacity((steps + 1) * 2);

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
        painter.add(egui::Shape::convex_polygon(
            points,
            color,
            egui::Stroke::NONE,
        ));
    }
}

/// Draw the central capacitor "sun" pattern (animated)
fn draw_capacitor_sun(
    painter: &egui::Painter,
    center: egui::Pos2,
    radius: f32,
    cap_pct: f32,
    heat_pct: f32,
    rotation: f32,
    pulse: f32,
) {
    let num_rays = 12;
    let ray_length = radius * 0.7;
    let ray_width = 1.8;

    // Capacitor color - golden/orange when full, dims when low
    // Gets more red when overheating
    let base_r = if heat_pct > 0.5 {
        255
    } else {
        (255.0 * (0.8 + cap_pct * 0.2)) as u8
    };
    let base_g = if heat_pct > 0.5 {
        (180.0 * (1.0 - heat_pct * 0.5)) as u8
    } else {
        (160.0 + cap_pct * 40.0) as u8
    };
    let base_b = if heat_pct > 0.5 {
        50
    } else {
        (60.0 + cap_pct * 20.0) as u8
    };

    let filled_rays = (cap_pct * num_rays as f32).ceil() as u32;

    for i in 0..num_rays {
        let base_angle = (i as f32 / num_rays as f32) * PI * 2.0 - PI / 2.0;
        let angle = base_angle + rotation;

        let is_filled = i < filled_rays;

        // Apply pulse to brightness
        let brightness = if is_filled { pulse } else { 0.3 };
        let color = egui::Color32::from_rgb(
            (base_r as f32 * brightness).min(255.0) as u8,
            (base_g as f32 * brightness).min(255.0) as u8,
            (base_b as f32 * brightness).min(255.0) as u8,
        );

        // Inner and outer distance for the ray
        let inner_dist = radius * 0.25;
        let ray_len = if is_filled {
            ray_length * (0.7 + cap_pct * 0.3)
        } else {
            ray_length * 0.5
        };
        let outer_dist = inner_dist + ray_len;

        let cos_a = angle.cos();
        let sin_a = angle.sin();
        let perp_cos = (angle + PI / 2.0).cos();
        let perp_sin = (angle + PI / 2.0).sin();

        let half_width = ray_width / 2.0;

        // Create tapered ray (wider at base, narrower at tip)
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
                center.x + cos_a * outer_dist + perp_cos * half_width * 0.3,
                center.y + sin_a * outer_dist + perp_sin * half_width * 0.3,
            ),
            egui::pos2(
                center.x + cos_a * outer_dist - perp_cos * half_width * 0.3,
                center.y + sin_a * outer_dist - perp_sin * half_width * 0.3,
            ),
        ];

        painter.add(egui::Shape::convex_polygon(
            points,
            color,
            egui::Stroke::NONE,
        ));
    }

    // Center dot (dark)
    painter.circle_filled(center, 3.0, egui::Color32::from_rgb(25, 30, 40));
}
