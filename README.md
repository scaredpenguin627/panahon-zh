# 菲律宾天气

去菲律宾旅行时用的安卓天气应用。预报和预警来自菲律宾气象局（PAGASA），空气质量来自菲律宾环境管理局（DENR-EMB）的公开监测站报表。

## 首页

- **当前天气**：实况、体感温度、今天最高和最低。
- **预警**：有生效中的 PAGASA 预警时，预警卡排在首页第一张。卡片列出标题和有效期，点开看原文。
- **预报**：五天概况和逐时预报。图标、温度、风、降水按菲律宾气象局的描述显示。
- **空气质量**：DENR-EMB 站点的圆环指数。点开看 PM2.5、PM10 等污染物的子指数。
- **其它**：日出日落、月相、气压、湿度。

空气质量和预警都按地点匹配。附近没有已核实的 EMB 站点时，空气质量卡不出现，不用远处的站点顶替。

## 数据来源

| 内容 | 来源 |
| --- | --- |
| 实况、逐时、五日预报、预警 | PAGASA PANaHON |
| 空气质量 | DENR-EMB Power BI 公开报表 |
| 湿度、露点、风的补充、气压 | Open-Meteo 模型 |

PAGASA 不发布气压这类要素，这几张卡由 Open-Meteo 的当地模型补齐。日出日落和月相由本地天文计算得出，不需要联网。页脚写明每一项的来源，点开可以看完整清单。

查询时会把坐标发给上表的数据源，除此之外不上传位置。

## 地点

内置 123 个菲律宾旅游地点，包括长滩岛、爱妮岛、科隆和薄荷岛周边。支持中文、英文和常见别名搜索，离线也能搜。也可以输入坐标添加任意地点。

## 界面

English、简体中文、繁體中文三种界面语言。中文用系统字体，跟随系统字号和字重，不额外下载字体。

## 安装

从 [GitHub Releases](https://github.com/scaredpenguin627/panahon-zh/releases/latest) 下载 APK。Pixel 10a 选 **arm64-v8a**；不确定设备架构时选 **universal**。其它安装包适用于 armeabi-v7a、x86 和 x86_64。每个版本附五个 APK 的 SHA-256 校验值。

正式版包名是 `org.panahon.zh.weather`，可以和 Breezy Weather 或本项目的调试版同时安装。发布证书的 SHA-256 指纹：

```
ab1a327e665d8e17899b075240762e5541978dce09402b4ef3158135b2572dba
```

## 构建

需要 JDK 21、Android SDK Platform 37.0 和 Build Tools 37.0.0。工程路径不能包含中文，Gradle 会拒绝非 ASCII 路径。Windows 下在仓库根目录运行：

```powershell
cd breezy-weather
.\gradlew.bat :app:testBasicDebugUnitTest :app:assembleBasicDebug
```

调试 APK 在 `app/build/outputs/apk/basic/debug/`。公开的安装包由版本标签触发签名和发布，流程见[发布工作流](.github/workflows/release.yml)。

## 许可与署名

安卓客户端基于 [Breezy Weather 上游提交 d91fcf4](https://github.com/breezy-weather/breezy-weather/tree/d91fcf41e4e5169024a564ea79981c75bf350250)，沿用 [LGPL-3.0](breezy-weather/LICENSE) 和[附加条款](breezy-weather/LICENSE_ADDITIONAL)。

天气和预警数据来自 PAGASA，空气质量数据来自 DENR-EMB，许可说明见[数据来源与许可](breezy-weather/app/src/main/assets/tourism/PANaHON-EMB-MIT.txt)。取数逻辑沿用原项目的署名，见 [LICENSE-MIT](LICENSE-MIT)。

各地实际可读的 EMB 报表见[全国 EMB 覆盖清单](breezy-weather/docs/EMB-COVERAGE.zh-CN.md)，旅游地点的坐标来源见[旅游地点说明](breezy-weather/docs/TOURISM.zh-CN.md)。
