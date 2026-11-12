# panahon-zh 项目盘点报告（阶段 2）

盘点范围：当前 `main` 分支的 `panahon-zh` 项目本身。不把阶段 1 的视频观察当作项目已有功能，不把 roadmap 或旧文档中的计划当作已实现功能。

状态标签：

- **已实现**：当前源码中存在明确实现；
- **部分实现**：只有部分流程、平台或状态存在；
- **仅计划**：只在文档或 roadmap 中出现；
- **未发现**：当前仓库没有找到；
- **未验证**：源码存在，但本轮无法运行验证。

## A. 环境和仓库状态

### Git 状态

初始工作区状态：

- 初始分支：`work`
- 初始 HEAD：`4d0d3b5 Match Huawei weather navigation and improve city search`
- 初始工作树：干净

按要求切换到 `main` 后，阶段 1 报告已提交并推送：

- 当前分支：`main`
- 当前 HEAD：`012cd17 Add Huawei weather video observation report`
- 远端：`origin/main` 已指向 `012cd17`
- 当前 `git status --short`：无输出

阶段 1 文件：`docs/huawei-weather-video-observation-report.md`。

### 工具版本

已观察：

- Rust：`rustc: command not found`
- Cargo：`cargo: command not found`
- Node.js：`v24.22.1`
- npm：`11.9.0`
- Python：`3.12.14`
- ffmpeg：`7.1.5`
- Git：`2.52.0`

Cargo 相关命令均因执行器没有安装 Cargo 直接以退出码 127 失败，不能据此判断 Rust 代码编译失败。

### 本轮验证命令

| 命令 | 结果 |
|---|---|
| `cargo metadata --no-deps --format-version 1` | 未验证：`cargo: command not found` |
| `cargo test --locked --workspace --features network` | 未验证：`cargo: command not found` |
| `cargo build --locked --features "desktop network"` | 未验证：`cargo: command not found` |
| `cargo run -p panahon-core --example live --features network` | 未验证：`cargo: command not found` |
| EMB AQI 全部 `node --check` | 已通过 |
| EMB `/health` 本地探活 | 已通过 |
| EMB `/api/aqi` 10 秒请求 | 未验证：上游请求超时 |

## B. 当前页面与用户功能清单

### 前端入口和页面

已实现：桌面入口在 `src/main.rs`，Android 入口在 `src/lib.rs`。主应用状态和页面分发在 `src/app.rs` 的 `PanahonApp`。

已实现的五个主页面：

1. **天气**：当前天气、逐小时、五日预报、AQI 摘要、地图入口、当前观测、日照、生活指数、预警卡、城市展望入口。
2. **空气质量**：刷新、AQI 摘要、趋势、历史记录、监测站列表和来源说明。
3. **天气地图**：雷达、卫星、闪电、台风、灾害五个图层；图像、时间轴、播放/暂停、刷新和台风摘要。
4. **预警**：当前地点预警、全国性预警、本地通知权限和提醒控制。
5. **设置/我的**：中文/English、地图和预警入口、当前地点、地点搜索、数据来源、版本和关于信息。

代码证据：`src/app.rs` 的 `weather_page`、`air_page`、`map_page`、`alerts_page`、`profile_page`，以及 `update` 的 tab 分发。

### 页面控件、卡片和滚动区域

已实现：

- 顶部返回/菜单、城市搜索、更多菜单；
- 中央页面垂直 `ScrollArea`；
- 逐小时横向 `ScrollArea`；
- 五日预报列表；
- AQI 仪表、趋势和历史图；
- 当前观测网格；
- 日出、日落、紫外线和能见度卡；
- 生活指数网格；
- 预警卡和复制原文按钮；
- 地图图层胶囊、时间轴滑块、播放/暂停和刷新按钮；
- 地点搜索文本框、搜索按钮和结果选择；
- 语言切换按钮；
- Android 通知权限、测试通知、30 分钟提醒开关。

### 功能状态

