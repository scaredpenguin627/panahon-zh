//! Screen building blocks. Every widget takes plain data plus a [`Palette`],
//! so pages stay readable and the layout is easy to tweak in one place.

use eframe::egui::{
    self, Align, Align2, Color32, FontId, Layout, Pos2, Rect, RichText, Sense, Vec2,
};
use panahon_core::{
    AirQuality, AirQualityTrendPoint, Alert, CurrentWeather, Dashboard, DataQuality, ForecastDay,
    HourlyForecast, LifeIndex, MapTrackPoint, SunWeather,
};
use std::time::Duration;

use crate::icons;
use crate::theme::{self, Palette};

const LANGUAGE_ID: &str = "panahon-ui-language";
const BLENDER_SKY_TEXTURE_ID: &str = "panahon-blender-sky-preview";
const BLENDER_SKY_PREVIEW: &[u8] =
    include_bytes!("../assets/blender_stage4/preview/cloud_sky_frame_0001.png");

/// Share the selected UI language with leaf widgets without widening every
/// renderer's public signature.
pub fn set_language(ctx: &egui::Context, english: bool) {
    ctx.data_mut(|data| data.insert_temp(egui::Id::new(LANGUAGE_ID), english));
}

fn english(ui: &egui::Ui) -> bool {
    ui.ctx()
        .data(|data| data.get_temp::<bool>(egui::Id::new(LANGUAGE_ID)))
        .unwrap_or(false)
}

fn condition_text(value: &str, icon: panahon_core::WeatherIcon, is_english: bool) -> String {
    if !is_english {
        return value.to_owned();
    }
    match value {
        "晴" | "晴朗" => "Clear".into(),
        "多云" => "Cloudy".into(),
        "局部多云" => "Partly cloudy".into(),
        "阵雨" | "局部阵雨" => "Showers".into(),
        "小雨" => "Light rain".into(),
        "大雨" => "Heavy rain".into(),
        "雷雨" => "Storm".into(),
        "雾" => "Fog".into(),
        "大风" => "Windy".into(),
        _ => icon.label().into(),
    }
}

fn life_text(title: &str, level: &str, advice: &str, is_english: bool) -> (String, String, String) {
    if !is_english {
        return (title.into(), level.into(), advice.into());
    }
    let title = match title {
        "穿衣" => "Clothing",
        "中暑" => "Heat risk",
        "出行" => "Travel",
        "晾晒" => "Laundry",
        _ => title,
    };
    let level = match level {
        "炎热" => "Hot",
        "较热" => "Warm",
        "舒适" => "Comfortable",
        "偏凉" => "Cool",
        "寒冷" => "Cold",
        "危险" => "Danger",
        "谨慎" => "Caution",
        "注意" => "Take care",
        "不宜" => "Avoid",
        "一般" => "Fair",
        "适宜" => "Good",
        "未知" => "Unknown",
        "暂无指数" => "Unavailable",
        _ => level,
    };
    let advice = match advice {
        "轻薄短袖，注意防晒补水" => "Light clothes; use sun protection and drink water",
        "短袖即可，室内注意通风" => "Short sleeves; keep indoor air moving",
        "短袖或薄长袖都合适" => "Short or light long sleeves",
        "建议加一件薄外套" => "A light jacket is recommended",
        "穿厚外套，注意保暖" => "Wear a warm coat",
        "尽量避免户外活动，及时补水" => "Avoid outdoor activity and hydrate",
        "减少长时间暴晒，随身带水" => "Limit sun exposure and carry water",
        "长时间户外活动要多补水" => "Hydrate during extended outdoor activity",
        "体感舒适，适合户外活动" => "Comfortable for outdoor activity",
        "有明显降雨，带伞并留意积水" => "Rain likely; bring an umbrella",
        "可能有阵雨，出门带伞" => "Showers possible; bring an umbrella",
        "降雨概率低，适合出行" => "Low rain risk; good for travel",
        "当前观测有降水，出门建议带伞" => "Rain is observed now; bring an umbrella",
        "当前观测降水较少，适合出行" => {
            "Little rain is observed now; check the forecast before going out"
        }
        "暂无降水观测，出行前请查看官方预报" => {
            "No precipitation observation; check the official forecast"
        }
        "暂无降水观测，晾晒前请查看逐小时预报" => {
            "No precipitation observation; check the hourly forecast"
        }
        "暂无官方热指数，避免长时间暴晒并查看预报" => {
            "Official heat index unavailable; limit sun exposure and check the forecast"
        }
        "结合逐小时和逐日预报、个人耐受、日照和风选择衣物。" => {
            "Use hourly and daily forecasts together with personal tolerance, sun and wind"
        }
        "缺少完整逐小时或逐日输入，无法生成生活指数。" => {
            "Complete hourly and daily inputs are required for a lifestyle index"
        }
        "缺少完整逐小时或逐日输入，无法确认热风险。" => {
            "Complete hourly and daily inputs are required to assess heat risk"
        }
        "当前没有可审计的官方出行指数；请查看逐小时预报和预警。" => {
            "No auditable official travel index; check the hourly forecast and alerts"
        }
        "湿度大或有降雨，不建议晾晒" => "Humid or rainy; laundry may not dry",
        "晾晒时间需要拉长" => "Allow extra drying time",
        "空气较干，适合晾晒" => "Dry air; good for laundry",
        _ => advice,
    };
    (title.into(), level.into(), advice.into())
}

/// Colour for an AQI band, used by bars and badges.
pub fn aqi_color(aqi: Option<u16>) -> Color32 {
    match aqi.unwrap_or(0) {
        0..=50 => Color32::from_rgb(0x5A, 0xD1, 0x8B),
        51..=100 => Color32::from_rgb(0xF2, 0xC9, 0x4C),
        101..=150 => Color32::from_rgb(0xF2, 0x99, 0x4A),
        151..=200 => Color32::from_rgb(0xEB, 0x57, 0x57),
        201..=300 => Color32::from_rgb(0xBB, 0x6B, 0xF0),
        _ => Color32::from_rgb(0x9B, 0x1C, 0x4E),
    }
}

/// A darker variant of the band colour for text on light cards.
#[allow(dead_code)]
pub fn aqi_text_color(aqi: Option<u16>) -> Color32 {
    match aqi.unwrap_or(0) {
        0..=50 => Color32::from_rgb(0x14, 0x7A, 0x44),
        51..=100 => Color32::from_rgb(0x8A, 0x64, 0x08),
        101..=150 => Color32::from_rgb(0xA8, 0x4F, 0x05),
        151..=200 => Color32::from_rgb(0xB3, 0x1D, 0x1D),
        201..=300 => Color32::from_rgb(0x6D, 0x2C, 0xA0),
        _ => Color32::from_rgb(0x6B, 0x12, 0x36),
    }
}

/// Translate the stable AQI band labels returned by the public data adapter.
pub fn aqi_category_text(value: &str, is_english: bool) -> String {
    if !is_english {
        return value.to_owned();
    }
    match value {
        "优" => "Good",
        "良" => "Moderate",
        "轻度污染" => "Unhealthy for sensitive groups",
        "中度污染" => "Unhealthy",
        "重度污染" => "Very unhealthy",
        "严重污染" => "Hazardous",
        "暂无" => "Unknown",
        value => value,
    }
    .to_owned()
}

