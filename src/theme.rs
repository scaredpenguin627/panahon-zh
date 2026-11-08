//! Colours, spacing and the sky gradient behind every screen.
//!
//! The palette follows the weather: a clear day is a blue sky, a rainy night is
//! almost black. Cards are translucent white so they read on every gradient.

use eframe::egui::{self, Color32, CornerRadius, FontId, Margin, Rect, RichText, Vec2};
use panahon_core::WeatherIcon;

pub const CARD_RADIUS: u8 = 22;

#[derive(Clone, Copy)]
pub struct Palette {
    pub top: Color32,
    pub bottom: Color32,
    pub text: Color32,
    pub dim: Color32,
    pub faint: Color32,
    pub card_line: Color32,
    pub accent: Color32,
    pub track: Color32,
}

fn rgb(hex: u32) -> Color32 {
    Color32::from_rgb(
        ((hex >> 16) & 0xFF) as u8,
        ((hex >> 8) & 0xFF) as u8,
        (hex & 0xFF) as u8,
    )
}

/// Sky colours for the current condition and time of day.
pub fn palette(icon: WeatherIcon, is_day: bool) -> Palette {
    let (top, bottom) = match (icon, is_day) {
        (WeatherIcon::Clear, true) => (rgb(0x1D6CA6), rgb(0x86BEE2)),
        (WeatherIcon::Clear, false) => (rgb(0x080F1C), rgb(0x1E2C45)),
        (WeatherIcon::PartlyCloudy, true) => (rgb(0x2E6E9E), rgb(0x9DC2D8)),
        (WeatherIcon::PartlyCloudy, false) => (rgb(0x0B1322), rgb(0x223047)),
        (WeatherIcon::Cloudy, true) => (rgb(0x4C6478), rgb(0x9AAAB8)),
        (WeatherIcon::Cloudy, false) => (rgb(0x111825), rgb(0x2B3646)),
        (WeatherIcon::Fog, true) => (rgb(0x62707C), rgb(0xA9B4BC)),
        (WeatherIcon::Fog, false) => (rgb(0x1A2028), rgb(0x333B45)),
        (icon, true) if icon.is_wet() => (rgb(0x37485A), rgb(0x7E92A4)),
        (icon, false) if icon.is_wet() => (rgb(0x0A1119), rgb(0x243240)),
        (_, true) => (rgb(0x2F6E9B), rgb(0x9BC1D9)),
        (_, false) => (rgb(0x0A1220), rgb(0x1F2C40)),
    };

    let dark = !is_day;
    Palette {
        top,
        bottom,
        text: Color32::WHITE,
        dim: Color32::from_rgba_unmultiplied(255, 255, 255, if dark { 176 } else { 208 }),
        faint: Color32::from_rgba_unmultiplied(255, 255, 255, if dark { 120 } else { 150 }),
        card_line: Color32::from_rgba_unmultiplied(255, 255, 255, if dark { 26 } else { 44 }),
        accent: rgb(0xFFD166),
        track: Color32::from_rgba_unmultiplied(255, 255, 255, 60),
    }
}

/// Vertical gradient drawn in bands; cheap and works on every backend.
pub fn paint_background(painter: &egui::Painter, rect: Rect, palette: &Palette) {
    const BANDS: usize = 96;
    let band = rect.height() / BANDS as f32;
    for index in 0..BANDS {
        let t = index as f32 / (BANDS - 1) as f32;
        let color = lerp_color(palette.top, palette.bottom, t);
        let top = rect.top() + band * index as f32;
        painter.rect_filled(
            Rect::from_min_max(
                egui::pos2(rect.left(), top),
                egui::pos2(rect.right(), top + band + 1.0),
            ),
            CornerRadius::ZERO,
            color,
        );
    }
}

pub fn lerp_color(from: Color32, to: Color32, t: f32) -> Color32 {
    let t = t.clamp(0.0, 1.0);
    let blend = |a: u8, b: u8| (a as f32 + (b as f32 - a as f32) * t).round() as u8;
    Color32::from_rgb(
        blend(from.r(), to.r()),
        blend(from.g(), to.g()),
        blend(from.b(), to.b()),
    )
}

