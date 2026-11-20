# 菲律宾天气

菲律宾天气是基于 Breezy Weather 的菲律宾旅行天气 Android 应用。PAGASA 提供当地天气和预警，菲律宾环境管理局（DENR-EMB）提供站点空气质量。手机直接读取公开数据。界面支持 English、简体中文和繁體中文。简体中文使用霞鹜文楷精简版。

## 安装

从 [GitHub Releases](https://github.com/scaredpenguin627/panahon-zh/releases/latest) 下载 APK。Pixel 10a 选择 **arm64-v8a**；不清楚设备架构时选择 **universal**。其他安装包适用于 armeabi-v7a、x86 和 x86_64。每个版本附上五个 APK 的 SHA-256 校验值。

正式版包名为 `org.panahon.zh.weather`，可与 Breezy Weather 或本项目的调试版共存。发布证书的 SHA-256 指纹为 `ab1a327e665d8e17899b075240762e5541978dce09402b4ef3158135b2572dba`。

## 地点与天气

- 菲律宾预报和预警可选择 **PAGASA PANaHON**。它按所选地点的坐标查询菲律宾气象局移动服务。列表中的 **PAGASA（菲律宾）** 也来自菲律宾气象局，使用官网测站观测和逐时预报；同一座岛和最近测站可能相距数十公里。
- PANaHON 的临近预报提供温度和体感温度。当前风和湿度优先使用附近有效的 PAGASA 观测，逐时风湿等补充数据来自 Open-Meteo 的当地模型。首页会标明补充数据的提供方。
- 在“添加地点”可浏览 **127 个菲律宾地点**，也可以输入中文、英文或常见别名。除了长滩岛，目录还收录爱妮岛的大泻湖、科隆的沉船潜点、薄荷岛周边的巴里卡萨岛等。地点使用各自坐标，离线可搜索，联网后刷新预报。[旅游地点、坐标来源和逐点核验记录](breezy-weather/docs/TOURISM.zh-CN.md)
- 在“设置 → 主界面 → 块”选择要显示的卡片。风、紫外线、花粉等卡片还需要所选数据源提供相应读数。可在桌面的小组件选择器中添加天气组件。部分第三方数据源需要在设置里填写自己的 API key，例如 OpenWeather。刷新提示会列出需要密钥的数据源和功能，点“设置”可填写该服务的密钥。PAGASA 和本应用接入的公开 EMB 报表无需密钥。

## 空气质量

在“编辑此位置 → 天气数据源 → 空气质量”选择包含 **DENR-EMB** 的来源并保存，即可同时查看当地官方站点和 Open-Meteo 模型。已有地点可分别更改。选择单独的 Open-Meteo 时，首页仅显示模型空气质量。

首页的 EMB 站点读数标明站名、地点、数据时间和官方来源。报表只给出统一更新时间时，界面标为“报表更新”。菲律宾 AQI 子指数与以 µg/m³ 等单位发布的浓度分开显示。缺少本地站点、报表缺测或暂时断网时，界面会说明状态；其他空气质量提供方的数据另标来源。

各地区实际可读的报表、站点、污染物及时间见[全国 EMB 覆盖清单](breezy-weather/docs/EMB-COVERAGE.zh-CN.md)。官方入口包括 [NCR 监测站](https://ncr.emb.gov.ph/realtimeairmonitoring/)、[环境空气监测](https://air.emb.gov.ph/ambient-air-quality-monitoring/)和[全国区域目录](https://air.emb.gov.ph/national-air-quality-data/)。天气数据见 [PAGASA PANaHON](https://panahon.gov.ph/)。应用访问的位置及服务见[隐私说明](breezy-weather/PRIVACY.md)。

## 构建与源码

使用 JDK 21、Android SDK Platform 37.0 和 Build Tools 37.0.0。Windows 用户可从仓库根目录运行：

```powershell
cd breezy-weather
.\gradlew.bat :app:testBasicDebugUnitTest :app:assembleBasicDebug
```

调试 APK 位于 `app/build/outputs/apk/basic/debug/`。公开的正式安装包来自对应版本标签，签名流程见[发布工作流](.github/workflows/release.yml)。

Android 客户端基于 [Breezy Weather 上游提交 d91fcf4](https://github.com/breezy-weather/breezy-weather/tree/d91fcf41e4e5169024a564ea79981c75bf350250)，随附 [LGPL-3.0 许可证](breezy-weather/LICENSE)及[附加条款](breezy-weather/LICENSE_ADDITIONAL)。字体来自[霞鹜文楷精简版](https://github.com/lxgw/LxgwWenKai-Lite)，随附 [SIL Open Font License 1.1](breezy-weather/app/src/main/assets/licenses/LXGW-WenKai-Lite-OFL.txt)。原项目的取数逻辑及相关许可保留[原作者署名](LICENSE-MIT)。

旧客户端和服务可从 [legacy-pre-breezy 归档](https://github.com/scaredpenguin627/panahon-zh/tree/legacy-pre-breezy) 获取。融合期间对旧服务的地点校验修复保存在 [c646f40](https://github.com/scaredpenguin627/panahon-zh/tree/c646f402fbc31e788d121c8e8d2215b0e052ca12/services)。