/// Compact rounded badge holding the AQI number.
pub fn aqi_badge(ui: &mut egui::Ui, aqi: Option<u16>) -> egui::Response {
    let (rect, response) = ui.allocate_exact_size(Vec2::new(42.0, 24.0), Sense::hover());
    ui.painter()
        .rect_filled(rect, egui::CornerRadius::same(8), aqi_color(aqi));
    ui.painter().text(
        rect.center(),
        Align2::CENTER_CENTER,
        aqi.map(|value| value.to_string())
            .unwrap_or_else(|| "--".into()),
        FontId::proportional(14.0),
        Color32::from_rgb(0x11, 0x1C, 0x24),
    );
    response
}

/// Big current-weather block: place, temperature, condition, high/low.
fn wind_dir_en(value: &str) -> String {
    match value {
        "北" => "N",
        "东北" => "NE",
        "东" => "E",
        "东南" => "SE",
        "南" => "S",
        "西南" => "SW",
        "西" => "W",
        "西北" => "NW",
        "静风" => "Calm",
        "未知" => "Unknown",
        other => other,
    }
    .to_owned()
}

fn sun_source_en(value: &str) -> String {
    value
        .replace("模型预报", "model forecast")
        .replace("离线示例", "offline sample")
        .replace("预报", "forecast")
}
fn place_name(value: &str, is_english: bool) -> String {
    if !is_english {
        return value.to_owned();
    }
    match value {
        "马尼拉" => "Manila".into(),
        "马尼拉大都会" => "Metro Manila".into(),
        other => other.to_owned(),
    }
}

fn updated_text(value: &str, is_english: bool) -> String {
    if is_english && value == "离线示例" {
        return "offline sample".into();
    }
    value.to_owned()
}
/// 主天气区：华为的当前天气块是居中、留白充足的，最小高 268dp。
///
/// 从上到下：城市 → 更新时间 → 大号温度 → 高低温 → 天气现象 · 空气质量。
pub fn hero(ui: &mut egui::Ui, dashboard: &Dashboard, air: Option<&AirQuality>, palette: &Palette) {
    let is_english = english(ui);
    let current = &dashboard.current;
    let (rect, _) = ui.allocate_exact_size(
        Vec2::new(ui.available_width(), theme::HERO_MIN_HEIGHT),
        Sense::hover(),
    );
    let mut ui = ui.new_child(
        egui::UiBuilder::new()
            .max_rect(rect)
            .layout(Layout::top_down(Align::Center)),
    );

    ui.add_space(12.0);
    ui.label(
        RichText::new(place_name(&dashboard.place.name, is_english))
            .size(26.0)
            .color(palette.text),
    );
    ui.add_space(2.0);
    ui.label(
        RichText::new(format!(
            "{}{}",
            if is_english { "Updated " } else { "更新于 " },
            short_time(&updated_text(&current.updated_at, is_english))
        ))
        .size(11.0)
        .color(palette.faint),
    );

    ui.add_space(4.0);
    if current.temperature_quality.is_available() {
        ui.label(
            RichText::new(format!("{:.0}°", current.temperature_c))
                .size(96.0)
                .color(palette.text),
        );
    } else {
        ui.label(RichText::new("—").size(96.0).color(palette.dim));
    }

    if let Some(day) = dashboard.forecast.first() {
        ui.label(
            RichText::new(match (day.low_c, day.high_c) {
                (Some(low), Some(high)) => format!("{low:.0}° / {high:.0}°"),
                _ => "—".into(),
            })
            .size(16.0)
            .color(palette.dim),
        );
    }

    ui.add_space(6.0);
    let mut line = condition_text(&current.condition, current.icon, is_english);
    if let Some(air) = air {
        line.push_str("  ·  ");
        match air.summary.max_aqi {
            Some(aqi) => {
                let category = aqi_category_text(
                    air.summary.category.as_deref().unwrap_or_default(),
                    is_english,
                );
                line.push_str(&format!("AQI {aqi} {category}"));
            }
            None => line.push_str("AQI --"),
        }
    }
    ui.label(RichText::new(line).size(15.0).color(palette.text));
}

/// Display the original Blender 5.3 sky prototype as a small, replaceable
/// visual layer. The PNG is an offline resource; the moving glow is drawn by
/// egui so the page still animates when video playback is unavailable.
pub fn blender_sky_card(ui: &mut egui::Ui, palette: &Palette) {
    let is_english = english(ui);
    let texture = ui
        .ctx()
        .data(|data| data.get_temp::<egui::TextureHandle>(egui::Id::new(BLENDER_SKY_TEXTURE_ID)));
    let texture = texture.or_else(|| {
        let decoded = image::load_from_memory(BLENDER_SKY_PREVIEW)
            .ok()?
            .to_rgba8();
        let size = [decoded.width() as usize, decoded.height() as usize];
        let pixels =
            egui::ColorImage::from_rgba_unmultiplied(size, decoded.as_flat_samples().as_slice());
        let handle =
            ui.ctx()
                .load_texture(BLENDER_SKY_TEXTURE_ID, pixels, egui::TextureOptions::LINEAR);
        ui.ctx().data_mut(|data| {
            data.insert_temp(egui::Id::new(BLENDER_SKY_TEXTURE_ID), handle.clone())
        });
        Some(handle)
    });

    theme::card().show(ui, |ui| {
        ui.set_min_width(ui.available_width());
        let (title, description) = if is_english {
            (
                "Sky motion prototype",
                "Blender 5.3 preview · egui glow fallback",
            )
        } else {
            ("天空动效原型", "Blender 5.3 预览 · egui 光晕回退")
        };
        card_header(ui, palette, title, "Original offline asset");
        ui.add_space(6.0);
        let width = ui.available_width();
        let height = (width * 0.56).clamp(96.0, 180.0);
        let (rect, _) = ui.allocate_exact_size(Vec2::new(width, height), Sense::hover());
        if let Some(texture) = texture {
            ui.painter().image(
                texture.id(),
                rect,
                egui::Rect::from_min_max(egui::pos2(0.0, 0.0), egui::pos2(1.0, 1.0)),
                Color32::WHITE,
            );
        } else {
            ui.painter()
                .rect_filled(rect, egui::CornerRadius::same(10), palette.track);
        }
        let t = ui.input(|input| input.time) as f32;
        let pulse = (t * 0.8).sin() * 0.5 + 0.5;
        ui.painter().circle_filled(
            rect.right_bottom() - Vec2::new(28.0, 24.0),
            9.0 + pulse * 4.0,
            Color32::from_rgba_unmultiplied(255, 226, 160, 24 + (pulse * 36.0) as u8),
        );
        ui.ctx().request_repaint_after(Duration::from_millis(80));
        ui.add_space(4.0);
        ui.label(RichText::new(description).size(10.0).color(palette.faint));
    });
}
/// 供 app.rs 复用的卡片标题行。
pub fn card_header_public(ui: &mut egui::Ui, palette: &Palette, title: &str, source: &str) {
    card_header(ui, palette, title, source);
}

