//! Vector weather glyphs.
//!
//! The app draws its own icons instead of shipping PAGASA's PNG assets, which
//! keeps the APK small and the clean-room boundary intact.

use eframe::egui::{self, Color32, Painter, Pos2, Rect, Shape, Stroke, Vec2};
use panahon_core::WeatherIcon;

const SUN: Color32 = Color32::from_rgb(0xFF, 0xD1, 0x66);
const MOON: Color32 = Color32::from_rgb(0xE9, 0xEF, 0xF8);
const CLOUD: Color32 = Color32::from_rgb(0xF2, 0xF6, 0xFA);
const RAIN: Color32 = Color32::from_rgb(0xA9, 0xD6, 0xF5);
const BOLT: Color32 = Color32::from_rgb(0xFF, 0xC9, 0x4D);

/// Allocate `size` square pixels and draw `icon` into it.
pub fn paint(
    ui: &mut egui::Ui,
    icon: WeatherIcon,
    is_day: bool,
    size: f32,
    background: Color32,
) -> egui::Response {
    let (rect, response) = ui.allocate_exact_size(Vec2::splat(size), egui::Sense::hover());
    draw(ui.painter(), rect, icon, is_day, background);
    response
}

pub fn draw(painter: &Painter, rect: Rect, icon: WeatherIcon, is_day: bool, background: Color32) {
    let s = rect.width().min(rect.height());
    let center = rect.center();

    match icon {
        WeatherIcon::Clear if is_day => sun(painter, center, s * 0.20, s * 0.34),
        WeatherIcon::Clear => crescent(painter, center, s * 0.22, background),
        WeatherIcon::PartlyCloudy => {
            if is_day {
                sun(
                    painter,
                    center + Vec2::new(-s * 0.14, -s * 0.16),
                    s * 0.14,
                    s * 0.24,
                );
            } else {
                crescent(
                    painter,
                    center + Vec2::new(-s * 0.14, -s * 0.16),
                    s * 0.15,
                    background,
                );
            }
            cloud(
                painter,
                center + Vec2::new(s * 0.05, s * 0.10),
                s * 0.66,
                CLOUD,
            );
        }
        WeatherIcon::Cloudy => {
            cloud(
                painter,
                center + Vec2::new(-s * 0.06, -s * 0.02),
                s * 0.58,
                dim(CLOUD),
            );
            cloud(
                painter,
                center + Vec2::new(s * 0.08, s * 0.10),
                s * 0.68,
                CLOUD,
            );
        }
        WeatherIcon::Fog => {
            cloud(
                painter,
                center + Vec2::new(0.0, -s * 0.06),
                s * 0.66,
                dim(CLOUD),
            );
            for index in 0..3 {
                let y = center.y + s * (0.16 + index as f32 * 0.07);
                let half = s * (0.24 - index as f32 * 0.03);
                painter.line_segment(
                    [Pos2::new(center.x - half, y), Pos2::new(center.x + half, y)],
                    Stroke::new(s * 0.045, dim(CLOUD)),
                );
            }
        }
        WeatherIcon::Wind => {
            for index in 0..3 {
                let y = center.y + s * (-0.12 + index as f32 * 0.12);
                let half = s * (0.26 - index as f32 * 0.04);
                painter.line_segment(
                    [Pos2::new(center.x - half, y), Pos2::new(center.x + half, y)],
                    Stroke::new(s * 0.05, CLOUD),
                );
            }
        }
        icon => {
            cloud(painter, center + Vec2::new(0.0, -s * 0.10), s * 0.68, CLOUD);
            match icon {
                WeatherIcon::Thunderstorm => {
                    bolt(painter, center + Vec2::new(0.0, s * 0.20), s * 0.30);
                }
                WeatherIcon::HeavyRain => {
                    drops(painter, center + Vec2::new(0.0, s * 0.18), s, 4, true)
                }
                _ => drops(painter, center + Vec2::new(0.0, s * 0.18), s, 3, false),
            }
        }
    }
}

fn dim(color: Color32) -> Color32 {
    Color32::from_rgba_unmultiplied(color.r(), color.g(), color.b(), 170)
}

fn sun(painter: &Painter, center: Pos2, radius: f32, ray_length: f32) {
    painter.circle_filled(center, radius, SUN);
    for index in 0..8 {
        let angle = std::f32::consts::TAU * index as f32 / 8.0;
        let direction = Vec2::new(angle.cos(), angle.sin());
        painter.line_segment(
            [
                center + direction * (radius * 1.45),
                center + direction * (radius * 1.45 + ray_length * 0.45),
            ],
            Stroke::new(radius * 0.28, SUN),
        );
    }
}

fn crescent(painter: &Painter, center: Pos2, radius: f32, background: Color32) {
    painter.circle_filled(center, radius, MOON);
    painter.circle_filled(
        center + Vec2::new(radius * 0.62, -radius * 0.20),
        radius * 0.88,
        background,
    );
}

fn cloud(painter: &Painter, center: Pos2, width: f32, color: Color32) {
    let half = width * 0.5;
    let base = Rect::from_min_max(
        Pos2::new(center.x - half, center.y + width * 0.02),
        Pos2::new(center.x + half, center.y + width * 0.30),
    );
    painter.rect_filled(base, egui::CornerRadius::same((width * 0.14) as u8), color);
    painter.circle_filled(
        Pos2::new(center.x - width * 0.22, center.y + width * 0.04),
        width * 0.18,
        color,
    );
    painter.circle_filled(
        Pos2::new(center.x + width * 0.02, center.y - width * 0.08),
        width * 0.24,
        color,
    );
    painter.circle_filled(
        Pos2::new(center.x + width * 0.26, center.y + width * 0.05),
        width * 0.16,
        color,
    );
}

fn drops(painter: &Painter, center: Pos2, scale: f32, count: usize, heavy: bool) {
    let width = if heavy { scale * 0.05 } else { scale * 0.04 };
    let length = if heavy { scale * 0.16 } else { scale * 0.12 };
    for index in 0..count {
        let offset = (index as f32 - (count as f32 - 1.0) * 0.5) * scale * 0.17;
        let top = Pos2::new(center.x + offset, center.y);
        let bottom = Pos2::new(center.x + offset - length * 0.35, center.y + length);
        painter.line_segment([top, bottom], Stroke::new(width, RAIN));
    }
}

fn bolt(painter: &Painter, center: Pos2, size: f32) {
    let points = vec![
        Pos2::new(center.x + size * 0.05, center.y - size * 0.55),
        Pos2::new(center.x - size * 0.32, center.y + size * 0.08),
        Pos2::new(center.x - size * 0.02, center.y + size * 0.08),
        Pos2::new(center.x - size * 0.14, center.y + size * 0.62),
        Pos2::new(center.x + size * 0.34, center.y - size * 0.12),
        Pos2::new(center.x + size * 0.04, center.y - size * 0.12),
    ];
    painter.add(Shape::convex_polygon(points, BOLT, Stroke::NONE));
}
