//! EVE-Style HUD Wheel
//!
//! EVE Online-inspired HUD display:
//! - Three concentric semicircular health arcs (Shield/Armor/Structure)
//! - Central HEAT gauge with radial spoke pattern (fills as heat builds)
//! - Speed display at bottom center
//! - Percentage readouts on left
//! - Heat status indicators

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

    // Very slow rotation for capacitor glow effect
    anim.rotation += dt * 0.15;
    if anim.rotation > PI * 2.0 {
        anim.rotation -= PI * 2.0;
    }

    // Subtle pulsing
    anim.pulse += anim.pulse_direction * dt * 0.5;
    if anim.pulse > 1.1 {
        anim.pulse = 1.1;
        anim.pulse_direction = -1.0;
    } else if anim.pulse < 0.9 {
        anim.pulse = 0.9;
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

    // Position at bottom center of screen
    let wheel_radius = 55.0;
    let center_x = window.width() / 2.0;
    let center_y = window.height() - 70.0;

    // Calculate percentages
    let shield_pct = (stats.shield / stats.max_shield).clamp(0.0, 1.0);
    let armor_pct = (stats.armor / stats.max_armor).clamp(0.0, 1.0);
    let hull_pct = (stats.hull / stats.max_hull).clamp(0.0, 1.0);
    let cap_pct = (stats.capacitor / stats.max_capacitor).clamp(0.0, 1.0);
    let heat_pct = heat_system.heat / 100.0;

    // Get speed
    let speed = movement.map(|m| m.velocity.length()).unwrap_or(0.0);

    // Draw using egui Area
    egui::Area::new(egui::Id::new("capacitor_wheel"))
        .fixed_pos(egui::pos2(
            center_x - wheel_radius - 45.0,
            center_y - wheel_radius - 25.0,
        ))
        .show(ctx, |ui| {
            let size = egui::vec2((wheel_radius + 50.0) * 2.0, (wheel_radius + 40.0) * 2.0);
            let (response, painter) = ui.allocate_painter(size, egui::Sense::hover());
            let center = egui::pos2(response.rect.center().x, response.rect.center().y - 5.0);

            // === OUTER SENSOR OVERLAY RING ===
            painter.circle_stroke(
                center,
                wheel_radius + 8.0,
                egui::Stroke::new(1.0, egui::Color32::from_rgb(35, 40, 50)),
            );

            // === MAIN DARK BACKGROUND ===
            painter.circle_filled(
                center,
                wheel_radius + 3.0,
                egui::Color32::from_rgba_unmultiplied(12, 15, 22, 250),
            );
            painter.circle_stroke(
                center,
                wheel_radius + 3.0,
                egui::Stroke::new(1.5, egui::Color32::from_rgb(40, 45, 55)),
            );

            // === HEALTH ARCS (top semicircle, EVE style) ===
            let arc_width = 7.0;
            let arc_gap = 2.0;
            let arc_start = -PI; // Left
            let arc_end = 0.0; // Right (top semicircle)

            // Shield arc (outermost) - grayish white
            let shield_radius = wheel_radius - 2.0;
            draw_eve_health_arc(
                &painter,
                center,
                shield_radius,
                arc_width,
                shield_pct,
                arc_start,
                arc_end,
                egui::Color32::from_rgb(210, 215, 225), // Filled - bright white-gray
                egui::Color32::from_rgb(35, 40, 50),    // Empty - dark
                24,
            );

            // Armor arc (middle)
            let armor_radius = shield_radius - arc_width - arc_gap;
            draw_eve_health_arc(
                &painter,
                center,
                armor_radius,
                arc_width,
                armor_pct,
                arc_start,
                arc_end,
                egui::Color32::from_rgb(195, 200, 210),
                egui::Color32::from_rgb(32, 37, 47),
                20,
            );

            // Structure/Hull arc (innermost)
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
                egui::Color32::from_rgb(28, 33, 43),
                16,
            );

            // === CAPACITOR RINGS (concentric circles of dashes) ===
            let cap_inner_radius = 18.0;
            let cap_outer_radius = structure_radius - arc_width - 5.0;
            draw_capacitor_rings(
                &painter,
                center,
                cap_inner_radius,
                cap_outer_radius,
                cap_pct,
                heat_pct,
                anim.pulse,
            );

            // === INNER SPEED DISPLAY CIRCLE ===
            painter.circle_filled(
                center,
                cap_inner_radius,
                egui::Color32::from_rgba_unmultiplied(20, 28, 40, 240),
            );
            painter.circle_stroke(
                center,
                cap_inner_radius,
                egui::Stroke::new(1.0, egui::Color32::from_rgb(50, 60, 75)),
            );

            // === OVERHEATING STATUS (orange indicators above capacitor) ===
            if heat_pct > 0.0 {
                draw_heat_indicators(&painter, center, wheel_radius, heat_pct);
            }

            // === PERCENTAGE TEXT (left side, stacked) ===
            let text_x = center.x - wheel_radius - 28.0;
            let text_y_start = center.y - 25.0;
            let text_spacing = 14.0;

            // Shield %
            let shield_color = health_text_color(shield_pct);
            painter.text(
                egui::pos2(text_x, text_y_start),
                egui::Align2::RIGHT_CENTER,
                format!("{:.0}%", shield_pct * 100.0),
                egui::FontId::monospace(10.0),
                shield_color,
            );

            // Armor %
            let armor_color = health_text_color(armor_pct);
            painter.text(
                egui::pos2(text_x, text_y_start + text_spacing),
                egui::Align2::RIGHT_CENTER,
                format!("{:.0}%", armor_pct * 100.0),
                egui::FontId::monospace(10.0),
                armor_color,
            );

            // Hull %
            let hull_color = health_text_color(hull_pct);
            painter.text(
                egui::pos2(text_x, text_y_start + text_spacing * 2.0),
                egui::Align2::RIGHT_CENTER,
                format!("{:.0}%", hull_pct * 100.0),
                egui::FontId::monospace(10.0),
                hull_color,
            );

            // === SPEED DISPLAY (center) ===
            painter.text(
                egui::pos2(center.x, center.y + 2.0),
                egui::Align2::CENTER_CENTER,
                format!("{:.0}", speed),
                egui::FontId::monospace(11.0),
                egui::Color32::from_rgb(100, 170, 210),
            );
            painter.text(
                egui::pos2(center.x, center.y + 14.0),
                egui::Align2::CENTER_CENTER,
                "m/s",
                egui::FontId::monospace(7.0),
                egui::Color32::from_rgb(70, 100, 130),
            );

            // === -/+ SPEED CONTROL INDICATORS ===
            let ctrl_y = center.y + wheel_radius + 12.0;

            // Minus button (left)
            painter.circle_filled(
                egui::pos2(center.x - 22.0, ctrl_y),
                8.0,
                egui::Color32::from_rgb(30, 35, 45),
            );
            painter.circle_stroke(
                egui::pos2(center.x - 22.0, ctrl_y),
                8.0,
                egui::Stroke::new(1.0, egui::Color32::from_rgb(55, 65, 80)),
            );
            painter.text(
                egui::pos2(center.x - 22.0, ctrl_y),
                egui::Align2::CENTER_CENTER,
                "−",
                egui::FontId::proportional(14.0),
                egui::Color32::from_rgb(140, 150, 165),
            );

            // Plus button (right)
            painter.circle_filled(
                egui::pos2(center.x + 22.0, ctrl_y),
                8.0,
                egui::Color32::from_rgb(30, 35, 45),
            );
            painter.circle_stroke(
                egui::pos2(center.x + 22.0, ctrl_y),
                8.0,
                egui::Stroke::new(1.0, egui::Color32::from_rgb(55, 65, 80)),
            );
            painter.text(
                egui::pos2(center.x + 22.0, ctrl_y),
                egui::Align2::CENTER_CENTER,
                "+",
                egui::FontId::proportional(14.0),
                egui::Color32::from_rgb(140, 150, 165),
            );

            // Speed value below center
            painter.text(
                egui::pos2(center.x, ctrl_y),
                egui::Align2::CENTER_CENTER,
                format!("{:.0} m/s", speed),
                egui::FontId::monospace(9.0),
                egui::Color32::from_rgb(80, 140, 180),
            );
        });
}

