//! Live smoke test for the network adapters.
//!
//! ```powershell
//! cargo run -p panahon-core --example live --features network
//! ```
//!
//! Prints one line per source so a broken upstream is obvious at a glance.

use panahon_core::{fetch_air_quality_at, fetch_public_dashboard, Place};

fn main() {
    let place = Place {
        id: "manila".into(),
        name: "马尼拉".into(),
        region: "国家首都区".into(),
        latitude: 14.5995,
        longitude: 120.9842,
    };

    match fetch_public_dashboard(&place) {
        Ok(dashboard) => {
            println!(
                "天气 OK：{} {:.0}°C {} · 体感 {:.0}°C · {}% · {:.0} km/h {} · 降水 {:.2} mm {} · 站点 {}",
                dashboard.place.name,
                dashboard.current.temperature_c,
                dashboard.current.condition,
                dashboard.current.feels_like_c,
                dashboard.current.humidity_percent,
                dashboard.current.wind_kph,
                dashboard.current.wind_direction,
                dashboard.current.precip_mm,
                dashboard.current.precip_desc,
                dashboard.current.station.as_deref().unwrap_or("-")
            );
            println!(
                "  图标 {:?} · 白天 {} · 气压 {:?} · 辐射 {:?}",
                dashboard.current.icon,
                dashboard.current.is_day,
                dashboard.current.pressure_hpa,
                dashboard.current.solar_wm2
            );
            for day in &dashboard.forecast {
                println!(
                    "  {} {} {}~{}°C {:.1}mm {:?}",
                    day.date,
                    day.date_label,
                    day.low_c
                        .map(|v| format!("{v:.0}"))
                        .unwrap_or_else(|| "—".into()),
                    day.high_c
                        .map(|v| format!("{v:.0}"))
                        .unwrap_or_else(|| "—".into()),
                    day.precip_mm,
                    day.icon
                );
            }
            println!("  逐小时 {} 条，前 3 条：", dashboard.hourly.len());
            for hour in dashboard.hourly.iter().take(3) {
                println!(
                    "    {} {}°C {:.1}mm {:?} day={}",
                    hour.time,
                    hour.temperature_c
                        .map(|v| format!("{v:.0}"))
                        .unwrap_or_else(|| "—".into()),
                    hour.precip_mm,
                    hour.icon,
                    hour.is_day
                );
            }
            println!("  预警 {} 条", dashboard.alerts.len());
            for alert in dashboard.alerts.iter().take(2) {
                println!(
                    "    [{}] {} · {} · 至 {}",
                    alert.kind, alert.title, alert.area, alert.valid_until
                );
            }
            for index in dashboard.life_indices() {
                println!("  指数 {}：{} — {}", index.title, index.level, index.advice);
            }
        }
        Err(error) => println!("天气 FAIL：{error}"),
    }

    match fetch_air_quality_at(place.latitude, place.longitude) {
        Ok(air) => {
            println!(
                "\n空气 OK：{} · {} 站 · 平均 {:?} · 最差 {:?} {} · 时间 {:?}",
                air.source.provider,
                air.summary.station_count,
                air.summary.avg_aqi,
                air.summary.max_aqi,
                air.summary.max_station.as_deref().unwrap_or("-"),
                air.data_clock
            );
            println!("  说明：{}", air.source.note);
            for station in air.stations.iter().take(5) {
                println!(
                    "    {:<16} AQI {:>4} {} PM10 {:?} PM2.5 {:?} {}",
                    station.name_zh,
                    station
                        .aqi
                        .map(|v| v.to_string())
                        .unwrap_or_else(|| "-".into()),
                    station.category,
                    station.pm10,
                    station.pm25,
                    station.pollutant.as_deref().unwrap_or("")
                );
            }
        }
        Err(error) => println!("\n空气 FAIL：{error}"),
    }
}
