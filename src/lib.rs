#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

use eframe::egui::{self, Align, Color32, CornerRadius, Layout, RichText, Vec2};
use panahon_core::{
    fetch_api_version, fetch_public_dashboard, search_places, Dashboard, DataSource, ForecastDay,
    HourlyForecast, Place,
};

#[derive(Clone, Copy, PartialEq, Eq)]
enum Tab {
    Places,
    Forecast,
    Map,
    Alerts,
    Services,
}

impl Tab {
    const ALL: [Self; 5] = [
        Self::Places,
        Self::Forecast,
        Self::Map,
        Self::Alerts,
        Self::Services,
    ];

    fn title(self) -> &'static str {
        match self {
            Self::Places => "地点",
            Self::Forecast => "预报",
            Self::Map => "地图",
            Self::Alerts => "预警",
            Self::Services => "服务",
        }
    }
}

#[derive(Clone, Copy, PartialEq, Eq)]
enum MapLayer {
    Radar,
    Satellite,
    Lightning,
    Typhoon,
    Hazard,
}

impl MapLayer {
    const ALL: [Self; 5] = [
        Self::Radar,
        Self::Satellite,
        Self::Lightning,
        Self::Typhoon,
        Self::Hazard,
    ];

    fn title(self) -> &'static str {
        match self {
            Self::Radar => "雷达",
            Self::Satellite => "卫星",
            Self::Lightning => "闪电",
            Self::Typhoon => "台风",
            Self::Hazard => "灾害风险",
        }
    }
}

pub struct PanahonApp {
    tab: Tab,
    dashboard: Dashboard,
    status: String,
    last_version: Option<String>,
    search_query: String,
    search_results: Vec<Place>,
    map_layer: MapLayer,
}

impl PanahonApp {
    pub fn new(cc: &eframe::CreationContext<'_>) -> Self {
        install_cjk_font(&cc.egui_ctx);
        let mut style = (*cc.egui_ctx.style()).clone();
        style.spacing.item_spacing = Vec2::new(10.0, 8.0);
        style.visuals.window_corner_radius = CornerRadius::same(18);
        style.visuals.widgets.noninteractive.corner_radius = CornerRadius::same(12);
        style.visuals.widgets.inactive.corner_radius = CornerRadius::same(12);
        style.visuals.widgets.hovered.corner_radius = CornerRadius::same(12);
        style.visuals.widgets.active.corner_radius = CornerRadius::same(12);
        cc.egui_ctx.set_style(style);
        Self {
            tab: Tab::Forecast,
            dashboard: Dashboard::sample(),
            status: "离线数据".into(),
            last_version: None,
            search_query: String::new(),
            search_results: Vec::new(),
            map_layer: MapLayer::Radar,
        }
    }

    fn header(&mut self, ui: &mut egui::Ui) {
        #[cfg(target_os = "android")]
        ui.add_space(28.0);
        ui.horizontal(|ui| {
            ui.label(
                RichText::new("PANaHON")
                    .size(26.0)
                    .strong()
                    .color(Color32::from_rgb(12, 91, 128)),
            );
            ui.label(
                RichText::new("中文天气")
                    .size(16.0)
                    .color(Color32::from_rgb(75, 95, 108)),
            );
            ui.with_layout(Layout::right_to_left(Align::Center), |ui| {
                if ui.button("刷新").clicked() {
                    self.refresh_data();
                }
            });
        });
        ui.separator();
    }

    fn tab_bar(&mut self, ui: &mut egui::Ui) {
        ui.horizontal_wrapped(|ui| {
            for tab in Tab::ALL {
                let selected = self.tab == tab;
                let label = if selected {
                    RichText::new(tab.title()).strong().color(Color32::WHITE)
                } else {
                    RichText::new(tab.title()).color(Color32::from_rgb(40, 61, 73))
                };
                let fill = if selected {
                    Color32::from_rgb(12, 112, 145)
                } else {
                    Color32::from_rgb(225, 239, 243)
                };
                if ui
                    .add(
                        egui::Button::new(label)
                            .fill(fill)
                            .min_size(Vec2::new(76.0, 36.0)),
                    )
                    .clicked()
                {
                    self.tab = tab;
                }
            }
        });
        ui.add_space(8.0);
    }