| 功能 | 状态 | 依据和限制 |
|---|---|---|
| 当前天气 | 已实现 | PAGASA dashboard/observation 解析存在；运行未验证 |
| 逐小时预报 | 已实现 | 最多取 24 条；运行未验证 |
| 逐日预报 | 已实现 | 最多取 5 天；运行未验证 |
| AQI | 已实现 | EMB Power BI + CAMS 回退；运行未验证 |
| 天气地图 | 已实现 | 五图层、雷达/卫星时间轴；运行未验证 |
| 台风摘要 | 已实现 | 路径点、中心、强度和半径摘要；无完整官方底图 |
| 预警 | 已实现 | 有效期和地点/全国范围过滤存在 |
| 预警复制 | 已实现 | `egui::Context::copy_text` 已调用 |
| 设置 | 已实现 | 地点、语言、数据来源、通知等入口存在 |
| 中文/English | 已实现 | UI 文案和 Android 偏好持久化存在 |
| 生活指数 | 部分实现 | 只有四项本地建议，缺少完整官方指数体系 |
| 城市展望 | 部分实现 | 只有 PAGASA URL 入口，没有应用内播放/缓存 |
| 后台通知 | 部分实现 | 进程内线程；没有系统级 AlarmManager |

## C. 数据源与数据可信度清单

### 网络层和数据源

已实现：

- PAGASA `/version`、`/nowcast/dashboard`、`/observation/automatic`、`/alerts`；
- Open-Meteo 日照、紫外线、能见度和 AQI 趋势；
- DENR-EMB Power BI 公开报表；
- PAGASA 雷达、卫星、闪电、台风和灾害接口；
- 可选的 `PANAHON_AQI_ENDPOINT` AQI 中间服务。

天气网络请求在后台线程执行，通过 channel 返回 UI。AQI 在 NCR 坐标范围内优先读取 EMB；范围外使用 CAMS；EMB 失败时在代码中回退到 CAMS。

### 数据模型

`core/src/lib.rs` 定义了 `Dashboard`、`CurrentWeather`、`ForecastDay`、`HourlyForecast`、`Alert`、`AirQuality`、`SunWeather`、`LifeIndex`、`RemoteMapSnapshot`、`WeatherIcon`、`DataSource` 和 `DataQuality`。

`DataQuality` 分为：

- `Official`；
- `Forecast`；
- `Estimated`；
- `Unavailable`。

### 数值字段来源和转换

| 字段 | 来源/转换 | 状态 |
|---|---|---|
| 当前温度 | 新鲜自动站 `temp`，否则 nowcast `tmp` | 已实现 |
| 湿度 | 自动站 `hum`，限制在 0–100 | 已实现 |
| 风速 | 自动站 `ws` 直接进入 `wind_kph` | 部分实现，单位仍需运行数据确认 |
| 热指数 | 官方 `heat_index` 或官方 nowcast `hindex` | 已实现 |
| 当前降水 | 自动站 `precip_1h`，缺失为 Unavailable | 部分实现，内部仍用 `0.0` 哨兵 |
| 逐日降水 | `tprecip`，缺失则质量为 Unavailable | 已实现 |
| 逐小时降水 | `tprecip`，缺失则质量为 Unavailable | 已实现 |
| 日期 | Manila UTC+8、RFC3339/本地格式、跨年最近日期 | 已实现 |
| AQI | PM10/PM2.5 中较大值，四舍五入并限制 0–500 | 已实现 |
| 日照/紫外线/能见度 | Open-Meteo 模型数据 | 已实现，明确标为模型预报 |
| 预警 | 官方正文、发布时间、有效期、区域 | 已实现 |

### 缓存和缺失数据

已实现：

- Android `weather_cache_v2` 保存最近天气快照；
- Android `air_history` 最多保存 48 个 AQI 点；
- 天气刷新失败时保留已有 dashboard；
- AQI 刷新失败时保留已有 AQI；
- 没有缓存且未设置 `PANAHON_DEMO=1` 时使用 `Dashboard::empty()`；
- `Dashboard::empty()` 将温度、湿度、风、热指数、降水和图标标为不可用；
- 当前观测 UI 会显示“暂无”或 `--`，而不是直接显示缺失值；
- 逐小时/逐日缺失降水时隐藏降水数值。

