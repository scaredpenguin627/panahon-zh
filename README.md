# 菲律宾天气

菲律宾天气是基于 Breezy Weather 的简体中文 Android 客户端。添加菲律宾城市后，应用会为预报、当前天气和预警推荐菲律宾气象局（PAGASA）的 PANaHON 数据源。手机直接向菲律宾气象局请求数据，无需部署本仓库的服务端。

## 在手机上使用

1. 安装下方 `basicDebug` 构建生成的 APK。Pixel 10a 使用 `arm64-v8a` 版本。
2. 打开应用，添加马尼拉、宿务或其他菲律宾地点。也可以授予定位权限，使用当前位置。
3. 在地点的数据源设置中查看预报、当前天气和预警。三项均支持 **PAGASA PANaHON**。如需使用 PAGASA 网站的气象站和逐时预报，可选择另一项 **PAGASA**。

界面默认使用简体中文和霞鹜文楷精简版。可在设置中更改语言。预警标题和正文保留官方原文。

## 在 Windows 上构建

构建需要 JDK 21、Android SDK Platform 37 和 Build Tools 37.0.0。设置好 `JAVA_HOME` 和 `ANDROID_HOME` 后，从仓库根目录运行：

```powershell
cd breezy-weather
.\gradlew.bat :app:assembleBasicDebug
.\gradlew.bat :app:testBasicDebugUnitTest --tests "*PanahonConverterTest"
```

构建产物位于 `app/build/outputs/apk/basic/debug/`。应用的目标为 Android 17（API 37）。调试包使用独立包名 `org.panahon.zh.weather.debug`，可与 Breezy Weather 同时安装。

## 菲律宾数据源

| 项目 | 官方接口 | 处理方式 |
| --- | --- | --- |
| 逐时和逐日预报 | `nowcast/dashboard` | 用菲律宾当地时间解析逐时、最高温和最低温 |
| 当前天气 | `nowcast/dashboard` | 显示所选地点两小时内的官方临近预报和体感温度 |
| 天气预警 | `alerts` | 按官方地名和全国范围标记筛选，保留官方原文和有效期 |

接口根地址为 `https://api.mobile.panahon.gov.ph/api/v1/`。启用 PANaHON 后，应用会将所选地点的经纬度发送至该地址。每个地点可单独选择其他天气源。详见[菲律宾气象局 PANaHON](https://panahon.gov.ph/) 和[隐私说明](breezy-weather/PRIVACY.md)。

## 源码与许可

| 目录 | 内容与许可 |
| --- | --- |
| [breezy-weather](breezy-weather/) | Android 融合版，基于 [Breezy Weather](https://github.com/breezy-weather/breezy-weather/tree/d91fcf41e4e5169024a564ea79981c75bf350250)。代码遵循 [LGPL-3.0](breezy-weather/LICENSE) 及[附加条款](breezy-weather/LICENSE_ADDITIONAL) |
| [android-app](android-app/) | 原有 Android 客户端，使用本仓库的天气 API |
| [services](services/) | 原有天气服务、移动端核心和 DENR-EMB 空气质量适配器，构建说明见各子目录 |

字体来自[霞鹜文楷精简版](https://github.com/lxgw/LxgwWenKai-Lite)，遵循 SIL Open Font License 1.1。原项目的许可文件保留在仓库根目录。融合版为社区维护的衍生项目。