/// 卡片标题行：左侧标题，右侧来源标签。
fn card_header(ui: &mut egui::Ui, palette: &Palette, title: &str, source: &str) {
    ui.horizontal(|ui| {
        theme::card_title(ui, palette, title);
        ui.with_layout(Layout::right_to_left(Align::Center), |ui| {
            if !source.is_empty() {
                ui.label(RichText::new(source).size(10.0).color(palette.faint));
            }
        });
    });
}

/// 华为风格的逐小时卡片：72dp 单元格，时间 → 图标 → 降水 → 温度。
pub fn hourly_strip(ui: &mut egui::Ui, hours: &[HourlyForecast], palette: &Palette) {
    let is_english = english(ui);
    if hours.is_empty() {
        return;
    }

    theme::card().show(ui, |ui| {
        ui.set_min_width(ui.available_width());
        let source = if hours.iter().any(|hour| !hour.precip_quality.is_official()) {
            if is_english {
                "Official forecast · rain partly estimated"
            } else {
                "官方预报 · 降水含估算"
            }
        } else if is_english {
            "Official forecast"
        } else {
            "官方预报"
        };
        card_header(
            ui,
            palette,
            if is_english {
                "Hourly forecast"
            } else {
                "逐小时预报"
            },
            source,
        );
        ui.add_space(6.0);
        egui::ScrollArea::horizontal()
            .auto_shrink([false, true])
            .show(ui, |ui| {
                ui.spacing_mut().item_spacing.x = 0.0;
                ui.horizontal(|ui| {
                    for hour in hours {
                        hour_cell(ui, hour, palette, is_english);
                    }
                });
            });
    });
}

fn hour_cell(ui: &mut egui::Ui, hour: &HourlyForecast, palette: &Palette, is_english: bool) {
    let (rect, _) =
        ui.allocate_exact_size(Vec2::new(theme::HOUR_CELL_WIDTH, 118.0), Sense::hover());
    let painter = ui.painter();
    let time = if is_english {
        if hour.time == "现在" {
            "Now".to_owned()
        } else if hour.time == "未知" {
            "--".to_owned()
        } else if let Some(value) = hour.time.strip_suffix('时') {
            format!("{value}:00")
        } else {
            hour.time.clone()
        }
    } else {
        hour.time.clone()
    };
    painter.text(
        Pos2::new(rect.center().x, rect.top() + 16.0),
        Align2::CENTER_TOP,
        time,
        FontId::proportional(13.0),
        palette.dim,
    );
    icons::draw(
        painter,
        Rect::from_center_size(
            Pos2::new(rect.center().x, rect.top() + 52.0),
            Vec2::splat(28.0),
        ),
        hour.icon,
        hour.is_day,
        palette.top,
    );
    if hour.precip_quality.is_available() && hour.precip_mm.is_some_and(|value| value > 0.05) {
        let precip = hour.precip_mm.unwrap_or_default();
        let prefix = if hour.precip_quality == DataQuality::Estimated {
            "≈"
        } else {
            ""
        };
        painter.text(
            Pos2::new(rect.center().x, rect.top() + 70.0),
            Align2::CENTER_TOP,
            format!("{prefix}{precip:.1}mm"),
            FontId::proportional(10.0),
            Color32::from_rgb(0xA9, 0xD6, 0xF5),
        );
    }
    painter.text(
        Pos2::new(rect.center().x, rect.top() + 88.0),
        Align2::CENTER_TOP,
        hour.temperature_c
            .map(|value| format!("{value:.0}°"))
            .unwrap_or_else(|| "—".into()),
        FontId::proportional(16.0),
        palette.text,
    );
}

/// 华为风格的多日预报：48dp 行，左侧星期+日期，中间图标，右侧高低温。
pub fn daily_list(ui: &mut egui::Ui, days: &[ForecastDay], palette: &Palette) {
    let is_english = english(ui);
    if days.is_empty() {
        return;
    }

    theme::card().show(ui, |ui| {
        ui.set_min_width(ui.available_width());
        let source = if is_english {
            "Official forecast"
        } else {
            "官方预报"
        };
        card_header(
            ui,
            palette,
            if is_english {
                "Daily forecast"
            } else {
                "多日天气"
            },
            source,
        );
        ui.add_space(2.0);
        for (index, day) in days.iter().enumerate() {
            if index > 0 {
                divider(ui, palette);
            }
            day_row(ui, day, palette, is_english);
        }
    });
}

fn day_row(ui: &mut egui::Ui, day: &ForecastDay, palette: &Palette, is_english: bool) {
    let width = ui.available_width();
    ui.allocate_ui_with_layout(
        Vec2::new(width, theme::DAY_ROW_HEIGHT),
        Layout::left_to_right(Align::Center),
        |ui| {
            ui.spacing_mut().item_spacing.x = 8.0;
            ui.allocate_ui_with_layout(
                Vec2::new(64.0, theme::DAY_ROW_HEIGHT),
                Layout::top_down(Align::LEFT),
                |ui| {
                    ui.add_space(6.0);
                    ui.add_sized(
                        [64.0, 20.0],
                        egui::Label::new(
                            RichText::new(if is_english {
                                weekday_en(&day.date)
                            } else {
                                day.date.clone()
                            })
                            .size(15.0)
                            .color(palette.text),
                        ),
                    );
                    if !day.date_label.is_empty() {
                        ui.add_sized(
                            [64.0, 14.0],
                            egui::Label::new(
                                RichText::new(&day.date_label)
                                    .size(11.0)
                                    .color(palette.faint),
                            ),
                        );
                    }
                },
            );

            ui.allocate_ui_with_layout(
                Vec2::new(56.0, theme::DAY_ROW_HEIGHT),
                Layout::top_down(Align::Center),
                |ui| {
                    ui.add_space(5.0);
                    icons::paint(ui, day.icon, true, 24.0, palette.top);
                    if day.precip_quality.is_available()
                        && day.precip_mm.is_some_and(|value| value > 0.05)
                    {
                        let precip = day.precip_mm.unwrap_or_default();
                        let prefix = if day.precip_quality == DataQuality::Estimated {
                            "≈"
                        } else {
                            ""
                        };
                        ui.add_sized(
                            [56.0, 12.0],
                            egui::Label::new(
                                RichText::new(format!("{prefix}{precip:.1}mm"))
                                    .size(10.0)
                                    .color(Color32::from_rgb(0xA9, 0xD6, 0xF5)),
                            ),
                        );
                    }
                },
            );

            ui.add_space(4.0);
            ui.allocate_ui_with_layout(
                Vec2::new(ui.available_width() - 104.0, theme::DAY_ROW_HEIGHT),
                Layout::left_to_right(Align::Center),
                |ui| {
                    ui.label(
                        RichText::new(condition_text(&day.summary, day.icon, is_english))
                            .size(12.0)
                            .color(palette.dim),
                    );
                },
            );

            ui.with_layout(Layout::right_to_left(Align::Center), |ui| {
                ui.label(
                    RichText::new(
                        day.high_c
                            .map(|value| format!("{value:.0}°"))
                            .unwrap_or_else(|| "—".into()),
                    )
                    .size(17.0)
                    .color(palette.text),
                );
                ui.label(RichText::new("/").size(13.0).color(palette.faint));
                ui.label(
                    RichText::new(
                        day.low_c
                            .map(|value| format!("{value:.0}°"))
                            .unwrap_or_else(|| "—".into()),
                    )
                    .size(17.0)
                    .color(palette.dim),
                );
            });
        },
    );
}