部分实现：非 Android 平台的偏好读写为空实现，因此桌面预览不会持久化地点、语言、缓存和通知状态。

### 九项可信度规则核对

1. **缺少降水不能显示 0 mm：部分实现。** 质量标记和 UI 隐藏逻辑存在，单测也覆盖了“缺失降水不作为零雨”；但核心模型仍将缺失值存为 `0.0`，当前观测显示“暂无”而不是完整的“暂无数据”。
2. **官方热指数和本地估算区分：已实现。** 热指数分级只在官方值或官方预报可用时使用，`DataQuality` 区分来源。
3. **Manila 时区日期：已实现。** `trust.rs` 使用 UTC+8，含跨年和非法日期单测。
4. **未知天气图标：部分实现。** 核心模型保留 `WeatherIcon::Unknown`，但 `src/icons.rs` 的默认绘制分支可能把未知图标画成云/降水样式。
5. **生活指数输入不完整时显示暂无指数：已实现保守回退。** 出行和晾晒明确标为暂无/查看预报；当前只有四项建议。
6. **显示来源、更新时间或不可用原因：已实现。** `DataQuality`、source badge、更新时间和错误状态均存在；实际渲染未运行验证。

## D. Rust/egui 前端结构

### 入口和状态

- `src/main.rs`：桌面 eframe 入口；
- `src/lib.rs`：模块出口、Android 入口、JNI、SharedPreferences、通知；
- `src/app.rs`：应用状态、后台请求、消息循环、页面组合；
- `src/widgets.rs`：页面组件和文案；
- `src/theme.rs`：颜色、天空渐变、卡片、圆角、间距；
- `src/icons.rs`：原创矢量天气图标。

`PanahonApp` 直接持有页面状态、网络加载状态、缓存状态、搜索状态、地图播放状态和通知状态。网络 worker 通过 `Msg` channel 回传 `Dashboard`、`Air`、`Map`、`Places` 和 `Version`。

### core 层

- `core/src/lib.rs`：模型、PAGASA、Open-Meteo、AQI 回退、地图适配和地点搜索；
- `core/src/trust.rs`：时间、日期、单位、新鲜度、热指数；
- `core/src/emb.rs`：Power BI DSR 解码和 EMB 字段映射；
- `core/src/advisory.rs`：预警句式处理；
- `core/examples/live.rs`：联网冒烟示例；
- `core/recipes/`：三份固定 EMB 查询配方。

### 数据绑定位置

当前绑定是直接的：`PanahonApp` 将 `Dashboard`、`AirQuality`、`RemoteMapSnapshot` 和 `Alert` 传入 widgets。没有单独的 view-model 层，页面布局与状态更新都集中在 `src/app.rs`。

## E. 已实现、部分实现、仅计划、未发现

### 已实现

- 五个主页面；
- 当前天气、逐小时、逐日天气；
- AQI 摘要、趋势、站点和本地历史；
- 雷达、卫星、闪电、台风、灾害图层；
- 雷达/卫星时间轴和播放；
- 台风路径摘要；
- 日照、紫外线、能见度；
- 预警过滤和官方正文复制；
- 中文/English 切换；
- 中文、英文、拼音地点搜索；
- 来源、质量、更新时间提示；
- Android 地点、语言、缓存和通知偏好；
- `PANAHON_DEMO=1` 离线样例；
- 网络失败时保留现有数据。

### 部分实现

- 生活指数只有四项本地建议；
- 通知是进程内线程，不是系统级后台服务；
- 城市展望只有外部链接；
- 地图没有完整地理底图、自动站/人工站图层和完整风圈覆盖；
- 缺失降水使用内部 `0.0` 哨兵；
- Unknown 天气图标的视觉显示不够明确；
- 非 Android 平台不持久化偏好；
- EMB Node 服务没有默认接入 Rust 应用。

### 仅计划

根据 `docs/feature-roadmap.md` 和 `docs/rewrite-context-summary.md`，以下项目目前只属于计划或剩余范围：

- 自动站/人工站分布图层；
- 城市展望应用内播放和离线缓存；
- 旅游天气接口；
- 桌面/主屏小组件；
- 下拉刷新；
- 系统级 AlarmManager；
- 完整台风底图和风圈覆盖。