    fn forecast_page(&mut self, ui: &mut egui::Ui) {
        let current = &self.dashboard.current;
        ui.group(|ui| {
            ui.label(
                RichText::new(&self.dashboard.place.name)
                    .size(20.0)
                    .strong(),
            );
            ui.label(RichText::new(&self.dashboard.place.region).color(Color32::GRAY));
            ui.add_space(4.0);
            ui.horizontal(|ui| {
                ui.label(
                    RichText::new(format!("{:.0} °C", current.temperature_c))
                        .size(52.0)
                        .strong(),
                );
                ui.vertical(|ui| {
                    ui.label(RichText::new(&current.condition).size(18.0));
                    ui.label(format!(
                        "体感 {:.0} °C · 湿度 {}%",
                        current.feels_like_c, current.humidity_percent
                    ));
                    ui.label(format!(
                        "风速 {:.0} km/h · 风向 {}",
                        current.wind_kph, current.wind_direction
                    ));
                });
            });
            ui.label(
                RichText::new(format!("更新时间：{}", current.updated_at))
                    .small()
                    .color(Color32::GRAY),
            );
        });
        ui.add_space(8.0);
        ui.label(RichText::new("五日预报").size(18.0).strong());
        for day in &self.dashboard.forecast {
            forecast_row(ui, day);
        }
        ui.add_space(8.0);
        ui.label(RichText::new("逐小时预报").size(18.0).strong());
        if self.dashboard.hourly.is_empty() {
            ui.label("当前接口没有返回逐小时数据。");
        } else {
            for hour in &self.dashboard.hourly {
                hourly_row(ui, hour);
            }
        }
        ui.add_space(8.0);
        if ui.button("刷新天气数据").clicked() {
            self.refresh_data();
        }
        ui.add_space(8.0);
        source_badge(
            ui,
            &self.dashboard.source,
            &self.status,
            self.last_version.as_deref(),
        );
    }

    fn refresh_data(&mut self) {
        let place = self.dashboard.place.clone();
        if let Ok(version) = fetch_api_version() {
            self.last_version = Some(version.latest_version);
        }
        match fetch_public_dashboard(&place) {
            Ok(dashboard) => {
                self.dashboard = dashboard;
                self.status = "已更新公开天气数据".into();
            }
            Err(error) => self.status = format!("刷新失败，继续显示当前数据：{error}"),
        }
    }

    fn places_page(&mut self, ui: &mut egui::Ui) {
        ui.heading("地点");
        ui.label("保存常用地点。切换地点后，点击“刷新”加载天气。");
        ui.add_space(8.0);
        ui.horizontal(|ui| {
            ui.add_sized(
                [250.0, 32.0],
                egui::TextEdit::singleline(&mut self.search_query),
            );
            if ui.button("搜索").clicked() {
                match search_places(&self.search_query) {
                    Ok(results) => {
                        self.status = format!("找到 {} 个地点", results.len());
                        self.search_results = results;
                    }
                    Err(error) => self.status = format!("地点搜索失败：{error}"),
                }
            }
        });
        for place in self.search_results.clone() {
            if ui
                .button(format!("{} · {}", place.name, place.region))
                .clicked()
            {
                self.dashboard.place = place;
                self.status = "已切换地点；点击刷新加载天气".into();
            }
        }
        ui.add_space(10.0);
        ui.group(|ui| {
            ui.label(RichText::new("当前地点").strong());
            ui.label(&self.dashboard.place.name);
            ui.label(format!(
                "坐标：{:.4}，{:.4}",
                self.dashboard.place.latitude, self.dashboard.place.longitude
            ));
            ui.label("定位权限仅在需要时申请。GrapheneOS 可以关闭应用的网络权限。");
        });
        ui.add_space(8.0);
        ui.label("示例地点");
        for place in ["宿务", "达沃", "碧瑶", "长滩岛", "香港"] {
            if ui.button(place).clicked() {
                self.status = format!("已选择 {place}。当前显示离线数据。");
            }
        }
    }