fn weekday_en(date: &str) -> String {
    match date {
        "今天" => "Today".into(),
        "明天" => "Tomorrow".into(),
        "后天" => "In 2 days".into(),
        "周一" => "Mon".into(),
        "周二" => "Tue".into(),
        "周三" => "Wed".into(),
        "周四" => "Thu".into(),
        "周五" => "Fri".into(),
        "周六" => "Sat".into(),
        "周日" => "Sun".into(),
        other => other.to_owned(),
    }
}

fn divider(ui: &mut egui::Ui, palette: &Palette) {
    let (rect, _) = ui.allocate_exact_size(Vec2::new(ui.available_width(), 1.0), Sense::hover());
    ui.painter()
        .rect_filled(rect, egui::CornerRadius::ZERO, palette.card_line);
}
/// 空气质量卡片：华为用左侧弧形仪表 + 右侧两行文字。
pub fn aqi_summary(ui: &mut egui::Ui, air: &AirQuality, palette: &Palette) {
    let is_english = english(ui);
    let max = air
        .summary
        .max_aqi
        .or_else(|| air.stations.first().and_then(|station| station.aqi));
    let color = aqi_color(max);

    theme::card().show(ui, |ui| {
        ui.set_min_width(ui.available_width());
        card_header(
            ui,
            palette,
            if is_english {
                "Air quality"
            } else {
                "空气质量"
            },
            if is_english {
                "DENR-EMB"
            } else {
                "DENR-EMB 官方"
            },
        );
        ui.add_space(10.0);

        ui.horizontal(|ui| {
            let (rect, _) = ui.allocate_exact_size(Vec2::new(78.0, 68.0), Sense::hover());
            draw_gauge(ui.painter(), rect, max, color, palette);
            ui.add_space(10.0);
            ui.vertical(|ui| {
                let category = aqi_category_text(
                    air.summary.category.as_deref().unwrap_or_default(),
                    is_english,
                );
                ui.horizontal(|ui| {
                    ui.label(RichText::new(&category).size(18.0).color(color));
                    ui.label(
                        RichText::new(format!(
                            "AQI {}",
                            max.map(|value| value.to_string())
                                .unwrap_or_else(|| "--".into())
                        ))
                        .size(12.0)
                        .color(palette.dim),
                    );
                });
                ui.add_space(4.0);
                ui.label(
                    RichText::new(air_advice(air.summary.category.as_deref(), is_english))
                        .size(11.0)
                        .color(palette.dim),
                );
                ui.add_space(2.0);
                let mut meta = format!(
                    "{} {}",
                    air.summary.station_count,
                    if is_english {
                        "stations"
                    } else {
                        "个监测站"
                    }
                );
                let highest = air
                    .stations
                    .iter()
                    .max_by_key(|station| station.aqi.unwrap_or(0))
                    .map(|station| {
                        if is_english {
                            station
                                .name
                                .clone()
                                .unwrap_or_else(|| station.name_zh.clone())
                        } else {
                            station.name_zh.clone()
                        }
                    });
                if let Some(station) = highest {
                    meta.push_str(&format!(
                        " · {}{}",
                        if is_english { "highest " } else { "最高 " },
                        station
                    ));
                }
                ui.label(RichText::new(meta).size(10.0).color(palette.faint));
                ui.label(
                    RichText::new(format!(
                        "{}{}",
                        if is_english {
                            "Data time "
                        } else {
                            "数据时间 "
                        },
                        air.data_clock.as_deref().unwrap_or("--")
                    ))
                    .size(10.0)
                    .color(palette.faint),
                );
                provenance_line(
                    ui,
                    palette,
                    &air.source.provider,
                    air.data_clock.as_deref().or(air.updated_at.as_deref()),
                    air.quality,
                );
            });
        });
    });
}

/// Show the pollutant fields promised by the AQI screen.  EMB currently
/// exposes PM2.5/PM10 station sub-indices through the adapter; the other
/// pollutants remain explicit unavailable values rather than being inferred.
pub fn aqi_pollutants(ui: &mut egui::Ui, air: &AirQuality, palette: &Palette) {
    let is_english = english(ui);
    let max_field = |field: fn(&panahon_core::AirQualityStation) -> Option<f32>| {
        air.stations
            .iter()
            .filter_map(field)
            .max_by(|a, b| a.total_cmp(b))
    };
    let rows = [
        (
            "PM2.5",
            "PM2.5",
            max_field(|station| station.pm25),
            "AQI 子指数",
            "AQI sub-index",
        ),
        (
            "PM10",
            "PM10",
            max_field(|station| station.pm10),
            "AQI 子指数",
            "AQI sub-index",
        ),
        ("SO₂", "SO2", max_field(|station| station.so2), "", ""),
        ("NO₂", "NO2", max_field(|station| station.no2), "", ""),
        ("CO", "CO", max_field(|station| station.co), "", ""),
        ("O₃", "O3", max_field(|station| station.o3), "", ""),
    ];
    theme::card().show(ui, |ui| {
        ui.set_min_width(ui.available_width());
        card_header(
            ui,
            palette,
            if is_english {
                "Pollutants"
            } else {
                "污染物"
            },
            &air.source.provider,
        );
        ui.add_space(6.0);
        for (index, (zh, en, value, unit_zh, unit_en)) in rows.into_iter().enumerate() {
            if index > 0 {
                ui.separator();
            }
            ui.horizontal(|ui| {
                ui.label(
                    RichText::new(if is_english { en } else { zh })
                        .size(12.0)
                        .color(palette.dim),
                );
                ui.with_layout(Layout::right_to_left(Align::Center), |ui| {
                    let unit = if is_english { unit_en } else { unit_zh };
                    let text = value
                        .map(|number| {
                            if unit.is_empty() {
                                format!("{number:.1}")
                            } else {
                                format!("{number:.1} {unit}")
                            }
                        })
                        .unwrap_or_else(|| {
                            if is_english {
                                "Unavailable".into()
                            } else {
                                "暂无数据".into()
                            }
                        });
                    ui.label(RichText::new(text).size(12.0).color(if value.is_some() {
                        palette.text
                    } else {
                        palette.faint
                    }));
                });
            });
        }
        ui.add_space(4.0);
        provenance_line(
            ui,
            palette,
            &air.source.provider,
            air.data_clock.as_deref().or(air.updated_at.as_deref()),
            air.quality,
        );
    });
}