/// Translucent card used by every block.
pub fn card() -> egui::Frame {
    egui::Frame::NONE
        .fill(Color32::from_rgba_unmultiplied(255, 255, 255, 26))
        .stroke(egui::Stroke::new(
            1.0_f32,
            Color32::from_rgba_unmultiplied(255, 255, 255, 34),
        ))
        .corner_radius(CornerRadius::same(CARD_RADIUS))
        .inner_margin(Margin::symmetric(16, 14))
}

/// Card with a stronger fill, for the hero blocks.
pub fn card_strong() -> egui::Frame {
    egui::Frame::NONE
        .fill(Color32::from_rgba_unmultiplied(255, 255, 255, 40))
        .stroke(egui::Stroke::new(
            1.0_f32,
            Color32::from_rgba_unmultiplied(255, 255, 255, 48),
        ))
        .corner_radius(CornerRadius::same(CARD_RADIUS))
        .inner_margin(Margin::symmetric(16, 16))
}

pub fn section_title(ui: &mut egui::Ui, text: &str, palette: &Palette) {
    ui.label(RichText::new(text).size(15.0).strong().color(palette.dim));
}

/// Small label + big value used by the detail grid.
pub fn metric(ui: &mut egui::Ui, palette: &Palette, label: &str, value: &str, hint: &str) {
    ui.vertical(|ui| {
        ui.add(
            egui::Label::new(RichText::new(label).size(12.0).color(palette.faint))
                .wrap_mode(egui::TextWrapMode::Extend),
        );
        ui.add_space(2.0);
        ui.add(
            egui::Label::new(RichText::new(value).size(19.0).strong().color(palette.text))
                .wrap_mode(egui::TextWrapMode::Extend),
        );
        if !hint.is_empty() {
            ui.add(
                egui::Label::new(RichText::new(hint).size(11.0).color(palette.faint))
                    .wrap_mode(egui::TextWrapMode::Extend),
            );
        }
    });
}

pub fn apply_style(ctx: &egui::Context) {
    let mut style = (*ctx.style()).clone();
    style.spacing.item_spacing = Vec2::new(10.0, 10.0);
    style.visuals.window_corner_radius = CornerRadius::same(20);
    style.visuals.widgets.noninteractive.corner_radius = CornerRadius::same(14);
    style.visuals.widgets.inactive.corner_radius = CornerRadius::same(14);
    style.visuals.widgets.hovered.corner_radius = CornerRadius::same(14);
    style.visuals.widgets.active.corner_radius = CornerRadius::same(14);

    // 半透明白色控件，避免默认的深色按钮在渐变背景上变成黑块
    let translucent = |alpha: u8| Color32::from_rgba_unmultiplied(255, 255, 255, alpha);
    style.visuals.widgets.inactive.bg_fill = translucent(38);
    style.visuals.widgets.inactive.weak_bg_fill = translucent(38);
    style.visuals.widgets.hovered.bg_fill = translucent(64);
    style.visuals.widgets.hovered.weak_bg_fill = translucent(64);
    style.visuals.widgets.active.bg_fill = translucent(90);
    style.visuals.widgets.active.weak_bg_fill = translucent(90);
    style.visuals.widgets.noninteractive.bg_fill = Color32::TRANSPARENT;
    style.visuals.extreme_bg_color = translucent(34);
    style.visuals.faint_bg_color = translucent(24);
    style.visuals.selection.bg_fill = Color32::from_rgba_unmultiplied(255, 209, 102, 90);
    style.visuals.selection.stroke = egui::Stroke::new(1.0_f32, Color32::WHITE);
    style.visuals.panel_fill = Color32::TRANSPARENT;
    style.visuals.override_text_color = Some(Color32::WHITE);
    style
        .text_styles
        .insert(egui::TextStyle::Body, FontId::proportional(15.0));
    style
        .text_styles
        .insert(egui::TextStyle::Button, FontId::proportional(14.0));
    style
        .text_styles
        .insert(egui::TextStyle::Heading, FontId::proportional(22.0));
    ctx.set_style(style);
}

/// Top padding so content clears the status bar on Android.
pub fn safe_top() -> f32 {
    if cfg!(target_os = "android") {
        36.0
    } else {
        12.0
    }
}
