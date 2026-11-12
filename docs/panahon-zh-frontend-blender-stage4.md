# panahon-zh 前端与 Blender 阶段 4 报告

本阶段依据阶段 1、阶段 2 和阶段 3 报告，只实现阶段 3 A 组首批闭环。没有把录屏中出现但当前数据层无法可靠提供的功能伪装成已实现，也没有复制华为的代码、字体、图片、商标或专有资源。

## 环境与基线

本次使用的工具版本为：

| 工具 | 实际版本 |
|---|---|
| Node | `v27.0.0-alpha.0.1.0` |
| Python | `3.15.2` |
| Rust | `rustc 1.100.0` / `cargo 1.100.0` |
| Blender | `Blender 5.3.0` |

Node、Python、Rust 和 Blender 均安装在用户可写目录；系统自带版本没有被冒充为目标版本。Python 3.15.2 的编译缺少若干可选模块（例如 `_sqlite3`、`_tkinter`），不影响本阶段的字体脚本和 Rust 前端构建。

## 修改的文件

- `core/src/lib.rs`：补齐降水、热指数、AQI、地图、预警、生活指数和日照数据的质量/来源/更新时间字段；缺失降水改用 `Option<f32>`；新增生活指数输入门槛和测试；保留旧缓存数值的反序列化兼容性。
- `core/src/emb.rs`：EMB AQI 标记为官方质量，补充污染物字段的明确不可用状态。
- `core/examples/live.rs`：适配可选降水值输出。
- `src/app.rs`：增加地图来源与质量状态、降水雷达标签、温度/风力无适配器状态、AQI 污染物入口、运行状态卡和首页动效入口；保持首页、AQI、地图、预警、设置的进入/返回关系。
- `src/widgets.rs`：增加统一质量标签和来源行、六项污染物状态、热指数来源显示、生活指数不可用原因、预警原文复制反馈，以及 Blender 静态预览和 egui 光晕回退。
- `src/icons.rs`：为 `WeatherIcon::Unknown` 绘制专用问号图标，不再退化为云雨图标。
- `assets/NotoSansCJKsc-Subset.otf`：用新文案重新生成并由前端实际嵌入。
- `tools/blender_stage4_cloud_sky.py`、`assets/blender_stage4/`：Blender 原型脚本、场景、预览和说明。

没有修改阶段 1、阶段 2、阶段 3 报告，也没有修改锁文件或 Node 服务代码。

## 保留与替换的模块

保留 `Dashboard`、`AirQuality`、`Alert`、`RemoteMapSnapshot`、`LifeIndex`、`DataSource`、`DataQuality`、PAGASA、Open-Meteo、DENR-EMB、CAMS 和现有地图适配器。保留 Manila UTC+8 日期、观测新鲜度、预警有效期/地点过滤、Android 缓存和现有数据契约。

重组 `src/app.rs` 的页面状态和导航编排，补充 `src/widgets.rs` 的来源标签、污染物列表、质量状态、动效预览和窄屏可用布局。UI 仍只消费 core 数据模型，没有在 UI 层重新解析原始 JSON，也没有重新实现时区、热指数、降水质量或 AQI 来源判断。

## 数据可信度修复

| 门槛 | 实现结果 |
|---|---|
| 缺少降水与真实 `0 mm` | 当前、逐小时和逐日降水均为 `Option<f32>`；缺失为 `None + Unavailable`，真实零值为 `Some(0.0)`，UI 不再用 0 mm 代替缺失。旧缓存的数字仍可反序列化。 |
| 官方/预报/估算热指数 | `Official`、`Forecast`、`Estimated`、`Unavailable` 保留在模型中；当前详情和生活指数显示对应质量。 |
| Manila 跨日/跨年 | 继续使用 `trust::forecast_date` 和 UTC+8；日期测试覆盖 2026-12-31 → 2027-01-01。 |
| 未知天气图标 | `Unknown` 有专用图标和不可用状态，不绘制成普通云雨。 |
| 生活指数输入门槛 | `Dashboard::life_indices()` 只有在逐小时和逐日输入都存在时才生成参考结论；否则每项显示“暂无指数”和缺少输入原因。 |
| 来源、更新时间、不可用原因 | 首页指标、AQI、污染物、地图、预警、生活指数和日照卡均显示来源/质量/时间，或显示不可用原因。 |
| 地图、AQI、预警、生活指数质量 | `RemoteMapSnapshot`、`AirQuality`、`Alert`、`LifeIndex` 均携带质量信息；地图错误会回到 `Unavailable`。 |

AQI 页面明确列出 PM2.5、PM10、SO₂、NO₂、CO、O₃。当前 EMB/CAMS 适配器可靠提供的是 PM2.5/PM10 **AQI 子指数**，其他字段保留为“暂无数据”，没有把子指数误标成浓度，也没有推算为 0。

## 页面和导航