fn air_advice(category: Option<&str>, is_english: bool) -> &'static str {
    match category.unwrap_or_default() {
        "优" => {
            if is_english {
                "Air is clean; outdoor activity is fine"
            } else {
                "空气很好，适合户外活动"
            }
        }
        "良" => {
            if is_english {
                "Air is acceptable for normal activity"
            } else {
                "空气良好，可正常户外活动"
            }
        }
        "轻度污染" => {
            if is_english {
                "Sensitive groups should limit outdoor exertion"
            } else {
                "敏感人群减少户外活动"
            }
        }
        "中度污染" => {
            if is_english {
                "Reduce outdoor activity"
            } else {
                "建议减少户外活动"
            }
        }
        _ => {
            if is_english {
                "Stay indoors and wear a mask if you must go out"
            } else {
                "尽量留在室内，外出请戴口罩"
            }
        }
    }
}

/// 弧形仪表：270° 轨道加按 AQI 比例填充的进度。
fn draw_gauge(
    painter: &egui::Painter,
    rect: Rect,
    aqi: Option<u16>,
    color: Color32,
    palette: &Palette,
) {
    let center = Pos2::new(rect.center().x, rect.bottom() - 8.0);
    let radius = rect.width().min(rect.height()) * 0.42;
    let start = std::f32::consts::PI * 0.75;
    let sweep = std::f32::consts::PI * 1.5;
    let steps = 48;

    let arc = |fraction: f32| -> Vec<Pos2> {
        (0..=steps)
            .map(|step| {
                let t = step as f32 / steps as f32;
                let angle = start + sweep * fraction * t;
                center + Vec2::new(angle.cos(), angle.sin()) * radius
            })
            .collect()
    };

    painter.add(egui::Shape::line(
        arc(1.0),
        egui::Stroke::new(6.0_f32, palette.track),
    ));
    let fraction = (aqi.unwrap_or(0) as f32 / 300.0).clamp(0.03, 1.0);
    painter.add(egui::Shape::line(
        arc(fraction),
        egui::Stroke::new(6.0_f32, color),
    ));

    painter.text(
        Pos2::new(center.x, center.y - radius * 0.55),
        Align2::CENTER_CENTER,
        aqi.map(|value| value.to_string())
            .unwrap_or_else(|| "--".into()),
        FontId::proportional(22.0),
        palette.text,
    );
}
fn precip_hint(current: &CurrentWeather, is_english: bool) -> String {
    match current.precip_quality {
        DataQuality::Unavailable => {
            if is_english {
                "No official precipitation observation".to_owned()
            } else {
                "暂无官方降水观测".to_owned()
            }
        }
        quality => {
            let base = if is_english {
                precip_text_en(&current.precip_desc)
            } else {
                current.precip_desc.clone()
            };
            if quality.is_official() {
                base
            } else if is_english {
                format!("{base} · estimated")
            } else {
                format!("{base} · 参考估算")
            }
        }
    }
}

fn precip_text_en(value: &str) -> String {
    match value {
        "无降雨" => "No rain".into(),
        "微量降雨" => "Trace rain".into(),
        "小雨" => "Light rain".into(),
        "中雨" => "Moderate rain".into(),
        "大雨" => "Heavy rain".into(),
        "暴雨" => "Intense rain".into(),
        other => other.to_owned(),
    }
}
/// 2×2 observation grid.
/// 空气质量趋势：模型给出的逐小时 AQI 曲线。
pub fn aqi_trend(ui: &mut egui::Ui, trend: &[AirQualityTrendPoint], palette: &Palette) {
    let is_english = english(ui);
    let points: Vec<(&str, u16, DataQuality)> = trend
        .iter()
        .filter_map(|point| {
            point
                .aqi
                .map(|value| (point.at.as_str(), value, point.quality))
        })
        .collect();
    if points.is_empty() {
        return;
    }

    theme::card().show(ui, |ui| {
        ui.set_min_width(ui.available_width());
        card_header(
            ui,
            palette,
            if is_english {
                "Air quality trend"
            } else {
                "空气质量趋势"
            },
            if is_english {
                "Model forecast"
            } else {
                "模型预报"
            },
        );
        ui.add_space(10.0);

        let (rect, _) =
            ui.allocate_exact_size(Vec2::new(ui.available_width(), 96.0), Sense::hover());
        let low = points.iter().map(|point| point.1).min().unwrap_or(0) as f32;
        let high = points.iter().map(|point| point.1).max().unwrap_or(1) as f32;
        let span = (high - low).max(10.0);
        let painter = ui.painter();

        for step in 0..3 {
            let y = rect.top() + rect.height() * (step as f32 + 1.0) / 4.0;
            painter.line_segment(
                [Pos2::new(rect.left(), y), Pos2::new(rect.right(), y)],
                egui::Stroke::new(1.0_f32, palette.card_line),
            );
        }

        let last = points.len().max(2) - 1;
        let coords: Vec<Pos2> = points
            .iter()
            .enumerate()
            .map(|(index, (_, value, _))| {
                let x = rect.left() + rect.width() * index as f32 / last as f32;
                let y = rect.bottom() - 8.0 - (*value as f32 - low) / span * (rect.height() - 20.0);
                Pos2::new(x, y)
            })
            .collect();
        painter.add(egui::Shape::line(
            coords.clone(),
            egui::Stroke::new(2.0_f32, palette.text),
        ));
        for (point, (_, value, quality)) in coords.iter().zip(points.iter()) {
            let color = if *quality == DataQuality::Official {
                palette.text
            } else {
                aqi_color(Some(*value))
            };
            painter.circle_filled(*point, 2.6, color);
        }

        ui.add_space(2.0);
        ui.horizontal(|ui| {
            ui.label(
                RichText::new(points.first().map(|point| point.0).unwrap_or_default())
                    .size(10.0)
                    .color(palette.faint),
            );
            ui.with_layout(Layout::right_to_left(Align::Center), |ui| {
                ui.label(
                    RichText::new(points.last().map(|point| point.0).unwrap_or_default())
                        .size(10.0)
                        .color(palette.faint),
                );
            });
        });
        ui.label(
            RichText::new(if is_english {
                "Dots use the AQI band colour when the value is modelled"
            } else {
                "彩色圆点表示模型推算值"
            })
            .size(10.0)
            .color(palette.faint),
        );
    });
}

/// 本地记录的历史 AQI：每次刷新追加一个点。
pub fn aqi_history(ui: &mut egui::Ui, history: &[(String, u16)], palette: &Palette) {
    let is_english = english(ui);
    if history.is_empty() {
        return;
    }

    theme::card().show(ui, |ui| {
        ui.set_min_width(ui.available_width());
        card_header(
            ui,
            palette,
            if is_english {
                "Recorded history"
            } else {
                "历史记录"
            },
            if is_english {
                "Saved on this device"
            } else {
                "本机记录"
            },
        );
        ui.add_space(10.0);

        let (rect, _) =
            ui.allocate_exact_size(Vec2::new(ui.available_width(), 72.0), Sense::hover());
        let painter = ui.painter();
        let count = history.len().min(24);
        let slice = &history[history.len() - count..];
        let slot = rect.width() / count as f32;
        let width = (slot * 0.55).clamp(3.0, 12.0);
        for (index, (_, value)) in slice.iter().enumerate() {
            let height = (*value as f32 / 200.0).clamp(0.04, 1.0) * (rect.height() - 6.0);
            let x = rect.left() + slot * index as f32 + (slot - width) * 0.5;
            painter.rect_filled(
                Rect::from_min_max(
                    Pos2::new(x, rect.bottom() - height),
                    Pos2::new(x + width, rect.bottom()),
                ),
                egui::CornerRadius::same(2),
                aqi_color(Some(*value)),
            );
        }

        ui.add_space(4.0);
        ui.horizontal(|ui| {
            ui.label(
                RichText::new(
                    slice
                        .first()
                        .map(|entry| entry.0.as_str())
                        .unwrap_or_default(),
                )
                .size(10.0)
                .color(palette.faint),
            );
            ui.with_layout(Layout::right_to_left(Align::Center), |ui| {
                ui.label(
                    RichText::new(
                        slice
                            .last()
                            .map(|entry| entry.0.as_str())
                            .unwrap_or_default(),
                    )
                    .size(10.0)
                    .color(palette.faint),
                );
            });
        });
    });
}

