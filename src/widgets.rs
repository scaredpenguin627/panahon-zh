//! Screen building blocks. Every widget takes plain data plus a [`Palette`],
//! so pages stay readable and the layout is easy to tweak in one place.

use eframe::egui::{
    self, Align, Align2, Color32, FontId, Layout, Pos2, Rect, RichText, Sense, Vec2,
};
use panahon_core::{
    AirQuality, Alert, CurrentWeather, Dashboard, ForecastDay, HourlyForecast, LifeIndex,
};

use crate::icons;
use crate::theme::{self, Palette};

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
pub fn hero(ui: &mut egui::Ui, dashboard: &Dashboard, palette: &Palette, refresh: &mut bool) {
    let current = &dashboard.current;
    let background = palette.top;

    ui.horizontal(|ui| {
        ui.label(
            RichText::new(&dashboard.place.name)
                .size(26.0)
                .strong()
                .color(palette.text),
        );
        ui.label(
            RichText::new(&dashboard.place.region)
                .size(13.0)
                .color(palette.faint),
        );
        ui.with_layout(Layout::right_to_left(Align::Center), |ui| {
            if pill_button(ui, "刷新", palette).clicked() {
                *refresh = true;
            }
        });
    });
    ui.add_space(2.0);
    ui.label(
        RichText::new(format!("更新于 {}", short_time(&current.updated_at)))
            .size(12.0)
            .color(palette.faint),
    );

    ui.add_space(6.0);
    let width = ui.available_width();
    ui.horizontal(|ui| {
        ui.allocate_ui_with_layout(
            Vec2::new(width * 0.60, 112.0),
            Layout::top_down(Align::LEFT),
            |ui| {
                ui.label(
                    RichText::new(format!("{:.0}°", current.temperature_c))
                        .size(84.0)
                        .strong()
                        .color(palette.text),
                );
            },
        );
        ui.allocate_ui_with_layout(
            Vec2::new(width * 0.38, 112.0),
            Layout::top_down(Align::RIGHT),
            |ui| {
                icons::paint(ui, current.icon, current.is_day, 64.0, background);
                ui.label(
                    RichText::new(&current.condition)
                        .size(17.0)
                        .color(palette.text),
                );
                ui.label(
                    RichText::new(format!("体感 {:.0}°", current.feels_like_c))
                        .size(12.0)
                        .color(palette.dim),
                );
            },
        );
    });

    ui.add_space(2.0);
    let today = dashboard.forecast.first();
    let mut line = String::new();
    if let Some(day) = today {
        line.push_str(&format!("最高 {:.0}° / 最低 {:.0}°", day.high_c, day.low_c));
    }
    if !current.heat_index_desc.is_empty() {
        if !line.is_empty() {
            line.push_str(" · ");
        }
        line.push_str(&current.heat_index_desc);
    }
    if !line.is_empty() {
        ui.label(RichText::new(line).size(13.0).color(palette.dim));
    }
}

/// Horizontal 24-hour strip.
pub fn hourly_strip(ui: &mut egui::Ui, hours: &[HourlyForecast], palette: &Palette) {
    if hours.is_empty() {
        ui.label(
            RichText::new("暂无逐小时数据")
                .size(13.0)
                .color(palette.faint),
        );
        return;
    }

    theme::card().show(ui, |ui| {
        ui.set_min_width(ui.available_width());
        theme::section_title(ui, "未来 24 小时", palette);
        ui.add_space(4.0);
        egui::ScrollArea::horizontal()
            .auto_shrink([false, true])
            .show(ui, |ui| {
                ui.horizontal(|ui| {
                    ui.spacing_mut().item_spacing.x = 4.0;
                    for hour in hours {
                        hour_cell(ui, hour, palette);
                    }
                });
            });
    });
}

