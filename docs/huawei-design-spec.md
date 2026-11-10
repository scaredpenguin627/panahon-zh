# 华为天气界面规格（从 APK 资源提取）

本文记录从 `huawei-weather-14-0-15-301.apk` 解包得到的界面事实，作为 Rust/egui 重做的依据。只记录结构和数值，不复制华为的代码、图片、字体和着色器。

## 取证方式与限制

- 解包：`apktool 2.11.1`，命令 `apktool d -f -q <apk> -o hw-decode --no-src`。
- 资源值：`aapt2 dump resources`，输出 74 984 行，覆盖全部 dimen/color/drawable/style。
- 布局：直接读取 `res/layout*/` 下解码后的 XML。
- **无法运行**：APK 安装到 Pixel 10a 后启动即崩，`java.lang.ClassNotFoundException: com.huawei.android.os.SystemPropertiesEx`。它依赖 EMUI 框架类，在 GrapheneOS 上跑不起来，所以没有可用的真机截图。本文的数值全部来自资源，比截图更精确。
- apktool 缺华为 framework（package id 2），部分 `@style` 引用无法解析。因此只采用能读到具体值的 dimen、color 和布局属性；字号使用相对层级，不照抄不可见的 sp 值。

## 页面结构

主界面 `weather_home.xml`：

- 顶部是城市名区域，`weather_home_city_container_marginTop` = 56dp。
- 中间是城市页 `WeatherHwViewPager`，每个城市一页。
- 底部是城市指示点 `HwDotsPageIndicator`，高 32dp。

单个城市页 `home_city_weather.xml`：

1. 当前天气区 `CurrentWeather`，垂直居中，最小高 `weather_current_Weather_height` = 268dp（高屏 300dp）。
2. 城市名，居中，最大宽 `cityname_max_width` = 250dp。
3. 大号温度 `CurrentTempView`。
4. 最高/最低温 `TempDiffColorView`，副标题字号。
5. 一行"天气现象 · 空气质量"，字号正文 1。
6. 可选告警横条，高 30dp，最多 56dp。

城市页本身是 `ListView`（`city_weather_listview.xml`），所以卡片是纵向列表，顺序由适配器决定。可用的卡片布局及其内部结构如下。

## 卡片通用样式

| 项目 | 值 | 来源 |
| --- | --- | --- |
| 卡片外边距 | 12dp（上、左、右） | `weather_life_index.xml` 等 |
| 卡片圆角 | 16dp | `bg_white_masking.xml` |
| 卡片底色 | `#19FFFFFF`（10% 白） | `white_10_percent_color` |
| 卡片标题字号 | 12sp | `days_forecast_header_view.xml` |
| 卡片标题颜色 | `#99FFFFFF`（60% 白） | `white_60_percent_color` |
| 卡片标题内边距 | 上 14dp，左 16dp | `days_forecast_header_view.xml` |
| 分隔线 | 1px | `forecast_line_height` |

## 逐小时

`weather_hourly_view.xml` 是圆角容器加一个横向 RecyclerView。单元格 `hourly_item.xml`：

- 最小宽 72dp，上下结构：时间 → 图标 → 温度。
- 时间：上边距 16dp，正文字号 3。
- 图标：28dp；图标下方是降水概率，宽 28dp，上边距 2dp，字号 4。
- 温度：上边距 8dp，下内边距 16dp，正文 1，`sans-serif-medium`。
- 设计资源里另有一份 `hourly_item_width` = 60dp，属于另一种皮肤；实现取 72dp。

## 多日天气

`weather_day_forecast_layout_item_new.xml` 是卡片容器，内含标题 `days_forecast_header_view` 和横向滚动的 `ForecastWeather`。单行 `future_weather.xml`：

- 行最小高 `weather_day_forecast_height` = 48dp。
- 左侧：星期 + 日期，两段文本，起始对齐。
- 中间：图标 24dp，下方降水概率，上边距 2dp。
- 右侧：温度，结束对齐。
- 卡片内边距 `forecast_padding` = 28dp。

## 空气质量

`weather_air_quality_layout_new.xml` 只做卡片外壳，内容在 `air_quality_content_new.xml`：

- 内容区外边距 18dp。
- 左侧弧形进度条 `ArcProgressBar`，78×68dp。
- 右侧两行：建议文案（正文 2）、说明（正文 3，次要色），行间距 6dp。

## 生活指数

`weather_life_index.xml` 是卡片外壳，内容为自适应网格。单元格 `adapter_life_index_item.xml`：

- 单元格最小高 102dp。
- 纵向：图标 24dp → 名称（正文 3，次要色，上边距 8dp）→ 结论（正文 2，主色，上边距 2dp，下边距 6dp）。
- 内容在单元格内垂直居中。

## 日出日落

`weather_sun_rise_set_new.xml`：

- 自定义 `WeatherSunView` 绘制太阳轨迹，最小高 190dp。
- 下方 `SunAndMoonDescriptionView` 给日出、日落、月相文本。
- 再下方"点击查看详情"，12sp，30% 白。

## 其他卡片

- `weather_featured_map_layout.xml`：顶部 `HwSubTabWidget` 分页，下方 `HwViewPager` 放地图，属于卡片内嵌分页。
- `weather_warning_card_layout.xml`：预警卡片，带级别图标。
- `weather_wind_new.xml`、`weather_feel.xml`：风与体感的小卡片。

## 字号层级

华为使用 EMUI 文本样式（`headline8`、`subtitle1`、`body1`、`body2`、`body3`、`body4`），具体 sp 值在华为 framework 中，本 APK 里读不到。实现按相对关系确定：

| 用途 | 相对层级 | 实现取值 |
| --- | --- | --- |
| 城市名 | headline8 | 26 |
| 当前温度 | 自定义大号 | 96 |
| 高低温 | subtitle1 | 16 |
| 天气现象 / AQI | body1 | 15 |
| 卡片标题 | 12sp | 12 |
| 小时时间 | body3 | 13 |
| 小时温度 | body1 | 16 |
| 小时降水 | body4 | 11 |
| 多日星期/日期 | future_weather_week/day | 15 / 13 |
| 生活指数名称 | body3 | 12 |
| 生活指数结论 | body2 | 14 |

## 翻译成 egui 的对应关系

| 华为 | 本项目 |
| --- | --- |
| 10% 白圆角卡片 | `theme::card()`：`#1AFFFFFF`，圆角 16 |
| 卡片标题 12sp 60% 白 | `theme::card_title()` |
| 逐小时 72dp 单元格 | `widgets::hourly_strip` 单元格宽 72 |
| 多日 48dp 行 | `widgets::daily_list` 行高 48，左侧星期+日期，右侧高低温 |
| AQI 弧线 | `widgets::aqi_summary` 用弧形进度条 + 两行文本 |
| 生活指数 102dp 单元格 | `widgets::life_indices` 三列网格，图标+名称+结论 |
| 日出日落 190dp 视图 | `widgets::sun_details` 画弧线轨迹 |
| 顶部城市名 56dp 偏移 | `theme::safe_top()` + 56 |

## 刻意不照搬的部分

- 不打包华为的天气动画资源、字体（`HW-digit-*`、`PorscheDesignFont`）和皮肤。
- 不使用华为的图标资源，天气图标继续矢量绘制。
- 不实现华为的卡片排序/隐藏设置、语音播报、华为账号与推送服务。
- 不追求逐像素一致，只复用信息层级和实测间距。