/// 台风路径：列出 PAGASA 公开的路径点，并画出大致走向。
pub fn typhoon_track(ui: &mut egui::Ui, track: &[MapTrackPoint], palette: &Palette, english: bool) {
    if track.is_empty() {
        return;
    }

    theme::card().show(ui, |ui| {
        ui.set_min_width(ui.available_width());
        card_header(
            ui,
            palette,
            if english {
                "Typhoon track"
            } else {
                "台风路径"
            },
            "PAGASA",
        );
        ui.add_space(8.0);

        // 经纬度直接映射到一个小画布，只表示相对走向。
        let (rect, _) =
            ui.allocate_exact_size(Vec2::new(ui.available_width(), 120.0), Sense::hover());
        let lats: Vec<f32> = track.iter().map(|point| point.latitude).collect();
        let lons: Vec<f32> = track.iter().map(|point| point.longitude).collect();
        let min_lat = lats.iter().cloned().fold(f32::INFINITY, f32::min);
        let max_lat = lats.iter().cloned().fold(f32::NEG_INFINITY, f32::max);
        let min_lon = lons.iter().cloned().fold(f32::INFINITY, f32::min);
        let max_lon = lons.iter().cloned().fold(f32::NEG_INFINITY, f32::max);
        let lat_span = (max_lat - min_lat).max(0.5);
        let lon_span = (max_lon - min_lon).max(0.5);
        let painter = ui.painter();
        let coords: Vec<Pos2> = track
            .iter()
            .map(|point| {
                let x = rect.left()
                    + 10.0
                    + (point.longitude - min_lon) / lon_span * (rect.width() - 20.0);
                let y = rect.bottom()
                    - 10.0
                    - (point.latitude - min_lat) / lat_span * (rect.height() - 20.0);
                Pos2::new(x, y)
            })
            .collect();
        if coords.len() > 1 {
            painter.add(egui::Shape::line(
                coords.clone(),
                egui::Stroke::new(1.6_f32, palette.dim),
            ));
        }
        for (index, point) in coords.iter().enumerate() {
            let color = if index + 1 == coords.len() {
                palette.accent
            } else {
                palette.text
            };
            painter.circle_filled(
                *point,
                if index + 1 == coords.len() { 4.0 } else { 2.6 },
                color,
            );
        }

        ui.add_space(6.0);
        for point in track.iter().rev().take(4) {
            ui.horizontal(|ui| {
                ui.label(
                    RichText::new(short_time(&point.at))
                        .size(11.0)
                        .color(palette.dim),
                );
                ui.with_layout(Layout::right_to_left(Align::Center), |ui| {
                    if let Some(radius) = point.radius_km {
                        ui.label(
                            RichText::new(format!("{radius:.0} km"))
                                .size(10.0)
                                .color(palette.faint),
                        );
                    }
                    ui.label(
                        RichText::new(format!("{:.1}, {:.1}", point.latitude, point.longitude))
                            .size(10.0)
                            .color(palette.faint),
                    );
                    ui.label(RichText::new(&point.kind).size(11.0).color(palette.text));
                });
            });
        }
    });
}

pub fn detail_grid(ui: &mut egui::Ui, current: &CurrentWeather, palette: &Palette) {
    let is_english = english(ui);
    theme::card().show(ui, |ui| {
        ui.set_min_width(ui.available_width());
        let source = match current.temperature_quality {
            DataQuality::Official => {
                if is_english {
                    "Official observation"
                } else {
                    "官方观测"
                }
            }
            DataQuality::Forecast => {
                if is_english {
                    "Official forecast"
                } else {
                    "官方预报"
                }
            }
            DataQuality::Estimated => {
                if is_english {
                    "Local estimate"
                } else {
                    "本地估算"
                }
            }
            DataQuality::Unavailable => {
                if is_english {
                    "No data"
                } else {
                    "暂无数据"
                }
            }
        };
        card_header(
            ui,
            palette,
            if is_english {
                "Current observations"
            } else {
                "当前观测"
            },
            source,
        );
        ui.add_space(10.0);
        provenance_line(
            ui,
            palette,
            "PAGASA 自动站",
            Some(&current.updated_at),
            current.temperature_quality,
        );
        ui.columns(2, |columns| {
            theme::metric(
                &mut columns[0],
                palette,
                if is_english { "Humidity" } else { "湿度" },
                &if current.humidity_quality.is_available() {
                    format!("{}%", current.humidity_percent)
                } else if is_english {
                    "--".to_owned()
                } else {
                    "--".to_owned()
                },
                if is_english {
                    "Relative humidity"
                } else {
                    "相对湿度"
                },
            );
            columns[0].add_space(14.0);
            theme::metric(
                &mut columns[0],
                palette,
                if is_english { "Wind" } else { "风速" },
                &if current.wind_quality.is_available() {
                    format!("{:.0} km/h", current.wind_kph)
                } else {
                    "--".to_owned()
                },
                &if current.wind_quality.is_available() {
                    if is_english {
                        wind_dir_en(&current.wind_direction)
                    } else {
                        current.wind_direction.clone()
                    }
                } else if is_english {
                    "No wind observation".to_owned()
                } else {
                    "暂无风观测".to_owned()
                },
            );

            theme::metric(
                &mut columns[1],
                palette,
                if is_english {
                    "Precipitation"
                } else {
                    "降水"
                },
                &if current.precip_quality == DataQuality::Unavailable {
                    if is_english {
                        "Unavailable".to_owned()
                    } else {
                        "暂无".to_owned()
                    }
                } else if let Some(value) = current.precip_mm {
                    format!("{value:.1} mm")
                } else {
                    if is_english {
                        "Unavailable".to_owned()
                    } else {
                        "暂无".to_owned()
                    }
                },
                &precip_hint(current, is_english),
            );
            let pressure = current
                .pressure_hpa
                .map(|value| format!("{value:.0} hPa"))
                .unwrap_or_else(|| {
                    if is_english {
                        "Unknown".into()
                    } else {
                        "暂无".into()
                    }
                });
            theme::metric(
                &mut columns[1],
                palette,
                if is_english { "Pressure" } else { "气压" },
                &pressure,
                if is_english {
                    "Measured at the station"
                } else {
                    "观测站测得"
                },
            );
        });
        ui.add_space(8.0);
        ui.horizontal(|ui| {
            ui.label(
                RichText::new(if is_english {
                    "Feels like / heat index"
                } else {
                    "体感温度 / 热指数"
                })
                .size(12.0)
                .color(palette.dim),
            );
            ui.with_layout(Layout::right_to_left(Align::Center), |ui| {
                let value = if current.heat_index_quality.is_available() {
                    format!("{:.0}°C", current.feels_like_c)
                } else if is_english {
                    "Unavailable".into()
                } else {
                    "暂无".into()
                };
                ui.label(RichText::new(value).size(13.0).color(palette.text));
            });
        });
        provenance_line(
            ui,
            palette,
            "PAGASA",
            Some(&current.updated_at),
            current.heat_index_quality,
        );
        if let Some(station) = &current.station {
            ui.add_space(8.0);
            ui.label(
                RichText::new(if is_english {
                    format!("Station: {station}")
                } else {
                    format!("观测站：{station}")
                })
                .size(10.0)
                .color(palette.faint),
            );
        }
    });
}