fn hour_cell(ui: &mut egui::Ui, hour: &HourlyForecast, palette: &Palette) {
    let (rect, _) = ui.allocate_exact_size(Vec2::new(58.0, 96.0), Sense::hover());
    let painter = ui.painter();
    painter.text(
        Pos2::new(rect.center().x, rect.top() + 8.0),
        Align2::CENTER_TOP,
        &hour.time,
        FontId::proportional(12.0),
        palette.dim,
    );
    icons::draw(
        painter,
        Rect::from_center_size(
            Pos2::new(rect.center().x, rect.top() + 44.0),
            Vec2::splat(38.0),
        ),
        hour.icon,
        hour.is_day,
        palette.top,
    );
    painter.text(
        Pos2::new(rect.center().x, rect.top() + 66.0),
        Align2::CENTER_TOP,
        format!("{:.0}°", hour.temperature_c),
        FontId::proportional(15.0),
        palette.text,
    );
    if hour.precip_mm > 0.05 {
        painter.text(
            Pos2::new(rect.center().x, rect.top() + 82.0),
            Align2::CENTER_TOP,
            format!("{:.1}mm", hour.precip_mm),
            FontId::proportional(10.0),
            Color32::from_rgb(0xA9, 0xD6, 0xF5),
        );
    }
}

/// Five-day list with a shared temperature range bar.
pub fn daily_list(ui: &mut egui::Ui, days: &[ForecastDay], palette: &Palette) {
    if days.is_empty() {
        return;
    }
    let low = days
        .iter()
        .map(|day| day.low_c)
        .fold(f32::INFINITY, f32::min);
    let high = days
        .iter()
        .map(|day| day.high_c)
        .fold(f32::NEG_INFINITY, f32::max);
    let span = (high - low).max(1.0);

    theme::card().show(ui, |ui| {
        ui.set_min_width(ui.available_width());
        theme::section_title(ui, "5 天预报", palette);
        ui.add_space(2.0);
        for (index, day) in days.iter().enumerate() {
            if index > 0 {
                divider(ui, palette);
            }
            day_row(ui, day, low, span, palette);
        }
    });
}

fn day_row(ui: &mut egui::Ui, day: &ForecastDay, low: f32, span: f32, palette: &Palette) {
    let bar_width = (ui.available_width() - 244.0).max(40.0);
    ui.spacing_mut().item_spacing.x = 6.0;
    ui.horizontal(|ui| {
        ui.allocate_ui_with_layout(Vec2::new(46.0, 40.0), Layout::top_down(Align::LEFT), |ui| {
            ui.set_min_height(40.0);
            ui.add_sized(
                [46.0, 20.0],
                egui::Label::new(RichText::new(&day.date).size(14.0).color(palette.text)),
            );
            if !day.date_label.is_empty() {
                ui.add_sized(
                    [46.0, 16.0],
                    egui::Label::new(
                        RichText::new(&day.date_label)
                            .size(10.0)
                            .color(palette.faint),
                    ),
                );
            }
        });

        icons::paint(ui, day.icon, true, 32.0, palette.top);

        ui.allocate_ui_with_layout(Vec2::new(56.0, 40.0), Layout::top_down(Align::LEFT), |ui| {
            ui.set_min_height(40.0);
            ui.add_sized(
                [56.0, 18.0],
                egui::Label::new(RichText::new(&day.summary).size(11.0).color(palette.dim)),
            );
            if day.precip_mm > 0.05 {
                ui.add_sized(
                    [56.0, 16.0],
                    egui::Label::new(
                        RichText::new(format!("{:.1} mm", day.precip_mm))
                            .size(10.0)
                            .color(Color32::from_rgb(0xA9, 0xD6, 0xF5)),
                    ),
                );
            }
        });

        ui.add_sized(
            [26.0, 20.0],
            egui::Label::new(
                RichText::new(format!("{:.0}°", day.low_c))
                    .size(14.0)
                    .color(palette.dim),
            ),
        );

        let (bar_rect, _) = ui.allocate_exact_size(Vec2::new(bar_width, 6.0), Sense::hover());
        let painter = ui.painter();
        painter.rect_filled(bar_rect, egui::CornerRadius::same(3), palette.track);
        let start = (day.low_c - low) / span;
        let end = (day.high_c - low) / span;
        let x0 = bar_rect.left() + bar_rect.width() * start;
        let x1 = bar_rect.left() + bar_rect.width() * end.max(start + 0.06);
        painter.rect_filled(
            Rect::from_min_max(
                Pos2::new(x0, bar_rect.top()),
                Pos2::new(x1.min(bar_rect.right()), bar_rect.bottom()),
            ),
            egui::CornerRadius::same(3),
            Color32::from_rgb(0xFF, 0xB0, 0x6A),
        );

        ui.add_sized(
            [30.0, 20.0],
            egui::Label::new(
                RichText::new(format!("{:.0}°", day.high_c))
                    .size(14.0)
                    .strong()
                    .color(palette.text),
            ),
        );
    });
}