路线图中“预警按地点过滤”和“AQI 最近 48 次历史”仍列在下一步，但当前源码已经有部分或完整实现，文档状态已滞后。

### 未发现

- 桌面/主屏小组件代码；
- AlarmManager receiver/service；
- 下拉刷新手势；
- 应用内城市展望播放器；
- 旅游天气数据源；
- 华为账号、华为推送或语音播报；
- 华为式卡片排序/隐藏设置；
- Rust/egui UI 自动化测试；
- EMB Node 自动化测试目录。

### 未验证

- Rust 工作区构建和测试；
- PAGASA/Open-Meteo/Power BI 实际联网；
- Android 权限、通知、SharedPreferences 和安全区；
- 地图图像和时间轴播放；
- 语言切换完整 UI；
- 预警复制真实剪贴板结果；
- 网络失败和缓存恢复的实际 UI 状态。

## F. 测试、构建和联网验证

源码中约有 30 个 Rust 单元测试，覆盖：

- 数据模型和离线样例；
- AQI 六段分级；
- 数据质量；
- 图标映射；
- 缺失降水；
- 生活指数；
- 单位解析；
- Manila 时区和跨年日期；
- 观测新鲜度；
- 站点匹配；
- 预警地点过滤；
- 城市别名；
- 预警句式；
- EMB DSR 解码。

这些测试本轮均未实际运行，因为环境没有 Cargo。

Node 语法检查全部通过，EMB `/health` 通过；EMB `/api/aqi` 在 10 秒内超时。当前没有 UI 截图测试、页面导航测试、语言切换测试、Android 通知测试、地图播放测试或 Node 上游集成测试。

## G. 当前风险和未知项

1. Rust/Cargo 缺失，当前无法证明仓库能构建或测试通过。
2. Power BI 配方依赖固定字段名，报表改版会使 AQI 抓取失败。
3. 本轮 `/api/aqi` 上游请求超时，实际网络数据尚未验证。
4. 缺失降水在模型中仍用 `0.0` 作为哨兵值，其他调用方可能误读。
5. Unknown 图标在 UI 默认绘制分支中可能显示成普通降水图标。
6. `PanahonApp` 把页面、状态、网络和缓存强耦合，后续 UI 替换成本较高。
7. 桌面平台偏好读写为空实现，与 README 对“重启后恢复”的描述不一致。
8. 生活指数目前只有四项，且穿衣质量字段当前硬编码为不可用。
9. README、roadmap 和当前代码存在状态滞后。
10. Android 实机行为、上游字段、服务条款和速率限制均未在本轮重新验证。

## H. 为阶段 3 准备的可替换 UI 边界

### 建议保留

- `Dashboard`、`AirQuality`、`Alert`、`RemoteMapSnapshot`、`LifeIndex` 等数据模型；
- `DataSource` 和 `DataQuality`；
- `trust.rs` 的时间、单位、新鲜度、日期和热指数规则；
- PAGASA、Open-Meteo、EMB 网络适配器；
- Power BI DSR 解码；
- 预警有效期和地点过滤；
- 来源、更新时间和不可用状态；
- 原创矢量图标和 clean-room 边界；
- Node `/api/aqi` 的 JSON 契约。

### 适合替换

- `src/app.rs` 的页面组合和交互编排；
- `src/widgets.rs` 的卡片、列表、指标和导航视觉层；
- `src/theme.rs` 的天空背景、卡片、圆角和色彩；
- `src/icons.rs` 的图标呈现层；
- `PanahonApp` 中把网络消息、缓存和 UI 事件混在一起的部分。

### 建议的阶段 3 边界

```text
PAGASA / Open-Meteo / EMB / Node AQI
            ↓
core data models + trust/source/quality
            ↓
view-model adapter
            ↓
新的 UI 页面、卡片、地图和设置
```

新的 UI 不应直接读取原始 JSON，也不应重新实现降水缺失、热指数来源、Manila 时区、预警范围和 AQI 来源判断；这些逻辑应继续留在 `core` 数据层。