    fn map_page(&mut self, ui: &mut egui::Ui) {
        ui.heading("天气地图");
        ui.label("选择图层后，应用会显示数据来源和接入状态。");
        ui.add_space(8.0);
        ui.horizontal_wrapped(|ui| {
            for layer in MapLayer::ALL {
                let selected = self.map_layer == layer;
                let fill = if selected {
                    Color32::from_rgb(12, 112, 145)
                } else {
                    Color32::from_rgb(225, 239, 243)
                };
                let color = if selected {
                    Color32::WHITE
                } else {
                    Color32::from_rgb(40, 61, 73)
                };
                if ui
                    .add(egui::Button::new(RichText::new(layer.title()).color(color)).fill(fill))
                    .clicked()
                {
                    self.map_layer = layer;
                }
            }
        });
        ui.add_space(8.0);
        let layer_status = match self.map_layer {
            MapLayer::Radar => ("雷达图层", "PAGASA 公开雷达图层待接入。"),
            MapLayer::Satellite => ("卫星图层", "PAGASA Himawari/GK2A 图层待接入。"),
            MapLayer::Lightning => ("闪电图层", "实时闪电和 10 分钟闪电图层待接入。"),
            MapLayer::Typhoon => ("台风图层", "台风路径和风圈图层待接入。"),
            MapLayer::Hazard => ("灾害风险图层", "洪水、风暴潮和灾害风险图层待接入。"),
        };
        ui.label(RichText::new(layer_status.0).strong());
        ui.label(layer_status.1);
        ui.group(|ui| {
            ui.allocate_ui_with_layout(
                Vec2::new(ui.available_width(), 170.0),
                Layout::centered_and_justified(egui::Direction::TopDown),
                |ui| {
                    ui.label(
                        RichText::new("交互地图\n\n当前只展示图层状态，不虚构远程数据")
                            .size(20.0)
                            .color(Color32::from_rgb(76, 106, 121)),
                    );
                },
            );
        });
        ui.add_space(8.0);
        ui.label("数据源：PAGASA 官方公开页面。联网和许可确认后再加载远程图层。");
    }

    fn alerts_page(&mut self, ui: &mut egui::Ui) {
        ui.heading("预警");
        ui.label("联网后刷新预警列表。");
        ui.add_space(8.0);
        for alert in &self.dashboard.alerts {
            ui.group(|ui| {
                ui.horizontal(|ui| {
                    ui.label(RichText::new("●").color(Color32::from_rgb(222, 135, 41)));
                    ui.label(RichText::new(&alert.title).strong());
                    ui.with_layout(Layout::right_to_left(Align::Center), |ui| {
                        ui.label(
                            RichText::new(&alert.severity).color(Color32::from_rgb(170, 94, 34)),
                        );
                    });
                });
                ui.label(&alert.area);
                ui.label(&alert.detail);
                ui.label(
                    RichText::new(format!("防御建议：{}", alert.defense))
                        .color(Color32::from_rgb(192, 155, 96)),
                );
                ui.label(RichText::new(&alert.issued_at).small().color(Color32::GRAY));
            });
            ui.add_space(6.0);
        }
        ui.group(|ui| {
            ui.label(RichText::new("本地通知").strong());
            ui.label("当前状态：未启用");
            ui.label("后续可由 Android 本地调度器发送，不依赖 FCM。需要用户主动允许通知权限。");
        });
    }

    fn services_page(&mut self, ui: &mut egui::Ui) {
        ui.heading("服务");
        ui.label("这些模块已预留界面边界。实时数据需要独立来源和许可。");
        ui.add_space(8.0);
        let services = [
            (
                "空气质量",
                "DENR-EMB 授权接口优先，OpenAQ 作为可选适配器。",
                "需要数据许可",
            ),
            ("旅游天气", "旅游区未来一周天气。", "公开接口待接入"),
            ("生活指数", "紫外线、穿衣、出行和户外活动建议。", "规则样例"),
            (
                "海洋天气",
                "海域风、浪高、水温和海上预警。",
                "公开数据待接入",
            ),
            (
                "农业天气",
                "降雨、热害、干旱和农业气象提示。",
                "公开数据待接入",
            ),
            ("交通天气", "能见度、降雨和道路风险提示。", "公开数据待接入"),
            (
                "水利汛情",
                "河流水位、洪水风险和防汛信息。",
                "公开数据待接入",
            ),
        ];
        for (title, summary, status) in services {
            ui.group(|ui| {
                ui.horizontal(|ui| {
                    ui.label(RichText::new(title).strong());
                    ui.with_layout(Layout::right_to_left(Align::Center), |ui| {
                        ui.label(
                            RichText::new(status)
                                .small()
                                .color(Color32::from_rgb(183, 141, 71)),
                        );
                    });
                });
                ui.label(summary);
            });
            ui.add_space(5.0);
        }
    }
}

