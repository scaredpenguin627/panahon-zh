# 重建证据

## 目标包

- 输入文件：`com.dostpagasa.panahon_1.1.4.xapk`。
- 包名：`com.dostpagasa.panahon`。
- 原应用：Flutter 发布包，包含 arm64 原生库。
- SDK：最低版本 29，目标版本 36。
- 可见功能：地点、当前天气、逐小时预报、五日预报、天气地图、预警、PAGASA 观测和城市展望。

## 分析顺序

第一步使用 IDA Pro 9.5 和 Hex-Rays IDA MCP 分析 APK 与 arm64 原生库。第二步使用 Craft Anything 保存包清单和 Flutter 识别结果。最后用独立的 Rust 数据模型和 egui 界面实现功能。

Craft Anything 的分析说明保存在 `docs/craft-anything-evidence/`。原始 JSON 只在本地生成，默认不纳入版本库。

## 证据边界

项目只参考包元数据、组件清单、资源目录和用户可见功能。Rust 实现没有复用原应用的 Flutter 二进制、图片、商标、认证请求、设备注册流程或私有令牌。

界面的天气图标是矢量绘制的，没有打包 PAGASA 或原应用的图片资源。中文字体使用 Noto Sans CJK SC 的子集，遵循 SIL Open Font License 1.1。

## 公开接口

### PAGASA

网络适配器使用公开的版本、观测、nowcast 和预警接口。应用不保存访问令牌。接口请求设置 10 秒超时，解析失败时继续显示离线样例。

2026 年 11 月 8 日在马尼拉坐标（14.5995，120.9842）做了只读检查：版本接口返回 `1.0.2`，自动观测接口返回站点 `Science Garden, Quezon City` 的观测，nowcast 接口返回 24 条逐小时和 5 天日预报，`/alerts` 返回 11 条预警。

地图图层读取 PAGASA 首页返回的短期 CSRF 令牌，再调用雷达、Himawari 和闪电接口。令牌只在内存中使用，不写入 APK 或日志。

### DENR-EMB

空气质量来自 DENR-EMB 的「发布到 Web」Power BI 报表。页面 `ncr.emb.gov.ph/realtimeairmonitoring/` 只是 WordPress 外壳，数据不在 HTML 里。

读取走匿名嵌入的三步协议：解析 iframe 的 `r=` 参数、请求 `modelsAndExploration` 拿 `modelId`、重放 `querydata` 拿行数据。解码逻辑在 `core/src/emb.rs`，查询配方在 `core/recipes/`。

2026 年 11 月 8 日的只读检查返回 16 个国控站，数据时间 06:00，平均 AQI 27，最高 纳沃塔斯 50。同一页面的 Batangas 和 Antipolo 报表返回 401，属于上游失效。

DENR-EMB 报表不可用时回退到 Open-Meteo 的 CAMS 位置预测，界面上标明实际来源。

## 已知限制

- 已在 Pixel 10a GrapheneOS 上完成安装、启动、五个页面切换、联网刷新、雷达图层加载和离线启动测试。
- 雷达/卫星地图时间轴播放、AQI 模型参考趋势、本机 AQI 刷新记录和台风文字/折线路径与风圈摘要已实现；后台连续监测、台风官方底图、城市展望解析和桌面小组件仍未实现。
- EMB 报表结构变化会导致取数失败，需要重新抓取查询配方。
- 预警正文保留 PAGASA 英文原文，卡片提供复制原文按钮，避免在同一段安全通告中混入不完整的机器翻译。
- Android APK 必须使用项目维护者自己的签名密钥。
- 接入更多公开数据前，需要再次确认服务条款、速率限制和字段格式。