fn divider(ui: &mut egui::Ui, palette: &Palette) {
    let (rect, _) = ui.allocate_exact_size(Vec2::new(ui.available_width(), 1.0), Sense::hover());
    ui.painter()
        .rect_filled(rect, egui::CornerRadius::ZERO, palette.card_line);
    ui.add_space(6.0);
}

/// Compact air-quality block for the weather page.
pub fn aqi_summary(ui: &mut egui::Ui, air: &AirQuality, palette: &Palette) {
    let max = air
        .summary
        .max_aqi
        .or(air.stations.first().and_then(|s| s.aqi));
    let color = aqi_text_color(max);

    theme::card().show(ui, |ui| {
        ui.set_min_width(ui.available_width());
        ui.horizontal(|ui| {
            theme::section_title(ui, "空气质量", palette);
            ui.with_layout(Layout::right_to_left(Align::Center), |ui| {
                ui.label(
                    RichText::new(format!("{} 站", air.summary.station_count))
                        .size(11.0)
                        .color(palette.faint),
                );
            });
        });
        ui.add_space(6.0);
        ui.horizontal(|ui| {
            ui.label(RichText::new("AQI").size(12.0).color(palette.faint));
            ui.label(
                RichText::new(
                    max.map(|value| value.to_string())
                        .unwrap_or_else(|| "--".into()),
                )
                .size(34.0)
                .strong()
                .color(color),
            );
            ui.vertical(|ui| {
                ui.label(
                    RichText::new(
                        air.summary
                            .category
                            .clone()
                            .unwrap_or_else(|| "暂无".into()),
                    )
                    .size(15.0)
                    .strong()
                    .color(color),
                );
                ui.label(
                    RichText::new(format!(
                        "平均 {}",
                        air.summary
                            .avg_aqi
                            .map(|value| value.to_string())
                            .unwrap_or_else(|| "--".into())
                    ))
                    .size(11.0)
                    .color(palette.faint),
                );
            });
        });

        for station in air.stations.iter().filter(|s| s.aqi.is_some()).take(3) {
            let aqi = station.aqi.unwrap_or(0);
            ui.add_space(6.0);
            ui.horizontal(|ui| {
                ui.label(
                    RichText::new(&station.name_zh)
                        .size(12.0)
                        .color(palette.dim),
                );
                ui.with_layout(Layout::right_to_left(Align::Center), |ui| {
                    ui.add_sized(
                        [26.0, 14.0],
                        egui::Label::new(
                            RichText::new(aqi.to_string())
                                .size(12.0)
                                .strong()
                                .color(aqi_text_color(station.aqi)),
                        ),
                    );
                    station_bar(ui, aqi, palette);
                });
            });
        }

        ui.add_space(8.0);
        ui.label(
            RichText::new(format!(
                "{} · {}",
                air.source.provider,
                air.data_clock.as_deref().unwrap_or("时间未知")
            ))
            .size(10.0)
            .color(palette.faint),
        );
    });
}
fn station_bar(ui: &mut egui::Ui, aqi: u16, palette: &Palette) {
    let (rect, _) = ui.allocate_exact_size(Vec2::new(80.0, 5.0), Sense::hover());
    let painter = ui.painter();
    painter.rect_filled(rect, egui::CornerRadius::same(2), palette.track);
    let fraction = (aqi as f32 / 200.0).clamp(0.06, 1.0);
    painter.rect_filled(
        Rect::from_min_max(
            rect.left_top(),
            Pos2::new(rect.left() + rect.width() * fraction, rect.bottom()),
        ),
        egui::CornerRadius::same(2),
        aqi_color(Some(aqi)),
    );
}

