# 菲律宾天气

菲律宾天气是基于 Breezy Weather 的简体中文 Android 应用。PAGASA 提供当地天气和预警，菲律宾环境管理局（DENR-EMB）提供站点空气质量。手机直接读取公开数据。简体中文界面使用霞鹜文楷精简版。

## 安装

从 [GitHub Releases](https://github.com/scaredpenguin627/panahon-zh/releases/latest) 下载 APK。Pixel 10a 选择 **arm64-v8a**；不清楚设备架构时选择 **universal**。其他安装包适用于 armeabi-v7a、x86 和 x86_64。每个版本附上五个 APK 的 SHA-256 校验值。

正式版包名为 `org.panahon.zh.weather`，可与 Breezy Weather 或本项目的调试版共存。发布证书的 SHA-256 指纹为 `ab1a327e665d8e17899b075240762e5541978dce09402b4ef3158135b2572dba`。

## 地点与天气

- 添加菲律宾地点后，在地点的数据源设置中可查看预报、当前天气和预警的提供方。三项均支持 PAGASA PANaHON。当前天气显示当地临近预报；需要测站实况时，还可选择 PAGASA 站点源。
- 搜索“长滩岛”“長灘島”“Boracay”或“Boracay Island”会找到同一座岛。它与马尼拉、宿务、达沃都支持离线地名搜索。天气刷新需要联网。
- 首页菜单“用其他应用查看此地点”会把经纬度交给支持地理位置链接的应用，例如地图。
- 在“设置 → 主界面 → 块”选择要显示的卡片。风、紫外线、花粉等卡片还需要所选数据源提供相应读数。可在桌面的小组件选择器中添加天气组件。部分第三方数据源需要在设置里填写自己的 API key，例如 OpenWeather；PAGASA 和本应用接入的公开 EMB 报表无需密钥。

## 空气质量

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