fn install_cjk_font(ctx: &egui::Context) {
    let mut fonts = egui::FontDefinitions::default();
    fonts.font_data.insert(
        "noto_sans_cjk".into(),
        egui::FontData::from_static(include_bytes!("../assets/NotoSansCJKsc-Subset.otf")).into(),
    );
    for family in [egui::FontFamily::Proportional, egui::FontFamily::Monospace] {
        fonts
            .families
            .entry(family)
            .or_default()
            .insert(0, "noto_sans_cjk".into());
    }
    ctx.set_fonts(fonts);
}

impl eframe::App for PanahonApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        egui::CentralPanel::default().show(ctx, |ui| {
            self.header(ui);
            self.tab_bar(ui);
            match self.tab {
                Tab::Places => self.places_page(ui),
                Tab::Forecast => self.forecast_page(ui),
                Tab::Map => self.map_page(ui),
                Tab::Alerts => self.alerts_page(ui),
                Tab::Services => self.services_page(ui),
            }
            ui.with_layout(Layout::bottom_up(Align::Center), |ui| {
                ui.add_space(4.0);
                ui.label(
                    RichText::new("公开数据 · Rust · 适用于 GrapheneOS")
                        .small()
                        .color(Color32::GRAY),
                );
            });
        });
    }
}

fn forecast_row(ui: &mut egui::Ui, day: &ForecastDay) {
    ui.horizontal(|ui| {
        ui.add_sized(
            [54.0, 20.0],
            egui::Label::new(RichText::new(&day.date).strong()),
        );
        ui.add_sized([110.0, 20.0], egui::Label::new(&day.summary));
        ui.add_sized(
            [70.0, 20.0],
            egui::Label::new(format!("{:.0} °C / {:.0} °C", day.low_c, day.high_c)),
        );
        ui.label(format!("降雨 {}%", day.rain_probability));
    });
}

fn hourly_row(ui: &mut egui::Ui, hour: &HourlyForecast) {
    ui.horizontal(|ui| {
        ui.add_sized(
            [84.0, 20.0],
            egui::Label::new(RichText::new(&hour.time).strong()),
        );
        ui.add_sized([130.0, 20.0], egui::Label::new(&hour.summary));
        ui.add_sized(
            [70.0, 20.0],
            egui::Label::new(format!("{:.0} °C", hour.temperature_c)),
        );
        ui.label(format!("降雨 {}%", hour.rain_probability));
    });
}

fn source_badge(ui: &mut egui::Ui, source: &DataSource, status: &str, version: Option<&str>) {
    let label = match source {
        DataSource::OfflineSample => "数据来源：离线样例",
        DataSource::PublicApi => "数据来源：公开接口",
    };
    ui.separator();
    ui.label(RichText::new(label).color(Color32::from_rgb(58, 108, 83)));
    ui.label(RichText::new(status).small().color(Color32::GRAY));
    if let Some(version) = version {
        ui.label(
            RichText::new(format!("公开服务版本：{version}"))
                .small()
                .color(Color32::GRAY),
        );
    }
}

#[cfg(target_os = "android")]
#[no_mangle]
pub fn android_main(app: winit::platform::android::activity::AndroidApp) {
    let options = eframe::NativeOptions {
        android_app: Some(app),
        ..Default::default()
    };
    let result = eframe::run_native(
        "PANaHON 中文天气",
        options,
        Box::new(|cc| Ok(Box::new(PanahonApp::new(cc)))),
    );
    if let Err(error) = result {
        eprintln!("Android UI failed to start: {error}");
    }
}

