# 重建证据

## 目标包

- 输入文件：`com.dostpagasa.panahon_1.1.4.xapk`。
- 包名：`com.dostpagasa.panahon`。
- 原应用：Flutter 发布包，包含 arm64 原生库。
- SDK：最低版本 29，目标版本 36。
- 可见功能：地点、当前天气、逐小时预报、五日预报、天气地图、预警、PAGASA 观测和城市展望。

## 分析顺序

第一步使用 IDA Pro 9.5 和 Hex-Rays IDA MCP 分析 APK 与 arm64 原生库。IDA 第一轮结果保存在 `docs/ida-first-pass.json`。第二步使用 Craft Anything 保存包清单和 Flutter 识别结果。最后用独立的 Rust 数据模型和 egui 界面实现功能。

Craft Anything 的分析说明保存在 `docs/craft-anything-evidence/`。原始 JSON 只在本地生成，默认不纳入版本库。这些记录包含包信息、Flutter 识别结果、权限和组件。

## 证据边界

项目只参考包元数据、组件清单、资源目录和用户可见功能。Rust 实现没有复用原应用的 Flutter 二进制、图片、商标、认证请求、设备注册流程或私有令牌。

## 公开接口

网络适配器使用公开服务的版本、观测、nowcast 和预警接口。应用不保存访问令牌。接口请求设置 10 秒超时，解析失败时继续显示离线样例。

2026 年 11 月 7 日，在马尼拉坐标（14.5995，120.9842）做了只读检查：版本接口返回 `1.0.4`，自动观测接口返回成功，nowcast 接口返回成功。

## 已知限制

- 已在 Pixel 10a GrapheneOS 上完成安装、启动、Network 权限、实时天气刷新、地点搜索、AQI 服务页和本地通知测试。
- 已接入 PAGASA 雷达和 Himawari 卫星最新图像，以及闪电、台风和灾害预警的公开数据摘要。地图时间轴播放、观测历史、城市展望解析和桌面小组件仍未实现。
- Android APK 必须使用项目维护者自己的签名密钥。
- 接入更多公开数据前，需要再次确认服务条款、速率限制和字段格式。