/// Supplemental sun, UV and visibility details from the model forecast.
pub fn sun_details(ui: &mut egui::Ui, sun: &SunWeather, palette: &Palette) {
    if sun.uv_index.is_none()
        && sun.visibility_km.is_none()
        && sun.sunrise.is_none()
        && sun.sunset.is_none()
    {
        return;
    }
    let is_english = english(ui);
    let sunrise = sun.sunrise.clone().unwrap_or_else(|| {
        if is_english {
            "Unknown".into()
        } else {
            "暂无".into()
        }
    });
    let sunset = sun.sunset.clone().unwrap_or_else(|| {
        if is_english {
            "Unknown".into()
        } else {
            "暂无".into()
        }
    });
    let uv = sun
        .uv_index
        .map(|value| {
            if is_english {
                format!("{value:.1}")
            } else {
                format!("{value:.1}")
            }
        })
        .unwrap_or_else(|| {
            if is_english {
                "Unknown".into()
            } else {
                "暂无".into()
            }
        });
    let visibility = sun
        .visibility_km
        .map(|value| format!("{value:.1} km"))
        .unwrap_or_else(|| {
            if is_english {
                "Unknown".into()
            } else {
                "暂无".into()
            }
        });
    theme::card().show(ui, |ui| {
        ui.set_min_width(ui.available_width());
        theme::section_title(
            ui,
            if is_english {
                "Sun, UV & visibility"
            } else {
                "日照、紫外线与能见度"
            },
            palette,
        );
        ui.add_space(8.0);
        ui.columns(2, |columns| {
            theme::metric(
                &mut columns[0],
                palette,
                if is_english { "Sunrise" } else { "日出" },
                &sunrise,
                if is_english {
                    "Local time"
                } else {
                    "当地时间"
                },
            );
            columns[0].add_space(12.0);
            theme::metric(
                &mut columns[0],
                palette,
                if is_english { "Sunset" } else { "日落" },
                &sunset,
                if is_english {
                    "Local time"
                } else {
                    "当地时间"
                },
            );
            theme::metric(
                &mut columns[1],
                palette,
                if is_english { "UV index" } else { "紫外线" },
                &uv,
                if is_english {
                    "Model forecast"
                } else {
                    "模型预报"
                },
            );
            columns[1].add_space(12.0);
            theme::metric(
                &mut columns[1],
                palette,
                if is_english {
                    "Visibility"
                } else {
                    "能见度"
                },
                &visibility,
                if is_english {
                    "Model forecast"
                } else {
                    "模型预报"
                },
            );
        });
        ui.add_space(6.0);
        ui.label(
            RichText::new(if is_english {
                format!("Source: {}", sun_source_en(&sun.source))
            } else {
                format!("数据来源：{}", sun.source)
            })
            .size(10.0)
            .color(palette.faint),
        );
        provenance_line(
            ui,
            palette,
            &sun.source,
            sun.updated_at.as_deref(),
            DataQuality::Forecast,
        );
    });
}

/// 生活指数：华为用等高单元格网格，名称在上、结论在下。
///
/// 每格都显示推算依据，读者能看出结论是从哪个观测值来的。
pub fn life_indices(ui: &mut egui::Ui, indices: &[LifeIndex], palette: &Palette) {
    let is_english = english(ui);
    theme::card().show(ui, |ui| {
        ui.set_min_width(ui.available_width());
        card_header(
            ui,
            palette,
            if is_english {
                "Lifestyle"
            } else {
                "生活指数"
            },
            if is_english {
                "Local estimate"
            } else {
                "本地参考"
            },
        );
        ui.add_space(8.0);

        let count = indices.len().min(4).max(1);
        ui.columns(count, |columns| {
            for (slot, index) in columns.iter_mut().zip(indices.iter()) {
                let (title, level, advice) =
                    life_text(&index.title, &index.level, &index.advice, is_english);
                slot.allocate_ui_with_layout(
                    Vec2::new(slot.available_width(), 102.0),
                    Layout::top_down(Align::Center),
                    |ui| {
                        ui.add_space(18.0);
                        ui.label(RichText::new(&level).size(16.0).color(
                            if index.quality == DataQuality::Unavailable {
                                palette.faint
                            } else {
                                palette.text
                            },
                        ));
                        ui.add_space(2.0);
                        ui.label(RichText::new(&title).size(12.0).color(palette.dim));
                        ui.add_space(2.0);
                        ui.label(
                            RichText::new(basis_text(&index.basis, is_english))
                                .size(10.0)
                                .color(palette.faint),
                        )
                        .on_hover_text(advice);
                        ui.label(
                            RichText::new(quality_label(index.quality, is_english))
                                .size(9.0)
                                .color(palette.faint),
                        )
                        .on_hover_text(format!(
                            "{} · {}",
                            index.source,
                            updated_text(&index.updated_at, is_english)
                        ));
                    },
                );
            }
        });

        ui.add_space(2.0);
        ui.label(
            RichText::new(if is_english {
                "Computed from the current observation, not an official index."
            } else {
                "按当前观测推算，不是官方发布的生活指数。"
            })
            .size(10.0)
            .color(palette.faint),
        );
    });
}

