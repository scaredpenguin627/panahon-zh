# Huawei Weather 14.0.15.301 evidence

This note records package/resource evidence from `huawei-weather-14-0-15-301.apk` before any clean-room UI rewrite. It is an observation record, not copied Huawei source or assets.

## Package and framework

- Package: `com.huawei.android.totemweather`
- Version: `14.0.15.301` / version code `140015301`
- min SDK 28, target SDK 31, compile SDK 34
- The APK contains five DEX files, `resources.arsc`, a large native arm64 set, and Android XML layouts.
- The UI is native Android View based, not Flutter: layouts use `RelativeLayout`, `LinearLayout`, `RecyclerView`, `ViewPager`, and Huawei custom widgets such as `TextViewEMUI`, `GuiTextView`, `TemperatureView`, `WeatherSunView`, `ForecastWeather`, `HwViewPager`, and `HwSubTabWidget`.
- DEX packages include Kotlin/AndroidX/Huawei UI code and custom weather classes under `com.huawei.android.totemweather.view`.

## Main activity structure

The manifest names `WeatherHome` and `WeatherMainActivity` as the primary exported activities. The main layout `weather_home.xml` contains:

1. A full-screen `WeatherHwViewPager` for city pages.
2. A bottom `HwDotsPageIndicator` for multi-city paging.
3. An update-time text view near the top.
4. An included header block for the city/location controls.
5. A top overlay region with centered city name and secondary location controls.

The single-city page `weather_single_city.xml` centers a vertical weather hero: city label, time/date/lunar rows, weather icon, numeric temperature and unit. This is the clearest evidence that Huawei's hero is vertically centered and deliberately spacious, rather than a compact top-aligned card.

## Home card order and composition

The following layout names are present in `resources.arsc` and map directly to the visible home modules:

- `weather_hourly_view` / `hourly_item`: a rounded card containing a horizontal-intercepting recycler view.
- `weather_day_forecast_layout_item_new` / `future_weather`: a rounded forecast card with a horizontally scrollable forecast view, date/condition/icon columns and a temperature range presentation.
- `weather_air_quality_layout_new`: AQI card with a reusable title include and content include.
- `weather_life_index` / `weather_life_index_item`: life-index card backed by an auto-fit recycler view.
- `weather_sun_rise_set_new`: sun card containing a dedicated `WeatherSunView` and `SunAndMoonDescriptionView`.
- `weather_featured_map_layout`: map card with Huawei `HwSubTabWidget` tabs and an `HwViewPager` below.
- `weather_warning_card_layout`, `weather_notice_layout`, `weather_wind_new`, `weather_feel`, and `weather_forecast_item`: additional modular cards used by the home composition.

Every major card uses a rounded background include (`RoundLinearLayout`/`FrameLayout`), horizontal margins, and a title include before the content. The card list is vertically scrollable and its modules are independently hideable or reorderable in the broader app (`activity_card_manage_city`, `main_page_setting_layout`).

## Resource and asset evidence

- Weather animation assets include cloud, fog, sun-ray, sun-flare, rain, snow and haze variants under `res/drawable-nodpi-v4`.
- Widget previews and multiple home/weather skins exist under `res/drawable-*` and `assets/*`.
- The APK includes `PorscheDesignFont.ttf`, `HarmonyClock_01-Bold.ttf`, and `UltimateDesignFont.ttf` in addition to Android/Huawei fonts.
- Native libraries include `libfb.so`, `libjslite.so`, `libyoga.so`, `libmmkv.so`, `libijm_linker.so`, and Huawei security/credential libraries. This suggests animation/layout/runtime helpers beyond the XML alone.

## Clean-room implementation implications

The Rust/egui implementation should reproduce the information architecture and measured spacing, not extract Huawei code/assets. The evidence supports these UI decisions:

- Keep a centered, spacious hero for city/time/temperature/icon.
- Use a vertical stack of rounded modules with consistent horizontal margins.
- Make hourly and daily forecast cards horizontally scrollable.
- Give AQI, life indices, sun details, warnings and maps their own cards.
- Treat the map as a tabbed card with a distinct content viewport.
- Preserve independent module visibility/order as a future settings capability.

The current project already implements several of these patterns in Rust/egui; this file identifies the APK-backed layout facts that should guide the next visual comparison pass.

## Measured default resource values

The default configuration reports these useful baseline dimensions:

| Resource | Value | Interpretation |
| --- | ---: | --- |
| `activity_horizontal_margin` | 16dp | Common outer page margin |
| `activity_vertical_margin` | 16dp | Common vertical page margin |
| `cardview_radius` | 16dp | Rounded module radius |
| `hourly_item_width` | 60dp | Width of each hourly forecast cell |
| `hourly_item_padding_left` | 8dp | Hourly cell leading inset |
| `hourly_item_padding_right` | 16dp | Hourly cell trailing inset |
| `forecast_padding` | 28dp | Daily forecast inner padding |
| `forecast_line_height` | 1px | Forecast divider thickness |
| `city_weather_padding_top` | 7dp | City weather content top inset |
| `city_weather_padding_bottom` | 48dp | City weather content bottom inset |

These values explain the visual rhythm more precisely than a screenshot alone: 16dp page margins, 16dp cards, 60dp hourly cells, and a larger 28dp daily-forecast inset.

## 关于"真机截图"的更正

早期版本曾把 `huawei-weather-home.png` 记为华为真机截图。核对该图后确认，它其实是本项目自己的界面
（顶部是当时的五个标签），不是华为天气。

华为天气 APK 在本机装不上：启动即抛
`ClassNotFoundException: com.huawei.android.os.SystemPropertiesEx`，因为它依赖 EMUI 框架类。
因此华为侧没有可用的真机截图，界面证据全部来自资源表与布局文件，整理在
`docs/huawei-design-spec.md`。