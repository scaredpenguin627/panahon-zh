//! 配色、间距与天空渐变。
//!
//! 尺寸取自华为天气 APK 的资源表（见 `docs/huawei-design-spec.md`）：
//! 卡片外边距 12dp、圆角 16dp、底色 10% 白、卡片标题 12sp 60% 白。

use eframe::egui::{self, Color32, CornerRadius, FontId, Margin, Rect, RichText, Vec2};
use panahon_core::WeatherIcon;

/// `cardview_radius` = 16dp
pub const CARD_RADIUS: u8 = 16;
/// 卡片左右外边距 12dp
pub const CARD_MARGIN: f32 = 12.0;
/// 主天气区最小高度 268dp
pub const HERO_MIN_HEIGHT: f32 = 268.0;
/// 逐小时单元格宽度 72dp
pub const HOUR_CELL_WIDTH: f32 = 72.0;
/// 多日预报行高 48dp
pub const DAY_ROW_HEIGHT: f32 = 48.0;

#[derive(Clone, Copy)]
pub struct Palette {
    pub top: Color32,
    pub bottom: Color32,
    pub text: Color32,
    /// 60% 白，用于次要文字与卡片标题
    pub dim: Color32,
    /// 30% 白，用于脚注
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
        (WeatherIcon::Clear, true) => (rgb(0x1F6FB2), rgb(0x7FB8DE)),
        (WeatherIcon::Clear, false) => (rgb(0x070E1A), rgb(0x1C2A42)),
        (WeatherIcon::PartlyCloudy, true) => (rgb(0x2E6E9E), rgb(0x93BBD3)),
        (WeatherIcon::PartlyCloudy, false) => (rgb(0x0B1322), rgb(0x223047)),
        (WeatherIcon::Cloudy, true) => (rgb(0x4C6478), rgb(0x97A7B5)),
        (WeatherIcon::Cloudy, false) => (rgb(0x111825), rgb(0x2B3646)),
        (WeatherIcon::Fog, true) => (rgb(0x62707C), rgb(0xA6B1B9)),
        (WeatherIcon::Fog, false) => (rgb(0x1A2028), rgb(0x333B45)),
        (icon, true) if icon.is_wet() => (rgb(0x37485A), rgb(0x7C90A2)),
        (icon, false) if icon.is_wet() => (rgb(0x0A1119), rgb(0x243240)),
        (_, true) => (rgb(0x2F6E9B), rgb(0x98BFD7)),
        (_, false) => (rgb(0x0A1220), rgb(0x1F2C40)),
    };

    Palette {
        top,
        bottom,
        text: Color32::WHITE,
        dim: Color32::from_rgba_unmultiplied(255, 255, 255, 153), // #99FFFFFF
        faint: Color32::from_rgba_unmultiplied(255, 255, 255, 77), // #4DFFFFFF
        card_line: Color32::from_rgba_unmultiplied(255, 255, 255, 38),
        accent: rgb(0xFFC94D),
        track: Color32::from_rgba_unmultiplied(255, 255, 255, 56),
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

/// 华为的卡片：10% 白、16dp 圆角、内部 16/14 边距。
pub fn card() -> egui::Frame {
    egui::Frame::NONE
        .fill(Color32::from_rgba_unmultiplied(255, 255, 255, 26))
        .corner_radius(CornerRadius::same(CARD_RADIUS))
        .inner_margin(Margin::symmetric(16, 14))
}

/// 卡片标题：12sp、60% 白。
pub fn card_title(ui: &mut egui::Ui, palette: &Palette, text: &str) {
    ui.label(RichText::new(text).size(12.0).color(palette.dim));
}

/// 旧名字，保持调用点不变。
pub fn section_title(ui: &mut egui::Ui, text: &str, palette: &Palette) {
    card_title(ui, palette, text);
}

/// 指标块：标签 12、数值 17、脚注 11。
pub fn metric(ui: &mut egui::Ui, palette: &Palette, label: &str, value: &str, hint: &str) {
    ui.vertical(|ui| {
        ui.add(
            egui::Label::new(RichText::new(label).size(12.0).color(palette.faint))
                .wrap_mode(egui::TextWrapMode::Extend),
        );
        ui.add_space(2.0);
        ui.add(
            egui::Label::new(RichText::new(value).size(17.0).color(palette.text))
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
    style.spacing.item_spacing = Vec2::new(10.0, 8.0);
    style.visuals.window_corner_radius = CornerRadius::same(16);
    style.visuals.widgets.noninteractive.corner_radius = CornerRadius::same(16);
    style.visuals.widgets.inactive.corner_radius = CornerRadius::same(16);
    style.visuals.widgets.hovered.corner_radius = CornerRadius::same(16);
    style.visuals.widgets.active.corner_radius = CornerRadius::same(16);

    let translucent = |alpha: u8| Color32::from_rgba_unmultiplied(255, 255, 255, alpha);
    style.visuals.widgets.inactive.bg_fill = translucent(26);
    style.visuals.widgets.inactive.weak_bg_fill = translucent(26);
    style.visuals.widgets.hovered.bg_fill = translucent(51);
    style.visuals.widgets.hovered.weak_bg_fill = translucent(51);
    style.visuals.widgets.active.bg_fill = translucent(77);
    style.visuals.widgets.active.weak_bg_fill = translucent(77);
    style.visuals.widgets.noninteractive.bg_fill = Color32::TRANSPARENT;
    style.visuals.extreme_bg_color = translucent(38);
    style.visuals.faint_bg_color = translucent(26);
    style.visuals.selection.bg_fill = Color32::from_rgba_unmultiplied(255, 201, 77, 90);
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

/// 安卓上避让状态栏，桌面预览留一点边距。
pub fn safe_top() -> f32 {
    if cfg!(target_os = "android") {
        36.0
    } else {
        12.0
    }
}