/// 2×2 observation grid.
pub fn detail_grid(ui: &mut egui::Ui, current: &CurrentWeather, palette: &Palette) {
    theme::card().show(ui, |ui| {
        ui.set_min_width(ui.available_width());
        theme::section_title(ui, "当前观测", palette);
        ui.add_space(10.0);
        ui.columns(2, |columns| {
            theme::metric(
                &mut columns[0],
                palette,
                "湿度",
                &format!("{}%", current.humidity_percent),
                "相对湿度",
            );
            columns[0].add_space(14.0);
            theme::metric(
                &mut columns[0],
                palette,
                "风速",
                &format!("{:.0} km/h", current.wind_kph),
                &current.wind_direction,
            );

            theme::metric(
                &mut columns[1],
                palette,
                "降水",
                &format!("{:.1} mm", current.precip_mm),
                &current.precip_desc,
            );
            columns[1].add_space(14.0);
            let pressure = current
                .pressure_hpa
                .map(|value| format!("{value:.0} hPa"))
                .unwrap_or_else(|| "暂无".into());
            theme::metric(&mut columns[1], palette, "气压", &pressure, "海平面气压");
        });
        if let Some(station) = &current.station {
            ui.add_space(8.0);
            ui.label(
                RichText::new(format!("观测站：{station}"))
                    .size(10.0)
                    .color(palette.faint),
            );
        }
    });
}

/// Four locally derived suggestions.
pub fn life_indices(ui: &mut egui::Ui, indices: &[LifeIndex], palette: &Palette) {
    theme::card().show(ui, |ui| {
        ui.set_min_width(ui.available_width());
        theme::section_title(ui, "生活指数", palette);
        ui.add_space(6.0);
        for index in indices {
            ui.horizontal(|ui| {
                ui.label(RichText::new(&index.title).size(13.0).color(palette.dim));
                ui.label(
                    RichText::new(&index.level)
                        .size(13.0)
                        .strong()
                        .color(palette.accent),
                );
                ui.with_layout(Layout::right_to_left(Align::Center), |ui| {
                    ui.label(RichText::new(&index.advice).size(11.0).color(palette.faint));
                });
            });
        }
    });
}

/// One alert entry.
pub fn alert_card(ui: &mut egui::Ui, alert: &Alert, palette: &Palette) {
    theme::card().show(ui, |ui| {
        ui.set_min_width(ui.available_width());
        ui.horizontal(|ui| {
            ui.label(
                RichText::new(&alert.title)
                    .size(15.0)
                    .strong()
                    .color(palette.text),
            );
            ui.with_layout(Layout::right_to_left(Align::Center), |ui| {
                if !alert.valid_until.is_empty() {
                    ui.label(
                        RichText::new(format!("至 {}", short_time(&alert.valid_until)))
                            .size(10.0)
                            .color(palette.faint),
                    );
                }
            });
        });
        ui.label(RichText::new(&alert.area).size(11.0).color(palette.dim));
        ui.add_space(2.0);
        ui.label(
            RichText::new(shorten(&alert.detail, 160))
                .size(12.0)
                .color(palette.dim),
        );
        ui.add_space(2.0);
        ui.label(
            RichText::new(format!("防御建议：{}", alert.defense))
                .size(11.0)
                .color(palette.accent),
        );
        ui.label(
            RichText::new(format!("发布于 {}", short_time(&alert.issued_at)))
                .size(10.0)
                .color(palette.faint),
        );
    });
}

/// Bottom tab bar. Returns the index the user tapped, if any.
pub fn bottom_nav(
    ui: &mut egui::Ui,
    active: usize,
    palette: &Palette,
    bottom_inset: f32,
) -> Option<usize> {
    const LABELS: [&str; 5] = ["天气", "空气", "地图", "预警", "我的"];
    let mut tapped = None;
    let rect = ui.max_rect();
    let usable_height = (rect.height() - bottom_inset).max(48.0);
    let icon_center_y = rect.top() + usable_height * 0.36;
    let label_top = rect.top() + usable_height * 0.60;

    ui.painter().rect_filled(
        Rect::from_min_max(
            Pos2::new(rect.left(), rect.top()),
            Pos2::new(rect.right(), rect.bottom()),
        ),
        egui::CornerRadius::ZERO,
        Color32::from_rgba_unmultiplied(0, 0, 0, 56),
    );
    ui.painter().line_segment(
        [rect.left_top(), rect.right_top()],
        egui::Stroke::new(1.0_f32, palette.card_line),
    );

    let cell = rect.width() / LABELS.len() as f32;
    for (index, label) in LABELS.iter().enumerate() {
        let cell_rect = Rect::from_min_size(
            Pos2::new(rect.left() + cell * index as f32, rect.top()),
            Vec2::new(cell, usable_height),
        );
        let response = ui.interact(cell_rect, ui.id().with(("nav", index)), Sense::click());
        let selected = index == active;
        let color = if selected {
            palette.text
        } else {
            palette.faint
        };
        nav_icon(
            ui.painter(),
            Rect::from_center_size(
                Pos2::new(cell_rect.center().x, icon_center_y),
                Vec2::splat(22.0),
            ),
            index,
            color,
        );
        ui.painter().text(
            Pos2::new(cell_rect.center().x, label_top),
            Align2::CENTER_TOP,
            label,
            FontId::proportional(11.0),
            color,
        );
        if selected {
            ui.painter().circle_filled(
                Pos2::new(cell_rect.center().x, label_top - 6.0),
                1.8,
                palette.accent,
            );
        }
        if response.clicked() {
            tapped = Some(index);
        }
    }
    tapped
}

