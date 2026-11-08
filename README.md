# 菲律宾天气（中文）

面向 GrapheneOS 的中文天气应用。用 Rust 和 egui 编写，打包成普通 Android APK，数据来自 PAGASA 与 DENR-EMB 的公开接口。

界面参考小米天气：一屏看完当前天气，往下依次是逐小时、五日预报、空气质量和观测详情。

## 界面

五个页面，底部导航切换：

- **天气**：当前温度、体感、最高最低、24 小时滚动预报、五日预报、空气质量摘要、观测详情、生活指数、最新预警。
- **空气**：DENR-EMB 的 16 个国控站，按 AQI 从高到低排列，标明首要污染物和 PM10/PM2.5 数值。
- **地图**：PAGASA 雷达、卫星、闪电、台风、灾害五个图层。
- **预警**：PAGASA 公开预警全文，附防御建议和本地提醒开关。
- **我的**：地点搜索与切换、数据来源清单、版本信息。

视觉上做了三件事：

1. 背景是随天气和昼夜变化的天空渐变，晴天偏蓝、雨天偏灰、夜间更深。
2. 天气图标全部用矢量绘制，不打包任何图片资源。
3. 卡片是半透明白色，圆角 22，间距统一。

真机截图见 `pixel10a-ui-home.png`、`pixel10a-ui-aqi.png`、`pixel10a-ui-offline.png`。

## 数据来源

| 模块 | 来源 | 接入方式 |
| --- | --- | --- |
| 实时天气、逐小时、五日 | PAGASA PANaHON 公开接口 | `api.mobile.panahon.gov.ph` JSON |
| 空气质量 | DENR-EMB 实时空气质量报表 | 直连 Power BI 公开报表 |
| 雷达、卫星 | PAGASA 公开图层 | 读取时间轴后取最新一帧 |
| 闪电、台风、灾害 | PAGASA 公开接口 | 只读摘要计数 |
| 预警 | PAGASA `/alerts` | JSON |
| 离线 | 应用内置样例 | 无网络时自动使用 |

空气质量有三级回退：先读 DENR-EMB 报表，读不到再用 Open-Meteo 的 CAMS 位置预测，都失败则保留上一次数据。界面上会标明实际来源。

EMB 的数据不在页面 HTML 里，藏在 Power BI 的「发布到 Web」报表中。`core/src/emb.rs` 按匿名嵌入协议取数：

1. 请求 `app.powerbi.com/view?r=…`，拿到报表所在集群。
2. 请求 `modelsAndExploration`，拿到 `modelId`。
3. 用 `recipes/` 里的查询体重放 `querydata`，得到 DSR 格式的行数据。

配方是从浏览器里抓下来的固定 JSON。EMB 改报表结构时需要重新抓，见文末。

## 运行

### 桌面预览

```powershell
cargo run --features "desktop network"
```

桌面窗口按 Pixel 10a 的逻辑分辨率开，方便对照真机。用 `PANAHON_TAB=1..4` 可以直接打开指定页面截图。

### Android

```powershell
$env:ANDROID_HOME = "C:\Android\Sdk"
$env:ANDROID_NDK_HOME = "C:\Android\Sdk\ndk\<版本>"
$env:CARGO_APK_RELEASE_KEYSTORE = "<签名文件>"
$env:CARGO_APK_RELEASE_KEYSTORE_PASSWORD = "<密码>"
.\build.ps1 -Network
```

产物在 `target\release\apk\panahon-zh.apk`，包名 `org.panahon.zh`，最低 SDK 29，目标 SDK 35。

首次出包需要先准备签名文件：

```powershell
keytool -genkeypair -v -keystore keystore\panahon-release.jks -alias panahon `
  -keyalg RSA -keysize 2048 -validity 10000 `
  -storepass <密码> -keypass <密码> `
  -dname "CN=Panahon Zh, O=Panahon, C=PH"
```

`keystore/` 已在 `.gitignore` 中，不会进版本库。

### 中文字体

APK 内置 Noto Sans CJK SC 的子集，只保留代码里出现过的字符。改了界面文案要重新生成：

```powershell
python tools\subset-font.py --source <NotoSansCJKsc-Regular.otf>
```

脚本会扫描 `src/` 和 `core/src/`，把用到的字符写进 `assets/ui-font-chars.txt`，再调用 `pyftsubset` 输出 `assets/NotoSansCJKsc-Subset.otf`。漏字的表现是界面上出现方框。

## 代码结构

```
src/
  app.rs      应用状态、五个页面、后台请求与消息循环
  theme.rs    配色、天空渐变、卡片与文字样式
  widgets.rs  卡片、逐小时条、五日预报、AQI 徽章、底部导航
  icons.rs    矢量天气图标
  lib.rs      库入口、Android 入口、通知与安全区 JNI
  main.rs     桌面预览入口
core/
  src/lib.rs  数据模型、PAGASA 接口适配、生活指数
  src/emb.rs  DENR-EMB Power BI 读取与 DSR 解码
  recipes/    三个查询配方
  examples/live.rs  联网冒烟测试
tools/
  subset-font.py     字体子集生成
  screenshot.ps1     窗口截图（带 DPI 处理）
```

## 行为约定

- 所有网络请求在后台线程执行，界面不阻塞；结果通过 channel 回到主线程。
- 请求失败时保留当前数据，页面顶部状态行显示失败原因。
- 断网时显示内置样例，温度、预报、预警页面都能打开。
- 状态栏和手势条高度从 Android `WindowInsets` 读取，不写死数值。
- 地点切换后自动刷新天气和空气质量。

## 测试

```powershell
cargo test -p panahon-core --features network
cargo run -p panahon-core --example live --features network
```

第一条跑单元测试，覆盖数据解析、图标映射、AQI 分级、DSR 解码和时间换算。第二条真的打网络，打印当前天气与空气质量，用来确认上游没变。

## 已知限制

- **EMB 改报表就得重抓配方。** 配方里写死了 `CAMS.Station`、`CAMS.PM10AQI` 这些字段名。
- **三个历史嵌入已失效。** `air.emb.gov.ph` 页面上的 Batangas、Antipolo 报表返回 401，本项目只用 NCR 主报表。
- **预警正文仍是英文。** 只翻译了标题和防御建议，正文保留 PAGASA 原文，便于和官网对照。
- **通知是进程内调度。** 系统回收进程后，30 分钟提醒需要重新打开应用才会继续。
- **地图只有最新一帧。** 时间轴播放还没做。

## 重新抓配方

EMB 换了报表结构时：

```powershell
node tools\capture-recipe.mjs <resourceKey>
```

脚本用无头 Chrome 打开报表，把首屏所有 `querydata` 请求体存到 `captured/`，每个文件都标注了查询的字段。挑出需要的改名为 `recipes/` 下的文件，再把 `modelId` 置为 `null`。

## 许可

代码使用 MIT 或 Apache-2.0 双许可证。PAGASA、PANaHON、DENR-EMB 及其他名称、图像和商标不属于本项目。

界面字体使用 Noto Sans CJK SC，遵循 SIL Open Font License 1.1，许可证见 `assets/OFL.txt`。

天气与空气质量数据版权归 PAGASA 和 DENR-EMB 所有。接入或再分发前请确认各自的使用条款。