/// Get text color based on health percentage
fn health_text_color(pct: f32) -> egui::Color32 {
    if pct < 0.20 {
        egui::Color32::from_rgb(255, 80, 80) // Critical - red
    } else if pct < 0.35 {
        egui::Color32::from_rgb(255, 180, 80) // Warning - orange
    } else {
        egui::Color32::from_rgb(170, 180, 190) // Normal - gray-white
    }
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
    let segment_gap = 0.02;
    let total_arc = arc_end - arc_start;
    let segment_arc = (total_arc / num_segments as f32) - segment_gap;

    // EVE fills segments from edges toward center
    let filled_segments = (fill_pct * num_segments as f32).ceil() as u32;

    for i in 0..num_segments {
        let angle_start = arc_start + (i as f32) * (total_arc / num_segments as f32);

        // Fill from both edges toward middle
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
    let steps = 8;
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

/// Draw HEAT display as radial spoke gauges (EVE Online style)
/// Empty = cool, Filled = hot
/// Color shifts: teal (cool) → golden (warm) → red (overheating)
fn draw_capacitor_rings(
    painter: &egui::Painter,
    center: egui::Pos2,
    inner_radius: f32,
    outer_radius: f32,
    _cap_pct: f32, // Unused - kept for API compatibility
    heat_pct: f32,
    pulse: f32,
) {
    // Radial spoke pattern - rectangular gauges arranged like wheel spokes
    let num_layers = 3;
    let gauges_per_layer = 16;
    let total_gauges = num_layers * gauges_per_layer;

    // Heat fills from inside-out (heat builds up from core)
    let filled_gauges = (heat_pct * total_gauges as f32).round() as u32;

    // Heat color gradient: teal → golden → orange → red
    let heat_color = if heat_pct < 0.3 {
        // Cool - teal/cyan
        let t = heat_pct / 0.3;
        egui::Color32::from_rgb(
            (60.0 + 140.0 * t) as u8,   // 60 → 200
            (180.0 - 30.0 * t) as u8,   // 180 → 150
            (200.0 - 100.0 * t) as u8,  // 200 → 100
        )
    } else if heat_pct < 0.6 {
        // Warm - golden/yellow
        let t = (heat_pct - 0.3) / 0.3;
        egui::Color32::from_rgb(
            (200.0 + 55.0 * t) as u8,  // 200 → 255
            (150.0 + 30.0 * t) as u8,  // 150 → 180
            (100.0 - 50.0 * t) as u8,  // 100 → 50
        )
    } else if heat_pct < 0.85 {
        // Hot - orange
        let t = (heat_pct - 0.6) / 0.25;
        egui::Color32::from_rgb(
            255,
            (180.0 - 80.0 * t) as u8,  // 180 → 100
            (50.0 - 30.0 * t) as u8,   // 50 → 20
        )
    } else {
        // Critical - red (pulsing)
        let critical_pulse = 0.7 + 0.3 * ((pulse - 0.9) * 5.0).sin().abs();
        egui::Color32::from_rgb(
            255,
            (60.0 * critical_pulse) as u8,
            (30.0 * critical_pulse) as u8,
        )
    };

    let empty_color = egui::Color32::from_rgb(20, 35, 45); // Dark teal-gray (cool look)
    let border_color = egui::Color32::from_rgb(40, 60, 75);

    // Layer spacing
    let layer_height = (outer_radius - inner_radius - 4.0) / num_layers as f32;
    let gauge_gap = 2.0;
    let gauge_height = layer_height - gauge_gap;

    // Draw gauges - fill from inside-out (heat builds from core)
    let mut gauge_index = 0;

    // Inner layers first (heat builds from inside out)
    for layer in 0..num_layers {
        let layer_inner = inner_radius + 2.0 + layer as f32 * layer_height;
        let layer_outer = layer_inner + gauge_height;

        for i in 0..gauges_per_layer {
            // Start from top (-PI/2), go clockwise
            let angle = -PI / 2.0 + (i as f32 / gauges_per_layer as f32) * PI * 2.0;
            let gauge_arc = (PI * 2.0 / gauges_per_layer as f32) * 0.75;

            // Fill from inside-out
            let is_filled = gauge_index < filled_gauges;
            gauge_index += 1;

            let fill_color = if is_filled {
                // Apply subtle pulse to filled gauges
                let pulse_factor = if heat_pct > 0.85 { pulse } else { 0.95 + 0.05 * pulse };
                egui::Color32::from_rgb(
                    (heat_color.r() as f32 * pulse_factor).min(255.0) as u8,
                    (heat_color.g() as f32 * pulse_factor).min(255.0) as u8,
                    (heat_color.b() as f32 * pulse_factor).min(255.0) as u8,
                )
            } else {
                empty_color
            };

            draw_radial_gauge(
                painter,
                center,
                layer_inner,
                layer_outer,
                angle,
                gauge_arc,
                fill_color,
                border_color,
            );
        }
    }

    // Center glow when heat is high (warning)
    if heat_pct > 0.5 {
        let glow_intensity = (heat_pct - 0.5) * 2.0; // 0.0 at 50%, 1.0 at 100%
        let glow_alpha = (glow_intensity * 0.5 * 255.0 * pulse) as u8;
        let glow_color = egui::Color32::from_rgba_unmultiplied(
            heat_color.r(),
            (heat_color.g() as f32 * 0.6) as u8,
            (heat_color.b() as f32 * 0.4) as u8,
            glow_alpha,
        );
        painter.circle_filled(center, inner_radius * 0.7, glow_color);
    }
}

/// Draw a single radial gauge (rectangular cell pointing outward)
fn draw_radial_gauge(
    painter: &egui::Painter,
    center: egui::Pos2,
    inner_radius: f32,
    outer_radius: f32,
    center_angle: f32,
    arc_width: f32,
    fill_color: egui::Color32,
    border_color: egui::Color32,
) {
    let half_arc = arc_width / 2.0;
    let start_angle = center_angle - half_arc;
    let end_angle = center_angle + half_arc;

    // Create trapezoid shape (wider at outer edge, narrower at inner)
    let points = vec![
        // Inner edge (narrower)
        egui::pos2(
            center.x + inner_radius * start_angle.cos(),
            center.y + inner_radius * start_angle.sin(),
        ),
        // Outer edge left
        egui::pos2(
            center.x + outer_radius * start_angle.cos(),
            center.y + outer_radius * start_angle.sin(),
        ),
        // Outer edge right
        egui::pos2(
            center.x + outer_radius * end_angle.cos(),
            center.y + outer_radius * end_angle.sin(),
        ),
        // Inner edge right
        egui::pos2(
            center.x + inner_radius * end_angle.cos(),
            center.y + inner_radius * end_angle.sin(),
        ),
    ];

    // Fill
    painter.add(egui::Shape::convex_polygon(
        points.clone(),
        fill_color,
        egui::Stroke::NONE,
    ));

    // Border
    painter.add(egui::Shape::closed_line(
        points,
        egui::Stroke::new(0.5, border_color),
    ));
}

/// Draw overheating status indicators (small orange/red marks)
fn draw_heat_indicators(
    painter: &egui::Painter,
    center: egui::Pos2,
    wheel_radius: f32,
    heat_pct: f32,
) {
    // Position above the capacitor area
    let indicator_radius = wheel_radius - 35.0;
    let num_indicators = 5;
    let filled = (heat_pct * num_indicators as f32).ceil() as u32;

    // Arc from -120 to -60 degrees (top area)
    let start_angle = -PI * 0.7;
    let end_angle = -PI * 0.3;
    let arc_span = end_angle - start_angle;

    for i in 0..num_indicators {
        let angle = start_angle + (i as f32 / (num_indicators - 1) as f32) * arc_span;
        let x = center.x + indicator_radius * angle.cos();
        let y = center.y + indicator_radius * angle.sin();

        let is_active = i < filled;
        let color = if is_active {
            if heat_pct > 0.8 {
                egui::Color32::from_rgb(255, 60, 40) // Critical red
            } else if heat_pct > 0.5 {
                egui::Color32::from_rgb(255, 140, 40) // Warning orange
            } else {
                egui::Color32::from_rgb(255, 200, 80) // Low heat yellow
            }
        } else {
            egui::Color32::from_rgb(40, 45, 55) // Inactive
        };

        // Small rectangular indicator
        let rect = egui::Rect::from_center_size(egui::pos2(x, y), egui::vec2(4.0, 8.0));
        painter.rect_filled(rect, 1.0, color);
    }
}