fn basis_text(value: &str, is_english: bool) -> String {
    if !is_english {
        return value.to_owned();
    }
    let mut text = value.to_owned();
    for (zh, en) in [
        ("气温", "Temp"),
        ("体感", "Feels"),
        ("降水", "Rain"),
        ("湿度", "Humidity"),
        ("无数据", "No data"),
        ("无雨", "no rain"),
        ("有雨", "rain observed"),
        (
            "缺少完整逐小时/逐日输入",
            "complete hourly/daily inputs unavailable",
        ),
        ("未提供官方指数", "no official index"),
        ("无官方指数", "no official index"),
    ] {
        text = text.replace(zh, en);
    }
    text
}
/// One alert entry.
///
/// PAGASA alerts are shown verbatim.  The app does not silently mix a local
/// template translation with the official message; users can copy the exact
/// title/body into Google Translate or another translation app themselves.
pub fn alert_card(ui: &mut egui::Ui, alert: &Alert, palette: &Palette, english: bool) {
    theme::card().show(ui, |ui| {
        ui.set_min_width(ui.available_width());
        ui.horizontal(|ui| {
            ui.label(
                RichText::new(if alert.title_en.is_empty() {
                    alert.title.clone()
                } else {
                    alert.title_en.clone()
                })
                .size(15.0)
                .strong()
                .color(palette.text),
            );
            ui.with_layout(Layout::right_to_left(Align::Center), |ui| {
                if !alert.valid_until.is_empty() {
                    ui.label(
                        RichText::new(if english {
                            format!("Until {}", short_time(&alert.valid_until))
                        } else {
                            format!("截至 {}", short_time(&alert.valid_until))
                        })
                        .size(10.0)
                        .color(palette.faint),
                    );
                }
            });
        });
        ui.label(RichText::new(&alert.area).size(11.0).color(palette.dim));
        ui.add_space(2.0);

        ui.label(
            RichText::new(shorten(&alert.detail, 280))
                .size(12.0)
                .color(palette.dim),
        );
        ui.add_space(4.0);
        let copy_id = egui::Id::new(("alert-copied", alert.id.as_str()));
        let copied = ui
            .data(|data| data.get_temp::<bool>(copy_id))
            .unwrap_or(false);
        let copy_text = format!(
            "{}\n{}\nIssued: {}\nUntil: {}\n\n{}",
            if alert.title_en.is_empty() {
                &alert.title
            } else {
                &alert.title_en
            },
            alert.area,
            alert.issued_at,
            alert.valid_until,
            alert.detail
        );
        if ui
            .add(
                egui::Button::new(
                    RichText::new(if copied {
                        if english {
                            "Copied original"
                        } else {
                            "已复制原文"
                        }
                    } else if english {
                        "Copy original"
                    } else {
                        "复制原文"
                    })
                    .size(11.0)
                    .color(Color32::from_rgb(0xFF, 0xD1, 0x66)),
                )
                .fill(Color32::TRANSPARENT),
            )
            .clicked()
        {
            ui.ctx().copy_text(copy_text);
            ui.data_mut(|data| data.insert_temp(copy_id, true));
        }
        ui.add_space(2.0);
        ui.label(
            RichText::new(if english {
                format!("Issued {}", short_time(&alert.issued_at))
            } else {
                format!("发布于 {}", short_time(&alert.issued_at))
            })
            .size(10.0)
            .color(palette.faint),
        );
        provenance_line(
            ui,
            palette,
            if alert.source.is_empty() {
                "PAGASA"
            } else {
                &alert.source
            },
            Some(&alert.issued_at),
            alert.quality,
        );
    });
}

/// Trim PAGASA's long English timestamps down to something that fits a phone.
pub fn short_time(value: &str) -> String {
    let trimmed = value.trim();
    if trimmed.is_empty() {
        return "未知".into();
    }

    // "2026-11-08T13:00:00Z" 或 "2026-11-08 13:00:00" 都只保留到分钟
    let separator = trimmed.as_bytes().get(10).copied();
    if trimmed.len() >= 16 && matches!(separator, Some(b'T') | Some(b' ')) {
        return trimmed[..16].replace('T', " ");
    }

    // PAGASA 英文格式："November 08, 2026 12:10 PM" → "11-08 12:10"
    let parts: Vec<&str> = trimmed.split_whitespace().collect();
    if parts.len() >= 5 {
        let month = month_number(parts[0]);
        let day: u32 = parts[1].trim_end_matches(',').parse().unwrap_or(0);
        let clock = parse_clock(parts[3], parts[4]);
        if let (Some(month), true, Some((hour, minute))) = (month, day > 0, clock) {
            return format!("{month:02}-{day:02} {hour:02}:{minute:02}");
        }
    }

    trimmed.to_owned()
}

fn month_number(name: &str) -> Option<u32> {
    Some(match name {
        "January" => 1,
        "February" => 2,
        "March" => 3,
        "April" => 4,
        "May" => 5,
        "June" => 6,
        "July" => 7,
        "August" => 8,
        "September" => 9,
        "October" => 10,
        "November" => 11,
        "December" => 12,
        _ => return None,
    })
}

fn parse_clock(clock: &str, meridiem: &str) -> Option<(u32, u32)> {
    let mut pieces = clock.split(':');
    let mut hour: u32 = pieces.next()?.parse().ok()?;
    let minute: u32 = pieces.next()?.parse().ok()?;
    if meridiem.eq_ignore_ascii_case("PM") && hour != 12 {
        hour += 12;
    } else if meridiem.eq_ignore_ascii_case("AM") && hour == 12 {
        hour = 0;
    }
    Some((hour % 24, minute))
}

pub fn shorten(value: &str, limit: usize) -> String {
    let single = value.replace('\n', " ");
    let mut chars = single.chars();
    let text: String = chars.by_ref().take(limit).collect();
    if chars.next().is_some() {
        format!("{text}…")
    } else {
        text
    }
}

/// Placeholder shown while a request is in flight.
pub fn loading_row(ui: &mut egui::Ui, palette: &Palette, text: &str) {
    ui.horizontal(|ui| {
        ui.spinner();
        ui.label(RichText::new(text).size(12.0).color(palette.dim));
    });
}

/// Translucent pill button used for every action in the app.
pub fn pill_button(ui: &mut egui::Ui, text: &str, palette: &Palette) -> egui::Response {
    ui.add(
        egui::Button::new(RichText::new(text).size(12.0).color(palette.text))
            .fill(Color32::from_rgba_unmultiplied(255, 255, 255, 42))
            .stroke(egui::Stroke::new(
                1.0_f32,
                Color32::from_rgba_unmultiplied(255, 255, 255, 60),
            ))
            .corner_radius(egui::CornerRadius::same(15))
            .min_size(Vec2::new(58.0, 30.0)),
    )
}

/// Small pill used for the data-source disclaimer.
pub fn source_badge(ui: &mut egui::Ui, palette: &Palette, text: &str) {
    ui.label(RichText::new(text).size(10.0).color(palette.faint));
}

/// One translation table for provenance labels introduced by the rebuilt UI.
pub fn quality_label(quality: DataQuality, english: bool) -> &'static str {
    match (quality, english) {
        (DataQuality::Official, false) => "官方",
        (DataQuality::Forecast, false) => "官方预报",
        (DataQuality::Estimated, false) => "本地估算",
        (DataQuality::Unavailable, false) => "暂无数据",
        (DataQuality::Official, true) => "Official",
        (DataQuality::Forecast, true) => "Official forecast",
        (DataQuality::Estimated, true) => "Local estimate",
        (DataQuality::Unavailable, true) => "Unavailable",
    }
}

pub fn provenance_line(
    ui: &mut egui::Ui,
    palette: &Palette,
    source: &str,
    updated_at: Option<&str>,
    quality: DataQuality,
) {
    let english = english(ui);
    let clock = updated_at
        .filter(|value| !value.is_empty())
        .unwrap_or_else(|| if english { "--" } else { "未知时间" });
    let label = if english {
        format!("{} · {} · {}", source, quality_label(quality, true), clock)
    } else {
        format!("{} · {} · {}", source, quality_label(quality, false), clock)
    };
    source_badge(ui, palette, &label);
}