- **天气首页**：城市、当前温度、高低温、逐小时横向滚动、五日预报、AQI 摘要、当前指标、热指数、生活指数、预警、地图入口、来源和更新时间均保留；加载、缓存、错误和不可用状态沿用现有状态机。
- **AQI 页面**：AQI 色阶/摘要、六项污染物、监测站列表、趋势、历史、来源、数据时间和 EMB/CAMS 回退状态可见。
- **天气地图**：实际适配器图层包括降水雷达、卫星、闪电、台风和灾害；雷达/卫星帧支持时间轴、播放、暂停、刷新、加载和失败状态。温度、风力入口显示“暂无数据适配器”，不会伪造地图帧。
- **预警页面**：保留地点/全国过滤、发布时间、有效期、官方原文、空/加载/错误状态；“复制原文”按钮调用 egui `copy_text` 并显示已复制反馈。
- **设置页面**：中文/English、城市搜索和单地点选择、地图/预警入口、来源、版本、缓存状态、通知状态、权限说明和桌面/Android 差异均可见。
- **首页导航**：AQI 卡、预警卡和地图卡可进入对应页面；子页返回按钮回到首页；设置页可再次选择地点并刷新天气/AQI。

## Blender 5.3 原型

原型文件：

- `assets/blender_stage4/cloud_sky_scene.blend`（150,186 bytes）
- `tools/blender_stage4_cloud_sky.py`
- `assets/blender_stage4/preview/cloud_sky_frame_0001.png`（236,191 bytes）
- `assets/blender_stage4/preview/cloud_sky_frame_0061.png`（236,603 bytes）
- `assets/blender_stage4/README.md`

使用实际 Blender 5.3.0 后台命令运行：

```bash
/workspace/blender-5.3/blender-5.3.0-linux-x64/blender \
  --background --python tools/blender_stage4_cloud_sky.py
```

脚本创建原创渐变天空、双层云团、太阳圆盘、城市剪影和窗口光点，并为云团设置 120 帧、24 fps 的往返关键帧。脚本先尝试 Blender 5.3 的 `BLENDER_EEVEE_NEXT` 枚举；此发行包暴露旧 Eevee 枚举时回退到 `BLENDER_EEVEE`，实际仍由 Blender 5.3.0 生成两张 360×640 RGBA PNG 和 `.blend`。运行只出现 Vulkan/EGL 无显示警告和 `use_nodes` 弃用警告，渲染成功。

首页通过 `include_bytes!` 嵌入第一帧 PNG，首次显示时转换为 egui 纹理；运行时用 egui 绘制缓慢变化的光晕并请求重绘。这样桌面和 Android 不需要视频解码，也不会把天气、AQI、预警或地图数据烘焙进不可更新视频。若 PNG 不可用，卡片会回落为深色占位和同一 egui 动画。

## 字体验证

系统字体是字体集合，先用 Python 3.15.2/fontTools 导出第 0 个字体面，再运行仓库脚本：

```bash
python - <<'PY'
from fontTools.ttLib import TTCollection
TTCollection('/usr/share/fonts/opentype/noto/NotoSansCJK-Regular.ttc').fonts[0].save(
    '/workspace/toolchains/NotoSansCJK-Regular.otf'
)
PY
python tools/subset-font.py --source /workspace/toolchains/NotoSansCJK-Regular.otf
```

结果为 `assets/NotoSansCJKsc-Subset.otf: 304916 bytes, 800 characters`。`src/app.rs` 继续通过 `include_bytes!` 实际使用该文件。代码检查确认新增的 Unknown、暂无数据、官方、估算和模型预报文案都在子集字符集中；没有进行带显示器的逐字形截图验证。

## 验证结果

已运行并通过：

```bash
cargo test --locked --workspace --features network      # 34 passed
cargo build --locked --features "desktop network"      # passed
for file in services/emb-aqi/server.mjs services/emb-aqi/src/*.mjs services/emb-aqi/tools/*.mjs; do
  node --check "$file"
done                                                     # passed with Node v27.0.0-alpha.0.1.0
```

Blender 5.3 后台脚本成功生成 `.blend` 和两张预览帧。`cargo run -p panahon-core --example live --features network` 能构建并启动，但当前执行环境的外网连接被拒绝：PAGASA 返回 `Connection refused`，Open-Meteo 返回 `Network is unreachable`。这记录为网络验证阻塞，不记为数据适配器通过。

没有可用的 `DISPLAY` 或 `WAYLAND_DISPLAY`，因此没有声称完成桌面视觉点击验证。Android 真机、系统剪贴板实际写入、网络地图帧、EMB Power BI 在线字段、通知权限和缓存恢复尚未在目标设备上验证。

## 尚未实现的阶段 3 B/C/D

本阶段没有实现 15 日、90 日、月度或长期预报；完整健康/过敏/防晒指数；霞光地图和打卡点；月相/月距；完整温度/风力地图适配器；桌面小组件；天气助手、专有天气视频、空间感天气资源；Huawei 账号、云空间、钱包、系统推送、服务条款和隐私正文。这些项目继续保留为阶段 3 的 B/C/D 范围，等待可靠数据、平台能力或后续原型验证。