fn nav_icon(painter: &egui::Painter, rect: Rect, index: usize, color: Color32) {
    let center = rect.center();
    let s = rect.width();
    let stroke = egui::Stroke::new(1.8_f32, color);
    match index {
        0 => {
            painter.circle_filled(center, s * 0.17, color);
            for step in 0..8 {
                let angle = std::f32::consts::TAU * step as f32 / 8.0;
                let direction = Vec2::new(angle.cos(), angle.sin());
                painter.line_segment(
                    [
                        center + direction * (s * 0.30),
                        center + direction * (s * 0.40),
                    ],
                    stroke,
                );
            }
        }
        1 => {
            // 表盘 + 指针，代表空气质量指数
            let radius = s * 0.32;
            let center = center + Vec2::new(0.0, s * 0.06);
            let mut points = Vec::new();
            for step in 0..=18 {
                let angle =
                    std::f32::consts::PI * 1.12 + std::f32::consts::PI * 0.76 * step as f32 / 18.0;
                points.push(center + Vec2::new(-angle.cos(), -angle.sin()) * radius);
            }
            painter.add(egui::Shape::line(points, stroke));
            painter.line_segment(
                [center, center + Vec2::new(radius * 0.62, -radius * 0.52)],
                stroke,
            );
            painter.circle_filled(center, s * 0.045, color);
        }
        2 => {
            let left = center - Vec2::new(s * 0.30, s * 0.22);
            let right = center + Vec2::new(s * 0.30, s * 0.22);
            let frame = Rect::from_min_max(left, right);
            painter.rect_stroke(
                frame,
                egui::CornerRadius::same(3),
                stroke,
                egui::StrokeKind::Inside,
            );
            painter.line_segment(
                [
                    Pos2::new(center.x - s * 0.08, frame.top()),
                    Pos2::new(center.x - s * 0.08, frame.bottom()),
                ],
                stroke,
            );
            painter.line_segment(
                [
                    Pos2::new(center.x + s * 0.12, frame.top()),
                    Pos2::new(center.x + s * 0.12, frame.bottom()),
                ],
                stroke,
            );
        }
        3 => {
            // 三角形警告牌 + 感叹号，比铃铛更容易一眼认出来
            let top = Pos2::new(center.x, center.y - s * 0.30);
            let left = Pos2::new(center.x - s * 0.32, center.y + s * 0.24);
            let right = Pos2::new(center.x + s * 0.32, center.y + s * 0.24);
            painter.add(egui::Shape::convex_polygon(
                vec![top, right, left],
                Color32::TRANSPARENT,
                stroke,
            ));
            painter.line_segment(
                [
                    Pos2::new(center.x, center.y - s * 0.10),
                    Pos2::new(center.x, center.y + s * 0.10),
                ],
                stroke,
            );
            painter.circle_filled(Pos2::new(center.x, center.y + s * 0.19), s * 0.035, color);
        }
        _ => {
            painter.circle_stroke(Pos2::new(center.x, center.y - s * 0.12), s * 0.14, stroke);
            painter.rect_stroke(
                Rect::from_min_max(
                    Pos2::new(center.x - s * 0.24, center.y + s * 0.04),
                    Pos2::new(center.x + s * 0.24, center.y + s * 0.36),
                ),
                egui::CornerRadius::same(9),
                stroke,
                egui::StrokeKind::Inside,
            );
        }
    }
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
